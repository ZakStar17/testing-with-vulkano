/// State of each key
#[derive(PartialEq, Clone, Copy)]
pub enum KeyState {
  Pressed,
  Released,
}

impl Default for KeyState {
  fn default() -> Self {
    KeyState::Released
  }
}

/// struct that contains information about each pressed / not pressed key
#[derive(Default)]
pub struct Keys {
  pub a: KeyState,
  pub w: KeyState,
  pub s: KeyState,
  pub d: KeyState,
  pub space: KeyState,
  pub l_ctrl: KeyState,
  pub up_key: KeyState,
  pub down_key: KeyState,
  pub left_key: KeyState,
  pub right_key: KeyState,
}
