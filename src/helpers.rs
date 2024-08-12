use image::{DynamicImage, ImageBuffer};

use crate::ffmpeg::util::frame::Video;

pub fn convert_to_image(frame: &Video) -> DynamicImage {
    let bytes = frame.data(0);
    let width = frame.width();
    let height = frame.height();

    let image_buffer = ImageBuffer::from_fn(width, height, |x, y| {
        let index = (x + y * width) as usize * 3;
        image::Rgb([bytes[index], bytes[index + 1], bytes[index + 2]])
    });

    DynamicImage::ImageRgb8(image_buffer)
}
