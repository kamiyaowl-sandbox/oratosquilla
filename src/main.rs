extern crate oratosquilla;

// #[cfg(test)]
mod test {
    use oratosquilla::maze::*;

    /// 最初の地点から一歩進むか
    #[test]
    pub fn ahead_start() {
        let mut m = Maze::new(Point { x: 10, y: 10 });

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
        let mut m = Maze::new(Point { x: 10, y: 10 });

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
        let mut m = Maze::new(Point { x: 10, y: 10 });

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
}
fn main() {
    use oratosquilla::maze::*;
}
