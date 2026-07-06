use leptos::*;

use crate::server_fns::network::{
    add_firewall_rule, delete_firewall_rule, get_connected_clients, get_firewall_rules,
    get_interfaces, get_network_config, get_usb_wifi_adapters, get_wan_status, save_network_config,
    toggle_firewall_rule,
};

#[component]
pub fn NetworkPage() -> impl IntoView {
    let wan = create_resource(|| (), |_| async { get_wan_status().await.ok() });
    let clients = create_resource(|| (), |_| async { get_connected_clients().await.ok() });
    let interfaces = create_resource(|| (), |_| async { get_interfaces().await.ok() });
    let net_config = create_resource(|| (), |_| async { get_network_config().await.ok() });
    let firewall_rules = create_resource(|| (), |_| async { get_firewall_rules().await.ok() });
    let usb_adapters = create_resource(|| (), |_| async { get_usb_wifi_adapters().await.ok() });

    view! {
        <div class="page">
            <h1>"Network"</h1>

            <div class="network-grid">
                <div class="card">
                    <div class="card-header"><span>"WAN Status"</span></div>
                    <div class="card-body">
                        <Suspense fallback=|| view! { <div class="loading">"Loading…"</div> }>
                            {move || {
                                wan.get().map(|w| match w {
                                    Some(status) => view! {
                                        <div class="wan-status">
                                            <div class="status-row">
                                                <span class="status-label">"Status"</span>
                                                {if status.online {
                                                    view! { <span class="status-badge status-ok">"Online"</span> }.into_view()
                                                } else {
                                                    view! { <span class="status-badge status-error">"Offline"</span> }.into_view()
                                                }}
                                            </div>
                                            <div class="status-row">
                                                <span class="status-label">"Active WAN"</span>
                                                <span class="status-value">{wan_source_text(status.active_source)}</span>
                                            </div>
                                            <div class="status-row">
                                                <span class="status-label">"IP Address"</span>
                                                <span class="status-value">{status.ip_address.clone().unwrap_or("—".into())}</span>
                                            </div>
                                            <div class="status-row">
                                                <span class="status-label">"Gateway"</span>
                                                <span class="status-value">{status.gateway.clone().unwrap_or("—".into())}</span>
                                            </div>
                                            <div class="wan-sources">
                                                <div class="wan-source-row" class:up={status.ethernet_up} class:down={!status.ethernet_up}>
                                                    <span class="wan-icon">"ETH"</span>
                                                    <span>{if status.ethernet_up { "Connected" } else { "Disconnected" }}</span>
                                                </div>
                                                <div class="wan-source-row" class:up={status.wifi_up} class:down={!status.wifi_up}>
                                                    <span class="wan-icon">"WiFi"</span>
                                                    <span>{if status.wifi_up { "Connected" } else { "Disconnected" }}</span>
                                                </div>
                                                <div class="wan-source-row" class:up={status.cellular_up} class:down={!status.cellular_up}>
                                                    <span class="wan-icon">"4G"</span>
                                                    <span>{if status.cellular_up { "Connected" } else { "Disconnected" }}</span>
                                                </div>
                                            </div>
                                        </div>
                                    }.into_view(),
                                    None => view! { <div class="empty-state">"Failed to load WAN status"</div> }.into_view(),
                                })
                            }}
                        </Suspense>
                    </div>
                </div>

                <div class="card">
                    <div class="card-header">
                        <span>"Connected Clients"</span>
                        <button class="btn btn-small" on:click=move |_| clients.refetch()>"Refresh"</button>
                    </div>
                    <div class="card-body">
                        <Suspense fallback=|| view! { <div class="loading">"Loading…"</div> }>
                            {move || {
                                clients.get().map(|c| match c {
                                    Some(list) if !list.is_empty() => view! {
                                        <div class="client-list">
                                            {list.iter().map(|c| view! {
                                                <div class="client-row">
                                                    <span class="client-hostname">{c.hostname.clone()}</span>
                                                    <span class="client-ip">{c.ip.clone()}</span>
                                                    <span class="client-mac">{c.mac.clone()}</span>
                                                    <span class="client-iface">{c.interface.clone()}</span>
                                                </div>
                                            }).collect_view()}
                                        </div>
                                    }.into_view(),
                                    _ => view! { <div class="empty-state">"No connected clients"</div> }.into_view(),
                                })
                            }}
                        </Suspense>
                    </div>
                </div>
            </div>

            <div class="card">
                <div class="card-header"><span>"Wi-Fi Configuration"</span></div>
                <div class="card-body">
                    <Suspense fallback=|| view! { <div class="loading">"Loading…"</div> }>
                        {move || {
                            net_config.get().map(|cfg| {
                                let nc = net_config;
                                cfg.map(|c| view! {
                                    <WifiConfigForm config={c} on_saved=move || { nc.refetch(); } />
                                })
                            })
                        }}
                    </Suspense>
                </div>
            </div>

            <div class="card">
                <div class="card-header">
                    <span>"Firewall Rules (Port Forwarding)"</span>
                    <button class="btn btn-small" on:click=move |_| firewall_rules.refetch()>"Refresh"</button>
                </div>
                <div class="card-body">
                    <Suspense fallback=|| view! { <div class="loading">"Loading…"</div> }>
                        {move || {
                            firewall_rules.get().map(|rules| {
                                let fr = firewall_rules;
                                rules.map(|r| view! {
                                    <FirewallRulesSection rules={r} on_change=move || { fr.refetch(); } />
                                })
                            })
                        }}
                    </Suspense>
                </div>
            </div>

            <div class="card">
                <div class="card-header">
                    <span>"USB Wi-Fi Adapters"</span>
                    <button class="btn btn-small" on:click=move |_| usb_adapters.refetch()>"Rescan"</button>
                </div>
                <div class="card-body">
                    <Suspense fallback=|| view! { <div class="loading">"Scanning…"</div> }>
                        {move || {
                            usb_adapters.get().map(|adapters| match adapters {
                                Some(list) if !list.is_empty() => view! {
                                    <div class="usb-wifi-list">
                                        {list.iter().map(|a| view! {
                                            <div class="usb-wifi-row">
                                                <span class="usb-iface">{a.interface.clone()}</span>
                                                <span class="usb-product">{a.product.clone()}</span>
                                                <span class="usb-vendor">{a.vendor.clone()}</span>
                                                <span class="usb-driver">{a.driver.clone()}</span>
                                                <span class="usb-cap" class:supported={a.supports_5g} class:unsupported={!a.supports_5g}>
                                                    {if a.supports_5g { "5G" } else { "2.4G only" }}
                                                </span>
                                                <span class="usb-cap" class:supported={a.supports_ap} class:unsupported={!a.supports_ap}>
                                                    {if a.supports_ap { "AP" } else { "STA only" }}
                                                </span>
                                            </div>
                                        }).collect_view()}
                                    </div>
                                }.into_view(),
                                _ => view! { <div class="empty-state">"No USB Wi-Fi adapters detected"</div> }.into_view(),
                            })
                        }}
                    </Suspense>
                </div>
            </div>

            <div class="card">
                <div class="card-header">
                    <span>"Interfaces"</span>
                    <button class="btn btn-small" on:click=move |_| interfaces.refetch()>"Refresh"</button>
                </div>
                <div class="card-body">
                    <Suspense fallback=|| view! { <div class="loading">"Loading…"</div> }>
                        {move || {
                            interfaces.get().map(|ifaces| match ifaces {
                                Some(list) if !list.is_empty() => view! {
                                    <div class="interface-table">
                                        <div class="interface-header-row">
                                            <span>"Name"</span>
                                            <span>"State"</span>
                                            <span>"Type"</span>
                                            <span>"IPv4"</span>
                                            <span>"MAC"</span>
                                            <span>"Speed"</span>
                                        </div>
                                        {list.iter().map(|i| view! {
                                            <div class="interface-row">
                                                <span class="iface-name">{i.name.clone()}</span>
                                                <span class:up={i.state == "up"} class:down={i.state != "up"}>
                                                    {i.state.clone()}
                                                </span>
                                                <span>{i.link_type.clone()}</span>
                                                <span>{i.ipv4.clone().unwrap_or("—".into())}</span>
                                                <span class="iface-mac">{i.mac.clone()}</span>
                                                <span>{i.speed_mbps.map(|s| format!("{} Mbps", s)).unwrap_or("—".into())}</span>
                                            </div>
                                        }).collect_view()}
                                    </div>
                                }.into_view(),
                                _ => view! { <div class="empty-state">"No interfaces found"</div> }.into_view(),
                            })
                        }}
                    </Suspense>
                </div>
            </div>
        </div>
    }
}

#[component]
fn WifiConfigForm(config: pi_kiosk_core::NetworkConfig, on_saved: impl Fn() + 'static) -> impl IntoView {
    let (ssid, set_ssid) = create_signal(config.ssid.clone());
    let (password, set_password) = create_signal(config.password.clone());
    let (channel, set_channel) = create_signal(config.channel);
    let (hidden, set_hidden) = create_signal(config.hidden);
    let (mode, set_mode) = create_signal(config.wifi_mode);
    let (band, set_band) = create_signal(config.band);
    let (encryption, set_encryption) = create_signal(config.encryption);
    let (dhcp_start, set_dhcp_start) = create_signal(config.dhcp_range_start.clone());
    let (dhcp_end, set_dhcp_end) = create_signal(config.dhcp_range_end.clone());
    let (iface, set_iface) = create_signal(config.ap_interface.clone());
    let (save_info, set_save_info) = create_signal::<Option<String>>(None);
    let (save_error, set_save_error) = create_signal::<Option<String>>(None);

    let on_saved = store_value(on_saved);

    let do_save = create_action(move |_: &()| {
        let cfg = pi_kiosk_core::NetworkConfig {
            wifi_mode: mode.get(),
            ssid: ssid.get(),
            password: password.get(),
            channel: channel.get(),
            band: band.get(),
            encryption: encryption.get(),
            country_code: config.country_code.clone(),
            hidden: hidden.get(),
            ap_interface: iface.get(),
            dhcp_range_start: dhcp_start.get(),
            dhcp_range_end: dhcp_end.get(),
            lease_time: config.lease_time.clone(),
            dns_servers: config.dns_servers.clone(),
        };
        async move {
            match save_network_config(cfg).await {
                Ok(()) => {
                    set_save_info.set(Some("Settings saved".into()));
                    set_save_error.set(None);
                    on_saved.with_value(|f| f());
                }
                Err(e) => {
                    set_save_error.set(Some(format!("{e}")));
                    set_save_info.set(None);
                }
            }
        }
    });

    view! {
        <div class="wifi-config-form">
            {move || save_info.get().map(|i| view! { <div class="info-msg">{i}</div> })}
            {move || save_error.get().map(|e| view! { <div class="error-msg">{e}</div> })}

            <div class="form-field">
                <label>"Mode"</label>
                <select on:change=move |ev| {
                    let val = event_target_value(&ev);
                    set_mode.set(match val.as_str() {
                        "ap" => pi_kiosk_core::WifiMode::Ap,
                        "client" => pi_kiosk_core::WifiMode::Client,
                        "repeater" => pi_kiosk_core::WifiMode::Repeater,
                        "hotspot" => pi_kiosk_core::WifiMode::Hotspot,
                        _ => pi_kiosk_core::WifiMode::Ap,
                    });
                }>
                    <option value="ap" selected={mode.get() == pi_kiosk_core::WifiMode::Ap}>"Access Point"</option>
                    <option value="client" selected={mode.get() == pi_kiosk_core::WifiMode::Client}>"Client (STA)"</option>
                    <option value="repeater" selected={mode.get() == pi_kiosk_core::WifiMode::Repeater}>"Repeater"</option>
                    <option value="hotspot" selected={mode.get() == pi_kiosk_core::WifiMode::Hotspot}>"Hotspot"</option>
                </select>
            </div>

            <div class="form-field">
                <label>"SSID"</label>
                <input type="text" prop:value={ssid.get()}
                    on:input=move |ev| set_ssid.set(event_target_value(&ev)) />
            </div>

            <div class="form-field">
                <label>"Password"</label>
                <input type="password" prop:value={password.get()}
                    on:input=move |ev| set_password.set(event_target_value(&ev)) />
            </div>

            <div class="form-row">
                <div class="form-field">
                    <label>"Band"</label>
                    <select on:change=move |ev| {
                        set_band.set(if event_target_value(&ev) == "5g" {
                            pi_kiosk_core::WifiBand::Band5G
                        } else {
                            pi_kiosk_core::WifiBand::Band24G
                        });
                    }>
                        <option value="2.4g" selected={band.get() == pi_kiosk_core::WifiBand::Band24G}>"2.4 GHz"</option>
                        <option value="5g" selected={band.get() == pi_kiosk_core::WifiBand::Band5G}>"5 GHz"</option>
                    </select>
                </div>

                <div class="form-field">
                    <label>"Channel"</label>
                    <input type="number" min="1" max="165" prop:value={channel.get()}
                        on:input=move |ev| {
                            if let Ok(n) = event_target_value(&ev).parse::<u8>() {
                                set_channel.set(n);
                            }
                        } />
                </div>
            </div>

            <div class="form-field">
                <label>"Encryption"</label>
                <select on:change=move |ev| {
                    let val = event_target_value(&ev);
                    set_encryption.set(match val.as_str() {
                        "open" => pi_kiosk_core::Encryption::Open,
                        "wpa3_sae" => pi_kiosk_core::Encryption::Wpa3Sae,
                        _ => pi_kiosk_core::Encryption::Wpa2Psk,
                    });
                }>
                    <option value="wpa2_psk" selected={encryption.get() == pi_kiosk_core::Encryption::Wpa2Psk}>"WPA2-PSK"</option>
                    <option value="wpa3_sae" selected={encryption.get() == pi_kiosk_core::Encryption::Wpa3Sae}>"WPA3-SAE"</option>
                    <option value="open" selected={encryption.get() == pi_kiosk_core::Encryption::Open}>"Open"</option>
                </select>
            </div>

            <div class="form-row">
                <div class="form-field">
                    <label>"AP Interface"</label>
                    <input type="text" prop:value={iface.get()}
                        on:input=move |ev| set_iface.set(event_target_value(&ev)) />
                </div>

                <label class="toggle-row">
                    <span>"Hidden SSID"</span>
                    <input type="checkbox" checked={hidden.get()}
                        on:change=move |ev| set_hidden.set(event_target_checked(&ev)) />
                </label>
            </div>

            <div class="form-row">
                <div class="form-field">
                    <label>"DHCP Range Start"</label>
                    <input type="text" prop:value={dhcp_start.get()}
                        on:input=move |ev| set_dhcp_start.set(event_target_value(&ev)) />
                </div>

                <div class="form-field">
                    <label>"DHCP Range End"</label>
                    <input type="text" prop:value={dhcp_end.get()}
                        on:input=move |ev| set_dhcp_end.set(event_target_value(&ev)) />
                </div>
            </div>

            <button class="btn btn-primary" on:click=move |_| do_save.dispatch(())>"Save Wi-Fi Settings"</button>
        </div>
    }
}

#[component]
fn FirewallRulesSection(
    rules: Vec<pi_kiosk_db::network::FirewallRule>,
    on_change: impl Fn() + 'static,
) -> impl IntoView {
    let (show_add, set_show_add) = create_signal(false);
    let (new_name, set_new_name) = create_signal(String::new());
    let (new_proto, set_new_proto) = create_signal("tcp".to_string());
    let (new_src_port, set_new_src_port) = create_signal(String::new());
    let (new_dest_ip, set_new_dest_ip) = create_signal(String::new());
    let (new_dest_port, set_new_dest_port) = create_signal(String::new());
    let (add_error, set_add_error) = create_signal::<Option<String>>(None);

    let on_change = store_value(on_change);

    let do_add = create_action(move |_: &()| {
        let name = new_name.get();
        let proto = new_proto.get();
        let src_port = {
            let s = new_src_port.get();
            if s.is_empty() { None } else { Some(s) }
        };
        let dest_ip = new_dest_ip.get();
        let dest_port = new_dest_port.get();
        let on_change = on_change;
        async move {
            if name.is_empty() || dest_ip.is_empty() || dest_port.is_empty() {
                set_add_error.set(Some("Name, destination IP, and port are required".into()));
                return;
            }
            match add_firewall_rule(name, proto, src_port, dest_ip, dest_port).await {
                Ok(()) => {
                    set_show_add.set(false);
                    set_add_error.set(None);
                    set_new_name.set(String::new());
                    set_new_dest_ip.set(String::new());
                    set_new_dest_port.set(String::new());
                    set_new_src_port.set(String::new());
                    on_change.with_value(|f| f());
                }
                Err(e) => set_add_error.set(Some(format!("{e}"))),
            }
        }
    });

    view! {
        <div class="firewall-section">
            {move || add_error.get().map(|e| view! { <div class="error-msg">{e}</div> })}

            {if rules.is_empty() {
                view! { <div class="empty-state">"No firewall rules configured"</div> }.into_view()
            } else {
                view! {
                    <div class="firewall-table">
                        <div class="firewall-header-row">
                            <span>"Name"</span>
                            <span>"Proto"</span>
                            <span>"Src Port"</span>
                            <span>"Dest IP"</span>
                            <span>"Dest Port"</span>
                            <span>"Enabled"</span>
                            <span>"Actions"</span>
                        </div>
                        {rules.iter().map(|r| {
                            let rule_id_toggle = r.id.clone();
                            let rule_id_delete = r.id.clone();
                            let enabled = r.enabled;
                            let on_change_inner = on_change;
                            view! {
                                <div class="firewall-row">
                                    <span class="fw-name">{r.name.clone()}</span>
                                    <span>{r.proto.clone()}</span>
                                    <span>{r.src_port.clone().unwrap_or("any".into())}</span>
                                    <span class="fw-ip">{r.dest_ip.clone()}</span>
                                    <span>{r.dest_port.clone()}</span>
                                    <span>
                                        <input type="checkbox" checked={enabled}
                                            on:change=move |ev| {
                                                let checked = event_target_checked(&ev);
                                                let id = rule_id_toggle.clone();
                                                spawn_local(async move {
                                                    if toggle_firewall_rule(id, checked).await.is_ok() {
                                                        on_change_inner.with_value(|f| f());
                                                    }
                                                });
                                            } />
                                    </span>
                                    <span>
                                        <button class="btn btn-small btn-danger" on:click=move |_| {
                                            let id = rule_id_delete.clone();
                                            let oc = on_change_inner;
                                            spawn_local(async move {
                                                if delete_firewall_rule(id).await.is_ok() {
                                                    oc.with_value(|f| f());
                                                }
                                            });
                                        }>"Delete"</button>
                                    </span>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                }.into_view()
            }}

            {move || if show_add.get() {
                view! {
                    <div class="firewall-add-form">
                        <h3>"Add Port Forwarding Rule"</h3>
                        <div class="form-row">
                            <div class="form-field">
                                <label>"Name"</label>
                                <input type="text" prop:value={new_name.get()}
                                    on:input=move |ev| set_new_name.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-field">
                                <label>"Protocol"</label>
                                <select on:change=move |ev| set_new_proto.set(event_target_value(&ev))>
                                    <option value="tcp">"TCP"</option>
                                    <option value="udp">"UDP"</option>
                                    <option value="any">"Any"</option>
                                </select>
                            </div>
                        </div>
                        <div class="form-row">
                            <div class="form-field">
                                <label>"Source Port (optional)"</label>
                                <input type="text" placeholder="e.g. 8080 or any"
                                    prop:value={new_src_port.get()}
                                    on:input=move |ev| set_new_src_port.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-field">
                                <label>"Destination IP"</label>
                                <input type="text" placeholder="e.g. 192.168.1.100"
                                    prop:value={new_dest_ip.get()}
                                    on:input=move |ev| set_new_dest_ip.set(event_target_value(&ev)) />
                            </div>
                        </div>
                        <div class="form-row">
                            <div class="form-field">
                                <label>"Destination Port"</label>
                                <input type="text" placeholder="e.g. 80"
                                    prop:value={new_dest_port.get()}
                                    on:input=move |ev| set_new_dest_port.set(event_target_value(&ev)) />
                            </div>
                        </div>
                        <div class="form-actions">
                            <button class="btn btn-primary" on:click=move |_| do_add.dispatch(())>"Add Rule"</button>
                            <button class="btn" on:click=move |_| { set_show_add.set(false); set_add_error.set(None); }>"Cancel"</button>
                        </div>
                    </div>
                }.into_view()
            } else {
                view! {
                    <button class="btn" on:click=move |_| set_show_add.set(true)>"+ Add Rule"</button>
                }.into_view()
            }}
        </div>
    }
}

fn wan_source_text(source: pi_kiosk_core::WanSource) -> &'static str {
    match source {
        pi_kiosk_core::WanSource::Ethernet => "Ethernet (PoE)",
        pi_kiosk_core::WanSource::Wifi => "Wi-Fi",
        pi_kiosk_core::WanSource::Cellular => "Cellular (4G)",
    }
}
