use super::direction::Direction;
use super::search_info::SearchInfo;

bitflags! {
    #[derive(Default)]
    pub struct CellFlag: u32 {
        const NO_FLAG = 0x00_00_00_00;
        /// セルの壁情報が更新済
        const IS_UPDATED = 0x00_00_00_01;
        /// 周辺セルを探索済
        const IS_SEARCH_AROUND = 0x00_00_00_02;
        /// 検索対象リストに追加されたことがあればtrue
        const IS_PROVIDER_PUSHED = 0x00_00_00_04;
        /// コストに有効な値をセットしたことがあればtrue
        const IS_COST_AVAILABLE = 0x00_00_00_08;
        /// 当初の探索時より少ないコストで到達できる場合フラグを立てる
        /// from_dirで逆順に戻った場合に、コストが非連続になる
        const IS_COST_DIRTY = 0x00_00_00_10;
        /// 逆順にたどった際の最短になっている場合はtrue
        const IS_ANSWER = 0x00_00_00_20;
        /// Goal発見後の探索で、理想最短コストが既存のコストを上回っている場合は探索しない
        const IS_INVALIDATED = 0x00_00_00_40;


        /// 右方向の壁が存在する
        const IS_EXISTS_RIGHT_WALL = 0x10_00_00_00;
        /// 右方向の壁にUpdateをかけたことがある
        const IS_UPDATED_RIGHT_WALL = 0x20_00_00_00;
        /// 上方向の壁が存在する
        const IS_EXISTS_UP_WALL = 0x40_00_00_00;
        /// 上方向の壁にUpdateをかけたことがある
        const IS_UPDATED_UP_WALL = 0x80_00_00_00;
    }
}
/// 各区画単位の管理情報
/// Optionはもともとの実態容量の倍になっていそうなので注意
#[derive(Copy, Clone, Debug)]
pub struct Cell {
    /// ここまでの到達に必要な手数(真値)
    /// IS_COST_AVAILABLEフラグを確認してから使う
    pub cost: usize,
    /// どのマスから来たか, cost_dirtyをつける際は付け替える
    /// goalからstartに戻る際に、最小コストの単方向リストとして完成しているはず
    pub from_info: SearchInfo,
    /// ステータスフラグ色々
    pub flag: CellFlag,
}
impl Default for Cell {
    fn default() -> Self {
        Self {
            cost: std::usize::MAX,
            from_info: SearchInfo::default(),
            flag: CellFlag::NO_FLAG,
        }
    }
}
impl Cell {
    /// コストがより良い方に更新します
    /// もし既存のコストより良いものが反映された場合stateが変更される
    /// 新しい値が代入されたらtrueが変える
    pub fn update_cost(&mut self, new_cost: usize, from_info: SearchInfo) {
        self.cost = if self.flag.contains(CellFlag::IS_COST_AVAILABLE) {
            if self.cost <= new_cost {
                self.cost
            } else {
                // より小さいコストでいけるのでフラグを立てておく
                self.flag.insert(CellFlag::IS_COST_DIRTY);
                self.from_info = from_info;
                new_cost
            }
        } else {
            self.from_info = from_info;
            new_cost
        };
        self.flag.insert(CellFlag::IS_COST_AVAILABLE);
    }
}
