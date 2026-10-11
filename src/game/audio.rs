//! 8-Bit Web Audio API Sound Synthesizer in Rust WASM

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    pub is_muted: bool,
    pub bgm_enabled: bool,
    pub volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            is_muted: false,
            bgm_enabled: true,
            volume: 0.3,
        }
    }
}

pub struct ChiptuneSynth {
    pub settings: AudioSettings,
}

impl Default for ChiptuneSynth {
    fn default() -> Self {
        Self {
            settings: AudioSettings::default(),
        }
    }
}

impl ChiptuneSynth {
    pub fn play_sfx(&self, sfx_name: &str) {
        if self.settings.is_muted {
            return;
        }

        #[cfg(target_arch = "wasm32")]
        {
            // Execute Web Audio oscillator notes via JS snippet or Web-Sys
            let js_code = format!(
                r#"
                if (window.playMarioSfx) {{
                    window.playMarioSfx('{}', {});
                }}
                "#,
                sfx_name, self.settings.volume
            );
            let _ = js_sys::eval(&js_code);
        }
    }
}
