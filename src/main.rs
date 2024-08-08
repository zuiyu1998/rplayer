use rplayer::{ffmpeg, player::Player};

#[tokio::main]
async fn main() {
    ffmpeg::init().unwrap();

    let player = Player::new();

    player.set_path("./test.mkv");

    loop {}
}
