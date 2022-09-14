#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Direction {
    Forward = 0,
    Back = 1,
    Up = 2,
    Down = 3,
    Left = 4,
    Right = 5,
}