use crate::Game;
use crate::Command;
use tokio::sync::oneshot;

use crossbeam::channel::Sender;

pub trait CommandSender {
    fn send_command<R, F>(&self, f: F) -> R
    where
        R: Send + 'static,
        F: FnOnce(&mut Game) -> R + Send + Sync + 'static;
}

impl CommandSender for Sender<Box<dyn Command>> {
    fn send_command<R, F>(&self, f: F) -> R
    where
        R: Send + 'static,
        F: FnOnce(&mut Game) -> R + Send + Sync + 'static,
        {
            let (tx, rx) = oneshot::channel();

            let cmd = Box::new(CommandWithResultSync {
                run: Box::new(f),
                respond_to: tx,
            });

            self.send(cmd).expect("Failed to send command to main thread");

            rx.blocking_recv().expect("Command failed")
        }
}

pub struct CommandWithResultSync<R> {
    pub run: Box<dyn FnOnce(&mut Game) -> R + Send + Sync>,
    pub respond_to: oneshot::Sender<R>,
}

impl<R: Send + 'static> Command for CommandWithResultSync<R> {
    fn apply(self: Box<Self>, game: &mut Game) {
        let result = (self.run)(game);
        let _ = self.respond_to.send(result);
    }
}
