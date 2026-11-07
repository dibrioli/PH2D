//! A pintura da aba das camadas: o que se lê antes ([`Quadro`]), as três partes e os alvos.

use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, paint_text_rotated_ccw, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::camada_das_listas;
use ph2d_editor_core::widget::{TextInput, paint_text_input_with_buffer};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_physics_ecs::MAX_LAYERS;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, StrokeToken, TypeToken};

use super::quadro::{Quadro, fotos, pairar_da_foto};
use super::{AREA, Onde, aberta};
use crate::ids as i;

/// A cor da aba: em cartões, o tom da lista (como o editor de animações); `PH2D_CARDS=0`, o de antes.
/// Macro: o tipo da cor é da `ph2d-vector`, que este painel não nomeia.
macro_rules! tom {
    ($t:expr, $theme:expr $(,)?) => {{
        let t: ColorToken = $t;
        let cartao = match t {
            ColorToken::PanelBg => Some(ColorToken::KanbanList),
            ColorToken::Bg1 => Some(ColorToken::KanbanCard),
            ColorToken::Bg2 => Some(ColorToken::CardField),
            ColorToken::Bg3 => Some(ColorToken::KanbanCardRaised),
            ColorToken::Text1 => Some(ColorToken::KanbanCardFg),
            ColorToken::Text2 | ColorToken::Text3 => Some(ColorToken::CardLabel),
            _ => None,
        };
        match cartao.filter(|_| ph2d_editor_core::panel::cards::ferramenta::cards_on()) {
            Some(c) => ph2d_editor_core::widget::card_ink::CardInk::of_list($theme)
                .color(c)
                .with_alpha(1.0),
            None => resolve(t, $theme),
        }
    }};
}

/// Pinta a aba, se à frente neste quadro. Corre no fim do `paint` do painel, aberto ou fechado.
pub(crate) fn paint(ctx: &mut PaintCtx) {
    fotos(ctx);
    let Some((area, fixa)) = AREA.take().filter(|_| aberta()) else {
        return;
    };
    if fixa {
        AREA.set(Some((area, true)));
    }
    pairar_da_foto(ctx);
    let q = Quadro::de(ctx, area);
    let canto = Rect::new(q.g.x, q.g.y, q.g.rotulo_w, q.g.rotulo_w);
    // O losango da matriz (canal MUNDO `CAMADAS`): UM, no canto dos rótulos.
    let theme = ctx.host.theme();
    let losango = camada_das_listas::por_cima(ctx.scene, |cena| {
        ph2d_editor_core::widget::key_mark::paint_decorator_for(
            cena,
            theme,
            canto,
            &i::PHYSICS_LAYER_CELL,
        )
    });
    q.lente();
    pinta_lista(ctx, &q);
    pinta_grade(ctx, &q);
    pinta_pe(ctx, &q);
    alvos(ctx, &q, losango, canto);
}

/// A lista «Layers»: o nome e quantos objetos, a selecionada acesa, e o «+ Add».
fn pinta_lista(ctx: &mut PaintCtx, q: &Quadro) {
    let theme = ctx.host.theme();
    let &Quadro {
        area,
        g,
        aberta_n,
        editando,
        font,
        raio,
        contagem_w,
        add_w,
        add_aceso,
        add_rotulo,
        ..
    } = q;
    let visiveis = &q.visiveis;
    let rotulos = &q.rotulos;
    let contagens = &q.contagens;
    let texto = &mut *ctx.text_system;
    camada_das_listas::por_cima(ctx.scene, |cena| {
        fill_rounded_rect(cena, area, 0.0, tom!(ColorToken::PanelBg, theme));
        for r in [g.lista, g.grade, g.camada] {
            fill_rounded_rect(cena, r, raio, tom!(ColorToken::Bg1, theme));
        }
        // A lista «Layers».
        let t = g.linha_da_lista(0);
        paint_text(
            texto,
            cena,
            tr("card.physics.tab_layers"),
            t.x + Spacing::Sm.px(),
            t.y - ROW_H_PX + (ROW_H_PX - font) * 0.5,
            font,
            t.w,
            tom!(ColorToken::Text3, theme),
        );
        for (p, &n) in visiveis.iter().enumerate() {
            let r = g.linha_da_lista(p);
            let sel = n == aberta_n;
            if sel {
                fill_rounded_rect(cena, r, raio, tom!(ColorToken::Bg3, theme));
            }
            if editando.is_some_and(|(e, onde)| e == n && onde == Onde::Lista) {
                continue;
            }
            let cor = if sel {
                ColorToken::Text1
            } else {
                ColorToken::Text2
            };
            paint_text(
                texto,
                cena,
                &rotulos[n],
                r.x + Spacing::Sm.px(),
                r.y + (r.h - font) * 0.5,
                font,
                r.w - contagem_w,
                tom!(cor, theme),
            );
            let c = &contagens[p];
            let cw = texto.prefix_width(c, font);
            paint_text(
                texto,
                cena,
                c,
                r.x + r.w - Spacing::Sm.px() - cw,
                r.y + (r.h - font) * 0.5,
                font,
                cw + Spacing::Xs.px(),
                tom!(ColorToken::Text3, theme),
            );
        }
        // O «+ Add», como os botões do editor de animações.
        let fundo = if add_aceso {
            ColorToken::Bg3
        } else {
            ColorToken::Bg2
        };
        fill_rounded_rect(cena, g.add(add_w), raio, tom!(fundo, theme));
        ph2d_editor_core::paint::paint_text_centered(
            texto,
            cena,
            add_rotulo,
            g.add(add_w),
            font,
            tom!(ColorToken::Text1, theme),
        );
    });
}

/// A grade «Who bumps into whom · N pairs», com as faixas acesas e os nomes nas duas bordas.
fn pinta_grade(ctx: &mut PaintCtx, q: &Quadro) {
    let theme = ctx.host.theme();
    let &Quadro {
        g,
        sob,
        editando,
        font,
        raio,
        cabeca_dir_w,
        matriz,
        ..
    } = q;
    let visiveis = &q.visiveis;
    let rotulos = &q.rotulos;
    let cabeca_dir = &q.cabeca_dir;
    let linha_acesa = |n: usize| q.linha_acesa(n);
    let coluna_acesa = |n: usize| q.coluna_acesa(n);
    let texto = &mut *ctx.text_system;
    let pad = Spacing::Md.px();
    camada_das_listas::por_cima(ctx.scene, |cena| {
        // A grade «Who bumps into whom · N pairs».
        let ty = g.grade.y + pad;
        paint_text(
            texto,
            cena,
            tr("card.physics.tab_grid_title"),
            g.grade.x + pad,
            ty,
            TypeToken::Base.px(),
            g.grade.w - cabeca_dir_w - (pad * 2.0 + Spacing::Lg.px()),
            tom!(ColorToken::Text1, theme),
        );
        paint_text(
            texto,
            cena,
            cabeca_dir,
            g.grade.x + g.grade.w - pad - cabeca_dir_w,
            ty,
            font,
            cabeca_dir_w + Spacing::Xs.px(),
            tom!(ColorToken::Text3, theme),
        );
        // ⭐ As faixas acesas vão de ponta a ponta (a da linha até à última coluna): na onda 8 a da
        //    linha parava na diagonal e as células a escondiam.
        let faixa = resolve(ColorToken::AccentSoft, theme);
        let fim = g.celula(g.k.saturating_sub(1), g.k.saturating_sub(1));
        for (p, &n) in visiveis.iter().enumerate() {
            if linha_acesa(n) {
                let r = g.rotulo_da_linha(p);
                fill_rounded_rect(
                    cena,
                    Rect::new(r.x, r.y, fim.x + fim.w - r.x, r.h),
                    raio,
                    faixa,
                );
            }
            if coluna_acesa(n) {
                let r = g.rotulo_da_coluna(p);
                fill_rounded_rect(
                    cena,
                    Rect::new(r.x, r.y, r.w, fim.y + fim.h - r.y),
                    raio,
                    faixa,
                );
            }
        }
        let tinta = |aceso: bool| {
            tom!(
                if aceso {
                    ColorToken::Text1
                } else {
                    ColorToken::Text2
                },
                theme,
            )
        };
        for (p, &n) in visiveis.iter().enumerate() {
            let r = g.rotulo_da_linha(p);
            if !editando.is_some_and(|(e, onde)| e == n && onde == Onde::Grade) {
                paint_text(
                    texto,
                    cena,
                    &rotulos[n],
                    r.x,
                    r.y + (r.h - font) * 0.5,
                    font,
                    r.w,
                    tinta(linha_acesa(n)),
                );
            }
            let c = g.rotulo_da_coluna(p);
            paint_text_rotated_ccw(
                texto,
                cena,
                &rotulos[n],
                c.x + (c.w - font) * 0.5,
                c.y + c.h - Spacing::Xs.px(),
                font,
                c.h,
                tinta(coluna_acesa(n)),
            );
        }
        for (p, &a) in visiveis.iter().enumerate() {
            for (q, &b) in visiveis[..=p].iter().enumerate() {
                let cel = g.celula(p, q);
                let cor = if matriz.collides(a, b) {
                    ColorToken::Accent
                } else {
                    ColorToken::Bg3
                };
                fill_rounded_rect(cena, cel, raio, resolve(cor, theme));
                let pairada = sob == Some((a, b));
                if a == b || pairada {
                    let cor = if pairada {
                        ColorToken::Text1
                    } else {
                        ColorToken::Text2
                    };
                    // FRAME-RAW-OK: a diagonal da grade (o eixo de simetria) e a célula sob o ponteiro
                    ph2d_editor_core::paint::stroke_rounded_rect(
                        cena,
                        cel,
                        raio,
                        StrokeToken::Default.px(),
                        resolve(cor, theme),
                    );
                }
            }
        }
    });
}

/// A frase fixa do pé, a linha de título da camada aberta e o campo do nome em edição.
fn pinta_pe(ctx: &mut PaintCtx, q: &Quadro) {
    let theme = ctx.host.theme();
    let &Quadro {
        g,
        font,
        raio,
        pares_w,
        ..
    } = q;
    let rotulos = &q.rotulos;
    let pe = &q.pe;
    let titulo = &q.titulo;
    let pares_da_camada = &q.pares_da_camada;
    let campo = &q.campo;
    let texto = &mut *ctx.text_system;
    let pad = Spacing::Md.px();
    camada_das_listas::por_cima(ctx.scene, |cena| {
        // O pé: a frase fixa, numa faixa própria (nunca sai da aba nem cobre uma linha).
        let r = g.pe();
        fill_rounded_rect(cena, r, raio, tom!(ColorToken::Bg2, theme));
        paint_text(
            texto,
            cena,
            pe,
            r.x + Spacing::Sm.px(),
            r.y + (r.h - font) * 0.5,
            font,
            r.w - Spacing::Sm.px() * 2.0,
            tom!(ColorToken::Text1, theme),
        );

        // A camada aberta: a linha de título (o resto é a onda 9b).
        let cy = g.camada.y + pad;
        paint_text(
            texto,
            cena,
            titulo,
            g.camada.x + pad,
            cy,
            TypeToken::Base.px(),
            g.camada.w - pares_w - (pad * 2.0 + Spacing::Lg.px()),
            tom!(ColorToken::Text1, theme),
        );
        paint_text(
            texto,
            cena,
            pares_da_camada,
            g.camada.x + g.camada.w - pad - pares_w,
            cy,
            font,
            pares_w + Spacing::Xs.px(),
            tom!(ColorToken::Text3, theme),
        );

        if let Some((n, id, buf, caret, (anchor, state), r)) = &campo {
            let input = TextInput::new(*id, "")
                .placeholder(rotulos[*n].clone())
                .visual((*state, 0.0));
            paint_text_input_with_buffer(
                &input,
                Some(buf),
                Some(*caret),
                *anchor,
                *r,
                cena,
                texto,
                theme,
            );
        }
    });
}

fn alvos(ctx: &mut PaintCtx, q: &Quadro, losango: Option<ph2d_a11y::NodeId>, canto: Rect) {
    let (g, visiveis, matriz, campo, add_w) = (&q.g, &q.visiveis, q.matriz, &q.campo, q.add_w);
    let cheia = visiveis.len() >= MAX_LAYERS;
    // Os alvos, por ordem: o último registado ganha o clique.
    let (_, hit) = ctx.host.store_and_hit_index_mut();
    ph2d_editor_core::interaction::toggle_paint::limpa_grupo(i::PHYSICS_LAYER_PAINT_GROUP);
    for (p, &a) in visiveis.iter().enumerate() {
        camada_das_listas::alvo(hit, i::PHYSICS_LAYER_LIST[a], g.linha_da_lista(p));
        for (q, &b) in visiveis[..=p].iter().enumerate() {
            let id = i::PHYSICS_LAYER_CELL[i::physics_layer_cell_index(a, b)];
            let cel = g.celula(p, q);
            camada_das_listas::alvo(hit, id, cel);
            ph2d_editor_core::interaction::toggle_paint::celula(
                id,
                i::PHYSICS_LAYER_PAINT_GROUP,
                matriz.collides(a, b),
                cel,
            );
        }
        camada_das_listas::alvo(hit, i::PHYSICS_LAYER_ROW_LABEL[a], g.rotulo_da_linha(p));
        camada_das_listas::alvo(hit, i::PHYSICS_LAYER_COL_LABEL[a], g.rotulo_da_coluna(p));
    }
    if !cheia {
        camada_das_listas::alvo(hit, i::PHYSICS_LAYER_ADD, g.add(add_w));
    }
    if let Some((_, id, _, _, _, r)) = &campo {
        camada_das_listas::alvo(hit, *id, *r);
    }
    if let Some(alvo) = losango {
        camada_das_listas::alvo(hit, alvo, canto);
    }
    ph2d_editor_core::widget::composto::grupo(i::PHYSICS_LAYER_CELL.iter().copied());
}
