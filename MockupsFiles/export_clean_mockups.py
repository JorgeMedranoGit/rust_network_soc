"""
EXPORTADOR DE MOCKUPS LIMPIOS MULTIPLATAFORMA (TEMA CLARO)
Requisitos del Usuario:
- Sin números (sin pines callout)
- Sin descripción (sin tablas de especificaciones ni notas)
- Solo Modo Claro (Tema Blanco)
- Las 3 vistas (PC Desktop, Tablet y Celular) en la misma foto
- Todas las fotos exactamente del mismo tamaño uniforme (2400 x 1100 px)
- Exportación simultánea en formatos de alta calidad .PNG y .JPG
"""

import os
import sys
import subprocess
from PIL import Image

BASE_DIR = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, BASE_DIR)

from generate_master_duo_mockups import (
    THEMES, esc,
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
from generate_all_in_one_sheets import extract_inner_svg, FONT_SANS, FONT_MONO

# Directorio de exportación dedicado
EXPORT_DIR = os.path.join(BASE_DIR, "Export_Mockups_Claro")
PNG_DIR = os.path.join(EXPORT_DIR, "PNG")
JPG_DIR = os.path.join(EXPORT_DIR, "JPG")

os.makedirs(PNG_DIR, exist_ok=True)
os.makedirs(JPG_DIR, exist_ok=True)

T = THEMES["light"]

# -------------------------------------------------------------------------
# METADATOS DE LAS 10 PANTALLAS
# -------------------------------------------------------------------------
CLEAN_METADATA = [
    {
        "index": 1,
        "screen_id": "01_login",
        "screen_title": "Login",
        "file_name": "mockup_01_login",
        "gen_fn": gen_screen_01_login
    },
    {
        "index": 2,
        "screen_id": "01b_cambio_password",
        "screen_title": "Rotación Forzada de Contraseña",
        "file_name": "mockup_02_rotacion_password",
        "gen_fn": gen_screen_01b_password_change
    },
    {
        "index": 3,
        "screen_id": "02_dashboard",
        "screen_title": "Dashboard Central SOC",
        "file_name": "mockup_03_dashboard_soc",
        "gen_fn": gen_screen_02_dashboard
    },
    {
        "index": 4,
        "screen_id": "03_topology",
        "screen_title": "Topología de Red Core GNS3",
        "file_name": "mockup_04_topologia_red_gns3",
        "gen_fn": gen_screen_03_topology
    },
    {
        "index": 5,
        "screen_id": "04_monitoring_charts",
        "screen_title": "Métricas ML LightGBM",
        "file_name": "mockup_05_metricas_ml_lightgbm",
        "gen_fn": gen_screen_04_monitoring_charts
    },
    {
        "index": 6,
        "screen_id": "05_realtime_stream",
        "screen_title": "Inspección eBPF en Tiempo Real",
        "file_name": "mockup_06_inspeccion_ebpf_tiempo_real",
        "gen_fn": gen_screen_05_realtime_stream
    },
    {
        "index": 7,
        "screen_id": "06_alerts",
        "screen_title": "Auditoría Forense y Alertas",
        "file_name": "mockup_07_auditoria_forense_alertas",
        "gen_fn": gen_screen_06_alerts
    },
    {
        "index": 8,
        "screen_id": "07_blast_radius",
        "screen_title": "Control de Blast Radius",
        "file_name": "mockup_08_control_blast_radius",
        "gen_fn": gen_screen_07_blast_radius
    },
    {
        "index": 9,
        "screen_id": "08_user_management",
        "screen_title": "Gobernanza RBAC Zero-Trust",
        "file_name": "mockup_09_gobernanza_rbac_zero_trust",
        "gen_fn": gen_screen_08_user_management
    },
    {
        "index": 10,
        "screen_id": "09_sandbox_acl",
        "screen_title": "Sandbox ACL en Subred Aislada",
        "file_name": "mockup_10_sandbox_acl_subred_aislada",
        "gen_fn": gen_screen_09_sandbox_acl
    }
]

# -------------------------------------------------------------------------
# RENDERIZADOR MAESTRO DE FOTO LIMPIA (2400 x 1100 PX UNIFORME)
# -------------------------------------------------------------------------
def render_clean_multidevice_sheet(sheet_num, screen_id, screen_title, svg_web, svg_tab, svg_mob):
    W, H = 2400, 1100
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}">']
    out.append(f'<rect width="{W}" height="{H}" fill="{T["bg_page"]}"/>')

    # 1. Cabecera limpia centrada
    out.append(f'<text x="1200" y="52" font-family="{FONT_SANS}" font-size="28" font-weight="900" fill="{T["text_primary"]}" text-anchor="middle">{sheet_num:02d} • {esc(screen_title)}</text>')
    out.append(f'<text x="1200" y="80" font-family="{FONT_SANS}" font-size="14.5" font-weight="600" fill="{T["accent"]}" text-anchor="middle">Plataforma Tigo SOC (ISP AS27839) • Vistas Multiplataforma Integradas (Modo Claro)</text>')
    out.append(f'<line x1="50" y1="96" x2="2350" y2="96" stroke="{T["border_subtle"]}" stroke-width="1"/>')

    # 2. VISTA 1: WEB DESKTOP (1440 x 900 px)
    web_x, web_y, web_w, web_h = 50, 130, 1240, 808
    out.append(f'<text x="{web_x+10}" y="{web_y-10}" font-family="{FONT_SANS}" font-size="12" font-weight="800" fill="{T["accent"]}">1. CONSOLA WEB DESKTOP (1440 x 900 PX)</text>')
    out.append(f'<rect x="{web_x}" y="{web_y}" width="{web_w}" height="{web_h}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
    out.append(f'<rect x="{web_x}" y="{web_y}" width="{web_w}" height="32" rx="6" fill="{T["bg_subtle"]}"/>')
    out.append(f'<rect x="{web_x}" y="{web_y+20}" width="{web_w}" height="12" fill="{T["bg_subtle"]}"/>')
    out.append(f'<circle cx="{web_x+18}" cy="{web_y+16}" r="4" fill="#EF4444"/>')
    out.append(f'<circle cx="{web_x+34}" cy="{web_y+16}" r="4" fill="#F59E0B"/>')
    out.append(f'<circle cx="{web_x+50}" cy="{web_y+16}" r="4" fill="#22C55E"/>')
    out.append(f'<rect x="{web_x+75}" y="{web_y+7}" width="380" height="18" rx="3" fill="{T["bg_panel"]}" stroke="{T["border_subtle"]}" stroke-width="0.8"/>')
    out.append(f'<text x="{web_x+85}" y="{web_y+20}" font-family="{FONT_MONO}" font-size="9.5" fill="{T["text_muted"]}">https://soc.tigo.net.bo/index.html#{esc(screen_id)}</text>')
    
    scale_web = 1240 / 1440
    out.append(f'<g transform="translate({web_x},{web_y+32}) scale({scale_web:.5f})">')
    out.append(svg_web)
    out.append('</g>')

    # 3. VISTA 2: TABLET TÁCTIL (1024 x 768 px) - Centrado vertical respecto al Desktop
    # Desktop ocupa y=130 a 938 (centro y=534). Tablet h=540 -> y = 534 - 270 = 264
    tab_x, tab_y, tab_w, tab_h = 1330, 264, 680, 540
    out.append(f'<text x="{tab_x+10}" y="{tab_y-10}" font-family="{FONT_SANS}" font-size="12" font-weight="800" fill="{T["accent"]}">2. TABLET TÁCTIL (1024 x 768 PX) • TOUCH</text>')
    out.append(f'<rect x="{tab_x}" y="{tab_y}" width="{tab_w}" height="{tab_h}" rx="6" fill="{T["bg_panel"]}" stroke="{T["border"]}" stroke-width="1"/>')
    out.append(f'<rect x="{tab_x}" y="{tab_y}" width="{tab_w}" height="30" rx="6" fill="{T["bg_subtle"]}"/>')
    out.append(f'<rect x="{tab_x}" y="{tab_y+18}" width="{tab_w}" height="12" fill="{T["bg_subtle"]}"/>')
    out.append(f'<text x="{tab_x+16}" y="{tab_y+20}" font-family="{FONT_SANS}" font-size="10.5" font-weight="800" fill="{T["accent"]}">Tigo SOC • Modo Tableta</text>')
    out.append(f'<circle cx="{tab_x+tab_w-16}" cy="{tab_y+15}" r="3.5" fill="{T["success"]}"/>')
    
    scale_tab = 680 / 1024
    out.append(f'<g transform="translate({tab_x},{tab_y+30}) scale({scale_tab:.5f})">')
    out.append(svg_tab)
    out.append('</g>')

    # 4. VISTA 3: MÓVIL SMARTPHONE PWA (390 x 844 px) - Centrado vertical
    # y = 534 - 360 = 174
    mob_x, mob_y, mob_w, mob_h = 2050, 174, 300, 720
    out.append(f'<text x="{mob_x+10}" y="{mob_y-10}" font-family="{FONT_SANS}" font-size="12" font-weight="800" fill="{T["accent"]}">3. MÓVIL (390 x 844 PX)</text>')
    device_bezel = "#1E293B"
    out.append(f'<rect x="{mob_x}" y="{mob_y}" width="{mob_w}" height="{mob_h}" rx="24" fill="{device_bezel}" stroke="{T["border"]}" stroke-width="1.5"/>')
    out.append(f'<rect x="{mob_x+8}" y="{mob_y+8}" width="{mob_w-16}" height="{mob_h-16}" rx="18" fill="{T["bg_page"]}"/>')
    out.append(f'<rect x="{mob_x+mob_w/2-32}" y="{mob_y+12}" width="64" height="8" rx="4" fill="{device_bezel}"/>')
    out.append(f'<circle cx="{mob_x+mob_w/2+18}" cy="{mob_y+16}" r="2" fill="#0B0F17"/>')
    
    scale_mob_x = (mob_w - 16) / 390
    scale_mob_y = (mob_h - 40) / 844
    out.append(f'<g transform="translate({mob_x+8},{mob_y+24}) scale({scale_mob_x:.5f}, {scale_mob_y:.5f})">')
    out.append(svg_mob)
    out.append('</g>')

    out.append('</svg>')
    return "".join(out)

# -------------------------------------------------------------------------
# GENERACIÓN DE LAS 10 LÁMINAS EN .PNG Y .JPG
# -------------------------------------------------------------------------
def main():
    print("=" * 80)
    print("EXPORTADOR DE MOCKUPS LIMPIOS MULTIPLATAFORMA (TEMA CLARO)")
    print("Directivas: Sin números, sin descripción, PC + Tablet + Móvil en la misma foto")
    print("Resolución uniforme: 2400 x 1100 px • Formatos: .PNG y .JPG")
    print("=" * 80)

    for item in CLEAN_METADATA:
        idx = item["index"]
        s_id = item["screen_id"]
        title = item["screen_title"]
        file_name = item["file_name"]
        gen_fn = item["gen_fn"]

        # Generar contenido puro sin pines
        raw_web = extract_inner_svg(gen_fn(T, "pc"))
        raw_tab = extract_inner_svg(gen_fn(T, "tablet"))
        raw_mob = extract_inner_svg(gen_fn(T, "celular"))

        sheet_svg = render_clean_multidevice_sheet(
            sheet_num=idx,
            screen_id=s_id,
            screen_title=title,
            svg_web=raw_web,
            svg_tab=raw_tab,
            svg_mob=raw_mob
        )

        svg_path = os.path.join(EXPORT_DIR, f"{file_name}.svg")
        png_path = os.path.join(PNG_DIR, f"{file_name}.png")
        jpg_path = os.path.join(JPG_DIR, f"{file_name}.jpg")

        with open(svg_path, "w", encoding="utf-8") as f:
            f.write(sheet_svg)

        # Convertir a PNG nativo con rsvg-convert (máxima nitidez vectorial)
        subprocess.run(["rsvg-convert", "-f", "png", "-o", png_path, svg_path], check=True)

        # Convertir a JPG de alta calidad (calidad 95%) con PIL
        img = Image.open(png_path).convert("RGB")
        img.save(jpg_path, "JPEG", quality=95)

        # Limpiar el SVG temporal
        if os.path.exists(svg_path):
            os.remove(svg_path)

        print(f"  [OK] Mockup {idx:02d}: {title:35} -> {file_name}.png & .jpg (2400x1100 px)")

    print("\n" + "=" * 80)
    print(f"EXPORTACIÓN COMPLETADA EN: {EXPORT_DIR}")
    print(f"  • Carpeta PNG: {PNG_DIR} (10 fotos)")
    print(f"  • Carpeta JPG: {JPG_DIR} (10 fotos)")
    print("=" * 80)

if __name__ == "__main__":
    main()
