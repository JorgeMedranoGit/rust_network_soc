#!/usr/bin/env python3
"""
Generador de Mockups / Wireframes en Tema Claro (Light Theme)
para las 8 páginas reales de tigo_soc_frontend en tres vistas:
1. PC (Desktop: 1440x900)
2. Tablet (1024x768)
3. Celular (Mobile: 390x844)

Cumple estrictamente con:
- Estilo Wireframe / Mockup de diseño (esquemático, líneas, cajas, sin aspecto de página final).
- Header global del sistema en todas las páginas (2 a 8), omitido en Login (1).
- Placeholders de imagen con cajas y diagonales 'X'.
- Placeholders de texto con ejemplos explícitos [Placeholder: "..." | Ej: "..."].
- Funcionalidades detalladas para cada botón [ Botón: "..." -> Acción ] y enlace < Enlace: "..." -> Acción >.
- Salida final en MockupsFiles/Final/ (SVG, PNG, PDF y Documentación).
"""

import os
import subprocess
from pypdf import PdfWriter

FINAL_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "Final")
os.makedirs(FINAL_DIR, exist_ok=True)

# ── Colores de Wireframe Claro ───────────────────────────────────────────
C_BG = "#FFFFFF"
C_PANEL_BG = "#F8FAFC"
C_PANEL_ALT = "#F1F5F9"
C_BORDER = "#475569"          # Gris grafito sólido
C_BORDER_LIGHT = "#94A3B8"    # Gris medio para guías y diagonales
C_BORDER_DASH = "#64748B"
C_TEXT = "#0F172A"            # Texto grafito oscuro
C_TEXT_MUTED = "#64748B"      # Texto secundario
C_TEXT_ACTION = "#0369A1"     # Azul tenue para anotaciones de acción
C_BTN_BG = "#E2E8F0"
C_BTN_ACTIVE = "#CBD5E1"
C_BADGE_BG = "#F1F5F9"

FONT = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
FONT_MONO = "'Courier New', Courier, monospace"

def svg_header(w, h):
    return f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">'

def svg_footer():
    return '</svg>'

def draw_placeholder_img(x, y, w, h, label):
    """Dibuja una caja de placeholder estándar de wireframe con líneas diagonales en X"""
    return f"""
    <g transform="translate({x}, {y})">
      <rect width="{w}" height="{h}" fill="{C_PANEL_ALT}" stroke="{C_BORDER}" stroke-width="1.5"/>
      <line x1="0" y1="0" x2="{w}" y2="{h}" stroke="{C_BORDER_LIGHT}" stroke-width="1" stroke-dasharray="4"/>
      <line x1="0" y1="{h}" x2="{w}" y2="0" stroke="{C_BORDER_LIGHT}" stroke-width="1" stroke-dasharray="4"/>
      <rect x="{w/2 - 130}" y="{h/2 - 14}" width="260" height="28" rx="4" fill="{C_BG}" stroke="{C_BORDER}" stroke-width="1"/>
      <text x="{w/2}" y="{h/2 + 4}" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11" font-weight="700" text-anchor="middle">[ Placeholder: {label} ]</text>
    </g>"""

def draw_btn(x, y, w, h, text, action, is_active=False):
    bg = C_BTN_ACTIVE if is_active else C_BTN_BG
    border_w = "2" if is_active else "1.5"
    return f"""
    <g transform="translate({x}, {y})">
      <rect width="{w}" height="{h}" rx="4" fill="{bg}" stroke="{C_BORDER}" stroke-width="{border_w}"/>
      <text x="{w/2}" y="{h/2 + 4}" fill="{C_TEXT}" font-family="{FONT}" font-size="11" font-weight="600" text-anchor="middle">{text}</text>
      <!-- Anotación de acción funcional -->
      <text x="{w/2}" y="{h + 12}" fill="{C_TEXT_ACTION}" font-family="{FONT_MONO}" font-size="8" text-anchor="middle">-&gt; {action}</text>
    </g>"""

def draw_input(x, y, w, h, label, placeholder, example):
    return f"""
    <g transform="translate({x}, {y})">
      <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="11" font-weight="700">{label}</text>
      <rect y="8" width="{w}" height="{h}" rx="4" fill="{C_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
      <text x="10" y="{8 + h/2 + 4}" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="11">[ Placeholder: "{placeholder}" | Ej: "{example}" ]</text>
    </g>"""

# ── CABECERAS GLOBALES (Mantenidas en páginas 2 a 8) ─────────────────────

TABS = [
    ("Dashboard", "Panel de Control", "ActiveTab::Dashboard"),
    ("Topology", "Topología 3D", "ActiveTab::Topology"),
    ("MonitoringCharts", "Gráficas", "ActiveTab::MonitoringCharts"),
    ("RealtimeStream", "Flujo en Vivo", "ActiveTab::RealtimeStream"),
    ("Alerts", "Alertas", "ActiveTab::Alerts"),
    ("BlastRadius", "Blast Radius [3]", "ActiveTab::BlastRadius"),
    ("UserManagement", "Usuarios &amp; RBAC", "ActiveTab::UserManagement"),
]

def render_header_pc(active_key):
    out = f"""
    <!-- ════════════════ HEADER GLOBAL PC (1440px) ════════════════ -->
    <rect width="1440" height="74" fill="{C_PANEL_ALT}" stroke="{C_BORDER}" stroke-width="2"/>
    
    <!-- Marca y Logo con Placeholder -->
    <g transform="translate(20, 14)">
      <rect width="36" height="36" fill="{C_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
      <line x1="0" y1="0" x2="36" y2="36" stroke="{C_BORDER_LIGHT}" stroke-width="1"/>
      <line x1="0" y1="36" x2="36" y2="0" stroke="{C_BORDER_LIGHT}" stroke-width="1"/>
      <text x="46" y="18" fill="{C_TEXT}" font-family="{FONT}" font-size="15" font-weight="800">TIGO SOC MONITOR</text>
      <text x="46" y="32" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="9">[ Placeholder Logo Escudo ] | Rust Tokio + Leptos WASM</text>
    </g>

    <!-- Barra de Pestañas / Navegación Global -->
    <g transform="translate(300, 18)">
    """
    x_offset = 0
    for key, label, target in TABS:
        is_act = (key == active_key)
        w = 125 if "Blast" in label else (115 if "Usuarios" in label else 105)
        bg = C_BTN_ACTIVE if is_act else C_BG
        stroke_w = "2" if is_act else "1.5"
        out += f"""
        <g transform="translate({x_offset}, 0)">
          <rect width="{w}" height="32" rx="4" fill="{bg}" stroke="{C_BORDER}" stroke-width="{stroke_w}"/>
          <text x="{w/2}" y="20" fill="{C_TEXT}" font-family="{FONT}" font-size="10" font-weight="700" text-anchor="middle">{label}</text>
          <text x="{w/2}" y="42" fill="{C_TEXT_ACTION}" font-family="{FONT_MONO}" font-size="7.5" text-anchor="middle">&lt;Link: {target}&gt;</text>
        </g>"""
        x_offset += w + 8

    out += f"""
    </g>

    <!-- Controles de Usuario y Estado del Header -->
    <g transform="translate(1140, 16)">
      <!-- Dropdown de Perfil RBAC -->
      <rect width="130" height="28" rx="4" fill="{C_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
      <text x="8" y="18" fill="{C_TEXT}" font-family="{FONT}" font-size="9.5">[ Dropdown: admin_soc (Admin) v ]</text>
      <text x="65" y="38" fill="{C_TEXT_ACTION}" font-family="{FONT_MONO}" font-size="7.5" text-anchor="middle">-&gt; Conmuta Perfil RBAC</text>

      <!-- Badge de Estado En Línea -->
      <rect x="140" y="2" width="70" height="24" rx="12" fill="{C_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
      <circle cx="150" cy="14" r="3.5" fill="{C_BORDER}"/>
      <text x="178" y="17" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="9" font-weight="700" text-anchor="middle">ONLINE</text>

      <!-- Botón de Logout -->
      <g transform="translate(220, 0)">
        <rect width="68" height="28" rx="4" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
        <text x="34" y="18" fill="{C_TEXT}" font-family="{FONT}" font-size="9.5" font-weight="700" text-anchor="middle">SALIR</text>
        <text x="34" y="38" fill="{C_TEXT_ACTION}" font-family="{FONT_MONO}" font-size="7.5" text-anchor="middle">-&gt; Logout API</text>
      </g>
    </g>
    <line x1="0" y1="74" x2="1440" y2="74" stroke="{C_BORDER}" stroke-width="2"/>
    """
    return out

def render_header_tablet(active_key):
    out = f"""
    <!-- ════════════════ HEADER GLOBAL TABLET (1024px) ════════════════ -->
    <rect width="1024" height="88" fill="{C_PANEL_ALT}" stroke="{C_BORDER}" stroke-width="2"/>
    <g transform="translate(16, 12)">
      <rect width="28" height="28" fill="{C_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
      <line x1="0" y1="0" x2="28" y2="28" stroke="{C_BORDER_LIGHT}" stroke-width="1"/>
      <line x1="0" y1="28" x2="28" y2="0" stroke="{C_BORDER_LIGHT}" stroke-width="1"/>
      <text x="36" y="18" fill="{C_TEXT}" font-family="{FONT}" font-size="13" font-weight="800">TIGO SOC MONITOR</text>
    </g>
    <g transform="translate(760, 12)">
      <rect width="130" height="26" rx="4" fill="{C_BG}" stroke="{C_BORDER}" stroke-width="1"/>
      <text x="8" y="17" fill="{C_TEXT}" font-family="{FONT}" font-size="9">[ Perfil: admin_soc v ]</text>
      <rect x="140" y="0" width="60" height="26" rx="4" fill="{C_BG}" stroke="{C_BORDER}" stroke-width="1"/>
      <text x="170" y="17" fill="{C_TEXT}" font-family="{FONT}" font-size="9" text-anchor="middle">SALIR</text>
    </g>

    <!-- Fila 2: Pestañas en Tablet -->
    <g transform="translate(16, 50)">
    """
    x_off = 0
    for key, label, target in TABS:
        is_act = (key == active_key)
        w = 136 if "Blast" in label else (134 if "Usuarios" in label else 120)
        bg = C_BTN_ACTIVE if is_act else C_BG
        out += f"""
        <g transform="translate({x_off}, 0)">
          <rect width="{w}" height="26" rx="4" fill="{bg}" stroke="{C_BORDER}" stroke-width="1.5"/>
          <text x="{w/2}" y="17" fill="{C_TEXT}" font-family="{FONT}" font-size="9" font-weight="700" text-anchor="middle">{label}</text>
        </g>"""
        x_off += w + 6
    out += "</g><line x1='0' y1='88' x2='1024' y2='88' stroke='" + C_BORDER + "' stroke-width='2'/>"
    return out

def render_header_mobile(active_key):
    return f"""
    <!-- ════════════════ HEADER GLOBAL CELULAR (390px) ════════════════ -->
    <rect width="390" height="56" fill="{C_PANEL_ALT}" stroke="{C_BORDER}" stroke-width="2"/>
    <g transform="translate(16, 14)">
      <rect width="26" height="26" fill="{C_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
      <line x1="0" y1="0" x2="26" y2="26" stroke="{C_BORDER_LIGHT}" stroke-width="1"/>
      <line x1="0" y1="26" x2="26" y2="0" stroke="{C_BORDER_LIGHT}" stroke-width="1"/>
      <text x="34" y="18" fill="{C_TEXT}" font-family="{FONT}" font-size="12" font-weight="800">TIGO SOC</text>
    </g>
    <!-- Selector y Menú -->
    <g transform="translate(230, 14)">
      <rect width="80" height="26" rx="4" fill="{C_BG}" stroke="{C_BORDER}" stroke-width="1"/>
      <text x="40" y="17" fill="{C_TEXT}" font-family="{FONT}" font-size="8.5" text-anchor="middle">admin_soc v</text>
      <!-- Botón Menú Hamburguesa -->
      <rect x="90" y="0" width="30" height="26" rx="4" fill="{C_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
      <line x1="96" y1="7" x2="114" y2="7" stroke="{C_BORDER}" stroke-width="1.5"/>
      <line x1="96" y1="13" x2="114" y2="13" stroke="{C_BORDER}" stroke-width="1.5"/>
      <line x1="96" y1="19" x2="114" y2="19" stroke="{C_BORDER}" stroke-width="1.5"/>
    </g>
    <line x1="0" y1="56" x2="390" y2="56" stroke="{C_BORDER}" stroke-width="2"/>
    """

def render_footer_mobile(active_key):
    """Barra inferior táctil para celulares"""
    return f"""
    <!-- Bottom Navigation Bar Móvil -->
    <g transform="translate(0, 784)">
      <rect width="390" height="60" fill="{C_PANEL_ALT}" stroke="{C_BORDER}" stroke-width="2"/>
      <text x="40" y="34" fill="{C_TEXT}" font-family="{FONT}" font-size="9" text-anchor="middle">Mando</text>
      <text x="115" y="34" fill="{C_TEXT}" font-family="{FONT}" font-size="9" text-anchor="middle">Topología</text>
      <text x="195" y="34" fill="{C_TEXT}" font-family="{FONT}" font-size="9" text-anchor="middle">Alertas (3)</text>
      <text x="275" y="34" fill="{C_TEXT}" font-family="{FONT}" font-size="9" text-anchor="middle">Mitigar</text>
      <text x="350" y="34" fill="{C_TEXT}" font-family="{FONT}" font-size="9" text-anchor="middle">Más</text>
      <line x1="0" y1="0" x2="390" y2="0" stroke="{C_BORDER}" stroke-width="1.5"/>
    </g>"""

# ═════════════════════════════════════════════════════════════════════════
# CONSTRUCTORES DE LAS 8 PÁGINAS REALES DE TIGO_SOC_FRONTEND
# ═════════════════════════════════════════════════════════════════════════

# ── PÁGINA 1: LoginView (Sin Header Global) ──────────────────────────────
def build_page_01_login(viewport):
    w, h = viewport["w"], viewport["h"]
    svg = svg_header(w, h)
    svg += f'<rect width="{w}" height="{h}" fill="{C_BG}"/>'
    
    if viewport["mode"] == "pc":
        svg += f"""
        <!-- Título Superior de Wireframe -->
        <text x="40" y="40" fill="{C_TEXT}" font-family="{FONT}" font-size="16" font-weight="800">WIREFRAME 01: PANTALLA DE ACCESO (LoginView.rs) - VISTA PC</text>
        <text x="40" y="58" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="11">[ Nota: No incluye header global conforme a la directiva de diseño ]</text>

        <!-- Cuadro Central de Login -->
        <g transform="translate(470, 120)">
          <rect width="500" height="650" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="2"/>
          
          <!-- Placeholder de Logo -->
          {draw_placeholder_img(180, 30, 140, 70, "Logo Tigo SOC")}
          <text x="250" y="125" fill="{C_TEXT}" font-family="{FONT}" font-size="16" font-weight="800" text-anchor="middle">ACCESO A CONSOLA SOC</text>
          <text x="250" y="142" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="10" text-anchor="middle">Autenticación Zero-Trust / Sesión Servidor Axum</text>

          <!-- Pestañas Conmutables: Login vs Register -->
          <g transform="translate(50, 165)">
            <rect width="200" height="34" fill="{C_BTN_ACTIVE}" stroke="{C_BORDER}" stroke-width="1.5"/>
            <text x="100" y="22" fill="{C_TEXT}" font-family="{FONT}" font-size="11" font-weight="700" text-anchor="middle">[ Tab: Iniciar Sesión ]</text>
            <rect x="200" y="0" width="200" height="34" fill="{C_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
            <text x="300" y="22" fill="{C_TEXT_MUTED}" font-family="{FONT}" font-size="11" text-anchor="middle">[ Tab: Registrarse ]</text>
          </g>

          <!-- Inputs -->
          <g transform="translate(50, 220)">
            {draw_input(0, 0, 400, 40, "USUARIO OPERATIVO:", "admin_soc", "admin_soc")}
            {draw_input(0, 75, 400, 40, "CONTRASEÑA DE ACCESO:", "••••••••••••", "admin")}
            
            <!-- Selector de Rol Inicial -->
            <text x="0" y="150" fill="{C_TEXT}" font-family="{FONT}" font-size="11" font-weight="700">PERFIL RBAC INICIAL:</text>
            <rect y="158" width="400" height="40" fill="{C_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
            <text x="12" y="183" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[ Dropdown: SOC_ADMIN (Nivel 5 - Acceso Total) v ]</text>
            <text x="200" y="210" fill="{C_TEXT_ACTION}" font-family="{FONT_MONO}" font-size="9" text-anchor="middle">&lt;Select: Conmuta roles iniciales en catálogo DB&gt;</text>

            <!-- Botón Principal -->
            {draw_btn(0, 225, 400, 48, "[ Botón: INICIAR SESIÓN ]", "Valida credenciales contra POST /api/v1/auth/login", True)}

            <!-- Enlace Recuperar -->
            <text x="200" y="305" fill="{C_TEXT_ACTION}" font-family="{FONT}" font-size="11" text-anchor="middle">&lt; Enlace: "¿Olvidó sus credenciales? Contacte a seguridad" -&gt; Modal Soporte &gt;</text>
          </g>

          <!-- Placeholder de Pie Criptográfico -->
          <g transform="translate(50, 580)">
            <rect width="400" height="40" fill="{C_BG}" stroke="{C_BORDER_LIGHT}" stroke-width="1" stroke-dasharray="3"/>
            <text x="200" y="25" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="10" text-anchor="middle">[ Placeholder: SHA-256 | TLS 1.3 | PostgreSQL Session Store ]</text>
          </g>
        </g>"""
    elif viewport["mode"] == "tablet":
        svg += f"""
        <text x="30" y="36" fill="{C_TEXT}" font-family="{FONT}" font-size="14" font-weight="800">WIREFRAME 01: LoginView.rs - VISTA TABLET (1024x768)</text>
        <g transform="translate(280, 80)">
          <rect width="464" height="630" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="2"/>
          {draw_placeholder_img(162, 24, 140, 60, "Logo Escudo Tigo")}
          <text x="232" y="110" fill="{C_TEXT}" font-family="{FONT}" font-size="15" font-weight="800" text-anchor="middle">ACCESO TABLET SOC</text>
          <g transform="translate(32, 130)">
            {draw_input(0, 0, 400, 38, "USUARIO:", "operador_red", "operador_red")}
            {draw_input(0, 70, 400, 38, "CONTRASEÑA:", "••••••••••••", "admin")}
            <text x="0" y="145" fill="{C_TEXT}" font-family="{FONT}" font-size="11" font-weight="700">ROL:</text>
            <rect y="153" width="400" height="38" fill="{C_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
            <text x="12" y="177" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[ Dropdown: SOC_OPERATOR v ]</text>
            {draw_btn(0, 215, 400, 46, "[ Botón: INICIAR SESIÓN ]", "Valida sesión y abre consola en tablet", True)}
          </g>
        </g>"""
    else: # mobile
        svg += f"""
        <text x="16" y="26" fill="{C_TEXT}" font-family="{FONT}" font-size="11" font-weight="800">WIREFRAME 01: LoginView.rs - CELULAR (390x844)</text>
        <g transform="translate(20, 50)">
          <rect width="350" height="730" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="2"/>
          {draw_placeholder_img(105, 30, 140, 60, "Logo Móvil")}
          <text x="175" y="116" fill="{C_TEXT}" font-family="{FONT}" font-size="14" font-weight="800" text-anchor="middle">ACCESO MÓVIL SOC</text>
          <g transform="translate(25, 140)">
            {draw_input(0, 0, 300, 44, "USUARIO:", "admin_soc", "admin_soc")}
            {draw_input(0, 75, 300, 44, "CONTRASEÑA:", "••••••••••••", "admin")}
            {draw_btn(0, 170, 300, 48, "[ Botón: ENTRAR ]", "Touch Target 48px Fitts", True)}
          </g>
          <g transform="translate(25, 450)">
            <rect width="300" height="120" fill="{C_BG}" stroke="{C_BORDER_LIGHT}" stroke-width="1" stroke-dasharray="4"/>
            <text x="150" y="40" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="9" text-anchor="middle">[ Placeholder: Re-auth 300s Info ]</text>
            <text x="150" y="65" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="9" text-anchor="middle">Ej: "Inactividad 15 min expira"</text>
          </g>
        </g>"""

    svg += svg_footer()
    return svg

# ── PÁGINA 2: ActiveTab::Dashboard (Panel de Control Principal) ──────────
def build_page_02_dashboard(viewport):
    w, h = viewport["w"], viewport["h"]
    svg = svg_header(w, h)
    svg += f'<rect width="{w}" height="{h}" fill="{C_BG}"/>'

    if viewport["mode"] == "pc":
        svg += render_header_pc("Dashboard")
        svg += f"""
        <!-- Contenedor Principal Dashboard -->
        <g transform="translate(40, 95)">
          <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="15" font-weight="800">PANEL DE CONTROL GENERAL (ActiveTab::Dashboard / SystemStatus + MonitoringChart + AlertsTable)</text>

          <!-- Fila de SystemStatus (5 Métricas Clave) -->
          <g transform="translate(0, 16)">
            <!-- Métrica 1 -->
            <rect width="250" height="74" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
            <text x="12" y="20" fill="{C_TEXT_MUTED}" font-family="{FONT}" font-size="10" font-weight="700">TOTAL ALERTAS:</text>
            <text x="12" y="48" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="22" font-weight="700">[ Ej: 1,420 ]</text>
            <text x="12" y="64" fill="{C_TEXT_ACTION}" font-family="{FONT_MONO}" font-size="8.5">&lt;Métrica: metrics.total_alerts&gt;</text>

            <!-- Métrica 2 -->
            <g transform="translate(265, 0)">
              <rect width="250" height="74" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
              <text x="12" y="20" fill="{C_TEXT_MUTED}" font-family="{FONT}" font-size="10" font-weight="700">MITIGADAS CON ÉXITO:</text>
              <text x="12" y="48" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="22" font-weight="700">[ Ej: 984 ]</text>
              <text x="12" y="64" fill="{C_TEXT_ACTION}" font-family="{FONT_MONO}" font-size="8.5">&lt;Métrica: metrics.mitigated&gt;</text>
            </g>

            <!-- Métrica 3 -->
            <g transform="translate(530, 0)">
              <rect width="250" height="74" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
              <text x="12" y="20" fill="{C_TEXT_MUTED}" font-family="{FONT}" font-size="10" font-weight="700">AMENAZAS CRÍTICAS (NIVEL 5):</text>
              <text x="12" y="48" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="22" font-weight="700">[ Ej: 3 ]</text>
              <text x="12" y="64" fill="{C_TEXT_ACTION}" font-family="{FONT_MONO}" font-size="8.5">&lt;Métrica: metrics.critical_threats&gt;</text>
            </g>

            <!-- Métrica 4 -->
            <g transform="translate(795, 0)">
              <rect width="250" height="74" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
              <text x="12" y="20" fill="{C_TEXT_MUTED}" font-family="{FONT}" font-size="10" font-weight="700">NODOS ACTIVOS EN TOPOLOGÍA:</text>
              <text x="12" y="48" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="22" font-weight="700">[ Ej: 12 ]</text>
              <text x="12" y="64" fill="{C_TEXT_ACTION}" font-family="{FONT_MONO}" font-size="8.5">&lt;Métrica: metrics.active_nodes&gt;</text>
            </g>

            <!-- Métrica 5 -->
            <g transform="translate(1060, 0)">
              <rect width="300" height="74" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
              <text x="12" y="20" fill="{C_TEXT_MUTED}" font-family="{FONT}" font-size="10" font-weight="700">TASA SNIFFER LIBPCAP:</text>
              <text x="12" y="48" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="22" font-weight="700">[ Ej: 1,842 pkts/s ]</text>
              <text x="12" y="64" fill="{C_TEXT_ACTION}" font-family="{FONT_MONO}" font-size="8.5">&lt;Métrica: metrics.packets_per_second&gt;</text>
            </g>
          </g>

          <!-- Gráfico de Monitoreo Central (Placeholder) -->
          <g transform="translate(0, 105)">
            {draw_placeholder_img(0, 0, 1360, 240, "Gráfico de Tráfico vs Detecciones ML (MonitoringChart.rs)")}
            <text x="20" y="30" fill="{C_TEXT}" font-family="{FONT}" font-size="12" font-weight="700">TELEMETRÍA EN TIEMPO REAL (Línea de Tráfico Normal + Puntos de Anomalía LightGBM)</text>
          </g>

          <!-- Tabla de Últimas Detecciones (AlertsTable) -->
          <g transform="translate(0, 365)">
            <rect width="1360" height="380" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
            <text x="20" y="28" fill="{C_TEXT}" font-family="{FONT}" font-size="13" font-weight="700">ÚLTIMAS DETECCIONES DE SEGURIDAD (MULTI-VENDOR) - AlertsTable.rs</text>
            
            <!-- Encabezados de Columna -->
            <g transform="translate(20, 45)">
              <rect width="1320" height="30" fill="{C_PANEL_ALT}" stroke="{C_BORDER}" stroke-width="1"/>
              <text x="15" y="20" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">ID ALERTA</text>
              <text x="140" y="20" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">TIMESTAMP</text>
              <text x="280" y="20" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">SEVERIDAD</text>
              <text x="420" y="20" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">VECTOR AMENAZA</text>
              <text x="640" y="20" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">IP ORIGEN (ATACANTE)</text>
              <text x="880" y="20" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">IP DESTINO (VÍCTIMA)</text>
              <text x="1100" y="20" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">ACCIONES DISPONIBLES</text>

              <!-- Fila 1 de Ejemplo -->
              <g transform="translate(0, 36)">
                <rect width="1320" height="42" fill="{C_BG}" stroke="{C_BORDER_LIGHT}" stroke-width="1"/>
                <text x="15" y="26" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: ALT-001]</text>
                <text x="140" y="26" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="11">[Ej: 16:54:12]</text>
                <text x="280" y="26" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: CRITICAL (5)]</text>
                <text x="420" y="26" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: DDOS_SYN_FLOOD]</text>
                <text x="640" y="26" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: 192.168.100.50]</text>
                <text x="880" y="26" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: 192.168.1.15 : 80]</text>
                <!-- Botón Acción -->
                {draw_btn(1100, 6, 100, 28, "[ Mitigar ]", "Abre Blast Radius modal")}
                {draw_btn(1210, 6, 95, 28, "[ Detalle ]", "Inspecciona L4")}
              </g>

              <!-- Fila 2 de Ejemplo -->
              <g transform="translate(0, 84)">
                <rect width="1320" height="42" fill="{C_BG}" stroke="{C_BORDER_LIGHT}" stroke-width="1"/>
                <text x="15" y="26" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: ALT-002]</text>
                <text x="140" y="26" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="11">[Ej: 16:52:05]</text>
                <text x="280" y="26" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: HIGH (4)]</text>
                <text x="420" y="26" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: DNS_C2_EXFILTRATION]</text>
                <text x="640" y="26" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: 192.168.1.12]</text>
                <text x="880" y="26" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: 208.67.222.222:53]</text>
                {draw_btn(1100, 6, 100, 28, "[ Mitigar ]", "Despacha Sinkhole")}
                {draw_btn(1210, 6, 95, 28, "[ Detalle ]", "Inspecciona DNS")}
              </g>
            </g>
          </g>
        </g>"""
    elif viewport["mode"] == "tablet":
        svg += render_header_tablet("Dashboard")
        svg += f"""
        <g transform="translate(20, 100)">
          <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="14" font-weight="800">DASHBOARD SOC - VISTA TABLET (1024x768)</text>
          <!-- Grid 2x2 Métricas -->
          <g transform="translate(0, 15)">
            <rect width="230" height="60" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
            <text x="10" y="20" fill="{C_TEXT_MUTED}" font-family="{FONT}" font-size="9">ALERTAS: [Ej: 1,420]</text>
            <g transform="translate(245, 0)">
              <rect width="230" height="60" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
              <text x="10" y="20" fill="{C_TEXT_MUTED}" font-family="{FONT}" font-size="9">MITIGADAS: [Ej: 984]</text>
            </g>
            <g transform="translate(490, 0)">
              <rect width="230" height="60" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
              <text x="10" y="20" fill="{C_TEXT_MUTED}" font-family="{FONT}" font-size="9">CRÍTICAS: [Ej: 3]</text>
            </g>
            <g transform="translate(735, 0)">
              <rect width="245" height="60" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
              <text x="10" y="20" fill="{C_TEXT_MUTED}" font-family="{FONT}" font-size="9">TRÁFICO: [Ej: 1.8k p/s]</text>
            </g>
          </g>
          {draw_placeholder_img(0, 95, 984, 200, "Gráfico de Tráfico en Tablet")}
          <g transform="translate(0, 310)">
            <rect width="984" height="290" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
            <text x="16" y="25" fill="{C_TEXT}" font-family="{FONT}" font-size="12" font-weight="700">Alertas Recientes (Vista Tablet)</text>
          </g>
        </g>"""
    else: # mobile
        svg += render_header_mobile("Dashboard")
        svg += f"""
        <g transform="translate(16, 70)">
          <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="12" font-weight="800">DASHBOARD MÓVIL (ActiveTab::Dashboard)</text>
          <!-- Métricas Resumen -->
          <g transform="translate(0, 12)">
            <rect width="170" height="56" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1"/>
            <text x="10" y="20" fill="{C_TEXT_MUTED}" font-family="{FONT}" font-size="8">ALERTAS: [Ej: 1,420]</text>
            <text x="10" y="44" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="15" font-weight="700">[Ej: 3 Críticas]</text>

            <g transform="translate(185, 0)">
              <rect width="170" height="56" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1"/>
              <text x="10" y="20" fill="{C_TEXT_MUTED}" font-family="{FONT}" font-size="8">TRÁFICO LIBPCAP</text>
              <text x="10" y="44" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="14" font-weight="700">[Ej: 1,842 p/s]</text>
            </g>
          </g>
          {draw_placeholder_img(0, 85, 358, 140, "Gráfico Móvil (MonitoringChart)")}
          
          <!-- Lista Móvil de Alertas -->
          <g transform="translate(0, 240)">
            <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="11" font-weight="700">Detecciones Recientes (AlertsTable)</text>
            <g transform="translate(0, 10)">
              <rect width="358" height="110" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
              <text x="12" y="24" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11" font-weight="700">[Ej: ALT-001 | CRÍTICA 5]</text>
              <text x="12" y="44" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="10">Atacante: [Ej: 192.168.100.50]</text>
              <text x="12" y="60" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="10">Víctima: [Ej: 192.168.1.15 : 80]</text>
              {draw_btn(12, 70, 150, 30, "[ Botón: Mitigar ]", "Abre modal móvil")}
              {draw_btn(175, 70, 170, 30, "[ Botón: Detalle ]", "Inspección forense")}
            </g>
          </g>
        </g>
        {render_footer_mobile("Dashboard")}"""

    svg += svg_footer()
    return svg

# ── PÁGINA 3: ActiveTab::Topology (Topología de Red) ─────────────────────
def build_page_03_topology(viewport):
    w, h = viewport["w"], viewport["h"]
    svg = svg_header(w, h)
    svg += f'<rect width="{w}" height="{h}" fill="{C_BG}"/>'

    if viewport["mode"] == "pc":
        svg += render_header_pc("Topology")
        svg += f"""
        <g transform="translate(40, 95)">
          <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="15" font-weight="800">TOPOLOGÍA DE RED GNS3 / DOCKER (ActiveTab::Topology / Topology3D.rs)</text>

          <!-- Barra de Controles de Cámara y Visualización -->
          <g transform="translate(0, 15)">
            <rect width="1360" height="42" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
            <text x="16" y="26" fill="{C_TEXT}" font-family="{FONT}" font-size="11" font-weight="700">CONTROLES DEL VISUALIZADOR:</text>
            {draw_btn(200, 7, 140, 28, "[ Resetear Cámara ]", "Vuelve a posición inicial")}
            {draw_btn(350, 7, 150, 28, "[ Pausar Rotación ]", "Fija visualización 3D")}
            {draw_btn(510, 7, 170, 28, "[ Centrar Atacante ]", "Zoom en 192.168.100.50")}
            {draw_btn(690, 7, 160, 28, "[ Centrar Víctimas ]", "Zoom en VLAN Alpine")}
            {draw_btn(860, 7, 170, 28, "[ Alternar Enlaces ]", "Muestra tráfico en vivo")}
          </g>

          <!-- Canvas 3D Principal (Placeholder con 'X') -->
          <g transform="translate(0, 70)">
            {draw_placeholder_img(0, 0, 960, 680, "Canvas WebGL / Three.js de Topología de Red (12 Nodos GNS3)")}
            
            <!-- Superposición de Leyenda de Nodos -->
            <g transform="translate(20, 20)">
              <rect width="280" height="110" fill="{C_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
              <text x="12" y="22" fill="{C_TEXT}" font-family="{FONT}" font-size="11" font-weight="700">ESTADOS DE LOS NODOS:</text>
              <circle cx="20" cy="42" r="5" fill="{C_BORDER}"/>
              <text x="34" y="46" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10">[ Normal: 10 hosts operativos ]</text>
              <circle cx="20" cy="66" r="5" fill="{C_BORDER}"/>
              <text x="34" y="70" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10">[ Bajo Ataque: VictimeAlpine5 ]</text>
              <circle cx="20" cy="90" r="5" fill="{C_BORDER}"/>
              <text x="34" y="94" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10">[ Atacante: AttackerKali (DMZ) ]</text>
            </g>
          </g>

          <!-- Panel Lateral de Inspector de Nodo Seleccionado -->
          <g transform="translate(980, 70)">
            <rect width="380" height="680" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="2"/>
            <text x="20" y="32" fill="{C_TEXT}" font-family="{FONT}" font-size="13" font-weight="700">INSPECTOR DE NODO SELECCIONADO</text>
            <line x1="20" y1="46" x2="360" y2="46" stroke="{C_BORDER}" stroke-width="1"/>

            <!-- Datos Técnicos -->
            <g transform="translate(20, 65)">
              {draw_placeholder_img(0, 0, 340, 110, "Esquema de Interfaces del Nodo")}
              
              <g transform="translate(0, 130)">
                <text x="0" y="0" fill="{C_TEXT_MUTED}" font-family="{FONT}" font-size="10" font-weight="700">HOSTNAME:</text>
                <text x="0" y="18" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="12" font-weight="700">[ Ej: "VictimeAlpine5" ]</text>

                <text x="0" y="44" fill="{C_TEXT_MUTED}" font-family="{FONT}" font-size="10" font-weight="700">DIRECCIÓN IP / MÁSCARA:</text>
                <text x="0" y="62" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="12">[ Ej: "192.168.1.15 / 24" ]</text>

                <text x="0" y="88" fill="{C_TEXT_MUTED}" font-family="{FONT}" font-size="10" font-weight="700">DIRECCIÓN MAC:</text>
                <text x="0" y="106" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="12">[ Ej: "02:42:ac:11:00:0f" ]</text>

                <text x="0" y="132" fill="{C_TEXT_MUTED}" font-family="{FONT}" font-size="10" font-weight="700">VENDOR / PLATAFORMA:</text>
                <text x="0" y="150" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="12">[ Ej: "GNS3 Alpine Linux Container" ]</text>

                <text x="0" y="176" fill="{C_TEXT_MUTED}" font-family="{FONT}" font-size="10" font-weight="700">PROTOCOLO ADMINISTRATIVO:</text>
                <text x="0" y="194" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="12">[ Ej: "SSH (Port 22) / NETCONF" ]</text>
              </g>

              <!-- Botones de Acción de Nodo -->
              <g transform="translate(0, 360)">
                {draw_btn(0, 0, 340, 38, "[ Botón: Aislar Nodo vía RADIUS CoA ]", "Envía UDP 3799 a switch")}
                {draw_btn(0, 50, 340, 38, "[ Botón: Bloquear IP en iptables ]", "Añade regla DROP")}
                {draw_btn(0, 100, 340, 38, "[ Botón: Iniciar Sniffer en eth0 ]", "Captura PCAP en vivo")}
              </g>
            </g>
          </g>
        </g>"""
    elif viewport["mode"] == "tablet":
        svg += render_header_tablet("Topology")
        svg += f"""
        <g transform="translate(20, 100)">
          <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="14" font-weight="800">TOPOLOGÍA EN TABLET (ActiveTab::Topology)</text>
          {draw_placeholder_img(0, 20, 984, 450, "Visualizador de Nodos GNS3 en Tablet")}
          <g transform="translate(0, 490)">
            <rect width="984" height="150" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
            <text x="20" y="28" fill="{C_TEXT}" font-family="{FONT}" font-size="12" font-weight="700">Nodo Seleccionado: [Ej: VictimeAlpine5 (192.168.1.15)]</text>
            {draw_btn(20, 50, 280, 40, "[ Aislar Host ]", "Directiva RADIUS CoA")}
            {draw_btn(320, 50, 280, 40, "[ Bloquear Atacante ]", "iptables DROP L3")}
          </g>
        </g>"""
    else: # mobile
        svg += render_header_mobile("Topology")
        svg += f"""
        <g transform="translate(16, 70)">
          <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="12" font-weight="800">TOPOLOGÍA MÓVIL (ActiveTab::Topology)</text>
          {draw_placeholder_img(0, 15, 358, 260, "Topología Esquemática Móvil")}
          <!-- Lista de Nodos -->
          <g transform="translate(0, 290)">
            <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="11" font-weight="700">Inventario Rápido de Nodos</text>
            <rect y="10" width="358" height="90" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1"/>
            <text x="12" y="32" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: AttackerKali | 192.168.100.50]</text>
            <text x="12" y="50" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="10">Estado: [ Atacante Identificado ]</text>
            {draw_btn(12, 60, 160, 28, "[ Bloquear ]", "DROP L3")}
          </g>
        </g>
        {render_footer_mobile("Topology")}"""

    svg += svg_footer()
    return svg

# ── PÁGINA 4: ActiveTab::MonitoringCharts (Gráficas de Monitoreo) ────────
def build_page_04_monitoring_charts(viewport):
    w, h = viewport["w"], viewport["h"]
    svg = svg_header(w, h)
    svg += f'<rect width="{w}" height="{h}" fill="{C_BG}"/>'

    if viewport["mode"] == "pc":
        svg += render_header_pc("MonitoringCharts")
        svg += f"""
        <g transform="translate(40, 95)">
          <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="15" font-weight="800">GRÁFICAS DE MONITOREO Y TELEMETRÍA GBDT (ActiveTab::MonitoringCharts)</text>

          <!-- Filtros de Ventana Temporal y Métricas -->
          <g transform="translate(0, 16)">
            <rect width="1360" height="42" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
            <text x="16" y="26" fill="{C_TEXT}" font-family="{FONT}" font-size="11" font-weight="700">VENTANA TEMPORAL:</text>
            {draw_btn(160, 7, 70, 28, "[ 5m ]", "Filtro 5 minutos")}
            {draw_btn(235, 7, 70, 28, "[ 15m ]", "Filtro 15 minutos", True)}
            {draw_btn(310, 7, 70, 28, "[ 1h ]", "Filtro 1 hora")}
            {draw_btn(385, 7, 70, 28, "[ 24h ]", "Filtro 24 horas")}

            <text x="500" y="26" fill="{C_TEXT}" font-family="{FONT}" font-size="11" font-weight="700">MÉTRICA ACTIVA:</text>
            {draw_btn(630, 7, 180, 28, "[ Tasa de Paquetes ]", "pkts/s en tiempo real", True)}
            {draw_btn(820, 7, 190, 28, "[ Confianza LightGBM ]", "Score de anomalía")}
            {draw_btn(1020, 7, 170, 28, "[ Latencia Inferencia ]", "Tiempo en microseg")}
          </g>

          <!-- Gráfico Superior Principal (Placeholder con X) -->
          <g transform="translate(0, 72)">
            {draw_placeholder_img(0, 0, 1360, 320, "Gráfico Series Temporales: Paquetes/segundo vs Umbral de Detección")}
          </g>

          <!-- Dos Paneles Inferiores: Métricas del Modelo + Distribución de Clases -->
          <g transform="translate(0, 410)">
            <!-- Panel 1: Estadísticas GBDT -->
            <g transform="translate(0, 0)">
              <rect width="665" height="340" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
              <text x="20" y="30" fill="{C_TEXT}" font-family="{FONT}" font-size="13" font-weight="700">ESTADÍSTICAS DEL MOTOR DE INFERENCIA RUST LIGHTGBM</text>
              <g transform="translate(20, 50)">
                <text x="0" y="20" fill="{C_TEXT_MUTED}" font-family="{FONT}" font-size="11">MODELO ENTRENADO:</text>
                <text x="260" y="20" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[ Ej: "ModelTrainedLightGBM/model.txt" ]</text>

                <text x="0" y="55" fill="{C_TEXT_MUTED}" font-family="{FONT}" font-size="11">TIEMPO PROMEDIO DE INFERENCIA:</text>
                <text x="260" y="55" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[ Ej: "0.142 milisegundos / flujo" ]</text>

                <text x="0" y="90" fill="{C_TEXT_MUTED}" font-family="{FONT}" font-size="11">FEATURES EVALUADOS POR PAQUETE:</text>
                <text x="260" y="90" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[ Ej: "22 variables (IAT, Flags, WinSize)" ]</text>

                <text x="0" y="125" fill="{C_TEXT_MUTED}" font-family="{FONT}" font-size="11">TASA DE FALSOS POSITIVOS (FPR):</text>
                <text x="260" y="125" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[ Ej: "0.002 (0.2%)" ]</text>
              </g>
            </g>

            <!-- Panel 2: Distribución de Clases de Ataque -->
            <g transform="translate(695, 0)">
              <rect width="665" height="340" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
              <text x="20" y="30" fill="{C_TEXT}" font-family="{FONT}" font-size="13" font-weight="700">DISTRIBUCIÓN DE CLASES DETECTADAS (ÚLTIMAS 24H)</text>
              {draw_placeholder_img(20, 50, 625, 260, "Histograma / Gráfico de Barras de Amenazas por Vector")}
            </g>
          </g>
        </g>"""
    elif viewport["mode"] == "tablet":
        svg += render_header_tablet("MonitoringCharts")
        svg += f"""
        <g transform="translate(20, 100)">
          <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="14" font-weight="800">GRÁFICAS EN TABLET (MonitoringCharts.rs)</text>
          {draw_placeholder_img(0, 20, 984, 280, "Gráfico de Series Temporales Tablet")}
          {draw_placeholder_img(0, 320, 984, 300, "Estadísticas del Clasificador GBDT")}
        </g>"""
    else: # mobile
        svg += render_header_mobile("MonitoringCharts")
        svg += f"""
        <g transform="translate(16, 70)">
          <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="12" font-weight="800">GRÁFICAS MÓVILES (MonitoringCharts)</text>
          {draw_placeholder_img(0, 15, 358, 220, "Gráfico de Tráfico Reducido")}
          {draw_placeholder_img(0, 250, 358, 200, "Métricas GBDT Móvil")}
        </g>
        {render_footer_mobile("MonitoringCharts")}"""

    svg += svg_footer()
    return svg

# ── PÁGINA 5: ActiveTab::RealtimeStream (Flujo en Vivo) ───────────────────
def build_page_05_realtime_stream(viewport):
    w, h = viewport["w"], viewport["h"]
    svg = svg_header(w, h)
    svg += f'<rect width="{w}" height="{h}" fill="{C_BG}"/>'

    if viewport["mode"] == "pc":
        svg += render_header_pc("RealtimeStream")
        svg += f"""
        <g transform="translate(40, 95)">
          <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="15" font-weight="800">FLUJO CONTINUO DE EVENTOS FORENSES EN VIVO (ActiveTab::RealtimeStream / RealtimeAlerts.rs)</text>

          <!-- Barra de Control del Stream WebSocket / SSE -->
          <g transform="translate(0, 16)">
            <rect width="1360" height="44" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
            <text x="16" y="27" fill="{C_TEXT}" font-family="{FONT}" font-size="11" font-weight="700">ESTADO DEL FEED EN VIVO:</text>
            <circle cx="210" cy="22" r="5" fill="{C_BORDER}"/>
            <text x="224" y="26" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10">[ WebSocket /api/v1/stream: ACTIVO ]</text>
            
            {draw_btn(460, 8, 140, 28, "[ Pausar Feed ]", "Detiene auto-scroll")}
            {draw_btn(610, 8, 140, 28, "[ Limpiar Eventos ]", "Vacía cola de memoria")}
            {draw_btn(760, 8, 170, 28, "[ Exportar PCAP Stream ]", "Descarga volcado .pcap")}
            {draw_btn(940, 8, 160, 28, "[ Filtrar por IP... ]", "Aplica regex")}
          </g>

          <!-- Terminal de Streaming de Eventos Forenses -->
          <g transform="translate(0, 75)">
            <rect width="1360" height="670" fill="{C_BG}" stroke="{C_BORDER}" stroke-width="2"/>
            <rect width="1360" height="32" fill="{C_PANEL_ALT}" stroke="{C_BORDER}" stroke-width="1"/>
            <text x="20" y="21" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11" font-weight="700">CONSOLA EN TIEMPO REAL: EVENTOS DE RED L2 - L7 (Nivel milisegundos)</text>

            <!-- Eventos Renderizados en Streaming -->
            <g transform="translate(20, 50)" font-family="{FONT_MONO}" font-size="11">
              <!-- Evento 1 -->
              <text x="0" y="20" fill="{C_TEXT}">[16:54:12.829] [PKT_INSPECT] IFACE=eth0 PROTO=TCP(6) SRC=192.168.100.50:48291 DST=192.168.1.15:80 FLAGS=[SYN] LEN=60</text>
              <text x="20" y="38" fill="{C_TEXT_ACTION}">-&gt; [ML_EVAL: LightGBM] Vector=DDOS_SYN_FLOOD Score=0.992 ActionRequired=YES</text>

              <line x1="0" y1="52" x2="1320" y2="52" stroke="{C_BORDER_LIGHT}" stroke-width="0.5" stroke-dasharray="2"/>

              <!-- Evento 2 -->
              <text x="0" y="75" fill="{C_TEXT}">[16:54:12.831] [PKT_INSPECT] IFACE=eth0 PROTO=TCP(6) SRC=192.168.100.50:48292 DST=192.168.1.15:80 FLAGS=[SYN] LEN=60</text>
              <text x="20" y="93" fill="{C_TEXT_ACTION}">-&gt; [ML_EVAL: LightGBM] Alerta acumulada ALT-001 elevada a SEVERITY_5</text>

              <line x1="0" y1="107" x2="1320" y2="107" stroke="{C_BORDER_LIGHT}" stroke-width="0.5" stroke-dasharray="2"/>

              <!-- Evento 3 -->
              <text x="0" y="130" fill="{C_TEXT}">[16:54:13.104] [DNS_QUERY] SRC=192.168.1.12:53210 DST=208.67.222.222:53 QUERY="exfil.attacker.tigo.net" TYPE=TXT</text>
              <text x="20" y="148" fill="{C_TEXT_ACTION}">-&gt; [ML_EVAL: LightGBM] Entropía de subdominio alta (4.82) -&gt; DNS_C2_EXFILTRATION (0.978)</text>

              <line x1="0" y1="162" x2="1320" y2="162" stroke="{C_BORDER_LIGHT}" stroke-width="0.5" stroke-dasharray="2"/>

              <!-- Evento 4 -->
              <text x="0" y="185" fill="{C_TEXT}">[16:54:14.002] [RADIUS_COA] Despachando UDP 3799 hacia 192.168.1.254 (MikroTik RouterOS Core)</text>
              <text x="20" y="203" fill="{C_TEXT_ACTION}">-&gt; [STATUS] Disconnect-Request enviado para MAC 00:00:0c:aa:bb:cc</text>
            </g>

            <!-- Placeholder de Puntero de Stream -->
            <g transform="translate(20, 610)">
              <rect width="1320" height="40" fill="{C_PANEL_ALT}" stroke="{C_BORDER}" stroke-width="1"/>
              <text x="20" y="25" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="11">[ Placeholder Stream Cursor: Escuchando nuevos paquetes... Auto-scroll activado ]</text>
            </g>
          </g>
        </g>"""
    elif viewport["mode"] == "tablet":
        svg += render_header_tablet("RealtimeStream")
        svg += f"""
        <g transform="translate(20, 100)">
          <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="14" font-weight="800">FLUJO EN VIVO - VISTA TABLET (RealtimeAlerts.rs)</text>
          <rect y="15" width="984" height="600" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
          <text x="20" y="50" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="11">[ Placeholder: Streaming de Eventos en Tablet con controles táctiles ]</text>
        </g>"""
    else: # mobile
        svg += render_header_mobile("RealtimeStream")
        svg += f"""
        <g transform="translate(16, 70)">
          <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="12" font-weight="800">FLUJO EN VIVO MÓVIL</text>
          <rect y="15" width="358" height="650" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
          <text x="12" y="45" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="9">[ Placeholder: Stream móvil con swipe vertical ]</text>
        </g>
        {render_footer_mobile("RealtimeStream")}"""

    svg += svg_footer()
    return svg

# ── PÁGINA 6: ActiveTab::Alerts (Alertas Forenses) ────────────────────────
def build_page_06_alerts(viewport):
    w, h = viewport["w"], viewport["h"]
    svg = svg_header(w, h)
    svg += f'<rect width="{w}" height="{h}" fill="{C_BG}"/>'

    if viewport["mode"] == "pc":
        svg += render_header_pc("Alerts")
        svg += f"""
        <g transform="translate(40, 95)">
          <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="15" font-weight="800">MONITOREO FORENSE DE ALERTAS (ActiveTab::Alerts / AlertsTable.rs)</text>

          <!-- Barra de Filtros y Búsqueda -->
          <g transform="translate(0, 16)">
            <rect width="1360" height="48" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
            <text x="16" y="30" fill="{C_TEXT}" font-family="{FONT}" font-size="11" font-weight="700">FILTRAR SEVERIDAD:</text>
            {draw_btn(170, 9, 80, 30, "[ Todos ]", "Todas las alertas", True)}
            {draw_btn(255, 9, 100, 30, "[ Crítica (5) ]", "Severidad 5")}
            {draw_btn(360, 9, 90, 30, "[ Alta (4) ]", "Severidad 4")}
            {draw_btn(455, 9, 90, 30, "[ Media (3) ]", "Severidad 3")}
            
            <!-- Campo de Búsqueda -->
            <rect x="580" y="9" width="350" height="30" fill="{C_BG}" stroke="{C_BORDER}" stroke-width="1"/>
            <text x="590" y="28" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="10">[ Placeholder Input: "Buscar por IP / Host / Tipo..." ]</text>
            
            {draw_btn(940, 9, 140, 30, "[ Exportar CSV ]", "Descarga reporte")}
            {draw_btn(1090, 9, 140, 30, "[ Exportar PDF ]", "Genera informe forense")}
          </g>

          <!-- Tabla Completa de Alertas -->
          <g transform="translate(0, 80)">
            <rect width="1360" height="665" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="2"/>
            
            <!-- Encabezados -->
            <g transform="translate(20, 20)">
              <rect width="1320" height="34" fill="{C_PANEL_ALT}" stroke="{C_BORDER}" stroke-width="1"/>
              <text x="15" y="22" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">ID ALERTA</text>
              <text x="120" y="22" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">HORA REGISTRO</text>
              <text x="240" y="22" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">SEVERIDAD</text>
              <text x="360" y="22" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">TIPO DE AMENAZA</text>
              <text x="550" y="22" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">ORIGEN (ATACANTE)</text>
              <text x="760" y="22" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">DESTINO (VÍCTIMA)</text>
              <text x="960" y="22" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">ESTADO</text>
              <text x="1110" y="22" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">ACCIONES OPERATIVAS</text>

              <!-- Fila 1 -->
              <g transform="translate(0, 40)">
                <rect width="1320" height="50" fill="{C_BG}" stroke="{C_BORDER_LIGHT}" stroke-width="1"/>
                <text x="15" y="30" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: ALT-2026-001]</text>
                <text x="120" y="30" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="11">[Ej: 16:54:12]</text>
                <text x="240" y="30" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: CRITICAL]</text>
                <text x="360" y="30" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: DDOS_SYN_FLOOD]</text>
                <text x="550" y="30" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: 192.168.100.50]</text>
                <text x="760" y="30" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: 192.168.1.15 : 80]</text>
                <text x="960" y="30" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: PENDING_MANUAL]</text>
                {draw_btn(1110, 8, 95, 32, "[ Mitigar ]", "Abre Blast Radius")}
                {draw_btn(1215, 8, 95, 32, "[ Analizar ]", "Triaje forense L4")}
              </g>

              <!-- Fila 2 -->
              <g transform="translate(0, 96)">
                <rect width="1320" height="50" fill="{C_BG}" stroke="{C_BORDER_LIGHT}" stroke-width="1"/>
                <text x="15" y="30" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: ALT-2026-002]</text>
                <text x="120" y="30" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="11">[Ej: 16:52:05]</text>
                <text x="240" y="30" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: HIGH]</text>
                <text x="360" y="30" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: DNS_C2_EXFILTRATION]</text>
                <text x="550" y="30" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: 192.168.1.12]</text>
                <text x="760" y="30" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: 208.67.222.222:53]</text>
                <text x="960" y="30" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: CONFIRMED]</text>
                {draw_btn(1110, 8, 95, 32, "[ Mitigar ]", "Despacha Sinkhole")}
                {draw_btn(1215, 8, 95, 32, "[ Analizar ]", "Detalle DNS")}
              </g>
            </g>
          </g>
        </g>"""
    elif viewport["mode"] == "tablet":
        svg += render_header_tablet("Alerts")
        svg += f"""
        <g transform="translate(20, 100)">
          <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="14" font-weight="800">TABLA DE ALERTAS EN TABLET</text>
          <rect y="20" width="984" height="600" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
          <text x="20" y="55" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="11">[ Placeholder: Tabla responsiva con columnas colapsables en Tablet ]</text>
        </g>"""
    else: # mobile
        svg += render_header_mobile("Alerts")
        svg += f"""
        <g transform="translate(16, 70)">
          <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="12" font-weight="800">ALERTAS FORENSES MÓVIL</text>
          <rect y="15" width="358" height="650" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
          <text x="12" y="45" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="9">[ Placeholder: Tarjetas verticales con botón de triaje rápido ]</text>
        </g>
        {render_footer_mobile("Alerts")}"""

    svg += svg_footer()
    return svg

# ── PÁGINA 7: ActiveTab::BlastRadius (Blast Radius & Mitigación) ──────────
def build_page_07_blast_radius(viewport):
    w, h = viewport["w"], viewport["h"]
    svg = svg_header(w, h)
    svg += f'<rect width="{w}" height="{h}" fill="{C_BG}"/>'

    if viewport["mode"] == "pc":
        svg += render_header_pc("BlastRadius")
        svg += f"""
        <g transform="translate(40, 95)">
          <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="15" font-weight="800">CONTROL DE BLAST RADIUS Y RETENCIÓN DE MITIGACIONES (ActiveTab::BlastRadius / BlastRadiusControl.rs)</text>

          <!-- Aviso de Política de Seguridad Crítica -->
          <g transform="translate(0, 16)">
            <rect width="1360" height="74" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="2"/>
            <text x="20" y="26" fill="{C_TEXT}" font-family="{FONT}" font-size="12" font-weight="700">POLÍTICA ZERO-TRUST: MITIGACIONES RETENIDAS POR RADIO DE IMPACTO (PENDING_MANUAL)</text>
            <text x="20" y="46" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="11">Acciones que afectan subredes críticas o aíslan switches requieren autorización explícita de operador RBAC.</text>
            <text x="20" y="62" fill="{C_TEXT_ACTION}" font-family="{FONT_MONO}" font-size="10">&lt;Parámetro: critical_action_timeout_secs = 300 (Ventana de re-autenticación de 5 minutos)&gt;</text>
          </g>

          <!-- Tabla de Mitigaciones Pendientes de Autorización -->
          <g transform="translate(0, 105)">
            <rect width="1360" height="280" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
            <text x="20" y="28" fill="{C_TEXT}" font-family="{FONT}" font-size="13" font-weight="700">COLA DE ACCIONES PENDIENTES DE APROBACIÓN</text>

            <g transform="translate(20, 45)">
              <rect width="1320" height="30" fill="{C_PANEL_ALT}" stroke="{C_BORDER}" stroke-width="1"/>
              <text x="15" y="20" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">ID TAREA</text>
              <text x="120" y="20" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">DIRECTIVA</text>
              <text x="320" y="20" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">DESTINO (TARGET)</text>
              <text x="540" y="20" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">DISPOSITIVO VENDOR</text>
              <text x="760" y="20" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">IMPACTO ESTIMADO</text>
              <text x="960" y="20" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">ACCIONES DISPONIBLES</text>

              <!-- Fila Pendiente 1 -->
              <g transform="translate(0, 36)">
                <rect width="1320" height="48" fill="{C_BG}" stroke="{C_BORDER_LIGHT}" stroke-width="1"/>
                <text x="15" y="28" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: TSK-0891]</text>
                <text x="120" y="28" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: ACL_BLACKHOLE_L3_L4]</text>
                <text x="320" y="28" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: 192.168.100.50]</text>
                <text x="540" y="28" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: MikroTik RouterOS / iptables]</text>
                <text x="760" y="28" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: 1 Host (Atacante Aislado)]</text>
                {draw_btn(960, 8, 160, 30, "[ Aprobar y Despachar ]", "Abre modal contraseña")}
                {draw_btn(1130, 8, 140, 30, "[ Descartar Acción ]", "Cancela mitigación")}
              </g>
            </g>
          </g>

          <!-- Modal Superpuesto de Re-autenticación (Elemento <dialog> Wireframe) -->
          <g transform="translate(360, 410)">
            <rect width="640" height="320" fill="{C_BG}" stroke="{C_BORDER}" stroke-width="2.5"/>
            <rect width="640" height="38" fill="{C_PANEL_ALT}" stroke="{C_BORDER}" stroke-width="1"/>
            <text x="20" y="25" fill="{C_TEXT}" font-family="{FONT}" font-size="13" font-weight="800">MODAL: RE-AUTENTICACIÓN REQUERIDA (Timeout Expirado: 0s restantes)</text>

            <g transform="translate(30, 60)">
              <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="11">Para despachar directivas perimetrales al ISP, ingrese su contraseña de usuario:</text>
              {draw_input(0, 20, 580, 42, "CONTRASEÑA DEL OPERADOR (admin_soc):", "••••••••••••", "admin")}
              
              <g transform="translate(0, 110)">
                <text x="0" y="0" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="10">[ Comando a inyectar: iptables -I DOCKER-USER -s 192.168.100.50 -j DROP ]</text>
              </g>

              <g transform="translate(0, 150)">
                {draw_btn(0, 0, 260, 44, "[ Cancelar Acción (Esc) ]", "Cierra diálogo")}
                {draw_btn(280, 0, 300, 44, "[ Confirmar Mitigación ]", "Valida pass y ejecuta", True)}
              </g>
            </g>
          </g>
        </g>"""
    elif viewport["mode"] == "tablet":
        svg += render_header_tablet("BlastRadius")
        svg += f"""
        <g transform="translate(20, 100)">
          <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="14" font-weight="800">CONTROL DE MITIGACIONES EN TABLET (BlastRadiusControl.rs)</text>
          <rect y="20" width="984" height="600" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
          <text x="20" y="60" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="11">[ Placeholder: Cola de aprobación y modal táctil de contraseña ]</text>
        </g>"""
    else: # mobile
        svg += render_header_mobile("BlastRadius")
        svg += f"""
        <g transform="translate(16, 70)">
          <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="12" font-weight="800">BLAST RADIUS MÓVIL (Bottom Sheet Modal)</text>
          <rect y="15" width="358" height="650" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
          <text x="12" y="45" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="9">[ Placeholder: Bottom Sheet con teclado numérico para PIN/Pass ]</text>
        </g>
        {render_footer_mobile("BlastRadius")}"""

    svg += svg_footer()
    return svg

# ── PÁGINA 8: ActiveTab::UserManagement (Usuarios y RBAC) ────────────────
def build_page_08_user_management(viewport):
    w, h = viewport["w"], viewport["h"]
    svg = svg_header(w, h)
    svg += f'<rect width="{w}" height="{h}" fill="{C_BG}"/>'

    if viewport["mode"] == "pc":
        svg += render_header_pc("UserManagement")
        svg += f"""
        <g transform="translate(40, 95)">
          <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="15" font-weight="800">ADMINISTRACIÓN DE USUARIOS Y CONTROL DE ACCESO RBAC (ActiveTab::UserManagement / UserManagement.rs)</text>

          <!-- Formulario de Creación de Usuario a la Izquierda (450px) -->
          <g transform="translate(0, 20)">
            <rect width="460" height="730" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="2"/>
            <text x="24" y="32" fill="{C_TEXT}" font-family="{FONT}" font-size="13" font-weight="700">CREAR / EDITAR OPERADOR RBAC</text>
            <line x1="24" y1="46" x2="436" y2="46" stroke="{C_BORDER}" stroke-width="1"/>

            <g transform="translate(24, 65)">
              {draw_input(0, 0, 412, 40, "NOMBRE DE USUARIO:", "analista_triaje", "analista_triaje")}
              {draw_input(0, 75, 412, 40, "CONTRASEÑA INICIAL:", "••••••••••••", "admin")}
              
              <!-- Selector de Rol -->
              <text x="0" y="150" fill="{C_TEXT}" font-family="{FONT}" font-size="11" font-weight="700">ROL ASIGNADO (PERMISOS):</text>
              <rect y="158" width="412" height="40" fill="{C_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
              <text x="12" y="183" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[ Dropdown: SOC_OPERATOR (Mitigación &amp; Rollback) v ]</text>
              <text x="206" y="212" fill="{C_TEXT_ACTION}" font-family="{FONT_MONO}" font-size="8.5" text-anchor="middle">&lt;Roles: Admin | Operador | Auditor | AnalistaL1&gt;</text>

              {draw_btn(0, 230, 412, 44, "[ Botón: GUARDAR USUARIO ]", "Envía POST /api/v1/users/create", True)}
            </g>

            <!-- Configuración Global de Parámetros de Seguridad -->
            <g transform="translate(24, 380)">
              <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="12" font-weight="700">PARÁMETROS GLOBALES DE SEGURIDAD (Zero-Trust)</text>
              <rect y="12" width="412" height="180" fill="{C_BG}" stroke="{C_BORDER_LIGHT}" stroke-width="1.5"/>
              
              <g transform="translate(16, 36)">
                <text x="0" y="0" fill="{C_TEXT_MUTED}" font-family="{FONT}" font-size="10" font-weight="700">TIMEOUT ACCIÓN CRÍTICA (SEGUNDOS):</text>
                <rect y="8" width="380" height="34" fill="{C_PANEL_ALT}" stroke="{C_BORDER}" stroke-width="1"/>
                <text x="12" y="30" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="12">[ Input: "300" (5 minutos) ]</text>

                <text x="0" y="65" fill="{C_TEXT_MUTED}" font-family="{FONT}" font-size="10" font-weight="700">EXPIRACIÓN INACTIVIDAD SESIÓN:</text>
                <rect y="73" width="380" height="34" fill="{C_PANEL_ALT}" stroke="{C_BORDER}" stroke-width="1"/>
                <text x="12" y="95" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="12">[ Input: "900" (15 minutos) ]</text>
              </g>

              {draw_btn(0, 205, 412, 38, "[ Actualizar Parámetros ]", "Guarda en tabla system_parameters")}
            </g>
          </g>

          <!-- Tabla de Usuarios Registrados a la Derecha (880px) -->
          <g transform="translate(480, 20)">
            <rect width="880" height="730" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="2"/>
            <text x="24" y="32" fill="{C_TEXT}" font-family="{FONT}" font-size="13" font-weight="700">CATÁLOGO DE OPERADORES Y AUDITORÍA RBAC</text>

            <g transform="translate(24, 55)">
              <rect width="832" height="30" fill="{C_PANEL_ALT}" stroke="{C_BORDER}" stroke-width="1"/>
              <text x="15" y="20" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">ID</text>
              <text x="60" y="20" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">USUARIO</text>
              <text x="220" y="20" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">ROL ASIGNADO</text>
              <text x="440" y="20" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">PERMISOS ACTIVOS</text>
              <text x="680" y="20" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="10" font-weight="700">GESTIÓN</text>

              <!-- Usuario 1 -->
              <g transform="translate(0, 36)">
                <rect width="832" height="48" fill="{C_BG}" stroke="{C_BORDER_LIGHT}" stroke-width="1"/>
                <text x="15" y="28" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[1]</text>
                <text x="60" y="28" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: admin_soc]</text>
                <text x="220" y="28" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: SOC_ADMIN]</text>
                <text x="440" y="28" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="10">Acceso Total / Bypass</text>
                {draw_btn(680, 8, 70, 30, "[ Editar ]", "Cambia rol")}
                {draw_btn(755, 8, 70, 30, "[ Reset ]", "Nueva clave")}
              </g>

              <!-- Usuario 2 -->
              <g transform="translate(0, 90)">
                <rect width="832" height="48" fill="{C_BG}" stroke="{C_BORDER_LIGHT}" stroke-width="1"/>
                <text x="15" y="28" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[2]</text>
                <text x="60" y="28" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: operador_red]</text>
                <text x="220" y="28" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: SOC_OPERATOR]</text>
                <text x="440" y="28" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="10">Mitigación / Rollback</text>
                {draw_btn(680, 8, 70, 30, "[ Editar ]", "Cambia rol")}
                {draw_btn(755, 8, 70, 30, "[ Reset ]", "Nueva clave")}
              </g>

              <!-- Usuario 3 -->
              <g transform="translate(0, 144)">
                <rect width="832" height="48" fill="{C_BG}" stroke="{C_BORDER_LIGHT}" stroke-width="1"/>
                <text x="15" y="28" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[3]</text>
                <text x="60" y="28" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: auditor_seguridad]</text>
                <text x="220" y="28" fill="{C_TEXT}" font-family="{FONT_MONO}" font-size="11">[Ej: SOC_AUDITOR]</text>
                <text x="440" y="28" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="10">Solo Lectura Forense</text>
                {draw_btn(680, 8, 70, 30, "[ Editar ]", "Cambia rol")}
                {draw_btn(755, 8, 70, 30, "[ Reset ]", "Nueva clave")}
              </g>
            </g>
          </g>
        </g>"""
    elif viewport["mode"] == "tablet":
        svg += render_header_tablet("UserManagement")
        svg += f"""
        <g transform="translate(20, 100)">
          <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="14" font-weight="800">GESTIÓN DE USUARIOS EN TABLET (UserManagement.rs)</text>
          <rect y="20" width="984" height="600" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
          <text x="20" y="55" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="11">[ Placeholder: Tabla de usuarios y formulario modal en tablet ]</text>
        </g>"""
    else: # mobile
        svg += render_header_mobile("UserManagement")
        svg += f"""
        <g transform="translate(16, 70)">
          <text x="0" y="0" fill="{C_TEXT}" font-family="{FONT}" font-size="12" font-weight="800">USUARIOS &amp; RBAC MÓVIL</text>
          <rect y="15" width="358" height="650" fill="{C_PANEL_BG}" stroke="{C_BORDER}" stroke-width="1.5"/>
          <text x="12" y="45" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="9">[ Placeholder: Tarjetas de operadores y permisos móviles ]</text>
        </g>
        {render_footer_mobile("UserManagement")}"""

    svg += svg_footer()
    return svg

# ═════════════════════════════════════════════════════════════════════════
# ORQUESTADOR PRINCIPAL
# ═════════════════════════════════════════════════════════════════════════

VIEWPORTS = {
    "pc": {"mode": "pc", "w": 1440, "h": 900},
    "tablet": {"mode": "tablet", "w": 1024, "h": 768},
    "celular": {"mode": "celular", "w": 390, "h": 844},
}

PAGES = [
    ("01_login", build_page_01_login, "Pantalla de Inicio de Sesión (LoginView.rs)"),
    ("02_dashboard", build_page_02_dashboard, "Panel de Control Principal (SystemStatus + MonitoringChart + AlertsTable)"),
    ("03_topology", build_page_03_topology, "Topología de Red 3D (Topology3D.rs)"),
    ("04_monitoring_charts", build_page_04_monitoring_charts, "Gráficas de Telemetría GBDT (MonitoringCharts.rs)"),
    ("05_realtime_stream", build_page_05_realtime_stream, "Flujo Continuo en Vivo (RealtimeAlerts.rs)"),
    ("06_alerts", build_page_06_alerts, "Monitoreo Forense de Alertas (AlertsTable.rs)"),
    ("07_blast_radius", build_page_07_blast_radius, "Control de Blast Radius y Re-autenticación (BlastRadiusControl.rs)"),
    ("08_user_management", build_page_08_user_management, "Administración de Usuarios y RBAC (UserManagement.rs)"),
]

def main():
    print(f"Generando 24 Wireframes / Mockups en Tema Claro en: {FINAL_DIR}")
    
    generated_pdfs = []

    for page_key, builder_fn, page_title in PAGES:
        for vp_key, vp_info in VIEWPORTS.items():
            base_name = f"wireframe_{page_key}_{vp_key}"
            svg_path = os.path.join(FINAL_DIR, f"{base_name}.svg")
            png_path = os.path.join(FINAL_DIR, f"{base_name}.png")
            pdf_path = os.path.join(FINAL_DIR, f"{base_name}.pdf")

            svg_content = builder_fn(vp_info)
            with open(svg_path, "w", encoding="utf-8") as f:
                f.write(svg_content)

            # Convertir a PNG (alta resolución)
            subprocess.run(f"rsvg-convert -f png -d 150 -p 150 -o '{png_path}' '{svg_path}'", shell=True, check=True)
            # Convertir a PDF individual
            subprocess.run(f"rsvg-convert -f pdf -o '{pdf_path}' '{svg_path}'", shell=True, check=True)

            generated_pdfs.append(pdf_path)
            print(f"  [OK] Generado: {base_name} ({vp_key.upper()})")

    # Compilar Libro Maestro de Wireframes PDF
    master_pdf = os.path.join(FINAL_DIR, "COMPILACION_MAESTRA_WIREFRAMES_TEMA_CLARO.pdf")
    writer = PdfWriter()
    for p in generated_pdfs:
        writer.append(p)
    with open(master_pdf, "wb") as f_out:
        writer.write(f_out)
    print(f"\nDocumento Maestro Compilado (24 pantallas): {master_pdf}")

if __name__ == "__main__":
    main()
