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

/// `1/√2` — o passo seguro da marcha.
///
/// ⚠️ **Não é um fator de conforto: é o recíproco de uma constante MEDIDA.** A W0 mediu ‖∇f‖
/// chegando a **√2** no operador de arredondamento exato (`01_resultados_spike.md` §3), onde duas
/// superfícies se tocam quase tangentes. Avançar `d` num campo assim **atravessa** a superfície, e
/// o furo aparece como pixel de fundo no meio da peça.
const SAFE_STEP: f32 = std::f32::consts::FRAC_1_SQRT_2;

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
mod shade;

pub use camera::{DEFAULT_HALF_FOV, Lens, ORTHO_START, Orbit, Screen};
pub use shade::Matcap;
pub use shade::shade;

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
#[must_use]
pub fn trace_cancellable(
    doc: &FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &Orbit,
    width: u32,
    height: u32,
    cancel: &std::sync::atomic::AtomicBool,
) -> Option<Gbuffer> {
    let g = trace_inner(doc, reg, cam, width, height, true, true, Some(cancel));
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
        doc, reg, cam, width, height, parallel, antialias, cancel, true,
    )
}

/// ⭐ **A marcha por ladrilho com o lado ESCOLHIDO** — a porta que a sonda do `TILE` dirige.
#[doc(hidden)]
#[must_use]
pub fn trace_tiled_for_test(
    doc: &FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &Orbit,
    width: u32,
    height: u32,
    tile: usize,
    antialias: bool,
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
    };
    Some(tiled_trace(
        doc, &rc, &scene, plane, bbox, true, antialias, None, tile,
    ))
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
    trace_inner_tiles(doc, reg, cam, width, height, true, true, None, false)
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
            doc, &rc, &scene, plane, bbox, parallel, antialias, cancel, TILE,
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

/// O que um ladrilho devolve: os índices dos pixels dele, a máscara e as normais.
type TileResult = (Vec<usize>, Vec<bool>, Vec<[f32; 3]>);

/// O lado de um ladrilho, em pixels — **medido, não escolhido**.
///
/// ⚠️ **É o vale entre duas contas que puxam para lados opostos:** a fita especializada custa menos
/// quanto **menor** for a pegada do ladrilho, e a montagem dela paga-se uma vez **por ladrilho**.
/// A varredura (`the_table_of_what_the_tiled_march_buys`, 640×480, mediana de 7, o quadro inteiro em
/// ms):
///
/// | arestas | 32 | 48 | **64** | 96 | 128 | 192 |
/// |---:|---:|---:|---:|---:|---:|---:|
/// | 56 | 61 | **43** | 44 | 47 | 50 | 139 |
/// | 168 | 143 | 102 | **90** | 93 | 130 | 216 |
/// | 664 | 523 | 362 | **330** | 355 | 487 | 838 |
///
/// ⭐ O vale é raso entre **48 e 96** e fundo fora dele: a 32 px a montagem domina (400 fitas por
/// quadro), a 192 px a pegada volta a trazer o contorno quase inteiro. Da tabela sai também o
/// modelo — **≈ 0,29 ms de montagem por ladrilho**, e o resto é avaliação.
const TILE: usize = 64;

/// ⭐⭐⭐ **A marcha por ladrilho, com uma árvore por região** — ver o `TILE`.
///
/// ⚠️ **A região é a caixa do FRUSTUM do ladrilho intersectada com a da peça**, e a marcha de cada
/// raio é presa à caixa da peça (`Scene::clip`). As duas metades juntas são o que torna a árvore
/// especializada válida em **todo** ponto avaliado: nenhuma amostra cai fora da região para que ela
/// foi construída.
#[allow(clippy::too_many_arguments)]
fn tiled_trace(
    doc: &FieldDoc,
    rc: &ph2d_field_eval::RegionCompiler,
    scene: &Scene<'_>,
    plane: Screen,
    bbox: ([f32; 3], [f32; 3]),
    parallel: bool,
    antialias: bool,
    cancel: Option<&std::sync::atomic::AtomicBool>,
    tile: usize,
) -> Gbuffer {
    let (w, h) = (plane.width() as usize, plane.height() as usize);
    let (out_w, out_h) = (plane.width() as u32, plane.height() as u32);
    let tiles: Vec<(usize, usize)> = (0..h.div_ceil(tile))
        .flat_map(|ty| (0..w.div_ceil(tile)).map(move |tx| (tx, ty)))
        .collect();
    let one = |&(tx, ty): &(usize, usize)| -> TileResult {
        let (x0, y0) = (tx * tile, ty * tile);
        let (x1, y1) = ((x0 + tile).min(w), (y0 + tile).min(h));
        let mut idx = Vec::with_capacity((x1 - x0) * (y1 - y0));
        let mut pts = Vec::with_capacity(idx.capacity());
        for y in y0..y1 {
            for x in x0..x1 {
                idx.push(y * w + x);
                pts.push(plane.plane_at(x as f32 + 0.5, y as f32 + 0.5));
            }
        }
        let empty = (
            idx.len(),
            vec![false; idx.len()],
            vec![[0.0f32; 3]; idx.len()],
        );
        if cancel.is_some_and(|c| c.load(std::sync::atomic::Ordering::Relaxed)) {
            return (idx, empty.1, empty.2);
        }
        // A caixa do frustum do ladrilho: os quatro raios de canto, de `t = 0` a `T_MAX`.
        let Some(region) = tile_region(
            scene.cam,
            plane,
            (x0, y0),
            (x1, y1),
            bbox,
            scene.sharp.normal,
        ) else {
            // A caixa do ladrilho não cruza a da peça ⇒ nenhum raio dele acerta em nada.
            return (idx, empty.1, empty.2);
        };
        let tree = rc.compile(doc, region.0, region.1);
        let local = ph2d_field_eval::hybrid::Hybrid::from_tree(tree);
        let tile_scene = Scene {
            shape: &local,
            cam: scene.cam,
            basis: scene.basis,
            sharp: scene.sharp,
            clip: Some(bbox),
        };
        let (hit, normal, _) = march(&tile_scene, &pts);
        (idx, hit, normal)
    };
    let done: Vec<TileResult> = if parallel {
        tiles.par_iter().map(one).collect()
    } else {
        tiles.iter().map(one).collect()
    };
    let mut hit = vec![false; w * h];
    let mut normal = vec![[0.0f32; 3]; w * h];
    for (idx, th, tn) in done {
        for (k, &i) in idx.iter().enumerate() {
            hit[i] = th[k];
            normal[i] = tn[k];
        }
    }
    let edges =
        if antialias && !cancel.is_some_and(|c| c.load(std::sync::atomic::Ordering::Relaxed)) {
            resample_edges(scene, plane, &hit, &normal, parallel)
        } else {
            Vec::new()
        };
    Gbuffer {
        width: out_w,
        height: out_h,
        hit,
        normal,
        edges,
    }
}

/// A caixa de mundo que contém tudo o que os raios deste ladrilho podem amostrar **dentro da peça**.
///
/// ⚠️ **Os quatro raios de CANTO bastam, e não é aproximação:** o frustum de um ladrilho é o casco
/// convexo dos quatro segmentos de canto (a lente é convergente ou paralela, e nas duas o ladrilho é
/// um quadrilátero plano), então a caixa dos oito extremos contém todo raio interior.
/// ⚠️ **`margin` não é folga de conforto — é a SONDA DA NORMAL.** Ela é uma diferença central em
/// `ponto ± ε`, e um `ε` que saia da região faria a árvore especializada responder onde ela não vale.
/// O sintoma medido: 90 pixels a **apagarem-se** (o gradiente saía nulo e a marcha desistia do
/// acerto), num quadro em que a máscara devia ser idêntica. *Uma região tem de conter tudo o que é
/// avaliado — inclusive o que é avaliado DEPOIS de o raio parar.*
pub(crate) fn tile_region(
    cam: &Orbit,
    plane: Screen,
    lo_px: (usize, usize),
    hi_px: (usize, usize),
    bbox: ([f32; 3], [f32; 3]),
    margin: f32,
) -> Option<([f32; 3], [f32; 3])> {
    // ⭐⭐ **A faixa de `t` é a da CAIXA, não `[0, T_MAX]`.**
    //
    // ⛔ **Medido:** com `T_MAX` o tubo do ladrilho é tão comprido que a caixa dele engole a peça
    // inteira — a região de **todo** ladrilho saía sendo a peça, e a especialização comprava `1,3×`
    // em vez dos `5×` que a tabela do §57.12 prometia. *Uma região que não é menor que a peça não é
    // uma região.*
    //
    // ⚠️ **E os extremos estão nos cantos**, o que torna quatro raios suficientes: a entrada e a
    // saída da caixa são máximo e mínimo de funções lineares sobre o quadrilátero do ladrilho.
    let corners = [
        (lo_px.0 as f32, lo_px.1 as f32),
        (hi_px.0 as f32, lo_px.1 as f32),
        (lo_px.0 as f32, hi_px.1 as f32),
        (hi_px.0 as f32, hi_px.1 as f32),
    ];
    let (mut t_lo, mut t_hi) = (f32::INFINITY, f32::NEG_INFINITY);
    for (px, py) in corners {
        let (sx, sy) = plane.plane_at(px, py);
        let (o, d) = cam.ray_at_plane(sx, sy);
        if let Some((a, b)) = slab(o, d, bbox.0, bbox.1) {
            t_lo = t_lo.min(a.max(0.0));
            t_hi = t_hi.max(b.min(T_MAX));
        }
    }
    if !t_lo.is_finite() || t_lo > t_hi {
        // Nenhum raio de canto alcança a caixa. ⚠️ Um raio INTERIOR ainda pode, então o que se faz é
        // **desistir da especialização** — nunca dar o ladrilho por vazio.
        return Some(bbox);
    }
    let (mut lo, mut hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
    for (px, py) in corners {
        let (sx, sy) = plane.plane_at(px, py);
        let (o, d) = cam.ray_at_plane(sx, sy);
        for t in [t_lo, t_hi] {
            for k in 0..3 {
                let v = d[k].mul_add(t, o[k]);
                lo[k] = lo[k].min(v);
                hi[k] = hi[k].max(v);
            }
        }
    }
    // …intersectada com a da peça: fora dela não há superfície nenhuma.
    let mut out = ([0.0f32; 3], [0.0f32; 3]);
    let pad = margin * 4.0;
    for k in 0..3 {
        out.0[k] = lo[k].max(bbox.0[k]);
        out.1[k] = hi[k].min(bbox.1[k]);
        if out.0[k] > out.1[k] {
            return None;
        }
        out.0[k] -= pad;
        out.1[k] += pad;
    }
    Some(out)
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

/// **Tudo o que uma marcha precisa de saber, e que não muda entre lotes.**
///
/// Os quatro viajam sempre juntos — a árvore compilada, a câmera, a base dela e as tolerâncias do
/// quadro. Passá-los soltos era o que fazia as duas funções abaixo crescerem para oito parâmetros,
/// e uma lista de oito é onde dois deles trocam de lugar sem o compilador reparar.
struct Scene<'a> {
    shape: &'a ph2d_field_eval::hybrid::Hybrid,
    cam: &'a Orbit,
    basis: ([f32; 3], [f32; 3], [f32; 3]),
    sharp: Sharpness,
    /// ⭐⭐ **A caixa a que a marcha se prende** (W56) — `None` = a marcha de sempre, do plano da
    /// câmera até `T_MAX`.
    ///
    /// ⚠️ **É ela que torna a árvore especializada VÁLIDA em todo ponto avaliado.** Com o recorte, o
    /// raio começa na **entrada** da caixa e pára na saída dela: nenhuma amostra cai fora, e a
    /// especialização — que só vale dentro da região — nunca é perguntada onde ela mente.
    ///
    /// ⭐ E ela paga-se sozinha: os passos de aproximação em espaço vazio deixam de existir.
    clip: Option<([f32; 3], [f32; 3])>,
}

/// **O núcleo**: marcha um lote arbitrário de raios e devolve `(acertou, normal de vista)`.
///
/// Recebe posições no **plano da câmera** (unidades de mundo), e não índices de pixel, e é isso que
/// o deixa servir às duas passagens — a linha inteira e as quatro amostras espalhadas de um pixel
/// de borda. *Uma marcha, um lugar.*
fn march(scene: &Scene<'_>, screen: &[(f32, f32)]) -> (Vec<bool>, Vec<[f32; 3]>, Vec<[f32; 3]>) {
    let (right, up, fwd) = scene.basis;
    let (cam, sharp) = (scene.cam, scene.sharp);
    let n = screen.len();
    // Um avaliador POR LOTE: a `fidget` precisa de estado mutável para avaliar, e partilhá-lo entre
    // threads exigiria trava. Criar o próprio é barato e mantém a escrita disjunta, que é a
    // condição do ADR-0109.
    let mut eval = scene.shape.fork();
    let mut hit = vec![false; n];
    let mut normal = vec![[0.0f32; 3]; n];
    // ⭐ **Onde o raio parou, no MUNDO.** Ele sai de graça (a marcha já sabe o `t`), e é o que uma
    // seleção por clique precisa de saber. Devolvê-lo aqui é o que impede uma segunda marcha de
    // existir só para responder à mesma pergunta.
    let mut point = vec![[0.0f32; 3]; n];
    if n == 0 {
        return (hit, normal, point);
    }

    // ⭐ **O raio vem da CÂMERA**, e não de uma segunda cópia da conta dela. Este laço reconstruía
    // a aritmética do `Orbit::ray` com um afastamento próprio — duas respostas para *"que raio sai
    // daqui?"*, no mesmo módulo cujo doc promete que a projeção é a mesma do gizmo. Com a lente
    // convergente a direção passou a ser **por raio**, e uma das duas cópias teria ficado paralela.
    let (mut ox, mut oy, mut oz) = (vec![0.0f32; n], vec![0.0f32; n], vec![0.0f32; n]);
    let mut dir = vec![[0.0f32; 3]; n];
    for (i, &(sx, sy)) in screen.iter().enumerate() {
        let (o, d) = cam.ray_at_plane(sx, sy);
        (ox[i], oy[i], oz[i]) = (o[0], o[1], o[2]);
        dir[i] = d;
    }

    let mut t = vec![0.0f32; n];
    // ⭐ **Cada raio entra e sai da caixa** — ver [`Scene::clip`]. Sem recorte, a marcha de sempre.
    let mut t_end = vec![T_MAX; n];
    let mut alive: Vec<u32> = Vec::with_capacity(n);
    for i in 0..n {
        match scene.clip {
            None => alive.push(i as u32),
            Some((lo, hi)) => {
                let o = [ox[i], oy[i], oz[i]];
                if let Some((a, b)) = slab(o, dir[i], lo, hi) {
                    t[i] = a.max(0.0);
                    t_end[i] = b.min(T_MAX);
                    if t[i] < t_end[i] {
                        alive.push(i as u32);
                    }
                }
            }
        }
    }
    let (mut xs, mut ys, mut zs) = (Vec::new(), Vec::new(), Vec::new());
    for _ in 0..MAX_STEPS {
        if alive.is_empty() {
            break;
        }
        xs.clear();
        ys.clear();
        zs.clear();
        for &i in &alive {
            let i = i as usize;
            xs.push(ox[i] + dir[i][0] * t[i]);
            ys.push(oy[i] + dir[i][1] * t[i]);
            zs.push(oz[i] + dir[i][2] * t[i]);
        }
        let Ok(out) = eval.eval(&xs, &ys, &zs) else {
            break;
        };
        let mut next = Vec::with_capacity(alive.len());
        for (k, &i) in alive.iter().enumerate() {
            let iu = i as usize;
            let d = out[k];
            if d < sharp.hit {
                hit[iu] = true;
                continue;
            }
            t[iu] += d * SAFE_STEP;
            if t[iu] < t_end[iu] {
                next.push(i);
            }
        }
        alive = next;
    }

    // Normais por diferença central, em lote, só onde acertou.
    let idx: Vec<usize> = (0..n).filter(|i| hit[*i]).collect();
    for &i in &idx {
        point[i] = [
            ox[i] + dir[i][0] * t[i],
            oy[i] + dir[i][1] * t[i],
            oz[i] + dir[i][2] * t[i],
        ];
    }
    if idx.is_empty() {
        return (hit, normal, point);
    }
    let mut gx = Vec::with_capacity(idx.len() * 6);
    let mut gy = Vec::with_capacity(idx.len() * 6);
    let mut gz = Vec::with_capacity(idx.len() * 6);
    for &i in &idx {
        let [px, py, pz] = point[i];
        let e = sharp.normal;
        for (dx, dy, dz) in [
            (e, 0.0, 0.0),
            (-e, 0.0, 0.0),
            (0.0, e, 0.0),
            (0.0, -e, 0.0),
            (0.0, 0.0, e),
            (0.0, 0.0, -e),
        ] {
            gx.push(px + dx);
            gy.push(py + dy);
            gz.push(pz + dz);
        }
    }
    if let Ok(g) = eval.eval(&gx, &gy, &gz) {
        for (k, &i) in idx.iter().enumerate() {
            let b = k * 6;
            let world = [g[b] - g[b + 1], g[b + 2] - g[b + 3], g[b + 4] - g[b + 5]];
            let len = (world[0] * world[0] + world[1] * world[1] + world[2] * world[2]).sqrt();
            if len <= 0.0 {
                hit[i] = false;
                continue;
            }
            let nrm = [world[0] / len, world[1] / len, world[2] / len];
            // Para o espaço de VISTA — é nele que o matcap vive.
            normal[i] = [
                nrm[0] * right[0] + nrm[1] * right[1] + nrm[2] * right[2],
                nrm[0] * up[0] + nrm[1] * up[1] + nrm[2] * up[2],
                nrm[0] * fwd[0] + nrm[1] * fwd[1] + nrm[2] * fwd[2],
            ];
        }
    }
    (hit, normal, point)
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
    };
    let (hit, _, point) = march(&scene, &[screen.plane_at(px[0], px[1])]);
    hit[0].then(|| point[0])
}

/// Quantos **pixels** de borda cada lote da segunda passagem leva (× 4 amostras cada).
///
/// ⚠️ **Pequeno de propósito, e o número saiu de um erro medido.** A primeira versão usava 4096, com
/// o raciocínio de "grande o bastante para o custo de montar um `tape` desaparecer" — e a medição
/// disse que um raio de borda custava **73×** um raio comum, o que é absurdo: os dois marcham o
/// mesmo campo. O 73 não era o preço do raio, era o preço de **três lotes numa máquina de 32
/// núcleos**: a segunda passagem corria com 3 threads enquanto a primeira usava todas.
///
/// *Um número de paralelismo dimensionado por uma intuição sobre overhead, e não pela contagem de
/// lotes que ele produz, é um `for` sequencial com um `par_` na frente.*
const EDGE_CHUNK: usize = 64;

fn resample_edges(
    scene: &Scene<'_>,
    plane: Screen,
    hit: &[bool],
    normal: &[[f32; 3]],
    parallel: bool,
) -> Vec<EdgePixel> {
    let (w, h) = (plane.width() as usize, plane.height() as usize);
    let differs = |a: usize, b: usize| -> bool {
        if hit[a] != hit[b] {
            return true;
        }
        if !hit[a] {
            return false;
        }
        let (p, q) = (normal[a], normal[b]);
        p[0] * q[0] + p[1] * q[1] + p[2] * q[2] < EDGE_COS
    };

    let mut is_edge = vec![false; w * h];
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            // Só direita e baixo: a aresta é uma relação entre DOIS pixels, e marcar os dois quando
            // ela aparece cobre os quatro vizinhos sem os visitar duas vezes.
            if x + 1 < w && differs(i, i + 1) {
                is_edge[i] = true;
                is_edge[i + 1] = true;
            }
            if y + 1 < h && differs(i, i + w) {
                is_edge[i] = true;
                is_edge[i + w] = true;
            }
        }
    }

    let pixels: Vec<u32> = (0..w * h)
        .filter(|i| is_edge[*i])
        .map(|i| i as u32)
        .collect();
    if pixels.is_empty() {
        return Vec::new();
    }

    let chunk = |c: &[u32]| -> Vec<EdgePixel> {
        let mut pts = Vec::with_capacity(c.len() * 4);
        for &p in c {
            let (x, y) = ((p as usize % w) as f32, (p as usize / w) as f32);
            for (dx, dy) in ROOK {
                pts.push(plane.plane_at(x + dx, y + dy));
            }
        }
        let (hits, normals, _) = march(scene, &pts);
        c.iter()
            .enumerate()
            .map(|(k, &p)| {
                let b = k * 4;
                EdgePixel {
                    pixel: p,
                    hit: [hits[b], hits[b + 1], hits[b + 2], hits[b + 3]],
                    normal: [normals[b], normals[b + 1], normals[b + 2], normals[b + 3]],
                }
            })
            .collect()
    };

    // ⚠️ `chunks` preserva a ordem em `collect()` mesmo em paralelo — é isso que mantém `edges`
    // ordenado por `pixel` e a saída independente de como as threads se dividiram (ADR-0109).
    let out: Vec<Vec<EdgePixel>> = if parallel {
        pixels.par_chunks(EDGE_CHUNK).map(chunk).collect()
    } else {
        pixels.chunks(EDGE_CHUNK).map(chunk).collect()
    };
    out.concat()
}

#[cfg(test)]
mod tests;
