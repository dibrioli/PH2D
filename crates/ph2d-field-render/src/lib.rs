//! `ph2d-field-render` — **o traçador**: marcha raios contra o campo e devolve o que a superfície
//! de facto é ([ADR-0161] §2).
//!
//! # Por que a tela não passa pela malha
//!
//! Medido na W0 (`docs/3DModeling/01_resultados_spike.md` §1c): traçando o campo, a quina do cubo
//! sai como uma navalha e o filete sai liso; a **mesma** cena extraída em malha serrilhava. A
//! geometria estava certa e o defeito era inteiramente da extração. Deixar a malha desenhar a tela
//! seria deixar **o caminho pior definir o teto do que se vê** — o que o [`CLAUDE.md §0`] proíbe.
//!
//! ⚠️ **A W20 curou aquela extração** (`ph2d_field_eval::extract`: a quina do cubo passou de `0/49`
//! faixas capturadas para `116/116`, com desvio `0,00` de célula), e a conclusão **não muda** —
//! muda de razão. Uma malha é uma resolução **escolhida** e um campo não tem nenhuma: ampliar dez
//! vezes revela a faceta da melhor malha e não revela nada num campo. *Curar o segundo lugar não o
//! promove.*
//!
//! # O que sai daqui é GEOMETRIA, não cor
//!
//! [`trace`] devolve um G-buffer: máscara e **normal em espaço de vista**. Nenhuma decisão de cor
//! mora nele. Quem quiser pixels passa um [`Matcap`] a [`shade`]; quem quiser saber **onde** a
//! superfície está sob um pixel usa [`surface_under`].
//!
//! ⚠️ Este parágrafo dizia *"e profundidade"* até 20/08, e o `Gbuffer` **nunca a teve** — um
//! comentário velho a descrever uma API que não existe. Corrigido ao escrever a seleção por clique,
//! que foi a primeira coisa a precisar dela e a descobrir que ela não estava lá.
//!
//! ⚠️ **Matcap, e não o rig de lâmpadas** — e a distinção é da casa, não minha:
//! *"o rig é do DOCUMENTO (a mesma lâmpada acende a tinta ao lado), o matcap é do OLHO"*
//! (`ph2d-mesh-render::matcap`). Um viewport de modelagem lê **forma**, e forma se lê com a luz
//! presa à câmera. Inventar um segundo rig aqui seria exatamente o erro que a `ph2d-light` existe
//! para impedir.
//!
//! [ADR-0161]: ../../../docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md
//! [`CLAUDE.md §0`]: ../../../CLAUDE.md

use ph2d_field::FieldDoc;
use rayon::prelude::*;

/// Distância abaixo da qual o raio considerou-se na superfície — **o teto**, nunca o valor fixo.
///
/// ⚠️ Ver [`Sharpness`]: o valor efetivo desce com o zoom.
const HIT_EPS: f32 = 2.0e-4;
/// Quanto um raio anda antes de desistir.
const T_MAX: f32 = 8.0;
const MAX_STEPS: usize = 400;
/// Passo da diferença central que devolve a normal — **o teto**, pelo mesmo motivo.
const NORMAL_EPS: f32 = 1.0e-4;
/// O piso dos dois, e ele nomeia o recurso de que é: a **precisão da representação**.
///
/// O campo é avaliado em `f32`. À escala de uma peça de tamanho unitário isso dá ~10⁻⁷ de erro
/// absoluto, então uma diferença central com passo abaixo de ~10⁻⁶ mede ruído de cancelamento, não
/// gradiente. Abaixo daqui, refinar **piora**.
const PRECISION_FLOOR: f32 = 1.0e-6;

/// ⭐⭐⭐ **O estêncil com que o produto lê a normal** — ver [`Stencil`], e **um** endereço.
///
/// ⚠️ **Ele é uma constante, e não um argumento, porque a resposta é do MÓDULO e não de quem
/// chama:** as seis portas de traçado (o quadro, a re-amostragem da borda, os dois `pick`, as duas
/// sondas) leem a mesma superfície, e um estêncil por porta faria a silhueta re-amostrada ler a
/// forma por outra lei que o interior — o defeito seria uma orla de sombreado à volta da peça, que
/// é precisamente o que o anti-serrilhado existe para não haver.
const NORMAL_STENCIL: Stencil = Stencil::Central6;

/// ⭐ **Quantas amostras de campo uma normal custa no produto** — **derivado** do
/// [`NORMAL_STENCIL`], nunca escrito ao lado dele.
///
/// ⚠️ Um `6` escrito à mão aqui seria a segunda resposta à mesma pergunta, e a que envelhece: o
/// gate que a lê passaria a defender o estêncil de ontem no dia em que a constante mudasse.
#[doc(hidden)]
pub const NORMAL_STENCIL_WIDTH: usize = NORMAL_STENCIL.offsets().len();

/// As duas tolerâncias da marcha, **derivadas do tamanho do pixel em mundo**.
///
/// # ⭐ Por que elas não são constantes
///
/// Um campo implícito tem detalhe infinito: aproximar a câmera devia mostrar mais forma. Com um
/// `HIT_EPS` fixo isso **para** — assim que o pixel fica mais fino que a tolerância, a superfície
/// deixa de ganhar nitidez e passa a ganhar franja, e o módulo herda um teto de zoom que nada na
/// física pedia.
///
/// A cura não é escrever um limite de zoom: é **remover a causa** (`CLAUDE.md §0`). As duas
/// tolerâncias descem com o pixel, e só param no [`PRECISION_FLOOR`], que é um limite legítimo
/// porque diz de que recurso é.
///
/// ⚠️ São **tetos**, e no enquadramento normal nada muda: a 480 px de altura com `half_extent = 0,8`
/// o pixel mede 0,0033 e o teto de 2·10⁻⁴ continua a mandar. A adaptação só morde a partir de ~4×
/// de aproximação, que é exatamente onde o problema começava.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Sharpness {
    hit: f32,
    normal: f32,
}

impl Sharpness {
    fn for_frame(half_extent: f32, side_px: usize) -> Self {
        let pixel = 2.0 * half_extent / (side_px.max(1) as f32);
        Self {
            hit: HIT_EPS.min(pixel * 0.25).max(PRECISION_FLOOR),
            normal: NORMAL_EPS.min(pixel * 0.5).max(PRECISION_FLOOR),
        }
    }
}

mod camera;
mod edges;
mod march;
mod shade;
mod tiles;
use edges::resample_edges;
use march::{Scene, march};
use tiles::{SLABS, TILE, tiled_trace};

pub use camera::{DEFAULT_HALF_FOV, Lens, ORTHO_START, Orbit, Screen};
pub use march::{
    FORKED, HIST, MARCH_RAYS, NORMAL_SAMPLES, SLAB_SAMPLES, SLABS_COUNTED, STEP_HIST, STEP_SAMPLES,
    Stencil,
};
#[doc(hidden)]
pub use shade::Matcap;
pub use shade::shade;
pub use tiles::{SLAB_SPEC, SPECIALISE_NS, SPECIALISED, TILE_MAX};

/// O padrão de re-amostragem de um pixel de borda: **4-rook (RGSS)**.
///
/// ⚠️ Quatro posições numa grelha ROTACIONADA, e não os quatro cantos de uma grelha 2×2. A razão é
/// a que a indústria mediu há trinta anos: uma grelha alinhada dá **duas** posições distintas a uma
/// aresta quase horizontal (as duas amostras de cima caem do mesmo lado), enquanto a rotacionada dá
/// **quatro** — o dobro dos níveis de cobertura exatamente nas arestas que mais aparecem.
const ROOK: [(f32, f32); 4] = [
    (0.125, 0.625),
    (0.375, 0.125),
    (0.625, 0.875),
    (0.875, 0.375),
];

/// O cosseno abaixo do qual duas normais vizinhas são **aresta**, e não curvatura.
///
/// ⚠️ Errar para o lado do EXCESSO é barato e errar para o lado da falta não é: um pixel de borda
/// marcado a mais custa quatro raios; um pixel de borda **não** marcado é um degrau visível que
/// nenhum passo posterior recupera. 0,9 ≈ 25°.
const EDGE_COS: f32 = 0.9;

/// Um pixel de borda, re-amostrado no padrão [`ROOK`].
///
/// ⚠️ A amostra do centro do pixel é **descartada** nestes: as quatro do padrão a substituem. Somar
/// as cinco daria peso duplo ao centro e enviesaria a cobertura para o lado que ele calhou de
/// apanhar.
#[derive(Clone, Copy, Debug)]
pub struct EdgePixel {
    pub pixel: u32,
    pub hit: [bool; 4],
    pub normal: [[f32; 3]; 4],
}

/// O que o traçado sabe da superfície, por pixel. **Sem cor.**
pub struct Gbuffer {
    pub width: u32,
    pub height: u32,
    /// O raio encontrou superfície?
    pub hit: Vec<bool>,
    /// Normal em **espaço de vista** (`z` para o observador), unitária. Lixo onde `!hit`.
    pub normal: Vec<[f32; 3]>,
    /// Os pixels onde a imagem tem **aresta** — de silhueta ou de quina —, com quatro amostras cada.
    ///
    /// Vazio quando o traçado corre sem anti-serrilhado. Ordenado por `pixel`, sempre: é o que faz
    /// a saída não depender de como as threads se dividiram.
    pub edges: Vec<EdgePixel>,
}

impl Gbuffer {
    #[must_use]
    pub fn hits(&self) -> usize {
        self.hit.iter().filter(|h| **h).count()
    }

    /// A **cobertura** do pixel `i` — 1,0 cheio, 0,0 vazio, e a fração exata numa borda.
    #[must_use]
    pub fn coverage(&self, i: usize) -> f32 {
        if let Ok(k) = self.edges.binary_search_by_key(&(i as u32), |e| e.pixel) {
            let n = self.edges[k].hit.iter().filter(|h| **h).count();
            return n as f32 / 4.0;
        }
        if self.hit[i] { 1.0 } else { 0.0 }
    }
}

/// Marcha o campo do documento e devolve o G-buffer, **com anti-serrilhado adaptativo**.
#[must_use]
pub fn trace(
    doc: &FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &Orbit,
    width: u32,
    height: u32,
) -> Gbuffer {
    trace_with(doc, reg, cam, width, height, true, true)
}

/// ⭐ **A marcha CANCELÁVEL** — `None` quando o pedido foi abandonado a meio.
///
/// ⚠️ **Ela existe para uma latência medida, não por elegância** (ADR-0161 W32): o refinamento de um
/// quadro cheio custa até **121 ms** na cena mais pesada, e enquanto ele corre a mão que recomeça a
/// mexer espera por ele. O `cancel` é lido **por linha**: uma marcha abandonada custa o resto das
/// linhas a zero, não o resto da imagem.
///
/// ⚠️ **Quem cancela decide, e a decisão não é daqui**: esta função não sabe o que vale a pena
/// abandonar. Ver `field3d_preview::cancels_the_inflight` — cancelar tudo faria a imagem **nunca
/// chegar** durante um arrasto contínuo.
///
/// ⭐⭐⭐ **`antialias` é o segundo botão do quadro de MOVIMENTO** (W71) — a mesma lei que a W69 já
/// ship: *grosso a mexer, nítido ao assentar*. A segunda passagem re-marcha a silhueta **quatro
/// vezes**, e isso custa **`1,30×`–`1,40×` do quadro**, medido a `640×360` em cinco densidades de
/// contorno (`measure_the_two_knobs_of_the_moving_frame`):
///
/// | arestas | 48 | 64 | 96 | 128 | 168 |
/// |---|---:|---:|---:|---:|---:|
/// | com | `14,6` | `17,5` | `22,3` | `28,8` | `35,7` |
/// | sem | `10,9` | `12,5` | `17,1` | `20,6` | `26,7` |
///
/// ⚠️ **Quem escolhe é o chamador, e ele já sabe a resposta**: o mesmo sítio que decide engrossar o
/// contorno sabe se a mão está a mexer. *Uma bandeira nova aqui não acrescenta uma decisão — ela
/// alcança a que já existe.*
#[must_use]
pub fn trace_cancellable(
    doc: &FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &Orbit,
    width: u32,
    height: u32,
    cancel: &std::sync::atomic::AtomicBool,
    antialias: bool,
) -> Option<Gbuffer> {
    let g = trace_inner(doc, reg, cam, width, height, true, antialias, Some(cancel));
    (!cancel.load(std::sync::atomic::Ordering::Relaxed)).then_some(g)
}

/// Igual a [`trace`], com o paralelismo sob controle — é o que o gate de byte-identidade dirige.
#[must_use]
pub fn trace_with_threads(
    doc: &FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &Orbit,
    width: u32,
    height: u32,
    parallel: bool,
) -> Gbuffer {
    trace_with(doc, reg, cam, width, height, parallel, true)
}

/// A porta completa.
///
/// # ⭐ O anti-serrilhado é ADAPTATIVO, e a razão é aritmética
///
/// Supersamplear a imagem inteira a 4× custa **4×**. Mas a serrilha só existe onde há **aresta**, e
/// aresta é uma pequena fração dos pixels (medido: ver `docs/3DModeling/05_resultados_imagem.md`).
/// Então: um raio por pixel, deteta-se onde a imagem tem descontinuidade, e **só esses** pixels
/// levam as quatro amostras do padrão [`ROOK`].
///
/// ⚠️ **O detector olha DUAS coisas, e a segunda é a que quase se esquece:** a máscara (silhueta
/// contra o fundo) *e* a **normal** (uma quina viva, ou uma superfície que passa à frente de
/// outra). Um detector só de máscara deixa serrilhada exatamente a aresta que este módulo existe
/// para entregar afiada.
#[must_use]
pub fn trace_with(
    doc: &FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &Orbit,
    width: u32,
    height: u32,
    parallel: bool,
    antialias: bool,
) -> Gbuffer {
    trace_inner(doc, reg, cam, width, height, parallel, antialias, None)
}

/// O corpo, com a bandeira de cancelamento **opcional** — ver [`trace_cancellable`].
///
/// ⚠️ Um só corpo de propósito: duas marchas seriam dois caminhos por onde a imagem pode divergir, e
/// a paridade delas não teria como ser medida sem uma terceira função para as comparar.
#[allow(clippy::too_many_arguments)]
fn trace_inner(
    doc: &FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &Orbit,
    width: u32,
    height: u32,
    parallel: bool,
    antialias: bool,
    cancel: Option<&std::sync::atomic::AtomicBool>,
) -> Gbuffer {
    trace_inner_tiles(
        doc,
        reg,
        cam,
        width,
        height,
        parallel,
        antialias,
        cancel,
        true,
        ph2d_field_eval::safe_march_step(doc),
        NORMAL_STENCIL,
    )
}

/// ⭐ **A marcha por ladrilho com o lado ESCOLHIDO** — a porta que a sonda do `TILE` dirige.
#[doc(hidden)]
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn trace_tiled_for_test(
    doc: &FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &Orbit,
    width: u32,
    height: u32,
    tile: usize,
    slabs: usize,
    antialias: bool,
    parallel: bool,
) -> Option<Gbuffer> {
    let shape = ph2d_field_eval::hybrid::Hybrid::new(doc, reg);
    let rc = ph2d_field_eval::RegionCompiler::new(doc);
    let bbox = ph2d_field_eval::bounds::bounding_ball(doc, reg)
        .map(ph2d_field_eval::bounds::Ball::aabb)?;
    if shape.sampled_count() != 0 || !rc.is_worth_it() {
        return None;
    }
    let plane = Screen::new(width, height, cam.half_extent);
    let scene = Scene {
        shape: &shape,
        cam,
        basis: cam.basis(),
        sharp: Sharpness::for_frame(cam.half_extent, (width as usize).min(height as usize)),
        clip: Some(bbox),
        step: ph2d_field_eval::safe_march_step(doc),
        stencil: NORMAL_STENCIL,
    };
    Some(tiled_trace(
        doc, &rc, &scene, plane, bbox, parallel, antialias, None, tile, slabs,
    ))
}

/// ⭐⭐ **A marcha com o PASSO escolhido** — a porta que a sonda do passo dirige.
///
/// ⚠️ Ela existe para que as duas respostas sejam medidas no **mesmo processo**: entre duas corridas
/// desta workstation a montagem — que não depende do passo — mexeu-se `14,4 -> 22,1 ms`, e um A/B
/// nessas condições mede o relógio da máquina, não a mudança.
#[doc(hidden)]
#[must_use]
pub fn trace_stepped_for_test(
    doc: &FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &Orbit,
    width: u32,
    height: u32,
    step: f32,
) -> Gbuffer {
    trace_inner_tiles(
        doc,
        reg,
        cam,
        width,
        height,
        true,
        true,
        None,
        true,
        step,
        NORMAL_STENCIL,
    )
}

/// ⭐ **Quantas fatias de profundidade o produto reparte** — ver [`tiles::SLABS`].
///
/// ⚠️ Ela existe porque um binário de teste não alcança um `pub(crate)` e por isso **escolhia um
/// número**: o `tape_budget` media com `2` desde a W70, e o produto ship `4` desde a W71. *Um gate
/// que escolhe a configuração mede a configuração que escolheu.*
#[doc(hidden)]
#[must_use]
pub const fn slabs_for_test() -> usize {
    SLABS
}

/// ⭐ **O lado do ladrilho que o produto usa** — ver [`tiles::TILE`], e pela mesma razão do
/// [`slabs_for_test`].
#[doc(hidden)]
#[must_use]
pub const fn tile_for_test() -> usize {
    TILE
}

/// ⭐⭐ **A marcha com o ESTÊNCIL escolhido** — a porta que a sonda da normal dirige.
///
/// ⚠️ Pela mesma razão da [`trace_stepped_for_test`]: as duas respostas têm de ser medidas no
/// **mesmo processo**, e a comparação que interessa é entre as duas IMAGENS — o ângulo entre as
/// normais, pixel a pixel, que não depende do relógio da máquina.
#[doc(hidden)]
#[must_use]
pub fn trace_stencil_for_test(
    doc: &FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &Orbit,
    width: u32,
    height: u32,
    stencil: Stencil,
) -> Gbuffer {
    trace_inner_tiles(
        doc,
        reg,
        cam,
        width,
        height,
        true,
        false,
        None,
        true,
        ph2d_field_eval::safe_march_step(doc),
        stencil,
    )
}

/// ⭐ **A MARCHA DE LINHA, forçada** — a porta que o gate de paridade dirige.
///
/// ⚠️ Ela existe porque, com um perfil no documento, o caminho por ladrilho passa a ser o **único**
/// alcançável — e uma paridade que não consegue chamar as duas metades não é uma paridade.
#[doc(hidden)]
#[must_use]
pub fn trace_by_rows_for_test(
    doc: &FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &Orbit,
    width: u32,
    height: u32,
) -> Gbuffer {
    trace_inner_tiles(
        doc,
        reg,
        cam,
        width,
        height,
        true,
        true,
        None,
        false,
        ph2d_field_eval::safe_march_step(doc),
        NORMAL_STENCIL,
    )
}

#[allow(clippy::too_many_arguments)]
fn trace_inner_tiles(
    doc: &FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &Orbit,
    width: u32,
    height: u32,
    parallel: bool,
    antialias: bool,
    cancel: Option<&std::sync::atomic::AtomicBool>,
    tiles_allowed: bool,
    step: f32,
    stencil: Stencil,
) -> Gbuffer {
    let shape = ph2d_field_eval::hybrid::Hybrid::new(doc, reg);
    let basis = cam.basis();
    let (w, h) = (width as usize, height as usize);
    // ⚠️ **A MESMA conta que o gizmo projeta** ([`Screen`]): a marcha constrói os raios a partir
    // dela e a alça pousa por ela. Duas cópias divergiriam meio pixel e o sintoma seria uma seta
    // que agarra ao lado da superfície que ela move.
    let plane = Screen::new(width, height, cam.half_extent);
    let sharp = Sharpness::for_frame(cam.half_extent, w.min(h));
    let scene = Scene {
        shape: &shape,
        cam,
        basis,
        sharp,
        clip: None,
        step,
        stencil,
    };

    // ⭐⭐⭐ **A MARCHA POR LADRILHO** (W56) — o consumidor da especialização por região.
    //
    // ⚠️ **Só quando o documento é todo analítico e tem forma de perfil.** Com escultura por baixo o
    // `Hybrid` parte-se em várias fitas e a árvore especializada não é o documento inteiro; sem
    // perfil nenhum, não há o que especializar e o ladrilho só acrescentaria montagem de fita.
    let rc = ph2d_field_eval::RegionCompiler::new(doc);
    // ⭐ **O recorte pela caixa da peça vale para as DUAS marchas**, e é uma melhoria por si só: o
    // raio começa na entrada da caixa e pára na saída, em vez de percorrer o vazio até `T_MAX`.
    //
    // ⚠️ **E é o que torna a paridade uma paridade.** Se só o ladrilho recortasse, as duas marchas
    // começariam em `t` diferentes, parariam em pontos ligeiramente diferentes, e numa **quina viva**
    // a normal salta — a diferença lida como defeito da especialização sem o ser.
    let bbox =
        ph2d_field_eval::bounds::bounding_ball(doc, reg).map(ph2d_field_eval::bounds::Ball::aabb);
    let scene = Scene {
        clip: bbox,
        ..scene
    };
    if let Some(bbox) =
        bbox.filter(|_| tiles_allowed && shape.sampled_count() == 0 && rc.is_worth_it())
    {
        return tiled_trace(
            doc, &rc, &scene, plane, bbox, parallel, antialias, cancel, TILE, SLABS,
        );
    }

    // Passo 1: um raio por pixel, uma fatia por linha.
    let row = |y: usize| -> (Vec<bool>, Vec<[f32; 3]>) {
        let pts: Vec<(f32, f32)> = (0..w)
            .map(|x| plane.plane_at(x as f32 + 0.5, y as f32 + 0.5))
            .collect();
        // O ponto de mundo não interessa a um quadro inteiro — quem o quer é a seleção por
        // clique (`surface_under`), um raio de cada vez.
        // ⚠️ **A bandeira é lida POR LINHA.** Uma marcha abandonada custa o resto das linhas a
        // zero — e não o resto da imagem, que é o que a espera de 121 ms era.
        if cancel.is_some_and(|c| c.load(std::sync::atomic::Ordering::Relaxed)) {
            return (vec![false; w], vec![[0.0; 3]; w]);
        }
        let (h, n, _) = march(&scene, &pts);
        (h, n)
    };
    let rows: Vec<(Vec<bool>, Vec<[f32; 3]>)> = if parallel {
        (0..h).into_par_iter().map(row).collect()
    } else {
        (0..h).map(row).collect()
    };
    let mut hit = Vec::with_capacity(w * h);
    let mut normal = Vec::with_capacity(w * h);
    for (rh, rn) in rows {
        hit.extend(rh);
        normal.extend(rn);
    }

    // Passo 2: re-amostrar as bordas.
    let edges =
        if antialias && !cancel.is_some_and(|c| c.load(std::sync::atomic::Ordering::Relaxed)) {
            resample_edges(&scene, plane, &hit, &normal, parallel)
        } else {
            Vec::new()
        };

    Gbuffer {
        width,
        height,
        hit,
        normal,
        edges,
    }
}

/// ⭐ **Onde o raio entra e sai da caixa** (o *slab test*), ou `None` se ele a falha.
///
/// ⚠️ Um raio **paralelo** a um eixo tem `1/d = ±∞`, e a aritmética de `f32` trata-o certo: o
/// intervalo sai `[-∞, +∞]` (dentro) ou vazio (fora), sem um caso especial que alguém esqueceria.
pub(crate) fn slab(o: [f32; 3], d: [f32; 3], lo: [f32; 3], hi: [f32; 3]) -> Option<(f32, f32)> {
    let (mut t0, mut t1) = (f32::NEG_INFINITY, f32::INFINITY);
    for k in 0..3 {
        let inv = 1.0 / d[k];
        let (a, b) = ((lo[k] - o[k]) * inv, (hi[k] - o[k]) * inv);
        let (a, b) = if a <= b { (a, b) } else { (b, a) };
        t0 = t0.max(a);
        t1 = t1.min(b);
    }
    (t0 <= t1 && t1 > 0.0).then_some((t0, t1))
}

/// ⭐ **Onde a superfície está sob um pixel** — a pergunta que uma seleção por clique faz.
///
/// ⚠️ **Um raio, pela MESMA marcha.** Uma função própria que repetisse o laço seria a segunda
/// resposta a *"onde está a superfície?"*, e as duas divergiriam no dia em que uma tolerância
/// mudasse — com o sintoma a aparecer como *"clicar na peça seleciona o objeto errado"*, que
/// ninguém liga a uma tolerância de marcha.
///
/// `None` quando o raio não encontrou nada: o clique caiu no fundo.
#[must_use]
pub fn surface_under(
    doc: &FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &Orbit,
    screen: Screen,
    px: [f32; 2],
) -> Option<[f32; 3]> {
    let shape = ph2d_field_eval::hybrid::Hybrid::new(doc, reg);
    let side = screen.width().min(screen.height()) as usize;
    let scene = Scene {
        shape: &shape,
        cam,
        basis: cam.basis(),
        sharp: Sharpness::for_frame(cam.half_extent, side),
        clip: None,
        step: ph2d_field_eval::safe_march_step(doc),
        stencil: NORMAL_STENCIL,
    };
    let (hit, _, point) = march(&scene, &[screen.plane_at(px[0], px[1])]);
    hit[0].then(|| point[0])
}

/// ⭐⭐⭐ **A MESMA PERGUNTA, PARA MUITOS PIXELS DE UMA VEZ** (W58) — a porta de um laço de seleção.
///
/// ⚠️ **Ela existe por um custo medido, e a irmã de cima é o caso de um.** A [`surface_under`]
/// compila a árvore do documento **a cada chamada** (é um JIT: `2,3 ms` num contorno de 168 arestas,
/// ver `tiles::TILE`). Um laço que amostrasse 300 pixels chamando-a 300 vezes pagaria 300 JITs —
/// ⛔ **quase um segundo** para responder a um arrasto. Aqui a árvore é compilada **uma** vez e a
/// marcha recebe o lote inteiro, que é exactamente o que ela já sabe fazer.
///
/// `None` numa posição = aquele raio caiu no fundo.
#[must_use]
pub fn surfaces_under(
    doc: &FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &Orbit,
    screen: Screen,
    px: &[[f32; 2]],
) -> Vec<Option<[f32; 3]>> {
    if px.is_empty() {
        return Vec::new();
    }
    let shape = ph2d_field_eval::hybrid::Hybrid::new(doc, reg);
    let side = screen.width().min(screen.height()) as usize;
    let scene = Scene {
        shape: &shape,
        cam,
        basis: cam.basis(),
        sharp: Sharpness::for_frame(cam.half_extent, side),
        clip: None,
        step: ph2d_field_eval::safe_march_step(doc),
        stencil: NORMAL_STENCIL,
    };
    let pts: Vec<(f32, f32)> = px.iter().map(|p| screen.plane_at(p[0], p[1])).collect();
    let (hit, _, point) = march(&scene, &pts);
    (0..px.len()).map(|i| hit[i].then(|| point[i])).collect()
}

#[cfg(test)]
mod tests;
