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
    };

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
            if t[iu] < T_MAX {
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
