use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use crate::ffmpeg::{
    self, decoder::Video, format::context::Input, frame::Video as VideoFrame, media::Type,
    packet::Packet,
};

use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    Mutex,
};

pub type FrameSender = UnboundedSender<Frame>;
pub type FrameReceiver = UnboundedReceiver<Frame>;

#[derive(Default, Clone, Debug)]
pub struct Buffer(HashMap<StreamType, Frame>);

impl Buffer {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn insert(&mut self, key: StreamType, frame: Frame) {
        self.0.insert(key, frame);
    }

    pub fn get_video(&self) -> Option<&VideoFrame> {
        self.0
            .get(&StreamType::Video)
            .and_then(|frame| frame.as_video())
    }
}

#[derive(Default, Clone)]
pub struct Buffers(Arc<Mutex<VecDeque<Buffer>>>);

impl Buffers {
    pub fn len(&self) -> usize {
        self.0.try_lock().unwrap().len()
    }

    pub fn pop(&self) -> Option<Buffer> {
        let mut guard = self.0.try_lock().unwrap();
        guard.pop_front()
    }

    pub fn push(&self, buffer: Buffer) {
        let mut guard = self.0.try_lock().unwrap();
        guard.push_back(buffer);
    }
}

#[derive(Clone)]
pub enum Frame {
    VideoFrame(Arc<VideoFrame>),
}

impl Frame {
    pub fn as_video(&self) -> Option<&VideoFrame> {
        match &self {
            Frame::VideoFrame(video) => Some(&video),
        }
    }

    pub fn pts(&self) -> Option<i64> {
        match &self {
            Frame::VideoFrame(video) => video.pts(),
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

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
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
pub struct SenderContainer(HashMap<usize, MessageSender>);

impl SenderContainer {
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

impl Deref for SenderContainer {
    type Target = HashMap<usize, MessageSender>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SenderContainer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Default)]
pub struct ReceiverContainer(HashMap<StreamType, FrameReceiver>);

impl Deref for ReceiverContainer {
    type Target = HashMap<StreamType, FrameReceiver>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ReceiverContainer {
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
