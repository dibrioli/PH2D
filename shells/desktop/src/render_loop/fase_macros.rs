//! Fase do quadro: **as MACROS** (ADR-0186, W5d) — composição só: os pedidos do painel e da linha
//! de etapas, os passos que as famílias gravaram neste quadro e o instantâneo do painel. A lei vive
//! na `ph2d-app-macros`.

use ph2d_app_macros::{Cena, Familias, erros};
use ph2d_tool_painter::PainterTool;

/// O relógio das macros: microssegundos desde o 1.º quadro (o núcleo nunca lê o de parede).
fn agora_us() -> u64 {
    static INICIO: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    let us = INICIO
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_micros();
    u64::try_from(us).unwrap_or(u64::MAX)
}

impl crate::App {
    /// Depois de o Painter consumir a entrada do quadro.
    pub(super) fn fase_macros(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let pintor = ph2d_editor_core::ToolId::new("painter");
        let painter_activo = gfx.tools.active().is_some_and(|t| t.id() == pintor);
        let painter = gfx
            .tools
            .tool_by_id_mut(&pintor)
            .and_then(|t| t.as_any_mut().downcast_mut::<PainterTool>());
        let mut f = Familias {
            painter,
            painter_activo,
            motion: &mut gfx.motion,
        };
        let mut c = Cena {
            sim: &mut gfx.sim,
            timeline: &mut self.timeline,
            intents: &mut self.timeline_intents,
        };
        let agora = agora_us();
        ph2d_app_macros::com(|m| m.quadro(&mut f, &mut c, agora));
        let hero = gfx.hero_screen.as_ref();
        let escolhido = hero.and_then(|h| h.gizmo.selection);
        // O erro da reprodução em cena: só a shell alcança os palcos.
        let erro = escolhido.and_then(|b| {
            gfx.painter_macros
                .erro_de(b)
                .map(erros::do_painter)
                .or_else(|| self.motion_shell.palcos.erro_de(b).map(erros::do_motion))
        });
        // O cartão MACRO do Inspector: o componente (`build_info`) e as etapas da família.
        #[cfg(feature = "panel-inspector")]
        ph2d_panel_inspector::set_current_inspector_macro(escolhido.and_then(|b| {
            let quantos = hero.map_or(0, |h| h.gizmo.selected_len());
            let mut i = ph2d_app_components::macro_inspector::build_info(c.sim, b, quantos)?;
            if let Some(r) = ph2d_app_macros::com(|m| m.resumo(c.sim, b, erro.clone())) {
                (i.etapas, i.editada_desde, i.erro) = (r.etapas, r.editada_desde, r.erro);
            }
            Some(i)
        }));
        #[cfg(feature = "panel-macros")]
        {
            use ph2d_editor_core::panel::Panel;
            if !hero
                .is_some_and(|h| h.is_panel_visible(<ph2d_panel_macros::MacrosPanel as Panel>::ID))
            {
                return;
            }
            let info = ph2d_app_macros::com(|m| m.info(&f, c.sim, escolhido, erro));
            ph2d_panel_macros::set_current_macros(info);
        }
    }
}
