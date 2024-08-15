use std::{collections::HashMap, mem};

use orx_priority_queue::{BinaryHeap, PriorityQueue};

use crate::message::{Buffer, Buffers, ReceiverContainer};

pub struct FrameKeep {
    buffers: Buffers,
    receiver: ReceiverContainer,
}

#[derive(Clone)]
pub struct BufferContainer {
    buffer: Buffer,
    pts: i64,
}

impl PartialEq for BufferContainer {
    fn eq(&self, other: &Self) -> bool {
        self.pts == other.pts
    }
}

impl PartialOrd for BufferContainer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.pts.partial_cmp(&other.pts)
    }
}

impl Eq for BufferContainer {}

impl Ord for BufferContainer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.pts.cmp(&other.pts)
    }
}

impl FrameKeep {
    pub fn new(buffers: Buffers, receiver: ReceiverContainer) -> Self {
        Self { buffers, receiver }
    }

    pub async fn run(&mut self) {
        let mut heap: BinaryHeap<BufferContainer, i64> = Default::default();
        //存储所有的buffer
        let mut wait_buffers: HashMap<i64, BufferContainer> = Default::default();
        //记录所有已完成但是未排序的buffer
        let mut buffer_ids = vec![];

        loop {
            if let Some((buffer_container, _)) = heap.pop() {
                self.buffers.push(buffer_container.buffer);
            }

            for (stream_type, receiver) in self.receiver.iter_mut() {
                while let Some(frame) = receiver.recv().await {
                    let pts = frame.pts().unwrap();

                    println!("pts:{}", pts);

                    let conainer = wait_buffers.entry(pts).or_insert(BufferContainer {
                        buffer: Buffer::default(),
                        pts,
                    });
                    conainer.buffer.insert(*stream_type, frame);

                    if conainer.buffer.len() == receiver.len() {
                        buffer_ids.push(pts);
                    }
                }
            }

            //完成的送入排序

            let finished = mem::take(&mut buffer_ids);

            for id in finished {
                if let Some(buffer) = wait_buffers.remove(&id) {
                    heap.push(buffer, id);
                }
            }
        }
    }
}
