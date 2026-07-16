//! **A cena pronta para o smoke do gizmo de SELEÇÃO** (`PH2D_FLIP_XFORM_SMOKE=1`, §4.A).
//!
//! O Enio não monta cena (feedback_ready_to_smoke_example): o app abre com 1 objeto
//! Flip de arte EXCLUSIVA — um retângulo (SELECIONADO) + um triângulo (não) —, a tool
//! Flip em modo **Edit** e o gizmo da SELEÇÃO já visível enquadrando o retângulo.
//!
//! Roteiro: arrastar uma **quina** gira (anel de hover) / escala o retângulo em torno
//! do centro DELE; uma **borda** escala num eixo; arrastar a arte (fora dos handles)
//! **move** a seleção (o gesto do W6.1). Conferir que o **triângulo não se mexe**, que
//! **Ctrl+Z** desfaz o gesto inteiro (1 passo), e que clicar no vazio (desmarca) **some**
//! com o gizmo. Trocar pro domínio **Point** e selecionar meia geometria: o gizmo passa
//! a enquadrar SÓ os pontos selecionados e gira só eles.

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Hold, KeyKind, Point, Rgba};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

static FRAME: AtomicU32 = AtomicU32::new(0);

fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_XFORM_SMOKE").is_some())
}

/// Uma polilinha fechada pelos vértices dados, largura de tela grossa.
fn shape(verts: &[Vec2], color: Rgba, selected: bool) -> FlipStroke {
    let mut s = FlipStroke::new();
    for &p in verts {
        s.push_point(Point {
            pos: p,
            width: 6.0,
            opacity: 1.0,
            color,
        });
    }
    s.closed = true;
    s.selected = selected;
    s
}

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
                    "[xform-smoke] retangulo SELECIONADO (roxo) + triangulo (azul); modo Edit, \
                     dominio Stroke. O gizmo da SELECAO enquadra o retangulo: quina = \
                     rotate(anel)/scale, borda = scale-1-eixo, arrastar a arte = move. \
                     Confira: o triangulo NAO se mexe; Ctrl+Z desfaz o gesto inteiro; clicar \
                     no vazio desmarca e o gizmo some. No dominio Point o gizmo enquadra so os \
                     pontos selecionados."
                );
            }
            9 => self.any_input_this_frame = true, // arma o baseline do undo
            _ => {}
        }
    }
}
