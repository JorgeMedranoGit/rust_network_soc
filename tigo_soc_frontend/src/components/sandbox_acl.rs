// * * * LÁMINA 10/21 DEL MANUAL DE MOCKUPS: SANDBOX ACL EN SUBRED AISLADA * * *
// Ambiente de pruebas VLAN 999 (10.200.0.0/24) — inyección de tráfico simulado,
// consola eBPF y promoción controlada a producción.
// Identificación visual EXCLUSIVA con índigo #4F46E5 según la paleta institucional.

use leptos::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::app::TIGO_INDIGO;
use crate::models::{AlertItem, RealtimeStreamEvent};

#[derive(Debug, Clone, PartialEq)]
struct SandboxLogLine {
    ts: String,
    level: String,
    msg: String,
}

fn now_hms() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}

/// Topología fija de la subred aislada documentada (VLAN 999 / 10.200.0.0/24).
/// No son datos simulados del SOC: es la ficha descriptiva del banco de pruebas.
const SANDBOX_HOSTS: [(&str, &str); 4] = [
    ("10.200.0.1", "GW-SANDBOX (ACL egress default-drop)"),
    ("10.200.0.10", "kali-sim-01 (inyector de tráfico)"),
    ("10.200.0.20", "victim-honeypot-01 (captura forense)"),
    ("10.200.0.30", "sensor-eBPF-01 (espejo de paquetes)"),
];

#[component]
pub fn SandboxAcl(
    /// Flujo en vivo compartido — los eventos sandbox se promueven aquí tras validación
    set_stream_events: WriteSignal<Vec<RealtimeStreamEvent>>,
    /// Alertas reales del backend para extraer firmas/promover incidentes de prueba
    alerts: ReadSignal<Vec<AlertItem>>,
) -> impl IntoView {
    let (sandbox_logs, set_sandbox_logs) = create_signal::<Vec<SandboxLogLine>>(Vec::new());
    let (injecting, set_injecting) = create_signal(false);
    let (promoted_count, set_promoted_count) = create_signal(0u32);
    let (selected_threat, set_selected_threat) = create_signal("DNS_EXFILTRATION".to_string());
    let (backend_error, set_backend_error) = create_signal::<Option<String>>(None);

    // ── Inyección de tráfico simulado → POST /api/v1/alerts/simulate-push (API REAL) ──
    let handle_inject = move |_| {
        if injecting.get() {
            return;
        }
        set_injecting.set(true);
        let threat = selected_threat.get();
        set_sandbox_logs.update(|l| {
            l.insert(
                0,
                SandboxLogLine {
                    ts: now_hms(),
                    level: "INFO".into(),
                    msg: format!(
                        "xdp-inject --vlan 999 --src 10.200.0.10 → enviando ráfaga '{}' a motor LightGBM…",
                        threat
                    ),
                },
            );
        });
        spawn_local(async move {
            match api::simulate_push(&threat, "CRITICAL").await {
                Ok(resp) => {
                    set_backend_error.set(None);
                    set_sandbox_logs.update(|l| {
                        l.insert(
                            0,
                            SandboxLogLine {
                                ts: now_hms(),
                                level: "OK".into(),
                                msg: format!(
                                    "Inyección sandbox completada (alerta id={} persistida en PostgreSQL). {}",
                                    resp.alert_id.unwrap_or(0),
                                    resp.message.unwrap_or_default()
                                ),
                            },
                        );
                    });
                }
                Err(e) => {
                    set_backend_error.set(Some(e.clone()));
                    set_sandbox_logs.update(|l| {
                        l.insert(
                            0,
                            SandboxLogLine {
                                ts: now_hms(),
                                level: "ERR".into(),
                                msg: format!("Fallo de inyección simulada: {}", e),
                            },
                        );
                    });
                }
            }
            set_injecting.set(false);
        });
    };

    // ── Promoción controlada: mueve la última alerta CRÍTICA real al flujo en vivo ──
    let handle_promote = move |_| {
        let last_critical = alerts
            .get()
            .iter()
            .find(|a| a.severity == crate::models::Severity::CRITICAL)
            .cloned();
        match last_critical {
            Some(a) => {
                let evt = RealtimeStreamEvent {
                    event_id: a.alert_id,
                    timestamp: a.detected_at.clone(),
                    threat_category: a.threat_name.clone(),
                    severity: a.severity.clone(),
                    layer: a.layer.clone(),
                    source_ip: a.source_ip.clone(),
                    destination_ip: a.destination_ip.clone(),
                    protocol: a.protocol.clone(),
                    message: format!(
                        "[PROMOCIÓN SANDBOX→PROD] {} validado en VLAN 999 y ascendido a vigilancia continua.",
                        a.threat_name
                    ),
                    mitigation_strategy: a.mitigation_action.clone(),
                    blast_radius: "CONTROLLED_SANDBOX_VALIDATED".to_string(),
                };
                set_stream_events.update(|list| {
                    list.insert(0, evt);
                    if list.len() > 30 {
                        list.pop();
                    }
                });
                set_promoted_count.update(|c| *c += 1);
                set_sandbox_logs.update(|l| {
                    l.insert(
                        0,
                        SandboxLogLine {
                            ts: now_hms(),
                            level: "WARN".into(),
                            msg: format!(
                                "Promoción aprobada: alerta #{} ({}) visible en Flujo en Vivo.",
                                a.alert_id, a.threat_name
                            ),
                        },
                    );
                });
            }
            None => {
                set_sandbox_logs.update(|l| {
                    l.insert(
                        0,
                        SandboxLogLine {
                            ts: now_hms(),
                            level: "ERR".into(),
                            msg: "Nada que promover: el polling de /api/v1/alerts aún no reporta alertas CRÍTICAS.".into(),
                        },
                    );
                });
            }
        }
    };

    let clear_console = move |_| set_sandbox_logs.set(Vec::new());

    view! {
        <div class="card sandbox-card" style=format!(
            "border: 1px solid {}; box-shadow: 0 0 0 1px {}22;",
            TIGO_INDIGO, TIGO_INDIGO
        )>
            <div class="card-header" style=format!(
                "background: linear-gradient(135deg, {}33, transparent); border-bottom: 1px solid {}55;",
                TIGO_INDIGO, TIGO_INDIGO
            )>
                <h3 style="font-size: 1.05rem; font-weight: 700;">
                    <span style=format!("color: {};", TIGO_INDIGO)>"🧪 Sandbox ACL — VLAN 999"</span>
                    <span class="mono-dim" style="margin-left: 0.75rem; font-size: 0.78rem;">
                        "Subred aislada 10.200.0.0/24 • Tráfico sintético • Sin impacto en producción"
                    </span>
                </h3>
                <div style="display:flex; gap:0.5rem; align-items:center;">
                    <span
                        class="badge"
                        style=format!(
                            "background: {}22; color: {}; border: 1px solid {};",
                            TIGO_INDIGO, TIGO_INDIGO, TIGO_INDIGO
                        )
                    >
                        {move || format!("Promovidas: {}", promoted_count.get())}
                    </span>
                </div>
            </div>

            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1.25rem; padding: 1.25rem;">
                // ══ Panel izquierdo: hosts de la subred + controles ══
                <div>
                    <h4 style="font-size: 0.85rem; font-weight: 700; margin-bottom: 0.6rem; color: var(--ctp-text);">
                        "Hosts registrados en la subred de pruebas"
                    </h4>
                    <table class="data-table" style="font-size: 0.78rem;">
                        <thead>
                            <tr>
                                <th>"IP (CIDR /32)"</th>
                                <th>"Rol del banco de pruebas"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {SANDBOX_HOSTS.iter().map(|(ip, role)| view! {
                                <tr>
                                    <td class="mono" style=format!("color: {};", TIGO_INDIGO)>{ip.to_string()}</td>
                                    <td style="color: var(--ctp-subtext0);">{role.to_string()}</td>
                                </tr>
                            }).collect_view()}
                        </tbody>
                    </table>

                    <div style="margin-top: 1.1rem; display:flex; flex-direction:column; gap:0.6rem;">
                        <label style="font-size:0.78rem; font-weight:600; color:var(--ctp-subtext1);">
                            "Escenario de ataque a inyectar (catálogo threat_types)"
                        </label>
                        <select
                            class="login-input"
                            value=move || selected_threat.get()
                            on:input=move |ev| set_selected_threat.set(event_target_value(&ev))
                        >
                            <option value="DNS_EXFILTRATION">"DNS_EXFILTRATION (L7 · sinkhole)"</option>
                            <option value="DDOS_SYN_FLOOD">"DDOS_SYN_FLOOD (L4 · blackhole)"</option>
                            <option value="MAC_SPOOFING">"MAC_SPOOFING (L2 · RADIUS CoA)"</option>
                            <option value="PORT_SCAN">"PORT_SCAN (L3 · ACL)"</option>
                        </select>

                        <div style="display:flex; gap:0.6rem; flex-wrap:wrap;">
                            <button
                                class="btn btn-primary"
                                style=format!("background: {}; border-color: {};", TIGO_INDIGO, TIGO_INDIGO)
                                disabled=move || injecting.get()
                                on:click=handle_inject
                            >
                                {move || if injecting.get() {
                                    "⏳ Inyectando tráfico…"
                                } else {
                                    "💉 Inyectar tráfico simulado"
                                }}
                            </button>
                            <button class="btn btn-inspect" on:click=handle_promote>
                                "⬆️ Promover a producción"
                            </button>
                            <button class="btn btn-inspect" on:click=clear_console>
                                "🧹 Limpiar consola"
                            </button>
                        </div>

                        {move || backend_error.get().map(|e| view! {
                            <div class="alert-box alert-danger" style="font-size:0.78rem;">
                                {format!("Backend rechazó la operación: {}", e)}
                            </div>
                        })}

                        <p style="font-size:0.72rem; color:var(--ctp-overlay1); line-height:1.5;">
                            "La inyección ejecuta una petición REAL contra POST /api/v1/alerts/simulate-push:
                            el motor LightGBM clasifica el vector sintético y persiste la alerta en PostgreSQL.
                            La promoción solo asciende alertas críticas ya validadas por el analista."
                        </p>
                    </div>
                </div>

                // ══ Panel derecho: consola eBPF ══
                <div>
                    <h4 style="font-size: 0.85rem; font-weight: 700; margin-bottom: 0.6rem; color: var(--ctp-text);">
                        "🖥️ Consola eBPF/XDP del sensor sandbox"
                    </h4>
                    <div
                        class="console-output"
                        style="min-height: 260px; max-height: 340px; overflow-y: auto; font-size: 0.72rem;"
                    >
                        {move || {
                            let logs = sandbox_logs.get();
                            if logs.is_empty() {
                                return view! {
                                    <div class="console-line" style="color: var(--ctp-overlay1);">
                                        "# tc qdisc add dev eth9 clsact  —  sensor en espera de órdenes…"
                                    </div>
                                }.into_view();
                            }
                            logs.into_iter().map(|line| {
                                let color = match line.level.as_str() {
                                    "OK" => "var(--status-green)",
                                    "ERR" => "var(--status-red)",
                                    "WARN" => "var(--status-amber)",
                                    _ => "var(--ctp-subtext0)",
                                };
                                view! {
                                    <div class="console-line">
                                        <span class="mono-dim">{format!("[{}] ", line.ts)}</span>
                                        <span style=format!("color: {}; font-weight:700;", color)>
                                            {format!("{:>4}", line.level)}
                                        </span>
                                        <span>" "</span>
                                        <span style="color: var(--ctp-text);">{line.msg}</span>
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
