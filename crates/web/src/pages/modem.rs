use leptos::*;

use crate::server_fns::modem::{
    get_modem_diagnostics, get_modem_status, get_sms_inbox, send_sms, ModemDiagnostics,
};

#[component]
pub fn ModemPage() -> impl IntoView {
    let status = create_resource(|| (), |_| async { get_modem_status().await.ok() });
    let diagnostics =
        create_resource(|| (), |_| async { get_modem_diagnostics().await.ok() });
    let sms_inbox = create_resource(|| (), |_| async { get_sms_inbox().await.ok() });

    view! {
        <div class="page">
            <h1>"Cellular Modem"</h1>

            <div class="card">
                <div class="card-header">
                    <span>"Modem Status"</span>
                    <button class="btn btn-small" on:click=move |_| status.refetch()>"Refresh"</button>
                </div>
                <div class="card-body">
                    <Suspense fallback=|| view! { <div class="loading">"Loading…"</div> }>
                        {move || {
                            status.get().map(|s| {
                                let s = s.unwrap_or_default();
                                view! {
                                    <div class="modem-status">
                                        <div class="status-row">
                                            <span class="status-label">"Present"</span>
                                            {if s.present {
                                                view! { <span class="status-badge status-ok">"Detected"</span> }.into_view()
                                            } else {
                                                view! { <span class="status-badge status-error">"Not detected"</span> }.into_view()
                                            }}
                                        </div>

                                        {if s.present {
                                            view! {
                                                <div class="status-row">
                                                    <span class="status-label">"SIM Status"</span>
                                                    {if s.sim_ready {
                                                        view! { <span class="status-badge status-ok">"Ready"</span> }.into_view()
                                                    } else {
                                                        view! { <span class="status-badge status-error">"Not ready"</span> }.into_view()
                                                    }}
                                                </div>
                                                <div class="status-row">
                                                    <span class="status-label">"Operator"</span>
                                                    <span class="status-value">{s.operator.clone().unwrap_or("—".into())}</span>
                                                </div>
                                                <div class="status-row">
                                                    <span class="status-label">"Signal"</span>
                                                    <span class="status-value">
                                                        {s.signal_strength.map(|v| format!("{} dBm", v)).unwrap_or("—".into())}
                                                    </span>
                                                </div>
                                                <div class="status-row">
                                                    <span class="status-label">"Technology"</span>
                                                    <span class="status-value">{s.access_technology.clone().unwrap_or("—".into())}</span>
                                                </div>
                                                <div class="status-row">
                                                    <span class="status-label">"Registered"</span>
                                                    {if s.registered {
                                                        view! { <span class="status-badge status-ok">"Yes"</span> }.into_view()
                                                    } else {
                                                        view! { <span class="status-badge status-error">"No"</span> }.into_view()
                                                    }}
                                                </div>
                                                <div class="status-row">
                                                    <span class="status-label">"Connected"</span>
                                                    {if s.connected {
                                                        view! { <span class="status-badge status-ok">"Yes"</span> }.into_view()
                                                    } else {
                                                        view! { <span class="status-badge status-error">"No"</span> }.into_view()
                                                    }}
                                                </div>
                                            }.into_view()
                                        } else {
                                            view! { <div class="empty-state">"No modem detected. Connect a SIM7600G-H via USB."</div> }.into_view()
                                        }}
                                    </div>
                                }
                            })
                        }}
                    </Suspense>
                </div>
            </div>

            <div class="card">
                <div class="card-header">
                    <span>"Diagnostics"</span>
                    <button class="btn btn-small" on:click=move |_| diagnostics.refetch()>"Refresh"</button>
                </div>
                <div class="card-body">
                    <Suspense fallback=|| view! { <div class="loading">"Running diagnostics…"</div> }>
                        {move || {
                            diagnostics.get().map(|d| {
                                let d = d.unwrap_or_default();
                                view! { <DiagnosticsSection diag={d} /> }
                            })
                        }}
                    </Suspense>
                </div>
            </div>

            <div class="card">
                <div class="card-header">
                    <span>"SMS"</span>
                    <button class="btn btn-small" on:click=move |_| sms_inbox.refetch()>"Refresh"</button>
                </div>
                <div class="card-body">
                    <SmsSection on_sent=move || { sms_inbox.refetch(); } />
                    <Suspense fallback=|| view! { <div class="loading">"Loading…"</div> }>
                        {move || {
                            sms_inbox.get().map(|msgs| {
                                let msgs = msgs.unwrap_or_default();
                                if msgs.is_empty() {
                                    view! { <div class="empty-state">"No SMS messages"</div> }.into_view()
                                } else {
                                    view! {
                                        <div class="sms-list">
                                            {msgs.iter().map(|m| view! {
                                                <div class="sms-row">
                                                    <span class="sms-sender">{m.sender.clone()}</span>
                                                    <span class="sms-body">{m.body.clone()}</span>
                                                    <span class="sms-time">{m.timestamp.format("%H:%M:%S").to_string()}</span>
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
fn DiagnosticsSection(diag: ModemDiagnostics) -> impl IntoView {
    view! {
        <div class="diagnostics-section">
            <div class="diag-row">
                <span class="diag-label">"ModemManager"</span>
                {if diag.modemmanager_available {
                    view! { <span class="status-badge status-ok">"Available"</span> }.into_view()
                } else {
                    view! { <span class="status-badge status-error">"Not installed"</span> }.into_view()
                }}
            </div>

            <div class="diag-row">
                <span class="diag-label">"Serial Ports"</span>
                <span class="diag-value">
                    {if diag.serial_ports.is_empty() {
                        "None found".to_string()
                    } else {
                        diag.serial_ports.join(", ")
                    }}
                </span>
            </div>

            {move || {
                diag.at_response.as_ref().map(|resp| {
                    view! {
                        <div class="at-diagnostics">
                            <h3>"AT Command Output"</h3>
                            <pre class="at-output">{resp.clone()}</pre>
                        </div>
                    }
                })
            }}
        </div>
    }
}

#[component]
fn SmsSection(on_sent: impl Fn() + 'static) -> impl IntoView {
    let (number, set_number) = create_signal(String::new());
    let (text, set_text) = create_signal(String::new());
    let (info, set_info) = create_signal::<Option<String>>(None);
    let (error, set_error) = create_signal::<Option<String>>(None);

    let on_sent = store_value(on_sent);

    let do_send = create_action(move |_: &()| {
        let n = number.get();
        let t = text.get();
        let on_sent = on_sent;
        async move {
            if n.is_empty() || t.is_empty() {
                set_error.set(Some("Number and text are required".into()));
                set_info.set(None);
                return;
            }
            match send_sms(n, t).await {
                Ok(()) => {
                    set_info.set(Some("SMS sent".into()));
                    set_error.set(None);
                    set_number.set(String::new());
                    set_text.set(String::new());
                    on_sent.with_value(|f| f());
                }
                Err(e) => {
                    set_error.set(Some(format!("{e}")));
                    set_info.set(None);
                }
            }
        }
    });

    view! {
        <div class="sms-send-form">
            {move || info.get().map(|i| view! { <div class="info-msg">{i}</div> })}
            {move || error.get().map(|e| view! { <div class="error-msg">{e}</div> })}

            <div class="form-field">
                <label>"Phone Number"</label>
                <input type="text" placeholder="+1234567890"
                    prop:value={number.get()}
                    on:input=move |ev| set_number.set(event_target_value(&ev)) />
            </div>
            <div class="form-field">
                <label>"Message"</label>
                <textarea rows="3"
                    prop:value={text.get()}
                    on:input=move |ev| set_text.set(event_target_value(&ev))></textarea>
            </div>
            <button class="btn btn-primary" on:click=move |_| do_send.dispatch(())>"Send SMS"</button>
        </div>
    }
}
