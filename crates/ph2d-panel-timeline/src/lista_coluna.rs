//! **A coluna de nomes de uma linha da lista de containers** — o nome, o lápis e o lixo.
//!
//! Saiu de `container_list.rs` quando a onda 3 (`docs/Timeline/16_onda3_containers_arrange.md`, maquete
//! A3) lhe deu a cara dos cartões: cubo · nome · «used 2×»/«not placed» à direita · lápis e lixo
//! discretos, a escolhida (a fonte que o `+` da faixa põe) em `ChipEscolhido`. No clássico é o código
//! de sempre, ao bit; os ids, os eventos e os alvos (pelo menos do tamanho de antes) não mudam.

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon, paint_text_elided, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::ContainerView;
use ph2d_tokens::{
    ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, Theme, TimelineTinta, TypeToken,
};

use crate::faixas_coluna::{BOTAO_W, ICONE_PX, alvo_vertical, botao_discreto, icone_em, texto_y};
use crate::stack_lane_paint::{STRIP_PAD_Y, clipped};
use crate::{geom, ids};

/// Paint one container row's label column (and, classic or cards, the row's background).
/// `escolhido` = this container is the source the lane's `+` places.
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    g: &geom::Geom,
    c: &ContainerView,
    (index, escolhido): (usize, bool),
    row: Rect,
) {
    if ph2d_editor_core::panel::cards::ferramenta::cards_on() {
        paint_cartoes(ctx, theme, g, c, (index, escolhido), row);
    } else {
        paint_classico(ctx, theme, g, c, index, row);
    }
}

/// O pintor de sempre (`PH2D_CARDS=0`), movido sem mudar uma linha.
fn paint_classico(
    ctx: &mut PaintCtx,
    theme: Theme,
    g: &geom::Geom,
    c: &ContainerView,
    index: usize,
    row: Rect,
) {
    fill_rounded_rect(
        ctx.scene,
        row,
        ph2d_editor_core::paint::frame_radius(theme, Radius::Xs.px()),
        resolve(ColorToken::BgElev, theme),
    );

    // ── the label column: the name, then the pencil and the trash ──
    //
    // Right-aligned like the lane row's controls, and the trash takes the far edge — the
    // clip cluster's `[✎][🗑]` order, one bar down.
    let btn_w = ROW_H_PX - STRIP_PAD_Y * 2.0;
    let btn_y = row.y + STRIP_PAD_Y;
    let trash = Rect::new(
        row.x + g.label_w - btn_w - Spacing::Xs.px(),
        btn_y,
        btn_w,
        btn_w,
    );
    let pencil = Rect::new(trash.x - btn_w - Spacing::Xs.px(), btn_y, btn_w, btn_w);
    let font = TypeToken::Sm.px();
    // The name's budget stops where the buttons start: text that elides UNDER a button reads
    // as a button with a word behind it (the lane row learned this first).
    let name_w = (pencil.x - row.x - Spacing::Sm.px() * 2.0).max(0.0);
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        &c.name,
        row.x + Spacing::Sm.px(),
        row.y + (row.h - font) * 0.5,
        font,
        name_w,
        resolve(ColorToken::Text1, theme),
    );
    // The band every hit is clipped to: a row scrolled half under the ruler must not register
    // its buttons where "+ Container" is painted.
    let band = Rect::new(g.rows.x, g.rows.y, g.label_w, g.rows.h);
    paint_row_button(
        ctx,
        theme,
        ids::TIMELINE_CONT_RENAME[index],
        pencil,
        band,
        IconId::Text,
    );
    paint_row_button(
        ctx,
        theme,
        ids::TIMELINE_CONT_DELETE[index],
        trash,
        band,
        IconId::Trash,
    );
}

/// ⭐ **Onde mora cada peça da coluna de um container nos cartões** (maquete A3). Pura: o pintor e o
/// teste do centro leem-na.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ColunaDoContainer {
    /// O cubo.
    pub icone: Rect,
    /// Onde o nome começa.
    pub nome_x: f32,
    /// Onde o «used 2×» acaba (alinhado à direita, antes do lápis).
    pub uso_fim: f32,
    /// O `y` do pintor de texto para o nome (`Sm`) e para o uso (`Xs`).
    pub texto_y: (f32, f32),
    /// Os alvos do lápis e do lixo.
    pub lapis: Rect,
    pub lixo: Rect,
}

/// A coluna da linha `row` numa coluna de nomes de largura `label_w`.
pub(crate) fn coluna_do_container(row: Rect, label_w: f32) -> ColunaDoContainer {
    let (alvo_y, alvo_h) = alvo_vertical(row);
    let xs = Spacing::Xs.px();
    let lixo = Rect::new(row.x + label_w - BOTAO_W - xs, alvo_y, BOTAO_W, alvo_h);
    let lapis = Rect::new(lixo.x - BOTAO_W - xs, alvo_y, BOTAO_W, alvo_h);
    let icone = icone_em(row);
    ColunaDoContainer {
        icone,
        nome_x: icone.x + ICONE_PX + xs,
        uso_fim: lapis.x - xs,
        texto_y: (
            texto_y(row, TypeToken::Sm.px()),
            texto_y(row, TypeToken::Xs.px()),
        ),
        lapis,
        lixo,
    }
}

/// ⭐ O piso da coluna nos cartões para a lista `cs`: o nome mais largo e o uso dele INTEIROS, mais o
/// que [`coluna_do_container`] põe à volta. O pintor dá ao uso no máximo metade do livre, e o piso
/// respeita-o.
pub(crate) fn piso(cs: &[ContainerView], largura: &mut dyn FnMut(&str, f32) -> f32) -> f32 {
    let k = coluna_do_container(Rect::new(0.0, 0.0, 0.0, geom::faixa_h()), 0.0);
    let fixo = k.nome_x - k.uso_fim;
    let (sm, xs) = (TypeToken::Sm.px(), TypeToken::Xs.px());
    cs.iter()
        .map(|c| {
            let nome = largura(&c.name, sm).ceil();
            let uso = largura(&crate::uso::rotulo(c.uses), xs).ceil();
            fixo + (nome + Spacing::Sm.px() + uso).max(uso * 2.0)
        })
        .fold(0.0, f32::max)
}

/// ⭐ A coluna de um container nos cartões (maquete A3).
fn paint_cartoes(
    ctx: &mut PaintCtx,
    theme: Theme,
    g: &geom::Geom,
    c: &ContainerView,
    (index, escolhido): (usize, bool),
    row: Rect,
) {
    crate::faixas_coluna::fundo(
        ctx,
        theme,
        g,
        row,
        escolhido.then_some(TimelineTinta::ChipEscolhido),
    );
    let k = coluna_do_container(row, g.label_w);
    let t = |x| crate::folha_tinta::tinta(x, theme);
    let nome = if escolhido {
        TimelineTinta::TextoCabecalho
    } else {
        TimelineTinta::TextoPropriedade
    };
    paint_icon(
        ctx.scene,
        IconId::Cube,
        k.icone,
        t(nome),
        StrokeToken::Default.px(),
    );
    // O uso à direita, antes do lápis; o nome tem o que sobra, e o uso nunca come mais de metade.
    let uso = crate::uso::rotulo(c.uses);
    let (sm, xs) = (TypeToken::Sm.px(), TypeToken::Xs.px());
    let livre = (k.uso_fim - k.nome_x).max(0.0);
    let uso_w = ctx.text_system.prefix_width(&uso, xs).min(livre * 0.5);
    let uso_x = k.uso_fim - uso_w;
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        &uso,
        uso_x,
        k.texto_y.1,
        xs,
        uso_w,
        t(TimelineTinta::TextoValor),
    );
    paint_text_elided(
        ctx.text_system,
        ctx.scene,
        &c.name,
        k.nome_x,
        k.texto_y.0,
        sm,
        (uso_x - k.nome_x - Spacing::Sm.px()).max(0.0),
        t(nome),
    );
    let band = Rect::new(g.rows.x, g.rows.y, g.label_w, g.rows.h);
    botao_discreto(
        ctx,
        theme,
        ids::TIMELINE_CONT_RENAME[index],
        (k.lapis, band),
        (IconId::Text, TimelineTinta::TextoValor),
    );
    botao_discreto(
        ctx,
        theme,
        ids::TIMELINE_CONT_DELETE[index],
        (k.lixo, band),
        (IconId::Trash, TimelineTinta::TextoValor),
    );
}

/// One of the row's two buttons (pencil / trash) — the transport chip's pair, one bar down.
fn paint_row_button(
    ctx: &mut PaintCtx,
    theme: Theme,
    id: ph2d_editor_core::NodeId,
    r: Rect,
    band: Rect,
    icon: IconId,
) {
    // ⭐ Raio e moldura pela porta do TEMA: o botão é plano num tema moderno.
    let btn_radius = ph2d_editor_core::paint::frame_radius(theme, Radius::Xs.px());
    fill_rounded_rect(ctx.scene, r, btn_radius, resolve(ColorToken::Bg3, theme));
    ph2d_editor_core::paint::stroke_frame(
        ctx.scene,
        r,
        btn_radius,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );
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
        resolve(ColorToken::Text3, theme),
        StrokeToken::Default.px(),
    );
    ctx.host.store_mut().register(id, InteractiveState::Plain);
    // Clipped to the visible band, like every other control in this column.
    if let Some(hit) = clipped(r, band) {
        ctx.host.hit_index_mut().register(id, hit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::faixas_coluna::glifo_em;
    use ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread;

    fn meio_das_maiusculas(y: f32, font: f32) -> f32 {
        let topo = y - (ph2d_text::displayed_font_px(font) - font) * 0.5;
        topo + ph2d_text::cap_centre_px(font)
    }

    /// ⭐ Doc 16 onda 3: na linha da lista (a altura da faixa), o nome, o uso, o cubo, o lápis e o
    /// lixo ficam a ≤ 1 px do meio; os alvos cabem na coluna sem se tocar e o uso acaba antes do lápis.
    #[test]
    fn a_linha_da_lista_fica_no_meio_e_os_alvos_cabem_na_coluna() {
        force_cards_on_this_thread(Some(true));
        let row = Rect::new(10.0, 200.0, 1200.0, geom::faixa_h());
        let meio = row.y + row.h * 0.5;
        let label_w = crate::tracks::LABEL_COL_W;
        let k = coluna_do_container(row, label_w);
        let centro = |r: Rect| r.y + r.h * 0.5;
        for (y, o_que) in [
            (meio_das_maiusculas(k.texto_y.0, TypeToken::Sm.px()), "nome"),
            (meio_das_maiusculas(k.texto_y.1, TypeToken::Xs.px()), "uso"),
            (centro(k.icone), "cubo"),
            (centro(glifo_em(k.lapis)), "lápis"),
            (centro(glifo_em(k.lixo)), "lixo"),
        ] {
            assert!(
                (y - meio).abs() <= 1.0,
                "{o_que} a {:.2} px do meio",
                y - meio
            );
        }
        assert!(
            k.lixo.x + k.lixo.w <= row.x + label_w,
            "o lixo sai da coluna"
        );
        assert!(k.lapis.x + k.lapis.w < k.lixo.x, "lápis sobre o lixo");
        assert!(k.uso_fim <= k.lapis.x, "o uso passa por baixo do lápis");
        force_cards_on_this_thread(None);
    }
}
