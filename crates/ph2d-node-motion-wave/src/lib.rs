#![forbid(unsafe_code)]
//! `motion.wave` — a **wave-equation ripple field**: a `rows×cols` grid whose
//! heights obey the discrete 2D wave equation, so a driven centre radiates
//! concentric ripples outward (Motion Nodes M4, simulation — doc 01 §3 / doc 22).
//! The field-dynamics counterpart to the particle sims: the disturbance PROPAGATES
//! at a finite speed (a real hyperbolic PDE), which no per-element oscillator can
//! do. The height maps to each dot's `size`, so the grid reads as expanding rings.
//!
//! **Algorithm — the classic finite-difference wave equation, leapfrog in time.**
//! `∂²h/∂t² = c²∇²h` discretises to
//! `h_next = 2·h − h_prev + C·∇²h` (leapfrog / Verlet in time), with the 5-point
//! Laplacian `∇²h[i,j] = h[i−1,j]+h[i+1,j]+h[i,j−1]+h[i,j+1] − 4·h[i,j]` and
//! reflecting (Neumann) edges. `C = (c·dt)²` is clamped below the CFL limit `0.5`
//! so it never explodes. A `damping` factor bleeds energy each step. The centre cell
//! is driven to the `drive` value input (a Dirichlet source — wire a `value.lfo`
//! and it emits continuous ripples). Pure arithmetic (HR-5: no `sqrt`, no trig).
//!
//! ## N PRODUTORES — por COMPOSIÇÃO, e está MEDIDO
//!
//! Este nó tem **uma** fonte embutida: a célula do centro. Um segundo produtor, em
//! qualquer posição e com qualquer forma, sai da cadeia que o **Grupo P** abriu —
//! `motion.drive(Custom…)` escreve numa coluna que o artista batiza, e `wave_h` é
//! uma coluna `Scalar` que o `is_bookkeeping_column` **não** protege:
//!
//! ```text
//! wave.out --pre--> field.box --> value.attribute("falloff") -->
//!     motion.drive(Custom "wave_h", Add) --> wave.state
//! ```
//!
//! Medido (`ph2d-node-registry-init/tests/measure_wave_producers.rs`, 21×21, 240
//! tiques): o campo passa de `max |h| = 0,2231` para `0,8056`, com um segundo pico
//! exactamente sobre a caixa, e **419 das 441 células fora da máscara se movem** —
//! o bump **propaga**, que é o que separa um PRODUTOR de tinta pintada no campo de
//! altura. Os cinco knobs de um *Producer* do AE Wave World saem dos nós que já
//! existem: **Position/Width/Height/Angle** são o `field.box`, e **Amplitude** é o
//! `scale` do `drive` (com uma `value.lfo` no valor, também Frequency e Phase).
//!
//! ⚠️ **O `pre` mora na aresta que ENTRA na cadeia**, nunca na que volta ao `state`:
//! é ela que quebra o ciclo, e os três nós são `Effect::Pure` ⇒ não carimbam
//! `sim_t`, então a onda ainda vê o `dt` do próprio relógio no tique seguinte.
//!
//! ⛔ **MEDIDO E REJEITADO, não refaça: encadear `wave A --> wave B.state`.** É a
//! tentativa natural de *"dois produtores"* e ela é um **no-op SILENCIOSO** — B lê
//! o `sim_t` que A acabou de carimbar, `dt` dá zero, o ramo de *hold* devolve o
//! campo intacto e o `drive` de B **nunca é aplicado**. Medido com o drive de B
//! cinco vezes mais forte que o de A: as duas saídas são **bit a bit idênticas**.
//! E fazer B dar um passo seria pior — dois passos por tique correm a física ao
//! dobro da velocidade, o que não é *"dois produtores"*, é o timestep quebrado.
//!
//! ## Topology (the `pre` self-loop)
//!
//! Sequential like the other sims: the height field `wave_h` and its previous frame
//! `wave_prev` ride the `state` feedback port (the `pre` self-loop). Tick 0
//! (`pre` = Empty) **seeds** a flat grid; from tick 1 it steps. `dt` derives from
//! the state's own `sim_t`, clamped to `[0, MAX_DT]` (a loop-wrap freezes one tick).
//!
//! Deterministic → `Effect::Temporal`, replays bit-for-bit.

use ph2d_node_registry::{NodeRegistry, ParamUnit, ParamUnitDecl, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `drive` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// Ceiling on a single step (see `motion.integrate`): guards a playhead jump.
const MAX_DT: f32 = 0.1;
/// CFL stability bound for the 2D leapfrog wave: `C = (c·dt)² ≤ 0.5`.
const CFL_MAX: f32 = 0.49;
/// Grid side clamp (field cost is O(rows·cols)).
const MAX_SIDE: i64 = 60;
/// Baseline dot size (a flat field), and how much a unit of height swells it.
const SIZE_BASE: f32 = 0.22;
const SIZE_GAIN: f32 = 1.4;

/// Quanto uma unidade de altura levanta a peça, em unidades de MUNDO (canal `Y`).
///
/// ⚠️ Escolhido para casar com a excursão que o `SIZE_GAIN` já entrega: um `z` que engorda a
/// peça de `0,22` para `1,62` levanta-a `1,4` — *a mesma amplitude, noutro canal*, para que
/// trocar o canal não obrigue a re-afinar a onda.
const Y_GAIN: f32 = 1.4;
/// Quanto uma unidade de altura inclina a peça, em GRAUS (canal `Rotation`).
const ROT_GAIN_DEG: f32 = 45.0;

/// **PARA ONDE a altura vai** (doc 89 folha 06 — *"o height map é escolhível na referência,
/// aqui é `size`"*).
///
/// ⚠️ **E é aqui que o SINAL da onda deixa de se perder.** O nó publicava `wave_h` assinado e
/// escrevia `size = BASE + GAIN·|z|` — o `abs()` é **correcto num tamanho** (uma peça de tamanho
/// negativo não quer dizer nada), mas como `size` era o ÚNICO destino, a metade negativa da onda
/// era indistinguível da positiva em tudo o que se via: uma crista e um vale desenhavam a mesma
/// bolha. *O defeito não era o `abs()`; era não haver para onde mais mandar a altura.*
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Channel {
    /// O de sempre: a peça ENGORDA com |altura|. `0` — byte-idêntico ao que shipou.
    Size,
    /// A peça SOBE e DESCE — o canal em que a onda de facto se lê como onda.
    Y,
    /// A peça INCLINA com a altura (uma bandeira, um cardume a virar).
    Rotation,
}

impl Channel {
    fn from_param(v: f32) -> Self {
        match v.round() as i32 {
            1 => Self::Y,
            2 => Self::Rotation,
            _ => Self::Size,
        }
    }
}

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.wave"),
    name: "motion.wave",
    inputs: &[
        // The source amplitude driven into the centre cell (animatable). Optional —
        // e desligada significa **fonte NENHUMA**, não uma fonte de valor zero: sem
        // ela o pino de Dirichlet não corre (ver `step`). Num campo que ninguém mais
        // excita as duas leituras coincidem (cravar zero num campo plano é a
        // identidade); elas divergem quando um produtor entra pelo laço de estado.
        PortSpec {
            name: "drive",
            ty: VALUE,
        },
        // The feedback port — auto-wired `out --pre--> state` on add.
        PortSpec {
            name: "state",
            ty: INST_VEC2,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "rows",
            default: 11.0,
        },
        ParamSpec {
            name: "cols",
            default: 11.0,
        },
        ParamSpec {
            name: "spacing",
            default: 0.5,
        },
        // Propagation coefficient ∈ [0, CFL_MAX]: how far a ripple travels per step.
        ParamSpec {
            name: "speed",
            default: 0.35,
        },
        ParamSpec {
            name: "damping",
            default: 0.02,
        },
        // Grid centre in world units (so several fields can sit side by side).
        ParamSpec {
            name: "center_x",
            default: 0.0,
        },
        ParamSpec {
            name: "center_y",
            default: 0.0,
        },
        // ⚠️ **PARA ONDE a altura vai** — ver [`Channel`]. Apendado; `0` (Size) ⇒ o que shipou.
        //
        // ⚠️ **Chama-se `height_channel` e NÃO `channel`, e a palavra tem dono.** No vocabulário
        // da casa `channel` significa *"o canal em que as MINHAS magnitudes se exprimem"* — é
        // isso que o `motion.stagger` e o `motion.wiggle` dizem —, e há um gate de arquitectura
        // a exigir que um nó com `channel` não declare `ParamUnit::Length` fixo em param nenhum
        // (`no_param_of_a_channel_driven_node_is_declared_a_fixed_length`): em Rotation, um
        // Length escalaria GRAUS por `pixels_per_meter`.
        //
        // O `spacing` deste nó é uma distância de mundo **em qualquer canal** — o que este param
        // escolhe é o DESTINO da altura, não a unidade de nada. Usar a palavra da casa fazia o
        // gate reprovar sobre produto correcto, e a cura é o nome, nunca afrouxar o gate.
        // *É a lição do `substeps` outra vez: um sub-passo local que usava a palavra do relógio
        // do grafo fazia o app correr as duas leis.*
        ParamSpec {
            name: "height_channel",
            default: 0.0,
        },
        // **O QUE A PAREDE FAZ** — `0` reflecte (o de sempre, ao bit), `1` absorve.
        //
        // ⚠️ **É o *Reflect Edges* do AE (Wave World), e a nossa metade que faltava era a
        // OUTRA:** o nó só sabia reflectir, e a caixa fechada faz a energia ficar a
        // ressoar para sempre — medido em 2026-08-18 com `damping = 0`, um pulso único
        // deixa a energia a subir e descer sem cair (130 → 775 → 303 → 565 em 30/60/120/240
        // tiques), que é a assinatura de uma sala sem porta.
        //
        // ⚠️ **A capacidade já existia por COMPOSIÇÃO** (quatro nós e o nome de uma coluna
        // de estado que nenhum picker oferece), e é por isso que a célula estava marcada
        // *ergonomia* e não *omissão*. O que este param compra é o GESTO — e ele traz de
        // borda a lei que a rota à mão tinha de honrar num segundo nó (ver [`step`]).
        ParamSpec {
            name: "edges",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Resolved parameters (arithmetic-ready).
struct Params {
    rows: usize,
    cols: usize,
    spacing: f32,
    speed: f32,
    damping: f32,
    center: [f32; 2],
    channel: Channel,
    /// O que a parede faz — ver [`EDGES_ABSORB`].
    edges: i32,
    /// O perfil da camada absorvente. ⚠️ **Não é um param do artista** — ele é
    /// resolvido em [`Sponge::SHIPPED`] e vive aqui para a sonda o poder VARRER
    /// sem uma segunda porta para dentro do `step`.
    sponge: Sponge,
}

impl Params {
    fn count(&self) -> usize {
        self.rows * self.cols
    }
    /// The centre cell index (the driven source).
    fn source(&self) -> usize {
        (self.rows / 2) * self.cols + self.cols / 2
    }
}

fn scalar_col(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// O valor de fonte deste tique, ou **`None` quando não chegou nenhum**.
///
/// ⚠️ **Ausente não é zero, e a distinção decide o pino de Dirichlet.** A porta
/// `drive` é opcional; desligada, a coluna `v` **não existe** e este `Option` é
/// `None`. Colapsá-lo num `0.0` faria a célula central ser *cravada em zero* por
/// uma fonte que ninguém ligou — a mesma leitura que o `falloff` ausente recusa
/// (ele lê `1.0`, o neutro do produto, e não `0.0`).
fn drive_value(vals: &[f32]) -> Option<f32> {
    vals.first().copied()
}

/// The fixed world positions of the grid, centred on `center`, row 0 at the top.
fn grid_positions(p: &Params) -> Vec<[f32; 2]> {
    let (w, h) = (
        (p.cols as f32 - 1.0) * p.spacing,
        (p.rows as f32 - 1.0) * p.spacing,
    );
    let mut out = Vec::with_capacity(p.count());
    for r in 0..p.rows {
        for c in 0..p.cols {
            out.push([
                p.center[0] + c as f32 * p.spacing - w * 0.5,
                p.center[1] + h * 0.5 - r as f32 * p.spacing,
            ]);
        }
    }
    out
}

/// **A BORDA ABSORVENTE** — `edges = 1`. `0` é a reflectora de sempre.
pub const EDGES_ABSORB: i32 = 1;

/// **O PERFIL da camada absorvente** — a largura da esponja e a mordida dela.
///
/// ⚠️ **Ele é um valor e não duas constantes soltas porque os dois números têm de
/// ser VARRIDOS JUNTOS:** uma esponja estreita e forte e uma larga e fraca tiram a
/// mesma energia total e reflectem quantidades muito diferentes, então medir um com
/// o outro fixo mede a combinação e não a lei. A sonda `sweeps_the_sponge_profile`
/// varre a grade dos dois.
#[derive(Copy, Clone, Debug)]
struct Sponge {
    /// A largura da camada, em CÉLULAS.
    cells: f32,
    /// Quanto da amplitude a parede tira por tique, no limite dela.
    ///
    /// ⚠️ **NÃO é `1,0`, e a razão é física e não gosto:** uma máscara que cai a
    /// zero de repente é ela própria um reflector — uma mudança abrupta de
    /// impedância reflecte, que é o fenómeno que esta borda existe para negar. A
    /// esponja é GRADUADA por isso, e o piso fica abaixo de 1 para a última fila
    /// ainda propagar alguma coisa em vez de virar uma segunda parede.
    strength: f32,
}

impl Sponge {
    /// O que SHIPA. ⚠️ **MEDIDO, não escolhido** (`sweeps_the_sponge_profile`, o ECO
    /// que volta ao miolo em % do que uma parede reflectora devolve):
    ///
    /// ```text
    ///   cells\str    0.02    0.05    0.10    0.15    0.25    0.50    1.00
    ///          2   55.81%  45.36%  37.95%  30.10%  31.54%  37.49%  46.49%
    ///          4   54.45%  40.98%  32.64%  30.37%  29.69%  36.67%  40.89%
    ///          6   50.72%  44.00%  27.77%  25.99%  32.30%  33.97%  40.16%
    ///          8   45.87%  41.63%  25.45%  28.83%  31.32%  36.44%  31.93%
    ///         12   39.09%  25.23%  27.63%  25.06%  30.05%  28.65%  29.26%
    /// ```
    ///
    /// ⭐ **A `strength` tem um U, e as duas pontas são ruins por razões DIFERENTES:**
    /// fraca demais mal absorve (`0,02` devolve metade), forte demais reflecte na
    /// própria escada de impedância (`1,00` é a pior coluna em toda a metade estreita).
    /// *A física escrita em [`Sponge::strength`] estava certa e agora está MEDIDA.*
    ///
    /// ⭐ **A largura paga até `6` e depois é ruído** (`6`→`12` vale ~1 ponto), e cada
    /// célula de esponja é miolo que o artista deixa de poder usar ⇒ o joelho é `6`.
    ///
    /// ⚠️ **LIMITE NOMEADO: isto reduz o eco a ~26 %, não a zero.** Uma fronteira
    /// verdadeiramente não-reflectora é a condição de radiação (Mur), que é a outra
    /// família nomeada em [`step`] — ela troca a VIZINHANÇA no bordo, não a amplitude.
    const SHIPPED: Self = Self {
        cells: 6.0,
        strength: 0.15,
    };

    /// **Quanto da amplitude a célula `(r, c)` guarda por tique** — `1` no miolo,
    /// caindo para dentro da parede.
    ///
    /// A queda é QUADRÁTICA na distância à parede mais próxima: é o perfil clássico
    /// de camada absorvente, e é o que faz a transição do miolo para a esponja ser
    /// suave o bastante para não reflectir de volta.
    fn at(self, r: usize, c: usize, rows: usize, cols: usize) -> f32 {
        // A distância à parede mais PRÓXIMA, em células: o mínimo das quatro.
        let dr = r.min(rows.saturating_sub(1).saturating_sub(r));
        let dc = c.min(cols.saturating_sub(1).saturating_sub(c));
        let d = dr.min(dc) as f32;
        if d >= self.cells || self.cells <= 0.0 {
            return 1.0;
        }
        let into = 1.0 - d / self.cells;
        1.0 - self.strength * into * into
    }
}

/// One leapfrog wave step over the height field. Reflecting (Neumann) edges: an
/// out-of-grid neighbour reads as the centre cell, so it contributes no gradient.
///
/// ⚠️ **Com `edges = Absorb` a mesma vizinhança é lida, e o que muda é a
/// AMPLITUDE**: a onda ainda chega à parede, mas chega já gasta. Trocar a
/// vizinhança (uma condição de radiação de Mur) é a outra família, e ela precisa
/// da velocidade local no bordo — que este nó tem, mas ao preço de uma segunda lei
/// a viver ao lado desta. A esponja é a que a COMPOSIÇÃO já exprimia (folha 06,
/// medido 2026-08-18), e trazer para o nó a lei que o artista já podia montar à
/// mão é o que fecha a célula sem inventar física nova.
fn step(h: &[f32], h_prev: &[f32], drive: Option<f32>, p: &Params) -> (Vec<f32>, Vec<f32>) {
    let (rows, cols) = (p.rows, p.cols);
    let coeff = p.speed.clamp(0.0, CFL_MAX);
    let keep = 1.0 - p.damping;
    // ⚠️ **`Reflect` devolve `1.0` para toda célula**, então a multiplicação a
    // jusante é a identidade e o campo é **byte-idêntico** ao que shipava.
    let sponge = (p.edges == EDGES_ABSORB).then_some(p.sponge);
    let mut next = vec![0.0f32; h.len()];
    for r in 0..rows {
        for c in 0..cols {
            let i = r * cols + c;
            let at = |rr: usize, cc: usize| h[rr * cols + cc];
            // 5-point Laplacian with reflecting edges (missing neighbour = centre).
            let up = if r > 0 { at(r - 1, c) } else { h[i] };
            let down = if r + 1 < rows { at(r + 1, c) } else { h[i] };
            let left = if c > 0 { at(r, c - 1) } else { h[i] };
            let right = if c + 1 < cols { at(r, c + 1) } else { h[i] };
            let lap = up + down + left + right - 4.0 * h[i];
            // ⚠️ **A esponja entra UMA vez por valor, no tique em que ele nasce** — e é
            // isso que dá de graça a lei que a rota por composição precisava de um
            // segundo nó para honrar (*amortecer só metade do par do leapfrog custa 4×
            // a energia do campo*, medido 2026-08-18): o `next` de hoje é o `h_prev` de
            // amanhã, logo quando ele chega ali **já passou** pela esponja. Multiplicar
            // também o `prev` no retorno seria passá-lo pela esponja DUAS vezes.
            let bite = sponge.map_or(1.0, |s| s.at(r, c, rows, cols));
            let mut v = (2.0 * h[i] - h_prev[i] + coeff * lap) * keep * bite;
            if !v.is_finite() {
                v = 0.0; // NaN guard: reset a diverged cell
            }
            next[i] = v;
        }
    }
    // Dirichlet source: the centre cell is driven to `drive` — e **só quando um
    // valor de fonte de facto chegou**.
    //
    // ⚠️ **Sem esta guarda o centro é um BURACO em todo campo que a fonte não
    // dirige.** Medido (`measure_wave_producers`, 21×21 com um produtor injectado
    // no laço de estado): a célula central lia `+0,000000` EXACTO entre vizinhas
    // de `+0,062` e `+0,020` — o número redondo que só uma atribuição produz.
    // Enquanto `drive` era o único jeito de excitar o campo isso era invisível
    // (um campo que ninguém dirige é plano, e cravar zero num campo já plano é a
    // identidade); o **Grupo P** mudou isso ao deixar um `motion.drive(Custom…)`
    // escrever `wave_h` de dentro do laço, e aí a cravação passou a apagar tinta
    // que outra pessoa pôs. *Quem move o número que tornava algo inalcançável tem
    // de reconferir a nota.*
    if let Some(d) = drive {
        next[p.source()] = d;
    }
    (next, h.to_vec())
}

/// The whole node as a pure function: seed a flat field on the first tick / a grid
/// change, else step. Emits `P` (fixed grid) + `size` (from |height|) + the
/// `wave_h`/`wave_prev`/`sim_t` state.
fn simulate(drive: Option<f32>, state: &Stream, playhead: f32, p: &Params) -> Stream {
    let n = p.count();
    let s_h = scalar_col(state, "wave_h");
    let s_prev = scalar_col(state, "wave_prev");

    let (h, prev) = if s_h.len() == n && s_prev.len() == n {
        let t_prev = scalar_col(state, "sim_t")
            .first()
            .copied()
            .unwrap_or(playhead);
        let dt = (playhead - t_prev).clamp(0.0, MAX_DT);
        if dt < 1e-6 {
            (s_h, s_prev) // loop-wrap / same tick → hold
        } else {
            step(&s_h, &s_prev, drive, p)
        }
    } else {
        (vec![0.0; n], vec![0.0; n]) // seed a flat field
    };

    // ⚠️ **A altura vai para UM canal, e os outros ficam no neutro.** Espalhá-la por dois
    // faria a mesma grandeza responder duas vezes, e o artista não teria como pedir só uma.
    let mut pos = grid_positions(p);
    let mut size = vec![[SIZE_BASE, SIZE_BASE]; n];
    let mut rot = vec![0.0f32; n];
    match p.channel {
        // ⚠️ O `abs()` fica SÓ aqui, e agora com a razão ao lado: uma peça de tamanho
        // negativo não quer dizer nada. Nos outros canais o sinal é a metade da onda.
        Channel::Size => {
            for (s, &z) in size.iter_mut().zip(&h) {
                let v = SIZE_BASE + SIZE_GAIN * z.abs();
                *s = [v, v];
            }
        }
        Channel::Y => {
            for (q, &z) in pos.iter_mut().zip(&h) {
                q[1] += z * Y_GAIN;
            }
        }
        Channel::Rotation => {
            for (r, &z) in rot.iter_mut().zip(&h) {
                *r = z * ROT_GAIN_DEG;
            }
        }
    }
    Stream::new(n)
        .with("P", Column::Vec2(pos))
        .with("size", Column::Vec2(size))
        .with("rot", Column::Scalar(rot))
        .with("wave_h", Column::Scalar(h))
        .with("wave_prev", Column::Scalar(prev))
        .with("sim_t", Column::Scalar(vec![playhead; n]))
}

struct MotionWave;

impl NodeOp for MotionWave {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let side = |name: &str| (ctx.param(name).round() as i64).clamp(2, MAX_SIDE) as usize;
        let p = Params {
            rows: side("rows"),
            cols: side("cols"),
            spacing: ctx.param("spacing").max(1e-3),
            speed: ctx.param("speed"),
            damping: ctx.param("damping").clamp(0.0, 0.99),
            center: [ctx.param("center_x"), ctx.param("center_y")],
            channel: Channel::from_param(ctx.param("height_channel")),
            edges: ctx.param("edges").round() as i32,
            sponge: Sponge::SHIPPED,
        };
        let playhead = ctx.playhead() as f32;
        let drive = drive_value(&scalar_col(ctx.input(0), VALUE_COL));
        let state = ctx.input(1);
        let out = simulate(drive, state, playhead, &p);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionWave))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Wave",
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "rows",
        label: "Rows",
        min: 2.0,
        max: 60.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "cols",
        label: "Cols",
        min: 2.0,
        max: 60.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "spacing",
        label: "Spacing",
        min: 0.1,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "speed",
        label: "Speed",
        min: 0.0,
        max: CFL_MAX,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "damping",
        label: "Damping",
        min: 0.0,
        max: 0.3,
        step: 0.005,
        widget: ParamWidget::Slider,
    },
    // ⚠️ **O rótulo diz o que a PAREDE faz, não como ela está implementada.** «Sponge» é o
    // nome da técnica; o artista quer saber se a onda volta ou se some — e é isso que ele lê.
    ParamUiHint {
        param: "edges",
        label: "Edges",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Reflect", "Absorb"],
        },
    },
    // ⚠️ **PARA ONDE a altura vai** — um seletor NOMEADO, nunca um slider de passos a decorar
    // (doc 89 folha 06). O vocabulário é o da casa (`motion.drive`), e por isso o artista que
    // aprendeu «Size» num nó não o re-aprende aqui.
    ParamUiHint {
        param: "height_channel",
        label: "Height Drives",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Size", "Y", "Rotation"],
        },
    },
    ParamUiHint {
        param: "center_x",
        label: "Center X",
        min: -20.0,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "center_y",
        label: "Center Y",
        min: -20.0,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
];

/// **What each of this node's numbers IS** (doc 88, Wave A) — never how it is
/// shown. A `Length` is stored in world METRES and the panel resolves the face
/// the artist reads (`px` or `m`) from `ProjectSettings::display_unit`; a node
/// that could pin one would be overriding a setting it does not own.
///
/// Only params whose value is a world COORDINATE or a world DISTANCE are declared
/// here. A weight, a fraction, a rate and a count are left bare on purpose: a unit
/// that is wrong is worse than a unit that is missing, because the artist can read
/// a bare number but a mislabelled one teaches them something false.
static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "spacing",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "center_x",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "center_y",
        unit: ParamUnit::Length,
    },
];

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
