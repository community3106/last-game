//! 入力 / Player input.
//!
//! この型はフロントエンド (キーボード・タッチ) から作られ、[`crate::World::step`] に渡されます。
//! テストでは [`Input::new`] にビルダー風メソッドを繋げて使います。

/// 1 フレーム分の入力状態 / input for a single simulation step.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Input {
    /// 左 / move left
    pub left: bool,
    /// 右 / move right
    pub right: bool,
    /// ジャンプ / jump (variable height: hold to jump higher)
    pub jump: bool,
    /// ダッシュ / run (hold to move faster)
    pub run: bool,
    /// 下 / duck (用途: 土管など、現在は予約)
    pub down: bool,
    /// ポーズ切替 / toggle pause
    pub pause: bool,
}

impl Input {
    /// なにも押されていない状態 / nothing pressed.
    pub const fn new() -> Self {
        Self {
            left: false,
            right: false,
            jump: false,
            run: false,
            down: false,
            pause: false,
        }
    }

    /// 左を押す / press left.
    pub const fn left(mut self, on: bool) -> Self {
        self.left = on;
        self
    }

    /// 右を押す / press right.
    pub const fn right(mut self, on: bool) -> Self {
        self.right = on;
        self
    }

    /// ジャンプを押す / press jump.
    pub const fn jump(mut self, on: bool) -> Self {
        self.jump = on;
        self
    }

    /// ダッシュを押す / press run.
    pub const fn run(mut self, on: bool) -> Self {
        self.run = on;
        self
    }

    /// 下を押す / press down.
    pub const fn down(mut self, on: bool) -> Self {
        self.down = on;
        self
    }

    /// ポーズを押す / press pause.
    pub const fn pause(mut self, on: bool) -> Self {
        self.pause = on;
        self
    }

    /// 水平方向の入力 (-1.0 / 0.0 / 1.0) / horizontal axis.
    pub fn axis(self) -> f32 {
        match (self.left, self.right) {
            (true, false) => -1.0,
            (false, true) => 1.0,
            _ => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_is_exclusive() {
        assert_eq!(Input::new().axis(), 0.0);
        assert_eq!(Input::new().left(true).axis(), -1.0);
        assert_eq!(Input::new().right(true).axis(), 1.0);
        // 両方押した場合はニュートラル / pressing both cancels out
        assert_eq!(Input::new().left(true).right(true).axis(), 0.0);
    }

    #[test]
    fn builder_is_chainable() {
        let input = Input::new().right(true).jump(true).run(true);
        assert!(input.right && input.jump && input.run && !input.left);
    }
}
