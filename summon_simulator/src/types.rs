use enumset::EnumSetType;
use serde::{Deserialize, Serialize};

#[derive(Hash, Debug, PartialOrd, Ord, EnumSetType, Serialize, Deserialize)]
pub enum Color {
    Red,
    Blue,
    Green,
    Colorless,
}

impl TryFrom<usize> for Color {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Color::Red,
            1 => Color::Blue,
            2 => Color::Green,
            3 => Color::Colorless,
            _ => return Err(()),
        })
    }
}

#[derive(Hash, Debug, PartialOrd, Ord, EnumSetType, Serialize, Deserialize)]
pub enum Pool {
    Focus,
    Fivestar,
    FourstarFocus,
    FourstarSpecial,
    Common,
}

impl TryFrom<usize> for Pool {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Pool::Focus,
            1 => Pool::Fivestar,
            2 => Pool::FourstarFocus,
            3 => Pool::FourstarSpecial,
            4 => Pool::Common,
            _ => return Err(()),
        })
    }
}
