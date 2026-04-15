use auth_token_demo_core::{
    load_profile_flow_now,
    login_user_flow_now,
    logout_user_flow_now,
};

pub fn handle_login_route_now() -> String {
    login_user_flow_now()
}

pub fn handle_logout_route_now() -> String {
    logout_user_flow_now()
}

pub fn handle_profile_route_now() -> String {
    load_profile_flow_now()
}
