use std::{fmt::Debug, sync::Arc};

use crate::ffmpeg::{format::context::Input, media::Type, packet::Packet};

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

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

pub struct StreamSender {
    pub stream_index: usize,
    pub sender: UnboundedSender<Message>,
}

pub struct StreamReceiver {
    pub receiver: UnboundedReceiver<Message>,
}

pub fn new_stream_channel(stream_typ: StreamType, ictx: &Input) -> (StreamSender, StreamReceiver) {
    let input = ictx.streams().best(stream_typ.as_media_type()).unwrap();
    let stream_index = input.index();

    let (sender, receiver) = unbounded_channel();

    (
        StreamSender {
            stream_index,
            sender,
        },
        StreamReceiver { receiver },
    )
}
