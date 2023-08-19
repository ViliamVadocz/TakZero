use fast_tak::{takparse::Move, Game, Reserves};
use rand::{seq::IteratorRandom, Rng};

pub trait Environment: Send + Sync + Clone + Default {
    type Action: Send + Sync + Clone + PartialEq;

    fn new_opening(rng: &mut impl Rng) -> Self;
    fn populate_actions(&self, actions: &mut Vec<Self::Action>);
    fn step(&mut self, action: Self::Action);
    fn terminal(&self) -> Option<Terminal>;
}

pub enum Terminal {
    Win,
    Loss,
    Draw,
}

impl<const N: usize, const HALF_KOMI: i8> Environment for Game<N, HALF_KOMI>
where
    Reserves<N>: Default,
{
    type Action = Move;

    fn new_opening(rng: &mut impl Rng) -> Self {
        let mut game = Self::default();
        let mut actions = Vec::new();
        for _ in 0..2 {
            game.populate_actions(&mut actions);
            game.play(actions.drain(..).choose(rng).unwrap()).unwrap();
        }
        game
    }

    fn populate_actions(&self, actions: &mut Vec<Self::Action>) {
        self.possible_moves(actions);
    }

    fn step(&mut self, action: Self::Action) {
        self.play(action).expect("Action should be valid.");
    }

    fn terminal(&self) -> Option<Terminal> {
        match self.result() {
            fast_tak::GameResult::Winner { color, .. } => {
                if color == self.to_move {
                    Some(Terminal::Win)
                } else {
                    Some(Terminal::Loss)
                }
            }
            fast_tak::GameResult::Draw { .. } => Some(Terminal::Draw),
            fast_tak::GameResult::Ongoing => None,
        }
    }
}

impl From<Terminal> for f32 {
    fn from(value: Terminal) -> Self {
        match value {
            Terminal::Win => 1.0,
            Terminal::Loss => -1.0,
            Terminal::Draw => 0.0,
        }
    }
}
