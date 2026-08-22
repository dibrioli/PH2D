//! **O QUE A MONTAGEM DIZ** — a recusa, o relatório e a proveniência de cada ponto.
//!
//! Irmão da [`crate::stitch`], e o corte foi **forçado pelo teto de LOC** (o pai
//! chegou a 714 contra 700). ⭐ **Mas ele é de ASSUNTO e não de conveniência:** lá
//! mora *como* a malha se monta; aqui, *o que se pode dizer sobre ela* — e é este
//! ficheiro que a auditoria de 2026-08-21 mostrou ser o mais importante dos dois.
//!
//! ⚠️ **A lição que mora aqui:** todo campo do [`FillReport`] menos os dois de
//! aresta é **função pura dos ÍNDICES**. Uma malha com as posições embaralhadas
//! devolve o relatório **byte-idêntico** — foi assim que 10.515 gates ficaram
//! verdes sobre um produto destruído.

use ph2d_mesh::Mesh;

/// Por que a malha não pôde ser montada.
// ⚠️ Deixou de ser `Eq` quando a `ArcNotOfThisMesh` passou a carregar os dois
// comprimentos: sem eles, a recusa diria *que* não bate e não **quanto**, e a
// diferença entre `1,0001×` e `5,40×` é a diferença entre ruído e catástrofe.
#[derive(Debug, Clone, PartialEq)]
pub enum FillError {
    /// ⚠️ **A lei do patch não bate com a quantização.** `L_i` tinha de ser
    /// `e_{i-1} + e_{i+1}`, e não é. Isto é **bug a montante**, não uma
    /// propriedade da malha: o F4 devolve `e` que satisfazem a lei por
    /// construção, logo ou os lados vieram fora de ordem ou o `e` é de outro
    /// patch. Recusar em vez de remendar é o que impede uma malha torcida.
    Mismatch {
        /// Qual patch.
        patch: usize,
        /// Qual lado.
        side: usize,
        /// O que a lei exigia.
        expected: u32,
        /// O que os arcos somaram.
        got: u32,
    },
    /// Um lado não emenda no seguinte — a fronteira do patch não fecha.
    ///
    /// ⚠️ **Ela carrega os VÉRTICES desde 2026-08-21**, e não só os índices: com
    /// `patch: 3, side: 0` e nada mais, a recusa diz *que* não fecha e não **onde**
    /// — e a diferença entre "o lado acaba num vértice que o seguinte não começa" e
    /// "um dos dois está vazio" manda procurar em sítios diferentes.
    Broken {
        /// Qual patch.
        patch: usize,
        /// Qual lado.
        side: usize,
        /// Onde este lado acaba.
        ends_at: Option<u32>,
        /// Onde o seguinte começa.
        next_starts_at: Option<u32>,
        /// Quantos lados o patch tem.
        sides: usize,
    },
    /// A malha resultante não monta.
    Mesh(String),
    /// ⭐⭐ **O LAYOUT NÃO É DESTA MALHA** — o defeito que destruiu o produto em
    /// 2026-08-21, e que nenhum dos 10.515 gates conseguia ver.
    ///
    /// ⚠️ **É a pré-condição mais barata que existe**, e ela existe porque o
    /// sintoma é invisível a jusante: um `arc_chain` de outra malha produz uma
    /// saída com **topologia perfeita** — 100 % quads, característica de Euler
    /// exacta, zero arestas de bordo, contagem de irregulares idêntica — e
    /// geometria destruída. *Nenhum número do [`FillReport`] muda.*
    ///
    /// A régua: o comprimento da polilinha de cada arco, medido **na malha que se
    /// vai amostrar**, tem de bater com o `arc_length` que o F3 declarou e que o
    /// F4 já usou para decidir quantos segmentos aquele arco leva. Medido: no
    /// caminho coerente a razão é **1,000 exacto** (é a mesma soma dos mesmos
    /// `f32`); no caminho destruído foi **5,40×**, com o pior arco a **9,04×**.
    /// *Três ordens de grandeza de margem — não há flake possível.*
    ArcNotOfThisMesh {
        /// Qual arco.
        arc: usize,
        /// O comprimento que o F3 declarou.
        declared: f32,
        /// O que a malha recebida de facto mede — ou `None` se um índice do arco
        /// nem sequer existe nela.
        measured: Option<f32>,
    },
}

/// O que a montagem mediu.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FillReport {
    /// Quantos quads.
    pub quads: usize,
    /// Quantas faces que **não** são quads. ⭐ Tem de ser **zero** — é a promessa
    /// inteira desta família de algoritmos.
    pub non_quads: usize,
    /// Quantos vértices.
    pub verts: usize,
    /// ⭐ Quantos vértices **irregulares** (valência ≠ 4). É a grandeza que o
    /// artista vê, e a que o pivô existiu para derrubar: a família local entregava
    /// 21 a 49 %, o oráculo entrega 0,2 %.
    pub irregular: usize,
    /// ⚠️ Quantas arestas ficaram com **uma** face só. Tem de ser zero numa
    /// superfície fechada: é o instrumento que denuncia a malha rasgada.
    pub boundary_edges: usize,
    /// Quantas rondas de alisamento correram.
    pub smoothing: usize,
    /// ⚠️ Quantas faces tiveram de ser **invertidas** para o volume ficar
    /// positivo. É `0` ou `todas` — qualquer outro número seria orientação
    /// inconsistente, que é outro defeito.
    pub flipped: usize,
    /// ⭐ **DE ONDE vêm os irregulares** — ver [`Provenance`]. Sem esta
    /// decomposição, `irregular: 47` diz que há trabalho e não diz **em que
    /// fase**: um irregular num canto do layout é dívida do F3, um no centro de
    /// um patch é da valência que o F3 entregou, e um no interior de uma grade
    /// seria um bug desta crate.
    pub by_provenance: [usize; Provenance::COUNT],
    /// ⭐⭐ **A ARESTA MAIS LONGA da saída** — e ela é a primeira grandeza
    /// GEOMÉTRICA que este relatório alguma vez teve.
    ///
    /// ⛔ **Todo o resto deste struct é função pura dos ÍNDICES.** `quads`,
    /// `non_quads`, `boundary_edges` e `irregular` saem da combinatória das faces;
    /// uma malha com as posições embaralhadas dá exactamente os mesmos números.
    /// Foi assim que 10.515 gates ficaram verdes sobre um produto destruído
    /// (auditoria de 2026-08-21): *não existia uma única asserção que olhasse uma
    /// coordenada.*
    ///
    /// A régua do chamador é a razão para o alvo dele. Medido: caminho correcto
    /// **≤ 4× o alvo**; caminho destruído **18×** — que era o **diâmetro da peça**,
    /// uma aresta a atravessar a esfera de lado a lado.
    pub edge_max: f32,
    /// A aresta mediana. ⭐ É a que diz se a DENSIDADE saiu no alvo — a máxima diz
    /// se alguma coisa se partiu, esta diz se a grade tem o passo pedido.
    pub edge_median: f32,
    /// ⭐⭐ **QUANTOS PATCHES ACHATARAM** — ver [`crate::param`].
    ///
    /// ⚠️ **É a régua que diz se a cura CHEGOU ao patch que dói.** Um patch que não
    /// achata volta à construção antiga — a que interpola em `ℝ³` e agarra à face
    /// mais próxima —, e ela dobra. Sem esta contagem, *"as dobras não caíram"* não
    /// distingue *"a cura não funciona"* de *"a cura não correu"*.
    pub flattened: usize,
    /// Quantos patches o layout tinha — o denominador do [`Self::flattened`].
    pub patches: usize,
    /// ⭐⭐ **QUANTOS PONTOS DE INTERIOR CAÍRAM FORA do achatamento** e tiveram de
    /// usar o caminho antigo.
    ///
    /// ⚠️ **Sem ele, `flattened: 19/19` mente por omissão.** Um patch pode achatar e
    /// mesmo assim não colocar ponto nenhum pelo domínio — basta o `uv` de cada
    /// ponto cair fora de todo triângulo. *Uma contagem de FASES não substitui uma
    /// contagem de PONTOS* ([[feedback_a_defect_count_without_provenance_names_the_wrong_phase]]).
    pub sample_misses: usize,
    /// Quantos pontos de interior o domínio de facto colocou.
    pub sampled: usize,
    /// ⚠️ **O pior resíduo com que um achatamento parou.** O teorema de Tutte fala
    /// da solução **convergida**; uma iteração parada a meio continua dentro do
    /// polígono mas pode ter triângulos virados. *Se este número não for pequeno, a
    /// garantia não se aplica.*
    pub flatten_residual: f32,
    /// Quantas rondas o achatamento mais caro gastou.
    pub flatten_rounds: usize,
    /// ⭐⭐ **QUANTAS FACES DOBRARAM** — ver [`folded_against`], que é quem a mede.
    ///
    /// ⚠️ **É o defeito que o artista fotografa**, e é geométrico: as fendas
    /// escuras de 2026-08-21 são faces cuja normal aponta para o lado oposto ao da
    /// superfície por baixo delas. Nenhum outro campo deste relatório a vê — uma
    /// malha com 100 % de quads, casca fechada e a contagem certa de irregulares
    /// pode estar cheia delas.
    pub folded: usize,
    /// ⭐⭐ **A SEGUNDA régua** — ver [`folded_by_neighbours`]. Ela não consulta a
    /// referência, então o pescoço fino não a confunde. ⚠️ **As duas juntas é que
    /// são a régua**: esta é cega a uma malha inteiramente ao contrário, aquela tem
    /// piso de ruído onde a superfície se dobra sobre si mesma.
    pub folded_local: usize,
    /// ⭐⭐ **DE QUE FASE são os vértices das faces DOBRADAS** — ver [`Provenance`].
    ///
    /// ⚠️ **É a régua que impede arranjar a fase errada.** `folded: 18` diz que há
    /// trabalho e **não diz onde**: uma dobra entre pontos de `Grid` é da construção
    /// do interior; uma entre pontos de `Arc` é do TRAÇADO, e nenhuma mudança na
    /// construção lhe toca. *Uma contagem de defeitos sem proveniência nomeia a fase
    /// errada.*
    pub folded_prov: [usize; Provenance::COUNT],
    /// ⭐⭐ **DE QUE FASE são as pontas das arestas LONGAS** (acima de `3×` a
    /// mediana) — ver [`Provenance`].
    ///
    /// ⚠️ **É a régua que diz até onde o slider pode ir.** Medido em 2026-08-21: ao
    /// pedir `15×` mais quads a mediana fica em `1,03×` o alvo — a densidade está
    /// certa — e a **máxima** vai de `2,71×` a `8,14×`. *Uma grandeza que não
    /// acompanha o alvo tem um dono, e sem esta decomposição procura-se por
    /// eliminação.*
    pub edge_long_prov: [usize; Provenance::COUNT],
    /// ⭐⭐⭐ **A FORMA DE CADA QUAD** — ver [`QuadShape`] e [`quad_shape`].
    ///
    /// ⛔ **Ela entrou em 2026-08-22, depois de a quarta foto do artista vir com a
    /// palavra «péssimo» sobre uma malha que passava em TODAS as réguas deste
    /// struct** — incluindo [`Self::edge_max`], que nesse mesmo dia tinha caído de
    /// `57 %` da peça para `5,5 %`.
    ///
    /// ⚠️ **Todas as outras grandezas geométricas daqui são GLOBAIS**: a aresta mais
    /// longa da malha, a mediana de todas as arestas. *Um quad de `0,02 × 0,30` não
    /// move nenhuma das duas* — a longa dele está muito abaixo da máxima e a curta
    /// afunda-se na mediana de dezenas de milhares. E o defeito da foto é exactamente
    /// esse: faces esmagadas em faixas, numa malha cujos extremos estão bem.
    pub shape: crate::shape::QuadShape,
}

/// **QUANTAS FACES DA SAÍDA APONTAM CONTRA A SUPERFÍCIE POR BAIXO DELAS.**
///
/// Para cada face da saída, acha a face da `reference` cujo centróide está mais
/// perto e pergunta se as duas normais concordam. ⭐ **A referência é o oráculo da
/// orientação**: `out.face_normals()` sozinho não responde nada, porque uma malha
/// inteiramente ao contrário é consistente consigo mesma.
///
/// ⛔⛔ **NÃO é o teste radial, e a diferença apanhou-me em 2026-08-21.** O teste
/// radial — *"a normal concorda com o raio a partir do centro da caixa?"* — só é
/// válido num sólido **estrelado**, e a fixtura do diagnóstico é justamente a que
/// não é: uma esfera com um BICO longo tem a barriga do gancho a apontar para
/// longe do centro **de forma legítima**. Medido lado a lado nessa peça, a mesma
/// malha: o teste radial acusou **17 faces (6,5 %)** e o motor local — que a
/// literatura e os nossos próprios gates dizem não dobrar — foi acusado de
/// **1,6 %** pelo mesmo instrumento. *Um detector que acusa a testemunha de
/// controlo está a medir a forma, não o defeito.*
///
/// ⚠️ **O raio de busca DOBRA até achar alguém**, com teto em `64×` a semente. Um
/// raio fixo devolve `usize::MAX` num quad grande sobre uma zona rala da
/// referência — e uma face sem vizinho não conta como dobrada, o que faria a
/// contagem **descer** exactamente onde a malha está pior.
#[must_use]
pub fn folded_against(reference: &Mesh, out: &Mesh) -> usize {
    let ref_normals = reference.face_normals();
    let rb = reference.bounds();
    let seed = norm(sub(rb.max, rb.min)) * 0.05;
    let pos = out.positions();
    let mut hits: Vec<u32> = Vec::new();
    let mut folded = 0usize;
    for f in out.faces() {
        let v = f.verts();
        let c = centroid(pos, v);
        let mut best = (f32::INFINITY, usize::MAX);
        let mut radius = seed;
        while best.1 == usize::MAX && radius < seed * 64.0 {
            reference.octree().faces_in_sphere(c, radius, &mut hits);
            for &fi in &hits {
                let rv = reference.faces()[fi as usize].verts();
                let d = norm(sub(centroid(reference.positions(), rv), c));
                if d < best.0 {
                    best = (d, fi as usize);
                }
            }
            radius *= 2.0;
        }
        if let Some(&rn) = ref_normals.get(best.1) {
            let n = face_normal(pos, v);
            if n[0].mul_add(rn[0], n[1].mul_add(rn[1], n[2] * rn[2])) < 0.0 {
                folded += 1;
            }
        }
    }
    folded
}

/// **QUANTO A GRADE OBEDECE AO RELEVO** — o desvio médio, em graus, entre cada
/// aresta da saída e a direção principal de curvatura da peça ali.
///
/// Devolve `(graus, confiança_média)`.
///
/// ⭐⭐ **É a régua que faltava para o report *"sem nenhuma obediência ao
/// relevo"*.** Ela não pergunta se a malha é bonita: pergunta se as arestas correm
/// ao longo da direção em que a superfície dobra — que é a promessa inteira de uma
/// retopologia por campo cruzado, e o que a distingue de um voxel remesh.
///
/// ⚠️ **O desvio é 4-RoSy**, dobrado em `[0°, 45°]`: uma grade rodada 90° está
/// **alinhada**, e medir o ângulo cru daria `90°` a um resultado perfeito.
///
/// ⭐ **A média é PONDERADA PELA ANISOTROPIA**, e sem isso a régua é ruído: numa
/// esfera as duas curvaturas são iguais, não há direção preferida, e o desvio ali é
/// uma coordenada aleatória. *Pesar pela confiança é o que faz a régua falar só
/// onde a forma tem o que dizer.*
///
/// ⚠️ **O ponto de comparação é `22,5°`** — a média de um ângulo uniforme em
/// `[0°, 45°]`, ou seja **uma grade que ignora o relevo por completo**. Um número
/// perto disso não é *"um pouco desalinhado"*: é *"não olhou"*.
#[must_use]
pub fn follows_relief(reference: &Mesh, out: &Mesh) -> (f32, f32) {
    let dirs = ph2d_mesh::principal_dirs(reference);
    let ref_normals = reference.face_normals();
    let rb = reference.bounds();
    let seed = norm(sub(rb.max, rb.min)) * 0.02;
    let pos = out.positions();
    let mut hits: Vec<u32> = Vec::new();
    let (mut wsum, mut asum, mut count) = (0.0f64, 0.0f64, 0usize);
    // Cada aresta da saída, uma vez.
    let mut seen: std::collections::BTreeSet<(u32, u32)> = std::collections::BTreeSet::new();
    for f in out.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            if !seen.insert((a.min(b), a.max(b))) {
                continue;
            }
            let (p, q) = (pos[a as usize], pos[b as usize]);
            let mid = [
                (p[0] + q[0]) * 0.5,
                (p[1] + q[1]) * 0.5,
                (p[2] + q[2]) * 0.5,
            ];
            let Some(rf) = nearest_face(reference, mid, seed, &mut hits) else {
                continue;
            };
            let pd = dirs[rf];
            if pd.anisotropy <= 0.0 {
                count += 1;
                continue;
            }
            // A aresta, projectada no plano tangente da face de referência.
            let n = ref_normals[rf];
            let e = sub(q, p);
            let along = e[0].mul_add(n[0], e[1].mul_add(n[1], e[2] * n[2]));
            let t = [
                along.mul_add(-n[0], e[0]),
                along.mul_add(-n[1], e[1]),
                along.mul_add(-n[2], e[2]),
            ];
            let lt = norm(t);
            if lt < 1.0e-9 {
                continue;
            }
            let c = (t[0].mul_add(pd.dir[0], t[1].mul_add(pd.dir[1], t[2] * pd.dir[2])) / lt)
                .clamp(-1.0, 1.0);
            // ⭐ Dobrado em `[0°, 45°]`: `|cos|` mata o sentido, e o `45 − |45 − x|`
            // mata a rotação de 90°.
            let deg = c.abs().acos().to_degrees();
            let folded = 45.0 - (45.0 - deg).abs();
            wsum += f64::from(folded) * f64::from(pd.anisotropy);
            asum += f64::from(pd.anisotropy);
            count += 1;
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let confidence = if count == 0 { 0.0 } else { asum / count as f64 };
    #[allow(clippy::cast_possible_truncation)]
    let deg = if asum > 0.0 {
        (wsum / asum) as f32
    } else {
        45.0
    };
    #[allow(clippy::cast_possible_truncation)]
    (deg, confidence as f32)
}

/// A face da `mesh` cujo centróide está mais perto de `p` — o raio DOBRA até
/// achar alguém. ⚠️ Porta única: três réguas deste ficheiro faziam a mesma busca,
/// e uma delas com um raio inicial diferente.
fn nearest_face(mesh: &Mesh, p: [f32; 3], seed: f32, hits: &mut Vec<u32>) -> Option<usize> {
    let mut best = (f32::INFINITY, usize::MAX);
    let mut radius = seed;
    while best.1 == usize::MAX && radius < seed * 64.0 {
        mesh.octree().faces_in_sphere(p, radius, hits);
        for &fi in hits.iter() {
            let rv = mesh.faces()[fi as usize].verts();
            let d = norm(sub(centroid(mesh.positions(), rv), p));
            if d < best.0 {
                best = (d, fi as usize);
            }
        }
        radius *= 2.0;
    }
    (best.1 != usize::MAX).then_some(best.1)
}

/// **QUANTO DA FORMA SE PERDEU** — a distância de cada vértice da referência ao
/// ponto mais próximo da saída, em fração da diagonal. Devolve `(p95, máx)`.
///
/// ⭐⭐ **O SENTIDO é o ponto inteiro, e o contrário é uma régua TAUTOLÓGICA.** A
/// última coisa que a montagem faz é pousar cada ponto na referência, então
/// `saída → referência` dá ~zero **mesmo quando a peça está destruída** — foi
/// exactamente isso que uma régua desta família mediu em 2026-08-21 (`0,0000` na
/// malha destruída contra `0,0015` na boa: *a destruída pontuava melhor*).
///
/// ⭐ **`referência → saída` pergunta a coisa certa:** *"todo pedaço que o artista
/// esculpiu tem malha nova por perto?"*. Uma orelha achatada deixa os vértices dela
/// longe de tudo, e o número dispara — mesmo com 100 % de quads e casca fechada.
///
/// ⚠️ **O `p95` vem primeiro porque é ele que descreve a peça.** O máximo é de um
/// vértice, e um único pico de importação move-o sem que nada se tenha perdido.
#[must_use]
pub fn detail_lost(reference: &Mesh, out: &Mesh) -> (f32, f32) {
    let b = reference.bounds();
    let d = sub(b.max, b.min);
    let diag = norm(d).max(1.0e-9);
    let seed = diag * 0.02;
    let mut worst: Vec<f32> = Vec::with_capacity(reference.vert_count());
    for &p in reference.positions() {
        let q = ph2d_remesh_iso::project_onto(out, p, seed);
        worst.push(norm(sub(q, p)) / diag);
    }
    worst.sort_by(f32::total_cmp);
    if worst.is_empty() {
        return (0.0, 0.0);
    }
    let k = (worst.len() * 95) / 100;
    (worst[k.min(worst.len() - 1)], worst[worst.len() - 1])
}

/// **QUANTAS FACES DISCORDAM DOS PRÓPRIOS VIZINHOS** — a segunda régua, e ela
/// **não consulta a referência**.
///
/// ⭐⭐ **Ela existe porque a primeira não é limpa em toda peça.** A
/// [`folded_against`] pergunta à face da referência mais próxima, e num **bico
/// fino** — uma fita da superfície dobrada sobre si mesma — a face mais próxima
/// pode estar do outro lado do pescoço. Medido em 2026-08-21 na `hooked_sphere`:
/// a própria malha remalhada isotropicamente, que **não tem dobra nenhuma**, é
/// acusada de **24 faces em 3 566 (0,67 %)**. *Um piso de ruído de 0,7 % debaixo
/// de um sinal de 7 % ainda deixa o sinal, mas não se optimiza contra uma régua sem
/// se conhecer o piso dela.*
///
/// Esta pergunta outra coisa: a normal de cada face contra a **média das faces que
/// partilham uma aresta com ela**. Uma face virada discorda dos vizinhos por
/// construção, e ⭐ **nenhuma vizinhança entra em jogo através do espaço** — a
/// adjacência é combinatória, então o pescoço fino não a confunde.
///
/// ⚠️ **Ela é cega ao que a outra vê**: uma malha inteiramente ao contrário
/// concorda consigo mesma e passa aqui. *As duas juntas é que são a régua; nenhuma
/// sozinha.*
#[must_use]
pub fn folded_by_neighbours(mesh: &Mesh) -> usize {
    folded_faces_by_neighbours(mesh).len()
}

/// **QUAIS** faces discordam dos vizinhos — a lista, para quem precisa de saber
/// **onde** e não só **quantas**.
///
/// ⭐ **É o que separa *"a cura não funciona"* de *"a cura não é desta fase"*.** Uma
/// contagem de dobras sem a proveniência dos vértices delas manda arranjar a fase
/// errada — foi assim que uma parametrização por patch inteira foi construída,
/// medida e **rejeitada**: as dobras não estavam no interior das grades.
#[must_use]
pub fn folded_faces_by_neighbours(mesh: &Mesh) -> Vec<u32> {
    let faces = mesh.faces();
    let pos = mesh.positions();
    let normals: Vec<[f32; 3]> = faces.iter().map(|f| face_normal(pos, f.verts())).collect();
    // Aresta -> as faces que a usam.
    let mut by_edge: std::collections::BTreeMap<(u32, u32), Vec<u32>> =
        std::collections::BTreeMap::new();
    for (i, f) in faces.iter().enumerate() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            by_edge
                .entry((a.min(b), a.max(b)))
                .or_default()
                .push(u32::try_from(i).unwrap_or(0));
        }
    }
    let mut nb: Vec<Vec<u32>> = vec![Vec::new(); faces.len()];
    for who in by_edge.values() {
        if who.len() == 2 {
            nb[who[0] as usize].push(who[1]);
            nb[who[1] as usize].push(who[0]);
        }
    }
    (0..faces.len())
        .filter(|&i| {
            if nb[i].is_empty() {
                return false;
            }
            let mut avg = [0.0f32; 3];
            for &j in &nb[i] {
                let n = normals[j as usize];
                let len = norm(n).max(f32::MIN_POSITIVE);
                for k in 0..3 {
                    avg[k] += n[k] / len;
                }
            }
            let n = normals[i];
            n[0].mul_add(avg[0], n[1].mul_add(avg[1], n[2] * avg[2])) < 0.0
        })
        .map(|i| u32::try_from(i).unwrap_or(0))
        .collect()
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn norm(a: [f32; 3]) -> f32 {
    a[0].mul_add(a[0], a[1].mul_add(a[1], a[2] * a[2])).sqrt()
}

#[allow(clippy::cast_precision_loss)]
fn centroid(pos: &[[f32; 3]], v: &[u32]) -> [f32; 3] {
    let mut c = [0.0f32; 3];
    for &i in v {
        let q = pos[i as usize];
        for k in 0..3 {
            c[k] += q[k] / v.len() as f32;
        }
    }
    c
}

/// A normal de uma face, pelo primeiro triângulo dela.
fn face_normal(pos: &[[f32; 3]], v: &[u32]) -> [f32; 3] {
    let (p0, p1, p2) = (
        pos[v[0] as usize],
        pos[v[1] as usize],
        pos[v[2 % v.len()] as usize],
    );
    let (u, w) = (sub(p1, p0), sub(p2, p0));
    [
        u[1].mul_add(w[2], -(u[2] * w[1])),
        u[2].mul_add(w[0], -(u[0] * w[2])),
        u[0].mul_add(w[1], -(u[1] * w[0])),
    ]
}

/// **De onde um vértice da saída veio** — a chave para saber de quem é a dívida.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Provenance {
    /// Um **canto do layout**: onde três ou mais arcos se encontram. A valência
    /// dele é o número de arcos, e ⭐ **é o F3 que a decide** — cada junção em T
    /// que o traçado cria é um canto a mais.
    Corner,
    /// O interior de um **arco** partilhado. Deviam ser todos regulares.
    Arc,
    /// O **centro** de um patch. A valência é a do patch, logo um patch de 3 ou 5
    /// lados produz aqui um irregular **por construção** — é o preço do leque.
    Center,
    /// O interior de um **raio** do leque, do centro ao corte de um lado.
    Spoke,
    /// O interior de uma **grade** de Coons. ⛔ Um irregular aqui seria bug desta
    /// crate: uma grade regular não tem nenhum.
    Grid,
}

impl Provenance {
    /// Quantas classes existem.
    pub const COUNT: usize = 5;
    /// Os nomes, na ordem do array de [`FillReport::by_provenance`].
    pub const NAMES: [&'static str; Self::COUNT] =
        ["canto (F3)", "arco", "centro (F3)", "raio", "grade"];
}

/// **OS PONTOS DA SAÍDA, com a origem de cada um.**
///
/// ⭐ **Existe para que a posição e a proveniência não possam divergir.** A
/// primeira versão eram dois `Vec` a crescer lado a lado com um comentário a pedir
/// que assim continuassem — e um `push` esquecido num dos cinco sítios daria uma
/// decomposição deslocada, que **soma certo** e atribui a dívida à fase errada.
/// Aqui há um único `push`, e ele exige as duas coisas.
pub(crate) struct Points {
    pub(crate) pos: Vec<[f32; 3]>,
    pub(crate) prov: Vec<Provenance>,
}

impl Points {
    pub(crate) fn new() -> Self {
        Self {
            pos: Vec::new(),
            prov: Vec::new(),
        }
    }

    /// Acrescenta um ponto e devolve o índice dele.
    pub(crate) fn push(&mut self, p: [f32; 3], from: Provenance) -> u32 {
        self.pos.push(p);
        self.prov.push(from);
        u32::try_from(self.pos.len() - 1).unwrap_or(u32::MAX)
    }

    /// **Acrescenta um ponto POUSADO na superfície.**
    ///
    /// ⭐⭐ **É a diferença entre construir a grade NO ESPAÇO e construí-la SOBRE a
    /// forma**, e ela vale mais do que qualquer alisamento posterior.
    ///
    /// ⛔ **A primeira versão interpolava tudo em linha reta e deixava a
    /// reprojecção para o alisamento no fim.** Numa esfera de raio 1,0 a corda de
    /// um raio de leque mergulha para dentro, o Coons construído sobre cordas
    /// mergulhadas fica pior ainda, e as faces **dobram sobre si mesmas** —
    /// exactamente as fendas escuras que o Enio fotografou em 2026-08-21.
    ///
    /// Medido nessa esfera (4 922 quads), faces dobradas contra rondas de
    /// alisamento: `0 → 405 · 1 → 403 · 3 → 289 · 6 → 205 · 12 → 135`. ⭐ **O
    /// alisamento REPARA e não CAUSA** — ele nunca chega a zero porque o estrago
    /// já veio pronto da construção. *Um remédio que melhora monotonicamente e não
    /// cura está a tratar o sintoma.*
    /// ⭐⭐ **E ele leva a DIREÇÃO de que lado o ponto veio.** Ver
    /// [`ph2d_remesh_iso::project_facing`]: dentro de um vinco côncavo o ponto mais
    /// próximo pode estar do **outro lado** da dobra — o eixo medial encosta na
    /// superfície —, e a face entre dois vizinhos assim aterrados vira uma lasca.
    /// `None` é o caminho antigo, e continua a ser o certo onde a direção seria uma
    /// **estimativa** em vez de um facto (ver o alisamento em [`crate::stitch`]).
    pub(crate) fn push_facing(
        &mut self,
        mesh: &Mesh,
        p: [f32; 3],
        seed: f32,
        facing: Option<[f32; 3]>,
        from: Provenance,
    ) -> u32 {
        self.push(ph2d_remesh_iso::project_facing(mesh, p, seed, facing), from)
    }
}
