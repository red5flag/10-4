use leptos::*;

use crate::server_fns::notifications::{
    add_alert_rule, clear_alert_log, delete_alert_rule, get_alert_log, get_alert_rules,
    test_alert_rule, toggle_alert_rule,
};

#[component]
pub fn NotificationsPage() -> impl IntoView {
    let rules = create_resource(|| (), |_| async { get_alert_rules().await.ok() });
    let log = create_resource(|| (), |_| async { get_alert_log(50).await.ok() });

    view! {
        <div class="page">
            <h1>"Notifications"</h1>

            <div class="card">
                <div class="card-header">
                    <span>"Alert Rules"</span>
                    <button class="btn btn-small" on:click=move |_| rules.refetch()>"Refresh"</button>
                </div>
                <div class="card-body">
                    <Suspense fallback=|| view! { <div class="loading">"Loading…"</div> }>
                        {move || {
                            rules.get().map(|r| {
                                let r = r.unwrap_or_default();
                                if r.is_empty() {
                                    view! { <div class="empty-state">"No alert rules. Create one below."</div> }.into_view()
                                } else {
                                    view! {
                                        <div class="alert-rules-list">
                                            {r.iter().map(|rule| {
                                                let id = rule.id.clone();
                                                let id_del = id.clone();
                                                let id_test = id.clone();
                                                let id_toggle = id.clone();
                                                let enabled = rule.enabled;
                                                view! {
                                                    <div class="alert-rule-row">
                                                        <div class="alert-rule-info">
                                                            <span class="alert-rule-event">{rule.event_type.clone()}</span>
                                                            <span class="alert-rule-action">{rule.action.to_string()}</span>
                                                            {rule.action_target.as_ref().map(|t| view! {
                                                                <span class="alert-rule-target">{format!("→ {}", t)}</span>
                                                            })}
                                                            <span class="alert-rule-cooldown">{format!("cooldown: {}s", rule.cooldown_s)}</span>
                                                            {if enabled {
                                                                view! { <span class="status-badge status-ok">"Enabled"</span> }.into_view()
                                                            } else {
                                                                view! { <span class="status-badge status-dim">"Disabled"</span> }.into_view()
                                                            }}
                                                        </div>
                                                        <div class="alert-rule-actions">
                                                            <button class="btn btn-small" on:click=move |_| {
                                                                let id = id_toggle.clone();
                                                                let r = rules;
                                                                spawn_local(async move {
                                                                    let _ = toggle_alert_rule(id, !enabled).await;
                                                                    r.refetch();
                                                                });
                                                            }>{if enabled { "Disable" } else { "Enable" }}</button>
                                                            <button class="btn btn-small btn-primary" on:click=move |_| {
                                                                let id = id_test.clone();
                                                                spawn_local(async move {
                                                                    let _ = test_alert_rule(id).await;
                                                                    log.refetch();
                                                                });
                                                            }>"Test"</button>
                                                            <button class="btn btn-small btn-danger" on:click=move |_| {
                                                                let id = id_del.clone();
                                                                let r = rules;
                                                                spawn_local(async move {
                                                                    let _ = delete_alert_rule(id).await;
                                                                    r.refetch();
                                                                });
                                                            }>"Delete"</button>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    }.into_view()
                                }
                            })
                        }}
                    </Suspense>
                </div>
            </div>

            <div class="card">
                <div class="card-header"><span>"Create Alert Rule"</span></div>
                <div class="card-body">
                    <AddRuleForm on_added=move || { rules.refetch(); } />
                </div>
            </div>

            <div class="card">
                <div class="card-header">
                    <span>"Notification Log"</span>
                    <div class="card-actions">
                        <button class="btn btn-small" on:click=move |_| log.refetch()>"Refresh"</button>
                        <button class="btn btn-small btn-danger" on:click=move |_| {
                            let l = log;
                            spawn_local(async move {
                                let _ = clear_alert_log().await;
                                l.refetch();
                            });
                        }>"Clear"</button>
                    </div>
                </div>
                <div class="card-body">
                    <Suspense fallback=|| view! { <div class="loading">"Loading…"</div> }>
                        {move || {
                            log.get().map(|entries| {
                                let entries = entries.unwrap_or_default();
                                if entries.is_empty() {
                                    view! { <div class="empty-state">"No notifications dispatched yet."</div> }.into_view()
                                } else {
                                    view! {
                                        <div class="alert-log-list">
                                            {entries.iter().map(|e| {
                                                let status_class = match e.status.as_str() {
                                                    "sent" => "status-ok",
                                                    "failed" => "status-error",
                                                    _ => "status-dim",
                                                };
                                                view! {
                                                    <div class="alert-log-row">
                                                        <span class="alert-log-time">{e.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()}</span>
                                                        <span class="alert-log-event">{e.event_type.clone()}</span>
                                                        <span class="alert-log-action">{e.action.clone()}</span>
                                                        <span class={format!("status-badge {}", status_class)}>{e.status.clone()}</span>
                                                        {e.message.as_ref().map(|m| view! {
                                                            <span class="alert-log-message">{m.clone()}</span>
                                                        })}
                                                    </div>
                                                }
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
fn AddRuleForm(on_added: impl Fn() + 'static) -> impl IntoView {
    let (event_type, set_event_type) = create_signal(String::new());
    let (action, set_action) = create_signal(pi_kiosk_core::AlertAction::Ui);
    let (action_target, set_action_target) = create_signal(String::new());
    let (cooldown_s, set_cooldown_s) = create_signal(300u32);
    let (info, set_info) = create_signal::<Option<String>>(None);
    let (error, set_error) = create_signal::<Option<String>>(None);

    let on_added = store_value(on_added);

    let do_add = create_action(move |_: &()| {
        let et = event_type.get();
        let a = action.get();
        let at = action_target.get();
        let cd = cooldown_s.get();
        let on_added = on_added;
        async move {
            if et.is_empty() {
                set_error.set(Some("Event type is required".into()));
                set_info.set(None);
                return;
            }
            let target = if at.is_empty() { None } else { Some(at) };
            match add_alert_rule(et, a, target, cd).await {
                Ok(_) => {
                    set_info.set(Some("Rule created".into()));
                    set_error.set(None);
                    set_event_type.set(String::new());
                    set_action_target.set(String::new());
                    on_added.with_value(|f| f());
                }
                Err(e) => {
                    set_error.set(Some(format!("{e}")));
                    set_info.set(None);
                }
            }
        }
    });

    let show_target = move || action.get() != pi_kiosk_core::AlertAction::Ui;

    view! {
        <div class="alert-add-form">
            {move || info.get().map(|i| view! { <div class="info-msg">{i}</div> })}
            {move || error.get().map(|e| view! { <div class="error-msg">{e}</div> })}

            <div class="form-row">
                <div class="form-field">
                    <label>"Event Type"</label>
                    <input type="text" placeholder="detection:motion, detection:person, *"
                        prop:value={event_type.get()}
                        on:input=move |ev| set_event_type.set(event_target_value(&ev)) />
                    <span class="form-hint">"e.g. detection:motion, detection:person, or * for all"</span>
                </div>
                <div class="form-field">
                    <label>"Action"</label>
                    <select on:change=move |ev| {
                        set_action.set(match event_target_value(&ev).as_str() {
                            "sms" => pi_kiosk_core::AlertAction::Sms,
                            "webhook" => pi_kiosk_core::AlertAction::Webhook,
                            _ => pi_kiosk_core::AlertAction::Ui,
                        });
                    }>
                        <option value="ui" selected>"UI (in-app)"</option>
                        <option value="sms">"SMS"</option>
                        <option value="webhook">"Webhook"</option>
                    </select>
                </div>
            </div>

            {move || {
                if show_target() {
                    let label = match action.get() {
                        pi_kiosk_core::AlertAction::Sms => "Phone Number",
                        pi_kiosk_core::AlertAction::Webhook => "Webhook URL",
                        _ => "Target",
                    };
                    let placeholder = match action.get() {
                        pi_kiosk_core::AlertAction::Sms => "+1234567890",
                        pi_kiosk_core::AlertAction::Webhook => "https://example.com/webhook",
                        _ => "",
                    };
                    view! {
                        <div class="form-field">
                            <label>{label}</label>
                            <input type="text" placeholder={placeholder}
                                prop:value={action_target.get()}
                                on:input=move |ev| set_action_target.set(event_target_value(&ev)) />
                        </div>
                    }.into_view()
                } else {
                    view! { <div></div> }.into_view()
                }
            }}

            <div class="form-field">
                <label>"Cooldown (seconds)"</label>
                <input type="number" min="0" max="86400"
                    prop:value={cooldown_s.get()}
                    on:input=move |ev| {
                        if let Ok(n) = event_target_value(&ev).parse::<u32>() {
                            set_cooldown_s.set(n);
                        }
                    } />
                <span class="form-hint">"Minimum seconds between repeated alerts for the same rule"</span>
            </div>

            <button class="btn btn-primary" on:click=move |_| do_add.dispatch(())>"Create Rule"</button>
        </div>
    }
}
