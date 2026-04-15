pub fn render_login_panel_now() -> String {
    "login-panel".to_string()
}

pub fn render_profile_panel_now() -> String {
    "profile-panel".to_string()
}

pub fn render_status_panel_now() -> String {
    format!(
        "{}|{}",
        render_login_panel_now(),
        render_profile_panel_now()
    )
}
