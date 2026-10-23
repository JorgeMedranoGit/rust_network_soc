// * * * COMPONENTE: FLUJO DINÁMICO DE ALERTAS DE SEGURIDAD EN TIEMPO REAL * * *

use leptos::*;
use crate::models::{RealtimeStreamEvent, Severity};

#[component]
pub fn RealtimeAlerts(
    events: ReadSignal<Vec<RealtimeStreamEvent>>,
    set_events: WriteSignal<Vec<RealtimeStreamEvent>>,
) -> impl IntoView {
    let (is_paused, set_is_paused) = create_signal(false);
    let (filter_layer, set_filter_layer) = create_signal(Option::<String>::None);
    let (filter_sev, set_filter_sev) = create_signal(Option::<Severity>::None);

    let clear_stream = move |_| {
        set_events.set(Vec::new());
    };

    let filtered_stream = create_memo(move |_| {
        let all_events = events.get();
        let layer_f = filter_layer.get();
        let sev_f = filter_sev.get();

        all_events
            .into_iter()
            .filter(|ev| {
                if let Some(ref l) = layer_f {
                    if &ev.layer != l {
                        return false;
                    }
                }
                if let Some(ref s) = sev_f {
                    if &ev.severity != s {
                        return false;
                    }
                }
                true
            })
            .collect::<Vec<RealtimeStreamEvent>>()
    });

    view! {
        <div class="realtime-alerts-section">
            // Barra de control del Streaming de Alertas
            <div class="stream-control-bar">
                <div style="display: flex; align-items: center; gap: 0.75rem;">
                    <div class="status-badge online" style=move || if is_paused.get() { "background-color: rgba(249, 226, 175, 0.15); color: var(--ctp-yellow); border-color: rgba(249, 226, 175, 0.3);" } else { "" }>
                        <span class="status-dot" style=move || if is_paused.get() { "color: var(--ctp-yellow); animation: none;" } else { "color: var(--ctp-green);" }></span>
                        <span>
                            {move || if is_paused.get() { "STREAMING PAUSADO" } else { "STREAMING EN VIVO (ACTIVO)" }}
                        </span>
                    </div>

                    <span style="font-size: 0.8125rem; color: var(--ctp-subtext0); font-family: var(--font-mono);">
                        {move || format!("Total en cola: {} eventos", events.get().len())}
                    </span>
                </div>

                <div style="display: flex; gap: 0.5rem; flex-wrap: wrap;">
                    <button
                        class="btn btn-inspect"
                        style="padding: 0.35rem 0.75rem; font-size: 0.75rem;"
                        on:click=move |_| set_is_paused.update(|p| *p = !*p)
                    >
                        {move || if is_paused.get() { "▶ Reanudar Stream" } else { "⏸ Pausar Stream" }}
                    </button>

                    <button
                        class="btn btn-inspect"
                        style="padding: 0.35rem 0.75rem; font-size: 0.75rem; color: var(--ctp-red);"
                        on:click=clear_stream
                    >
                        "🗑 Limpiar"
                    </button>
                </div>
            </div>

            // Filtros rápidos del Feed
            <div class="filter-bar" style="margin-bottom: 1rem; padding: 0.75rem 1rem;">
                <div class="filter-group">
                    <span class="filter-label">"Filtrar por Capa:"</span>
                    <button
                        class=move || if filter_layer.get().is_none() { "filter-chip active" } else { "filter-chip" }
                        on:click=move |_| set_filter_layer.set(None)
                    >
                        "Todas"
                    </button>
                    <button
                        class=move || if filter_layer.get() == Some("L2".to_string()) { "filter-chip active" } else { "filter-chip" }
                        on:click=move |_| set_filter_layer.set(Some("L2".to_string()))
                    >
                        "L2"
                    </button>
                    <button
                        class=move || if filter_layer.get() == Some("L3".to_string()) { "filter-chip active" } else { "filter-chip" }
                        on:click=move |_| set_filter_layer.set(Some("L3".to_string()))
                    >
                        "L3"
                    </button>
                    <button
                        class=move || if filter_layer.get() == Some("L4".to_string()) { "filter-chip active" } else { "filter-chip" }
                        on:click=move |_| set_filter_layer.set(Some("L4".to_string()))
                    >
                        "L4"
                    </button>
                    <button
                        class=move || if filter_layer.get() == Some("L7".to_string()) { "filter-chip active" } else { "filter-chip" }
                        on:click=move |_| set_filter_layer.set(Some("L7".to_string()))
                    >
                        "L7"
                    </button>
                </div>

                <div class="filter-group">
                    <span class="filter-label">"Severidad:"</span>
                    <button
                        class=move || if filter_sev.get().is_none() { "filter-chip active" } else { "filter-chip" }
                        on:click=move |_| set_filter_sev.set(None)
                    >
                        "Todas"
                    </button>
                    <button
                        class=move || if filter_sev.get() == Some(Severity::CRITICAL) { "filter-chip chip-critical active" } else { "filter-chip" }
                        on:click=move |_| set_filter_sev.set(Some(Severity::CRITICAL))
                    >
                        "Crítica"
                    </button>
                    <button
                        class=move || if filter_sev.get() == Some(Severity::HIGH) { "filter-chip chip-high active" } else { "filter-chip" }
                        on:click=move |_| set_filter_sev.set(Some(Severity::HIGH))
                    >
                        "Alta"
                    </button>
                </div>
            </div>

            // Feed dinámico de eventos en streaming
            <div class="stream-feed-list">
                {move || {
                    let items = filtered_stream.get();
                    if items.is_empty() {
                        view! {
                            <div class="stream-empty-card">
                                "Esperando nuevos eventos forenses en el canal de telemetría..."
                            </div>
                        }.into_view()
                    } else {
                        items.into_iter().map(|ev| {
                            let sev_badge_class = format!("badge-sev {}", ev.severity.as_str());
                            let layer_badge_class = format!("badge-layer {}", ev.layer);
                            let is_massive = ev.blast_radius == "MASSIVE_NETWORK_WIDE";

                            let border_color = match ev.severity {
                                Severity::CRITICAL => "border-left: 4px solid var(--ctp-red);",
                                Severity::HIGH => "border-left: 4px solid var(--ctp-peach);",
                                Severity::MEDIUM => "border-left: 4px solid var(--ctp-yellow);",
                                Severity::LOW => "border-left: 4px solid var(--ctp-green);",
                            };

                            view! {
                                <div class="stream-event-card" style=border_color>
                                    <div class="stream-event-header">
                                        <div style="display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap;">
                                            <span class="stream-timestamp mono">
                                                {ev.timestamp.clone()}
                                            </span>
                                            <span class=layer_badge_class>
                                                {ev.layer.clone()}
                                            </span>
                                            <strong style="color: var(--ctp-text); font-size: 0.9375rem;">
                                                {ev.threat_category.clone()}
                                            </strong>
                                            <span class=sev_badge_class>
                                                {ev.severity.as_str()}
                                            </span>
                                        </div>

                                        <div>
                                            {if is_massive {
                                                view! {
                                                    <span class="badge-blast-massive">
                                                        "⚠️ BLAST RADIUS MASIVO (PENDING_MANUAL)"
                                                    </span>
                                                }.into_view()
                                            } else {
                                                view! {
                                                    <span class="badge-blast-singular">
                                                        "✓ BLAST RADIUS SINGULAR (AUTO)"
                                                    </span>
                                                }.into_view()
                                            }}
                                        </div>
                                    </div>

                                    <div class="stream-event-body">
                                        <div style="color: var(--ctp-subtext1); font-size: 0.84rem; margin-bottom: 0.5rem;">
                                            {ev.message.clone()}
                                        </div>

                                        <div class="stream-flow-details">
                                            <span>
                                                <span class="prop-label">"Flujo: "</span>
                                                <span class="mono" style="color: var(--ctp-text);">{format!("{} -> {} ({})", ev.source_ip, ev.destination_ip, ev.protocol)}</span>
                                            </span>
                                            <span>
                                                <span class="prop-label">"Estrategia Orquestada: "</span>
                                                <span class="mono" style="color: var(--ctp-blue); font-weight: 600;">{ev.mitigation_strategy.clone()}</span>
                                            </span>
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect_view()
                    }
                }}
            </div>
        </div>
    }
}
