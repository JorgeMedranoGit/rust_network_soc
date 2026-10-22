#!/usr/bin/env python3
"""
Compila la Memoria Técnica de Decisiones de Diseño y Mockups
en un documento PDF vectorial de alta calidad usando rsvg-convert y pypdf.
Estructurado en formato Manual y Guía Académica con estricto apego a las
instrucciones, objetivos, indicaciones y matriz de evaluación.
"""

import os
import subprocess
from pypdf import PdfWriter, PdfReader

OUT_DIR = os.path.dirname(os.path.abspath(__file__))
FINAL_DIR = os.path.join(OUT_DIR, "Final")

FONT_SANS = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif"
FONT_MONO = "'SF Mono', 'Fira Code', 'Roboto Mono', Menlo, Consolas, monospace"

C_BG = "#0B0F17"
C_PANEL = "#131A29"
C_SURFACE = "#1B2436"
C_BORDER = "#27354F"
C_TEXT_PRI = "#F8FAFC"
C_TEXT_SEC = "#94A3B8"
C_TEXT_MUTED = "#64748B"
C_ACCENT = "#38BDF8"
C_CRITICAL = "#EF4444"
C_SUCCESS = "#22C55E"
C_WARNING = "#F59E0B"
C_SANDBOX = "#818CF8"

def create_page(header_title, page_num, total_pages, body_svg):
    return f"""<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="1600" viewBox="0 0 1200 1600">
  <rect width="1200" height="1600" fill="{C_BG}"/>
  <rect x="50" y="50" width="1100" height="1500" rx="8" fill="{C_PANEL}" stroke="{C_BORDER}" stroke-width="1"/>

  <!-- Encabezado de Página -->
  <g transform="translate(80, 100)">
    <text x="0" y="0" fill="{C_ACCENT}" font-family="{FONT_MONO}" font-size="12" font-weight="700" letter-spacing="1">PROYECTO DE GRADO - INGENIERIA DE SISTEMAS INFORMATICOS</text>
    <text x="0" y="24" fill="{C_TEXT_PRI}" font-family="{FONT_SANS}" font-size="18" font-weight="800">{header_title}</text>
    <line x1="0" y1="40" x2="1040" y2="40" stroke="{C_BORDER}" stroke-width="1"/>
  </g>

  <!-- Cuerpo del Documento -->
  <g transform="translate(80, 170)">
    {body_svg}
  </g>

  <!-- Pie de Página -->
  <g transform="translate(80, 1500)">
    <line x1="0" y1="0" x2="1040" y2="0" stroke="{C_BORDER}" stroke-width="1"/>
    <text x="0" y="24" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="10">TIGO SOC DEFENSE PLATFORM - ZERO-TRUST ARCHITECTURE</text>
    <text x="1040" y="24" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="10" text-anchor="end">PAGINA {page_num} DE {total_pages}</text>
  </g>
</svg>"""

pages = []

# PÁGINA 1: MARCO ACADÉMICO, INSTRUCCIONES, OBJETIVOS E INDICACIONES
p1_body = f"""
  <text x="0" y="30" fill="{C_TEXT_PRI}" font-family="{FONT_SANS}" font-size="26" font-weight="800">MANUAL TECNICO Y GUIA DE MOCKUPS MULTIPLATAFORMA</text>
  <text x="0" y="58" fill="{C_ACCENT}" font-family="{FONT_SANS}" font-size="15" font-weight="600">Sistema de Deteccion y Mitigacion de Ataques de Red en Tiempo Real (Tigo SOC)</text>
  <text x="0" y="86" fill="{C_TEXT_SEC}" font-family="{FONT_SANS}" font-size="11.5">Postulante: Jorge Esteban Medrano • Asignacion ISP: AS27839 • Octubre 2026</text>

  <!-- Card 1: Instrucciones y Objetivo del Proyecto -->
  <rect x="0" y="115" width="1040" height="250" rx="6" fill="{C_SURFACE}" stroke="{C_BORDER}" stroke-width="1"/>
  <text x="30" y="145" fill="{C_ACCENT}" font-family="{FONT_SANS}" font-size="15" font-weight="800">1. INSTRUCCIONES Y OBJETIVO DEL PROYECTO DE GRADO</text>
  
  <text x="30" y="175" fill="{C_TEXT_PRI}" font-family="{FONT_SANS}" font-size="11.5" font-weight="700">Marco Academico:</text>
  <text x="160" y="175" fill="{C_TEXT_SEC}" font-family="{FONT_SANS}" font-size="11">Como parte del proyecto de grado en Ingenieria de Sistemas Informaticos, se solicita disenar los mockups</text>
  <text x="160" y="193" fill="{C_TEXT_SEC}" font-family="{FONT_SANS}" font-size="11">que incluya tanto la version web como la version movil (De acuerdo a cada proyecto).</text>

  <text x="30" y="220" fill="{C_TEXT_PRI}" font-family="{FONT_SANS}" font-size="11.5" font-weight="700">Objetivo Principal:</text>
  <text x="160" y="220" fill="{C_TEXT_SEC}" font-family="{FONT_SANS}" font-size="11">El objetivo es que los mockups reflejen claramente la funcionalidad principal del sistema, la experiencia de usuario,</text>
  <text x="160" y="238" fill="{C_TEXT_SEC}" font-family="{FONT_SANS}" font-size="11">y la adaptacion a diferentes dispositivos. Se debe mostrar como interactuan los usuarios con el sistema y como se</text>
  <text x="160" y="256" fill="{C_TEXT_SEC}" font-family="{FONT_SANS}" font-size="11">garantizan los objetivos funcionales del mismo en ambas plataformas.</text>

  <text x="30" y="285" fill="{C_TEXT_PRI}" font-family="{FONT_SANS}" font-size="11.5" font-weight="700">Dominio Operacional:</text>
  <text x="160" y="285" fill="{C_TEXT_SEC}" font-family="{FONT_SANS}" font-size="11">Infraestructura critica de telecomunicaciones (AS27839 Tigo), integrando telemetria eBPF/XDP de alta tasa,</text>
  <text x="160" y="303" fill="{C_TEXT_SEC}" font-family="{FONT_SANS}" font-size="11">clasificacion mediante ML (LightGBM) y mitigacion automatizada en kernel Linux.</text>

  <!-- Card 2: Indicaciones Metodológicas y Decisiones de Diseño -->
  <rect x="0" y="385" width="1040" height="420" rx="6" fill="{C_SURFACE}" stroke="{C_BORDER}" stroke-width="1"/>
  <text x="30" y="415" fill="{C_SUCCESS}" font-family="{FONT_SANS}" font-size="15" font-weight="800">2. INDICACIONES METODOLOGICAS CUMPLIDAS Y DECISIONES DE DISENO</text>

  <text x="30" y="445" fill="{C_TEXT_PRI}" font-family="{FONT_SANS}" font-size="11.5" font-weight="700">• Diseno Multi-Plataforma Unificado:</text>
  <text x="50" y="465" fill="{C_TEXT_SEC}" font-family="{FONT_SANS}" font-size="11">Mockups para interfaz Web (1440x900), Tablet (1024x768) y Movil PWA (390x844) integrados en la misma lamina.</text>
  <text x="50" y="483" fill="{C_TEXT_SEC}" font-family="{FONT_SANS}" font-size="11">Todas las laminas mantienen escala uniforme de 2400x1500 px para lectura armonica.</text>

  <text x="30" y="510" fill="{C_TEXT_PRI}" font-family="{FONT_SANS}" font-size="11.5" font-weight="700">• Pantallas y Flujos Clave:</text>
  <text x="50" y="530" fill="{C_TEXT_SEC}" font-family="{FONT_SANS}" font-size="11">Cobertura de 10 pantallas criticas: autenticacion SSO, rotacion forzada de clave, dashboard de transporte Gbps,</text>
  <text x="50" y="548" fill="{C_TEXT_SEC}" font-family="{FONT_SANS}" font-size="11">topologia carrier, metricas ML, stream eBPF en tiempo real, auditoria de alertas, blast radius y sandbox de reglas.</text>

  <text x="30" y="575" fill="{C_TEXT_PRI}" font-family="{FONT_SANS}" font-size="11.5" font-weight="700">• Coherencia Funcional:</text>
  <text x="50" y="595" fill="{C_TEXT_SEC}" font-family="{FONT_SANS}" font-size="11">La logica de seguridad, contratos de API REST y respuestas del kernel son identicos en todas las plataformas,</text>
  <text x="50" y="613" fill="{C_TEXT_SEC}" font-family="{FONT_SANS}" font-size="11">adaptando la ergonomia tactil con botones de 48px y navegacion vertical en dispositivos moviles.</text>

  <text x="30" y="640" fill="{C_TEXT_PRI}" font-family="{FONT_SANS}" font-size="11.5" font-weight="700">• Erradicacion de Anti-Patrones y Estilo Semi-Plano:</text>
  <text x="50" y="660" fill="{C_TEXT_SEC}" font-family="{FONT_SANS}" font-size="11">Cero emojis en toda la interfaz (iconografia vectorial tecnica pura). Contenedores semi-planos con bordes uniformes</text>
  <text x="50" y="678" fill="{C_TEXT_SEC}" font-family="{FONT_SANS}" font-size="11">de 1px (sin barras gruesas en el borde izquierdo) y fuentes de cifras tabulares para telemetria de red.</text>

  <text x="30" y="705" fill="{C_TEXT_PRI}" font-family="{FONT_SANS}" font-size="11.5" font-weight="700">• Matriz de Evaluacion y Guias de Flujo:</text>
  <text x="50" y="725" fill="{C_TEXT_SEC}" font-family="{FONT_SANS}" font-size="11">Pines numerados [1] a [4] en cada pantalla con llamadas API exactas y diagramas de flujo con flechas vectoriales.</text>

  <!-- Card 3: Dual-Theme Armonizado -->
  <rect x="0" y="825" width="1040" height="345" rx="6" fill="{C_SURFACE}" stroke="{C_BORDER}" stroke-width="1"/>
  <text x="30" y="855" fill="{C_ACCENT}" font-family="{FONT_SANS}" font-size="15" font-weight="800">3. ARQUITECTURA CROMATICA ARMONIZADA (DUAL-THEME WCAG 2.2)</text>

  <rect x="30" y="880" width="470" height="265" rx="4" fill="#F8FAFC" stroke="#CBD5E1" stroke-width="1"/>
  <text x="50" y="910" fill="#0F172A" font-family="{FONT_SANS}" font-size="13" font-weight="800">TEMA CLARO (LIGHT THEME)</text>
  <text x="50" y="935" fill="#475569" font-family="{FONT_SANS}" font-size="11">• Fondo Base: #F8FAFC (Slate 50 de bajo brillo)</text>
  <text x="50" y="955" fill="#475569" font-family="{FONT_SANS}" font-size="11">• Paneles: #FFFFFF con borde semi-plano 1px</text>
  <text x="50" y="975" fill="#475569" font-family="{FONT_SANS}" font-size="11">• Texto Principal: #0F172A (Contraste 13:1 WCAG)</text>
  <text x="50" y="995" fill="#475569" font-family="{FONT_SANS}" font-size="11">• Acento Tigo: #0284C7 (Corporate Sky Blue)</text>
  <text x="50" y="1015" fill="#475569" font-family="{FONT_SANS}" font-size="11">• Semantica: Critico (#DC2626), Exito (#16A34A)</text>
  <text x="50" y="1035" fill="#475569" font-family="{FONT_SANS}" font-size="11">• Sandbox: #4F46E5 (Subred aislada VLAN 999)</text>
  <text x="50" y="1060" fill="#0284C7" font-family="{FONT_MONO}" font-size="10.5" font-weight="700">Optimizado para auditorias y revision diurna</text>

  <rect x="540" y="880" width="470" height="265" rx="4" fill="#0B0F17" stroke="#27354F" stroke-width="1"/>
  <text x="560" y="910" fill="#F8FAFC" font-family="{FONT_SANS}" font-size="13" font-weight="800">TEMA OSCURO (DARK THEME)</text>
  <text x="560" y="935" fill="#94A3B8" font-family="{FONT_SANS}" font-size="11">• Fondo Base: #0B0F17 (Obsidian Navy ergonomico)</text>
  <text x="560" y="955" fill="#94A3B8" font-family="{FONT_SANS}" font-size="11">• Paneles: #131A29 con borde semi-plano 1px</text>
  <text x="560" y="975" fill="{C_TEXT_SEC}" font-family="{FONT_SANS}" font-size="11">• Texto Principal: #F8FAFC (Contraste 14:1 WCAG)</text>
  <text x="560" y="995" fill="{C_TEXT_SEC}" font-family="{FONT_SANS}" font-size="11">• Acento Tigo: #38BDF8 (Sky Blue luminoso)</text>
  <text x="560" y="1015" fill="{C_TEXT_SEC}" font-family="{FONT_SANS}" font-size="11">• Semantica: Critico (#EF4444), Exito (#22C55E)</text>
  <text x="560" y="1035" fill="{C_TEXT_SEC}" font-family="{FONT_SANS}" font-size="11">• Sandbox: #818CF8 (Subred aislada VLAN 999)</text>
  <text x="560" y="1060" fill="#38BDF8" font-family="{FONT_MONO}" font-size="10.5" font-weight="700">Disenado para centros de operaciones SOC 24/7</text>
"""
pages.append(create_page("MARCO ACADEMICO Y MANUAL DE DECISIONES DE DISENO", 1, 2, p1_body))

# PÁGINA 2: ÍNDICE DE LAS 10 PANTALLAS Y DOCUMENTOS MAESTROS CONSOLIDADOS
p2_body = f"""
  <text x="0" y="30" fill="{C_TEXT_PRI}" font-family="{FONT_SANS}" font-size="24" font-weight="800">INDICE DE LAMINAS MULTIPLATAFORMA Y COMPENDIOS MAESTROS</text>
  <text x="0" y="58" fill="{C_TEXT_SEC}" font-family="{FONT_SANS}" font-size="11.5">Cada lamina contiene simultaneamente Web, Tablet y Movil con formato de titulo: 'Pagina X - [Nombre]'</text>

  <!-- Tabla de 10 pantallas -->
  <rect x="0" y="85" width="1040" height="630" rx="6" fill="{C_SURFACE}" stroke="{C_BORDER}" stroke-width="1"/>
  <text x="30" y="115" fill="{C_ACCENT}" font-family="{FONT_SANS}" font-size="14" font-weight="800">Catalogo de las 10 Paginas del Sistema Tigo SOC:</text>

  <g transform="translate(30, 135)">
"""

screens_info = [
    ("Pagina 1", "01_login", "Login de Alta Seguridad", "SSO Email + Password + 2FA TOTP 6 digitos + Rate Limiting 3 intentos + TLS 1.3"),
    ("Pagina 2", "01b_cambio_password", "Rotacion Forzada de Clave", "Primer acceso forzado: medidor de entropia dinamico (94 bits) + 5 requisitos ISO 27001"),
    ("Pagina 3", "02_dashboard", "Dashboard Central del SOC", "Telemetria ISP en vivo, 4 KPIs, curva Gbps vs ataques, tabla de incidentes, header fijo"),
    ("Pagina 4", "03_topology", "Topologia de Red y Nodos ISP", "Mapa GNS3 Core AS27839, enlaces 10G/40G, inspector lateral con estado eBPF/XDP"),
    ("Pagina 5", "04_monitoring_charts", "Metricas ML LightGBM", "Inferencia 0.42 ms, precision 99.42%, ranking de features, matriz de confusion multiclase"),
    ("Pagina 6", "05_realtime_stream", "Flujo eBPF en Vivo", "Captura continua de 14.2k pkts/s, filtros BPF, veredicto ML y deserializacion hex dump"),
    ("Pagina 7", "06_alerts", "Auditoria Forense de Alertas", "Historial de incidentes, severidad critica/alta, exportacion CSV/PDF y cadena de custodia"),
    ("Pagina 8", "07_blast_radius", "Blast Radius y Re-Auth", "Mitigaciones retenidas en cuarentena, timer de rollback 300s, modal de re-auth FIDO2"),
    ("Pagina 9", "08_user_management", "Gobernanza RBAC Zero-Trust", "CRUD de operadores, enforcement 2FA al 100%, politicas de rotacion y matriz de roles"),
    ("Pagina 10", "09_sandbox_acl", "Sandbox ACL en Subred Aislada", "Subred 10.200.0.0/24, inyector de trafico de prueba, consola dinamica eBPF y promocion")
]

for si, (p_num, s_code, s_name, s_desc) in enumerate(screens_info):
    y_pos = si * 46
    p2_body += f"""
    <rect x="0" y="{y_pos}" width="980" height="40" rx="4" fill="{C_PANEL}" stroke="{C_BORDER}" stroke-width="0.8"/>
    <text x="18" y="{y_pos+24}" fill="{C_ACCENT}" font-family="{FONT_MONO}" font-size="10.5" font-weight="700">{p_num}</text>
    <text x="110" y="{y_pos+24}" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="10">{s_code}</text>
    <text x="240" y="{y_pos+24}" fill="{C_TEXT_PRI}" font-family="{FONT_SANS}" font-size="11" font-weight="700">{s_name}</text>
    <text x="440" y="{y_pos+24}" fill="{C_TEXT_SEC}" font-family="{FONT_SANS}" font-size="10">{s_desc}</text>
"""

p2_body += f"""
  </g>

  <!-- Documentos Maestros Compilados -->
  <rect x="0" y="735" width="1040" height="390" rx="6" fill="{C_SURFACE}" stroke="{C_BORDER}" stroke-width="1"/>
  <text x="30" y="765" fill="{C_SUCCESS}" font-family="{FONT_SANS}" font-size="14" font-weight="800">Compendios Maestros Compilados en PDF (Tamano Uniforme 2400x1500 px):</text>
  <text x="30" y="790" fill="{C_TEXT_SEC}" font-family="{FONT_SANS}" font-size="11">Integracion total de factores de forma, matrices tecnicas de botones y diagramas de flujo:</text>

  <g transform="translate(30, 810)">
    <rect x="0" y="0" width="980" height="68" rx="4" fill="{C_PANEL}" stroke="{C_SUCCESS}" stroke-width="1.2"/>
    <text x="20" y="26" fill="{C_SUCCESS}" font-family="{FONT_SANS}" font-size="12" font-weight="800">1. ENTREGA_MAESTRA_MOCKUPS_MULTI_DISPOSITIVO.pdf (22 Paginas Uniformes 2400x1500 px)</text>
    <text x="20" y="44" fill="{C_TEXT_PRI}" font-family="{FONT_SANS}" font-size="10.5">Gran Entrega Completa: Lamina 00 (Manual) + Paginas 1 a 10 en Tema Claro y Tema Oscuro</text>
    <text x="20" y="58" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="9.5">MockupsFiles/Final/ENTREGA_MAESTRA_MOCKUPS_MULTI_DISPOSITIVO.pdf</text>

    <rect x="0" y="80" width="980" height="68" rx="4" fill="{C_PANEL}" stroke="{C_BORDER}" stroke-width="1"/>
    <text x="20" y="106" fill="{C_ACCENT}" font-family="{FONT_SANS}" font-size="12" font-weight="800">2. COMPILACION_MOCKUPS_MULTI_DISPOSITIVO_TEMA_CLARO.pdf (11 Paginas Uniformes)</text>
    <text x="20" y="124" fill="{C_TEXT_PRI}" font-family="{FONT_SANS}" font-size="10.5">Edicion para impresion y revision diurna (Lamina 00 + Paginas 1 a 10)</text>
    <text x="20" y="138" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="9.5">MockupsFiles/Final/COMPILACION_MOCKUPS_MULTI_DISPOSITIVO_TEMA_CLARO.pdf</text>

    <rect x="0" y="160" width="980" height="68" rx="4" fill="{C_PANEL}" stroke="{C_BORDER}" stroke-width="1"/>
    <text x="20" y="186" fill="{C_ACCENT}" font-family="{FONT_SANS}" font-size="12" font-weight="800">3. COMPILACION_MOCKUPS_MULTI_DISPOSITIVO_TEMA_OSCURO.pdf (11 Paginas Uniformes)</text>
    <text x="20" y="204" fill="{C_TEXT_PRI}" font-family="{FONT_SANS}" font-size="10.5">Edicion para salas NOC/SOC de alta ergonomia visual 24/7 (Lamina 00 + Paginas 1 a 10)</text>
    <text x="20" y="218" fill="{C_TEXT_MUTED}" font-family="{FONT_MONO}" font-size="9.5">MockupsFiles/Final/COMPILACION_MOCKUPS_MULTI_DISPOSITIVO_TEMA_OSCURO.pdf</text>

    <rect x="0" y="240" width="980" height="50" rx="4" fill="{C_PANEL}" stroke="{C_BORDER}" stroke-width="0.8"/>
    <text x="20" y="262" fill="{C_TEXT_PRI}" font-family="{FONT_SANS}" font-size="11" font-weight="700">Laminas Individuales PNG y PDF:</text>
    <text x="20" y="278" fill="{C_TEXT_SEC}" font-family="{FONT_MONO}" font-size="9.5">MockupsFiles/Final/sheets/light/lamina_*.png|.pdf  y  MockupsFiles/Final/sheets/dark/lamina_*.png|.pdf</text>
  </g>
"""
pages.append(create_page("INDICE DE LAMINAS Y ENTREGABLES MAESTROS", 2, 2, p2_body))

# Compilar a PDF usando rsvg-convert y pypdf
tmp_pdfs = []
for idx, page_svg in enumerate(pages):
    svg_fn = f"/tmp/memoria_page_{idx+1}.svg"
    pdf_fn = f"/tmp/memoria_page_{idx+1}.pdf"
    with open(svg_fn, "w", encoding="utf-8") as f:
        f.write(page_svg)
    subprocess.run(["rsvg-convert", "-f", "pdf", "-o", pdf_fn, svg_fn], check=True)
    tmp_pdfs.append(pdf_fn)

final_pdf = os.path.join(FINAL_DIR, "DESCRIPCION_DETALLADA_PAGINAS_SOC.pdf")
writer = PdfWriter()
for pdf_file in tmp_pdfs:
    reader = PdfReader(pdf_file)
    for p in reader.pages:
        writer.add_page(p)

with open(final_pdf, "wb") as f_out:
    writer.write(f_out)

print(f"[OK] Memoria tecnica compilada en: {final_pdf}")
