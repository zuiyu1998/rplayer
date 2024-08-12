use std::{fs::File, io::Write};

use image::{DynamicImage, ImageBuffer};
use rplayer::{
    message::{Frames, StreamType},
    player::{Player, PlayerState},
    tokio,
};

#[tokio::main]
async fn main() {
    let mut player = Player::new();

    player.set_path("./test.mkv");

    let mut frame = 0;

    loop {
        let state = player.recv().await;

        match state {
            PlayerState::Ready => {
                println!("ready");
            }
            PlayerState::End => {
                println!("end")
            }
            PlayerState::Frames(frames) => {
                let _ = save_file(&frames, frame);

                frame += 1;
            }
        }
    }
}

fn save_file(frames: &Frames, index: usize) -> std::result::Result<(), std::io::Error> {
    let frame = frames
        .0
        .get(&StreamType::Video)
        .unwrap()
        .as_video()
        .unwrap();

    let bytes = frame.data(0);
    let width = frame.width();
    let height = frame.height();

    let image_buffer = ImageBuffer::from_fn(width, height, |x, y| {
        let index = (x + y * width) as usize * 3;
        image::Rgb([bytes[index], bytes[index + 1], bytes[index + 2]])
    });

    let image = DynamicImage::ImageRgb8(image_buffer);

    image.save(format!("frame{}.png", index)).unwrap();

    // let mut file = File::create(format!("frame{}.ppm", index))?;
    // file.write_all(format!("P6\n{} {}\n255\n", frame.width(), frame.height()).as_bytes())?;
    // file.write_all(frame.data(0))?;
    Ok(())
}
