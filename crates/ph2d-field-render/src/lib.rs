//! `ph2d-field-render` — **o traçador**: marcha raios contra o campo e devolve o que a superfície
//! de facto é ([ADR-0161] §2).
//!
//! # Por que a tela não passa pela malha
//!
//! Medido na W0 (`docs/3DModeling/01_resultados_spike.md` §1c): traçando o campo, a quina do cubo
//! sai como uma navalha e o filete sai liso; a **mesma** cena extraída em malha serrilha. A
//! geometria estava certa e o defeito era inteiramente da extração. Deixar a malha desenhar a tela
//! seria deixar **o caminho pior definir o teto do que se vê** — o que o [`CLAUDE.md §0`] proíbe.
//!
//! # O que sai daqui é GEOMETRIA, não cor
//!
//! [`trace`] devolve um G-buffer: máscara, **normal em espaço de vista** e profundidade. Nenhuma
//! decisão de cor mora nele. Quem quiser pixels passa um [`Matcap`] a [`shade`].
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
use ph2d_field_eval::Engine;
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
struct Sharpness {
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

/// A câmera. **Ortográfica**, e a orientação é um **quaternion**.
///
/// # ⭐ Por que não é `yaw`/`pitch`
///
/// Uma câmera de dois ângulos tem **polos por construção**: a elevação satura em ±90°, e a partir
/// dali arrastar na vertical não faz nada. Com o enquadramento inicial já a 30° de cima, meio
/// centímetro de rato para baixo bate na parede — e o que o artista vê é *"só roda para um lado"*
/// (Enio, 2026-08-19). A câmera da casa (`ph2d_mesh_render::camera`) tem exatamente o mesmo teto,
/// e **prende-o com um `clamp`**.
///
/// Um `clamp` é o remédio para o sintoma. A causa é a **representação**: dois ângulos não conseguem
/// exprimir uma orientação livre, então nenhum número melhor a devolve. Guardando a orientação
/// inteira, a rotação passa a ser *uma* composição de quaternions — sem polo, sem `clamp`, sem caso
/// especial, e sem o eixo vertical do mundo a decidir o que a mão pode fazer.
///
/// ⚠️ O preço é real e está aceite: **o horizonte deixa de ser fixo**. Uma câmera de dois ângulos
/// nunca inclina; esta inclina, porque é isso que *rotação livre* significa. A volta é
/// [`Orbit::from_yaw_pitch`], que é o que a tecla de repor a vista chama.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Orbit {
    /// A orientação, como quaternion `(x, y, z, w)`: leva os eixos **locais** da câmera para o
    /// mundo.
    pub rotation: [f32; 4],
    /// Quantas unidades de mundo cabem em meia altura de tela. Menor = mais perto.
    pub half_extent: f32,
    /// O ponto que fica no centro do quadro.
    pub target: [f32; 3],
}

impl Default for Orbit {
    fn default() -> Self {
        // Três-quartos, ligeiramente por cima: o ângulo em que uma aresta viva e um filete se
        // distinguem sem ambiguidade (escolhido na W0, ao olhar as imagens).
        Self {
            rotation: Self::from_yaw_pitch(0.72, 0.52).rotation,
            half_extent: 0.8,
            target: [0.0; 3],
        }
    }
}

impl Orbit {
    /// A orientação que os dois ângulos de uma câmera de prato giratório dariam.
    ///
    /// Continua a existir por duas razões, e nenhuma é nostalgia: é como se escreve um
    /// **enquadramento nomeado** (o inicial, a vista de frente, a de topo), e é o que **repõe** a
    /// vista depois de a rotação livre a ter inclinado.
    #[must_use]
    pub fn from_yaw_pitch(yaw: f32, pitch: f32) -> Self {
        // `R = Ry(yaw) · Rx(−pitch)` — a composição que reproduz exatamente a base antiga
        // (`fwd = (cos p·sin y, sin p, cos p·cos y)`), verificada por gate.
        let (sy, cy) = (yaw * 0.5).sin_cos();
        let (sp, cp) = (-pitch * 0.5).sin_cos();
        Self {
            rotation: quat_mul([0.0, sy, 0.0, cy], [sp, 0.0, 0.0, cp]),
            half_extent: 0.8,
            target: [0.0; 3],
        }
    }

    /// A base ortonormal da câmera: `(direita, cima, para-o-observador)`.
    ///
    /// ⚠️ **Projeção ortográfica**, e isso é uma escolha com data: é a que a W0 validou, e é a que
    /// o matcap pressupõe (`ph2d-mesh-render::matcap` amostra pela normal de vista, com a vista em
    /// `(0,0,1)`). Perspectiva é item ABERTO — ela muda o *feel* de um modelador e merece a sua
    /// própria comparação lado a lado, não uma troca silenciosa.
    ///
    /// ⚠️ A trigonometria daqui **não** fere o HR-5: a câmera é estado de VISTA — não entra no
    /// documento salvo, não entra no undo e não entra em hash de replay nenhum.
    #[must_use]
    pub fn basis(&self) -> ([f32; 3], [f32; 3], [f32; 3]) {
        let q = self.rotation;
        (
            quat_rotate(q, [1.0, 0.0, 0.0]),
            quat_rotate(q, [0.0, 1.0, 0.0]),
            quat_rotate(q, [0.0, 0.0, 1.0]),
        )
    }

    /// ⭐ **Rotação LIVRE**: gira em torno de um eixo dado nas coordenadas da **própria câmera**.
    ///
    /// É a composição pela direita (`q ⊗ Δ`), e é ela que faz a rotação ser local — o eixo é o que
    /// o gesto nomeia na tela, e não um eixo do mundo. Daí não haver polo: nenhum eixo do mundo
    /// participa da conta.
    pub fn turn_local(&mut self, axis: [f32; 3], angle: f32) {
        self.rotation = quat_normalize(quat_mul(self.rotation, quat_axis_angle(axis, angle)));
    }

    /// Gira em torno de um eixo do **mundo** (composição pela esquerda) — o prato giratório.
    pub fn turn_world(&mut self, axis: [f32; 3], angle: f32) {
        self.rotation = quat_normalize(quat_mul(quat_axis_angle(axis, angle), self.rotation));
    }
}

/// `a ⊗ b` — aplicar `b` **depois** de `a` no referencial de `a`.
fn quat_mul(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let ([ax, ay, az, aw], [bx, by, bz, bw]) = (a, b);
    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ]
}

fn quat_axis_angle(axis: [f32; 3], angle: f32) -> [f32; 4] {
    let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if len <= 0.0 || !len.is_finite() {
        return [0.0, 0.0, 0.0, 1.0];
    }
    let (s, c) = (angle * 0.5).sin_cos();
    [axis[0] / len * s, axis[1] / len * s, axis[2] / len * s, c]
}

/// ⚠️ **Re-normalizar a cada giro não é zelo.** Uma rotação livre é uma composição *acumulada*: um
/// arrasto longo são centenas de multiplicações, e o erro de `f32` faz a norma derivar. Um
/// quaternion que deixa de ser unitário deixa de ser uma rotação — ele passa a **escalar** a peça,
/// e o sintoma é a forma a encolher devagar enquanto se gira.
fn quat_normalize(q: [f32; 4]) -> [f32; 4] {
    let n = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if n <= 0.0 || !n.is_finite() {
        return [0.0, 0.0, 0.0, 1.0];
    }
    [q[0] / n, q[1] / n, q[2] / n, q[3] / n]
}

fn quat_rotate(q: [f32; 4], v: [f32; 3]) -> [f32; 3] {
    // `v + 2·w·(u×v) + 2·u×(u×v)`, com `u` a parte vetorial — a forma sem construir a matriz.
    let u = [q[0], q[1], q[2]];
    let cross = |a: [f32; 3], b: [f32; 3]| {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    let t = cross(u, v);
    let tt = cross(u, t);
    [
        v[0] + 2.0 * (q[3] * t[0] + tt[0]),
        v[1] + 2.0 * (q[3] * t[1] + tt[1]),
        v[2] + 2.0 * (q[3] * t[2] + tt[2]),
    ]
}

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
pub fn trace(doc: &FieldDoc, cam: &Orbit, width: u32, height: u32) -> Gbuffer {
    trace_with(doc, cam, width, height, true, true)
}

/// Igual a [`trace`], com o paralelismo sob controle — é o que o gate de byte-identidade dirige.
#[must_use]
pub fn trace_with_threads(
    doc: &FieldDoc,
    cam: &Orbit,
    width: u32,
    height: u32,
    parallel: bool,
) -> Gbuffer {
    trace_with(doc, cam, width, height, parallel, true)
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
    cam: &Orbit,
    width: u32,
    height: u32,
    parallel: bool,
    antialias: bool,
) -> Gbuffer {
    let tree = ph2d_field_eval::compile(doc);
    let shape = Engine::from(tree);
    let basis = cam.basis();
    let (w, h) = (width as usize, height as usize);
    let plane = Plane::new(w, h, cam.half_extent);
    let sharp = Sharpness::for_frame(cam.half_extent, w.min(h));
    let scene = Scene {
        shape: &shape,
        cam,
        basis,
        sharp,
    };

    // Passo 1: um raio por pixel, uma fatia por linha.
    let row = |y: usize| -> (Vec<bool>, Vec<[f32; 3]>) {
        let pts: Vec<(f32, f32)> = (0..w)
            .map(|x| plane.at(x as f32 + 0.5, y as f32 + 0.5))
            .collect();
        march(&scene, &pts)
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
    let edges = if antialias {
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

/// **Tudo o que uma marcha precisa de saber, e que não muda entre lotes.**
///
/// Os quatro viajam sempre juntos — a árvore compilada, a câmera, a base dela e as tolerâncias do
/// quadro. Passá-los soltos era o que fazia as duas funções abaixo crescerem para oito parâmetros,
/// e uma lista de oito é onde dois deles trocam de lugar sem o compilador reparar.
struct Scene<'a> {
    shape: &'a Engine,
    cam: &'a Orbit,
    basis: ([f32; 3], [f32; 3], [f32; 3]),
    sharp: Sharpness,
}

/// **O núcleo**: marcha um lote arbitrário de raios e devolve `(acertou, normal de vista)`.
///
/// Recebe posições no **plano da câmera** (unidades de mundo), e não índices de pixel, e é isso que
/// o deixa servir às duas passagens — a linha inteira e as quatro amostras espalhadas de um pixel
/// de borda. *Uma marcha, um lugar.*
fn march(scene: &Scene<'_>, screen: &[(f32, f32)]) -> (Vec<bool>, Vec<[f32; 3]>) {
    let (right, up, fwd) = scene.basis;
    let (shape, cam, sharp) = (scene.shape, scene.cam, scene.sharp);
    let n = screen.len();
    // Um avaliador POR LOTE: a `fidget` precisa de estado mutável para avaliar, e partilhá-lo entre
    // threads exigiria trava. Criar o próprio é barato e mantém a escrita disjunta, que é a
    // condição do ADR-0109.
    let tape = shape.float_slice_tape(Default::default());
    let mut eval = Engine::new_float_slice_eval();
    let mut hit = vec![false; n];
    let mut normal = vec![[0.0f32; 3]; n];
    if n == 0 {
        return (hit, normal);
    }

    let (mut ox, mut oy, mut oz) = (vec![0.0f32; n], vec![0.0f32; n], vec![0.0f32; n]);
    for (i, &(sx, sy)) in screen.iter().enumerate() {
        const START: f32 = 4.0;
        ox[i] = cam.target[0] + right[0] * sx + up[0] * sy + fwd[0] * START;
        oy[i] = cam.target[1] + right[1] * sx + up[1] * sy + fwd[1] * START;
        oz[i] = cam.target[2] + right[2] * sx + up[2] * sy + fwd[2] * START;
    }
    let dir = [-fwd[0], -fwd[1], -fwd[2]];

    let mut t = vec![0.0f32; n];
    let mut alive: Vec<u32> = (0..n as u32).collect();
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
            xs.push(ox[i] + dir[0] * t[i]);
            ys.push(oy[i] + dir[1] * t[i]);
            zs.push(oz[i] + dir[2] * t[i]);
        }
        let Ok(out) = eval.eval(&tape, &xs, &ys, &zs) else {
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
            if t[iu] < T_MAX {
                next.push(i);
            }
        }
        alive = next;
    }

    // Normais por diferença central, em lote, só onde acertou.
    let idx: Vec<usize> = (0..n).filter(|i| hit[*i]).collect();
    if idx.is_empty() {
        return (hit, normal);
    }
    let mut gx = Vec::with_capacity(idx.len() * 6);
    let mut gy = Vec::with_capacity(idx.len() * 6);
    let mut gz = Vec::with_capacity(idx.len() * 6);
    for &i in &idx {
        let (px, py, pz) = (
            ox[i] + dir[0] * t[i],
            oy[i] + dir[1] * t[i],
            oz[i] + dir[2] * t[i],
        );
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
    if let Ok(g) = eval.eval(&tape, &gx, &gy, &gz) {
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
    (hit, normal)
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

/// A conversão pixel → plano da câmera, num tipo `Copy` em vez de num fecho.
///
/// ⚠️ Não é arrumação: um `dyn Fn` não é `Sync`, e a segunda passagem é paralela. Um fecho aqui
/// **não compila** — e a alternativa (repetir a conta nos dois sítios) seria a segunda resposta à
/// mesma pergunta, na função cujo trabalho inteiro é saber onde cai uma amostra.
#[derive(Clone, Copy)]
struct Plane {
    w: f32,
    h: f32,
    /// Metade do lado menor, em pixels — é ele que fixa a escala, para o quadro não deformar.
    half: f32,
    half_extent: f32,
}

impl Plane {
    fn new(w: usize, h: usize, half_extent: f32) -> Self {
        Self {
            w: w as f32,
            h: h as f32,
            half: (w.min(h) as f32) * 0.5,
            half_extent,
        }
    }

    fn at(self, x: f32, y: f32) -> (f32, f32) {
        (
            (x - self.w * 0.5) / self.half * self.half_extent,
            -(y - self.h * 0.5) / self.half * self.half_extent,
        )
    }
}

fn resample_edges(
    scene: &Scene<'_>,
    plane: Plane,
    hit: &[bool],
    normal: &[[f32; 3]],
    parallel: bool,
) -> Vec<EdgePixel> {
    let (w, h) = (plane.w as usize, plane.h as usize);
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
                pts.push(plane.at(x + dx, y + dy));
            }
        }
        let (hits, normals) = march(scene, &pts);
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

/// Os texels de um matcap, **em linear**, lado × lado, RGB.
///
/// ⚠️ Quem os fornece é o chamador — ver a nota do `Cargo.toml` sobre não arrastar o `wgpu` para
/// dentro de um traçador de CPU.
pub struct Matcap<'a> {
    pub side: u32,
    /// `side * side * 3` valores lineares.
    pub rgb_linear: &'a [f32],
}

impl Matcap<'_> {
    /// Amostra **bilinear**, em linear.
    ///
    /// ⚠️ Bilinear e não vizinho-mais-próximo: a normal varia **continuamente** sobre uma superfície
    /// curva, e o vizinho-mais-próximo transforma essa rampa contínua nos degraus da grelha do
    /// matcap. O sintoma é **banda** numa barriga lisa.
    ///
    /// ⚠️ **Quanto isto morde depende do matcap, e não foi medido no asset da casa** (749², onde a
    /// grelha é fina o bastante para o efeito ser pequeno). Está aqui por ser o certo em qualquer
    /// tamanho — não por ter sido a causa de um sintoma relatado. *Uma correção certa não precisa
    /// de reivindicar um bug que ninguém provou que ela cura.*
    #[must_use]
    fn sample(&self, u: f32, v: f32) -> [f32; 3] {
        let side = self.side as usize;
        // −0,5 porque o texel é uma ÁREA e a coordenada dele é o CENTRO dela: sem isso a imagem
        // desloca-se meio texel e as bordas do matcap espelham-se erradas.
        let fx = (u * side as f32 - 0.5).clamp(0.0, side as f32 - 1.0);
        let fy = (v * side as f32 - 0.5).clamp(0.0, side as f32 - 1.0);
        let (x0, y0) = (fx.floor() as usize, fy.floor() as usize);
        let (x1, y1) = ((x0 + 1).min(side - 1), (y0 + 1).min(side - 1));
        let (tx, ty) = (fx - x0 as f32, fy - y0 as f32);
        let texel = |x: usize, y: usize| -> [f32; 3] {
            let t = (y * side + x) * 3;
            [
                self.rgb_linear[t],
                self.rgb_linear[t + 1],
                self.rgb_linear[t + 2],
            ]
        };
        let (a, b, c, d) = (texel(x0, y0), texel(x1, y0), texel(x0, y1), texel(x1, y1));
        let mut out = [0.0f32; 3];
        for k in 0..3 {
            let top = a[k] + (b[k] - a[k]) * tx;
            let bot = c[k] + (d[k] - c[k]) * tx;
            out[k] = top + (bot - top) * ty;
        }
        out
    }

    /// A cor de uma normal de vista. A lei de amostragem é a do matcap: `uv = n.xy * 0.5 + 0.5`.
    #[must_use]
    fn colour(&self, n: [f32; 3]) -> [f32; 3] {
        let u = (n[0] * 0.5 + 0.5).clamp(0.0, 1.0);
        let v = (1.0 - (n[1] * 0.5 + 0.5)).clamp(0.0, 1.0);
        self.sample(u, v)
    }
}

/// Colore o G-buffer com um matcap e devolve RGBA8 **pré-multiplicado**.
///
/// A lei de amostragem é a do matcap: `uv = n.xy * 0.5 + 0.5`, com `n` em espaço de vista. É por
/// isso que ela mora aqui, ao lado de quem produz essa normal — e não do outro lado do repositório,
/// onde a convenção teria de ser re-afirmada num comentário.
///
/// # ⚠️ Pré-multiplicado, e a resolução é em LINEAR
///
/// Duas escolhas que não são gosto:
///
/// - **Pré-multiplicado** porque a imagem vai ser **filtrada** ao ser desenhada, e num alfa direto o
///   filtro mistura a cor de pixels transparentes — cuja cor não significa nada. O sintoma é a
///   auréola escura à volta da peça, e é *o* bug clássico de compor imagem com borda macia.
/// - **Média em linear**, nunca em bytes sRGB. Metade de branco com metade de preto não é cinza-127
///   — é cinza-188. Fazer a média em sRGB escurece toda borda, que é o outro bug clássico e o mais
///   difícil de ver porque parece só "um contorno".
#[must_use]
pub fn shade(g: &Gbuffer, m: &Matcap<'_>, background: [u8; 4]) -> Vec<u8> {
    let bg_a = f32::from(background[3]) / 255.0;
    // O fundo, já pré-multiplicado e em linear — é ele que entra na média de um pixel de borda.
    let bg = [
        ph2d_color::srgb::srgb_to_linear_byte(background[0]) * bg_a,
        ph2d_color::srgb::srgb_to_linear_byte(background[1]) * bg_a,
        ph2d_color::srgb::srgb_to_linear_byte(background[2]) * bg_a,
        bg_a,
    ];
    let write = |px: &mut [u8], c: [f32; 4]| {
        px[0] = ph2d_color::srgb::linear_to_srgb_byte(c[0]);
        px[1] = ph2d_color::srgb::linear_to_srgb_byte(c[1]);
        px[2] = ph2d_color::srgb::linear_to_srgb_byte(c[2]);
        px[3] = (c[3].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    };

    let mut out = vec![0u8; (g.width as usize) * (g.height as usize) * 4];
    if m.side == 0 {
        for px in out.chunks_exact_mut(4) {
            px.copy_from_slice(&background);
        }
        return out;
    }

    for (i, px) in out.chunks_exact_mut(4).enumerate() {
        if g.hit[i] {
            let rgb = m.colour(g.normal[i]);
            write(px, [rgb[0], rgb[1], rgb[2], 1.0]);
        } else {
            // ⚠️ Copiado, e não passado pela conversão: um pixel de fundo puro tem de sair
            // **exatamente** com os bytes que o chamador pediu. Levá-lo pela ida-e-volta sRGB faria
            // a cor do fundo depender da precisão de uma tabela — e um fundo que quase bate é a
            // costura mais difícil de ver que existe.
            px.copy_from_slice(&background);
        }
    }

    // As bordas, resolvidas em COR — e não pela média das normais.
    //
    // ⚠️ A diferença aparece exatamente onde uma superfície passa à frente de outra: ali as duas
    // normais podem ser quase opostas, e a média delas aponta para um sítio do matcap que não é
    // nenhuma das duas cores. Média de normais é interpolar a GEOMETRIA; o que se quer é interpolar
    // o que se vê.
    for e in &g.edges {
        let i = e.pixel as usize;
        let mut acc = [0.0f32; 4];
        for k in 0..4 {
            let c = if e.hit[k] {
                let rgb = m.colour(e.normal[k]);
                [rgb[0], rgb[1], rgb[2], 1.0]
            } else {
                bg
            };
            for j in 0..4 {
                acc[j] += c[j] * 0.25;
            }
        }
        write(&mut out[i * 4..i * 4 + 4], acc);
    }
    out
}

#[cfg(test)]
mod tests;
