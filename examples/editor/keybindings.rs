use bevy::prelude::*;

#[derive(Clone, Copy, Debug)]
pub(crate) enum KeyBind {
    Key(KeyCode),
    Mouse(MouseButton),
}
