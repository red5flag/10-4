use leptos::*;

use crate::server_fns::failover::{
    clear_failover_events, get_failover_config, get_failover_events, save_failover_config,
    trigger_failover,
};

#[component]
pub fn FailoverPage() -> impl IntoView {
    let config = create_resource(|| (), |_| async { get_failover_config().await.ok() });
    let events = create_resource(|| (), |_| async { get_failover_events(50).await.ok() });

    view! {
        <div class="page">
            <h1>"WAN Failover"</h1>

            <div class="card">
                <div class="card-header"><span>"Failover Configuration"</span></div>
                <div class="card-body">
                    <Suspense fallback=|| view! { <div class="loading">"Loading…"</div> }>
                        {move || {
                            config.get().map(|cfg| {
                                let cfg = cfg.unwrap_or_default();
                                view! {
                                    <FailoverConfigForm
                                        config={cfg}
                                        on_saved=move || { config.refetch(); }
                                    />
                                }
                            })
                        }}
                    </Suspense>
                </div>
            </div>

            <div class="card">
                <div class="card-header">
                    <span>"Manual Failover"</span>
                </div>
                <div class="card-body">
                    <p class="form-hint">"Force a switch to a specific WAN source:"</p>
                    <div class="failover-trigger-buttons">
                        <button class="btn" on:click=move |_| {
                            let ev = events;
                            spawn_local(async move {
                                let _ = trigger_failover(pi_kiosk_core::WanSource::Ethernet).await;
                                ev.refetch();
                            });
                        }>"Switch to Ethernet"</button>

                        <button class="btn" on:click=move |_| {
                            let ev = events;
                            spawn_local(async move {
                                let _ = trigger_failover(pi_kiosk_core::WanSource::Wifi).await;
                                ev.refetch();
                            });
                        }>"Switch to Wi-Fi"</button>

                        <button class="btn" on:click=move |_| {
                            let ev = events;
                            spawn_local(async move {
                                let _ = trigger_failover(pi_kiosk_core::WanSource::Cellular).await;
                                ev.refetch();
                            });
                        }>"Switch to Cellular"</button>
                    </div>
                </div>
            </div>

            <div class="card">
                <div class="card-header">
                    <span>"Failover Event Log"</span>
                    <div class="card-actions">
                        <button class="btn btn-small" on:click=move |_| events.refetch()>"Refresh"</button>
                        <button class="btn btn-small btn-danger" on:click=move |_| {
                            let ev = events;
                            spawn_local(async move {
                                let _ = clear_failover_events().await;
                                ev.refetch();
                            });
                        }>"Clear"</button>
                    </div>
                </div>
                <div class="card-body">
                    <Suspense fallback=|| view! { <div class="loading">"Loading…"</div> }>
                        {move || {
                            events.get().map(|evs| {
                                let evs = evs.unwrap_or_default();
                                if evs.is_empty() {
                                    view! { <div class="empty-state">"No failover events recorded"</div> }.into_view()
                                } else {
                                    view! {
                                        <div class="failover-log">
                                            {evs.iter().map(|e| view! {
                                                <div class="failover-event-row">
                                                    <span class="fo-time">
                                                        {e.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()}
                                                    </span>
                                                    <span class="fo-from">
                                                        {e.from_source.map(|s| wan_text(s)).unwrap_or("—".into())}
                                                    </span>
                                                    <span class="fo-arrow">"→"</span>
                                                    <span class="fo-to">{wan_text(e.to_source)}</span>
                                                    <span class="fo-reason">{e.reason.clone()}</span>
                                                </div>
                                            }).collect_view()}
                                        </div>
                                    }.into_view()
                                }
                            })
                        }}
                    </Suspense>
                </div>
            </div>
        </div>
    }
}

#[component]
fn FailoverConfigForm(
    config: pi_kiosk_core::FailoverConfig,
    on_saved: impl Fn() + 'static,
) -> impl IntoView {
    let (enabled, set_enabled) = create_signal(config.enabled);
    let (interval, set_interval) = create_signal(config.health_check_interval_s);
    let (target, set_target) = create_signal(config.health_check_target.clone());
    let (check_type, set_check_type) = create_signal(config.health_check_type);
    let (priority, set_priority) = create_signal(config.priority.clone());
    let (info, set_info) = create_signal::<Option<String>>(None);
    let (error, set_error) = create_signal::<Option<String>>(None);

    let on_saved = store_value(on_saved);

    let do_save = create_action(move |_: &()| {
        let cfg = pi_kiosk_core::FailoverConfig {
            enabled: enabled.get(),
            priority: priority.get(),
            health_check_interval_s: interval.get(),
            health_check_target: target.get(),
            health_check_type: check_type.get(),
        };
        let on_saved = on_saved;
        async move {
            match save_failover_config(cfg).await {
                Ok(()) => {
                    set_info.set(Some("Configuration saved".into()));
                    set_error.set(None);
                    on_saved.with_value(|f| f());
                }
                Err(e) => {
                    set_error.set(Some(format!("{e}")));
                    set_info.set(None);
                }
            }
        }
    });

    let move_up = move |idx: usize| {
        if idx > 0 {
            let mut p = priority.get();
            p.swap(idx - 1, idx);
            set_priority.set(p);
        }
    };

    let move_down = move |idx: usize| {
        let mut p = priority.get();
        if idx + 1 < p.len() {
            p.swap(idx, idx + 1);
            set_priority.set(p);
        }
    };

    view! {
        <div class="failover-config-form">
            {move || info.get().map(|i| view! { <div class="info-msg">{i}</div> })}
            {move || error.get().map(|e| view! { <div class="error-msg">{e}</div> })}

            <label class="toggle-row">
                <span>"Enable automatic failover"</span>
                <input type="checkbox" checked={enabled.get()}
                    on:change=move |ev| set_enabled.set(event_target_checked(&ev)) />
            </label>

            <div class="form-row">
                <div class="form-field">
                    <label>"Health Check Type"</label>
                    <select on:change=move |ev| {
                        let val = event_target_value(&ev);
                        set_check_type.set(match val.as_str() {
                            "dns" => pi_kiosk_core::HealthCheckType::Dns,
                            "http" => pi_kiosk_core::HealthCheckType::Http,
                            _ => pi_kiosk_core::HealthCheckType::Ping,
                        });
                    }>
                        <option value="ping" selected={check_type.get() == pi_kiosk_core::HealthCheckType::Ping}>"Ping"</option>
                        <option value="dns" selected={check_type.get() == pi_kiosk_core::HealthCheckType::Dns}>"DNS"</option>
                        <option value="http" selected={check_type.get() == pi_kiosk_core::HealthCheckType::Http}>"HTTP"</option>
                    </select>
                </div>

                <div class="form-field">
                    <label>"Health Check Target"</label>
                    <input type="text" prop:value={target.get()}
                        on:input=move |ev| set_target.set(event_target_value(&ev)) />
                </div>
            </div>

            <div class="form-field">
                <label>"Check Interval (seconds)"</label>
                <input type="number" min="5" max="300" prop:value={interval.get()}
                    on:input=move |ev| {
                        if let Ok(n) = event_target_value(&ev).parse::<u32>() {
                            set_interval.set(n);
                        }
                    } />
            </div>

            <div class="form-field">
                <label>"WAN Priority (top = preferred)"</label>
                <div class="priority-list">
                    {move || {
                        priority.get().iter().enumerate().map(|(idx, &src)| {
                            view! {
                                <div class="priority-row">
                                    <span class="priority-num">{idx + 1}</span>
                                    <span class="priority-name">{wan_text(src)}</span>
                                    <div class="priority-controls">
                                        <button class="btn btn-small" on:click=move |_| move_up(idx)
                                            disabled={idx == 0}
                                        >"↑"</button>
                                        <button class="btn btn-small" on:click=move |_| move_down(idx)
                                            disabled={idx + 1 >= priority.get().len()}
                                        >"↓"</button>
                                    </div>
                                </div>
                            }
                        }).collect_view()
                    }}
                </div>
            </div>

            <button class="btn btn-primary" on:click=move |_| do_save.dispatch(())>"Save Configuration"</button>
        </div>
    }
}

fn wan_text(src: pi_kiosk_core::WanSource) -> String {
    match src {
        pi_kiosk_core::WanSource::Ethernet => "Ethernet".into(),
        pi_kiosk_core::WanSource::Wifi => "Wi-Fi".into(),
        pi_kiosk_core::WanSource::Cellular => "Cellular".into(),
    }
}
