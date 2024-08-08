use crate::ffmpeg::{
    self,
    format::{context::Input, input, Pixel},
    media::Type,
};

use std::{collections::HashMap, path::Path, sync::Arc};

use crate::message::{new_stream_channel, Message, StreamType};

pub struct Player;

impl Player {
    pub fn init() -> Player {
        ffmpeg::init().unwrap();

        Player
    }

    pub fn set_path(&self, path: impl AsRef<Path>) {
        let path = path.as_ref().to_path_buf();

        let mut ictx = input(&path).unwrap();

        let (video_sender, mut video_receiver) = new_stream_channel(StreamType::Video, &ictx);

        let mut senders = HashMap::new();

        tokio::spawn(async move {
            loop {
                if let Some(message) = video_receiver.receiver.recv().await {
                    println!("receiver message: {:?}", message);
                }
            }
        });

        senders.insert(video_sender.stream_index, video_sender);

        tokio::spawn(async move {
            for (stream, packet) in ictx.packets() {
                if let Some(sender) = senders.get_mut(&stream.index()) {
                    if let Err(e) = sender.sender.send(Message::Packet(Arc::new(packet))) {
                        println!("sender error: {}", e);
                    }
                }
            }

            for sender in senders.values_mut() {
                if let Err(e) = sender.sender.send(Message::End) {
                    println!("sender error: {}", e);
                }
            }
        });
    }
}
