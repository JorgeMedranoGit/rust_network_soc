# Contexto del Sistema: Desarrollo de Controlador RADIUS CoA (Capa 2)
El sistema backend en Rust mitiga actualmente en Capa 3 usando iptables y la API de Cisco ISE. Para aislar atacantes a nivel de enlace de datos (Capa 2) y evadir la pérdida de direcciones MAC generada por el ruteo IP, se requiere implementar mitigación vía RADIUS CoA (Change of Authorization).

## Misión de Desarrollo:
1. Crea el archivo `src/mitigation/radius_coa.rs`.
2. Define la estructura `RadiusClient` que administre la IP del equipo de red (NAS-IP-Address) y un Shared Secret.
3. Implementa una función asíncrona que despache un paquete RADIUS `Disconnect-Request` al puerto UDP 3799 del switch. Incorpora un "Mock Inteligente" tolerante a fallos que registre en consola la "Asignación a VLAN de Cuarentena", simulando éxito en la red de pruebas.
4. Refactoriza `src/mitigation/mod.rs` para integrar esta estrategia:
   - Añade `SwitchRadius` al enum `Vendor`.
   - Ajusta el Orquestador Multi-Vendor para priorizar el aislamiento de Capa 2 vía RADIUS cuando se detecte un nodo en la topología local, usándolo antes de recurrir a políticas perimetrales de Capa 3.

## Restricciones Técnicas
Utiliza operaciones asíncronas de red con Tokio (UDP sockets) y gestión estricta de errores con `Result`. El servicio de monitoreo debe continuar operando sin interrupciones si el puerto destino es inalcanzable.
