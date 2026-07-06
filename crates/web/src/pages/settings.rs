use leptos::*;

use crate::server_fns::settings::{
    backup_database, export_config, generate_self_signed_cert, get_settings, import_config,
    list_backups, restore_database, save_settings,
};

#[component]
pub fn SettingsPage() -> impl IntoView {
    let settings = create_resource(|| (), |_| async { get_settings().await.ok() });
    let backups = create_resource(|| (), |_| async { list_backups().await.ok() });

    let (export_text, set_export_text) = create_signal(String::new());
    let (import_text, set_import_text) = create_signal(String::new());
    let (info, set_info) = create_signal::<Option<String>>(None);
    let (error, set_error) = create_signal::<Option<String>>(None);

    let show_info = move |msg: String| {
        set_info.set(Some(msg));
        set_error.set(None);
    };
    let show_error = move |msg: String| {
        set_error.set(Some(msg));
        set_info.set(None);
    };

    view! {
        <div class="page">
            <h1>"Settings"</h1>

            {move || info.get().map(|i| view! { <div class="info-msg">{i}</div> })}
            {move || error.get().map(|e| view! { <div class="error-msg">{e}</div> })}

            <Suspense fallback=|| view! { <div class="loading">"Loading…"</div> }>
                {move || {
                    settings.get().map(|s| {
                        let s = s.unwrap_or_default();
                        view! {
                            <SettingsForm config={s} on_saved=move || {
                                show_info("Settings saved".into());
                                settings.refetch();
                            } on_error=show_error />
                        }
                    })
                }}
            </Suspense>

            <div class="card">
                <div class="card-header"><span>"TLS / HTTPS"</span></div>
                <div class="card-body">
                    <p class="form-hint">"Generate a self-signed certificate for HTTPS access."</p>
                    <button class="btn btn-primary" on:click=move |_| {
                        let settings = settings;
                        spawn_local(async move {
                            match generate_self_signed_cert().await {
                                Ok(path) => {
                                    show_info(format!("Certificate generated: {}", path));
                                    settings.refetch();
                                }
                                Err(e) => show_error(format!("{e}")),
                            }
                        });
                    }>"Generate Self-Signed Certificate"</button>
                </div>
            </div>

            <div class="card">
                <div class="card-header"><span>"Configuration Export / Import"</span></div>
                <div class="card-body">
                    <div class="settings-actions">
                        <button class="btn" on:click=move |_| {
                            spawn_local(async move {
                                match export_config().await {
                                    Ok(text) => set_export_text.set(text),
                                    Err(e) => show_error(format!("{e}")),
                                }
                            });
                        }>"Export Config"</button>
                    </div>

                    {move || {
                        let text = export_text.get();
                        if text.is_empty() {
                            view! { <div></div> }.into_view()
                        } else {
                            view! {
                                <div class="form-field">
                                    <label>"Exported Configuration"</label>
                                    <textarea class="config-textarea" readonly>{text}</textarea>
                                </div>
                            }.into_view()
                        }
                    }}

                    <div class="form-field">
                        <label>"Import Configuration (JSON)"</label>
                        <textarea class="config-textarea"
                            placeholder="Paste configuration JSON here…"
                            on:input=move |ev| set_import_text.set(event_target_value(&ev))>
                        </textarea>
                    </div>
                    <button class="btn btn-primary" on:click=move |_| {
                        let text = import_text.get();
                        let settings = settings;
                        spawn_local(async move {
                            if text.is_empty() {
                                show_error("Paste configuration JSON first".into());
                                return;
                            }
                            match import_config(text).await {
                                Ok(_) => {
                                    show_info("Configuration imported".into());
                                    settings.refetch();
                                }
                                Err(e) => show_error(format!("{e}")),
                            }
                        });
                    }>"Import Config"</button>
                </div>
            </div>

            <div class="card">
                <div class="card-header">
                    <span>"Database Backup / Restore"</span>
                    <button class="btn btn-small" on:click=move |_| backups.refetch()>"Refresh"</button>
                </div>
                <div class="card-body">
                    <div class="settings-actions">
                        <button class="btn btn-primary" on:click=move |_| {
                            let backups = backups;
                            spawn_local(async move {
                                match backup_database().await {
                                    Ok(path) => {
                                        show_info(format!("Backup created: {}", path));
                                        backups.refetch();
                                    }
                                    Err(e) => show_error(format!("{e}")),
                                }
                            });
                        }>"Create Backup"</button>
                    </div>

                    <Suspense fallback=|| view! { <div class="loading">"Loading…"</div> }>
                        {move || {
                            backups.get().map(|b| {
                                let b = b.unwrap_or_default();
                                if b.is_empty() {
                                    view! { <div class="empty-state">"No backups available."</div> }.into_view()
                                } else {
                                    view! {
                                        <div class="backup-list">
                                            {b.iter().map(|name| {
                                                let name_restore = name.clone();
                                                view! {
                                                    <div class="backup-row">
                                                        <span class="backup-name">{name.clone()}</span>
                                                        <button class="btn btn-small" on:click=move |_| {
                                                            let n = name_restore.clone();
                                                            spawn_local(async move {
                                                                match restore_database(n).await {
                                                                    Ok(_) => show_info("Database restored. Restart may be needed.".into()),
                                                                    Err(e) => show_error(format!("{e}")),
                                                                }
                                                            });
                                                        }>"Restore"</button>
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
fn SettingsForm(
    config: pi_kiosk_core::AppConfig,
    on_saved: impl Fn() + 'static,
    on_error: impl Fn(String) + 'static,
) -> impl IntoView {
    let (device_name, set_device_name) = create_signal(config.device_name.clone());
    let (listen_addr, set_listen_addr) = create_signal(config.listen_addr.clone());
    let (retention_days, set_retention_days) = create_signal(config.storage.max_retention_days);
    let (max_disk_pct, set_max_disk_pct) = create_signal(config.storage.max_disk_usage_pct);
    let (tls_enabled, set_tls_enabled) = create_signal(config.tls.enabled);
    let (cert_path, set_cert_path) = create_signal(config.tls.cert_path.clone());
    let (key_path, set_key_path) = create_signal(config.tls.key_path.clone());

    let on_saved = store_value(on_saved);
    let on_error = store_value(on_error);

    let do_save = create_action(move |_: &()| {
        let dn = device_name.get();
        let la = listen_addr.get();
        let rd = retention_days.get();
        let md = max_disk_pct.get();
        let te = tls_enabled.get();
        let cp = cert_path.get();
        let kp = key_path.get();
        let on_saved = on_saved;
        let on_error = on_error;
        async move {
            let mut cfg = pi_kiosk_core::AppConfig::default();
            if let Ok(s) = get_settings().await {
                cfg = s;
            }
            cfg.device_name = dn;
            cfg.listen_addr = la;
            cfg.storage.max_retention_days = rd;
            cfg.storage.max_disk_usage_pct = md;
            cfg.tls.enabled = te;
            cfg.tls.cert_path = cp;
            cfg.tls.key_path = kp;

            match save_settings(cfg).await {
                Ok(_) => on_saved.with_value(|f| f()),
                Err(e) => on_error.with_value(|f| f(format!("{e}"))),
            }
        }
    });

    view! {
        <div class="card">
            <div class="card-header"><span>"Device Settings"</span></div>
            <div class="card-body">
                <div class="form-row">
                    <div class="form-field">
                        <label>"Device Name"</label>
                        <input type="text"
                            prop:value={device_name.get()}
                            on:input=move |ev| set_device_name.set(event_target_value(&ev)) />
                    </div>
                    <div class="form-field">
                        <label>"Listen Address"</label>
                        <input type="text"
                            prop:value={listen_addr.get()}
                            on:input=move |ev| set_listen_addr.set(event_target_value(&ev)) />
                    </div>
                </div>

                <div class="form-row">
                    <div class="form-field">
                        <label>"Clip Retention (days)"</label>
                        <input type="number" min="1" max="365"
                            prop:value={retention_days.get()}
                            on:input=move |ev| {
                                if let Ok(n) = event_target_value(&ev).parse::<u32>() {
                                    set_retention_days.set(n);
                                }
                            } />
                    </div>
                    <div class="form-field">
                        <label>"Max Disk Usage (%)"</label>
                        <input type="number" min="50" max="99"
                            prop:value={max_disk_pct.get()}
                            on:input=move |ev| {
                                if let Ok(n) = event_target_value(&ev).parse::<u8>() {
                                    set_max_disk_pct.set(n);
                                }
                            } />
                    </div>
                </div>

                <div class="form-field">
                    <label>
                        <input type="checkbox"
                            prop:checked={tls_enabled.get()}
                            on:change=move |ev| set_tls_enabled.set(event_target_checked(&ev)) />
                        " Enable HTTPS (TLS)"
                    </label>
                </div>

                {move || {
                    if tls_enabled.get() {
                        view! {
                            <div class="form-row">
                                <div class="form-field">
                                    <label>"Certificate Path"</label>
                                    <input type="text"
                                        prop:value={cert_path.get()}
                                        on:input=move |ev| set_cert_path.set(event_target_value(&ev)) />
                                </div>
                                <div class="form-field">
                                    <label>"Key Path"</label>
                                    <input type="text"
                                        prop:value={key_path.get()}
                                        on:input=move |ev| set_key_path.set(event_target_value(&ev)) />
                                </div>
                            </div>
                        }.into_view()
                    } else {
                        view! { <div></div> }.into_view()
                    }
                }}

                <button class="btn btn-primary" on:click=move |_| do_save.dispatch(())>"Save Settings"</button>
            </div>
        </div>
    }
}
