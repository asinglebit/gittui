#![allow(non_snake_case)]

#[rustfmt::skip]
use ratatui::style::Color;

#[derive(PartialEq, Eq)]
pub enum ThemeNames {
    Classic,
    Ansi,
    Monochrome
}

pub struct Theme {
    pub name: ThemeNames,
    pub COLOR_RED: Color,
    pub COLOR_PINK: Color,
    pub COLOR_PURPLE: Color,
    pub COLOR_DURPLE: Color,
    pub COLOR_INDIGO: Color,
    pub COLOR_BLUE: Color,
    pub COLOR_CYAN: Color,
    pub COLOR_TEAL: Color,
    pub COLOR_GREEN: Color,
    pub COLOR_GRASS: Color,
    pub COLOR_LIME: Color,
    pub COLOR_YELLOW: Color,
    pub COLOR_AMBER: Color,
    pub COLOR_ORANGE: Color,
    pub COLOR_GRAPEFRUIT: Color,
    pub COLOR_BROWN: Color,
    pub COLOR_DARK_RED: Color,
    pub COLOR_LIGHT_GREEN_900: Color,
    pub COLOR_GREY_50: Color,
    pub COLOR_GREY_100: Color,
    pub COLOR_GREY_200: Color,
    pub COLOR_GREY_300: Color,
    pub COLOR_GREY_400: Color,
    pub COLOR_GREY_500: Color,
    pub COLOR_GREY_600: Color,
    pub COLOR_GREY_700: Color,
    pub COLOR_GREY_800: Color,
    pub COLOR_GREY_900: Color,
    pub COLOR_BORDER: Color,
    pub COLOR_TEXT: Color,
    pub COLOR_TEXT_SELECTED: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::classic()
    }
}

impl Theme {

    pub fn classic() -> Self {
        Self {
            name:                   ThemeNames::Classic,
            COLOR_RED:              Color::Rgb(239, 83, 80),
            COLOR_PINK:             Color::Rgb(236, 64, 122),
            COLOR_PURPLE:           Color::Rgb(171, 71, 188),
            COLOR_DURPLE:           Color::Rgb(126, 87, 194),
            COLOR_INDIGO:           Color::Rgb(92, 107, 192),
            COLOR_BLUE:             Color::Rgb(66, 165, 245),
            COLOR_CYAN:             Color::Rgb(38, 198, 218),
            COLOR_TEAL:             Color::Rgb(38, 166, 154),
            COLOR_GREEN:            Color::Rgb(102, 187, 106),
            COLOR_GRASS:            Color::Rgb(156, 204, 101),
            COLOR_LIME:             Color::Rgb(212, 225, 87),
            COLOR_YELLOW:           Color::Rgb(255, 238, 88),
            COLOR_AMBER:            Color::Rgb(255, 202, 40),
            COLOR_ORANGE:           Color::Rgb(255, 167, 38),
            COLOR_GRAPEFRUIT:       Color::Rgb(255, 112, 67),
            COLOR_BROWN:            Color::Rgb(141, 110, 99),
            COLOR_DARK_RED:         Color::Rgb(82, 31, 31),
            COLOR_LIGHT_GREEN_900:  Color::Rgb(34, 57, 37),
            COLOR_GREY_50:          Color::Rgb(250, 250, 250),
            COLOR_GREY_100:         Color::Rgb(245, 245, 245),
            COLOR_GREY_200:         Color::Rgb(238, 238, 238),
            COLOR_GREY_300:         Color::Rgb(224, 224, 224),
            COLOR_GREY_400:         Color::Rgb(189, 189, 189),
            COLOR_GREY_500:         Color::Rgb(158, 158, 158),
            COLOR_GREY_600:         Color::Rgb(117, 117, 117),
            COLOR_GREY_700:         Color::Rgb(97, 97, 97),
            COLOR_GREY_800:         Color::Rgb(66, 66, 66),
            COLOR_GREY_900:         Color::Rgb(33, 33, 33),
            COLOR_BORDER:           Color::Rgb(66, 66, 66),
            COLOR_TEXT:             Color::Rgb(97, 97, 97),
            COLOR_TEXT_SELECTED:    Color::Rgb(224, 224, 224),
        }
    }
    pub fn ansi() -> Self {
        Self {
            name:                   ThemeNames::Ansi,
            COLOR_RED:              Color::Red,
            COLOR_PINK:             Color::LightRed,
            COLOR_PURPLE:           Color::Magenta,
            COLOR_DURPLE:           Color::LightMagenta,
            COLOR_INDIGO:           Color::Blue,
            COLOR_BLUE:             Color::LightBlue,
            COLOR_CYAN:             Color::Cyan,
            COLOR_TEAL:             Color::LightCyan,
            COLOR_GREEN:            Color::Green,
            COLOR_GRASS:            Color::LightGreen,
            COLOR_LIME:             Color::Yellow,
            COLOR_YELLOW:           Color::LightYellow,
            COLOR_AMBER:            Color::Red,
            COLOR_ORANGE:           Color::LightRed,
            COLOR_GRAPEFRUIT:       Color::Magenta,
            COLOR_BROWN:            Color::LightMagenta,
            COLOR_DARK_RED:         Color::Reset,
            COLOR_LIGHT_GREEN_900:  Color::Reset,
            COLOR_GREY_50:          Color::Gray,
            COLOR_GREY_100:         Color::Gray,
            COLOR_GREY_200:         Color::Gray,
            COLOR_GREY_300:         Color::Gray,
            COLOR_GREY_400:         Color::DarkGray,
            COLOR_GREY_500:         Color::DarkGray,
            COLOR_GREY_600:         Color::DarkGray,
            COLOR_GREY_700:         Color::DarkGray,
            COLOR_GREY_800:         Color::DarkGray,
            COLOR_GREY_900:         Color::Reset,
            COLOR_BORDER:           Color::DarkGray,
            COLOR_TEXT:             Color::Gray,
            COLOR_TEXT_SELECTED:    Color::Reset,
        }
    }
    pub fn monochrome() -> Self {
        Self {
            name:                   ThemeNames::Monochrome,
            COLOR_RED:              Color::White,
            COLOR_PINK:             Color::White,
            COLOR_PURPLE:           Color::White,
            COLOR_DURPLE:           Color::White,
            COLOR_INDIGO:           Color::White,
            COLOR_BLUE:             Color::White,
            COLOR_CYAN:             Color::White,
            COLOR_TEAL:             Color::White,
            COLOR_GREEN:            Color::White,
            COLOR_GRASS:            Color::White,
            COLOR_LIME:             Color::White,
            COLOR_YELLOW:           Color::White,
            COLOR_AMBER:            Color::White,
            COLOR_ORANGE:           Color::White,
            COLOR_GRAPEFRUIT:       Color::White,
            COLOR_BROWN:            Color::White,
            COLOR_DARK_RED:         Color::DarkGray,
            COLOR_LIGHT_GREEN_900:  Color::DarkGray,
            COLOR_GREY_50:          Color::Gray,
            COLOR_GREY_100:         Color::Gray,
            COLOR_GREY_200:         Color::Gray,
            COLOR_GREY_300:         Color::Gray,
            COLOR_GREY_400:         Color::DarkGray,
            COLOR_GREY_500:         Color::DarkGray,
            COLOR_GREY_600:         Color::DarkGray,
            COLOR_GREY_700:         Color::DarkGray,
            COLOR_GREY_800:         Color::DarkGray,
            COLOR_GREY_900:         Color::Reset,
            COLOR_BORDER:           Color::DarkGray,
            COLOR_TEXT:             Color::Gray,
            COLOR_TEXT_SELECTED:    Color::Reset,
        }
    }
}
