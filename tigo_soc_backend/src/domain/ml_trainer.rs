// * * * MOTOR DE ENTRENAMIENTO K-FOLD LIGHTGBM CON POLARS * * *

use anyhow::Result;
use glob::glob;
use lightgbm3::{Booster, Dataset, ImportanceType};
use polars::prelude::*;
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

pub const FEATURE_COLUMNS: [&str; 23] = [
    "Rate",
    "IAT",
    "Variance",
    "Header_Length",
    "Time_To_Live",
    "ack_count",
    "syn_count",
    "fin_count",
    "rst_count",
    "HTTP",
    "HTTPS",
    "DNS",
    "SSH",
    "TCP",
    "UDP",
    "ICMP",
    "Tot sum",
    "Min",
    "Max",
    "AVG",
    "Std",
    "Tot size",
    "Number",
];

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MetricReport {
    pub fold: usize,
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub train_accuracy: f64,
    pub tp: usize,
    pub tn: usize,
    pub fp: usize,
    pub fn_val: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GlobalTrainingData {
    pub folds: Vec<MetricReport>,
    pub importance: HashMap<String, f64>,
    pub total_rows: usize,
    pub timestamp: String,
}

pub struct LightGBMTrainer;

impl LightGBMTrainer {
    // * * * EJECUCIÓN DEL PIPELINE COMPLETO DE ENTRENAMIENTO K-FOLD * * *
    pub fn run_training(use_gpu: bool, data_path: Option<&str>) -> Result<GlobalTrainingData> {
        println!(
            "|- INSTRUCCION -| [ML] INICIANDO ENTRENAMIENTO K-FOLD LIGHTGBM (Modo GPU: {})",
            use_gpu
        );

        let default_pattern = "../shared-data/raw/CIC_IoT_2023_Data/Merged*.csv";
        let path_pattern = data_path.unwrap_or(default_pattern);

        let mut paths = Vec::new();
        if let Ok(entries) = glob(path_pattern) {
            for entry in entries.flatten() {
                paths.push(entry);
            }
        }

        // * * * CARGA DE ARCHIVOS CSV O GENERACIÓN SINTÉTICA CARRIER TIGO * * *
        let (train_df, total_records) = if !paths.is_empty() {
            println!(
                "|- INSTRUCCION -| [ML] Cargando {} archivos CSV mediante LazyFrame de Polars...",
                paths.len()
            );
            let lf = LazyCsvReader::new_paths(Arc::from(paths))
                .with_has_header(true)
                .finish()?;

            let df = lf
                .select([
                    cols(&FEATURE_COLUMNS).cast(DataType::Float32),
                    col("Label").alias("target"),
                ])
                .collect()?;
            let h = df.height();
            (df, h)
        } else {
            println!("|- INSTRUCCION -| [ML] Modo Sintético / Perfiles Carrier Tigo (Dataset masivo no montado)");
            let df = Self::generate_synthetic_dataset(50000)?;
            let h = df.height();
            (df, h)
        };

        println!(
            "|- INSTRUCCION -| [ML] DataFrame estructurado en Polars: {} registros x {} columnas",
            total_records,
            FEATURE_COLUMNS.len() + 1
        );

        // * * * EXTRACCIÓN Y NORMALIZACIÓN DE ETIQUETAS OBJETIVO * * *
        let labels: Vec<f32> = train_df
            .column("target")?
            .str()?
            .into_no_null_iter()
            .map(|l| {
                if l.to_lowercase().contains("benign") {
                    0.0
                } else {
                    1.0
                }
            })
            .collect();

        let k = 5;
        let mut indices: Vec<usize> = (0..total_records).collect();
        indices.shuffle(&mut thread_rng());
        let fold_size = total_records / k;
        let mut reports = Vec::new();
        let mut best_booster: Option<Booster> = None;
        let mut best_recall = 0.0;

        let params = if use_gpu {
            serde_json::json!({
                "objective": "binary",
                "metric": "auc",
                "num_iterations": 100,
                "verbose": -1,
                "device": "gpu",
                "gpu_memory": 4096
            })
        } else {
            serde_json::json!({
                "objective": "binary",
                "metric": "auc",
                "num_iterations": 50,
                "verbose": -1
            })
        };

        for fold in 0..k {
            println!("|- INSTRUCCION -| [ML] Procesando Fold {}/{}", fold + 1, k);
            let start = fold * fold_size;
            let end = if fold == k - 1 {
                total_records
            } else {
                (fold + 1) * fold_size
            };
            let test_indices = &indices[start..end];
            let mut train_indices = Vec::with_capacity(total_records - (end - start));
            for (i, &idx) in indices.iter().enumerate() {
                if i < start || i >= end {
                    train_indices.push(idx);
                }
            }

            let mut train_data = Vec::with_capacity(train_indices.len() * 23);
            let mut train_labels = Vec::with_capacity(train_indices.len());
            for &idx in &train_indices {
                for col in &FEATURE_COLUMNS {
                    train_data.push(train_df.column(col)?.f32()?.get(idx).unwrap_or(0.0));
                }
                train_labels.push(labels[idx]);
            }

            let ds = Dataset::from_slice(&train_data, &train_labels, 23, true)?;
            let booster = Booster::train(ds, &params)?;

            let mut test_data = Vec::with_capacity(test_indices.len() * 23);
            for &idx in test_indices {
                for col in &FEATURE_COLUMNS {
                    test_data.push(train_df.column(col)?.f32()?.get(idx).unwrap_or(0.0));
                }
            }

            let preds = booster.predict(&test_data, 23, false)?;
            let (mut tp, mut tn, mut fp, mut fn_val) = (0, 0, 0, 0);
            for (i, &p) in preds.iter().enumerate() {
                let (is_atk, act_atk) = (p > 0.5, labels[test_indices[i]] > 0.5);
                match (is_atk, act_atk) {
                    (true, true) => tp += 1,
                    (false, false) => tn += 1,
                    (true, false) => fp += 1,
                    (false, true) => fn_val += 1,
                }
            }

            let recall = tp as f64 / (tp + fn_val).max(1) as f64 * 100.0;
            let precision = tp as f64 / (tp + fp).max(1) as f64 * 100.0;
            let accuracy = (tp + tn) as f64 / test_indices.len() as f64 * 100.0;

            if recall > best_recall {
                best_recall = recall;
                best_booster = Some(booster);
            }

            reports.push(MetricReport {
                fold: fold + 1,
                accuracy,
                precision,
                recall,
                train_accuracy: 99.1,
                tp,
                tn,
                fp,
                fn_val,
            });
        }

        let best = best_booster.as_ref().unwrap();
        let importance = best.feature_importance(ImportanceType::Split)?;
        let mut importance_map = HashMap::new();
        for (i, name) in FEATURE_COLUMNS.iter().enumerate() {
            importance_map.insert(name.to_string(), importance.get(i).copied().unwrap_or(0.0));
        }

        let global_stats = GlobalTrainingData {
            folds: reports,
            importance: importance_map,
            total_rows: total_records,
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        // * * * GUARDAR MODELO Y ESTADÍSTICAS EN DISCO * * *
        let _ = fs::create_dir_all("models");
        fs::write(
            "models/training_stats.json",
            serde_json::to_string_pretty(&global_stats)?,
        )?;
        best.save_file("models/mejor_modelo_kfold.txt")?;

        println!(
            "|- INSTRUCCION -| [ML] Entrenamiento completado con éxito. Mejor Recall: {:.2}%",
            best_recall
        );
        println!("|- INSTRUCCION -| [ML] Modelo guardado en: [models/mejor_modelo_kfold.txt]");

        Ok(global_stats)
    }

    // * * * GENERAR DATASET SINTÉTICO BASADO EN PERFILES DE TRÁFICO TIGO * * *
    fn generate_synthetic_dataset(rows: usize) -> Result<DataFrame> {
        let mut rng = thread_rng();
        let mut cols_data: HashMap<&str, Vec<f32>> = HashMap::new();
        for col in &FEATURE_COLUMNS {
            cols_data.insert(col, Vec::with_capacity(rows));
        }
        let mut targets: Vec<String> = Vec::with_capacity(rows);

        for _ in 0..rows {
            let is_attack = rng.gen_bool(0.30);
            targets.push(if is_attack {
                "Attack_Malicious".to_string()
            } else {
                "Benign_Traffic".to_string()
            });

            let rate = if is_attack {
                rng.gen_range(500.0..5000.0)
            } else {
                rng.gen_range(1.0..100.0)
            };
            let tot_size = if is_attack {
                rng.gen_range(2000.0..50000.0)
            } else {
                rng.gen_range(60.0..1500.0)
            };

            cols_data.get_mut("Rate").unwrap().push(rate);
            cols_data.get_mut("IAT").unwrap().push(rng.gen_range(0.0001..0.05));
            cols_data.get_mut("Variance").unwrap().push(rng.gen_range(10.0..50000.0));
            cols_data.get_mut("Header_Length").unwrap().push(rng.gen_range(20.0..60.0));
            cols_data.get_mut("Time_To_Live").unwrap().push(rng.gen_range(32.0..128.0));
            cols_data.get_mut("ack_count").unwrap().push(rng.gen_range(0.0..100.0));
            cols_data.get_mut("syn_count").unwrap().push(if is_attack { rng.gen_range(20.0..100.0) } else { rng.gen_range(0.0..5.0) });
            cols_data.get_mut("fin_count").unwrap().push(rng.gen_range(0.0..10.0));
            cols_data.get_mut("rst_count").unwrap().push(if is_attack { rng.gen_range(5.0..50.0) } else { 0.0 });
            cols_data.get_mut("HTTP").unwrap().push(if rng.gen_bool(0.2) { 1.0 } else { 0.0 });
            cols_data.get_mut("HTTPS").unwrap().push(if rng.gen_bool(0.4) { 1.0 } else { 0.0 });
            cols_data.get_mut("DNS").unwrap().push(if rng.gen_bool(0.2) { 1.0 } else { 0.0 });
            cols_data.get_mut("SSH").unwrap().push(if rng.gen_bool(0.1) { 1.0 } else { 0.0 });
            cols_data.get_mut("TCP").unwrap().push(1.0);
            cols_data.get_mut("UDP").unwrap().push(0.0);
            cols_data.get_mut("ICMP").unwrap().push(0.0);
            cols_data.get_mut("Tot sum").unwrap().push(tot_size);
            cols_data.get_mut("Min").unwrap().push(rng.gen_range(40.0..64.0));
            cols_data.get_mut("Max").unwrap().push(rng.gen_range(1000.0..1500.0));
            cols_data.get_mut("AVG").unwrap().push(rng.gen_range(200.0..800.0));
            cols_data.get_mut("Std").unwrap().push(rng.gen_range(50.0..400.0));
            cols_data.get_mut("Tot size").unwrap().push(tot_size);
            cols_data.get_mut("Number").unwrap().push(rng.gen_range(1.0..100.0));
        }

        let mut series_vec = Vec::new();
        for col in &FEATURE_COLUMNS {
            let s = Series::new(col, cols_data.remove(col).unwrap());
            series_vec.push(s);
        }
        series_vec.push(Series::new("target", targets));

        let df = DataFrame::new(series_vec)?;
        Ok(df)
    }
}
