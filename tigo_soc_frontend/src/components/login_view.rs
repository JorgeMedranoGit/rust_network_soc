// * * * COMPONENTE: VISTA DE AUTENTICACIÓN Y CREACIÓN DE ADMINISTRADOR INICIAL * * *

use leptos::*;
use crate::models::{RoleType, SocUser, UserSessionState};

// ---------------------------------------------------------------------------
// Helper: devuelve el timestamp UNIX actual como f64 (milisegundos / 1000)
// ---------------------------------------------------------------------------
fn now_timestamp() -> f64 {
    js_sys_now()
}

#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    // Importa Date.now() del JS runtime
    #[wasm_bindgen::prelude::wasm_bindgen(js_namespace = Date, js_name = now)]
    fn js_sys_now() -> f64;
}

// ---------------------------------------------------------------------------
// Sub-componente interno: fondo animado de partículas SVG
// ---------------------------------------------------------------------------
#[component]
fn ParticleBackground() -> impl IntoView {
    view! {
        <div style="
            position: fixed;
            inset: 0;
            z-index: 0;
            overflow: hidden;
            pointer-events: none;
        ">
            // Gradiente radial base
            <div style="
                position: absolute;
                inset: 0;
                background:
                    radial-gradient(ellipse 80% 60% at 20% 30%, rgba(137,180,250,0.08) 0%, transparent 60%),
                    radial-gradient(ellipse 60% 80% at 80% 70%, rgba(203,166,247,0.07) 0%, transparent 60%),
                    radial-gradient(ellipse 50% 50% at 50% 50%, rgba(148,226,213,0.04) 0%, transparent 70%);
            "></div>

            // Partículas decorativas SVG
            <svg
                style="position:absolute;inset:0;width:100%;height:100%;"
                xmlns="http://www.w3.org/2000/svg"
                aria-hidden="true"
            >
                // Líneas de cuadrícula tenue
                <defs>
                    <pattern id="login-grid" width="60" height="60" patternUnits="userSpaceOnUse">
                        <path
                            d="M 60 0 L 0 0 0 60"
                            fill="none"
                            stroke="rgba(88,91,112,0.18)"
                            stroke-width="0.5"
                        />
                    </pattern>
                    <radialGradient id="particle-glow-blue" cx="50%" cy="50%" r="50%">
                        <stop offset="0%" stop-color="#89b4fa" stop-opacity="0.6"/>
                        <stop offset="100%" stop-color="#89b4fa" stop-opacity="0"/>
                    </radialGradient>
                    <radialGradient id="particle-glow-mauve" cx="50%" cy="50%" r="50%">
                        <stop offset="0%" stop-color="#cba6f7" stop-opacity="0.5"/>
                        <stop offset="100%" stop-color="#cba6f7" stop-opacity="0"/>
                    </radialGradient>
                    <radialGradient id="particle-glow-teal" cx="50%" cy="50%" r="50%">
                        <stop offset="0%" stop-color="#94e2d5" stop-opacity="0.5"/>
                        <stop offset="100%" stop-color="#94e2d5" stop-opacity="0"/>
                    </radialGradient>
                </defs>

                <rect width="100%" height="100%" fill="url(#login-grid)"/>

                // Orbes flotantes animados
                <circle cx="15%" cy="25%" r="140" fill="url(#particle-glow-blue)" opacity="0.6">
                    <animateTransform
                        attributeName="transform" type="translate"
                        values="0,0; 20,30; -10,15; 0,0"
                        dur="18s" repeatCount="indefinite"
                    />
                </circle>
                <circle cx="80%" cy="65%" r="120" fill="url(#particle-glow-mauve)" opacity="0.55">
                    <animateTransform
                        attributeName="transform" type="translate"
                        values="0,0; -25,-20; 10,-35; 0,0"
                        dur="22s" repeatCount="indefinite"
                    />
                </circle>
                <circle cx="55%" cy="10%" r="90" fill="url(#particle-glow-teal)" opacity="0.45">
                    <animateTransform
                        attributeName="transform" type="translate"
                        values="0,0; 15,20; -20,10; 0,0"
                        dur="15s" repeatCount="indefinite"
                    />
                </circle>

                // Pequeños puntos flotantes
                <circle cx="10%" cy="80%" r="3" fill="#89b4fa" opacity="0.35">
                    <animate attributeName="cy" values="80%;75%;80%" dur="6s" repeatCount="indefinite"/>
                    <animate attributeName="opacity" values="0.35;0.7;0.35" dur="6s" repeatCount="indefinite"/>
                </circle>
                <circle cx="90%" cy="20%" r="2.5" fill="#cba6f7" opacity="0.4">
                    <animate attributeName="cy" values="20%;25%;20%" dur="7s" repeatCount="indefinite"/>
                    <animate attributeName="opacity" values="0.4;0.8;0.4" dur="7s" repeatCount="indefinite"/>
                </circle>
                <circle cx="45%" cy="90%" r="2" fill="#94e2d5" opacity="0.3">
                    <animate attributeName="cy" values="90%;85%;90%" dur="9s" repeatCount="indefinite"/>
                    <animate attributeName="opacity" values="0.3;0.6;0.3" dur="9s" repeatCount="indefinite"/>
                </circle>
                <circle cx="70%" cy="40%" r="2" fill="#a6e3a1" opacity="0.3">
                    <animate attributeName="cy" values="40%;45%;40%" dur="8s" repeatCount="indefinite"/>
                </circle>
                <circle cx="30%" cy="55%" r="1.5" fill="#f5c2e7" opacity="0.25">
                    <animate attributeName="cy" values="55%;50%;55%" dur="10s" repeatCount="indefinite"/>
                </circle>
            </svg>

            // Línea de escaneo animada
            <div style="
                position: absolute;
                left: 0; right: 0;
                height: 1px;
                background: linear-gradient(90deg, transparent, rgba(137,180,250,0.4), transparent);
                animation: scan-line 8s linear infinite;
                top: 0;
            "></div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Sub-componente: SVG Shield Logo del SOC
// ---------------------------------------------------------------------------
#[component]
fn SocShieldLogo() -> impl IntoView {
    view! {
        <svg
            width="72"
            height="80"
            viewBox="0 0 72 80"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
            style="filter: drop-shadow(0 0 18px rgba(137,180,250,0.5));"
            aria-label="Escudo SOC"
        >
            <defs>
                <linearGradient id="shield-grad" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stop-color="#89b4fa"/>
                    <stop offset="50%" stop-color="#cba6f7"/>
                    <stop offset="100%" stop-color="#94e2d5"/>
                </linearGradient>
                <linearGradient id="shield-inner" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stop-color="#313244" stop-opacity="0.9"/>
                    <stop offset="100%" stop-color="#1e1e2e" stop-opacity="0.95"/>
                </linearGradient>
            </defs>
            // Escudo exterior
            <path
                d="M36 2 L68 14 L68 38 C68 57 52 72 36 78 C20 72 4 57 4 38 L4 14 Z"
                fill="url(#shield-grad)"
                opacity="0.9"
            />
            // Relleno interior del escudo
            <path
                d="M36 8 L62 18 L62 38 C62 54 48 67 36 72 C24 67 10 54 10 38 L10 18 Z"
                fill="url(#shield-inner)"
            />
            // Símbolo de ojo/radar en el centro
            <circle cx="36" cy="38" r="14" stroke="#89b4fa" stroke-width="1.5" fill="none" opacity="0.6"/>
            <circle cx="36" cy="38" r="9" stroke="#cba6f7" stroke-width="1.2" fill="none" opacity="0.7"/>
            <circle cx="36" cy="38" r="4" fill="#89b4fa" opacity="0.9"/>
            // Cruz de mira
            <line x1="36" y1="22" x2="36" y2="26" stroke="#89b4fa" stroke-width="1.5" opacity="0.7"/>
            <line x1="36" y1="50" x2="36" y2="54" stroke="#89b4fa" stroke-width="1.5" opacity="0.7"/>
            <line x1="20" y1="38" x2="24" y2="38" stroke="#89b4fa" stroke-width="1.5" opacity="0.7"/>
            <line x1="48" y1="38" x2="52" y2="38" stroke="#89b4fa" stroke-width="1.5" opacity="0.7"/>
        </svg>
    }
}

// ---------------------------------------------------------------------------
// Componente principal: LoginView
// ---------------------------------------------------------------------------
#[component]
pub fn LoginView(
    users: ReadSignal<Vec<SocUser>>,
    set_users: WriteSignal<Vec<SocUser>>,
    session: ReadSignal<UserSessionState>,
    set_session: WriteSignal<UserSessionState>,
    on_login_success: Callback<()>,
) -> impl IntoView {

    // ── Estados del formulario de login ──────────────────────────────────────
    let (login_username, set_login_username) = create_signal(String::new());
    let (login_password, set_login_password) = create_signal(String::new());
    let (login_error, set_login_error) = create_signal(Option::<String>::None);

    // ── Panel de creación de primer administrador ─────────────────────────────
    let (show_admin_create, set_show_admin_create) = create_signal(false);
    let (admin_username, set_admin_username) = create_signal(String::new());
    let (admin_password, set_admin_password) = create_signal(String::new());
    let (admin_confirm_pw, set_admin_confirm_pw) = create_signal(String::new());
    let (admin_create_error, set_admin_create_error) = create_signal(Option::<String>::None);
    let (admin_create_ok, set_admin_create_ok) = create_signal(Option::<String>::None);

    // ── Modal de re-autenticación ─────────────────────────────────────────────
    let (show_reauth_modal, set_show_reauth_modal) = create_signal(false);
    let (reauth_password, set_reauth_password) = create_signal(String::new());
    let (reauth_error, set_reauth_error) = create_signal(Option::<String>::None);
    let (reauth_ok, set_reauth_ok) = create_signal(false);

    // ── Computed: si hay usuarios en el sistema ───────────────────────────────
    let _has_users = move || !users.get().is_empty();

    // ── Computed: usuario actualmente en sesión (para mostrar post-login) ─────
    // (La vista de login se muestra solo cuando no hay sesión iniciada,
    //  pero necesitamos el modal de re-auth que puede aparecer sobre el dashboard)

    // ── Lógica de LOGIN ───────────────────────────────────────────────────────
    let handle_login = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();
        set_login_error.set(None);

        let uname = login_username.get().trim().to_string();
        let pass  = login_password.get();

        if uname.is_empty() || pass.is_empty() {
            set_login_error.set(Some(
                "Por favor ingrese su nombre de usuario y contraseña.".to_string()
            ));
            return;
        }

        let found = users.get()
            .into_iter()
            .find(|u| u.username.to_lowercase() == uname.to_lowercase() && u.is_active);

        match found {
            Some(user) => {
                // Validación de contraseña (simulada: admin="admin", otros cualquier no-vacía)
                let pass_ok = match user.role {
                    RoleType::Admin => pass == "admin" || !pass.is_empty(),
                    _               => !pass.is_empty(),
                };

                if pass_ok {
                    let ts = now_timestamp() / 1000.0; // segundos
                    set_session.update(|s| {
                        s.current_user = user.clone();
                        s.session_token = format!(
                            "tok_{}_{}",
                            user.user_id,
                            (ts as u64) & 0xFFFFFFFF
                        );
                        s.expires_at = "24h (Sesión en Servidor)".to_string();
                        s.last_reauth_timestamp = ts;
                    });
                    set_login_username.set(String::new());
                    set_login_password.set(String::new());
                    on_login_success.call(());
                } else {
                    set_login_error.set(Some(
                        "Usuario no encontrado o contraseña incorrecta.".to_string()
                    ));
                }
            }
            None => {
                set_login_error.set(Some(
                    "Usuario no encontrado o contraseña incorrecta.".to_string()
                ));
            }
        }
    };

    // ── Lógica de CREACIÓN DE ADMINISTRADOR INICIAL ───────────────────────────
    let handle_create_admin = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();
        set_admin_create_error.set(None);
        set_admin_create_ok.set(None);

        let uname   = admin_username.get().trim().to_string();
        let pass    = admin_password.get();
        let confirm = admin_confirm_pw.get();

        if uname.is_empty() {
            set_admin_create_error.set(Some("El nombre de usuario no puede estar vacío.".to_string()));
            return;
        }
        if pass.len() < 6 {
            set_admin_create_error.set(Some("La contraseña debe tener al menos 6 caracteres.".to_string()));
            return;
        }
        if pass != confirm {
            set_admin_create_error.set(Some("Las contraseñas no coinciden.".to_string()));
            return;
        }

        // Verificar duplicado
        if users.get().iter().any(|u| u.username.to_lowercase() == uname.to_lowercase()) {
            set_admin_create_error.set(Some(format!("El usuario '{}' ya existe.", uname)));
            return;
        }

        let next_id = users.get().iter().map(|u| u.user_id).max().unwrap_or(0) + 1;
        let created_now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let admin_user = SocUser {
            user_id: next_id,
            username: uname.clone(),
            role: RoleType::Admin,
            created_at: created_now,
            is_active: true,
        };

        set_users.update(|list| list.push(admin_user));
        set_admin_username.set(String::new());
        set_admin_password.set(String::new());
        set_admin_confirm_pw.set(String::new());
        set_admin_create_ok.set(Some(format!(
            "✓ Administrador '{}' creado exitosamente. Ahora puede iniciar sesión.",
            uname
        )));
        set_show_admin_create.set(false);
    };

    // ── Lógica de RE-AUTENTICACIÓN ────────────────────────────────────────────
    let handle_reauth = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();
        set_reauth_error.set(None);

        let pass  = reauth_password.get();
        let uname = session.get().current_user.username.to_lowercase();

        // Buscar el usuario en la lista y verificar contraseña
        let found = users.get()
            .into_iter()
            .find(|u| u.username.to_lowercase() == uname);

        let ok = match found {
            Some(user) => match user.role {
                RoleType::Admin => pass == "admin" || !pass.is_empty(),
                _               => !pass.is_empty(),
            },
            None => false,
        };

        if ok {
            let ts = now_timestamp() / 1000.0;
            set_session.update(|s| s.last_reauth_timestamp = ts);
            set_reauth_ok.set(true);
            set_reauth_password.set(String::new());
            // Ocultar modal después de 1.5 s usando Effect
            let set_modal = set_show_reauth_modal;
            set_modal.set(false);
        } else {
            set_reauth_error.set(Some("Contraseña incorrecta. Re-autenticación fallida.".to_string()));
        }
    };

    view! {
        // ── Estilos de animación específicos del login ────────────────────────
        <style>
            "
            @keyframes scan-line {
                0%   { top: -2px; }
                100% { top: 100%; }
            }
            @keyframes login-card-in {
                from { opacity: 0; transform: translateY(24px) scale(0.97); }
                to   { opacity: 1; transform: translateY(0)      scale(1);    }
            }
            @keyframes badge-pulse {
                0%, 100% { box-shadow: 0 0 0 0 rgba(137,180,250,0.4); }
                50%       { box-shadow: 0 0 0 6px rgba(137,180,250,0); }
            }
            @keyframes shield-float {
                0%, 100% { transform: translateY(0px); }
                50%       { transform: translateY(-6px); }
            }
            @keyframes spinner {
                from { transform: rotate(0deg); }
                to   { transform: rotate(360deg); }
            }
            .login-card-anim {
                animation: login-card-in 0.5s cubic-bezier(0.34, 1.56, 0.64, 1) both;
            }
            .shield-float {
                animation: shield-float 3.5s ease-in-out infinite;
            }
            .login-input {
                background: rgba(30,30,46,0.7);
                border: 1px solid rgba(88,91,112,0.6);
                color: #cdd6f4;
                padding: 0.7rem 1rem;
                border-radius: 8px;
                font-size: 0.92rem;
                outline: none;
                width: 100%;
                transition: border-color 0.2s, box-shadow 0.2s;
                font-family: var(--font-sans);
                backdrop-filter: blur(4px);
            }
            .login-input:focus {
                border-color: #89b4fa;
                box-shadow: 0 0 0 3px rgba(137,180,250,0.15);
            }
            .login-input::placeholder { color: #6c7086; }
            .login-btn-primary {
                width: 100%;
                padding: 0.8rem 1.5rem;
                background: linear-gradient(135deg, #89b4fa, #cba6f7);
                color: #1e1e2e;
                border: none;
                border-radius: 8px;
                font-size: 0.95rem;
                font-weight: 700;
                cursor: pointer;
                transition: all 0.2s;
                letter-spacing: 0.02em;
            }
            .login-btn-primary:hover {
                filter: brightness(1.12);
                transform: translateY(-1px);
                box-shadow: 0 6px 20px rgba(137,180,250,0.3);
            }
            .login-btn-primary:active { transform: translateY(0); }
            .login-btn-secondary {
                background: rgba(49,50,68,0.7);
                border: 1px solid rgba(88,91,112,0.6);
                color: #bac2de;
                padding: 0.65rem 1.2rem;
                border-radius: 8px;
                font-size: 0.85rem;
                font-weight: 600;
                cursor: pointer;
                transition: all 0.2s;
            }
            .login-btn-secondary:hover {
                background: rgba(69,71,90,0.8);
                border-color: #89b4fa;
                color: #cdd6f4;
            }
            .login-glass-card {
                background: rgba(24,24,37,0.75);
                border: 1px solid rgba(88,91,112,0.5);
                backdrop-filter: blur(20px);
                -webkit-backdrop-filter: blur(20px);
                border-radius: 16px;
                box-shadow:
                    0 8px 32px rgba(0,0,0,0.4),
                    0 0 0 1px rgba(137,180,250,0.05) inset;
            }
            .admin-panel {
                background: rgba(17,17,27,0.6);
                border: 1px dashed rgba(203,166,247,0.4);
                border-radius: 12px;
                padding: 1.25rem;
                margin-top: 1rem;
                animation: login-card-in 0.35s ease both;
            }
            .role-badge-display {
                display: inline-flex;
                align-items: center;
                gap: 0.4rem;
                padding: 0.3rem 0.8rem;
                border-radius: 20px;
                font-size: 0.75rem;
                font-weight: 700;
                letter-spacing: 0.04em;
                text-transform: uppercase;
            }
            "
        </style>

        // ── Contenedor raíz fullscreen ────────────────────────────────────────
        <div style="
            position: relative;
            min-height: 100vh;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            background-color: #1e1e2e;
            overflow: hidden;
        ">
            <ParticleBackground/>

            // ── Contenido sobre el fondo ──────────────────────────────────────
            <div style="
                position: relative;
                z-index: 10;
                width: 100%;
                max-width: 460px;
                padding: 1.5rem;
                display: flex;
                flex-direction: column;
                gap: 1rem;
            ">

                // ── Cabecera con logo ─────────────────────────────────────────
                <div style="text-align: center; margin-bottom: 0.5rem;">
                    <div class="shield-float" style="display:inline-block; margin-bottom: 1rem;">
                        <SocShieldLogo/>
                    </div>
                    <h1 style="
                        font-size: 1.75rem;
                        font-weight: 800;
                        color: #cdd6f4;
                        letter-spacing: -0.03em;
                        line-height: 1.2;
                        margin-bottom: 0.35rem;
                    ">
                        "Tigo SOC"
                        <span style="color: #89b4fa;"> " Platform"</span>
                    </h1>
                    <p style="
                        font-size: 0.82rem;
                        color: #6c7086;
                        letter-spacing: 0.06em;
                        text-transform: uppercase;
                        font-weight: 600;
                    ">
                        "Centro de Operaciones de Seguridad · Acceso Controlado"
                    </p>
                </div>

                // ── Banner de estado del sistema ──────────────────────────────
                <div style="
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    gap: 0.6rem;
                    padding: 0.5rem 1rem;
                    border-radius: 20px;
                    background: rgba(166,227,161,0.1);
                    border: 1px solid rgba(166,227,161,0.25);
                    width: fit-content;
                    margin: 0 auto;
                ">
                    <span class="status-dot" style="background: #a6e3a1; color: #a6e3a1;"></span>
                    <span style="font-size: 0.78rem; color: #a6e3a1; font-weight: 600; font-family: var(--font-mono);">
                        "SISTEMA OPERATIVO · LightGBM ACTIVO"
                    </span>
                </div>

                // ── Tarjeta principal de Login ────────────────────────────────
                <div class="login-glass-card login-card-anim" style="padding: 2rem;">

                    // Título del formulario + contador de usuarios
                    <div style="margin-bottom: 1.5rem;">
                        <h2 style="
                            font-size: 1.1rem;
                            font-weight: 700;
                            color: #cdd6f4;
                            margin-bottom: 0.3rem;
                        ">
                            "Autenticación de Operador"
                        </h2>
                        <p style="font-size: 0.8rem; color: #6c7086;">
                            {move || {
                                let count = users.get().len();
                                if count == 0 {
                                    "⚠ Sin usuarios registrados — cree el administrador inicial primero.".to_string()
                                } else {
                                    format!("{} operador{} registrado{} en el sistema.",
                                        count,
                                        if count == 1 { "" } else { "es" },
                                        if count == 1 { "" } else { "s" }
                                    )
                                }
                            }}
                        </p>
                    </div>

                    // ── Mensajes de feedback del login ────────────────────────
                    {move || {
                        if let Some(err) = login_error.get() {
                            view! {
                                <div class="alert-box alert-danger" style="margin-bottom: 1rem; font-size: 0.83rem;">
                                    <strong>"⚠ Error: "</strong>{err}
                                </div>
                            }.into_view()
                        } else {
                            view! {}.into_view()
                        }
                    }}

                    // ── Formulario de login ───────────────────────────────────
                    <form style="display: flex; flex-direction: column; gap: 1rem;">
                        // Usuario
                        <div>
                            <label style="
                                display: block;
                                font-size: 0.78rem;
                                color: #a6adc8;
                                font-weight: 600;
                                margin-bottom: 0.4rem;
                                letter-spacing: 0.02em;
                            " for="login-username">
                                "👤 Nombre de Usuario"
                            </label>
                            <input
                                id="login-username"
                                type="text"
                                class="login-input"
                                placeholder="Ingrese su nombre de usuario"
                                autocomplete="username"
                                prop:value=move || login_username.get()
                                on:input=move |ev| set_login_username.set(event_target_value(&ev))
                                on:keydown=move |ev| {
                                    if ev.key() == "Enter" {
                                        let _ = ev.prevent_default();
                                    }
                                }
                            />
                        </div>

                        // Contraseña
                        <div>
                            <label style="
                                display: block;
                                font-size: 0.78rem;
                                color: #a6adc8;
                                font-weight: 600;
                                margin-bottom: 0.4rem;
                                letter-spacing: 0.02em;
                            " for="login-password">
                                "🔑 Contraseña"
                            </label>
                            <input
                                id="login-password"
                                type="password"
                                class="login-input"
                                placeholder="Ingrese su contraseña"
                                autocomplete="current-password"
                                prop:value=move || login_password.get()
                                on:input=move |ev| set_login_password.set(event_target_value(&ev))
                                on:keydown=move |ev| {
                                    if ev.key() == "Enter" {
                                        let _ = ev.prevent_default();
                                    }
                                }
                            />
                        </div>

                        // Botón de ingreso
                        <button
                            type="submit"
                            class="login-btn-primary"
                            id="login-submit-btn"
                            on:click=handle_login
                        >
                            "🔐 Iniciar Sesión"
                        </button>
                    </form>

                    // Separador
                    <div style="
                        display: flex;
                        align-items: center;
                        gap: 0.75rem;
                        margin: 1.25rem 0;
                    ">
                        <div style="flex: 1; height: 1px; background: rgba(88,91,112,0.5);"></div>
                        <span style="font-size: 0.72rem; color: #585b70; font-weight: 600; white-space: nowrap;">
                            "ADMINISTRACIÓN"
                        </span>
                        <div style="flex: 1; height: 1px; background: rgba(88,91,112,0.5);"></div>
                    </div>

                    // ── Botón para mostrar panel de admin inicial ─────────────
                    <div style="display: flex; gap: 0.75rem; flex-wrap: wrap;">
                        <button
                            type="button"
                            class="login-btn-secondary"
                            id="toggle-admin-create-btn"
                            style="flex: 1;"
                            on:click=move |_| set_show_admin_create.update(|v| *v = !*v)
                        >
                            {move || if show_admin_create.get() {
                                "✕ Cancelar Creación"
                            } else {
                                "⚙ Crear Administrador Inicial"
                            }}
                        </button>

                        // Botón de re-autenticación (solo si hay sesión activa)
                        {move || {
                            let u = session.get().current_user.clone();
                            if !u.username.is_empty() {
                                view! {
                                    <button
                                        type="button"
                                        class="login-btn-secondary"
                                        id="trigger-reauth-btn"
                                        style="flex: 1; border-color: rgba(250,179,135,0.5); color: #fab387;"
                                        on:click=move |_| {
                                            set_show_reauth_modal.set(true);
                                            set_reauth_ok.set(false);
                                        }
                                    >
                                        "🔒 Re-autenticar"
                                    </button>
                                }.into_view()
                            } else {
                                view! {}.into_view()
                            }
                        }}
                    </div>

                    // ── Panel de creación de administrador inicial ─────────────
                    {move || {
                        if show_admin_create.get() {
                            view! {
                                <div class="admin-panel">
                                    <div style="
                                        display: flex;
                                        align-items: center;
                                        gap: 0.5rem;
                                        margin-bottom: 1rem;
                                        padding-bottom: 0.75rem;
                                        border-bottom: 1px solid rgba(203,166,247,0.2);
                                    ">
                                        <span style="font-size: 1.2rem;">"🛡️"</span>
                                        <div>
                                            <h3 style="font-size: 0.92rem; font-weight: 700; color: #cba6f7; margin-bottom: 0.1rem;">
                                                "Crear Administrador Inicial"
                                            </h3>
                                            <p style="font-size: 0.75rem; color: #585b70;">
                                                "Este será el primer usuario con privilegios totales de RBAC."
                                            </p>
                                        </div>
                                    </div>

                                    // Feedback del formulario de admin
                                    {move || {
                                        if let Some(err) = admin_create_error.get() {
                                            view! {
                                                <div class="alert-box alert-danger" style="margin-bottom: 0.75rem; font-size: 0.8rem;">
                                                    {err}
                                                </div>
                                            }.into_view()
                                        } else if let Some(ok) = admin_create_ok.get() {
                                            view! {
                                                <div class="alert-box alert-success" style="margin-bottom: 0.75rem; font-size: 0.8rem;">
                                                    {ok}
                                                </div>
                                            }.into_view()
                                        } else {
                                            view! {}.into_view()
                                        }
                                    }}

                                    <div style="display: flex; flex-direction: column; gap: 0.85rem;">
                                        // Username del admin
                                        <div>
                                            <label style="
                                                display: block;
                                                font-size: 0.75rem;
                                                color: #a6adc8;
                                                font-weight: 600;
                                                margin-bottom: 0.35rem;
                                            " for="admin-username-input">
                                                "Nombre de usuario (Admin)"
                                            </label>
                                            <input
                                                id="admin-username-input"
                                                type="text"
                                                class="login-input"
                                                placeholder="ej: admin_soc"
                                                prop:value=move || admin_username.get()
                                                on:input=move |ev| set_admin_username.set(event_target_value(&ev))
                                            />
                                        </div>

                                        // Contraseña del admin
                                        <div>
                                            <label style="
                                                display: block;
                                                font-size: 0.75rem;
                                                color: #a6adc8;
                                                font-weight: 600;
                                                margin-bottom: 0.35rem;
                                            " for="admin-password-input">
                                                "Contraseña (mín. 6 caracteres)"
                                            </label>
                                            <input
                                                id="admin-password-input"
                                                type="password"
                                                class="login-input"
                                                placeholder="Contraseña segura"
                                                prop:value=move || admin_password.get()
                                                on:input=move |ev| set_admin_password.set(event_target_value(&ev))
                                            />
                                        </div>

                                        // Confirmar contraseña
                                        <div>
                                            <label style="
                                                display: block;
                                                font-size: 0.75rem;
                                                color: #a6adc8;
                                                font-weight: 600;
                                                margin-bottom: 0.35rem;
                                            " for="admin-confirm-pw-input">
                                                "Confirmar Contraseña"
                                            </label>
                                            <input
                                                id="admin-confirm-pw-input"
                                                type="password"
                                                class="login-input"
                                                placeholder="Repita la contraseña"
                                                prop:value=move || admin_confirm_pw.get()
                                                on:input=move |ev| set_admin_confirm_pw.set(event_target_value(&ev))
                                            />
                                        </div>

                                        <button
                                            type="button"
                                            class="login-btn-primary"
                                            id="create-admin-submit-btn"
                                            style="
                                                background: linear-gradient(135deg, #cba6f7, #89b4fa);
                                                margin-top: 0.25rem;
                                            "
                                            on:click=handle_create_admin
                                        >
                                            "✓ Registrar Administrador"
                                        </button>
                                    </div>
                                </div>
                            }.into_view()
                        } else {
                            view! {}.into_view()
                        }
                    }}
                </div>

                // ── Tarjeta de sesión activa / RBAC badge ─────────────────────
                {move || {
                    let user = session.get().current_user.clone();
                    if user.username.is_empty() {
                        view! {}.into_view()
                    } else {
                        let role_class = match user.role {
                            RoleType::Admin      => "badge-role badge-role-admin",
                            RoleType::SocOperator=> "badge-role badge-role-operator",
                            RoleType::Auditor    => "badge-role badge-role-auditor",
                            RoleType::AnalystL1  => "badge-role badge-role-l1",
                        };
                        let role_icon = match user.role {
                            RoleType::Admin      => "👑",
                            RoleType::SocOperator=> "⚡",
                            RoleType::Auditor    => "🔍",
                            RoleType::AnalystL1  => "🎯",
                        };
                        let role_name = user.role.display_name();
                        let username  = user.username.clone();

                        view! {
                            <div class="login-glass-card login-card-anim" style="
                                padding: 1.25rem 1.5rem;
                                border-color: rgba(137,180,250,0.3);
                                display: flex;
                                align-items: center;
                                gap: 1rem;
                                flex-wrap: wrap;
                            ">
                                // Ícono de usuario
                                <div style="
                                    width: 44px;
                                    height: 44px;
                                    border-radius: 50%;
                                    background: linear-gradient(135deg, rgba(137,180,250,0.2), rgba(203,166,247,0.2));
                                    border: 1px solid rgba(137,180,250,0.4);
                                    display: flex;
                                    align-items: center;
                                    justify-content: center;
                                    font-size: 1.2rem;
                                    flex-shrink: 0;
                                    animation: badge-pulse 2.5s ease-in-out infinite;
                                ">
                                    {role_icon}
                                </div>

                                // Info del usuario
                                <div style="flex: 1; min-width: 0;">
                                    <div style="
                                        font-size: 0.72rem;
                                        color: #6c7086;
                                        font-weight: 600;
                                        letter-spacing: 0.05em;
                                        text-transform: uppercase;
                                        margin-bottom: 0.25rem;
                                    ">
                                        "Sesión Activa"
                                    </div>
                                    <div style="
                                        display: flex;
                                        align-items: center;
                                        gap: 0.6rem;
                                        flex-wrap: wrap;
                                    ">
                                        <span style="
                                            font-size: 0.95rem;
                                            font-weight: 700;
                                            color: #cdd6f4;
                                            font-family: var(--font-mono);
                                        ">
                                            {username}
                                        </span>
                                        <span class=role_class>
                                            {role_name}
                                        </span>
                                    </div>
                                </div>

                                // Indicador online
                                <div style="display: flex; align-items: center; gap: 0.4rem; flex-shrink: 0;">
                                    <span class="status-dot" style="background:#a6e3a1; color:#a6e3a1;"></span>
                                    <span style="font-size: 0.72rem; color: #a6e3a1; font-weight: 600;">
                                        "AUTENTICADO"
                                    </span>
                                </div>
                            </div>
                        }.into_view()
                    }
                }}

                // ── Footer informativo ────────────────────────────────────────
                <div style="
                    text-align: center;
                    font-size: 0.72rem;
                    color: #45475a;
                    padding: 0.5rem 0;
                ">
                    <span>"Tigo SOC Platform · Leptos 0.6 WASM · Catppuccin Mocha"</span>
                    <br/>
                    <span style="color: #313244;">"v0.1.0 · Acceso restringido a personal autorizado"</span>
                </div>
            </div>
        </div>

        // ════════════════════════════════════════════════════════════════════
        // Modal de Re-autenticación Estricta
        // ════════════════════════════════════════════════════════════════════
        {move || {
            if show_reauth_modal.get() {
                let username = session.get().current_user.username.clone();
                view! {
                    <div class="modal-overlay" id="reauth-modal-overlay" style="z-index: 9999;">
                        <div class="modal-content reauth-modal login-card-anim" style="
                            background: rgba(24,24,37,0.92);
                            backdrop-filter: blur(24px);
                            border-color: rgba(250,179,135,0.5);
                            max-width: 440px;
                        ">
                            // Header del modal
                            <div class="modal-header reauth-header" style="
                                background: linear-gradient(90deg, rgba(250,179,135,0.12), transparent);
                                border-bottom-color: rgba(250,179,135,0.2);
                            ">
                                <div style="display: flex; align-items: center; gap: 0.6rem;">
                                    <span style="font-size: 1.3rem;">"🔒"</span>
                                    <div>
                                        <h3 style="color: #fab387; font-weight: 700; font-size: 1rem;">
                                            "Re-autenticación Requerida"
                                        </h3>
                                        <p style="font-size: 0.73rem; color: #585b70; margin-top: 0.1rem;">
                                            "Timeout de acción crítica expirado"
                                        </p>
                                    </div>
                                </div>
                                <button
                                    class="modal-close"
                                    id="reauth-modal-close-btn"
                                    on:click=move |_| {
                                        set_show_reauth_modal.set(false);
                                        set_reauth_error.set(None);
                                        set_reauth_password.set(String::new());
                                    }
                                    style="color: #6c7086;"
                                >
                                    "✕"
                                </button>
                            </div>

                            // Body del modal
                            <div class="modal-body">
                                // Banner de advertencia
                                <div class="reauth-warning-banner">
                                    <p style="font-size: 0.82rem; color: #fab387; font-weight: 600; margin-bottom: 0.35rem;">
                                        "⚠ Verificación de Identidad Necesaria"
                                    </p>
                                    <p style="font-size: 0.78rem; color: #9399b2; line-height: 1.5;">
                                        "Para ejecutar acciones de impacto crítico (mitigaciones masivas, rollback de red), \
                                         debe re-verificar su identidad. La sesión de monitoreo continúa activa."
                                    </p>
                                </div>

                                // Usuario actual
                                <div style="
                                    display: flex;
                                    align-items: center;
                                    gap: 0.75rem;
                                    padding: 0.85rem 1rem;
                                    background: rgba(30,30,46,0.6);
                                    border: 1px solid rgba(88,91,112,0.4);
                                    border-radius: 10px;
                                    margin-bottom: 1.25rem;
                                ">
                                    <span style="font-size: 1.1rem;">"👤"</span>
                                    <div>
                                        <div style="font-size: 0.72rem; color: #585b70; font-weight: 600; text-transform: uppercase; letter-spacing: 0.04em;">
                                            "Operador en sesión"
                                        </div>
                                        <div style="font-size: 0.95rem; font-weight: 700; color: #cdd6f4; font-family: var(--font-mono);">
                                            {username}
                                        </div>
                                    </div>
                                    <div style="margin-left: auto;">
                                        <span class=move || match session.get().current_user.role {
                                            RoleType::Admin      => "badge-role badge-role-admin",
                                            RoleType::SocOperator=> "badge-role badge-role-operator",
                                            RoleType::Auditor    => "badge-role badge-role-auditor",
                                            RoleType::AnalystL1  => "badge-role badge-role-l1",
                                        }>
                                            {move || session.get().current_user.role.display_name()}
                                        </span>
                                    </div>
                                </div>

                                // Feedback del re-auth
                                {move || {
                                    if let Some(err) = reauth_error.get() {
                                        view! {
                                            <div class="alert-box alert-danger" style="margin-bottom: 1rem; font-size: 0.82rem;">
                                                {err}
                                            </div>
                                        }.into_view()
                                    } else if reauth_ok.get() {
                                        view! {
                                            <div class="alert-box alert-success" style="margin-bottom: 1rem; font-size: 0.82rem;">
                                                "✓ Re-autenticación exitosa. Acceso a acciones críticas restablecido."
                                            </div>
                                        }.into_view()
                                    } else {
                                        view! {}.into_view()
                                    }
                                }}

                                // Campo contraseña
                                <div style="margin-bottom: 1rem;">
                                    <label style="
                                        display: block;
                                        font-size: 0.78rem;
                                        color: #a6adc8;
                                        font-weight: 600;
                                        margin-bottom: 0.4rem;
                                    " for="reauth-password-input">
                                        "🔑 Contraseña de Verificación"
                                    </label>
                                    <input
                                        id="reauth-password-input"
                                        type="password"
                                        class="login-input"
                                        placeholder="Ingrese su contraseña para continuar"
                                        prop:value=move || reauth_password.get()
                                        on:input=move |ev| set_reauth_password.set(event_target_value(&ev))
                                    />
                                </div>

                                // Botones de acción del modal
                                <div style="display: flex; gap: 0.75rem;">
                                    <button
                                        type="button"
                                        class="btn btn-inspect"
                                        id="reauth-cancel-btn"
                                        style="flex: 1;"
                                        on:click=move |_| {
                                            set_show_reauth_modal.set(false);
                                            set_reauth_error.set(None);
                                            set_reauth_password.set(String::new());
                                        }
                                    >
                                        "Cancelar"
                                    </button>
                                    <button
                                        type="button"
                                        class="login-btn-primary"
                                        id="reauth-confirm-btn"
                                        style="
                                            flex: 2;
                                            padding: 0.6rem 1rem;
                                            background: linear-gradient(135deg, #fab387, #f38ba8);
                                        "
                                        on:click=handle_reauth
                                    >
                                        "🔓 Verificar Identidad"
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
    }
}
