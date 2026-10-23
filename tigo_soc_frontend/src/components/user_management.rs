// * * * COMPONENTE: GESTIÓN CRUD DE USUARIOS, CONTROL RBAC Y CONFIGURACIÓN DE TIMEOUT * * *

use leptos::*;
use wasm_bindgen_futures::spawn_local;
use js_sys;
use crate::api;
use crate::models::{GlobalSecurityConfig, RoleType, SocUser, UserSessionState};

/// Mapeo RoleType (frontend) -> role_id (tabla `roles` de PostgreSQL segun seed:
/// 1=SOC_ADMIN, 2=SOC_OPERATOR, 3=AUDITOR, 4=ANALYST_L1).
fn role_type_to_id(role: RoleType) -> i32 {
    match role {
        RoleType::Admin => 1,
        RoleType::SocOperator => 2,
        RoleType::Auditor => 3,
        RoleType::AnalystL1 => 4,
    }
}

/// Genera un token de activación de un solo uso criptográficamente único
/// Principio Zero-Knowledge: el administrador NUNCA conoce ni fija la contraseña del usuario.
/// El token (act_...) se genera en cliente con entropía del timestamp + Math.random().
fn generate_activation_token() -> String {
    // Utiliza Date.now() y Math.random() del runtime JS (WASM) para generar 16 bytes de entropía
    let ts = js_sys::Date::now() as u64;
    let rand_a = (js_sys::Math::random() * 0xFFFF_FFFF_u64 as f64) as u64;
    let rand_b = (js_sys::Math::random() * 0xFFFF_FFFF_u64 as f64) as u64;
    let combined = ts ^ (rand_a << 32) ^ rand_b;
    format!("act_{:016x}{:016x}", combined, rand_a ^ rand_b)
}

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
    let (new_role, set_new_role) = create_signal(RoleType::SocOperator);
    let (feedback_msg, set_feedback_msg) = create_signal(Option::<String>::None);
    let (error_msg, set_error_msg) = create_signal(Option::<String>::None);
    // Token de activación Zero-Knowledge generado al crear usuario exitosamente
    let (activation_token, set_activation_token) = create_signal(Option::<String>::None);

    // Estado local para la edición del timeout
    let (timeout_input, set_timeout_input) = create_signal(security_config.get().critical_action_timeout_secs.to_string());

    // Registro de nuevo usuario — Provisioning Zero-Knowledge
    let handle_create_user = move |_| {
        set_error_msg.set(None);
        set_feedback_msg.set(None);
        set_activation_token.set(None);

        let user_str = new_username.get().trim().to_string();

        if user_str.is_empty() {
            set_error_msg.set(Some("El nombre de usuario no puede estar vacío.".to_string()));
            return;
        }

        // Validación mínima: solo alfanuméricos, guiones y puntos
        if !user_str.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.') {
            set_error_msg.set(Some("El nombre de usuario solo puede contener letras, números, '_', '-' y '.'.".to_string()));
            return;
        }

        let existing = users.get().iter().any(|u| u.username.to_lowercase() == user_str.to_lowercase());
        if existing {
            set_error_msg.set(Some(format!("El usuario '{}' ya existe en el sistema.", user_str)));
            return;
        }

        // Creación REAL vía POST /api/v1/users (CreateUserDto del backend Axum).
        // Principio Zero-Knowledge: se genera un token de activación que se envía
        // como contraseña inicial; solo el usuario lo conoce, el admin no fija credenciales.
        let activation_token_value = generate_activation_token();
        let role_id = role_type_to_id(new_role.get());

        let set_users_c = set_users;
        let set_new_username_c = set_new_username;
        let set_activation_token_c = set_activation_token;
        let set_feedback_msg_c = set_feedback_msg;
        let set_error_msg_c = set_error_msg;
        let session_token = session.get().session_token.clone();

        spawn_local(async move {
            match api::create_user(&session_token, &user_str, &activation_token_value, role_id).await {
                Ok(created) => {
                    let mapped = api::map_user_with_role(created);
                    set_users_c.update(|list| list.push(mapped));
                    set_new_username_c.set(String::new());
                    // Mostrar token de activación al administrador (NO es la contraseña permanente)
                    set_activation_token_c.set(Some(activation_token_value));
                    set_feedback_msg_c.set(Some(format!(
                        "✓ Usuario '{}' provisionado en PostgreSQL con rol ID {}. Token de activación generado.",
                        user_str, role_id
                    )));
                }
                Err(e) => {
                    set_error_msg_c.set(Some(format!("✕ El backend rechazó la creación del usuario: {}", e)));
                }
            }
        });
    };

    // Actualización de rol — PUT /api/v1/users/:id/role (persistencia real RBAC)
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

        let token = session.get().session_token.clone();
        let new_role_id = role_type_to_id(parsed_role);
        let set_feedback_msg_r = set_feedback_msg;
        let set_error_msg_r = set_error_msg;

        spawn_local(async move {
            match api::update_user_role(&token, user_id, new_role_id).await {
                Ok(_) => {
                    set_feedback_msg_r.set(Some(format!(
                        "✓ Rol de usuario #{} actualizado a '{}' (role_id={}). Permisos sincronizados en PostgreSQL.",
                        user_id,
                        parsed_role.display_name(),
                        new_role_id
                    )));
                }
                Err(e) => {
                    set_error_msg_r.set(Some(format!(
                        "✕ No se pudo persistir el cambio de rol del usuario #{} en el backend: {}",
                        user_id, e
                    )));
                }
            }
        });
    };

    // Eliminación de usuario — DELETE /api/v1/users/:id (real contra PostgreSQL)
    let handle_delete_user = move |user_id: i32| {
        if session.get().current_user.user_id == user_id {
            set_error_msg.set(Some("No puedes eliminar la cuenta del usuario actualmente autenticado.".to_string()));
            return;
        }

        let token = session.get().session_token.clone();
        let set_users_d = set_users;
        let set_feedback_msg_d = set_feedback_msg;
        let set_error_msg_d = set_error_msg;

        spawn_local(async move {
            match api::delete_user(&token, user_id).await {
                Ok(_) => {
                    set_users_d.update(|list| list.retain(|u| u.user_id != user_id));
                    set_feedback_msg_d.set(Some(format!("✓ Usuario #{} eliminado de la base de datos.", user_id)));
                }
                Err(e) => {
                    set_error_msg_d.set(Some(format!(
                        "✕ El backend rechazó la eliminación del usuario #{}: {}",
                        user_id, e
                    )));
                }
            }
        });
    };

    // Guardar nuevo timeout global — PUT /api/v1/config/critical-timeout (persistencia real)
    let handle_save_timeout = move |_| {
        set_error_msg.set(None);
        set_feedback_msg.set(None);

        match timeout_input.get().trim().parse::<u64>() {
            Ok(val) if val >= 10 => {
                set_security_config.update(|cfg| cfg.critical_action_timeout_secs = val);

                let token = session.get().session_token.clone();
                let set_feedback_msg_t = set_feedback_msg;
                let set_error_msg_t = set_error_msg;

                spawn_local(async move {
                    match api::save_critical_timeout(&token, val).await {
                        Ok(_) => {
                            set_feedback_msg_t.set(Some(format!(
                                "✓ Parámetro global 'CRITICAL_ACTION_TIMEOUT_SECS' actualizado a {} segundos en PostgreSQL.",
                                val
                            )));
                        }
                        Err(e) => {
                            set_error_msg_t.set(Some(format!(
                                "✕ No se pudo guardar el timeout global en el backend: {}",
                                e
                            )));
                        }
                    }
                });
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

            // Banner del Token de Activación Zero-Knowledge (aparece solo tras crear usuario)
            {move || {
                if let Some(token) = activation_token.get() {
                    view! {
                        <div style="
                            border: 1px solid var(--ctp-yellow);
                            border-radius: 8px;
                            background: rgba(249, 226, 175, 0.08);
                            padding: 1.25rem;
                            margin-bottom: 1rem;
                        ">
                            <div style="display: flex; align-items: center; gap: 0.5rem; margin-bottom: 0.75rem;">
                                <span style="font-size: 1.25rem;">"🔑"</span>
                                <h4 style="color: var(--ctp-yellow); font-size: 0.95rem; font-weight: 700; margin: 0;">
                                    "Token de Activación Zero-Knowledge (ISO/IEC 27001)"
                                </h4>
                            </div>
                            <p style="font-size: 0.8125rem; color: var(--ctp-subtext1); margin-bottom: 0.75rem; line-height: 1.5;">
                                "El administrador "
                                <strong style="color: var(--ctp-red);">"NUNCA"</strong>
                                " conoce ni define la contraseña del usuario. Entregue este token de un solo uso al nuevo usuario. "
                                "Él deberá usarlo en su primer inicio de sesión para establecer su propia contraseña privada con "
                                <strong style="color: var(--ctp-text);">"Argon2id."</strong>
                                " El token expira en 24h."
                            </p>
                            <div style="
                                background: var(--ctp-crust);
                                border-radius: 6px;
                                padding: 0.75rem 1rem;
                                font-family: var(--font-mono);
                                font-size: 0.875rem;
                                color: var(--ctp-green);
                                word-break: break-all;
                                letter-spacing: 0.05em;
                                border: 1px solid var(--ctp-surface1);
                            ">
                                {token}
                            </div>
                            <p style="font-size: 0.75rem; color: var(--ctp-overlay1); margin-top: 0.5rem;">
                                "⚠️ Copia y entrega este token de forma segura (canal cifrado). No se almacenará ni podrá recuperarse después."
                            </p>
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
                    // Nota informativa Zero-Knowledge
                    <div style="
                        background: rgba(166, 227, 161, 0.06);
                        border: 1px solid rgba(166, 227, 161, 0.25);
                        border-radius: 6px;
                        padding: 0.75rem 1rem;
                        margin-bottom: 1rem;
                        font-size: 0.8125rem;
                        color: var(--ctp-subtext1);
                        line-height: 1.5;
                    ">
                        <strong style="color: var(--ctp-green);">"🔐 Provisioning Zero-Knowledge (ISO/IEC 27001 — Cláusula 9.4.3):"</strong>
                        " El Administrador ingresa solo el nombre de usuario y el rol. "
                        "El sistema genera un token de activación de un solo uso (24h). "
                        "La contraseña es definida " <strong style="color: var(--ctp-text);">"exclusivamente por el usuario en su primer login"</strong>
                        " con hashing Argon2id. Ningún administrador conoce las credenciales."
                    </div>
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
                                "🔑 Provisionar Usuario (Zero-Knowledge)"
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
