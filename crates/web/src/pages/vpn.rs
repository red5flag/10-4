use leptos::*;

use crate::server_fns::vpn::{
    add_vpn_config, delete_vpn_config, export_vpn_config,
    get_tor_config, get_vpn_configs, get_vpn_status, import_vpn_config, save_tor_config,
    start_tor, start_vpn, stop_tor, stop_vpn,
};

#[component]
pub fn VpnPage() -> impl IntoView {
    let status = create_resource(|| (), |_| async { get_vpn_status().await.ok() });
    let configs = create_resource(|| (), |_| async { get_vpn_configs().await.ok() });
    let tor_config = create_resource(|| (), |_| async { get_tor_config().await.ok() });

    view! {
        <div class="page">
            <h1>"VPN & Proxy"</h1>

            <div class="card">
                <div class="card-header">
                    <span>"VPN Status"</span>
                    <button class="btn btn-small" on:click=move |_| status.refetch()>"Refresh"</button>
                </div>
                <div class="card-body">
                    <Suspense fallback=|| view! { <div class="loading">"Loading…"</div> }>
                        {move || {
                            status.get().map(|s| {
                                let s = s.unwrap_or_default();
                                view! {
                                    <div class="vpn-status-grid">
                                        <div class="vpn-status-item">
                                            <span class="vpn-status-label">"WireGuard"</span>
                                            {if s.wireguard_active {
                                                view! { <span class="status-badge status-ok">{format!("Active ({})", s.wireguard_interface.clone().unwrap_or("—".into()))}</span> }.into_view()
                                            } else {
                                                view! { <span class="status-badge status-error">"Inactive"</span> }.into_view()
                                            }}
                                        </div>
                                        <div class="vpn-status-item">
                                            <span class="vpn-status-label">"OpenVPN"</span>
                                            {if s.openvpn_active {
                                                view! { <span class="status-badge status-ok">"Active"</span> }.into_view()
                                            } else {
                                                view! { <span class="status-badge status-error">"Inactive"</span> }.into_view()
                                            }}
                                        </div>
                                        <div class="vpn-status-item">
                                            <span class="vpn-status-label">"Tor SOCKS"</span>
                                            {if s.tor_active {
                                                view! { <span class="status-badge status-ok">{format!("Active (:{}", s.tor_socks_port.unwrap_or(9050))}</span> }.into_view()
                                            } else {
                                                view! { <span class="status-badge status-error">"Inactive"</span> }.into_view()
                                            }}
                                        </div>
                                        <div class="vpn-status-item">
                                            <span class="vpn-status-label">"Kill-Switch"</span>
                                            {if s.kill_switch_active {
                                                view! { <span class="status-badge status-ok">"Enabled"</span> }.into_view()
                                            } else {
                                                view! { <span class="status-badge status-error">"Disabled"</span> }.into_view()
                                            }}
                                        </div>
                                    </div>
                                }
                            })
                        }}
                    </Suspense>
                </div>
            </div>

            <div class="card">
                <div class="card-header">
                    <span>"VPN Configurations"</span>
                    <button class="btn btn-small" on:click=move |_| configs.refetch()>"Refresh"</button>
                </div>
                <div class="card-body">
                    <Suspense fallback=|| view! { <div class="loading">"Loading…"</div> }>
                        {move || {
                            configs.get().map(|cfgs| {
                                let cfgs = cfgs.unwrap_or_default();
                                if cfgs.is_empty() {
                                    view! { <div class="empty-state">"No VPN configurations. Add one below."</div> }.into_view()
                                } else {
                                    view! {
                                        <div class="vpn-config-list">
                                            {cfgs.iter().map(|c| {
                                                let id = c.id.clone();
                                                let id_stop = id.clone();
                                                let id_start = id.clone();
                                                let id_del = id.clone();
                                                let id_exp = id.clone();
                                                view! {
                                                    <div class="vpn-config-row">
                                                        <div class="vpn-config-info">
                                                            <span class="vpn-config-name">{c.name.clone()}</span>
                                                            <span class="vpn-config-type">{c.vpn_type.to_string()}</span>
                                                            <span class="vpn-config-mode">{c.mode.to_string()}</span>
                                                            {if c.kill_switch {
                                                                view! { <span class="status-badge status-ok">"Kill-Switch"</span> }.into_view()
                                                            } else {
                                                                view! { <span class="status-badge status-dim">"No KS"</span> }.into_view()
                                                            }}
                                                        </div>
                                                        <div class="vpn-config-actions">
                                                            <button class="btn btn-small btn-primary" on:click=move |_| {
                                                                let id = id_start.clone();
                                                                let cfgs = configs;
                                                                spawn_local(async move {
                                                                    let _ = start_vpn(id).await;
                                                                    cfgs.refetch();
                                                                    status.refetch();
                                                                });
                                                            }>"Start"</button>
                                                            <button class="btn btn-small" on:click=move |_| {
                                                                let id = id_stop.clone();
                                                                let cfgs = configs;
                                                                spawn_local(async move {
                                                                    let _ = stop_vpn(id).await;
                                                                    cfgs.refetch();
                                                                    status.refetch();
                                                                });
                                                            }>"Stop"</button>
                                                            <button class="btn btn-small" on:click=move |_| {
                                                                let id = id_exp.clone();
                                                                spawn_local(async move {
                                                                    let _ = export_vpn_config(id).await;
                                                                });
                                                            }>"Export"</button>
                                                            <button class="btn btn-small btn-danger" on:click=move |_| {
                                                                let id = id_del.clone();
                                                                let cfgs = configs;
                                                                spawn_local(async move {
                                                                    let _ = delete_vpn_config(id).await;
                                                                    cfgs.refetch();
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
                <div class="card-header"><span>"Add VPN Configuration"</span></div>
                <div class="card-body">
                    <AddVpnForm on_added=move || { configs.refetch(); } />
                </div>
            </div>

            <div class="card">
                <div class="card-header"><span>"Import Configuration"</span></div>
                <div class="card-body">
                    <ImportVpnForm on_imported=move || { configs.refetch(); } />
                </div>
            </div>

            <div class="card">
                <div class="card-header"><span>"Tor SOCKS Proxy"</span></div>
                <div class="card-body">
                    <Suspense fallback=|| view! { <div class="loading">"Loading…"</div> }>
                        {move || {
                            tor_config.get().map(|tc| {
                                let tc = tc.unwrap_or_default();
                                view! {
                                    <TorConfigForm
                                        config={tc}
                                        on_saved=move || { tor_config.refetch(); status.refetch(); }
                                    />
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
fn AddVpnForm(on_added: impl Fn() + 'static) -> impl IntoView {
    let (name, set_name) = create_signal(String::new());
    let (vpn_type, set_vpn_type) = create_signal(pi_kiosk_core::VpnType::Wireguard);
    let (mode, set_mode) = create_signal(pi_kiosk_core::VpnMode::Client);
    let (config_path, set_config_path) = create_signal(String::new());
    let (kill_switch, set_kill_switch) = create_signal(false);
    let (info, set_info) = create_signal::<Option<String>>(None);
    let (error, set_error) = create_signal::<Option<String>>(None);

    let on_added = store_value(on_added);

    let do_add = create_action(move |_: &()| {
        let n = name.get();
        let t = vpn_type.get();
        let m = mode.get();
        let p = config_path.get();
        let ks = kill_switch.get();
        let on_added = on_added;
        async move {
            if n.is_empty() || p.is_empty() {
                set_error.set(Some("Name and config path are required".into()));
                set_info.set(None);
                return;
            }
            match add_vpn_config(n, t, m, p, ks).await {
                Ok(_) => {
                    set_info.set(Some("Configuration added".into()));
                    set_error.set(None);
                    set_name.set(String::new());
                    set_config_path.set(String::new());
                    on_added.with_value(|f| f());
                }
                Err(e) => {
                    set_error.set(Some(format!("{e}")));
                    set_info.set(None);
                }
            }
        }
    });

    view! {
        <div class="vpn-add-form">
            {move || info.get().map(|i| view! { <div class="info-msg">{i}</div> })}
            {move || error.get().map(|e| view! { <div class="error-msg">{e}</div> })}

            <div class="form-row">
                <div class="form-field">
                    <label>"Name"</label>
                    <input type="text" prop:value={name.get()}
                        on:input=move |ev| set_name.set(event_target_value(&ev)) />
                </div>
                <div class="form-field">
                    <label>"Type"</label>
                    <select on:change=move |ev| {
                        set_vpn_type.set(match event_target_value(&ev).as_str() {
                            "openvpn" => pi_kiosk_core::VpnType::Openvpn,
                            _ => pi_kiosk_core::VpnType::Wireguard,
                        });
                    }>
                        <option value="wireguard" selected>"WireGuard"</option>
                        <option value="openvpn">"OpenVPN"</option>
                    </select>
                </div>
            </div>

            <div class="form-row">
                <div class="form-field">
                    <label>"Mode"</label>
                    <select on:change=move |ev| {
                        set_mode.set(match event_target_value(&ev).as_str() {
                            "server" => pi_kiosk_core::VpnMode::Server,
                            _ => pi_kiosk_core::VpnMode::Client,
                        });
                    }>
                        <option value="client" selected>"Client"</option>
                        <option value="server">"Server"</option>
                    </select>
                </div>
                <div class="form-field">
                    <label>"Config Path"</label>
                    <input type="text" placeholder="/etc/pi-kiosk/vpn/wg0.conf"
                        prop:value={config_path.get()}
                        on:input=move |ev| set_config_path.set(event_target_value(&ev)) />
                </div>
            </div>

            <label class="toggle-row">
                <span>"Kill-Switch (block traffic outside VPN tunnel)"</span>
                <input type="checkbox" checked={kill_switch.get()}
                    on:change=move |ev| set_kill_switch.set(event_target_checked(&ev)) />
            </label>

            <button class="btn btn-primary" on:click=move |_| do_add.dispatch(())>"Add Configuration"</button>
        </div>
    }
}

#[component]
fn ImportVpnForm(on_imported: impl Fn() + 'static) -> impl IntoView {
    let (name, set_name) = create_signal(String::new());
    let (vpn_type, set_vpn_type) = create_signal(pi_kiosk_core::VpnType::Wireguard);
    let (config_text, set_config_text) = create_signal(String::new());
    let (info, set_info) = create_signal::<Option<String>>(None);
    let (error, set_error) = create_signal::<Option<String>>(None);

    let on_imported = store_value(on_imported);

    let do_import = create_action(move |_: &()| {
        let n = name.get();
        let t = vpn_type.get();
        let c = config_text.get();
        let on_imported = on_imported;
        async move {
            if n.is_empty() || c.is_empty() {
                set_error.set(Some("Name and config text are required".into()));
                set_info.set(None);
                return;
            }
            match import_vpn_config(n, c, t).await {
                Ok(_) => {
                    set_info.set(Some("Configuration imported".into()));
                    set_error.set(None);
                    set_name.set(String::new());
                    set_config_text.set(String::new());
                    on_imported.with_value(|f| f());
                }
                Err(e) => {
                    set_error.set(Some(format!("{e}")));
                    set_info.set(None);
                }
            }
        }
    });

    view! {
        <div class="vpn-import-form">
            {move || info.get().map(|i| view! { <div class="info-msg">{i}</div> })}
            {move || error.get().map(|e| view! { <div class="error-msg">{e}</div> })}

            <div class="form-row">
                <div class="form-field">
                    <label>"Name"</label>
                    <input type="text" prop:value={name.get()}
                        on:input=move |ev| set_name.set(event_target_value(&ev)) />
                </div>
                <div class="form-field">
                    <label>"Type"</label>
                    <select on:change=move |ev| {
                        set_vpn_type.set(match event_target_value(&ev).as_str() {
                            "openvpn" => pi_kiosk_core::VpnType::Openvpn,
                            _ => pi_kiosk_core::VpnType::Wireguard,
                        });
                    }>
                        <option value="wireguard" selected>"WireGuard"</option>
                        <option value="openvpn">"OpenVPN"</option>
                    </select>
                </div>
            </div>

            <div class="form-field">
                <label>"Config Text"</label>
                <textarea rows="8" class="config-textarea"
                    prop:value={config_text.get()}
                    on:input=move |ev| set_config_text.set(event_target_value(&ev))></textarea>
            </div>

            <button class="btn btn-primary" on:click=move |_| do_import.dispatch(())>"Import"</button>
        </div>
    }
}

#[component]
fn TorConfigForm(
    config: pi_kiosk_core::TorConfig,
    on_saved: impl Fn() + 'static,
) -> impl IntoView {
    let (enabled, set_enabled) = create_signal(config.enabled);
    let (socks_port, set_socks_port) = create_signal(config.socks_port);
    let (control_port, set_control_port) = create_signal(config.control_port);
    let (use_bridges, set_use_bridges) = create_signal(config.use_bridges);
    let (bridge_lines, set_bridge_lines) = create_signal(config.bridge_lines.join("\n"));
    let (info, set_info) = create_signal::<Option<String>>(None);
    let (error, set_error) = create_signal::<Option<String>>(None);

    let on_saved = store_value(on_saved);

    let do_save = create_action(move |_: &()| {
        let cfg = pi_kiosk_core::TorConfig {
            enabled: enabled.get(),
            socks_port: socks_port.get(),
            control_port: control_port.get(),
            use_bridges: use_bridges.get(),
            bridge_lines: bridge_lines.get()
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect(),
        };
        let on_saved = on_saved;
        async move {
            match save_tor_config(cfg).await {
                Ok(()) => {
                    set_info.set(Some("Tor configuration saved".into()));
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

    let do_start = create_action(move |_: &()| {
        let on_saved = on_saved;
        async move {
            match start_tor().await {
                Ok(()) => {
                    set_info.set(Some("Tor started".into()));
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

    let do_stop = create_action(move |_: &()| {
        let on_saved = on_saved;
        async move {
            match stop_tor().await {
                Ok(()) => {
                    set_info.set(Some("Tor stopped".into()));
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

    view! {
        <div class="tor-config-form">
            {move || info.get().map(|i| view! { <div class="info-msg">{i}</div> })}
            {move || error.get().map(|e| view! { <div class="error-msg">{e}</div> })}

            <p class="form-hint">
                "Tor routes traffic through a SOCKS5 proxy at 127.0.0.1:9050 for anonymity. \
                 Applications must be configured to use the SOCKS proxy."
            </p>

            <label class="toggle-row">
                <span>"Enable Tor SOCKS proxy"</span>
                <input type="checkbox" checked={enabled.get()}
                    on:change=move |ev| set_enabled.set(event_target_checked(&ev)) />
            </label>

            <div class="form-row">
                <div class="form-field">
                    <label>"SOCKS Port"</label>
                    <input type="number" min="1024" max="65535"
                        prop:value={socks_port.get()}
                        on:input=move |ev| {
                            if let Ok(n) = event_target_value(&ev).parse::<u16>() {
                                set_socks_port.set(n);
                            }
                        } />
                </div>
                <div class="form-field">
                    <label>"Control Port"</label>
                    <input type="number" min="1024" max="65535"
                        prop:value={control_port.get()}
                        on:input=move |ev| {
                            if let Ok(n) = event_target_value(&ev).parse::<u16>() {
                                set_control_port.set(n);
                            }
                        } />
                </div>
            </div>

            <label class="toggle-row">
                <span>"Use bridges (for censorship circumvention)"</span>
                <input type="checkbox" checked={use_bridges.get()}
                    on:change=move |ev| set_use_bridges.set(event_target_checked(&ev)) />
            </label>

            {move || {
                if use_bridges.get() {
                    view! {
                        <div class="form-field">
                            <label>"Bridge Lines (one per line)"</label>
                            <textarea rows="4" class="config-textarea"
                                prop:value={bridge_lines.get()}
                                on:input=move |ev| set_bridge_lines.set(event_target_value(&ev))></textarea>
                        </div>
                    }.into_view()
                } else {
                    view! { <div></div> }.into_view()
                }
            }}

            <div class="tor-actions">
                <button class="btn btn-primary" on:click=move |_| do_save.dispatch(())>"Save Config"</button>
                <button class="btn" on:click=move |_| do_start.dispatch(())>"Start Tor"</button>
                <button class="btn btn-danger" on:click=move |_| do_stop.dispatch(())>"Stop Tor"</button>
            </div>
        </div>
    }
}
