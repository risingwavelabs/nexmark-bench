use std::{collections::VecDeque, sync::mpsc};

pub struct Sender<T> {
    size: usize,
    queue: VecDeque<T>,
    send_channel: mpsc::SyncSender<VecDeque<T>>,
}

impl<T> Sender<T> {
    pub fn new(size: usize, send_channel: mpsc::SyncSender<VecDeque<T>>) -> Self {
        Self {
            size,
            queue: VecDeque::with_capacity(size),
            send_channel,
        }
    }

    pub fn send(&mut self, msg: T) -> Result<(), mpsc::SendError<VecDeque<T>>> {
        if self.queue.len() == self.size {
            self.send_channel.send(std::mem::replace(
                &mut self.queue,
                VecDeque::with_capacity(self.size),
            ))?;
        }
        self.queue.push_back(msg);
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), mpsc::SendError<VecDeque<T>>> {
        self.send_channel.send(std::mem::replace(
            &mut self.queue,
            VecDeque::with_capacity(self.size),
        ))
    }
}
