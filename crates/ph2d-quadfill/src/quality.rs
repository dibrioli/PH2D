//! ⭐⭐ **A QUALIDADE DA SAÍDA CONTRA A SUPERFÍCIE** — dobras, relevo e detalhe.
//!
//! ⚠️ **Irmão do [`crate::report`] pelo teto de LOC (HR-18, 700) e por ASSUNTO:**
//! lá o que a montagem **diz de si** (a recusa, as contagens, a proveniência de cada
//! ponto); aqui as quatro grandezas que só existem por **comparação com a peça
//! original**. *São elas que sobrevivem a uma malha com as posições embaralhadas —
//! todo o resto do relatório é função pura dos índices.*

use ph2d_mesh::Mesh;

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

/// ⭐⭐⭐ **A DIREÇÃO QUE A SUPERFÍCIE PREFERE, no centro de cada face da saída** — o
/// que o acabamento precisa de saber para não desalinhar a grade enquanto a endireita.
///
/// ⚠️ **É a MESMA fonte que a régua [`follows_relief`] usa** ([`ph2d_mesh::principal_dirs`]),
/// e isso é deliberado: o acabamento passa a mirar exactamente aquilo por que é julgado, em
/// vez de o estragar por acidente. ⛔ *Não é mirar a régua* — a direção principal é também o
/// que o campo cruzado (F2) persegue, então as duas metades da cadeia passam a querer a
/// mesma coisa. A régua continua a poder reprovar: ela mede **arestas**, isto orienta
/// **faces**, e o peso abaixo faz a maior parte das faces quase não ser puxada.
///
/// ⭐ **O peso é a ANISOTROPIA**, em `[0, 1]`, tal como ela sai da estimativa: numa esfera
/// as duas curvaturas são iguais, não há direção preferida, e o peso `0` devolve a lei do
/// quadrado puro. *Um alinhamento sem confiança põe costura onde a forma não pede nenhuma.*
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Hint {
    /// A direção principal, em espaço de MUNDO.
    pub dir: [f32; 3],
    /// Quanto ela vale, em `[0, 1]` — ver [`ph2d_mesh::PrincipalDir::anisotropy`].
    pub weight: f32,
}

/// Amostra [`Hint`] no centróide de cada face de `out`. ⚠️ **Uma vez, não por ronda:** o
/// campo de direções é liso e o acabamento move os vértices uma fração de aresta, então
/// re-amostrar por ronda pagaria a busca `N` vezes para mudar o alvo quase nada.
#[must_use]
pub fn surface_hint(surface: &Mesh, out: &Mesh) -> Vec<Hint> {
    let dirs = ph2d_mesh::principal_dirs(surface);
    let rb = surface.bounds();
    let seed = norm(sub(rb.max, rb.min)) * 0.02;
    let pos = out.positions();
    let mut hits: Vec<u32> = Vec::new();
    out.faces()
        .iter()
        .map(|f| {
            let c = centroid(pos, f.verts());
            let Some(rf) = nearest_face(surface, c, seed, &mut hits) else {
                return Hint::default();
            };
            let pd = dirs[rf];
            Hint {
                dir: pd.dir,
                weight: pd.anisotropy,
            }
        })
        .collect()
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
