use rplayer::player::Player;

#[tokio::main]
async fn main() {
    let player = Player::init();

    player.set_path("./test.mkv");

    loop {}
}
