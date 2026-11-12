//! **A coluna de nomes de uma faixa (lane)** — o nome, o peso, o mudo, o `+` e a superfície do menu.
//!
//! Saiu de `stack_lane_paint.rs` (tecto de LOC) quando a onda 3 (`docs/Timeline/16_onda3_containers_arrange.md`)
//! lhe deu a cara dos cartões. No clássico é o código de sempre, ao bit; nos cartões a maquete B3 —
//! ícone de camadas, nome, peso numa pílula, olho e `+` discretos —, com os MESMOS ids e eventos e
//! cada alvo pelo menos do tamanho de antes (`as_faixas_nos_cartoes::os_alvos_das_faixas_nao_encolheram`).

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::{InteractiveState, TimelineHitKind};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon, paint_text_elided, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::LaneView;
use ph2d_tokens::{
    ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, Theme, TimelineTinta, TypeToken,
};

use crate::stack_lane_paint::{STRIP_PAD_Y, clipped};
use crate::{geom, ids};

/// The lane weight field's width.
const WEIGHT_W: f32 = 38.0; // LITERAL-PX-OK: "1.00" at TypeToken::Sm, plus the field's padding
/// How much of the label column the lane's controls take (weight + two buttons).
const CONTROLS_W: f32 = WEIGHT_W + (ROW_H_PX - STRIP_PAD_Y * 2.0) * 2.0 + 12.0; // LITERAL-PX-OK: three Xs gaps
/// A lane's weight step per drag notch, and the decimals it shows.
const WEIGHT_STEP: f64 = 0.01; // LITERAL-PX-OK: 1% of the [0, 1] range
/// Decimals shown in the weight field.
const WEIGHT_DECIMALS: usize = 2; // LITERAL-PX-OK: 1% resolution needs two

/// A largura de um botão da coluna (mudo, `+`, lápis, lixo): a de antes, nos dois modos — nos
/// cartões o alvo só cresce em altura, para a etiqueta da faixa não perder largura.
pub(crate) const BOTAO_W: f32 = ROW_H_PX - STRIP_PAD_Y * 2.0;
/// O ícone da coluna nos cartões: do tamanho da letra da faixa (a maquete B3: 12 px).
pub(crate) const ICONE_PX: f32 = TypeToken::Sm.px();

/// A dica do olho diz o que o clique FARÁ, pelo estado de agora — a mesma nos dois modos.
fn dica_do_olho(ctx: &mut PaintCtx, index: usize, muted: bool) {
    let id = ids::TIMELINE_LANE_MUTE[index];
    let dica = ph2d_i18n::tr(if muted {
        "panel.timeline.lane_unmute"
    } else {
        "panel.timeline.lane_mute"
    });
    let store = ctx.host.store_mut();
    if store.tooltip_for(id) != Some(dica) {
        store.set_tooltip(id, dica);
    }
}

/// Paint one lane's label column — the classic one, or the cards one.
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    g: &geom::Geom,
    lane: &LaneView,
    index: usize,
    row: Rect,
) {
    dica_do_olho(ctx, index, lane.muted);
    if ph2d_editor_core::panel::cards::ferramenta::cards_on() {
        paint_cartoes(ctx, theme, g, lane, index, row);
    } else {
        paint_classico(ctx, theme, g, lane, index, row);
    }
}

/// O pintor de sempre (`PH2D_CARDS=0`), movido sem mudar uma linha.
fn paint_classico(
    ctx: &mut PaintCtx,
    theme: Theme,
    g: &geom::Geom,
    lane: &LaneView,
    index: usize,
    row: Rect,
) {
    fill_rounded_rect(
        ctx.scene,
        row,
        ph2d_editor_core::paint::frame_radius(theme, Radius::Xs.px()),
        resolve(ColorToken::BgElev, theme),
    );

    // ── the label column: name, and what the lane DOES ──
    let font = TypeToken::Sm.px();
    let text_y = row.y + (row.h - font) * 0.5;
    let dim = lane.muted || lane.weight <= 0.0;
    // The name's budget stops at the controls, not at the column's edge — text that
    // elides UNDER a button looks like a button with a word behind it.
    let name_w = (g.label_w - CONTROLS_W - Spacing::Sm.px() * 2.0).max(0.0);
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        &lane.name,
        row.x + Spacing::Sm.px(),
        text_y,
        font,
        name_w,
        resolve(
            if dim {
                ColorToken::Text3
            } else {
                ColorToken::Text1
            },
            theme,
        ),
    );

    // ── the lane's controls, right-aligned in the label column ──
    //
    // The WEIGHT is here and the MODE is in the right-click menu, and that split is
    // the point: weight is a number you nudge and re-nudge while watching the
    // canvas, so it belongs under the pointer; mode is set once per lane. Delete
    // Lane is in the menu too — not because deleting is rare, but because a row
    // this narrow has no room for a third button, and a control that only appears
    // when the column is wide enough is worse than one that is always one click in.
    let btn_w = ROW_H_PX - STRIP_PAD_Y * 2.0;
    let btn_y = row.y + STRIP_PAD_Y;
    let add = Rect::new(
        row.x + g.label_w - btn_w - Spacing::Xs.px(),
        btn_y,
        btn_w,
        btn_w,
    );
    let mute = Rect::new(add.x - btn_w - Spacing::Xs.px(), btn_y, btn_w, btn_w);
    let weight = Rect::new(
        mute.x - WEIGHT_W - Spacing::Xs.px(),
        row.y + STRIP_PAD_Y,
        WEIGHT_W,
        (row.h - STRIP_PAD_Y * 2.0).max(0.0),
    );
    // Every hit in the label column is clipped to the visible band: a lane scrolled
    // half under the ruler must not register its mute where "+ Lane" is painted.
    let band = Rect::new(g.rows.x, g.rows.y, g.label_w, g.rows.h);
    paint_weight(
        ctx,
        theme,
        ids::TIMELINE_LANE_WEIGHT[index],
        weight,
        band,
        lane,
    );
    // Os dois botões CARREGAM ÍCONE. Sem ele são dois quadrados vazios idênticos, e o artista
    // tem de descobrir por eliminação qual silencia a faixa e qual acrescenta um clipe (o Enio
    // achou isso no smoke). Um botão sem rótulo nem ícone não é um botão, é um enigma.
    paint_lane_button(
        ctx,
        theme,
        ids::TIMELINE_LANE_MUTE[index],
        mute,
        band,
        lane.muted,
        // Mudo = a faixa não contribui NADA (não é peso zero — é estar fora da pilha). O olho
        // fechado é o mesmo verbo que a hierarquia já usa para "isto não conta".
        if lane.muted {
            IconId::EyeClosed
        } else {
            IconId::Eye
        },
    );
    paint_lane_button(
        ctx,
        theme,
        ids::TIMELINE_LANE_ADD_STRIP[index],
        add,
        band,
        false,
        IconId::Add,
    );

    // The label itself is the right-click surface (mode, Delete Lane). Its rect
    // STOPS where the controls start, so it cannot steal their clicks — the hit
    // index does not need a z-order rule it would only have to remember.
    let label = Rect::new(row.x, row.y, (weight.x - row.x).max(0.0), row.h);
    register_label(ctx, index, row, label, band);
}

/// The label surface (right-click: mode, Delete Lane), clipped to the visible band.
fn register_label(ctx: &mut PaintCtx, index: usize, row: Rect, label: Rect, band: Rect) {
    if let Some(r) = clipped(label, band) {
        ctx.host.store_mut().register(
            ids::TIMELINE_LANE_ROW[index],
            InteractiveState::TimelineSurface {
                parent: ph2d_editor_core::ids::TIMELINE_PANEL,
                kind: TimelineHitKind::LaneHeader { lane: index },
                canvas: row,
            },
        );
        ctx.host
            .hit_index_mut()
            .register(ids::TIMELINE_LANE_ROW[index], r);
    }
}

/// ⭐ **Onde mora cada peça da coluna de uma faixa nos cartões** (maquete B3). Pura: o pintor e o
/// teste do centro leem-na. Os alvos têm a largura de antes e a altura da faixa menos a margem da
/// peça; a etiqueta acaba onde o peso começa, como no clássico.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ColunaDaFaixa {
    /// O ícone de camadas.
    pub icone: Rect,
    /// Onde o nome começa e quanto pode ocupar.
    pub nome: (f32, f32),
    /// O `y` que o pintor de texto recebe para o nome (`TypeToken::Sm`).
    pub texto_y: f32,
    /// O alvo do campo do peso.
    pub peso: Rect,
    /// O alvo do olho (mudo).
    pub mudo: Rect,
    /// O alvo do `+` (pôr a fonte no cursor).
    pub mais: Rect,
    /// A superfície do menu da faixa.
    pub etiqueta: Rect,
}

/// A coluna da faixa `row` numa coluna de nomes de largura `label_w`.
pub(crate) fn coluna_da_faixa(row: Rect, label_w: f32) -> ColunaDaFaixa {
    let (alvo_y, alvo_h) = alvo_vertical(row);
    let xs = Spacing::Xs.px();
    let mais = Rect::new(row.x + label_w - BOTAO_W - xs, alvo_y, BOTAO_W, alvo_h);
    let mudo = Rect::new(mais.x - BOTAO_W - xs, alvo_y, BOTAO_W, alvo_h);
    let peso = Rect::new(mudo.x - WEIGHT_W - xs, alvo_y, WEIGHT_W, alvo_h);
    let icone = icone_em(row);
    let nome_x = icone.x + ICONE_PX + xs;
    ColunaDaFaixa {
        icone,
        nome: (nome_x, (peso.x - nome_x - Spacing::Sm.px()).max(0.0)),
        texto_y: texto_y(row, TypeToken::Sm.px()),
        peso,
        mudo,
        mais,
        etiqueta: Rect::new(row.x, row.y, (peso.x - row.x).max(0.0), row.h),
    }
}

/// ⭐ O piso da coluna nos cartões para as faixas `nomes`: o nome mais largo INTEIRO (a `TypeToken::Sm`)
/// mais o que [`coluna_da_faixa`] põe à volta dele — lido da própria conta, não repetido.
pub(crate) fn piso<'a>(
    nomes: impl Iterator<Item = &'a str>,
    largura: &mut dyn FnMut(&str, f32) -> f32,
) -> f32 {
    let k = coluna_da_faixa(Rect::new(0.0, 0.0, 0.0, geom::faixa_h()), 0.0);
    let fixo = k.nome.0 - k.peso.x + Spacing::Sm.px();
    let nome = nomes
        .map(|n| largura(n, TypeToken::Sm.px()).ceil())
        .fold(0.0, f32::max);
    fixo + nome
}

/// O topo e a altura dos alvos da coluna numa faixa: a faixa menos a margem da peça.
pub(crate) fn alvo_vertical(row: Rect) -> (f32, f32) {
    let m = geom::peca_margem();
    (row.y + m, (row.h - m * 2.0).max(0.0))
}

/// O ícone da esquerda da coluna, centrado na altura da faixa.
pub(crate) fn icone_em(row: Rect) -> Rect {
    Rect::new(
        row.x + Spacing::Sm.px(),
        row.y + (row.h - ICONE_PX) * 0.5,
        ICONE_PX,
        ICONE_PX,
    )
}

/// O `y` do pintor de texto que põe o meio das maiúsculas de `font` no meio de `r`.
pub(crate) fn texto_y(r: Rect, font: f32) -> f32 {
    r.y + ph2d_editor_core::paint::folga_do_texto(r.h, font)
}

/// Um glifo de `ICONE_PX` centrado em `r`.
pub(crate) fn glifo_em(r: Rect) -> Rect {
    Rect::new(
        r.x + (r.w - ICONE_PX) * 0.5,
        r.y + (r.h - ICONE_PX) * 0.5,
        ICONE_PX,
        ICONE_PX,
    )
}

/// ⭐ O fundo de uma linha do stack nos cartões: a célula da coluna, a fita e o fio de baixo de
/// cada um — a folha da aba Keys sem zebra (`linhas::pinta_linha_maquete`). `celula` troca a tinta
/// da linha inteira (a escolhida da lista).
pub(crate) fn fundo(
    ctx: &mut PaintCtx,
    theme: Theme,
    g: &geom::Geom,
    row: Rect,
    celula: Option<TimelineTinta>,
) {
    let t = |x| crate::folha_tinta::tinta(x, theme);
    let fio = ph2d_editor_core::panel::cards::banda::SEPARADOR_W;
    let fita_w = (row.w - g.label_w).max(0.0);
    let (nomes, fita) = celula.map_or(
        (
            TimelineTinta::NomesPropriedade,
            TimelineTinta::FitaPropriedade,
        ),
        |c| (c, c),
    );
    fill_rounded_rect(
        ctx.scene,
        Rect::new(row.x, row.y, g.label_w, row.h),
        0.0,
        t(nomes),
    );
    fill_rounded_rect(
        ctx.scene,
        Rect::new(row.x + g.label_w, row.y, fita_w, row.h),
        0.0,
        t(fita),
    );
    let fy = row.y + row.h - fio;
    fill_rounded_rect(
        ctx.scene,
        Rect::new(row.x, fy, g.label_w, fio),
        0.0,
        t(TimelineTinta::NomesFio),
    );
    fill_rounded_rect(
        ctx.scene,
        Rect::new(row.x + g.label_w, fy, fita_w, fio),
        0.0,
        t(TimelineTinta::FitaFio),
    );
}

/// ⭐ Um botão discreto da coluna nos cartões: só o glifo; um fundo de pílula sob o rato. O alvo é o
/// rect inteiro, recortado à banda, e o registo o de sempre (`Plain`).
pub(crate) fn botao_discreto(
    ctx: &mut PaintCtx,
    theme: Theme,
    id: ph2d_editor_core::NodeId,
    (r, band): (Rect, Rect),
    (icone, tinta): (IconId, TimelineTinta),
) {
    if ctx.host.store().hover_live(id) > 0.0 {
        fill_rounded_rect(
            ctx.scene,
            r,
            ph2d_tokens::TIMELINE_PESO_RAIO_PX,
            crate::folha_tinta::tinta(TimelineTinta::PesoPilula, theme),
        );
    }
    paint_icon(
        ctx.scene,
        icone,
        glifo_em(r),
        crate::folha_tinta::tinta(tinta, theme),
        StrokeToken::Default.px(),
    );
    ctx.host.store_mut().register(id, InteractiveState::Plain);
    if let Some(hit) = clipped(r, band) {
        ctx.host.hit_index_mut().register(id, hit);
    }
}

/// ⭐ A coluna de uma faixa nos cartões (maquete B3): camadas · nome · peso na pílula · olho · `+`.
fn paint_cartoes(
    ctx: &mut PaintCtx,
    theme: Theme,
    g: &geom::Geom,
    lane: &LaneView,
    index: usize,
    row: Rect,
) {
    fundo(ctx, theme, g, row, None);
    let c = coluna_da_faixa(row, g.label_w);
    let dim = lane.muted || lane.weight <= 0.0;
    let texto = if dim {
        TimelineTinta::TextoValor
    } else {
        TimelineTinta::TextoPropriedade
    };
    let cor = crate::folha_tinta::tinta(texto, theme);
    paint_icon(
        ctx.scene,
        IconId::Layers,
        c.icone,
        cor,
        StrokeToken::Default.px(),
    );
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        &lane.name,
        c.nome.0,
        c.texto_y,
        TypeToken::Sm.px(),
        c.nome.1,
        cor,
    );
    let band = Rect::new(g.rows.x, g.rows.y, g.label_w, g.rows.h);
    paint_weight(
        ctx,
        theme,
        ids::TIMELINE_LANE_WEIGHT[index],
        c.peso,
        band,
        lane,
    );
    let olho = if lane.muted {
        (IconId::EyeClosed, TimelineTinta::OlhoMudo)
    } else {
        (IconId::Eye, TimelineTinta::TextoValor)
    };
    botao_discreto(
        ctx,
        theme,
        ids::TIMELINE_LANE_MUTE[index],
        (c.mudo, band),
        olho,
    );
    botao_discreto(
        ctx,
        theme,
        ids::TIMELINE_LANE_ADD_STRIP[index],
        (c.mais, band),
        (IconId::Add, TimelineTinta::TextoValor),
    );
    register_label(ctx, index, row, c.etiqueta, band);
}

/// A pílula do peso dentro do alvo `peso`, para um texto de largura `texto_w` (maquete: 26 × 15).
pub(crate) fn pilula(peso: Rect, texto_w: f32) -> Rect {
    let w = (texto_w + Spacing::Xs.px() * 2.0).min(peso.w);
    let h = (TypeToken::Xs.px() + Spacing::Xxs.px() * 2.0).min(peso.h);
    Rect::new(
        peso.x + (peso.w - w) * 0.5,
        peso.y + (peso.h - h) * 0.5,
        w,
        h,
    )
}

/// The lane's weight, as a bounded number field: drag it, or type into it.
///
/// The range is REGISTERED (`set_number_range`), not merely clamped afterwards —
/// dispatch scales a body-drag by the field's range, so a field that never
/// declared one drags at the default scale and a lane's `[0, 1]` weight would fly
/// past both ends in a few pixels ([[reference_number_input_register_range]]).
///
/// Nos cartões o número mora numa pílula (maquete B3) e o campo só aparece a escrever; o alvo é o
/// rect inteiro, nos dois modos.
pub(crate) fn paint_weight(
    ctx: &mut PaintCtx,
    theme: Theme,
    id: ph2d_editor_core::NodeId,
    r: Rect,
    band: Rect,
    lane: &LaneView,
) {
    use ph2d_editor_core::widget::{NumberInput, paint_number_input_with_buffer, showcase};
    {
        let store = ctx.host.store_mut();
        crate::transport::mirror_number(store, id, lane.weight, WEIGHT_DECIMALS);
        store.set_number_range(id, 0.0, 1.0, WEIGHT_STEP);
    }
    let (state, _v, buf, caret, anchor) = showcase::read_number_input(ctx.host.store(), id);
    let buf = buf.to_string();
    let a_escrever = matches!(state, ph2d_editor_core::widget::TextInputState::Focused)
        && ctx.host.store().number_input_drag().map(|d| d.id) != Some(id);
    if ph2d_editor_core::panel::cards::ferramenta::cards_on() && !a_escrever {
        paint_pilula(ctx, theme, r, &buf);
    } else {
        let input = NumberInput::new(id, "", lane.weight)
            .step(WEIGHT_STEP)
            .visual((state, ctx.host.store().hover_live(id)));
        paint_number_input_with_buffer(
            &input,
            Some(&buf),
            ctx.host.store().number_input_mixed(id),
            caret,
            anchor,
            r,
            ctx.scene,
            ctx.text_system,
            theme,
        );
    }
    if let Some(hit) = clipped(r, band) {
        ctx.host.hit_index_mut().register(id, hit);
    }
}

/// O peso nos cartões: o número de sempre numa pílula `PesoPilula`.
fn paint_pilula(ctx: &mut PaintCtx, theme: Theme, peso: Rect, texto: &str) {
    let font = TypeToken::Xs.px();
    let texto_w = ctx.text_system.prefix_width(texto, font);
    let p = pilula(peso, texto_w);
    fill_rounded_rect(
        ctx.scene,
        p,
        ph2d_tokens::TIMELINE_PESO_RAIO_PX,
        crate::folha_tinta::tinta(TimelineTinta::PesoPilula, theme),
    );
    let w = texto_w.min(p.w);
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        texto,
        p.x + (p.w - w) * 0.5,
        texto_y(p, font),
        font,
        p.w,
        crate::folha_tinta::tinta(TimelineTinta::TextoPropriedade, theme),
    );
}

/// One of a lane's header controls. Square, and `on` fills it — the mute reads as
/// pressed, and a lane that is muted is dimmed everywhere else too.
fn paint_lane_button(
    ctx: &mut PaintCtx,
    theme: Theme,
    id: ph2d_editor_core::NodeId,
    r: Rect,
    band: Rect,
    on: bool,
    icon: IconId,
) {
    fill_rounded_rect(
        ctx.scene,
        r,
        ph2d_editor_core::paint::frame_radius(theme, Radius::Xs.px()),
        resolve(
            if on {
                ColorToken::Accent
            } else {
                ColorToken::Bg3
            },
            theme,
        ),
    );
    // ⭐ Pela porta do TEMA: o botão é plano num tema moderno.
    ph2d_editor_core::paint::stroke_frame(
        ctx.scene,
        r,
        ph2d_editor_core::paint::frame_radius(theme, Radius::Xs.px()),
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );
    // O glifo, com folga para não encostar na borda do botão.
    let pad = Spacing::Xs.px();
    paint_icon(
        ctx.scene,
        icon,
        Rect::new(
            r.x + pad,
            r.y + pad,
            (r.w - pad * 2.0).max(0.0),
            (r.h - pad * 2.0).max(0.0),
        ),
        resolve(
            if on {
                ColorToken::Text1
            } else {
                ColorToken::Text3
            },
            theme,
        ),
        StrokeToken::Default.px(),
    );
    ctx.host.store_mut().register(id, InteractiveState::Plain);
    if let Some(hit) = clipped(r, band) {
        ctx.host.hit_index_mut().register(id, hit);
    }
}

#[cfg(test)]
#[path = "faixas_coluna_tests.rs"]
mod tests;
