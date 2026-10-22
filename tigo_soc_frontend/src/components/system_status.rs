// * * * COMPONENTE: PANEL DE ESTADO DEL SISTEMA EN TIEMPO REAL * * *

use leptos::*;
use crate::models::SystemMetrics;

#[component]
pub fn SystemStatus(metrics: ReadSignal<SystemMetrics>) -> impl IntoView {
    view! {
        <div class="system-status-section">
            <div class="metrics-grid">
                // Tarjeta 1: Eventos Procesados
                <div class="metric-card">
                    <div class="metric-title">"Telemetría Procesada"</div>
                    <div class="metric-value">
                        {move || metrics.get().total_events.to_string()}
                    </div>
                    <div class="metric-sub">
                        {move || format!("Tasa actual: {} pps en tiempo real", metrics.get().traffic_pps)}
                    </div>
                </div>

                // Tarjeta 2: Mitigaciones Automáticas Ejecutadas
                <div class="metric-card auto-card">
                    <div class="metric-title">"Bloqueos Automáticos (Auto)"</div>
                    <div class="metric-value" style="color: var(--ctp-green);">
                        {move || metrics.get().blocked_auto.to_string()}
                    </div>
                    <div class="metric-sub">
                        "L2 RADIUS CoA / L3-L4 Blackhole ejecutados"
                    </div>
                </div>

                // Tarjeta 3: Mitigaciones Retenidas por Blast Radius (PENDING_MANUAL)
                <div class="metric-card pending-card">
                    <div class="metric-title">"Blast Radius (Pending Manual)"</div>
                    <div class="metric-value" style="color: var(--ctp-peach);">
                        {move || metrics.get().pending_manual.to_string()}
                    </div>
                    <div class="metric-sub">
                        "Acciones L7/Globales en espera de aprobación"
                    </div>
                </div>

                // Tarjeta 4: Motor de Inferencia LightGBM
                <div class="metric-card ml-card">
                    <div class="metric-title">"Motor IA / LightGBM"</div>
                    <div class="metric-value" style="color: var(--ctp-mauve); font-size: 1.35rem; padding-top: 0.35rem;">
                        {move || format!("{:.1} µs", metrics.get().avg_latency_ns / 1000.0)}
                    </div>
                    <div class="metric-sub">
                        {move || metrics.get().lightgbm_status}
                    </div>
                </div>
            </div>

            // Estado de subsistemas y enlaces
            <div class="metric-card" style="margin-bottom: 1.5rem;">
                <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 1rem;">
                    <div>
                        <span style="font-weight: 600; font-size: 0.875rem; color: var(--ctp-text);">
                            "Estado del Pipeline: "
                        </span>
                        <span style="color: var(--ctp-green); font-weight: 700; font-family: var(--font-mono); font-size: 0.875rem;">
                            "ZERO-ALLOCATION EN LÍNEA"
                        </span>
                        <span style="color: var(--ctp-overlay1); font-size: 0.75rem; margin-left: 0.75rem;">
                            {move || format!("(Sincronización: {})", metrics.get().last_sync)}
                        </span>
                    </div>

                    <div style="display: flex; gap: 1rem; align-items: center; font-size: 0.8125rem;">
                        <span style="display: flex; align-items: center; gap: 0.35rem;">
                            <span class="status-dot" style="color: var(--ctp-green);"></span>
                            "PostgreSQL BCNF"
                        </span>
                        <span style="display: flex; align-items: center; gap: 0.35rem;">
                            <span class="status-dot" style="color: var(--ctp-green);"></span>
                            "Zero-Data FCM"
                        </span>
                        <span style="display: flex; align-items: center; gap: 0.35rem;">
                            <span class="status-dot" style="color: var(--ctp-blue);"></span>
                            "Multi-Vendor Core"
                        </span>
                    </div>
                </div>
            </div>
        </div>
    }
}
