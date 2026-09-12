//! **A cena pronta para o smoke do gizmo de POSE** (`PH2D_FLIP_POSE_SMOKE=1`, W7.5).
//!
//! O Enio não monta cena (feedback_ready_to_smoke_example): o app abre com 1 objeto
//! Flip cuja chave 0 desenha um quadrado, a chave 12 é uma **INSTÂNCIA** dele já
//! movida (pose ≠ identidade), o playhead parado NA instância e a tool Flip em modo
//! **Edit** — o gizmo da pose já visível enquadrando a arte posada.
//!
//! Roteiro: arrastar uma **quina** gira (anel de hover) / escala; uma **borda**
//! escala num eixo; arrastar a ARTE move (o gesto de sempre). Conferir que a chave 0
//! e o objeto **não se mexem**, que Ctrl+Z desfaz o gesto inteiro, e que voltar o
//! playhead à chave 0 (arte exclusiva) **não** mostra gizmo de pose.

use ph2d_core::Vec2;
use ph2d_flip::{DupMode, FlipStroke, Hold, KeyKind, Point, Pose, Rgba};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

/// O frame corrente do roteiro (mesmo padrão do `build_smoke`).
pub static FRAME: AtomicU32 = AtomicU32::new(0);

pub fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_POSE_SMOKE").is_some())
}

/// Um quadrado de traço grosso (px de tela) entre `a` e `b`, fechado.
pub fn square(a: Vec2, b: Vec2, color: Rgba) -> FlipStroke {
    let mut s = FlipStroke::new();
    for p in [
        Vec2::new(a.x, a.y),
        Vec2::new(b.x, a.y),
        Vec2::new(b.x, b.y),
        Vec2::new(a.x, b.y),
    ] {
        s.push_point(Point {
            pos: p,
            width: 6.0,
            opacity: 1.0,
            color,
        });
    }
    s.closed = true;
    s
}

/// **Arma a cena deste smoke** (W2/L5 Fase B, 2026-09-11).
///
/// ⭐ Era um `impl crate::App`, e o que a `App` de facto dava eram **quatro tipos de crates de
/// módulo**. ⚠️ O `HeroScreen` entra por PARÂMETRO e isso tem precedente no próprio substrato
/// ([`ph2d_app_host::canvas_area::visible`]): o que o HOWTO §1.5 proíbe é um **método do trait
/// DEVOLVER** um handle — o que deixaria a família alcançar tudo, a qualquer hora. Um parâmetro
/// é a shell a escolher o que entrega, num sítio que ela controla.
///
/// Devolve **se armou input autorado** — o `any_input_this_frame` é da shell, e um `&mut bool`
/// atravessaria a fronteira por um campo em vez de por um valor.
/// Roda no prólogo do frame (ao lado do `build_smoke`). No-op sem a env.
pub fn arm(
    flip: &mut ph2d_flip::FlipDoc,
    tools: &mut ph2d_editor::ToolRegistry,
    hero: Option<&mut ph2d_editor::HeroScreen>,
    playhead: &mut ph2d_core::Playhead,
) -> bool {
    let mut armou = false;
    if !enabled() {
        return false;
    }
    match FRAME.fetch_add(1, Ordering::Relaxed) {
        // A cena: 1 objeto, chave 0 = quadrado, chave 12 = INSTÂNCIA movida.
        3 => {
            let _ = tools.set_active(&ph2d_editor::ToolId::new("flip"));
            let oid = flip.push_object("Pose Smoke");
            let obj = flip.object_mut(oid).expect("objeto recém-criado");
            obj.fps = 12.0;
            let l = obj.add_layer("L");
            if let Some(d) = obj.insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe) {
                obj.drawing_mut(d).expect("desenho").strokes.push(square(
                    Vec2::new(-1.0, -1.0),
                    Vec2::new(1.0, 1.0),
                    Rgba::new(0.85, 0.2, 0.7, 1.0),
                ));
            }
            assert!(
                obj.duplicate_frame(l, 0, 12, DupMode::Instance),
                "a instância do smoke tem de nascer"
            );
            // A instância já MOVIDA — pose ≠ identidade, o alvo do gizmo.
            obj.set_frame_pose(l, 12, Pose::from_translation(Vec2::new(2.5, 0.8)));
        }
        // O estado em que o Enio começa: playhead NA instância, modo Edit.
        8 => {
            playhead.pause();
            playhead.seek_frame(12, 12.0);
            if let Some(hero) = hero {
                // O MESMO evento que o pill Edit do painel emite — o modo troca
                // pela porta real (`FlipTool::handle_panel_event`).
                hero.bus
                    .push(ph2d_editor::action_bus::EditorAction::ToolPanelEvent(
                        ph2d_editor::tool::PanelEvent::Click(ph2d_editor::ids::FLIP_MODE_EDIT),
                    ));
            }
            eprintln!(
                "[pose-smoke] chave 0 = quadrado; chave 12 = INSTÂNCIA movida (+2.5, +0.8); \
                 playhead no 12, modo Edit. O gizmo da pose enquadra a arte posada: \
                 quina = rotate (anel)/scale, borda = scale-1-eixo, arrastar a arte = move. \
                 Confira: a chave 0 e o objeto NÃO se mexem; Ctrl+Z desfaz o gesto inteiro; \
                 na chave 0 (arte exclusiva) o gizmo de pose NÃO aparece."
            );
        }
        // Arma o baseline do undo (a cena nasceu sem input; sem isto o 1º gesto
        // arrastaria a criação para dentro do mesmo passo — igual ao build_smoke).
        9 => armou = true,
        _ => {}
    }
    armou
}
