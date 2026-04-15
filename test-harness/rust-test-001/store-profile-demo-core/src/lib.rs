pub mod audit;
pub mod cache;

pub fn fetch_user_record_now() -> String {
    let cache = cache::read_profile_cache_now();
    format!("user:{cache}")
}

pub fn save_session_record_now() -> String {
    let audit = audit::write_audit_entry_now();
    let cache = cache::clear_profile_cache_now();
    format!("session:{audit}:{cache}")
}

pub fn fetch_profile_record_now() -> String {
    let cache = cache::warm_profile_cache_now();
    let audit = audit::read_audit_entry_now();
    format!("profile:{cache}:{audit}")
}
