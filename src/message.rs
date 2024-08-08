use std::{
    collections::HashMap,
    fmt::Debug,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use crate::ffmpeg::{
    self, decoder::Video, format::context::Input, frame::Video as VideoFrame, media::Type,
    packet::Packet,
};

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub type FrameSender = UnboundedSender<Frame>;
pub type FrameReceiver = UnboundedReceiver<Frame>;

pub type FramesSender = UnboundedSender<Frames>;
pub type FramesReceiver = UnboundedReceiver<Frames>;

pub struct Frames(pub Vec<Frame>);

pub enum Frame {
    VideoFrame(Arc<VideoFrame>),
}

impl Frame {
    pub fn as_bytes(&self) -> &[u8] {
        match &self {
            Frame::VideoFrame(video) => video.data(0),
        }
    }
}

impl Debug for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Frame::VideoFrame(video) => f.write_str(&format!("{:?}", video.pts())),
        }
    }
}

pub enum Message {
    Packet(Arc<Packet>),
    End,
}

impl Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::End => f.write_str("end"),
            Message::Packet(packet) => f.write_str(&format!("{:?}", packet.pts())),
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum StreamType {
    Video,
}

impl StreamType {
    pub fn as_media_type(&self) -> Type {
        match *self {
            StreamType::Video => Type::Video,
        }
    }
}

#[derive(Default)]
pub struct MessageContainer(HashMap<usize, MessageSender>);

impl MessageContainer {
    pub fn add_video_stream(&mut self, ictx: &Input) -> (MessageReceiver, Video) {
        let input = ictx
            .streams()
            .best(StreamType::Video.as_media_type())
            .unwrap();
        let stream_index = input.index();

        let context_decoder =
            ffmpeg::codec::context::Context::from_parameters(input.parameters()).unwrap();
        let decoder = context_decoder.decoder().video().unwrap();

        let (sender, receiver) = unbounded_channel();

        let stream_sender = MessageSender {
            stream_index,
            sender,
        };

        self.insert(stream_index, stream_sender);

        (MessageReceiver { receiver }, decoder)
    }
}

impl Deref for MessageContainer {
    type Target = HashMap<usize, MessageSender>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MessageContainer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct MessageSender {
    pub stream_index: usize,
    pub sender: UnboundedSender<Message>,
}

pub struct MessageReceiver {
    pub receiver: UnboundedReceiver<Message>,
}
