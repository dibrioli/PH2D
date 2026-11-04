//! ⭐ **O texto em cartões** (fatia G4, `docs/CartoesUI/12_VECTOR_PLANO.md`): Text, Font, Paragraph, Axes
//! e Text on Path. A forma é a do painel de tipografia do Figma (a família, os números dois a dois, o
//! alinhamento em ícones ao lado do modo da largura), dentro dos mesmos cartões. Mesmos ids e eventos:
//! cada campo é o chip ligado à barra de sempre, e é a barra que avisa o painel.
//!
//! As perguntas de antes continuam aqui: Weight só numa fonte que publica `wght`, sem Width num texto
//! preso a um caminho, Axes até `MAX_TEXT_VARIATION_AXES`.

use crate::cards_fx::{com_unidade, fila};
use crate::cards_tinta::{aberto, dica, escolha, par_de_botoes, seg, seg_icone, semeia, spec};
use crate::{ids, state};
use ph2d_a11y::NodeId;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Widget, corpo};
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_i18n::tr;
use ph2d_tool_vector::params::{
    DEFAULT_TEXT_LINE_HEIGHT, DEFAULT_TEXT_SIZE, DEFAULT_TEXT_TRACKING, DEFAULT_TEXT_WEIGHT,
    DEFAULT_TEXT_WRAP,
};
use ph2d_vec_text::TextAlign;

const PASSO_MEDIO: f64 = 0.1; // LITERAL-PX-OK: passo do valor (tamanho, fator), não geometria
const PASSO_FINO: f64 = 0.01; // LITERAL-PX-OK: passo do arrasto, no domínio do documento
const PASSO_DO_PESO: f64 = 10.0; // LITERAL-PX-OK: passo da seta no eixo `wght` (100..900)
const PASSOS_DO_EIXO: f64 = 40.0; // LITERAL-PX-OK: em quantos passos o arrasto atravessa o eixo, o do `seed_number_fields`
const PASSO_DA_LARGURA: f64 = 1.0; // LITERAL-PX-OK: passo da seta na largura de quebra, em mundo

/// Com o store MUTÁVEL, antes de pintar: as marcas e as dicas. As sementes são as de sempre (`paint.rs`).
pub(crate) fn prepare(store: &mut WidgetStore) {
    for (id, seta, scrub, fabrica) in [
        (
            ids::VECTOR_TEXT_SIZE_NUM,
            PASSO_MEDIO,
            PASSO_FINO,
            DEFAULT_TEXT_SIZE,
        ),
        (
            ids::VECTOR_TEXT_WEIGHT_NUM,
            PASSO_DO_PESO,
            1.0,
            DEFAULT_TEXT_WEIGHT,
        ),
        (
            ids::VECTOR_TEXT_LINE_HEIGHT_NUM,
            PASSO_MEDIO,
            PASSO_FINO,
            DEFAULT_TEXT_LINE_HEIGHT,
        ),
        (
            ids::VECTOR_TEXT_TRACKING_NUM,
            PASSO_FINO,
            PASSO_FINO,
            DEFAULT_TEXT_TRACKING,
        ),
        (
            ids::VECTOR_TEXT_WRAP_W_NUM,
            PASSO_DA_LARGURA,
            PASSO_MEDIO,
            DEFAULT_TEXT_WRAP,
        ),
        (ids::VECTOR_TEXTPATH_OFFSET_NUM, PASSO_FINO, PASSO_FINO, 0.0),
    ] {
        store.mark_card_field(id, spec(seta, scrub, Some(fabrica)));
    }
    // O caminho não tem semente própria: o campo do cartão lê o store.
    if state::linked() {
        semeia(
            store,
            ids::VECTOR_TEXTPATH_OFFSET_NUM,
            state::offset(),
            false,
        );
    }
    let faixas: Vec<(f64, f64)> =
        state::with_text_axes(|a| a.iter().map(|s| (s.min, s.max)).collect());
    for (i, (min, max)) in faixas
        .into_iter()
        .enumerate()
        .take(crate::ids::MAX_TEXT_VARIATION_AXES)
    {
        let passo = ((max - min) / PASSOS_DO_EIXO).max(PASSO_FINO);
        store.mark_card_field(crate::ids::vector_text_axis_id(i), spec(passo, passo, None));
    }
    for (id, chave) in [
        (crate::ids::VECTOR_TEXT_FONT_PREV, "card.vector.font_prev"),
        (crate::ids::VECTOR_TEXT_FONT_NEXT, "card.vector.font_next"),
        (
            crate::ids::VECTOR_TEXT_ALIGN_LEFT,
            "card.vector.text_align_left",
        ),
        (
            crate::ids::VECTOR_TEXT_ALIGN_CENTER,
            "card.vector.text_align_center",
        ),
        (
            crate::ids::VECTOR_TEXT_ALIGN_RIGHT,
            "card.vector.text_align_right",
        ),
    ] {
        dica(store, id, chave);
    }
}

/// ⭐ **O cartão Text**: a frase, só de leitura (escreve-se no canvas), e o tamanho e o peso.
pub(crate) fn text(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let frase = state::current_text().unwrap_or_default();
    let y = if frase.trim().is_empty() {
        c.hint(tr("panel.vector.text.click_the_canvas_and_type"), y)
    } else {
        let numa_linha = frase.replace('\n', " / ");
        c.full(None, Widget::Readout { text: &numa_linha }, y)
    };
    // Tamanho e largura são de MUNDO: não passam pela conversão do Transform, por isso sempre em metros.
    let tamanho = com_unidade(tr("panel.vector.text.size"), tr("card.vector.unit.metres"));
    let mut par = vec![(
        tamanho.as_str(),
        ids::VECTOR_TEXT_SIZE_NUM,
        crate::ids::VECTOR_TEXT_SIZE,
    )];
    // Sem `fvar` o `skrifa` ignora o eixo: numa fonte estática o Weight seria um campo morto.
    if state::text_has_weight_axis() {
        par.push((
            tr("panel.vector.text.weight"),
            ids::VECTOR_TEXT_WEIGHT_NUM,
            crate::ids::VECTOR_TEXT_WEIGHT,
        ));
    }
    fila(c, &par, y)
}

/// ⭐ **O cartão Font**: `‹ família ›`, e Import. As setas ciclam; a caixa abre a lista de sempre, cada
/// família no próprio contorno (`font_dropdown`, no passe diferido).
pub(crate) fn font(c: &mut CardCtx<'_>, y: f32) -> f32 {
    const FAMILIA: &[NodeId] = &[crate::ids::VECTOR_TEXT_FONT_DD];
    let nome = state::current_text_font().unwrap_or_default();
    let anterior = [seg_icone(
        crate::ids::VECTOR_TEXT_FONT_PREV,
        IconId::ChevronLeft,
        false,
    )];
    let seguinte = [seg_icone(
        crate::ids::VECTOR_TEXT_FONT_NEXT,
        IconId::ChevronRight,
        false,
    )];
    let y = c.row(
        &[
            Cell::new(Widget::Bare(&anterior)),
            Cell::new(Widget::Dropdown {
                id: FAMILIA[0],
                text: &nome,
                options: &[],
            })
            .label(tr("card.vector.font_family"))
            .key(FAMILIA),
            Cell::new(Widget::Bare(&seguinte)),
        ],
        y,
    );
    if let Some(chip) = aberto(c, FAMILIA[0]) {
        state::set_pending_font_dd(Some(chip));
    }
    c.full(
        None,
        Widget::Button {
            id: crate::ids::VECTOR_TEXT_FONT_IMPORT,
            text: tr("panel.vector.text.import_font"),
            primary: false,
        },
        y,
    )
}

/// ⭐ **O cartão Paragraph**: entrelinha e tracking, depois o alinhamento em ícones e o modo da largura.
pub(crate) fn paragraph(c: &mut CardCtx<'_>, y: f32) -> f32 {
    const ALINHA: &[NodeId] = &[
        crate::ids::VECTOR_TEXT_ALIGN_LEFT,
        crate::ids::VECTOR_TEXT_ALIGN_CENTER,
        crate::ids::VECTOR_TEXT_ALIGN_RIGHT,
    ];
    const LARGURA: &[NodeId] = &[
        crate::ids::VECTOR_TEXT_WRAP_AUTO,
        crate::ids::VECTOR_TEXT_WRAP_FIXED,
    ];
    let entrelinha = com_unidade(
        tr("panel.vector.text.line_height"),
        tr("card.vector.unit.times"),
    );
    let tracking = com_unidade(tr("panel.vector.text.tracking"), tr("card.vector.unit.em"));
    let y = fila(
        c,
        &[
            (
                entrelinha.as_str(),
                ids::VECTOR_TEXT_LINE_HEIGHT_NUM,
                crate::ids::VECTOR_TEXT_LINE_HEIGHT,
            ),
            (
                tracking.as_str(),
                ids::VECTOR_TEXT_TRACKING_NUM,
                crate::ids::VECTOR_TEXT_TRACKING,
            ),
        ],
        y,
    );
    let vigente = state::current_text_align().unwrap_or(TextAlign::Left);
    let alinhamento = [
        (ALINHA[0], IconId::CardTextLeft, TextAlign::Left),
        (ALINHA[1], IconId::CardTextCenter, TextAlign::Center),
        (ALINHA[2], IconId::CardTextRight, TextAlign::Right),
    ]
    .map(|(id, icone, a)| seg_icone(id, icone, a == vigente));
    let alinha = Cell::new(Widget::Icons(&alinhamento))
        .label(tr("panel.vector.text.align"))
        .key(ALINHA);
    // Num caminho o motor descarta a largura: o texto segue o arco e não há caixa para refluir.
    if state::linked() {
        return c.row(&[alinha], y);
    }
    let quebra = state::current_text_wrap();
    let modos = [
        seg(LARGURA[0], tr("panel.vector.text.auto"), quebra.is_none()),
        seg(LARGURA[1], tr("panel.vector.text.fixed"), quebra.is_some()),
    ];
    let y = c.row(
        &[
            alinha,
            Cell::new(Widget::Choice(&modos))
                .label(tr("panel.vector.text.width"))
                .key(LARGURA),
        ],
        y,
    );
    if quebra.is_none() {
        return y;
    }
    let nome = com_unidade(
        tr("panel.vector.text.wrap_width"),
        tr("card.vector.unit.metres"),
    );
    fila(
        c,
        &[(
            nome.as_str(),
            ids::VECTOR_TEXT_WRAP_W_NUM,
            crate::ids::VECTOR_TEXT_WRAP_W,
        )],
        y,
    )
}

/// ⭐ **O cartão Axes**: um campo por eixo da fonte, dois a dois, até ao tecto do registo. O corpo de
/// antes também não marcava: os eixos ainda não têm canal na tabela de `marcas*.rs`.
pub(crate) fn axes(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let nomes: Vec<String> = state::with_text_axes(|a| a.iter().map(|s| s.name.clone()).collect());
    let celulas: Vec<Cell<'_>> = nomes
        .iter()
        .take(crate::ids::MAX_TEXT_VARIATION_AXES)
        .enumerate()
        .map(|(i, nome)| {
            Cell::new(Widget::Field {
                id: crate::ids::vector_text_axis_id(i),
                glyph: Glyph::Text(corpo::letra_da_grandeza(nome)),
            })
            .label(nome)
        })
        .collect();
    celulas.chunks(2).fold(y, |y, par| c.row(par, y))
}

/// ⭐ **O cartão Text on Path**. Solto: as portas de prender (a automática só com um texto E um caminho
/// na selecção; o Picker sempre). Preso: onde começa, de que lado, e como sair.
pub(crate) fn textpath(c: &mut CardCtx<'_>, y: f32) -> f32 {
    const LADO: &[NodeId] = &[
        crate::ids::VECTOR_TEXTPATH_FLIP_OFF,
        crate::ids::VECTOR_TEXTPATH_FLIP,
    ];
    if !state::linked() {
        let escolher = (
            crate::ids::VECTOR_TEXTPATH_PICK,
            tr("panel.vector.textpath.pick_path"),
        );
        if state::can_link() {
            let prender = (
                crate::ids::VECTOR_TEXTPATH_LINK,
                tr("panel.vector.textpath.text_on_path"),
            );
            return par_de_botoes(c, prender, escolher, y);
        }
        return c.full(
            None,
            Widget::Button {
                id: escolher.0,
                text: escolher.1,
                primary: false,
            },
            y,
        );
    }
    let y = fila(
        c,
        &[(
            tr("panel.vector.textpath.offset"),
            ids::VECTOR_TEXTPATH_OFFSET_NUM,
            crate::ids::VECTOR_TEXTPATH_OFFSET,
        )],
        y,
    );
    let virado = state::flip();
    let lados = [
        seg(LADO[0], tr("panel.vector.textpath.this_side"), !virado),
        seg(LADO[1], tr("panel.vector.textpath.other_side"), virado),
    ];
    let y = escolha(c, tr("panel.vector.textpath.side"), LADO, &lados, y);
    c.full(
        None,
        Widget::Button {
            id: crate::ids::VECTOR_TEXTPATH_DETACH,
            text: tr("panel.vector.textpath.detach_from_path"),
            primary: false,
        },
        y,
    )
}
