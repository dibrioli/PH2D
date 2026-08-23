//! ⭐⭐⭐ **O INTERIOR DE UM PATCH SEGUE O CAMPO** — a cura do defeito que o artista
//! chamou «péssimo» (2026-08-22), atacada onde a medição a pôs.
//!
//! # ⛔ O que estava errado, medido
//!
//! O [`crate::param`] achata o patch por **Tutte**: a fronteira vai para o polígono
//! e cada vértice interior fica na média ponderada dos vizinhos. ⚠️ **Isso é um mapa
//! harmónico — ele conhece a FRONTEIRA e mais nada.** As linhas da grade no interior
//! saem de interpolar as curvas de bordo, e é isso que enviesa:
//!
//! | | só a família `u` | ⭐ **as duas famílias** |
//! |---|---|---|
//! | oráculo, gancho | `5,1°` | **`7,6°`** — mal se move |
//! | ⛔ nós, gancho | `9,9°` | ⛔ **`19,2°`** — mais do dobro |
//!
//! *A nossa 1.ª família de linhas segue o campo; a 2.ª não fica ortogonal a ela.* É a
//! assinatura da interpolação transfinita: ela casa com a fronteira e **enviesa no
//! meio**. ⛔ E uma relaxação não cura isto — dezasseis rondas de ajuste de quadrado
//! levaram o enviesamento mediano de `27°` para `26°` (ver [`crate::relax`]).
//!
//! # ⭐ A lei: o campo entra pelo LADO DIREITO da mesma iteração
//!
//! Em vez de pedir que `uv` seja a média dos vizinhos, pede-se que **cada passo até
//! um vizinho valha o que o campo diz que ele vale**:
//!
//! ```text
//!     minimizar   Σ  w_ij · | (uv_j − uv_i) − c·d_ij |²
//!                i~j
//! ```
//!
//! com `d_ij` = o deslocamento `p_j − p_i` **lido na moldura da cruz** e `c` uma
//! similaridade global. A equação normal dá:
//!
//! ```text
//!     uv_i = Σ w_ij (uv_j − c·d_ij) / Σ w_ij
//! ```
//!
//! ⭐⭐ **Com `d = 0` isto é EXACTAMENTE a lei de hoje**, termo a termo — é por isso
//! que ligar o campo não pode partir a garantia de Tutte por acidente, e é por isso
//! que o gate de inércia é trivial de escrever.
//!
//! ⭐ **Os pesos continuam os de VALOR MÉDIO**, que são sempre positivos. ⛔ A
//! formulação de manual usaria o Laplaciano cotangente (é o gradiente exacto da
//! energia de Dirichlet), e ele **admite peso negativo num triângulo obtuso** — é
//! aí que o teorema de Tutte deixa de valer. *Preferimos o operador que preserva a
//! garantia e uma direita aproximada a um gradiente exacto sem garantia nenhuma;
//! a rede que apanha a diferença é a contagem de triângulos virados.*
//!
//! # ⚠️ A escala e a rotação NÃO são escolhidas — saem da fronteira
//!
//! O campo diz a **forma** do gradiente; ele não sabe quantas unidades de domínio
//! vale um metro, nem como o eixo `u` do polígono está rodado em relação à cruz.
//! Esses dois números lêem-se de graça: sobre as arestas da **fronteira** (onde o
//! `uv` já está preso) conhecem-se as duas pontas, e a similaridade `c` que melhor
//! as liga sai em forma fechada — a mesma projecção complexa do
//! [`crate::relax::nearest_square`]:
//!
//! ```text
//!     c = Σ conj(d)·Δuv / Σ |d|²
//! ```
//!
//! ⭐ *Nenhuma constante mágica entra neste ficheiro.*
//!
//! # ⚠️ A holonomia, e por que ela é MEDIDA e não assumida
//!
//! Uma cruz tem quatro braços; para somar deslocamentos ao longo de um patch é
//! preciso **pentear** o campo — escolher, face a face, o braço que continua o do
//! vizinho. Isso só é consistente se o patch não contiver singularidade no
//! interior, que é o que o traçado promete (elas ficam nos CANTOS). ⛔ **Promessa
//! não é medição**: [`Aligned::defects`] conta quantas voltas fechadas devolvem o
//! braço rodado, e a sonda imprime-o. *Um patch com singularidade dentro aparece
//! como um inteiro `> 0`, não como uma malha estranha sem explicação.*
//!
//! ⛔⛔ **A primeira versão desta medição estava errada e prescreveu uma obra.** Ela
//! media a **rugosidade** ([`Aligned::rough_deg`]) — limitada a `45°` por construção —
//! e comparava, na aresta que fecha ciclo, o braço **cru** do vizinho em vez do já
//! penteado. Ver `ph2d_crossfield::comb`, onde a mesma falha está datada e provada
//! por mutação.

/// ⭐⭐ **DE ONDE NASCE O INTERIOR DE UM PATCH.**
///
/// ⚠️ **Um `bool` diria «alinhado: sim/não» e o caminho antigo ficaria sem nome.**
/// Aqui os dois têm nome, e o antigo diz o que ele de facto faz — *interpola a
/// fronteira* —, que é exactamente a frase que o diagnóstico de 2026-08-22 pôs em
/// causa. ⛔ Ele fica porque é a testemunha de controlo de toda tabela desta cura,
/// não porque alguém possa querer usá-lo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interior {
    /// ⛔ **O caminho antigo:** o interior sai de interpolar as curvas de bordo
    /// (Tutte harmónico). Enviesa no meio — ver o doc deste módulo.
    FromBoundary,
    /// ⭐ **O interior segue o campo cruzado** que o layout trouxe.
    AlignedToField,
}

/// ⛔⛔ **O QUE SHIPA — e é o caminho ANTIGO, por MEDIÇÃO** (2026-08-23).
///
/// | orelha `d = 1,0` | fronteira | ⭐ campo |
/// |---|---|---|
/// | enviesamento p50 | `27°` | `27°` |
/// | faces `> 60°` | 9 146 | 9 062 |
/// | dobras | 170 | 161 |
/// | detalhe perdido p95 | `0,219 %` | `0,189 %` |
///
/// ⇒ **Melhora três colunas por margens de ruído e não move o alvo.** ⛔ Ligar
/// complexidade por isso seria pagar sem comprar.
///
/// # ⛔⛔⛔ O MECANISMO que esta nota atribuía está REFUTADO (2026-08-23, mesmo dia)
///
/// Durante algumas horas esta linha dizia: *«a holonomia mede 29° a 44° nas três
/// fixturas, logo o campo dentro de um patch não é combável, e a dívida é do **F3**»*.
/// ⛔ **A grandeza que dizia isso não conseguia dizer isso** — era a rugosidade,
/// limitada a `45°`, e no ramo que fecha ciclo comparava o braço cru. Ver o doc deste
/// módulo e `ph2d_crossfield::comb`.
///
/// ⭐⭐⭐ **Com a régua a sério, a resposta inverte-se em duas das três fixturas:**
///
/// | fixtura | patches | ⭐ **com singularidade DENTRO** | ciclos testados | a régua antiga |
/// |---|---|---|---|---|
/// | **orelha** | 17 | ⭐ **0** | 2 154 | `29,3°` |
/// | gancho | 26 | ⛔ **2** (6 voltas, pior `2/4`) | 1 312 | `44,1°` |
/// | enrugada | 14 | ⭐ **0** | 2 207 | `15,6°` |
///
/// ⚠️ **A orelha é a fixtura da tabela de rejeição acima, e ela está LIMPA.** ⇒ o
/// alinhamento correu sobre um campo perfeitamente combável e ainda assim deixou o
/// enviesamento em `27°`. *A não-melhoria é real; a desculpa não era.*
///
/// ⚠️ E repare porque nenhuma barra podia salvar a régua antiga: a enrugada
/// (`15,6°`, limpa) e a orelha (`29,3°`, limpa) ficam **dos dois lados** de qualquer
/// corte que também acuse o gancho.
///
/// # ⇒ O que sobra, e é a quarta vez que a mesma coisa aparece
///
/// A fronteira do patch chega a esta fase **já pregada por comprimento de arco**, e
/// o interior só pode obedecer ao campo *na medida em que a fronteira o deixe*.
/// Quatro curas independentes bateram no mesmo sítio:
///
/// | cura | onde parou |
/// |---|---|
/// | mapa conforme (LSCM) | domínio `1,0° → 21,4°` — a marcação por arco desalinha |
/// | quadrilátero extremal | idem |
/// | re-graduação do arco | emparelha o lado errado; o par certo atravessa a peça |
/// | ⭐ **interior alinhado ao campo** | **campo limpo (medido hoje) e mesmo assim nada** |
///
/// ⭐ **A canalização FICA e é o valor desta jornada:** o campo **chega** ao F5
/// (`PatchLayout::face_dir`) e a holonomia passou a ser mensurável por patch. ⛔ Mas
/// religar isto **já não espera pelo F3** — espera por uma marcação de fronteira que
/// não seja escolhida localmente.
///
/// ⛔⛔ **As 2 do gancho são um defeito REAL do F3 e nunca tinham sido vistas** — uma
/// delas com **meia volta**. Não são a causa desta tabela, mas são obra, e agora têm
/// número.
pub const INTERIOR: Interior = Interior::FromBoundary;

/// Uma cruz tem quatro braços a 90°.
const QUARTER: f32 = std::f32::consts::FRAC_PI_2;

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1].mul_add(b[2], -(a[2] * b[1])),
        a[2].mul_add(b[0], -(a[0] * b[2])),
        a[0].mul_add(b[1], -(a[1] * b[0])),
    ]
}

fn norm(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}

fn unit(a: [f32; 3]) -> Option<[f32; 3]> {
    let l = norm(a);
    (l > 1.0e-12).then(|| [a[0] / l, a[1] / l, a[2] / l])
}

/// A componente de `v` no plano de `n`, normalizada.
fn tangent(v: [f32; 3], n: [f32; 3]) -> Option<[f32; 3]> {
    let d = dot(v, n);
    unit([
        d.mul_add(-n[0], v[0]),
        d.mul_add(-n[1], v[1]),
        d.mul_add(-n[2], v[2]),
    ])
}

/// **O ALVO DE CADA PASSO, no domínio** — ver o doc do módulo.
pub(crate) struct Aligned {
    /// Por vértice local e por **posição em `nb[v]`**, o passo que o campo pede de
    /// `v` até aquele vizinho, já em unidades de domínio.
    ///
    /// ⚠️ **Alinhado com `nb`, não indexado por vértice** — é o que permite ao laço
    /// de Gauss–Seidel ler os dois em paralelo sem uma segunda busca.
    pub(crate) step: Vec<Vec<[f32; 2]>>,
    /// ⛔ **A RUGOSIDADE do campo**, em graus (a pior aresta): o resto depois de virar
    /// cada braço para o quarto de volta mais próximo. **Limitada a 45° por
    /// construção** — ⛔ *não* diz se há singularidade dentro do patch, e foi lida
    /// como se dissesse. Ver `ph2d_crossfield::comb`.
    pub(crate) rough_deg: f32,
    /// ⭐⭐⭐ **A HOLONOMIA: quantas voltas fechadas devolvem o braço RODADO.** `0` = o
    /// patch é combável e uma lei alinhada ao campo tem sentido nele; `> 0` = há
    /// singularidade **dentro** dele, e a dívida é do traçado, não desta fase.
    pub(crate) defects: usize,
}

/// Constrói os alvos. `None` quando o patch não dá para pentear ou a fronteira não
/// determina a similaridade — e ⚠️ **`None` é uma resposta**: o chamador fica com o
/// achatamento harmónico de sempre.
pub(crate) fn build(
    tris: &[[u32; 3]],
    tri_face: &[u32],
    pos: &[[f32; 3]],
    face_dir: &[[f32; 3]],
    nb: &[Vec<(u32, f32)>],
    uv: &[[f32; 2]],
    fixed: &[bool],
) -> Option<Aligned> {
    if tris.is_empty() || tris.len() != tri_face.len() {
        return None;
    }
    // ── A moldura crua de cada triângulo: a normal e a cruz projectada nela.
    let mut normal: Vec<[f32; 3]> = Vec::with_capacity(tris.len());
    let mut raw: Vec<[f32; 3]> = Vec::with_capacity(tris.len());
    for (t, &f) in tris.iter().zip(tri_face) {
        let (a, b, c) = (pos[t[0] as usize], pos[t[1] as usize], pos[t[2] as usize]);
        let n = unit(cross(sub(b, a), sub(c, a)))?;
        let d = tangent(*face_dir.get(f as usize)?, n)?;
        normal.push(n);
        raw.push(d);
    }

    // ── Vizinhança entre TRIÂNGULOS, pela aresta partilhada.
    let mut share: std::collections::BTreeMap<(u32, u32), Vec<usize>> =
        std::collections::BTreeMap::new();
    for (i, t) in tris.iter().enumerate() {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            share.entry((a.min(b), a.max(b))).or_default().push(i);
        }
    }
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); tris.len()];
    for owners in share.values() {
        if owners.len() == 2 {
            adj[owners[0]].push(owners[1]);
            adj[owners[1]].push(owners[0]);
        }
    }

    // ── ⭐ PENTEAR: cada triângulo escolhe o braço da cruz que continua o do pai.
    //
    // ⚠️ **Largura primeiro a partir do triângulo 0**, e a ordem é determinística
    // porque `adj` foi construída a partir de um `BTreeMap`. *Uma travessia
    // dependente de `HashMap` daria campos diferentes em corridas diferentes, e a
    // malha do produto deixaria de ser reproduzível.*
    // ⚠️ **Semente por COMPONENTE.** Um patch cuja adjacência esteja partida (uma
    // ponte de um vértice só) deixaria o resto sem direcção penteada, e a fase
    // seguinte lê `None` como «sem alvo» em vez de «região partida».
    let mut combed: Vec<Option<[f32; 3]>> = vec![None; tris.len()];
    let mut tree: std::collections::BTreeSet<(usize, usize)> = std::collections::BTreeSet::new();
    for seed in 0..tris.len() {
        if combed[seed].is_some() {
            continue;
        }
        combed[seed] = Some(raw[seed]);
        let mut queue = std::collections::VecDeque::from([seed]);
        while let Some(t) = queue.pop_front() {
            let x = combed[t]?;
            for &u in &adj[t] {
                if combed[u].is_some() {
                    continue;
                }
                // A referência: o braço do pai trazido para o plano do vizinho.
                let Some(r) = tangent(x, normal[u]) else {
                    continue;
                };
                let d = raw[u];
                let (c, s) = (dot(d, r), dot(cross(normal[u], d), r));
                // O múltiplo de 90° que leva `d` para junto de `r`.
                let k = (s.atan2(c) / QUARTER).round();
                combed[u] = Some(turn(d, normal[u], k as i32));
                tree.insert((t.min(u), t.max(u)));
                queue.push_back(u);
            }
        }
    }

    // ── ⭐⭐⭐ AS DUAS GRANDEZAS, medidas depois de pentear (ver `ph2d_crossfield::comb`).
    //
    // ⛔⛔ **Até 2026-08-23 isto era UM número com o nome da outra grandeza.** Media-se
    // dentro da travessia, contra o braço **cru** do vizinho, e o resultado — limitado
    // a 45° por construção — leu-se como «há singularidade dentro do patch, a dívida é
    // do F3». *Provado por mutação: com aquela lei, uma singularidade de índice `+¼`
    // fabricada de propósito imprime `11,25°` e `0` defeitos.*
    let (mut rough, mut defects) = (0.0f32, 0usize);
    let mut seen: std::collections::BTreeSet<(usize, usize)> = std::collections::BTreeSet::new();
    for (t, ns) in adj.iter().enumerate() {
        let Some(x) = combed[t] else {
            continue;
        };
        for &u in ns {
            let key = (t.min(u), t.max(u));
            if !seen.insert(key) {
                continue;
            }
            let (Some(r), Some(d)) = (tangent(x, normal[u]), combed[u]) else {
                continue;
            };
            let (c, s) = (dot(d, r), dot(cross(normal[u], d), r));
            let k = (s.atan2(c) / QUARTER).round() as i32;
            let turned = turn(d, normal[u], k);
            rough = rough.max(dot(turned, r).clamp(-1.0, 1.0).acos().to_degrees());
            // ⭐ **A holonomia só se lê onde o ciclo fecha** — numa aresta de árvore o
            // braço do filho foi *definido* como o mais próximo do pai.
            if !tree.contains(&key) {
                let q = k.rem_euclid(4);
                if q.min(4 - q) != 0 {
                    defects += 1;
                }
            }
        }
    }

    // ── O passo alvo de cada aresta, lido na moldura da cruz e MEDIADO entre as
    // duas faces que a partilham.
    let mut acc: std::collections::BTreeMap<(u32, u32), ([f32; 2], f32)> =
        std::collections::BTreeMap::new();
    for (i, t) in tris.iter().enumerate() {
        let Some(x) = combed[i] else {
            continue;
        };
        let y = cross(normal[i], x);
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            let e = sub(pos[b as usize], pos[a as usize]);
            let d = [dot(x, e), dot(y, e)];
            let slot = acc.entry((a, b)).or_insert(([0.0; 2], 0.0));
            slot.0[0] += d[0];
            slot.0[1] += d[1];
            slot.1 += 1.0;
            let back = acc.entry((b, a)).or_insert(([0.0; 2], 0.0));
            back.0[0] -= d[0];
            back.0[1] -= d[1];
            back.1 += 1.0;
        }
    }
    let target = |a: u32, b: u32| -> [f32; 2] {
        acc.get(&(a, b)).map_or([0.0; 2], |(v, n)| {
            let inv = 1.0 / n.max(1.0);
            [v[0] * inv, v[1] * inv]
        })
    };

    // ── ⭐⭐ A SIMILARIDADE, lida da FRONTEIRA. Ver o doc do módulo.
    let (mut num, mut den) = ([0.0f64; 2], 0.0f64);
    for (v, list) in nb.iter().enumerate() {
        if !fixed[v] {
            continue;
        }
        for &(w, _) in list {
            if !fixed[w as usize] {
                continue;
            }
            let d = target(u32::try_from(v).ok()?, w);
            let delta = [uv[w as usize][0] - uv[v][0], uv[w as usize][1] - uv[v][1]];
            // `conj(d) · Δ`, em complexos.
            num[0] += f64::from(d[0].mul_add(delta[0], d[1] * delta[1]));
            num[1] += f64::from(d[0].mul_add(delta[1], -(d[1] * delta[0])));
            den += f64::from(d[0].mul_add(d[0], d[1] * d[1]));
        }
    }
    if den <= 1.0e-20 {
        return None;
    }
    let c = [(num[0] / den) as f32, (num[1] / den) as f32];
    if !c[0].is_finite() || !c[1].is_finite() || c[0].hypot(c[1]) <= 1.0e-12 {
        return None;
    }

    // ── O passo de cada vizinhança, já multiplicado pela similaridade.
    let mut step: Vec<Vec<[f32; 2]>> = Vec::with_capacity(nb.len());
    for (v, list) in nb.iter().enumerate() {
        let mut row = Vec::with_capacity(list.len());
        for &(w, _) in list {
            let d = target(u32::try_from(v).ok()?, w);
            // `c · d`, em complexos.
            row.push([
                c[0].mul_add(d[0], -(c[1] * d[1])),
                c[0].mul_add(d[1], c[1] * d[0]),
            ]);
        }
        step.push(row);
    }
    Some(Aligned {
        step,
        rough_deg: rough,
        defects,
    })
}

/// `d` rodado de `k` quartos de volta em torno de `n`.
fn turn(d: [f32; 3], n: [f32; 3], k: i32) -> [f32; 3] {
    let p = cross(n, d);
    match k.rem_euclid(4) {
        1 => p,
        2 => [-d[0], -d[1], -d[2]],
        3 => [-p[0], -p[1], -p[2]],
        _ => d,
    }
}

/// **QUANTOS TRIÂNGULOS VIRARAM no domínio** — a rede desta cura.
///
/// ⛔ **Ela existe porque a direita nova pode empurrar um vértice para fora do
/// fecho dos vizinhos**, e aí o embutimento deixa de ser o de Tutte: a garantia
/// «sem dobras» é do problema harmónico puro, não deste. ⚠️ *Não se assume que a
/// garantia sobrevive — conta-se.*
///
/// A comparação é com o sinal da MAIORIA, não com um sinal escolhido: o polígono
/// pode ser percorrido nos dois sentidos, e fixar um deles daria «todos virados»
/// numa metade dos casos.
pub(crate) fn flipped(tris: &[[u32; 3]], uv: &[[f32; 2]]) -> usize {
    let mut pos = 0usize;
    let mut neg = 0usize;
    for t in tris {
        let (a, b, c) = (uv[t[0] as usize], uv[t[1] as usize], uv[t[2] as usize]);
        let area = (b[0] - a[0]).mul_add(c[1] - a[1], -((b[1] - a[1]) * (c[0] - a[0])));
        if area > 0.0 {
            pos += 1;
        } else if area < 0.0 {
            neg += 1;
        }
    }
    pos.min(neg)
}
