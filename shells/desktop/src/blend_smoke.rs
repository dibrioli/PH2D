//! **A cena pronta para o smoke do Blend Object VIVO** (`PH2D_BLEND_SMOKE`, ADR-0122).
//!
//! O Blend destrutivo (a `BlendSession` de [`crate::vec_blend`]) tem o próprio smoke em
//! `PH2D_BUILD_SMOKE=7..=9`. Este é o do **objeto VIVO**: um objeto único, não-destrutivo, cujos
//! passos são virtuais e cujas fontes seguem editáveis. O que se olha aqui é UMA coisa: os passos
//! aparecem ENTRE as formas, e mover uma fonte (o gizmo) refaz a transição — sem re-clicar "Blend".
//!
//! - `PH2D_BLEND_SMOKE=1` — **estrela → elipse**, 5 passos, VIVO. Depois de aparecer, arraste ou
//!   **gire** qualquer das duas formas: os passos se recalculam. A ponta é uma elipse (não um
//!   círculo) DE PROPÓSITO — um círculo é rotacionalmente simétrico, e girá-lo não muda nada; a
//!   elipse tem orientação, então girar a ÚLTIMA forma também influencia os intermediários.
//! - `PH2D_BLEND_SMOKE=2` — **retângulo → estrela → elipse** (3 formas em CADEIA), 4 passos por
//!   elo. É a capacidade nova do ADR-0122 (até 5 formas); a transição corre pelas três na ordem.
//!   Gire a do MEIO: os ângulos das intermediárias dos dois lados se adaptam.
//! - `PH2D_BLEND_SMOKE=3` — o **SPINE editável**: estrela → elipse com um spine CURVO autorado. Os
//!   6 passos **fluem ao longo do arco**, não pela reta. Modo Node no objeto: arraste os pontos do
//!   spine e os passos re-fluem (é o "path unindo as formas, editável" do pedido do Enio).
//! - `PH2D_BLEND_SMOKE=4` — **Pick Shapes** (C2b): três formas + o modo **Pick** já ativo. Clique
//!   as formas **na ordem** que quiser (a linha azul costura a cadeia; clicar de novo remove),
//!   depois Painel Vector > Blend. A ordem do blend é a de CLIQUE, não a de z (o pedido do Enio de
//!   "escolher as formas na ordem que queremos").

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
                // Elipse NÃO-circular (2×1,3): orientada, então girá-la influencia os passos (um
                // círculo 2×2 seria simétrico e "não influenciaria" — foi o que confundiu no smoke).
                scene.push_path(shape(
                    ShapeKind::Ellipse,
                    [1.4, -0.65],
                    [3.4, 0.65],
                    &[],
                    [200, 120, 80],
                ));
            }
            // A cena do SPINE editável: estrela → elipse, bem separadas (há espaço para a curva
            // subir). No frame 8 o blend nasce com um spine CURVO e autorado.
            3 if level == 3 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
                let scene = &mut gfx.vec_scene;
                scene.push_path(shape(
                    ShapeKind::Star,
                    [-4.4, -1.0],
                    [-2.4, 1.0],
                    &[5.0, 0.45, 0.0],
                    [70, 110, 190],
                ));
                scene.push_path(shape(
                    ShapeKind::Ellipse,
                    [2.4, -0.65],
                    [4.4, 0.65],
                    &[],
                    [200, 120, 80],
                ));
            }
            // Pick Shapes (C2b): 3 formas distintas, espalhadas. O blend NÃO nasce pronto — o
            // artista entra no modo Pick (frame 8) e as clica na ordem que quiser.
            3 if level == 4 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
                let scene = &mut gfx.vec_scene;
                scene.push_path(shape(
                    ShapeKind::Star,
                    [-4.4, -1.0],
                    [-2.4, 1.0],
                    &[5.0, 0.45, 0.0],
                    [70, 110, 190],
                ));
                scene.push_path(shape(
                    ShapeKind::Ellipse,
                    [-0.6, -0.7],
                    [1.4, 0.7],
                    &[],
                    [200, 120, 80],
                ));
                scene.push_path(shape(
                    ShapeKind::Rectangle,
                    [2.4, -1.0],
                    [4.4, 1.0],
                    &[],
                    [110, 190, 130],
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
                    [2.4, -0.65],
                    [4.4, 0.65],
                    &[],
                    [110, 190, 130],
                ));
            }
            // O SPINE editável: cria o blend e o entrega com um spine CURVO e AUTORADO — os passos
            // FLUEM ao longo do arco (não pela reta). É o que a edição no modo Node faz ao vivo:
            // selecione o objeto, modo Node, arraste os pontos do spine e os passos re-fluem.
            8 if level == 3 => {
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
                let mut made = crate::blend_live::create(&mut gfx.vec_scene, &xf, &ids, 6);
                if let Some((spine, blend)) = made.as_mut() {
                    blend.spine_authored = true; // o artista "editou" a curva
                    // Curva o spine num arco: pontas nos centros das fontes, pico acima do meio.
                    if let Some(p) = gfx.vec_scene.path_mut(*spine)
                        && p.verts.len() == 2
                    {
                        let (a, b) = (p.verts[0].anchor, p.verts[1].anchor);
                        let peak = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5 + 3.0];
                        p.verts = [a, peak, b].map(ph2d_vec_scene::VecVertex::corner).to_vec();
                    }
                }
                if let Some((spine, _)) = &made {
                    self.vec_pen.select_many(&[*spine]);
                }
                self.vec_blend_pending = made;
                // Entra em modo Node: o spine sobe para o TOPO (acima das formas e dos passos) e
                // aparece com as âncoras — pronto para arrastar (ADR-0122). Em Select ele ficaria
                // no z dele (traço sutil), possivelmente sob as formas.
                self.vec_set_draw_mode(ph2d_tool_vector::DrawMode::Node);
                self.any_input_this_frame = true;
                eprintln!(
                    "[blend-smoke] SPINE editavel: 6 passos ao longo de um ARCO autorado, em modo \
                     Node (o spine no TOPO, com ancoras). Arraste os pontos do spine e os passos \
                     re-fluem; as PONTAS voltam para os centros das fontes (ADR-0122)."
                );
            }
            // Pick Shapes (C2b): entra no modo Pick. NÃO cria o blend — o artista clica as formas
            // na ordem que quiser (a linha azul mostra a cadeia se formando) e aperta Blend.
            8 if level == 4 => {
                self.vec_set_draw_mode(ph2d_tool_vector::DrawMode::PickBlend);
                self.any_input_this_frame = true;
                eprintln!(
                    "[blend-smoke] Pick Shapes: modo Pick ATIVO. Clique as 3 formas NA ORDEM que \
                     quiser (a linha azul costura a cadeia; clicar de novo numa escolhida a \
                     remove), depois Painel Vector > Blend. A ordem do blend e a de CLIQUE, nao a \
                     de z (ADR-0122 C2b)."
                );
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
                let made = crate::blend_live::create(&mut gfx.vec_scene, &xf, &ids, steps);
                // Seleciona o OBJETO blend (o spine) — assim o slider Steps do painel já mira nele
                // (arraste-o e veja os passos mudarem ao vivo, inclusive além de 12). Clique numa
                // fonte para movê-la/girá-la: a transição se refaz sozinha.
                if let Some((spine, _)) = &made {
                    self.vec_pen.select_many(&[*spine]);
                }
                self.vec_blend_pending = made;
                self.any_input_this_frame = true;
                eprintln!(
                    "[blend-smoke] Blend Object VIVO sobre {} forma(s), {steps} passos/elo. \
                     Painel Vector > Blend: arraste Steps (retuna ao vivo); clique numa fonte para \
                     move-la/gira-la (ADR-0122).",
                    ids.len()
                );
            }
            _ => {}
        }
    }
}
