extern crate oratosquilla;

#[cfg(test)]
mod test {
    use oratosquilla::maze::*;

    /// 最初の地点から一歩進むか
    #[test]
    pub fn ahead_start() {
        let mut m = Maze::new(Point { x: 10, y: 10 });

        let p = Point {x: 0, y: 0, };
        let mut info = UpdateInfo::default();
        info.p = p;
        info.up = Some(false);
        info.down = None;
        info.left = Some(true);
        info.right = Some(true);
        m.update(&info);
        m.fetch_targets(p);
        let next_p = m.get_next();
        // m.debug_print().unwrap();

        assert_eq!(next_p.is_some(), true);
        assert_eq!(next_p.unwrap().x, 0);
        assert_eq!(next_p.unwrap().y, 1);
    }

}
fn main() {
    use oratosquilla::maze::*;
}
