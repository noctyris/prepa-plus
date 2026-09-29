pub fn parse_fr(s: &str) -> Option<f32> {
    s.trim().replace(",", ".").parse().ok()
}
