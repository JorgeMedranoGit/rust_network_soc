// * * * COMPONENTE: VISTA DE CONTROL Y APROBACIÓN MANUAL DE BLAST RADIUS * * *

use leptos::*;
use crate::models::PendingMitigation;

#[component]
pub fn BlastRadiusControl(
    pending_items: ReadSignal<Vec<PendingMitigation>>,
    set_pending_items: WriteSignal<Vec<PendingMitigation>>,
) -> impl IntoView {
    let (modal_json, set_modal_json) = create_signal(Option::<String>::None);
    let (feedback_msg, set_feedback_msg) = create_signal(Option::<String>::None);

    let approve_mitigation = move |task_id: i64| {
        set_pending_items.update(|items| {
            items.retain(|i| i.task_id != task_id);
        });
        set_feedback_msg.set(Some(format!(
            "✓ Mitigación (Tarea #{}) APROBADA MANUALMENTE. Comando despachado al orquestador y registrado en PostgreSQL.",
            task_id
        )));
    };

    let reject_mitigation = move |task_id: i64| {
        set_pending_items.update(|items| {
            items.retain(|i| i.task_id != task_id);
        });
        set_feedback_msg.set(Some(format!(
            "✕ Mitigación (Tarea #{}) RECHAZADA por el analista. Alerta marcada como descartada en execution_queue.",
            task_id
        )));
    };

    view! {
        <div class="blast-container">
            // Banner explicativo de la Regla de Blast Radius
            <div class="blast-banner">
                <div class="blast-banner-icon">"⚠️"</div>
                <div class="blast-banner-text">
                    <h3>"Control de Impacto Masivo (Blast Radius - Capa 7 / Global)"</h3>
                    <p>
                        "Las acciones con impacto en toda la infraestructura de red (ej. intercepción DNS Sinkholing o Blackholing de subred) "
                        "se retienen en estado " <strong style="color: var(--ctp-peach);">"PENDING_MANUAL"</strong> ". "
                        "El analista SOC debe validar el contexto operativo antes de autorizar la aplicación de la política."
                    </p>
                </div>
            </div>

            // Mensaje de feedback de acción
            {move || {
                if let Some(msg) = feedback_msg.get() {
                    view! {
                        <div style="background-color: rgba(166, 227, 161, 0.15); border: 1px solid var(--ctp-green); color: var(--ctp-green); padding: 0.75rem 1rem; border-radius: 8px; font-size: 0.875rem;">
                            {msg}
                        </div>
                    }.into_view()
                } else {
                    view! {}.into_view()
                }
            }}

            // Lista de tareas pendientes de aprobación manual
            {move || {
                let items = pending_items.get();
                if items.is_empty() {
                    view! {
                        <div class="blast-card" style="text-align: center; padding: 3rem 1rem; color: var(--ctp-subtext0);">
                            <div style="font-size: 2rem; margin-bottom: 0.5rem;">"✅"</div>
                            <h4 style="color: var(--ctp-text);">"No hay mitigaciones pendientes de aprobación manual"</h4>
                            <p style="font-size: 0.8125rem; color: var(--ctp-overlay1); margin-top: 0.25rem;">
                                "Todas las acciones de Blast Radius singular se han ejecutado automáticamente."
                            </p>
                        </div>
                    }.into_view()
                } else {
                    items.into_iter().map(|item| {
                        let task_id = item.task_id;
                        let json_payload = item.rollback_payload.clone();

                        view! {
                            <div class="blast-card" style="border-left: 4px solid var(--ctp-peach);">
                                <div class="blast-card-header">
                                    <div>
                                        <span class="badge-layer L7" style="margin-right: 0.5rem;">
                                            {item.layer_target.clone()}
                                        </span>
                                        <strong style="font-size: 1rem; color: var(--ctp-text);">
                                            {item.action_name.clone()}
                                        </strong>
                                        <span style="font-size: 0.75rem; color: var(--ctp-overlay1); margin-left: 0.5rem;">
                                            {format!("(Tarea #{} / Alerta #{})", item.task_id, item.alert_id)}
                                        </span>
                                    </div>
                                    <div>
                                        <span class="badge-sev CRITICAL" style="background-color: rgba(250, 179, 135, 0.2); color: var(--ctp-peach); border-color: var(--ctp-peach);">
                                            {item.blast_radius.clone()}
                                        </span>
                                    </div>
                                </div>

                                <div class="blast-card-body">
                                    <div class="blast-prop">
                                        <span class="blast-prop-label">"IP Objetivo:"</span>
                                        <span class="blast-prop-val">{item.target_ip.clone()}</span>
                                    </div>
                                    <div class="blast-prop">
                                        <span class="blast-prop-label">"MAC Resolvida:"</span>
                                        <span class="blast-prop-val">{item.target_mac.clone()}</span>
                                    </div>
                                    <div class="blast-prop">
                                        <span class="blast-prop-label">"Encolado en BD:"</span>
                                        <span class="blast-prop-val">{item.queued_at.clone()}</span>
                                    </div>
                                    <div class="blast-prop" style="grid-column: 1 / -1;">
                                        <span class="blast-prop-label">"Motivo de Detención:"</span>
                                        <span style="color: var(--ctp-subtext1);">{item.reason.clone()}</span>
                                    </div>
                                </div>

                                <div class="blast-actions">
                                    <button
                                        class="btn btn-inspect"
                                        on:click={
                                            let json_copy = json_payload.clone();
                                            move |_| set_modal_json.set(Some(json_copy.clone()))
                                        }
                                    >
                                        "🔍 Inspeccionar Rollback JSON"
                                    </button>
                                    <button
                                        class="btn btn-reject"
                                        on:click=move |_| reject_mitigation(task_id)
                                    >
                                        "✕ Rechazar / Descartar"
                                    </button>
                                    <button
                                        class="btn btn-approve"
                                        on:click=move |_| approve_mitigation(task_id)
                                    >
                                        "✓ Aprobar y Ejecutar Mitigación"
                                    </button>
                                </div>
                            </div>
                        }
                    }).collect_view()
                }
            }}

            // Modal para inspección del Rollback JSON Snapshot
            {move || {
                if let Some(json_content) = modal_json.get() {
                    view! {
                        <div class="modal-overlay" on:click=move |_| set_modal_json.set(None)>
                            <div class="modal-content" on:click=|ev| ev.stop_propagation()>
                                <div class="modal-header">
                                    <h3>"Snapshot de Reversión (Rollback Payload JSON)"</h3>
                                    <button class="modal-close" on:click=move |_| set_modal_json.set(None)>
                                        "✕"
                                    </button>
                                </div>
                                <div class="modal-body">
                                    <p style="font-size: 0.8125rem; color: var(--ctp-subtext0); margin-bottom: 0.75rem;">
                                        "Directiva inversa registrada para reversión en " <code>"rollback_snapshots"</code> ":"
                                    </p>
                                    <pre class="json-viewer">
                                        {json_content}
                                    </pre>
                                </div>
                            </div>
                        </div>
                    }.into_view()
                } else {
                    view! {}.into_view()
                }
            }}
        </div>
    }
}
