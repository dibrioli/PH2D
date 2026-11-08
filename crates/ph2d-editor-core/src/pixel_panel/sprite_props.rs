//! **Sprite ▸ Properties…** (P12 U, O17): a secção aberta é estado da VISTA, partilhado pelo painel
//! PIXEL (que a pinta) e pela família (o botão do menu e o Run Command).

thread_local! {
    static ABERTA: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// A secção Sprite Properties está aberta.
#[must_use]
pub fn is_open() -> bool {
    ABERTA.with(std::cell::Cell::get)
}

/// Abre (`true`) ou fecha a secção.
pub fn set_open(open: bool) {
    ABERTA.with(|a| a.set(open));
}

/// O botão do menu: abre a fechada, fecha a aberta.
pub fn toggle() {
    set_open(!is_open());
}
