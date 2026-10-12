// * * * COMPONENTE: TABLA INTERACTIVA DE ALERTAS DE SEGURIDAD * * *

use leptos::*;
use crate::models::{AlertItem, Severity};

#[component]
pub fn AlertsTable(
    alerts: ReadSignal<Vec<AlertItem>>,
    selected_severity: ReadSignal<Option<Severity>>,
    set_selected_severity: WriteSignal<Option<Severity>>,
) -> impl IntoView {
    let (search_query, set_search_query) = create_signal(String::new());
    let (selected_layer, set_selected_layer) = create_signal(Option::<String>::None);
    let (expanded_alert_id, set_expanded_alert_id) = create_signal(Option::<i64>::None);

    // Filtrado reactivo de alertas por severidad, capa y término de búsqueda
    let filtered_alerts = create_memo(move |_| {
        let current_alerts = alerts.get();
        let sev_filter = selected_severity.get();
        let layer_filter = selected_layer.get();
        let query = search_query.get().to_lowercase();

        current_alerts
            .into_iter()
            .filter(|alert| {
                if let Some(ref s) = sev_filter {
                    if &alert.severity != s {
                        return false;
                    }
                }
                if let Some(ref l) = layer_filter {
                    if &alert.layer != l {
                        return false;
                    }
                }
                if !query.is_empty() {
                    let matches_ip = alert.source_ip.contains(&query) || alert.destination_ip.contains(&query);
                    let matches_threat = alert.threat_name.to_lowercase().contains(&query);
                    if !matches_ip && !matches_threat {
                        return false;
                    }
                }
                true
            })
            .collect::<Vec<AlertItem>>()
    });

    view! {
        <div class="alerts-table-section">
            // Barra interactiva de filtros
            <div class="filter-bar">
                // Filtro por Severidad
                <div class="filter-group">
                    <span class="filter-label">"Severidad:"</span>
                    <button
                        class=move || {
                            if selected_severity.get().is_none() { "filter-chip active" } else { "filter-chip" }
                        }
                        on:click=move |_| set_selected_severity.set(None)
                    >
                        "Todas"
                    </button>
                    <button
                        class=move || {
                            if selected_severity.get() == Some(Severity::CRITICAL) { "filter-chip chip-critical active" } else { "filter-chip" }
                        }
                        on:click=move |_| set_selected_severity.set(Some(Severity::CRITICAL))
                    >
                        "Crítica"
                    </button>
                    <button
                        class=move || {
                            if selected_severity.get() == Some(Severity::HIGH) { "filter-chip chip-high active" } else { "filter-chip" }
                        }
                        on:click=move |_| set_selected_severity.set(Some(Severity::HIGH))
                    >
                        "Alta"
                    </button>
                    <button
                        class=move || {
                            if selected_severity.get() == Some(Severity::MEDIUM) { "filter-chip chip-medium active" } else { "filter-chip" }
                        }
                        on:click=move |_| set_selected_severity.set(Some(Severity::MEDIUM))
                    >
                        "Media"
                    </button>
                    <button
                        class=move || {
                            if selected_severity.get() == Some(Severity::LOW) { "filter-chip active" } else { "filter-chip" }
                        }
                        on:click=move |_| set_selected_severity.set(Some(Severity::LOW))
                    >
                        "Baja"
                    </button>
                </div>

                // Filtro por Capa de Red
                <div class="filter-group">
                    <span class="filter-label">"Capa:"</span>
                    <button
                        class=move || if selected_layer.get().is_none() { "filter-chip active" } else { "filter-chip" }
                        on:click=move |_| set_selected_layer.set(None)
                    >
                        "Todas"
                    </button>
                    <button
                        class=move || if selected_layer.get() == Some("L2".to_string()) { "filter-chip active" } else { "filter-chip" }
                        on:click=move |_| set_selected_layer.set(Some("L2".to_string()))
                    >
                        "L2 (Enlace)"
                    </button>
                    <button
                        class=move || if selected_layer.get() == Some("L3".to_string()) { "filter-chip active" } else { "filter-chip" }
                        on:click=move |_| set_selected_layer.set(Some("L3".to_string()))
                    >
                        "L3 (Red)"
                    </button>
                    <button
                        class=move || if selected_layer.get() == Some("L4".to_string()) { "filter-chip active" } else { "filter-chip" }
                        on:click=move |_| set_selected_layer.set(Some("L4".to_string()))
                    >
                        "L4 (Transp.)"
                    </button>
                    <button
                        class=move || if selected_layer.get() == Some("L7".to_string()) { "filter-chip active" } else { "filter-chip" }
                        on:click=move |_| set_selected_layer.set(Some("L7".to_string()))
                    >
                        "L7 (Aplicación)"
                    </button>
                </div>

                // Buscador en tiempo real
                <div class="filter-group">
                    <input
                        type="text"
                        class="search-input"
                        placeholder="Buscar IP o categoría..."
                        prop:value=search_query
                        on:input=move |ev| set_search_query.set(event_target_value(&ev))
                    />
                </div>
            </div>

            // Contenedor de la Tabla
            <div class="table-container">
                <table class="soc-table">
                    <thead>
                        <tr>
                            <th>"ID"</th>
                            <th>"Fecha / Hora"</th>
                            <th>"Capa"</th>
                            <th>"Amenaza Detectada"</th>
                            <th>"IP Origen"</th>
                            <th>"IP Destino"</th>
                            <th>"Protocolo"</th>
                            <th>"Severidad"</th>
                            <th>"Score ML"</th>
                            <th>"Estado"</th>
                            <th>"Acciones"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            let items = filtered_alerts.get();
                            if items.is_empty() {
                                view! {
                                    <tr>
                                        <td colspan="11" style="text-align: center; padding: 2rem; color: var(--ctp-overlay0);">
                                            "No se encontraron alertas con los filtros seleccionados."
                                        </td>
                                    </tr>
                                }.into_view()
                            } else {
                                items.into_iter().map(|item| {
                                    let alert_id = item.alert_id;
                                    let layer_class = format!("badge-layer {}", item.layer);
                                    let sev_class = format!("badge-sev {}", item.severity.as_str());
                                    let is_expanded = move || expanded_alert_id.get() == Some(alert_id);
                                    let status_color = if item.status == "PENDING_MANUAL" {
                                        "color: var(--ctp-peach); font-weight: 700;"
                                    } else if item.status == "MITIGATED" {
                                        "color: var(--ctp-green); font-weight: 600;"
                                    } else {
                                        "color: var(--ctp-subtext0);"
                                    };

                                    let feature_json = item.feature_details.clone();

                                    view! {
                                        <tr>
                                            <td class="mono">{format!("#{}", item.alert_id)}</td>
                                            <td class="mono">{item.detected_at}</td>
                                            <td><span class=layer_class>{item.layer}</span></td>
                                            <td>
                                                <strong style="color: var(--ctp-text);">{item.threat_name}</strong>
                                                <div style="font-size: 0.75rem; color: var(--ctp-overlay1);">
                                                    {item.mitigation_action}
                                                </div>
                                            </td>
                                            <td class="mono">{item.source_ip}</td>
                                            <td class="mono">{item.destination_ip}</td>
                                            <td class="mono">{item.protocol}</td>
                                            <td><span class=sev_class>{item.severity.as_str()}</span></td>
                                            <td class="mono" style="color: var(--ctp-mauve);">
                                                {format!("{:.3}", item.anomaly_score)}
                                            </td>
                                            <td>
                                                <span style=status_color>{item.status}</span>
                                            </td>
                                            <td>
                                                <button
                                                    class="btn btn-inspect"
                                                    style="padding: 0.25rem 0.5rem; font-size: 0.75rem;"
                                                    on:click=move |_| {
                                                        if is_expanded() {
                                                            set_expanded_alert_id.set(None);
                                                        } else {
                                                            set_expanded_alert_id.set(Some(alert_id));
                                                        }
                                                    }
                                                >
                                                    {move || if is_expanded() { "Ocultar" } else { "Vector IA" }}
                                                </button>
                                            </td>
                                        </tr>
                                        {move || {
                                            if is_expanded() {
                                                view! {
                                                    <tr>
                                                        <td colspan="11" style="background-color: var(--ctp-crust); padding: 1rem;">
                                                            <div style="font-size: 0.75rem; color: var(--ctp-subtext0); margin-bottom: 0.35rem;">
                                                                "Vector de Características Forenses (Polars / LightGBM 23-features):"
                                                            </div>
                                                            <pre class="json-viewer" style="max-height: 120px;">
                                                                {feature_json.clone()}
                                                            </pre>
                                                        </td>
                                                    </tr>
                                                }.into_view()
                                            } else {
                                                view! {}.into_view()
                                            }
                                        }}
                                    }
                                }).collect_view()
                            }
                        }}
                    </tbody>
                </table>
            </div>
        </div>
    }
}
