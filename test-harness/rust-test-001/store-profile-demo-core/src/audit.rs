pub fn write_audit_entry_now() -> String {
    "audit-write".to_string()
}

pub fn read_audit_entry_now() -> String {
    trim_audit_entry_now()
}

pub fn trim_audit_entry_now() -> String {
    "audit-trim".to_string()
}
