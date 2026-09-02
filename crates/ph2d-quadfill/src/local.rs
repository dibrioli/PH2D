//! ⭐⭐⭐ **A RÉGUA LOCAL — a forma de UM quad no espaço, e ONDE ele está.**
//!
//! Irmã da [`crate::shape`] por RESPONSABILIDADE, e a fronteira é a pergunta:
//! aquela mede **os cantos** de cada face (aspecto e enviesamento) e resume em
//! percentis; esta mede o que os cantos **não podem ver** — se o quad é plano, se
//! ele se dobra sobre si próprio, e se a face aponta ao contrário da vizinha.
//!
//! ⛔⛔⛔ **Ela existe por um report que NENHUMA régua desta linha via** (Enio,
//! 2026-08-29/30, duas fotos e três setas: *«buracos nas pontas, faces emboladas
//! nas pontas»*). Na mesma peça, o botão relatava **`χ = 2` · `0` bordo · `0`
//! não-manifold · `1` ilha · `100 %` de quads · `>60° = 0`** — *toda* régua verde.
//!
//! ⚠️⚠️ **E é por isso que a palavra «buraco» do report não pode ser lida como
//! bordo:** não há bordo nenhum naquela malha. Um quad **virado ao contrário** do
//! vizinho renderiza pelo lado de dentro e lê-se, num viewport sombreado,
//! exactamente como um furo — e uma **gravata** (o quad que se auto-intersecta)
//! tem os quatro cantos com ângulos perfeitamente bons. *As duas passam em
//! `QuadShape`, em `χ`, no bordo e no não-manifold.*
//!
//! ⚠️ **O outro meio da régua é a LOCALIZAÇÃO.** O defeito dele é espacial — «nas
//! pontas» —, e uma contagem global não responde a isso: três faces más numa ponta
//! não movem uma mediana de milhares ([`crate::shape`] mede medianas e percentis).
//! Por isso [`LocalShape`] traz a **fracção radial** de cada defeito: `1,0` é a
//! ponta, `0,0` é o centro da peça.

use ph2d_mesh::{Face, Mesh};

/// **O QUE UM QUAD É, no plano dele** — ver [`FaceLocal::kind`].
///
/// ⚠️ **A classificação é por CONTAGEM DE SINAL, não por um limiar**: em cada
/// canto, o produto vectorial das duas arestas que o formam projecta-se na normal
/// de Newell do polígono. Num quad convexo os quatro dão o mesmo sinal.
///
/// ⭐ **`Bowtie` é o caso que nenhuma outra régua apanha:** ele tem exactamente
/// **dois** cantos discordantes, área quase nula (as duas metades cancelam-se) e
/// **ângulos de canto normais** — logo passa em aspecto, em enviesamento e no
/// `>60°`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuadKind {
    /// Os quatro cantos concordam.
    Convex,
    /// Um canto reentrante. É uma forma legítima, só feia.
    Concave,
    /// ⛔ Auto-intersecta — a face «embolada».
    Bowtie,
}

/// **A FORMA DE UMA FACE no espaço** — ver [`local_shape`].
#[derive(Debug, Clone, Copy)]
pub struct FaceLocal {
    /// ⭐ **Torção**: o ângulo, em graus, entre as normais das duas metades do
    /// quad. `0` é plano.
    ///
    /// ⚠️⚠️ **É o MÁXIMO sobre as DUAS diagonais — e a razão que este doc dava
    /// era FALSA.** Ele afirmava que *«um quad em sela é plano ao longo de uma
    /// diagonal e torcido ao longo da outra»*; uma prova de mutação sobreviveu a
    /// trocar o `max` por uma diagonal só, e a medição explicou porquê: **quatro
    /// pontos ou são coplanares ou não são**, então as duas diagonais acusam
    /// *sempre* as duas. Medido: sela `109,47 / 109,47`, canto levantado
    /// `63,20 / 70,25`, assimétrico `68,67 / 60,19` — **nenhuma leitura perto de
    /// zero**, e a razão entre elas fica em `1,14`.
    ///
    /// ⭐ **O `max` fica, com a razão CERTA:** ele torna o número independente da
    /// diagonal que alguém escolheu. Um renderizador triangula o quad numa das
    /// duas e não diz qual; medir só uma daria dois valores diferentes para a
    /// mesma face conforme quem pergunta, e o maior é a leitura **conservadora**.
    /// O gate que o defende é `a_torcao_nao_depende_da_diagonal_escolhida`.
    pub warp_deg: f32,
    /// O que a face é no plano dela.
    pub kind: QuadKind,
    /// **Área ÷ (aresta média)²** — `1,0` é um quadrado, `0` é degenerada.
    ///
    /// ⚠️ Uma face degenerada é indistinguível de um furo no ecrã, e tem
    /// aspecto e enviesamento **bem definidos e bons** até ao momento em que
    /// colapsa.
    pub squareness: f32,
    /// **Onde ela está**: distância do centroide da face ao centroide da peça,
    /// dividida pela maior dessas distâncias. `1,0` é a ponta.
    pub radial: f32,
}

impl FaceLocal {
    /// ⚠️ **O que conta como DEFEITO, numa porta só.** Escrever esta condição em
    /// cada consumidor daria duas respostas à mesma pergunta, e a que diverge é a
    /// que nenhum gate dirige.
    #[must_use]
    pub fn is_defect(&self) -> bool {
        self.kind == QuadKind::Bowtie
            || self.warp_deg > WARP_DEFECT_DEG
            || self.squareness < SQUARENESS_DEFECT
    }
}

/// ⭐ **A barra da torção**, em graus.
///
/// ⚠️ **Ela NÃO é escolhida por gosto: é o ângulo a partir do qual as duas metades
/// de um quad deixam de poder ser sombreadas como uma superfície só.** Um
/// renderizador triangula o quad numa diagonal e interpola a normal; com as duas
/// metades a `45°` uma da outra, a interpolação atravessa `45°` dentro de uma face
/// — mais do que a maior curvatura que a peça tem entre faces VIZINHAS. ⛔ Acima
/// disto a face não é uma aproximação má da superfície: ela **deixa de descrever
/// uma superfície**.
pub const WARP_DEFECT_DEG: f32 = 45.0;

/// ⭐ **A barra da degenerescência.** Um quadrado dá `1,0`; um rectângulo `4:1` dá
/// `0,64`; abaixo de `0,10` a face tem menos de um décimo da área que as arestas
/// dela prometem — é uma lasca, e no ecrã é um vinco ou um vazio.
pub const SQUARENESS_DEFECT: f32 = 0.10;

/// ⭐ **Onde começa «a ponta»**, em fracção do raio máximo.
///
/// ⚠️ **Este número é da RÉGUA, não do produto** — ele só decide o que a linha
/// `nas pontas` do relatório conta. Mudá-lo não muda malha nenhuma.
pub const TIP_FRACTION: f32 = 0.75;

/// **O RESUMO** — ver [`local_shape`].
#[derive(Debug, Clone, Default)]
pub struct LocalShape {
    /// Quantas faces se auto-intersectam.
    pub bowties: usize,
    /// Quantas passam de [`WARP_DEFECT_DEG`].
    pub warped: usize,
    /// Quantas ficam abaixo de [`SQUARENESS_DEFECT`].
    pub slivers: usize,
    /// A pior torção da malha, em graus.
    pub warp_max: f32,
    /// Torção — percentil 99. ⚠️ *Ao lado do máximo de propósito*: um `warp_max`
    /// alto com `p99` baixo é **um** defeito localizado, que é precisamente o que
    /// uma mediana esconde.
    pub warp_p99: f32,
    /// ⭐⭐ **Quantas das faces defeituosas estão na PONTA** (radial ≥
    /// [`TIP_FRACTION`]). É esta a coluna que responde ao report do artista.
    pub defects_at_tip: usize,
    /// Quantas faces defeituosas há no total.
    pub defects: usize,
    /// Quantas faces vivem na ponta, defeituosas ou não — o **denominador** de
    /// `defects_at_tip`.
    ///
    /// ⚠️ **Sem ele a coluna acima não é interpretável**: «12 defeitos na ponta»
    /// significa coisas opostas se a ponta tem 20 faces ou 2 000.
    pub faces_at_tip: usize,
}

/// **MEDE A FORMA LOCAL DE CADA FACE** — ver [`LocalShape`] e [`FaceLocal`].
///
/// Devolve o resumo **e** a régua por face, porque quem quer curar precisa de saber
/// *qual* face, e quem quer relatar precisa do número.
#[must_use]
pub fn local_shape(mesh: &Mesh) -> (LocalShape, Vec<FaceLocal>) {
    local_shape_of(mesh.positions(), mesh.faces())
}

/// A mesma lei sobre PARTES — o idioma de [`crate::shape::quad_shape_of`], pela
/// mesma razão (uma relaxação não pode publicar a malha a cada ronda).
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn local_shape_of(pos: &[[f32; 3]], faces: &[Face]) -> (LocalShape, Vec<FaceLocal>) {
    let mut per_face: Vec<FaceLocal> = Vec::with_capacity(faces.len());
    if faces.is_empty() {
        return (LocalShape::default(), per_face);
    }

    // ⚠️ **O centro é a média dos CENTROIDES DE FACE, não dos vértices.** Uma
    // ponta densa em vértices puxaria o centro para si, e a fracção radial de todas
    // as outras faces mudaria por causa da malha — não da forma.
    let centroids: Vec<[f32; 3]> = faces.iter().map(|f| centroid(pos, f)).collect();
    let mut c = [0.0f64; 3];
    for p in &centroids {
        for k in 0..3 {
            c[k] += f64::from(p[k]);
        }
    }
    let n = centroids.len() as f64;
    let center = [(c[0] / n) as f32, (c[1] / n) as f32, (c[2] / n) as f32];
    let dists: Vec<f32> = centroids.iter().map(|p| dist(*p, center)).collect();
    let far = dists.iter().copied().fold(0.0f32, f32::max).max(1.0e-9);

    for (i, f) in faces.iter().enumerate() {
        let v = f.verts();
        let kind = classify(pos, v);
        per_face.push(FaceLocal {
            warp_deg: warp_of(pos, v),
            kind,
            squareness: squareness_of(pos, v),
            radial: dists[i] / far,
        });
    }

    let mut warps: Vec<f32> = per_face.iter().map(|d| d.warp_deg).collect();
    warps.sort_by(f32::total_cmp);
    let p99 = {
        let i = ((warps.len() - 1) as f32 * 0.99).round() as usize;
        warps[i.min(warps.len() - 1)]
    };

    let mut s = LocalShape {
        warp_max: warps[warps.len() - 1],
        warp_p99: p99,
        ..LocalShape::default()
    };
    for d in &per_face {
        if d.kind == QuadKind::Bowtie {
            s.bowties += 1;
        }
        if d.warp_deg > WARP_DEFECT_DEG {
            s.warped += 1;
        }
        if d.squareness < SQUARENESS_DEFECT {
            s.slivers += 1;
        }
        let tip = d.radial >= TIP_FRACTION;
        if tip {
            s.faces_at_tip += 1;
        }
        if d.is_defect() {
            s.defects += 1;
            if tip {
                s.defects_at_tip += 1;
            }
        }
    }
    (s, per_face)
}

/// A normal de **Newell** — a única que faz sentido num polígono não-plano, porque
/// é a soma das áreas projectadas e não depende de que diagonal alguém escolheu.
fn newell(pos: &[[f32; 3]], v: &[u32]) -> [f32; 3] {
    let mut n = [0.0f32; 3];
    for k in 0..v.len() {
        let a = pos[v[k] as usize];
        let b = pos[v[(k + 1) % v.len()] as usize];
        n[0] += (a[1] - b[1]) * (a[2] + b[2]);
        n[1] += (a[2] - b[2]) * (a[0] + b[0]);
        n[2] += (a[0] - b[0]) * (a[1] + b[1]);
    }
    n
}

/// ⭐⭐ **A classificação por contagem de sinal** — ver [`QuadKind`].
///
/// ⚠️ **Vale para qualquer valência**, e a leitura é a mesma: `0` discordantes é
/// convexo, `1` é um canto reentrante, e `2` ou mais é auto-intersecção.
fn classify(pos: &[[f32; 3]], v: &[u32]) -> QuadKind {
    let n = newell(pos, v);
    let len = n[0].mul_add(n[0], n[1].mul_add(n[1], n[2] * n[2])).sqrt();
    if len <= 1.0e-20 {
        // ⚠️ Área projectada nula: as duas metades cancelaram-se exactamente. É a
        // assinatura de uma gravata perfeita, não de um polígono bom.
        return QuadKind::Bowtie;
    }
    let mut neg = 0usize;
    for k in 0..v.len() {
        let prev = pos[v[(k + v.len() - 1) % v.len()] as usize];
        let cur = pos[v[k] as usize];
        let next = pos[v[(k + 1) % v.len()] as usize];
        let e0 = sub(cur, prev);
        let e1 = sub(next, cur);
        if dot(cross(e0, e1), n) < 0.0 {
            neg += 1;
        }
    }
    match neg {
        0 => QuadKind::Convex,
        1 => QuadKind::Concave,
        _ => QuadKind::Bowtie,
    }
}

/// A torção — o maior ângulo entre as normais das duas metades, sobre as **duas**
/// diagonais. Ver [`FaceLocal::warp_deg`] para *por que* o máximo.
fn warp_of(pos: &[[f32; 3]], v: &[u32]) -> f32 {
    let (a, b) = warp_splits(pos, v);
    a.max(b)
}

/// ⭐ **As DUAS leituras, expostas** — a diagonal `0–2` e a `1–3`.
///
/// ⚠️ **Ela existe para o gate poder afirmar a desigualdade**: a lei que
/// [`FaceLocal::warp_deg`] declara é *«o número não depende da diagonal
/// escolhida»*, e isso não é observável de fora sem ver as duas. *Uma lei que só
/// se vê pelo resultado colapsado não tem gate — foi assim que a primeira versão
/// sobreviveu a uma mutação que trocava o `max` por uma diagonal só.*
pub(crate) fn warp_splits(pos: &[[f32; 3]], v: &[u32]) -> (f32, f32) {
    if v.len() != 4 {
        return (0.0, 0.0);
    }
    let p: Vec<[f32; 3]> = v.iter().map(|&i| pos[i as usize]).collect();
    let split = |a: usize, b: usize, c: usize, d: usize| -> f32 {
        let n1 = cross(sub(p[b], p[a]), sub(p[c], p[a]));
        let n2 = cross(sub(p[c], p[a]), sub(p[d], p[a]));
        angle_between(n1, n2)
    };
    (split(0, 1, 2, 3), split(1, 2, 3, 0))
}

/// Área ÷ (aresta média)².
fn squareness_of(pos: &[[f32; 3]], v: &[u32]) -> f32 {
    let mut perim = 0.0f32;
    for k in 0..v.len() {
        perim += dist(pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
    }
    #[allow(clippy::cast_precision_loss)]
    let mean = perim / v.len() as f32;
    if mean <= 1.0e-9 {
        return 0.0;
    }
    let n = newell(pos, v);
    let area = 0.5 * n[0].mul_add(n[0], n[1].mul_add(n[1], n[2] * n[2])).sqrt();
    area / (mean * mean)
}

fn angle_between(a: [f32; 3], b: [f32; 3]) -> f32 {
    let la = a[0].mul_add(a[0], a[1].mul_add(a[1], a[2] * a[2])).sqrt();
    let lb = b[0].mul_add(b[0], b[1].mul_add(b[1], b[2] * b[2])).sqrt();
    if la <= 1.0e-20 || lb <= 1.0e-20 {
        return 0.0;
    }
    (dot(a, b) / (la * lb)).clamp(-1.0, 1.0).acos().to_degrees()
}

fn centroid(pos: &[[f32; 3]], f: &Face) -> [f32; 3] {
    let v = f.verts();
    let mut c = [0.0f32; 3];
    for &i in v {
        let p = pos[i as usize];
        for k in 0..3 {
            c[k] += p[k];
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let n = v.len() as f32;
    [c[0] / n, c[1] / n, c[2] / n]
}

pub(crate) fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = sub(a, b);
    d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
}

pub(crate) fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub(crate) fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

pub(crate) fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1].mul_add(b[2], -(a[2] * b[1])),
        a[2].mul_add(b[0], -(a[0] * b[2])),
        a[0].mul_add(b[1], -(a[1] * b[0])),
    ]
}

#[cfg(test)]
#[path = "local_tests.rs"]
mod tests;

/// ⭐⭐⭐ **A RAZÃO PONTA/CORPO de um campo escalar sobre pontos** — a lei que
/// responde *«a ponta é mais fina que o corpo?»*, e ela tem **dois** consumidores
/// de propósito.
///
/// ⚠️⚠️ **É a mesma pergunta feita ao PEDIDO e à ENTREGA**, e é por isso que ela
/// vive numa porta: o campo de passo que a cadeia *pede* é um valor por **vértice
/// da malha de trabalho**, e o que ela *entrega* é a raiz da área por **face da
/// saída** — domínios diferentes, lei igual. *Medi-los com duas funções daria dois
/// números que ninguém pode dividir um pelo outro.*
///
/// ⚠️ **O «corpo» é a casca MAIS POVOADA, nunca uma escolhida.** Numa bola de
/// espinhos as cascas do meio têm milhares de pontos e a de fora dezenas; dividir
/// por uma casca fixa mede a forma da peça em vez da densidade.
///
/// Devolve `(razão, quantos pontos na casca da ponta)` — ⚠️ **a contagem vai
/// junto**: uma mediana de 12 amostras não é uma medição, e sem o denominador ao
/// lado ninguém sabe distinguir as duas.
///
/// ⛔⛔ **`n == 0` significa NÃO MEDIDO, e a razão vem `0,0` — que se lê como o
/// melhor resultado possível.** Achado por auditoria em 2026-08-30: quando todos os
/// pontos estão à MESMA distância do centro (um ponto só, ou uma peça sem relevo
/// nenhum) eles caem todos na casca `0` e a casca da ponta fica vazia. *Um zero de
/// «não medido» e um zero de «perfeito» são o mesmo byte* — a lição que esta linha
/// já pagou nas duas réguas de valência. ⇒ **quem imprime tem de olhar a contagem
/// antes do número**, e há gate a exigi-lo.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn tip_body_ratio(points: &[[f32; 3]], values: &[f32]) -> (f32, usize) {
    const SHELLS: usize = 5;
    if points.is_empty() || points.len() != values.len() {
        return (0.0, 0);
    }
    let mut c = [0.0f64; 3];
    for p in points {
        for k in 0..3 {
            c[k] += f64::from(p[k]);
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let n = points.len() as f64;
    let mid = [(c[0] / n) as f32, (c[1] / n) as f32, (c[2] / n) as f32];
    let d: Vec<f32> = points.iter().map(|p| dist(*p, mid)).collect();
    let far = d.iter().copied().fold(0.0f32, f32::max).max(1.0e-9);

    let mut shells: Vec<Vec<f32>> = vec![Vec::new(); SHELLS];
    for (i, v) in values.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let s = ((d[i] / far) * SHELLS as f32)
            .floor()
            .min((SHELLS - 1) as f32) as usize;
        shells[s].push(*v);
    }
    for s in &mut shells {
        s.sort_by(f32::total_cmp);
    }
    let med = |s: &Vec<f32>| if s.is_empty() { 0.0 } else { s[s.len() / 2] };
    let body = shells.iter().max_by_key(|s| s.len()).map_or(0.0, med);
    let tip = med(&shells[SHELLS - 1]);
    let ratio = if body > 1.0e-9 { tip / body } else { 0.0 };
    (ratio, shells[SHELLS - 1].len())
}
