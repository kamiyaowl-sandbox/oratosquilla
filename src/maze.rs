extern crate arrayvec;
pub const MAZE_WIDTH: usize = 32;
pub const MAZE_HEIGHT: usize = 32;
pub const SEARCH_INFO_STORE_SIZE: usize = MAZE_WIDTH * MAZE_HEIGHT; // 暫定値、組み込みはSRAMが貧相だぞ

/// 座標
#[derive(Copy, Clone, Debug)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}
impl Default for Point {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

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

/// 各区画単位の管理情報
#[derive(Copy, Clone, Debug)]
pub struct Cell {
    // 区画上方向の壁有無
    pub up_wall: Option<bool>,
    // 区画右方向の壁有無
    pub right_wall: Option<bool>,
    /// ここまでの到達に必要な手数
    pub cost: Option<usize>,

    // TODO: boolだとサイズがでかくて無理な場合、別クラスでビットフラグ管理で管理
    /// セルの壁情報が更新済
    pub is_updated: bool,
    /// 周辺セルを探索済
    pub is_search_around: bool,
    /// 検索対象リストに追加されたことがあればtrue
    pub is_provider_pushed: bool,
    /// 当初の探索時より少ないコストで到達できる場合フラグを立てる
    /// from_dirで逆順に戻った場合に、コストが非連続になる
    pub is_cost_dirty: bool,

    /// 逆順にたどった際の最短になっている場合はtrue
    pub is_answer: bool,
    /// Goal発見後の探索で、理想最短コストが既存のコストを上回っている場合は探索しない
    pub is_invalidated: bool,
    /// どのマスから来たか, cost_dirtyをつける際は付け替える
    /// goalからstartに戻る際に、最小コストの単方向リストとして完成しているはず
    pub from_dir: Direction,
}
impl Default for Cell {
    fn default() -> Self {
        Self {
            up_wall: None,
            right_wall: None,
            cost: None,
            is_updated: false,
            is_search_around: false,
            is_provider_pushed: false,
            is_cost_dirty: false,

            is_answer: false,
            is_invalidated: false,
            from_dir: Direction::NoDir,
        }
    }
}
impl Cell {
    /// コストがより良い方に更新します
    /// もし既存のコストより良いものが反映された場合stateが変更される
    pub fn update_cost(&mut self, new_cost: usize) {
        self.cost = Some(match self.cost {
            None => new_cost,
            Some(c) if c <= new_cost => c,
            // コストが小さいルートが発見された
            _ => {
                self.is_cost_dirty = true;
                new_cost
            }
        });
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

/// 普段はARMなのでx,y等すべてu32で扱いたいが、サイズがでかくなるのでここだけ圧縮する
#[derive(Copy, Clone, Debug)]
pub struct SearchInfo {
    x: u8,
    y: u8,
}
impl Default for SearchInfo {
    fn default() -> Self {
        Self {
            x: 0xff_u8, // dummy value
            y: 0xff_u8, // dummy value
        }
    }
}
/// Stackにして深さ優先、追加履歴が可能な限り近いところから取り出す
pub struct SearchInfoProvider {
    pub datas: [SearchInfo; SEARCH_INFO_STORE_SIZE],
    pub wr_ptr: usize,
}
impl Default for SearchInfoProvider {
    fn default() -> Self {
        Self {
            datas: [SearchInfo::default(); SEARCH_INFO_STORE_SIZE],
            wr_ptr: 0,
        }
    }
}
impl SearchInfoProvider {
    pub fn get_count(&self) -> usize {
        self.wr_ptr
    }
    pub fn get_free(&self) -> usize {
        SEARCH_INFO_STORE_SIZE - self.wr_ptr
    }
    pub fn clear(&mut self) {
        self.wr_ptr = 0;
    }
    pub fn push(&mut self, p: Point) -> bool {
        if self.wr_ptr < (SEARCH_INFO_STORE_SIZE - 1) {
            debug_assert!(p.x < 0x100);
            debug_assert!(p.y < 0x100);

            let data = SearchInfo {
                x: p.x as u8,
                y: p.y as u8,
            };
            self.datas[self.wr_ptr] = data;
            self.wr_ptr += 1;
            true
        } else {
            // 無理
            // TODO: 古すぎる探索は省略していいならoverwriteするような仕組みがあってもよい?
            false
        }
    }
    pub fn pop(&mut self) -> Option<Point> {
        if self.wr_ptr > 0 {
            self.wr_ptr -= 1; // read有効データは書き込み先のひとつ下
            let data = self.datas[self.wr_ptr];

            Some(Point {
                x: usize::from(data.x),
                y: usize::from(data.y),
            })
        } else {
            None
        }
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

/// 迷路管理の親
pub struct Maze {
    pub cells: [[Cell; MAZE_WIDTH]; MAZE_HEIGHT],
    pub start: Point,
    pub goal: Point,
    pub provider: SearchInfoProvider,
}

impl Default for Maze {
    fn default() -> Self {
        Self {
            cells: [[Cell::default(); MAZE_WIDTH]; MAZE_HEIGHT],
            start: Point { x: 0, y: 0 },
            goal: Point { x: 0, y: 0 },
            provider: SearchInfoProvider::default(),
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
        dst.cells[0][0].is_provider_pushed = true; // 0を再度探索する必要はない
        dst
    }
    /// 次に進むべき座標を取得します
    pub fn get_next(&mut self) -> Option<Point> {
        self.provider.pop()
    }
    /// 壁情報を更新する
    pub fn update(&mut self, info: &UpdateInfo) {
        debug_assert!(info.p.x < MAZE_WIDTH);
        debug_assert!(info.p.y < MAZE_HEIGHT);
        debug_assert!(!self.cells[info.p.y][info.p.x].is_updated);
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
        // 探索済セルに追加
        self.cells[info.p.y][info.p.x].is_updated = true;
    }
    /// 周辺セルを探索対象として追加します
    /// 追加する際に優先度が高い順になるようにすることでa*もどきっぽく振る舞います
    pub fn fetch_targets(&mut self, p: Point) {
        debug_assert!(self.cells[p.y][p.x].cost.is_some());
        let current_cost = self.cells[p.y][p.x].cost.unwrap() + 1;

        // 座標, cost_total
        // costとcost_totalを更新してソートして追加する
        use arrayvec::ArrayVec;
        const TARGET_NUM: usize = 8; // 8方位
        let mut targets = ArrayVec::<[(Point, Option<usize>); TARGET_NUM]>::new();

        // 探索Stackに余裕がなければ諦める
        if TARGET_NUM > self.provider.get_free() {
            return;
        }

        // 上下左右の区画に移動可能かを判定する
        let is_passing_up = p.y < MAZE_HEIGHT - 1 && self.cells[p.y][p.x].up_wall == Some(false);
        let is_passing_right =
            p.x < MAZE_WIDTH - 1 && self.cells[p.y][p.x].right_wall == Some(false);
        let is_passing_down = p.y > 0 && self.cells[p.y - 1][p.x].up_wall == Some(false);
        let is_passing_left = p.x > 0 && self.cells[p.y][p.x - 1].right_wall == Some(false);
        // 斜め方向の区画に移動可能か判定する。斜め走行前提
        // 迂回ルートは2種類あるので、どちらかを満たしていればよい
        let is_passing_up_left = (p.x > 0)
            && ((is_passing_up && self.cells[p.y + 1][p.x - 1].right_wall == Some(false))
                || (is_passing_left && self.cells[p.y][p.x - 1].up_wall == Some(false)));
        let is_passing_up_right = (is_passing_up
            && self.cells[p.y + 1][p.x].right_wall == Some(false))
            || (is_passing_right && self.cells[p.y][p.x + 1].up_wall == Some(false));

        let is_passing_down_left = (p.x > 0 && p.y > 0)
            && ((is_passing_down && self.cells[p.y - 1][p.x - 1].right_wall == Some(false))
                || (is_passing_left && self.cells[p.y - 1][p.x - 1].up_wall == Some(false)));
        let is_passing_down_right = (p.y > 0)
            && ((is_passing_down && self.cells[p.y - 1][p.x].right_wall == Some(false))
                || (is_passing_right && self.cells[p.y - 1][p.x + 1].up_wall == Some(false)));

        if is_passing_up {
            targets.push((p.get_around(Direction::Up), None));
        }
        if is_passing_right {
            targets.push((p.get_around(Direction::Right), None));
        }
        if is_passing_down {
            targets.push((p.get_around(Direction::Down), None));
        }
        if is_passing_left {
            targets.push((p.get_around(Direction::Left), None));
        }
        if is_passing_up_left {
            targets.push((p.get_around(Direction::UpLeft), None));
        }
        if is_passing_up_right {
            targets.push((p.get_around(Direction::UpRight), None));
        }
        if is_passing_down_left {
            targets.push((p.get_around(Direction::DownLeft), None));
        }
        if is_passing_down_right {
            targets.push((p.get_around(Direction::DownRight), None));
        }
        for (target_point, target_cost) in &mut targets {
            // コスト更新
            self.cells[target_point.y][target_point.x].update_cost(current_cost);
            // 検索予約に追加
            if !self.cells[target_point.y][target_point.x].is_search_around
                && !self.cells[target_point.y][target_point.x].is_provider_pushed
            {
                self.cells[target_point.y][target_point.x].is_provider_pushed = true;

                *target_cost = Some(
                    self.cells[target_point.y][target_point.x].cost.unwrap()
                        + self.goal.distance(*target_point),
                );
            }
        }

        // コストの大きい順に追加する
        targets.sort_by_key(|&(_point, cost)| {
            if let Some(c) = cost {
                c
            } else {
                std::usize::MAX
            }
        });
        targets.reverse();
        for (target_point, target_cost) in &targets {
            if let Some(_) = target_cost {
                self.provider.push(*target_point);
            }
        }

        // 周辺探索完了フラグ
        self.cells[p.y][p.x].is_search_around = true;
    }

    /// 現在の迷路情報を出力
    /// TODO: no_stdでの関数削除、というかもっとリッチにしろ
    pub fn debug_print(&self, filename: &str, header: &str) -> Result<(), std::io::Error> {
        const CELL_WIDTH: usize = 6;
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
                        1 if self.cells[MAZE_HEIGHT - 1 - j][i].cost.is_some() => write!(
                            out,
                            " {:>4} ",
                            self.cells[MAZE_HEIGHT - 1 - j][i].cost.unwrap()
                        )?,
                        2 if self.start.x == i && self.start.y == (MAZE_HEIGHT - 1 - j) => {
                            write!(out, " *SS* ")?
                        }
                        2 if self.goal.x == i && self.goal.y == (MAZE_HEIGHT - 1 - j) => {
                            write!(out, " *GG* ")?
                        }
                        2 => write!(
                            out,
                            " {}{}{}{} ",
                            if self.cells[MAZE_HEIGHT - 1 - j][i].is_updated {
                                "U"
                            } else {
                                " "
                            },
                            if self.cells[MAZE_HEIGHT - 1 - j][i].is_search_around {
                                "S"
                            } else {
                                " "
                            },
                            if self.cells[MAZE_HEIGHT - 1 - j][i].is_provider_pushed {
                                "P"
                            } else {
                                " "
                            },
                            if self.cells[MAZE_HEIGHT - 1 - j][i].is_cost_dirty {
                                "D"
                            } else {
                                " "
                            }
                        )?,
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
        writeln!(out, "\n\n\n")?;
        out.flush()?;

        Ok(())
    }
}
