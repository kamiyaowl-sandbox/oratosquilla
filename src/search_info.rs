use super::point::Point;
use super::explorer::*;
use super::cell::*;
use super::direction::Direction;

pub const SEARCH_INFO_STORE_SIZE: usize = MAZE_WIDTH * MAZE_HEIGHT; // 暫定値、組み込みはSRAMが貧相だぞ

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
            // 無理だけど普通にfalse返すだけで良さげ
            debug_assert!(false);
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
impl Explorer {
    /// 次に進むべき座標を取得します
    pub fn get_next(&mut self) -> Option<Point> {
        self.provider.pop()
    }

    /// 周辺セルを探索対象として追加します
    /// 追加する際に優先度が高い順になるようにすることでa*もどきっぽく振る舞います
    pub fn fetch_targets(&mut self, p: Point) {
        debug_assert!(self.cells[p.y][p.x]
            .flag
            .contains(CellFlag::IS_COST_AVAILABLE));
        let current_cost = self.cells[p.y][p.x].cost + 1;

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
        let is_passing_up = p.y < MAZE_HEIGHT - 1
            && self.cells[p.y][p.x]
                .flag
                .contains(CellFlag::IS_UPDATED_UP_WALL)
            && !self.cells[p.y][p.x]
                .flag
                .contains(CellFlag::IS_EXISTS_UP_WALL);
        let is_passing_right = p.x < MAZE_WIDTH - 1
            && self.cells[p.y][p.x]
                .flag
                .contains(CellFlag::IS_UPDATED_RIGHT_WALL)
            && !self.cells[p.y][p.x]
                .flag
                .contains(CellFlag::IS_EXISTS_RIGHT_WALL);

        let is_passing_down = p.y > 0
            && self.cells[p.y - 1][p.x]
                .flag
                .contains(CellFlag::IS_UPDATED_UP_WALL)
            && !self.cells[p.y - 1][p.x]
                .flag
                .contains(CellFlag::IS_EXISTS_UP_WALL);
        let is_passing_left = p.x > 0
            && self.cells[p.y][p.x - 1]
                .flag
                .contains(CellFlag::IS_UPDATED_RIGHT_WALL)
            && !self.cells[p.y][p.x - 1]
                .flag
                .contains(CellFlag::IS_EXISTS_RIGHT_WALL);

        // 斜め方向の区画に移動可能か判定する。斜め走行前提
        // 迂回ルートは2種類あるので、どちらかを満たしていればよい
        let is_passing_up_left = (p.x > 0)
            && ((is_passing_up
                && self.cells[p.y + 1][p.x - 1]
                    .flag
                    .contains(CellFlag::IS_UPDATED_RIGHT_WALL)
                && !self.cells[p.y + 1][p.x - 1]
                    .flag
                    .contains(CellFlag::IS_EXISTS_RIGHT_WALL))
                || (is_passing_left
                    && self.cells[p.y][p.x - 1]
                        .flag
                        .contains(CellFlag::IS_UPDATED_UP_WALL)
                    && !self.cells[p.y][p.x - 1]
                        .flag
                        .contains(CellFlag::IS_EXISTS_UP_WALL)));

        let is_passing_up_right = (is_passing_up
            && self.cells[p.y + 1][p.x]
                .flag
                .contains(CellFlag::IS_UPDATED_RIGHT_WALL)
            && !self.cells[p.y + 1][p.x]
                .flag
                .contains(CellFlag::IS_EXISTS_RIGHT_WALL))
            || (is_passing_right
                && self.cells[p.y][p.x + 1]
                    .flag
                    .contains(CellFlag::IS_UPDATED_UP_WALL)
                && !self.cells[p.y][p.x + 1]
                    .flag
                    .contains(CellFlag::IS_EXISTS_UP_WALL));

        let is_passing_down_left = (p.x > 0 && p.y > 0)
            && ((is_passing_down
                && self.cells[p.y - 1][p.x - 1]
                    .flag
                    .contains(CellFlag::IS_UPDATED_RIGHT_WALL)
                && !self.cells[p.y - 1][p.x - 1]
                    .flag
                    .contains(CellFlag::IS_EXISTS_RIGHT_WALL))
                || (is_passing_left
                    && self.cells[p.y - 1][p.x - 1]
                        .flag
                        .contains(CellFlag::IS_UPDATED_UP_WALL)
                    && !self.cells[p.y - 1][p.x - 1]
                        .flag
                        .contains(CellFlag::IS_EXISTS_UP_WALL)));

        let is_passing_down_right = (p.y > 0)
            && ((is_passing_down
                && self.cells[p.y - 1][p.x]
                    .flag
                    .contains(CellFlag::IS_UPDATED_RIGHT_WALL)
                && !self.cells[p.y - 1][p.x]
                    .flag
                    .contains(CellFlag::IS_EXISTS_RIGHT_WALL))
                || (is_passing_right
                    && self.cells[p.y - 1][p.x + 1]
                        .flag
                        .contains(CellFlag::IS_UPDATED_UP_WALL)
                    && !self.cells[p.y - 1][p.x + 1]
                        .flag
                        .contains(CellFlag::IS_EXISTS_UP_WALL)));

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
            if !self.cells[target_point.y][target_point.x]
                .flag
                .contains(CellFlag::IS_SEARCH_AROUND)
                && !self.cells[target_point.y][target_point.x]
                    .flag
                    .contains(CellFlag::IS_PROVIDER_PUSHED)
            {
                self.cells[target_point.y][target_point.x]
                    .flag
                    .insert(CellFlag::IS_PROVIDER_PUSHED);

                *target_cost = Some(
                    self.cells[target_point.y][target_point.x].cost
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
        self.cells[p.y][p.x].flag.insert(CellFlag::IS_SEARCH_AROUND);
    }
}
