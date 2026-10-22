# Especificación de Requerimientos de Software - Sprint 0
**Proyecto:** Sistema Web Aplicando LightGBM para Detección de Exfiltración de Datos  
**Caso:** Tigo Business Bolivia (AS27839)  
**Autor:** Jorge Esteban Medrano Chacolla

---

## 1. Requerimientos Funcionales (RF)

| ID | DESCRIPCION |
| :---: | :--- |
| **RF1** | Autenticación de usuarios y sesiones seguras en servidor mediante hashing Argon2id, tokens opacos de 64 caracteres, control de expiración y persistencia en tabla user_sessions. |
| **RF2** | Control de acceso basado en roles (RBAC) con permisos granulares (USER_MANAGE, MITIGATE_EXECUTE, ROLLBACK_EXECUTE, READ_ONLY) validados mediante middleware en cada endpoint REST. |
| **RF3** | Rotación obligatoria de credenciales y validación dinámica de entropía criptográfica según directrices ISO/IEC 27001 para contraseñas de analistas del SOC. |
| **RF4** | Ingesta asíncrona continua de tráfico de red a velocidad de cable (AF_PACKET / eBPF) sobre interfaces físicas y virtuales de la red carrier AS27839. |
| **RF5** | Extracción columnar de 23 características de red en microsegundos utilizando el motor Polars con operaciones vectorizadas SIMD en memoria RAM libre de GIL. |
| **RF6** | Inferencia probabilística de exfiltración de datos en tiempo sub-milisegundo (< 1.0 ms) mediante el algoritmo LightGBM optimizado con técnicas GOSS y EFB. |
| **RF7** | Detección multivectorial de canales encubiertos de fuga de datos, incluyendo DNS Tunneling (dnscat2, iodined), exfiltración HTTP/S y tráfico anómalo Slow & Low. |
| **RF8** | Generación de alertas forenses de seguridad indexadas en base de datos PostgreSQL en Tercera Forma Normal (BCNF), selladas criptográficamente con hash SHA-256. |
| **RF9** | Despacho de notificaciones push de ciberseguridad en tiempo real dirigidas a terminales móviles y estaciones del SOC mediante Firebase Cloud Messaging (FCM). |
| **RF10** | Streaming bidireccional de telemetría y flujo continuo de paquetes inspeccionados a 60 FPS mediante canal seguro WebSockets (/api/v1/telemetry/stream). |
| **RF11** | Visualización interactiva de la topología de red carrier AS27839 con grafos de dispositivos descubiertos pasivamente mediante OUI y direccionamiento IP. |
| **RF12** | Dashboard analítico de métricas del modelo de Inteligencia Artificial (curvas ROC, precisión-exhaustividad PR, matrices de confusión y feature importance XAI). |
| **RF13** | Orquestación de mitigación automática en Capa 2 de enlace de datos mediante desconexión de clientes y reasignación de VLAN de cuarentena vía RADIUS CoA / Cisco ISE. |
| **RF14** | Orquestación de mitigación en Capas 3 y 4 de red y transporte mediante inyección de rutas BGP Remotely Triggered Black Hole (RTBH) y listas de control de acceso (ACLs). |
| **RF15** | Orquestación de mitigación en Capa 7 de aplicación mediante DNS Sinkholing para redirigir peticiones a dominios de comando y control hacia un sumidero controlado. |
| **RF16** | Control estricto de radio de impacto (Blast Radius) clasificando incidentes entre Singulares (auto-ejecutables) y Masivos (sujetos a aprobación manual por operador L2). |
| **RF17** | Mecanismo preventivo de reversión automática (Rollback) respaldado por instantáneas JSON con temporizador regresivo de 300 segundos ante falsos positivos. |
| **RF18** | Entorno Sandbox aislado de pruebas (VLAN 999: 10.200.0.0/24) para inyección controlada de tráfico sintético y verificación previa de reglas ACL antes de producción. |
| **RF19** | Registro inmutable de pistas de auditoría del sistema (system_audit_logs) para registrar accesos, cambios de parámetros y ejecución de comandos de mitigación. |
| **RF20** | Interfaz de usuario web reactiva multiplataforma (Desktop 1440x900, Tablet 1024x768 y Móvil 390x844) compilada a WebAssembly (Leptos) con soporte ergonómico para Modo Claro y Modo Oscuro. |

---

## 2. Requerimientos No Funcionales (RNF) - Norma ISO/IEC 25010

| ID | DESCRIPCION |
| :---: | :--- |
| **RNF1** | Eficiencia de Desempeño (Latencia de Inferencia): El tiempo de inferencia del modelo LightGBM en el backend compilado en Rust debe ser estrictamente menor a 1.0 ms por ventana de evaluación (meta alcanzada: ~0.08 ms). |
| **RNF2** | Eficiencia de Desempeño (Latencia de Feature Engineering): El preprocesamiento y extracción columnar de las 23 características en Polars debe ejecutarse en menos de 500 microsegundos en memoria. |
| **RNF3** | Eficiencia de Desempeño (Throughput de Ingesta): El recolector asíncrono debe ser capaz de procesar un throughput continuo de hasta 15,000 paquetes por segundo sin pérdida de paquetes (packet drop < 0.001%). |
| **RNF4** | Precisión y Calidad Predictiva: El clasificador LightGBM debe alcanzar un F1-Score igual o superior a 0.99 (99.0%) y mantener una tasa de falsos positivos (FPR) inferior al 0.5% en la clasificación de exfiltración. |
| **RNF5** | Seguridad (Confidencialidad de Accesos): Las contraseñas deben cifrarse mediante la función de derivación de claves Argon2id (mínimo 64MB RAM, 3 iteraciones) y las sesiones autenticadas deben usar tokens criptográficos aleatorios. |
| **RNF6** | Seguridad (Integridad Forense): Todas las evidencias forenses, instantáneas de estado y cargas útiles de reversión deben almacenarse con un resumen criptográfico SHA-256 para garantizar la no repudiación de eventos. |
| **RNF7** | Seguridad Operativa (Protección de Red): Toda acción de mitigación masiva debe exigir re-autenticación por contraseña si el analista ha permanecido inactivo por más de 300 segundos (5 minutos). |
| **RNF8** | Confiabilidad y Resiliencia: El sistema debe contar con un generador de tráfico sintético calibrado con perfiles de Tigo Business que permita operar y validar el motor analítico incluso si no se dispone de capturas PCAP crudas en disco. |
| **RNF9** | Usabilidad y Ergonomía Visual: La interfaz debe cumplir con las pautas de accesibilidad WCAG 2.1 nivel AA en contraste cromático tanto en Modo Claro como en Modo Oscuro, con un tiempo de renderizado reactivo inferior a 100 ms. |
| **RNF10** | Portabilidad y Arquitectura Ligera: El frontend debe compilar a WebAssembly nativo (WASM) sin requerir frameworks pesados de JavaScript (Node.js en runtime), siendo compatible con los navegadores modernos Chrome, Firefox, Edge y Safari. |
| **RNF11** | Mantenibilidad y Calidad de Código: El código fuente en Rust debe estructurarse bajo Arquitectura Limpia modular (Domain, Infrastructure, Presentation, Mitigation) con cero fugas de memoria garantizadas por el Borrow Checker. |
| **RNF12** | Escalabilidad de Almacenamiento: El esquema PostgreSQL debe estructurarse en Tercera Forma Normal (BCNF), desacoplando las métricas de tráfico agregadas de los registros forenses de anomalías para optimizar el espacio en disco. |
