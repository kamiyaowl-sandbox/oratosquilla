mod maze {
    pub const MAZE_WIDTH: usize = 32;
    pub const MAZE_HEIGHT: usize = 32;
    pub const SEARCH_INFO_STORE_SIZE: usize = 64; // 暫定値、組み込みはSRAMが貧相だぞ

    /// 座標
    #[derive(Copy, Clone, Debug)]
    pub struct Point {
        pub x: usize,
        pub y: usize,
    }
    impl Default for Point {
        fn default() -> Self {
            Self {
                x: 0,
                y: 0,
            }
        }
    }

    /// セルの検索状態
    #[derive(Copy,Clone,Debug)]
    pub enum SearchState {
        /// 初期状態
        Init, 
        /// センサ情報更新済
        Updated,
        /// 周りのセルをSearchInfoProviderに追加済
        /// 探索状態としては最終
        AroundSearchReserved,
        /// 最短経路になっている
        /// 探索後に指定
        Answer,
    }

    /// 各区画単位の管理情報
    #[derive(Copy, Clone, Debug)]
    pub struct Cell {
        // 区画上方向の壁有無
        pub up_wall: Option<bool>,
        // 区画右方向の壁有無
        pub right_wall: Option<bool>,
        /// ここまでの到達に必要な手数
        pub cost: Option<usize>,
        /// セルの探索状態
        pub state: SearchState,
    }

    impl Default for Cell {
        fn default() -> Self {
            Self {
                up_wall: None,
                right_wall: None,
                cost: None,
                state: SearchState::Init,
            }
        }
    }

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

    /// 探索対象のリスト
    #[derive(Copy, Clone, Debug)]
    pub struct SearchInfo {
        /// 探索元座標、コスト計算に使う
        pub from: Point,
        /// 探索先座標
        pub to: Point,
    }

    impl Default for SearchInfo {
        fn default() -> Self {
            Self {
                from: Point::default(),
                to: Point::default(),
            }
        }
    }

    /// Stackにして深さ優先、追加履歴が可能な限り近いところから取り出す
    pub struct SearchInfoProvider {
        pub infos: [SearchInfo; SEARCH_INFO_STORE_SIZE],
    }
    impl Default for SearchInfoProvider {
        fn default() -> Self {
            Self {
                infos: [SearchInfo::default(); SEARCH_INFO_STORE_SIZE],
            }
        }
    }



    use std::ops::Sub;
    impl Sub for Point {
        type Output = usize;
        /// ビルドオプション指定がなければ、チェビシフ距離を返します
        fn sub(self, other: Point) -> usize {
            use std::cmp;

            let dx = cmp::max(self.x, other.x) - cmp::min(self.x, other.x);
            let dy = cmp::max(self.y, other.y) - cmp::min(self.y, other.y);

            if cfg!(distance_method = "manhattan") {
                dx + dy
            } else {
                cmp::max(dx, dy)
            }
        }
    }

    /// 迷路管理の親
    #[derive(Debug)]
    pub struct Maze {
        pub cells: [[Cell; MAZE_WIDTH]; MAZE_HEIGHT],
        pub start: Point,
        pub goal: Point,
    }

    impl Default for Maze {
        fn default() -> Self {
            Self {
                cells: [[Cell::default(); MAZE_WIDTH]; MAZE_HEIGHT],
                start: Point { x: 0, y: 0 },
                goal: Point { x: 0, y: 0 },
            }
        }
    }

    impl Maze {
        pub fn new(goal: Point) -> Maze {
            let mut dst = Maze::default();

            dst.cells = [[Cell::default(); MAZE_WIDTH]; MAZE_HEIGHT];
            dst.start = Point { x: 0, y: 0 };
            dst.goal = goal;
            // 上端、右端の壁初期化
            for j in 0..MAZE_HEIGHT {
                dst.cells[j][MAZE_WIDTH - 1].right_wall = Some(true);
            }
            for i in 0..MAZE_WIDTH {
                dst.cells[MAZE_HEIGHT - 1][i].up_wall = Some(true);
            }
            dst.cells[0][0].cost = Some(0);
            dst
        }
        /// 壁情報を更新する
        pub fn update(&mut self, info: &UpdateInfo) {
            debug_assert!(info.p.x < MAZE_WIDTH);
            debug_assert!(info.p.y < MAZE_HEIGHT);
            // 壁情報の更新
            if let Some(up_wall) = info.up {
                self.cells[info.p.y][info.p.x].up_wall = Some(up_wall);
            }
            if let Some(right_wall) = info.right {
                self.cells[info.p.y][info.p.x].right_wall = Some(right_wall);
            }
            // 下、左は隣のセル情報に格納されている
            if info.p.y > 0 {
                if let Some(down_wall) = info.down {
                    self.cells[info.p.y - 1][info.p.x].up_wall = Some(down_wall);
                }
            }
            if info.p.x > 0 {
                if let Some(left_wall) = info.left {
                    self.cells[info.p.y][info.p.x - 1].right_wall = Some(left_wall);
                }
            }
        }

        /// 現在の迷路情報を出力
        /// TODO: no_stdでの関数削除、というかもっとリッチにしろ
        pub fn debug_print(&self) -> Result<(), std::io::Error> {
            const CELL_WIDTH: usize = 4;
            const CELL_HEIGHT: usize = 2;
            const UNKNOWN_STR: &str = "?";
            const NO_WALL_STR: &str  = " ";
            const WALL_STR: &str  = "+";
            const INTERSECT_STR: &str  = ".";

            // stdoutをロックしてまとめて書く
            use std::io::{stdout, Write, BufWriter};
            let out = stdout();
            let mut out = BufWriter::new(out.lock());

            for j in 0..MAZE_HEIGHT {//printのy方向と反転しているので注意
                // とりあえず1行書く
                for i in 0..MAZE_WIDTH {
                    write!(out, "{}", INTERSECT_STR)?;
                    // 水平壁
                    let c = match self.cells[MAZE_HEIGHT - 1 - j][i].up_wall {
                        Some(true) => WALL_STR,
                        Some(false) => NO_WALL_STR,
                        None => UNKNOWN_STR,
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
                            0 if self.start.x == i && self.start.y == (MAZE_HEIGHT - 1 - j) => write!(out, " SS ")?,
                            0 if self.goal.x == i && self.goal.y == (MAZE_HEIGHT - 1 - j) => write!(out, " GG ")?,
                            1 if self.cells[MAZE_HEIGHT - 1 - j][i].cost.is_some() => write!(out, "{:>4}", self.cells[MAZE_HEIGHT - 1 - j][i].cost.unwrap())?,
                            _ => {
                                for _ in 0..CELL_WIDTH {
                                    write!(out, "{}", NO_WALL_STR)?;
                                }
                            }
                        }
                        // 垂直壁
                        let c = match self.cells[MAZE_HEIGHT - 1 - j][i].right_wall {
                            Some(true) => WALL_STR,
                            Some(false) => NO_WALL_STR,
                            None => UNKNOWN_STR,
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
            writeln!(out, "")?;

            Ok(())
        }
    }
}

fn main() {
    use maze::*;

    let mut m = Maze::new(Point{x: 10, y:10});
    let info = UpdateInfo::default();
    m.update(&info);
    m.debug_print();
}
