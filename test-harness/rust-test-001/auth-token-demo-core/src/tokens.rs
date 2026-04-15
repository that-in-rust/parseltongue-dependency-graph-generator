use store_profile_demo_core::{
    fetch_profile_record_now,
    save_session_record_now,
};

pub fn issue_login_token_now() -> String {
    let session = save_session_record_now();
    format!("issue:{session}")
}

pub fn revoke_login_token_now() -> String {
    let session = save_session_record_now();
    format!("revoke:{session}")
}

pub fn read_profile_claims_now() -> String {
    let profile = fetch_profile_record_now();
    format!("claims:{profile}")
}
