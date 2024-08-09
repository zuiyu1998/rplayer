use std::path::PathBuf;

use iced::futures::channel::mpsc::Sender;
use iced::futures::SinkExt;
use iced::subscription;
use iced::Subscription;
use rplayer::message::Frames;
use rplayer::player::Player;
use rplayer::player::PlayerState;
use rplayer::tokio;
use rplayer::tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub type PlayerCommandSender = UnboundedSender<PlayerCommand>;

pub fn run_player(path: PathBuf) -> Subscription<PlayerEvent> {
    struct Connect;

    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        |mut output| async move {
            let mut state = State::Ready(path);

            let mut player: Player = Player::new();

            loop {
                match &mut state {
                    State::Ready(path) => {
                        let (sender, receiver) = unbounded_channel();

                        let _ = output.send(PlayerEvent::Sender(sender)).await;

                        player.set_path(path);

                        state = State::Running(receiver);
                    }
                    State::Running(receiver) => {
                        tokio::select! {
                            player_state = player.recv() => {
                                handle_player_state(player_state, &mut output).await;
                            }
                            Some(_command) = receiver.recv() => {

                            }
                        }
                    }
                }
            }
        },
    )
}

async fn handle_player_state(player_state: PlayerState, output: &mut Sender<PlayerEvent>) {
    match player_state {
        PlayerState::Ready => {
            tokio::time::sleep(tokio::time::Duration::from_millis(41)).await;
        }
        PlayerState::Frames(frames) => {
            let _ = output.send(PlayerEvent::Frames(frames)).await;
        }
        _ => {
            tokio::time::sleep(tokio::time::Duration::from_millis(41)).await;
        }
    }
}

#[derive(Debug, Clone)]
pub enum PlayerEvent {
    Test,
    Sender(UnboundedSender<PlayerCommand>),
    Frames(Frames),
}

pub enum PlayerCommand {}

pub enum State {
    Ready(PathBuf),
    Running(UnboundedReceiver<PlayerCommand>),
}
