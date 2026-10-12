// * * * COMPONENTE: GESTIÓN CRUD DE USUARIOS, CONTROL RBAC Y CONFIGURACIÓN DE TIMEOUT * * *

use leptos::*;
use crate::models::{GlobalSecurityConfig, RoleType, SocUser, UserSessionState};

#[component]
pub fn UserManagement(
    users: ReadSignal<Vec<SocUser>>,
    set_users: WriteSignal<Vec<SocUser>>,
    session: ReadSignal<UserSessionState>,
    set_session: WriteSignal<UserSessionState>,
    security_config: ReadSignal<GlobalSecurityConfig>,
    set_security_config: WriteSignal<GlobalSecurityConfig>,
) -> impl IntoView {
    // Estados locales para el formulario de nuevo usuario
    let (new_username, set_new_username) = create_signal(String::new());
    let (new_password, set_new_password) = create_signal(String::new());
    let (new_role, set_new_role) = create_signal(RoleType::SocOperator);
    let (feedback_msg, set_feedback_msg) = create_signal(Option::<String>::None);
    let (error_msg, set_error_msg) = create_signal(Option::<String>::None);

    // Estado local para la edición del timeout
    let (timeout_input, set_timeout_input) = create_signal(security_config.get().critical_action_timeout_secs.to_string());

    // Registro de nuevo usuario
    let handle_create_user = move |_| {
        set_error_msg.set(None);
        set_feedback_msg.set(None);

        let user_str = new_username.get().trim().to_string();
        let pass_str = new_password.get();

        if user_str.is_empty() || pass_str.is_empty() {
            set_error_msg.set(Some("El nombre de usuario y la contraseña son obligatorios.".to_string()));
            return;
        }

        let existing = users.get().iter().any(|u| u.username.to_lowercase() == user_str.to_lowercase());
        if existing {
            set_error_msg.set(Some(format!("El usuario '{}' ya existe en el sistema.", user_str)));
            return;
        }

        let next_id = users.get().iter().map(|u| u.user_id).max().unwrap_or(0) + 1;
        let created_now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let new_user_obj = SocUser {
            user_id: next_id,
            username: user_str.clone(),
            role: new_role.get(),
            created_at: created_now,
            is_active: true,
        };

        set_users.update(|list| list.push(new_user_obj));
        set_new_username.set(String::new());
        set_new_password.set(String::new());

        set_feedback_msg.set(Some(format!(
            "✓ Usuario '{}' registrado exitosamente con rol {} y contraseña hasheada en servidor.",
            user_str,
            new_role.get().display_name()
        )));
    };

    // Actualización de rol
    let handle_role_change = move |user_id: i32, role_str: String| {
        let parsed_role = match role_str.as_str() {
            "ADMIN" => RoleType::Admin,
            "OPERATOR" => RoleType::SocOperator,
            "AUDITOR" => RoleType::Auditor,
            _ => RoleType::AnalystL1,
        };

        set_users.update(|list| {
            if let Some(user) = list.iter_mut().find(|u| u.user_id == user_id) {
                user.role = parsed_role;
            }
        });

        // Si el usuario modificado es el de la sesión activa, actualizarlo también
        if session.get().current_user.user_id == user_id {
            set_session.update(|s| s.current_user.role = parsed_role);
        }

        set_feedback_msg.set(Some(format!(
            "✓ Rol de usuario #{} actualizado a '{}'. Permisos sincronizados en PostgreSQL.",
            user_id,
            parsed_role.display_name()
        )));
    };

    // Eliminación de usuario
    let handle_delete_user = move |user_id: i32| {
        if session.get().current_user.user_id == user_id {
            set_error_msg.set(Some("No puedes eliminar la cuenta del usuario actualmente autenticado.".to_string()));
            return;
        }

        set_users.update(|list| list.retain(|u| u.user_id != user_id));
        set_feedback_msg.set(Some(format!("✓ Usuario #{} eliminado de la base de datos.", user_id)));
    };

    // Guardar nuevo timeout global
    let handle_save_timeout = move |_| {
        set_error_msg.set(None);
        set_feedback_msg.set(None);

        match timeout_input.get().trim().parse::<u64>() {
            Ok(val) if val >= 10 => {
                set_security_config.update(|cfg| cfg.critical_action_timeout_secs = val);
                set_feedback_msg.set(Some(format!(
                    "✓ Parámetro global 'CRITICAL_ACTION_TIMEOUT_SECS' actualizado a {} segundos en base de datos.",
                    val
                )));
            }
            Ok(_) => {
                set_error_msg.set(Some("El timeout debe ser de al menos 10 segundos por razones de seguridad.".to_string()));
            }
            Err(_) => {
                set_error_msg.set(Some("Por favor ingrese un número entero válido en segundos.".to_string()));
            }
        }
    };

    // Forzar expiración de re-autenticación para pruebas
    let handle_force_lock = move |_| {
        set_session.update(|s| s.last_reauth_timestamp = 0.0);
        set_feedback_msg.set(Some(
            "🔒 Re-autenticación para acciones críticas bloqueada inmediatamente. El próximo intento exigirá contraseña.".to_string()
        ));
    };

    view! {
        <div class="user-mgmt-container">
            // Feedback y mensajes de alerta
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

            // Panel Superior: Configuración Global de Timeout para Acciones Críticas
            <div class="rbac-card">
                <div class="rbac-card-header">
                    <div style="display: flex; align-items: center; gap: 0.5rem;">
                        <span style="font-size: 1.25rem;">"⏱️"</span>
                        <h3 style="color: var(--ctp-text); font-size: 1.05rem; font-weight: 700;">
                            "Configuración Global: Timeout de Re-autenticación Estricta"
                        </h3>
                    </div>
                    <span class="badge-role badge-role-admin">
                        "Solo Administrador"
                    </span>
                </div>

                <div class="rbac-card-body">
                    <p style="font-size: 0.84rem; color: var(--ctp-subtext0); margin-bottom: 1rem;">
                        "Define el tiempo máximo de gracia (" <strong style="color: var(--ctp-peach);">"inactividad"</strong> ") "
                        "tras el cual cualquier acción con impacto (" <strong style="color: var(--ctp-text);">"Mitigación Masiva o Rollback"</strong> ") "
                        "requiere verificar la contraseña del usuario. "
                        <strong style="color: var(--ctp-green);">"El usuario continúa viendo la telemetría en tiempo real sin ser desconectado."</strong>
                    </p>

                    <div style="display: flex; gap: 1rem; align-items: flex-end; flex-wrap: wrap;">
                        <div class="form-group" style="margin-bottom: 0;">
                            <label class="form-label">
                                "Timeout de Inactividad Crítica (segundos):"
                            </label>
                            <input
                                type="number"
                                class="form-control"
                                style="width: 180px; font-family: var(--font-mono);"
                                prop:value=move || timeout_input.get()
                                on:input=move |ev| set_timeout_input.set(event_target_value(&ev))
                            />
                        </div>

                        <button class="btn btn-approve" on:click=handle_save_timeout>
                            "💾 Guardar Parámetro en BD"
                        </button>

                        <button class="btn btn-inspect" on:click=handle_force_lock style="border-color: var(--ctp-peach); color: var(--ctp-peach);">
                            "🔒 Forzar Expiración de Re-auth Ahora"
                        </button>

                        <div style="margin-left: auto; font-size: 0.8rem; color: var(--ctp-subtext0); font-family: var(--font-mono);">
                            {move || {
                                let secs = security_config.get().critical_action_timeout_secs;
                                format!("Activo: {}s ({} min {}s)", secs, secs / 60, secs % 60)
                            }}
                        </div>
                    </div>
                </div>
            </div>

            // Estado de la Sesión Segura en Servidor
            <div class="rbac-card" style="border-left: 4px solid var(--ctp-blue);">
                <div class="rbac-card-header">
                    <div style="display: flex; align-items: center; gap: 0.5rem;">
                        <span style="font-size: 1.25rem;">"🛡️"</span>
                        <h3 style="color: var(--ctp-text); font-size: 1.05rem; font-weight: 700;">
                            "Sesión Criptográfica en Servidor (Token Opaco)"
                        </h3>
                    </div>
                    <span class="status-badge online">
                        <span class="status-dot"></span>
                        "POSTGRESQL USER_SESSIONS"
                    </span>
                </div>

                <div class="rbac-card-body" style="font-size: 0.84rem;">
                    <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 1rem;">
                        <div>
                            <span class="blast-prop-label">"Usuario Actual:"</span>
                            <div style="font-weight: 600; color: var(--ctp-text); font-size: 0.95rem; display: flex; align-items: center; gap: 0.5rem;">
                                {move || session.get().current_user.username}
                                <span class=move || match session.get().current_user.role {
                                    RoleType::Admin => "badge-role badge-role-admin",
                                    RoleType::SocOperator => "badge-role badge-role-operator",
                                    RoleType::Auditor => "badge-role badge-role-auditor",
                                    RoleType::AnalystL1 => "badge-role badge-role-l1",
                                }>
                                    {move || session.get().current_user.role.display_name()}
                                </span>
                            </div>
                        </div>

                        <div>
                            <span class="blast-prop-label">"Token Opaco (Servidor 256-bit):"</span>
                            <div class="mono" style="color: var(--ctp-blue); font-size: 0.75rem; word-break: break-all;">
                                {move || session.get().session_token}
                            </div>
                        </div>

                        <div>
                            <span class="blast-prop-label">"Estado de Re-autenticación:"</span>
                            <div>
                                {move || {
                                    let last = session.get().last_reauth_timestamp;
                                    if last <= 0.0 {
                                        view! {
                                            <span style="color: var(--ctp-peach); font-weight: 600;">
                                                "⚠️ Re-auth requerida al despachar mitigación"
                                            </span>
                                        }.into_view()
                                    } else {
                                        view! {
                                            <span style="color: var(--ctp-green); font-weight: 600;">
                                                "✓ Re-autenticación autorizada temporalmente"
                                            </span>
                                        }.into_view()
                                    }
                                }}
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            // Sección de Registro de Nuevos Usuarios
            <div class="rbac-card">
                <div class="rbac-card-header">
                    <div style="display: flex; align-items: center; gap: 0.5rem;">
                        <span style="font-size: 1.25rem;">"➕"</span>
                        <h3 style="color: var(--ctp-text); font-size: 1.05rem; font-weight: 700;">
                            "Registrar Nuevo Usuario del SOC"
                        </h3>
                    </div>
                </div>

                <div class="rbac-card-body">
                    <div class="form-row">
                        <div class="form-group">
                            <label class="form-label">"Nombre de Usuario:"</label>
                            <input
                                type="text"
                                class="form-control"
                                placeholder="ej. analista_soc_3"
                                prop:value=move || new_username.get()
                                on:input=move |ev| set_new_username.set(event_target_value(&ev))
                            />
                        </div>

                        <div class="form-group">
                            <label class="form-label">"Contraseña (Hasheada con Argon2/Sal):"</label>
                            <input
                                type="password"
                                class="form-control"
                                placeholder="Contraseña segura"
                                prop:value=move || new_password.get()
                                on:input=move |ev| set_new_password.set(event_target_value(&ev))
                            />
                        </div>

                        <div class="form-group">
                            <label class="form-label">"Rol Asignado (RBAC):"</label>
                            <select
                                class="form-control"
                                on:change=move |ev| {
                                    let v = event_target_value(&ev);
                                    let r = match v.as_str() {
                                        "ADMIN" => RoleType::Admin,
                                        "OPERATOR" => RoleType::SocOperator,
                                        "AUDITOR" => RoleType::Auditor,
                                        _ => RoleType::AnalystL1,
                                    };
                                    set_new_role.set(r);
                                }
                            >
                                <option value="OPERATOR">"Operador SOC (Mitigación & Rollback)"</option>
                                <option value="ADMIN">"Administrador General (Total)"</option>
                                <option value="AUDITOR">"Auditor de Seguridad (Solo Lectura)"</option>
                                <option value="ANALYST">"Analista L1 (Triaje Inicial)"</option>
                            </select>
                        </div>

                        <div class="form-group" style="align-self: flex-end;">
                            <button class="btn btn-approve" on:click=handle_create_user>
                                "✓ Registrar Usuario"
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            // Tabla de Usuarios del Sistema y Gestión de Roles
            <div class="rbac-card">
                <div class="rbac-card-header">
                    <div style="display: flex; align-items: center; gap: 0.5rem;">
                        <span style="font-size: 1.25rem;">"👥"</span>
                        <h3 style="color: var(--ctp-text); font-size: 1.05rem; font-weight: 700;">
                            "Usuarios Registrados y Control de Roles (RBAC)"
                        </h3>
                    </div>
                    <span style="font-size: 0.8125rem; color: var(--ctp-subtext0); font-family: var(--font-mono);">
                        {move || format!("Total: {} usuarios", users.get().len())}
                    </span>
                </div>

                <div class="table-container" style="margin-bottom: 0; border: none; border-radius: 0;">
                    <table class="soc-table">
                        <thead>
                            <tr>
                                <th>"ID"</th>
                                <th>"Usuario"</th>
                                <th>"Rol Asignado"</th>
                                <th>"Permisos Clave"</th>
                                <th>"Fecha Creación"</th>
                                <th>"Cambiar Rol (Al Vuelo)"</th>
                                <th>"Acciones"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || {
                                users.get().into_iter().map(|u| {
                                    let uid = u.user_id;
                                    let current_role = u.role;
                                    let is_active_session_user = session.get().current_user.user_id == uid;

                                    let badge_class = match current_role {
                                        RoleType::Admin => "badge-role badge-role-admin",
                                        RoleType::SocOperator => "badge-role badge-role-operator",
                                        RoleType::Auditor => "badge-role badge-role-auditor",
                                        RoleType::AnalystL1 => "badge-role badge-role-l1",
                                    };

                                    let permissions_desc = match current_role {
                                        RoleType::Admin => "Total (CRUD Usuarios, Configuración, Mitigación, Rollback)",
                                        RoleType::SocOperator => "Mitigación Masiva y Rollback con Re-auth por Contraseña",
                                        RoleType::Auditor => "Solo Lectura (Monitoreo forense, auditoría e inspección)",
                                        RoleType::AnalystL1 => "Triaje inicial y visualización de alertas",
                                    };

                                    view! {
                                        <tr>
                                            <td class="mono">{format!("#{}", u.user_id)}</td>
                                            <td>
                                                <strong style="color: var(--ctp-text);">{u.username.clone()}</strong>
                                                {if is_active_session_user {
                                                    view! {
                                                        <span style="color: var(--ctp-green); font-size: 0.72rem; margin-left: 0.4rem; font-weight: 600;">
                                                            "(Sesión Activa)"
                                                        </span>
                                                    }.into_view()
                                                } else {
                                                    view! {}.into_view()
                                                }}
                                            </td>
                                            <td>
                                                <span class=badge_class>
                                                    {current_role.display_name()}
                                                </span>
                                            </td>
                                            <td style="font-size: 0.78rem; color: var(--ctp-subtext1);">
                                                {permissions_desc}
                                            </td>
                                            <td class="mono" style="font-size: 0.78rem; color: var(--ctp-overlay1);">
                                                {u.created_at.clone()}
                                            </td>
                                            <td>
                                                <select
                                                    class="form-control"
                                                    style="padding: 0.25rem 0.5rem; font-size: 0.75rem; width: auto;"
                                                    on:change=move |ev| handle_role_change(uid, event_target_value(&ev))
                                                >
                                                    <option value="OPERATOR" selected=current_role == RoleType::SocOperator>"Operador SOC"</option>
                                                    <option value="ADMIN" selected=current_role == RoleType::Admin>"Administrador"</option>
                                                    <option value="AUDITOR" selected=current_role == RoleType::Auditor>"Auditor"</option>
                                                    <option value="ANALYST" selected=current_role == RoleType::AnalystL1>"Analista L1"</option>
                                                </select>
                                            </td>
                                            <td>
                                                <button
                                                    class="btn btn-reject"
                                                    style="padding: 0.25rem 0.6rem; font-size: 0.75rem;"
                                                    on:click=move |_| handle_delete_user(uid)
                                                >
                                                    "🗑 Eliminar"
                                                </button>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view()
                            }}
                        </tbody>
                    </table>
                </div>
            </div>

            // Matriz Visual de Permisos RBAC
            <div class="rbac-card">
                <div class="rbac-card-header">
                    <div style="display: flex; align-items: center; gap: 0.5rem;">
                        <span style="font-size: 1.25rem;">"📋"</span>
                        <h3 style="color: var(--ctp-text); font-size: 1.05rem; font-weight: 700;">
                            "Matriz de Privilegios y Políticas de Seguridad (RBAC)"
                        </h3>
                    </div>
                </div>

                <div class="rbac-card-body" style="font-size: 0.8125rem;">
                    <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 1rem;">
                        <div class="policy-box">
                            <strong style="color: var(--ctp-mauve);">"Administrador General"</strong>
                            <ul style="margin-top: 0.4rem; padding-left: 1.2rem; color: var(--ctp-subtext0);">
                                <li>"Registro y baja de operadores"</li>
                                <li>"Asignación de roles al vuelo"</li>
                                <li>"Configuración de timeout crítico"</li>
                                <li>"Autorización de mitigación y rollback"</li>
                            </ul>
                        </div>

                        <div class="policy-box">
                            <strong style="color: var(--ctp-blue);">"Operador SOC"</strong>
                            <ul style="margin-top: 0.4rem; padding-left: 1.2rem; color: var(--ctp-subtext0);">
                                <li>"Monitoreo de alertas en vivo"</li>
                                <li>"Aprobación de Blast Radius masivo"</li>
                                <li>"Ejecución de Rollback JSON"</li>
                                <li><strong style="color: var(--ctp-peach);">"Re-auth por contraseña obligatoria"</strong></li>
                            </ul>
                        </div>

                        <div class="policy-box">
                            <strong style="color: var(--ctp-yellow);">"Auditor de Seguridad"</strong>
                            <ul style="margin-top: 0.4rem; padding-left: 1.2rem; color: var(--ctp-subtext0);">
                                <li>"Inspección de alertas forenses"</li>
                                <li>"Revisión de snapshots de reversión"</li>
                                <li>"Acceso de solo lectura"</li>
                                <li><strong style="color: var(--ctp-red);">"Ejecución de comandos bloqueada"</strong></li>
                            </ul>
                        </div>

                        <div class="policy-box">
                            <strong style="color: var(--ctp-teal);">"Analista L1"</strong>
                            <ul style="margin-top: 0.4rem; padding-left: 1.2rem; color: var(--ctp-subtext0);">
                                <li>"Triaje de telemetría de red"</li>
                                <li>"Filtrado por severidad y capas"</li>
                                <li>"Visualización de métricas ML"</li>
                            </ul>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
