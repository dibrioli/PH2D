//! ⭐ **T29 — o corpo de um cartão com a cara do cartão** (`docs/CartoesUI/04_plano_de_construcao.md`).
//!
//! As secções de componente (física, vida, câmera…) pintam-se pelos ajudantes de linha do Inspector
//! (`ph2d-panel-inspector/src/sections/rows.rs`), não pelo [`CardCtx`]. Enquanto um corpo de cartão
//! corre, [`paint_card_with`](super::paint_card_with) publica aqui a tinta dele; os ajudantes
//! perguntam [`card_body`] e, quando há, pintam pela [`fields`]/[`choices`] (rótulo EM CIMA com a unidade nele — D13
//! —, campo Figma com glifo, segmentos pela largura do conteúdo). Fora de um cartão nada muda.

#[path = "cards_corpo_campos.rs"]
mod campos;
pub use campos::{
    LABEL_PX, ROW_GAP, bar_field, caixa_ao_lado_do_rotulo, colunas_iguais, facts, fields_in, swatch_grid,
    swatch_in, swatch_strip,
};

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
use ph2d_tokens::ColorToken;
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
const MAIUSCULAS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// A letra de um campo só: a inicial, em maiúscula, da ÚLTIMA palavra do rótulo — a que nomeia a
/// grandeza ou o eixo (T56: «Half Width» ⇒ `W`, nunca o `H` de «Half»; «Offset Y» ⇒ `Y`; o nome de
/// um script, «jump_power» ⇒ `P`). A unidade entre parênteses não conta.
#[must_use]
pub fn letra_da_grandeza(rotulo: &str) -> &str {
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
    if let [id] = ids {
        let texto =
            unidade.map_or_else(|| rotulo.to_owned(), |u| format!("{rotulo} ({u})"));
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
    let y = y + m::FIELD_ROW_GAP;
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
        false,
    );
    if censo::ligado() {
        let pede = m::CHECK_BOX
            + m::CHECK_GAP
            + text_system.prefix_width(label.texto, m::VALUE_PX).ceil()
            + label.reserva_da_marca();
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

/// A caixa de marcar do cartão no rect `r`: o quadradinho, o nome e a marca de animação no fim.
#[allow(clippy::too_many_arguments)]
pub(super) fn check_em(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    r: Rect,
    (id, label): (NodeId, Rotulo<'_>),
    (valor, parada): (CheckboxValue, bool),
    colada: bool,
) {
    let (x, y, w, h) = (r.x, r.y, r.w, r.h);
    let (b, k) = (m::CHECK_BOX, label.reserva_da_marca());
    hit_index.register(id, r);
    if censo::ligado() {
        let tw = text_system.prefix_width(label.texto, m::VALUE_PX);
        censo::texto(label.texto, b + m::CHECK_GAP + tw + k, w);
    }
    let caixa = crate::widget::card_check_rect(x, y, h);
    let hover = !parada && store.hover_live(id) > 0.0;
    crate::widget::paint_card_check_value(scene, ink, caixa, valor, hover);
    let tx = x + b + m::CHECK_GAP;
    crate::paint::paint_text_elided(
        text_system,
        scene,
        label.texto,
        tx,
        y + crate::paint::folga_do_texto(h, m::VALUE_PX),
        m::VALUE_PX,
        (x + w - tx - k).max(0.0),
        ink.color(if parada {
            ColorToken::CardLabel
        } else {
            ColorToken::KanbanCardFg
        }),
    );
    // Numa coluna de duas a marca anda com o nome: no fim ela encostava na vizinha e lia-se dela (05 §8).
    let dx = if colada {
        let tw = text_system.prefix_width(label.texto, m::VALUE_PX).ceil();
        (tx + tw).min(x + w - k)
    } else {
        x + w - k
    };
    let dot = Rect::new(dx, y + (h - k) * 0.5, k, k);
    if let Some(alvo) =
        crate::widget::key_mark::paint_decorator_campo(scene, ink.theme, dot, &[id], label)
    {
        hit_index.register(alvo, dot);
    }
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

    /// ⭐ 05 §8: em nenhuma largura o par de caixas corta um nome: emparelha quando os dois cabem com a
    /// coluna de animação, e senão empilha. *Mutação: `caixa_do_cartao` sem o `KEY_MARK_BOX`.*
    #[test]
    fn a_pair_of_checks_never_cuts_a_name_at_any_width() {
        use crate::property_row::{Seccao, cartao, paint_check_rows};
        use crate::text_elide::elisao;
        use ph2d_tokens::Theme;
        let (nomes, ids) = (["Active", "Look Through"], [NodeId(80), NodeId(81)]);
        let tema = Theme::default();
        let ink = CardInk::of_list(tema);
        let mut ts = TextSystem::without_system_fonts();
        for w in 150..=460 {
            let mut scene = VectorScene::new();
            let mut hit = HitIndex::default();
            let store = WidgetStore::default();
            let sec = Seccao::medida(&mut ts, 1, &nomes);
            let caixas = [(ids[0], nomes[0], true), (ids[1], nomes[1], false)];
            let (_, medidos) = elisao::medindo(|| {
                cartao::com(LINHAS, || {
                    crate::widget::card_ink::com(ink, || {
                        paint_check_rows(
                            &mut scene, &mut ts, tema, &mut hit, &store, 0.0, w as f32, 0.0,
                            &caixas, sec,
                        )
                    })
                })
            });
            for nome in nomes {
                assert!(
                    medidos.iter().any(|m| m.texto == nome && m.coube()),
                    "{nome:?} saiu cortado a {w} px: {medidos:?}"
                );
            }
        }
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
