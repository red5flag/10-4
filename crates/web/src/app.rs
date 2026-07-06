use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="pi-kiosk" href="/style/main.css"/>
        <Router>
            <Routes>
                <Route path="/login" view=move || view! { <crate::pages::login::LoginPage/> }/>
                <Route path="/*" view=AppShell/>
            </Routes>
        </Router>
    }
}

#[component]
fn AppShell() -> impl IntoView {
    view! {
        <div class="app-shell">
            <nav class="sidebar">
                <div class="sidebar-header">"Pi Kiosk"</div>
                <NavLink href="/" label="Dashboard" icon="home"/>
                <NavLink href="/camera" label="Camera" icon="camera"/>
                <NavLink href="/detection" label="Detection" icon="eye"/>
                <NavLink href="/network" label="Network" icon="wifi"/>
                <NavLink href="/modem" label="Modem" icon="signal"/>
                <NavLink href="/failover" label="Failover" icon="swap"/>
                <NavLink href="/vpn" label="VPN" icon="lock"/>
                <NavLink href="/notifications" label="Alerts" icon="bell"/>
                <NavLink href="/settings" label="Settings" icon="cog"/>
            </nav>
            <main class="content">
                <Routes>
                    <Route path="/" view=move || view! { <crate::pages::dashboard::DashboardPage/> }/>
                    <Route path="/camera" view=move || view! { <crate::pages::camera::CameraPage/> }/>
                    <Route path="/detection" view=move || view! { <crate::pages::detection::DetectionPage/> }/>
                    <Route path="/network" view=move || view! { <crate::pages::network::NetworkPage/> }/>
                    <Route path="/modem" view=move || view! { <crate::pages::modem::ModemPage/> }/>
                    <Route path="/failover" view=move || view! { <crate::pages::failover::FailoverPage/> }/>
                    <Route path="/vpn" view=move || view! { <crate::pages::vpn::VpnPage/> }/>
                    <Route path="/notifications" view=move || view! { <crate::pages::notifications::NotificationsPage/> }/>
                    <Route path="/settings" view=move || view! { <crate::pages::settings::SettingsPage/> }/>
                </Routes>
            </main>
        </div>
    }
}

#[component]
fn NavLink(href: &'static str, label: &'static str, icon: &'static str) -> impl IntoView {
    view! {
        <a href=href class="nav-link">
            <span class="nav-icon">{icon}</span>
            <span class="nav-label">{label}</span>
        </a>
    }
}
