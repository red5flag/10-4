use leptos::*;

use crate::server_fns::camera::{
    delete_clip, is_recording, list_clips, list_snapshots, set_night_vision as set_night_vision_fn,
    start_recording, stop_recording, take_snapshot,
};

#[component]
pub fn CameraPage() -> impl IntoView {
    let (recording, set_recording) = create_signal(false);
    let (night_vision, set_night_vision) = create_signal(false);
    let (error, set_error) = create_signal(None::<String>);
    let (info, set_info) = create_signal(None::<String>);

    let clips_resource = create_resource(|| (), |_| async { list_clips().await.ok() });
    let snapshots_resource = create_resource(|| (), |_| async { list_snapshots().await.ok() });

    let refresh_status = create_action(move |_: &()| async move {
        if let Ok(rec) = is_recording().await {
            set_recording.set(rec);
        }
    });

    refresh_status.dispatch(());

    let do_start_recording = create_action(move |_: &()| async move {
        set_error.set(None);
        set_info.set(None);
        match start_recording().await {
            Ok(filename) => {
                set_info.set(Some(format!("Recording: {}", filename)));
                set_recording.set(true);
            }
            Err(e) => set_error.set(Some(format!("{e}"))),
        }
    });

    let do_stop_recording = create_action(move |_: &()| async move {
        set_error.set(None);
        set_info.set(None);
        match stop_recording().await {
            Ok(()) => {
                set_info.set(Some("Recording stopped".into()));
                set_recording.set(false);
                clips_resource.refetch();
            }
            Err(e) => set_error.set(Some(format!("{e}"))),
        }
    });

    let do_snapshot = create_action(move |_: &()| async move {
        set_error.set(None);
        set_info.set(None);
        match take_snapshot().await {
            Ok(filename) => {
                set_info.set(Some(format!("Snapshot: {}", filename)));
                snapshots_resource.refetch();
            }
            Err(e) => set_error.set(Some(format!("{e}"))),
        }
    });

    let do_delete_clip = create_action(move |filename: &String| {
        let filename = filename.clone();
        async move {
            delete_clip(filename).await.ok();
            clips_resource.refetch();
        }
    });

    let do_toggle_night_vision = create_action(move |enabled: &bool| {
        let enabled = *enabled;
        async move {
            set_error.set(None);
            match set_night_vision_fn(enabled).await {
                Ok(()) => set_night_vision.set(enabled),
                Err(e) => set_error.set(Some(format!("{e}"))),
            }
        }
    });

    view! {
        <div class="page">
            <h1>"Camera"</h1>

            {move || error.get().map(|e| view! { <div class="error">{e}</div> })}
            {move || info.get().map(|i| view! { <div class="info-msg">{i}</div> })}

            <div class="camera-grid">
                <div class="card camera-preview-card">
                    <div class="card-header">
                        <span>"Live Preview"</span>
                        {move || if recording.get() {
                            view! { <span class="rec-indicator">"● REC"</span> }.into_view()
                        } else {
                            view! { <span class="status-badge">"Idle"</span> }.into_view()
                        }}
                    </div>
                    <div class="card-body camera-preview-body">
                        <img
                            class="camera-preview"
                            src="/api/stream"
                            alt="Camera preview"
                        />
                    </div>
                </div>

                <div class="card">
                    <div class="card-header"><span>"Controls"</span></div>
                    <div class="card-body camera-controls">
                        {move || if !recording.get() {
                            view! {
                                <button class="btn btn-record" on:click=move |_| do_start_recording.dispatch(())>
                                    "Start Recording"
                                </button>
                            }.into_view()
                        } else {
                            view! {
                                <button class="btn btn-stop" on:click=move |_| do_stop_recording.dispatch(())>
                                    "Stop Recording"
                                </button>
                            }.into_view()
                        }}
                        <button class="btn btn-snapshot" on:click=move |_| do_snapshot.dispatch(())>
                            "Take Snapshot"
                        </button>
                        <label class="toggle-row">
                            <span>"Night Vision (IR)"</span>
                            <input
                                type="checkbox"
                                checked=move || night_vision.get()
                                on:change=move |ev| {
                                    do_toggle_night_vision.dispatch(event_target_checked(&ev));
                                }
                            />
                        </label>
                    </div>
                </div>
            </div>

            <div class="camera-clips-section">
                <div class="card">
                    <div class="card-header">
                        <span>"Recordings"</span>
                        <button class="btn btn-small" on:click=move |_| clips_resource.refetch()>"Refresh"</button>
                    </div>
                    <div class="card-body">
                        <Suspense fallback=|| view! { <div class="loading">"Loading…"</div> }>
                            {move || {
                                clips_resource.get().map(|clips| match clips {
                                    Some(list) if !list.is_empty() => view! {
                                        <div class="clip-list">
                                            {list.iter().map(|c| {
                                                let filename = c.filename.clone();
                                                view! {
                                                <div class="clip-row">
                                                    <a href={format!("/api/clips/{}", c.filename)} target="_blank">
                                                        {c.filename.clone()}
                                                    </a>
                                                    <span class="clip-size">{format_bytes(c.size_bytes)}</span>
                                                    <span class="clip-time">{c.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()}</span>
                                                    <button class="btn btn-small btn-delete"
                                                        on:click={
                                                            let fname = filename.clone();
                                                            move |_| {
                                                                do_delete_clip.dispatch(fname.clone());
                                                            }
                                                        }
                                                    >"Delete"</button>
                                                </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    }.into_view(),
                                    _ => view! { <div class="empty-state">"No recordings"</div> }.into_view(),
                                })
                            }}
                        </Suspense>
                    </div>
                </div>

                <div class="card">
                    <div class="card-header">
                        <span>"Snapshots"</span>
                        <button class="btn btn-small" on:click=move |_| snapshots_resource.refetch()>"Refresh"</button>
                    </div>
                    <div class="card-body">
                        <Suspense fallback=|| view! { <div class="loading">"Loading…"</div> }>
                            {move || {
                                snapshots_resource.get().map(|snaps| match snaps {
                                    Some(list) if !list.is_empty() => view! {
                                        <div class="snapshot-grid">
                                            {list.iter().map(|s| view! {
                                                <div class="snapshot-item">
                                                    <img
                                                        src={format!("/api/clips/{}", s.filename)}
                                                        alt={s.filename.clone()}
                                                        class="snapshot-thumb"
                                                    />
                                                    <span class="snapshot-time">{s.timestamp.format("%H:%M:%S").to_string()}</span>
                                                </div>
                                            }).collect_view()}
                                        </div>
                                    }.into_view(),
                                    _ => view! { <div class="empty-state">"No snapshots"</div> }.into_view(),
                                })
                            }}
                        </Suspense>
                    </div>
                </div>
            </div>
        </div>
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
