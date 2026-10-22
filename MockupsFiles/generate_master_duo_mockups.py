#!/usr/bin/env python3
"""
Generador Maestro de Mockups Profesionales de Alta Fidelidad - Tigo SOC Platform
Cumple estrictamente con:
1. Requisitos del Usuario:
   - Mockups de alta fidelidad visual (no wireframes esquemáticos).
   - Cero Emojis (uso exclusivo de iconos vectoriales SVG limpios y texto técnico).
   - Seguridad Reforzada en Autenticación:
     * Login con Correo Corporativo (SSO), Contraseña y 2FA/TOTP de 6 dígitos.
     * Protección contra Fuerza Bruta con Rate-Limiting visible y TLS 1.3.
     * Eliminación del cambio arbitrario de cuenta en Header (Sesión fija, rol badge, Logout explícito).
     * Pantalla de Cambio Obligatorio de Contraseña por defecto en el primer acceso (medidor de entropía, requisitos ISO 27001).
   - Pestaña de Sandbox ACL / Reglas Dinámicas:
     * Subred alterna aislada (10.200.0.0/24 / VLAN 999).
     * Inyección de vectores de ataque simulados (SYN flood, UDP reflection).
     * Consola de ejecución de reglas dinámicas eBPF/XDP e iptables.
     * Telemetría de verificación: Tasa de mitigación, 0.00% falsos positivos, impacto en latencia.
     * Flujo de gobernanza: Probar en Sandbox -> Descartar o Promover a Producción.
   - Generación paralela en Tema Claro y Tema Oscuro con armonización cromática estricta.
   - Vistas para PC (1440x900), Tablet (1024x768) y Celular (390x844).
2. Directivas Anti-Patrones UI/UX de IA:
   - Eliminación del Cardocalypse y glassmorphism injustificado.
   - Radios concéntricos matemáticos: r_inner = max(0, r_outer - padding).
   - Tipografía sans-serif con números tabulares para telemetría de red.
   - Paletas sobrias basadas en ciberseguridad ISP (Slate / Tigo Blue / Emerald / Crimson).
"""

import os
import sys
import subprocess
from pypdf import PdfWriter, PdfReader

BASE_DIR = os.path.dirname(os.path.abspath(__file__))
FINAL_DIR = os.path.join(BASE_DIR, "Final")
LIGHT_DIR = os.path.join(FINAL_DIR, "light")
DARK_DIR = os.path.join(FINAL_DIR, "dark")

# Directorios para compilaciones individuales (se crean solo si se ejecutan directamente)

# -------------------------------------------------------------------------
# PALETAS ARMONIZADAS DE DISEÑO (LIGHT & DARK THEMES)
# -------------------------------------------------------------------------
THEMES = {
    "light": {
        "name": "Tema Claro",
        "bg_page": "#F8FAFC",       # Slate 50
        "bg_panel": "#FFFFFF",      # Blanco puro
        "bg_subtle": "#F1F5F9",     # Slate 100
        "bg_active": "#E2E8F0",     # Slate 200
        "border": "#CBD5E1",        # Slate 300
        "border_subtle": "#E2E8F0", # Slate 200
        "text_primary": "#0F172A",  # Slate 900
        "text_secondary": "#475569",# Slate 600
        "text_muted": "#64748B",    # Slate 500
        "accent": "#0284C7",        # Tigo Sky Blue (700/600)
        "accent_bg": "#E0F2FE",     # Sky 100
        "accent_text": "#0369A1",   # Sky 700
        "accent_hover": "#0369A1",
        "critical": "#DC2626",      # Red 600
        "critical_bg": "#FEF2F2",   # Red 50
        "critical_text": "#991B1B", # Red 800
        "critical_border": "#FECACA",
        "warning": "#D97706",       # Amber 600
        "warning_bg": "#FFFBEB",    # Amber 50
        "warning_text": "#92400E",  # Amber 800
        "warning_border": "#FDE68A",
        "success": "#16A34A",       # Green 600
        "success_bg": "#F0FDF4",    # Green 50
        "success_text": "#166534",  # Green 800
        "success_border": "#BBF7D0",
        "sandbox": "#4F46E5",       # Indigo 600
        "sandbox_bg": "#EEF2FF",    # Indigo 50
        "sandbox_text": "#3730A3",  # Indigo 800
        "sandbox_border": "#C7D2FE",
        "code_bg": "#0F172A",
        "code_text": "#38BDF8",
        "card_shadow": "rgba(15, 23, 42, 0.05)",
    },
    "dark": {
        "name": "Tema Oscuro",
        "bg_page": "#0B0F17",       # Obsidian Navy
        "bg_panel": "#131A29",      # Midnight Slate
        "bg_subtle": "#1B2436",     # Slate 850
        "bg_active": "#24324D",     # Active panel
        "border": "#27354F",        # Slate border
        "border_subtle": "#1B2436",
        "text_primary": "#F8FAFC",  # Slate 50
        "text_secondary": "#94A3B8",# Slate 400
        "text_muted": "#64748B",    # Slate 500
        "accent": "#38BDF8",        # Sky 400
        "accent_bg": "#0C4A6E",     # Sky 900
        "accent_text": "#BAE6FD",   # Sky 200
        "accent_hover": "#0284C7",
        "critical": "#EF4444",      # Red 500
        "critical_bg": "#450A0A",   # Red 950
        "critical_text": "#FCA5A5", # Red 300
        "critical_border": "#7F1D1D",
        "warning": "#F59E0B",       # Amber 500
        "warning_bg": "#451A03",    # Amber 950
        "warning_text": "#FDE68A",  # Amber 200
        "warning_border": "#78350F",
        "success": "#22C55E",       # Green 500
        "success_bg": "#052E16",    # Green 950
        "success_text": "#86EFAC",  # Green 300
        "success_border": "#14532D",
        "sandbox": "#818CF8",       # Indigo 400
        "sandbox_bg": "#1E1B4B",    # Indigo 950
        "sandbox_text": "#C7D2FE",  # Indigo 200
        "sandbox_border": "#3730A3",
        "code_bg": "#060A10",
        "code_text": "#38BDF8",
        "card_shadow": "rgba(0, 0, 0, 0.4)",
    }
}

FONT_SANS = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif"
FONT_MONO = "'SF Mono', 'Fira Code', 'Roboto Mono', Menlo, Consolas, monospace"

def esc(text):
    """Escapa caracteres para XML/SVG de forma segura"""
    if text is None:
        return ""
    return str(text).replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;").replace('"', "&quot;")

# -------------------------------------------------------------------------
# ICONOS VECTORIALES SVG LIMPIOS (CERO EMOJIS)
# -------------------------------------------------------------------------
def icon_shield(x, y, size=16, color="#0284C7"):
    return f'''<g transform="translate({x},{y})">
        <path d="M{size/2} {size*0.05} L{size*0.9} {size*0.2} C{size*0.9} {size*0.65} {size/2} {size*0.95} {size/2} {size*0.95} C{size/2} {size*0.95} {size*0.1} {size*0.65} {size*0.1} {size*0.2} Z" fill="none" stroke="{color}" stroke-width="1.6" stroke-linejoin="round"/>
        <path d="M{size*0.35} {size*0.5} L{size*0.48} {size*0.65} L{size*0.7} {size*0.35}" fill="none" stroke="{color}" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/>
    </g>'''

def icon_lock(x, y, size=16, color="#64748B"):
    return f'''<g transform="translate({x},{y})">
        <rect x="{size*0.2}" y="{size*0.45}" width="{size*0.6}" height="{size*0.48}" rx="2" fill="none" stroke="{color}" stroke-width="1.6"/>
        <path d="M{size*0.32} {size*0.45} V{size*0.3} C{size*0.32} {size*0.18} {size*0.68} {size*0.18} {size*0.68} {size*0.3} V{size*0.45}" fill="none" stroke="{color}" stroke-width="1.6" stroke-linecap="round"/>
        <circle cx="{size/2}" cy="{size*0.68}" r="{size*0.08}" fill="{color}"/>
    </g>'''

def icon_check(x, y, size=16, color="#16A34A"):
    return f'''<g transform="translate({x},{y})">
        <circle cx="{size/2}" cy="{size/2}" r="{size*0.45}" fill="none" stroke="{color}" stroke-width="1.5"/>
        <path d="M{size*0.28} {size*0.52} L{size*0.44} {size*0.68} L{size*0.72} {size*0.35}" fill="none" stroke="{color}" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
    </g>'''

def icon_alert(x, y, size=16, color="#DC2626"):
    return f'''<g transform="translate({x},{y})">
        <path d="M{size/2} {size*0.1} L{size*0.92} {size*0.88} H{size*0.08} Z" fill="none" stroke="{color}" stroke-width="1.6" stroke-linejoin="round"/>
        <line x1="{size/2}" y1="{size*0.4}" x2="{size/2}" y2="{size*0.62}" stroke="{color}" stroke-width="1.6" stroke-linecap="round"/>
        <circle cx="{size/2}" cy="{size*0.76}" r="{size*0.07}" fill="{color}"/>
    </g>'''

def icon_network(x, y, size=16, color="#0284C7"):
    return f'''<g transform="translate({x},{y})">
        <rect x="{size*0.35}" y="{size*0.1}" width="{size*0.3}" height="{size*0.25}" rx="1" fill="none" stroke="{color}" stroke-width="1.5"/>
        <rect x="{size*0.08}" y="{size*0.65}" width="{size*0.3}" height="{size*0.25}" rx="1" fill="none" stroke="{color}" stroke-width="1.5"/>
        <rect x="{size*0.62}" y="{size*0.65}" width="{size*0.3}" height="{size*0.25}" rx="1" fill="none" stroke="{color}" stroke-width="1.5"/>
        <line x1="{size/2}" y1="{size*0.35}" x2="{size/2}" y2="{size*0.5}" stroke="{color}" stroke-width="1.4"/>
        <line x1="{size*0.23}" y1="{size*0.5}" x2="{size*0.77}" y2="{size*0.5}" stroke="{color}" stroke-width="1.4"/>
        <line x1="{size*0.23}" y1="{size*0.5}" x2="{size*0.23}" y2="{size*0.65}" stroke="{color}" stroke-width="1.4"/>
        <line x1="{size*0.77}" y1="{size*0.5}" x2="{size*0.77}" y2="{size*0.65}" stroke="{color}" stroke-width="1.4"/>
    </g>'''

def icon_chart(x, y, size=16, color="#0284C7"):
    return f'''<g transform="translate({x},{y})">
        <path d="M{size*0.1} {size*0.85} H{size*0.9}" stroke="{color}" stroke-width="1.5" stroke-linecap="round"/>
        <path d="M{size*0.1} {size*0.15} V{size*0.85}" stroke="{color}" stroke-width="1.5" stroke-linecap="round"/>
        <path d="M{size*0.22} {size*0.65} L{size*0.42} {size*0.42} L{size*0.62} {size*0.55} L{size*0.85} {size*0.25}" fill="none" stroke="{color}" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/>
    </g>'''

def icon_terminal(x, y, size=16, color="#0284C7"):
    return f'''<g transform="translate({x},{y})">
        <rect x="{size*0.08}" y="{size*0.12}" width="{size*0.84}" height="{size*0.76}" rx="2" fill="none" stroke="{color}" stroke-width="1.5"/>
        <path d="M{size*0.25} {size*0.35} L{size*0.45} {size*0.5} L{size*0.25} {size*0.65}" fill="none" stroke="{color}" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        <line x1="{size*0.52}" y1="{size*0.65}" x2="{size*0.75}" y2="{size*0.65}" stroke="{color}" stroke-width="1.6" stroke-linecap="round"/>
    </g>'''

def icon_flask(x, y, size=16, color="#4F46E5"):
    return f'''<g transform="translate({x},{y})">
        <path d="M{size*0.38} {size*0.1} H{size*0.62} M{size/2} {size*0.1} V{size*0.35} L{size*0.15} {size*0.85} C{size*0.12} {size*0.9} {size*0.18} {size*0.95} {size*0.25} {size*0.95} H{size*0.75} C{size*0.82} {size*0.95} {size*0.88} {size*0.9} {size*0.85} {size*0.85} L{size/2} {size*0.35}" fill="none" stroke="{color}" stroke-width="1.5" stroke-linejoin="round"/>
        <line x1="{size*0.28}" y1="{size*0.72}" x2="{size*0.72}" y2="{size*0.72}" stroke="{color}" stroke-width="1.4" stroke-dasharray="2,2"/>
    </g>'''

def icon_user(x, y, size=16, color="#64748B"):
    return f'''<g transform="translate({x},{y})">
        <circle cx="{size/2}" cy="{size*0.35}" r="{size*0.22}" fill="none" stroke="{color}" stroke-width="1.5"/>
        <path d="M{size*0.15} {size*0.88} C{size*0.15} {size*0.65} {size*0.32} {size*0.62} {size/2} {size*0.62} C{size*0.68} {size*0.62} {size*0.85} {size*0.65} {size*0.85} {size*0.88}" fill="none" stroke="{color}" stroke-width="1.5" stroke-linecap="round"/>
    </g>'''

def icon_logout(x, y, size=16, color="#DC2626"):
    return f'''<g transform="translate({x},{y})">
        <path d="M{size*0.55} {size*0.18} H{size*0.22} C{size*0.15} {size*0.18} {size*0.15} {size*0.25} {size*0.15} {size*0.25} V{size*0.75} C{size*0.15} {size*0.82} {size*0.22} {size*0.82} {size*0.22} {size*0.82} H{size*0.55}" fill="none" stroke="{color}" stroke-width="1.5" stroke-linecap="round"/>
        <path d="M{size*0.48} {size/2} H{size*0.88} M{size*0.72} {size*0.32} L{size*0.88} {size/2} L{size*0.72} {size*0.68}" fill="none" stroke="{color}" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
    </g>'''

def icon_cpu(x, y, size=16, color="#0284C7"):
    return f'''<g transform="translate({x},{y})">
        <rect x="{size*0.22}" y="{size*0.22}" width="{size*0.56}" height="{size*0.56}" rx="2" fill="none" stroke="{color}" stroke-width="1.5"/>
        <rect x="{size*0.38}" y="{size*0.38}" width="{size*0.24}" height="{size*0.24}" fill="{color}" opacity="0.3"/>
        <line x1="{size*0.32}" y1="{size*0.08}" x2="{size*0.32}" y2="{size*0.22}" stroke="{color}" stroke-width="1.3"/>
        <line x1="{size*0.68}" y1="{size*0.08}" x2="{size*0.68}" y2="{size*0.22}" stroke="{color}" stroke-width="1.3"/>
        <line x1="{size*0.32}" y1="{size*0.78}" x2="{size*0.32}" y2="{size*0.92}" stroke="{color}" stroke-width="1.3"/>
        <line x1="{size*0.68}" y1="{size*0.78}" x2="{size*0.68}" y2="{size*0.92}" stroke="{color}" stroke-width="1.3"/>
        <line x1="{size*0.08}" y1="{size*0.32}" x2="{size*0.22}" y2="{size*0.32}" stroke="{color}" stroke-width="1.3"/>
        <line x1="{size*0.08}" y1="{size*0.68}" x2="{size*0.22}" y2="{size*0.68}" stroke="{color}" stroke-width="1.3"/>
        <line x1="{size*0.78}" y1="{size*0.32}" x2="{size*0.92}" y2="{size*0.32}" stroke="{color}" stroke-width="1.3"/>
        <line x1="{size*0.78}" y1="{size*0.68}" x2="{size*0.92}" y2="{size*0.68}" stroke="{color}" stroke-width="1.3"/>
    </g>'''

def icon_key(x, y, size=16, color="#D97706"):
    return f'''<g transform="translate({x},{y})">
        <circle cx="{size*0.35}" cy="{size*0.4}" r="{size*0.25}" fill="none" stroke="{color}" stroke-width="1.5"/>
        <path d="M{size*0.55} {size*0.55} L{size*0.85} {size*0.85} L{size*0.85} {size*0.72} M{size*0.75} {size*0.75} L{size*0.75} {size*0.65}" fill="none" stroke="{color}" stroke-width="1.5" stroke-linecap="round"/>
    </g>'''

def icon_play(x, y, size=16, color="#16A34A"):
    return f'''<g transform="translate({x},{y})">
        <polygon points="{size*0.25},{size*0.15} {size*0.85},{size/2} {size*0.25},{size*0.85}" fill="{color}"/>
    </g>'''

# -------------------------------------------------------------------------
# COMPONENTES DE INTERFAZ PROFESIONALES (MOCKUP ENGINE)
# -------------------------------------------------------------------------
def render_header_pc(T, active_tab="dashboard"):
    """
    Header estandarizado para PC (1440x900).
    IMPORTANTE:
    - Sin selector arbitrario de usuarios (Zero-Trust SOC).
    - Muestra: Usuario logueado 'Carlos Mendoza', username 'admin_soc', badge 'SOC_ADMIN', estado 'EN LÍNEA (TLS 1.3)' y botón 'Cerrar Sesión'.
    - 8 Pestañas de navegación técnica incluyendo la nueva 'Sandbox ACL'.
    """
    tabs = [
        ("dashboard", "Dashboard"),
        ("topology", "Topología"),
        ("monitoring", "Métricas ML"),
        ("realtime", "Flujo en Vivo"),
        ("alerts", "Alertas Forenses"),
        ("blast_radius", "Blast Radius"),
        ("sandbox_acl", "Sandbox ACL"),
        ("users", "Gestión RBAC")
    ]
    
    out = []
    # Fondo de Header
    out.append(f'<rect x="0" y="0" width="1440" height="64" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
    
    # Logo TIGO SOC
    out.append(f'<rect x="24" y="16" width="36" height="32" rx="4" fill="{T["accent"]}"/>')
    out.append(f'<text x="42" y="37" font-family="{FONT_SANS}" font-size="13" font-weight="800" fill="#FFFFFF" text-anchor="middle">TI</text>')
    out.append(f'<text x="68" y="32" font-family="{FONT_SANS}" font-size="16" font-weight="800" fill="{T["text_primary"]}" letter-spacing="0.5">TIGO <tspan fill="{T["accent"]}">SOC</tspan></text>')
    out.append(f'<text x="68" y="44" font-family="{FONT_SANS}" font-size="9" font-weight="600" fill="{T["text_muted"]}" letter-spacing="1">ISP CYBER DEFENSE</text>')
    
    # Badge Entorno ISP
    out.append(f'<rect x="180" y="20" width="120" height="24" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="1"/>')
    out.append(f'<circle cx="192" cy="32" r="3.5" fill="{T["success"]}"/>')
    out.append(f'<text x="202" y="36" font-family="{FONT_MONO}" font-size="10" font-weight="600" fill="{T["text_secondary"]}">AS27839 - PROD</text>')
    
    # Pestañas Centrales
    start_x = 320
    tab_w = 98
    for i, (tab_id, label) in enumerate(tabs):
        tx = start_x + (i * tab_w)
        is_active = (tab_id == active_tab)
        if is_active:
            out.append(f'<rect x="{tx}" y="16" width="{tab_w-6}" height="32" rx="4" fill="{T["accent_bg"]}" stroke="{T["accent"]}" stroke-width="1.2"/>')
            out.append(f'<text x="{tx + (tab_w-6)/2}" y="36" font-family="{FONT_SANS}" font-size="11.5" font-weight="700" fill="{T["accent_text"]}" text-anchor="middle">{esc(label)}</text>')
        else:
            out.append(f'<text x="{tx + (tab_w-6)/2}" y="36" font-family="{FONT_SANS}" font-size="11.5" font-weight="500" fill="{T["text_secondary"]}" text-anchor="middle">{esc(label)}</text>')

    # Perfil de Usuario Fijo (Sin cambio de cuenta arbitrario)
    user_x = 1115
    out.append(f'<rect x="{user_x}" y="14" width="305" height="36" rx="6" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="1"/>')
    out.append(icon_user(user_x + 10, 24, 16, T["accent"]))
    out.append(f'<text x="{user_x + 32}" y="28" font-family="{FONT_SANS}" font-size="11.5" font-weight="700" fill="{T["text_primary"]}">Carlos Mendoza <tspan font-family="{FONT_MONO}" font-weight="400" fill="{T["text_muted"]}">[admin_soc]</tspan></text>')
    
    # Badge Rol SOC_ADMIN
    out.append(f'<rect x="{user_x + 32}" y="32" width="68" height="14" rx="2" fill="{T["success_bg"]}" stroke="{T["success_border"]}" stroke-width="0.8"/>')
    out.append(f'<text x="{user_x + 66}" y="42" font-family="{FONT_MONO}" font-size="8.5" font-weight="700" fill="{T["success_text"]}" text-anchor="middle">SOC_ADMIN</text>')
    
    # TLS 1.3 Status
    out.append(f'<circle cx="{user_x + 110}" cy="39" r="2.5" fill="{T["success"]}"/>')
    out.append(f'<text x="{user_x + 116}" y="42" font-family="{FONT_SANS}" font-size="9" font-weight="500" fill="{T["text_muted"]}">TLS 1.3 / 2FA</text>')
    
    # Botón Cerrar Sesión
    btn_logout_x = user_x + 205
    out.append(f'<rect x="{btn_logout_x}" y="18" width="92" height="28" rx="4" fill="{T["critical_bg"]}" stroke="{T["critical_border"]}" stroke-width="1"/>')
    out.append(icon_logout(btn_logout_x + 8, 24, 14, T["critical"]))
    out.append(f'<text x="{btn_logout_x + 28}" y="36" font-family="{FONT_SANS}" font-size="10.5" font-weight="700" fill="{T["critical_text"]}">Cerrar Sesión</text>')

    return "".join(out)

def render_header_tablet(T, active_tab="dashboard"):
    """Header optimizado para Tablet (1024x768)"""
    out = []
    out.append(f'<rect x="0" y="0" width="1024" height="58" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
    out.append(f'<rect x="18" y="14" width="30" height="28" rx="4" fill="{T["accent"]}"/>')
    out.append(f'<text x="33" y="33" font-family="{FONT_SANS}" font-size="12" font-weight="800" fill="#FFFFFF" text-anchor="middle">TI</text>')
    out.append(f'<text x="56" y="32" font-family="{FONT_SANS}" font-size="15" font-weight="800" fill="{T["text_primary"]}">TIGO SOC</text>')
    
    # Tabs condensados
    tabs = [("dashboard", "Dash"), ("topology", "Topología"), ("monitoring", "ML"), ("realtime", "Flujo"), ("alerts", "Alertas"), ("blast_radius", "Blast"), ("sandbox_acl", "Sandbox"), ("users", "RBAC")]
    start_x = 180
    tab_w = 68
    for i, (tab_id, label) in enumerate(tabs):
        tx = start_x + (i * tab_w)
        is_active = (tab_id == active_tab)
        if is_active:
            out.append(f'<rect x="{tx}" y="14" width="{tab_w-4}" height="28" rx="4" fill="{T["accent_bg"]}" stroke="{T["accent"]}" stroke-width="1"/>')
            out.append(f'<text x="{tx + (tab_w-4)/2}" y="32" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="{T["accent_text"]}" text-anchor="middle">{esc(label)}</text>')
        else:
            out.append(f'<text x="{tx + (tab_w-4)/2}" y="32" font-family="{FONT_SANS}" font-size="11" font-weight="500" fill="{T["text_secondary"]}" text-anchor="middle">{esc(label)}</text>')

    # Perfil y Logout
    out.append(f'<rect x="740" y="14" width="268" height="28" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="1"/>')
    out.append(f'<text x="750" y="32" font-family="{FONT_SANS}" font-size="11" font-weight="600" fill="{T["text_primary"]}">admin_soc [SOC_ADMIN]</text>')
    out.append(f'<circle cx="895" cy="28" r="3" fill="{T["success"]}"/>')
    out.append(f'<rect x="915" y="14" width="88" height="28" rx="4" fill="{T["critical_bg"]}" stroke="{T["critical_border"]}" stroke-width="1"/>')
    out.append(icon_logout(922, 21, 14, T["critical"]))
    out.append(f'<text x="942" y="32" font-family="{FONT_SANS}" font-size="10.5" font-weight="700" fill="{T["critical_text"]}">Cerrar</text>')
    return "".join(out)

def render_header_mobile(T, title="Dashboard"):
    """App Bar nativo para Móvil (390x844)"""
    out = []
    # Status bar simulado (hora y red)
    out.append(f'<rect x="0" y="0" width="390" height="40" fill="{T["bg_panel"]}"/>')
    out.append(f'<text x="24" y="24" font-family="{FONT_SANS}" font-size="12" font-weight="700" fill="{T["text_primary"]}">14:45</text>')
    out.append(f'<text x="366" y="24" font-family="{FONT_SANS}" font-size="11" font-weight="600" fill="{T["text_secondary"]}" text-anchor="end">5G • 100%</text>')
    
    # Barra de navegación móvil
    out.append(f'<rect x="0" y="40" width="390" height="52" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
    out.append(f'<rect x="16" y="50" width="28" height="28" rx="4" fill="{T["accent"]}"/>')
    out.append(f'<text x="30" y="68" font-family="{FONT_SANS}" font-size="11" font-weight="800" fill="#FFFFFF" text-anchor="middle">TI</text>')
    out.append(f'<text x="52" y="68" font-family="{FONT_SANS}" font-size="14" font-weight="800" fill="{T["text_primary"]}">{esc(title)}</text>')
    
    # Chip Rol & Botón Salir
    out.append(f'<rect x="250" y="52" width="68" height="22" rx="3" fill="{T["success_bg"]}" stroke="{T["success_border"]}" stroke-width="0.8"/>')
    out.append(f'<text x="284" y="67" font-family="{FONT_MONO}" font-size="8.5" font-weight="700" fill="{T["success_text"]}" text-anchor="middle">SOC_ADMIN</text>')
    out.append(f'<rect x="326" y="52" width="48" height="24" rx="4" fill="{T["critical_bg"]}" stroke="{T["critical_border"]}" stroke-width="1"/>')
    out.append(icon_logout(332, 57, 14, T["critical"]))
    out.append(f'<text x="348" y="68" font-family="{FONT_SANS}" font-size="10" font-weight="700" fill="{T["critical_text"]}">Salir</text>')
    return "".join(out)

# -------------------------------------------------------------------------
# GENERADORES ESPECÍFICOS DE LAS 10 PANTALLAS DEL SISTEMA
# -------------------------------------------------------------------------

def gen_screen_01_login(T, device="pc"):
    """PANTALLA 1: Login Reforzado (Email Corporativo + Password + 2FA TOTP + Rate Limit + TLS 1.3)"""
    if device == "pc":
        w, h = 1440, 900
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        
        # Grid de fondo sutil de infraestructura
        for gy in range(0, h, 60):
            out.append(f'<line x1="0" y1="{gy}" x2="{w}" y2="{gy}" stroke="{T["border_subtle"]}" stroke-width="0.5" stroke-dasharray="3,3"/>')
        for gx in range(0, w, 60):
            out.append(f'<line x1="{gx}" y1="0" x2="{gx}" y2="{h}" stroke="{T["border_subtle"]}" stroke-width="0.5" stroke-dasharray="3,3"/>')

        # Tarjeta Central de Autenticación
        cw, ch = 520, 680
        cx, cy = (w - cw) // 2, (h - ch) // 2
        out.append(f'<rect x="{cx}" y="{cy}" width="{cw}" height="{ch}" rx="8" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1.2"/>')
        
        # Header de Tarjeta
        out.append(f'<rect x="{cx+36}" y="{cy+36}" width="42" height="38" rx="6" fill="{T["accent"]}"/>')
        out.append(icon_shield(cx+44, cy+44, 26, "#FFFFFF"))
        out.append(f'<text x="{cx+90}" y="{cy+54}" font-family="{FONT_SANS}" font-size="20" font-weight="800" fill="{T["text_primary"]}">TIGO <tspan fill="{T["accent"]}">SOC</tspan> GATEWAY</text>')
        out.append(f'<text x="{cx+90}" y="{cy+70}" font-family="{FONT_SANS}" font-size="11.5" font-weight="600" fill="{T["text_muted"]}">Acceso Zero-Trust para Operadores de Red e Infraestructura ISP</text>')
        
        # Banner de Certificado TLS 1.3 y Auditoría
        out.append(f'<rect x="{cx+36}" y="{cy+92}" width="{cw-72}" height="32" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="1"/>')
        out.append(icon_lock(cx+46, cy+100, 15, T["success"]))
        out.append(f'<text x="{cx+68}" y="{cy+112}" font-family="{FONT_MONO}" font-size="10.5" font-weight="600" fill="{T["text_secondary"]}">TLS 1.3 / mTLS Corporativo Activo • SHA-256 Validado</text>')

        # Campo 1: Correo Corporativo
        fy1 = cy + 144
        out.append(f'<text x="{cx+36}" y="{fy1}" font-family="{FONT_SANS}" font-size="12" font-weight="700" fill="{T["text_primary"]}">Correo Electrónico Corporativo (SSO / Tigo ID)</text>')
        out.append(f'<rect x="{cx+36}" y="{fy1+8}" width="{cw-72}" height="44" rx="4" fill="{T["bg_panel"]}" stroke="{T["accent"]}" stroke-width="1.5"/>')
        out.append(icon_user(cx+48, fy1+20, 18, T["accent"]))
        out.append(f'<text x="{cx+76}" y="{fy1+34}" font-family="{FONT_MONO}" font-size="13" font-weight="500" fill="{T["text_primary"]}">carlos.mendoza@tigo.net.bo</text>')

        # Campo 2: Contraseña
        fy2 = fy1 + 76
        out.append(f'<text x="{cx+36}" y="{fy2}" font-family="{FONT_SANS}" font-size="12" font-weight="700" fill="{T["text_primary"]}">Contraseña de Red</text>')
        out.append(f'<rect x="{cx+36}" y="{fy2+8}" width="{cw-72}" height="44" rx="4" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1.2"/>')
        out.append(icon_lock(cx+48, fy2+20, 18, T["text_muted"]))
        out.append(f'<text x="{cx+76}" y="{fy2+36}" font-family="{FONT_MONO}" font-size="16" fill="{T["text_primary"]}">••••••••••••••••••••</text>')
        out.append(f'<text x="{cx+cw-50}" y="{fy2+34}" font-family="{FONT_SANS}" font-size="11" font-weight="600" fill="{T["accent"]}" text-anchor="end">[Ocultar]</text>')

        # Campo 3: Autenticación de Dos Factores (2FA / TOTP)
        fy3 = fy2 + 76
        out.append(f'<text x="{cx+36}" y="{fy3}" font-family="{FONT_SANS}" font-size="12" font-weight="700" fill="{T["text_primary"]}">Código de Segundo Factor 2FA / TOTP (Hardware o Authenticator)</text>')
        out.append(f'<text x="{cx+36}" y="{fy3+16}" font-family="{FONT_SANS}" font-size="10.5" fill="{T["text_muted"]}">Ingrese el token temporal de 6 dígitos emitido por su autenticador registrado:</text>')
        
        # 6 casillas numéricas de token
        box_w, box_gap = 48, 12
        token_start = cx + 36
        digits = ["7", "4", "9", "2", "0", "1"]
        for di, dig in enumerate(digits):
            bx = token_start + di * (box_w + box_gap)
            if di >= 3:
                bx += 14  # Espacio separador de 3 en 3
            out.append(f'<rect x="{bx}" y="{fy3+28}" width="{box_w}" height="48" rx="4" fill="{T["bg_subtle"]}" stroke="{T["accent"] if di<5 else T["border"]}" stroke-width="1.5"/>')
            out.append(f'<text x="{bx+box_w/2}" y="{fy3+58}" font-family="{FONT_MONO}" font-size="20" font-weight="700" fill="{T["text_primary"]}" text-anchor="middle">{dig}</text>')

        # Aviso de Rate Limiting y Control de Intentos
        fy4 = fy3 + 104
        out.append(f'<rect x="{cx+36}" y="{fy4}" width="{cw-72}" height="56" rx="4" fill="{T["warning_bg"]}" stroke="{T["warning_border"]}" stroke-width="1"/>')
        out.append(icon_alert(cx+48, fy4+14, 18, T["warning"]))
        out.append(f'<text x="{cx+76}" y="{fy4+24}" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="{T["warning_text"]}">CONTROL DE INTENTOS DE ACCESO</text>')
        out.append(f'<text x="{cx+76}" y="{fy4+40}" font-family="{FONT_SANS}" font-size="10" font-weight="500" fill="{T["warning_text"]}">Intentos restantes permitidos: 3 antes de bloqueo temporal de IP (15 min).</text>')

        # Botón de Inicio de Sesión
        btn_y = fy4 + 72
        out.append(f'<rect x="{cx+36}" y="{btn_y}" width="{cw-72}" height="46" rx="4" fill="{T["accent"]}"/>')
        out.append(f'<text x="{cx + cw/2}" y="{btn_y+28}" font-family="{FONT_SANS}" font-size="13" font-weight="700" fill="#FFFFFF" text-anchor="middle">Validar Credenciales e Ingresar</text>')

        # Footer con enlaces de seguridad
        foot_y = btn_y + 68
        out.append(f'<text x="{cx+cw/2}" y="{foot_y}" font-family="{FONT_SANS}" font-size="10.5" fill="{T["text_muted"]}" text-anchor="middle">¿Problemas con su token 2FA? Contactar a soporte SOC (Ext: 4401)</text>')
        out.append(f'<text x="{cx+cw/2}" y="{foot_y+18}" font-family="{FONT_SANS}" font-size="9.5" fill="{T["text_muted"]}" text-anchor="middle">Cumplimiento ISO/IEC 27001 • Conexión cifrada de extremo a extremo</text>')

        out.append('</svg>')
        return "".join(out)

    elif device == "tablet":
        w, h = 1024, 768
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        cw, ch = 480, 620
        cx, cy = (w - cw) // 2, (h - ch) // 2
        out.append(f'<rect x="{cx}" y="{cy}" width="{cw}" height="{ch}" rx="8" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1.2"/>')
        out.append(f'<rect x="{cx+32}" y="{cy+30}" width="36" height="32" rx="4" fill="{T["accent"]}"/>')
        out.append(icon_shield(cx+40, cy+36, 20, "#FFFFFF"))
        out.append(f'<text x="{cx+80}" y="{cy+48}" font-family="{FONT_SANS}" font-size="18" font-weight="800" fill="{T["text_primary"]}">TIGO SOC GATEWAY</text>')
        out.append(f'<text x="{cx+80}" y="{cy+62}" font-family="{FONT_SANS}" font-size="11" fill="{T["text_muted"]}">Autenticación Corporativa de Alta Seguridad (Tablet)</text>')
        
        # Formulario
        y = cy + 90
        out.append(f'<text x="{cx+32}" y="{y}" font-family="{FONT_SANS}" font-size="11.5" font-weight="700" fill="{T["text_primary"]}">Correo SSO:</text>')
        out.append(f'<rect x="{cx+32}" y="{y+8}" width="{cw-64}" height="38" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{cx+44}" y="{y+32}" font-family="{FONT_MONO}" font-size="12" fill="{T["text_primary"]}">carlos.mendoza@tigo.net.bo</text>')
        
        y += 64
        out.append(f'<text x="{cx+32}" y="{y}" font-family="{FONT_SANS}" font-size="11.5" font-weight="700" fill="{T["text_primary"]}">Contraseña:</text>')
        out.append(f'<rect x="{cx+32}" y="{y+8}" width="{cw-64}" height="38" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{cx+44}" y="{y+32}" font-family="{FONT_MONO}" font-size="14" fill="{T["text_primary"]}">••••••••••••••••</text>')
        
        y += 64
        out.append(f'<text x="{cx+32}" y="{y}" font-family="{FONT_SANS}" font-size="11.5" font-weight="700" fill="{T["text_primary"]}">Token 2FA (6 Dígitos):</text>')
        for di, dig in enumerate(["7", "4", "9", "2", "0", "1"]):
            bx = cx + 32 + di * 52 + (10 if di>=3 else 0)
            out.append(f'<rect x="{bx}" y="{y+8}" width="44" height="42" rx="4" fill="{T["bg_subtle"]}" stroke="{T["accent"]}" stroke-width="1.2"/>')
            out.append(f'<text x="{bx+22}" y="{y+34}" font-family="{FONT_MONO}" font-size="18" font-weight="700" fill="{T["text_primary"]}" text-anchor="middle">{dig}</text>')
        
        y += 72
        out.append(f'<rect x="{cx+32}" y="{y}" width="{cw-64}" height="46" rx="4" fill="{T["warning_bg"]}" stroke="{T["warning_border"]}" stroke-width="1"/>')
        out.append(icon_alert(cx+42, y+14, 16, T["warning"]))
        out.append(f'<text x="{cx+66}" y="{y+26}" font-family="{FONT_SANS}" font-size="10" font-weight="700" fill="{T["warning_text"]}">Rate Limiting Activo: 3 intentos restantes antes de bloqueo temporal</text>')
        
        y += 62
        out.append(f'<rect x="{cx+32}" y="{y}" width="{cw-64}" height="44" rx="4" fill="{T["accent"]}"/>')
        out.append(f'<text x="{cx+cw/2}" y="{y+27}" font-family="{FONT_SANS}" font-size="13" font-weight="700" fill="#FFFFFF" text-anchor="middle">Ingresar al SOC</text>')
        out.append('</svg>')
        return "".join(out)

    else: # mobile (390x844)
        w, h = 390, 844
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        # Status bar
        out.append(f'<rect x="0" y="0" width="390" height="44" fill="{T["bg_panel"]}"/>')
        out.append(f'<text x="24" y="28" font-family="{FONT_SANS}" font-size="13" font-weight="700" fill="{T["text_primary"]}">14:45</text>')
        
        # Tarjeta móvil
        cx, cy, cw, ch = 20, 60, 350, 750
        out.append(f'<rect x="{cx}" y="{cy}" width="{cw}" height="{ch}" rx="8" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1.2"/>')
        out.append(f'<rect x="{cx+20}" y="{cy+24}" width="36" height="32" rx="4" fill="{T["accent"]}"/>')
        out.append(icon_shield(cx+28, cy+30, 20, "#FFFFFF"))
        out.append(f'<text x="{cx+66}" y="{cy+42}" font-family="{FONT_SANS}" font-size="16" font-weight="800" fill="{T["text_primary"]}">TIGO SOC MOBILE</text>')
        out.append(f'<text x="{cx+66}" y="{cy+56}" font-family="{FONT_SANS}" font-size="10" fill="{T["text_muted"]}">Zero-Trust PWA Mobile Console</text>')
        
        y = cy + 80
        out.append(f'<rect x="{cx+20}" y="{y}" width="{cw-40}" height="30" rx="3" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
        out.append(icon_lock(cx+26, y+7, 14, T["success"]))
        out.append(f'<text x="{cx+46}" y="{y+19}" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_secondary"]}">mTLS + Biometría Habilitada</text>')
        
        y += 46
        out.append(f'<text x="{cx+20}" y="{y}" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="{T["text_primary"]}">Correo Corporativo Tigo:</text>')
        out.append(f'<rect x="{cx+20}" y="{y+6}" width="{cw-40}" height="40" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{cx+32}" y="{y+30}" font-family="{FONT_MONO}" font-size="11.5" fill="{T["text_primary"]}">carlos.mendoza@tigo.net.bo</text>')
        
        y += 62
        out.append(f'<text x="{cx+20}" y="{y}" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="{T["text_primary"]}">Contraseña:</text>')
        out.append(f'<rect x="{cx+20}" y="{y+6}" width="{cw-40}" height="40" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{cx+32}" y="{y+32}" font-family="{FONT_MONO}" font-size="14" fill="{T["text_primary"]}">••••••••••••••••</text>')
        
        y += 62
        out.append(f'<text x="{cx+20}" y="{y}" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="{T["text_primary"]}">Token 2FA / TOTP (6 Dígitos):</text>')
        for di, dig in enumerate(["7", "4", "9", "2", "0", "1"]):
            bx = cx + 20 + di * 48 + (6 if di>=3 else 0)
            out.append(f'<rect x="{bx}" y="{y+6}" width="42" height="42" rx="4" fill="{T["bg_subtle"]}" stroke="{T["accent"]}" stroke-width="1.2"/>')
            out.append(f'<text x="{bx+21}" y="{y+32}" font-family="{FONT_MONO}" font-size="17" font-weight="700" fill="{T["text_primary"]}" text-anchor="middle">{dig}</text>')
        
        y += 68
        out.append(f'<rect x="{cx+20}" y="{y}" width="{cw-40}" height="48" rx="4" fill="{T["warning_bg"]}" stroke="{T["warning_border"]}" stroke-width="1"/>')
        out.append(icon_alert(cx+28, y+14, 16, T["warning"]))
        out.append(f'<text x="{cx+50}" y="{y+24}" font-family="{FONT_SANS}" font-size="9.5" font-weight="700" fill="{T["warning_text"]}">Protección Anti Fuerza Bruta Activa</text>')
        out.append(f'<text x="{cx+50}" y="{y+38}" font-family="{FONT_SANS}" font-size="8.5" fill="{T["warning_text"]}">3 intentos restantes. Bloqueo 15 min.</text>')
        
        y += 64
        out.append(f'<rect x="{cx+20}" y="{y}" width="{cw-40}" height="46" rx="4" fill="{T["accent"]}"/>')
        out.append(f'<text x="{cx+cw/2}" y="{y+28}" font-family="{FONT_SANS}" font-size="13" font-weight="700" fill="#FFFFFF" text-anchor="middle">Autenticar e Ingresar</text>')
        
        y += 64
        out.append(f'<text x="{cx+cw/2}" y="{y}" font-family="{FONT_SANS}" font-size="9.5" fill="{T["text_muted"]}" text-anchor="middle">Soporte SOC: 24/7 Monitoreo de Amenazas</text>')
        out.append('</svg>')
        return "".join(out)

def gen_screen_01b_password_change(T, device="pc"):
    """PANTALLA 1B: Cambio Obligatorio de Contraseña por Defecto (Seguridad Forzada al Primer Inicio)"""
    if device == "pc":
        w, h = 1440, 900
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        
        # Fondo técnico
        for gy in range(0, h, 60):
            out.append(f'<line x1="0" y1="{gy}" x2="{w}" y2="{gy}" stroke="{T["border_subtle"]}" stroke-width="0.5" stroke-dasharray="3,3"/>')
        for gx in range(0, w, 60):
            out.append(f'<line x1="{gx}" y1="0" x2="{gx}" y2="{h}" stroke="{T["border_subtle"]}" stroke-width="0.5" stroke-dasharray="3,3"/>')

        cw, ch = 640, 720
        cx, cy = (w - cw) // 2, (h - ch) // 2
        out.append(f'<rect x="{cx}" y="{cy}" width="{cw}" height="{ch}" rx="8" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1.2"/>')
        
        # Alerta Superior Crítica: Contraseña Temporal Detectada
        out.append(f'<rect x="{cx+36}" y="{cy+32}" width="{cw-72}" height="68" rx="6" fill="{T["critical_bg"]}" stroke="{T["critical_border"]}" stroke-width="1.2"/>')
        out.append(icon_alert(cx+48, cy+46, 24, T["critical"]))
        out.append(f'<text x="{cx+82}" y="{cy+54}" font-family="{FONT_SANS}" font-size="12.5" font-weight="800" fill="{T["critical_text"]}">POLÍTICA DE ROTACIÓN DE CONTRASEÑA OBLIGATORIA</text>')
        out.append(f'<text x="{cx+82}" y="{cy+70}" font-family="{FONT_SANS}" font-size="11" font-weight="500" fill="{T["critical_text"]}">Su cuenta <tspan font-family="{FONT_MONO}" font-weight="700">admin_soc</tspan> fue iniciada con la credencial temporal por defecto ("admin").</text>')
        out.append(f'<text x="{cx+82}" y="{cy+84}" font-family="{FONT_SANS}" font-size="10.5" font-weight="500" fill="{T["critical_text"]}">Por normativa de seguridad ISO 27001 y Zero-Trust, debe establecer una clave nueva antes de continuar.</text>')

        # Campo 1: Contraseña Temporal Actual
        fy1 = cy + 120
        out.append(f'<text x="{cx+36}" y="{fy1}" font-family="{FONT_SANS}" font-size="11.5" font-weight="700" fill="{T["text_primary"]}">Contraseña Temporal Actual:</text>')
        out.append(f'<rect x="{cx+36}" y="{fy1+8}" width="{cw-72}" height="40" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(icon_lock(cx+48, fy1+18, 16, T["text_muted"]))
        out.append(f'<text x="{cx+76}" y="{fy1+33}" font-family="{FONT_MONO}" font-size="13" fill="{T["text_secondary"]}">••••••••• (admin)</text>')

        # Campo 2: Nueva Contraseña
        fy2 = fy1 + 64
        out.append(f'<text x="{cx+36}" y="{fy2}" font-family="{FONT_SANS}" font-size="11.5" font-weight="700" fill="{T["text_primary"]}">Nueva Contraseña Corporativa:</text>')
        out.append(f'<rect x="{cx+36}" y="{fy2+8}" width="{cw-72}" height="40" rx="4" fill="{T["bg_panel"]}" stroke="{T["accent"]}" stroke-width="1.5"/>')
        out.append(icon_key(cx+48, fy2+18, 16, T["accent"]))
        out.append(f'<text x="{cx+76}" y="{fy2+33}" font-family="{FONT_MONO}" font-size="15" fill="{T["text_primary"]}">T1g0_S0c_Pr0t3ct#2026!</text>')

        # Medidor de Entropía Dinámico (Barras de progreso visual)
        fy_ent = fy2 + 54
        bar_w = (cw - 72 - 18) / 4
        for bi in range(4):
            bx = cx + 36 + bi * (bar_w + 6)
            out.append(f'<rect x="{bx}" y="{fy_ent}" width="{bar_w}" height="6" rx="3" fill="{T["success"]}"/>')

        # Campo 3: Confirmación de Contraseña
        fy3 = fy_ent + 24
        out.append(f'<text x="{cx+36}" y="{fy3}" font-family="{FONT_SANS}" font-size="11.5" font-weight="700" fill="{T["text_primary"]}">Confirmar Nueva Contraseña:</text>')
        out.append(f'<rect x="{cx+36}" y="{fy3+8}" width="{cw-72}" height="40" rx="4" fill="{T["bg_panel"]}" stroke="{T["success"]}" stroke-width="1.5"/>')
        out.append(icon_check(cx+48, fy3+18, 16, T["success"]))
        out.append(f'<text x="{cx+76}" y="{fy3+33}" font-family="{FONT_MONO}" font-size="15" fill="{T["text_primary"]}">T1g0_S0c_Pr0t3ct#2026!</text>')

        # Panel de Requisitos de Complejidad (Checklist de Seguridad)
        fy_chk = fy3 + 64
        out.append(f'<rect x="{cx+36}" y="{fy_chk}" width="{cw-72}" height="140" rx="6" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="1"/>')
        out.append(f'<text x="{cx+52}" y="{fy_chk+24}" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="{T["text_primary"]}">Requisitos de Seguridad de Contraseña (Zero-Trust Standard):</text>')
        
        reqs = [
            "Mínimo 14 caracteres de longitud (Actual: 22 caracteres)",
            "Al menos una letra mayúscula y una letra minúscula (A-Z, a-z)",
            "Al menos dos números y un símbolo especial (!, @, #, $, %, ^, &amp;, *)",
            "Sin coincidencias con datos personales, usuario o diccionario de contraseñas de red",
            "No reutilizar ninguna de las últimas 5 contraseñas históricas"
        ]
        for ri, req in enumerate(reqs):
            ry = fy_chk + 46 + ri * 19
            out.append(icon_check(cx+52, ry-10, 12, T["success"]))
            out.append(f'<text x="{cx+72}" y="{ry}" font-family="{FONT_SANS}" font-size="10.5" fill="{T["text_secondary"]}">{req}</text>')

        # Botones de Acción
        btn_y = fy_chk + 160
        out.append(f'<rect x="{cx+36}" y="{btn_y}" width="{cw-72}" height="46" rx="4" fill="{T["success"]}"/>')
        out.append(f'<text x="{cx + cw/2}" y="{btn_y+28}" font-family="{FONT_SANS}" font-size="13" font-weight="700" fill="#FFFFFF" text-anchor="middle">Actualizar Contraseña y Acceder a la Consola SOC</text>')

        out.append(f'<text x="{cx+cw/2}" y="{btn_y+64}" font-family="{FONT_SANS}" font-size="11" font-weight="600" fill="{T["critical"]}" text-anchor="middle">Cancelar y Cerrar Sesión Segura</text>')

        out.append('</svg>')
        return "".join(out)

    elif device == "tablet":
        w, h = 1024, 768
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        cw, ch = 560, 640
        cx, cy = (w - cw) // 2, (h - ch) // 2
        out.append(f'<rect x="{cx}" y="{cy}" width="{cw}" height="{ch}" rx="8" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1.2"/>')
        out.append(f'<rect x="{cx+28}" y="{cy+24}" width="{cw-56}" height="56" rx="4" fill="{T["critical_bg"]}" stroke="{T["critical_border"]}" stroke-width="1"/>')
        out.append(icon_alert(cx+36, cy+38, 20, T["critical"]))
        out.append(f'<text x="{cx+64}" y="{cy+48}" font-family="{FONT_SANS}" font-size="11.5" font-weight="800" fill="{T["critical_text"]}">CAMBIO OBLIGATORIO DE CONTRASEÑA TEMPORAL</text>')
        out.append(f'<text x="{cx+64}" y="{cy+64}" font-family="{FONT_SANS}" font-size="10.5" fill="{T["critical_text"]}">Debe establecer una nueva clave antes de acceder al SOC.</text>')
        
        y = cy + 100
        out.append(f'<text x="{cx+28}" y="{y}" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="{T["text_primary"]}">Nueva Contraseña:</text>')
        out.append(f'<rect x="{cx+28}" y="{y+6}" width="{cw-56}" height="38" rx="4" fill="{T["bg_subtle"]}" stroke="{T["accent"]}" stroke-width="1.2"/>')
        out.append(f'<text x="{cx+40}" y="{y+30}" font-family="{FONT_MONO}" font-size="13" fill="{T["text_primary"]}">T1g0_S0c_Pr0t3ct#2026!</text>')
        
        y += 58
        out.append(f'<text x="{cx+28}" y="{y}" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="{T["text_primary"]}">Confirmación:</text>')
        out.append(f'<rect x="{cx+28}" y="{y+6}" width="{cw-56}" height="38" rx="4" fill="{T["bg_subtle"]}" stroke="{T["success"]}" stroke-width="1.2"/>')
        out.append(f'<text x="{cx+40}" y="{y+30}" font-family="{FONT_MONO}" font-size="13" fill="{T["text_primary"]}">T1g0_S0c_Pr0t3ct#2026!</text>')
        
        y += 58
        out.append(f'<rect x="{cx+28}" y="{y}" width="{cw-56}" height="120" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="1"/>')
        out.append(f'<text x="{cx+40}" y="{y+20}" font-family="{FONT_SANS}" font-size="10.5" font-weight="700" fill="{T["text_primary"]}">Requisitos Cumplidos (100%):</text>')
        for ri, req in enumerate(["Mínimo 14 caracteres (OK)", "Mayúsculas y minúsculas (OK)", "Números y símbolos especiales (OK)", "Entropía alta de 94 bits (OK)"]):
            out.append(icon_check(cx+40, y+34+ri*20, 12, T["success"]))
            out.append(f'<text x="{cx+58}" y="{y+44+ri*20}" font-family="{FONT_SANS}" font-size="10" fill="{T["text_secondary"]}">{req}</text>')
        
        y += 140
        out.append(f'<rect x="{cx+28}" y="{y}" width="{cw-56}" height="42" rx="4" fill="{T["success"]}"/>')
        out.append(f'<text x="{cx+cw/2}" y="{y+26}" font-family="{FONT_SANS}" font-size="12" font-weight="700" fill="#FFFFFF" text-anchor="middle">Actualizar y Entrar al SOC</text>')
        out.append('</svg>')
        return "".join(out)

    else: # mobile (390x844)
        w, h = 390, 844
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        out.append(render_header_mobile(T, "Rotación de Clave"))
        
        cx, cy, cw, ch = 16, 100, 358, 710
        out.append(f'<rect x="{cx}" y="{cy}" width="{cw}" height="{ch}" rx="8" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1.2"/>')
        out.append(f'<rect x="{cx+16}" y="{cy+16}" width="{cw-32}" height="64" rx="4" fill="{T["critical_bg"]}" stroke="{T["critical_border"]}" stroke-width="1"/>')
        out.append(icon_alert(cx+24, cy+28, 18, T["critical"]))
        out.append(f'<text x="{cx+48}" y="{cy+38}" font-family="{FONT_SANS}" font-size="10.5" font-weight="800" fill="{T["critical_text"]}">CAMBIO OBLIGATORIO DE CLAVE</text>')
        out.append(f'<text x="{cx+48}" y="{cy+52}" font-family="{FONT_SANS}" font-size="9" fill="{T["critical_text"]}">Clave por defecto ("admin") no permitida.</text>')
        out.append(f'<text x="{cx+48}" y="{cy+66}" font-family="{FONT_SANS}" font-size="9" fill="{T["critical_text"]}">Debe crear una nueva contraseña.</text>')
        
        y = cy + 96
        out.append(f'<text x="{cx+16}" y="{y}" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="{T["text_primary"]}">Nueva Contraseña:</text>')
        out.append(f'<rect x="{cx+16}" y="{y+6}" width="{cw-32}" height="38" rx="4" fill="{T["bg_subtle"]}" stroke="{T["accent"]}" stroke-width="1.2"/>')
        out.append(f'<text x="{cx+28}" y="{y+30}" font-family="{FONT_MONO}" font-size="12" fill="{T["text_primary"]}">T1g0_S0c_Pr0t3ct#2026!</text>')
        
        y += 56
        out.append(f'<text x="{cx+16}" y="{y}" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="{T["text_primary"]}">Confirmar Contraseña:</text>')
        out.append(f'<rect x="{cx+16}" y="{y+6}" width="{cw-32}" height="38" rx="4" fill="{T["bg_subtle"]}" stroke="{T["success"]}" stroke-width="1.2"/>')
        out.append(f'<text x="{cx+28}" y="{y+30}" font-family="{FONT_MONO}" font-size="12" fill="{T["text_primary"]}">T1g0_S0c_Pr0t3ct#2026!</text>')
        
        y += 56
        out.append(f'<rect x="{cx+16}" y="{y}" width="{cw-32}" height="140" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="1"/>')
        out.append(f'<text x="{cx+26}" y="{y+20}" font-family="{FONT_SANS}" font-size="10" font-weight="700" fill="{T["text_primary"]}">Checklist de Seguridad:</text>')
        reqs = ["Mínimo 14 caracteres (OK)", "Mayúsculas y minúsculas (OK)", "Números y símbolos (OK)", "Entropía alta de 94 bits (OK)", "No repite anteriores (OK)"]
        for ri, req in enumerate(reqs):
            out.append(icon_check(cx+26, y+32+ri*20, 10, T["success"]))
            out.append(f'<text x="{cx+42}" y="{y+41+ri*20}" font-family="{FONT_SANS}" font-size="9" fill="{T["text_secondary"]}">{req}</text>')
        
        y += 160
        out.append(f'<rect x="{cx+16}" y="{y}" width="{cw-32}" height="44" rx="4" fill="{T["success"]}"/>')
        out.append(f'<text x="{cx+cw/2}" y="{y+27}" font-family="{FONT_SANS}" font-size="12" font-weight="700" fill="#FFFFFF" text-anchor="middle">Actualizar y Continuar</text>')
        out.append('</svg>')
        return "".join(out)

def gen_screen_02_dashboard(T, device="pc"):
    """PANTALLA 2: Dashboard Central del SOC & Telemetría Global"""
    if device == "pc":
        w, h = 1440, 900
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        out.append(render_header_pc(T, "dashboard"))
        
        # 4 Tarjetas Métricas KPI Superiores
        kpis = [
            ("Tráfico Agregado ISP", "4.28 Gbps", "Capacidad: 10 Gbps (42.8%)", T["accent"], icon_network),
            ("Paquetes Inspeccionados eBPF", "1,842,910 pps", "Latencia XDP: 12 µs • Cero Drops", T["success"], icon_cpu),
            ("Anomalías Mitigadas Hoy", "14 Eventos L4/L7", "Eficacia de Bloqueo: 99.8%", T["critical"], icon_alert),
            ("Estado del Clúster ML", "99.42% Precisión", "LightGBM v4.2 • Inferencia 0.42 ms", T["sandbox"], icon_chart)
        ]
        card_w = (1440 - 48 - (3 * 20)) / 4
        for ki, (title, val, sub, col, icon_fn) in enumerate(kpis):
            kx = 24 + ki * (card_w + 20)
            out.append(f'<rect x="{kx}" y="80" width="{card_w}" height="100" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
            out.append(f'<rect x="{kx}" y="80" width="{card_w}" height="3" rx="1.5" fill="{col}"/>')
            out.append(icon_fn(kx+16, 96, 20, col))
            out.append(f'<text x="{kx+44}" y="110" font-family="{FONT_SANS}" font-size="11" font-weight="600" fill="{T["text_muted"]}">{esc(title)}</text>')
            out.append(f'<text x="{kx+16}" y="145" font-family="{FONT_SANS}" font-size="22" font-weight="800" fill="{T["text_primary"]}">{esc(val)}</text>')
            out.append(f'<text x="{kx+16}" y="165" font-family="{FONT_MONO}" font-size="9.5" font-weight="500" fill="{T["text_secondary"]}">{esc(sub)}</text>')

        # Gráfico Principal: Telemetría de Ancho de Banda y Detección de Anomalías
        gx, gy, gw, gh = 24, 196, 920, 360
        out.append(f'<rect x="{gx}" y="{gy}" width="{gw}" height="{gh}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{gx+20}" y="{gy+28}" font-family="{FONT_SANS}" font-size="14" font-weight="800" fill="{T["text_primary"]}">Telemetría de Tráfico en Tiempo Real &amp; Detección de Anomalías eBPF</text>')
        out.append(f'<text x="{gx+20}" y="{gy+44}" font-family="{FONT_SANS}" font-size="10.5" fill="{T["text_muted"]}">Monitoreo continuo de ancho de banda ISP (Gbps) comparado con tráfico sospechoso detectado por LightGBM</text>')
        
        # Leyenda del Gráfico
        out.append(f'<rect x="{gx+gw-310}" y="{gy+20}" width="12" height="12" rx="2" fill="{T["accent"]}"/>')
        out.append(f'<text x="{gx+gw-292}" y="{gy+30}" font-family="{FONT_SANS}" font-size="10.5" fill="{T["text_secondary"]}">Tráfico Legítimo</text>')
        out.append(f'<rect x="{gx+gw-190}" y="{gy+20}" width="12" height="12" rx="2" fill="{T["critical"]}"/>')
        out.append(f'<text x="{gx+gw-172}" y="{gy+30}" font-family="{FONT_SANS}" font-size="10.5" fill="{T["text_secondary"]}">Ataque Mitigado</text>')

        # Área de Gráfica (Ejes e Intervalos)
        plot_x, plot_y, plot_w, plot_h = gx + 50, gy + 70, gw - 70, gh - 100
        for i in range(5):
            ly = plot_y + i * (plot_h / 4)
            val_label = f"{5 - i * 1.25:.1f}G"
            out.append(f'<text x="{plot_x-10}" y="{ly+4}" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_muted"]}" text-anchor="end">{val_label}</text>')
            out.append(f'<line x1="{plot_x}" y1="{ly}" x2="{plot_x+plot_w}" y2="{ly}" stroke="{T["border_subtle"]}" stroke-width="0.8" stroke-dasharray="2,2"/>')

        # Curva de Tráfico Normal (Área + Línea)
        pts_normal = [
            (plot_x, plot_y+160), (plot_x+100, plot_y+150), (plot_x+200, plot_y+155),
            (plot_x+300, plot_y+140), (plot_x+400, plot_y+130), (plot_x+500, plot_y+135),
            (plot_x+600, plot_y+120), (plot_x+700, plot_y+115), (plot_x+800, plot_y+110),
            (plot_x+plot_w, plot_y+105)
        ]
        poly_str = f"{plot_x},{plot_y+plot_h} " + " ".join(f"{px},{py}" for px, py in pts_normal) + f" {plot_x+plot_w},{plot_y+plot_h}"
        out.append(f'<polygon points="{poly_str}" fill="{T["accent_bg"]}" opacity="0.4"/>')
        out.append(f'<polyline points="{" ".join(f"{px},{py}" for px,py in pts_normal)}" fill="none" stroke="{T["accent"]}" stroke-width="2.2" stroke-linecap="round"/>')

        # Pico de Ataque Mitigado (Rojo)
        pts_attack = [
            (plot_x+450, plot_y+135), (plot_x+500, plot_y+40), (plot_x+530, plot_y+20),
            (plot_x+560, plot_y+35), (plot_x+600, plot_y+120)
        ]
        out.append(f'<polyline points="{" ".join(f"{px},{py}" for px,py in pts_attack)}" fill="none" stroke="{T["critical"]}" stroke-width="2.5" stroke-dasharray="3,1"/>')
        # Marcador de Mitigación eBPF
        out.append(f'<rect x="{plot_x+500}" y="{plot_y+12}" width="150" height="22" rx="3" fill="{T["critical_bg"]}" stroke="{T["critical"]}" stroke-width="1"/>')
        out.append(f'<text x="{plot_x+575}" y="{plot_y+27}" font-family="{FONT_MONO}" font-size="9" font-weight="700" fill="{T["critical_text"]}" text-anchor="middle">DROP eBPF XDP: 350k pps</text>')

        # Panel Lateral Derecho: Estado de Topología & Nodos
        rx, ry, rw, rh = 960, 196, 456, 360
        out.append(f'<rect x="{rx}" y="{ry}" width="{rw}" height="{rh}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{rx+20}" y="{ry+28}" font-family="{FONT_SANS}" font-size="14" font-weight="800" fill="{T["text_primary"]}">Infraestructura de Red ISP</text>')
        out.append(f'<text x="{rx+20}" y="{ry+44}" font-family="{FONT_SANS}" font-size="10.5" fill="{T["text_muted"]}">Estado de enlaces de transporte y filtros de borde</text>')
        
        nodes = [
            ("Core-Router-01 (AS27839)", "172.16.10.1", "eBPF/XDP: Activo", "1.84 Gbps", T["success"]),
            ("Carrier-Peering-02", "198.51.100.1", "BGP Flowspec: OK", "1.42 Gbps", T["success"]),
            ("Metro-Ethernet-Agg01", "10.10.0.5", "Tráfico Elevado", "820 Mbps", T["warning"]),
            ("Honeypot-Sensor-01", "172.28.0.10", "Ataque L4 Detectado", "350k pps", T["critical"]),
            ("Sandbox-Router (Simulación)", "10.200.0.1", "Subred Aislada VLAN 999", "En Espera", T["sandbox"]),
        ]
        for ni, (n_name, n_ip, n_status, n_rate, n_col) in enumerate(nodes):
            ny = ry + 62 + ni * 56
            out.append(f'<rect x="{rx+16}" y="{ny}" width="{rw-32}" height="48" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="1"/>')
            out.append(f'<circle cx="{rx+32}" cy="{ny+24}" r="4" fill="{n_col}"/>')
            out.append(f'<text x="{rx+46}" y="{ny+20}" font-family="{FONT_SANS}" font-size="11.5" font-weight="700" fill="{T["text_primary"]}">{esc(n_name)}</text>')
            out.append(f'<text x="{rx+46}" y="{ny+36}" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_muted"]}">{esc(n_ip)} • {esc(n_status)}</text>')
            out.append(f'<text x="{rx+rw-32}" y="{ny+28}" font-family="{FONT_MONO}" font-size="11" font-weight="700" fill="{n_col}" text-anchor="end">{esc(n_rate)}</text>')

        # Tabla Inferior: Alertas Forenses Recientes
        tx, ty, tw, th = 24, 572, 1392, 304
        out.append(f'<rect x="{tx}" y="{ty}" width="{tw}" height="{th}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{tx+20}" y="{ty+28}" font-family="{FONT_SANS}" font-size="14" font-weight="800" fill="{T["text_primary"]}">Últimas Alertas de Seguridad &amp; Acciones de Mitigación Automática</text>')
        
        # Encabezado de Tabla
        th_y = ty + 46
        out.append(f'<rect x="{tx+16}" y="{th_y}" width="{tw-32}" height="32" rx="3" fill="{T["bg_subtle"]}"/>')
        cols = [
            ("ID INCIDENTE", 32), ("TIMESTAMP", 150), ("VECTOR DE AMENAZA", 300),
            ("IP ORIGEN / AS", 520), ("IP DESTINO", 740), ("SCORE ML", 900),
            ("ACCIÓN APLICADA", 1040), ("ESTADO", 1240)
        ]
        for col_name, cx_pos in cols:
            out.append(f'<text x="{tx+cx_pos}" y="{th_y+20}" font-family="{FONT_SANS}" font-size="10" font-weight="700" fill="{T["text_muted"]}">{col_name}</text>')

        # Filas de Alertas
        rows = [
            ("INC-2026-8841", "14:44:12", "DDoS SYN Flood Volumétrico", "198.51.100.44 (AS15169)", "10.10.20.5", "0.998", "eBPF XDP DROP", "MITIGADO", T["critical"]),
            ("INC-2026-8840", "14:42:05", "DNS Amplification Reflection", "203.0.113.89 (AS13335)", "10.10.20.12", "0.985", "BGP Flowspec Rate-Limit", "ACTIVO", T["warning"]),
            ("INC-2026-8839", "14:38:50", "Port Scan Estructurado", "185.220.101.5 (AS60729)", "172.16.10.1", "0.942", "ACL Blackhole Temporal", "RESUELTO", T["success"]),
            ("INC-2026-8838", "14:30:11", "UDP Burst Anomaly", "192.0.2.77 (AS3356)", "10.10.20.5", "0.912", "Inspección Sandbox ACL", "EVALUANDO", T["sandbox"]),
        ]
        for ri, (r_id, r_time, r_type, r_src, r_dst, r_ml, r_action, r_st, r_col) in enumerate(rows):
            ry_row = th_y + 40 + ri * 46
            out.append(f'<line x1="{tx+16}" y1="{ry_row+38}" x2="{tx+tw-16}" y2="{ry_row+38}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
            out.append(f'<text x="{tx+32}" y="{ry_row+24}" font-family="{FONT_MONO}" font-size="11" font-weight="700" fill="{T["text_primary"]}">{r_id}</text>')
            out.append(f'<text x="{tx+150}" y="{ry_row+24}" font-family="{FONT_MONO}" font-size="11" fill="{T["text_secondary"]}">{r_time}</text>')
            out.append(f'<text x="{tx+300}" y="{ry_row+24}" font-family="{FONT_SANS}" font-size="11.5" font-weight="600" fill="{T["text_primary"]}">{r_type}</text>')
            out.append(f'<text x="{tx+520}" y="{ry_row+24}" font-family="{FONT_MONO}" font-size="11" fill="{T["text_secondary"]}">{r_src}</text>')
            out.append(f'<text x="{tx+740}" y="{ry_row+24}" font-family="{FONT_MONO}" font-size="11" fill="{T["text_secondary"]}">{r_dst}</text>')
            out.append(f'<text x="{tx+900}" y="{ry_row+24}" font-family="{FONT_MONO}" font-size="11" font-weight="700" fill="{r_col}">{r_ml}</text>')
            out.append(f'<rect x="{tx+1036}" y="{ry_row+8}" width="140" height="24" rx="3" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="1"/>')
            out.append(f'<text x="{tx+1106}" y="{ry_row+24}" font-family="{FONT_MONO}" font-size="9.5" font-weight="600" fill="{T["text_primary"]}" text-anchor="middle">{r_action}</text>')
            out.append(f'<rect x="{tx+1236}" y="{ry_row+8}" width="90" height="24" rx="3" fill="{T["success_bg"] if r_st=="MITIGADO" or r_st=="RESUELTO" else T["warning_bg"]}" stroke="{r_col}" stroke-width="0.8"/>')
            out.append(f'<text x="{tx+1281}" y="{ry_row+24}" font-family="{FONT_MONO}" font-size="9.5" font-weight="700" fill="{r_col}" text-anchor="middle">{r_st}</text>')

        out.append('</svg>')
        return "".join(out)

    elif device == "tablet":
        w, h = 1024, 768
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        out.append(render_header_tablet(T, "dashboard"))
        
        # 4 KPIs compactos
        kpis = [("Tráfico ISP", "4.28 Gbps", T["accent"]), ("Inspección", "1.84M pps", T["success"]), ("Mitigadas", "14 Eventos", T["critical"]), ("LightGBM", "99.42%", T["sandbox"])]
        card_w = (1024 - 40 - 36) / 4
        for ki, (t_lbl, v_lbl, col) in enumerate(kpis):
            kx = 20 + ki * (card_w + 12)
            out.append(f'<rect x="{kx}" y="70" width="{card_w}" height="76" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
            out.append(f'<text x="{kx+14}" y="94" font-family="{FONT_SANS}" font-size="11" fill="{T["text_muted"]}">{t_lbl}</text>')
            out.append(f'<text x="{kx+14}" y="126" font-family="{FONT_SANS}" font-size="18" font-weight="800" fill="{col}">{v_lbl}</text>')
        
        # Gráfico Tablet
        gx, gy, gw, gh = 20, 160, 984, 280
        out.append(f'<rect x="{gx}" y="{gy}" width="{gw}" height="{gh}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{gx+16}" y="{gy+26}" font-family="{FONT_SANS}" font-size="13" font-weight="700" fill="{T["text_primary"]}">Telemetría de Red y Detección de Ataques (Tablet)</text>')
        
        # Simulación de curva
        out.append(f'<polyline points="{gx+40},{gy+220} {gx+200},{gy+180} {gx+400},{gy+190} {gx+550},{gy+60} {gx+700},{gy+160} {gx+940},{gy+140}" fill="none" stroke="{T["accent"]}" stroke-width="2.5"/>')
        out.append(f'<circle cx="{gx+550}" cy="{gy+60}" r="5" fill="{T["critical"]}"/>')
        out.append(f'<text x="{gx+550}" y="{gy+48}" font-family="{FONT_MONO}" font-size="10" font-weight="700" fill="{T["critical"]}" text-anchor="middle">Ataque SYN Mitigado</text>')

        # Tabla compacta
        tx, ty, tw, th = 20, 456, 984, 290
        out.append(f'<rect x="{tx}" y="{ty}" width="{tw}" height="{th}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{tx+16}" y="{ty+26}" font-family="{FONT_SANS}" font-size="13" font-weight="700" fill="{T["text_primary"]}">Incidentes Recientes</text>')
        
        for ri, (inc, time, vec, src, st) in enumerate([("INC-8841", "14:44", "SYN Flood", "198.51.100.44", "MITIGADO"), ("INC-8840", "14:42", "DNS Amplif.", "203.0.113.89", "ACTIVO")]):
            ry_row = ty + 46 + ri * 52
            out.append(f'<rect x="{tx+12}" y="{ry_row}" width="{tw-24}" height="44" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="1"/>')
            out.append(f'<text x="{tx+24}" y="{ry_row+26}" font-family="{FONT_MONO}" font-size="11" font-weight="700" fill="{T["text_primary"]}">{inc}</text>')
            out.append(f'<text x="{tx+130}" y="{ry_row+26}" font-family="{FONT_SANS}" font-size="11" fill="{T["text_secondary"]}">{vec} ({src})</text>')
            out.append(f'<text x="{tx+tw-40}" y="{ry_row+26}" font-family="{FONT_MONO}" font-size="10" font-weight="700" fill="{T["success"] if st=="MITIGADO" else T["warning"]}" text-anchor="end">{st}</text>')

        out.append('</svg>')
        return "".join(out)

    else: # mobile (390x844)
        w, h = 390, 844
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        out.append(render_header_mobile(T, "Dashboard SOC"))
        
        # 2 Tarjetas KPI Principales
        out.append(f'<rect x="16" y="100" width="170" height="74" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="28" y="122" font-family="{FONT_SANS}" font-size="10.5" fill="{T["text_muted"]}">Tráfico ISP</text>')
        out.append(f'<text x="28" y="152" font-family="{FONT_SANS}" font-size="18" font-weight="800" fill="{T["accent"]}">4.28 Gbps</text>')
        
        out.append(f'<rect x="204" y="100" width="170" height="74" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="216" y="122" font-family="{FONT_SANS}" font-size="10.5" fill="{T["text_muted"]}">Mitigadas Hoy</text>')
        out.append(f'<text x="216" y="152" font-family="{FONT_SANS}" font-size="18" font-weight="800" fill="{T["critical"]}">14 Alertas</text>')
        
        # Gráfico Móvil Simplificado
        out.append(f'<rect x="16" y="186" width="358" height="220" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="30" y="210" font-family="{FONT_SANS}" font-size="12" font-weight="700" fill="{T["text_primary"]}">Tráfico en Vivo (Gbps)</text>')
        out.append(f'<polyline points="30,360 80,330 140,340 200,240 250,320 340,300" fill="none" stroke="{T["accent"]}" stroke-width="2.2"/>')
        out.append(f'<circle cx="200" cy="240" r="4" fill="{T["critical"]}"/>')
        out.append(f'<text x="200" y="230" font-family="{FONT_MONO}" font-size="9" font-weight="700" fill="{T["critical"]}" text-anchor="middle">SYN Drop</text>')

        # Lista de Incidentes Móvil
        out.append(f'<rect x="16" y="420" width="358" height="400" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="30" y="444" font-family="{FONT_SANS}" font-size="12" font-weight="700" fill="{T["text_primary"]}">Alertas Recientes</text>')
        
        incidents = [
            ("INC-8841", "DDoS SYN Flood", "198.51.100.44", "eBPF DROP", T["critical"]),
            ("INC-8840", "DNS Amplification", "203.0.113.89", "Flowspec", T["warning"]),
            ("INC-8839", "Port Scan", "185.220.101.5", "Blackhole", T["success"])
        ]
        for ii, (iid, itype, isrc, iact, icol) in enumerate(incidents):
            iy = 460 + ii * 66
            out.append(f'<rect x="28" y="{iy}" width="334" height="54" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="1"/>')
            out.append(f'<circle cx="42" cy="{iy+27}" r="4" fill="{icol}"/>')
            out.append(f'<text x="56" y="{iy+22}" font-family="{FONT_MONO}" font-size="11" font-weight="700" fill="{T["text_primary"]}">{iid} • {itype}</text>')
            out.append(f'<text x="56" y="{iy+40}" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_muted"]}">Src: {isrc} • Acción: {iact}</text>')

        out.append('</svg>')
        return "".join(out)

def gen_screen_09_sandbox_acl(T, device="pc"):
    """
    PANTALLA NUEVA REQUERIDA: Sandbox ACL & Reglas Dinámicas de Mitigación
    - Subred aislada simulada (10.200.0.0/24 - VLAN 999 ISP Sandbox).
    - Inyector de tráfico de ataque vs tráfico de control benigno.
    - Consola de compilación de reglas eBPF / iptables.
    - Telemetría en tiempo real: Tasa de mitigación, Falsos positivos (0.0%), Impacto en latencia.
    - Flujo de gobernanza: [ Probar en Sandbox ] -> [ Descartar ] o [ Promover a Producción ].
    """
    if device == "pc":
        w, h = 1440, 900
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        out.append(render_header_pc(T, "sandbox_acl"))

        # Banner Superior de Entorno Aislado
        bx, by, bw, bh = 24, 76, 1392, 58
        out.append(f'<rect x="{bx}" y="{by}" width="{bw}" height="{bh}" rx="6" fill="{T["sandbox_bg"]}" stroke="{T["sandbox_border"]}" stroke-width="1.2"/>')
        out.append(icon_flask(bx+20, by+16, 26, T["sandbox"]))
        out.append(f'<text x="{bx+58}" y="{by+26}" font-family="{FONT_SANS}" font-size="13" font-weight="800" fill="{T["sandbox_text"]}">ENTORNO AISLADO DE PRUEBAS SANDBOX • SUBRED DE SIMULACIÓN: 10.200.0.0/24 (VLAN 999 ISOLATED)</text>')
        out.append(f'<text x="{bx+58}" y="{by+44}" font-family="{FONT_SANS}" font-size="11" font-weight="500" fill="{T["sandbox_text"]}">Topología virtualizada veth-isolated sin conexión a la red de producción del ISP. Las reglas ejecutadas aquí no afectan el tráfico de clientes reales.</text>')
        
        # Chip de Estado del Sandbox
        out.append(f'<rect x="{bx+bw-220}" y="{by+14}" width="200" height="30" rx="4" fill="{T["bg_panel"]}" stroke="{T["sandbox"]}" stroke-width="1"/>')
        out.append(f'<circle cx="{bx+bw-205}" cy="{by+29}" r="4" fill="{T["success"]}"/>')
        out.append(f'<text x="{bx+bw-192}" y="{by+33}" font-family="{FONT_MONO}" font-size="10.5" font-weight="700" fill="{T["text_primary"]}">MOTOR eBPF: LISTO</text>')

        # Panel 1: Inyector de Tráfico Simulado al Sandbox (Izquierda Superior)
        p1_x, p1_y, p1_w, p1_h = 24, 146, 684, 250
        out.append(f'<rect x="{p1_x}" y="{p1_y}" width="{p1_w}" height="{p1_h}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{p1_x+20}" y="{p1_y+28}" font-family="{FONT_SANS}" font-size="13.5" font-weight="800" fill="{T["text_primary"]}">1. Generador de Tráfico de Ataque y Carga de Control</text>')
        out.append(f'<text x="{p1_x+20}" y="{p1_y+44}" font-family="{FONT_SANS}" font-size="10.5" fill="{T["text_muted"]}">Configure los flujos sintéticos para validar la regla antes de autorizar en producción</text>')

        # Selector de Vector de Ataque
        out.append(f'<text x="{p1_x+20}" y="{p1_y+72}" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="{T["text_primary"]}">Vector de Ataque Sintético:</text>')
        out.append(f'<rect x="{p1_x+20}" y="{p1_y+80}" width="{p1_w-40}" height="36" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{p1_x+34}" y="{p1_y+103}" font-family="{FONT_MONO}" font-size="11.5" fill="{T["text_primary"]}">SYN Flood L4 Volumétrico (Tasa objetivo: 500,000 pps hacia 10.200.0.50)</text>')

        # Selector de Tráfico Benigno de Fondo
        out.append(f'<text x="{p1_x+20}" y="{p1_y+136}" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="{T["text_primary"]}">Tráfico Benigno Legítimo de Control (Validación de Falsos Positivos):</text>')
        out.append(f'<rect x="{p1_x+20}" y="{p1_y+144}" width="{p1_w-40}" height="36" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{p1_x+34}" y="{p1_y+167}" font-family="{FONT_MONO}" font-size="11.5" fill="{T["text_primary"]}">Flujo HTTP/HTTPS Mixto Clientes ISP (100,000 pps de control)</text>')

        # Botón Inyectar
        btn_inj_y = p1_y + 194
        out.append(f'<rect x="{p1_x+20}" y="{btn_inj_y}" width="260" height="38" rx="4" fill="{T["accent"]}"/>')
        out.append(icon_play(p1_x+32, btn_inj_y+11, 16, "#FFFFFF"))
        out.append(f'<text x="{p1_x+145}" y="{btn_inj_y+24}" font-family="{FONT_SANS}" font-size="11.5" font-weight="700" fill="#FFFFFF" text-anchor="middle">Inyectar Tráfico de Prueba al Sandbox</text>')
        
        # Estado de Simulación
        out.append(f'<rect x="{p1_x+295}" y="{btn_inj_y}" width="{p1_w-315}" height="38" rx="4" fill="{T["success_bg"]}" stroke="{T["success_border"]}" stroke-width="1"/>')
        out.append(f'<circle cx="{p1_x+310}" cy="{btn_inj_y+19}" r="3.5" fill="{T["success"]}"/>')
        out.append(f'<text x="{p1_x+322}" y="{btn_inj_y+23}" font-family="{FONT_MONO}" font-size="10.5" font-weight="700" fill="{T["success_text"]}">INYECCIÓN ACTIVA: 600k pps</text>')

        # Panel 2: Editor y Compilador de Reglas Dinámicas ACL (Derecha Superior)
        p2_x, p2_y, p2_w, p2_h = 732, 146, 684, 250
        out.append(f'<rect x="{p2_x}" y="{p2_y}" width="{p2_w}" height="{p2_h}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{p2_x+20}" y="{p2_y+28}" font-family="{FONT_SANS}" font-size="13.5" font-weight="800" fill="{T["text_primary"]}">2. Consola de Definición de Reglas Dinámicas de Mitigación</text>')
        out.append(f'<text x="{p2_x+20}" y="{p2_y+44}" font-family="{FONT_SANS}" font-size="10.5" fill="{T["text_muted"]}">Sintaxis eBPF XDP y comandos iptables RAW validados en el kernel del router sandbox</text>')

        # Caja de Código de la Regla
        code_y = p2_y + 56
        out.append(f'<rect x="{p2_x+20}" y="{code_y}" width="{p2_w-40}" height="126" rx="4" fill="{T["code_bg"]}"/>')
        code_lines = [
            "# REGLA DINÁMICA ACL EVALUADA EN SANDBOX (veth-isolated):",
            "iptables -t raw -A PREROUTING -i veth-sb0 -s 10.200.0.128/28 -p tcp --syn -j DROP",
            "# Compilación eBPF XDP Bypass (Kernel 6.8):",
            "SEC(\"xdp\") int xdp_sandbox_filter(struct xdp_md *ctx) {",
            "    if (iph->saddr == htonl(0x0AC80080) && tcph->syn) return XDP_DROP;",
            "    return XDP_PASS; }"
        ]
        for cli, cline in enumerate(code_lines):
            col_line = T["code_text"] if cli > 0 else "#64748B"
            out.append(f'<text x="{p2_x+32}" y="{code_y+20+cli*18}" font-family="{FONT_MONO}" font-size="10" fill="{col_line}">{esc(cline)}</text>')

        # Botones de Ejecución Sandbox
        btn_eval_y = code_y + 138
        out.append(f'<rect x="{p2_x+20}" y="{btn_eval_y}" width="220" height="38" rx="4" fill="{T["sandbox"]}"/>')
        out.append(icon_terminal(p2_x+30, btn_eval_y+11, 16, "#FFFFFF"))
        out.append(f'<text x="{p2_x+125}" y="{btn_eval_y+24}" font-family="{FONT_SANS}" font-size="11.5" font-weight="700" fill="#FFFFFF" text-anchor="middle">Ejecutar Regla en Sandbox</text>')

        out.append(f'<rect x="{p2_x+250}" y="{btn_eval_y}" width="180" height="38" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{p2_x+340}" y="{btn_eval_y+24}" font-family="{FONT_SANS}" font-size="11" font-weight="600" fill="{T["text_secondary"]}" text-anchor="middle">Rollback de Regla</text>')

        # Panel 3: Verificación y Telemetría de Impacto en Sandbox (Mitad Inferior)
        p3_x, p3_y, p3_w, p3_h = 24, 412, 1392, 380
        out.append(f'<rect x="{p3_x}" y="{p3_y}" width="{p3_w}" height="{p3_h}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{p3_x+20}" y="{p3_y+28}" font-family="{FONT_SANS}" font-size="14" font-weight="800" fill="{T["text_primary"]}">3. Métricas de Validación en Tiempo Real &amp; Certificación de Regla</text>')
        out.append(f'<text x="{p3_x+20}" y="{p3_y+44}" font-family="{FONT_SANS}" font-size="10.5" fill="{T["text_muted"]}">Evaluación matemática del impacto antes de autorizar el despliegue al backbone de producción de Tigo</text>')

        # 3 Métricas Clave de Validación
        vm_w = (p3_w - 60) / 3
        vmetrics = [
            ("Eficacia de Bloqueo de Ataque", "99.85%", "499,250 de 500,000 pps maliciosos descartados", T["success"], icon_shield),
            ("Tasa de Falsos Positivos", "0.00%", "100,000 de 100,000 paquetes benignos pasaron intactos", T["success"], icon_check),
            ("Sobrecarga de Latencia en Router", "+0.016 ms", "Impacto imperceptible en CPU (Kernel XDP Bypass)", T["accent"], icon_cpu)
        ]
        for vmi, (vtitle, vval, vsub, vcol, vicon) in enumerate(vmetrics):
            vx = p3_x + 20 + vmi * (vm_w + 10)
            out.append(f'<rect x="{vx}" y="{p3_y+58}" width="{vm_w}" height="90" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="1"/>')
            out.append(vicon(vx+16, p3_y+72, 20, vcol))
            out.append(f'<text x="{vx+44}" y="{p3_y+84}" font-family="{FONT_SANS}" font-size="11" font-weight="600" fill="{T["text_muted"]}">{esc(vtitle)}</text>')
            out.append(f'<text x="{vx+16}" y="{p3_y+118}" font-family="{FONT_SANS}" font-size="22" font-weight="800" fill="{vcol}">{esc(vval)}</text>')
            out.append(f'<text x="{vx+16}" y="{p3_y+136}" font-family="{FONT_MONO}" font-size="9" fill="{T["text_secondary"]}">{esc(vsub)}</text>')

        # Veredicto de Certificación del Sandbox
        vy = p3_y + 164
        out.append(f'<rect x="{p3_x+20}" y="{vy}" width="{p3_w-40}" height="96" rx="6" fill="{T["success_bg"]}" stroke="{T["success_border"]}" stroke-width="1.2"/>')
        out.append(icon_check(p3_x+36, vy+20, 28, T["success"]))
        out.append(f'<text x="{p3_x+74}" y="{vy+32}" font-family="{FONT_SANS}" font-size="13" font-weight="800" fill="{T["success_text"]}">CERTIFICACIÓN DE SEGURIDAD APROBADA: REGLA SEGURA PARA DESPLIEGUE EN PRODUCCIÓN</text>')
        out.append(f'<text x="{p3_x+74}" y="{vy+50}" font-family="{FONT_SANS}" font-size="11" font-weight="500" fill="{T["success_text"]}">• Cero colisiones con clientes residenciales o corporativos (Prefijo objetivo validado en subred de simulación 10.200.0.0/24).</text>')
        out.append(f'<text x="{p3_x+74}" y="{vy+66}" font-family="{FONT_SANS}" font-size="11" font-weight="500" fill="{T["success_text"]}">• Eficacia probada del 99.85% contra el vector de ataque sin requerir consumo de recursos en hardware fuera del kernel.</text>')
        out.append(f'<text x="{p3_x+74}" y="{vy+82}" font-family="{FONT_MONO}" font-size="10" font-weight="600" fill="{T["text_muted"]}">Firma Criptográfica Sandbox: sha256:7f4a98d02bc149e918c5e6273411a0bb38e24c</text>')

        # Barra de Gobernanza y Aprobación
        gy_bar = vy + 112
        out.append(f'<line x1="{p3_x+20}" y1="{gy_bar}" x2="{p3_x+p3_w-20}" y2="{gy_bar}" stroke="{T["border_subtle"]}" stroke-width="1"/>')
        
        # Información de Auditoría
        out.append(f'<text x="{p3_x+20}" y="{gy_bar+32}" font-family="{FONT_MONO}" font-size="11" fill="{T["text_secondary"]}">Operador Evaluador: <tspan font-weight="700" fill="{T["text_primary"]}">carlos.mendoza [admin_soc]</tspan> • Sesión TLS 1.3 Verificada</text>')

        # Botón Descartar
        btn_disc_x = p3_x + p3_w - 560
        out.append(f'<rect x="{btn_disc_x}" y="{gy_bar+12}" width="200" height="42" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{btn_disc_x+100}" y="{gy_bar+37}" font-family="{FONT_SANS}" font-size="12" font-weight="600" fill="{T["text_secondary"]}" text-anchor="middle">Descartar Regla Sandbox</text>')

        # Botón Aprobar y Promover a Producción
        btn_prom_x = p3_x + p3_w - 340
        out.append(f'<rect x="{btn_prom_x}" y="{gy_bar+12}" width="320" height="42" rx="4" fill="{T["success"]}"/>')
        out.append(icon_shield(btn_prom_x+18, gy_bar+23, 20, "#FFFFFF"))
        out.append(f'<text x="{btn_prom_x+170}" y="{gy_bar+37}" font-family="{FONT_SANS}" font-size="12.5" font-weight="800" fill="#FFFFFF" text-anchor="middle">Aprobar y Promover a Producción</text>')

        out.append('</svg>')
        return "".join(out)

    elif device == "tablet":
        w, h = 1024, 768
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        out.append(render_header_tablet(T, "sandbox_acl"))
        
        # Banner Tablet
        out.append(f'<rect x="20" y="70" width="984" height="46" rx="4" fill="{T["sandbox_bg"]}" stroke="{T["sandbox_border"]}" stroke-width="1"/>')
        out.append(icon_flask(28, 80, 20, T["sandbox"]))
        out.append(f'<text x="56" y="94" font-family="{FONT_SANS}" font-size="12" font-weight="800" fill="{T["sandbox_text"]}">SANDBOX ACL: Subred Aislada 10.200.0.0/24 (VLAN 999) • Pruebas Seguras</text>')
        
        # Panel Superior de Regla
        out.append(f'<rect x="20" y="126" width="984" height="150" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="36" y="150" font-family="{FONT_SANS}" font-size="12" font-weight="700" fill="{T["text_primary"]}">Regla Dinámica eBPF / iptables:</text>')
        out.append(f'<rect x="36" y="160" width="952" height="60" rx="4" fill="{T["code_bg"]}"/>')
        out.append(f'<text x="48" y="186" font-family="{FONT_MONO}" font-size="10.5" fill="{T["code_text"]}">iptables -t raw -A PREROUTING -i veth-sb0 -s 10.200.0.128/28 -p tcp --syn -j DROP</text>')
        out.append(f'<text x="48" y="206" font-family="{FONT_MONO}" font-size="10" fill="#64748B"># XDP Bypass: OK • Compilación en Kernel 6.8 exitosa</text>')
        
        # Métricas
        out.append(f'<rect x="20" y="288" width="984" height="340" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="36" y="314" font-family="{FONT_SANS}" font-size="13" font-weight="700" fill="{T["text_primary"]}">Resultados de la Simulación en Sandbox</text>')
        
        for mi, (mlbl, mval, col) in enumerate([("Mitigación", "99.85%", T["success"]), ("Falsos Positivos", "0.00%", T["success"]), ("Latencia", "+0.016 ms", T["accent"])]):
            mx = 36 + mi * 318
            out.append(f'<rect x="{mx}" y="330" width="300" height="70" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="1"/>')
            out.append(f'<text x="{mx+16}" y="354" font-family="{FONT_SANS}" font-size="11" fill="{T["text_muted"]}">{mlbl}</text>')
            out.append(f'<text x="{mx+16}" y="384" font-family="{FONT_SANS}" font-size="20" font-weight="800" fill="{col}">{mval}</text>')
            
        out.append(f'<rect x="36" y="420" width="952" height="68" rx="4" fill="{T["success_bg"]}" stroke="{T["success_border"]}" stroke-width="1"/>')
        out.append(icon_check(48, 436, 22, T["success"]))
        out.append(f'<text x="80" y="450" font-family="{FONT_SANS}" font-size="12" font-weight="800" fill="{T["success_text"]}">CERTIFICACIÓN EXITOSA: REGLA LISTA PARA PRODUCCIÓN</text>')
        out.append(f'<text x="80" y="468" font-family="{FONT_SANS}" font-size="10.5" fill="{T["success_text"]}">No afecta tráfico legítimo ni satura hardware.</text>')
        
        # Botones
        out.append(f'<rect x="36" y="520" width="220" height="42" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="146" y="546" font-family="{FONT_SANS}" font-size="11.5" font-weight="600" fill="{T["text_secondary"]}" text-anchor="middle">Descartar</text>')
        out.append(f'<rect x="280" y="520" width="340" height="42" rx="4" fill="{T["success"]}"/>')
        out.append(f'<text x="450" y="546" font-family="{FONT_SANS}" font-size="12" font-weight="700" fill="#FFFFFF" text-anchor="middle">Promover Regla a Producción</text>')
        
        out.append('</svg>')
        return "".join(out)

    else: # mobile (390x844)
        w, h = 390, 844
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        out.append(render_header_mobile(T, "Sandbox ACL"))
        
        # Banner móvil
        out.append(f'<rect x="16" y="100" width="358" height="54" rx="4" fill="{T["sandbox_bg"]}" stroke="{T["sandbox_border"]}" stroke-width="1"/>')
        out.append(icon_flask(24, 112, 18, T["sandbox"]))
        out.append(f'<text x="48" y="122" font-family="{FONT_SANS}" font-size="10" font-weight="800" fill="{T["sandbox_text"]}">SANDBOX AISLADO 10.200.0.0/24</text>')
        out.append(f'<text x="48" y="138" font-family="{FONT_SANS}" font-size="8.5" fill="{T["sandbox_text"]}">Simulación sin riesgo para producción.</text>')
        
        # Regla
        out.append(f'<rect x="16" y="164" width="358" height="130" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="28" y="184" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="{T["text_primary"]}">Regla Dinámica Evaluada:</text>')
        out.append(f'<rect x="28" y="194" width="334" height="52" rx="4" fill="{T["code_bg"]}"/>')
        out.append(f'<text x="36" y="214" font-family="{FONT_MONO}" font-size="8.5" fill="{T["code_text"]}">iptables -t raw -A PREROUTING</text>')
        out.append(f'<text x="36" y="228" font-family="{FONT_MONO}" font-size="8.5" fill="{T["code_text"]}">-s 10.200.0.128/28 -p tcp --syn -j DROP</text>')
        
        # Botón Probar
        out.append(f'<rect x="28" y="254" width="334" height="30" rx="4" fill="{T["sandbox"]}"/>')
        out.append(f'<text x="195" y="273" font-family="{FONT_SANS}" font-size="10.5" font-weight="700" fill="#FFFFFF" text-anchor="middle">Ejecutar Simulación</text>')

        # Métricas Móvil
        out.append(f'<rect x="16" y="304" width="358" height="420" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="28" y="328" font-family="{FONT_SANS}" font-size="11.5" font-weight="700" fill="{T["text_primary"]}">Telemetría de Validación</text>')
        
        for mi, (mlbl, mval, col) in enumerate([("Eficacia Mitigación", "99.85%", T["success"]), ("Falsos Positivos", "0.00%", T["success"]), ("Latencia Extra", "+0.016 ms", T["accent"])]):
            my = 342 + mi * 54
            out.append(f'<rect x="28" y="{my}" width="334" height="46" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="1"/>')
            out.append(f'<text x="40" y="{my+20}" font-family="{FONT_SANS}" font-size="10" fill="{T["text_muted"]}">{mlbl}</text>')
            out.append(f'<text x="40" y="{my+38}" font-family="{FONT_SANS}" font-size="15" font-weight="800" fill="{col}">{mval}</text>')

        # Certificado y Botón Móvil
        out.append(f'<rect x="28" y="520" width="334" height="60" rx="4" fill="{T["success_bg"]}" stroke="{T["success_border"]}" stroke-width="1"/>')
        out.append(icon_check(36, 532, 18, T["success"]))
        out.append(f'<text x="60" y="544" font-family="{FONT_SANS}" font-size="10.5" font-weight="800" fill="{T["success_text"]}">CERTIFICADA PARA PRODUCCIÓN</text>')
        out.append(f'<text x="60" y="558" font-family="{FONT_SANS}" font-size="9" fill="{T["success_text"]}">Zero-Trust &amp; ISO 27001 aprobados.</text>')
        
        out.append(f'<rect x="28" y="600" width="334" height="44" rx="4" fill="{T["success"]}"/>')
        out.append(f'<text x="195" y="627" font-family="{FONT_SANS}" font-size="12" font-weight="800" fill="#FFFFFF" text-anchor="middle">Promover a Producción</text>')
        out.append(f'<rect x="28" y="654" width="334" height="38" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="195" y="677" font-family="{FONT_SANS}" font-size="11" font-weight="600" fill="{T["text_secondary"]}" text-anchor="middle">Descartar Regla</text>')

        out.append('</svg>')
        return "".join(out)

def gen_screen_03_topology(T, device="pc"):
    """PANTALLA 3: Topología de Red y Nodos de Infraestructura ISP (GNS3 Core)"""
    if device == "pc":
        w, h = 1440, 900
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        out.append(render_header_pc(T, "topology"))

        # Canvas de Topología
        tx, ty, tw, th = 24, 76, 1020, 800
        out.append(f'<rect x="{tx}" y="{ty}" width="{tw}" height="{th}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{tx+20}" y="{ty+28}" font-family="{FONT_SANS}" font-size="14" font-weight="800" fill="{T["text_primary"]}">Topología de Infraestructura ISP &amp; Estado de Sensores eBPF</text>')
        out.append(f'<text x="{tx+20}" y="{ty+44}" font-family="{FONT_SANS}" font-size="10.5" fill="{T["text_muted"]}">Nodos de transporte, agregación FTTH, honeypots y subred aislada de simulación (AS27839)</text>')

        # Controles de Zoom y Filtros
        out.append(f'<rect x="{tx+tw-240}" y="{ty+18}" width="220" height="28" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="1"/>')
        out.append(f'<text x="{tx+tw-220}" y="{ty+36}" font-family="{FONT_SANS}" font-size="10.5" font-weight="600" fill="{T["text_secondary"]}">Zoom: 100% • [ + ] [ - ] [ Centrar ]</text>')

        # Enlaces de Red (Líneas con capacidades)
        links = [
            (tx+450, ty+180, tx+250, ty+320, "10G Fiber", T["success"]),
            (tx+450, ty+180, tx+650, ty+320, "40G Core", T["success"]),
            (tx+250, ty+320, tx+150, ty+500, "10G GPON", T["success"]),
            (tx+250, ty+320, tx+350, ty+500, "10G Metro", T["success"]),
            (tx+650, ty+320, tx+550, ty+500, "10G Server", T["success"]),
            (tx+650, ty+320, tx+780, ty+500, "Ataque L4", T["critical"]),
            (tx+450, ty+180, tx+450, ty+660, "VLAN 999 Sandbox", T["sandbox"])
        ]
        for lx1, ly1, lx2, ly2, label, col in links:
            dash = ' stroke-dasharray="4,3"' if col == T["sandbox"] or col == T["critical"] else ''
            out.append(f'<line x1="{lx1}" y1="{ly1}" x2="{lx2}" y2="{ly2}" stroke="{col}" stroke-width="2"{dash}/>')
            mid_x, mid_y = (lx1 + lx2) // 2, (ly1 + ly2) // 2
            out.append(f'<rect x="{mid_x-36}" y="{mid_y-9}" width="72" height="18" rx="3" fill="{T["bg_panel"]}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
            out.append(f'<text x="{mid_x}" y="{mid_y+4}" font-family="{FONT_MONO}" font-size="8.5" font-weight="600" fill="{col}" text-anchor="middle">{esc(label)}</text>')

        # Nodos
        nodes = [
            ("Core-Router-01", "172.16.10.1", tx+450, ty+180, T["accent"], "AS27839 Gateway"),
            ("Carrier-Peering-01", "198.51.100.1", tx+250, ty+320, T["success"], "Upstream ISP"),
            ("DC-Backend-Tigo", "10.10.0.1", tx+650, ty+320, T["success"], "Data Center Core"),
            ("OLT-ZonaSur-FTTH", "10.20.10.1", tx+150, ty+500, T["success"], "Acceso Residencial"),
            ("OLT-Centro-FTTH", "10.20.20.1", tx+350, ty+500, T["success"], "Acceso Corporativo"),
            ("Honeypot-Sensor-01", "172.28.0.10", tx+550, ty+500, T["warning"], "Decoy Monitoreado"),
            ("Attacker-Sim (Kali)", "198.51.100.44", tx+780, ty+500, T["critical"], "Generador de Ataque"),
            ("Sandbox-Router-SB0", "10.200.0.1", tx+450, ty+660, T["sandbox"], "Subred Aislada VLAN 999")
        ]
        for name, ip, nx, ny, col, role in nodes:
            out.append(f'<rect x="{nx-70}" y="{ny-32}" width="140" height="64" rx="6" fill="{T["bg_panel"]}" stroke="{col}" stroke-width="1.8"/>')
            out.append(f'<circle cx="{nx}" cy="{ny-10}" r="8" fill="{col}"/>')
            out.append(f'<text x="{nx}" y="{ny+12}" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="{T["text_primary"]}" text-anchor="middle">{esc(name)}</text>')
            out.append(f'<text x="{nx}" y="{ny+24}" font-family="{FONT_MONO}" font-size="9" fill="{T["text_muted"]}" text-anchor="middle">{esc(ip)}</text>')

        # Panel Inspector Lateral Derecho (Detalles del Nodo Seleccionado)
        ix, iy, iw, ih = 1060, 76, 356, 800
        out.append(f'<rect x="{ix}" y="{iy}" width="{iw}" height="{ih}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{ix+20}" y="{iy+28}" font-family="{FONT_SANS}" font-size="14" font-weight="800" fill="{T["text_primary"]}">Inspector de Nodo de Red</text>')
        out.append(f'<text x="{ix+20}" y="{iy+44}" font-family="{FONT_SANS}" font-size="10.5" fill="{T["text_muted"]}">Telemetría profunda del elemento seleccionado</text>')

        # Tarjeta de Nodo Seleccionado
        out.append(f'<rect x="{ix+16}" y="{iy+60}" width="{iw-32}" height="110" rx="6" fill="{T["bg_subtle"]}" stroke="{T["accent"]}" stroke-width="1.2"/>')
        out.append(icon_cpu(ix+28, iy+72, 22, T["accent"]))
        out.append(f'<text x="{ix+58}" y="{iy+84}" font-family="{FONT_SANS}" font-size="13" font-weight="800" fill="{T["text_primary"]}">Core-Router-01 (VyOS / BGP)</text>')
        out.append(f'<text x="{ix+58}" y="{iy+100}" font-family="{FONT_MONO}" font-size="11" fill="{T["text_secondary"]}">IP: 172.16.10.1 • AS27839</text>')
        out.append(f'<rect x="{ix+28}" y="{iy+116}" width="80" height="22" rx="3" fill="{T["success_bg"]}" stroke="{T["success_border"]}" stroke-width="0.8"/>')
        out.append(f'<text x="{ix+68}" y="{iy+131}" font-family="{FONT_MONO}" font-size="9.5" font-weight="700" fill="{T["success_text"]}" text-anchor="middle">ONLINE</text>')
        out.append(f'<text x="{ix+120}" y="{iy+131}" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_muted"]}">Filtro eBPF: xdp_flow.o</text>')

        # Estadísticas de Interfaces
        out.append(f'<text x="{ix+20}" y="{iy+196}" font-family="{FONT_SANS}" font-size="12" font-weight="700" fill="{T["text_primary"]}">Interfaces Activas y Carga:</text>')
        ifaces = [
            ("eth0 (Carrier Peering)", "1.84 Gbps", "680k pps", T["success"]),
            ("eth1 (FTTH Distribution)", "1.42 Gbps", "420k pps", T["success"]),
            ("eth2 (DC Interconnect)", "880 Mbps", "180k pps", T["success"]),
            ("eth3 (veth-sandbox)", "0 pps", "Aislado VLAN 999", T["sandbox"])
        ]
        for ifi, (if_name, if_rate, if_pps, if_col) in enumerate(ifaces):
            ify = iy + 210 + ifi * 54
            out.append(f'<rect x="{ix+16}" y="{ify}" width="{iw-32}" height="46" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="1"/>')
            out.append(f'<text x="{ix+28}" y="{ify+20}" font-family="{FONT_MONO}" font-size="10.5" font-weight="700" fill="{T["text_primary"]}">{if_name}</text>')
            out.append(f'<text x="{ix+28}" y="{ify+34}" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_muted"]}">{if_rate} • {if_pps}</text>')
            out.append(f'<circle cx="{ix+iw-36}" cy="{ify+23}" r="4" fill="{if_col}"/>')

        # Acciones Rápidas
        out.append(f'<text x="{ix+20}" y="{iy+450}" font-family="{FONT_SANS}" font-size="12" font-weight="700" fill="{T["text_primary"]}">Acciones de Mitigación eBPF:</text>')
        out.append(f'<rect x="{ix+16}" y="{iy+464}" width="{iw-32}" height="40" rx="4" fill="{T["accent"]}"/>')
        out.append(f'<text x="{ix+iw/2}" y="{iy+489}" font-family="{FONT_SANS}" font-size="11.5" font-weight="700" fill="#FFFFFF" text-anchor="middle">Inspeccionar eBPF Maps en Vivo</text>')

        out.append(f'<rect x="{ix+16}" y="{iy+514}" width="{iw-32}" height="40" rx="4" fill="{T["sandbox_bg"]}" stroke="{T["sandbox_border"]}" stroke-width="1"/>')
        out.append(f'<text x="{ix+iw/2}" y="{iy+539}" font-family="{FONT_SANS}" font-size="11.5" font-weight="700" fill="{T["sandbox_text"]}" text-anchor="middle">Clonar Configuración a Sandbox ACL</text>')

        out.append('</svg>')
        return "".join(out)

    elif device == "tablet":
        w, h = 1024, 768
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        out.append(render_header_tablet(T, "topology"))
        
        # Canvas de Topología (Izquierda: 680x670)
        tx, ty, tw, th = 20, 70, 680, 670
        out.append(f'<rect x="{tx}" y="{ty}" width="{tw}" height="{th}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{tx+16}" y="{ty+26}" font-family="{FONT_SANS}" font-size="13" font-weight="800" fill="{T["text_primary"]}">Topología Core ISP &amp; Sensores eBPF</text>')
        out.append(f'<text x="{tx+16}" y="{ty+42}" font-family="{FONT_SANS}" font-size="10" fill="{T["text_muted"]}">Nodos BGP AS27839 • Enlaces 10G/40G y Subred Aislada</text>')
        
        # Controles de zoom
        out.append(f'<rect x="{tx+tw-170}" y="{ty+14}" width="154" height="26" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="1"/>')
        out.append(f'<text x="{tx+tw-93}" y="{ty+31}" font-family="{FONT_SANS}" font-size="9.5" font-weight="600" fill="{T["text_secondary"]}" text-anchor="middle">Zoom 100% [ + ] [ - ]</text>')

        # Enlaces
        tlinks = [
            (tx+340, ty+140, tx+160, ty+260, "10G Fiber", T["success"]),
            (tx+340, ty+140, tx+520, ty+260, "40G Core", T["success"]),
            (tx+160, ty+260, tx+90, ty+420, "10G GPON", T["success"]),
            (tx+160, ty+260, tx+240, ty+420, "10G Metro", T["success"]),
            (tx+520, ty+260, tx+440, ty+420, "10G Decoy", T["warning"]),
            (tx+520, ty+260, tx+600, ty+420, "Ataque L4", T["critical"]),
            (tx+340, ty+140, tx+340, ty+560, "VLAN 999", T["sandbox"])
        ]
        for lx1, ly1, lx2, ly2, lbl, col in tlinks:
            dash = ' stroke-dasharray="4,3"' if col == T["sandbox"] or col == T["critical"] else ''
            out.append(f'<line x1="{lx1}" y1="{ly1}" x2="{lx2}" y2="{ly2}" stroke="{col}" stroke-width="2"{dash}/>')
            mx, my = (lx1 + lx2)//2, (ly1 + ly2)//2
            out.append(f'<rect x="{mx-26}" y="{my-8}" width="52" height="16" rx="2" fill="{T["bg_panel"]}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
            out.append(f'<text x="{mx}" y="{my+4}" font-family="{FONT_MONO}" font-size="7.5" font-weight="700" fill="{col}" text-anchor="middle">{lbl}</text>')

        # Nodos
        tnodes = [
            ("Core-Router-01", "172.16.10.1", tx+340, ty+140, T["accent"]),
            ("Carrier-Peering", "198.51.100.1", tx+160, ty+260, T["success"]),
            ("DC-Backend", "10.10.0.1", tx+520, ty+260, T["success"]),
            ("OLT-ZonaSur", "10.20.10.1", tx+90, ty+420, T["success"]),
            ("OLT-Centro", "10.20.20.1", tx+240, ty+420, T["success"]),
            ("Honeypot-01", "172.28.0.10", tx+440, ty+420, T["warning"]),
            ("Attacker-Sim", "198.51.100.44", tx+600, ty+420, T["critical"]),
            ("Sandbox-SB0", "10.200.0.1", tx+340, ty+560, T["sandbox"])
        ]
        for name, ip, nx, ny, col in tnodes:
            out.append(f'<rect x="{nx-52}" y="{ny-22}" width="104" height="46" rx="5" fill="{T["bg_panel"]}" stroke="{col}" stroke-width="1.6"/>')
            out.append(f'<circle cx="{nx}" cy="{ny-8}" r="5" fill="{col}"/>')
            out.append(f'<text x="{nx}" y="{ny+8}" font-family="{FONT_SANS}" font-size="9" font-weight="700" fill="{T["text_primary"]}" text-anchor="middle">{name}</text>')
            out.append(f'<text x="{nx}" y="{ny+18}" font-family="{FONT_MONO}" font-size="7.5" fill="{T["text_muted"]}" text-anchor="middle">{ip}</text>')

        # Panel Inspector Lateral Derecho (284x670)
        ix, iy, iw, ih = 720, 70, 284, 670
        out.append(f'<rect x="{ix}" y="{iy}" width="{iw}" height="{ih}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{ix+16}" y="{iy+26}" font-family="{FONT_SANS}" font-size="13" font-weight="800" fill="{T["text_primary"]}">Inspector de Nodo</text>')
        out.append(f'<text x="{ix+16}" y="{iy+42}" font-family="{FONT_SANS}" font-size="9.5" fill="{T["text_muted"]}">Telemetría activa en tiempo real</text>')

        # Tarjeta Nodo Seleccionado
        out.append(f'<rect x="{ix+14}" y="{iy+56}" width="{iw-28}" height="86" rx="5" fill="{T["bg_subtle"]}" stroke="{T["accent"]}" stroke-width="1.2"/>')
        out.append(icon_cpu(ix+24, iy+68, 18, T["accent"]))
        out.append(f'<text x="{ix+48}" y="{iy+78}" font-family="{FONT_SANS}" font-size="11" font-weight="800" fill="{T["text_primary"]}">Core-Router-01 (VyOS)</text>')
        out.append(f'<text x="{ix+48}" y="{iy+92}" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_secondary"]}">172.16.10.1 • AS27839</text>')
        out.append(f'<rect x="{ix+24}" y="{iy+106}" width="60" height="18" rx="2" fill="{T["success_bg"]}" stroke="{T["success_border"]}" stroke-width="0.8"/>')
        out.append(f'<text x="{ix+54}" y="{iy+119}" font-family="{FONT_MONO}" font-size="8.5" font-weight="700" fill="{T["success_text"]}" text-anchor="middle">ONLINE</text>')
        out.append(f'<text x="{ix+96}" y="{iy+119}" font-family="{FONT_MONO}" font-size="8.5" fill="{T["text_muted"]}">eBPF: xdp_flow.o</text>')

        # Interfaces
        out.append(f'<text x="{ix+16}" y="{iy+164}" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="{T["text_primary"]}">Carga de Interfaces:</text>')
        tifaces = [
            ("eth0 (Peering)", "1.84 Gbps", "680k pps", T["success"]),
            ("eth1 (FTTH)", "1.42 Gbps", "420k pps", T["success"]),
            ("eth2 (DC Core)", "880 Mbps", "180k pps", T["success"]),
            ("eth3 (Sandbox)", "0 pps", "VLAN 999", T["sandbox"])
        ]
        for ifi, (iname, irate, ipps, icol) in enumerate(tifaces):
            ify = iy + 178 + ifi * 48
            out.append(f'<rect x="{ix+14}" y="{ify}" width="{iw-28}" height="40" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
            out.append(f'<text x="{ix+24}" y="{ify+18}" font-family="{FONT_MONO}" font-size="9.5" font-weight="700" fill="{T["text_primary"]}">{iname}</text>')
            out.append(f'<text x="{ix+24}" y="{ify+31}" font-family="{FONT_MONO}" font-size="8.5" fill="{T["text_muted"]}">{irate} • {ipps}</text>')
            out.append(f'<circle cx="{ix+iw-28}" cy="{ify+20}" r="3.5" fill="{icol}"/>')

        # Acciones
        out.append(f'<text x="{ix+16}" y="{iy+396}" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="{T["text_primary"]}">Acciones Operativas:</text>')
        out.append(f'<rect x="{ix+14}" y="{iy+410}" width="{iw-28}" height="38" rx="4" fill="{T["accent"]}"/>')
        out.append(f'<text x="{ix+iw/2}" y="{iy+434}" font-family="{FONT_SANS}" font-size="10.5" font-weight="700" fill="#FFFFFF" text-anchor="middle">Inspeccionar eBPF Maps</text>')

        out.append(f'<rect x="{ix+14}" y="{iy+456}" width="{iw-28}" height="38" rx="4" fill="{T["warning_bg"]}" stroke="{T["warning_border"]}" stroke-width="1"/>')
        out.append(f'<text x="{ix+iw/2}" y="{iy+480}" font-family="{FONT_SANS}" font-size="10.5" font-weight="700" fill="{T["warning_text"]}" text-anchor="middle">Aislar Nodo de Red</text>')

        out.append(f'<rect x="{ix+14}" y="{iy+502}" width="{iw-28}" height="38" rx="4" fill="{T["sandbox_bg"]}" stroke="{T["sandbox_border"]}" stroke-width="1"/>')
        out.append(f'<text x="{ix+iw/2}" y="{iy+526}" font-family="{FONT_SANS}" font-size="10" font-weight="700" fill="{T["sandbox_text"]}" text-anchor="middle">Clonar a Sandbox ACL</text>')

        out.append('</svg>')
        return "".join(out)

    else: # mobile (390x844)
        w, h = 390, 844
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        out.append(render_header_mobile(T, "Topología de Red"))

        # Banner de Estado del Core
        out.append(f'<rect x="16" y="96" width="358" height="52" rx="5" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(icon_network(26, 110, 18, T["accent"]))
        out.append(f'<text x="52" y="117" font-family="{FONT_SANS}" font-size="11" font-weight="800" fill="{T["text_primary"]}">Infraestructura Carrier AS27839</text>')
        out.append(f'<text x="52" y="133" font-family="{FONT_MONO}" font-size="9" fill="{T["text_muted"]}">Gateway: Core-Router-01 • eBPF: OK</text>')

        # Tarjetas de Nodos
        out.append(f'<rect x="16" y="156" width="358" height="520" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="28" y="178" font-family="{FONT_SANS}" font-size="11.5" font-weight="700" fill="{T["text_primary"]}">Nodos de Transporte &amp; Borde:</text>')
        
        mnodes = [
            ("Core-Router-01 (VyOS / BGP)", "172.16.10.1", "1.84 Gbps • 680k pps", "ONLINE", T["success"]),
            ("Carrier-Peering-01 (Upstream)", "198.51.100.1", "1.42 Gbps • 420k pps", "BGP OK", T["success"]),
            ("DC-Backend-Tigo (Data Center)", "10.10.0.1", "880 Mbps • 180k pps", "ACTIVO", T["success"]),
            ("Honeypot-Sensor-01 (Decoy)", "172.28.0.10", "350k pps • Ataque L4", "ALERTA", T["critical"]),
            ("Sandbox-Router-SB0 (Aislado)", "10.200.0.1", "VLAN 999 • 0 pps fuga", "SANDBOX", T["sandbox"])
        ]
        for mi, (mname, mip, mload, mst, mcol) in enumerate(mnodes):
            my = 192 + mi * 66
            out.append(f'<rect x="26" y="{my}" width="338" height="56" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
            out.append(f'<circle cx="40" cy="{my+28}" r="4" fill="{mcol}"/>')
            out.append(f'<text x="52" y="{my+22}" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="{T["text_primary"]}">{mname}</text>')
            out.append(f'<text x="52" y="{my+37}" font-family="{FONT_MONO}" font-size="9" fill="{T["text_muted"]}">{mip} • {mload}</text>')
            out.append(f'<rect x="290" y="{my+16}" width="64" height="18" rx="2" fill="{T["bg_panel"]}" stroke="{mcol}" stroke-width="0.8"/>')
            out.append(f'<text x="322" y="{my+29}" font-family="{FONT_MONO}" font-size="8" font-weight="700" fill="{mcol}" text-anchor="middle">{mst}</text>')

        # Telemetría de interfaces del nodo seleccionado
        out.append(f'<text x="28" y="542" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="{T["text_primary"]}">Telemetría eth0-eth3 (Core-01):</text>')
        out.append(f'<rect x="26" y="552" width="338" height="42" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
        out.append(f'<text x="38" y="570" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_secondary"]}">eth0: 1.84G (680k pps) • eth1: 1.42G</text>')
        out.append(f'<text x="38" y="584" font-family="{FONT_MONO}" font-size="9" fill="{T["text_muted"]}">Filtro XDP: Activo • Driver nativo 10GbE</text>')

        # Botones de Acción Móvil
        out.append(f'<rect x="26" y="604" width="338" height="34" rx="4" fill="{T["accent"]}"/>')
        out.append(f'<text x="195" y="626" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="#FFFFFF" text-anchor="middle">Inspeccionar eBPF Maps</text>')

        out.append(f'<rect x="26" y="646" width="338" height="34" rx="4" fill="{T["sandbox_bg"]}" stroke="{T["sandbox_border"]}" stroke-width="1"/>')
        out.append(f'<text x="195" y="668" font-family="{FONT_SANS}" font-size="10.5" font-weight="700" fill="{T["sandbox_text"]}" text-anchor="middle">Clonar Configuración a Sandbox</text>')

        out.append('</svg>')
        return "".join(out)

def gen_screen_04_monitoring_charts(T, device="pc"):
    """PANTALLA 4: Métricas ML (LightGBM) & Análisis Estadístico"""
    matrix_rows = [
        ("Benigno", "99.82%", "0.12%", "0.04%", "0.02%"),
        ("SYN Flood", "0.21%", "99.45%", "0.22%", "0.12%"),
        ("UDP Amplif", "0.08%", "0.15%", "99.62%", "0.15%"),
        ("Exfiltración", "0.11%", "0.32%", "0.25%", "99.32%")
    ]
    if device == "pc":
        w, h = 1440, 900
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        out.append(render_header_pc(T, "monitoring"))

        # Top Bar: Filtros y Parámetros del Modelo
        fx, fy, fw, fh = 24, 76, 1392, 54
        out.append(f'<rect x="{fx}" y="{fy}" width="{fw}" height="{fh}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(icon_chart(fx+16, fy+18, 18, T["accent"]))
        out.append(f'<text x="{fx+42}" y="{fy+32}" font-family="{FONT_SANS}" font-size="13" font-weight="800" fill="{T["text_primary"]}">Métricas de Inferencia LightGBM v4.2 • CIC-DDoS2019 + ISP NetFlow</text>')
        out.append(f'<text x="{fx+fw-280}" y="{fy+32}" font-family="{FONT_MONO}" font-size="11" font-weight="600" fill="{T["text_secondary"]}">Ventana Temporal: Últimas 6 Horas</text>')

        # 4 KPIs de Rendimiento ML
        ml_kpis = [
            ("Precisión Global (Accuracy)", "99.42%", "Evaluado en 2.4M flujos", T["success"]),
            ("F1-Score Ponderado", "0.9906", "Balance Óptimo Precision/Recall", T["success"]),
            ("Tiempo de Inferencia", "0.42 ms", "Inferencia en Tiempo Real", T["accent"]),
            ("Tasa de Falsos Positivos", "0.08%", "Bajo umbral crítico (0.1%)", T["success"])
        ]
        card_w = (1392 - 60) / 4
        for ki, (kt, kv, ks, kc) in enumerate(ml_kpis):
            kx = 24 + ki * (card_w + 20)
            out.append(f'<rect x="{kx}" y="146" width="{card_w}" height="90" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
            out.append(f'<text x="{kx+16}" y="172" font-family="{FONT_SANS}" font-size="11" font-weight="600" fill="{T["text_muted"]}">{kt}</text>')
            out.append(f'<text x="{kx+16}" y="206" font-family="{FONT_SANS}" font-size="22" font-weight="800" fill="{kc}">{kv}</text>')
            out.append(f'<text x="{kx+16}" y="224" font-family="{FONT_MONO}" font-size="9" fill="{T["text_secondary"]}">{ks}</text>')

        # Gráfico 1: Feature Importance Ranking (Izquierda)
        g1_x, g1_y, g1_w, g1_h = 24, 252, 684, 340
        out.append(f'<rect x="{g1_x}" y="{g1_y}" width="{g1_w}" height="{g1_h}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{g1_x+20}" y="{g1_y+28}" font-family="{FONT_SANS}" font-size="13.5" font-weight="800" fill="{T["text_primary"]}">Ranking de Importancia de Características (Tree Splits)</text>')
        
        feats = [
            ("flow_duration (Duración de Flujo)", 0.88, "2,410 splits"),
            ("fwd_pkt_len_mean (Longitud Media Pkt)", 0.82, "2,150 splits"),
            ("bwd_pkt_len_std (Desv Estándar Pkt)", 0.74, "1,890 splits"),
            ("flow_iat_min (Mínimo Inter-Arrival)", 0.65, "1,620 splits"),
            ("syn_flag_count (Conteo Flags SYN)", 0.58, "1,410 splits"),
            ("bwd_packets_s (Tasa Paquetes Bwd)", 0.49, "1,180 splits")
        ]
        for fi, (fname, fval, fsplits) in enumerate(feats):
            fy_bar = g1_y + 60 + fi * 42
            out.append(f'<text x="{g1_x+20}" y="{fy_bar+12}" font-family="{FONT_MONO}" font-size="10.5" fill="{T["text_primary"]}">{fname}</text>')
            out.append(f'<text x="{g1_x+g1_w-20}" y="{fy_bar+12}" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_muted"]}" text-anchor="end">{fsplits}</text>')
            out.append(f'<rect x="{g1_x+20}" y="{fy_bar+18}" width="{g1_w-40}" height="10" rx="3" fill="{T["bg_subtle"]}"/>')
            out.append(f'<rect x="{g1_x+20}" y="{fy_bar+18}" width="{(g1_w-40)*fval}" height="10" rx="3" fill="{T["accent"]}"/>')

        # Gráfico 2: Matriz de Confusión Multiclase (Derecha)
        g2_x, g2_y, g2_w, g2_h = 732, 252, 684, 340
        out.append(f'<rect x="{g2_x}" y="{g2_y}" width="{g2_w}" height="{g2_h}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{g2_x+20}" y="{g2_y+28}" font-family="{FONT_SANS}" font-size="13.5" font-weight="800" fill="{T["text_primary"]}">Matriz de Confusión de Clasificación de Tráfico</text>')

        matrix_rows = [
            ("Benigno", "99.82%", "0.12%", "0.04%", "0.02%"),
            ("SYN Flood", "0.21%", "99.45%", "0.22%", "0.12%"),
            ("UDP Amplif", "0.08%", "0.15%", "99.62%", "0.15%"),
            ("Exfiltración", "0.11%", "0.32%", "0.25%", "99.32%")
        ]
        mcols = ["Clase Real", "Pred: Benigno", "Pred: SYN", "Pred: UDP", "Pred: Exfil"]
        col_offsets = [36, 175, 300, 420, 540]
        for ci, ctitle in enumerate(mcols):
            out.append(f'<text x="{g2_x+col_offsets[ci]}" y="{g2_y+64}" font-family="{FONT_SANS}" font-size="10" font-weight="700" fill="{T["text_muted"]}">{ctitle}</text>')

        for ri, (rname, p1, p2, p3, p4) in enumerate(matrix_rows):
            my_r = g2_y + 82 + ri * 52
            out.append(f'<rect x="{g2_x+20}" y="{my_r}" width="{g2_w-40}" height="42" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
            out.append(f'<text x="{g2_x+36}" y="{my_r+25}" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="{T["text_primary"]}">{rname}</text>')
            vals = [p1, p2, p3, p4]
            for vi, val in enumerate(vals):
                is_diag = (vi == ri)
                col_txt = T["success"] if is_diag else T["text_muted"]
                out.append(f'<text x="{g2_x+col_offsets[vi+1]}" y="{my_r+25}" font-family="{FONT_MONO}" font-size="11" font-weight="{"700" if is_diag else "400"}" fill="{col_txt}">{val}</text>')

        # Gráfico Inferior: Distribución de Scores de Anomalía
        g3_x, g3_y, g3_w, g3_h = 24, 608, 1392, 268
        out.append(f'<rect x="{g3_x}" y="{g3_y}" width="{g3_w}" height="{g3_h}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{g3_x+20}" y="{g3_y+28}" font-family="{FONT_SANS}" font-size="13.5" font-weight="800" fill="{T["text_primary"]}">Distribución de Probabilidad de Anomalía (Histograma)</text>')
        out.append(f'<text x="{g3_x+20}" y="{g3_y+44}" font-family="{FONT_SANS}" font-size="10.5" fill="{T["text_muted"]}">Umbral de corte de decisión: 0.85 (Zona de Mitigación Inmediata eBPF)</text>')
        
        # Histograma barras
        hist_y = g3_y + 60
        bar_count = 24
        bar_w = (g3_w - 60) / bar_count
        for bi in range(bar_count):
            bx = g3_x + 30 + bi * bar_w
            # Forma bimodal
            if bi < 8:
                h_val = 140 - bi * 12
                col_bar = T["accent"]
            elif bi < 18:
                h_val = 15 + (bi % 3) * 6
                col_bar = T["text_muted"]
            else:
                h_val = 20 + (bi - 17) * 22
                col_bar = T["critical"]
            out.append(f'<rect x="{bx}" y="{hist_y + 160 - h_val}" width="{bar_w - 4}" height="{h_val}" rx="2" fill="{col_bar}" opacity="0.8"/>')

        out.append('</svg>')
        return "".join(out)

    elif device == "tablet":
        w, h = 1024, 768
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        out.append(render_header_tablet(T, "monitoring"))
        
        # 4 KPIs Tablet
        card_w = (1024 - 40 - 36) / 4
        tkpis = [
            ("Precisión Global", "99.42%", T["success"]),
            ("F1-Score", "0.9906", T["success"]),
            ("Inferencia ML", "0.42 ms", T["accent"]),
            ("Falsos Positivos", "0.08%", T["success"])
        ]
        for ki, (kt, kv, kc) in enumerate(tkpis):
            kx = 20 + ki * (card_w + 12)
            out.append(f'<rect x="{kx}" y="70" width="{card_w}" height="76" rx="5" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
            out.append(f'<text x="{kx+14}" y="94" font-family="{FONT_SANS}" font-size="10.5" fill="{T["text_muted"]}">{kt}</text>')
            out.append(f'<text x="{kx+14}" y="126" font-family="{FONT_SANS}" font-size="19" font-weight="800" fill="{kc}">{kv}</text>')

        # Panel Izquierdo: Feature Importance (470x340)
        p1_x, p1_y, p1_w, p1_h = 20, 160, 474, 340
        out.append(f'<rect x="{p1_x}" y="{p1_y}" width="{p1_w}" height="{p1_h}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{p1_x+16}" y="{p1_y+26}" font-family="{FONT_SANS}" font-size="12.5" font-weight="800" fill="{T["text_primary"]}">Ranking de Características (Tree Splits)</text>')
        
        tfeats = [
            ("flow_duration (Duración Flujo)", 0.88, "2,410"),
            ("fwd_pkt_len_mean (Media Pkt)", 0.82, "2,150"),
            ("bwd_pkt_len_std (Desv Pkt)", 0.74, "1,890"),
            ("flow_iat_min (Inter-Arrival)", 0.65, "1,620"),
            ("syn_flag_count (Flags SYN)", 0.58, "1,410")
        ]
        for fi, (fname, fval, fsplits) in enumerate(tfeats):
            fy = p1_y + 46 + fi * 54
            out.append(f'<text x="{p1_x+16}" y="{fy+14}" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_primary"]}">{fname}</text>')
            out.append(f'<text x="{p1_x+p1_w-16}" y="{fy+14}" font-family="{FONT_MONO}" font-size="9" fill="{T["text_muted"]}" text-anchor="end">{fsplits} splits</text>')
            out.append(f'<rect x="{p1_x+16}" y="{fy+20}" width="{p1_w-32}" height="10" rx="3" fill="{T["bg_subtle"]}"/>')
            out.append(f'<rect x="{p1_x+16}" y="{fy+20}" width="{(p1_w-32)*fval}" height="10" rx="3" fill="{T["accent"]}"/>')

        # Panel Derecho: Matriz de Confusión (490x340)
        p2_x, p2_y, p2_w, p2_h = 510, 160, 494, 340
        out.append(f'<rect x="{p2_x}" y="{p2_y}" width="{p2_w}" height="{p2_h}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{p2_x+16}" y="{p2_y+26}" font-family="{FONT_SANS}" font-size="12.5" font-weight="800" fill="{T["text_primary"]}">Matriz de Confusión Multiclase</text>')
        
        tm_cols = ["Clase Real", "Benigno", "SYN Flood", "UDP Ampl", "Exfil"]
        tm_offsets = [20, 130, 220, 310, 400]
        for ci, ctitle in enumerate(tm_cols):
            out.append(f'<text x="{p2_x+tm_offsets[ci]}" y="{p2_y+58}" font-family="{FONT_SANS}" font-size="9" font-weight="700" fill="{T["text_muted"]}">{ctitle}</text>')

        for ri, (rname, p1, p2, p3, p4) in enumerate(matrix_rows):
            ry = p2_y + 74 + ri * 60
            out.append(f'<rect x="{p2_x+14}" y="{ry}" width="{p2_w-28}" height="48" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
            out.append(f'<text x="{p2_x+20}" y="{ry+28}" font-family="{FONT_SANS}" font-size="10.5" font-weight="700" fill="{T["text_primary"]}">{rname}</text>')
            vals = [p1, p2, p3, p4]
            for vi, val in enumerate(vals):
                is_diag = (vi == ri)
                col_txt = T["success"] if is_diag else T["text_muted"]
                out.append(f'<text x="{p2_x+tm_offsets[vi+1]}" y="{ry+28}" font-family="{FONT_MONO}" font-size="10.5" font-weight="{"700" if is_diag else "400"}" fill="{col_txt}">{val}</text>')

        # Panel Inferior: Histograma de Anomalía (984x230)
        p3_x, p3_y, p3_w, p3_h = 20, 514, 984, 230
        out.append(f'<rect x="{p3_x}" y="{p3_y}" width="{p3_w}" height="{p3_h}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{p3_x+16}" y="{p3_y+26}" font-family="{FONT_SANS}" font-size="12.5" font-weight="800" fill="{T["text_primary"]}">Distribución de Score de Anomalía (Umbral de Mitigación: 0.85)</text>')
        
        hist_y = p3_y + 50
        t_bar_count = 20
        t_bar_w = (p3_w - 40) / t_bar_count
        for bi in range(t_bar_count):
            bx = p3_x + 20 + bi * t_bar_w
            if bi < 6:
                h_val = 120 - bi * 14
                col_b = T["accent"]
            elif bi < 14:
                h_val = 14 + (bi % 3) * 6
                col_b = T["text_muted"]
            else:
                h_val = 20 + (bi - 13) * 22
                col_b = T["critical"]
            out.append(f'<rect x="{bx}" y="{hist_y + 140 - h_val}" width="{t_bar_w - 4}" height="{h_val}" rx="2" fill="{col_b}" opacity="0.8"/>')

        out.append('</svg>')
        return "".join(out)

    else: # mobile (390x844)
        w, h = 390, 844
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        out.append(render_header_mobile(T, "Métricas ML"))

        # 4 KPIs en Grid 2x2
        mkpis = [
            ("Precisión Global", "99.42%", T["success"]),
            ("F1-Score", "0.9906", T["success"]),
            ("Inferencia ML", "0.42 ms", T["accent"]),
            ("Falsos Positivos", "0.08%", T["success"])
        ]
        for mi, (mt, mv, mc) in enumerate(mkpis):
            col_idx = mi % 2
            row_idx = mi // 2
            kx = 16 + col_idx * 184
            ky = 96 + row_idx * 68
            out.append(f'<rect x="{kx}" y="{ky}" width="174" height="60" rx="5" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
            out.append(f'<text x="{kx+12}" y="{ky+18}" font-family="{FONT_SANS}" font-size="9.5" fill="{T["text_muted"]}">{mt}</text>')
            out.append(f'<text x="{kx+12}" y="{ky+44}" font-family="{FONT_SANS}" font-size="16" font-weight="800" fill="{mc}">{mv}</text>')

        # Ranking de Características (Top 4)
        out.append(f'<rect x="16" y="240" width="358" height="230" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="28" y="262" font-family="{FONT_SANS}" font-size="11.5" font-weight="800" fill="{T["text_primary"]}">Top Características Explicables (Shapley):</text>')
        
        mfeats = [
            ("flow_duration (Duración)", 0.88, "88%"),
            ("fwd_pkt_len_mean (Media Pkt)", 0.82, "82%"),
            ("syn_flag_count (SYN Flags)", 0.58, "58%"),
            ("flow_iat_min (Inter-Arrival)", 0.49, "49%")
        ]
        for fi, (fname, fval, fpct) in enumerate(mfeats):
            fy = 274 + fi * 46
            out.append(f'<text x="28" y="{fy+12}" font-family="{FONT_MONO}" font-size="9" fill="{T["text_primary"]}">{fname}</text>')
            out.append(f'<text x="360" y="{fy+12}" font-family="{FONT_MONO}" font-size="8.5" font-weight="700" fill="{T["accent"]}" text-anchor="end">{fpct}</text>')
            out.append(f'<rect x="28" y="{fy+18}" width="332" height="8" rx="2" fill="{T["bg_subtle"]}"/>')
            out.append(f'<rect x="28" y="{fy+18}" width="{332*fval}" height="8" rx="2" fill="{T["accent"]}"/>')

        # Matriz de Confusión Resumida
        out.append(f'<rect x="16" y="480" width="358" height="240" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="28" y="502" font-family="{FONT_SANS}" font-size="11.5" font-weight="800" fill="{T["text_primary"]}">Matriz de Confusión (Tasa Acierto):</text>')
        
        mm_rows = [
            ("Tráfico Benigno", "99.82%", "Pasa intacto", T["success"]),
            ("DDoS SYN Flood", "99.45%", "Descarte XDP", T["success"]),
            ("UDP Amplification", "99.62%", "Rate-Limit L4", T["success"]),
            ("Exfiltración Datos", "99.32%", "Bloqueo BGP", T["success"])
        ]
        for ri, (rname, raci, rdesc, rcol) in enumerate(mm_rows):
            ry = 514 + ri * 48
            out.append(f'<rect x="26" y="{ry}" width="338" height="40" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
            out.append(f'<text x="38" y="{ry+18}" font-family="{FONT_SANS}" font-size="10.5" font-weight="700" fill="{T["text_primary"]}">{rname}</text>')
            out.append(f'<text x="38" y="{ry+31}" font-family="{FONT_MONO}" font-size="8.5" fill="{T["text_muted"]}">{rdesc}</text>')
            out.append(f'<text x="350" y="{ry+24}" font-family="{FONT_MONO}" font-size="12" font-weight="800" fill="{rcol}" text-anchor="end">{raci}</text>')

        # Umbral eBPF
        out.append(f'<rect x="16" y="730" width="358" height="42" rx="5" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="1"/>')
        out.append(f'<text x="28" y="756" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_secondary"]}">Umbral Decisión: <tspan font-weight="700" fill="{T["critical"]}">0.85</tspan> • Latencia: <tspan font-weight="700" fill="{T["accent"]}">0.42 ms</tspan></text>')

        out.append('</svg>')
        return "".join(out)

def gen_screen_05_realtime_stream(T, device="pc"):
    """PANTALLA 5: Inspección y Telemetría de Paquetes en Vivo (eBPF Stream)"""
    if device == "pc":
        w, h = 1440, 900
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        out.append(render_header_pc(T, "realtime"))

        # Toolbar de Control del Stream
        tx, ty, tw, th = 24, 76, 1392, 54
        out.append(f'<rect x="{tx}" y="{ty}" width="{tw}" height="{th}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<circle cx="{tx+24}" cy="{ty+27}" r="4" fill="{T["success"]}"/>')
        out.append(f'<text x="{tx+36}" y="{ty+32}" font-family="{FONT_MONO}" font-size="11" font-weight="700" fill="{T["text_primary"]}">EN VIVO: 14,280 PKTS/S • BUFFER: 500 FLUIDO</text>')
        
        # Filtro BPF
        out.append(f'<rect x="{tx+380}" y="{ty+12}" width="420" height="30" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{tx+394}" y="{ty+31}" font-family="{FONT_MONO}" font-size="10" fill="{T["text_secondary"]}">Filtro BPF: ip.proto == TCP and tcp.flags.syn == 1</text>')

        # Botones
        out.append(f'<rect x="{tx+tw-240}" y="{ty+12}" width="110" height="30" rx="4" fill="{T["accent"]}"/>')
        out.append(f'<text x="{tx+tw-185}" y="{ty+31}" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="#FFFFFF" text-anchor="middle">Pausar Flujo</text>')
        out.append(f'<rect x="{tx+tw-120}" y="{ty+12}" width="105" height="30" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{tx+tw-68}" y="{ty+31}" font-family="{FONT_SANS}" font-size="11" font-weight="600" fill="{T["text_secondary"]}" text-anchor="middle">Limpiar</text>')

        # Tabla de Paquetes en Vivo
        tbl_x, tbl_y, tbl_w, tbl_h = 24, 142, 1392, 730
        out.append(f'<rect x="{tbl_x}" y="{tbl_y}" width="{tbl_w}" height="{tbl_h}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        
        # Header de Columnas
        th_y = tbl_y + 16
        out.append(f'<rect x="{tbl_x+16}" y="{th_y}" width="{tbl_w-32}" height="32" rx="3" fill="{T["bg_subtle"]}"/>')
        pcols = [("TIMESTAMP", 30), ("IFACE", 140), ("IP ORIGEN", 220), ("PUERTO", 380), ("IP DESTINO", 460), ("PROTO", 620), ("LEN", 700), ("SCORE ML", 780), ("VEREDICTO", 920), ("PAYLOAD HEX DUMP", 1080)]
        for col_name, cpos in pcols:
            out.append(f'<text x="{tbl_x+cpos}" y="{th_y+20}" font-family="{FONT_SANS}" font-size="10" font-weight="700" fill="{T["text_muted"]}">{col_name}</text>')

        # 12 Filas de Telemetría Real
        pkts = [
            ("14:45:12.802", "eth0", "198.51.100.44", "44120", "10.10.20.5", "TCP [SYN]", "64 B", "0.998", "DROP XDP", "45 00 00 3c 7a ...", T["critical"]),
            ("14:45:12.802", "eth0", "198.51.100.44", "44121", "10.10.20.5", "TCP [SYN]", "64 B", "0.998", "DROP XDP", "45 00 00 3c 8b ...", T["critical"]),
            ("14:45:12.801", "eth1", "181.188.45.12", "52341", "10.10.0.1", "TCP [ACK]", "1460 B", "0.012", "PASS", "45 00 05 dc 12 ...", T["success"]),
            ("14:45:12.800", "eth1", "181.188.45.12", "52341", "10.10.0.1", "TCP [PSH]", "890 B", "0.015", "PASS", "45 00 03 7a 4f ...", T["success"]),
            ("14:45:12.798", "eth0", "203.0.113.89", "53", "10.10.20.12", "UDP [DNS]", "512 B", "0.985", "RATE LIMIT", "45 00 02 00 e1 ...", T["warning"]),
            ("14:45:12.795", "eth0", "198.51.100.44", "44122", "10.10.20.5", "TCP [SYN]", "64 B", "0.998", "DROP XDP", "45 00 00 3c 9c ...", T["critical"]),
            ("14:45:12.792", "eth1", "190.129.88.5", "443", "10.10.0.1", "TLS 1.3", "1420 B", "0.008", "PASS", "45 00 05 8c 33 ...", T["success"]),
            ("14:45:12.790", "eth2", "10.200.0.128", "55102", "10.200.0.50", "TCP [SYN]", "60 B", "0.994", "SANDBOX DROP", "45 00 00 3c ff ...", T["sandbox"]),
            ("14:45:12.788", "eth1", "181.115.12.9", "61240", "10.10.0.1", "HTTPS", "1280 B", "0.011", "PASS", "45 00 05 00 71 ...", T["success"]),
            ("14:45:12.785", "eth0", "198.51.100.44", "44123", "10.10.20.5", "TCP [SYN]", "64 B", "0.998", "DROP XDP", "45 00 00 3c a1 ...", T["critical"]),
            ("14:45:12.782", "eth1", "181.188.90.3", "58412", "10.10.0.1", "TCP [ACK]", "1460 B", "0.014", "PASS", "45 00 05 dc 82 ...", T["success"]),
            ("14:45:12.780", "eth0", "198.51.100.44", "44124", "10.10.20.5", "TCP [SYN]", "64 B", "0.998", "DROP XDP", "45 00 00 3c b3 ...", T["critical"]),
        ]
        for pi, (ptime, pif, psrc, psport, pdst, pproto, plen, pml, pact, phex, pcol) in enumerate(pkts):
            py_row = th_y + 40 + pi * 44
            out.append(f'<line x1="{tbl_x+16}" y1="{py_row+36}" x2="{tbl_x+tbl_w-16}" y2="{py_row+36}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
            out.append(f'<text x="{tbl_x+30}" y="{py_row+22}" font-family="{FONT_MONO}" font-size="10.5" fill="{T["text_primary"]}">{ptime}</text>')
            out.append(f'<text x="{tbl_x+140}" y="{py_row+22}" font-family="{FONT_MONO}" font-size="10.5" fill="{T["text_secondary"]}">{pif}</text>')
            out.append(f'<text x="{tbl_x+220}" y="{py_row+22}" font-family="{FONT_MONO}" font-size="10.5" fill="{T["text_primary"]}">{psrc}</text>')
            out.append(f'<text x="{tbl_x+380}" y="{py_row+22}" font-family="{FONT_MONO}" font-size="10.5" fill="{T["text_muted"]}">{psport}</text>')
            out.append(f'<text x="{tbl_x+460}" y="{py_row+22}" font-family="{FONT_MONO}" font-size="10.5" fill="{T["text_secondary"]}">{pdst}</text>')
            out.append(f'<text x="{tbl_x+620}" y="{py_row+22}" font-family="{FONT_MONO}" font-size="10.5" fill="{T["text_primary"]}">{pproto}</text>')
            out.append(f'<text x="{tbl_x+700}" y="{py_row+22}" font-family="{FONT_MONO}" font-size="10.5" fill="{T["text_muted"]}">{plen}</text>')
            out.append(f'<text x="{tbl_x+780}" y="{py_row+22}" font-family="{FONT_MONO}" font-size="10.5" font-weight="700" fill="{pcol}">{pml}</text>')
            out.append(f'<rect x="{tbl_x+916}" y="{py_row+6}" width="120" height="24" rx="3" fill="{T["bg_subtle"]}" stroke="{pcol}" stroke-width="0.8"/>')
            out.append(f'<text x="{tbl_x+976}" y="{py_row+22}" font-family="{FONT_MONO}" font-size="9" font-weight="700" fill="{pcol}" text-anchor="middle">{pact}</text>')
            out.append(f'<text x="{tbl_x+1080}" y="{py_row+22}" font-family="{FONT_MONO}" font-size="10" fill="{T["code_text"]}">{phex}</text>')

        out.append('</svg>')
        return "".join(out)

    elif device == "tablet":
        w, h = 1024, 768
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        out.append(render_header_tablet(T, "realtime"))

        # Toolbar Tablet
        tx, ty, tw, th = 20, 70, 984, 52
        out.append(f'<rect x="{tx}" y="{ty}" width="{tw}" height="{th}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<circle cx="{tx+20}" cy="{ty+26}" r="4" fill="{T["success"]}"/>')
        out.append(f'<text x="{tx+32}" y="{ty+30}" font-family="{FONT_MONO}" font-size="10.5" font-weight="700" fill="{T["text_primary"]}">EN VIVO: 14,280 PKTS/S • BUFFER: 500 FLUIDO</text>')
        
        # Filtro BPF
        out.append(f'<rect x="{tx+330}" y="{ty+11}" width="340" height="30" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border"]}" stroke-width="0.8"/>')
        out.append(f'<text x="{tx+342}" y="{ty+29}" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_secondary"]}">Filtro BPF: ip.proto == TCP and syn == 1</text>')

        # Botones
        out.append(f'<rect x="{tx+tw-210}" y="{ty+11}" width="95" height="30" rx="4" fill="{T["accent"]}"/>')
        out.append(f'<text x="{tx+tw-162}" y="{ty+29}" font-family="{FONT_SANS}" font-size="10" font-weight="700" fill="#FFFFFF" text-anchor="middle">Pausar Flujo</text>')
        out.append(f'<rect x="{tx+tw-105}" y="{ty+11}" width="90" height="30" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{tx+tw-60}" y="{ty+29}" font-family="{FONT_SANS}" font-size="10" font-weight="600" fill="{T["text_secondary"]}" text-anchor="middle">Limpiar</text>')

        # Tabla de Paquetes Tablet
        tbl_x, tbl_y, tbl_w, tbl_h = 20, 134, 984, 430
        out.append(f'<rect x="{tbl_x}" y="{tbl_y}" width="{tbl_w}" height="{tbl_h}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        
        th_y = tbl_y + 12
        out.append(f'<rect x="{tbl_x+12}" y="{th_y}" width="{tbl_w-24}" height="28" rx="3" fill="{T["bg_subtle"]}"/>')
        tpcols = [("TIMESTAMP", 20), ("IFACE", 120), ("ORIGEN", 185), ("PUERTO", 310), ("DESTINO", 375), ("PROTO", 495), ("SCORE ML", 590), ("VEREDICTO eBPF", 700), ("PAYLOAD HEX", 830)]
        for col_name, cpos in tpcols:
            out.append(f'<text x="{tbl_x+cpos}" y="{th_y+18}" font-family="{FONT_SANS}" font-size="9" font-weight="700" fill="{T["text_muted"]}">{col_name}</text>')

        tpkts = [
            ("14:45:12.802", "eth0", "198.51.100.44", "44120", "10.10.20.5", "TCP [SYN]", "0.998", "DROP XDP", "45 00 00 3c 7a ...", T["critical"]),
            ("14:45:12.801", "eth1", "181.188.45.12", "52341", "10.10.0.1", "TCP [ACK]", "0.012", "PASS", "45 00 05 dc 12 ...", T["success"]),
            ("14:45:12.800", "eth1", "181.188.45.12", "52341", "10.10.0.1", "TCP [PSH]", "0.015", "PASS", "45 00 03 7a 4f ...", T["success"]),
            ("14:45:12.798", "eth0", "203.0.113.89", "53", "10.10.20.12", "UDP [DNS]", "0.985", "RATE LIMIT", "45 00 02 00 e1 ...", T["warning"]),
            ("14:45:12.795", "eth0", "198.51.100.44", "44122", "10.10.20.5", "TCP [SYN]", "0.998", "DROP XDP", "45 00 00 3c 9c ...", T["critical"]),
            ("14:45:12.792", "eth1", "190.129.88.5", "443", "10.10.0.1", "TLS 1.3", "0.008", "PASS", "45 00 05 8c 33 ...", T["success"]),
            ("14:45:12.790", "eth2", "10.200.0.128", "55102", "10.200.0.50", "TCP [SYN]", "0.994", "SANDBOX DROP", "45 00 00 3c ff ...", T["sandbox"]),
            ("14:45:12.785", "eth0", "198.51.100.44", "44123", "10.10.20.5", "TCP [SYN]", "0.998", "DROP XDP", "45 00 00 3c a1 ...", T["critical"])
        ]
        for pi, (ptime, pif, psrc, psport, pdst, pproto, pml, pact, phex, pcol) in enumerate(tpkts):
            py_row = th_y + 36 + pi * 44
            out.append(f'<line x1="{tbl_x+12}" y1="{py_row+36}" x2="{tbl_x+tbl_w-12}" y2="{py_row+36}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
            out.append(f'<text x="{tbl_x+20}" y="{py_row+22}" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_primary"]}">{ptime}</text>')
            out.append(f'<text x="{tbl_x+120}" y="{py_row+22}" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_secondary"]}">{pif}</text>')
            out.append(f'<text x="{tbl_x+185}" y="{py_row+22}" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_primary"]}">{psrc}</text>')
            out.append(f'<text x="{tbl_x+310}" y="{py_row+22}" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_muted"]}">{psport}</text>')
            out.append(f'<text x="{tbl_x+375}" y="{py_row+22}" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_secondary"]}">{pdst}</text>')
            out.append(f'<text x="{tbl_x+495}" y="{py_row+22}" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_primary"]}">{pproto}</text>')
            out.append(f'<text x="{tbl_x+590}" y="{py_row+22}" font-family="{FONT_MONO}" font-size="9.5" font-weight="700" fill="{pcol}">{pml}</text>')
            out.append(f'<rect x="{tbl_x+696}" y="{py_row+6}" width="100" height="22" rx="3" fill="{T["bg_subtle"]}" stroke="{pcol}" stroke-width="0.8"/>')
            out.append(f'<text x="{tbl_x+746}" y="{py_row+21}" font-family="{FONT_MONO}" font-size="8" font-weight="700" fill="{pcol}" text-anchor="middle">{pact}</text>')
            out.append(f'<text x="{tbl_x+830}" y="{py_row+22}" font-family="{FONT_MONO}" font-size="8.5" fill="{T["code_text"]}">{phex}</text>')

        # Inspector Hex Dump Inferior
        hx, hy, hw, hh = 20, 576, 984, 168
        out.append(f'<rect x="{hx}" y="{hy}" width="{hw}" height="{hh}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{hx+16}" y="{hy+24}" font-family="{FONT_SANS}" font-size="12" font-weight="800" fill="{T["text_primary"]}">Inspector de Payload (Hex Dump &amp; Deserialización TCP/IP):</text>')
        out.append(f'<rect x="{hx+14}" y="{hy+36}" width="{hw-28}" height="116" rx="4" fill="{T["code_bg"]}"/>')
        out.append(f'<text x="{hx+26}" y="{hy+58}" font-family="{FONT_MONO}" font-size="10" fill="{T["code_text"]}">0000   45 00 00 3c 7a 1b 40 00 40 06 c5 32 c6 33 64 2c   E..&lt;z.@.@..2.3d,</text>')
        out.append(f'<text x="{hx+26}" y="{hy+76}" font-family="{FONT_MONO}" font-size="10" fill="{T["code_text"]}">0010   0a 0a 14 05 ac 58 00 50 1a 82 9f 30 00 00 00 00   .....X.P...0....</text>')
        out.append(f'<text x="{hx+26}" y="{hy+94}" font-family="{FONT_MONO}" font-size="10" fill="{T["code_text"]}">0020   a0 02 fa f0 6b 4f 00 00 02 04 05 b4 04 02 08 0a   ....kO..........</text>')
        out.append(f'<text x="{hx+26}" y="{hy+120}" font-family="{FONT_MONO}" font-size="9.5" fill="{T["accent"]}">Decodificación: IP 198.51.100.44:44120 -&gt; 10.10.20.5:80 • Flags: [SYN] Window=64240 Len=60 • Veredicto: DROP XDP (Score 0.998)</text>')

        out.append('</svg>')
        return "".join(out)

    else: # mobile (390x844)
        w, h = 390, 844
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        out.append(render_header_mobile(T, "Flujo en Vivo"))

        # Toolbar Móvil
        out.append(f'<rect x="16" y="96" width="358" height="48" rx="5" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<circle cx="28" cy="120" r="3.5" fill="{T["success"]}"/>')
        out.append(f'<text x="38" y="124" font-family="{FONT_MONO}" font-size="9" font-weight="700" fill="{T["text_primary"]}">14.2k pkts/s</text>')
        out.append(f'<rect x="120" y="106" width="140" height="28" rx="3" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
        out.append(f'<text x="130" y="124" font-family="{FONT_MONO}" font-size="8.5" fill="{T["text_secondary"]}">BPF: tcp.flags.syn==1</text>')
        out.append(f'<rect x="270" y="106" width="94" height="28" rx="3" fill="{T["accent"]}"/>')
        out.append(f'<text x="317" y="124" font-family="{FONT_SANS}" font-size="9.5" font-weight="700" fill="#FFFFFF" text-anchor="middle">Pausar</text>')

        # Stream de Tarjetas de Paquetes
        out.append(f'<rect x="16" y="152" width="358" height="490" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="28" y="174" font-family="{FONT_SANS}" font-size="11.5" font-weight="700" fill="{T["text_primary"]}">Paquetes Capturados (Kernel Hook):</text>')

        mpkts = [
            ("14:45:12.802", "eth0", "198.51.100.44:44120", "10.10.20.5:80", "TCP [SYN]", "0.998", "DROP XDP", T["critical"]),
            ("14:45:12.801", "eth1", "181.188.45.12:52341", "10.10.0.1:443", "TCP [ACK]", "0.012", "PASS", T["success"]),
            ("14:45:12.798", "eth0", "203.0.113.89:53", "10.10.20.12:53", "UDP [DNS]", "0.985", "RATE LIMIT", T["warning"]),
            ("14:45:12.795", "eth0", "198.51.100.44:44122", "10.10.20.5:80", "TCP [SYN]", "0.998", "DROP XDP", T["critical"]),
            ("14:45:12.790", "eth2", "10.200.0.128:55102", "10.200.0.50:80", "TCP [SYN]", "0.994", "SANDBOX DROP", T["sandbox"]),
        ]
        for mi, (mtime, mif, msrc, mdst, mproto, mscore, mact, mcol) in enumerate(mpkts):
            my = 186 + mi * 86
            out.append(f'<rect x="26" y="{my}" width="338" height="78" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
            out.append(f'<text x="36" y="{my+18}" font-family="{FONT_MONO}" font-size="9.5" font-weight="700" fill="{T["text_primary"]}">{mtime} • {mif}</text>')
            out.append(f'<rect x="260" y="{my+6}" width="94" height="20" rx="3" fill="{T["bg_panel"]}" stroke="{mcol}" stroke-width="0.8"/>')
            out.append(f'<text x="307" y="{my+19}" font-family="{FONT_MONO}" font-size="8" font-weight="700" fill="{mcol}" text-anchor="middle">{mact}</text>')
            
            out.append(f'<text x="36" y="{my+36}" font-family="{FONT_MONO}" font-size="9" fill="{T["text_secondary"]}">Src: {msrc}</text>')
            out.append(f'<text x="36" y="{my+50}" font-family="{FONT_MONO}" font-size="9" fill="{T["text_muted"]}">Dst: {mdst}</text>')
            out.append(f'<text x="36" y="{my+66}" font-family="{FONT_MONO}" font-size="8.5" fill="{T["accent"]}">{mproto} • Score ML: {mscore}</text>')

        # Inspector Hex Dump Móvil
        out.append(f'<rect x="16" y="650" width="358" height="120" rx="5" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="28" y="670" font-family="{FONT_SANS}" font-size="10.5" font-weight="800" fill="{T["text_primary"]}">Hex Dump Paquete Seleccionado:</text>')
        out.append(f'<rect x="26" y="678" width="338" height="52" rx="3" fill="{T["code_bg"]}"/>')
        out.append(f'<text x="34" y="696" font-family="{FONT_MONO}" font-size="8.5" fill="{T["code_text"]}">45 00 00 3c 7a 1b 40 00 40 06 c5 32</text>')
        out.append(f'<text x="34" y="712" font-family="{FONT_MONO}" font-size="8.5" fill="{T["code_text"]}">c6 33 64 2c 0a 0a 14 05 ac 58 00 50</text>')
        out.append(f'<text x="28" y="752" font-family="{FONT_MONO}" font-size="8.5" fill="{T["success_text"]}">• eBPF: XDP_DROP ejecutado en 12 µs</text>')

        out.append('</svg>')
        return "".join(out)

def gen_screen_06_alerts(T, device="pc"):
    """PANTALLA 6: Auditoría Forense y Gestión Centralizada de Alertas"""
    if device == "pc":
        w, h = 1440, 900
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        out.append(render_header_pc(T, "alerts"))

        # Filtros de Auditoría
        fx, fy, fw, fh = 24, 76, 1392, 54
        out.append(f'<rect x="{fx}" y="{fy}" width="{fw}" height="{fh}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(icon_alert(fx+16, fy+18, 18, T["critical"]))
        out.append(f'<text x="{fx+42}" y="{fy+32}" font-family="{FONT_SANS}" font-size="13" font-weight="800" fill="{T["text_primary"]}">Registro de Incidentes Forenses &amp; Trazabilidad de Evidencia</text>')
        out.append(f'<rect x="{fx+fw-320}" y="{fy+12}" width="150" height="30" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{fx+fw-245}" y="{fy+31}" font-family="{FONT_SANS}" font-size="11" font-weight="600" fill="{T["text_secondary"]}" text-anchor="middle">Exportar CSV</text>')
        out.append(f'<rect x="{fx+fw-160}" y="{fy+12}" width="145" height="30" rx="4" fill="{T["accent"]}"/>')
        out.append(f'<text x="{fx+fw-88}" y="{fy+31}" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="#FFFFFF" text-anchor="middle">Reporte PDF</text>')

        # Tabla Maestra de Incidentes
        tbl_x, tbl_y, tbl_w, tbl_h = 24, 142, 1392, 730
        out.append(f'<rect x="{tbl_x}" y="{tbl_y}" width="{tbl_w}" height="{tbl_h}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        
        th_y = tbl_y + 16
        out.append(f'<rect x="{tbl_x+16}" y="{th_y}" width="{tbl_w-32}" height="32" rx="3" fill="{T["bg_subtle"]}"/>')
        acols = [("INCIDENTE", 30), ("FECHA Y HORA", 150), ("VECTOR FORENSE", 280), ("ORIGEN / AS", 500), ("DESTINO", 680), ("SEVERIDAD", 820), ("ACCIÓN APLICADA", 960), ("ANALISTA", 1140), ("ESTADO", 1270)]
        for col_name, cpos in acols:
            out.append(f'<text x="{tbl_x+cpos}" y="{th_y+20}" font-family="{FONT_SANS}" font-size="10" font-weight="700" fill="{T["text_muted"]}">{col_name}</text>')

        alerts_data = [
            ("INC-2026-8841", "2026-10-17 14:44:12", "DDoS SYN Flood Volumétrico", "198.51.100.44 (AS15169)", "10.10.20.5", "CRÍTICA", "eBPF XDP Drop", "carlos.mendoza", "MITIGADO", T["critical"]),
            ("INC-2026-8840", "2026-10-17 14:42:05", "DNS Amplification 50x", "203.0.113.89 (AS13335)", "10.10.20.12", "ALTA", "BGP Flowspec Rate-Limit", "carlos.mendoza", "ACTIVO", T["warning"]),
            ("INC-2026-8839", "2026-10-17 14:38:50", "Port Scan Estructurado L4", "185.220.101.5 (AS60729)", "172.16.10.1", "MEDIA", "ACL Blackhole Temporal", "operador_n1", "RESUELTO", T["success"]),
            ("INC-2026-8838", "2026-10-17 14:30:11", "UDP Burst Anomaly (Sandbox)", "10.200.0.128 (VLAN 999)", "10.200.0.50", "MEDIA", "Validado en Sandbox ACL", "carlos.mendoza", "CERTIFICADO", T["sandbox"]),
            ("INC-2026-8837", "2026-10-17 14:15:22", "BGP Hijack Anomaly", "192.0.2.11 (AS3356)", "172.16.10.1", "CRÍTICA", "Filtro RPKI Desplegado", "admin_soc", "RESUELTO", T["success"]),
            ("INC-2026-8836", "2026-10-17 13:50:04", "ICMP Flood Reflection", "198.18.0.55 (AS2856)", "10.10.0.1", "BAJA", "Rate Limit In-line", "operador_n2", "RESUELTO", T["success"])
        ]
        for ai, (aid, atime, avec, asrc, adst, asev, aact, aop, ast, acol) in enumerate(alerts_data):
            ay_row = th_y + 40 + ai * 54
            out.append(f'<line x1="{tbl_x+16}" y1="{ay_row+44}" x2="{tbl_x+tbl_w-16}" y2="{ay_row+44}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
            out.append(f'<text x="{tbl_x+30}" y="{ay_row+26}" font-family="{FONT_MONO}" font-size="11" font-weight="700" fill="{T["text_primary"]}">{aid}</text>')
            out.append(f'<text x="{tbl_x+150}" y="{ay_row+26}" font-family="{FONT_MONO}" font-size="10.5" fill="{T["text_secondary"]}">{atime}</text>')
            out.append(f'<text x="{tbl_x+280}" y="{ay_row+26}" font-family="{FONT_SANS}" font-size="11.5" font-weight="600" fill="{T["text_primary"]}">{avec}</text>')
            out.append(f'<text x="{tbl_x+500}" y="{ay_row+26}" font-family="{FONT_MONO}" font-size="10.5" fill="{T["text_secondary"]}">{asrc}</text>')
            out.append(f'<text x="{tbl_x+680}" y="{ay_row+26}" font-family="{FONT_MONO}" font-size="10.5" fill="{T["text_secondary"]}">{adst}</text>')
            out.append(f'<rect x="{tbl_x+816}" y="{ay_row+10}" width="76" height="24" rx="3" fill="{T["critical_bg"] if asev=="CRÍTICA" else T["warning_bg"]}" stroke="{acol}" stroke-width="0.8"/>')
            out.append(f'<text x="{tbl_x+854}" y="{ay_row+26}" font-family="{FONT_MONO}" font-size="9" font-weight="700" fill="{acol}" text-anchor="middle">{asev}</text>')
            out.append(f'<text x="{tbl_x+960}" y="{ay_row+26}" font-family="{FONT_MONO}" font-size="10.5" fill="{T["text_primary"]}">{aact}</text>')
            out.append(f'<text x="{tbl_x+1140}" y="{ay_row+26}" font-family="{FONT_SANS}" font-size="11" fill="{T["text_muted"]}">{aop}</text>')
            out.append(f'<rect x="{tbl_x+1266}" y="{ay_row+10}" width="96" height="24" rx="3" fill="{T["success_bg"] if ast=="MITIGADO" or ast=="RESUELTO" else T["bg_subtle"]}" stroke="{acol}" stroke-width="0.8"/>')
            out.append(f'<text x="{tbl_x+1314}" y="{ay_row+26}" font-family="{FONT_MONO}" font-size="9" font-weight="700" fill="{acol}" text-anchor="middle">{ast}</text>')

        out.append('</svg>')
        return "".join(out)

    elif device == "tablet":
        w, h = 1024, 768
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        out.append(render_header_tablet(T, "alerts"))

        # Toolbar Tablet
        fx, fy, fw, fh = 20, 70, 984, 52
        out.append(f'<rect x="{fx}" y="{fy}" width="{fw}" height="{fh}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(icon_alert(fx+16, fy+17, 18, T["critical"]))
        out.append(f'<text x="{fx+40}" y="{fy+31}" font-family="{FONT_SANS}" font-size="12.5" font-weight="800" fill="{T["text_primary"]}">Registro de Incidentes Forenses &amp; Trazabilidad</text>')
        
        out.append(f'<rect x="{fx+fw-240}" y="{fy+11}" width="110" height="30" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border"]}" stroke-width="0.8"/>')
        out.append(f'<text x="{fx+fw-185}" y="{fy+29}" font-family="{FONT_SANS}" font-size="10" font-weight="600" fill="{T["text_secondary"]}" text-anchor="middle">Exportar CSV</text>')
        out.append(f'<rect x="{fx+fw-120}" y="{fy+11}" width="110" height="30" rx="4" fill="{T["accent"]}"/>')
        out.append(f'<text x="{fx+fw-65}" y="{fy+29}" font-family="{FONT_SANS}" font-size="10" font-weight="700" fill="#FFFFFF" text-anchor="middle">Reporte PDF</text>')

        # Tabla de Incidentes Tablet
        tbl_x, tbl_y, tbl_w, tbl_h = 20, 134, 984, 610
        out.append(f'<rect x="{tbl_x}" y="{tbl_y}" width="{tbl_w}" height="{tbl_h}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        
        th_y = tbl_y + 12
        out.append(f'<rect x="{tbl_x+12}" y="{th_y}" width="{tbl_w-24}" height="28" rx="3" fill="{T["bg_subtle"]}"/>')
        tacols = [("INCIDENTE", 20), ("TIMESTAMP", 130), ("VECTOR FORENSE", 230), ("ORIGEN / AS", 410), ("SEVERIDAD", 560), ("ACCIÓN APLICADA", 670), ("ANALISTA", 820), ("ESTADO", 900)]
        for col_name, cpos in tacols:
            out.append(f'<text x="{tbl_x+cpos}" y="{th_y+18}" font-family="{FONT_SANS}" font-size="9" font-weight="700" fill="{T["text_muted"]}">{col_name}</text>')

        talerts_data = [
            ("INC-2026-8841", "14:44:12", "DDoS SYN Flood Volumétrico", "198.51.100.44 (AS15169)", "CRÍTICA", "eBPF XDP Drop", "carlos.mendoza", "MITIGADO", T["critical"]),
            ("INC-2026-8840", "14:42:05", "DNS Amplification 50x", "203.0.113.89 (AS13335)", "ALTA", "Flowspec Rate-Limit", "carlos.mendoza", "ACTIVO", T["warning"]),
            ("INC-2026-8839", "14:38:50", "Port Scan Estructurado L4", "185.220.101.5 (AS60729)", "MEDIA", "ACL Blackhole Temporal", "operador_n1", "RESUELTO", T["success"]),
            ("INC-2026-8838", "14:30:11", "UDP Burst (Sandbox)", "10.200.0.128 (VLAN 999)", "MEDIA", "Validado en Sandbox", "carlos.mendoza", "CERTIFICADO", T["sandbox"]),
            ("INC-2026-8837", "14:15:22", "BGP Hijack Anomaly", "192.0.2.11 (AS3356)", "CRÍTICA", "Filtro RPKI Desplegado", "admin_soc", "RESUELTO", T["success"]),
            ("INC-2026-8836", "13:50:04", "ICMP Flood Reflection", "198.18.0.55 (AS2856)", "BAJA", "Rate Limit In-line", "operador_n2", "RESUELTO", T["success"])
        ]
        for ai, (aid, atime, avec, asrc, asev, aact, aop, ast, acol) in enumerate(talerts_data):
            ay_row = th_y + 36 + ai * 54
            out.append(f'<line x1="{tbl_x+12}" y1="{ay_row+44}" x2="{tbl_x+tbl_w-12}" y2="{ay_row+44}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
            out.append(f'<text x="{tbl_x+20}" y="{ay_row+24}" font-family="{FONT_MONO}" font-size="10" font-weight="700" fill="{T["text_primary"]}">{aid}</text>')
            out.append(f'<text x="{tbl_x+130}" y="{ay_row+24}" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_secondary"]}">{atime}</text>')
            out.append(f'<text x="{tbl_x+230}" y="{ay_row+24}" font-family="{FONT_SANS}" font-size="10.5" font-weight="600" fill="{T["text_primary"]}">{avec}</text>')
            out.append(f'<text x="{tbl_x+410}" y="{ay_row+24}" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_secondary"]}">{asrc}</text>')
            out.append(f'<rect x="{tbl_x+556}" y="{ay_row+8}" width="66" height="22" rx="3" fill="{T["bg_subtle"]}" stroke="{acol}" stroke-width="0.8"/>')
            out.append(f'<text x="{tbl_x+589}" y="{ay_row+23}" font-family="{FONT_MONO}" font-size="8" font-weight="700" fill="{acol}" text-anchor="middle">{asev}</text>')
            out.append(f'<text x="{tbl_x+670}" y="{ay_row+24}" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_primary"]}">{aact}</text>')
            out.append(f'<text x="{tbl_x+820}" y="{ay_row+24}" font-family="{FONT_SANS}" font-size="10" fill="{T["text_muted"]}">{aop}</text>')
            out.append(f'<text x="{tbl_x+940}" y="{ay_row+24}" font-family="{FONT_MONO}" font-size="8.5" font-weight="700" fill="{T["success"] if ast=="MITIGADO" or ast=="RESUELTO" else acol}" text-anchor="end">{ast}</text>')

        out.append('</svg>')
        return "".join(out)

    else: # mobile (390x844)
        w, h = 390, 844
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        out.append(render_header_mobile(T, "Alertas Forenses"))

        # Píldoras de filtro móvil
        out.append(f'<rect x="16" y="96" width="358" height="42" rx="5" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<rect x="24" y="104" width="70" height="26" rx="3" fill="{T["accent"]}"/>')
        out.append(f'<text x="59" y="121" font-family="{FONT_SANS}" font-size="9.5" font-weight="700" fill="#FFFFFF" text-anchor="middle">Todas (6)</text>')
        out.append(f'<rect x="100" y="104" width="74" height="26" rx="3" fill="{T["bg_subtle"]}" stroke="{T["critical_border"]}" stroke-width="0.8"/>')
        out.append(f'<text x="137" y="121" font-family="{FONT_SANS}" font-size="9" font-weight="700" fill="{T["critical"]}" text-anchor="middle">Crítica (2)</text>')
        out.append(f'<rect x="180" y="104" width="60" height="26" rx="3" fill="{T["bg_subtle"]}" stroke="{T["warning_border"]}" stroke-width="0.8"/>')
        out.append(f'<text x="210" y="121" font-family="{FONT_SANS}" font-size="9" font-weight="600" fill="{T["warning"]}" text-anchor="middle">Alta (1)</text>')
        out.append(f'<rect x="246" y="104" width="70" height="26" rx="3" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
        out.append(f'<text x="281" y="121" font-family="{FONT_SANS}" font-size="9" font-weight="600" fill="{T["text_muted"]}" text-anchor="middle">Media (2)</text>')

        # 4 Tarjetas de Incidentes Móvil
        out.append(f'<rect x="16" y="146" width="358" height="520" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="28" y="168" font-family="{FONT_SANS}" font-size="11.5" font-weight="700" fill="{T["text_primary"]}">Incidentes Prioritarios:</text>')

        m_alerts = [
            ("INC-2026-8841", "CRÍTICA", "DDoS SYN Flood Volumétrico", "198.51.100.44 (AS15169)", "eBPF XDP Drop", "carlos.mendoza", "MITIGADO", T["critical"]),
            ("INC-2026-8840", "ALTA", "DNS Amplification 50x", "203.0.113.89 (AS13335)", "BGP Flowspec Rate-Limit", "carlos.mendoza", "ACTIVO", T["warning"]),
            ("INC-2026-8839", "MEDIA", "Port Scan Estructurado L4", "185.220.101.5 (AS60729)", "ACL Blackhole Temporal", "operador_n1", "RESUELTO", T["success"]),
            ("INC-2026-8838", "MEDIA", "UDP Burst (Sandbox VLAN 999)", "10.200.0.128", "Validado en Sandbox ACL", "carlos.mendoza", "CERTIFICADO", T["sandbox"]),
        ]
        for mi, (maid, masev, mavec, masrc, maact, maop, mast, macol) in enumerate(m_alerts):
            my = 180 + mi * 116
            out.append(f'<rect x="26" y="{my}" width="338" height="106" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
            out.append(f'<text x="36" y="{my+18}" font-family="{FONT_MONO}" font-size="10" font-weight="700" fill="{T["text_primary"]}">{maid}</text>')
            out.append(f'<rect x="270" y="{my+6}" width="84" height="20" rx="3" fill="{T["bg_panel"]}" stroke="{macol}" stroke-width="0.8"/>')
            out.append(f'<text x="312" y="{my+19}" font-family="{FONT_MONO}" font-size="8.5" font-weight="700" fill="{macol}" text-anchor="middle">{masev}</text>')

            out.append(f'<text x="36" y="{my+36}" font-family="{FONT_SANS}" font-size="10.5" font-weight="700" fill="{T["text_primary"]}">{mavec}</text>')
            out.append(f'<text x="36" y="{my+52}" font-family="{FONT_MONO}" font-size="9" fill="{T["text_secondary"]}">Origen: {masrc}</text>')
            out.append(f'<text x="36" y="{my+68}" font-family="{FONT_MONO}" font-size="9" fill="{T["accent"]}">Acción: {maact}</text>')
            out.append(f'<text x="36" y="{my+86}" font-family="{FONT_SANS}" font-size="8.5" fill="{T["text_muted"]}">Analista: {maop} • Estado: <tspan font-weight="700" fill="{macol}">{mast}</tspan></text>')

        # Botón de exportación pericial móvil
        out.append(f'<rect x="16" y="680" width="358" height="44" rx="5" fill="{T["accent"]}"/>')
        out.append(f'<text x="195" y="707" font-family="{FONT_SANS}" font-size="11.5" font-weight="700" fill="#FFFFFF" text-anchor="middle">Generar Informe PDF Pericial</text>')

        out.append('</svg>')
        return "".join(out)

def gen_screen_07_blast_radius(T, device="pc"):
    """PANTALLA 7: Control de Mitigación y Blast Radius con Re-Autenticación Obligatoria"""
    if device == "pc":
        w, h = 1440, 900
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        out.append(render_header_pc(T, "blast_radius"))

        # Panel de Mitigaciones en Retención / Cola de Aprobación
        bx, by, bw, bh = 24, 76, 1392, 320
        out.append(f'<rect x="{bx}" y="{by}" width="{bw}" height="{bh}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{bx+20}" y="{by+28}" font-family="{FONT_SANS}" font-size="14" font-weight="800" fill="{T["text_primary"]}">Mitigaciones en Retención Preventiva (Control de Blast Radius)</text>')
        out.append(f'<text x="{bx+20}" y="{by+44}" font-family="{FONT_SANS}" font-size="10.5" fill="{T["text_muted"]}">Acciones pausadas para verificar el impacto en la red antes de su aplicación.</text>')

        # Fila de Cola Retenida
        out.append(f'<rect x="{bx+20}" y="{by+60}" width="{bw-40}" height="76" rx="4" fill="{T["warning_bg"]}" stroke="{T["warning_border"]}" stroke-width="1"/>')
        out.append(icon_alert(bx+34, by+76, 22, T["warning"]))
        out.append(f'<text x="{bx+68}" y="{by+84}" font-family="{FONT_SANS}" font-size="12" font-weight="700" fill="{T["warning_text"]}">Retención Preventiva: Bloqueo de Prefijo BGP /24 (198.51.100.0/24)</text>')
        out.append(f'<text x="{bx+68}" y="{by+100}" font-family="{FONT_MONO}" font-size="10.5" fill="{T["warning_text"]}">Razón: Supresión de ataque SYN Flood • Impacto estimado: 0 clientes legítimos afectados</text>')
        out.append(f'<text x="{bx+68}" y="{by+114}" font-family="{FONT_MONO}" font-size="10" font-weight="700" fill="{T["critical"]}">Temporizador de Rollback Automático: 04:18 min restantes</text>')

        # Modal Flotante de Re-Autenticación (Blast Radius Dialog)
        mx, my, mw, mh = 440, 360, 560, 480
        out.append(f'<rect x="{mx}" y="{my}" width="{mw}" height="{mh}" rx="8" fill="{T["bg_panel"]}" stroke="{T["critical"]}" stroke-width="1.8"/>')
        out.append(f'<rect x="{mx}" y="{my}" width="{mw}" height="6" rx="3" fill="{T["critical"]}"/>')
        
        out.append(icon_lock(mx+28, my+26, 24, T["critical"]))
        out.append(f'<text x="{mx+64}" y="{my+38}" font-family="{FONT_SANS}" font-size="15" font-weight="800" fill="{T["text_primary"]}">Confirmación de Mitigación</text>')
        out.append(f'<text x="{mx+64}" y="{my+54}" font-family="{FONT_SANS}" font-size="11" fill="{T["text_muted"]}">Despliegue de Regla de Mitigación (Prefijo /24)</text>')

        # Detalles del Impacto
        out.append(f'<rect x="{mx+28}" y="{my+72}" width="{mw-56}" height="110" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="1"/>')
        out.append(f'<text x="{mx+40}" y="{my+94}" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="{T["text_primary"]}">Parámetros de la Mitigación Solicitada:</text>')
        out.append(f'<text x="{mx+40}" y="{my+112}" font-family="{FONT_MONO}" font-size="10.5" fill="{T["text_secondary"]}">• Prefijo Objetivo: 198.51.100.0/24 (BGP Flowspec Drop)</text>')
        out.append(f'<text x="{mx+40}" y="{my+128}" font-family="{FONT_MONO}" font-size="10.5" fill="{T["text_secondary"]}">• Nodos de Aplicación: Core-Router-01, Carrier-Peering-02</text>')
        out.append(f'<text x="{mx+40}" y="{my+144}" font-family="{FONT_MONO}" font-size="10.5" fill="{T["success_text"]}">• Análisis de Tráfico: 99.4% Malicioso (Seguro de aplicar)</text>')
        out.append(f'<text x="{mx+40}" y="{my+160}" font-family="{FONT_MONO}" font-size="10.5" fill="{T["text_muted"]}">• Prueba Previa en Sandbox ACL: REALIZADA (Eficacia 99.85%)</text>')

        # Campo de Re-Autenticación
        out.append(f'<text x="{mx+28}" y="{my+206}" font-family="{FONT_SANS}" font-size="11.5" font-weight="700" fill="{T["text_primary"]}">Re-ingrese su Contraseña de Administrador o Token FIDO2:</text>')
        out.append(f'<rect x="{mx+28}" y="{my+216}" width="{mw-56}" height="42" rx="4" fill="{T["bg_panel"]}" stroke="{T["critical"]}" stroke-width="1.2"/>')
        out.append(icon_key(mx+38, my+228, 18, T["critical"]))
        out.append(f'<text x="{mx+66}" y="{my+243}" font-family="{FONT_MONO}" font-size="15" fill="{T["text_primary"]}">••••••••••••••••••••</text>')

        # Botones del Modal
        btn_y = my + 280
        out.append(f'<rect x="{mx+28}" y="{btn_y}" width="{mw-56}" height="46" rx="4" fill="{T["critical"]}"/>')
        out.append(icon_shield(mx+mw/2-110, btn_y+13, 20, "#FFFFFF"))
        out.append(f'<text x="{mx+mw/2+10}" y="{btn_y+28}" font-family="{FONT_SANS}" font-size="12.5" font-weight="800" fill="#FFFFFF" text-anchor="middle">Autorizar y Aplicar Mitigación</text>')

        out.append(f'<rect x="{mx+28}" y="{btn_y+56}" width="{mw-56}" height="42" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{mx+mw/2}" y="{btn_y+82}" font-family="{FONT_SANS}" font-size="12" font-weight="600" fill="{T["text_secondary"]}" text-anchor="middle">Cancelar Mitigación y Revertir</text>')

        out.append('</svg>')
        return "".join(out)

    elif device == "tablet":
        w, h = 1024, 768
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        out.append(render_header_tablet(T, "blast_radius"))

        # Top Warning Card
        bx, by, bw, bh = 20, 70, 984, 82
        out.append(f'<rect x="{bx}" y="{by}" width="{bw}" height="{bh}" rx="6" fill="{T["warning_bg"]}" stroke="{T["warning_border"]}" stroke-width="1.2"/>')
        out.append(icon_alert(bx+18, by+18, 22, T["warning"]))
        out.append(f'<text x="{bx+50}" y="{by+26}" font-family="{FONT_SANS}" font-size="12" font-weight="800" fill="{T["warning_text"]}">Retención Preventiva: Bloqueo de Prefijo BGP /24 (198.51.100.0/24)</text>')
        out.append(f'<text x="{bx+50}" y="{by+44}" font-family="{FONT_MONO}" font-size="10.5" fill="{T["warning_text"]}">Razón: Supresión de ataque SYN Flood • Impacto: 0 clientes legítimos afectados</text>')
        out.append(f'<text x="{bx+50}" y="{by+62}" font-family="{FONT_MONO}" font-size="10" font-weight="700" fill="{T["critical"]}">Temporizador de Rollback Automático: 04:18 min restantes</text>')

        # Modal Centrado de Confirmación Crítica
        mx, my, mw, mh = 232, 170, 560, 560
        out.append(f'<rect x="{mx}" y="{my}" width="{mw}" height="{mh}" rx="8" fill="{T["bg_panel"]}" stroke="{T["critical"]}" stroke-width="1.8"/>')
        out.append(f'<rect x="{mx}" y="{my}" width="{mw}" height="6" rx="3" fill="{T["critical"]}"/>')
        
        out.append(icon_lock(mx+28, my+26, 24, T["critical"]))
        out.append(f'<text x="{mx+64}" y="{my+38}" font-family="{FONT_SANS}" font-size="14.5" font-weight="800" fill="{T["text_primary"]}">Confirmación de Mitigación</text>')
        out.append(f'<text x="{mx+64}" y="{my+54}" font-family="{FONT_SANS}" font-size="10.5" fill="{T["text_muted"]}">Despliegue de Regla de Mitigación (Prefijo /24)</text>')

        # Parámetros
        out.append(f'<rect x="{mx+24}" y="{my+72}" width="{mw-48}" height="126" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="1"/>')
        out.append(f'<text x="{mx+36}" y="{my+94}" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="{T["text_primary"]}">Parámetros de la Mitigación Solicitada:</text>')
        out.append(f'<text x="{mx+36}" y="{my+114}" font-family="{FONT_MONO}" font-size="10" fill="{T["text_secondary"]}">• Prefijo Objetivo: 198.51.100.0/24 (BGP Flowspec Drop)</text>')
        out.append(f'<text x="{mx+36}" y="{my+132}" font-family="{FONT_MONO}" font-size="10" fill="{T["text_secondary"]}">• Nodos de Aplicación: Core-Router-01, Carrier-Peering-02</text>')
        out.append(f'<text x="{mx+36}" y="{my+150}" font-family="{FONT_MONO}" font-size="10" fill="{T["success_text"]}">• Análisis de Tráfico: 99.4% Malicioso (Seguro de aplicar)</text>')
        out.append(f'<text x="{mx+36}" y="{my+168}" font-family="{FONT_MONO}" font-size="10" fill="{T["text_muted"]}">• Prueba Previa en Sandbox ACL: REALIZADA (Eficacia 99.85%)</text>')

        # Campo Re-Autenticación
        out.append(f'<text x="{mx+24}" y="{my+226}" font-family="{FONT_SANS}" font-size="11.5" font-weight="700" fill="{T["text_primary"]}">Re-ingrese su Contraseña de Administrador o Token FIDO2:</text>')
        out.append(f'<rect x="{mx+24}" y="{my+238}" width="{mw-48}" height="42" rx="4" fill="{T["bg_panel"]}" stroke="{T["critical"]}" stroke-width="1.2"/>')
        out.append(icon_key(mx+34, my+250, 18, T["critical"]))
        out.append(f'<text x="{mx+62}" y="{my+265}" font-family="{FONT_MONO}" font-size="15" fill="{T["text_primary"]}">••••••••••••••••••••</text>')

        # Botones
        btn_y = my + 304
        out.append(f'<rect x="{mx+24}" y="{btn_y}" width="{mw-48}" height="46" rx="4" fill="{T["critical"]}"/>')
        out.append(icon_shield(mx+mw/2-110, btn_y+13, 20, "#FFFFFF"))
        out.append(f'<text x="{mx+mw/2+10}" y="{btn_y+28}" font-family="{FONT_SANS}" font-size="12" font-weight="800" fill="#FFFFFF" text-anchor="middle">Autorizar y Aplicar Mitigación</text>')

        out.append(f'<rect x="{mx+24}" y="{btn_y+58}" width="{mw-48}" height="42" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{mx+mw/2}" y="{btn_y+84}" font-family="{FONT_SANS}" font-size="11.5" font-weight="600" fill="{T["text_secondary"]}" text-anchor="middle">Cancelar Mitigación y Revertir</text>')

        out.append('</svg>')
        return "".join(out)

    else: # mobile (390x844)
        w, h = 390, 844
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        out.append(render_header_mobile(T, "Blast Radius"))

        # Warning Banner Móvil
        out.append(f'<rect x="16" y="96" width="358" height="68" rx="5" fill="{T["warning_bg"]}" stroke="{T["warning_border"]}" stroke-width="1.2"/>')
        out.append(icon_alert(26, 110, 18, T["warning"]))
        out.append(f'<text x="50" y="117" font-family="{FONT_SANS}" font-size="10.5" font-weight="800" fill="{T["warning_text"]}">Retención Preventiva: Bloqueo /24</text>')
        out.append(f'<text x="50" y="132" font-family="{FONT_MONO}" font-size="9" fill="{T["warning_text"]}">Prefijo 198.51.100.0/24 • SYN Flood</text>')
        out.append(f'<text x="50" y="148" font-family="{FONT_MONO}" font-size="9" font-weight="700" fill="{T["critical"]}">Rollback automático en 04:18 min</text>')

        # Tarjeta Parámetros Móvil
        out.append(f'<rect x="16" y="176" width="358" height="230" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="28" y="200" font-family="{FONT_SANS}" font-size="11.5" font-weight="700" fill="{T["text_primary"]}">Parámetros de la Mitigación:</text>')
        out.append(f'<rect x="26" y="210" width="338" height="180" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
        out.append(f'<text x="36" y="234" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_secondary"]}">• Prefijo: 198.51.100.0/24</text>')
        out.append(f'<text x="36" y="254" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_secondary"]}">• Nodos: Core-01, Peering-02</text>')
        out.append(f'<text x="36" y="274" font-family="{FONT_MONO}" font-size="9.5" fill="{T["success_text"]}">• Análisis: 99.4% Malicioso</text>')
        out.append(f'<text x="36" y="294" font-family="{FONT_MONO}" font-size="9.5" fill="{T["success_text"]}">• Clientes afectados: 0 legítimos</text>')
        out.append(f'<text x="36" y="314" font-family="{FONT_MONO}" font-size="9.5" fill="{T["sandbox_text"]}">• Sandbox ACL: APROBADA (99.85%)</text>')
        out.append(f'<text x="36" y="334" font-family="{FONT_MONO}" font-size="9" fill="{T["text_muted"]}">Firma: sha256:7f4a98d02bc149...</text>')

        # Re-Autenticación
        out.append(f'<rect x="16" y="420" width="358" height="150" rx="6" fill="{T["bg_panel"]}" stroke="{T["critical"]}" stroke-width="1.2"/>')
        out.append(f'<text x="28" y="444" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="{T["text_primary"]}">Re-Autenticación de Administrador:</text>')
        out.append(f'<rect x="26" y="456" width="338" height="42" rx="4" fill="{T["bg_subtle"]}" stroke="{T["critical"]}" stroke-width="1"/>')
        out.append(icon_key(36, 468, 16, T["critical"]))
        out.append(f'<text x="60" y="483" font-family="{FONT_MONO}" font-size="14" fill="{T["text_primary"]}">••••••••••••••••</text>')
        out.append(f'<text x="28" y="516" font-family="{FONT_SANS}" font-size="8.5" fill="{T["text_muted"]}">Zero-Trust Enforcement: Token FIDO2 requerido.</text>')

        # Botones Móvil
        out.append(f'<rect x="16" y="590" width="358" height="46" rx="5" fill="{T["critical"]}"/>')
        out.append(f'<text x="195" y="618" font-family="{FONT_SANS}" font-size="12" font-weight="800" fill="#FFFFFF" text-anchor="middle">Autorizar y Desplegar</text>')

        out.append(f'<rect x="16" y="646" width="358" height="42" rx="5" fill="{T["bg_subtle"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="195" y="672" font-family="{FONT_SANS}" font-size="11" font-weight="600" fill="{T["text_secondary"]}" text-anchor="middle">Rechazar y Rollback</text>')

        out.append('</svg>')
        return "".join(out)

def gen_screen_08_user_management(T, device="pc"):
    """PANTALLA 8: Administración de Usuarios y Políticas Zero-Trust RBAC"""
    if device == "pc":
        w, h = 1440, 900
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        out.append(render_header_pc(T, "users"))

        # Governance Status Banner
        bx, by, bw, bh = 24, 76, 1392, 54
        out.append(f'<rect x="{bx}" y="{by}" width="{bw}" height="{bh}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(icon_shield(bx+16, by+18, 18, T["accent"]))
        out.append(f'<text x="{bx+44}" y="{by+32}" font-family="{FONT_SANS}" font-size="13" font-weight="800" fill="{T["text_primary"]}">Gobernanza RBAC Zero-Trust • 2FA Obligatorio en 100% de Cuentas • Rotación 90 Días</text>')
        out.append(f'<rect x="{bx+bw-220}" y="{by+12}" width="200" height="30" rx="4" fill="{T["accent"]}"/>')
        out.append(f'<text x="{bx+bw-120}" y="{by+31}" font-family="{FONT_SANS}" font-size="11" font-weight="700" fill="#FFFFFF" text-anchor="middle">+ Registrar Operador</text>')

        # Tabla de Usuarios
        tbl_x, tbl_y, tbl_w, tbl_h = 24, 142, 1392, 730
        out.append(f'<rect x="{tbl_x}" y="{tbl_y}" width="{tbl_w}" height="{tbl_h}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        
        th_y = tbl_y + 16
        out.append(f'<rect x="{tbl_x+16}" y="{th_y}" width="{tbl_w-32}" height="32" rx="3" fill="{T["bg_subtle"]}"/>')
        ucols = [("USUARIO", 30), ("NOMBRE COMPLETO", 180), ("CORREO CORPORATIVO", 380), ("ROL RBAC", 620), ("ESTADO MFA", 780), ("ÚLTIMO ACCESO", 940), ("ESTADO", 1100), ("ACCIONES DE SEGURIDAD", 1220)]
        for col_name, cpos in ucols:
            out.append(f'<text x="{tbl_x+cpos}" y="{th_y+20}" font-family="{FONT_SANS}" font-size="10" font-weight="700" fill="{T["text_muted"]}">{col_name}</text>')

        users_data = [
            ("admin_soc", "Carlos Mendoza", "carlos.mendoza@tigo.net.bo", "SOC_ADMIN", "FIDO2 Key Activo", "Hoy 14:45 (TLS 1.3)", "ACTIVA", T["success"]),
            ("operador_n1", "Ana Rodríguez", "ana.rodriguez@tigo.net.bo", "OPERADOR_N2", "TOTP App Activo", "Hoy 13:20", "ACTIVA", T["success"]),
            ("auditor_sec", "Roberto Gómez", "roberto.gomez@tigo.net.bo", "AUDITOR", "TOTP App Activo", "Ayer 18:04", "ACTIVA", T["success"]),
            ("operador_n2", "Valeria Paz", "valeria.paz@tigo.net.bo", "OPERADOR_N1", "FIDO2 Key Activo", "16-Oct 11:15", "ACTIVA", T["success"]),
            ("temp_user01", "Usuario Temporal", "temp01@tigo.net.bo", "INVITADO", "Pendiente Config", "Primer Acceso", "ROTACIÓN OBLIG", T["warning"]),
            ("ex_operador", "Luis Salinas", "luis.salinas@tigo.net.bo", "OPERADOR_N1", "Revocado", "10-Oct 09:00", "BLOQUEADA", T["critical"])
        ]
        for ui, (uusr, unom, umail, urol, umfa, ulast, ust, ucol) in enumerate(users_data):
            uy_row = th_y + 40 + ui * 54
            out.append(f'<line x1="{tbl_x+16}" y1="{uy_row+44}" x2="{tbl_x+tbl_w-16}" y2="{uy_row+44}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
            out.append(f'<text x="{tbl_x+30}" y="{uy_row+26}" font-family="{FONT_MONO}" font-size="11" font-weight="700" fill="{T["text_primary"]}">{uusr}</text>')
            out.append(f'<text x="{tbl_x+180}" y="{uy_row+26}" font-family="{FONT_SANS}" font-size="11.5" font-weight="600" fill="{T["text_primary"]}">{unom}</text>')
            out.append(f'<text x="{tbl_x+380}" y="{uy_row+26}" font-family="{FONT_MONO}" font-size="11" fill="{T["text_secondary"]}">{umail}</text>')
            out.append(f'<rect x="{tbl_x+616}" y="{uy_row+10}" width="100" height="24" rx="3" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
            out.append(f'<text x="{tbl_x+666}" y="{uy_row+26}" font-family="{FONT_MONO}" font-size="9.5" font-weight="700" fill="{T["text_primary"]}" text-anchor="middle">{urol}</text>')
            out.append(f'<text x="{tbl_x+780}" y="{uy_row+26}" font-family="{FONT_SANS}" font-size="10.5" fill="{T["text_secondary"]}">{umfa}</text>')
            out.append(f'<text x="{tbl_x+940}" y="{uy_row+26}" font-family="{FONT_MONO}" font-size="10.5" fill="{T["text_muted"]}">{ulast}</text>')
            out.append(f'<rect x="{tbl_x+1096}" y="{uy_row+10}" width="90" height="24" rx="3" fill="{T["success_bg"] if ust=="ACTIVA" else T["warning_bg"] if ust=="ROTACIÓN OBLIG" else T["critical_bg"]}" stroke="{ucol}" stroke-width="0.8"/>')
            out.append(f'<text x="{tbl_x+1141}" y="{uy_row+26}" font-family="{FONT_MONO}" font-size="9" font-weight="700" fill="{ucol}" text-anchor="middle">{ust}</text>')
            out.append(f'<text x="{tbl_x+1220}" y="{uy_row+26}" font-family="{FONT_SANS}" font-size="11" font-weight="600" fill="{T["accent"]}">[ Permisos ] [ Forzar Clave ]</text>')

        out.append('</svg>')
        return "".join(out)

    elif device == "tablet":
        w, h = 1024, 768
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        out.append(render_header_tablet(T, "users"))

        # Toolbar Tablet
        bx, by, bw, bh = 20, 70, 984, 52
        out.append(f'<rect x="{bx}" y="{by}" width="{bw}" height="{bh}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(icon_shield(bx+16, by+17, 18, T["accent"]))
        out.append(f'<text x="{bx+42}" y="{by+31}" font-family="{FONT_SANS}" font-size="12.5" font-weight="800" fill="{T["text_primary"]}">Gobernanza RBAC Zero-Trust • 2FA Obligatorio al 100%</text>')
        out.append(f'<rect x="{bx+bw-180}" y="{by+11}" width="165" height="30" rx="4" fill="{T["accent"]}"/>')
        out.append(f'<text x="{bx+bw-97}" y="{by+29}" font-family="{FONT_SANS}" font-size="10.5" font-weight="700" fill="#FFFFFF" text-anchor="middle">+ Registrar Operador</text>')

        # Tabla Tablet
        tbl_x, tbl_y, tbl_w, tbl_h = 20, 134, 984, 610
        out.append(f'<rect x="{tbl_x}" y="{tbl_y}" width="{tbl_w}" height="{tbl_h}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        
        th_y = tbl_y + 12
        out.append(f'<rect x="{tbl_x+12}" y="{th_y}" width="{tbl_w-24}" height="28" rx="3" fill="{T["bg_subtle"]}"/>')
        tucols = [("USUARIO", 20), ("NOMBRE COMPLETO", 140), ("ROL RBAC", 320), ("ESTADO MFA", 470), ("ÚLTIMO ACCESO", 630), ("ESTADO", 770), ("ACCIONES", 880)]
        for col_name, cpos in tucols:
            out.append(f'<text x="{tbl_x+cpos}" y="{th_y+18}" font-family="{FONT_SANS}" font-size="9" font-weight="700" fill="{T["text_muted"]}">{col_name}</text>')

        tusers_data = [
            ("admin_soc", "Carlos Mendoza", "SOC_ADMIN", "FIDO2 Key Activo", "Hoy 14:45", "ACTIVA", T["success"]),
            ("operador_n1", "Ana Rodríguez", "OPERADOR_N2", "TOTP App Activo", "Hoy 13:20", "ACTIVA", T["success"]),
            ("auditor_sec", "Roberto Gómez", "AUDITOR", "TOTP App Activo", "Ayer 18:04", "ACTIVA", T["success"]),
            ("operador_n2", "Valeria Paz", "OPERADOR_N1", "FIDO2 Key Activo", "16-Oct 11:15", "ACTIVA", T["success"]),
            ("temp_user01", "Usuario Temporal", "INVITADO", "Pendiente Config", "Primer Acceso", "ROTACIÓN OBLIG", T["warning"])
        ]
        for ui, (uusr, unom, urol, umfa, ulast, ust, ucol) in enumerate(tusers_data):
            uy_row = th_y + 36 + ui * 54
            out.append(f'<line x1="{tbl_x+12}" y1="{uy_row+44}" x2="{tbl_x+tbl_w-12}" y2="{uy_row+44}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
            out.append(f'<text x="{tbl_x+20}" y="{uy_row+24}" font-family="{FONT_MONO}" font-size="10.5" font-weight="700" fill="{T["text_primary"]}">{uusr}</text>')
            out.append(f'<text x="{tbl_x+140}" y="{uy_row+24}" font-family="{FONT_SANS}" font-size="11" font-weight="600" fill="{T["text_primary"]}">{unom}</text>')
            out.append(f'<rect x="{tbl_x+316}" y="{uy_row+8}" width="90" height="22" rx="3" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
            out.append(f'<text x="{tbl_x+361}" y="{uy_row+23}" font-family="{FONT_MONO}" font-size="8.5" font-weight="700" fill="{T["text_primary"]}" text-anchor="middle">{urol}</text>')
            out.append(f'<text x="{tbl_x+470}" y="{uy_row+24}" font-family="{FONT_SANS}" font-size="10" fill="{T["text_secondary"]}">{umfa}</text>')
            out.append(f'<text x="{tbl_x+630}" y="{uy_row+24}" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_muted"]}">{ulast}</text>')
            out.append(f'<rect x="{tbl_x+766}" y="{uy_row+8}" width="80" height="22" rx="3" fill="{T["success_bg"] if ust=="ACTIVA" else T["warning_bg"]}" stroke="{ucol}" stroke-width="0.8"/>')
            out.append(f'<text x="{tbl_x+806}" y="{uy_row+23}" font-family="{FONT_MONO}" font-size="8" font-weight="700" fill="{ucol}" text-anchor="middle">{ust}</text>')
            out.append(f'<text x="{tbl_x+880}" y="{uy_row+24}" font-family="{FONT_SANS}" font-size="10.5" font-weight="600" fill="{T["accent"]}">[Permisos] [Clave]</text>')

        out.append('</svg>')
        return "".join(out)

    else: # mobile (390x844)
        w, h = 390, 844
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">']
        out.append(f'<rect width="{w}" height="{h}" fill="{T["bg_page"]}"/>')
        out.append(render_header_mobile(T, "Gestión RBAC"))

        # Header Resumen RBAC
        out.append(f'<rect x="16" y="96" width="358" height="52" rx="5" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(icon_shield(26, 110, 18, T["accent"]))
        out.append(f'<text x="50" y="116" font-family="{FONT_SANS}" font-size="11" font-weight="800" fill="{T["text_primary"]}">Políticas Zero-Trust Activas</text>')
        out.append(f'<text x="50" y="132" font-family="{FONT_MONO}" font-size="9" fill="{T["text_muted"]}">5 Cuentas • 100% MFA Activo</text>')
        out.append(f'<rect x="256" y="106" width="108" height="30" rx="3" fill="{T["accent"]}"/>')
        out.append(f'<text x="310" y="125" font-family="{FONT_SANS}" font-size="9.5" font-weight="700" fill="#FFFFFF" text-anchor="middle">+ Registrar Operador</text>')

        # 4 Tarjetas de Operadores Móvil
        out.append(f'<rect x="16" y="156" width="358" height="510" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="28" y="178" font-family="{FONT_SANS}" font-size="11.5" font-weight="700" fill="{T["text_primary"]}">Operadores de Seguridad:</text>')

        m_users = [
            ("carlos.mendoza", "Carlos Mendoza", "SOC_ADMIN", "FIDO2 Activo", "Hoy 14:45", "ACTIVA", T["success"]),
            ("ana.rodriguez", "Ana Rodríguez", "OPERADOR_N2", "TOTP Activo", "Hoy 13:20", "ACTIVA", T["success"]),
            ("valeria.paz", "Valeria Paz", "OPERADOR_N1", "FIDO2 Activo", "16-Oct 11:15", "ACTIVA", T["success"]),
            ("temp_user01", "Usuario Temporal", "INVITADO", "Sin Configurar", "Primer Acceso", "ROTACIÓN OBLIG", T["warning"])
        ]
        for mi, (muid, munom, murol, mumfa, mulast, must, mucol) in enumerate(m_users):
            my = 190 + mi * 114
            out.append(f'<rect x="26" y="{my}" width="338" height="104" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
            out.append(f'<text x="36" y="{my+20}" font-family="{FONT_SANS}" font-size="11.5" font-weight="700" fill="{T["text_primary"]}">{munom}</text>')
            out.append(f'<rect x="260" y="{my+8}" width="94" height="20" rx="3" fill="{T["bg_panel"]}" stroke="{mucol}" stroke-width="0.8"/>')
            out.append(f'<text x="307" y="{my+21}" font-family="{FONT_MONO}" font-size="8.5" font-weight="700" fill="{mucol}" text-anchor="middle">{must}</text>')

            out.append(f'<text x="36" y="{my+38}" font-family="{FONT_MONO}" font-size="9" fill="{T["text_secondary"]}">User: {muid} • Rol: {murol}</text>')
            out.append(f'<text x="36" y="{my+54}" font-family="{FONT_SANS}" font-size="9" fill="{T["text_muted"]}">MFA: {mumfa} • Acceso: {mulast}</text>')
            out.append(f'<rect x="36" y="{my+68}" width="84" height="24" rx="3" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="0.8"/>')
            out.append(f'<text x="78" y="{my+84}" font-family="{FONT_SANS}" font-size="9" font-weight="600" fill="{T["accent"]}" text-anchor="middle">Permisos</text>')
            out.append(f'<rect x="130" y="{my+68}" width="110" height="24" rx="3" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="0.8"/>')
            out.append(f'<text x="185" y="{my+84}" font-family="{FONT_SANS}" font-size="9" font-weight="600" fill="{T["critical"]}" text-anchor="middle">Forzar Rotación</text>')

        out.append('</svg>')
        return "".join(out)

# -------------------------------------------------------------------------
# COMPILADOR MAESTRO Y EXPORTADOR A PNG, SVG Y PDF
# -------------------------------------------------------------------------
SCREENS = [
    ("01_login", "Login de Alta Seguridad (2FA + Email + Rate Limit)", gen_screen_01_login),
    ("01b_cambio_password", "Cambio Obligatorio de Contraseña por Defecto", gen_screen_01b_password_change),
    ("02_dashboard", "Dashboard Central SOC & Telemetría Global", gen_screen_02_dashboard),
    ("03_topology", "Topología de Infraestructura ISP & Sensores eBPF", gen_screen_03_topology),
    ("04_monitoring_charts", "Métricas ML LightGBM & Matriz de Confusión", gen_screen_04_monitoring_charts),
    ("05_realtime_stream", "Flujo eBPF en Vivo & Deserialización de Paquetes", gen_screen_05_realtime_stream),
    ("06_alerts", "Auditoría Forense & Gestión Centralizada de Alertas", gen_screen_06_alerts),
    ("07_blast_radius", "Blast Radius & Re-Autenticación Crítica", gen_screen_07_blast_radius),
    ("08_user_management", "Gobernanza RBAC & Políticas Zero-Trust", gen_screen_08_user_management),
    ("09_sandbox_acl", "Sandbox ACL & Reglas Dinámicas en Subred Aislada", gen_screen_09_sandbox_acl),
]

DEVICES = [
    ("pc", "PC Desktop (1440x900)"),
    ("tablet", "Tablet (1024x768)"),
    ("celular", "Móvil PWA (390x844)")
]

def main():
    print("=" * 80)
    print("GENERADOR MAESTRO DE MOCKUPS DE ALTA FIDELIDAD - TIGO SOC PLATFORM")
    print("Cero Emojis • Login 2FA • Cambio Clave Forzado • Sandbox ACL • Tema Claro & Oscuro")
    print("=" * 80)

    light_pdfs = []
    dark_pdfs = []

    for theme_key in ["light", "dark"]:
        T = THEMES[theme_key]
        theme_dir = LIGHT_DIR if theme_key == "light" else DARK_DIR
        print(f"\nGenerando Mockups para: {T['name'].upper()} en {theme_dir}...")

        theme_pdf_list = []

        for screen_id, screen_desc, gen_fn in SCREENS:
            for dev_id, dev_desc in DEVICES:
                file_prefix = f"mockup_{screen_id}_{dev_id}"
                svg_path = os.path.join(theme_dir, f"{file_prefix}.svg")
                png_path = os.path.join(theme_dir, f"{file_prefix}.png")
                pdf_path = os.path.join(theme_dir, f"{file_prefix}.pdf")

                # 1. Generar contenido SVG
                svg_content = gen_fn(T, dev_id)
                with open(svg_path, "w", encoding="utf-8") as f:
                    f.write(svg_content)

                # 2. Convertir a PNG de alta resolución usando rsvg-convert
                subprocess.run(["rsvg-convert", "-f", "png", "-o", png_path, svg_path], check=True)

                # 3. Convertir a PDF vectorial
                subprocess.run(["rsvg-convert", "-f", "pdf", "-o", pdf_path, svg_path], check=True)

                theme_pdf_list.append(pdf_path)
                print(f"  [OK] {theme_key.upper():5} | {screen_id:20} | {dev_id:8} -> {file_prefix}.png/.pdf")

        # Compilar PDF maestro por tema
        theme_master_pdf = os.path.join(FINAL_DIR, f"COMPILACION_MOCKUPS_TEMA_{theme_key.upper()}.pdf")
        writer = PdfWriter()
        for p in theme_pdf_list:
            reader = PdfReader(p)
            for page in reader.pages:
                writer.add_page(page)
        with open(theme_master_pdf, "wb") as f_out:
            writer.write(f_out)
        print(f"\n>>> Compilación Maestro de {T['name']} creada exitosamente:")
        print(f"    {theme_master_pdf} ({len(theme_pdf_list)} páginas)")

        if theme_key == "light":
            light_pdfs = theme_pdf_list
        else:
            dark_pdfs = theme_pdf_list

    # Compilar Gran Entrega Maestra Combinada (Claro + Oscuro = 60 páginas)
    grand_master_pdf = os.path.join(FINAL_DIR, "ENTREGA_FINAL_MOCKUPS_CLARO_Y_OSCURO.pdf")
    grand_writer = PdfWriter()
    for p in light_pdfs + dark_pdfs:
        reader = PdfReader(p)
        for page in reader.pages:
            grand_writer.add_page(page)
    with open(grand_master_pdf, "wb") as f_out:
        grand_writer.write(f_out)
    
    print("\n" + "=" * 80)
    print(f"GRAN ENTREGA FINAL MAESTRA GENERADA (60 PÁGINAS):")
    print(f"-> {grand_master_pdf}")
    print("=" * 80)

if __name__ == "__main__":
    main()
