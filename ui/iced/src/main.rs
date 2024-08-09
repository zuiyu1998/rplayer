use std::env;

use iced::executor;
use iced::widget::image::Handle;
use iced::widget::{container, text, Image};
use iced::{Application, Command, Element, Length, Settings, Subscription, Theme};
use iced_impl::{run_player, PlayerCommandSender, PlayerEvent};
use rplayer::message::Frames;

pub fn main() -> iced::Result {
    App::run(Settings::default())
}

#[derive(Default)]
struct App {
    sender: Option<PlayerCommandSender>,
    image: Option<Frames>,
}

#[derive(Debug, Clone)]
enum Message {
    PlayerEvent(PlayerEvent),
}

impl Application for App {
    type Message = Message;
    type Theme = Theme;
    type Flags = ();
    type Executor = executor::Default;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        rplayer::ffmpeg::init().unwrap();

        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("App - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        println!("message: {:?}", message);

        match message {
            Message::PlayerEvent(e) => match e {
                PlayerEvent::Sender(sender) => {
                    self.sender = Some(sender);
                }
                PlayerEvent::Frames(frames) => {
                    self.image = Some(frames);
                }
                _ => {}
            },
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        let path = env::current_dir().unwrap().join("./test.mkv");

        run_player(path).map(Message::PlayerEvent)
    }

    fn view(&self) -> Element<Message> {
        if self.image.is_none() {
            container(text("empty"))
                .height(Length::Fill)
                .padding(20)
                .into()
        } else {
            let bytes = self
                .image
                .as_ref()
                .unwrap()
                .video_bytes()
                .as_ref()
                .unwrap()
                .to_vec();

            let handle = Handle::from_pixels(1280, 720, bytes);
            container(Image::new(handle))
                .height(Length::Fill)
                .padding(20)
                .into()
        }
    }
}
