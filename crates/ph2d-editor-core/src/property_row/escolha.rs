//! ⭐⭐⭐ **A LINHA DE ESCOLHA — o nome à esquerda, o grupo segmentado na coluna do valor.**
//!
//! ⛔⛔ **Ordens do dono:** *«Label acima do campo numérico! Muito ruim!»* (2026-09-14, repetida a
//! 15/09 para as outras linhas do Inspector) e *«quanto ao alinhamento precisamos melhorar em
//! todos os lugares»* (2026-09-21).
//!
//! **Medido em 2026-09-23:** o Inspector sozinho tinha **OITO** implementações de «uma escolha com
//! nome» — `rows::seg_row` (nome ao lado), `anim::segmented_row`, `topdown::seg_row`,
//! `particles::seg_row`, `hud::seg_row`, `tween_editor::grupo` (nome POR CIMA), mais três inline
//! (`Strategy`, `Format`, as acções) — e a pintura do app inteiro achava **59** grupos
//! segmentados a começar na borda esquerda do conteúdo, onde deveria estar um nome. ⛔ O gate
//! textual que proíbe o nome por cima ficou cego a esta família **três** vezes, pelo NOME da
//! variável (`label_h` · `label_font` · `font`).
//!
//! # ⭐⭐ A lei é a que o `rows::seg_row` do Inspector já escrevia
//!
//! *«O controlo REFLUI e a coluna do rótulo não»* — um grupo com opções compridas quebra em mais de
//! uma fileira **dentro** da coluna do valor, e a altura devolvida é a que ele ocupou. *O que
//! mantém a secção legível é a coluna da esquerda estar sempre no mesmo `x`.* É também a lei do
//! Blender (`use_property_split`).
//!
//! ⚠️ **A coluna é da SECÇÃO** ([`Seccao`]), a mesma dos campos e das caixas — o `rows::seg_row`
//! antigo pedia a coluna de omissão (`property_row_columns`) e por isso não alinhava com um vizinho
//! cuja secção pedia a coluna mais larga.

use std::cell::{Cell, RefCell};

use crate::interaction::{HitIndex, WidgetStore};
use crate::widget::Seccao;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ROW_H_PX, Theme};
use ph2d_vector::VectorScene;

/// Uma escolha pintada pela porta: a 1.ª peça (que identifica o grupo) e quantas fileiras ele
/// ocupou na coluna do valor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EscolhaPintada {
    /// O id da primeira peça — é por ele que o censo geométrico reconhece o grupo.
    pub primeira: NodeId,
    /// Quantas peças.
    pub pecas: usize,
    /// Em quantas fileiras ele ocuparia AO LADO do nome — a grandeza que decide a forma.
    pub fileiras: usize,
    /// ⭐ A forma que a porta escolheu: `true` = ao lado do nome; `false` = a toda a largura, com o
    /// nome por cima (uma PALETA que não cabe numa fileira da coluna do valor).
    pub ao_lado: bool,
}

thread_local! {
    // ⚠️ A bandeira é da THREAD — ver o doc do `text_elide::elisao::ARMADO`, que pagou a GLOBAL.
    static ARMADO: Cell<bool> = const { Cell::new(false) };
    static PINTADAS: RefCell<Vec<EscolhaPintada>> = const { RefCell::new(Vec::new()) };
}

/// ⭐ **A porta de um gate:** arma, corre, desarma, devolve o que passou pela porta.
pub fn medindo<R>(f: impl FnOnce() -> R) -> (R, Vec<EscolhaPintada>) {
    ARMADO.set(true);
    PINTADAS.with_borrow_mut(Vec::clear);
    let r = f();
    let out = PINTADAS.with_borrow(Clone::clone);
    ARMADO.set(false);
    (r, out)
}

/// ⭐ **Em quantas fileiras esta escolha reflui ao lado do nome** — a MESMA conta que o pintor faz
/// (`segmented_row_counts` sobre as larguras naturais), sem pintar nada.
#[must_use]
pub fn fileiras_ao_lado(
    text_system: &mut TextSystem,
    x: f32,
    w: f32,
    rotulos: &[&str],
    seccao: Seccao,
) -> usize {
    let controlo = super::caixa_do_controlo(x, w, 0.0, seccao);
    let larguras = crate::widget::segmented_natural_widths(rotulos, text_system);
    crate::widget::segmented_row_counts(controlo_w(controlo), &larguras).len()
}

/// ⚠️ A largura do controlo é a da COLUNA, não a da caixa de UM campo: com `apenas_campos(1)` as
/// duas coincidem, e é isso que a porta pede.
fn controlo_w(r: Rect) -> f32 {
    r.w
}

/// ⭐⭐⭐ **A LINHA DE ESCOLHA — e a FORMA dela sai de uma medição, não de um gosto.**
///
/// ⛔⛔ **Medido em 2026-09-23 pela porta do produto, com as 22 escolhas do Inspector armado:** AO
/// LADO do nome, na coluna do valor (~`128 px` no dock de omissão), **só `4` cabem numa fileira**;
/// `9` ocupam duas, `4` três, `2` quatro, e as famílias de easing e os canais do Tween ocupariam
/// **seis e sete** — uma torre. *«Sempre ao lado» seria pior do que o que havia.*
///
/// ⇒ **o vale está entre UMA e DUAS fileiras**, e a lei é essa:
///
/// | ao lado do nome, ela… | forma |
/// |---|---|
/// | cabe numa fileira | **ao lado**, alinhada com os campos da secção |
/// | não cabe | **PALETA**: o nome por cima e o grupo a toda a largura |
///
/// ⚠️ **A decisão vive AQUI e só aqui** — nenhum chamador escolhe a forma.
///
/// ⛔ **DECISÃO DO DONO (2026-09-23): a PALETA fica.** Posta a alternativa — um MENU SUSPENSO ao lado
/// do nome para as escolhas que não cabem numa fileira (a forma do Godot e do Unity) —, depois de
/// ver o app inteiro convertido, ele escolheu *«o formato atual»*. ⇒ não reabrir sem uma ordem nova
/// dele; se ela vier, a troca continua a ser só nesta função.
///
/// `segmentos` é `(rótulo, aceso, id)`; com **nenhum** aceso o grupo diz «misto». Devolve o `y`
/// seguinte.
#[allow(clippy::too_many_arguments)]
pub fn paint_choice_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    segmentos: &[(&str, bool, NodeId)],
    seccao: Seccao,
) -> f32 {
    let rotulos: Vec<&str> = segmentos.iter().map(|(t, _, _)| *t).collect();
    let fileiras = fileiras_ao_lado(text_system, x, w, &rotulos, seccao);
    let ao_lado = fileiras <= 1;
    let fim = if ao_lado {
        let row =
            super::paint_label_row(scene, text_system, theme, x, w, y, ROW_H_PX, label, seccao);
        let seg_h = crate::widget::panel_chrome::paint_segmented_group_adaptive(
            row.control,
            segmentos,
            scene,
            text_system,
            theme,
            store,
            hit_index,
        );
        crate::widget::paint_decorator_dot(scene, theme, row.dot);
        y + seg_h.max(ROW_H_PX) + ph2d_tokens::control_gap_px()
    } else {
        // ⭐ A PALETA: o nome é o cabeçalho dela. O degrau entre o cabeçalho e o grupo é o
        //    `SECTION_LABEL_TO_CONTROL_PX` da casa, e não um `font + Xs` escrito em cada secção.
        let font = ph2d_tokens::TypeToken::Sm.px();
        crate::paint::paint_text(
            text_system,
            scene,
            label,
            x,
            y,
            font,
            w,
            crate::paint::resolve(ph2d_tokens::ColorToken::Text2, theme),
        );
        let gy = y + font + crate::widget::panel_chrome::SECTION_LABEL_TO_CONTROL_PX;
        let seg_h = crate::widget::panel_chrome::paint_segmented_group_adaptive(
            Rect::new(x, gy, w, ROW_H_PX),
            segmentos,
            scene,
            text_system,
            theme,
            store,
            hit_index,
        );
        gy + seg_h.max(ROW_H_PX) + ph2d_tokens::control_gap_px()
    };
    if ARMADO.get()
        && let Some((_, _, primeira)) = segmentos.first()
    {
        PINTADAS.with_borrow_mut(|v| {
            v.push(EscolhaPintada {
                primeira: *primeira,
                pecas: segmentos.len(),
                fileiras,
                ao_lado,
            });
        });
    }
    fim
}
