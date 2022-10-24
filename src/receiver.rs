use anyhow::Result;
use std::{collections::VecDeque, sync::mpsc};
pub struct Receiver<T> {
    queue: VecDeque<T>,
    receive_channel: mpsc::Receiver<VecDeque<T>>,
}

impl<T> Receiver<T> {
    pub fn new(receive_channel: mpsc::Receiver<VecDeque<T>>) -> Self {
        Self {
            queue: VecDeque::new(),
            receive_channel,
        }
    }

    pub fn recv(&mut self) -> Result<T> {
        // if the queue is empty, wait for an event to be emitted
        match self.queue.pop_front() {
            Some(t) => Ok(t),
            None => {
                self.queue = self.receive_channel.recv()?;
                self.queue
                    .pop_front()
                    .ok_or_else(|| anyhow::Error::msg("no items in queue"))
            }
        }
    }
}
