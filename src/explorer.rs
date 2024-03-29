use super::cell::*;
use super::point::Point;
use super::search_info::*;

pub const MAZE_WIDTH: usize = 32;
pub const MAZE_HEIGHT: usize = 32;

/// 迷路管理の親
pub struct Explorer {
    /// 開始位置
    pub start: Point,
    /// ゴール位置
    pub goal: Point,
    /// 各マスごとの情報
    pub cells: [[Cell; MAZE_WIDTH]; MAZE_HEIGHT],
    /// 最短経路探索先供給
    pub provider: SearchInfoProvider,
    /// 現在の最小コスト、ゴールするまではNone
    pub min_cost: Option<usize>,
}

impl Default for Explorer {
    fn default() -> Self {
        Self {
            cells: [[Cell::default(); MAZE_WIDTH]; MAZE_HEIGHT],
            start: Point { x: 0, y: 0 },
            goal: Point { x: 0, y: 0 },
            provider: SearchInfoProvider::default(),
            min_cost: None,
        }
    }
}

impl Explorer {
    pub fn new(goal: Point) -> Explorer {
        let mut dst = Explorer::default();

        dst.cells = [[Cell::default(); MAZE_WIDTH]; MAZE_HEIGHT];
        dst.start = Point { x: 0, y: 0 };
        dst.goal = goal;
        // 上端、右端の壁初期化
        for j in 0..MAZE_HEIGHT {
            dst.cells[j][MAZE_WIDTH - 1]
                .flag
                .insert(CellFlag::IS_EXISTS_RIGHT_WALL | CellFlag::IS_UPDATED_RIGHT_WALL);
        }
        for i in 0..MAZE_WIDTH {
            dst.cells[MAZE_HEIGHT - 1][i]
                .flag
                .insert(CellFlag::IS_EXISTS_UP_WALL | CellFlag::IS_UPDATED_UP_WALL);
        }
        // 有効コスト設定と検索対象外設定
        dst.cells[0][0].cost = 0;
        dst.cells[0][0]
            .flag
            .insert(CellFlag::IS_COST_AVAILABLE | CellFlag::IS_PROVIDER_PUSHED);
        dst
    }

    /// 現在の迷路情報を出力
    /// TODO: no_stdでの関数削除、というかもっとリッチにしろ
    pub fn debug_print(&self, filename: &str, header: &str) -> Result<(), std::io::Error> {
        const CELL_WIDTH: usize = 7;
        const CELL_HEIGHT: usize = 3;
        const UNKNOWN_STR: &str = "?";
        const NO_WALL_STR: &str = " ";
        const WALL_STR: &str = "+";
        const INTERSECT_STR: &str = ".";

        use std::fs::OpenOptions;
        use std::io::prelude::*;
        use std::io::BufWriter;
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(filename.to_string())?;
        let mut out = BufWriter::new(file);

        // おしゃれなヘッダ
        for _ in 0..(CELL_HEIGHT + 1) * MAZE_WIDTH {
            write!(out, "=")?;
        }
        writeln!(out, "\n{}", header.to_string())?;
        for _ in 0..(CELL_HEIGHT + 1) * MAZE_WIDTH {
            write!(out, "=")?;
        }
        writeln!(out, "")?;

        for j in 0..MAZE_HEIGHT {
            //printのy方向と反転しているので注意
            // とりあえず1行書く
            for i in 0..MAZE_WIDTH {
                write!(out, "{}", INTERSECT_STR)?;
                // 水平壁
                let c = if self.cells[MAZE_HEIGHT - 1 - j][i]
                    .flag
                    .contains(CellFlag::IS_UPDATED_UP_WALL)
                {
                    if self.cells[MAZE_HEIGHT - 1 - j][i]
                        .flag
                        .contains(CellFlag::IS_EXISTS_UP_WALL)
                    {
                        WALL_STR
                    } else {
                        NO_WALL_STR
                    }
                } else {
                    UNKNOWN_STR
                };
                for _ in 0..CELL_WIDTH {
                    write!(out, "{}", c)?;
                }
            }
            writeln!(out, "{}", INTERSECT_STR)?;
            // 残りの行
            for local_j in 0..CELL_HEIGHT {
                write!(out, "{}", WALL_STR)?; // 左端
                for i in 0..MAZE_WIDTH {
                    // 壁間の空間
                    match local_j {
                        0 if self.cells[MAZE_HEIGHT - 1 - j][i]
                            .flag
                            .contains(CellFlag::IS_COST_AVAILABLE) =>
                        {
                            write!(out, "  {:>4} ", self.cells[MAZE_HEIGHT - 1 - j][i].cost)?;
                        }
                        1 if self.cells[MAZE_HEIGHT - 1 - j][i]
                            .flag
                            .contains(CellFlag::IS_PROVIDER_PUSHED)
                            && self.cells[MAZE_HEIGHT - 1 - j][i].from_info.x < (MAZE_WIDTH as u8) =>
                        {
                            write!(
                                out,
                                "({:>2},{:>2})",
                                self.cells[MAZE_HEIGHT - 1 - j][i].from_info.x,
                                self.cells[MAZE_HEIGHT - 1 - j][i].from_info.y
                            )?;
                        }
                        2 => {
                            let f = self.cells[MAZE_HEIGHT - 1 - j][i].flag;
                            write!(
                                out,
                                " {}{}{}{}{}{}",
                                if f.contains(CellFlag::IS_ANSWER) {
                                    "A"
                                } else {
                                    " "
                                },
                                if f.contains(CellFlag::IS_INVALIDATED) {
                                    "I"
                                } else {
                                    " "
                                },
                                if f.contains(CellFlag::IS_COST_DIRTY) {
                                    "D"
                                } else {
                                    " "
                                },
                                if f.contains(CellFlag::IS_PROVIDER_PUSHED) {
                                    "P"
                                } else {
                                    " "
                                },
                                if f.contains(CellFlag::IS_SEARCH_AROUND) {
                                    "S"
                                } else {
                                    " "
                                },
                                if f.contains(CellFlag::IS_UPDATED) {
                                    "U"
                                } else {
                                    " "
                                },
                            )?;
                        }
                        _ => {
                            for _ in 0..CELL_WIDTH {
                                write!(out, "{}", NO_WALL_STR)?;
                            }
                        }
                    }
                    // 垂直壁
                    let c = if self.cells[MAZE_HEIGHT - 1 - j][i]
                        .flag
                        .contains(CellFlag::IS_UPDATED_RIGHT_WALL)
                    {
                        if self.cells[MAZE_HEIGHT - 1 - j][i]
                            .flag
                            .contains(CellFlag::IS_EXISTS_RIGHT_WALL)
                        {
                            WALL_STR
                        } else {
                            NO_WALL_STR
                        }
                    } else {
                        UNKNOWN_STR
                    };
                    write!(out, "{}", c)?;
                }
                writeln!(out, "")?;
            }
        }
        // 一番下
        for _i in 0..MAZE_WIDTH {
            write!(out, "{}", INTERSECT_STR)?;
            for _ in 0..CELL_WIDTH {
                write!(out, "{}", WALL_STR)?;
            }
        }
        writeln!(out, "\n\n\n")?;
        out.flush()?;

        Ok(())
    }
}
