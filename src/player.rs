use crate::{
    ffmpeg::format::input,
    frame_keep::FrameKeep,
    message::{Buffer, Buffers, StreamType},
    video_decoder::VideoDecoder,
};

use tokio::sync::mpsc::unbounded_channel;

use std::{path::Path, sync::Arc};

use crate::message::{Message, ReceiverContainer, SenderContainer};

pub struct Player {
    pub buffers: Buffers,
    buffer: Option<Buffer>,
}

impl Player {
    pub fn new() -> Player {
        Player {
            buffers: Default::default(),
            buffer: None,
        }
    }

    pub fn set_path(&self, path: impl AsRef<Path>) {
        let path = path.as_ref().to_path_buf();

        let mut ictx = input(&path).unwrap();

        let mut sender_container = SenderContainer::default();
        let mut receiver_container = ReceiverContainer::default();

        let (video_receiver, video) = sender_container.add_video_stream(&ictx);

        let (frame_sender, frame_receiver) = unbounded_channel();

        receiver_container.insert(StreamType::Video, frame_receiver);

        let mut video_decoder = VideoDecoder::new(video, frame_sender);

        let buffers = self.buffers.clone();

        let mut frame_keep = FrameKeep::new(buffers, receiver_container);

        //同步帧
        tokio::spawn(async move {
            frame_keep.run().await;
        });

        tokio::spawn(async move {
            video_decoder.run(video_receiver).await;
        });

        tokio::spawn(async move {
            for (stream, packet) in ictx.packets() {
                if let Some(sender) = sender_container.get_mut(&stream.index()) {
                    if let Err(e) = sender.sender.send(Message::Packet(Arc::new(packet))) {
                        println!("sender error: {}", e);
                    }
                }
            }

            for sender in sender_container.values_mut() {
                if let Err(e) = sender.sender.send(Message::End) {
                    println!("sender error: {}", e);
                }
            }
        });
    }
}
