#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Direction {
    /// Positive Z
    Forward = 0,
    /// Negative Z
    Back = 1,
    /// Positive Y
    Up = 2,
    /// Negative Y
    Down = 3,
    /// Positive X
    Right = 4,
    /// Negative X
    Left = 5,
}