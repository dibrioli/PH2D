//! **O ARRANJO do corpo** — tudo o que `rows`/`cols` respondiam, reescrito como
//! FATO da forma de repouso.
//!
//! Este nó nasceu com a malha `rows × cols` como se ela fosse o corpo, e três
//! respostas ficaram penduradas nesses dois números:
//!
//! 1. **quem é pino** (`i < cols` — a linha de topo),
//! 2. **qual é a fronteira** que a pressão defende (o passeio do anel),
//! 3. **como o corpo se divide em regiões** (bandas de índice ao longo do lado
//!    maior).
//!
//! Nenhuma das três é sobre a grelha. A primeira é *a aresta de cima do
//! repouso*; a segunda é *o contorno do repouso*; a terceira é *bandas ao longo
//! do eixo maior do repouso*. Escrevê-las assim é o que deixa a porta `shape`
//! entregar uma nuvem qualquer — a referência que o plano cita (Cavalry põe um
//! Forge Soft Body em QUALQUER forma; o Vellum em qualquer geometria).
//!
//! # ⚠️ A grelha continua a ser a grelha, ao BIT
//!
//! O caminho autorado não passa a ser um caso da nuvem: ele é o seu **próprio
//! fornecedor** destas três respostas, e cada uma devolve a MESMA sequência de
//! índices que o código de 2026-08 percorria à mão. Isso não é zelo — o anel e
//! as regiões alimentam somas em `f32`, e uma sequência com os mesmos elementos
//! noutra ordem daria outro número, movendo arte já autorada por ruído de
//! arredondamento. O que se partilha é a LEI (a área de um anel, o ajuste de uma
//! região); o que difere é de onde o anel e as regiões vêm.
//!
//! ⚠️ E há gate a afirmar mais do que isso: o **casco** de uma grelha entregue
//! pela porta é o mesmo anel que o passeio da grelha produz, índice a índice —
//! ou seja, alimentar a porta com `rest_shape(rows, cols, spacing)` devolve o
//! corpo autorado, e não um primo dele.

use crate::cluster::{counts, span};

/// Como o corpo está ARRUMADO — o que `rows`/`cols` costumavam responder.
///
/// ⚠️ **O repouso é sempre centrado no centroide não-ponderado**, e isso é
/// correção e não arrumação: o termo de pressão escala os goals sobre o centro
/// do quadro ajustado, e o `shape_goals_weighted` só pode tratar isso como a
/// mesma operação porque `Σ M·qᵢ = M·Σ qᵢ = 0`. Um repouso descentrado faria a
/// pressão **transladar** o corpo enquanto o inflasse.
pub(crate) struct BodyLayout {
    /// As posições de repouso, centradas no centroide.
    pub(crate) rest: Vec<[f32; 2]>,
    /// O anel de fronteira, por índice, no sentido HORÁRIO neste referencial
    /// y-para-cima (área assinada negativa para um corpo saudável — ver
    /// `shape::ring_area`).
    ring: Vec<usize>,
    /// Quem é pino quando o param `pin` está ligado: a aresta de cima.
    pinned: Vec<bool>,
    /// De onde as regiões saem.
    bands: Bands,
}

/// O fornecedor das regiões — a única coisa que a grelha e a nuvem não
/// partilham.
enum Bands {
    /// A malha autorada: bandas de ÍNDICE, exactamente como sempre foram.
    Grid { rows: usize, cols: usize },
    /// Uma nuvem entregue pela porta: bandas de COORDENADA sobre a caixa do
    /// repouso, com a mesma sobreposição de meia banda.
    Cloud {
        /// Canto inferior-esquerdo da caixa do repouso.
        min: [f32; 2],
        /// Extensão da caixa (nunca zero — ver `Bands::cloud`).
        ext: [f32; 2],
        /// O tamanho da grelha uniforme que teria esta contagem e esta
        /// proporção — é ela que decide quantas bandas cabem por eixo.
        eff: [usize; 2],
    },
}

impl BodyLayout {
    /// A malha autorada `rows × cols`, espaçada por `spacing`.
    pub(crate) fn from_grid(rows: usize, cols: usize, spacing: f32) -> Self {
        let rest = grid_rest(rows, cols, spacing);
        Self {
            ring: grid_ring(rows, cols),
            // A linha 0 é o topo (max y) por construção do `grid_rest`.
            pinned: (0..rest.len()).map(|i| i < cols).collect(),
            bands: Bands::Grid { rows, cols },
            rest,
        }
    }

    /// Uma nuvem qualquer, entregue pela porta `shape`.
    ///
    /// ⚠️ **A forma é lida a cada tique**, e não só ao semear: mexer as posições
    /// de montante sem mexer a CONTAGEM não re-semeia o corpo — ele passa a
    /// perseguir o repouso novo. É o que faz uma forma de repouso ANIMADA
    /// funcionar sem um caminho próprio, e o preço é que uma forma que se mexe
    /// depressa arrasta o corpo, que é exactamente o que ela pede.
    pub(crate) fn from_cloud(points: &[[f32; 2]]) -> Self {
        let n = points.len();
        let inv = if n == 0 { 0.0 } else { 1.0 / n as f32 };
        let mut c = [0.0f32; 2];
        for p in points {
            c[0] += p[0];
            c[1] += p[1];
        }
        c = [c[0] * inv, c[1] * inv];
        let rest: Vec<[f32; 2]> = points.iter().map(|p| [p[0] - c[0], p[1] - c[1]]).collect();
        let bands = Bands::cloud(&rest);
        Self {
            ring: hull_ring(&rest),
            pinned: top_edge(&rest, bands.row_height()),
            bands,
            rest,
        }
    }

    /// Quantas partículas o corpo tem.
    pub(crate) fn len(&self) -> usize {
        self.rest.len()
    }

    /// O anel de fronteira, por índice.
    pub(crate) fn ring(&self) -> &[usize] {
        &self.ring
    }

    /// Se a partícula `i` é pino (só quando o param `pin` está ligado).
    pub(crate) fn is_pinned(&self, i: usize) -> bool {
        self.pinned.get(i).copied().unwrap_or(false)
    }

    /// As regiões sobrepostas de Müller et al. 2005 §4.3, cada uma como a lista
    /// dos índices que contém, **em ordem crescente**.
    ///
    /// A ordem é dupla e as duas metades importam: a das REGIÕES decide em que
    /// ordem os goals se somam em `sum[i]`, e a dos ÍNDICES dentro de uma região
    /// decide em que ordem o centroide e as matrizes dela se acumulam. As duas
    /// são `f32`, e é por isso que o fornecedor da grelha reproduz o laço
    /// `for cj { for rj { for r { for c } } } }` que sempre existiu, e não
    /// meramente os mesmos conjuntos.
    pub(crate) fn buckets(&self, clusters: usize) -> Vec<Vec<usize>> {
        match self.bands {
            Bands::Grid { rows, cols } => grid_buckets(rows, cols, clusters),
            Bands::Cloud { min, ext, eff } => cloud_buckets(&self.rest, min, ext, eff, clusters),
        }
    }
}

/// A malha de repouso: uma grelha `rows × cols` centrada na origem. A linha 0 é
/// o TOPO (max y). Row-major, então `0..cols` é a linha de cima.
pub(crate) fn grid_rest(rows: usize, cols: usize, spacing: f32) -> Vec<[f32; 2]> {
    let (w, h) = ((cols as f32 - 1.0) * spacing, (rows as f32 - 1.0) * spacing);
    let mut q = Vec::with_capacity(rows * cols);
    for r in 0..rows {
        for c in 0..cols {
            q.push([c as f32 * spacing - w * 0.5, h * 0.5 - r as f32 * spacing]);
        }
    }
    q
}

/// O anel da grelha, no sentido horário a partir do canto superior-esquerdo
/// (índice 0): linha de cima da esquerda para a direita, coluna da direita a
/// descer, linha de baixo da direita para a esquerda, coluna da esquerda a
/// subir. **É a sequência exacta** que o `boundary_area` percorria à mão.
///
/// Vazio quando a malha é degenerada (menos de duas linhas ou colunas), que é
/// como o `ring_area` devolve os `0.0` que aquele guarda devolvia.
pub(crate) fn grid_ring(rows: usize, cols: usize) -> Vec<usize> {
    if rows < 2 || cols < 2 {
        return Vec::new();
    }
    let at = |r: usize, c: usize| r * cols + c;
    let mut ring = Vec::with_capacity(2 * (rows + cols) - 4);
    ring.push(at(0, 0));
    for c in 1..cols {
        ring.push(at(0, c));
    }
    for r in 1..rows {
        ring.push(at(r, cols - 1));
    }
    for c in (0..cols - 1).rev() {
        ring.push(at(rows - 1, c));
    }
    for r in (1..rows - 1).rev() {
        ring.push(at(r, 0));
    }
    ring
}

/// As regiões da grelha: bandas de índice sobrepostas, na ordem em que sempre
/// foram visitadas.
fn grid_buckets(rows: usize, cols: usize, clusters: usize) -> Vec<Vec<usize>> {
    let (nr, nc) = counts(rows, cols, clusters);
    let mut out = Vec::with_capacity(nr * nc);
    for cj in 0..nc {
        let (c0, c1) = span(cj, nc, cols);
        for rj in 0..nr {
            let (r0, r1) = span(rj, nr, rows);
            let mut idx = Vec::with_capacity((r1 - r0) * (c1 - c0));
            for r in r0..r1 {
                for c in c0..c1 {
                    idx.push(r * cols + c);
                }
            }
            out.push(idx);
        }
    }
    out
}

/// Quem está na aresta de CIMA de uma nuvem: as partículas dentro de MEIA
/// FILEIRA do `y` máximo.
///
/// ⚠️ **A espessura é uma FILEIRA, e não um epsilon — e a diferença é o gesto
/// inteiro.** Com uma tolerância mínima, uma malha é pregada pela linha de cima
/// (dezenas de partículas) e um DISCO é pregado pelo seu ponto mais alto, um só:
/// medido na cena `=87`, um anel assim pendurado balança como um pêndulo e a
/// envergadura dele cresce **1,74×** em dois segundos. Não é o solver a falhar —
/// é o corpo a estar preso por um prego em vez de uma barra.
///
/// Meia fileira é o número que faz a lei reduzir-se à antiga sobre a malha
/// autorada: a linha 1 está a um espaçamento inteiro da 0, logo fica FORA, e a
/// fatia é exactamente `0..cols`. Numa nuvem, `row` sai da grelha equivalente
/// (ver [`Bands::cloud`]), então a barra é a mesma pergunta — *a que distância
/// está a fileira seguinte?* — respondida pela forma em vez de por um literal.
fn top_edge(rest: &[[f32; 2]], row: f32) -> Vec<bool> {
    let hi = rest.iter().fold(f32::NEG_INFINITY, |a, p| a.max(p[1]));
    let tol = (row * 0.5).max(f32::MIN_POSITIVE);
    rest.iter().map(|p| p[1] >= hi - tol).collect()
}

/// O contorno de uma nuvem: o casco convexo **com os pontos colineares
/// mantidos**, horário, a começar no ponto mais alto (desempate: o mais à
/// esquerda).
///
/// ⚠️ **Manter os colineares não é detalhe — é o que faz uma grelha entregue
/// pela porta devolver o anel da grelha, índice a índice.** Um casco estrito
/// devolveria quatro cantos, cuja área é a mesma em aritmética exacta e
/// **outra** em `f32`, e o corpo autorado passaria a defender um volume
/// ligeiramente diferente ao atravessar a porta.
///
/// ⚠️ **E o casco é o contorno de um conjunto de pontos SEM ligações**, o que é
/// a resposta canónica e não a resposta perfeita: uma forma de repouso CÔNCAVA
/// (uma lua, uma estrela) tem o seu envelope defendido em vez da sua área. A
/// pressão fica mais fraca, nunca invertida — e o sinal continua a ser o do
/// repouso, então um corpo virado do avesso continua a ser detectado.
fn hull_ring(rest: &[[f32; 2]]) -> Vec<usize> {
    if rest.len() < 3 {
        return Vec::new();
    }
    let mut order: Vec<usize> = (0..rest.len()).collect();
    order.sort_by(|&a, &b| {
        rest[a][0]
            .partial_cmp(&rest[b][0])
            .unwrap_or(core::cmp::Ordering::Equal)
            .then(
                rest[a][1]
                    .partial_cmp(&rest[b][1])
                    .unwrap_or(core::cmp::Ordering::Equal),
            )
            .then(a.cmp(&b))
    });
    // `< 0.0` (e não `<= 0.0`) mantém os colineares no casco.
    let cross = |o: usize, a: usize, b: usize| {
        (rest[a][0] - rest[o][0]) * (rest[b][1] - rest[o][1])
            - (rest[a][1] - rest[o][1]) * (rest[b][0] - rest[o][0])
    };
    let chain = |it: &mut dyn Iterator<Item = usize>| -> Vec<usize> {
        let mut h: Vec<usize> = Vec::new();
        for i in it {
            while h.len() >= 2 && cross(h[h.len() - 2], h[h.len() - 1], i) < 0.0 {
                h.pop();
            }
            h.push(i);
        }
        h
    };
    let lower = chain(&mut order.iter().copied());
    let upper = chain(&mut order.iter().rev().copied());
    if lower.len() < 2 || upper.len() < 2 {
        return Vec::new(); // todos colineares: não há área a defender
    }
    // Anti-horário; o resto do nó fala horário.
    let mut ring: Vec<usize> = lower[..lower.len() - 1]
        .iter()
        .chain(upper[..upper.len() - 1].iter())
        .copied()
        .collect();
    ring.reverse();
    // Começa no mais alto (desempate à esquerda) — o canto superior-esquerdo,
    // que numa grelha é o índice 0.
    let start = (0..ring.len())
        .max_by(|&a, &b| {
            let (pa, pb) = (rest[ring[a]], rest[ring[b]]);
            pa[1]
                .partial_cmp(&pb[1])
                .unwrap_or(core::cmp::Ordering::Equal)
                .then(
                    pb[0]
                        .partial_cmp(&pa[0])
                        .unwrap_or(core::cmp::Ordering::Equal),
                )
                .then(b.cmp(&a))
        })
        .unwrap_or(0);
    ring.rotate_left(start);
    ring
}

impl Bands {
    /// A caixa do repouso, mais a grelha uniforme EQUIVALENTE: aquela que teria
    /// esta contagem de partículas e esta proporção.
    ///
    /// ⚠️ **É essa equivalência que dá ao `clusters` o mesmo significado nos dois
    /// corpos.** O `counts` cobra `MIN_SPAN` partículas por banda, e a única
    /// forma de cobrar o mesmo a uma nuvem é dizer quantas partículas ela teria
    /// por eixo se fosse regular — senão o mesmo knob partiria uma nuvem esparsa
    /// em regiões vazias e o artista leria isso como o corpo a desfazer-se.
    fn cloud(rest: &[[f32; 2]]) -> Self {
        let (mut lo, mut hi) = ([f32::INFINITY; 2], [f32::NEG_INFINITY; 2]);
        for p in rest {
            for k in 0..2 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
        // Uma extensão nula (uma linha de pontos) vira 1 para a divisão não
        // explodir; a banda então contém tudo, que é a resposta certa.
        let ext = [
            (hi[0] - lo[0]).max(f32::MIN_POSITIVE),
            (hi[1] - lo[1]).max(f32::MIN_POSITIVE),
        ];
        let n = rest.len().max(1);
        let (long, short) = if ext[0] >= ext[1] {
            (ext[0], ext[1])
        } else {
            (ext[1], ext[0])
        };
        // ⚠️ **A EXTENSÃO conta intervalos; a contagem conta pontos** — e a
        // diferença de um é o que faz a nuvem aceitar um `clusters` diferente do
        // da grelha que ela imita. Uma malha `16 × 8` mede `15 s × 7 s`, razão
        // **2,143** e não 2: derivar o tamanho equivalente da razão crua dava
        // `17 × 7`, que o `counts` cortava a metade das regiões.
        //
        // A grelha uniforme com `n` pontos e esta razão de extensões resolve
        // `la · lb = n` com `la − 1 = r (lb − 1)`, ou seja
        // `r·lb² + (1−r)·lb − n = 0` — e a raiz positiva devolve `8` para a
        // malha acima, exactamente.
        let r = long / short;
        let (len_long, len_short) = if !r.is_finite() || r >= n as f32 {
            (n, 1) // uma fila de pontos: uma banda só no eixo curto
        } else {
            let b = 1.0 - r;
            let lb = ((-b + (b * b + 4.0 * r * n as f32).sqrt()) / (2.0 * r)).round();
            let lb = (lb as usize).clamp(1, n);
            ((n / lb).max(1), lb)
        };
        let eff = if ext[0] >= ext[1] {
            [len_short, len_long] // [linhas (y), colunas (x)]
        } else {
            [len_long, len_short]
        };
        Bands::Cloud { min: lo, ext, eff }
    }
}

impl Bands {
    /// A que distância vertical fica a fileira seguinte, na grelha equivalente.
    fn row_height(&self) -> f32 {
        match *self {
            // A grelha não passa por aqui (ela conhece a própria linha de topo
            // por índice), mas responder o mesmo que a nuvem responderia mantém
            // a lei uma só se alguém a chamar.
            Bands::Grid { rows, .. } => 1.0 / (rows.max(2) - 1) as f32,
            Bands::Cloud { ext, eff, .. } => ext[1] / (eff[0].max(2) - 1) as f32,
        }
    }
}

/// As regiões de uma nuvem: bandas de coordenada com a mesma sobreposição de
/// meia banda, e cada partícula depositada nas suas em ordem crescente de
/// índice.
///
/// ⚠️ **Cada partícula toca no máximo três bandas por eixo**, e é por isso que
/// isto custa `O(n)` e não `O(n · bandas)`: a banda `j` cobre
/// `[j/n − ½/n, (j+1)/n + ½/n]`, logo `t·n` cai numa janela de largura 2.
/// Testar cada partícula contra cada banda seria o mesmo desenho a `512²`
/// partículas e `256²` regiões.
fn cloud_buckets(
    rest: &[[f32; 2]],
    min: [f32; 2],
    ext: [f32; 2],
    eff: [usize; 2],
    clusters: usize,
) -> Vec<Vec<usize>> {
    let (nr, nc) = counts(eff[0], eff[1], clusters);
    let mut out = vec![Vec::new(); nr * nc];
    // `for cj { for rj }`, o mesmo enrolamento da grelha: o índice do balde é
    // `cj * nr + rj`.
    for (i, p) in rest.iter().enumerate() {
        let bx = band_range((p[0] - min[0]) / ext[0], nc);
        let by = band_range((min[1] + ext[1] - p[1]) / ext[1], nr);
        for cj in bx.0..bx.1 {
            for rj in by.0..by.1 {
                out[cj * nr + rj].push(i);
            }
        }
    }
    out
}

/// O intervalo semi-aberto `[lo, hi)` de bandas que contêm a coordenada
/// normalizada `t`, dada uma partição de `n` bandas crescidas de meia banda
/// para cada lado.
fn band_range(t: f32, n: usize) -> (usize, usize) {
    let x = t.clamp(0.0, 1.0) * n as f32;
    // j − ½ ≤ x ≤ j + 1 + ½  ⟺  x − 1,5 ≤ j ≤ x + 0,5
    let lo = (x - 1.5).ceil().max(0.0) as usize;
    let hi = (((x + 0.5).floor() + 1.0).max(0.0) as usize).min(n);
    (lo.min(hi), hi)
}

#[cfg(test)]
#[path = "layout_tests.rs"]
mod layout_tests;
