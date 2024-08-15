use std::{fs::File, io::Write};

use image::{DynamicImage, ImageBuffer};
use rplayer::{
    message::{Buffer, StreamType},
    player::Player,
    tokio,
};

#[tokio::main]
async fn main() {
    let mut player = Player::new();

    player.set_path("./test.mkv");

    loop {
        println!("buffers len: {}", player.buffers.len())
    }
}

fn save_file(frames: &Buffer, index: usize) -> std::result::Result<(), std::io::Error> {
    let frame = frames.get_video().unwrap();

    let bytes = frame.data(0);
    let width = frame.width();
    let height = frame.height();

    let image_buffer = ImageBuffer::from_fn(width, height, |x, y| {
        let index = (x + y * width) as usize * 3;
        image::Rgb([bytes[index], bytes[index + 1], bytes[index + 2]])
    });

    let image = DynamicImage::ImageRgb8(image_buffer);

    image.save(format!("frame{}.png", index)).unwrap();

    Ok(())
}
