// * * * COMPONENTE PRINCIPAL DE LA APLICACIÓN SPA (LEPTOS WASM) * * *

use leptos::*;

use crate::components::{
    AlertsTable, BlastRadiusControl, MonitoringChart, RealtimeAlerts, SystemStatus, UserManagement,
};
use crate::models::{
    get_initial_alerts, get_initial_chart_data, get_initial_metrics,
    get_initial_pending_mitigations, get_initial_realtime_events,
    get_initial_security_config, get_initial_session, get_initial_users,
    AlertItem, GlobalSecurityConfig, MetricDataPoint, PendingMitigation,
    RealtimeStreamEvent, RoleType, Severity, SocUser, SystemMetrics, UserSessionState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Dashboard,
    MonitoringCharts,
    RealtimeStream,
    Alerts,
    BlastRadius,
    UserManagement,
}

#[component]
pub fn App() -> impl IntoView {
    // Estados globales reactivos de la aplicación SOC
    let (metrics, set_metrics) = create_signal::<SystemMetrics>(get_initial_metrics());
    let (alerts, _set_alerts) = create_signal::<Vec<AlertItem>>(get_initial_alerts());
    let (pending_items, set_pending_items) = create_signal::<Vec<PendingMitigation>>(get_initial_pending_mitigations());
    let (chart_data, set_chart_data) = create_signal::<Vec<MetricDataPoint>>(get_initial_chart_data());
    let (stream_events, set_stream_events) = create_signal::<Vec<RealtimeStreamEvent>>(get_initial_realtime_events());
    let (selected_severity, set_selected_severity) = create_signal::<Option<Severity>>(None);
    let (active_tab, set_active_tab) = create_signal::<ActiveTab>(ActiveTab::Dashboard);

    // Estados reactivos de gestión de usuarios, sesiones y RBAC
    let (users_list, set_users_list) = create_signal::<Vec<SocUser>>(get_initial_users());
    let (session, set_session) = create_signal::<UserSessionState>(get_initial_session());
    let (security_config, set_security_config) = create_signal::<GlobalSecurityConfig>(get_initial_security_config());

    // Efecto para sincronizar la métrica de mitigaciones pendientes cuando el usuario interactúa
    create_effect(move |_| {
        let count = pending_items.get().len() as u64;
        set_metrics.update(|m| {
            m.pending_manual = count;
        });
    });

    // Selector rápido de usuario para demostración de perfiles y RBAC
    let handle_switch_user = move |user_id_str: String| {
        if let Ok(uid) = user_id_str.parse::<i32>() {
            if let Some(target_user) = users_list.get().into_iter().find(|u| u.user_id == uid) {
                set_session.update(|s| {
                    s.current_user = target_user;
                    s.last_reauth_timestamp = 0.0; // Requiere re-auth al cambiar de usuario
                });
            }
        }
    };

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
                        class=move || if active_tab.get() == ActiveTab::MonitoringCharts { "nav-btn active" } else { "nav-btn" }
                        on:click=move |_| set_active_tab.set(ActiveTab::MonitoringCharts)
                    >
                        "📈 Gráficas de Monitoreo"
                    </button>
                    <button
                        class=move || if active_tab.get() == ActiveTab::RealtimeStream { "nav-btn active" } else { "nav-btn" }
                        on:click=move |_| set_active_tab.set(ActiveTab::RealtimeStream)
                    >
                        "⚡ Flujo en Vivo (Stream)"
                    </button>
                    <button
                        class=move || if active_tab.get() == ActiveTab::Alerts { "nav-btn active" } else { "nav-btn" }
                        on:click=move |_| set_active_tab.set(ActiveTab::Alerts)
                    >
                        "🚨 Alertas"
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
                    <button
                        class=move || if active_tab.get() == ActiveTab::UserManagement { "nav-btn active" } else { "nav-btn" }
                        on:click=move |_| set_active_tab.set(ActiveTab::UserManagement)
                    >
                        "👥 Usuarios & RBAC"
                    </button>
                </nav>

                <div class="soc-header-status">
                    // Selector rápido de perfil activo para pruebas RBAC
                    <div style="display: flex; align-items: center; gap: 0.4rem;">
                        <span style="font-size: 0.75rem; color: var(--ctp-subtext0);">"Perfil:"</span>
                        <select
                            class="form-control"
                            style="padding: 0.2rem 0.5rem; font-size: 0.75rem; width: auto; background-color: var(--ctp-crust);"
                            on:change=move |ev| handle_switch_user(event_target_value(&ev))
                        >
                            {move || {
                                users_list.get().into_iter().map(|u| {
                                    let is_sel = u.user_id == session.get().current_user.user_id;
                                    view! {
                                        <option value=u.user_id.to_string() selected=is_sel>
                                            {format!("{} ({})", u.username, u.role.display_name())}
                                        </option>
                                    }
                                }).collect_view()
                            }}
                        </select>
                    </div>

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
                            <div style="margin-bottom: 1.5rem;">
                                <MonitoringChart
                                    chart_data=chart_data
                                    set_chart_data=set_chart_data
                                />
                            </div>
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
                    ActiveTab::MonitoringCharts => view! {
                        <div>
                            <h2 style="font-size: 1.25rem; font-weight: 700; margin-bottom: 1rem; color: var(--ctp-text);">
                                "Telemetría en Tiempo Real y Métricas del Motor GBDT"
                            </h2>
                            <SystemStatus metrics=metrics />
                            <MonitoringChart
                                chart_data=chart_data
                                set_chart_data=set_chart_data
                            />
                        </div>
                    }.into_view(),
                    ActiveTab::RealtimeStream => view! {
                        <div>
                            <h2 style="font-size: 1.25rem; font-weight: 700; margin-bottom: 1rem; color: var(--ctp-text);">
                                "Flujo Continuo de Eventos Forenses y Alertas en Streaming"
                            </h2>
                            <RealtimeAlerts
                                events=stream_events
                                set_events=set_stream_events
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
                                session=session
                                set_session=set_session
                                security_config=security_config
                            />
                        </div>
                    }.into_view(),
                    ActiveTab::UserManagement => view! {
                        <div>
                            <h2 style="font-size: 1.25rem; font-weight: 700; margin-bottom: 1rem; color: var(--ctp-text);">
                                "Administración de Usuarios, Control de Acceso (RBAC) y Seguridad Zero-Trust"
                            </h2>
                            <UserManagement
                                users=users_list
                                set_users=set_users_list
                                session=session
                                set_session=set_session
                                security_config=security_config
                                set_security_config=set_security_config
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
