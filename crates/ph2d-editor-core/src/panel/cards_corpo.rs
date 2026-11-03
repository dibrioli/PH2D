//! ⭐ **T29 — o corpo de um cartão com a cara do cartão** (`docs/CartoesUI/04_plano_de_construcao.md`).
//!
//! As secções de componente (física, vida, câmera…) pintam-se pelos ajudantes de linha do Inspector
//! (`ph2d-panel-inspector/src/sections/rows.rs`), não pelo [`CardCtx`]. Enquanto um corpo de cartão
//! corre, [`paint_card_with`](super::paint_card_with) publica aqui a tinta dele; os ajudantes
//! perguntam [`card_body`] e, quando há, pintam pela [`fields`]/[`choices`] (rótulo EM CIMA com a unidade nele — D13
//! —, campo Figma com glifo, segmentos pela largura do conteúdo). Fora de um cartão nada muda.

use super::{CardCtx, Cell, Seg, Widget};
use crate::icons::IconId;
use crate::interaction::{HitIndex, WidgetStore};
use crate::widget::card_field::CardFieldSpec;
use crate::widget::card_field::Glyph;
use crate::widget::card_ink::CardInk;
use crate::widget::key_mark::{Eixos, Rotulo};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_vector::VectorScene;

/// O glifo da pega de um número: a letra da caixa numa linha de vários ([`Eixos`]: X · Y · Z · W,
/// ou W · H num tamanho); num campo só, a letra DECLARADA ([`Eixos::Letra`]) ou a inicial da
/// ÚLTIMA palavra do rótulo — [`letra_da_grandeza`].
fn glifo(rotulo: &str, eixos: Eixos, n: usize, i: usize) -> Glyph<'_> {
    if n > 1 || matches!(eixos, Eixos::Letra(_)) {
        return Glyph::Text(eixos.letra(i));
    }
    Glyph::Text(letra_da_grandeza(rotulo))
}

/// A letra de um campo só: a inicial, em maiúscula, da ÚLTIMA palavra do rótulo — a que nomeia a
/// grandeza ou o eixo (T56: «Half Width» ⇒ `W`, nunca o `H` de «Half»; «Offset Y» ⇒ `Y`; o nome de
/// um script, «jump_power» ⇒ `P`). A unidade entre parênteses não conta.
#[must_use]
pub fn letra_da_grandeza(rotulo: &str) -> &str {
    const MAIUSCULAS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let palavra = rotulo
        .split(|c: char| c.is_whitespace() || c == '_')
        .rev()
        .find(|p| p.starts_with(char::is_alphanumeric))
        .unwrap_or(rotulo);
    match palavra.chars().next() {
        Some(c) if c.is_ascii_lowercase() => {
            let i = usize::from(c as u8 - b'a');
            &MAIUSCULAS[i..=i]
        }
        Some(c) => &palavra[..c.len_utf8()],
        None => "",
    }
}

/// Uma coluna de uma linha de números de cartão: um rótulo (com a unidade nele, D13) sobre as
/// caixas dos `ids`, com a marca de animação deles. Várias colunas = vários rótulos numa linha
/// (maquete: «Min (°)» | «Max (°)»).
#[derive(Clone, Copy, Debug)]
pub struct Coluna<'a> {
    pub rotulo: &'a str,
    pub eixos: Eixos,
    pub unidade: Option<&'a str>,
    pub ids: &'a [NodeId],
}

/// ⭐ **A linha de números do cartão** a partir de um ajudante que não tem o [`CardCtx`]: o rótulo
/// EM CIMA com a unidade nele («Mass (kg)», D13), uma caixa Figma por id (glifo + valor, sem
/// sufixo). Os ids sem [`CardFieldSpec`] do painel registam-se aqui com o `step` do ajudante (a seta
/// anda um passo; a pega, um passo por pixel). Devolve o fundo.
#[allow(clippy::too_many_arguments)]
pub fn fields(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    x: f32,
    w: f32,
    y: f32,
    rotulo: &str,
    eixos: Eixos,
    unidade: Option<&str>,
    ids: &[NodeId],
    step: f64,
) -> f32 {
    let col = Coluna {
        rotulo,
        eixos,
        unidade,
        ids,
    };
    columns(
        scene,
        text_system,
        store,
        hit_index,
        ink,
        x,
        w,
        y,
        &[col],
        step,
    )
}

/// [`fields`] com várias colunas numa linha só (T56: dois campos por linha quando faz sentido).
#[allow(clippy::too_many_arguments)]
pub fn columns(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    x: f32,
    w: f32,
    y: f32,
    cols: &[Coluna<'_>],
    step: f64,
) -> f32 {
    let spec = CardFieldSpec {
        step,
        scrub: step,
        default: None,
    };
    for &id in cols.iter().flat_map(|c| c.ids) {
        store.mark_card_field_at_paint(id, spec);
    }
    let textos: Vec<String> = cols
        .iter()
        .map(|c| {
            c.unidade
                .map_or_else(|| c.rotulo.to_owned(), |u| format!("{} ({u})", c.rotulo))
        })
        .collect();
    let mut cells: Vec<Cell<'_>> = Vec::new();
    for (c, texto) in cols.iter().zip(&textos) {
        let n = c.ids.len();
        for (i, &id) in c.ids.iter().enumerate() {
            let cell = Cell::new(Widget::Field {
                id,
                glyph: glifo(c.rotulo, c.eixos, n, i),
            });
            let cell = if i == 0 { cell.label(texto) } else { cell };
            cells.push(if i + 1 == n { cell.key(c.ids) } else { cell });
        }
    }
    ctx(scene, text_system, store, hit_index, ink, x, w).row(&cells, y)
}

/// Duas escolhas segmentadas lado a lado, cada uma com o seu rótulo (T56: «Freeze X» | «Freeze Y»)
/// — `None` quando os segmentos de alguma não cabem em meia linha (o chamador pinta uma por linha).
#[allow(clippy::too_many_arguments)]
pub fn choices_pair(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    x: f32,
    w: f32,
    y: f32,
    a: (&str, &[NodeId], &[Seg<'_>]),
    b: (&str, &[NodeId], &[Seg<'_>]),
) -> Option<f32> {
    let meia = (w - m::FIELD_GAP) * 0.5 - m::SEG_PAD * 2.0;
    let cabe = |ts: &mut TextSystem, segs: &[Seg<'_>]| {
        let need: f32 = segs
            .iter()
            .map(|s| ts.prefix_width(s.text, m::SEG_TEXT_PX) + m::SEG_INSET * 2.0 + m::SEG_PAD)
            .sum();
        need <= meia
    };
    if !(cabe(text_system, a.2) && cabe(text_system, b.2)) {
        return None;
    }
    let cells = [
        Cell::new(Widget::Choice(a.2)).label(a.0).key(a.1),
        Cell::new(Widget::Choice(b.2)).label(b.0).key(b.1),
    ];
    Some(ctx(scene, text_system, store, hit_index, ink, x, w).row(&cells, y))
}

/// ⭐ **A escolha segmentada do cartão**: segmentos com a largura do CONTEÚDO, quebrando em linhas
/// quando não cabem (os 8 «Layer» e os 9 «Join As» da física não cabem numa). O rótulo e a marca
/// de animação vão na 1.ª. Devolve o fundo.
#[allow(clippy::too_many_arguments)]
pub fn choices(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    x: f32,
    w: f32,
    y: f32,
    rotulo: &str,
    ids: &[NodeId],
    segs: &[Seg<'_>],
) -> f32 {
    let livre = w - m::SEG_PAD * 2.0;
    let mut linhas: Vec<&[Seg<'_>]> = Vec::new();
    let (mut ini, mut usado) = (0, 0.0);
    for (i, s) in segs.iter().enumerate() {
        let need =
            text_system.prefix_width(s.text, m::SEG_TEXT_PX) + m::SEG_INSET * 2.0 + m::SEG_PAD;
        if i > ini && usado + need > livre {
            linhas.push(&segs[ini..i]);
            (ini, usado) = (i, 0.0);
        }
        usado += need;
    }
    linhas.push(&segs[ini..]);
    let mut c = ctx(scene, text_system, store, hit_index, ink, x, w);
    let mut y = y;
    for (k, l) in linhas.into_iter().enumerate() {
        let cell = Cell::new(Widget::Choice(l));
        let cell = if k == 0 {
            cell.label(rotulo).key(ids)
        } else {
            cell
        };
        y = c.row(&[cell], y);
    }
    y
}

/// As linhas que este corpo pinta para o `property_row`, injectadas pelo
/// [`com`](crate::property_row::cartao::com).
pub(super) const LINHAS: crate::property_row::cartao::CardRows =
    crate::property_row::cartao::CardRows {
        fields,
        choices: choices_de_fora,
        check: check_de_fora,
    };

#[allow(clippy::too_many_arguments)]
fn choices_de_fora(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    x: f32,
    w: f32,
    y: f32,
    rotulo: &str,
    segmentos: &[(&str, bool, NodeId)],
) -> f32 {
    let ids: Vec<NodeId> = segmentos.iter().map(|&(_, _, id)| id).collect();
    let segs: Vec<Seg<'_>> = segmentos
        .iter()
        .map(|&(text, on, id)| Seg {
            id,
            icon: None,
            text,
            on,
        })
        .collect();
    choices(
        scene,
        text_system,
        store,
        hit_index,
        ink,
        x,
        w,
        y,
        rotulo,
        &ids,
        &segs,
    )
}

#[allow(clippy::too_many_arguments)]
fn check_de_fora(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    x: f32,
    w: f32,
    y: f32,
    id: NodeId,
    label: Rotulo<'_>,
    on: bool,
) -> f32 {
    check(
        scene,
        text_system,
        store,
        hit_index,
        ink,
        x,
        w,
        y,
        Marca { id, label, on },
    )
}

/// Uma caixa de marcar de cartão: o id do `Checkbox` de hoje, o nome e o valor do MODELO.
#[derive(Clone, Copy, Debug)]
pub struct Marca<'a> {
    pub id: NodeId,
    pub label: Rotulo<'a>,
    pub on: bool,
}

/// ⭐ **T64 — a caixa de marcar do cartão, à Figma** (dono 02/11): o quadradinho com ✓ e o nome à
/// direita, numa linha; a linha inteira é o alvo (o clique no nome marca) e a marca de animação
/// fica no fim. Devolve o fundo.
#[allow(clippy::too_many_arguments)]
pub fn check(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    x: f32,
    w: f32,
    y: f32,
    c: Marca<'_>,
) -> f32 {
    let (h, b, k) = (m::FIELD_H, m::CHECK_BOX, m::KEY_MARK_BOX);
    let y = y + m::FIELD_ROW_GAP;
    hit_index.register(c.id, Rect::new(x, y, w, h));
    let caixa = Rect::new(x, y + (h - b) * 0.5, b, b);
    if c.on {
        let fundo = ink.color(ColorToken::KanbanBlueBold);
        crate::paint::fill_rounded_rect(scene, caixa, m::SEG_RADIUS, fundo);
        let marca = ink.color(ColorToken::KanbanBtnOnFg);
        crate::paint::paint_icon(
            scene,
            IconId::Check,
            caixa,
            marca,
            StrokeToken::Default.px(),
        );
    } else {
        let hover = store.hover_live(c.id) > 0.0;
        let fundo = if hover {
            ColorToken::CardFieldHover
        } else {
            ColorToken::CardField
        };
        crate::paint::fill_rounded_rect(scene, caixa, m::SEG_RADIUS, ink.color(fundo));
        let borda = ink.color(ColorToken::CardLabel);
        crate::paint::stroke_rounded_rect(scene, caixa, m::SEG_RADIUS, m::FIELD_BORDER, borda);
    }
    let tx = x + b + m::CHECK_GAP;
    crate::paint::paint_text_elided(
        text_system,
        scene,
        c.label.texto,
        tx,
        y + (h - m::VALUE_PX) * 0.5,
        m::VALUE_PX,
        (x + w - tx - k).max(0.0),
        ink.color(ColorToken::KanbanCardFg),
    );
    let dot = Rect::new(x + w - k, y + (h - k) * 0.5, k, k);
    if let Some(alvo) =
        crate::widget::key_mark::paint_decorator_campo(scene, ink.theme, dot, &[c.id], c.label)
    {
        hit_index.register(alvo, dot);
    }
    y + h
}

fn ctx<'s>(
    scene: &'s mut VectorScene,
    text_system: &'s mut TextSystem,
    store: &'s WidgetStore,
    hit_index: &'s mut HitIndex,
    ink: CardInk,
    x: f32,
    w: f32,
) -> CardCtx<'s> {
    CardCtx {
        scene,
        text_system,
        store,
        hit_index,
        ink,
        x,
        w,
        first: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn letras(eixos: Eixos, n: usize) -> Vec<String> {
        (0..n)
            .map(|i| match glifo("Half Size", eixos, n, i) {
                Glyph::Text(t) => t.to_owned(),
                g => panic!("glifo inesperado {g:?}"),
            })
            .collect()
    }

    /// ⭐ T29: um TAMANHO mostra W · H (como o Figma), um retângulo X · Y · W · H; um eixo X · Y.
    #[test]
    fn a_size_pair_shows_w_and_h_like_figma() {
        assert_eq!(letras(Eixos::Tamanho, 2), ["W", "H"]);
        assert_eq!(letras(Eixos::Retangulo, 4), ["X", "Y", "W", "H"]);
        assert_eq!(letras(Eixos::Xyzw, 2), ["X", "Y"]);
        assert_eq!(
            letras(Eixos::Tamanho, 1),
            ["S"],
            "um campo só leva a inicial da última palavra"
        );
    }

    /// ⭐ T56: num campo só a letra é a da palavra que nomeia a grandeza ou o eixo, nunca a 1.ª.
    #[test]
    fn a_single_field_takes_the_letter_of_its_last_word() {
        let um = |r, e| match glifo(r, e, 1, 0) {
            Glyph::Text(t) => t.to_owned(),
            g => panic!("glifo inesperado {g:?}"),
        };
        assert_eq!(um("Half Width", Eixos::Xyzw), "W");
        assert_eq!(um("Offset Y", Eixos::Xyzw), "Y");
        assert_eq!(um("Radius (m)", Eixos::Xyzw), "R");
        assert_eq!(um("Half Width", Eixos::Letra("R")), "R");
        assert_eq!(
            um("Half height", Eixos::Xyzw),
            "H",
            "a letra é sempre maiúscula"
        );
        assert_eq!(um("jump_power", Eixos::Xyzw), "P", "o nome de um script");
        assert_eq!(um("speed", Eixos::Xyzw), "S");
    }
}
