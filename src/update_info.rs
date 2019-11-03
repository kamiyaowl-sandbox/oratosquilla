use super::cell::*;
use super::explorer::*;
use super::point::Point;

/// 実機から迷路情報の更新に使う情報
#[derive(Debug)]
pub struct UpdateInfo {
    /// 更新対象の区画
    pub p: Point,
    pub up: Option<bool>,
    pub down: Option<bool>,
    pub left: Option<bool>,
    pub right: Option<bool>,
}
impl Default for UpdateInfo {
    fn default() -> Self {
        Self {
            p: Point::default(),
            up: None,
            down: None,
            left: None,
            right: None,
        }
    }
}

impl Explorer {
    /// 壁情報を更新する
    pub fn update(&mut self, info: &UpdateInfo) {
        debug_assert!(info.p.x < MAZE_WIDTH);
        debug_assert!(info.p.y < MAZE_HEIGHT);
        debug_assert!(!self.cells[info.p.y][info.p.x]
            .flag
            .contains(CellFlag::IS_UPDATED));
        // 壁情報の更新
        if let Some(up_wall) = info.up {
            self.cells[info.p.y][info.p.x]
                .flag
                .insert(CellFlag::IS_UPDATED_UP_WALL);
            if up_wall {
                self.cells[info.p.y][info.p.x]
                    .flag
                    .insert(CellFlag::IS_EXISTS_UP_WALL);
            } else {
                self.cells[info.p.y][info.p.x]
                    .flag
                    .remove(CellFlag::IS_EXISTS_UP_WALL);
            }
        }
        if let Some(right_wall) = info.right {
            self.cells[info.p.y][info.p.x]
                .flag
                .insert(CellFlag::IS_UPDATED_RIGHT_WALL);
            if right_wall {
                self.cells[info.p.y][info.p.x]
                    .flag
                    .insert(CellFlag::IS_EXISTS_RIGHT_WALL);
            } else {
                self.cells[info.p.y][info.p.x]
                    .flag
                    .remove(CellFlag::IS_EXISTS_RIGHT_WALL);
            }
        }
        // 下、左は隣のセル情報に格納されている
        if info.p.y > 0 {
            if let Some(down_wall) = info.down {
                self.cells[info.p.y - 1][info.p.x]
                    .flag
                    .insert(CellFlag::IS_UPDATED_UP_WALL);
                if down_wall {
                    self.cells[info.p.y - 1][info.p.x]
                        .flag
                        .insert(CellFlag::IS_EXISTS_UP_WALL);
                } else {
                    self.cells[info.p.y - 1][info.p.x]
                        .flag
                        .remove(CellFlag::IS_EXISTS_UP_WALL);
                }
            }
        }
        if info.p.x > 0 {
            if let Some(left_wall) = info.left {
                self.cells[info.p.y][info.p.x - 1]
                    .flag
                    .insert(CellFlag::IS_UPDATED_RIGHT_WALL);
                if left_wall {
                    self.cells[info.p.y][info.p.x - 1]
                        .flag
                        .insert(CellFlag::IS_EXISTS_RIGHT_WALL);
                } else {
                    self.cells[info.p.y][info.p.x - 1]
                        .flag
                        .remove(CellFlag::IS_EXISTS_RIGHT_WALL);
                }
            }
        }
        // 探索済セルに追加
        self.cells[info.p.y][info.p.x]
            .flag
            .insert(CellFlag::IS_UPDATED);
    }
}
