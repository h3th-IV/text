
pub fn validate_email(email:&str) -> bool {
    let ends = vec![".com", ".net", ".io", ".co", ".tech"];
    for i in  0..ends.len() {
        assert!(email.ends_with(ends[i]));
        assert!(email.contains("@"));
    }
    true
}