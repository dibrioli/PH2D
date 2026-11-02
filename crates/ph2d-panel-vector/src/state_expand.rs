//! Os knobs do **Offset Path** — irmão de `state.rs` pelo teto de 600 LOC.
//!
//! São estado PANEL-LOCAL, e é isso que os separa dos irmãos `state_*`: os outros publicam o que a
//! shell OBSERVOU na cena, estes guardam o que o artista ESCOLHEU e a shell vem ler no quadro. A
//! exceção é o [`offset_live_on`], que a shell publica: só com um offset vivo na seleção o painel
//! mostra `Apply` e `Remove`.
//!
//! ⚠️ Não confundir com o Corner/Side do **Contour**, que faz a mesma pergunta com os mesmos
//! códigos mas tem resposta por-FORMA (vive no componente, não aqui): lá o chip retuna um efeito
//! que já está na tela, aqui ele arma o PRÓXIMO offset.

use std::cell::Cell;

/// ⭐ **Tudo o que o Offset Path escolhe menos a distância** — o espelho de
/// `ph2d_ecs::VecOffset` sem o `d` (o painel não depende do ECS).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OffsetKnobs {
    /// A quina (`Corners`): `0` Miter · `1` Round · `2` Bevel.
    pub join: u8,
    /// Que contornos andam, o código do motor: `0` só o de fora (`Holes: Keep`) · `1` só os furos
    /// (documentos antigos; o painel já não o oferece) · `2` todos (`Holes: Offset too`).
    pub side: u8,
    /// O `Limit` do Miter.
    pub miter: f64,
    /// `Result: New shape behind` (a fonte fica por cima) × `Replace`.
    pub behind: bool,
}

/// Os knobs com que o painel NASCE — Miter, os furos andam junto, limite 4 e uma forma NOVA por
/// trás (o *Offset Path* do Illustrator faz uma cópia; dono 02/11: «faça tudo como no inspector»).
///
/// `pub` porque a SHELL guarda os knobs que viu no quadro anterior para distinguir *"o artista
/// clicou"* de *"o painel está no valor de sempre"*, e esse valor tem de nascer IGUAL a este.
pub const EXPAND_KNOBS_AT_BIRTH: OffsetKnobs = OffsetKnobs {
    join: 0,
    side: 2,
    miter: 4.0, // LITERAL-PX-OK: razão do Miter (adimensional), = ph2d_ecs::MITER_AT_BIRTH (gate na app-vec)
    behind: true,
};

thread_local! {
    static KNOBS: Cell<OffsetKnobs> = const { Cell::new(EXPAND_KNOBS_AT_BIRTH) };
    static LIVE: Cell<bool> = const { Cell::new(false) };
}

/// Os knobs do Offset Path como o painel os guarda agora.
#[must_use]
pub fn offset_knobs() -> OffsetKnobs {
    KNOBS.with(Cell::get)
}

/// Escreve os knobs — o clique de um chip, o commit do `Limit`, e o ESPELHO da forma selecionada
/// (a shell escreve os knobs DELA, senão o chip mentiria sobre a forma que está na tela).
pub fn set_offset_knobs(k: OffsetKnobs) {
    KNOBS.with(|c| c.set(k));
}

fn edit(f: impl FnOnce(&mut OffsetKnobs)) {
    let mut k = offset_knobs();
    f(&mut k);
    set_offset_knobs(k);
}

/// A junção escolhida (`0` Miter · `1` Round · `2` Bevel).
#[must_use]
pub fn expand_join() -> u8 {
    offset_knobs().join
}

pub fn set_expand_join(v: u8) {
    edit(|k| k.join = v);
}

/// Que contornos andam (`0` só o de fora · `1` só os furos · `2` todos).
#[must_use]
pub fn expand_side() -> u8 {
    offset_knobs().side
}

pub fn set_expand_side(v: u8) {
    edit(|k| k.side = v);
}

pub(crate) fn set_expand_behind(v: bool) {
    edit(|k| k.behind = v);
}

pub(crate) fn set_expand_miter(v: f64) {
    edit(|k| k.miter = v.max(1.0));
}

/// A seleção tem um offset VIVO? — publicado pela shell a cada quadro.
#[must_use]
pub fn offset_live_on() -> bool {
    LIVE.with(Cell::get)
}

pub fn set_offset_live_on(v: bool) {
    LIVE.with(|c| c.set(v));
}
