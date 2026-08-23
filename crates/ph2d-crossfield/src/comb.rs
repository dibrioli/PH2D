//! ⭐⭐⭐ **PENTEAR UM CAMPO DE CRUZES sobre uma região, e MEDIR o que sobra.**
//!
//! # Porque isto existe
//!
//! Uma cruz tem quatro braços a 90°, então «a direcção do campo nesta face» é uma
//! escolha entre quatro. Para **somar deslocamentos** ao longo de uma região — que é
//! o que qualquer parametrização alinhada faz — é preciso escolher, face a face, o
//! braço que continua o do vizinho. A isso chama-se **pentear**.
//!
//! ⭐ **Pentear só é possível sem resíduo se a região não contiver singularidade no
//! interior.** Dar a volta a uma singularidade de índice `±¼` devolve o braço rodado
//! de 90°, e não há escolha face-a-face que o evite — é topologia, não numérica.
//!
//! ⛔ **É por isso que a decomposição em patches promete pôr as singularidades nos
//! CANTOS.** [`holonomy`] mede se essa promessa se cumpre, e devolve o resíduo em
//! graus: perto de `0°` a região é penteável; grande, há singularidade **dentro** e
//! nenhuma lei sobre o interior daquele patch pode funcionar — o campo que ela
//! seguiria não existe lá de forma consistente.
//!
//! ⚠️ **A medição de 2026-08-23 que motivou este módulo era um MÁXIMO** sobre todas
//! as arestas de todos os patches (`29°` na orelha, `44°` no gancho), e um máximo não
//! diz **quantos** patches estão sujos. É por isso que [`Holonomy`] traz a
//! distribuição, e não um número.

use ph2d_mesh::Mesh;

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

fn unit(a: [f32; 3]) -> Option<[f32; 3]> {
    let l = dot(a, a).sqrt();
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

/// **O QUE SOBRA depois de pentear uma região** — a distribuição, não um número.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Holonomy {
    /// Quantas arestas interiores entraram na conta.
    pub edges: usize,
    /// ⚠️ **Quantas faces ficaram de FORA** (degeneradas, ou com a direcção do campo
    /// sem componente no plano delas). ⛔ *Uma região medida a meio não é uma região
    /// limpa* — quem lê [`Self::is_clean`] tem de olhar esta coluna ao lado.
    pub skipped: usize,
    /// O desacordo mediano, em graus.
    pub p50: f32,
    /// O percentil 95.
    pub p95: f32,
    /// O pior.
    pub max: f32,
}

impl Holonomy {
    /// ⛔⛔ **NÃO EXISTE BARRA AQUI, e a ausência é o resultado** (2026-08-23).
    ///
    /// A primeira versão deste tipo trazia um `CLEAN_DEG = 1.0` sobre [`Self::max`],
    /// escrito do raciocínio *«um campo penteável dá resíduo de arredondamento; a
    /// alternativa topológica mais próxima é 90°, logo qualquer barra entre 1° e 10°
    /// separa as duas classes»*. ⚠️ **O raciocínio é limpo e a medição desmente-o.**
    ///
    /// | `max` por patch | p50 mediano | p95 mediano |
    /// |---|---|---|
    /// | ⛔ nós, orelha `29,3°` | `0,479°` | `2,52°` |
    /// | ⭐ **oráculo, orelha `18,6°`** | **`0,470°`** | **`3,19°`** |
    /// | ⛔ nós, gancho `44,1°` | `0,892°` | `3,70°` |
    /// | ⭐ **oráculo, gancho `38,4°`** | **`0,726°`** | **`3,63°`** |
    ///
    /// ⇒ **A referência tem exactamente a mesma coisa.** Uma barra de `1°` sobre o
    /// máximo classifica **12 de 12** patches do oráculo como sujos — *um predicado
    /// que reprova a testemunha de controlo não mede o defeito, mede a discretização*.
    ///
    /// ⭐⭐ **E é por isso que este tipo devolve a DISTRIBUIÇÃO.** O `max` é um
    /// extremo sobre milhares de arestas e apanha o arredondamento do braço da cruz
    /// perto dos 45°; o `p50` e o `p95` é que dizem o estado da região. *É a mesma
    /// lição do extremo global contra a régua por-face
    /// ([[feedback_a_global_extreme_is_not_a_per_face_ruler]]), um nível acima.*
    ///
    /// ⛔ **Quem quiser voltar a pôr uma barra aqui tem de a derivar do oráculo com
    /// este mesmo código** — e a tabela acima diz que ela não separa nada.
    const _MEASURED_AND_REJECTED_CLEAN_BAR: () = ();
}

/// ⭐ **PENTEIA `faces` e devolve o resíduo.** `dirs` é uma direcção da cruz por face
/// **da malha** (o índice é o da `mesh`, não o da fatia `faces`).
///
/// Devolve também as direcções penteadas, na ordem de `faces`.
///
/// ⚠️ **A travessia é por largura a partir de `faces[0]`, com a vizinhança derivada
/// de um `BTreeMap`** — determinística de propósito. *Uma travessia dependente de
/// `HashMap` daria campos diferentes em corridas diferentes, e a malha do produto
/// deixaria de ser reproduzível ([`CLAUDE.md` §5.1, a espinha do determinismo]).*
///
/// `None` quando uma face é degenerada ou a direcção dela não tem componente no plano
/// — ⚠️ **é uma resposta e não uma falha**: o chamador fica sem alinhamento, que é o
/// caminho antigo.
///
/// ⛔ **Uma região DESLIGADA conta como penteável** e não é mentira: cada componente
/// é penteada a partir da sua própria semente, e não há aresta entre elas para
/// carregar resíduo. *O que a região partida perde é a comparabilidade das duas
/// metades, e isso é problema de quem a partiu.*
#[must_use]
pub fn comb(mesh: &Mesh, faces: &[u32], dirs: &[[f32; 3]]) -> Option<(Vec<[f32; 3]>, Holonomy)> {
    if faces.is_empty() {
        return None;
    }
    let pos = mesh.positions();
    let mut normal: Vec<[f32; 3]> = Vec::with_capacity(faces.len());
    let mut raw: Vec<[f32; 3]> = Vec::with_capacity(faces.len());
    // ⛔⛔ **UMA FACE IMPOSSÍVEL NÃO PODE MATAR A REGIÃO INTEIRA.**
    //
    // ⚠️ **Medido em 2026-08-23:** a primeira versão devolvia `None` ao primeiro
    // triângulo degenerado, e sobre a decomposição do oráculo isso deu `None` nos
    // **12 patches de 12** — a sonda imprimiu «0 sujos» sobre **zero patches
    // medidos**, e «0 sujos» lê-se como *limpo*. *Skip gracioso não é verde*
    // (`CLAUDE.md` §5.0), e um `None` de tudo-ou-nada é a mesma doença com outro
    // nome. A face impossível fica **de fora e CONTADA**; a região continua a ser
    // medida pelo resto.
    let mut skipped = 0usize;
    let mut keep: Vec<u32> = Vec::with_capacity(faces.len());
    for &f in faces {
        let Some(face) = mesh.faces().get(f as usize) else {
            skipped += 1;
            continue;
        };
        let v = face.verts();
        if v.len() < 3 {
            skipped += 1;
            continue;
        }
        let (a, b, c) = (pos[v[0] as usize], pos[v[1] as usize], pos[v[2] as usize]);
        let (Some(n), Some(d)) = (
            unit(cross(sub(b, a), sub(c, a))),
            dirs.get(f as usize).copied(),
        ) else {
            skipped += 1;
            continue;
        };
        let Some(t) = tangent(d, n) else {
            skipped += 1;
            continue;
        };
        raw.push(t);
        normal.push(n);
        keep.push(f);
    }
    if keep.is_empty() {
        return None;
    }
    let faces: &[u32] = &keep;

    // ── Vizinhança pela aresta partilhada, dentro da região.
    let mut share: std::collections::BTreeMap<(u32, u32), Vec<usize>> =
        std::collections::BTreeMap::new();
    for (i, &f) in faces.iter().enumerate() {
        let v = mesh.faces()[f as usize].verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            share.entry((a.min(b), a.max(b))).or_default().push(i);
        }
    }
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); faces.len()];
    for owners in share.values() {
        if owners.len() == 2 {
            adj[owners[0]].push(owners[1]);
            adj[owners[1]].push(owners[0]);
        }
    }

    let mut combed: Vec<Option<[f32; 3]>> = vec![None; faces.len()];
    let mut left: Vec<f32> = Vec::new();
    // ⚠️ **Semente por COMPONENTE**, não uma só: uma região desligada deixaria
    // metade das faces sem direcção penteada, e o `unwrap` a jusante leria isso
    // como «o campo é mau» em vez de «a região está partida».
    for seed in 0..faces.len() {
        if combed[seed].is_some() {
            continue;
        }
        combed[seed] = Some(raw[seed]);
        let mut queue = std::collections::VecDeque::from([seed]);
        while let Some(t) = queue.pop_front() {
            let x = combed[t]?;
            for &u in &adj[t] {
                let Some(r) = tangent(x, normal[u]) else {
                    continue;
                };
                let d = raw[u];
                let (c, s) = (dot(d, r), dot(cross(normal[u], d), r));
                let k = (s.atan2(c) / std::f32::consts::FRAC_PI_2).round();
                #[allow(clippy::cast_possible_truncation)]
                let turned = turn(d, normal[u], k as i32);
                // ⭐ **O que sobra depois de virar é a HOLONOMIA.** Num campo
                // penteável `turned` e `r` coincidem; o resto é a singularidade que
                // ficou dentro da região.
                left.push(dot(turned, r).clamp(-1.0, 1.0).acos().to_degrees());
                if combed[u].is_none() {
                    combed[u] = Some(turned);
                    queue.push_back(u);
                }
            }
        }
    }
    let out: Vec<[f32; 3]> = combed.into_iter().collect::<Option<_>>()?;
    left.sort_by(f32::total_cmp);
    let pct = |p: f32| -> f32 {
        if left.is_empty() {
            return 0.0;
        }
        #[allow(clippy::cast_precision_loss, clippy::cast_sign_loss)]
        let i = ((left.len() - 1) as f32 * p).round() as usize;
        left[i.min(left.len() - 1)]
    };
    Some((
        out,
        Holonomy {
            edges: left.len(),
            skipped,
            p50: pct(0.50),
            p95: pct(0.95),
            max: left.last().copied().unwrap_or(0.0),
        },
    ))
}

/// **Só o resíduo** — ver [`comb`], de que esta é a metade que a maioria dos
/// chamadores quer.
#[must_use]
pub fn holonomy(mesh: &Mesh, faces: &[u32], dirs: &[[f32; 3]]) -> Option<Holonomy> {
    comb(mesh, faces, dirs).map(|(_, h)| h)
}
