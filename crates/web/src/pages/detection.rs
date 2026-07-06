use leptos::*;

use crate::server_fns::detection::{
    clear_all_events, delete_event, get_detection_state, get_recent_events,
    update_detection_config, DetectionState,
};

#[component]
pub fn DetectionPage() -> impl IntoView {
    let (error, set_error) = create_signal(None::<String>);
    let (info, set_info) = create_signal(None::<String>);

    let events = create_resource(|| (), |_| async { get_recent_events(100).await.ok() });
    let detection_state = create_resource(|| (), |_| async { get_detection_state().await.ok() });

    let do_clear = create_action(move |_: &()| async move {
        set_error.set(None);
        set_info.set(None);
        match clear_all_events().await {
            Ok(()) => {
                set_info.set(Some("All events cleared".into()));
                events.refetch();
            }
            Err(e) => set_error.set(Some(format!("{e}"))),
        }
    });

    let do_delete = create_action(move |event_id: &String| {
        let event_id = event_id.clone();
        async move {
            set_error.set(None);
            match delete_event(event_id).await {
                Ok(()) => events.refetch(),
                Err(e) => set_error.set(Some(format!("{e}"))),
            }
        }
    });

    view! {
        <div class="page">
            <h1>"Detection"</h1>

            {move || error.get().map(|e| view! { <div class="error">{e}</div> })}
            {move || info.get().map(|i| view! { <div class="info-msg">{i}</div> })}

            <div class="detection-config-section">
                <div class="card">
                    <div class="card-header"><span>"Detection Settings"</span></div>
                    <div class="card-body">
                        <Suspense fallback=|| view! { <div class="loading">"Loading…"</div> }>
                            {move || {
                                detection_state.get().map(|state| match state {
                                    Some(ds) => view! {
                                        <DetectionConfigForm state={ds}/>
                                    }.into_view(),
                                    None => view! { <div class="empty-state">"Failed to load settings"</div> }.into_view(),
                                })
                            }}
                        </Suspense>
                    </div>
                </div>
            </div>

            <div class="card">
                <div class="card-header">
                    <span>"Event Timeline"</span>
                    <div class="card-header-actions">
                        <button class="btn btn-small" on:click=move |_| events.refetch()>"Refresh"</button>
                        <button class="btn btn-small btn-delete" on:click=move |_| do_clear.dispatch(())>"Clear All"</button>
                    </div>
                </div>
                <div class="card-body">
                    <Suspense fallback=|| view! { <div class="loading">"Loading…"</div> }>
                        {move || {
                            events.get().map(|evs| match evs {
                                Some(list) if !list.is_empty() => view! {
                                    <div class="event-timeline">
                                        {list.iter().map(|e| {
                                            let eid = e.id.clone();
                                            view! {
                                                <div class="event-row">
                                                    <div class="event-icon" class:event-motion={e.kind == pi_kiosk_core::DetectionKind::Motion}
                                                         class:event-person={e.kind == pi_kiosk_core::DetectionKind::Person}
                                                         class:event-tamper={e.kind == pi_kiosk_core::DetectionKind::Tamper}>
                                                        {event_icon_text(e.kind)}
                                                    </div>
                                                    <div class="event-info">
                                                        <span class="event-type">{event_type_text(e.kind)}</span>
                                                        {e.confidence.map(|c| view! {
                                                            <span class="event-confidence">{format!("confidence: {:.0}%", c * 100.0)}</span>
                                                        })}
                                                        <span class="event-time">{e.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()}</span>
                                                    </div>
                                                    <button class="btn btn-small btn-delete"
                                                        on:click={
                                                            move |_| {
                                                                do_delete.dispatch(eid.clone());
                                                            }
                                                        }
                                                    >"Delete"</button>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_view(),
                                _ => view! { <div class="empty-state">"No events recorded"</div> }.into_view(),
                            })
                        }}
                    </Suspense>
                </div>
            </div>
        </div>
    }
}

#[component]
fn DetectionConfigForm(state: DetectionState) -> impl IntoView {
    let (motion_enabled, set_motion_enabled) = create_signal(state.motion_enabled);
    let (person_enabled, _set_person_enabled) = create_signal(state.person_enabled);
    let (tamper_enabled, set_tamper_enabled) = create_signal(state.tamper_enabled);
    let (motion_threshold, set_motion_threshold) = create_signal(state.motion_threshold);
    let (_person_confidence, _set_person_confidence) = create_signal(state.person_confidence);
    let (cooldown, set_cooldown) = create_signal(state.cooldown_seconds);
    let (interval, set_interval) = create_signal(state.capture_interval_ms);

    let (save_info, set_save_info) = create_signal(None::<String>);
    let (save_error, set_save_error) = create_signal(None::<String>);

    let do_save = create_action(move |_: &()| {
        let me = motion_enabled.get();
        let pe = person_enabled.get();
        let te = tamper_enabled.get();
        let mt = motion_threshold.get();
        let pc = _person_confidence.get();
        let cd = cooldown.get();
        let iv = interval.get();
        async move {
            set_save_info.set(None);
            set_save_error.set(None);
            match update_detection_config(
                Some(me), Some(pe), Some(te), Some(mt), Some(pc), Some(cd), Some(iv),
            ).await {
                Ok(()) => set_save_info.set(Some("Settings saved".into())),
                Err(e) => set_save_error.set(Some(format!("{e}"))),
            }
        }
    });

    view! {
        <div class="detection-config-form">
            {move || save_error.get().map(|e| view! { <div class="error">{e}</div> })}
            {move || save_info.get().map(|i| view! { <div class="info-msg">{i}</div> })}

            <label class="toggle-row">
                <span>"Motion Detection"</span>
                <input type="checkbox" checked=move || motion_enabled.get()
                    on:change=move |ev| set_motion_enabled.set(event_target_checked(&ev))
                />
            </label>

            <label class="toggle-row">
                <span>"Person Detection"</span>
                <input type="checkbox" checked=move || person_enabled.get()
                    on:change=move |ev| _set_person_enabled.set(event_target_checked(&ev))
                />
            </label>

            <label class="toggle-row">
                <span>"Tamper Detection"</span>
                <input type="checkbox" checked=move || tamper_enabled.get()
                    on:change=move |ev| set_tamper_enabled.set(event_target_checked(&ev))
                />
            </label>

            <div class="form-field">
                <label>"Motion Threshold: " {move || format!("{:.2}", motion_threshold.get())}</label>
                <input type="range" min="0.01" max="0.20" step="0.01"
                    prop:value=move || motion_threshold.get()
                    on:input=move |ev| {
                        let v = event_target_value(&ev).parse::<f32>().unwrap_or(0.05);
                        set_motion_threshold.set(v);
                    }
                />
            </div>

            <div class="form-field">
                <label>"Cooldown (seconds)"</label>
                <input type="number" min="5" max="300"
                    prop:value=move || cooldown.get()
                    on:input=move |ev| {
                        let v = event_target_value(&ev).parse::<u32>().unwrap_or(30);
                        set_cooldown.set(v);
                    }
                />
            </div>

            <div class="form-field">
                <label>"Capture Interval (ms)"</label>
                <input type="number" min="100" max="10000"
                    prop:value=move || interval.get()
                    on:input=move |ev| {
                        let v = event_target_value(&ev).parse::<u32>().unwrap_or(1000);
                        set_interval.set(v);
                    }
                />
            </div>

            <button class="btn btn-snapshot" on:click=move |_| do_save.dispatch(())>"Save Settings"</button>
        </div>
    }
}

fn event_icon_text(kind: pi_kiosk_core::DetectionKind) -> &'static str {
    match kind {
        pi_kiosk_core::DetectionKind::Motion => "M",
        pi_kiosk_core::DetectionKind::Person => "P",
        pi_kiosk_core::DetectionKind::Tamper => "T",
    }
}

fn event_type_text(kind: pi_kiosk_core::DetectionKind) -> &'static str {
    match kind {
        pi_kiosk_core::DetectionKind::Motion => "Motion Detected",
        pi_kiosk_core::DetectionKind::Person => "Person Detected",
        pi_kiosk_core::DetectionKind::Tamper => "Tamper Alert",
    }
}
