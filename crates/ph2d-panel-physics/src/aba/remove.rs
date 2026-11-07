//! ⭐ **O «Remove» da aba** (onda 9b, maquete B estado 2): uma camada sem objetos sai logo; com objetos, a
//! camada aberta pergunta para onde eles vão («Move and remove» / «Cancel»). A 0 (Default) não se remove.

use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, paint_text_centered};
use ph2d_editor_core::panel::{PaintCtx, camada_das_listas};
use ph2d_editor_core::widget::ButtonState;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing};

use super::geometria::fila;
use crate::ids as i;

/// Quantos nomes a pergunta diz antes do «+N more».
const NOMES_NA_PERGUNTA: usize = 3;

/// A pergunta, em coordenadas relativas ao canto do cartão (ver [`Pergunta::desloca`]).
#[derive(Clone, Debug, Default)]
pub(super) struct Pergunta {
    pub(super) frase: String,
    pub(super) frase_y: f32,
    pub(super) rotulo_y: f32,
    /// (camada de destino, botão).
    pub(super) destinos: Vec<(usize, Rect)>,
    pub(super) escolhido: usize,
    pub(super) ok: Rect,
    pub(super) cancela: Rect,
    pub(super) altura: f32,
}

impl Pergunta {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn monta(
        w: f32,
        aberta: usize,
        visiveis: &[usize],
        rotulos: &[String],
        objetos: &[String],
        escolhido: usize,
        largura: &mut dyn FnMut(&str) -> f32,
    ) -> Self {
        let pad = Spacing::Md.px();
        let row = ROW_H_PX;
        let dentro = (w - pad * 2.0).max(0.0);
        let mut nomes = super::lista_de_nomes(&objetos[..objetos.len().min(NOMES_NA_PERGUNTA)]);
        if objetos.len() > NOMES_NA_PERGUNTA {
            let resto = objetos.len() - NOMES_NA_PERGUNTA;
            nomes = format!(
                "{nomes}{}{}",
                tr("card.physics.tab_names_last"),
                tr_with("card.physics.tab_more", &[("n", &resto.to_string())])
            );
        }
        let frase = tr_with(
            "card.physics.tab_remove_ask",
            &[
                ("n", &objetos.len().to_string()),
                ("layer", &rotulos[aberta]),
                ("names", &nomes),
            ],
        );
        let mut y = pad + row;
        let frase_y = y;
        y += row;
        let rotulo_y = y;
        y += row;
        let botao =
            |s: &str, largura: &mut dyn FnMut(&str) -> f32| largura(s) + Spacing::Lg.px() * 2.0;
        let outras: Vec<usize> = visiveis.iter().copied().filter(|&d| d != aberta).collect();
        let larg: Vec<f32> = outras
            .iter()
            .map(|&d| botao(&rotulos[d], largura))
            .collect();
        let (rs, linhas) = fila(pad, y, dentro, &larg);
        let destinos = outras.into_iter().zip(rs).collect();
        y += linhas.max(1) as f32 * (row + Spacing::Xs.px()) + Spacing::Sm.px();
        let larg = [
            botao(tr("card.physics.tab_move_ok"), largura),
            botao(tr("card.physics.tab_cancel"), largura),
        ];
        let (rs, linhas) = fila(pad, y, dentro, &larg);
        y += linhas as f32 * (row + Spacing::Xs.px());
        Self {
            frase,
            frase_y,
            rotulo_y,
            destinos,
            escolhido,
            ok: rs[0],
            cancela: rs[1],
            altura: y + pad,
        }
    }

    pub(super) fn desloca(mut self, x: f32, y: f32) -> Self {
        let m = |r: &mut Rect| {
            r.x += x;
            r.y += y;
        };
        self.frase_y += y;
        self.rotulo_y += y;
        self.destinos.iter_mut().for_each(|d| m(&mut d.1));
        m(&mut self.ok);
        m(&mut self.cancela);
        self
    }
}

/// Pinta a pergunta (a linha de título é do pé).
pub(super) fn pinta(ctx: &mut PaintCtx, q: &super::quadro::Quadro, p: &Pergunta) {
    let theme = ctx.host.theme();
    let (g, font, raio) = (q.g, q.font, q.raio);
    let aceso = |id| ctx.host.store().button_state(id) == Some(ButtonState::Hovered);
    let acesos: Vec<bool> = p
        .destinos
        .iter()
        .map(|&(d, _)| aceso(i::PHYSICS_LAYER_MOVE_TO[d]))
        .collect();
    let (ok_aceso, cancela_aceso) = (
        aceso(i::PHYSICS_LAYER_MOVE_OK),
        aceso(i::PHYSICS_LAYER_MOVE_CANCEL),
    );
    let texto = &mut *ctx.text_system;
    let x = g.camada.x + Spacing::Md.px();
    let w = g.camada.w - Spacing::Md.px() * 2.0;
    camada_das_listas::por_cima(ctx.scene, |cena| {
        let linha = |texto: &mut _, cena: &mut _, s: &str, y: f32, cor| {
            paint_text(
                texto,
                cena,
                s,
                x,
                y + (ROW_H_PX - font) * 0.5,
                font,
                w,
                tom!(cor, theme),
            );
        };
        linha(texto, cena, &p.frase, p.frase_y, ColorToken::Text1);
        linha(
            texto,
            cena,
            tr("card.physics.tab_move_to"),
            p.rotulo_y,
            ColorToken::Text3,
        );
        let botao = |texto: &mut _, cena: &mut _, r: Rect, s: &str, fundo, tinta| {
            fill_rounded_rect(cena, r, raio, fundo);
            paint_text_centered(texto, cena, s, r, font, tinta);
        };
        let normal = |hover: bool| {
            tom!(
                if hover {
                    ColorToken::Bg3
                } else {
                    ColorToken::Bg2
                },
                theme
            )
        };
        for (&(d, r), &hover) in p.destinos.iter().zip(&acesos) {
            let (fundo, tinta) = if d == p.escolhido {
                (
                    ph2d_editor_core::paint::resolve(ColorToken::Accent, theme),
                    ph2d_editor_core::paint::resolve(ColorToken::AccentFg, theme),
                )
            } else {
                (normal(hover), tom!(ColorToken::Text1, theme))
            };
            botao(texto, cena, r, &q.rotulos[d], fundo, tinta);
        }
        botao(
            texto,
            cena,
            p.ok,
            tr("card.physics.tab_move_ok"),
            normal(ok_aceso),
            tom!(ColorToken::Text1, theme),
        );
        botao(
            texto,
            cena,
            p.cancela,
            tr("card.physics.tab_cancel"),
            normal(cancela_aceso),
            tom!(ColorToken::Text1, theme),
        );
    });
}

/// Os alvos e as dicas da pergunta.
pub(super) fn alvos(ctx: &mut PaintCtx, p: &Pergunta) {
    {
        let store = ctx.host.store_mut();
        let dicas = p
            .destinos
            .iter()
            .map(|&(d, _)| (i::PHYSICS_LAYER_MOVE_TO[d], "card.physics.tab_move_to_tip"))
            .chain([
                (i::PHYSICS_LAYER_MOVE_OK, "card.physics.tab_move_ok_tip"),
                (i::PHYSICS_LAYER_MOVE_CANCEL, "card.physics.tab_cancel_tip"),
            ]);
        for (id, chave) in dicas {
            if store.tooltip_for(id).is_none() {
                store.set_tooltip(id, tr(chave));
            }
        }
    }
    let (_, hit) = ctx.host.store_and_hit_index_mut();
    for &(d, r) in &p.destinos {
        camada_das_listas::alvo(hit, i::PHYSICS_LAYER_MOVE_TO[d], r);
    }
    camada_das_listas::alvo(hit, i::PHYSICS_LAYER_MOVE_OK, p.ok);
    camada_das_listas::alvo(hit, i::PHYSICS_LAYER_MOVE_CANCEL, p.cancela);
}
