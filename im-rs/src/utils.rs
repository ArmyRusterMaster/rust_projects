pub fn calculate_dimensions(
    orig_w: u32,
    orig_h: u32,
    target_w: Option<u32>,
    target_h: Option<u32>,
) -> (u32, u32) {
    match (target_w, target_h) {
        (Some(w), Some(h)) => (w, h),
        (Some(w), None) => {
            let h = (w as f32 * (orig_h as f32 / orig_w as f32)) as u32;
            (w, h)
        }
        (None, Some(h)) => {
            let w = (h as f32 * (orig_w as f32 / orig_h as f32)) as u32;
            (w, h)
        }
        _ => (orig_w, orig_h),
    }
}
