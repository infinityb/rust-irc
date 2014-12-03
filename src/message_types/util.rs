pub fn is_valid_target(target: &str) -> bool {
    target.as_bytes().iter().any(|&byte| {
        byte == b' ' || byte == b'\n'
    })
}