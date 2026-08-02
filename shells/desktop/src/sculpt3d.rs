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

use ph2d_light::LightRig;
use ph2d_mesh::{Mesh, Ray};
use ph2d_mesh_render::{Camera3d, MeshRenderer};
use ph2d_sculpt3d::{Brush, Dab, Grip, SculptStroke, Symmetry, Verb};

/// **A DOAÇÃO** — o carimbo, a rasterização e o interruptor de três posições.
/// Filho (`#[path]`) para alcançar os campos privados da cena; o corte é *o que
/// o escultor FAZ* (aqui) contra *o que a forma DOA* (lá).
#[path = "sculpt3d_donation.rs"]
pub(crate) mod donation;

/// **O GESTO** — as portas de ponteiro, roda e teclado. Filho (`#[path]`) para
/// alcançar os campos privados da cena; o corte é *o que a cena É* (aqui) contra
/// *o que a mão FAZ* (lá), o mesmo que separa a [`donation`].
#[path = "sculpt3d_input.rs"]
mod input;

use donation::FormRole;
use donation::FormStamp;

/// Quantos radianos um pixel de arrasto vale.
///
/// Decisão de **smoke**, como a tolerância do RDP do Flip: 0,01 dá meia volta a
/// cada ~314 px, que é uma varredura confortável de trackpad. Não é um teto de
/// recurso, então não tem tabela de medição ao lado — tem o olho do Enio.
const ORBIT_RAD_PER_PX: f32 = 0.01;

/// O raio do pincel, em **pixels de tela**.
///
/// ⚠️ **Pixels, não fração do modelo** — o raio de MUNDO é derivado por dab
/// (`Camera3d::world_radius_for_screen_px`), então o pincel mantém o tamanho
/// aparente quando a câmera aproxima. É o `computeWorldRadius2` do SculptGL, e
/// é o que Blender e ZBrush entregam: aproximar É como se alcança detalhe fino,
/// e um raio ancorado no modelo tornava isso impossível (o pincel crescia junto
/// com a imagem).
///
/// **50 px é MEDIDO, não escolhido:** é o que reproduz o tamanho aparente do
/// default anterior (0,12 do span) na cena do smoke a 720p — ver
/// `ph2d-mesh-render/tests/measure_screen_radius.rs`.
const DEFAULT_RADIUS_PX: f32 = 50.0;

/// Passo das teclas de LUZ (`Q`/`E` giram, `R`/`F` sobem e descem), em graus
/// inteiros — que é a unidade em que o rig é autorado. Quinze graus porque o
/// gesto é *"ver a forma reacender"*, não afinar: um passo de 1° pediria vinte
/// toques para a mudança ficar óbvia.
const LIGHT_STEP_DEG: u16 = 15;

/// Passo do `[` / `]`. Multiplicativo pelo motivo do `dolly`: o gesto tem o
/// mesmo efeito *aparente* com pincel grande e pequeno.
const RADIUS_STEP: f32 = 1.15;

/// O piso do raio, em pixels. **Quem aperta é a TELA, não a malha** — e isso é
/// medição, não herança: a régua antiga dizia *"menor que uma aresta não pega
/// vértice"*, e na cena do smoke um disco de **0,5 px já pega um vértice**
/// (`measure_screen_radius.rs`), porque a malha é densa. O que de fato quebra
/// abaixo de um pixel é o artista **ver onde está mirando**.
/// Quantos passos um clique de blur/sharpen dá.
///
/// ⚠️ **Número de SMOKE, não teto de recurso.** Um passo é pequeno de propósito
/// (`BLUR_MIX = 0,5`, para o gesto não apagar a própria borda de uma vez), então
/// o clique precisa de vários para o artista ver a diferença — e clicar de novo
/// borra mais, que é o que o gesto significa.
const MASK_OP_PASSES: u32 = 6;

const RADIUS_MIN_PX: f32 = 1.0;

/// O teto do raio, em fração da ALTURA do viewport.
///
/// ⚠️ **Fração da tela, e não um número fixo de pixels, porque um teto fixo muda
/// de SIGNIFICADO com a resolução:** medido, 160 px cobre **91% da altura do
/// modelo a 1280×720 e 45% a 2560×1440**. `0,125` é a mesma promessa do teto
/// antigo (*acima de meio modelo o "pincel" é um deformador global, que é outra
/// ferramenta*) escrita no recurso que de fato aperta — com o enquadramento
/// padrão o modelo ocupa 49% da altura, então 1/8 de tela é meio modelo.
const RADIUS_MAX_FRAC_OF_HEIGHT: f32 = 0.125;

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

/// As quatro operações de máscara — ver [`Sculpt3dScene::mask_op`].
#[derive(Clone, Copy, PartialEq, Eq)]
enum MaskOp {
    Clear,
    Invert,
    Blur,
    Sharpen,
}

impl MaskOp {
    fn label(self) -> &'static str {
        match self {
            Self::Clear => "limpa",
            Self::Invert => "inverte",
            Self::Blur => "borra",
            Self::Sharpen => "afia",
        }
    }
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
    /// A entrada é a máscara INTEIRA (uma operação de máscara), e não a janela
    /// de um traço. ⚠️ Sem esta marca, `masks: None` seria ambíguo: *"foi um
    /// traço de geometria"* e *"a máscara estava limpa e tem de voltar a
    /// estar"* são a mesma ausência, e desfazer um `Invert` sobre malha virgem
    /// deixaria tudo protegido para sempre.
    whole_mask: bool,
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
    /// O raio autorado, em **pixels de tela** — ver [`DEFAULT_RADIUS_PX`]. O raio
    /// de MUNDO é derivado por dab, contra a câmera e o ponto de acerto.
    radius_px: f32,
    /// Onde o traço carimbou pela última vez, em pixels — **a âncora do
    /// espaçamento**, e ela é separada do `last` de propósito: o `last` é o
    /// delta de TODO arrasto (a órbita precisa dele por evento) e esta só anda
    /// quando um dab de fato saiu. Colapsá-las apagaria o carry.
    stroke_anchor: [f32; 2],
    /// Onde a mão **pegou** — o ponto de mundo do pen-down e o pixel dele. É a
    /// âncora dos DOIS grips que puxam, e é dela que os dois derivam o mundo:
    /// o [`Grip::Hold`] mede o puxão total até aqui, o [`Grip::Hook`] mede o
    /// incremento entre dois passos do caminho.
    grab: Option<([f32; 3], (f32, f32))>,
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
        Self {
            mesh,
            camera,
            renderer: MeshRenderer::new(device, ph2d_render::GameRt::FORMAT),
            uploaded: false,
            drag: None,
            last: (0.0, 0.0),
            viewport: (1, 1),
            brush: Brush::default(),
            radius_px: DEFAULT_RADIUS_PX,
            stroke_anchor: [0.0, 0.0],
            grab: None,
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

    /// O raio autorado, **já clampado contra a tela desta janela**.
    ///
    /// Porta única, e é ela que faz um `resize` re-clampar sozinho: o cru é o
    /// estado autorado e o limite é do viewport, então guardar o clampado seria
    /// o mesmo número em dois lugares — e o segundo fica velho no primeiro
    /// arrasto de janela.
    fn radius_px(&self) -> f32 {
        let ceiling =
            (RADIUS_MAX_FRAC_OF_HEIGHT * self.viewport.1.max(1) as f32).max(RADIUS_MIN_PX);
        self.radius_px.clamp(RADIUS_MIN_PX, ceiling)
    }

    /// O pincel com o raio resolvido em unidades de MUNDO, no ponto `at`.
    ///
    /// ⚠️ **O raio é função do ACERTO, não do pincel** — o mesmo pincel cobre
    /// menos mundo perto da câmera e mais longe dela, que é o que "tamanho em
    /// pixels" significa. Guardar um raio de mundo no `Brush` seria o mesmo
    /// número em dois lugares, e o segundo ficaria velho a cada `dolly`.
    fn armed_brush(&self, at: [f32; 3]) -> Brush {
        Brush {
            radius: self
                .camera
                .world_radius_for_screen_px(at, self.radius_px(), self.viewport)
                .max(1e-6),
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

    /// **PEGA o barro** — o pen-down dos dois grips que puxam. Devolve `false`
    /// se o raio errou a malha (e aí o botão vira órbita, como em todo gesto).
    ///
    /// ⚠️ **Nem o Grab nem o Snake Hook re-picam depois disto**, e o Hook é o
    /// caso surpreendente: ele arrasta uma ESFERA pelo espaço, não um acerto de
    /// superfície (`Drag.js:59` consulta `pickVerticesInSphere` num centro
    /// deslocado, sem raycast, e o `makeStroke` dele devolve `true` sempre).
    /// Sair do modelo no meio do gesto não interrompe um espinho — é assim que
    /// se puxa uma ponta para fora da silhueta.
    fn take_hold(&mut self, x: f32, y: f32) -> bool {
        let ray = self.ray_at(x, y);
        let Some(hit) = self.mesh.raycast(&ray) else {
            return false;
        };
        self.grab = Some((hit.point, (x, y)));
        true
    }

    /// Onde o dedo está, em MUNDO, na profundidade da pegada.
    ///
    /// Porta única dos dois grips que puxam, e é ela que os liga: o
    /// [`Grip::Hold`] pede o vetor da âncora até aqui (o puxão TOTAL) e o
    /// [`Grip::Hook`] pede a diferença entre dois destes (o INCREMENTO). Duas
    /// aritméticas para *"onde o dedo está"* divergiriam no dia em que uma delas
    /// ganhasse a perspectiva e a outra não.
    fn finger_world(&self, at: [f32; 3], from: (f32, f32), x: f32, y: f32) -> [f32; 3] {
        let d = self
            .camera
            .screen_delta_to_world(at, x - from.0, y - from.1, self.viewport);
        [at[0] + d[0], at[1] + d[1], at[2] + d[2]]
    }

    /// **O gesto de quem SEGURA** ([`Grip::Hold`]): a pegada fica onde foi
    /// presa e o que cresce é o puxão.
    fn grab_at(&mut self, x: f32, y: f32) {
        let Some((at, from)) = self.grab else {
            return;
        };
        let f = self.finger_world(at, from, x, y);
        let pull = [f[0] - at[0], f[1] - at[1], f[2] - at[2]];
        let brush = self.armed_brush(at);
        let eye = self.ray_at(x, y).dir();
        self.stroke.dab(
            &mut self.mesh,
            &brush,
            &Dab::pulling(at, brush.radius, eye, pull),
            self.symmetry,
        );
        Self::mesh_changed(
            &mut self.dirty,
            &mut self.edits,
            self.stroke.last_gpu_dirty(),
        );
    }

    /// **Um passo de quem ARRASTA** ([`Grip::Hook`]): a pegada anda de `from`
    /// até `to` (pixels) e o dab recebe o INCREMENTO daquele trecho.
    ///
    /// ⚠️ **Os dois centros saem da ÂNCORA, não um do outro.** Somar
    /// incrementos passo a passo acumularia o erro de cada conversão ao longo
    /// de um arrasto inteiro, e a pegada iria escorregando para longe do
    /// cursor. Derivando os dois da âncora, o incremento é a diferença de dois
    /// absolutos e o centro está sempre onde o dedo está.
    fn hook_step(&mut self, from: [f32; 2], to: [f32; 2]) {
        let Some((at, origin)) = self.grab else {
            return;
        };
        let c0 = self.finger_world(at, origin, from[0], from[1]);
        let c1 = self.finger_world(at, origin, to[0], to[1]);
        let step = [c1[0] - c0[0], c1[1] - c0[1], c1[2] - c0[2]];
        let brush = self.armed_brush(c1);
        let eye = self.ray_at(to[0], to[1]).dir();
        self.stroke.dab(
            &mut self.mesh,
            &brush,
            &Dab::hooking(c1, brush.radius, eye, step),
            self.symmetry,
        );
        Self::mesh_changed(
            &mut self.dirty,
            &mut self.edits,
            self.stroke.last_gpu_dirty(),
        );
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
        let brush = self.armed_brush(hit.point);
        self.stroke.dab(
            &mut self.mesh,
            &brush,
            // ⚠️ **O olho é o `dir` do raio que ACABOU de produzir este acerto**,
            // e não uma direção derivada da câmera de novo: duas respostas para
            // *"de onde se está olhando"* divergem no frame em que a câmera se
            // move entre o pick e o dab.
            &Dab::at(hit.point, brush.radius, ray.dir()),
            self.symmetry,
        );
        Self::mesh_changed(
            &mut self.dirty,
            &mut self.edits,
            // ⚠️ **`last_gpu_dirty`, não `last_refreshed`.** Um traço de máscara
            // não move geometria, então ele não refresca normal nenhuma — e
            // perguntar *"o que refresquei?"* devolveria VAZIO, deixando a
            // máscara invisível na GPU com todos os gates de CPU verdes.
            self.stroke.last_gpu_dirty(),
        );
        true
    }

    /// **Uma operação de máscara**, com o undo e o upload que ela implica.
    ///
    /// ⚠️ **A entrada de undo é a MÁSCARA INTEIRA, e não a janela de um traço.**
    /// Estas operações agem na malha toda por definição (o `blur` alcança todo
    /// vértice cuja vizinhança tem máscara), então uma janela seria uma mentira
    /// sobre o que mudou — e o que ela custa é `4 B × vértices`, o mesmo que o
    /// plano que ela desfaz.
    ///
    /// ⚠️ E a GPU tem de re-ler a malha INTEIRA: o `dirty` incremental é a
    /// janela de um dab, e aqui não houve dab.
    fn mask_op(&mut self, op: MaskOp) {
        let before = self.mesh.masks().map(<[f32]>::to_vec);
        match op {
            MaskOp::Clear => {
                if !ph2d_sculpt3d::mask_ops::clear(&mut self.mesh) {
                    return;
                }
            }
            MaskOp::Invert => ph2d_sculpt3d::mask_ops::invert(&mut self.mesh),
            MaskOp::Blur => ph2d_sculpt3d::mask_ops::blur(&mut self.mesh, MASK_OP_PASSES),
            MaskOp::Sharpen => ph2d_sculpt3d::mask_ops::sharpen(&mut self.mesh, MASK_OP_PASSES),
        }
        self.undo.push(StrokeUndo {
            verts: Vec::new(),
            positions: Vec::new(),
            masks: before,
            whole_mask: true,
        });
        self.uploaded = false;
        self.edits += 1;
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
            whole_mask: false,
        });
    }

    /// Desfaz o último traço. Devolve `false` se não havia nada.
    fn undo_stroke(&mut self) -> bool {
        let Some(entry) = self.undo.pop() else {
            return false;
        };
        if entry.whole_mask {
            // Uma operação de máscara mexeu na malha inteira: o estado anterior
            // é o plano INTEIRO, e `None` quer dizer *não havia máscara* — o
            // que se desfaz REMOVENDO o plano, não zerando-o.
            match entry.masks {
                Some(m) => self.mesh.put_masks(m),
                None => {
                    self.mesh.take_masks();
                }
            }
            self.uploaded = false;
            self.edits += 1;
        } else if let Some(masks) = &entry.masks {
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
