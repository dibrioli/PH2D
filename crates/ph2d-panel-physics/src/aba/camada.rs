//! ⭐ **A camada aberta** (onda 9b, maquete B estado 1): «Bumps into» com uma caixinha por camada
//! visível (a MESMA edição da célula da grade e da fila do Inspector: [`crate::alterna_par`]) e
//! «Objects on this layer» com um botão por objeto, «Select all» e, passadas três linhas, «+N more».

use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_layer_outline, paint_text, paint_text_centered, resolve,
};
use ph2d_editor_core::panel::{PaintCtx, camada_das_listas};
use ph2d_editor_core::widget::ButtonState;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, StrokeToken, TypeToken};

use super::geometria::{Geometria, fila};

const MARGEM_DO_CARTAO: f32 = Geometria::MARGEM_DO_CARTAO;
use crate::ids as i;

/// Quantas linhas de objetos se mostram antes do «+N more».
const LINHAS_DE_OBJETOS: usize = 3;

/// A camada aberta, em coordenadas relativas ao canto do cartão (ver [`Camada::desloca`]).
#[derive(Clone, Debug, Default)]
pub(super) struct Camada {
    pub(super) bumps_y: f32,
    /// Projeto novo (só a Default à vista; maquete B estado 4): a frase que ensina, no lugar de «Bumps into».
    pub(super) so_default: bool,
    /// (camada, caixinha).
    pub(super) caixas: Vec<(usize, Rect)>,
    pub(super) objetos_y: f32,
    /// (índice do objeto, nome, botão).
    pub(super) objetos: Vec<(usize, String, Rect)>,
    pub(super) mais: Option<(Rect, String)>,
    pub(super) todos: Option<Rect>,
    pub(super) vazio_y: Option<f32>,
    pub(super) altura: f32,
}

impl Camada {
    /// Monta a camada aberta numa largura `w`; `largura` mede um texto na fonte da aba.
    pub(super) fn monta(
        w: f32,
        visiveis: &[usize],
        rotulos: &[String],
        nomes: &[String],
        mais_aberto: bool,
        // `(texto, botão)`: a largura do texto na letra do painel, ou a do botão inteiro (onda 9g).
        largura: &mut dyn FnMut(&str, bool) -> f32,
    ) -> Self {
        let pad = MARGEM_DO_CARTAO;
        let row = ROW_H_PX;
        let dentro = (w - pad * 2.0).max(0.0);
        let caixa = TypeToken::Sm.px();
        let mut y = pad + row; // a linha de título
        let bumps_y = y;
        let so_default = visiveis.len() <= 1;
        let caixas = if so_default {
            // A frase quebra na largura do cartão: uma linha por largura medida, mais uma de folga.
            let linhas =
                (largura(tr("card.physics.tab_only_default"), false) / dentro.max(1.0)).ceil();
            y += (linhas + 1.0) * row;
            Vec::new()
        } else {
            y += row;
            let larg: Vec<f32> = visiveis
                .iter()
                .map(|&n| caixa + Spacing::Md.px() + Spacing::Sm.px() + largura(&rotulos[n], false))
                .collect();
            let (rs, linhas) = fila(pad, y, dentro, &larg, Geometria::fila_de_linhas());
            y += linhas as f32 * (row + Spacing::Xs.px()) + Spacing::Sm.px();
            visiveis.iter().copied().zip(rs).collect()
        };
        let objetos_y = y;
        y += row;
        let botao = |s: &str, largura: &mut dyn FnMut(&str, bool) -> f32| largura(s, true);
        let todos_rot = tr("card.physics.tab_select_all");
        let todos_w = botao(todos_rot, largura);
        let objs: Vec<f32> = nomes.iter().map(|n| botao(n, largura)).collect();
        let total = nomes.len().min(i::PHYSICS_LAYER_OBJECTS);
        let mut cabe = total;
        let mut mais = None;
        let monta = |n: usize, extra: &[f32]| {
            let mut v = objs[..n].to_vec();
            v.extend_from_slice(extra);
            fila(pad, y, dentro, &v, Geometria::fila_de_botoes())
        };
        let (mut rs, mut linhas) = monta(total, &[todos_w]);
        if !mais_aberto && linhas > LINHAS_DE_OBJETOS {
            let rotulo_mais = |n: usize| tr_with("card.physics.tab_more", &[("n", &n.to_string())]);
            while cabe > 0 {
                cabe -= 1;
                let m = rotulo_mais(nomes.len() - cabe);
                let (r, l) = monta(cabe, &[botao(&m, largura), todos_w]);
                if l <= LINHAS_DE_OBJETOS {
                    (rs, linhas) = (r, l);
                    mais = Some(m);
                    break;
                }
            }
        } else if nomes.len() > total {
            let m = tr_with(
                "card.physics.tab_more",
                &[("n", &(nomes.len() - total).to_string())],
            );
            (rs, linhas) = monta(total, &[botao(&m, largura), todos_w]);
            mais = Some(m);
        }
        let vazio_y = nomes.is_empty().then_some(y);
        let objetos = nomes[..cabe]
            .iter()
            .cloned()
            .enumerate()
            .zip(rs.iter().copied())
            .map(|((k, n), r)| (k, n, r))
            .collect();
        let mais = mais.map(|m| (rs[cabe], m));
        let todos = (!nomes.is_empty()).then(|| rs[rs.len() - 1]);
        let linhas = if nomes.is_empty() { 1 } else { linhas };
        let (bh, _, bv) = Geometria::fila_de_botoes();
        y += linhas as f32 * (bh + bv);
        Self {
            bumps_y,
            so_default,
            caixas,
            objetos_y,
            objetos,
            mais,
            todos,
            vazio_y,
            altura: y + pad,
        }
    }

    /// Passa para o ecrã: o canto do cartão em `(x, y)`.
    pub(super) fn desloca(mut self, x: f32, y: f32) -> Self {
        let m = |r: &mut Rect| {
            r.x += x;
            r.y += y;
        };
        self.bumps_y += y;
        self.objetos_y += y;
        self.vazio_y = self.vazio_y.map(|v| v + y);
        self.caixas.iter_mut().for_each(|c| m(&mut c.1));
        self.objetos.iter_mut().for_each(|o| m(&mut o.2));
        if let Some((r, _)) = self.mais.as_mut() {
            m(r);
        }
        if let Some(r) = self.todos.as_mut() {
            m(r);
        }
        self
    }
}

/// Pinta «Bumps into» e «Objects on this layer» (a linha de título é do pé).
pub(super) fn pinta(ctx: &mut PaintCtx, q: &super::quadro::Quadro) {
    let theme = ctx.host.theme();
    let (c, g, font, raio) = (&q.camada, q.g, q.font, q.raio);
    let aceso = |id| ctx.host.store().button_state(id) == Some(ButtonState::Hovered);
    let caixas_acesas: Vec<bool> = c
        .caixas
        .iter()
        .map(|&(n, _)| aceso(i::PHYSICS_LAYER_BUMPS[n]))
        .collect();
    let objetos_acesos: Vec<bool> = c
        .objetos
        .iter()
        .map(|o| aceso(i::PHYSICS_LAYER_OBJECT[o.0]))
        .collect();
    let (mais_aceso, todos_aceso) = (
        aceso(i::PHYSICS_LAYER_MORE),
        aceso(i::PHYSICS_LAYER_SELECT_ALL),
    );
    let texto = &mut *ctx.text_system;
    let x = g.camada.x + MARGEM_DO_CARTAO;
    let w = g.camada.w - MARGEM_DO_CARTAO * 2.0;
    camada_das_listas::por_cima(ctx.scene, |cena| {
        let rotulo = |texto: &mut _, cena: &mut _, s: &str, y: f32| {
            paint_text(
                texto,
                cena,
                s,
                x,
                y + (ROW_H_PX - font) * 0.5,
                font,
                w,
                tom!(ColorToken::Text3, theme),
            );
        };
        if c.so_default {
            ph2d_editor_core::paint::paint_text_block(
                texto,
                cena,
                tr("card.physics.tab_only_default"),
                x,
                c.bumps_y + (ROW_H_PX - font) * 0.5,
                font,
                w,
                tom!(ColorToken::Text2, theme),
            );
        } else {
            rotulo(texto, cena, tr("card.physics.tab_bumps_into"), c.bumps_y);
        }
        for (&(n, r), &hover) in c.caixas.iter().zip(&caixas_acesas) {
            let on = q.matriz.collides(q.aberta_n, n);
            let fundo = if hover {
                ColorToken::Bg3
            } else {
                ColorToken::Bg2
            };
            fill_rounded_rect(cena, r, raio, tom!(fundo, theme));
            let lado = TypeToken::Sm.px();
            let b = Rect::new(r.x + Spacing::Sm.px(), r.y + (r.h - lado) * 0.5, lado, lado);
            if on {
                fill_rounded_rect(cena, b, raio, resolve(ColorToken::Accent, theme));
                paint_text_centered(
                    texto,
                    cena,
                    tr("card.physics.tab_check"),
                    b,
                    font,
                    resolve(ColorToken::AccentFg, theme),
                );
            } else {
                // FRAME-RAW-OK: a caixinha desligada (contorno, como a maquete B)
                ph2d_editor_core::paint::stroke_rounded_rect(
                    cena,
                    b,
                    raio,
                    StrokeToken::Default.px(),
                    tom!(ColorToken::Text3, theme),
                );
            }
            let (tx, topo) = (b.x + b.w + Spacing::Sm.px(), r.y + (r.h - font) * 0.5);
            let cabe = r.w - b.w - Spacing::Sm.px() * 2.0;
            let rot = &q.rotulos[n];
            paint_text(
                texto,
                cena,
                rot,
                tx,
                topo,
                font,
                cabe,
                tom!(ColorToken::Text1, theme),
            );
            // O contorno da cor da camada na moldura; desligada, mais apagado (onda 9f: a língua do Inspector).
            paint_layer_outline(cena, theme, i::PHYSICS_LAYER_BUMPS[n], n, r, raio, on);
        }
        rotulo(texto, cena, tr("card.physics.tab_objects"), c.objetos_y);
        if let Some(y) = c.vazio_y {
            rotulo(texto, cena, tr("card.physics.tab_no_objects"), y);
        }
        let botao = |texto: &mut _, cena: &mut _, r: Rect, s: &str, hover: bool| {
            let fundo = if hover {
                ColorToken::Bg3
            } else {
                ColorToken::Bg2
            };
            fill_rounded_rect(cena, r, raio, tom!(fundo, theme));
            let base = TypeToken::Base.px();
            paint_text_centered(texto, cena, s, r, base, tom!(ColorToken::Text1, theme));
        };
        for (o, &hover) in c.objetos.iter().zip(&objetos_acesos) {
            botao(texto, cena, o.2, &o.1, hover);
        }
        if let Some((r, s)) = &c.mais {
            botao(texto, cena, *r, s, mais_aceso);
        }
        if let Some(r) = c.todos {
            botao(
                texto,
                cena,
                r,
                tr("card.physics.tab_select_all"),
                todos_aceso,
            );
        }
    });
}

/// Os alvos da camada aberta e as dicas.
pub(super) fn alvos(ctx: &mut PaintCtx, c: &Camada) {
    {
        let store = ctx.host.store_mut();
        let dicas = c
            .caixas
            .iter()
            .map(|&(n, _)| (i::PHYSICS_LAYER_BUMPS[n], "card.physics.tab_bump_tip"))
            .chain(
                c.objetos
                    .iter()
                    .map(|o| (i::PHYSICS_LAYER_OBJECT[o.0], "card.physics.tab_object_tip")),
            )
            .chain([
                (i::PHYSICS_LAYER_MORE, "card.physics.tab_more_tip"),
                (
                    i::PHYSICS_LAYER_SELECT_ALL,
                    "card.physics.tab_select_all_tip",
                ),
            ]);
        for (id, chave) in dicas {
            if store.tooltip_for(id).is_none() {
                store.set_tooltip(id, tr(chave));
            }
        }
    }
    let (_, hit) = ctx.host.store_and_hit_index_mut();
    for &(n, r) in &c.caixas {
        camada_das_listas::alvo(hit, i::PHYSICS_LAYER_BUMPS[n], r);
    }
    for o in &c.objetos {
        camada_das_listas::alvo(hit, i::PHYSICS_LAYER_OBJECT[o.0], o.2);
    }
    if let Some((r, _)) = &c.mais {
        camada_das_listas::alvo(hit, i::PHYSICS_LAYER_MORE, *r);
    }
    if let Some(r) = c.todos {
        camada_das_listas::alvo(hit, i::PHYSICS_LAYER_SELECT_ALL, r);
    }
}
