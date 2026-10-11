use serde::{Deserialize, Serialize};

/// レベル定義 — JSONでシリアライズし、NIP-44で暗号化して配信する想定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level {
    pub name: String,
    pub width: usize,  // タイル幅
    pub height: usize, // タイル高
    /// 2Dタイルマップ: 0=空, 1=地面, 2=ブロック, 3=土管, 4=コイン, 5=敵スポーン, 9=ゴール
    pub tiles: Vec<Vec<u8>>,
    pub player_spawn: (f32, f32),
    pub bg: String,
}

impl Level {
    pub fn from_json(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
    pub fn tile_at(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return 1; // 画面外は壁扱い (落下死は別で判定)
        }
        self.tiles[y as usize][x as usize]
    }
    pub fn is_solid(&self, x: i32, y: i32) -> bool {
        matches!(self.tile_at(x, y), 1 | 2 | 3)
    }
}
