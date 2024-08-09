use crate::{
    ffmpeg::format::input,
    message::{Frames, FramesReceiver, FramesSender, StreamType},
    video_decoder::VideoDecoder,
};

use tokio::sync::mpsc::{error::TryRecvError, unbounded_channel};

use std::{path::Path, sync::Arc};

use crate::message::{Message, MessageContainer};

pub enum PlayerState {
    Ready,
    Frames(Frames),
    End,
}

pub struct Player {
    frames_reciever: FramesReceiver,
    frames_sender: FramesSender,
}

impl Player {
    pub fn new() -> Player {
        let (frames_sender, frames_reciever) = unbounded_channel();

        Player {
            frames_sender,
            frames_reciever,
        }
    }

    pub async fn recv(&mut self) -> PlayerState {
        match self.frames_reciever.try_recv() {
            Ok(frames) => PlayerState::Frames(frames),
            Err(e) => {
                if e == TryRecvError::Empty {
                    PlayerState::Ready
                } else {
                    PlayerState::End
                }
            }
        }
    }

    pub fn set_path(&self, path: impl AsRef<Path>) {
        let path = path.as_ref().to_path_buf();

        let mut ictx = input(&path).unwrap();

        let mut stream_container = MessageContainer::default();

        let (video_receiver, video) = stream_container.add_video_stream(&ictx);

        let (frame_sender, mut frame_receiver) = unbounded_channel();

        let mut video_decoder = VideoDecoder::new(video, frame_sender);

        let frames_sender = self.frames_sender.clone();

        tokio::spawn(async move {
            loop {
                if let Ok(frame) = frame_receiver.try_recv() {
                    println!("frame {:?}", frame);

                    let mut frames = Frames::default();

                    frames.0.insert(StreamType::Video, frame);

                    if let Err(e) = frames_sender.send(frames) {
                        println!("send frame error {}", e);
                    }
                }
            }
        });

        tokio::spawn(async move {
            video_decoder.run(video_receiver).await;
        });

        tokio::spawn(async move {
            for (stream, packet) in ictx.packets() {
                if let Some(sender) = stream_container.get_mut(&stream.index()) {
                    if let Err(e) = sender.sender.send(Message::Packet(Arc::new(packet))) {
                        println!("sender error: {}", e);
                    }
                }
            }

            for sender in stream_container.values_mut() {
                if let Err(e) = sender.sender.send(Message::End) {
                    println!("sender error: {}", e);
                }
            }
        });
    }
}
