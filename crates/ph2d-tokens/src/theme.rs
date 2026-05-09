//! Theme enum. Procreate-style: Dark default + Light high-contrast alt.

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Theme {
    /// Dark Mode charcoal "unobtrusive" — Procreate default. Padrão PH2D.
    #[default]
    Dark,
    /// Light Mode mais contrastante — para ambientes muito iluminados.
    Light,
}

impl Theme {
    pub fn toggle(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Dark,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_dark() {
        assert_eq!(Theme::default(), Theme::Dark);
    }

    #[test]
    fn toggle_round_trips() {
        assert_eq!(Theme::Dark.toggle().toggle(), Theme::Dark);
        assert_eq!(Theme::Light.toggle().toggle(), Theme::Light);
    }
}
