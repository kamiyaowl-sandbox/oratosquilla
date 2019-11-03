mod maze {
    pub const MAZE_WIDTH: usize = 32;
    pub const MAZE_HEIGHT: usize = 32;

    #[derive(Copy, Clone, Debug)]
    pub struct Point {
        pub x: usize,
        pub y: usize,
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
    }

    impl Default for Cell {
        fn default() -> Self {
            Self {
                up_wall: None,
                right_wall: None,
                cost: None,
            }
        }
    }

    /// 実機から迷路情報の更新に使う情報
    #[derive(Debug)]
    pub struct UpdateInfo {
        pub p: Point,
        pub up: Option<bool>,
        pub down: Option<bool>,
        pub left: Option<bool>,
        pub right: Option<bool>,
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
        pub previous: Option<Point>,
        pub start: Point,
        pub goal: Point,
    }

    impl Default for Maze {
        fn default() -> Self {
            Self {
                cells: [[Cell::default(); MAZE_WIDTH]; MAZE_HEIGHT],
                previous: None,
                start: Point { x: 0, y: 0 },
                goal: Point { x: 0, y: 0 },
            }
        }
    }

    impl Maze {
        pub fn init(&mut self, goal: Point) {
            self.cells = [[Cell::default(); MAZE_WIDTH]; MAZE_HEIGHT];
            self.start = Point { x: 0, y: 0 };
            self.goal = goal;
            // 上端、右端の壁初期化
            for j in 0..MAZE_HEIGHT {
                self.cells[j][MAZE_WIDTH - 1].right_wall = Some(true);
            }
            for i in 0..MAZE_WIDTH {
                self.cells[MAZE_HEIGHT - 1][i].up_wall = Some(true);
            }
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
            // 最終更新履歴を残す
            self.previous = Some(info.p);
        }

        /// 現在の迷路情報を出力
        /// TODO: no_stdでの関数削除
        pub fn debug_print(&self) {
            let cellWidth = 4;
            let cellHeight = 2;
            let unknownStr = "?";
            let noWallStr = " ";
            let wallStr = "+";
            let intersectStr = ".";

            for j in 0..MAZE_HEIGHT {//printのy方向と反転しているので注意
                // とりあえず1行書く
                for i in 0..MAZE_WIDTH {
                    print!("{}", intersectStr);
                    // 水平壁
                    let c = match self.cells[MAZE_HEIGHT - 1 - j][i].up_wall {
                        Some(true) => wallStr,
                        Some(false) => noWallStr,
                        None => unknownStr,
                    };
                    for _ in 0..cellWidth {
                        print!("{}", c);
                    }
                }
                println!("{}", intersectStr);
                // 残りの行
                for _ in 0..cellHeight {
                    print!("{}", wallStr); // 左端
                    for i in 0..MAZE_WIDTH {
                        // 空間
                        for _ in 0..cellWidth {
                            print!("{}", noWallStr); // todo:cost表示
                        }
                        // 垂直壁
                        let c = match self.cells[MAZE_HEIGHT - 1 - j][i].right_wall {
                            Some(true) => wallStr,
                            Some(false) => noWallStr,
                            None => unknownStr,
                        };
                        print!("{}", c);
                    }
                    println!("");
                }
            }
            // 一番下
            for _i in 0..MAZE_WIDTH {
                print!("{}", intersectStr);
                for _ in 0..cellWidth {
                    print!("{}", wallStr);
                }
            }
            println!("");

        }
    }
}

fn main() {
    use maze::*;
    let mut m = Maze::default();
    m.init(Point{x: 10, y:10});
    m.debug_print();
}
