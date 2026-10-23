// * * * COMPONENTE: VISTA DE CONTROL Y APROBACIÓN MANUAL DE BLAST RADIUS CON RE-AUTENTICACIÓN ESTRICTA * * *

use leptos::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;
use crate::models::{GlobalSecurityConfig, PendingMitigation, RoleType, UserSessionState};

#[component]
pub fn BlastRadiusControl(
    pending_items: ReadSignal<Vec<PendingMitigation>>,
    set_pending_items: WriteSignal<Vec<PendingMitigation>>,
    session: ReadSignal<UserSessionState>,
    set_session: WriteSignal<UserSessionState>,
    security_config: ReadSignal<GlobalSecurityConfig>,
) -> impl IntoView {
    let (modal_json, set_modal_json) = create_signal(Option::<String>::None);
    let (feedback_msg, set_feedback_msg) = create_signal(Option::<String>::None);
    let (error_msg, set_error_msg) = create_signal(Option::<String>::None);

    // Estado del Modal de Re-autenticación Crítica: Option<(action_type, task_id, description)>
    let (pending_critical_action, set_pending_critical_action) = create_signal(Option::<(String, i64, String)>::None);
    let (reauth_password, set_reauth_password) = create_signal(String::new());
    let (reauth_error, set_reauth_error) = create_signal(Option::<String>::None);

    // Evaluación reactiva de la validez del timeout de re-autenticación
    let is_reauth_valid = move || {
        let last = session.get().last_reauth_timestamp;
        if last <= 0.0 {
            return false;
        }
        let now = chrono::Utc::now().timestamp() as f64;
        let timeout = security_config.get().critical_action_timeout_secs as f64;
        (now - last) < timeout
    };

    // Tiempo restante de gracia para acciones críticas
    let remaining_seconds = move || {
        let last = session.get().last_reauth_timestamp;
        if last <= 0.0 {
            return 0i64;
        }
        let now = chrono::Utc::now().timestamp() as f64;
        let timeout = security_config.get().critical_action_timeout_secs as f64;
        let rem = timeout - (now - last);
        if rem > 0.0 { rem as i64 } else { 0i64 }
    };

    // Despacho REAL de la mitigación aprobada: POST /api/v1/alerts/simulate-push
    // (el orquestador del backend procesa el evento CEF y aplica la directiva).
    // Solo se retira de la cola local si el servidor Axum lo confirma.
    let execute_approved_mitigation = move |task_id: i64| {
        let token = session.get().session_token.clone();
        let alert_snapshot = pending_items.get().into_iter().find(|i| i.task_id == task_id);
        let set_pending_items_e = set_pending_items;
        let set_feedback_msg_e = set_feedback_msg;
        let set_error_msg_e = set_error_msg;

        spawn_local(async move {
            match alert_snapshot {
                Some(alert) => match api::dispatch_mitigation(&token, &alert).await {
                    Ok(_) => {
                        set_pending_items_e.update(|items| {
                            items.retain(|i| i.task_id != task_id);
                        });
                        set_feedback_msg_e.set(Some(format!(
                            "✓ Mitigación Masiva (Tarea #{}) APROBADA Y DESPACHADA AL ORQUESTADOR (POST /api/v1/alerts/simulate-push). Re-autenticación verificada.",
                            task_id
                        )));
                    }
                    Err(e) => {
                        set_error_msg_e.set(Some(format!(
                            "✕ El backend rechazó el despacho de la mitigación (Tarea #{}): {}",
                            task_id, e
                        )));
                    }
                },
                None => {
                    set_error_msg_e.set(Some(format!("La Tarea #{} ya no está encolada (fue consumida por el polling).", task_id)));
                }
            }
        });
    };

    // Rollback: mismo canal de despacho (directiva inversa registrada en mitigation_audit)
    let execute_rollback_action = move |task_id: i64| {
        let token = session.get().session_token.clone();
        let mut snapshot = pending_items.get().into_iter().find(|i| i.task_id == task_id);
        let set_pending_items_r = set_pending_items;
        let set_modal_json_r = set_modal_json;
        let set_feedback_msg_r = set_feedback_msg;
        let set_error_msg_r = set_error_msg;

        if let Some(a) = snapshot.as_mut() {
            a.action_name = format!("ROLLBACK_{}", a.action_name);
        }

        spawn_local(async move {
            match snapshot {
                Some(alert) => match api::dispatch_mitigation(&token, &alert).await {
                    Ok(_) => {
                        set_pending_items_r.update(|items| {
                            items.retain(|i| i.task_id != task_id);
                        });
                        set_modal_json_r.set(None);
                        set_feedback_msg_r.set(Some(format!(
                            "🔄 Rollback (Tarea #{}) EJECUTADO. Directiva inversa despachada al hardware perimetral vía backend y registrada en mitigation_audit.",
                            task_id
                        )));
                    }
                    Err(e) => {
                        set_error_msg_r.set(Some(format!(
                            "✕ El backend rechazó el rollback (Tarea #{}): {}",
                            task_id, e
                        )));
                    }
                },
                None => {
                    set_error_msg_r.set(Some(format!("La Tarea #{} ya no está disponible para rollback.", task_id)));
                }
            }
        });
    };

    // Despacho de mitigación con control RBAC y timeout de re-autenticación
    let request_approve_mitigation = move |task_id: i64| {
        set_error_msg.set(None);
        set_feedback_msg.set(None);

        let user_role = session.get().current_user.role;
        if !user_role.can_execute_destructive_actions() {
            set_error_msg.set(Some(
                "✕ ACCESO DENEGADO (RBAC): Su rol actual no posee permisos para despachar mitigaciones de red. Roles autorizados: Administrador General y Operador SOC."
                    .to_string(),
            ));
            return;
        }

        if is_reauth_valid() {
            execute_approved_mitigation(task_id);
        } else {
            // Desplegar Modal de Re-autenticación Obligatoria por Contraseña
            set_pending_critical_action.set(Some((
                "MITIGATE".to_string(),
                task_id,
                format!("Aprobación y despacho de mitigación masiva de Capa 7 / Global (Tarea #{})", task_id),
            )));
            set_reauth_password.set(String::new());
            set_reauth_error.set(None);
        }
    };

    // Despacho de Rollback con control RBAC y timeout de re-autenticación
    let request_execute_rollback = move |task_id: i64| {
        set_error_msg.set(None);
        set_feedback_msg.set(None);

        let user_role = session.get().current_user.role;
        if !user_role.can_execute_destructive_actions() {
            set_error_msg.set(Some(
                "✕ ACCESO DENEGADO (RBAC): Su rol actual no posee permisos para ejecutar reversiones (Rollback). Roles autorizados: Administrador General y Operador SOC."
                    .to_string(),
            ));
            return;
        }

        if is_reauth_valid() {
            execute_rollback_action(task_id);
        } else {
            // Desplegar Modal de Re-autenticación Obligatoria por Contraseña
            set_pending_critical_action.set(Some((
                "ROLLBACK".to_string(),
                task_id,
                format!("Ejecución de Rollback JSON Snapshot para reversión de topología (Tarea #{})", task_id),
            )));
            set_reauth_password.set(String::new());
            set_reauth_error.set(None);
        }
    };

    // Rechazo: descarte LOCAL de la cola. La alerta permanece en PostgreSQL con
    // estado PENDING_MANUAL; el siguiente ciclo de polling (15 s) la reincorporará
    // si el backend no la marca como descartada por otra vía.
    let reject_mitigation = move |task_id: i64| {
        let user_role = session.get().current_user.role;
        if !user_role.can_execute_destructive_actions() {
            set_error_msg.set(Some(
                "✕ ACCESO DENEGADO (RBAC): Su rol no permite descartar o modificar mitigaciones pendientes. Roles autorizados: Administrador y Operador SOC.".to_string(),
            ));
            return;
        }

        set_pending_items.update(|items| {
            items.retain(|i| i.task_id != task_id);
        });
        set_feedback_msg.set(Some(format!(
            "✕ Mitigación (Tarea #{}) RECHAZADA por el analista. No se despachó ninguna directiva al orquestador.",
            task_id
        )));
    };

    // Validación REAL de la contraseña ingresada en el Modal de Re-autenticación:
    // POST /api/v1/auth/reauth contra Axum (verificación BCrypt + ventana Zero-Trust).
    let confirm_reauth = move |_| {
        set_reauth_error.set(None);
        let pass = reauth_password.get();

        if pass.trim().is_empty() {
            set_reauth_error.set(Some("Por favor ingrese la contraseña del usuario activo.".to_string()));
            return;
        }

        // Re-autenticación REAL contra el backend: POST /api/v1/auth/reauth
        // (Axum verifica BCrypt y actualiza last_reauth_at en user_sessions).
        let token = session.get().session_token.clone();
        let set_session_c = set_session;
        let set_reauth_error_c = set_reauth_error;
        let pending_action_snapshot = pending_critical_action.get();
        let exec_ok = execute_approved_mitigation.clone();
        let rollback_ok = execute_rollback_action.clone();
        let set_pending_critical_action_c = set_pending_critical_action;

        spawn_local(async move {
            match api::reauth(&token, &pass).await {
                Ok(resp) => {
                    // Sincronizar el timestamp de re-autenticación informado por el servidor
                    let server_ts = resp
                        .reauth_timestamp
                        .map(|dt| dt.timestamp() as f64)
                        .unwrap_or_else(|| chrono::Utc::now().timestamp() as f64);
                    set_session_c.update(|s| s.last_reauth_timestamp = server_ts);

                    // Ejecutar la acción crítica pendiente tras validación del servidor
                    if let Some((action_type, task_id, _)) = pending_action_snapshot {
                        if action_type == "MITIGATE" {
                            exec_ok(task_id);
                        } else if action_type == "ROLLBACK" {
                            rollback_ok(task_id);
                        }
                    }
                    set_pending_critical_action_c.set(None);
                }
                Err(e) => {
                    set_reauth_error_c.set(Some(format!(
                        "✕ El backend rechazó la re-autenticación: {}",
                        e
                    )));
                }
            }
        });
    };

    view! {
        <div class="blast-container">
            // Banner explicativo de la Regla de Blast Radius y Re-autenticación
            <div class="blast-banner">
                <div class="blast-banner-icon">"⚠️"</div>
                <div class="blast-banner-text">
                    <h3>"Control de Impacto Masivo (Blast Radius) y Re-autenticación Estricta"</h3>
                    <p>
                        "Las mitigaciones masivas (ej. intercepción DNS Sinkholing o Blackholing perimetral) "
                        "y las directivas de Rollback exigen " <strong style="color: var(--ctp-peach);">"re-autenticación por contraseña"</strong> ". "
                        "Si el tiempo de inactividad supera el umbral configurable ("
                        <strong style="color: var(--ctp-text);">{move || format!("{}s", security_config.get().critical_action_timeout_secs)}</strong>
                        "), el sistema solicitará la contraseña antes de despachar el comando sin expulsar al usuario de la vista de alertas."
                    </p>
                </div>
            </div>

            // Barra de estado de re-autenticación activa
            <div class="stream-control-bar" style="margin-bottom: 0;">
                <div style="display: flex; align-items: center; gap: 0.75rem; flex-wrap: wrap;">
                    <span style="font-size: 0.84rem; color: var(--ctp-subtext0);">
                        "Operador activo: " <strong style="color: var(--ctp-text);">{move || session.get().current_user.username}</strong>
                    </span>
                    <span class=move || match session.get().current_user.role {
                        RoleType::Admin => "badge-role badge-role-admin",
                        RoleType::SocOperator => "badge-role badge-role-operator",
                        RoleType::Auditor => "badge-role badge-role-auditor",
                        RoleType::AnalystL1 => "badge-role badge-role-l1",
                    }>
                        {move || session.get().current_user.role.display_name()}
                    </span>
                </div>

                <div style="display: flex; align-items: center; gap: 0.75rem;">
                    {move || {
                        if is_reauth_valid() {
                            let rem = remaining_seconds();
                            view! {
                                <div class="status-badge online" style="background-color: rgba(166, 227, 161, 0.15); color: var(--ctp-green);">
                                    <span class="status-dot"></span>
                                    <span>{format!("Re-autenticación Válida ({}s restantes)", rem)}</span>
                                </div>
                            }.into_view()
                        } else {
                            view! {
                                <div class="status-badge" style="background-color: rgba(250, 179, 135, 0.15); color: var(--ctp-peach); border: 1px solid rgba(250, 179, 135, 0.3);">
                                    <span>"🔒 Re-autenticación Requerida para Despacho"</span>
                                </div>
                            }.into_view()
                        }
                    }}
                </div>
            </div>

            // Mensajes de Feedback o Error
            {move || {
                if let Some(msg) = feedback_msg.get() {
                    view! {
                        <div class="alert-box alert-success">
                            {msg}
                        </div>
                    }.into_view()
                } else if let Some(err) = error_msg.get() {
                    view! {
                        <div class="alert-box alert-danger">
                            {err}
                        </div>
                    }.into_view()
                } else {
                    view! {}.into_view()
                }
            }}

            // Lista de tareas pendientes de aprobación manual
            {move || {
                let items = pending_items.get();
                if items.is_empty() {
                    view! {
                        <div class="blast-card" style="text-align: center; padding: 3rem 1rem; color: var(--ctp-subtext0);">
                            <div style="font-size: 2rem; margin-bottom: 0.5rem;">"✅"</div>
                            <h4 style="color: var(--ctp-text);">"No hay mitigaciones pendientes de aprobación manual"</h4>
                            <p style="font-size: 0.8125rem; color: var(--ctp-overlay1); margin-top: 0.25rem;">
                                "Todas las acciones de Blast Radius singular se han ejecutado automáticamente."
                            </p>
                        </div>
                    }.into_view()
                } else {
                    items.into_iter().map(|item| {
                        let task_id = item.task_id;
                        let json_payload = item.rollback_payload.clone();
                        let is_no_perms = !session.get().current_user.role.can_execute_destructive_actions();

                        view! {
                            <div class="blast-card" style="border-left: 4px solid var(--ctp-peach);">
                                <div class="blast-card-header">
                                    <div>
                                        <span class="badge-layer L7" style="margin-right: 0.5rem;">
                                            {item.layer_target.clone()}
                                        </span>
                                        <strong style="font-size: 1rem; color: var(--ctp-text);">
                                            {item.action_name.clone()}
                                        </strong>
                                        <span style="font-size: 0.75rem; color: var(--ctp-overlay1); margin-left: 0.5rem;">
                                            {format!("(Tarea #{} / Alerta #{})", item.task_id, item.alert_id)}
                                        </span>
                                    </div>
                                    <div>
                                        <span class="badge-sev CRITICAL" style="background-color: rgba(250, 179, 135, 0.2); color: var(--ctp-peach); border-color: var(--ctp-peach);">
                                            {item.blast_radius.clone()}
                                        </span>
                                    </div>
                                </div>

                                <div class="blast-card-body">
                                    <div class="blast-prop">
                                        <span class="blast-prop-label">"IP Objetivo:"</span>
                                        <span class="blast-prop-val">{item.target_ip.clone()}</span>
                                    </div>
                                    <div class="blast-prop">
                                        <span class="blast-prop-label">"MAC Resolvida:"</span>
                                        <span class="blast-prop-val">{item.target_mac.clone()}</span>
                                    </div>
                                    <div class="blast-prop">
                                        <span class="blast-prop-label">"Encolado en BD:"</span>
                                        <span class="blast-prop-val">{item.queued_at.clone()}</span>
                                    </div>
                                    <div class="blast-prop" style="grid-column: 1 / -1;">
                                        <span class="blast-prop-label">"Motivo de Detención:"</span>
                                        <span style="color: var(--ctp-subtext1);">{item.reason.clone()}</span>
                                    </div>
                                </div>

                                <div class="blast-actions">
                                    <button
                                        class="btn btn-inspect"
                                        on:click={
                                            let json_copy = json_payload.clone();
                                            move |_| set_modal_json.set(Some(json_copy.clone()))
                                        }
                                    >
                                        "🔍 Inspeccionar Rollback JSON"
                                    </button>
                                    <button
                                        class="btn btn-reject"
                                        disabled=is_no_perms
                                        on:click=move |_| reject_mitigation(task_id)
                                    >
                                        "✕ Rechazar / Descartar"
                                    </button>
                                    <button
                                        class="btn btn-approve"
                                        disabled=is_no_perms
                                        on:click=move |_| request_approve_mitigation(task_id)
                                    >
                                        "✓ Aprobar y Ejecutar Mitigación"
                                    </button>
                                </div>
                            </div>
                        }
                    }).collect_view()
                }
            }}

            // Modal para inspección del Rollback JSON Snapshot y Ejecución Reversible
            {move || {
                if let Some(json_content) = modal_json.get() {
                    let task_id_opt = pending_items.get().first().map(|i| i.task_id).unwrap_or(1);
                    view! {
                        <div class="modal-overlay" on:click=move |_| set_modal_json.set(None)>
                            <div class="modal-content" on:click=|ev| ev.stop_propagation()>
                                <div class="modal-header">
                                    <h3>"Snapshot de Reversión (Rollback Payload JSON)"</h3>
                                    <button class="modal-close" on:click=move |_| set_modal_json.set(None)>
                                        "✕"
                                    </button>
                                </div>
                                <div class="modal-body">
                                    <p style="font-size: 0.8125rem; color: var(--ctp-subtext0); margin-bottom: 0.75rem;">
                                        "Directiva inversa registrada para reversión en " <code>"rollback_snapshots"</code> ":"
                                    </p>
                                    <pre class="json-viewer">
                                        {json_content}
                                    </pre>

                                    <div style="margin-top: 1rem; display: flex; justify-content: flex-end; gap: 0.75rem;">
                                        <button class="btn btn-inspect" on:click=move |_| set_modal_json.set(None)>
                                            "Cerrar Visor"
                                        </button>
                                        <button
                                            class="btn btn-approve"
                                            style="background-color: var(--ctp-mauve); color: var(--ctp-crust);"
                                            disabled=!session.get().current_user.role.can_execute_destructive_actions()
                                            on:click=move |_| request_execute_rollback(task_id_opt)
                                        >
                                            "🔄 Revertir Estado (Ejecutar Rollback JSON)"
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }.into_view()
                } else {
                    view! {}.into_view()
                }
            }}

            // Modal de Re-autenticación Estricta por Contraseña para Acciones Críticas
            {move || {
                if let Some((_, _, desc)) = pending_critical_action.get() {
                    view! {
                        <div class="modal-overlay" on:click=move |_| set_pending_critical_action.set(None)>
                            <div class="modal-content reauth-modal" on:click=|ev| ev.stop_propagation()>
                                <div class="modal-header reauth-header">
                                    <div style="display: flex; align-items: center; gap: 0.5rem;">
                                        <span style="font-size: 1.25rem;">"🔒"</span>
                                        <h3 style="color: var(--ctp-peach); font-weight: 700;">
                                            "Re-autenticación Obligatoria por Contraseña"
                                        </h3>
                                    </div>
                                    <button class="modal-close" on:click=move |_| set_pending_critical_action.set(None)>
                                        "✕"
                                    </button>
                                </div>

                                <div class="modal-body">
                                    <div class="reauth-warning-banner">
                                        <p style="font-size: 0.84rem; color: var(--ctp-subtext1); margin-bottom: 0.5rem;">
                                            <strong style="color: var(--ctp-text);">"Acción Crítica:" </strong>
                                            {desc}
                                        </p>
                                        <p style="font-size: 0.8rem; color: var(--ctp-subtext0);">
                                            "Por política de seguridad Zero-Trust, el despacho de comandos de mitigación masiva o reversión "
                                            "exige confirmar su contraseña antes de transmitir la directiva a los enrutadores y firewalls."
                                        </p>
                                    </div>

                                    {move || {
                                        if let Some(err) = reauth_error.get() {
                                            view! {
                                                <div class="alert-box alert-danger" style="margin-bottom: 1rem;">
                                                    {err}
                                                </div>
                                            }.into_view()
                                        } else {
                                            view! {}.into_view()
                                        }
                                    }}

                                    <div class="form-group" style="margin-top: 1rem;">
                                        <label class="form-label">
                                            {format!("Ingrese la contraseña de {}:", session.get().current_user.username)}
                                        </label>
                                        <input
                                            type="password"
                                            class="form-control"
                                            placeholder="Contraseña del usuario activo"
                                            prop:value=move || reauth_password.get()
                                            on:input=move |ev| set_reauth_password.set(event_target_value(&ev))
                                        />
                                    </div>

                                    <div style="display: flex; justify-content: flex-end; gap: 0.75rem; margin-top: 1.5rem;">
                                        <button
                                            class="btn btn-inspect"
                                            on:click=move |_| set_pending_critical_action.set(None)
                                        >
                                            "Cancelar"
                                        </button>
                                        <button
                                            class="btn btn-approve"
                                            on:click=confirm_reauth
                                        >
                                            "🔓 Confirmar y Despachar Comando"
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }.into_view()
                } else {
                    view! {}.into_view()
                }
            }}
        </div>
    }
}
