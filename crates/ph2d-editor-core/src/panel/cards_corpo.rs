//! ⭐ **T29 — o corpo de um cartão com a cara do cartão** (`docs/CartoesUI/04_plano_de_construcao.md`).
//!
//! As secções de componente (física, vida, câmera…) pintam-se pelos ajudantes de linha do Inspector
//! (`ph2d-panel-inspector/src/sections/rows.rs`), não pelo [`CardCtx`]. Enquanto um corpo de cartão
//! corre, [`paint_card_with`](super::paint_card_with) publica aqui a tinta dele; os ajudantes
//! perguntam [`card_body`] e, quando há, pintam pela [`fields`]/[`choices`] (rótulo EM CIMA com a unidade nele — D13
//! —, campo Figma com glifo, segmentos pela largura do conteúdo). Fora de um cartão nada muda.

#[path = "cards_corpo_campos.rs"]
mod campos;
#[path = "cards_corpo_marca.rs"]
mod marca;
pub use campos::{
    LABEL_PX, ROW_GAP, bar_field, check_in, colunas_iguais, facts, fields_in, swatch_grid,
    swatch_in, swatch_strip, text_in,
};
pub(crate) use marca::pede as pede_marca;
pub(super) use marca::{FAIXA as FAIXA_DA_MARCA, check_em, check_em_fila, pede_em_fila};

use super::{CardCtx, Cell, Seg, Widget, censo, compacta};
use crate::interaction::{HitIndex, WidgetStore};
use crate::widget::CheckboxValue;
use crate::widget::card_field::CardFieldSpec;
use crate::widget::card_field::Glyph;
use crate::widget::card_ink::CardInk;
use crate::widget::key_mark::{Eixos, Rotulo};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_text::TextSystem;
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

/// A tabela de onde [`letra_da_grandeza`] fatia a letra (um `&str` que vive sem alocar).
pub(crate) const MAIUSCULAS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// As palavras que nunca dão a letra de um campo; se todas o forem, vale a primeira («To state» ⇒ `T`).
pub(crate) const SEM_LETRA: [&str; 9] = [
    "at", "after", "on", "of", "per", "from", "to", "for", "state",
];

/// A letra de um campo só: a inicial, em maiúscula, da ÚLTIMA palavra do rótulo — a que nomeia a
/// grandeza ou o eixo (T56: «Half Width» ⇒ `W`, nunca o `H` de «Half»; «Offset Y» ⇒ `Y`; o nome de
/// um script, «jump_power» ⇒ `P`). A unidade entre parênteses não conta.
#[must_use]
pub fn letra_da_grandeza(rotulo: &str) -> &str {
    let mut palavras = rotulo
        .split(|c: char| c.is_whitespace() || c == '_')
        .filter(|p| p.starts_with(char::is_alphanumeric));
    let primeira = palavras.next().unwrap_or(rotulo);
    // «H Frames» ⇒ `H`: uma letra sozinha à frente é a própria letra.
    let palavra = if primeira.chars().count() == 1 {
        primeira
    } else {
        // «Repath At» ⇒ `R`: uma preposição no fim não nomeia a grandeza.
        let todas: Vec<&str> = std::iter::once(primeira).chain(palavras).collect();
        todas
            .iter()
            .rev()
            .find(|p| !SEM_LETRA.contains(&p.to_ascii_lowercase().as_str()))
            .copied()
            .unwrap_or(primeira)
    };
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
    if let [id] = ids {
        let texto = unidade.map_or_else(|| rotulo.to_owned(), |u| format!("{rotulo} ({u})"));
        let letra = match glifo(rotulo, eixos, 1, 0) {
            Glyph::Text(t) => t,
            _ => "",
        };
        let spec = CardFieldSpec {
            step,
            scrub: step,
            default: None,
        };
        if let Some(fim) = compacta::pede_numero(
            scene,
            text_system,
            store,
            hit_index,
            ink,
            (x, w, y),
            texto,
            letra,
            (*id, *id),
            spec,
        ) {
            return fim;
        }
    }
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
    // X · Y / W · H, como no Figma: quatro caixas numa linha cortam o número (05 §4, lei 7).
    if let ([c], [texto]) = (cols, textos.as_slice())
        && let (Eixos::Retangulo, [a, b, d, e]) = (c.eixos, c.ids)
    {
        let campo = |id, i| {
            Cell::new(Widget::Field {
                id,
                glyph: Glyph::Text(c.eixos.letra(i)),
            })
        };
        let mut linha = ctx(scene, text_system, store, hit_index, ink, x, w);
        let y = linha.row(&[campo(*a, 0).label(texto), campo(*b, 1).key(c.ids)], y);
        return linha.row(&[campo(*d, 2), campo(*e, 3)], y);
    }
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
    if linhas.len() == 1
        && segs.len() == compacta::ESCOLHA_CURTA_MAX
        && !rotulo.is_empty()
        && let Some(fim) = compacta::pede_escolha(
            scene,
            text_system,
            store,
            hit_index,
            ink,
            (x, w, y),
            rotulo,
            ids,
            segs,
        )
    {
        return fim;
    }
    let mut c = ctx(scene, text_system, store, hit_index, ink, x, w);
    let mut y = y;
    for (k, l) in linhas.into_iter().enumerate() {
        let mut cell = Cell::new(Widget::Choice(l));
        // Sem nome, a fila é o corpo do cartão (o título diz); sem ids, um ajuste que não anima (D17).
        if k == 0 && !rotulo.is_empty() {
            cell = cell.label(rotulo);
        }
        if k == 0 && !ids.is_empty() {
            cell = cell.key(ids);
        }
        y = c.row(&[cell], y);
    }
    y
}

/// As linhas que este corpo pinta para o `property_row`, injectadas pelo
/// [`com`](crate::property_row::cartao::com).
pub(super) const LINHAS: crate::property_row::cartao::CardRows =
    crate::property_row::cartao::CardRows {
        fields,
        swatch: swatch_de_fora,
        choices: choices_de_fora,
        check: check_de_fora,
        text: text_de_fora,
        label: rotulo_em_cima,
    };

#[allow(clippy::too_many_arguments)]
fn swatch_de_fora(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    x: f32,
    w: f32,
    y: f32,
    rotulo: Rotulo<'_>,
    id: NodeId,
    rgba: Option<[u8; 4]>,
) -> f32 {
    swatch_grid(
        scene,
        text_system,
        store,
        hit_index,
        ink,
        x,
        w,
        y,
        rotulo.texto,
        &[(id, rgba)],
        1,
    )
}

#[allow(clippy::too_many_arguments)]
fn text_de_fora(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    x: f32,
    w: f32,
    y: f32,
    rotulo: Rotulo<'_>,
    id: NodeId,
    placeholder: &str,
) -> f32 {
    let ids = [id];
    let cell = Cell::new(Widget::Text { id, placeholder })
        .label(rotulo.texto)
        .key(&ids);
    ctx(scene, text_system, store, hit_index, ink, x, w).row(&[cell], y)
}

/// ⭐ O rótulo EM CIMA de um controlo que o chamador constrói (T29, os 11 `property_label_row`): as
/// colunas devolvidas têm o controlo a toda a largura por baixo e a marca na linha do rótulo (D17).
/// `control.y + control.h` é o fundo; fora de um cartão a mesma conta dá a linha de sempre.
pub(super) fn rotulo_em_cima(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    ink: CardInk,
    x: f32,
    w: f32,
    y: f32,
    rotulo: &str,
) -> crate::widget::PropertyRow {
    let (k, ly) = (m::KEY_MARK_BOX, y + m::LABEL_TOP);
    crate::widget::card_field::paint_label(scene, text_system, ink, x, ly, w - k, rotulo);
    let fundo = ly + m::LABEL_PX + m::LABEL_BOTTOM + m::FIELD_H;
    if censo::ligado() {
        let pede = text_system.prefix_width(rotulo, m::LABEL_PX).ceil() + k;
        censo::rotulo((x, w), ly, fundo, rotulo, pede);
    }
    crate::widget::PropertyRow {
        label: Rect::new(x, ly, w - k, m::LABEL_PX),
        control: Rect::new(x, ly + m::LABEL_PX + m::LABEL_BOTTOM, w, m::FIELD_H),
        dot: Rect::new(x + w - k, ly + (m::LABEL_PX - k) * 0.5, k, k),
    }
}

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
    valor: CheckboxValue,
    parada: bool,
) -> f32 {
    check_valor(
        scene,
        text_system,
        store,
        hit_index,
        ink,
        x,
        w,
        y,
        (id, label),
        (valor, parada),
    )
}

/// Uma caixa de marcar de cartão: o id do `Checkbox` de hoje, o nome e o valor do MODELO.
#[derive(Clone, Copy, Debug)]
pub struct Marca<'a> {
    pub id: NodeId,
    pub label: Rotulo<'a>,
    pub on: bool,
}

/// ⭐ **A caixa de marcar do cartão, numa MOLDURA à Godot** (dono 13/11; antes T64 à Figma): o nome
/// em cima com o losango, a moldura com ✓ e «On» por baixo; nome e moldura marcam. Devolve o fundo.
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
    let valor = if c.on {
        CheckboxValue::Checked
    } else {
        CheckboxValue::Unchecked
    };
    check_valor(
        scene,
        text_system,
        store,
        hit_index,
        ink,
        x,
        w,
        y,
        (c.id, c.label),
        (valor, false),
    )
}

/// [`check`] com o valor inteiro: «misto» (o traço) e parada (o estado desligado do `Checkbox`).
#[allow(clippy::too_many_arguments)]
fn check_valor(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    x: f32,
    w: f32,
    y: f32,
    (id, label): (NodeId, Rotulo<'_>),
    (valor, parada): (CheckboxValue, bool),
) -> f32 {
    if let Some(fim) = compacta::pede_marca(
        scene,
        text_system,
        store,
        hit_index,
        ink,
        (x, w, y),
        id,
        label,
        valor,
        parada,
    ) {
        return fim;
    }
    let y = y + m::LABEL_TOP + FAIXA_DA_MARCA;
    let r = Rect::new(x, y, w, m::FIELD_H);
    check_em(
        scene,
        text_system,
        store,
        hit_index,
        ink,
        r,
        (id, label),
        (valor, parada),
    );
    if censo::ligado() {
        let pede = pede_marca(text_system, label);
        censo::linha((x, w), y, y + m::FIELD_H, || {
            vec![censo::CelulaMedida {
                nome: label.texto.to_owned(),
                curta: true,
                pede,
            }]
        });
    }
    y + m::FIELD_H
}

pub(super) fn ctx<'s>(
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
#[path = "cards_corpo_tests.rs"]
mod tests;
