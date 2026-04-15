pub mod http;
pub mod view;

pub fn boot_shell_flow_now() -> String {
    let route = http::handle_login_route_now();
    let panel = view::render_status_panel_now();
    format!("boot:{route}:{panel}")
}

pub fn sync_shell_state_now() -> String {
    let route = http::handle_profile_route_now();
    let panel = view::render_profile_panel_now();
    format!("sync:{route}:{panel}")
}

pub fn show_shell_status_now() -> String {
    view::render_status_panel_now()
}
