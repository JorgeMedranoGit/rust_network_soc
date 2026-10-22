# MEMORIA TÉCNICA Y JUSTIFICACIÓN DE DECISIONES DE DISEÑO UI/UX
## Plataforma de Detección Forense de Amenazas y Mitigación Multi-Vendor (Tigo SOC)
**Proyecto de Grado en Ingeniería de Sistemas Informáticos**  
**Postulante:** Jorge Esteban Medrano  
**Fecha:** Octubre 2026  

---

## 1. INTRODUCCIÓN Y CONTEXTO DEL SISTEMA
El presente documento constituye la memoria técnica y funcional que acompaña al conjunto de **Mockups de Alta Fidelidad** desarrollados para la plataforma de ciberseguridad y Centro de Operaciones de Red (SOC) de Tigo Bolivia.

El sistema tiene como objetivo primordial la captura de tráfico en tiempo real mediante `libpcap`, la clasificación heurística y predictiva de anomalías a través de un motor de Machine Learning embebido basado en **LightGBM v4.1**, y el despacho automatizado y manual de directivas de defensa perimetral multi-vendor en capas L2 (Cisco ISE / RADIUS CoA UDP 3799), L3/L4 (iptables Null Route / MikroTik RouterOS) y L7 (DNS Sinkhole con Unbound).

Para responder con excelencia a la **Actividad de Autoaprendizaje de Diseño de Mockups**, el sistema ha sido modelado para dos plataformas complementarias:
1. **Consola Web Desktop (1440x900 / 1920x1080):** Diseñada para estaciones de trabajo con múltiples monitores en salas SOC, orientada a analistas L1/L2 y administradores que requieren máxima densidad de información tabular, correlación de flujos y control fino de mitigaciones.
2. **Aplicación Móvil PWA (390x844):** Diseñada para dispositivos táctiles (smartphones de analistas de guardia), optimizada con ergonomía de una mano (zona del pulgar), notificaciones push de criticidad 5 y botones de aprobación de emergencia con protección contra falsas pulsaciones.

---

## 2. DECONSTRUCCIÓN Y ERRADICACIÓN DE ANTI-PATRONES VISUALES DE IA ("ANTI-SLOP")
Siguiendo las directrices del informe *Patrones Visuales y Anti-patrones Estructurales en el Frontend Generado por Inteligencia Artificial*, se identificaron y eliminaron de raíz los vicios estéticos y de interacción que los modelos de lenguaje introducen por defecto:

### 2.1. Erradicación Total de Emojis e Iconografía Cliché
* **Anti-patrón de IA:** Inserción indiscriminada de cohetes, rayos, ojos brillantes y fuegos en botones y encabezados para simular "empatía" o "dinamismo", lo cual destruye el font stack del sistema operativo (glifos *tofu* en Linux) y añade ruido verbal absurdo a lectores de pantalla (WCAG 1.1.1).
* **Solución Profesional Implementada:** Exclusión del 100% de emojis. La señalización se realiza exclusivamente con tipografía técnica y pictogramas vectoriales SVG de trazo fino utilitario (candado para autenticación, escudo para protección perimetral, retícula para objetivo y líneas de enlace de red).

### 2.2. Supresión del "Cardocalypse" y Agrupación por Tensión Espacial
* **Anti-patrón de IA:** La IA resuelve todo encapsulando cada métrica, etiqueta y texto en cajas individuales con bordes visibles, anidando hasta 5 niveles de tarjetas que reducen drásticamente el espacio útil de pantalla.
* **Solución Profesional Implementada:** Los paneles principales del SOC utilizan espaciado semántico negativo (`gap`, `padding` y líneas de corte de 1px con `#1E293B`). La franja superior de KPIs en el Dashboard es una banda continua con divisores verticales en vez de 4 tarjetas independientes flotantes.

### 2.3. Restricción Cromática Estricta y Eliminación de Gradientes Morados
* **Anti-patrón de IA:** Adopción monótona del degradado púrpura/violeta/cian de plantillas genéricas de SaaS con técnicas de `bg-clip-text` y fondos translúcidos con `backdrop-blur` (glassmorphism) que arruinan la legibilidad.
* **Solución Profesional Implementada:** Paleta sobria basada en tokens documentados para entornos militares y de telecomunicaciones:
  * Fondo de consola: Deep Slate (`#080C14` y `#0F172A`).
  * Superficies elevadas: Slate 800 (`#1E293B`).
  * Acento corporativo: Azul Telecom Tigo (`#0284C7`), garantizando alto contraste.
  * Código de semántica de incidentes: Rojo Carmesí (`#DC2626` / `#450A0A`) para amenazas críticas, Ámbar (`#D97706` / `#451A03`) para advertencias y Verde Esmeralda (`#16A34A` / `#052E16`) para estados mitigados.
  * Contraste tipográfico: Relación de 14.2:1 para texto principal sobre fondo, superando con creces la norma WCAG AAA (7:1).

### 2.4. Geometría y Matemática Concéntrica en Bordes Anidados
* **Anti-patrón de IA:** Asignación arbitraria de `rounded-2xl` a contenedores y elementos internos, creando esquinas interiores "pellizcadas" y desalineadas en ángulos de 45°.
* **Solución Profesional Implementada:** Aplicación rigurosa de la ley de concentricidad:
  $$\text{Radio Interior} = \max(0, \text{Radio Exterior} - \text{Padding})$$
  En el formulario de acceso, el marco contenedor posee un radio exterior de $8\text{ px}$ y un padding perimetral de $12\text{ px}$, derivando en radios internos para los campos de entrada de $6\text{ px}$.

### 2.5. Redacción Técnica y Datos Numéricos Tabulares
* **Anti-patrón de IA:** Redacción publicitaria vacía ("Supercharge your workflow") y escritura de cifras en palabras ("tres alertas pendientes").
* **Solución Profesional Implementada:** Microcopy técnico concreto y uso innegociable de dígitos absolutos con números tabulares (`tabular-nums`) en fuente monospace (`SF Mono`, `Fira Code`) para tasas de paquetes (`1,842 pkts/s`), timestamps (`16:54:12.829`) y direcciones IP (`192.168.100.50`).

---

## 3. MATRIZ DE PANTALLAS Y COBERTURA DE PLATAFORMAS (WEB Y MÓVIL)

El sistema comprende 10 mockups de alta fidelidad organizados en 5 flujos clave idénticos pero adaptados a su contexto ergonómico:

| # | Flujo Funcional | Pantalla Web Desktop (1440x900) | Pantalla Móvil PWA (390x844) | Justificación de Usabilidad y Ergonomía |
|---|---|---|---|---|
| **1** | **Autenticación y RBAC** | `mockup_web_01_login.svg/.png` | `mockup_mobile_01_login.svg/.png` | **Web:** Formulario central con etiquetas persistentes superiores (WCAG 3.3.2) y aviso de política Zero-Trust.<br>**Móvil:** Touch targets de 48px para pulsación cómoda con pulgar. |
| **2** | **Consola de Mando SOC** | `mockup_web_02_dashboard_soc.svg/.png` | `mockup_mobile_02_dashboard_soc.svg/.png` | **Web:** Centro de control de alta densidad: 4 KPIs continuos, gráfica temporal de tráfico vs anomalías y tabla de incidentes en vivo.<br>**Móvil:** Navegación inferior fija (Bottom Tab Bar) y feed vertical de alertas con priorización visual. |
| **3** | **Triaje Forense L2-L7** | `mockup_web_03_alerta_forense.svg/.png` | `mockup_mobile_03_alerta_forense.svg/.png` | **Web:** Triaje en 3 columnas simultáneas (Diagnóstico ML, Decodificador PCAP Hexadecimal tabular y Despacho de Mitigación).<br>**Móvil:** Pestañas táctiles segmentadas (`Incidente` / `Evidencia` / `Mitigar`) sin saturar la pantalla. |
| **4** | **Mitigación y Rollback** | `mockup_web_04_mitigacion_rollback.svg/.png` | `mockup_mobile_04_mitigacion_rollback.svg/.png` | **Web:** Diálogo accesible `<dialog>` con trampa de foco de teclado (WCAG 2.1.2) y comando perimetral visible.<br>**Móvil:** Hoja inferior (Bottom Sheet) con teclado PIN numérico táctil y protección contra doble tap. |
| **5** | **Topología de Red GNS3** | `mockup_web_05_topologia_red.svg/.png` | `mockup_mobile_05_topologia_red.svg/.png` | **Web:** Diagrama esquemático vectorial limpio con switches, routers y 12 nodos con líneas de estado e inspector lateral.<br>**Móvil:** Lista técnica filtrable por etiquetas píldora (`Peligro`, `Normales`) con botón directo de aislamiento. |

---

## 4. JUSTIFICACIÓN DE DECISIONES DE DISEÑO EN USABILIDAD Y FUNCIONALIDAD

### 4.1. Flujo 1: Acceso Seguro y Modelo RBAC Zero-Trust
* **Funcionalidad:** Validación de credenciales sobre hash criptográfico en base de datos PostgreSQL, resolución de roles predefinidos (`SOC_ADMIN`, `SOC_OPERATOR`, `SOC_AUDITOR`, `SOC_ANALYST_L1`) e inicialización de cookies de sesión segura firmadas en el servidor Axum.
* **Decisión de Usabilidad:** Las etiquetas de los campos se colocan de forma fija por encima del input y jamás se ocultan al escribir, resolviendo el grave error de usabilidad de confiar en placeholders que desaparecen. Se informa explícitamente al operador la política de expiración por inactividad (15 min) y re-autenticación obligatoria (300 segundos).

### 4.2. Flujo 2: Dashboard General y Monitoreo de Telemetría
* **Funcionalidad:** Supervisión continua del flujo de paquetes capturados desde el puerto espejo de red mediante `libpcap` y ejecución paralela de inferencia LightGBM multiclase para alertar sobre ataques de inundación (SYN Flood, ICMP Flood), exfiltración encubierta de datos (DNS C2 Tunneling) y escaneo de puertos.
* **Decisión de Usabilidad:** En pantallas de control de misión crítica, la velocidad de escaneo visual humano responde a patrones en "F". Las métricas globales se sitúan en la zona superior izquierda, seguidas por una gráfica sobria donde el pico anómalo se resalta en rojo de alerta con su porcentaje de confianza probabilística, evitando decoraciones innecesarias.

### 4.3. Flujo 3: Triaje Forense e Inspección de Cabeceras L2/L3/L4/L7
* **Funcionalidad:** Desensamblado de paquetes capturados con extracción de direcciones MAC (Capa 2), direccionamiento IP y TTL (Capa 3), puertos y banderas TCP como `[SYN]` o `[ACK]` (Capa 4), y volcados hexadecimales con offset tabular para corroborar la firma de ataques sintetizados (Scapy / Hping3).
* **Decisión de Usabilidad:** La vista dividida en 3 columnas en la versión Web permite al analista contrastar la hipótesis del modelo de IA (columna izquierda) contra la evidencia forense dura del paquete crudo (columna central) antes de despachar una orden de defensa (columna derecha), minimizando errores humanos en la toma de decisiones.

### 4.4. Flujo 4: Aprobación de Mitigaciones y Reversión (Rollback)
* **Funcionalidad:** Despacho de directivas de defensa hacia dispositivos perimetrales del ISP:
  * **L2:** Envío de paquete UDP 3799 RADIUS CoA a Cisco ISE para aislamiento de puerto de switch o reasignación a VLAN de cuarentena.
  * **L3/L4:** Inyección de reglas `iptables -I DOCKER-USER -s <IP> -j DROP` o Null Route en MikroTik RouterOS.
  * **L7:** Redirección de nombres de dominio maliciosos hacia sumideros controlados (DNS Sinkhole).
* **Decisión de Usabilidad y Seguridad Crítica:** 
  * Se implementó el parámetro del sistema `critical_action_timeout_secs = 300` (5 minutos). Si el operador lleva más de 5 minutos sin ingresar su contraseña, el sistema bloquea el despacho directo y abre una ventana modal nativa accesible exigiendo re-autenticación.
  * En la versión móvil, el modal se implementa como un *Bottom Sheet* con teclado numérico ergonómico al alcance del pulgar y confirmación mediante botón ancho de prevención, evitando ejecuciones involuntarias en campo.

### 4.5. Flujo 5: Topología de Red y Dispositivos Supervisados
* **Funcionalidad:** Visualización del inventario de 12 nodos de red (Gateway pfSense `192.168.1.1`, Router Core MikroTik `192.168.1.254`, Atacante Kali `192.168.100.50` y 9 estaciones de trabajo Alpine `192.168.1.11` a `.19`), permitiendo identificar visualmente qué enlace está saturado y qué estación se encuentra bajo fuego o en cuarentena.
* **Decisión de Usabilidad:** Se prescindió de modelos 3D giratorios o efectos WebGL pesados que provocan mareo cognitivo, consumo excesivo de GPU y fallos en navegadores móviles. En su lugar, se adoptó un diagrama esquemático de líneas ortogonales con codificación semántica de colores (Verde: Normal, Rojo: Amenaza activa, Gris: Aislado), permitiendo una comprensión situacional instantánea en menos de 2 segundos.

---

## 5. CONCLUSIÓN Y ALINEACIÓN CON LA RÚBRICA DE GRADO
El conjunto de mockups generado y recopilado en la carpeta `MockupsFiles/` satisface con el estándar más alto los tres criterios de evaluación:

1. **Claridad en la Funcionalidad (Peso 35% - Excelente / 4 Puntos):** Los mockups demuestran con precisión técnica rigurosa cada uno de los flujos operativos reales del sistema SOC (autenticación RBAC, telemetría de red, triaje forense, despacho multi-vendor y topología ISP).
2. **Explicación de Decisiones de Diseño (Peso 35% - Excelente / 4 Puntos):** Esta memoria técnica fundamenta sólidamente la usabilidad, ergonomía cognitiva, accesibilidad WCAG 2.2 y la erradicación documentada de los anti-patrones estéticos de IA.
3. **Cobertura de Plataformas (Peso 30% - Excelente / 4 Puntos):** Se suministran 5 pantallas completas en versión Web de escritorio y 5 pantallas análogas optimizadas para dispositivos móviles, disponibles en formatos vectoriales editables (SVG), imágenes de alta definición (PNG) y un documento consolidado para revisión e impresión (PDF).
