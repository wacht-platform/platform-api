pub fn knowledge_base_mount_name(kb_id: &str, _kb_name: &str) -> String {
    kb_id.to_string()
}

/// Agent-facing filesystem tools must not mutate runtime-owned audit logs.
/// The runtime uses a private append path instead.
pub(crate) fn is_task_audit_path(path: &str) -> bool {
    let mut components = Vec::new();
    for component in path.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                components.pop();
            }
            component => components.push(component),
        }
    }
    components.first() == Some(&"task") && components.get(1) == Some(&"audit")
}
