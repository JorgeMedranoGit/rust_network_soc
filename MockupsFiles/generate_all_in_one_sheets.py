#!/usr/bin/env python3
"""
Generador Maestro de Láminas Todo-en-Uno (Web + Tablet + Móvil en la Misma Página)
Plataforma de Ciberdefensa ISP - Tigo SOC

Cumple estrictamente con las directivas actualizadas:
1. Cobertura exhaustiva estilo manual técnico: cada pantalla explica entre 5 y 6 controles clave sin limitarse a 4.
2. Posicionamiento impecable de pines de callout [1]..[6] en esquinas e indicadores sin tapar textos, etiquetas ni botones.
3. Menos colores en las descripciones (estricto Blanco y Negro / Monocromo). Textos más amplios y descriptivos.
4. Sin mención a llamadas de API técnicas (como POST /api/...). Se describe claramente "QUÉ HACE" y su "FUNCIÓN EN EL SISTEMA".
5. Panel "DESCRIPCIÓN MULTIPLATAFORMA (WEB, TABLET Y MÓVIL)" con 3 fichas estructuradas.
6. La página de introducción (Lámina 00) aparece UNA SOLA VEZ para todo el documento al inicio de la entrega formal. El tema oscuro inicia directamente en la Página 1.
7. Compilación única de 21 páginas uniformes (1 Intro + 10 Láminas Blanco + 10 Láminas Oscuro).
8. Limpieza rigurosa de la carpeta Final/, dejando únicamente Blanco/, Oscuro/ y ENTREGA_FINAL_MOCKUPS.pdf.
"""

import os
import sys
import shutil
import subprocess
from pypdf import PdfWriter, PdfReader

BASE_DIR = os.path.dirname(os.path.abspath(__file__))
FINAL_DIR = os.path.join(BASE_DIR, "Final")
DIR_BLANCO = os.path.join(FINAL_DIR, "Blanco")
DIR_OSCURO = os.path.join(FINAL_DIR, "Oscuro")

os.makedirs(DIR_BLANCO, exist_ok=True)
os.makedirs(DIR_OSCURO, exist_ok=True)

sys.path.insert(0, BASE_DIR)
from generate_master_duo_mockups import (
    THEMES, esc, icon_shield, icon_lock, icon_check, icon_alert,
    icon_network, icon_chart, icon_cpu, icon_flask,
    icon_user, icon_logout, icon_play, icon_key,
    gen_screen_01_login,
    gen_screen_01b_password_change,
    gen_screen_02_dashboard,
    gen_screen_03_topology,
    gen_screen_04_monitoring_charts,
    gen_screen_05_realtime_stream,
    gen_screen_06_alerts,
    gen_screen_07_blast_radius,
    gen_screen_08_user_management,
    gen_screen_09_sandbox_acl
)

FONT_SANS = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif"
FONT_MONO = "'SF Mono', 'Fira Code', 'Roboto Mono', Menlo, Consolas, monospace"

def extract_inner_svg(svg_str):
    """Extrae el contenido gráfico interno de un SVG descartando las etiquetas envolventes"""
    first_close = svg_str.find('>')
    last_open = svg_str.rfind('</svg>')
    if first_close != -1 and last_open != -1:
        return svg_str[first_close + 1 : last_open]
    return svg_str

def draw_callout_pin(x, y, num, color="#0284C7", r=13, font_size=12):
    """Dibuja un pin flotante circular numerado [1]..[6] calibrado para cada escala"""
    return f'''<g transform="translate({x},{y})">
        <circle cx="0" cy="0" r="{r}" fill="{color}" stroke="#FFFFFF" stroke-width="2.5"/>
        <text x="0" y="{font_size * 0.38}" font-family="{FONT_SANS}" font-size="{font_size}" font-weight="900" fill="#FFFFFF" text-anchor="middle">{num}</text>
    </g>'''

def draw_flow_arrow(x1, y1, x2, y2, color):
    """Dibuja una flecha vectorial horizontal limpia"""
    out = []
    out.append(f'<line x1="{x1}" y1="{y1}" x2="{x2-12}" y2="{y2}" stroke="{color}" stroke-width="2" stroke-linecap="round"/>')
    out.append(f'<polygon points="{x2},{y2} {x2-12},{y2-5} {x2-12},{y2+5}" fill="{color}"/>')
    return "".join(out)

# -------------------------------------------------------------------------
# LÁMINA 00: INTRODUCCIÓN FORMAL (SIN RECUADROS, SIN FOOTER, TÍTULO CENTRADO)
# -------------------------------------------------------------------------
def render_sheet_00_portada(T):
    W, H = 2400, 1500
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}">']
    out.append(f'<rect width="{W}" height="{H}" fill="{T["bg_page"]}"/>')

    # 1. TÍTULO PRINCIPAL CENTRADO
    out.append(f'<text x="1200" y="85" font-family="{FONT_SANS}" font-size="28" font-weight="900" fill="{T["text_primary"]}" text-anchor="middle">Mockups del Sistema web aplicando Light Gradient Boosting Machine para la optimización del tiempo y precisión en la detección de exfiltración de datos</text>')
    out.append(f'<text x="1200" y="122" font-family="{FONT_SANS}" font-size="16" font-weight="600" fill="{T["accent"]}" text-anchor="middle">Proyecto de Grado en Ingeniería de Sistemas Informáticos • Plataforma Tigo SOC (ISP AS27839) • Postulante: Jorge Esteban Medrano</text>')

    # 2. COLUMNA 1: OBJETIVO (x: 80, width: 680)
    c1_x = 80
    c1_y = 175
    out.append(f'<text x="{c1_x}" y="{c1_y}" font-family="{FONT_SANS}" font-size="24" font-weight="800" fill="{T["text_primary"]}">Objetivo.</text>')
    
    obj_texts = [
        "El objetivo es diseñar los mockups multiplataforma (Web y Móvil) para reflejar con",
        "máxima fidelidad técnica la funcionalidad principal del sistema de ciberdefensa,",
        "la experiencia de usuario y la adaptación responsiva ante flujos masivos de red.",
        "",
        "Se demuestra cómo interactúan los analistas y operadores de seguridad en tiempo real,",
        "garantizando la coherencia de los objetivos funcionales tanto en consolas de control",
        "de alta resolución como en dispositivos táctiles de movilidad para turnos de guardia.",
        "",
        "El sistema integra detección algorítmica acelerada con Light Gradient Boosting Machine",
        "(LightGBM) para optimizar el tiempo de inferencia (< 1 ms) y la precisión en la detección",
        "de exfiltración de datos, junto con mitigación reactiva inmediata a nivel de kernel Linux",
        "mediante hooks eBPF/XDP y filtrado de paquetes en la infraestructura de transporte ISP."
    ]
    for idx, line in enumerate(obj_texts):
        if line == "":
            continue
        out.append(f'<text x="{c1_x}" y="{c1_y + 40 + idx*26}" font-family="{FONT_SANS}" font-size="14.5" font-weight="500" fill="#0F172A">{esc(line)}</text>')

    # 3. COLUMNA 2 Y 3: USABILIDAD Y FUNCIONALIDAD (x: 820, width: 1500)
    c2_x = 820
    c2_y = 175
    out.append(f'<text x="{c2_x}" y="{c2_y}" font-family="{FONT_SANS}" font-size="24" font-weight="800" fill="{T["text_primary"]}">Usabilidad y Funcionalidad.</text>')

    # Bloque 1: Modo Claro y Modo Oscuro
    out.append(f'<text x="{c2_x}" y="{c2_y+40}" font-family="{FONT_SANS}" font-size="17" font-weight="800" fill="{T["accent"]}">Modo Claro y Modo Oscuro</text>')
    txt_themes = [
        ("• Modo Oscuro:", "Diseñado para salas de control y monitoreo continuo (SOC/NOC 24/7). Reduce drásticamente la fatiga visual"),
        ("", "de los analistas que supervisan flujos ininterrumpidos de paquetes y telemetría en ambientes con baja iluminación."),
        ("• Modo Claro:", "Optimizado para auditorías diurnas, revisión forense documental y generación de informes ejecutivos en"),
        ("", "estaciones con alta iluminación ambiental, asegurando máxima nitidez y confort de lectura."),
        ("• Accesibilidad:", "Ambos esquemas cumplen la norma WCAG 2.2 con ratios de contraste superiores a 12:1 en textos y controles.")
    ]
    for idx, (label, text) in enumerate(txt_themes):
        ty = c2_y + 64 + idx * 21
        if label:
            out.append(f'<text x="{c2_x}" y="{ty}" font-family="{FONT_SANS}" font-size="13.5" fill="#0F172A"><tspan font-weight="800">{esc(label)}</tspan> {esc(text)}</text>')
        else:
            out.append(f'<text x="{c2_x+14}" y="{ty}" font-family="{FONT_SANS}" font-size="13.5" font-weight="500" fill="#0F172A">{esc(text)}</text>')

    # Bloque 2: Justificación de paleta de colores
    out.append(f'<text x="{c2_x}" y="{c2_y+190}" font-family="{FONT_SANS}" font-size="17" font-weight="800" fill="{T["accent"]}">Justificación de paleta de colores</text>')
    txt_colors = [
        ("• Azul (#0284C7):", "Navegación principal, selección de filtros y botones de acción primaria en los paneles del sistema."),
        ("• Verde (#16A34A):", "Confirmación de tráfico normal verificado, cifrado TLS 1.3, autenticación 2FA válida y enlaces activos."),
        ("• Rojo (#DC2626):", "Alertas críticas de exfiltración de datos, ataques volumétricos DDoS y acciones de bloqueo o descarte."),
        ("• Ámbar (#D97706):", "Avisos de límites de tasa (rate limiting), retención preventiva y advertencias de umbrales operacionales."),
        ("• Índigo (#4F46E5):", "Identificación visual exclusiva para la subred de pruebas y simulación aislada (VLAN 999 Sandbox).")
    ]
    for idx, (label, text) in enumerate(txt_colors):
        ty = c2_y + 214 + idx * 21
        out.append(f'<text x="{c2_x}" y="{ty}" font-family="{FONT_SANS}" font-size="13.5" fill="#0F172A"><tspan font-weight="800">{esc(label)}</tspan> {esc(text)}</text>')

    # Bloque 3: Justificación del Diseño
    out.append(f'<text x="{c2_x}" y="{c2_y+340}" font-family="{FONT_SANS}" font-size="17" font-weight="800" fill="{T["accent"]}">Justificación del Diseño</text>')
    txt_design = [
        ("• Consistencia Multiplataforma:", "La versión Web Desktop (1920x1080) despliega la consola completa de control con telemetría en vivo;"),
        ("", "la versión Tablet (1024x768) adapta los controles para triaje táctil en salas de servidores; y la versión Móvil (390x844) ofrece"),
        ("", "supervisión esencial con tarjetas apiladas, visualización de alertas críticas y confirmaciones seguras para turnos de guardia."),
        ("• Tipografía Tabular de Red:", "Para direcciones IP, prefijos CIDR, puertos TCP/UDP, tasas pps y marcas temporales se emplean fuentes"),
        ("", "con números tabulares de ancho fijo, asegurando una perfecta alineación vertical y legibilidad inmediata en todas las tablas."),
        ("• Estructura Modular Limpia:", "Distribución en cuadrícula con jerarquía clara que permite evaluar el estado de la red en menos de 3 segundos.")
    ]
    for idx, (label, text) in enumerate(txt_design):
        ty = c2_y + 364 + idx * 21
        if label:
            out.append(f'<text x="{c2_x}" y="{ty}" font-family="{FONT_SANS}" font-size="13.5" fill="#0F172A"><tspan font-weight="800">{esc(label)}</tspan> {esc(text)}</text>')
        else:
            out.append(f'<text x="{c2_x+14}" y="{ty}" font-family="{FONT_SANS}" font-size="13.5" font-weight="500" fill="#0F172A">{esc(text)}</text>')

    # 4. ÍNDICE CONTEMPLANDO MODO CLARO Y MODO OSCURO (y: 680 a 1450)
    iy = 680
    out.append(f'<text x="{c1_x}" y="{iy}" font-family="{FONT_SANS}" font-size="24" font-weight="800" fill="{T["text_primary"]}">Índice.</text>')
    out.append(f'<text x="{c1_x+100}" y="{iy}" font-family="{FONT_SANS}" font-size="15" font-weight="600" fill="{T["accent"]}">Estructura del Manual: Sección I (Modo Claro) y Sección II (Modo Oscuro) • 21 Láminas Totales</text>')

    col1_x = 80
    col2_x = 1240
    col_w = 1080

    # Cabeceras de Sección
    out.append(f'<text x="{col1_x}" y="{iy+38}" font-family="{FONT_SANS}" font-size="16.5" font-weight="900" fill="{T["text_primary"]}">SECCIÓN I: MODO CLARO (Páginas 2 a 11)</text>')
    out.append(f'<line x1="{col1_x}" y1="{iy+48}" x2="{col1_x+col_w}" y2="{iy+48}" stroke="{T["accent"]}" stroke-width="2"/>')

    out.append(f'<text x="{col2_x}" y="{iy+38}" font-family="{FONT_SANS}" font-size="16.5" font-weight="900" fill="{T["text_primary"]}">SECCIÓN II: MODO OSCURO (Páginas 12 a 21)</text>')
    out.append(f'<line x1="{col2_x}" y1="{iy+48}" x2="{col2_x+col_w}" y2="{iy+48}" stroke="{T["accent"]}" stroke-width="2"/>')

    catalog_light = [
        ("Pág. 02", "Lámina 1", "01_login", "Login",
         "Autenticación corporativa con correo SSO, validación de token 2FA/TOTP y control de intentos fallidos."),
        ("Pág. 03", "Lámina 2", "01b_cambio_password", "Rotación Forzada de Contraseña",
         "Primer acceso obligatorio con medidor dinámico de entropía (94 bits) y checklist de 5 reglas ISO 27001."),
        ("Pág. 04", "Lámina 3", "02_dashboard", "Dashboard Central SOC",
         "Telemetría de transporte ISP (42.8 Gbps), KPIs de red, curva temporal de tráfico y mitigación en kernel."),
        ("Pág. 05", "Lámina 4", "03_topology", "Topología de Red Core GNS3",
         "Infraestructura carrier AS27839, enlaces 10G/40G e inspector de interfaces perimetrales eBPF/XDP."),
        ("Pág. 06", "Lámina 5", "04_monitoring_charts", "Métricas ML LightGBM",
         "Inferencia algorítmica en 0.42 ms, precisión de 99.42%, ranking de features y matriz de confusión forense."),
        ("Pág. 07", "Lámina 6", "05_realtime_stream", "Inspección eBPF en Tiempo Real",
         "Captura continua de 14.2k paquetes/seg, filtros de red BPF y deserialización hexadecimal y ASCII."),
        ("Pág. 08", "Lámina 7", "06_alerts", "Auditoría Forense y Alertas",
         "Registro inmutable de incidentes, clasificación por severidad crítica, firma SHA-256 y reporte pericial."),
        ("Pág. 09", "Lámina 8", "07_blast_radius", "Control de Blast Radius",
         "Cuarentena preventiva de mitigaciones, temporizador de rollback de 300s y re-autenticación FIDO2."),
        ("Pág. 10", "Lámina 9", "08_user_management", "Gobernanza RBAC Zero-Trust",
         "Gestión de operadores SOC, cumplimiento del 100% de doble factor 2FA y matriz de privilegios granulares."),
        ("Pág. 11", "Lámina 10", "09_sandbox_acl", "Sandbox ACL en Subred Aislada",
         "Ambiente de pruebas en VLAN 999 (10.200.0.0/24), inyección de tráfico simulado, consola eBPF y promoción.")
    ]

    catalog_dark = [
        ("Pág. 12", "Lámina 1", "01_login", "Login",
         "Autenticación corporativa con correo SSO, validación de token 2FA/TOTP y control de intentos fallidos."),
        ("Pág. 13", "Lámina 2", "01b_cambio_password", "Rotación Forzada de Contraseña",
         "Primer acceso obligatorio con medidor dinámico de entropía (94 bits) y checklist de 5 reglas ISO 27001."),
        ("Pág. 14", "Lámina 3", "02_dashboard", "Dashboard Central SOC",
         "Telemetría de transporte ISP (42.8 Gbps), KPIs de red, curva temporal de tráfico y mitigación en kernel."),
        ("Pág. 15", "Lámina 4", "03_topology", "Topología de Red Core GNS3",
         "Infraestructura carrier AS27839, enlaces 10G/40G e inspector de interfaces perimetrales eBPF/XDP."),
        ("Pág. 16", "Lámina 5", "04_monitoring_charts", "Métricas ML LightGBM",
         "Inferencia algorítmica en 0.42 ms, precisión de 99.42%, ranking de features y matriz de confusión forense."),
        ("Pág. 17", "Lámina 6", "05_realtime_stream", "Inspección eBPF en Tiempo Real",
         "Captura continua de 14.2k paquetes/seg, filtros de red BPF y deserialización hexadecimal y ASCII."),
        ("Pág. 18", "Lámina 7", "06_alerts", "Auditoría Forense y Alertas",
         "Registro inmutable de incidentes, clasificación por severidad crítica, firma SHA-256 y reporte pericial."),
        ("Pág. 19", "Lámina 8", "07_blast_radius", "Control de Blast Radius",
         "Cuarentena preventiva de mitigaciones, temporizador de rollback de 300s y re-autenticación FIDO2."),
        ("Pág. 20", "Lámina 9", "08_user_management", "Gobernanza RBAC Zero-Trust",
         "Gestión de operadores SOC, cumplimiento del 100% de doble factor 2FA y matriz de privilegios granulares."),
        ("Pág. 21", "Lámina 10", "09_sandbox_acl", "Sandbox ACL en Subred Aislada",
         "Ambiente de pruebas en VLAN 999 (10.200.0.0/24), inyección de tráfico simulado, consola eBPF y promoción.")
    ]

    for col_idx, (cx, cat) in enumerate([(col1_x, catalog_light), (col2_x, catalog_dark)]):
        for ci, (p_doc, p_lamina, p_id, p_title, p_desc) in enumerate(cat):
            cy = iy + 62 + ci * 68
            
            # Línea 1: Pág. XX • Lámina Y [id] + Título
            out.append(f'<text x="{cx}" y="{cy+18}" font-family="{FONT_MONO}" font-size="12.5" font-weight="800" fill="{T["accent"]}">{p_doc} • {p_lamina} <tspan fill="{T["text_primary"]}">[{p_id}]</tspan></text>')
            out.append(f'<text x="{cx+340}" y="{cy+18}" font-family="{FONT_SANS}" font-size="15" font-weight="900" fill="#0F172A">{esc(p_title)}</text>')
            
            # Línea 2: Descripción en alto contraste (negro/slate 900, nada de plomo)
            out.append(f'<text x="{cx}" y="{cy+40}" font-family="{FONT_SANS}" font-size="13" font-weight="500" fill="#0F172A">{esc(p_desc)}</text>')
            
            # Línea 3: Separador sutil
            out.append(f'<line x1="{cx}" y1="{cy+54}" x2="{cx+col_w}" y2="{cy+54}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')

    out.append('</svg>')
    return "".join(out)

# -------------------------------------------------------------------------
# LÁMINAS TODO-EN-UNO: PÁGINAS 1 A 10 (DINÁMICO 4, 5 O 6 CONTROLES)
# -------------------------------------------------------------------------
def render_all_in_one_sheet(
    T,
    sheet_num,
    screen_id,
    screen_title,
    svg_web_content,
    svg_tablet_content,
    svg_mobile_content,
    multiplatform_notes,
    button_cards_data,
    flow_steps_data
):
    W, H = 2400, 1500
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}">']
    out.append(f'<rect width="{W}" height="{H}" fill="{T["bg_page"]}"/>')

    # 1. TÍTULO CENTRADO SIN RECUADRO Y SIN DESCRIPCIONES EN PARÉNTESIS O SECUNDARIAS
    out.append(f'<text x="1200" y="55" font-family="{FONT_SANS}" font-size="28" font-weight="900" fill="{T["text_primary"]}" text-anchor="middle">Página {sheet_num} - {esc(screen_title)}</text>')

    # 2. ZONA DE MOCKUPS INTEGRADOS (y: 85 a 935)
    # [VISTA 1: WEB DESKTOP (PC)]
    web_x, web_y, web_w, web_h = 40, 85, 1380, 850
    out.append(f'<rect x="{web_x}" y="{web_y}" width="{web_w}" height="{web_h}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
    out.append(f'<rect x="{web_x}" y="{web_y}" width="{web_w}" height="32" rx="6" fill="{T["bg_subtle"]}"/>')
    out.append(f'<rect x="{web_x}" y="{web_y+20}" width="{web_w}" height="12" fill="{T["bg_subtle"]}"/>')
    out.append(f'<circle cx="{web_x+18}" cy="{web_y+16}" r="4" fill="#EF4444"/>')
    out.append(f'<circle cx="{web_x+34}" cy="{web_y+16}" r="4" fill="#F59E0B"/>')
    out.append(f'<circle cx="{web_x+50}" cy="{web_y+16}" r="4" fill="#22C55E"/>')
    out.append(f'<rect x="{web_x+75}" y="{web_y+7}" width="380" height="18" rx="3" fill="{T["bg_panel"]}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
    out.append(f'<text x="{web_x+85}" y="{web_y+20}" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_muted"]}">https://soc.tigo.net.bo/index.html#{esc(screen_id)}</text>')
    out.append(f'<text x="{web_x+web_w-20}" y="{web_y+21}" font-family="{FONT_SANS}" font-size="11" font-weight="800" fill="{T["accent"]}" text-anchor="end">1. VISTA WEB DESKTOP (1440 x 900 PX)</text>')
    
    out.append(f'<g transform="translate({web_x},{web_y+32}) scale(0.9583, 0.9088)">')
    out.append(svg_web_content)
    out.append('</g>')

    # [VISTA 2: TABLET (1024x768)]
    tab_x, tab_y, tab_w, tab_h = 1445, 85, 535, 420
    out.append(f'<rect x="{tab_x}" y="{tab_y}" width="{tab_w}" height="{tab_h}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
    out.append(f'<rect x="{tab_x}" y="{tab_y}" width="{tab_w}" height="28" rx="6" fill="{T["bg_subtle"]}"/>')
    out.append(f'<rect x="{tab_x}" y="{tab_y+16}" width="{tab_w}" height="12" fill="{T["bg_subtle"]}"/>')
    out.append(f'<text x="{tab_x+16}" y="{tab_y+18}" font-family="{FONT_SANS}" font-size="10" font-weight="800" fill="{T["accent"]}">2. VISTA TABLET (1024 x 768 PX) • TOUCH</text>')
    out.append(f'<circle cx="{tab_x+tab_w-16}" cy="{tab_y+14}" r="3.5" fill="{T["success"]}"/>')
    
    out.append(f'<g transform="translate({tab_x},{tab_y+28}) scale(0.5224, 0.5104)">')
    out.append(svg_tablet_content)
    out.append('</g>')

    # [PANEL: DESCRIPCIÓN MULTIPLATAFORMA (x: 1445, y: 520, w: 535, h: 415)]
    note_x, note_y, note_w, note_h = 1445, 520, 535, 415
    out.append(f'<rect x="{note_x}" y="{note_y}" width="{note_w}" height="{note_h}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
    out.append(f'<rect x="{note_x}" y="{note_y}" width="{note_w}" height="32" rx="6" fill="{T["bg_subtle"]}"/>')
    out.append(f'<rect x="{note_x}" y="{note_y+20}" width="{note_w}" height="12" fill="{T["bg_subtle"]}"/>')
    out.append(f'<text x="{note_x+18}" y="{note_y+21}" font-family="{FONT_SANS}" font-size="12" font-weight="800" fill="{T["text_primary"]}">DESCRIPCIÓN MULTIPLATAFORMA (WEB, TABLET Y MÓVIL)</text>')
    
    lbl_color = "#334155" if T["name"] == "Tema Claro" else "#94A3B8"
    val_color = "#0F172A" if T["name"] == "Tema Claro" else "#F8FAFC"

    out.append(f'<g transform="translate({note_x+16},{note_y+42})">')
    for ni, (d_name, d_lines) in enumerate(multiplatform_notes):
        ny_item = ni * 122
        out.append(f'<rect x="0" y="{ny_item}" width="{note_w-32}" height="114" rx="5" fill="{T["bg_subtle"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<rect x="0" y="{ny_item}" width="{note_w-32}" height="28" rx="5" fill="{T["bg_panel"]}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
        out.append(f'<circle cx="14" cy="{ny_item+14}" r="4.5" fill="{T["accent"]}"/>')
        out.append(f'<text x="26" y="{ny_item+19}" font-family="{FONT_SANS}" font-size="12.5" font-weight="800" fill="{T["text_primary"]}">{esc(d_name)}</text>')
        out.append(f'<text x="14" y="{ny_item+48}" font-family="{FONT_SANS}" font-size="12" font-weight="600" fill="{val_color}">{esc(d_lines[0])}</text>')
        out.append(f'<text x="14" y="{ny_item+70}" font-family="{FONT_SANS}" font-size="11.5" fill="{val_color}">{esc(d_lines[1])}</text>')
        if len(d_lines) > 2:
            out.append(f'<text x="14" y="{ny_item+92}" font-family="{FONT_SANS}" font-size="11" fill="{lbl_color}">{esc(d_lines[2])}</text>')
    out.append('</g>')

    # [VISTA 3: MÓVIL SMARTPHONE PWA (390x844)]
    mob_x, mob_y, mob_w, mob_h = 2005, 85, 355, 850
    device_bezel = "#334155" if T["name"] == "Tema Claro" else "#1E293B"
    out.append(f'<rect x="{mob_x}" y="{mob_y}" width="{mob_w}" height="{mob_h}" rx="24" fill="{device_bezel}" stroke="{T["border"]}" stroke-width="1.5"/>')
    out.append(f'<rect x="{mob_x+10}" y="{mob_y+10}" width="{mob_w-20}" height="{mob_h-20}" rx="18" fill="{T["bg_page"]}"/>')
    out.append(f'<rect x="{mob_x+mob_w/2-36}" y="{mob_y+14}" width="72" height="10" rx="5" fill="{device_bezel}"/>')
    out.append(f'<circle cx="{mob_x+mob_w/2+22}" cy="{mob_y+19}" r="2.5" fill="#0B0F17"/>')
    
    out.append(f'<g transform="translate({mob_x+10},{mob_y+26}) scale(0.8589, 0.9526)">')
    out.append(svg_mobile_content)
    out.append('</g>')
    out.append(f'<text x="{mob_x+mob_w/2}" y="{mob_y+mob_h-12}" font-family="{FONT_SANS}" font-size="10.5" font-weight="800" fill="#FFFFFF" text-anchor="middle">3. VISTA MÓVIL PWA (390 x 844 PX)</text>')

    # 3. ZONA DE ESPECIFICACIONES (y: 955 a 1475)
    mat_x, mat_y, mat_w, mat_h = 40, 955, 2320, 515
    out.append(f'<rect x="{mat_x}" y="{mat_y}" width="{mat_w}" height="{mat_h}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
    
    out.append(f'<rect x="{mat_x}" y="{mat_y}" width="{mat_w}" height="36" rx="6" fill="{T["bg_subtle"]}"/>')
    out.append(f'<rect x="{mat_x}" y="{mat_y+22}" width="{mat_w}" height="14" fill="{T["bg_subtle"]}"/>')
    out.append(f'<text x="{mat_x+24}" y="{mat_y+23}" font-family="{FONT_SANS}" font-size="14.5" font-weight="900" fill="{T["text_primary"]}">ESPECIFICACIÓN DE FUNCIONALIDAD DE CONTROLES &amp; EFECTO EN EL SISTEMA</text>')
    out.append(f'<text x="{mat_x+mat_w-24}" y="{mat_y+23}" font-family="{FONT_MONO}" font-size="11" font-weight="700" fill="{T["accent"]}" text-anchor="end">CALIDAD DE SOFTWARE ISO/IEC 25010 • PROTOCOLOS Y RESPUESTAS DEL BACKEND</text>')

    # DISPOSICIÓN DINÁMICA DE TARJETAS (4, 5 O 6 CONTROLES)
    num_cards = len(button_cards_data)
    padding_x = 20
    gap = 14 if num_cards == 5 else (11 if num_cards >= 6 else 16)
    total_gaps = (num_cards - 1) * gap
    card_w = (mat_w - (2 * padding_x) - total_gaps) / num_cards
    
    pin_r = 10.5 if num_cards >= 6 else 11.5
    pin_font_size = 11.5 if num_cards >= 6 else 12.5
    title_font_size = 11 if num_cards >= 6 else (11.5 if num_cards == 5 else 12.5)
    label_font_size = 9.5 if num_cards >= 6 else 10.2
    body_font_size = 11 if num_cards >= 6 else (11.6 if num_cards == 5 else 12)
    body2_font_size = 10.5 if num_cards >= 6 else (11.2 if num_cards == 5 else 11.5)

    for bi, (b_pin, b_title, b_user_action, b_what_it_does, b_system_effect, b_multiplat) in enumerate(button_cards_data):
        bx = mat_x + padding_x + bi * (card_w + gap)
        by = mat_y + 46
        
        out.append(f'<rect x="{bx}" y="{by}" width="{card_w}" height="295" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<rect x="{bx}" y="{by}" width="{card_w}" height="33" rx="4" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
        
        out.append(f'<circle cx="{bx+17}" cy="{by+16.5}" r="{pin_r}" fill="{T["accent"]}"/>')
        out.append(f'<text x="{bx+17}" y="{by+20.5}" font-family="{FONT_SANS}" font-size="{pin_font_size}" font-weight="900" fill="#FFFFFF" text-anchor="middle">{b_pin}</text>')
        out.append(f'<text x="{bx+34}" y="{by+20.5}" font-family="{FONT_SANS}" font-size="{title_font_size}" font-weight="800" fill="{val_color}">{esc(b_title)}</text>')
        
        ty = by + 48
        details = [
            ("ACCIÓN DEL OPERADOR:", b_user_action),
            ("QUÉ HACE:", b_what_it_does),
            ("FUNCIÓN EN EL SISTEMA:", b_system_effect),
            ("ADAPTACIÓN MULTIPLATAFORMA:", b_multiplat)
        ]
        stride = 59
        for d_lbl, d_val in details:
            out.append(f'<text x="{bx+10}" y="{ty}" font-family="{FONT_MONO}" font-size="{label_font_size}" font-weight="700" fill="{lbl_color}">{d_lbl}</text>')
            out.append(f'<text x="{bx+10}" y="{ty+15.5}" font-family="{FONT_SANS}" font-size="{body_font_size}" font-weight="600" fill="{val_color}">{esc(d_val[0])}</text>')
            if len(d_val) > 1:
                out.append(f'<text x="{bx+10}" y="{ty+30}" font-family="{FONT_SANS}" font-size="{body2_font_size}" fill="{val_color}">{esc(d_val[1])}</text>')
            ty += stride

    # 4. SECCIÓN FLUJOS Y TRANSICIONES (y: 1365 a 1455)
    flow_y = mat_y + 355
    out.append(f'<line x1="{mat_x+20}" y1="{flow_y}" x2="{mat_x+mat_w-20}" y2="{flow_y}" stroke="{T["border"]}" stroke-width="1"/>')
    out.append(f'<text x="{mat_x+24}" y="{flow_y+20}" font-family="{FONT_SANS}" font-size="13.5" font-weight="800" fill="{T["text_primary"]}">Flujos y Transiciones</text>')
    
    num_steps = len(flow_steps_data)
    node_w = 340
    node_gap = (mat_w - 100 - (num_steps * node_w)) / (num_steps - 1) if num_steps > 1 else 0
    
    for si, (step_num, step_name, step_detail, step_color) in enumerate(flow_steps_data):
        nx = mat_x + 30 + si * (node_w + node_gap)
        ny = flow_y + 30
        
        out.append(f'<rect x="{nx}" y="{ny}" width="{node_w}" height="70" rx="4" fill="{T["bg_subtle"]}" stroke="{T["border"]}" stroke-width="1"/>')
        out.append(f'<text x="{nx+16}" y="{ny+20}" font-family="{FONT_MONO}" font-size="11" font-weight="800" fill="{val_color}">PASO {step_num}: {esc(step_name).upper()}</text>')
        out.append(f'<text x="{nx+16}" y="{ny+39}" font-family="{FONT_SANS}" font-size="11.5" font-weight="600" fill="{val_color}">{esc(step_detail[0])}</text>')
        out.append(f'<text x="{nx+16}" y="{ny+56}" font-family="{FONT_SANS}" font-size="10.5" fill="{lbl_color}">{esc(step_detail[1])}</text>')
        
        if si < num_steps - 1:
            arrow_start_x = nx + node_w + 10
            arrow_end_x = nx + node_w + node_gap - 10
            arrow_y = ny + 35
            out.append(draw_flow_arrow(arrow_start_x, arrow_y, arrow_end_x, arrow_y, T["accent"]))

    out.append('</svg>')
    return "".join(out)

# -------------------------------------------------------------------------
# METADATOS TÉCNICOS DETALLADOS (TÍTULOS SIN PARÉNTESIS, 5 O 6 CONTROLES)
# -------------------------------------------------------------------------
SHEETS_METADATA = [
    # ── PÁGINA 1: LOGIN (5 CONTROLES) ────────────────────────────────────
    {
        "screen_id": "01_login",
        "screen_title": "Login",
        "gen_fn": gen_screen_01_login,
        "web_pins": [(476, 262, 1), (476, 338, 2), (476, 434, 3), (476, 510, 4), (944, 582, 5)],
        "tablet_pins": [(284, 172, 1), (284, 236, 2), (284, 300, 3), (284, 364, 4), (720, 426, 5)],
        "mobile_pins": [(24, 192, 1), (24, 254, 2), (24, 316, 3), (24, 378, 4), (350, 442, 5)],
        "multiplatform_notes": [
            ("1. Consola Web Desktop (1440 x 900 px)", [
                "Entorno de acceso con autenticación mTLS 1.3 y auto-completado de correo corporativo SSO.",
                "Diseño centrado con advertencia de intentos fallidos en tiempo real y soporte de contraseña."
            ]),
            ("2. Tablet Táctil Profesional (1024 x 768 px)", [
                "Campos de entrada con áreas táctiles ampliadas a 48px y teclado numérico nativo para código 2FA.",
                "Transición automática de foco entre casillas numéricas para agilizar el ingreso en salas SOC."
            ]),
            ("3. Móvil Smartphone PWA (390 x 844 px)", [
                "Disposición vertical compacta con formulario continuo adaptado a ergonomía de pulgar.",
                "Integración con aplicaciones de autenticación móvil y botón de validación de ancho completo."
            ])
        ],
        "button_cards": [
            ("1", "Campo [ Correo SSO ]",
             ["Digita identidad corporativa", "Formato: usuario@tigo.net.bo"],
             ["Valida sintaxis y dominio Tigo", "Verifica autorización de analista"],
             ["Restringe acceso a personal SOC", "Impide intentos de dominios externos"],
             ["Web: Foco y auto-completado", "Tablet: Teclado correo • Móvil: Teclado @"]),

            ("2", "Campo [ Contraseña ]",
             ["Ingresa contraseña de red", "Soporta ocultar y mostrar texto"],
             ["Aplica hash seguro Argon2id", "con salt de 128 bits en backend"],
             ["Previene fuga visual de clave", "Protege sesión en terminal compartida"],
             ["Web: Botón ocular integrado", "Tablet: Toque revelar • Móvil: Timeout 5s"]),

            ("3", "Casillas [ Token 2FA ]",
             ["Ingresa código de 6 dígitos", "Generado por Google Authenticator"],
             ["Valida HMAC-SHA1 RFC 6238", "Tolerancia estricta de ±30 segundos"],
             ["Garantiza segundo factor MFA", "Neutraliza ataques de replay token"],
             ["Web: 6 casillas tabuladas", "Tablet: Auto-foco • Móvil: Teclado numérico"]),

            ("4", "Banner [ Rate Limit ]",
             ["Supervisa intentos restantes (3)", "Monitorea intentos fallidos"],
             ["Bloquea dirección IP por 15 min", "tras agotar los 3 intentos fallidos"],
             ["Protege el acceso al sistema", "Registra eventos en auditoría"],
             ["Web: Banner visible en tarjeta", "Tablet: Alerta superior • Móvil: Banner compacto"]),

            ("5", "Botón [ Validar e Ingresar ]",
             ["Pulsa para enviar formulario", "Valida los 3 factores juntos"],
             ["Emite cookie HttpOnly con JWT", "y activa sesión cifrada en SOC"],
             ["Concede acceso según perfil RBAC", "Verifica si clave es temporal"],
             ["Web: Clic con soporte mTLS", "Tablet: Touch 48px • Móvil: Botón ancho completo"])
        ],
        "flow_steps": [
            (1, "Ingreso Credenciales", ["Correo SSO institucional", "y contraseña de red"], "#0284C7"),
            (2, "Validación 2FA", ["Código TOTP 6 dígitos", "RFC 6238 sincronizado"], "#D97706"),
            (3, "Control Rate Limit", ["Verificación en memoria", "menor a 3 intentos fallidos"], "#4F46E5"),
            (4, "¿Clave Temporal?", ["Si es 'admin' va a Pág 2", "Si es permanente va a Pág 3"], "#16A34A")
        ]
    },

    # ── PÁGINA 2: ROTACIÓN FORZADA DE CONTRASEÑA (5 CONTROLES) ───────────
    {
        "screen_id": "01b_cambio_password",
        "screen_title": "Rotación Forzada de Contraseña",
        "gen_fn": gen_screen_01b_password_change,
        "web_pins": [(418, 156, 1), (418, 302, 2), (1022, 331, 3), (418, 440, 4), (1022, 599, 5)],
        "tablet_pins": [(244, 88, 1), (244, 170, 2), (764, 228, 3), (244, 290, 4), (764, 420, 5)],
        "mobile_pins": [(20, 116, 1), (20, 202, 2), (358, 258, 3), (20, 318, 4), (358, 468, 5)],
        "multiplatform_notes": [
            ("1. Consola Web Desktop (1440 x 900 px)", [
                "Panel de rotación forzada que intercepta cuentas con clave inicial por defecto 'admin'.",
                "Despliegue horizontal completo del medidor de entropía y checklist de 5 reglas ISO 27001."
            ]),
            ("2. Tablet Táctil Profesional (1024 x 768 px)", [
                "Botón ocular táctil para alternar visualización y prevenir errores de digitación.",
                "Checklist reactivo con marcas de verificación instantáneas y targets cómodos para teclado táctil."
            ]),
            ("3. Móvil Smartphone PWA (390 x 844 px)", [
                "Estructura vertical secuencial que guía al operador paso a paso hasta cumplir el 100% de entropía.",
                "Ocultamiento automático de contraseña ante pérdida de foco para salvaguardar credenciales."
            ])
        ],
        "button_cards": [
            ("1", "Banner [ Clave Inicial 'admin' ]",
             ["Informa interceptación de cuenta", "Bloquea navegación a la consola"],
             ["Detecta flag must_change_password", "en perfil de operador al iniciar"],
             ["Evita operar con claves de fábrica", "Cumple directiva Zero-Trust"],
             ["Web: Alerta roja en tarjeta", "Tablet: Banner superior • Móvil: Ficha alerta"]),

            ("2", "Campo [ Nueva Contraseña ]",
             ["Escribe nueva contraseña segura", "Observa retroalimentación activa"],
             ["Evalúa fortaleza en tiempo real", "sin enviar texto plano al backend"],
             ["Garantiza creación de secreto robusto", "Rechaza patrones comunes y fechas"],
             ["Web: Input con foco automático", "Tablet: Teclado táctil • Móvil: Input adaptado"]),

            ("3", "Medidor [ Entropía 94 Bits ]",
             ["Supervisa barra de fortaleza", "Comprueba alcance de 4 segmentos"],
             ["Calcula log2(N^L) con zxcvbn", "midiendo variedad de caracteres"],
             ["Asegura resistencia a fuerza bruta", "Garantiza entropía mayor a 80 bits"],
             ["Web: Barra de 4 segmentos verde", "Tablet: Indicador visual • Móvil: Barra compacta"]),

            ("4", "Checklist [ 5 Criterios ISO ]",
             ["Verifica requisitos satisfechos", "14 chars, símbolos, mayúsculas"],
             ["Chequea reglas en memoria local", "marcando con visto verde cada logro"],
             ["Obliga cumplimiento de ISO 27001", "Elimina contraseñas vulnerables"],
             ["Web: Grid de dos columnas", "Tablet: Lista vertical • Móvil: Fichas compactas"]),

            ("5", "Botón [ Actualizar y Acceder ]",
             ["Confirma actualización de clave", "Activo solo con 100% de requisitos"],
             ["Aplica hash Argon2id y salt", "Desactiva flag de rotación requerida"],
             ["Permite acceso al Dashboard SOC", "Descarta últimas 5 claves previas"],
             ["Web: Botón verde destacado", "Tablet: Touch 48px • Móvil: Botón ancho completo"])
        ],
        "flow_steps": [
            (1, "Detección 'admin'", ["Backend detecta clave", "temporal bloqueando SOC"], "#DC2626"),
            (2, "Digitación Clave", ["Operador ingresa clave", "y evalúa entropía en vivo"], "#D97706"),
            (3, "Checklist ISO", ["Validación de 14 chars,", "números y símbolos"], "#4F46E5"),
            (4, "Hash Argon2id", ["Registro seguro y paso", "directo a Dashboard (Pág 3)"], "#16A34A")
        ]
    },

    # ── PÁGINA 3: DASHBOARD CENTRAL SOC (6 CONTROLES) ────────────────────
    {
        "screen_id": "02_dashboard",
        "screen_title": "Dashboard Central SOC",
        "gen_fn": gen_screen_02_dashboard,
        "web_pins": [(1380, 24, 1), (44, 85, 2), (395, 85, 3), (44, 225, 4), (1340, 225, 5), (44, 575, 6)],
        "tablet_pins": [(970, 22, 1), (30, 80, 2), (280, 80, 3), (30, 260, 4), (970, 260, 5), (30, 520, 6)],
        "mobile_pins": [(350, 60, 1), (26, 105, 2), (200, 105, 3), (26, 250, 4), (340, 250, 5), (26, 520, 6)],
        "multiplatform_notes": [
            ("1. Consola Web Desktop (1440 x 900 px)", [
                "Supervisión global continua con gráfica panorámica de 42.8 Gbps y distribución de tráfico.",
                "Grid horizontal de 4 KPIs críticos con actualización en subsegundo vía telemetría eBPF."
            ]),
            ("2. Tablet Táctil Profesional (1024 x 768 px)", [
                "Reorganización de KPIs en matriz ergonómica 2x2 para lectura ágil sin necesidad de scroll.",
                "Curva de tráfico interactiva con tooltip al toque para inspeccionar picos anómalos L4/L7."
            ]),
            ("3. Móvil Smartphone PWA (390 x 844 px)", [
                "Tarjetas de telemetría apiladas con métricas esenciales de paquetes por segundo e inferencia ML.",
                "Gráfico simplificado de alta visibilidad para monitoreo de enlaces troncales en guardias."
            ])
        ],
        "button_cards": [
            ("1", "Botón [ Cerrar Sesión ]",
             ["Finaliza turno del analista", "Pulsa en cabecera superior"],
             ["Invalida token JWT en servidor", "Limpia cookies de sesión segura"],
             ["Previene accesos no supervisados", "Registra timestamp de desconexión"],
             ["Web: Botón con icono en header", "Tablet: Icono táctil • Móvil: Opción cabecera"]),

            ("2", "KPI [ 42.8 Gbps Total ]",
             ["Supervisa ancho de banda ISP", "Evalúa carga de tráfico de red"],
             ["Agrega telemetría de peering", "Diferencia tráfico benigno vs saturado"],
             ["Alerta ante picos volumétricos", "Garantiza capacidad de transporte"],
             ["Web: Tarjeta KPI destacada", "Tablet: Cuadrante 2x2 • Móvil: Ficha superior"]),

            ("3", "KPI [ 1.84M pps ]",
             ["Monitorea tasa de procesamiento", "Controla latencia de 12 µs"],
             ["Suma contadores atómicos eBPF", "Inspecciona volumen de paquetes"],
             ["Garantiza rendimiento sin drops", "Confirma operación óptima de CPU"],
             ["Web: Tarjeta métrica precisa", "Tablet: Cuadrante táctil • Móvil: Ficha apilada"]),

            ("4", "Gráfica [ Tráfico Vivo ]",
             ["Observa curvas Gbps vs Ataque", "Identifica desviaciones de flujo"],
             ["Renderiza series temporales WebGL", "Destaca paquetes anómalos en rojo"],
             ["Anticipa incidentes de saturación", "Permite respuesta rápida en SOC"],
             ["Web: Gráfica ancha interactiva", "Tablet: Tooltip táctil • Móvil: Curva simple"]),

            ("5", "Selector [ Ventana (1h/6h/24h) ]",
             ["Cambia intervalo de supervisión", "Alterna entre 1h, 6h y 24h"],
             ["Recalcula agregados temporales", "Actualiza gráficos e incidentes"],
             ["Facilita comparación histórica", "Permite triaje retrospectivo"],
             ["Web: Píldoras horizontales", "Tablet: Botones táctiles • Móvil: Menú selector"]),

            ("6", "Tabla [ Incidentes Mitigados ]",
             ["Examina últimos eventos eBPF", "Verifica IP origen y veredicto"],
             ["Consulta bitácora en tiempo real", "Muestra protocolo y acción DROP"],
             ["Permite auditar mitigaciones vivas", "Enlaza a módulo de alertas forenses"],
             ["Web: Tabla completa 5 columnas", "Tablet: Scroll táctil • Móvil: Lista resumida"])
        ],
        "flow_steps": [
            (1, "Streaming Telemetría", ["Recepción de métricas eBPF", "en subsegundo en tiempo real"], "#0284C7"),
            (2, "Supervisión de KPIs", ["Control de Gbps, paquetes/s", "y estado de enlaces core"], "#4F46E5"),
            (3, "¿Anomalía Detectada?", ["Alerta visual de pico anómalo", "clasificado por LightGBM"], "#DC2626"),
            (4, "Mitigación Rápida", ["Paso a Sandbox ACL (Pág 10)", "o aplicación de regla en Blast Rad"], "#16A34A")
        ]
    },

    # ── PÁGINA 4: TOPOLOGÍA DE RED CORE GNS3 (6 CONTROLES) ───────────────
    {
        "screen_id": "03_topology",
        "screen_title": "Topología de Red Core GNS3",
        "gen_fn": gen_screen_03_topology,
        "web_pins": [(544, 224, 1), (344, 364, 2), (644, 544, 3), (544, 704, 4), (1400, 286, 5), (1400, 540, 6)],
        "tablet_pins": [(350, 190, 1), (240, 310, 2), (440, 440, 3), (350, 560, 4), (980, 240, 5), (980, 480, 6)],
        "mobile_pins": [(355, 202, 1), (355, 268, 2), (355, 400, 3), (355, 466, 4), (355, 558, 5), (355, 612, 6)],
        "multiplatform_notes": [
            ("1. Consola Web Desktop (1440 x 900 px)", [
                "Diagrama interactivo de topología carrier AS27839 con enlaces 10G/40G y routers BGP VyOS.",
                "Panel lateral desplegable con telemetría de interfaces físicas y acciones de aislamiento eBPF."
            ]),
            ("2. Tablet Táctil Profesional (1024 x 768 px)", [
                "Navegación táctil con arrastre suave y toques optimizados de 32px para selección de routers.",
                "Cajón modal derecho para inspeccionar estado de puertos eth0-eth3 sin tapar el mapa."
            ]),
            ("3. Móvil Smartphone PWA (390 x 844 px)", [
                "Lista estructurada de nodos de infraestructura con indicadores semánticos de estado operativo.",
                "Ficha expandible de telemetría y botones de acción rápida para contención de incidentes."
            ])
        ],
        "button_cards": [
            ("1", "Nodo [ Core-Router-01 ]",
             ["Selecciona gateway de red VyOS", "Inspecciona salud del router BGP"],
             ["Carga parámetros de IP 172.16.10.1", "y estado activo de mapas eBPF"],
             ["Supervisa punto crítico de entrada", "Identifica cuellos de botella"],
             ["Web: Clic en nodo SVG", "Tablet: Toque 32px • Móvil: Tarjeta en lista"]),

            ("2", "Nodo [ Carrier-Peering ]",
             ["Revisa enlace upstream 10G", "Comprueba tráfico inter-AS"],
             ["Monitorea sesión BGP 198.51.100.1", "Detecta anomalías en peering"],
             ["Protege contra saturación externa", "Valida rutas BGP anunciadas"],
             ["Web: Nodo con enlace resaltado", "Tablet: Toque táctil • Móvil: Ficha nodo"]),

            ("3", "Nodo [ Honeypot Decoy ]",
             ["Supervisa sensor de engaño", "Evalúa ataques de sondeo L4"],
             ["Atrae tráfico de reconocimiento", "Alerta sobre vectores emergentes"],
             ["Aísla atacantes antes del core", "Genera firmas tempranas de bloqueo"],
             ["Web: Nodo ámbar con telemetría", "Tablet: Toque directo • Móvil: Indicador alerta"]),

            ("4", "Nodo [ Sandbox VLAN 999 ]",
             ["Verifica subred 10.200.0.1 aislada", "Supervisa entorno de pruebas"],
             ["Garantiza 0 pps de fuga a clientes", "Conecta con consola Sandbox ACL"],
             ["Permite validar reglas sin riesgo", "Aísla tráfico simulado de ataque"],
             ["Web: Nodo morado con enlace veth", "Tablet: Ficha táctil • Móvil: Tarjeta sandbox"]),

            ("5", "Inspector [ eth0-eth3 ]",
             ["Audita tasas Gbps y conteo pps", "Revisa interfaces eth0 a eth3"],
             ["Recupera contadores de interfaz", "Detecta errores CRC y saturación"],
             ["Brinda visibilidad por interfaz", "Respalda decisiones de balanceo"],
             ["Web: Tabla en panel lateral", "Tablet: Fichas táctiles • Móvil: Bloque eth"]),

            ("6", "Botón [ eBPF / Aislamiento ]",
             ["Aísla nodo o inspecciona mapas", "Aplica contención de emergencia"],
             ["Desconecta interfaces afectadas", "Mantiene telemetría de monitoreo"],
             ["Evita propagación lateral de ataque", "Contiene exfiltración en segundos"],
             ["Web: Botones en panel lateral", "Tablet: Botones táctiles • Móvil: Acciones fijas"])
        ],
        "flow_steps": [
            (1, "Inspección de Nodos", ["Visualización de routers BGP", "y enlaces de transporte 10G"], "#0284C7"),
            (2, "Selección Afectado", ["Selección del router bajo", "anomalía de exfiltración"], "#4F46E5"),
            (3, "Clonación a Sandbox", ["Replicación del contexto de", "red hacia subred aislada"], "#D97706"),
            (4, "Prueba de Regla", ["Paso a consola Sandbox ACL", "para validar mitigación (Pág 10)"], "#16A34A")
        ]
    },

    # ── PÁGINA 5: MÉTRICAS ML LIGHTGBM (6 CONTROLES) ─────────────────────
    {
        "screen_id": "04_monitoring_charts",
        "screen_title": "Métricas ML LightGBM",
        "gen_fn": gen_screen_04_monitoring_charts,
        "web_pins": [(36, 84, 1), (36, 154, 2), (36, 260, 3), (744, 260, 4), (36, 616, 5), (1084, 154, 6)],
        "tablet_pins": [(30, 42, 1), (30, 78, 2), (30, 168, 3), (520, 168, 4), (30, 522, 5), (750, 78, 6)],
        "mobile_pins": [(350, 60, 1), (26, 105, 2), (26, 248, 3), (26, 488, 4), (26, 738, 5), (200, 105, 6)],
        "multiplatform_notes": [
            ("1. Consola Web Desktop (1440 x 900 px)", [
                "Vista analítica de inferencia LightGBM con precisión 99.42% y tiempo de cómputo de 0.42 ms.",
                "Despliegue simultáneo del ranking de características discriminantes y matriz de confusión 4x4."
            ]),
            ("2. Tablet Táctil Profesional (1024 x 768 px)", [
                "Matriz de confusión interactiva con celdas táctiles de alto contraste para auditar falsos positivos.",
                "Barras de importancia con etiquetas legibles y porcentajes en formato numérico destacado."
            ]),
            ("3. Móvil Smartphone PWA (390 x 844 px)", [
                "Métricas sintetizadas en cuadrante 2x2 y tarjetas individuales para ranking de características.",
                "Resumen tabular de tasas de detección para validación ágil del modelo durante guardias técnicas."
            ])
        ],
        "button_cards": [
            ("1", "Barra [ Parámetros Inferencia ]",
             ["Supervisa versión del modelo ML", "Comprueba latencia de 0.42 ms"],
             ["Muestra metadata LightGBM v4.2", "con dataset CIC-DDoS2019 + ISP"],
             ["Valida tiempo de cómputo ultra bajo", "Garantiza inferencia en kernel"],
             ["Web: Barra superior completa", "Tablet: Píldora cabecera • Móvil: Barra compacta"]),

            ("2", "KPI [ Precisión Global 99.42% ]",
             ["Evalúa efectividad algorítmica", "Controla F1-Score ponderado 0.9906"],
             ["Agrega métricas sobre 2.4M flujos", "Comprueba consistencia de detección"],
             ["Certifica robustez del clasificador", "Asegura detección de exfiltración"],
             ["Web: Tarjeta KPI principal", "Tablet: Cuadrante 2x2 • Móvil: Ficha superior"]),

            ("3", "Ranking [ Tree Splits Features ]",
             ["Audita peso de variables de red", "Revisa flow_duration y syn_flags"],
             ["Calcula importancia de características", "por número de divisiones de árbol"],
             ["Aporta explicabilidad ISO 24029", "Justifica qué parámetros deciden"],
             ["Web: Gráfica barras horizontales", "Tablet: Barras táctiles • Móvil: Top 4 features"]),

            ("4", "Matriz [ Confusión Multiclase ]",
             ["Inspecciona aciertos por tipo", "Benigno, SYN, UDP, Exfiltración"],
             ["Cruza clases reales vs predichas", "Demuestra 99.82% benigno intacto"],
             ["Certifica ausencia de sobre-bloqueo", "Minimiza interrupción comercial"],
             ["Web: Matriz 4x4 alineada", "Tablet: Heatmap táctil • Móvil: Resumen porcentual"]),

            ("5", "Histograma [ Score Anomalía ]",
             ["Evalúa corte de umbral en 0.85", "Comprueba distribución bimodal"],
             ["Agrupa flujos según probabilidad", "Separa tráfico benigno de ataque"],
             ["Define punto de mitigación eBPF", "Evita falsas alarmas operativas"],
             ["Web: Histograma panorámico", "Tablet: Gráfico bimodal • Móvil: Umbral inferior"]),

            ("6", "KPI [ Falsos Positivos 0.08% ]",
             ["Controla tasa de error en tráfico", "Verifica umbral menor a 0.1%"],
             ["Mide falsas alarmas en producción", "Garantiza paso de tráfico de clientes"],
             ["Preserva disponibilidad de servicios", "Asegura calidad de servicio ISP"],
             ["Web: Tarjeta KPI destacada", "Tablet: Cuadrante 2x2 • Móvil: Métrica compacta"])
        ],
        "flow_steps": [
            (1, "Ingestión de Flujos", ["Sensores NetFlow ingestan flujos", "de paquetes en tiempo real"], "#0284C7"),
            (2, "Extracción Features", ["Cálculo de intervalos de llegada", "y tamaño de ventanas TCP"], "#4F46E5"),
            (3, "Inferencia LightGBM", ["Clasificación en 0.42 ms", "con modelo de 80 árboles"], "#D97706"),
            (4, "Emisión de Veredicto", ["Puntaje de anomalía enviado a", "la tabla de alertas forenses"], "#16A34A")
        ]
    },

    # ── PÁGINA 6: INSPECCIÓN eBPF EN TIEMPO REAL (6 CONTROLES) ───────────
    {
        "screen_id": "05_realtime_stream",
        "screen_title": "Inspección eBPF en Tiempo Real",
        "gen_fn": gen_screen_05_realtime_stream,
        "web_pins": [(36, 84, 1), (190, 84, 2), (1390, 84, 3), (36, 154, 4), (1390, 220, 5), (36, 594, 6)],
        "tablet_pins": [(30, 78, 1), (350, 78, 2), (980, 78, 3), (30, 144, 4), (980, 210, 5), (30, 584, 6)],
        "mobile_pins": [(26, 104, 1), (160, 104, 2), (355, 104, 3), (26, 162, 4), (355, 206, 5), (26, 656, 6)],
        "multiplatform_notes": [
            ("1. Consola Web Desktop (1440 x 900 px)", [
                "Flujo en vivo de 14.2k paquetes por segundo capturados mediante hooks eBPF a nivel de kernel.",
                "Inspección integrada con tabla de paquetes, veredictos de paso/descarte y visor hexadecimal/ASCII."
            ]),
            ("2. Tablet Táctil Profesional (1024 x 768 px)", [
                "Congelamiento instantáneo del stream al tocar cualquier paquete para análisis detenido.",
                "Vista dividida que mantiene visible la lista de eventos y el desglose de cabeceras de red."
            ]),
            ("3. Móvil Smartphone PWA (390 x 844 px)", [
                "Tarjetas de eventos con datos clave (IP origen, puerto, protocolo y acción eBPF).",
                "Visor hexadecimal deslizable para verificación rápida de cargas sospechosas en movilidad."
            ])
        ],
        "button_cards": [
            ("1", "Barra [ Tasa: 14.2k pps ]",
             ["Supervisa volumen capturado en vivo", "Controla búfer circular de 500 pkts"],
             ["Refleja rendimiento de ingestión", "Previene desbordamiento en memoria"],
             ["Demuestra capacidad nominal 10G", "Garantiza cero pérdida de muestras"],
             ["Web: Indicador verde pulsante", "Tablet: Barra estado • Móvil: Banner superior"]),

            ("2", "Campo [ Filtro BPF Kernel ]",
             ["Escribe filtro sintáctico en vivo", "Ej: ip.proto==TCP and syn==1"],
             ["Compila filtro en bytecode eBPF", "Descarta paquetes no coincidentes"],
             ["Reduce consumo de ancho de banda", "Enfoca análisis en vector de ataque"],
             ["Web: Input con autocompletado", "Tablet: Píldoras filtro • Móvil: Filtro táctil"]),

            ("3", "Botón [ Pausar Flujo ]",
             ["Congela la tabla para inspección", "Búfer de captura sigue en memoria"],
             ["Detiene renderizado de interfaz", "Evita desplazamiento de filas"],
             ["Permite examinar paquete exacto", "Facilita copia de datos forenses"],
             ["Web: Botón azul en barra superior", "Tablet: Botón pausa • Móvil: Botón anclado"]),

            ("4", "Tabla [ Flujo Paquetes ]",
             ["Examina cabeceras IP y TCP", "Timestamp, tamaño y protocolo"],
             ["Muestra eventos en tiempo real", "Deserializa metadatos de kernel"],
             ["Brinda visibilidad granular", "Permite triaje inmediato de flujo"],
             ["Web: Tabla completa 9 columnas", "Tablet: Tabla 8 filas • Móvil: Tarjetas evento"]),

            ("5", "Badge [ Veredicto XDP DROP ]",
             ["Comprueba descarte en placa de red", "Verifica score de anomalía > 0.85"],
             ["Ejecuta hook XDP_DROP en driver", "Filtra antes del stack del kernel"],
             ["Neutraliza ataque sin costo CPU", "Preserva estabilidad del core ISP"],
             ["Web: Badge rojo en fila", "Tablet: Badge táctil • Móvil: Indicador lateral"]),

            ("6", "Visor [ Hex Dump & ASCII ]",
             ["Inspecciona bytes sin procesar", "Cabeceras IP, TCP y payload hex"],
             ["Deserializa secuencia binaria", "Muestra decodificación en texto"],
             ["Evidencia firmas de exfiltración", "Genera prueba pericial inmutable"],
             ["Web: Panel inferior completo", "Tablet: Visor dividido • Móvil: Ficha hex"])
        ],
        "flow_steps": [
            (1, "Ingreso a Tarjeta 10G", ["Paquete ingresa a interfaz", "física eth0 del Core Router"], "#0284C7"),
            (2, "Hook eBPF XDP", ["Inspección en kernel antes de", "consumir memoria en el OS"], "#4F46E5"),
            (3, "Inferencia LightGBM", ["Comparación con modelo: score > 0.85", "clasificado como exfiltración"], "#DC2626"),
            (4, "Acción XDP_DROP", ["Descarte inmediato en placa", "y registro forense a la UI"], "#16A34A")
        ]
    },

    # ── PÁGINA 7: AUDITORÍA FORENSE Y ALERTAS (6 CONTROLES) ──────────────
    {
        "screen_id": "06_alerts",
        "screen_title": "Auditoría Forense y Alertas",
        "gen_fn": gen_screen_06_alerts,
        "web_pins": [(44, 84, 1), (44, 154, 2), (740, 220, 3), (1150, 220, 4), (1390, 84, 5), (1220, 84, 6)],
        "tablet_pins": [(30, 78, 1), (30, 144, 2), (690, 200, 3), (840, 200, 4), (980, 78, 5), (870, 78, 6)],
        "mobile_pins": [(26, 104, 1), (26, 162, 2), (355, 202, 3), (355, 238, 4), (355, 760, 5), (200, 104, 6)],
        "multiplatform_notes": [
            ("1. Consola Web Desktop (1440 x 900 px)", [
                "Consola de gestión forense con bitácora inmutable de incidentes de seguridad clasificados.",
                "Exportación pericial en un clic a formato PDF firmado con hash SHA-256 para custodia."
            ]),
            ("2. Tablet Táctil Profesional (1024 x 768 px)", [
                "Filtros táctiles por severidad (Crítica, Alta, Media, Baja) para triaje prioritario de amenazas.",
                "Modal de detalle forense con registro cronológico completo y descarga de capturas PCAP."
            ]),
            ("3. Móvil Smartphone PWA (390 x 844 px)", [
                "Tarjetas de incidentes priorizadas por nivel de criticidad con datos esenciales de origen y destino.",
                "Acciones directas de asignación y escalamiento de alertas para respuesta inmediata en guardias."
            ])
        ],
        "button_cards": [
            ("1", "Selector [ Filtros Severidad ]",
             ["Filtra alertas por criticidad", "Opciones: Crítica, Alta, Media, Baja"],
             ["Aísla incidentes en tiempo real", "Actualiza vista de tabla forense"],
             ["Prioriza respuesta a amenazas graves", "Optimiza triaje en guardias SOC"],
             ["Web: Píldoras de severidad", "Tablet: Selectores táctiles • Móvil: Barra filtros"]),

            ("2", "Bitácora [ Tabla Incidentes ]",
             ["Audita registro histórico inmutable", "ID, timestamp, vector, analista"],
             ["Consulta base de datos forense", "Indexa eventos con trazabilidad"],
             ["Cumple cadena de custodia pericial", "Garantiza principio no-repudio"],
             ["Web: Tabla tabular 8 columnas", "Tablet: Scroll táctil • Móvil: Lista tarjetas"]),

            ("3", "Badge [ Severidad CRÍTICA ]",
             ["Identifica ataques de alto impacto", "DDoS volumétrico o exfiltración"],
             ["Señaliza prioridad máxima en fila", "Dispara notificación a ingenieros N2"],
             ["Canaliza recursos de mitigación", "Protege estabilidad de enlaces core"],
             ["Web: Badge rojo en fila de tabla", "Tablet: Badge táctil • Móvil: Badge en tarjeta"]),

            ("4", "Acción [ eBPF XDP Aplicado ]",
             ["Verifica contramedida en kernel", "Descarte ejecutado en interfaz"],
             ["Vincula regla aplicada al incidente", "Muestra operador responsable"],
             ["Confirma contención de amenaza", "Verifica efectividad de defensa"],
             ["Web: Badge verde mitigado", "Tablet: Detalle modal • Móvil: Ficha resumen"]),

            ("5", "Botón [ Generar Informe PDF ]",
             ["Descarga informe formal sellado", "Para comités y auditorías externas"],
             ["Compila evidencia y gráficas", "Estampa firma digital SHA-256"],
             ["Sustenta auditorías ISO 27001", "Certifica validez de evidencia"],
             ["Web: Botón primario en header", "Tablet: Botón de barra • Móvil: Botón exportar"]),

            ("6", "Botón [ Exportar CSV ]",
             ["Descarga registros en formato plano", "Para integración con SIEM externo"],
             ["Genera archivo CSV delimitado", "Preserva timestamps UTC exactos"],
             ["Facilita ingesta en Wazuh/Splunk", "Permite correlación prolongada"],
             ["Web: Descarga directa a disco", "Tablet: Guardar archivo • Móvil: Enlace CSV"])
        ],
        "flow_steps": [
            (1, "Registro de Incidente", ["LightGBM genera alerta INC-8841", "con severidad crítica"], "#DC2626"),
            (2, "Asignación Analista", ["Asignado a carlos.mendoza", "para triaje y verificación"], "#D97706"),
            (3, "Mitigación eBPF Drop", ["Aplicación de contramedida", "registrada con firma digital"], "#4F46E5"),
            (4, "Cierre de Alerta", ["Resolución de incidente y", "emisión de reporte ejecutivo"], "#16A34A")
        ]
    },

    # ── PÁGINA 8: CONTROL DE BLAST RADIUS (5 CONTROLES) ──────────────────
    {
        "screen_id": "07_blast_radius",
        "screen_title": "Control de Blast Radius",
        "gen_fn": gen_screen_07_blast_radius,
        "web_pins": [(44, 148, 1), (44, 188, 2), (960, 436, 3), (960, 597, 4), (960, 663, 5)],
        "tablet_pins": [(20, 96, 1), (20, 132, 2), (764, 246, 3), (764, 429, 4), (764, 497, 5)],
        "mobile_pins": [(18, 115, 1), (362, 148, 2), (362, 184, 3), (362, 432, 4), (362, 488, 5)],
        "multiplatform_notes": [
            ("1. Consola Web Desktop (1440 x 900 px)", [
                "Módulo de contención de radio de impacto (Blast Radius) para mitigaciones sobre prefijos /24.",
                "Temporizador regresivo de 300 segundos y re-autenticación FIDO2 contra bloqueos accidentales."
            ]),
            ("2. Tablet Táctil Profesional (1024 x 768 px)", [
                "Diálogo modal con fondo atenuado que bloquea cualquier interacción periférica durante la decisión.",
                "Compatibilidad con sensores biométricos de la tablet para autorizaciones rápidas en crisis."
            ]),
            ("3. Móvil Smartphone PWA (390 x 844 px)", [
                "Presentación vertical enfocada en la decisión crítica con contador de tiempo claramente visible.",
                "Doble botón de confirmación con separación ergonómica para evitar toques accidentales."
            ])
        ],
        "button_cards": [
            ("1", "Banner [ Cuarentena /24 ]",
             ["Supervisa mitigación retenida", "Rango: 198.51.100.0/24 (BGP)"],
             ["Pausa despliegue en routers", "Evalúa impacto en servicios"],
             ["Previene bloqueos accidentales", "Permite revisión antes de aplicar"],
             ["Web: Banner ámbar visible", "Tablet: Alerta superior • Móvil: Banner compacto"]),

            ("2", "Temporizador [ Rollback 300s ]",
             ["Monitorea cuenta regresiva", "5 minutos de ventana de seguridad"],
             ["Ejecuta liberación automática", "si expira el tiempo sin confirmación"],
             ["Garantiza reversibilidad segura", "Evita bloqueos perpetuos olvidados"],
             ["Web: Reloj con minutos y segundos", "Tablet: Contador táctil • Móvil: Timer cabecera"]),

            ("3", "Tarjeta [ Auditoría y Firma ]",
             ["Examina parámetros de la regla", "Nodos, 99.4% malicioso, 0 clientes"],
             ["Muestra hash SHA-256 de solicitud", "Verifica reporte previo de Sandbox"],
             ["Garantiza trazabilidad de decisión", "Asegura fundamento de bloqueo"],
             ["Web: Ficha detallada en modal", "Tablet: Ficha de datos • Móvil: Resumen parámetros"]),

            ("4", "Campo [ Re-Auth FIDO2 ]",
             ["Introduce clave de administrador", "O valida llave física WebAuthn"],
             ["Exige re-autenticación", "para cambios de configuración"],
             ["Previene acciones no autorizadas", "Confirma identidad del operador"],
             ["Web: Input con soporte FIDO2", "Tablet: Sensor huella • Móvil: Biometría táctil"]),

            ("5", "Botones [ Autorizar / Rollback ]",
             ["Ratifica o cancela la mitigación", "Opciones de aplicar o revertir"],
             ["Aplica la regla de filtrado", "o cancela la operación"],
             ["Aplica contramedida en red", "o restaura enrutamiento"],
             ["Web: Botones duales centrados", "Tablet: Botones táctiles • Móvil: Botones apilados"])
        ],
        "flow_steps": [
            (1, "Propuesta Mitigación", ["Regla de mitigación para", "un prefijo /24"], "#DC2626"),
            (2, "Retención Preventiva", ["Acción pausada con", "timer regresivo de 300s"], "#D97706"),
            (3, "Re-Autenticación", ["Operador ingresa clave admin", "o valida llave FIDO2"], "#4F46E5"),
            (4, "Despliegue o Rollback", ["Si ratifica despliega regla;", "si rechaza ejecuta rollback"], "#16A34A")
        ]
    },

    # ── PÁGINA 9: GOBERNANZA RBAC ZERO-TRUST (5 CONTROLES) ───────────────
    {
        "screen_id": "08_user_management",
        "screen_title": "Gobernanza RBAC Zero-Trust",
        "gen_fn": gen_screen_08_user_management,
        "web_pins": [(1390, 84, 1), (36, 154, 2), (700, 84, 3), (1380, 220, 4), (1050, 220, 5)],
        "tablet_pins": [(980, 78, 1), (30, 144, 2), (500, 78, 3), (980, 210, 4), (750, 210, 5)],
        "mobile_pins": [(355, 60, 1), (26, 162, 2), (355, 198, 3), (355, 238, 4), (26, 238, 5)],
        "multiplatform_notes": [
            ("1. Consola Web Desktop (1440 x 900 px)", [
                "Gobernanza de control de acceso basado en roles (RBAC) con sesión inmutable y 100% MFA.",
                "Tabla completa de operadores, privilegios de kernel eBPF y revocación de credenciales."
            ]),
            ("2. Tablet Táctil Profesional (1024 x 768 px)", [
                "Gestión operativa en movilidad para jefes de seguridad y coordinadores de turno de guardia.",
                "Formularios modales simplificados con validaciones en tiempo real para asignación ágil de funciones."
            ]),
            ("3. Móvil Smartphone PWA (390 x 844 px)", [
                "Tarjetas individuales de personal con estado de autenticación de doble factor y último acceso.",
                "Botón de acción rápida para forzar rotación de contraseñas ante sospecha de vulneración."
            ])
        ],
        "button_cards": [
            ("1", "Botón [ + Registrar Operador ]",
             ["Da de alta nuevo personal SOC", "Introduce correo corporativo y rol"],
             ["Genera invitación con enrolamiento", "Asigna flag de cambio forzado"],
             ["Incorpora analista bajo políticas", "Garantiza enrolamiento 2FA inicial"],
             ["Web: Botón primario en header", "Tablet: Acción de barra • Móvil: Botón cabecera"]),

            ("2", "Tabla [ Gestión Usuarios ]",
             ["Audita personal y roles activos", "SOC_ADMIN, ANALYST_N2, OPERATOR"],
             ["Muestra privilegios por operador", "Verifica permisos en eBPF y kernel"],
             ["Cumple principio de mínimo privilegio", "Permite auditar roles asignados"],
             ["Web: Tabla completa 7 columnas", "Tablet: Scroll táctil • Móvil: Lista tarjetas"]),

            ("3", "Badge [ 100% MFA Activo ]",
             ["Supervisa adopción de doble factor", "Verifica 5/5 cuentas protegidas"],
             ["Comprueba métodos TOTP y FIDO2", "Bloquea accesos sin segundo factor"],
             ["Refuerza la seguridad de cuentas", "Cumple directivas de acceso"],
             ["Web: Badge verde en cabecera", "Tablet: Indicador visual • Móvil: Badge en tarjeta"]),

            ("4", "Acción [ Forzar Rotación ]",
             ["Obliga cambio inmediato de clave", "Aplica ante sospecha de fuga"],
             ["Marca flag must_change_password", "Invalida tokens JWT existentes"],
             ["Neutraliza credenciales filtradas", "Fuerza paso por Lámina 01b"],
             ["Web: Botón en fila de tabla", "Tablet: Menú de acción • Móvil: Botón en tarjeta"]),

            ("5", "Bitácora [ Último Acceso ]",
             ["Examina timestamps de actividad", "Verifica IP y cliente utilizado"],
             ["Registra accesos por operador", "Detecta inicios de sesión anómalos"],
             ["Garantiza trazabilidad de acciones", "Respalda auditorías internas"],
             ["Web: Columna con fechas UTC", "Tablet: Detalle en fila • Móvil: Texto en tarjeta"])
        ],
        "flow_steps": [
            (1, "Alta de Operador", ["Registro de correo corporativo", "y asignación de rol RBAC"], "#0284C7"),
            (2, "Enrolamiento 2FA", ["Vinculación obligatoria de", "app Authenticator o llave"], "#4F46E5"),
            (3, "Primer Inicio", ["Detección de clave temporal", "y rotación obligatoria (Pág 2)"], "#D97706"),
            (4, "Acceso Autorizado", ["Consola habilitada bajo", "políticas de seguridad"], "#16A34A")
        ]
    },

    # ── PÁGINA 10: SANDBOX ACL EN SUBRED AISLADA (6 CONTROLES) ───────────
    {
        "screen_id": "09_sandbox_acl",
        "screen_title": "Sandbox ACL en Subred Aislada",
        "gen_fn": gen_screen_09_sandbox_acl,
        "web_pins": [(36, 84, 1), (410, 310, 2), (480, 84, 3), (800, 310, 4), (36, 566, 5), (1390, 760, 6)],
        "tablet_pins": [(30, 78, 1), (450, 160, 2), (30, 144, 3), (450, 280, 4), (30, 370, 5), (980, 520, 6)],
        "mobile_pins": [(26, 104, 1), (355, 340, 2), (26, 172, 3), (355, 276, 4), (26, 466, 5), (355, 620, 6)],
        "multiplatform_notes": [
            ("1. Consola Web Desktop (1440 x 900 px)", [
                "Entorno de prueba en subred aislada 10.200.0.0/24 (VLAN 999) para validación de mitigaciones.",
                "Flujo completo: inyección de tráfico de test, evaluación eBPF y promoción a producción."
            ]),
            ("2. Tablet Táctil Profesional (1024 x 768 px)", [
                "Controles táctiles para configurar vectores de simulación (SYN Flood, UDP y exfiltración).",
                "Consola de código con desplazamiento fluido para revisar sintaxis iptables/eBPF antes de certificar."
            ]),
            ("3. Móvil Smartphone PWA (390 x 844 px)", [
                "Resumen ejecutivo del estado de la regla con tarjetas de eficacia y falsos positivos.",
                "Botón anclado para autorizar la promoción al router carrier tras verificar métricas favorables."
            ])
        ],
        "button_cards": [
            ("1", "Banner [ Subred VLAN 999 ]",
             ["Supervisa entorno 10.200.0.0/24", "Topología veth aislada de ISP"],
             ["Garantiza aislamiento total de red", "Reglas no afectan tráfico de abonados"],
             ["Permite probar tráfico simulado", "Protege la red de producción"],
             ["Web: Banner superior destacado", "Tablet: Alerta superior • Móvil: Banner compacto"]),

            ("2", "Botón [ Inyectar Tráfico ]",
             ["Genera paquetes de prueba", "con tráfico de control"],
             ["Inicia generador en contenedores", "Satura interfaz de prueba veth-sb0"],
             ["Prueba la regla ante flujo de paquetes", "Verifica consumo y uso de CPU"],
             ["Web: Botón azul en generador", "Tablet: Botón táctil • Móvil: Botón simulación"]),

            ("3", "Consola [ Regla eBPF/iptables ]",
             ["Edita comando dinámico de filtro", "Sintaxis iptables raw y eBPF XDP"],
             ["Chequea sintaxis y compila filtro", "Aplica instrucción solo en sandbox"],
             ["Permite depurar reglas antes de uso", "Elimina errores de tipeo en routers"],
             ["Web: Editor con código coloreado", "Tablet: Consola táctil • Móvil: Visor de código"]),

            ("4", "Botón [ Ejecutar en Sandbox ]",
             ["Aplica la regla sobre la simulación", "Inicia medición inmediata de drops"],
             ["Carga bytecode en veth-sandbox", "Calcula métricas de filtrado en vivo"],
             ["Permite validar la regla en acción", "Mide latencia añadida en microsegundos"],
             ["Web: Botón primario de consola", "Tablet: Disparador táctil • Móvil: Botón ejecutar"]),

            ("5", "Certificado [ 99.85% Mitigado ]",
             ["Audita dictamen de efectividad", "Comprueba 0.00% falsos positivos"],
             ["Valida métricas tras la simulación", "Emite sello formal de seguridad"],
             ["Garantiza seguridad previa a despliegue", "Cumple norma de gestión de cambios"],
             ["Web: Panel verde certificado", "Tablet: Tarjeta métricas • Móvil: Ficha resumen"]),

            ("6", "Botón [ Promover a Producción ]",
             ["Transfiere la regla al router core", "Despliega en interfaces de peering"],
             ["Inyecta filtro en routers VyOS", "Estampa firma SHA-256 en auditoría"],
             ["Aplica el filtro validado en el router", "Asegura el servicio sin interrupciones"],
             ["Web: Botón verde destacado", "Tablet: Botón amplio • Móvil: Botón anclado"])
        ],
        "flow_steps": [
            (1, "Edición en Sandbox", ["Definición de regla dinámica", "en subred aislada 10.200.0.0/24"], "#0284C7"),
            (2, "Inyección Tráfico", ["Disparo de 500k pps ataque", "combinado con tráfico legítimo"], "#D97706"),
            (3, "Certificación", ["Telemetría confirma 99.85% drop", "y 0.00% de falsos positivos"], "#4F46E5"),
            (4, "Promoción a Core", ["Despliegue al router carrier", "con firma digital inmutable"], "#16A34A")
        ]
    }
]

# -------------------------------------------------------------------------
# COMPILADOR MAESTRO Y LIMPIEZA DE CARPETA FINAL/
# -------------------------------------------------------------------------
def main():
    print("=" * 80)
    print("GENERADOR MAESTRO DE MOCKUPS MULTIPLATAFORMA (TAMAÑO UNIFORME 2400 x 1500 PX)")
    print("Directivas: Cobertura exhaustiva 5-6 controles por lámina, Pines en esquinas")
    print("sin tapar texto, B&W en descripciones, Multiplataforma 3 fichas, 21 páginas")
    print("=" * 80)

    # 1. Limpieza de carpetas de salida
    shutil.rmtree(DIR_BLANCO, ignore_errors=True)
    shutil.rmtree(DIR_OSCURO, ignore_errors=True)
    os.makedirs(DIR_BLANCO, exist_ok=True)
    os.makedirs(DIR_OSCURO, exist_ok=True)

    compiled_light_pdfs = []
    compiled_dark_pdfs = []

    for theme_key in ["light", "dark"]:
        T = THEMES[theme_key]
        out_sub_dir = DIR_BLANCO if theme_key == "light" else DIR_OSCURO
        theme_name = "Blanco" if theme_key == "light" else "Oscuro"
        print(f"\nGenerando láminas para: Tema {theme_name} en {out_sub_dir}...")

        theme_pdf_list = []

        # ── LÁMINA 00: PORTADA E INTRODUCCIÓN (SOLO EN TEMA CLARO - UNA SOLA VEZ) ──
        if theme_key == "light":
            portada_svg = render_sheet_00_portada(T)
            portada_svg_path = os.path.join(out_sub_dir, "lamina_00_portada.svg")
            portada_png_path = os.path.join(out_sub_dir, "lamina_00_portada.png")
            portada_pdf_path = os.path.join(out_sub_dir, "lamina_00_portada.pdf")

            with open(portada_svg_path, "w", encoding="utf-8") as f:
                f.write(portada_svg)
            subprocess.run(["rsvg-convert", "-f", "png", "-o", portada_png_path, portada_svg_path], check=True)
            subprocess.run(["rsvg-convert", "-f", "pdf", "-o", portada_pdf_path, portada_svg_path], check=True)
            theme_pdf_list.append(portada_pdf_path)
            print(f"  [OK] Lámina 00: Introducción formal sin boxes (única para el manual) -> lamina_00_portada.pdf/.png")

        # ── LÁMINAS 01 A 10: CON PINES CALIBRADOS EN ESQUINAS DE LAS 3 PANTALLAS ───
        for idx, meta in enumerate(SHEETS_METADATA):
            sheet_num = idx + 1
            s_id = meta["screen_id"]
            s_title = meta["screen_title"]
            gen_fn = meta["gen_fn"]

            raw_web_svg = gen_fn(T, "pc")
            raw_tab_svg = gen_fn(T, "tablet")
            raw_mob_svg = gen_fn(T, "celular")

            web_inner = extract_inner_svg(raw_web_svg)
            tab_inner = extract_inner_svg(raw_tab_svg)
            mob_inner = extract_inner_svg(raw_mob_svg)

            # Inyectar pines numerados [1]..[6] en las 3 pantallas
            pins_web_svg = "".join(draw_callout_pin(px, py, pnum, T["accent"], r=13, font_size=12) for px, py, pnum in meta["web_pins"])
            pins_tab_svg = "".join(draw_callout_pin(px, py, pnum, T["accent"], r=16, font_size=14) for px, py, pnum in meta["tablet_pins"])
            pins_mob_svg = "".join(draw_callout_pin(px, py, pnum, T["accent"], r=13, font_size=12) for px, py, pnum in meta["mobile_pins"])

            web_with_pins = web_inner + pins_web_svg
            tab_with_pins = tab_inner + pins_tab_svg
            mob_with_pins = mob_inner + pins_mob_svg

            sheet_svg = render_all_in_one_sheet(
                T=T,
                sheet_num=sheet_num,
                screen_id=s_id,
                screen_title=s_title,
                svg_web_content=web_with_pins,
                svg_tablet_content=tab_with_pins,
                svg_mobile_content=mob_with_pins,
                multiplatform_notes=meta["multiplatform_notes"],
                button_cards_data=meta["button_cards"],
                flow_steps_data=meta["flow_steps"]
            )

            file_prefix = f"lamina_{sheet_num:02d}_{s_id}"
            svg_path = os.path.join(out_sub_dir, f"{file_prefix}.svg")
            png_path = os.path.join(out_sub_dir, f"{file_prefix}.png")
            pdf_path = os.path.join(out_sub_dir, f"{file_prefix}.pdf")

            with open(svg_path, "w", encoding="utf-8") as f:
                f.write(sheet_svg)
            subprocess.run(["rsvg-convert", "-f", "png", "-o", png_path, svg_path], check=True)
            subprocess.run(["rsvg-convert", "-f", "pdf", "-o", pdf_path, svg_path], check=True)

            theme_pdf_list.append(pdf_path)
            num_c = len(meta["button_cards"])
            print(f"  [OK] Página {sheet_num:02d}: {s_id:20} -> {file_prefix}.pdf/.png ({num_c} Controles calibrados)")

        # Compilar PDF por tema
        theme_master_pdf = os.path.join(out_sub_dir, f"COMPILACION_MOCKUPS_TEMA_{theme_name.upper()}.pdf")
        writer = PdfWriter()
        for p in theme_pdf_list:
            reader = PdfReader(p)
            for page in reader.pages:
                writer.add_page(page)
        with open(theme_master_pdf, "wb") as f_out:
            writer.write(f_out)

        if theme_key == "light":
            compiled_light_pdfs = theme_pdf_list
            print(f"  [+] PDF Tema Blanco compilado: {len(theme_pdf_list)} páginas (1 Intro + 10 Láminas)")
        else:
            compiled_dark_pdfs = theme_pdf_list
            print(f"  [+] PDF Tema Oscuro compilado: {len(theme_pdf_list)} páginas (10 Láminas)")

    # Compilar el único PDF final de informe maestro: ENTREGA_FINAL_MOCKUPS.pdf (Exactamente 21 páginas uniformes)
    grand_master_pdf = os.path.join(FINAL_DIR, "ENTREGA_FINAL_MOCKUPS.pdf")
    grand_writer = PdfWriter()
    for p in compiled_light_pdfs + compiled_dark_pdfs:
        reader = PdfReader(p)
        for page in reader.pages:
            grand_writer.add_page(page)
    with open(grand_master_pdf, "wb") as f_out:
        grand_writer.write(f_out)

    print(f"\n>>> Compilación final generada: {grand_master_pdf} (21 páginas uniformes: 1 Intro + 10 Blanco + 10 Oscuro)")

    # 4. Limpieza estricta de la carpeta Final/:
    for item in os.listdir(FINAL_DIR):
        item_path = os.path.join(FINAL_DIR, item)
        if item in ["Blanco", "Oscuro", "ENTREGA_FINAL_MOCKUPS.pdf"]:
            continue
        if os.path.isdir(item_path):
            shutil.rmtree(item_path, ignore_errors=True)
        else:
            os.remove(item_path)

    print("\n" + "=" * 80)
    print("CARPETA FINAL/ DEPURADA RIGUROSAMENTE:")
    for f in sorted(os.listdir(FINAL_DIR)):
        print(f"  - {f}")
    print("=" * 80)

if __name__ == "__main__":
    main()
