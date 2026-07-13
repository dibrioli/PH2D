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
            10 if level == 2 => {
                self.build_down(IN_STAR, false, false);
                self.build_move(IN_PENT);
            }
            f if f > 10 && level == 2 => {
                self.build_move(IN_STAR);
                self.build_move(IN_PENT);
            }
            // **Níveis 3 e 4 — o undo pelo CAMINHO REAL.** Nada de chamar `build_up` e
            // `undo_request` na mão: aqui entram `on_mouse_input` e `key_input`, que é por
            // onde o winit entra. É a diferença entre provar o mecanismo e provar o produto.
            //
            // O baseline precisa ser ARMADO: o hook cria as formas sem input nenhum, e o undo
            // global registra por diff **em frames com input** — sem isto o 1º clique
            // arrastaria "as 3 formas nasceram" para dentro do mesmo passo, e o Ctrl+Z
            // voltaria para a cena VAZIA. (No produto isso não acontece: desenhar é input.)
            9 if level >= 3 => {
                self.any_input_this_frame = true;
                self.smoke_state("baseline (3 formas)");
            }
            12 if level >= 3 => self.smoke_click(IN_STAR),
            13 if level >= 3 => self.smoke_state("depois do clique"),
            14 if level == 3 || level == 4 => self.smoke_undo(false), // Ctrl+Z
            15 if level == 3 || level == 4 => self.smoke_state("depois do UNDO"),
            // Nível 3: redo direto. Nível 4: um clique no VAZIO ANTES do redo — é aí que um
            // passo espúrio apareceria, e um passo espúrio **limpa a pilha de redo**.
            16 if level == 4 => self.smoke_click([9.0, 9.0]),
            17 if level == 4 => self.smoke_state("depois de um clique no nada"),
            18 if level == 3 || level == 4 => self.smoke_undo(true), // Ctrl+Shift+Z
            19 if level == 3 || level == 4 => self.smoke_state("depois do REDO"),
            // **Nível 5 — os BOTÕES da barra**, clicados com o mouse de verdade: o ponteiro
            // acha o chip no hit-index, o widget emite o Click, o chrome despacha, o bus é
            // drenado e o shell desfaz. É o caminho inteiro, sem atalho nenhum.
            14 if level == 5 => self.smoke_rail_click(ph2d_editor::ids::TOOL_UNDO, "Undo"),
            15 if level == 5 => self.smoke_state("depois do BOTÃO Undo"),
            16 if level == 5 => self.smoke_rail_click(ph2d_editor::ids::TOOL_REDO, "Redo"),
            17 if level == 5 => self.smoke_state("depois do BOTÃO Redo"),
            _ => {}
        }
    }

    /// Um clique no ponto de MUNDO `w`, pelo caminho do winit: cursor → botão → botão.
    fn smoke_click(&mut self, w: [f64; 2]) {
        let Some(gfx) = self.gfx.as_ref() else { return };
        let win = gfx.surface.size();
        let s = gfx.camera.world_to_screen([w[0] as f32, w[1] as f32], win);
        self.on_cursor_moved(winit::dpi::PhysicalPosition::new(
            f64::from(s.0),
            f64::from(s.1),
        ));
        self.on_mouse_input(
            winit::event::ElementState::Pressed,
            winit::event::MouseButton::Left,
        );
        self.on_mouse_input(
            winit::event::ElementState::Released,
            winit::event::MouseButton::Left,
        );
    }

    /// Ctrl+Z (ou Ctrl+Shift+Z) pelo roteamento REAL do teclado.
    fn smoke_undo(&mut self, redo: bool) {
        use winit::keyboard::ModifiersState;
        let mods = if redo {
            ModifiersState::CONTROL | ModifiersState::SHIFT
        } else {
            ModifiersState::CONTROL
        };
        self.modifiers = mods;
        self.key_input(
            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyZ),
            winit::event::ElementState::Pressed,
            false,
            None,
        );
        self.modifiers = ModifiersState::empty();
    }

    /// Acha um chip do rail no hit-index e o clica **com o ponteiro** (é o caminho do
    /// usuário: pixel → hit → widget → chrome → bus → shell).
    fn smoke_rail_click(&mut self, id: ph2d_editor::NodeId, label: &str) {
        let Some(hero) = self.gfx.as_ref().and_then(|g| g.hero_screen.as_ref()) else {
            return;
        };
        let mut found = None;
        'scan: for y in (0..1000).step_by(4) {
            for x in (0..120).step_by(4) {
                if hero.hit_index.hit(x as f32, y as f32) == Some(id) {
                    found = Some((x as f32, y as f32));
                    break 'scan;
                }
            }
        }
        let Some((x, y)) = found else {
            eprintln!("[build-smoke] o chip {label} NÃO está no hit-index (não é clicável!)");
            return;
        };
        eprintln!("[build-smoke] clicando no botão {label} em ({x}, {y})");
        self.on_cursor_moved(winit::dpi::PhysicalPosition::new(
            f64::from(x),
            f64::from(y),
        ));
        self.on_mouse_input(
            winit::event::ElementState::Pressed,
            winit::event::MouseButton::Left,
        );
        self.on_mouse_input(
            winit::event::ElementState::Released,
            winit::event::MouseButton::Left,
        );
    }

    fn smoke_state(&mut self, when: &str) {
        let n = self.gfx.as_ref().map_or(0, |g| g.vec_scene.paths().len());
        eprintln!(
            "[build-smoke] {when}: {n} path(s) · undo={} redo={}",
            self.undo.depth(),
            usize::from(self.undo.can_redo()),
        );
    }
}
