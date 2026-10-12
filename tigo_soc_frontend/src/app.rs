// * * * COMPONENTE PRINCIPAL DE LA APLICACIÓN SPA (LEPTOS WASM) * * *

use leptos::*;

use crate::components::{AlertsTable, BlastRadiusControl, SystemStatus};
use crate::models::{
    get_initial_alerts, get_initial_metrics, get_initial_pending_mitigations,
    AlertItem, PendingMitigation, Severity, SystemMetrics,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Dashboard,
    Alerts,
    BlastRadius,
}

#[component]
pub fn App() -> impl IntoView {
    // Estados globales reactivos de la aplicación SOC
    let (metrics, set_metrics) = create_signal::<SystemMetrics>(get_initial_metrics());
    let (alerts, _set_alerts) = create_signal::<Vec<AlertItem>>(get_initial_alerts());
    let (pending_items, set_pending_items) = create_signal::<Vec<PendingMitigation>>(get_initial_pending_mitigations());
    let (selected_severity, set_selected_severity) = create_signal::<Option<Severity>>(None);
    let (active_tab, set_active_tab) = create_signal::<ActiveTab>(ActiveTab::Dashboard);

    // Efecto para sincronizar la métrica de mitigaciones pendientes cuando el usuario interactúa
    create_effect(move |_| {
        let count = pending_items.get().len() as u64;
        set_metrics.update(|m| {
            m.pending_manual = count;
        });
    });

    view! {
        <div class="soc-app">
            // Barra de Navegación Superior
            <header class="soc-header">
                <div class="soc-brand">
                    <div class="soc-logo-icon">"🛡️"</div>
                    <div>
                        <h1>"TIGO "<span>"SOC"</span>" MONITOR"</h1>
                    </div>
                </div>

                <nav class="soc-nav">
                    <button
                        class=move || if active_tab.get() == ActiveTab::Dashboard { "nav-btn active" } else { "nav-btn" }
                        on:click=move |_| set_active_tab.set(ActiveTab::Dashboard)
                    >
                        "📊 Panel de Control"
                    </button>
                    <button
                        class=move || if active_tab.get() == ActiveTab::Alerts { "nav-btn active" } else { "nav-btn" }
                        on:click=move |_| set_active_tab.set(ActiveTab::Alerts)
                    >
                        "🚨 Alertas de Seguridad"
                    </button>
                    <button
                        class=move || if active_tab.get() == ActiveTab::BlastRadius { "nav-btn active" } else { "nav-btn" }
                        on:click=move |_| set_active_tab.set(ActiveTab::BlastRadius)
                    >
                        "⚠️ Control Blast Radius "
                        <span style="background-color: var(--ctp-peach); color: var(--ctp-crust); padding: 0.1rem 0.4rem; border-radius: 10px; font-size: 0.6875rem; font-weight: 700; margin-left: 0.25rem;">
                            {move || pending_items.get().len().to_string()}
                        </span>
                    </button>
                </nav>

                <div class="soc-header-status">
                    <div class="status-badge online">
                        <span class="status-dot"></span>
                        <span>"SISTEMA EN LÍNEA"</span>
                    </div>
                </div>
            </header>

            // Contenedor de Vistas Reactivas
            <main class="soc-main">
                {move || match active_tab.get() {
                    ActiveTab::Dashboard => view! {
                        <div>
                            <SystemStatus metrics=metrics />
                            <h2 style="font-size: 1.125rem; font-weight: 700; margin-bottom: 0.75rem; color: var(--ctp-text);">
                                "Últimas Detecciones de Seguridad (Multi-Vendor)"
                            </h2>
                            <AlertsTable
                                alerts=alerts
                                selected_severity=selected_severity
                                set_selected_severity=set_selected_severity
                            />
                        </div>
                    }.into_view(),
                    ActiveTab::Alerts => view! {
                        <div>
                            <h2 style="font-size: 1.25rem; font-weight: 700; margin-bottom: 1rem; color: var(--ctp-text);">
                                "Monitoreo Forense de Alertas en Tiempo Real"
                            </h2>
                            <AlertsTable
                                alerts=alerts
                                selected_severity=selected_severity
                                set_selected_severity=set_selected_severity
                            />
                        </div>
                    }.into_view(),
                    ActiveTab::BlastRadius => view! {
                        <div>
                            <h2 style="font-size: 1.25rem; font-weight: 700; margin-bottom: 1rem; color: var(--ctp-text);">
                                "Mitigaciones Retenidas por Radio de Impacto (PENDING_MANUAL)"
                            </h2>
                            <BlastRadiusControl
                                pending_items=pending_items
                                set_pending_items=set_pending_items
                            />
                        </div>
                    }.into_view(),
                }}
            </main>

            // Pie de Página
            <footer class="soc-footer">
                <p>
                    "Tigo SOC © 2026 | Arquitectura de Ciberseguridad Agnostica Multi-Vendor (Rust Tokio + Leptos WASM + LightGBM + Axum)"
                </p>
            </footer>
        </div>
    }
}
