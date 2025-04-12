use crate::Game;

pub trait Command {
    fn apply(self: Box<Self>, game: &mut Game);
}
