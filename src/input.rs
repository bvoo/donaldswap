use anyhow::Result;
use enigo::{Enigo, Key, Keyboard};

pub fn send_esc() -> Result<()> {
    let mut enigo = Enigo::new(&enigo::Settings::default())?;
    enigo.key(Key::Escape, enigo::Direction::Click)?;
    Ok(())
}
