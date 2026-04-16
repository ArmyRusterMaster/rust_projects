use anyhow::{Result, anyhow};
use fast_image_resize as fr;
use image::{DynamicImage, ImageBuffer, Rgba};
use std::num::NonZeroU32;

pub fn resize_image(
    img: DynamicImage,
    new_w: u32,
    new_h: u32,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let width = NonZeroU32::new(img.width()).unwrap();
    let height = NonZeroU32::new(img.height()).unwrap();

    // Конвертируем в формат, который понимает fast_image_resize
    let src_image = fr::Image::from_vec_u8(
        width,
        height,
        img.to_rgba8().into_raw(),
        fr::PixelType::U8x4,
    )?;

    let dst_width = NonZeroU32::new(new_w).ok_or_else(|| anyhow!("Width is 0"))?;
    let dst_height = NonZeroU32::new(new_h).ok_or_else(|| anyhow!("Height is 0"))?;
    let mut dst_image = fr::Image::new(dst_width, dst_height, src_image.pixel_type());

    // Создаем ресайзер (Lanczos3 — лучший баланс скорости и качества)
    let mut resizer = fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3));

    // Выполняем ресайз (здесь автоматически включится SIMD на ARM)
    resizer.resize(&src_image.view(), &mut dst_image.view_mut())?;

    // Возвращаем результат как ImageBuffer
    ImageBuffer::from_raw(new_w, new_h, dst_image.into_vec())
        .ok_or_else(|| anyhow!("Failed to create ImageBuffer"))
}
