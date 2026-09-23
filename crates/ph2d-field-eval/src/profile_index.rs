//! ⭐⭐ **O PERFIL DEIXA DE SER UMA FITA E PASSA A SER UMA CONSULTA** (W56).
//!
//! # O número que abriu esta wave
//!
//! O [`crate::profile`] baixa o contorno numa árvore de avaliação: ~20 nós por aresta, **todos**
//! percorridos em **toda** amostra. Medido (`the_table_that_says_where_a_profile_spends_its_time`,
//! máquina calma):
//!
//! | arestas | ns/ponto | × um cilindro analítico | ns/ponto/**aresta** |
//! |---:|---:|---:|---:|
//! | — | 2,0 | 1,00× | — |
//! | 56 | 52,5 | 26,6× | 0,937 |
//! | **168** (o default da W55) | **155,8** | **79,0×** | 0,927 |
//! | 664 (o teto da W55) | 636,2 | 322,7× | 0,958 |
//! | 940 | 877,7 | 445,1× | 0,934 |
//!
//! ⭐ **Linear perfeito** — 0,95 ns por ponto por aresta ao longo de toda a faixa. É esse o preço
//! que faz o teto de `Resolution` ser **16** e não o que o artista quisesse.
//!
//! # ⛔ E a cura que estava PRESCRITA não serviria
//!
//! O [`docs/3DModeling/04_resultados_perfis.md`] §7 escreveu, em 2026-08-19, o gatilho e a direção:
//! *"aceleração espacial **dentro da árvore** — partir o perfil numa hierarquia de `min`/`max` por
//! caixa, para que **a poda por intervalo** volte a morder"*.
//!
//! ⚠️ **Ninguém avalia intervalos neste caminho.** O `Hybrid` monta `float_slice_tape` (ponto a
//! ponto) e `grad_slice_tape`; não há passe por ladrilho, não há `simplify`, e a extração varre uma
//! grade uniforme — também ponto a ponto. Uma hierarquia de `min`/`max` numa fita ponto-a-ponto é
//! avaliada **inteira**: ela não moveria o traçado um milissegundo. *Meça o mecanismo antes de
//! construir o que a nota prescreve.*
//!
//! # O que este módulo faz, e por que é EXATO
//!
//! Duas estruturas, porque as duas metades da distância com sinal têm naturezas diferentes:
//!
//! | metade | estrutura | por quê |
//! |---|---|---|
//! | **distância** | BVH sobre os segmentos, com poda por caixa | um `min` poda por ramo-e-limite em qualquer ponto, dentro ou fora |
//! | **sinal** | grelha sobre a caixa do perfil, com o enrolamento **pré-somado** por célula | o enrolamento é uma soma: ela não poda — mas é um **invariante de caminho**, e isso sim se pré-computa |
//!
//! ⭐ **O sinal fora da caixa é ZERO, e é exato:** o enrolamento de uma curva fechada contida na
//! caixa é nulo para qualquer ponto fora dela. Então a grelha só precisa de cobrir a caixa, que é
//! onde o traçado passa pouco tempo.
//!
//! ⭐⭐ **E dentro da caixa o enrolamento é um INVARIANTE DE CAMINHO**: `w(p) = w(c) + cruzamentos do
//! caminho c→p`. Guardando `w` no canto de cada célula, a conta no ponto só olha as arestas que
//! **atravessam aquela célula** — porque o caminho canto→ponto não sai dela.
//!
//! # ⚠️ Onde mora o quê (teto de LOC, 2026-09-23: `728` contra `700`)
//!
//! | ficheiro | responsabilidade |
//! |---|---|
//! | **este** | a construção, a consulta por PONTO e a grelha do enrolamento |
//! | [`corte`] | que arestas uma REGIÃO precisa — os três conjuntos, e a costura do eixo |
//! | [`crate::profile_dist`] | a aritmética que os dois perguntam |
//!
//! ⚠️ **Duas implementações da mesma lei, e a lei tem um JUIZ.** É o mesmo compromisso (e a mesma
//! defesa) do [`crate::hybrid`]: o gate `the_query_is_the_same_law_as_the_tape` avalia as duas
//! formas no mesmo perfil, ponto a ponto, e exige o mesmo número.

use ph2d_field::{FillRule, Profile};

/// Quantas arestas cabem numa folha do BVH antes de valer a pena partir.
///
/// ⚠️ **Número de estrutura, não de gosto**: abaixo dele o custo da descida passa a dominar o da
/// varredura linear da folha. Quatro é o valor com que a tabela do módulo foi medida; movê-lo pede
/// re-correr a sonda `the_table_that_says_where_a_profile_spends_its_time`.
const LEAF: usize = 4;

/// Lado da grelha do enrolamento, em células.
///
/// ⚠️ O custo de construção é `células × arestas` (uma varredura de raio por canto), e o de consulta
/// é o número de arestas que **atravessam** uma célula. Com 32 a construção de um contorno de 168
/// arestas é ~172 k operações — abaixo do ruído de montar a fita, que a mesma sonda mediu em 2,4 ms.
const GRID: usize = 32;

/// A pilha da descida no BVH, içada para fora do laço do lote.
///
/// ⚠️ **32 e não 64**: a profundidade é `2·log2(arestas/LEAF)` — para as 940 arestas do teto medido
/// dá 16. Uma pilha maior não compra nada e é memória tocada por lote.
struct Stack([u32; 32]);

impl Stack {
    fn new() -> Self {
        Self([0; 32])
    }
}

/// Uma aresta do contorno, com o que a distância pede pré-calculado.
///
/// ⭐⭐⭐ **Desde 2026-09-16 ela pode ser um ARCO** (`arco = Some`): `a`, `b` e `e` passam a ser os da
/// CORDA, que é o que o enrolamento percorre, e o arco entra pela porta única
/// ([`crate::profile_arc`]). ⚠️ Todo ponto do arco está a menos da `flecha` da corda — é essa
/// desigualdade, e só ela, que mantém o corte espacial CONSERVADOR sem que ele conheça o arco.
#[derive(Clone, Copy, Debug)]
struct Edge {
    a: [f32; 2],
    b: [f32; 2],
    e: [f32; 2],
    inv_ee: f32,
    arco: Option<crate::profile_arc::Arco>,
}

impl Edge {
    /// Quanto o arco pode sair da corda (`0` numa recta).
    #[allow(clippy::cast_possible_truncation)]
    fn flecha(&self) -> f32 {
        self.arco.map_or(0.0, |k| k.flecha as f32)
    }

    /// A caixa da PRIMITIVA — a da corda engordada pela flecha.
    fn caixa(&self) -> ([f32; 2], [f32; 2]) {
        let f = self.flecha();
        (
            [self.a[0].min(self.b[0]) - f, self.a[1].min(self.b[1]) - f],
            [self.a[0].max(self.b[0]) + f, self.a[1].max(self.b[1]) + f],
        )
    }
}

#[derive(Clone, Copy, Debug)]
struct BvhNode {
    lo: [f32; 2],
    hi: [f32; 2],
    /// Filhos, ou `u32::MAX` numa folha. ⚠️ **Os dois, explícitos**: a construção é pós-ordem, então
    /// o irmão direito **não** fica em `left + 1` — a versão que o supôs lia um nó de outra
    /// sub-árvore, e a distância saía ora certa ora não.
    left: u32,
    right: u32,
    first: u32,
    count: u32,
}

/// ⭐ **O perfil pronto a consultar** — construído uma vez, consultado por ponto.
///
/// ⚠️ **Ele é DERIVADO e não entra em documento nenhum.** O que se salva é o [`Profile`]; isto é o
/// que se constrói a partir dele para avaliar. A lei é a mesma que mantém a árvore compilada fora
/// do componente (`ph2d-field-ecs`): estado derivado num save envenena o undo.
pub struct ProfileIndex {
    edges: Vec<Edge>,
    order: Vec<u32>,
    nodes: Vec<BvhNode>,
    lo: [f32; 2],
    cell: [f32; 2],
    /// Enrolamento no PONTO DE PARTIDA de cada célula (`GRID × GRID`, em ordem de linha).
    base: Vec<i32>,
    /// ⭐⭐ **O ponto de partida de cada célula** — um ponto dela que não assenta em aresta nenhuma.
    ///
    /// ⛔ **Era o canto mínimo, e isso é o defeito da W56 um nível abaixo** (medido 2026-09-16):
    /// o `base` sai do raio `+x` e o caminho até ao ponto do semi-aberto, e sobre uma aresta as duas
    /// regras discordam de que lado o canto está. A polilinha densa quase nunca punha uma aresta num
    /// canto da grelha; as cordas redondas de uma decomposição com arcos põem, e o sinal saía errado
    /// numa célula inteira (mascarado em `NonZero`, que lia `−2` onde devia ler `−1`; a paridade
    /// denunciou-o). A cura é a da árvore por região ([`crate::profile`] — `anchor_in`): partir de um
    /// ponto LONGE das arestas.
    start: Vec<[f32; 2]>,
    /// Arestas que atravessam cada célula — `cross[cross_at[c]..cross_at[c + 1]]`.
    cross_at: Vec<u32>,
    cross: Vec<u32>,
    non_zero: bool,
}

impl ProfileIndex {
    /// Constrói o índice a partir do perfil cozido.
    #[must_use]
    pub fn build(profile: &Profile) -> Self {
        let mut edges: Vec<Edge> = Vec::with_capacity(profile.prim_count());
        for (ci, contour) in profile.contours().iter().enumerate() {
            // ⭐⭐⭐ **A decomposição exacta, quando existe** — a MESMA que a árvore global lê. A
            // polilinha densa descreve a mesma curva a menos da tolerância, mas as NORMAIS dela
            // saltam a cada segmento, e foi isso que o modo MODEL mostrou como faixas.
            let arcs = profile.arcs().get(ci).filter(|a| !a.is_empty());
            let pts: Vec<([f32; 2], f32)> = match arcs {
                Some(a) => a.clone(),
                None => contour.iter().map(|p| (*p, 0.0)).collect(),
            };
            let n = pts.len();
            for i in 0..n {
                let (a, bulge) = pts[i];
                let b = pts[(i + 1) % n].0;
                let e = [b[0] - a[0], b[1] - a[1]];
                // `Profile::new` tirou os pontos repetidos consecutivos ⇒ a aresta tem comprimento.
                let inv_ee = 1.0 / e[0].mul_add(e[0], e[1] * e[1]);
                let arco = (bulge != 0.0).then(|| {
                    crate::profile_arc::arco(
                        [f64::from(a[0]), f64::from(a[1])],
                        [f64::from(b[0]), f64::from(b[1])],
                        f64::from(bulge),
                    )
                });
                edges.push(Edge {
                    a,
                    b,
                    e,
                    inv_ee,
                    arco,
                });
            }
        }
        let (lo, hi) = profile.bounds();
        // ⚠️ Uma caixa degenerada num eixo (um perfil achatado) daria célula de largura zero e uma
        // divisão por zero na consulta. O piso é a tolerância do próprio perfil — o mesmo número
        // com que ele foi achatado, e não um epsilon inventado aqui.
        let floor = profile.tolerance().max(f32::MIN_POSITIVE);
        let span = [(hi[0] - lo[0]).max(floor), (hi[1] - lo[1]).max(floor)];
        let cell = [span[0] / GRID as f32, span[1] / GRID as f32];

        let mut order: Vec<u32> = (0..edges.len() as u32).collect();
        let mut nodes = Vec::new();
        build_bvh(&edges, &mut order, &mut nodes, 0, edges.len());

        let non_zero = profile.fill() == FillRule::NonZero;
        let mut base = vec![0i32; GRID * GRID];
        let mut start = vec![[0.0f32; 2]; GRID * GRID];
        let mut cross_at = vec![0u32; GRID * GRID + 1];
        let mut cross: Vec<u32> = Vec::new();
        for gy in 0..GRID {
            for gx in 0..GRID {
                let c = gy * GRID + gx;
                let corner = [lo[0] + cell[0] * gx as f32, lo[1] + cell[1] * gy as f32];
                let (bl, bh) = (corner, [corner[0] + cell[0], corner[1] + cell[1]]);
                cross_at[c] = cross.len() as u32;
                for (i, e) in edges.iter().enumerate() {
                    let (elo, ehi) = e.caixa();
                    if elo[0] <= bh[0] && ehi[0] >= bl[0] && elo[1] <= bh[1] && ehi[1] >= bl[1] {
                        cross.push(i as u32);
                    }
                }
                // Só uma aresta que toca a célula pode passar por um ponto dela.
                let lista = &cross[cross_at[c] as usize..];
                start[c] = partida_segura(&edges, lista, bl, bh);
                base[c] = ray_winding(&edges, start[c]);
            }
        }
        cross_at[GRID * GRID] = cross.len() as u32;

        Self {
            edges,
            order,
            nodes,
            lo,
            cell,
            base,
            start,
            cross_at,
            cross,
            non_zero,
        }
    }

    /// ⭐ **A distância com sinal do ponto ao perfil** — negativa dentro, como a árvore.
    ///
    /// ⚠️ **Para um lote, use a [`Self::sd_batch`]**: a fita da `fidget` avalia oito pontos por
    /// instrução, e comparar um laço escalar com ela mede a forma da chamada, não a estrutura.
    #[must_use]
    pub fn sd(&self, u: f32, v: f32) -> f32 {
        let mut stack = Stack::new();
        self.sd_at(&mut stack, u, v)
    }

    /// ⭐⭐ **O lote** — a porta que o avaliador usa, e a única que se compara com a fita.
    ///
    /// ⚠️ A pilha da descida é **içada para fora do laço**: uma `[u32; 64]` por ponto são 256 bytes
    /// de escrita antes de a primeira caixa ser testada, e a medição da primeira versão pagava-os
    /// 200 mil vezes. *Uma estrutura de aceleração medida por chamada mede a chamada.*
    pub fn sd_batch(&self, xs: &[f32], ys: &[f32], out: &mut Vec<f32>) {
        out.clear();
        out.reserve(xs.len());
        let mut stack = Stack::new();
        for (u, v) in xs.iter().zip(ys) {
            out.push(self.sd_at(&mut stack, *u, *v));
        }
    }

    /// ⚠️ **Só para a sonda**: as duas metades separadas, para saber qual paga o relógio.
    #[doc(hidden)]
    #[must_use]
    pub fn probe_dist_only(&self, xs: &[f32], ys: &[f32]) -> f32 {
        let mut stack = Stack::new();
        let mut d = 0.0f32;
        for (u, v) in xs.iter().zip(ys) {
            d += self.dist2(&mut stack, [*u, *v]);
        }
        d
    }

    /// ⚠️ Só para a sonda: o sinal, sem a distância.
    #[doc(hidden)]
    #[must_use]
    pub fn probe_sign_only(&self, xs: &[f32], ys: &[f32]) -> f32 {
        let mut w = 0.0f32;
        for (u, v) in xs.iter().zip(ys) {
            w += f32::from(u8::from(self.inside([*u, *v])));
        }
        w
    }

    /// ⭐⭐⭐ **O LOTE COM AS ARESTAS CORTADAS** — a forma que persegue o tecto medido.
    ///
    /// # Por que é aqui, e não numa busca por ponto
    ///
    /// A fita da `fidget` custa **0,95 ns por ponto por aresta** com JIT e oito faixas — por aresta
    /// ela é quase óptima. Uma busca escalar do segmento mais próximo, por melhor que pode-se, ganha
    /// **1,9×** (medido). O que fecha a distância para o tecto é outra coisa: **tocar menos
    /// arestas**, mantendo o laço tenso e vectorizável.
    ///
    /// ⚠️ **O corte tem de ser CONSERVADOR ou a marcha atravessa a peça.** Deitar fora uma aresta
    /// que podia ser a mais próxima faz a distância sair **maior** que a verdadeira — e uma
    /// esfera-marcha que sobre-estima o passo salta a superfície. A regra abaixo é a exacta:
    ///
    /// ```text
    /// dmax = min sobre as arestas de (maior distância de um canto da caixa àquela aresta)
    /// fica  = toda aresta cuja MENOR distância à caixa é <= dmax
    /// ```
    ///
    /// A distância a um segmento é **convexa**, então o máximo sobre a caixa está num canto dela —
    /// é isso que torna `dmax` calculável em quatro avaliações por aresta.
    ///
    /// ⚠️ **Quem chama decide a coerência do lote.** Uma linha inteira de ecrã tem pegada larga e
    /// não corta nada; um punhado de raios vizinhos tem pegada pequena e corta quase tudo. *O corte
    /// mede a compacidade de quem o chamou.*
    pub fn sd_batch_culled(
        &self,
        xs: &[f32],
        ys: &[f32],
        scratch: &mut Vec<u32>,
        out: &mut Vec<f32>,
    ) {
        out.clear();
        out.reserve(xs.len());
        if xs.is_empty() {
            return;
        }
        let (mut lo, mut hi) = ([f32::INFINITY; 2], [f32::NEG_INFINITY; 2]);
        for (u, v) in xs.iter().zip(ys) {
            lo[0] = lo[0].min(*u);
            lo[1] = lo[1].min(*v);
            hi[0] = hi[0].max(*u);
            hi[1] = hi[1].max(*v);
        }
        self.cull(lo, hi, scratch);
        for (u, v) in xs.iter().zip(ys) {
            let p = [*u, *v];
            let mut best = f32::INFINITY;
            for i in scratch.iter() {
                best = best.min(edge_dist2(p, &self.edges[*i as usize]));
            }
            let d = best.sqrt();
            out.push(if self.inside(p) { -d } else { d });
        }
    }

    /// O arco da aresta `i`, se ela for um — ver [`crate::profile_arc`].
    pub(crate) fn arco(&self, i: u32) -> Option<crate::profile_arc::Arco> {
        self.edges[i as usize].arco
    }

    /// Os dois extremos da aresta `i` — o que a baixagem especializada precisa de constantes.
    #[must_use]
    pub fn edge(&self, i: u32) -> ([f32; 2], [f32; 2]) {
        let e = self.edges[i as usize];
        (e.a, e.b)
    }

    /// ⭐ **O enrolamento no ponto, pela regra do raio `+x`** — o mesmo número que a árvore calcula.
    ///
    /// É ele que vira a **constante** de uma região especializada.
    #[must_use]
    pub fn winding_at(&self, p: [f32; 2]) -> i32 {
        ray_winding(&self.edges, p)
    }

    /// A menor distância (ao quadrado) do ponto às arestas dadas — `∞` para uma lista vazia.
    ///
    /// ⚠️ Ela existe para uma pergunta de **robustez**, não de geometria: *este ponto serve de âncora
    /// do enrolamento?* Ver [`crate::profile::sd_profile_in_region`].
    #[must_use]
    pub fn min_dist2_to(&self, edges: &[u32], p: [f32; 2]) -> f32 {
        edges.iter().fold(f32::INFINITY, |acc, i| {
            acc.min(seg_dist2(p, &self.edges[*i as usize]))
        })
    }

    /// ⚠️ Só para o gate: os atravessamentos do caminho `a → b`, somados sobre **todas** as arestas.
    #[doc(hidden)]
    #[must_use]
    pub fn probe_path_winding(&self, a: [f32; 2], b: [f32; 2]) -> i32 {
        self.edges.iter().map(|e| path_crossing(a, b, e)).sum()
    }

    /// Quantas arestas o perfil tem, na ordem em que [`Self::edge`] as endereça.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// A lei de preenchimento — `true` para `NonZero`.
    #[must_use]
    pub fn is_non_zero(&self) -> bool {
        self.non_zero
    }

    fn sd_at(&self, stack: &mut Stack, u: f32, v: f32) -> f32 {
        let d = self.dist2(stack, [u, v]).sqrt();
        if self.inside([u, v]) { -d } else { d }
    }

    /// O quadrado da distância ao contorno mais próximo, por ramo-e-limite sobre o BVH.
    fn dist2(&self, stack: &mut Stack, p: [f32; 2]) -> f32 {
        let mut best = f32::INFINITY;
        let stack = &mut stack.0;
        let mut top = 1usize;
        stack[0] = 0;
        while top > 0 {
            top -= 1;
            let n = self.nodes[stack[top] as usize];
            if box_dist2(p, n.lo, n.hi) >= best {
                continue;
            }
            if n.left == u32::MAX {
                for k in 0..n.count {
                    let e = self.edges[self.order[(n.first + k) as usize] as usize];
                    best = best.min(edge_dist2(p, &e));
                }
            } else {
                // O filho mais perto primeiro: é o que faz o `best` apertar cedo e podar o irmão.
                let (l, r) = (n.left, n.right);
                let (dl, dr) = (
                    box_dist2(p, self.nodes[l as usize].lo, self.nodes[l as usize].hi),
                    box_dist2(p, self.nodes[r as usize].lo, self.nodes[r as usize].hi),
                );
                let (near, far) = if dl <= dr { (l, r) } else { (r, l) };
                stack[top] = far;
                stack[top + 1] = near;
                top += 2;
            }
        }
        best
    }

    /// Dentro ou fora — pela lei de preenchimento do perfil.
    fn inside(&self, p: [f32; 2]) -> bool {
        let gx = ((p[0] - self.lo[0]) / self.cell[0]).floor();
        let gy = ((p[1] - self.lo[1]) / self.cell[1]).floor();
        // ⭐ **Fora da caixa o enrolamento é ZERO, e é exato**: a curva fechada está toda dentro
        // dela. É o que dispensa a grelha de cobrir o espaço onde a marcha passa a maior parte do
        // tempo.
        if !(0.0..GRID as f32).contains(&gx) || !(0.0..GRID as f32).contains(&gy) {
            return false;
        }
        let c = gy as usize * GRID + gx as usize;
        let corner = self.start[c];
        let mut w = self.base[c];
        for k in self.cross_at[c]..self.cross_at[c + 1] {
            let e = self.edges[self.cross[k as usize] as usize];
            w += path_crossing(corner, p, &e);
            if let Some(arco) = e.arco {
                // ⚠️ O `orient` é o MESMO que o `path_crossing` usou como `d2`: o empate de um ponto
                // sobre a corda tem de cair do mesmo lado nas duas metades.
                w += crate::profile_arc::meia_lua_caminho(
                    [f64::from(p[0]), f64::from(p[1])],
                    &arco,
                    orient(e.a, e.b, p),
                    self.non_zero,
                );
            }
        }
        if self.non_zero { w != 0 } else { w % 2 != 0 }
    }
}

/// Constrói o BVH por mediana no eixo mais longo, e devolve o índice do nó criado.
fn build_bvh(
    edges: &[Edge],
    order: &mut [u32],
    nodes: &mut Vec<BvhNode>,
    first: usize,
    count: usize,
) -> u32 {
    let (mut lo, mut hi) = ([f32::INFINITY; 2], [f32::NEG_INFINITY; 2]);
    for k in first..first + count {
        let (elo, ehi) = edges[order[k] as usize].caixa();
        for axis in 0..2 {
            lo[axis] = lo[axis].min(elo[axis]);
            hi[axis] = hi[axis].max(ehi[axis]);
        }
    }
    let me = nodes.len() as u32;
    nodes.push(BvhNode {
        lo,
        hi,
        left: u32::MAX,
        right: u32::MAX,
        first: first as u32,
        count: count as u32,
    });
    if count <= LEAF {
        return me;
    }
    let axis = usize::from(hi[1] - lo[1] > hi[0] - lo[0]);
    let key = |i: u32| {
        let e = &edges[i as usize];
        e.a[axis] + e.b[axis]
    };
    order[first..first + count].sort_by(|x, y| key(*x).total_cmp(&key(*y)));
    let half = count / 2;
    let l = build_bvh(edges, order, nodes, first, half);
    let r = build_bvh(edges, order, nodes, first + half, count - half);
    nodes[me as usize].left = l;
    nodes[me as usize].right = r;
    me
}

#[path = "profile_dist.rs"]
mod dist;
use dist::{box_dist2, edge_dist2, seg_dist2};

/// ⭐⭐⭐ **O CORTE** — ver o cabeçalho do [`corte`].
#[path = "profile_corte.rs"]
mod corte;

#[path = "profile_winding.rs"]
mod winding;
use winding::{orient, partida_segura, path_crossing, ray_winding};

#[cfg(test)]
#[path = "profile_index_tests.rs"]
mod tests;

/// ⛔⛔⛔ **O gate da COSTURA DO EIXO** — ver o cabeçalho do [`eixo_tests`].
#[cfg(test)]
#[path = "profile_index_eixo_tests.rs"]
mod eixo_tests;

#[cfg(test)]
#[path = "profile_index_tables.rs"]
mod tables;
