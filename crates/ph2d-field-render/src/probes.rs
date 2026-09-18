//! ⭐⭐⭐ **AS SONDAS DE IRRADIÂNCIA — a luz devolvida recolhida em pontos FIXOS do espaço, e o pixel
//! a INTERPOLAR entre elas** (o report do dono de 2026-09-17, 2.ª ronda: *«completamente
//! imprestável. Parece mais um reflexo mal feito»*).
//!
//! # ⛔⛔⛔ Porque a recolha POR PIXEL não tem cura, com a fita que o prova
//!
//! Com `N` direcções fixas, cada pixel avalia a luz devolvida **na sua própria posição**, e a
//! função *«o que a direcção `j` acerta a partir de `x`»* SALTA quando `x` anda um pixel: a
//! silhueta da peça entra e sai do raio. Aberta a soma numa linha da face do cubo da foto dele:
//!
//! ```text
//!    18 ............###########....
//!    23 .............#######.......
//!    26 ........##############.....
//!    29 ...##########..............
//!    31 ################...........
//!    34 ....###################....
//!    39 ############...............
//! ```
//!
//! Cada direcção acende um **intervalo contíguo com arestas duras** — a silhueta do vaso projectada
//! ao longo dela —, `29` direcções trocam de resposta `21` vezes em `54` pixels, e a soma de `29`
//! projecções deslocadas da mesma peça **É** uma imagem esborratada dela com arestas. *«Reflexo mal
//! feito» é a descrição exacta.* ⛔ Mais direcções não curam (a `256` a estrutura é a mesma, mais
//! fina, a `5,3×` o preço); borrar mais também não (o joelho está medido em duas passagens).
//!
//! # ⭐⭐⭐ A cura é trocar ONDE se recolhe, não QUANTO
//!
//! Uma sonda recolhe a radiância de **centenas** de direcções a partir de um ponto FIXO; o pixel
//! pergunta a irradiância às oito sondas da célula em que está e **interpola**. A discontinuidade
//! em `x` desaparece **por construção**: a posição do pixel só entra pela interpolação, que é
//! contínua, e a soma sobre centenas de direcções é suave na NORMAL (o cosseno é contínuo).
//!
//! E o preço INVERTE: `4 096` sondas × `256` direcções são `1 M` raios, contra `2 M` pixels × `24`
//! direcções úteis = `50 M` por quadro assente a `1080p` — e as sondas são de MUNDO, logo só se
//! refazem quando a cena ou a luz mudam, nunca quando a câmera roda.
//!
//! ⚠️ **É o modelo DDGI** (as sondas de irradiância dinâmicas que a indústria usa desde 2019), e o
//! nosso substrato é o que o torna barato: os raios das sondas marcham num campo de distância, e a
//! visibilidade que impede a luz de VAZAR através de uma parede sai do mesmo campo.
//!
//! # ⭐ O que a medição decidiu (`docs/Render3d/08` §14 tem as tabelas inteiras)
//!
//! - **SEM raio de visibilidade pixel→sonda**: a suave leu pior que a binária, e a binária
//!   **piorava** a única faixa onde age (o pé das paredes, `40,7 → 35,2 %` sem) — ver a recusa no
//!   doc da [`gather_probes_por`]. O que barra as sondas erradas é a bandeira «dentro» e o peso
//!   «está à frente».
//! - **O borrão de duas passagens fica** ([`crate::BOUNCE_BLUR_PASSES`]): é ele que apaga os vincos
//!   da interpolação trilinear (face `0,32 → 0,13`, o nível do céu).
//! - **Nove coeficientes esféricos** ([`SH_COEFFS`]) custam `< 1 %` contra a soma das `256`
//!   direcções, em todas as células — e são o que faz a recolha por pixel caber na placa.
//! - **A grelha** ([`PROBE_GRID`]): a escada `16³ → 24³ → 32³` não saturou; o que a segura é a
//!   referência de CPU, não o dispositivo.
//! - **O resíduo que fica é um DESVIO DE NÍVEL de `~6 %` para cima**, suave: uma sonda a um passo
//!   da superfície vê mais céu do que o ponto vê. E o interior fino do vaso empata com o por-pixel
//!   a `256` direcções em vez de o bater — a cavidade tem poucas sondas de largura.
//!
//! ⚠️ A régua que separa as duas leis é a [`crate::banda::estrutura`] (média frequência do resíduo):
//! face `0,106 → 0,035`. Os terraços a um pixel **não** as separavam (`0,17` contra `0,13`).

use crate::march::{self, Scene};
use crate::shade_render::ViewBasis;
use crate::{Gbuffer, Orbit, PointLamp, Surfaces};
use ph2d_field::FieldDoc;
use ph2d_field_eval::hybrid::Registry;

/// ⭐ **Quantas direcções cada sonda recolhe** — a esfera INTEIRA, porque uma sonda não tem normal.
///
/// `256` é onde a projecção em nove coeficientes deixa de mudar (`512` mede o mesmo, ver o doc de
/// `docs/Render3d/08` §14), e o preço é de assar, não de pintar.
pub const PROBE_DIRS: u32 = 256;

/// ⭐⭐ **Quantas sondas por aresta da grelha** — a grelha cobre a bola que envolve a peça.
///
/// # ⚠️ Medido, e NÃO saturado
///
/// Erro contra a recolha convergida (`1024` direcções por pixel), com o borrão de duas passagens:
///
/// | grelha | face do cubo | interior do vaso | Cornell (sangramento, verdade `+0,040`) |
/// |---:|---:|---:|---:|
/// | por pixel, `48` dir | `16,7 %` | `19,1 %` | `+0,073` |
/// | `16³` | `20,4 %` | `19,8 %` | `+0,032` |
/// | `24³` | `16,1 %` | `14,0 %` | `+0,037` |
/// | **`32³`** | **`12,8 %`** | **`11,1 %`** | **`+0,042`** |
///
/// A escada continua a descer a `32³`. ⚠️ **O recurso que a segura aqui é a REFERÊNCIA de CPU**
/// (a `32³` assar custa `5 s` na CPU, e cada gate de paridade assa uma vez) — não é o tecto do
/// dispositivo, onde `32³ × 256` são `8,4 M` raios, da ordem de `0,7 ms`. *Quem subir isto sobe
/// com a tabela ao lado e com o relógio dos gates medido.*
pub const PROBE_GRID: usize = 32;

/// A folga da grelha à volta da bola — as sondas da borda ficam FORA da peça.
pub const PROBE_MARGIN: f32 = 1.05;

// ⛔ **RECUSA MEDIDA (2026-09-17): erguer a consulta ao longo da normal (o «normal offset» das
// sondas dinâmicas, `0,25` do passo) não é observável aqui.** Medido por mutação nos dois gates de
// valor contínuo: Cornell `+0,0418/−0,0729` com e sem; a face do cubo `0,0403 → 0,0404`. O que ele
// compra no DDGI — afastar a consulta das sondas atrás da superfície — o peso «está à frente»
// (`((cos+1)/2)²`) já compra sozinho. *Uma constante que nenhuma mutação mata é comentário com
// sintaxe de código*, e saiu.

/// ⭐ A GEOMETRIA da grelha para uma bola — o canto e o passo. UMA função, lida pela CPU e
/// transcrita no shader linha a linha (a paridade mede-o).
#[must_use]
pub fn grid_geometry(bola: &ph2d_field_eval::bounds::Ball, n: usize) -> ([f32; 3], f32) {
    let raio = bola.radius.max(1e-3) * PROBE_MARGIN;
    #[allow(clippy::cast_precision_loss)]
    let step = 2.0 * raio / (n.max(2) - 1) as f32;
    (
        [
            bola.center[0] - raio,
            bola.center[1] - raio,
            bola.center[2] - raio,
        ],
        step,
    )
}

/// A grelha de sondas assada para uma cena e uma luz.
#[derive(Clone, Debug)]
pub struct ProbeGrid {
    /// O canto da grelha, em mundo.
    pub origin: [f32; 3],
    /// A distância entre sondas vizinhas.
    pub step: f32,
    /// Sondas por aresta.
    pub n: usize,
    /// As direcções recolhidas (as mesmas para todas as sondas).
    pub dirs: Vec<[f32; 3]>,
    /// `n³ × dirs`: a radiância fosca que volta por cada direcção de cada sonda.
    pub radiance: Vec<[f32; 3]>,
    /// `n³`: os [`SH_COEFFS`] coeficientes esféricos (`l ≤ 2`) da radiância de cada sonda, por canal
    /// — a forma COMPACTA que a placa lê. Ver [`ProbeGrid::irradiancia`].
    pub sh: Vec<[[f32; 3]; SH_COEFFS]>,
    /// `n³`: a sonda está DENTRO da peça (ou colada a ela) e não conta.
    pub inside: Vec<bool>,
}

/// ⭐ **Quantos coeficientes esféricos por sonda** — `l ≤ 2`, os nove de Ramamoorthi–Hanrahan: a
/// irradiância de uma superfície difusa fica capturada a `< 3 %` com eles, e a avaliação por pixel
/// custa nove multiplicações por canal em vez de uma soma sobre centenas de direcções.
pub const SH_COEFFS: usize = 9;

/// A base real de harmónicas esféricas até `l = 2`, avaliada na direcção `d` (unitária).
#[must_use]
pub fn sh_basis(d: [f32; 3]) -> [f32; SH_COEFFS] {
    let [x, y, z] = d;
    [
        0.282_095,
        0.488_603 * y,
        0.488_603 * z,
        0.488_603 * x,
        1.092_548 * x * y,
        1.092_548 * y * z,
        0.315_392 * (3.0 * z * z - 1.0),
        1.092_548 * x * z,
        0.546_274 * (x * x - y * y),
    ]
}

/// ⭐ **A convolução com o cosseno, por banda** — `Â_l / π`: `1`, `2/3`, `1/4`. Com ela a soma dos
/// coeficientes devolve a **média da radiância pesada pelo cosseno**, que é a convenção do
/// ricochete desta casa (uma radiância uniforme `L` devolve exactamente `L`).
pub const SH_COSSENO: [f32; SH_COEFFS] = [
    1.0,
    2.0 / 3.0,
    2.0 / 3.0,
    2.0 / 3.0,
    0.25,
    0.25,
    0.25,
    0.25,
    0.25,
];

/// A irradiância (na convenção do ricochete) que os coeficientes `sh` entregam à normal `n` —
/// a MESMA aritmética que o shader corre, e o que a paridade CPU↔dispositivo mede.
#[must_use]
pub fn sh_irradiancia(sh: &[[f32; 3]; SH_COEFFS], n: [f32; 3]) -> [f32; 3] {
    let y = sh_basis(n);
    let mut e = [0.0f32; 3];
    for (l, c) in sh.iter().enumerate() {
        let a = SH_COSSENO[l] * y[l];
        e = [e[0] + a * c[0], e[1] + a * c[1], e[2] + a * c[2]];
    }
    // ⚠️ Nove coeficientes podem oscilar abaixo de zero ao lado de uma fonte muito contrastada;
    // luz negativa não existe, e o grampo é a cura de sempre das sondas em harmónicas.
    [e[0].max(0.0), e[1].max(0.0), e[2].max(0.0)]
}

impl ProbeGrid {
    fn idx(&self, x: usize, y: usize, z: usize) -> usize {
        (z * self.n + y) * self.n + x
    }

    fn pos(&self, x: usize, y: usize, z: usize) -> [f32; 3] {
        #[allow(clippy::cast_precision_loss)]
        [
            self.origin[0] + x as f32 * self.step,
            self.origin[1] + y as f32 * self.step,
            self.origin[2] + z as f32 * self.step,
        ]
    }

    /// A irradiância que a sonda `k` entrega a uma superfície de normal `n`, pelos coeficientes.
    fn irradiancia(&self, k: usize, n: [f32; 3]) -> [f32; 3] {
        sh_irradiancia(&self.sh[k], n)
    }

    /// A MESMA pergunta, somando as direcções uma a uma — a régua contra a qual os coeficientes
    /// foram medidos (ver a sonda `sonda_as_sondas_contra_o_por_pixel`).
    #[must_use]
    pub fn irradiancia_directa(&self, k: usize, n: [f32; 3]) -> [f32; 3] {
        let base = k * self.dirs.len();
        let (mut soma, mut peso) = ([0.0f32; 3], 0.0f32);
        for (j, d) in self.dirs.iter().enumerate() {
            let c = n[0] * d[0] + n[1] * d[1] + n[2] * d[2];
            if c <= 0.0 {
                continue;
            }
            let l = self.radiance[base + j];
            soma = [soma[0] + c * l[0], soma[1] + c * l[1], soma[2] + c * l[2]];
            peso += c;
        }
        if peso <= 0.0 {
            return [0.0; 3];
        }
        [soma[0] / peso, soma[1] / peso, soma[2] / peso]
    }
}

pub(crate) fn scene_solta<'a>(
    shape: &'a ph2d_field_eval::hybrid::Hybrid,
    doc: &FieldDoc,
    reg: &Registry,
    cam: &'a Orbit,
    lado_px: usize,
) -> Scene<'a> {
    Scene {
        shape,
        cam,
        basis: cam.basis(),
        // ⚠️ A tolerância de acerto é a do QUADRO (`w.min(h)`), a mesma que o dispositivo põe em
        // `hit_eps` — a 1.ª redacção fixava `256` e a paridade mediria dois epsilons.
        sharp: crate::Sharpness::for_frame(cam.half_extent, lado_px),
        clip: None,
        step: ph2d_field_eval::safe_march_step(doc),
        shrink: ph2d_field_eval::field_shrink(doc, reg),
        stencil: crate::Stencil::Tetra4,
    }
}

/// ⭐⭐⭐ **ASSAR as sondas** — cada uma recolhe [`PROBE_DIRS`] direcções pela MESMA lei que o pixel
/// recolhia ([`crate::bounce::radiancia_devolvida`]).
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn bake_probes(
    doc: &FieldDoc,
    reg: &Registry,
    cam: &Orbit,
    surfaces: &Surfaces<'_>,
    lampadas: &[PointLamp],
    n: usize,
    dirs: u32,
    lado_px: usize,
) -> ProbeGrid {
    let bola = ph2d_field_eval::bounds::bounding_ball(doc, reg)
        .unwrap_or(ph2d_field_eval::bounds::Ball::EMPTY);
    let raio = bola.radius.max(1e-3) * PROBE_MARGIN;
    let (origin, step) = grid_geometry(&bola, n);
    let dirs_v: Vec<[f32; 3]> = (0..dirs)
        .map(|k| crate::occlusion::cone_dir(k, dirs))
        .collect();
    let mut grid = ProbeGrid {
        origin,
        step,
        n,
        dirs: dirs_v,
        radiance: vec![[0.0; 3]; n * n * n * dirs as usize],
        sh: vec![[[0.0; 3]; SH_COEFFS]; n * n * n],
        inside: vec![false; n * n * n],
    };
    if lampadas.is_empty() || surfaces.all.is_empty() || n < 2 {
        return grid;
    }
    let shape = ph2d_field_eval::hybrid::Hybrid::new(doc, reg);
    let scene = scene_solta(&shape, doc, reg, cam, lado_px);
    let base = ViewBasis::of(cam);
    let lift = scene.sharp.hit * march::BIAS;

    // ── 1. quem está dentro ───────────────────────────────────────────────────────────────────
    let (mut xs, mut ys, mut zs) = (Vec::new(), Vec::new(), Vec::new());
    for z in 0..n {
        for y in 0..n {
            for x in 0..n {
                let p = grid.pos(x, y, z);
                xs.push(p[0]);
                ys.push(p[1]);
                zs.push(p[2]);
            }
        }
    }
    let mut eval = shape.fork();
    if let Ok(d) = eval.eval(&xs, &ys, &zs) {
        for (k, dk) in d.iter().enumerate() {
            // ⚠️ Colada à superfície conta como dentro: a marcha acertaria em todas as direcções no
            // primeiro passo, e a sonda leria a própria parede.
            grid.inside[k] = *dk < scene.sharp.hit * 4.0;
        }
    }

    // ── 2. os raios, um lote só ───────────────────────────────────────────────────────────────
    let mut origens: Vec<[f32; 3]> = Vec::new();
    let mut raios: Vec<[f32; 3]> = Vec::new();
    let mut quais: Vec<usize> = Vec::new();
    for k in 0..n * n * n {
        if grid.inside[k] {
            continue;
        }
        let p = [xs[k], ys[k], zs[k]];
        for d in &grid.dirs {
            origens.push(p);
            raios.push(*d);
            quais.push(k);
        }
    }
    // Até onde um raio vai: a diagonal da caixa da grelha.
    let alcance = 2.0 * raio * 3.0f32.sqrt();
    let sai = crate::bounce::radiancia_devolvida(
        &scene, &base, lift, alcance, &origens, &raios, surfaces, lampadas,
    );
    let nd = grid.dirs.len();
    for (r, s3) in sai.iter().enumerate() {
        grid.radiance[quais[r] * nd + r % nd] = *s3;
    }
    // ── 3. a projecção em harmónicas: `c_lm = (4π/N) Σ_j L_j Y_lm(d_j)` ─────────────────────────
    #[allow(clippy::cast_precision_loss)]
    let peso = 4.0 * std::f32::consts::PI / nd as f32;
    for k in 0..n * n * n {
        if grid.inside[k] {
            continue;
        }
        let mut c = [[0.0f32; 3]; SH_COEFFS];
        for (j, d) in grid.dirs.iter().enumerate() {
            let l = grid.radiance[k * nd + j];
            if l == [0.0; 3] {
                continue;
            }
            let y = sh_basis(*d);
            for (m, cm) in c.iter_mut().enumerate() {
                let a = peso * y[m];
                *cm = [cm[0] + a * l[0], cm[1] + a * l[1], cm[2] + a * l[2]];
            }
        }
        grid.sh[k] = c;
    }
    grid
}

/// ⭐⭐⭐ **RECOLHER nos pixels** — a irradiância das oito sondas da célula, pesada por trilinear ×
/// «está à frente da superfície» × «vê-se daqui», que é o que impede a luz de vazar por uma parede.
#[must_use]
pub fn gather_probes(
    doc: &FieldDoc,
    reg: &Registry,
    cam: &Orbit,
    g: &Gbuffer,
    grid: &ProbeGrid,
) -> Vec<[f32; 3]> {
    gather_probes_por(doc, reg, cam, g, grid, false)
}

/// A mesma, com `directa = true` a somar as direcções uma a uma em vez de ler os coeficientes —
/// a porta pela qual a sonda mediu o que os nove coeficientes custam.
///
/// # ⛔⛔ RECUSA MEDIDA (2026-09-17): o raio de VISIBILIDADE pixel→sonda saiu
///
/// A recolha teve um raio binário por candidata (*«vê-se daqui?»*, a marcha no campo) — o que as
/// sondas dinâmicas fazem com um teste de Chebyshev. Medido de três maneiras:
///
/// - **suave** (dureza `8`/`2`, o cone da sombra) lia PIOR que a binária: `0,67`/`0,76` de
///   terraços na face do cubo contra `0,32`;
/// - **binária** contra **nenhuma**: nenhum gate a vê — paridade `100,000 %` com e sem, Cornell
///   `−0,0729 → −0,0734`, face `0,0403 → 0,0409`;
/// - e na única faixa onde ela AGE — o chão de Cornell a menos de um passo das paredes — ela
///   **piora**: erro `40,7 %` com, `35,2 %` sem (miolo `29,9` contra `30,0`). A sonda escura do
///   outro lado da parede aproximava, por acaso, a oclusão que a própria parede faz ao pé dela.
///
/// ⇒ sem visibilidade, e sem as oito marchas por pixel na placa. O que fica a barrar as sondas
/// erradas é a bandeira «dentro» ([`ProbeGrid::inside`], observável: a mutação sangra a
/// `99,825 %`) e o peso «está à frente» (`((cos+1)/2)²`). ⏳ O erro de `~30 %` no pé das paredes é
/// o desvio de nível das sondas (a sonda vê mais do que o ponto vê), nomeado em
/// `docs/Render3d/08` §14.6 — a cura publicada é a oclusão LOCAL, não um raio até à sonda.
#[must_use]
pub fn gather_probes_por(
    _doc: &FieldDoc,
    _reg: &Registry,
    cam: &Orbit,
    g: &Gbuffer,
    grid: &ProbeGrid,
    directa: bool,
) -> Vec<[f32; 3]> {
    let pixels = g.hit.len();
    let mut out = vec![[0.0f32; 3]; pixels];
    if grid.n < 2 || grid.radiance.is_empty() {
        return out;
    }
    let base = ViewBasis::of(cam);
    // ⚠️ A cerca «a sonda em cima do ponto não conta» usa a tolerância do QUADRO, a mesma do
    // dispositivo (`hit_eps · 4`).
    let lift = crate::Sharpness::for_frame(cam.half_extent, g.width.min(g.height) as usize).hit
        * march::BIAS;

    for (i, o) in out.iter_mut().enumerate() {
        if !g.hit[i] {
            continue;
        }
        let nrm = base.view_to_world(g.normal[i]);
        *o = consulta_sondas(grid, g.point[i], nrm, lift, directa);
    }
    out
}

/// ⭐⭐⭐ **A CONSULTA, num ponto qualquer** — as oito sondas da célula, pesadas por trilinear ×
/// «está à frente da superfície», normalizadas pelo peso que de facto entrou.
///
/// ⚠️⚠️ **Ela é UMA função porque a lei é UMA, e desde 2026-09-17 ela tem DOIS consumidores** — o
/// pixel que acerta na peça ([`gather_probes_por`]) e o **pixel de CHÃO** ([`crate::ground`], que
/// consulta com a normal `[0,1,0]`). Duas cópias divergiriam no dia em que alguém mexesse numa
/// delas, e a metade que o artista vê primeiro é a que envelhece.
///
/// ⚠️ **A consulta AGARRA-SE À BORDA da grelha** (o `clamp` do `u`), e isso é inofensivo numa
/// superfície fechada — todo ponto da peça está dentro da caixa — e **mentiria num plano infinito**:
/// ver a cerca medida em [`crate::ground::bounce_reach`], que é quem decide onde o chão deixa de
/// perguntar.
pub(crate) fn consulta_sondas(
    grid: &ProbeGrid,
    p: [f32; 3],
    nrm: [f32; 3],
    lift: f32,
    directa: bool,
) -> [f32; 3] {
    if grid.n < 2 || grid.radiance.is_empty() {
        return [0.0; 3];
    }
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    // A célula é a do próprio ponto (ver a recusa medida do «erguer» no topo do módulo).
    let u = [
        ((p[0] - grid.origin[0]) / grid.step).clamp(0.0, (grid.n - 1) as f32),
        ((p[1] - grid.origin[1]) / grid.step).clamp(0.0, (grid.n - 1) as f32),
        ((p[2] - grid.origin[2]) / grid.step).clamp(0.0, (grid.n - 1) as f32),
    ];
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let c0 = [
        (u[0].floor() as usize).min(grid.n - 2),
        (u[1].floor() as usize).min(grid.n - 2),
        (u[2].floor() as usize).min(grid.n - 2),
    ];
    #[allow(clippy::cast_precision_loss)]
    let f = [
        (u[0] - c0[0] as f32).clamp(0.0, 1.0),
        (u[1] - c0[1] as f32).clamp(0.0, 1.0),
        (u[2] - c0[2] as f32).clamp(0.0, 1.0),
    ];
    let (mut soma, mut total) = ([0.0f32; 3], 0.0f32);
    for dz in 0..2 {
        for dy in 0..2 {
            for dx in 0..2 {
                let (x, y, z) = (c0[0] + dx, c0[1] + dy, c0[2] + dz);
                let k = grid.idx(x, y, z);
                if grid.inside[k] {
                    continue;
                }
                let tri = (if dx == 1 { f[0] } else { 1.0 - f[0] })
                    * (if dy == 1 { f[1] } else { 1.0 - f[1] })
                    * (if dz == 1 { f[2] } else { 1.0 - f[2] });
                let sp = grid.pos(x, y, z);
                let para = [sp[0] - p[0], sp[1] - p[1], sp[2] - p[2]];
                let dist = (para[0] * para[0] + para[1] * para[1] + para[2] * para[2]).sqrt();
                if dist <= lift {
                    continue;
                }
                let dir = [para[0] / dist, para[1] / dist, para[2] / dist];
                // ⭐ «Está à frente»: uma sonda atrás da superfície vê o outro lado dela.
                let cos = nrm[0] * dir[0] + nrm[1] * dir[1] + nrm[2] * dir[2];
                let frente = ((cos + 1.0) * 0.5).powi(2);
                let peso = tri * frente;
                if peso <= 1e-6 {
                    continue;
                }
                let e = if directa {
                    grid.irradiancia_directa(k, nrm)
                } else {
                    grid.irradiancia(k, nrm)
                };
                soma = [
                    soma[0] + peso * e[0],
                    soma[1] + peso * e[1],
                    soma[2] + peso * e[2],
                ];
                total += peso;
            }
        }
    }
    if total > 0.0 {
        [soma[0] / total, soma[1] / total, soma[2] / total]
    } else {
        [0.0; 3]
    }
}

/// ⭐⭐⭐ **A LEI DO PRODUTO numa chamada** — as sondas de [`PROBE_GRID`]³ × [`PROBE_DIRS`], assadas
/// para esta cena e esta luz, recolhidas nos pixels de `g`. É a REFERÊNCIA de CPU contra a qual o
/// pintor do dispositivo é medido, e quem a chama borra o resultado com o [`crate::blur_bounce`]
/// como a metade do céu.
#[must_use]
pub fn probe_bounce(
    doc: &FieldDoc,
    reg: &Registry,
    cam: &Orbit,
    g: &Gbuffer,
    surfaces: &Surfaces<'_>,
    lampadas: &[PointLamp],
) -> Vec<[f32; 3]> {
    let grid = bake_probes(
        doc,
        reg,
        cam,
        surfaces,
        lampadas,
        PROBE_GRID,
        PROBE_DIRS,
        g.width.min(g.height) as usize,
    );
    gather_probes(doc, reg, cam, g, &grid)
}
