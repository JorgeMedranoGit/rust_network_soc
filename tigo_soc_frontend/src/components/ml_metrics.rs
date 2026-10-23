// * * * LÁMINA 5/21 DEL MANUAL DE MOCKUPS: MÉTRICAS ML LIGHTGBM * * *
// Inferencia algorítmica en vivo, ranking de features y ficha del modelo
// entrenado — datos REALES vía GET /api/v1/ml/stats y GET /api/v1/ml/model-info.

use leptos::*;
use serde_json::Value;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::app::{TIGO_AMBER, TIGO_BLUE, TIGO_GREEN, TIGO_RED};

/// Extrae un número de un JSON con claves candidatas (training_stats.json varía
/// entre camelCase y snake_case según la versión del exportador Python/Rust).
fn json_num(v: &Value, keys: &[&str]) -> Option<f64> {
    for k in keys {
        if let Some(n) = v.get(*k) {
            if let Some(f) = n.as_f64() {
                return Some(f);
            }
            if let Some(s) = n.as_str() {
                if let Ok(f) = s.parse::<f64>() {
                    return Some(f);
                }
            }
        }
    }
    None
}

fn json_str(v: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .filter_map(|k| v.get(*k).and_then(|n| n.as_str()).map(String::from))
        .next()
}

/// Ranking de importancia de features: acepta objetos {nombre: valor} o
/// arreglos [{feature|name, importance|gain}].
fn parse_feature_ranking(v: &Value) -> Vec<(String, f64)> {
    let mut out: Vec<(String, f64)> = Vec::new();
    match v {
        Value::Object(map) => {
            for (k, val) in map {
                if let Some(f) = val.as_f64() {
                    out.push((k.clone(), f));
                }
            }
        }
        Value::Array(arr) => {
            for item in arr {
                let name = json_str(item, &["feature", "name", "column"])
                    .unwrap_or_else(|| "?".to_string());
                if let Some(f) = json_num(item, &["importance", "gain", "split_count", "value"]) {
                    out.push((name, f));
                }
            }
        }
        _ => {}
    }
    out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    out.truncate(10);
    out
}

#[derive(Debug, Clone)]
struct ConfusionCell {
    label: &'static str,
    value: i64,
    color: &'static str,
}

#[component]
pub fn MlMetrics() -> impl IntoView {
    let (ml_stats, set_ml_stats) = create_signal<Option<Value>>(None);
    let (model_info, set_model_info) = create_signal<Option<Value>>(None);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (loading, set_loading) = create_signal(true);

    // ── Carga real de los dos endpoints ML al montar (una sola vez; son
    //    métricas de entrenamiento/inferencia de baja variabilidad) ──
    create_effect(move |_| {
        spawn_local(async move {
            let stats = api::fetch_ml_stats().await;
            let info = api::fetch_model_info().await;
            match (stats, info) {
                (Ok(s), Ok(i)) => {
                    set_ml_stats.set(Some(s));
                    set_model_info.set(Some(i));
                    set_error.set(None);
                }
                (s, i) => {
                    if let Ok(i) = i {
                        set_model_info.set(Some(i));
                    }
                    if let Ok(s) = s {
                        set_ml_stats.set(Some(s));
                    }
                    set_error.set(Some(format!(
                        "{}{}",
                        s.err().unwrap_or_default(),
                        i.err()
                            .map(|e| format!(" | {}", e))
                            .unwrap_or_default()
                    )));
                }
            }
            set_loading.set(false);
        });
    });

    // KPIs derivados de /api/v1/ml/stats (inferencia en vivo del detector)
    let inference_ms = create_memo(move |_| {
        ml_stats
            .get()
            .as_ref()
            .and_then(|v| json_num(v, &["avgInferenceMs", "avg_inference_ms", "inferenceMs", "inference_ms"]))
    });
    let accuracy = create_memo(move |_| {
        ml_stats
            .get()
            .as_ref()
            .and_then(|v| json_num(v, &["accuracy", "precision", "accuracyPct"]))
    });
    let predictions = create_memo(move |_| {
        ml_stats
            .get()
            .as_ref()
            .and_then(|v| json_num(v, &["totalPredictions", "total_predictions", "predictionsCount", "count"]))
    });
    let true_positives = create_memo(move |_| {
        ml_stats
            .get()
            .as_ref()
            .and_then(|v| json_num(v, &["truePositives", "true_positives", "tp"]))
    });
    let false_negatives = create_memo(move |_| {
        ml_stats
            .get()
            .as_ref()
            .and_then(|v| json_num(v, &["falseNegatives", "false_negatives", "fn"]))
    });

    // Ranking de features desde model-info (training_stats.json)
    let feature_ranking = create_memo(move |_| -> Vec<(String, f64)> {
        model_info
            .get()
            .as_ref()
            .and_then(|v| {
                v.get("featureImportance")
                    .or_else(|| v.get("feature_importance"))
                    .or_else(|| v.get("features"))
            })
            .map(parse_feature_ranking)
            .unwrap_or_default()
    });

    // Matriz de confusión forense si el exportador la incluye
    let confusion = create_memo(move |_| -> Vec<ConfusionCell> {
        let Some(info) = model_info.get() else {
            return Vec::new();
        };
        let cm = match info
            .get("confusionMatrix")
            .or_else(|| info.get("confusion_matrix"))
        {
            Some(c) => c,
            None => return Vec::new(),
        };
        let g = |keys: &[&str]| -> i64 {
            json_num(cm, keys).map(|f| f as i64).unwrap_or(0)
        };
        vec![
            ConfusionCell { label: "VP", value: g(&["tp", "truePositive", "true_positive"]), color: TIGO_GREEN },
            ConfusionCell { label: "FN", value: g(&["fn", "falseNegative", "false_negative"]), color: TIGO_RED },
            ConfusionCell { label: "FP", value: g(&["fp", "falsePositive", "false_positive"]), color: TIGO_AMBER },
            ConfusionCell { label: "VN", value: g(&["tn", "trueNegative", "true_negative"]), color: TIGO_BLUE },
        ]
    });

    let max_importance = create_memo(move |_| {
        feature_ranking
            .get()
            .iter()
            .map(|(_, v)| *v)
            .fold(0.0f64, f64::max)
            .max(1.0)
    });

    view! {
        <div class="card ml-card">
            <div class="card-header">
                <h3 style="font-size: 1.05rem; font-weight: 700;">
                    <span style=format!("color: {};", TIGO_BLUE)>"🤖 Métricas del Motor LightGBM"</span>
                    <span class="mono-dim" style="margin-left: 0.75rem; font-size: 0.78rem;">
                        "GET /api/v1/ml/stats • GET /api/v1/ml/model-info"
                    </span>
                </h3>
                {move || {
                    let st = ml_stats.get();
                    let ok = json_str(st.as_ref()?, &["status", "modelStatus"]).unwrap_or_default();
                    (!ok.is_empty()).then(|| view! {
                        <span class="badge" style=format!("background: {}22; color: {};", TIGO_GREEN, TIGO_GREEN)>
                            {ok}
                        </span>
                    })
                }}
            </div>

            {move || error.get().map(|e| view! {
                <div class="alert-box alert-danger" style="margin: 1rem 1.25rem 0; font-size: 0.8rem;">
                    {format!("Backend ML no disponible: {}", e)}
                </div>
            })}

            <div style="padding: 1.25rem; display:flex; flex-direction:column; gap:1.25rem;">
                // ══ Fila de KPIs de inferencia ══
                <div class="metrics-grid" style="display:grid; grid-template-columns:repeat(auto-fit,minmax(170px,1fr)); gap:0.9rem;">
                    <div class="metric-tile">
                        <span class="metric-label">"Latencia de inferencia"</span>
                        <span class="metric-value" style=format!("color:{};", TIGO_BLUE)>
                            {move || match inference_ms.get() {
                                Some(ms) => format!("{:.2} ms", ms),
                                None if loading.get() => "…".to_string(),
                                None => "—".to_string(),
                            }}
                        </span>
                        <span class="metric-hint">"< 1 ms objetivo (LightGBM)"</span>
                    </div>
                    <div class="metric-tile">
                        <span class="metric-label">"Precisión validada"</span>
                        <span class="metric-value" style=format!("color:{};", TIGO_GREEN)>
                            {move || match accuracy.get() {
                                Some(a) if a <= 1.0 => format!("{:.2}%", a * 100.0),
                                Some(a) => format!("{:.2}%", a),
                                None if loading.get() => "…".to_string(),
                                None => "—".to_string(),
                            }}
                        </span>
                        <span class="metric-hint">"K-Fold Cross-Validation (k=5)"</span>
                    </div>
                    <div class="metric-tile">
                        <span class="metric-label">"Predicciones en vivo"</span>
                        <span class="metric-value" style=format!("color:{};", TIGO_AMBER)>
                            {move || predictions.get().map(|p| format!("{:.0}", p)).unwrap_or_else(|| if loading.get() { "…".into() } else { "—".into() })}
                        </span>
                        <span class="metric-hint">"Vectores clasificados desde arranque"</span>
                    </div>
                    <div class="metric-tile">
                        <span class="metric-label">"Detecciones VP / FN"</span>
                        <span class="metric-value" style=format!("color:{};", TIGO_RED)>
                            {move || match (true_positives.get(), false_negatives.get()) {
                                (Some(tp), Some(fn_)) => format!("{:.0} / {:.0}", tp, fn_),
                                _ => if loading.get() { "…".to_string() } else { "—".to_string() },
                            }}
                        </span>
                        <span class="metric-hint">"Sensibilidad del detector de exfiltración"</span>
                    </div>
                </div>

                // ══ Ficha del modelo entrenado ══
                <div style="display:grid; grid-template-columns: 1fr 1fr; gap:1.25rem;">
                    <div>
                        <h4 style="font-size:0.85rem; font-weight:700; margin-bottom:0.6rem;">
                            "Ficha del modelo (training_stats.json)"
                        </h4>
                        <table class="data-table" style="font-size:0.78rem;">
                            <tbody>
                                {move || {
                                    let info = model_info.get();
                                    let rows: Vec<(&str, String)> = match info.as_ref() {
                                        Some(v) => vec![
                                            ("Modelo", json_str(v, &["modelType", "model_type", "model"]).unwrap_or_else(|| if loading.get() { "cargando…".into() } else { "—".into() })),
                                            ("Validación", json_str(v, &["validation", "cvStrategy"]).unwrap_or_else(|| "—".into())),
                                            ("Framework", json_str(v, &["framework", "engine"]).unwrap_or_else(|| "—".into())),
                                            ("Estado", json_str(v, &["status", "state"]).unwrap_or_else(|| "—".into())),
                                            ("Features", v.get("nFeatures").or_else(|| v.get("n_features")).and_then(|x| x.as_i64())
                                                .map(|n| format!("{} columnas", n))
                                                .unwrap_or_else(|| "23 (planilla CISDA)".into())),
                                        ],
                                        None => Vec::new(),
                                    };
                                    rows.into_iter().map(|(k, val)| view! {
                                        <tr>
                                            <td style="color:var(--ctp-subtext0); width:35%;">{k}</td>
                                            <td class="mono">{val}</td>
                                        </tr>
                                    }).collect_view()
                                }}
                            </tbody>
                        </table>

                        // Matriz de confusión forense (si el exportador la provee)
                        {move || {
                            let cells = confusion.get();
                            if cells.is_empty() {
                                return view! {
                                    <p style="font-size:0.72rem; color:var(--ctp-overlay1); margin-top:0.75rem;">
                                        "La matriz de confusión se muestra cuando training_stats.json expone confusionMatrix."
                                    </p>
                                }.into_view();
                            }
                            view! {
                                <div style="margin-top:0.9rem;">
                                    <h4 style="font-size:0.8rem; font-weight:700; margin-bottom:0.5rem;">
                                        "Matriz de confusión forense"
                                    </h4>
                                    <div style="display:grid; grid-template-columns:1fr 1fr; gap:0.5rem; max-width:320px;">
                                        {cells.into_iter().map(|c| view! {
                                            <div style=format!(
                                                "padding:0.6rem; border-radius:8px; text-align:center; background:{}18; border:1px solid {}55;",
                                                c.color, c.color
                                            )}>
                                                <div style="font-size:0.7rem; font-weight:700; color:var(--ctp-subtext0);">{c.label}</div>
                                                <div class="mono" style=format!("font-size:1.05rem; font-weight:700; color:{};", c.color)>
                                                    {c.value.to_string()}
                                                </div>
                                            </div>
                                        }).collect_view()}
                                    </div>
                                </div>
                            }.into_view()
                        }}
                    </div>

                    // ══ Ranking de features (gráfica de barras horizontales) ══
                    <div>
                        <h4 style="font-size:0.85rem; font-weight:700; margin-bottom:0.6rem;">
                            "Ranking de features (importancia GBDT)"
                        </h4>
                        {move || {
                            let ranking = feature_ranking.get();
                            let maxv = max_importance.get();
                            if ranking.is_empty() {
                                return view! {
                                    <div class="empty-state" style="padding:1.5rem; font-size:0.78rem;">
                                        {if loading.get() {
                                            "Consultando /api/v1/ml/model-info…"
                                        } else {
                                            "El backend aún no expone featureImportance en training_stats.json."
                                        }}
                                    </div>
                                }.into_view();
                            }
                            ranking.into_iter().map(|(name, imp)| {
                                let pct = (imp / maxv * 100.0).max(2.0);
                                view! {
                                    <div style="margin-bottom:0.55rem;">
                                        <div style="display:flex; justify-content:space-between; font-size:0.72rem; margin-bottom:2px;">
                                            <span class="mono" style="color:var(--ctp-text);">{name}</span>
                                            <span class="mono" style="color:var(--ctp-subtext0);">{format!("{:.1}", imp)}</span>
                                        </div>
                                        <div style="height:7px; border-radius:4px; background:var(--ctp-surface0); overflow:hidden;">
                                            <div style=format!(
                                                "height:100%; width:{:.1}%; background:linear-gradient(90deg, {}, {});",
                                                pct, TIGO_BLUE, TIGO_GREEN
                                            )></div>
                                        </div>
                                    </div>
                                }
                            }).collect_view()
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}
