// * * * COMPONENTE PRINCIPAL DE LA APLICACIÓN SPA (LEPTOS WASM) * * *
// Conexión activa con Backend Axum en :3000 via gloo-net HTTP (polling cada 15s)

use leptos::*;
use gloo_net::http::Request;
use gloo_timers::future::IntervalStream;
use wasm_bindgen_futures::spawn_local;

use crate::components::{
    AlertsTable, BlastRadiusControl, LoginView, MonitoringChart, RealtimeAlerts,
    SystemStatus, Topology3D, UserManagement,
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
    Topology,
    MonitoringCharts,
    RealtimeStream,
    Alerts,
    BlastRadius,
    UserManagement,
}

#[component]
pub fn App() -> impl IntoView {
    // ── Estados globales reactivos ──────────────────────────────────────────
    let (metrics, set_metrics) = create_signal::<SystemMetrics>(get_initial_metrics());
    let (alerts, _set_alerts) = create_signal::<Vec<AlertItem>>(get_initial_alerts());
    let (pending_items, set_pending_items) =
        create_signal::<Vec<PendingMitigation>>(get_initial_pending_mitigations());
    let (chart_data, set_chart_data) =
        create_signal::<Vec<MetricDataPoint>>(get_initial_chart_data());
    let (stream_events, set_stream_events) =
        create_signal::<Vec<RealtimeStreamEvent>>(get_initial_realtime_events());
    let (selected_severity, set_selected_severity) =
        create_signal::<Option<Severity>>(None);
    let (active_tab, set_active_tab) = create_signal::<ActiveTab>(ActiveTab::Dashboard);

    // ── Estados de gestión de usuarios, sesiones y RBAC ────────────────────
    let (users_list, set_users_list) =
        create_signal::<Vec<SocUser>>(get_initial_users());
    let (session, set_session) =
        create_signal::<UserSessionState>(get_initial_session());
    let (security_config, set_security_config) =
        create_signal::<GlobalSecurityConfig>(get_initial_security_config());

    // ── Estado de autenticación ─────────────────────────────────────────────
    // is_authenticated: false = pantalla de login, true = dashboard SOC
    let (is_authenticated, set_is_authenticated) = create_signal::<bool>(false);

    // Efecto: sincronizar mitigaciones pendientes con las métricas
    create_effect(move |_| {
        let count = pending_items.get().len() as u64;
        set_metrics.update(|m| {
            m.pending_manual = count;
        });
    });

    // Callback de login exitoso
    let on_login_success = Callback::new(move |()| {
        set_is_authenticated.set(true);
    });

    // Logout
    let handle_logout = move |_| {
        set_session.update(|s| {
            s.last_reauth_timestamp = 0.0;
            s.session_token = String::new();
        });
        set_is_authenticated.set(false);
        set_active_tab.set(ActiveTab::Dashboard);
    };

    view! {
        {move || {
            if !is_authenticated.get() {
                // ── Pantalla de Login / Registro ──────────────────────────────
                view! {
                    <LoginView
                        users=users_list
                        set_users=set_users_list
                        session=session
                        set_session=set_session
                        on_login_success=on_login_success
                    />
                }.into_view()
            } else {
                // ── Aplicación SOC Principal ──────────────────────────────────
                view! {
                    <div class="soc-app">
                        // ── Barra de Navegación Superior ─────────────────────
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
                                    class=move || if active_tab.get() == ActiveTab::Topology { "nav-btn active" } else { "nav-btn" }
                                    on:click=move |_| set_active_tab.set(ActiveTab::Topology)
                                >
                                    "🌐 Topología 3D"
                                </button>
                                <button
                                    class=move || if active_tab.get() == ActiveTab::MonitoringCharts { "nav-btn active" } else { "nav-btn" }
                                    on:click=move |_| set_active_tab.set(ActiveTab::MonitoringCharts)
                                >
                                    "📈 Gráficas"
                                </button>
                                <button
                                    class=move || if active_tab.get() == ActiveTab::RealtimeStream { "nav-btn active" } else { "nav-btn" }
                                    on:click=move |_| set_active_tab.set(ActiveTab::RealtimeStream)
                                >
                                    "⚡ Flujo en Vivo"
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
                                    "⚠️ Blast Radius "
                                    <span style="background-color: var(--ctp-peach); color: var(--ctp-crust); padding: 0.1rem 0.4rem; border-radius: 10px; font-size: 0.6875rem; font-weight: 700; margin-left: 0.25rem;">
                                        {move || pending_items.get().len().to_string()}
                                    </span>
                                </button>
                                // Pestaña de Gestión de Usuarios — solo visible para Administrador General
                                {move || {
                                    if session.get().current_user.role == RoleType::Admin {
                                        view! {
                                            <button
                                                class=move || if active_tab.get() == ActiveTab::UserManagement { "nav-btn active" } else { "nav-btn" }
                                                on:click=move |_| set_active_tab.set(ActiveTab::UserManagement)
                                            >
                                                "👥 Usuarios & RBAC"
                                            </button>
                                        }.into_view()
                                    } else {
                                        view! {}.into_view()
                                    }
                                }}
                            </nav>

                            <div class="soc-header-status">
                                // Badge de usuario autenticado
                                <div style="display:flex; align-items:center; gap:0.4rem; font-size:0.78rem; color:var(--ctp-subtext0);">
                                    <span style="color:var(--ctp-green);">"✓"</span>
                                    {move || session.get().current_user.username.clone()}
                                    <span class=move || match session.get().current_user.role {
                                        RoleType::Admin => "badge-role badge-role-admin",
                                        RoleType::SocOperator => "badge-role badge-role-operator",
                                        RoleType::Auditor => "badge-role badge-role-auditor",
                                        RoleType::AnalystL1 => "badge-role badge-role-l1",
                                    }>
                                        {move || session.get().current_user.role.display_name()}
                                    </span>
                                </div>

                                <div class="status-badge online">
                                    <span class="status-dot"></span>
                                    <span>"EN LÍNEA"</span>
                                </div>

                                // Botón de logout
                                <button
                                    class="btn btn-inspect"
                                    style="padding: 0.35rem 0.7rem; font-size: 0.75rem; border-color: var(--ctp-red); color: var(--ctp-red);"
                                    on:click=handle_logout
                                >
                                    "⏏ Cerrar Sesión"
                                </button>
                            </div>
                        </header>

                        // ── Contenedor Principal de Vistas ────────────────────
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

                                ActiveTab::Topology => view! {
                                    <div>
                                        <Topology3D />
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
                                        {move || {
                                            if session.get().current_user.role == RoleType::Admin {
                                                view! {
                                                    <UserManagement
                                                        users=users_list
                                                        set_users=set_users_list
                                                        session=session
                                                        set_session=set_session
                                                        security_config=security_config
                                                        set_security_config=set_security_config
                                                    />
                                                }.into_view()
                                            } else {
                                                view! {
                                                    <div style="
                                                        display: flex;
                                                        flex-direction: column;
                                                        align-items: center;
                                                        justify-content: center;
                                                        padding: 4rem 2rem;
                                                        text-align: center;
                                                    ">
                                                        <div style="font-size: 3.5rem; margin-bottom: 1rem;">"🔒"</div>
                                                        <h3 style="font-size: 1.5rem; font-weight: 700; color: var(--ctp-red); margin-bottom: 0.5rem;">
                                                            "403 — Acceso Denegado"
                                                        </h3>
                                                        <p style="font-size: 0.9rem; color: var(--ctp-subtext0); max-width: 480px; line-height: 1.6;">
                                                            "El módulo de administración de usuarios y control RBAC está restringido "
                                                            "exclusivamente al rol "
                                                            <strong style="color: var(--ctp-peach);">"Administrador General"</strong>
                                                            ". Su perfil actual no posee los permisos requeridos (ISO/IEC 27001 — Principio de Mínimo Privilegio)."
                                                        </p>
                                                        <div style="margin-top: 1.5rem;">
                                                            <span class="badge-role badge-role-admin" style="font-size: 0.875rem; padding: 0.4rem 0.75rem;">
                                                                "Solo Administrador General"
                                                            </span>
                                                        </div>
                                                    </div>
                                                }.into_view()
                                            }
                                        }}
                                    </div>
                                }.into_view(),
                            }}
                        </main>

                        // ── Pie de Página ─────────────────────────────────────
                        <footer class="soc-footer">
                            <p>
                                "Tigo SOC © 2026 | Arquitectura Ciberseguridad Multi-Vendor (Rust Tokio + Leptos WASM + LightGBM + Axum) | Bolivia"
                            </p>
                        </footer>
                    </div>
                }.into_view()
            }
        }}
    }
}
