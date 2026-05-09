use image::GenericImageView;

pub fn prepare_image(image_data: &[u8]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let img = image::load_from_memory(image_data)?;
    let img = img.resize_exact(28, 28, image::imageops::FilterType::Lanczos3);
    let img = img.to_luma8();
    let mut pixels = Vec::with_capacity(784);
    for pixel in img.pixels() {
        let v = pixel.0[0] as f32 / 255.0;
        // Инверсия: MNIST ожидает чёрное на белом
        pixels.push(1.0 - v);
    }
    Ok(pixels)
}
