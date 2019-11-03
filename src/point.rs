use super::direction::Direction;

/// 座標
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}
impl Default for Point {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl Point {
    /// ビルドオプション指定がなければ、チェビシフ距離を返します
    pub fn distance(&self, other: Point) -> usize {
        use std::cmp;

        let dx = cmp::max(self.x, other.x) - cmp::min(self.x, other.x);
        let dy = cmp::max(self.y, other.y) - cmp::min(self.y, other.y);

        if cfg!(distance_method = "manhattan") {
            dx + dy
        } else {
            cmp::max(dx, dy)
        }
    }
    /// 指定した方向にある座標を取得します。例外処理は内包していません
    pub fn get_around(&self, dir: Direction) -> Point {
        match dir {
            Direction::NoDir => self.clone(),
            Direction::Up => Point {
                x: self.x,
                y: self.y + 1,
            },
            Direction::Down => Point {
                x: self.x,
                y: self.y - 1,
            },
            Direction::Left => Point {
                x: self.x - 1,
                y: self.y,
            },
            Direction::Right => Point {
                x: self.x + 1,
                y: self.y,
            },
            Direction::UpLeft => Point {
                x: self.x - 1,
                y: self.y + 1,
            },
            Direction::UpRight => Point {
                x: self.x + 1,
                y: self.y + 1,
            },
            Direction::DownLeft => Point {
                x: self.x - 1,
                y: self.y - 1,
            },
            Direction::DownRight => Point {
                x: self.x + 1,
                y: self.y - 1,
            },
        }
    }
}
