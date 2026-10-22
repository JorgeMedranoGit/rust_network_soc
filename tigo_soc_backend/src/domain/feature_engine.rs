// * * * MOTOR DE FEATURE ENGINEERING COLUMNAR ESTRUCTURADO EN POLARS * * *

use anyhow::Result;
use polars::prelude::*;
use std::time::Instant;

use crate::domain::models::{
    ExtractedFeatures, L4Protocol, NetworkEvent, TCP_FLAG_ACK, TCP_FLAG_FIN, TCP_FLAG_RST,
    TCP_FLAG_SYN,
};

pub struct PolarsFeatureEngine;

impl PolarsFeatureEngine {
    // * * * EXTRACCIÓN DE 23 CARACTERÍSTICAS MEDIANTE PROCESAMIENTO COLUMNAR EN POLARS * * *
    pub fn extract_features_columnar(events: &[NetworkEvent]) -> Result<(ExtractedFeatures, f64)> {
        let start_time = Instant::now();

        if events.is_empty() {
            return Err(anyhow::anyhow!("Lote de eventos vacío para Feature Engineering"));
        }

        let n = events.len();
        let mut packet_sizes = Vec::with_capacity(n);
        let mut header_lens = Vec::with_capacity(n);
        let mut ttls = Vec::with_capacity(n);
        let mut timestamps = Vec::with_capacity(n);
        let mut acks = Vec::with_capacity(n);
        let mut syns = Vec::with_capacity(n);
        let mut fins = Vec::with_capacity(n);
        let mut rsts = Vec::with_capacity(n);
        let mut is_http = Vec::with_capacity(n);
        let mut is_https = Vec::with_capacity(n);
        let mut is_dns = Vec::with_capacity(n);
        let mut is_ssh = Vec::with_capacity(n);
        let mut is_tcp = Vec::with_capacity(n);
        let mut is_udp = Vec::with_capacity(n);
        let mut is_icmp = Vec::with_capacity(n);

        for ev in events {
            packet_sizes.push(ev.packet_size as f32);
            header_lens.push(ev.header_length as f32);
            ttls.push(ev.ttl as f32);
            timestamps.push(ev.timestamp.timestamp_millis() as f64 / 1000.0);

            acks.push(if (ev.flags & TCP_FLAG_ACK) != 0 { 1.0f32 } else { 0.0f32 });
            syns.push(if (ev.flags & TCP_FLAG_SYN) != 0 { 1.0f32 } else { 0.0f32 });
            fins.push(if (ev.flags & TCP_FLAG_FIN) != 0 { 1.0f32 } else { 0.0f32 });
            rsts.push(if (ev.flags & TCP_FLAG_RST) != 0 { 1.0f32 } else { 0.0f32 });

            let is_p_http = ev.source_port == 80 || ev.destination_port == 80;
            let is_p_https = ev.source_port == 443 || ev.destination_port == 443;
            let is_p_dns = ev.source_port == 53 || ev.destination_port == 53;
            let is_p_ssh = ev.source_port == 22 || ev.destination_port == 22;

            is_http.push(if is_p_http { 1.0f32 } else { 0.0f32 });
            is_https.push(if is_p_https { 1.0f32 } else { 0.0f32 });
            is_dns.push(if is_p_dns { 1.0f32 } else { 0.0f32 });
            is_ssh.push(if is_p_ssh { 1.0f32 } else { 0.0f32 });

            is_tcp.push(if ev.protocol == L4Protocol::TCP { 1.0f32 } else { 0.0f32 });
            is_udp.push(if ev.protocol == L4Protocol::UDP { 1.0f32 } else { 0.0f32 });
            is_icmp.push(if ev.protocol == L4Protocol::ICMP { 1.0f32 } else { 0.0f32 });
        }

        // * * * 1. CONSTRUIR DATAFRAME COLUMNAR EN POLARS * * *
        let df = df!(
            "packet_size" => &packet_sizes,
            "header_len" => &header_lens,
            "ttl" => &ttls,
            "timestamp" => &timestamps,
            "ack" => &acks,
            "syn" => &syns,
            "fin" => &fins,
            "rst" => &rsts,
            "http" => &is_http,
            "https" => &is_https,
            "dns" => &is_dns,
            "ssh" => &is_ssh,
            "tcp" => &is_tcp,
            "udp" => &is_udp,
            "icmp" => &is_icmp,
        )?;

        // * * * CALCULAR DELTA TEMPORAL (IAT) Y VENTANA DE TASA * * *
        let first_ts = timestamps.first().copied().unwrap_or(0.0);
        let last_ts = timestamps.last().copied().unwrap_or(first_ts);
        let raw_duration = (last_ts - first_ts).abs();
        let duration_secs = if raw_duration >= 0.05 {
            raw_duration
        } else {
            1.0 // * * * NORMALIZAR RÁFAGAS SUB-50MS A BASE DE 1 SEGUNDO * * *
        };

        // * * * 2. EJECUTAR AGREGACIONES COLUMNAR MEDIANTE POLARS LAZYFRAME * * *
        let aggregated = df
            .lazy()
            .with_column(
                (col("timestamp") - col("timestamp").shift(lit(1)))
                    .fill_null(lit(0.0))
                    .alias("iat_raw"),
            )
            .select([
                // * * * CANTIDAD TOTAL DE PAQUETES * * *
                col("packet_size").count().cast(DataType::Float32).alias("Number"),
                // * * * SUMA TOTAL DE BYTES TRANSFERIDOS * * *
                col("packet_size").sum().cast(DataType::Float32).alias("Tot sum"),
                // * * * TAMAÑO MÍNIMO DE PAQUETE * * *
                col("packet_size").min().cast(DataType::Float32).alias("Min"),
                // * * * TAMAÑO MÁXIMO DE PAQUETE * * *
                col("packet_size").max().cast(DataType::Float32).alias("Max"),
                // * * * TAMAÑO PROMEDIO DE PAQUETES * * *
                col("packet_size").mean().cast(DataType::Float32).alias("AVG"),
                // * * * DESVIACIÓN ESTÁNDAR MUESTRAL * * *
                col("packet_size").std(1).fill_null(lit(0.0)).cast(DataType::Float32).alias("Std"),
                // * * * VARIANZA ESTADÍSTICA DEL TAMAÑO * * *
                col("packet_size").var(1).fill_null(lit(0.0)).cast(DataType::Float32).alias("Variance"),
                // * * * TIEMPO INTER-LLEGADA PROMEDIO (IAT) * * *
                col("iat_raw").mean().fill_null(lit(0.0)).cast(DataType::Float32).alias("IAT"),
                // * * * LONGITUD PROMEDIO DEL ENCABEZADO * * *
                col("header_len").mean().cast(DataType::Float32).alias("Header_Length"),
                // * * * TIME TO LIVE PROMEDIO * * *
                col("ttl").mean().cast(DataType::Float32).alias("Time_To_Live"),
                // * * * CONTEO DE BANDERAS TCP * * *
                col("ack").sum().cast(DataType::Float32).alias("ack_count"),
                col("syn").sum().cast(DataType::Float32).alias("syn_count"),
                col("fin").sum().cast(DataType::Float32).alias("fin_count"),
                col("rst").sum().cast(DataType::Float32).alias("rst_count"),
                // * * * INDICADORES DE PROTOCOLOS DE APLICACIÓN Y TRANSPORTE * * *
                col("http").max().cast(DataType::Float32).alias("HTTP"),
                col("https").max().cast(DataType::Float32).alias("HTTPS"),
                col("dns").max().cast(DataType::Float32).alias("DNS"),
                col("ssh").max().cast(DataType::Float32).alias("SSH"),
                col("tcp").max().cast(DataType::Float32).alias("TCP"),
                col("udp").max().cast(DataType::Float32).alias("UDP"),
                col("icmp").max().cast(DataType::Float32).alias("ICMP"),
            ])
            .collect()?;

        // * * * EXTRAER VALORES ESCALARES DE LAS COLUMNAS COMPUTADAS EN POLARS * * *
        let get_scalar = |col_name: &str| -> f32 {
            aggregated
                .column(col_name)
                .ok()
                .and_then(|c| c.f32().ok())
                .and_then(|ca| ca.get(0))
                .unwrap_or(0.0)
        };

        let number = get_scalar("Number");
        let tot_sum = get_scalar("Tot sum");
        let min_size = get_scalar("Min");
        let max_size = get_scalar("Max");
        let avg_size = get_scalar("AVG");
        let std_size = get_scalar("Std");
        let variance = get_scalar("Variance");
        let iat = get_scalar("IAT");
        let header_length = get_scalar("Header_Length");
        let ttl = get_scalar("Time_To_Live");
        let ack_count = get_scalar("ack_count");
        let syn_count = get_scalar("syn_count");
        let fin_count = get_scalar("fin_count");
        let rst_count = get_scalar("rst_count");
        let http = get_scalar("HTTP");
        let https = get_scalar("HTTPS");
        let dns = get_scalar("DNS");
        let ssh = get_scalar("SSH");
        let tcp = get_scalar("TCP");
        let udp = get_scalar("UDP");
        let icmp = get_scalar("ICMP");
        let tot_size = tot_sum;
        let rate = number / (duration_secs as f32);

        let features = ExtractedFeatures {
            rate,
            iat,
            variance,
            header_length,
            ttl,
            ack_count,
            syn_count,
            fin_count,
            rst_count,
            http,
            https,
            dns,
            ssh,
            tcp,
            udp,
            icmp,
            tot_sum,
            min_size,
            max_size,
            avg_size,
            std_size,
            tot_size,
            number,
        };

        let elapsed_us = start_time.elapsed().as_nanos() as f64 / 1000.0;
        Ok((features, elapsed_us))
    }

    // * * * EXTRACCIÓN DE CARACTERÍSTICAS PARA EVENTO INDIVIDUAL * * *
    #[allow(dead_code)]
    pub fn extract_single_event(event: &NetworkEvent) -> Result<(ExtractedFeatures, f64)> {
        Self::extract_features_columnar(&[*event])
    }
}
