pub mod guards;
pub mod tokens;

use store_profile_demo_core::{
    fetch_profile_record_now,
    fetch_user_record_now,
    save_session_record_now,
};

pub fn login_user_flow_now() -> String {
    if guards::check_password_match_now() {
        let user = fetch_user_record_now();
        let token = tokens::issue_login_token_now();
        let session = save_session_record_now();
        format!("login:{user}:{token}:{session}")
    } else {
        "login-denied".to_string()
    }
}

pub fn logout_user_flow_now() -> String {
    if guards::check_session_guard_now() {
        let token = tokens::revoke_login_token_now();
        let session = save_session_record_now();
        format!("logout:{token}:{session}")
    } else {
        "logout-denied".to_string()
    }
}

pub fn load_profile_flow_now() -> String {
    if guards::require_profile_scope_now() {
        let claims = tokens::read_profile_claims_now();
        let profile = fetch_profile_record_now();
        format!("profile:{claims}:{profile}")
    } else {
        "profile-blocked".to_string()
    }
}
