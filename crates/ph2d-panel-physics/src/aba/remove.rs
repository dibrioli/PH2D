//! ⭐ **O «Remove» da aba** (onda 9b, maquete B estado 2): uma camada sem objetos sai logo; com objetos, a
//! camada aberta pergunta para onde eles vão («Move and remove» / «Cancel»). A 0 (Default) não se remove.
//! Onda 9e: a pergunta mora numa CAIXA realçada (`DangerSoft` com contorno `Danger`), e o destino escolhe-se
//! numa LISTA (uma linha por camada, com a moldura na cor dela; a escolhida acesa). Fecho 07/11: a lista é
//! SUSPENSA (maquete B), pela porta das listas do app (`Dropdown` + `paint_dropdown_list`).

use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, paint_text_centered};
use ph2d_editor_core::panel::{PaintCtx, camada_das_listas, paint_dropdown_list};
use ph2d_editor_core::widget::{
    ButtonState, Dropdown, DropdownOption, DropdownState, PopoverRow, paint_dropdown_chip,
    paint_popover_row_highlight,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, StrokeToken};

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
    /// As camadas para onde os objetos podem ir (as opções da lista suspensa).
    pub(super) destinos: Vec<usize>,
    /// O chip da lista suspensa do destino (`PHYSICS_LAYER_MOVE_PICK`).
    pub(super) chip: Rect,
    pub(super) escolhido: usize,
    /// A caixa realçada que contém a pergunta inteira.
    pub(super) caixa: Rect,
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
        let pad = super::Geometria::MARGEM_DO_CARTAO;
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
        let dentro_da_caixa = Spacing::Sm.px();
        let topo = pad + row;
        let (x, w) = (
            pad + dentro_da_caixa,
            (dentro - dentro_da_caixa * 2.0).max(0.0),
        );
        let mut y = topo + dentro_da_caixa;
        let frase_y = y;
        y += row;
        let rotulo_y = y;
        y += row;
        let botao =
            |s: &str, largura: &mut dyn FnMut(&str) -> f32| largura(s) + Spacing::Lg.px() * 2.0;
        let destinos: Vec<usize> = visiveis.iter().copied().filter(|&d| d != aberta).collect();
        // A lista SUSPENSA do destino (maquete B): o chip pela opção mais comprida, nunca pela escolhida.
        let texto_w = destinos
            .iter()
            .map(|&d| largura(&rotulos[d]))
            .fold(0.0_f32, f32::max);
        let chip_w = ph2d_editor_core::widget::dropdown_chip_width_for(texto_w, row).min(w);
        let chip = Rect::new(x, y, chip_w, row);
        y += row + Spacing::Sm.px();
        let larg = [
            botao(tr("card.physics.tab_move_ok"), largura),
            botao(tr("card.physics.tab_cancel"), largura),
        ];
        let (rs, linhas) = fila(x, y, w, &larg, super::Geometria::fila_de_linhas());
        y += linhas as f32 * (row + Spacing::Xs.px()) - Spacing::Xs.px() + dentro_da_caixa;
        Self {
            frase,
            frase_y,
            rotulo_y,
            destinos,
            chip,
            escolhido,
            caixa: Rect::new(pad, topo, dentro, y - topo),
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
        m(&mut self.chip);
        m(&mut self.ok);
        m(&mut self.cancela);
        m(&mut self.caixa);
        self
    }
}

/// A lista suspensa do destino: as camadas que sobram, com a escolhida.
pub(super) fn lista(q: &super::quadro::Quadro, p: &Pergunta) -> Dropdown<usize> {
    let opcoes = p
        .destinos
        .iter()
        .map(|&d| DropdownOption::new(i::PHYSICS_LAYER_MOVE_TO[d], d, q.rotulos[d].clone()))
        .collect();
    let mut dd = Dropdown::new(i::PHYSICS_LAYER_MOVE_PICK, "", opcoes);
    dd.select(p.escolhido);
    dd
}

/// A lista suspensa do destino está aberta?
pub(super) fn lista_aberta(store: &ph2d_editor_core::interaction::WidgetStore) -> bool {
    matches!(
        store.get(i::PHYSICS_LAYER_MOVE_PICK),
        Some(InteractiveState::Dropdown { open: true, .. })
    )
}

/// Abre a lista suspensa do destino (a foto `remove_lista`; a tela virtual não recebe rato).
pub(super) fn abre_a_lista(store: &mut ph2d_editor_core::interaction::WidgetStore) {
    if let Some(InteractiveState::Dropdown { open, .. }) = store.get_mut(i::PHYSICS_LAYER_MOVE_PICK)
    {
        *open = true;
    }
}

/// Fecha a lista suspensa do destino (escolheu, desistiu, ou a pergunta acabou).
pub(crate) fn fecha_a_lista(store: &mut ph2d_editor_core::interaction::WidgetStore) {
    if let Some(InteractiveState::Dropdown { open, .. }) = store.get_mut(i::PHYSICS_LAYER_MOVE_PICK)
    {
        *open = false;
    }
}

/// Pinta a pergunta (a linha de título é do pé).
pub(super) fn pinta(ctx: &mut PaintCtx, q: &super::quadro::Quadro, p: &Pergunta) {
    let theme = ctx.host.theme();
    let (font, raio) = (q.font, q.raio);
    let aceso = |id| ctx.host.store().button_state(id) == Some(ButtonState::Hovered);
    let (ok_aceso, cancela_aceso) = (
        aceso(i::PHYSICS_LAYER_MOVE_OK),
        aceso(i::PHYSICS_LAYER_MOVE_CANCEL),
    );
    let aberta = lista_aberta(ctx.host.store());
    let dd = lista(q, p)
        .open(aberta)
        .visual(ctx.host.store().dropdown_visual(i::PHYSICS_LAYER_MOVE_PICK));
    let texto = &mut *ctx.text_system;
    let x = p.caixa.x + Spacing::Sm.px();
    let w = p.caixa.w - Spacing::Sm.px() * 2.0;
    camada_das_listas::por_cima(ctx.scene, |cena| {
        fill_rounded_rect(cena, p.caixa, raio, fundo_da_pergunta(theme));
        // FRAME-RAW-OK: a caixa realçada da pergunta (maquete B, estado 2) leva o contorno do perigo
        ph2d_editor_core::paint::stroke_rounded_rect(
            cena,
            p.caixa,
            raio,
            StrokeToken::Thin.px(),
            ph2d_editor_core::paint::resolve(ColorToken::Danger, theme),
        );
        let linha = |texto: &mut _, cena: &mut _, s: &str, y: f32, cor| {
            paint_text(texto, cena, s, x, y + (ROW_H_PX - font) * 0.5, font, w, cor);
        };
        linha(
            texto,
            cena,
            &p.frase,
            p.frase_y,
            tom!(ColorToken::Text1, theme),
        );
        linha(
            texto,
            cena,
            tr("card.physics.tab_move_to"),
            p.rotulo_y,
            tinta_do_rotulo(theme),
        );
        paint_dropdown_chip(&dd, p.chip, cena, texto, theme);
        ph2d_editor_core::paint::paint_layer_outline(
            cena,
            theme,
            i::PHYSICS_LAYER_MOVE_PICK,
            p.escolhido,
            p.chip,
            raio,
            true,
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

/// A lista aberta, pela porta das listas do app (por cima de tudo, rola, light-dismiss); cada linha
/// com o contorno da sua camada.
pub(super) fn pinta_a_lista(ctx: &mut PaintCtx, q: &super::quadro::Quadro, p: &Pergunta) {
    if !lista_aberta(ctx.host.store()) {
        return;
    }
    let theme = ctx.host.theme();
    let dd = lista(q, p).open(true).state(DropdownState::Focused);
    let raio = q.raio;
    let destinos = p.destinos.clone();
    let mut linha = |cena: &mut ph2d_vector::VectorScene,
                     texto: &mut ph2d_text::TextSystem,
                     row: PopoverRow| {
        let Some(&d) = destinos.get(row.index) else {
            return;
        };
        if row.selected {
            paint_popover_row_highlight(cena, row.rect, theme);
        }
        let tinta = ph2d_editor_core::widget::resolve_opaque(row.ink(), theme);
        paint_text_centered(texto, cena, &q.rotulos[d], row.rect, q.font, tinta);
        if row.selected {
            // A ✓ da escolhida: no cartão o realce da linha é discreto (a mesma marca das caixinhas).
            let b = Rect::new(row.rect.x, row.rect.y, Spacing::Lg.px(), row.rect.h);
            paint_text_centered(texto, cena, tr("card.physics.tab_check"), b, q.font, tinta);
        }
        ph2d_editor_core::paint::paint_layer_outline(
            cena,
            theme,
            i::PHYSICS_LAYER_MOVE_TO[d],
            d,
            row.rect,
            raio,
            true,
        );
    };
    // Os alvos das linhas vão por cima dos da caixa da pergunta (senão «Move and remove», por baixo da lista,
    // ganharia o clique numa linha).
    let marca = ctx.host.store_and_hit_index_mut().1.len();
    paint_dropdown_list(ctx, &dd, p.chip, q.area, theme, Some(&mut linha));
    camada_das_listas::alvos_desde(ctx.host.store_and_hit_index_mut().1, marca);
}

/// O fundo da caixa da pergunta: `DangerSoft`; num tema em que o texto forte não se lê nele a
/// [`ph2d_tokens::contrast::TEXT_MIN_RATIO`] (medido 07/11: PlumberRed, 4,01:1), o do cartão — o contorno `Danger`
/// continua a dizer que é a pergunta do perigo.
#[must_use]
pub fn fundo_da_pergunta(theme: ph2d_tokens::Theme) -> ph2d_vector::Color {
    let perigo = tom!(ColorToken::DangerSoft, theme);
    if razao(tinta_do_destino(theme), perigo) >= ph2d_tokens::contrast::TEXT_MIN_RATIO {
        perigo
    } else {
        tom!(ColorToken::Bg1, theme)
    }
}

/// A razão WCAG entre duas cores pintadas.
fn razao(a: ph2d_vector::Color, b: ph2d_vector::Color) -> f64 {
    let t = |c: ph2d_vector::Color| {
        let [r, g, b, a] = c.to_rgba8().to_u8_array();
        ph2d_tokens::Color { r, g, b, a }
    };
    t(a).contrast_ratio(&t(b))
}

/// O rótulo «Move them to» sobre o fundo da pergunta: o `Text2` onde se lê a
/// [`ph2d_tokens::contrast::TEXT_MIN_RATIO`], senão o texto forte (era `Text3`, sem medida; fecho 07/11).
#[must_use]
pub fn tinta_do_rotulo(theme: ph2d_tokens::Theme) -> ph2d_vector::Color {
    let fraco = tom!(ColorToken::Text2, theme);
    if razao(fraco, fundo_da_pergunta(theme)) >= ph2d_tokens::contrast::TEXT_MIN_RATIO {
        fraco
    } else {
        tinta_do_destino(theme)
    }
}

/// O nome de cada destino, escolhido ou não: o texto FORTE (onda 9f: o `Text2` sobre o `DangerSoft` lia-se
/// fraco; a escolhida diz-se pelo fundo e pela ✓). O teste do contraste mede-o sobre [`fundo_da_pergunta`].
#[must_use]
pub fn tinta_do_destino(theme: ph2d_tokens::Theme) -> ph2d_vector::Color {
    tom!(ColorToken::Text1, theme)
}

/// Os alvos e as dicas da pergunta.
pub(super) fn alvos(ctx: &mut PaintCtx, p: &Pergunta) {
    {
        let store = ctx.host.store_mut();
        let dicas = p
            .destinos
            .iter()
            .map(|&d| (i::PHYSICS_LAYER_MOVE_TO[d], "card.physics.tab_move_to_tip"))
            .chain([
                (i::PHYSICS_LAYER_MOVE_PICK, "card.physics.tab_move_to_tip"),
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
    camada_das_listas::alvo(hit, i::PHYSICS_LAYER_MOVE_PICK, p.chip);
    camada_das_listas::alvo(hit, i::PHYSICS_LAYER_MOVE_OK, p.ok);
    camada_das_listas::alvo(hit, i::PHYSICS_LAYER_MOVE_CANCEL, p.cancela);
}
