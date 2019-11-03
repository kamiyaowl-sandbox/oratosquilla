extern crate oratosquilla;

// #[cfg(test)]
mod test {
    use oratosquilla::prelude::*;

    /// 最初の地点から一歩進むか
    #[test]
    pub fn ahead_start() {
        let mut m = Explorer::new(Point { x: 10, y: 10 });

        let p = Point { x: 0, y: 0 };
        let mut info = UpdateInfo::default();
        info.p = p;
        info.up = Some(false);
        info.down = None;
        info.left = Some(true);
        info.right = Some(true);
        m.update(&info);
        m.fetch_targets(p);
        let next_p = m.get_next();
        m.debug_print("test.log", "ahead_start").unwrap();

        assert_eq!(next_p.is_some(), true);
        assert_eq!(next_p.unwrap().x, 0);
        assert_eq!(next_p.unwrap().y, 1);
    }

    /// 最初の地点で2マス更新をかけた際に、斜めマスを最短とするか
    /// .++++.++++.????
    /// +         +    
    /// +   1    1+    
    /// .    .++++.????
    /// + SS +    ?    
    /// +   0+    ?    
    /// .++++.++++.++++
    #[test]
    pub fn diagonal_start() {
        let mut m = Explorer::new(Point { x: 10, y: 10 });

        let p = Point { x: 0, y: 0 };
        let mut info = UpdateInfo::default();
        info.p = p;
        info.up = Some(false);
        info.down = None;
        info.left = Some(true);
        info.right = Some(true);
        m.update(&info);

        let mut info = UpdateInfo::default();
        info.p = Point { x: 0, y: 1 };
        info.up = Some(true);
        info.down = None;
        info.left = Some(true);
        info.right = Some(false);
        m.update(&info);

        m.fetch_targets(p);
        let next_p = m.get_next();
        m.debug_print("test.log", "diagonal_start").unwrap();

        assert_eq!(next_p.is_some(), true);
        assert_eq!(next_p.unwrap().x, 1);
        assert_eq!(next_p.unwrap().y, 1);
    }

    /// 最初の地点で2マス更新をかけた際に、斜めマスを最短としてさらに探索を続けるか
    ///        ↓ここ
    /// .????.????.????.????.
    /// +    ?    ?    ?    ?
    /// +    ?   2?    ?    ?
    /// .++++.    .????.????.
    /// +              ?    ?
    /// +   1    1    2?    ?
    /// .    .    .????.????.
    /// + SS +    ?    ?    ?
    /// +   0+   2?    ?    ?
    /// .++++.++++.++++.++++.
    #[test]
    pub fn diagonal_start_2() {
        let mut m = Explorer::new(Point { x: 10, y: 10 });

        let p = Point { x: 0, y: 0 };
        let mut info = UpdateInfo::default();
        info.p = p;
        info.up = Some(false);
        info.down = None;
        info.left = Some(true);
        info.right = Some(true);
        m.update(&info);

        let mut info = UpdateInfo::default();
        info.p = Point { x: 0, y: 1 };
        info.up = Some(true);
        info.down = None;
        info.left = Some(true);
        info.right = Some(false);
        m.update(&info);

        m.fetch_targets(p);
        let next_p = m.get_next().unwrap();
        info.p = next_p;
        info.up = Some(false);
        info.down = Some(false);
        info.left = None;
        info.right = Some(false);
        m.update(&info);
        m.fetch_targets(next_p);

        let last_p = m.get_next();
        m.debug_print("test.log", "diagonal_start_2").unwrap();

        assert_eq!(last_p.is_some(), true);
        // 上が最短になるはず
        assert_eq!(last_p.unwrap().x, 1);
        assert_eq!(last_p.unwrap().y, 2);
    }

    /// ゴールしてかつ他に探索可能区間がない場合に探索が止まるか
    #[test]
    pub fn goal_and_stop() {
        let mut m = Explorer::new(Point { x: 0, y: 1 });

        let p = Point { x: 0, y: 0 };
        let mut info = UpdateInfo::default();
        info.p = p;
        info.up = Some(false);
        info.down = None;
        info.left = Some(true);
        info.right = Some(true);
        m.update(&info);
        m.fetch_targets(p);

        let next_p = m.get_next().unwrap();
        info.p = next_p;
        info.up = Some(true);
        info.down = None;
        info.left = Some(true);
        info.right = Some(true);
        m.update(&info);
        m.fetch_targets(next_p);

        let last_p = m.get_next();
        m.debug_print("test.log", "goal_and_stop").unwrap();

        assert_eq!(last_p.is_some(), false);
    }

    /// 一切壁がなく右端がゴールの場合に最短距離ですすめるか
    #[test]
    pub fn no_wall_move_x() {
        let mut m = Explorer::new(Point {
            x: MAZE_WIDTH - 1,
            y: 0,
        });

        let mut p = Point { x: 0, y: 0 };
        for _i in 0..MAZE_WIDTH - 1 {
            let mut info = UpdateInfo::default();
            info.p = p;
            info.up = Some(false);
            info.down = Some(false);
            info.left = Some(false);
            info.right = Some(false);
            m.update(&info);
            m.fetch_targets(p);
            p = m.get_next().unwrap();
            // m.debug_print("no_wall_move_x.log", "index").unwrap();
        }
        m.debug_print("test.log", "no_wall_move_x").unwrap();

        assert_eq!(p.x, MAZE_WIDTH - 1);
        assert_eq!(p.y, 0);
    }

    /// 一切壁がなく上端がゴールの場合に最短距離ですすめるか
    #[test]
    pub fn no_wall_move_y() {
        let mut m = Explorer::new(Point {
            x: 0,
            y: MAZE_HEIGHT - 1,
        });

        let mut p = Point { x: 0, y: 0 };
        for _i in 0..MAZE_HEIGHT - 1 {
            let mut info = UpdateInfo::default();
            info.p = p;
            info.up = Some(false);
            info.down = Some(false);
            info.left = Some(false);
            info.right = Some(false);
            m.update(&info);
            m.fetch_targets(p);
            p = m.get_next().unwrap();
            // m.debug_print("no_wall_move_y.log", "index").unwrap();
        }
        m.debug_print("test.log", "no_wall_move_y").unwrap();

        assert_eq!(p.x, 0);
        assert_eq!(p.y, MAZE_HEIGHT - 1);
    }
    /// 一切壁がなく右上端がゴールの場合に最短距離ですすめるか
    #[test]
    pub fn no_wall_move_xy() {
        let mut m = Explorer::new(Point {
            x: MAZE_WIDTH - 1,
            y: MAZE_HEIGHT - 1,
        });

        let mut p = Point { x: 0, y: 0 };
        for _i in 0..31 {
            let mut info = UpdateInfo::default();
            info.p = p;
            info.up = Some(false);
            info.down = Some(false);
            info.left = Some(false);
            info.right = Some(false);
            m.update(&info);
            // 斜め走行を最初から出すためには隣のマスの情報が必要
            if p.x < MAZE_WIDTH - 1 {
                info.p = Point { x: p.x + 1, y: p.y };
                info.up = Some(false);
                info.down = Some(false);
                info.left = Some(false);
                info.right = Some(p.x >= MAZE_WIDTH - 2);
                m.update(&info);
            }
            if p.y < MAZE_HEIGHT - 1 {
                info.p = Point { x: p.x, y: p.y + 1 };
                info.up = Some(p.y >= MAZE_HEIGHT - 2);
                info.down = Some(false);
                info.left = Some(false);
                info.right = Some(false);
                m.update(&info);
            }

            m.fetch_targets(p);
            p = m.get_next().unwrap();
            // m.debug_print("no_wall_move_xy.log", "index").unwrap();
        }
        m.debug_print("test.log", "no_wall_move_xy").unwrap();

        assert_eq!(p.x, MAZE_WIDTH - 1);
        assert_eq!(p.y, MAZE_HEIGHT - 1);
    }
}
fn main() {
    // use oratosquilla::prelude::*;
}
