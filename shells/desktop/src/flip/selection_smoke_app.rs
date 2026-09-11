//! **A metade que precisa da `App`** do `selection_smoke` (W2/L5, 2026-09-11).
//!
//! O resto do módulo vive em [`ph2d_app_flip::selection_smoke`] — as leis e a cena, que não
//! precisam da shell. Aqui fica só o que toca os agregados DELA: `AppGfx` (o `FlipDoc`
//! vivo, o registo de ferramentas), o `HeroScreen` (o barramento dos painéis) e o
//! relógio. É a lista que o substrato da `ph2d-app-host` tem de cobrir para isto
//! também sair (W2 Fase B).

#[allow(unused_imports)]
use ph2d_app_flip::selection_smoke::*;
use ph2d_core::Vec2;
use ph2d_flip::{Hold, KeyKind, Rgba};
use std::sync::atomic::Ordering;

impl crate::App {
    /// Roda no prólogo do frame (ao lado dos outros smokes). No-op sem a env.
    pub(crate) fn flip_selection_smoke(&mut self) {
        if !enabled() || self.gfx.is_none() {
            return;
        }
        match FRAME.fetch_add(1, Ordering::Relaxed) {
            3 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("flip"));
                let oid = gfx.flip.push_object("Xform Smoke");
                let obj = gfx.flip.object_mut(oid).expect("objeto recém-criado");
                obj.fps = 12.0;
                let l = obj.add_layer("L");
                if let Some(d) = obj.insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe) {
                    let dr = obj.drawing_mut(d).expect("desenho");
                    // Retângulo SELECIONADO (o alvo do gizmo) à esquerda.
                    dr.strokes.push(shape(
                        &[
                            Vec2::new(-3.0, -1.0),
                            Vec2::new(-1.0, -1.0),
                            Vec2::new(-1.0, 1.0),
                            Vec2::new(-3.0, 1.0),
                        ],
                        Rgba::new(0.85, 0.2, 0.7, 1.0),
                        true,
                    ));
                    // Triângulo NÃO selecionado à direita — a testemunha de que o resto
                    // do desenho fica parado.
                    dr.strokes.push(shape(
                        &[
                            Vec2::new(1.5, -1.0),
                            Vec2::new(3.0, -1.0),
                            Vec2::new(2.25, 1.2),
                        ],
                        Rgba::new(0.2, 0.7, 0.9, 1.0),
                        false,
                    ));
                }
                self.playhead.pause();
            }
            // Entra no Edit pela porta REAL (o mesmo evento do pill do painel). O
            // domínio começa em Stroke; a seleção do retângulo já está armada.
            8 => {
                if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                    hero.bus
                        .push(ph2d_editor::action_bus::EditorAction::ToolPanelEvent(
                            ph2d_editor::tool::PanelEvent::Click(ph2d_editor::ids::FLIP_MODE_EDIT),
                        ));
                }
                eprintln!(
                    "[xform-smoke] retangulo VAZADO e SELECIONADO (roxo) + triangulo (azul); \
                     modo Edit, dominio Stroke. O gizmo da SELECAO enquadra o retangulo: quina \
                     = rotate(anel)/scale, borda = scale-1-eixo. AREA do gizmo: arrastar do \
                     MEIO do retangulo (sem tinta ali) AGARRA a selecao. Confira: o triangulo \
                     NAO se mexe; Ctrl+Z desfaz o gesto inteiro; clicar no vazio FORA da caixa \
                     desmarca e o gizmo some; as 3 linhas do triangulo e as 4 do retangulo sao \
                     todas clicaveis (a costura, BUGS #18). O toggle Select:Point SOME com o \
                     gizmo na hora porque comeca DESSELECIONADO; UMA ancora = sem gizmo (so o \
                     realce, e ela arrasta), DUAS ou mais = gizmo enquadrando SO elas -- com \
                     FOLGA: os handles ficam FORA das ancoras e nunca se sobrepoem, nem com \
                     duas na mesma horizontal. Voltar a Stroke promove por any() e o gizmo \
                     volta ao traco inteiro."
                );
            }
            9 => self.any_input_this_frame = true, // arma o baseline do undo
            _ => {}
        }
    }
}
