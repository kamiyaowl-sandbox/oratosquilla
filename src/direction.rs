/// 方向を示す
#[derive(Copy, Clone, Debug)]
pub enum Direction {
    NoDir,
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}