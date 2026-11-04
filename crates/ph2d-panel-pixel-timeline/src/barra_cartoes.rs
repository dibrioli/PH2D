//! A barra da linha do tempo na moldura sem cartão (`docs/CartoesUI/14_CHROME_TIMELINE_MOTION.md`): os mesmos grupos e
//! ids da [`crate::paint`], na fila de ícones, nos segmentos e no chip do cartão. Começa na tira do cabeçalho; o que não
//! couber desce em fileiras. A grelha não muda.

use crate::{PixelTimelinePanelState, ids};
use ph2d_a11y::NodeId;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::banda::{self, Icone, metrics as m};
use ph2d_editor_core::panel::cards::{CardCtx, Seg};
use ph2d_editor_core::pixel_panel::{PixelFrameAction, PixelPanelInfo, PixelSpeed};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::widget::dropdown_chip_width_for;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::TypeToken;

fn fila(id: NodeId, icon: IconId) -> Icone {
    Icone {
        id: Some(id),
        icon,
        on: false,
    }
}

/// Os grupos de ícones: navegar e tocar, e o menu Frame.
fn grupos(info: &PixelPanelInfo) -> Vec<Vec<Icone>> {
    let transporte = ids::TRANSPORT.iter().map(|(a, id, icon, _)| {
        let icon = if *a == PixelFrameAction::Play && info.playing {
            IconId::Stop
        } else {
            *icon
        };
        fila(*id, icon)
    });
    let mut v: Vec<Vec<Icone>> = vec![transporte.collect()];
    for g in ids::ACTION_GROUPS {
        v.push(
            g.iter()
                .map(|&i| fila(ids::ACTIONS[i].1, ids::ACTIONS[i].2))
                .collect(),
        );
    }
    v
}

/// Onde a próxima peça cabe: a tira do cabeçalho e, depois dela, as fileiras do corpo.
struct Fluxo {
    corpo: Rect,
    x: f32,
    y: f32,
    fim: f32,
    fileiras: usize,
}

impl Fluxo {
    fn lugar(&mut self, w: f32) -> (f32, f32) {
        if self.x + w > self.fim {
            self.fileiras += 1;
            self.x = self.corpo.x;
            self.fim = self.corpo.x + self.corpo.w;
            self.y = self.corpo.y + (self.fileiras - 1) as f32 * (m::FIELD_H + m::FIELD_ROW_GAP)
                - banda::SOBRA_DO_CABECALHO;
        }
        let p = (self.x, self.y);
        self.x += w + m::FIELD_GAP;
        p
    }
}

/// Pinta a barra e devolve onde a grelha começa.
pub(crate) fn paint(
    st: &mut PixelTimelinePanelState,
    ctx: &mut PaintCtx,
    info: &PixelPanelInfo,
    cab: Rect,
    corpo: Rect,
) -> f32 {
    let ink = CardInk::of_list(ctx.host.theme());
    let aberta = matches!(
        ctx.host.store().get(ids::PLAY_SPEED),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let px = TypeToken::Base.px();
    let chip_w = dropdown_chip_width_for(
        PixelSpeed::ALL
            .iter()
            .map(|&v| ctx.text_system.prefix_width(tr(ids::speed_key(v)), px))
            .fold(0.0_f32, f32::max),
        m::FIELD_H,
    );
    let p = info.playback;
    let speed_texto = tr(ids::speed_key(p.speed));
    let vegetal: Vec<Seg<'_>> = ids::TOGGLES
        .iter()
        .zip([info.onion, info.onion_tint])
        .map(|((_, id, k), on)| Seg {
            id: *id,
            icon: None,
            text: tr(k),
            on,
        })
        .collect();
    let toque: Vec<Seg<'_>> = ids::PLAY_TOGGLES
        .iter()
        .zip([p.once, p.all, p.subtags, p.rewind])
        .map(|((id, k, _), on)| Seg {
            id: *id,
            icon: None,
            text: tr(k),
            on,
        })
        .collect();
    let (store, hit) = ctx.host.store_and_hit_index_mut();
    let mut c = CardCtx::flutuante(
        ctx.scene,
        ctx.text_system,
        store,
        hit,
        ink,
        corpo.x,
        corpo.w,
    );
    let mut f = Fluxo {
        corpo,
        x: cab.x,
        y: cab.y,
        fim: cab.x + cab.w,
        fileiras: 0,
    };
    for g in grupos(info) {
        let (x, y) = f.lugar(banda::icones_w(g.len()));
        c.icones_em_fila(&g, x, y);
    }
    let w = c.segmentos_em_fila_w(&vegetal);
    let (x, y) = f.lugar(w);
    c.segmentos_em_fila(&vegetal, x, y);
    // O chip da velocidade e os interruptores do Play são um grupo: o menu do botão Play do Aseprite.
    let w = c.segmentos_em_fila_w(&toque);
    let (x, y) = f.lugar(chip_w + m::FIELD_GAP + w);
    let chip = Rect::new(x, y, chip_w, m::FIELD_H);
    c.lista_em(ids::PLAY_SPEED, speed_texto, aberta, chip);
    c.segmentos_em_fila(&toque, x + chip_w + m::FIELD_GAP, y);
    if aberta {
        st.speed_open = Some((chip, p.speed));
    }
    // A grelha sobe até à barra: cada linha a menos é uma camada a mais à vista (doc 14).
    f.y + m::FIELD_H + banda::VAO_ATE_A_FOLHA
}
