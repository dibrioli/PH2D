//! **A cena pronta para o smoke do Blend Object VIVO** (`PH2D_BLEND_SMOKE`, ADR-0122).
//!
//! O Blend destrutivo (a `BlendSession` de [`crate::vec_blend`]) tem o próprio smoke em
//! `PH2D_BUILD_SMOKE=7..=9`. Este é o do **objeto VIVO**: um objeto único, não-destrutivo, cujos
//! passos são virtuais e cujas fontes seguem editáveis. O que se olha aqui é UMA coisa: os passos
//! aparecem ENTRE as formas, e mover uma fonte (o gizmo) refaz a transição — sem re-clicar "Blend".
//!
//! - `PH2D_BLEND_SMOKE=1` — **estrela → círculo**, 5 passos, VIVO. O par que o Enio testou. Depois
//!   de aparecer, arraste uma das duas formas: os passos se recalculam sozinhos.
//! - `PH2D_BLEND_SMOKE=2` — **retângulo → estrela → círculo** (3 formas em CADEIA), 4 passos por
//!   elo. É a capacidade nova do ADR-0122 (até 5 formas); a transição corre pelas três na ordem.

use ph2d_vec_scene::{Paint, Rgba8, ShapeKind, VecPath, cook};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

/// O frame corrente do roteiro (o hook não pode acrescentar campo em `App`).
static FRAME: AtomicU32 = AtomicU32::new(0);

/// O nível pedido, lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn level() -> u32 {
    static LEVEL: OnceLock<u32> = OnceLock::new();
    *LEVEL.get_or_init(|| {
        std::env::var("PH2D_BLEND_SMOKE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    })
}

fn shape(kind: ShapeKind, a: [f64; 2], b: [f64; 2], v: &[f64], rgb: [u8; 3]) -> VecPath {
    let mut p = cook(kind, a, b, v);
    p.fill = Some(Paint::solid(Rgba8::new(rgb[0], rgb[1], rgb[2], 255)));
    p
}

impl crate::App {
    /// Roda no prólogo do frame, ao lado do [`crate::App::build_smoke`]. No-op sem a env.
    pub(crate) fn blend_smoke(&mut self) {
        let level = level();
        if level == 0 || self.gfx.is_none() {
            return;
        }
        match FRAME.fetch_add(1, Ordering::Relaxed) {
            // A cena. A geometria entra em MUNDO com o `Transform` na identidade — é como a
            // Shape tool deixa uma forma recém-desenhada; o `settle_origins` do frame a centra
            // no local 0 e põe a pose na entidade (ADR-0111/0112).
            3 if level == 1 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
                let scene = &mut gfx.vec_scene;
                scene.push_path(shape(
                    ShapeKind::Star,
                    [-3.4, -1.0],
                    [-1.4, 1.0],
                    &[5.0, 0.45, 0.0],
                    [70, 110, 190],
                ));
                scene.push_path(shape(
                    ShapeKind::Ellipse,
                    [1.4, -1.0],
                    [3.4, 1.0],
                    &[],
                    [200, 120, 80],
                ));
            }
            3 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
                let scene = &mut gfx.vec_scene;
                scene.push_path(shape(
                    ShapeKind::Rectangle,
                    [-4.4, -1.0],
                    [-2.4, 1.0],
                    &[],
                    [70, 110, 190],
                ));
                scene.push_path(shape(
                    ShapeKind::Star,
                    [-1.0, -1.0],
                    [1.0, 1.0],
                    &[5.0, 0.45, 0.0],
                    [200, 120, 80],
                ));
                scene.push_path(shape(
                    ShapeKind::Ellipse,
                    [2.4, -1.0],
                    [4.4, 1.0],
                    &[],
                    [110, 190, 130],
                ));
            }
            // Cria o Blend Object VIVO sobre as formas da cena, na ordem de z. As fontes
            // sobrevivem e seguem editáveis — o `create` só empurra o spine (invisível) e enfileira
            // o componente; o `sync`/`upkeep`/`recook` do frame fazem o resto.
            8 => {
                let ids: Vec<u64> = self
                    .gfx
                    .as_ref()
                    .expect("gfx")
                    .vec_scene
                    .paths()
                    .iter()
                    .map(|p| p.id)
                    .collect();
                self.vec_set_draw_mode(ph2d_tool_vector::DrawMode::Select);
                let Some(gfx) = self.gfx.as_mut() else { return };
                let xf = crate::vec_transform::build(&gfx.sim, &self.vec_entities);
                let steps = if level == 1 { 5 } else { 4 };
                self.vec_blend_pending =
                    crate::blend_live::create(&mut gfx.vec_scene, &xf, &ids, steps);
                self.vec_pen.select_many(&ids); // as FONTES seguem selecionáveis/editáveis
                self.any_input_this_frame = true;
                eprintln!(
                    "[blend-smoke] Blend Object VIVO sobre {} forma(s), {steps} passos/elo. \
                     Arraste uma fonte: a transicao se refaz sozinha (ADR-0122).",
                    ids.len()
                );
            }
            _ => {}
        }
    }
}
