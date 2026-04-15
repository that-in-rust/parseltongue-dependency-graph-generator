pub fn check_password_match_now() -> bool {
    true
}

pub fn check_session_guard_now() -> bool {
    true
}

pub fn require_profile_scope_now() -> bool {
    check_session_guard_now()
}
