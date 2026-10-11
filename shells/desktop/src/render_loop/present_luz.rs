//! ⭐ **A LUZ 2D do quadro adquirido** (W3–W5e) — irmão de `present.rs` pelo tecto de LOC por função:
//! a re-coleta, o solver com as camadas das famílias que bloqueiam e não são sprite nem caminho (o
//! traço do Flip, as formas do Motion — `CamadaDeOclusao`), e os três receptores (o passe de sprite,
//! a colagem do vetor e o blit do Flip). Corre antes do primeiro passe de sprite. ⛔ Sem `Luz`: nada
//! disto muda o quadro.

use crate::AppGfx;

impl crate::App {
    pub(super) fn present_luz(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let AppGfx {
            surface,
            renderer,
            present,
            tonemap,
            band_blit,
            motion,
            sim,
            luz2d,
            flip,
            flip_composite,
            flip_compose,
            ..
        } = gfx;
        luz2d.recolher(sim.world());
        let flip_models = ph2d_flip_entities::transform::build(sim, &self.flip_state.entities);
        let mut flip_luz = ph2d_app_flip::pass::luz::OclusaoDoFlip {
            doc: flip,
            composite: flip_composite,
            models: &flip_models,
            playhead: &self.playhead,
            entidades: &self.flip_state.entities,
        };
        let placa = &mut self.motion_shell.placa;
        let mut motion_luz = ph2d_app_motion::motion_shape_placa::luz::OclusaoDoMotion::do_quadro(
            placa,
            motion,
            sim.world(),
        );
        luz2d.gravar_com(
            surface.gpu(),
            present,
            renderer,
            &mut [&mut flip_luz, &mut motion_luz],
        );
        luz2d.ligar(surface.gpu(), renderer, band_blit, tonemap);
        let na_tela = luz2d.na_tela();
        flip_compose.receber_luz(surface.gpu(), na_tela.as_ref().map(|(l, m)| (l, *m)));
    }
}
