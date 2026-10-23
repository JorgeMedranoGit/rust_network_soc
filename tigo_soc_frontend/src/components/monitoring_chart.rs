// * * * COMPONENTE: GRÁFICAS DE MONITOREO EN TIEMPO REAL (SVG & CATPPUCCIN MOCHA) * * *

use leptos::*;
use crate::models::MetricDataPoint;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartMetricMode {
    PacketsPerSec,
    LatencyUs,
    DualView,
}

#[component]
pub fn MonitoringChart(
    chart_data: ReadSignal<Vec<MetricDataPoint>>,
) -> impl IntoView {
    let (metric_mode, set_metric_mode) = create_signal(ChartMetricMode::DualView);
    let (hovered_point, set_hovered_point) = create_signal(Option::<MetricDataPoint>::None);

    // Dimensiones del lienzo SVG
    let svg_width = 800.0;
    let svg_height = 280.0;
    let pad_left = 65.0;
    let pad_right = 30.0;
    let pad_top = 25.0;
    let pad_bottom = 40.0;
    let plot_w = svg_width - pad_left - pad_right;
    let plot_h = svg_height - pad_top - pad_bottom;

    // Cálculos reactivos de escalas y trayectorias SVG
    let chart_calc = create_memo(move |_| {
        let pts = chart_data.get();
        if pts.is_empty() {
            return (
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                Vec::<(f64, f64, MetricDataPoint)>::new(),
                Vec::<(f64, f64, MetricDataPoint)>::new(),
                0.0,
                0.0,
                0.0,
                0.0,
            );
        }

        let max_pps = pts.iter().map(|p| p.packets_per_sec).fold(100.0, f64::max) * 1.15;
        let min_pps = 0.0;
        let max_lat = pts.iter().map(|p| p.latency_us).fold(1.0, f64::max) * 1.25;
        let min_lat = 0.0;

        let n = pts.len();
        let step_x = if n > 1 { plot_w / (n - 1) as f64 } else { plot_w };

        let mut pps_coords = Vec::new();
        let mut lat_coords = Vec::new();
        let mut pps_path_str = String::new();
        let mut pps_area_str = String::new();
        let mut lat_path_str = String::new();
        let mut lat_area_str = String::new();

        for (i, p) in pts.iter().enumerate() {
            let x = pad_left + (i as f64 * step_x);
            // Escala PPS
            let pps_ratio = (p.packets_per_sec - min_pps) / (max_pps - min_pps);
            let y_pps = pad_top + plot_h - (pps_ratio * plot_h);
            pps_coords.push((x, y_pps, p.clone()));

            // Escala Latencia
            let lat_ratio = (p.latency_us - min_lat) / (max_lat - min_lat);
            let y_lat = pad_top + plot_h - (lat_ratio * plot_h);
            lat_coords.push((x, y_lat, p.clone()));

            if i == 0 {
                pps_path_str.push_str(&format!("M {:.1},{:.1}", x, y_pps));
                lat_path_str.push_str(&format!("M {:.1},{:.1}", x, y_lat));
            } else {
                pps_path_str.push_str(&format!(" L {:.1},{:.1}", x, y_pps));
                lat_path_str.push_str(&format!(" L {:.1},{:.1}", x, y_lat));
            }
        }

        // Construir paths cerrados para áreas rellenas
        if let (Some(first), Some(last)) = (pps_coords.first(), pps_coords.last()) {
            let y_base = pad_top + plot_h;
            pps_area_str = format!(
                "{} L {:.1},{:.1} L {:.1},{:.1} Z",
                pps_path_str, last.0, y_base, first.0, y_base
            );
        }
        if let (Some(first), Some(last)) = (lat_coords.first(), lat_coords.last()) {
            let y_base = pad_top + plot_h;
            lat_area_str = format!(
                "{} L {:.1},{:.1} L {:.1},{:.1} Z",
                lat_path_str, last.0, y_base, first.0, y_base
            );
        }

        (
            pps_path_str,
            pps_area_str,
            lat_path_str,
            lat_area_str,
            pps_coords,
            lat_coords,
            min_pps,
            max_pps,
            min_lat,
            max_lat,
        )
    });

    view! {
        <div class="monitoring-chart-card">
            // Cabecera interactiva del componente
            <div class="chart-header">
                <div>
                    <h3 style="font-size: 1.05rem; font-weight: 700; color: var(--ctp-text);">
                        "Telemetría Temporal: Caudal de Red y Latencia IA"
                    </h3>
                    <p style="font-size: 0.75rem; color: var(--ctp-subtext0);">
                        "Monitoreo en streaming continuo: Inferencia LightGBM GBDT y Tráfico perimetral"
                    </p>
                </div>

                <div style="display: flex; gap: 0.5rem; align-items: center; flex-wrap: wrap;">
                    // Selector de Modo de Serie
                    <div class="chart-selector">
                        <button
                            class=move || if metric_mode.get() == ChartMetricMode::DualView { "chart-btn active" } else { "chart-btn" }
                            on:click=move |_| set_metric_mode.set(ChartMetricMode::DualView)
                        >
                            "Dual (Tráfico + IA)"
                        </button>
                        <button
                            class=move || if metric_mode.get() == ChartMetricMode::PacketsPerSec { "chart-btn active" } else { "chart-btn" }
                            on:click=move |_| set_metric_mode.set(ChartMetricMode::PacketsPerSec)
                        >
                            "Tráfico (pps)"
                        </button>
                        <button
                            class=move || if metric_mode.get() == ChartMetricMode::LatencyUs { "chart-btn active" } else { "chart-btn" }
                            on:click=move |_| set_metric_mode.set(ChartMetricMode::LatencyUs)
                        >
                            "Latencia GBDT (µs)"
                        </button>
                    </div>

                    // Los datos provienen exclusivamente del polling GET /api/v1/telemetry/metrics
                </div>
            </div>

            // Contenedor Gráfico SVG Reactivo
            <div class="chart-svg-container">
                <svg
                    viewBox=format!("0 0 {} {}", svg_width, svg_height)
                    class="soc-svg-chart"
                >
                    <defs>
                        // Gradiente para Tráfico PPS (Catppuccin Blue/Sky)
                        <linearGradient id="ppsGradient" x1="0" y1="0" x2="0" y2="1">
                            <stop offset="0%" stop-color="#89b4fa" stop-opacity="0.35" />
                            <stop offset="100%" stop-color="#89b4fa" stop-opacity="0.0" />
                        </linearGradient>
                        // Gradiente para Latencia IA (Catppuccin Mauve)
                        <linearGradient id="latGradient" x1="0" y1="0" x2="0" y2="1">
                            <stop offset="0%" stop-color="#cba6f7" stop-opacity="0.35" />
                            <stop offset="100%" stop-color="#cba6f7" stop-opacity="0.0" />
                        </linearGradient>
                    </defs>

                    // Líneas de Rejilla Horizontales
                    {
                        let num_lines = 4;
                        (0..=num_lines).map(|idx| {
                            let ratio = idx as f64 / num_lines as f64;
                            let y = pad_top + (plot_h * ratio);
                            view! {
                                <line
                                    x1=pad_left
                                    y1=y
                                    x2=svg_width - pad_right
                                    y2=y
                                    stroke="var(--ctp-surface0)"
                                    stroke-dasharray="3,3"
                                    stroke-width="1"
                                />
                            }
                        }).collect_view()
                    }

                    // Eje X Base
                    <line
                        x1=pad_left
                        y1=pad_top + plot_h
                        x2=svg_width - pad_right
                        y2=pad_top + plot_h
                        stroke="var(--ctp-surface1)"
                        stroke-width="1.5"
                    />

                    // Etiquetas de Eje Y (Izquierda: Tráfico PPS)
                    {move || {
                        let (_, _, _, _, _, _, min_pps, max_pps, _, _) = chart_calc.get();
                        let show_pps = metric_mode.get() != ChartMetricMode::LatencyUs;
                        if show_pps {
                            view! {
                                <g>
                                    <text x="12" y=pad_top + 10.0 fill="var(--ctp-blue)" font-size="10" font-family="var(--font-mono)">
                                        {format!("{:.0} pps", max_pps)}
                                    </text>
                                    <text x="12" y=pad_top + (plot_h * 0.5) fill="var(--ctp-blue)" font-size="10" font-family="var(--font-mono)">
                                        {format!("{:.0} pps", (max_pps + min_pps) / 2.0)}
                                    </text>
                                    <text x="12" y=pad_top + plot_h fill="var(--ctp-blue)" font-size="10" font-family="var(--font-mono)">
                                        {format!("{:.0} pps", min_pps)}
                                    </text>
                                </g>
                            }.into_view()
                        } else {
                            view! {}.into_view()
                        }
                    }}

                    // Etiquetas de Eje Y (Derecha: Latencia IA µs en modo Dual o Latency)
                    {move || {
                        let (_, _, _, _, _, _, _, _, min_lat, max_lat) = chart_calc.get();
                        let show_lat = metric_mode.get() != ChartMetricMode::PacketsPerSec;
                        if show_lat {
                            view! {
                                <g>
                                    <text x=svg_width - 25.0 y=pad_top + 10.0 fill="var(--ctp-mauve)" font-size="10" font-family="var(--font-mono)" text-anchor="end">
                                        {format!("{:.1} µs", max_lat)}
                                    </text>
                                    <text x=svg_width - 25.0 y=pad_top + plot_h fill="var(--ctp-mauve)" font-size="10" font-family="var(--font-mono)" text-anchor="end">
                                        {format!("{:.1} µs", min_lat)}
                                    </text>
                                </g>
                            }.into_view()
                        } else {
                            view! {}.into_view()
                        }
                    }}

                    // Serie de Área y Línea: Tráfico PPS
                    {move || {
                        let (pps_path, pps_area, _, _, pps_points, _, _, _, _, _) = chart_calc.get();
                        let mode = metric_mode.get();
                        if mode == ChartMetricMode::PacketsPerSec || mode == ChartMetricMode::DualView {
                            view! {
                                <g>
                                    <path d=pps_area fill="url(#ppsGradient)" />
                                    <path d=pps_path fill="none" stroke="var(--ctp-blue)" stroke-width="2.5" />
                                    {
                                        pps_points.into_iter().map(|(x, y, pt)| {
                                            let pt_copy = pt.clone();
                                            view! {
                                                <circle
                                                    cx=x
                                                    cy=y
                                                    r="3.5"
                                                    fill="var(--ctp-base)"
                                                    stroke="var(--ctp-blue)"
                                                    stroke-width="2"
                                                    class="chart-dot"
                                                    on:mouseenter=move |_| set_hovered_point.set(Some(pt_copy.clone()))
                                                />
                                            }
                                        }).collect_view()
                                    }
                                </g>
                            }.into_view()
                        } else {
                            view! {}.into_view()
                        }
                    }}

                    // Serie de Área y Línea: Latencia IA GBDT
                    {move || {
                        let (_, _, lat_path, lat_area, _, lat_points, _, _, _, _) = chart_calc.get();
                        let mode = metric_mode.get();
                        if mode == ChartMetricMode::LatencyUs || mode == ChartMetricMode::DualView {
                            view! {
                                <g>
                                    <path d=lat_area fill="url(#latGradient)" />
                                    <path d=lat_path fill="none" stroke="var(--ctp-mauve)" stroke-width="2.0" stroke-dasharray="4,2" />
                                    {
                                        lat_points.into_iter().map(|(x, y, pt)| {
                                            let pt_copy = pt.clone();
                                            view! {
                                                <circle
                                                    cx=x
                                                    cy=y
                                                    r="3.0"
                                                    fill="var(--ctp-base)"
                                                    stroke="var(--ctp-mauve)"
                                                    stroke-width="1.8"
                                                    class="chart-dot"
                                                    on:mouseenter=move |_| set_hovered_point.set(Some(pt_copy.clone()))
                                                />
                                            }
                                        }).collect_view()
                                    }
                                </g>
                            }.into_view()
                        } else {
                            view! {}.into_view()
                        }
                    }}

                    // Etiquetas de Eje X (Horarios)
                    {move || {
                        let (_, _, _, _, pps_points, _, _, _, _, _) = chart_calc.get();
                        pps_points.into_iter().enumerate().filter(|(idx, _)| idx % 2 == 0).map(|(_, (x, _, pt))| {
                            view! {
                                <text
                                    x=x
                                    y=pad_top + plot_h + 18.0
                                    fill="var(--ctp-subtext0)"
                                    font-size="9"
                                    font-family="var(--font-mono)"
                                    text-anchor="middle"
                                >
                                    {pt.time_label}
                                </text>
                            }
                        }).collect_view()
                    }}
                </svg>
            </div>

            // Leyenda y Valores Inspeccionados al vuelo
            <div class="chart-footer-stats">
                <div class="legend-group">
                    <span class="legend-item" style="color: var(--ctp-blue);">
                        <span class="legend-box" style="background-color: var(--ctp-blue);"></span>
                        "Tráfico de Red (pps)"
                    </span>
                    <span class="legend-item" style="color: var(--ctp-mauve);">
                        <span class="legend-box" style="background-color: var(--ctp-mauve);"></span>
                        "Latencia Inferencia GBDT (µs)"
                    </span>
                </div>

                <div class="hovered-details">
                    {move || {
                        if let Some(pt) = hovered_point.get() {
                            view! {
                                <span style="font-family: var(--font-mono); font-size: 0.8125rem; color: var(--ctp-text);">
                                    {format!("Hora: {} | Tráfico: {:.0} pps | Latencia: {:.2} µs | CPU: {:.1}%",
                                        pt.timestamp, pt.packets_per_sec, pt.latency_us, pt.cpu_usage_pct
                                    )}
                                </span>
                            }.into_view()
                        } else {
                            view! {
                                <span style="font-size: 0.75rem; color: var(--ctp-overlay1);">
                                    "Pasa el cursor sobre los nodos para inspeccionar valores exactos"
                                </span>
                            }.into_view()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
