//! **A cena pronta para o smoke do Shape Builder** (`PH2D_BUILD_SMOKE`).
//!
//! O Enio não deve ter de montar a cena para testar a ferramenta — e o agente que só
//! *imagina* o roteiro do smoke escreve gates verdes sobre um produto quebrado (foi o que
//! aconteceu com a 1ª versão desta feature). Este hook monta a cena do print dele —
//! **pentágono + estrela + retângulo arredondado, sobrepostos** —, seleciona as três e entra
//! no modo Build. O canvas já abre como mesa de trabalho.
//!
//! - `PH2D_BUILD_SMOKE=1` — a cena, selecionada, no modo Build. **Passe o mouse** (o realce
//!   segue o cursor), **arraste** para unir, **Alt+arraste** para apagar.
//! - `PH2D_BUILD_SMOKE=2` — idem, e o gesto é dirigido por CÓDIGO: o dedo pousa e arrasta por
//!   duas faces, sem soltar. É o harness visual do véu — a única parte da feature cujo
//!   oráculo é o pixel, e a que estava sem gate quando o Enio reprovou.

use ph2d_vec_scene::{Paint, Rgba8, ShapeKind, VecPath, cook};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

/// O frame corrente do roteiro (o hook não pode acrescentar campo em `App`).
static FRAME: AtomicU32 = AtomicU32::new(0);

/// Dois pontos de MUNDO dentro de faces diferentes: um na estrela, outro no pentágono.
const IN_STAR: [f64; 2] = [0.35, 0.15];
const IN_PENT: [f64; 2] = [-1.2, 0.0];

/// O nível pedido, lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn level() -> u32 {
    static LEVEL: OnceLock<u32> = OnceLock::new();
    *LEVEL.get_or_init(|| {
        std::env::var("PH2D_BUILD_SMOKE")
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
    /// Roda no prólogo do frame, ANTES do `build_session_upkeep`. No-op sem a env.
    pub(crate) fn build_smoke(&mut self) {
        let level = level();
        if level == 0 || self.gfx.is_none() {
            return;
        }
        match FRAME.fetch_add(1, Ordering::Relaxed) {
            // A cena. A geometria entra em MUNDO com o `Transform` na identidade — é como a
            // Shape tool deixa uma forma recém-desenhada; o `settle_origins` do frame a
            // centra no local 0 e põe a pose na entidade (ADR-0111/0112).
            3 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
                let scene = &mut gfx.vec_scene;
                scene.push_path(shape(
                    ShapeKind::RoundRect,
                    [-1.6, -1.1],
                    [1.6, 1.1],
                    &[0.4, 0.0, 0.0, 0.0, 0.0],
                    [70, 110, 190],
                ));
                scene.push_path(shape(
                    ShapeKind::Polygon,
                    [-1.9, -0.9],
                    [-0.1, 0.9],
                    &[5.0, 0.0],
                    [200, 120, 80],
                ));
                scene.push_path(shape(
                    ShapeKind::Star,
                    [-0.3, -1.0],
                    [1.7, 1.0],
                    &[5.0, 0.45, 0.0],
                    [110, 190, 130],
                ));
            }
            // Seleciona as três e entra no Build — o estado em que o Enio começa a testar.
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
                self.vec_pen.select_many(&ids);
                self.vec_set_draw_mode(ph2d_tool_vector::DrawMode::Build);
                eprintln!(
                    "[build-smoke] cena pronta, {} formas, modo Build",
                    ids.len()
                );
            }
            // O dedo pousa e arrasta por duas faces — e NÃO solta (o véu das pintadas fica
            // na tela para ser olhado).
            10 if level >= 2 => {
                self.build_down(IN_STAR, false, false);
                self.build_move(IN_PENT);
            }
            f if f > 10 && level >= 2 => {
                self.build_move(IN_STAR);
                self.build_move(IN_PENT);
            }
            _ => {}
        }
    }
}
