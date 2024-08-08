use std::sync::Arc;

use crate::{
    ffmpeg::{
        decoder::Video as Decoder,
        format::Pixel,
        software::scaling::{flag::Flags, Context},
        util::frame::Video,
    },
    message::{Frame, FrameSender, Message, MessageReceiver},
};

pub struct Scaler(Context);

pub struct VideoDecoder {
    decoder: Decoder,
    scaler: Scaler,
    frame_sender: FrameSender,
}

unsafe impl Send for Scaler {}

impl VideoDecoder {
    pub fn new(decoder: Decoder, frame_sender: FrameSender) -> Self {
        let scaler = Scaler(
            Context::get(
                decoder.format(),
                decoder.width(),
                decoder.height(),
                Pixel::RGB24,
                decoder.width(),
                decoder.height(),
                Flags::BILINEAR,
            )
            .unwrap(),
        );
        VideoDecoder {
            decoder,
            scaler,
            frame_sender,
        }
    }

    pub async fn run(&mut self, mut video_receiver: MessageReceiver) {
        loop {
            if let Some(message) = video_receiver.receiver.recv().await {
                match message {
                    Message::Packet(ref packet) => {
                        if let Err(e) = self.decoder.send_packet((*packet).as_ref()) {
                            println!("send packet error: {}", e);
                        }

                        self.receive_and_process_decoded_frames();
                    }

                    Message::End => {
                        if let Err(e) = self.decoder.send_eof() {
                            println!("send packet end error: {}", e);
                        }

                        self.receive_and_process_decoded_frames();

                        break;
                    }
                }
            }
        }
    }

    pub fn receive_and_process_decoded_frames(&mut self) {
        let mut decoded = Video::empty();

        while self.decoder.receive_frame(&mut decoded).is_ok() {
            let mut rgb_frame = Video::empty();
            if let Err(e) = self.scaler.0.run(&decoded, &mut rgb_frame) {
                println!("scaler error: {}", e);
            } else {
                rgb_frame.set_pts(decoded.pts());

                if let Err(e) = self
                    .frame_sender
                    .send(Frame::VideoFrame(Arc::new(rgb_frame)))
                {
                    println!("frame sender {}", e);
                }
            }
        }
    }
}
