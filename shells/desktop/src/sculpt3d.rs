//! A costura do módulo 3D com o shell — **a cena, o gesto e o passe**.
//!
//! ⚠️ **A navegação orbital E o gesto de escultura moram AQUI, nunca numa
//! `Tool`.** Girar o modelo não é esculpir: o artista gira com o pincel na mão,
//! e uma `Tool` que capturasse o ponteiro para navegar teria de devolvê-lo a
//! cada gesto. É também o que mantém o contrato congelado intacto (ADR-0150) —
//! nenhum método novo em `Tool`.
//!
//! ⚠️ **Tudo isto é inerte sem a cena armada.** `AppGfx.sculpt3d` nasce `None` e
//! só o smoke a cria, então num run normal cada porta daqui devolve `false` no
//! primeiro `if` e o frame 2D é byte-idêntico.

use std::sync::atomic::{AtomicBool, Ordering};

use ph2d_light::LightRig;
use ph2d_mesh::{Mesh, Ray};
use ph2d_mesh_render::{Camera3d, MeshRenderer};
use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

use crate::app_state::App;

/// **A DOAÇÃO** — o carimbo, a rasterização e o interruptor de três posições.
/// Filho (`#[path]`) para alcançar os campos privados da cena; o corte é *o que
/// o escultor FAZ* (aqui) contra *o que a forma DOA* (lá).
#[path = "sculpt3d_donation.rs"]
pub(crate) mod donation;

use donation::FormRole;
use donation::FormStamp;

/// Quantos radianos um pixel de arrasto vale.
///
/// Decisão de **smoke**, como a tolerância do RDP do Flip: 0,01 dá meia volta a
/// cada ~314 px, que é uma varredura confortável de trackpad. Não é um teto de
/// recurso, então não tem tabela de medição ao lado — tem o olho do Enio.
const ORBIT_RAD_PER_PX: f32 = 0.01;

/// O raio do pincel, em fração do maior lado do modelo.
///
/// ⚠️ **Fração, não valor absoluto** — uma cabeça e um planeta chegam pela mesma
/// porta, e um raio em metros seria invisível num e engoliria o outro. É o mesmo
/// argumento do `REACH_FRACTION` da crate de escultura, um nível acima.
const DEFAULT_RADIUS_FRAC: f32 = 0.12;

/// Passo das teclas de LUZ (`Q`/`E` giram, `R`/`F` sobem e descem), em graus
/// inteiros — que é a unidade em que o rig é autorado. Quinze graus porque o
/// gesto é *"ver a forma reacender"*, não afinar: um passo de 1° pediria vinte
/// toques para a mudança ficar óbvia.
const LIGHT_STEP_DEG: u16 = 15;

/// Passo do `[` / `]`. Multiplicativo pelo motivo do `dolly`: o gesto tem o
/// mesmo efeito *aparente* com pincel grande e pequeno.
const RADIUS_STEP: f32 = 1.15;

/// Os limites do slider de raio, em fração do modelo. O piso existe porque um
/// pincel menor que uma aresta não pega vértice nenhum e o artista conclui que a
/// ferramenta quebrou; o teto porque acima de meio modelo o "pincel" é um
/// deformador global, que é outra ferramenta (W6).
const RADIUS_RANGE: (f32, f32) = (0.005, 0.5);

/// A cena está armada? (`PH2D_SCULPT3D_SMOKE` em `1` ou `2`.)
pub(crate) fn smoke_armed() -> bool {
    matches!(
        std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref(),
        Some("1" | "2")
    )
}

/// `=2` — a cena da **DOAÇÃO**: a esfera E uma tela branca para pintar.
///
/// ⚠️ Cena própria, e não um passo a mais na `=1`: julgar a escultura e julgar a
/// doação são duas perguntas, e a segunda precisa de uma tela que a primeira não
/// quer ver. Misturá-las faria o smoke do barro abrir com um retângulo branco
/// atrás dele sem nada explicando por quê.
pub(crate) fn donation_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("2")
}

/// O que o arrasto está fazendo.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Drag {
    Orbit,
    Pan,
    Sculpt,
}

/// O estado anterior de um traço — **a entrada de undo**.
///
/// Não há um segundo sistema a construir: a lei do traço já congela o `pre` por
/// vértice tocado, e `touched` + `base_positions` É a janela. Isto aqui só
/// guarda a cópia depois que o traço fecha.
struct StrokeUndo {
    verts: Vec<u32>,
    positions: Vec<[f32; 3]>,
    masks: Option<Vec<f32>>,
}

/// A cena 3D viva: a malha, a câmera, o pincel e o pipeline que a desenha.
pub(crate) struct Sculpt3dScene {
    pub(crate) mesh: Mesh,
    pub(crate) camera: Camera3d,
    renderer: MeshRenderer,
    /// A malha já subiu inteira ao device? Depois disso só sobem REGIÕES.
    uploaded: bool,
    drag: Option<Drag>,
    last: (f32, f32),
    viewport: (u32, u32),

    brush: Brush,
    /// O raio autorado, em fração do modelo — ver [`DEFAULT_RADIUS_FRAC`].
    radius_frac: f32,
    /// O tamanho do modelo, capturado UMA vez.
    ///
    /// ⚠️ Lê-lo vivo faria o pincel *respirar* enquanto a escultura cresce — a
    /// mesma cautela do `stroke_radius_px` do Painter, que divide pelo raio do
    /// TRAÇO e não pelo raio vivo justamente para o padrão não re-fasar.
    model_span: f32,
    symmetry: Symmetry,
    /// **O rig de luz do artista** — as mesmas quatro lâmpadas que acendem a tinta
    /// do Painter (`ph2d-light`).
    ///
    /// ⚠️ A cena guarda uma INSTÂNCIA porque hoje ela é um viewport solto, e o
    /// viewport é o documento dela. Quando a escultura virar uma camada de um
    /// documento do Painter (W3.M4) o rig passa a ser o DELE — a estrutura já tem
    /// um dono só, e o que falta unificar é o dado. Um segundo rig permanente
    /// aqui seria exatamente o que `docs/3D/05.2` proíbe.
    rig: LightRig,
    stroke: SculptStroke,
    undo: Vec<StrokeUndo>,
    /// Os vértices que a GPU ainda não viu — acumulados entre frames, porque
    /// vários eventos de ponteiro cabem num quadro.
    dirty: Vec<u32>,
    /// Quantas vezes a MALHA mudou. Entra no carimbo da doação — ver
    /// `Sculpt3dScene::mesh_changed`, a porta única que o move.
    edits: u64,
    /// **O interruptor da doação** — ver [`FormRole`]. Nasce em `Clay` porque a
    /// primeira coisa que se faz com uma escultura é esculpi-la; a doação é o
    /// passo seguinte, e o `D` o dá.
    role: FormRole,
    /// O carimbo da última doação entregue — `None` enquanto nada foi doado.
    donated: Option<FormStamp>,
}

impl Sculpt3dScene {
    pub(crate) fn new(device: &wgpu::Device, mesh: Mesh, aspect: f32) -> Self {
        let mut camera = Camera3d {
            yaw: 0.6,
            pitch: 0.35,
            ..Camera3d::default()
        };
        camera.frame(mesh.bounds(), aspect);
        let model_span = mesh.bounds().longest_edge().max(1e-6);
        Self {
            mesh,
            camera,
            renderer: MeshRenderer::new(device, ph2d_render::GameRt::FORMAT),
            uploaded: false,
            drag: None,
            last: (0.0, 0.0),
            viewport: (1, 1),
            brush: Brush::default(),
            radius_frac: DEFAULT_RADIUS_FRAC,
            model_span,
            // ⚠️ **DESLIGADA por default, e é decisão do smoke.** O ZBrush
            // nasce com espelho ligado — e MOSTRA isso. Aqui o artista clicava
            // de um lado e via uma segunda protuberância do outro, sem nada na
            // tela explicando por quê: *"o local onde está esculpindo não
            // coincide com a posição do mouse"*. Um default que só se descobre
            // por acidente é pior que um default menos ambicioso; o `X` liga.
            symmetry: Symmetry::default(),
            rig: LightRig::default(),
            stroke: SculptStroke::default(),
            undo: Vec::new(),
            dirty: Vec::new(),
            edits: 0,
            role: FormRole::Clay,
            donated: None,
        }
    }

    /// O pincel com o raio resolvido em unidades de MUNDO.
    ///
    /// Porta única: a fração é o estado autorado e o raio absoluto é derivado —
    /// guardar os dois seria o mesmo número em dois lugares, e eles divergem no
    /// dia em que alguém escrever num só.
    fn armed_brush(&self) -> Brush {
        Brush {
            radius: (self.radius_frac * self.model_span).max(1e-6),
            ..self.brush
        }
    }

    /// Desenha a malha sobre o que já está no alvo. O upload acontece na
    /// primeira passagem — é aqui que o device é conhecido.
    pub(crate) fn render(
        &mut self,
        gpu: &ph2d_gpu::GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        color: &wgpu::TextureView,
        size: (u32, u32),
    ) {
        self.viewport = size;
        // ⚠️ O viewport é atualizado ANTES da recusa: ele é o que converte um
        // clique em raio, e um viewport parado faria o pincel cair no lugar
        // errado no instante em que o artista voltasse ao barro.
        if !self.shows_clay() {
            return;
        }
        self.sync_mesh(&gpu.device, &gpu.queue);
        // O rig é RESOLVIDO por frame, não guardado resolvido: a resolução é
        // barata (quatro lâmpadas) e uma cópia resolvida seria uma segunda
        // verdade sobre onde a luz está — a que fica velha no frame seguinte ao
        // artista mexer no card.
        let resolved = ph2d_light::resolve(&self.rig);
        self.renderer.render(
            &gpu.device,
            &gpu.queue,
            encoder,
            color,
            &self.camera,
            resolved.as_ref(),
            size,
        );
    }

    /// O raio do cursor, pela câmera desta cena.
    fn ray_at(&self, x: f32, y: f32) -> Ray {
        self.camera.ray_through(x, y, self.viewport)
    }

    /// Aplica um dab onde o cursor aponta. Devolve `false` se o raio errou a
    /// malha — e errar é normal: a mão sai do modelo o tempo todo.
    fn sculpt_at(&mut self, x: f32, y: f32) -> bool {
        let ray = self.ray_at(x, y);
        let Some(hit) = self.mesh.raycast(&ray) else {
            return false;
        };
        if std::env::var("PH2D_SCULPT3D_DIAG").ok().as_deref() == Some("1") {
            // ⚠️ **O instrumento que responde *"o pincel cai onde o cursor
            // aponta?"* com um NÚMERO.** Ele reprojeta o acerto pela porta
            // `project` — o inverso exato do `ray_through` — e imprime o erro em
            // pixels. Um desvio grande acusa a fiação (viewport, escala, um
            // flip); zero acusa a percepção, e aí a causa é outra.
            let back = self.camera.project(hit.point, self.viewport);
            let err = back.map(|(bx, by)| ((bx - x).hypot(by - y), bx, by));
            eprintln!(
                "[sculpt3d] clique ({x:.1}, {y:.1}) viewport {:?} -> acerto {:?} \
                 -> volta {err:?}",
                self.viewport, hit.point
            );
        }
        let brush = self.armed_brush();
        self.stroke.dab(
            &mut self.mesh,
            &brush,
            &Dab::at(hit.point, brush.radius),
            self.symmetry,
        );
        Self::mesh_changed(
            &mut self.dirty,
            &mut self.edits,
            self.stroke.last_refreshed(),
        );
        true
    }

    /// Fecha o traço e guarda o desfazer.
    fn close_stroke(&mut self) {
        if self.stroke.touched().is_empty() {
            return;
        }
        self.undo.push(StrokeUndo {
            verts: self.stroke.touched().to_vec(),
            positions: self.stroke.base_positions().to_vec(),
            masks: self
                .brush
                .verb
                .paints_mask()
                .then(|| self.stroke.base_masks().to_vec()),
        });
    }

    /// Desfaz o último traço. Devolve `false` se não havia nada.
    fn undo_stroke(&mut self) -> bool {
        let Some(entry) = self.undo.pop() else {
            return false;
        };
        if let Some(masks) = &entry.masks {
            let out = self.mesh.masks_mut();
            for (&v, m) in entry.verts.iter().zip(masks) {
                out[v as usize] = *m;
            }
        } else {
            let out = self.mesh.positions_mut();
            for (&v, p) in entry.verts.iter().zip(&entry.positions) {
                out[v as usize] = *p;
            }
            // ⚠️ O `rebuild` inteiro, e não um `refresh_region`: desfazer devolve
            // posições que o refit incremental já tinha "seguido" para outro
            // lugar, e um refit sobre a volta deixaria caixas frouxas grandes
            // demais acumulando a cada Ctrl+Z. Um undo é user-paced — é o lugar
            // certo para pagar a resposta exata.
            self.mesh.rebuild();
            self.mesh_rebuilt();
        }
        true
    }
}

impl App {
    /// `PH2D_SCULPT3D_SMOKE=1` — a cena pronta: uma esfera de barro para
    /// esculpir. `=2` acrescenta a TELA e é a cena da **doação**
    /// (`crate::sculpt3d::donation`). Roda uma vez, no primeiro frame com GPU.
    pub(crate) fn sculpt3d_smoke(&mut self) {
        // Guard estático, o mesmo idioma dos outros smokes do shell — evita um
        // campo em `App` que só existe para dizer "já rodei".
        static ARMED: AtomicBool = AtomicBool::new(false);
        if !crate::sculpt3d::smoke_armed()
            || self.gfx.is_none()
            || ARMED.swap(true, Ordering::Relaxed)
        {
            return;
        }
        let mesh = ph2d_mesh::shapes::uv_sphere(96, 144, 1.0);
        // A cena IMPRIME o que montou. Um smoke que não se declara deixa o
        // artista sem saber se está vendo a feature ou o app vazio — a lição
        // que o smoke do Colorize pagou.
        eprintln!(
            "[sculpt3d] esfera com {} vértices / {} faces / {} triângulos\n\
             [sculpt3d] ESQUERDO esculpe (fora do modelo, gira) · DIREITO gira · MEIO desloca · RODA aproxima\n\
             [sculpt3d] Shift = Smooth enquanto segurar · Ctrl inverte Draw/Inflate/Clay/Crease e limpa a mascara\n\
             [sculpt3d] 1..9,0 escolhem o verbo · M mascara · [ ] tamanho · X/Y/Z espelho · Ctrl+Z desfaz\n\
             [sculpt3d] A LUZ e o rig do artista (o mesmo que acende a tinta): Q/E giram a lampada, R/F a sobem\n\
             [sculpt3d] o espelho nasce DESLIGADO; PH2D_SCULPT3D_DIAG=1 mede se o pincel cai sob o cursor",
            mesh.vert_count(),
            mesh.face_count(),
            mesh.triangle_count()
        );
        if crate::sculpt3d::donation_scene() {
            eprintln!(
                "[sculpt3d] =2 A DOACAO: ha uma TELA BRANCA embaixo, e a tecla D alterna\n\
                 [sculpt3d]    BARRO (esculpir) -> LUZ (a forma acende a tinta) -> DESLIGADA (o A/B)\n\
                 [sculpt3d]    esculpa, aperte D, pegue o Painter e pinte CHAPADO: a tinta tem de sair ACESA\n\
                 [sculpt3d]    aperte D de novo e a MESMA tinta fica plana -- e essa diferenca e a wave"
            );
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let size = gfx.surface.size();
        let aspect = size.width as f32 / size.height.max(1) as f32;
        let device = std::sync::Arc::clone(&gfx.surface.gpu().device);
        gfx.sculpt3d = Some(Sculpt3dScene::new(&device, mesh, aspect));
    }

    /// O botão apertou. Devolve `true` se a cena 3D tomou o gesto.
    pub(crate) fn sculpt3d_pointer_down(&mut self, button: winit::event::MouseButton) -> bool {
        let pos = self.last_pointer;
        // ⚠️ Um clique SOBRE PAINEL não é da cena — e sem esta pergunta a cena 3D
        // engolia todo botão do app, inclusive os do rail. O `Move` e o `Up` NÃO
        // a fazem de propósito: um arrasto em curso continua sendo do gesto que
        // o abriu, mesmo que o cursor passeie por cima de um painel (a regra de
        // captura que todo gizmo deste shell segue). É a MESMA porta que a roda
        // já usava.
        if crate::forwarding::cursor_over_hero_panel(self.gfx.as_ref(), pos.0, pos.1) {
            return false;
        }
        let mods = self.modifiers;
        let (ctrl, shift) = (mods.control_key(), mods.shift_key());
        let Some(scene) = self.sculpt3d_scene_mut() else {
            return false;
        };
        // ⚠️ **Com o barro fora da tela, o ponteiro NÃO é da cena.** Sem esta
        // pergunta a doação seria inalcançável pelo motivo mais bobo possível:
        // o artista troca para o modo LUZ, vai pintar, e cada clique orbita um
        // modelo invisível. É a mesma classe do clique-sobre-painel que o smoke
        // da W2 pegou — quem não está na tela não recebe o gesto.
        if !scene.shows_clay() {
            return false;
        }
        match button {
            winit::event::MouseButton::Left => {
                // ⚠️ Os modificadores são lidos UMA vez, no pen-down, e valem o
                // traço inteiro. Soltar o Shift no meio de uma pincelada faria
                // metade dela ser outra ferramenta — e nenhum app de escultura
                // faz isso, porque a lei do traço congela um `pre` só.
                scene.brush.invert = ctrl;
                let verb = scene.brush.verb;
                if shift {
                    scene.brush.verb = Verb::Smooth;
                }
                scene.stroke.begin(&scene.mesh);
                if scene.sculpt_at(pos.0, pos.1) {
                    scene.drag = Some(Drag::Sculpt);
                } else {
                    // Errou o modelo: o botão vira ÓRBITA. É o que o SculptGL
                    // faz, e é o que impede o gesto mais comum do mundo —
                    // arrastar no vazio — de não fazer nada.
                    scene.brush.verb = verb;
                    scene.drag = Some(Drag::Orbit);
                }
            }
            winit::event::MouseButton::Right => scene.drag = Some(Drag::Orbit),
            winit::event::MouseButton::Middle => scene.drag = Some(Drag::Pan),
            _ => return false,
        }
        scene.last = pos;
        true
    }

    /// O botão soltou.
    pub(crate) fn sculpt3d_pointer_up(&mut self) -> bool {
        let Some(scene) = self.sculpt3d_scene_mut() else {
            return false;
        };
        let was = scene.drag.take();
        if was == Some(Drag::Sculpt) {
            scene.close_stroke();
        }
        was.is_some()
    }

    /// O ponteiro moveu. Só consome com um arrasto EM CURSO — senão a cena 3D
    /// engoliria todo hover do app.
    pub(crate) fn sculpt3d_pointer_move(&mut self, x: f32, y: f32) -> bool {
        let Some(scene) = self.sculpt3d_scene_mut() else {
            return false;
        };
        let Some(drag) = scene.drag else {
            return false;
        };
        let (dx, dy) = (x - scene.last.0, y - scene.last.1);
        scene.last = (x, y);
        let height = scene.viewport.1.max(1) as f32;
        match drag {
            // ⚠️ **Manipulação direta: o modelo segue a mão.** `yaw` positivo
            // leva o OLHO para `+X`, e a câmera indo para a direita faz o
            // modelo *parecer* ir para a esquerda — então arrastar para a
            // direita pede `yaw -= dx`. E arrastar para BAIXO mostra o TOPO
            // (o modelo tomba para a frente), que é `pitch += dy`.
            //
            // Os DOIS sinais estavam trocados e o smoke os pegou; o gate que os
            // prende (`dragging_right_turns_the_model_right`) mede o modelo NA
            // TELA em vez de argumentar sobre sinais, que foi como o erro entrou.
            Drag::Orbit => scene
                .camera
                .orbit(-dx * ORBIT_RAD_PER_PX, dy * ORBIT_RAD_PER_PX),
            Drag::Pan => scene.camera.pan(dx / height, dy / height),
            Drag::Sculpt => {
                scene.sculpt_at(x, y);
            }
        }
        true
    }

    /// A roda aproxima.
    pub(crate) fn sculpt3d_wheel(&mut self, steps: f32) -> bool {
        let Some(scene) = self.sculpt3d_scene_mut() else {
            return false;
        };
        // Mesma lei do `pointer_down`: barro fora da tela, roda do 2D. Sem isto
        // o zoom do canvas ficaria preso enquanto a forma acende a tinta.
        if !scene.shows_clay() {
            return false;
        }
        scene.camera.dolly(steps);
        true
    }

    /// As teclas da cena 3D. Devolve `true` se consumiu.
    pub(crate) fn sculpt3d_key(&mut self, code: winit::keyboard::KeyCode, ctrl: bool) -> bool {
        use winit::keyboard::KeyCode as K;
        let Some(scene) = self.sculpt3d_scene_mut() else {
            return false;
        };
        if ctrl {
            return code == K::KeyZ && scene.undo_stroke();
        }
        // Os dez primeiros verbos por número; o Mask fica no `M` porque ele não
        // é uma escultura a mais, é o canal que todos os outros respeitam.
        const BY_NUMBER: [Verb; 10] = [
            Verb::Draw,
            Verb::Inflate,
            Verb::Smooth,
            Verb::Sharpen,
            Verb::Flatten,
            Verb::Fill,
            Verb::Scrape,
            Verb::Clay,
            Verb::Pinch,
            Verb::Crease,
        ];
        let verb = match code {
            K::Digit1 => Some(BY_NUMBER[0]),
            K::Digit2 => Some(BY_NUMBER[1]),
            K::Digit3 => Some(BY_NUMBER[2]),
            K::Digit4 => Some(BY_NUMBER[3]),
            K::Digit5 => Some(BY_NUMBER[4]),
            K::Digit6 => Some(BY_NUMBER[5]),
            K::Digit7 => Some(BY_NUMBER[6]),
            K::Digit8 => Some(BY_NUMBER[7]),
            K::Digit9 => Some(BY_NUMBER[8]),
            K::Digit0 => Some(BY_NUMBER[9]),
            K::KeyM => Some(Verb::Mask),
            _ => None,
        };
        if let Some(v) = verb {
            scene.brush.verb = v;
            eprintln!("[sculpt3d] verbo: {}", v.label());
            return true;
        }
        match code {
            K::BracketLeft | K::BracketRight => {
                let f = if code == K::BracketRight {
                    RADIUS_STEP
                } else {
                    1.0 / RADIUS_STEP
                };
                scene.radius_frac = (scene.radius_frac * f).clamp(RADIUS_RANGE.0, RADIUS_RANGE.1);
                eprintln!(
                    "[sculpt3d] raio: {:.1}% do modelo",
                    scene.radius_frac * 100.0
                );
                true
            }
            // **O INTERRUPTOR DA DOAÇÃO** — barro ⇄ luz ⇄ desligada.
            //
            // ⚠️ Gesto de SMOKE, como o `Q`/`E`/`R`/`F` da luz: a UI final é o
            // toggle *"iluminada pela forma abaixo"* na pilha de camadas
            // (`docs/3D/05.2`), e ele espera a escultura ser uma CAMADA do
            // documento. Enquanto ela é um viewport solto, uma tecla é a única
            // porta honesta — um checkbox num painel diria que a forma pertence
            // a um documento a que ela ainda não pertence.
            K::KeyD => {
                let label = scene.cycle_role();
                eprintln!("[sculpt3d] a forma agora e: {label}");
                true
            }
            K::KeyX | K::KeyY | K::KeyZ => {
                let axis = match code {
                    K::KeyX => &mut scene.symmetry.x,
                    K::KeyY => &mut scene.symmetry.y,
                    _ => &mut scene.symmetry.z,
                };
                *axis = !*axis;
                eprintln!("[sculpt3d] espelho: {:?}", scene.symmetry);
                true
            }
            // **A LUZ.** Girar a lâmpada principal em torno da cena e subi-la.
            //
            // ⚠️ Isto é o gesto do SMOKE, não a UI final: o card de Lighting do
            // Painter já é o lugar onde este rig se autora, e é ele que a M4
            // conecta. Um segundo card aqui seria a segunda porta para o mesmo
            // número. Estas teclas existem para o Enio poder ver a forma reacender
            // sem abrir um documento de pintura.
            K::KeyQ | K::KeyE => {
                let d = if code == K::KeyE {
                    LIGHT_STEP_DEG
                } else {
                    360 - LIGHT_STEP_DEG
                };
                let l = scene.rig.current_mut();
                l.angle_deg = (l.angle_deg + d) % 360;
                eprintln!(
                    "[sculpt3d] luz: azimute {}deg elevacao {}deg",
                    l.angle_deg, l.elev_deg
                );
                true
            }
            K::KeyR | K::KeyF => {
                let l = scene.rig.current_mut();
                let up = code == K::KeyR;
                // Clampado no piso do resolvedor, e não em 0: abaixo dele a
                // resposta plana vai a zero e o modelo relativo dividiria por ~0.
                l.elev_deg = if up {
                    (l.elev_deg + LIGHT_STEP_DEG).min(90)
                } else {
                    l.elev_deg
                        .saturating_sub(LIGHT_STEP_DEG)
                        .max(ph2d_light::MIN_ELEV_DEG)
                };
                eprintln!(
                    "[sculpt3d] luz: azimute {}deg elevacao {}deg",
                    l.angle_deg, l.elev_deg
                );
                true
            }
            _ => false,
        }
    }

    fn sculpt3d_scene_mut(&mut self) -> Option<&mut Sculpt3dScene> {
        self.gfx.as_mut()?.sculpt3d.as_mut()
    }
}
