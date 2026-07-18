//! **A cena pronta para o smoke do Shape Builder** (`PH2D_BUILD_SMOKE`).
//!
//! O Enio não deve ter de montar a cena para testar a ferramenta — e o agente que só
//! *imagina* o roteiro do smoke escreve gates verdes sobre um produto quebrado (foi o que
//! aconteceu com a 1ª versão desta feature). Este hook monta a cena do print dele —
//! **pentágono + estrela + retângulo arredondado, sobrepostos** —, seleciona as três e entra
//! no modo Build. O canvas já abre como mesa de trabalho.
//!
//! - `PH2D_BUILD_SMOKE=12` — a cena do **WARP GROUP** (ADR-0129 Fatia 3): DUAS elipses sob UMA
//!   gaiola. As duas curvam pela mesma perspectiva (NODE), e o gizmo do Select abraça e move as
//!   duas juntas (SELECT). É o que separa um container de dois envelopes soltos.
//! - `PH2D_BUILD_SMOKE=11` — a cena do **ENVELOPE** (ADR-0129): uma elipse **já deformada** por
//!   uma gaiola de perspectiva. OLHE O MEIO DOS SEGMENTOS — as laterais curvam liso; é aí que
//!   o defeito ingênuo (só os cantos) apareceria, não nos cantos.
//! - `PH2D_BUILD_SMOKE=10` — a cena do **MORPH** (o `t` animável): um quadrado e uma estrela, já
//!   SELECIONADOS. Clique **Morph** no painel — nasce UMA forma no meio do caminho. Arraste
//!   **Morph t**: ela caminha entre as duas, ao vivo. As fontes ficam (mexa numa e a forma
//!   refaz-se). Para animar: com o morph selecionado, **+ Track → Morph** na timeline, ponha o
//!   playhead e aperte **K** — o `t` vira uma curva.
//! - `PH2D_BUILD_SMOKE=9` — estrela → **círculo**, 5 passos — o par que o Enio testou à mão. A
//!   transição tem de ser limpa (as pontas encolhem radialmente, sem torcer). Rotate/Reverse Match
//!   foram removidos; o ajuste é editar as formas-fonte.
//! - `PH2D_BUILD_SMOKE=8` — a cena do **GIRO**: quadrado → **círculo**, 5 passos. É o par do 2º
//!   smoke do Enio (*"o porquê da rotação?"*). As quinas têm de caminhar **reto** para fora — sem
//!   rodar 45° e voltar.
//! - `PH2D_BUILD_SMOKE=7` — a cena do **BLEND**: um quadrado e uma estrela, com 3 passos entre eles.
//!   As quinas do quadrado casam com as PONTAS da estrela (não os vales), e as arestas retas ficam
//!   retas em todo o caminho.
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

pub(crate) fn shape(kind: ShapeKind, a: [f64; 2], b: [f64; 2], v: &[f64], rgb: [u8; 3]) -> VecPath {
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
        let f = FRAME.fetch_add(1, Ordering::Relaxed);
        // As cenas do ENVELOPE (níveis 11 e 12) vivem no módulo irmão `envelope_smoke` — teto de
        // LOC. Elas só usam os frames 3 e 4 e nenhum braço compartilhado, então sair do `match`
        // aqui é a MESMA sequência de antes: um nível fora de 11/12 nunca entrava nesses braços, e
        // 11/12 nunca chegavam aos genéricos (os específicos vinham primeiro).
        if matches!(level, 11 | 12) {
            crate::envelope_smoke::frame(self, f, level);
            return;
        }
        match f {
            // A cena. A geometria entra em MUNDO com o `Transform` na identidade — é como a
            // Shape tool deixa uma forma recém-desenhada; o `settle_origins` do frame a
            // centra no local 0 e põe a pose na entidade (ADR-0111/0112).
            // A cena do BLEND: duas formas distantes, com contagens de âncora diferentes (4 e
            // 10) — é o caso em que a correspondência importa.
            3 if level == 7 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
                let scene = &mut gfx.vec_scene;
                scene.push_path(shape(
                    ShapeKind::Rectangle,
                    [-3.4, -1.0],
                    [-1.4, 1.0],
                    &[],
                    [70, 110, 190],
                ));
                scene.push_path(shape(
                    ShapeKind::Star,
                    [1.4, -1.0],
                    [3.4, 1.0],
                    &[5.0, 0.45, 0.0],
                    [200, 120, 80],
                ));
            }
            // A cena do MORPH: as duas formas do blend, mas o objetivo é UMA forma animável. Elas
            // ficam SELECIONADAS (frame 4, depois de o `sync` lhes ter dado entidade) para o smoke
            // ser um clique só — o Enio não deve ter de montar nada.
            3 if level == 10 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
                let scene = &mut gfx.vec_scene;
                scene.push_path(shape(
                    ShapeKind::Rectangle,
                    [-3.4, -1.0],
                    [-1.4, 1.0],
                    &[],
                    [70, 110, 190],
                ));
                scene.push_path(shape(
                    ShapeKind::Star,
                    [1.4, -1.0],
                    [3.4, 1.0],
                    &[5.0, 0.45, 0.0],
                    [200, 120, 80],
                ));
            }
            4 if level == 10 => {
                let ids: Vec<_> = self
                    .gfx
                    .as_ref()
                    .expect("gfx")
                    .vec_scene
                    .paths()
                    .iter()
                    .map(|p| p.id)
                    .collect();
                self.vec_pen.select_many(&ids);
                eprintln!(
                    "[smoke] morph: 2 formas selecionadas — clique **Morph** no painel, depois \
                     arraste **Morph t**"
                );
            }
            // A cena do GIRO (o 2º smoke do Enio): quadrado → CÍRCULO. Ele teve de desenhar o
            // círculo à MÃO da última vez, porque a cena não o oferecia — e é justamente o par em
            // que o defeito aparecia (as intermediárias rodavam 45° e voltavam).
            3 if level == 8 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
                let scene = &mut gfx.vec_scene;
                scene.push_path(shape(
                    ShapeKind::Rectangle,
                    [-3.4, -1.0],
                    [-1.4, 1.0],
                    &[],
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
            // A cena estrela → CÍRCULO — o par que o Enio testou à mão. Um lado tem quinas (a
            // estrela), o outro é liso (o círculo): a transição tem de encolher radialmente, sem
            // torcer. Rotate/Reverse Match foram removidos; o ajuste é editar as formas.
            3 if level == 9 => {
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
            // BLEND, o par do GIRO: quadrado → círculo, 5 passos (com 3 o giro de 45° era fácil
            // de confundir com "a forma está virando um círculo"). O que se olha aqui é UMA coisa:
            // as quinas caminham RETO para fora, sem rodar e voltar.
            8 if level == 8 => {
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
                self.vec_set_draw_mode(ph2d_tool_vector::DrawMode::Select);
                let Some(gfx) = self.gfx.as_mut() else { return };
                let xf = crate::vec_transform::build(&gfx.sim, &self.vec_entities);
                crate::vec_blend::apply(
                    &mut gfx.vec_scene,
                    &mut self.vec_history,
                    &mut self.vec_pen,
                    &xf,
                    &mut self.vec_blend,
                    5,
                    true,
                );
                self.vec_restack = self
                    .vec_blend
                    .as_ref()
                    .map(crate::vec_blend::BlendSession::stack)
                    .into_iter()
                    .collect();
                self.any_input_this_frame = true;
                eprintln!(
                    "[blend-smoke] quadrado -> CIRCULO, 5 passos (o par que girava 45 graus). \
                     As quinas tem de sair RETO, sem rodar."
                );
            }
            // estrela -> circulo, blendado. A transicao tem de ser limpa (pontas encolhem
            // radialmente, sem torcer). Rotate/Reverse Match foram removidos.
            8 if level == 9 => {
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
                self.vec_set_draw_mode(ph2d_tool_vector::DrawMode::Select);
                let Some(gfx) = self.gfx.as_mut() else { return };
                let xf = crate::vec_transform::build(&gfx.sim, &self.vec_entities);
                crate::vec_blend::apply(
                    &mut gfx.vec_scene,
                    &mut self.vec_history,
                    &mut self.vec_pen,
                    &xf,
                    &mut self.vec_blend,
                    5,
                    true,
                );
                self.vec_restack = self
                    .vec_blend
                    .as_ref()
                    .map(crate::vec_blend::BlendSession::stack)
                    .into_iter()
                    .collect();
                self.any_input_this_frame = true;
                eprintln!(
                    "[blend-smoke] estrela -> CIRCULO, 5 passos. A transicao tem de encolher \
                     radialmente, sem torcer. Rotate/Reverse Match foram removidos."
                );
            }
            // BLEND: seleciona as duas e roda o blend pelo caminho REAL (a mesma função que o
            // botão do painel chama). O artista já abre o app com os passos na tela.
            8 if level == 7 => {
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
                self.vec_set_draw_mode(ph2d_tool_vector::DrawMode::Select);
                let Some(gfx) = self.gfx.as_mut() else { return };
                let xf = crate::vec_transform::build(&gfx.sim, &self.vec_entities);
                crate::vec_blend::apply(
                    &mut gfx.vec_scene,
                    &mut self.vec_history,
                    &mut self.vec_pen,
                    &xf,
                    &mut self.vec_blend,
                    3,
                    true, // os passos sobem: cada um acima do anterior
                );
                self.vec_restack = self
                    .vec_blend
                    .as_ref()
                    .map(crate::vec_blend::BlendSession::stack)
                    .into_iter()
                    .collect();
                self.any_input_this_frame = true;
                eprintln!(
                    "[blend-smoke] quadrado -> estrela, 3 passos. As quinas casam com as PONTAS \
                     (nao os vales); as arestas retas ficam retas."
                );
            }
            // Seleciona as três e entra no Build — o estado em que o Enio começa a testar.
            //
            // ⚠️ Gate `level <= 6`, NÃO catch-all. Este arm é do Shape Builder (a cena default
            // `3 =>`, 3 formas, níveis 1-6). Os níveis de OBJETO vetorial (7-11) têm cena própria e
            // NÃO querem Build: 7/8/9 já têm seu `8 if level == N` acima; 10 (morph) fica no Select
            // que a cena deixou; 11 (envelope) fica no NODE que o frame 4 armou — e é o Build deste
            // arm, quando era catch-all, que engolia esse Node e sumia com a gaiola (a alça só
            // aparece no Node). Adicionar um nível novo sem cena de Build? ele cai em `_ => {}`.
            8 if level <= 6 => {
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
            //
            // O `>= 3` de antes VAZAVA para as cenas 7 e 8 (o Blend): elas passam do 3, então o
            // harness injetava um CLIQUE sintético em (0,35 · 0,15) quatro frames depois do blend
            // rodar — e o passo do meio nasce em cima desse ponto. No modo Select o clique PEGA a
            // forma (ADR-0112), a seleção trocava, e o Enio abria o smoke num estado que a doc
            // não descreve. O harness de undo é dos níveis 3..=6 e de mais ninguém.
            9 if (3..=6).contains(&level) => {
                self.any_input_this_frame = true;
                self.smoke_state("baseline (3 formas)");
            }
            12 if (3..=5).contains(&level) => self.smoke_click(IN_STAR),
            13 if (3..=5).contains(&level) => self.smoke_state("depois do clique"),
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
            // **Nível 6 — o bug do Enio: "undo só faz uma etapa".** Duas ações, depois três
            // Ctrl+Z, com o DOWN e o UP em frames SEPARADOS (é o que o winit entrega).
            12 if level == 6 => self.smoke_click(IN_STAR),
            13 if level == 6 => self.smoke_state("ação 1 (build na estrela)"),
            16 if level == 6 => self.smoke_click(IN_PENT),
            17 if level == 6 => self.smoke_state("ação 2 (build no pentágono)"),
            20 if level == 6 => self.smoke_key_z(false, true),
            21 if level == 6 => self.smoke_key_z(false, false),
            22 if level == 6 => self.smoke_state("Ctrl+Z #1"),
            24 if level == 6 => self.smoke_key_z(false, true),
            25 if level == 6 => self.smoke_key_z(false, false),
            26 if level == 6 => self.smoke_state("Ctrl+Z #2"),
            28 if level == 6 => self.smoke_key_z(false, true),
            29 if level == 6 => self.smoke_key_z(false, false),
            30 if level == 6 => self.smoke_state("Ctrl+Z #3"),
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

    /// Ctrl+Z (ou Ctrl+Shift+Z) pelo roteamento REAL do teclado — **inclusive o RELEASE**.
    ///
    /// O release não é enfeite: é um evento de input como qualquer outro, e o
    /// `post_frame_undo` faz a varredura de diff **em todo frame com input**. Enquanto o
    /// harness só mandava o `Pressed`, o frame do release nunca existia — e era exatamente
    /// nele que o passo espúrio nascia.
    fn smoke_key_z(&mut self, redo: bool, down: bool) {
        use winit::keyboard::ModifiersState;
        self.modifiers = if redo {
            ModifiersState::CONTROL | ModifiersState::SHIFT
        } else {
            ModifiersState::CONTROL
        };
        self.key_input(
            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyZ),
            if down {
                winit::event::ElementState::Pressed
            } else {
                winit::event::ElementState::Released
            },
            false,
            None,
        );
        if !down {
            self.modifiers = ModifiersState::empty();
        }
    }

    fn smoke_undo(&mut self, redo: bool) {
        self.smoke_key_z(redo, true);
        self.smoke_key_z(redo, false);
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
        // A ORDEM da cena e o `RootOrder` de cada forma — é o que o restore embaralha.
        if let Some(gfx) = self.gfx.as_ref() {
            let ids: Vec<u64> = gfx.vec_scene.paths().iter().map(|p| p.id).collect();
            let ro: Vec<(u64, String)> = ids
                .iter()
                .map(|id| {
                    let r = self
                        .vec_entities
                        .get(id)
                        .and_then(|&b| {
                            gfx.sim
                                .world()
                                .get::<ph2d_ecs::RootOrder>(ph2d_ecs::Entity::from_bits(b))
                        })
                        .map(|r| {
                            if r.0 == u32::MAX {
                                "MAX".to_string()
                            } else {
                                r.0.to_string()
                            }
                        })
                        .unwrap_or_else(|| "—".to_string());
                    (*id, r)
                })
                .collect();
            eprintln!("[build-smoke]    ordem da cena + RootOrder: {ro:?}");
        }
    }
}
