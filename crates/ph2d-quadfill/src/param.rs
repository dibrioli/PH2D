//! **A PARAMETRIZAÇÃO DE UM PATCH** — a grade deixa de ser construída no ESPAÇO e
//! passa a ser construída **na superfície**.
//!
//! # ⛔ O que ela substitui, e por que a substituição é de espécie
//!
//! O Coons interpola as quatro **curvas de bordo** e devolve um ponto no espaço
//! `ℝ³` que, em geral, **não está na peça**; o remédio era agarrá-lo à face mais
//! próxima ([`ph2d_remesh_iso::project_onto`]). Sobre um patch grande e curvo —
//! uma esfera com um gancho — o ponto interpolado mergulha para dentro da forma e
//! o ponto mais próximo dele fica **do outro lado**. A face entre dois vizinhos
//! assim aterrados vira: é a fenda escura que o artista fotografa.
//!
//! ⚠️ **Medido em 2026-08-21: o alisamento REPARA e não CAUSA** (`0 → 25 dobras`,
//! `6 → 17`, `24 → 18`), e trocar a superfície da projeção não move o número
//! (`17 → 15`). *Um remédio que melhora monotonicamente e estagna está a tratar o
//! sintoma.*
//!
//! # A construção certa, e a garantia que ela traz
//!
//! 1. Achata-se o patch — as suas faces, que são faces **da malha** — sobre um
//!    polígono convexo, pelo **embutimento baricêntrico de Tutte** (1963): a
//!    fronteira vai para o polígono, e cada vértice interior fica na média dos
//!    vizinhos. ⭐ **Com pesos positivos e fronteira convexa, o embutimento é
//!    garantidamente SEM DOBRAS** — é o teorema, não uma esperança.
//! 2. A grade é construída no **domínio**, onde ela é uma grade de um quadrado.
//! 3. Cada ponto dela volta pela triangulação achatada e aterra **exactamente
//!    sobre uma face do patch**.
//!
//! ⇒ nenhum ponto interior nasce fora da peça, e nenhum ponto de um patch aterra
//! no vizinho. *A projeção deixa de ser a forma de corrigir a construção e passa a
//! ser só o passo que troca a malha remalhada pela original.*
//!
//! ⚠️ **Clean-room e sem dono:** Tutte (1963) e a interpolação transfinita são
//! matemática clássica. Nenhuma linha vem de fonte GPL — ver ADR-0161.

use std::collections::BTreeMap;

use ph2d_mesh::Mesh;

/// ⚠️ **Quantas rondas de Gauss–Seidel o achatamento corre no máximo.**
///
/// ⭐ **Ele é um teto de ESPERA e não de qualidade**, e o número saiu da medição e
/// não de uma opinião: a sonda `how_flat_does_the_patch_get` imprime, por patch, em
/// quantas rondas o resíduo desce abaixo de [`FLATTEN_TOL`]. ⛔ Quem o atingir sai
/// com um achatamento **válido** (Tutte só exige pesos positivos e fronteira
/// convexa, que a iteração preserva a cada passo) e apenas menos relaxado.
const FLATTEN_ROUNDS: usize = 4_000;

/// O resíduo abaixo do qual o achatamento pára — em unidades do domínio, que tem
/// raio `1`. ⚠️ `1e-5` é meia parte por 100 000 da peça: abaixo disso o movimento
/// já não muda a face em que um ponto de grade cai.
const FLATTEN_TOL: f32 = 1.0e-5;

/// **UM PATCH ACHATADO** — a triangulação dele com uma coordenada 2D por vértice.
pub(crate) struct PatchParam {
    /// Por vértice local, o ponto no domínio.
    uv: Vec<[f32; 2]>,
    /// Por vértice local, a posição 3D na malha que o layout indexa.
    pos: Vec<[f32; 3]>,
    /// Os triângulos, em índices locais.
    tris: Vec<[u32; 3]>,
    /// Balde uniforme sobre `[-1,1]²`, para localizar o triângulo de um `uv`.
    bucket: Vec<Vec<u32>>,
    /// Quantas células por lado.
    side: usize,
    /// Quantas rondas o achatamento gastou — ⭐ é o que a sonda lê.
    pub(crate) rounds: usize,
    /// O resíduo com que ele parou.
    pub(crate) residual: f32,
}

/// O domínio vai de `-1` a `1` nos dois eixos (o polígono é inscrito no círculo
/// unitário; o quadrado usa os quatro cantos `(±1, ±1)/√2`).
const LO: f32 = -1.0;
const SPAN: f32 = 2.0;

impl PatchParam {
    /// **ACHATA UM PATCH.**
    ///
    /// `faces` são os índices, na `mesh`, das faces deste patch. `boundary` é a
    /// fronteira dele **por lado**: `boundary[i]` é a cadeia ordenada de vértices
    /// de malha do lado `i`, do canto de entrada ao de saída (o último vértice de
    /// um lado é o primeiro do seguinte).
    ///
    /// Devolve `None` quando o patch não é um disco triangulável — e ⚠️ **`None` é
    /// uma resposta e não uma falha**: o chamador volta à construção antiga, que
    /// dobra mas devolve malha.
    pub(crate) fn build(
        mesh: &Mesh,
        faces: &[u32],
        boundary: &[Vec<u32>],
        tau: &[Vec<f32>],
    ) -> Option<Self> {
        let n = boundary.len();
        if n < 3 || faces.is_empty() {
            return None;
        }
        // ── Índices locais, em ordem determinística.
        let mut local: BTreeMap<u32, u32> = BTreeMap::new();
        let mut tris: Vec<[u32; 3]> = Vec::with_capacity(faces.len());
        let mesh_faces = mesh.faces();
        for &f in faces {
            let v = mesh_faces.get(f as usize)?.verts();
            // ⚠️ Um leque sobre o vértice 0 basta: as faces desta fase são
            // triângulos (o F1 tria a malha) e um quad convexo parte em dois.
            for k in 1..v.len() - 1 {
                let mut t = [0u32; 3];
                for (slot, &g) in t.iter_mut().zip([v[0], v[k], v[k + 1]].iter()) {
                    let next = u32::try_from(local.len()).ok()?;
                    *slot = *local.entry(g).or_insert(next);
                }
                if t[0] != t[1] && t[1] != t[2] && t[2] != t[0] {
                    tris.push(t);
                }
            }
        }
        let pos_src = mesh.positions();
        let mut pos = vec![[0.0f32; 3]; local.len()];
        for (&g, &l) in &local {
            pos[l as usize] = *pos_src.get(g as usize)?;
        }

        // ── A fronteira vai para o polígono convexo, por COMPRIMENTO DE ARCO.
        //
        // ⚠️ **Por comprimento e não por contagem**, e é a mesma lei do
        // [`crate::fan::resample`]: a cadeia de malha tem arestas de tamanhos
        // diferentes, e distribuir por contagem faria o domínio herdar a densidade
        // da triangulação em vez da geometria do lado. *É isto que faz o `uv` de um
        // ponto de saída — que o F5 amostrou por comprimento — bater com o `uv` dos
        // vértices de malha à volta dele.*
        let mut uv = vec![[0.0f32; 2]; local.len()];
        let mut fixed = vec![false; local.len()];
        let poly = corners_for(n);
        for (i, chain) in boundary.iter().enumerate() {
            let (a, b) = (poly[i], poly[(i + 1) % n]);
            // ⭐⭐ **Pelo `τ` do layout, a MESMA régua que o [`crate::patch::side_uv`]
            // usa para os pontos de saída.** Sem graduação ele é o comprimento de
            // arco e nada muda; com ela, um vértice de malha e um ponto de saída na
            // mesma posição do arco recebem o mesmo `uv`. *Duas réguas aqui dariam
            // uma grade torcida junto ao bordo, sem nada a acusar.*
            let side_tau = tau.get(i)?;
            if side_tau.len() != chain.len() {
                return None;
            }
            let total: f32 = side_tau.last().copied().unwrap_or(0.0);
            for (k, &g) in chain.iter().enumerate() {
                let run = side_tau[k];
                let f = if total > 0.0 { run / total } else { 0.0 };
                let l = *local.get(&g)? as usize;
                // ⚠️ O último vértice de um lado é o primeiro do seguinte; escrever
                // os dois dá o mesmo canto, então a ordem não importa.
                uv[l] = [f.mul_add(b[0] - a[0], a[0]), f.mul_add(b[1] - a[1], a[1])];
                fixed[l] = true;
            }
        }
        if fixed.iter().all(|&f| f) {
            // Um patch sem interior nenhum: o achatamento é a fronteira, e ele
            // continua a servir para localizar pontos.
            let mut me = Self::with(uv, pos, tris);
            me.rounds = 0;
            return Some(me);
        }

        // ── Tutte: cada interior na média PONDERADA dos vizinhos, por Gauss–Seidel.
        //
        // ⭐⭐ **Os pesos são as COORDENADAS DE VALOR MÉDIO** (Floater 2003), e a
        // troca não é cosmética. Medido em 2026-08-21 com pesos **uniformes** — o
        // Tutte de manual — na `hooked_sphere`: o embutimento é válido (nenhuma
        // dobra nasce dele) e **distorcidíssimo**, e a grade construída sobre ele
        // saiu com aresta máxima **4,80×** o alvo contra 3,62× da construção antiga,
        // sem baixar as dobras. *Um mapa válido não é um mapa útil: um passo igual
        // no domínio tem de valer um passo parecido na superfície.*
        //
        // ⚠️ **E elas mantêm a garantia**: `tan(α/2) + tan(β/2)` sobre `|pᵢ − pⱼ|` é
        // **sempre positivo** num triângulo, e o teorema de Tutte só exige pesos
        // positivos com fronteira convexa. *Cotangente seria harmónico e admite peso
        // negativo num triângulo obtuso — e é aí que a garantia se perde.*
        let nb = mean_value_weights(&tris, &pos);
        // ⚠️ Um interior sem vizinho nenhum não tem média: ele não pertence a
        // triângulo nenhum deste patch, e o achatamento não o pode colocar.
        if (0..local.len()).any(|v| !fixed[v] && nb[v].is_empty()) {
            return None;
        }
        let (mut rounds, mut residual) = (0usize, f32::INFINITY);
        for r in 0..FLATTEN_ROUNDS {
            let mut worst = 0.0f32;
            for v in 0..local.len() {
                if fixed[v] {
                    continue;
                }
                let (mut sx, mut sy, mut sw) = (0.0f32, 0.0f32, 0.0f32);
                for &(w, k) in &nb[v] {
                    sx += k * uv[w as usize][0];
                    sy += k * uv[w as usize][1];
                    sw += k;
                }
                if sw <= 0.0 {
                    continue;
                }
                let inv = 1.0 / sw;
                let next = [sx * inv, sy * inv];
                worst = worst.max((next[0] - uv[v][0]).abs().max((next[1] - uv[v][1]).abs()));
                uv[v] = next;
            }
            rounds = r + 1;
            residual = worst;
            if worst < FLATTEN_TOL {
                break;
            }
        }
        let mut me = Self::with(uv, pos, tris);
        me.rounds = rounds;
        me.residual = residual;
        Some(me)
    }

    fn with(uv: Vec<[f32; 2]>, pos: Vec<[f32; 3]>, tris: Vec<[u32; 3]>) -> Self {
        // ⚠️ **O balde é dimensionado pelo número de triângulos**, não por uma
        // constante: um patch de 20 faces e um de 20 000 pedem grelhas diferentes,
        // e uma célula com metade do patch dentro faz a localização voltar a ser
        // linear.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        #[allow(clippy::cast_precision_loss)]
        let side = ((tris.len() as f32).sqrt().ceil() as usize).clamp(1, 128);
        let mut bucket = vec![Vec::new(); side * side];
        for (i, t) in tris.iter().enumerate() {
            let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
            for &v in t {
                let p = uv[v as usize];
                for k in 0..2 {
                    lo[k] = lo[k].min(p[k]);
                    hi[k] = hi[k].max(p[k]);
                }
            }
            let (c0, c1) = (cell(lo, side), cell(hi, side));
            for cy in c0[1]..=c1[1] {
                for cx in c0[0]..=c1[0] {
                    bucket[cy * side + cx].push(u32::try_from(i).unwrap_or(0));
                }
            }
        }
        Self {
            uv,
            pos,
            tris,
            bucket,
            side,
            rounds: 0,
            residual: 0.0,
        }
    }

    /// **DE VOLTA À SUPERFÍCIE** — o ponto 3D sobre o triângulo que contém `q`.
    ///
    /// ⚠️ **Procura na célula do balde e depois nas oito à volta.** Um `uv` sobre a
    /// aresta partilhada por dois triângulos cai no primeiro que o aceitar, e os
    /// dois devolvem o mesmo ponto — a interpolação baricêntrica é contínua
    /// através da aresta.
    pub(crate) fn sample(&self, q: [f32; 2]) -> Option<([f32; 3], [f32; 3])> {
        let c = cell(q, self.side);
        for ring in 0..=1i64 {
            for dy in -ring..=ring {
                for dx in -ring..=ring {
                    if ring > 0 && dx.abs() < ring && dy.abs() < ring {
                        continue;
                    }
                    let (x, y) = (c[0] as i64 + dx, c[1] as i64 + dy);
                    if x < 0 || y < 0 || x >= self.side as i64 || y >= self.side as i64 {
                        continue;
                    }
                    #[allow(clippy::cast_sign_loss)]
                    for &t in &self.bucket[y as usize * self.side + x as usize] {
                        if let Some(p) = self.at(t as usize, q) {
                            return Some(p);
                        }
                    }
                }
            }
        }
        None
    }

    /// O ponto 3D e a **NORMAL do triângulo** em que ele caiu.
    ///
    /// ⭐⭐ **A normal viaja com o ponto porque a reprojeção precisa dela.** Ver
    /// [`ph2d_remesh_iso::project_facing`]: dentro de um vinco côncavo o ponto mais
    /// próximo da superfície original pode estar do **outro lado** da dobra, e é
    /// isso que rasga a malha. O ponto sabe de que lado veio — ele nasceu sobre uma
    /// face concreta da malha achatada —, e essa informação estava a ser deitada
    /// fora uma linha antes de quem precisava dela.
    fn at(&self, t: usize, q: [f32; 2]) -> Option<([f32; 3], [f32; 3])> {
        let tri = self.tris[t];
        let (a, b, c) = (
            self.uv[tri[0] as usize],
            self.uv[tri[1] as usize],
            self.uv[tri[2] as usize],
        );
        let area = edge(a, b, c);
        if area.abs() <= f32::EPSILON {
            return None;
        }
        let inv = 1.0 / area;
        let w = [
            edge(q, b, c) * inv,
            edge(a, q, c) * inv,
            edge(a, b, q) * inv,
        ];
        // ⚠️ A folga é do TAMANHO DO DOMÍNIO e não relativa ao triângulo: um `uv`
        // exactamente sobre a fronteira do patch tem de ser aceite por alguém, e o
        // arredondamento de `f32` num domínio de raio 1 vive perto de `1e-7`.
        if w.iter().any(|&x| x < -1.0e-4) {
            return None;
        }
        let mut p = [0.0f32; 3];
        for (k, &v) in tri.iter().enumerate() {
            let s = self.pos[v as usize];
            for j in 0..3 {
                p[j] = w[k].mul_add(s[j], p[j]);
            }
        }
        let (p0, p1, p2) = (
            self.pos[tri[0] as usize],
            self.pos[tri[1] as usize],
            self.pos[tri[2] as usize],
        );
        let (e1, e2) = (
            [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]],
            [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]],
        );
        let n = [
            e1[1].mul_add(e2[2], -(e1[2] * e2[1])),
            e1[2].mul_add(e2[0], -(e1[0] * e2[2])),
            e1[0].mul_add(e2[1], -(e1[1] * e2[0])),
        ];
        Some((p, n))
    }
}

/// **AS COORDENADAS DE VALOR MÉDIO** (Floater, 2003) — por vértice, a lista de
/// `(vizinho, peso)`.
///
/// `w_ij = (tan(α/2) + tan(β/2)) / |pᵢ − pⱼ|`, onde `α` e `β` são os ângulos em
/// `pᵢ` nos dois triângulos que partilham a aresta `ij`. ⭐ **Sempre positivo**, o
/// que preserva a garantia de Tutte, e ⭐ **reproduz funções lineares**, que é o que
/// o peso uniforme não faz e é por isso que ele distorce.
///
/// ⚠️ **A soma é por CANTO e não por aresta.** A mesma aresta aparece nos dois
/// triângulos vizinhos, e cada um contribui com o seu `tan(α/2)`; somar por aresta
/// contaria o ângulo errado.
fn mean_value_weights(tris: &[[u32; 3]], pos: &[[f32; 3]]) -> Vec<Vec<(u32, f32)>> {
    let mut acc: Vec<BTreeMap<u32, f32>> = vec![BTreeMap::new(); pos.len()];
    for t in tris {
        for k in 0..3 {
            let (i, a, b) = (
                t[k] as usize,
                t[(k + 1) % 3] as usize,
                t[(k + 2) % 3] as usize,
            );
            let (p, q, r) = (pos[i], pos[a], pos[b]);
            // O ângulo em `p` entre `q` e `r` — o `α` do canto.
            let (u, v) = (sub(q, p), sub(r, p));
            let (lu, lv) = (norm3(u), norm3(v));
            if lu <= 0.0 || lv <= 0.0 {
                continue;
            }
            let cos = (dot3(u, v) / (lu * lv)).clamp(-1.0, 1.0);
            // `tan(θ/2) = (1 − cos θ) / sin θ`, e a forma `sin/(1+cos)` é a estável
            // perto de `θ = 0`, que é o caso comum num triângulo fino.
            let half = (1.0 - cos * cos).max(0.0).sqrt() / (1.0 + cos).max(1.0e-12);
            *acc[i].entry(t[(k + 1) % 3]).or_insert(0.0) += half / lu;
            *acc[i].entry(t[(k + 2) % 3]).or_insert(0.0) += half / lv;
        }
    }
    acc.into_iter()
        .map(|m| m.into_iter().filter(|&(_, w)| w > 0.0).collect())
        .collect()
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

fn norm3(a: [f32; 3]) -> f32 {
    dot3(a, a).sqrt()
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// **OS CANTOS DO POLÍGONO** — o regular de `n` lados, inscrito no círculo unitário.
///
/// ⭐ **Ela é a fonte única dos cantos** — o achatamento e o `uv` dos pontos de
/// saída ([`crate::stitch`]) chamam-na com o mesmo `n`. *Duas contas do mesmo canto
/// dariam um ponto de fronteira cujo domínio discorda do da triangulação.*
///
/// ⛔⛔ **O polígono PROPORCIONAL AO COMPRIMENTO DOS LADOS foi construído, medido e
/// REJEITADO** (2026-08-21). O raciocínio era bom — um patch de lados `1` e `5`
/// achatado num quadrado tem um esticão embutido, e um passo igual no domínio vale
/// cinco vezes mais de um lado que do outro. Mas inscrever os cantos no círculo com
/// **ângulo proporcional ao comprimento** dá um polígono que pode ser quase
/// degenerado, e o embutimento esmaga triângulos até a localização de ponto falhar:
///
/// | fixtura | regular | proporcional |
/// |---|---|---|
/// | esfera 48×72 | **2,1 %** dobras, `0` pontos fora | ⛔ 13,2 %, **313** fora |
/// | esfera 24×36 | 5,5 %, `0` fora | 5,4 %, **62** fora |
/// | esfera esculpida | **0,9 %**, `0` fora | ⛔ 1,7 %, **95** fora |
///
/// *Um domínio sem esticão que não se consegue amostrar é pior que um esticado que
/// se consegue.* ⚠️ **A distorção continua a ser um problema real** — ela é o que
/// sobra na coluna `raio`/`grade` da proveniência —, mas a cura tem de manter os
/// triângulos amostráveis. *A próxima hipótese não é o polígono; é o mapa.*
#[must_use]
pub(crate) fn corners_for(n: usize) -> Vec<[f32; 2]> {
    (0..n)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let a = std::f32::consts::TAU * i as f32 / n as f32;
            [a.cos(), a.sin()]
        })
        .collect()
}

fn cell(p: [f32; 2], side: usize) -> [usize; 2] {
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    let f = |v: f32| ((((v - LO) / SPAN) * side as f32).floor().max(0.0) as usize).min(side - 1);
    [f(p[0]), f(p[1])]
}

fn edge(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
    (b[0] - a[0]).mul_add(c[1] - a[1], -((c[0] - a[0]) * (b[1] - a[1])))
}
