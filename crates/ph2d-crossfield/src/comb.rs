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
//! CANTOS.** [`holonomy`] mede se essa promessa se cumpre — e a resposta é
//! [`Holonomy::defects`], um **inteiro**: quantas voltas fechadas devolvem o braço
//! rodado. `0` = a região é penteável; `> 0` = há singularidade **dentro** dela, e
//! nenhuma lei sobre o seu interior pode seguir um campo que lá não existe.
//!
//! # ⛔⛔⛔ A régua que este módulo teve primeiro, e por que ela não podia responder
//!
//! ⚠️ **Isto é o segundo instrumento com este nome, e o primeiro media outra coisa.**
//! Ele devolvia um ângulo — o resto depois de virar cada braço para o quarto de volta
//! mais próximo — e leu-se `29°` na orelha e `44°` no gancho como *«há singularidade
//! dentro dos nossos patches, a dívida é do F3»*.
//!
//! ⛔ **Esse ângulo não pode passar de 45°, por construção.** Uma singularidade dá
//! `90°`, que aquela linha nunca teve como escrever; e `29°`–`44°` é o *tecto* da
//! grandeza, não um defeito grande. ⛔⛔ Pior: no único ramo onde a holonomia se lê —
//! a aresta que **fecha ciclo** — ela comparava o braço cru do vizinho em vez do que
//! já lhe fora atribuído. *O teste de fecho não estava saturado; não existia.*
//!
//! ⇒ Aquela grandeza sobrevive aqui com o nome certo ([`Holonomy::rough_max`], a
//! **rugosidade** do campo) e sem barra, porque o oráculo mede o mesmo. A holonomia
//! a sério é inteira e dispensa barra. Controlo positivo e negativo em
//! `comb_tests.rs` — *uma régua nova sem os dois controlos é a mesma aposta outra vez.*

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

/// **O QUE SOBRA depois de pentear uma região — e são DUAS coisas, não uma.**
///
/// ⛔⛔ **A primeira versão deste tipo trazia só a primeira, com o nome da segunda**
/// — ver [`Self::_A_ROUGHNESS_RULER_CANNOT_SEE_A_SINGULARITY`]. As duas colunas
/// vivem aqui separadas de propósito:
///
/// | | o que é | alcance |
/// |---|---|---|
/// | `rough_*` | a **rugosidade** do campo: o resto depois de virar cada braço para o quarto de volta mais próximo | ⛔ **`[0°, 45°]` por construção** |
/// | ⭐ `cycles`/`defects`/`turn_max` | a **holonomia**: o campo volta rodado ao dar a volta a um ciclo? | `{0, 1, 2}` quartos de volta |
///
/// *Só a segunda linha responde «há singularidade dentro da região».*
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Holonomy {
    /// Quantas arestas duais interiores entraram na conta — ⚠️ **cada uma UMA vez**
    /// (a versão que media dentro da travessia contava as duas direcções).
    pub edges: usize,
    /// ⚠️ **Quantas faces ficaram de FORA** (degeneradas, ou com a direcção do campo
    /// sem componente no plano delas). ⛔ *Uma região medida a meio não é uma região
    /// limpa* — quem lê `defects` tem de olhar esta coluna ao lado.
    pub skipped: usize,
    /// A **rugosidade** mediana, em graus. ⛔ Não é holonomia — ver o doc do tipo.
    pub rough_p50: f32,
    /// O percentil 95 da rugosidade.
    pub rough_p95: f32,
    /// A pior rugosidade. ⛔ **Nunca pode passar de 45°.**
    pub rough_max: f32,
    /// ⭐⭐⭐ **Quantas arestas duais FECHAM CICLO** — as que não entraram na árvore
    /// da travessia. É sobre elas, e só sobre elas, que a holonomia se lê: numa
    /// aresta de árvore o braço foi *definido* como o mais próximo do pai, e o
    /// desacordo é zero por construção.
    pub cycles: usize,
    /// ⭐⭐⭐ **De quantas dessas o campo volta RODADO** de um quarto de volta ou
    /// mais. `0` = a região é penteável; `> 0` = há singularidade **dentro**, e
    /// nenhuma lei sobre o interior dela pode seguir um campo que lá não existe.
    pub defects: usize,
    /// O maior desacordo em **quartos de volta** (`0`, `1` ou `2`), medido como
    /// distância à identidade — `3` quartos num sentido é `1` no outro.
    pub turn_max: i32,
}

impl Holonomy {
    /// ⛔⛔⛔ **UMA RÉGUA DE RUGOSIDADE NÃO CONSEGUE VER UMA SINGULARIDADE** — e foi
    /// preciso pedir-lhe isso duas vezes para reparar (2026-08-23).
    ///
    /// # O que aconteceu, na ordem em que aconteceu
    ///
    /// A primeira versão deste tipo trazia um `CLEAN_DEG = 1.0`, escrito do
    /// raciocínio *«um campo penteável dá resíduo de arredondamento; a alternativa
    /// topológica mais próxima é 90°, logo qualquer barra entre 1° e 10° separa as
    /// duas classes»*. A medição pareceu desmenti-lo:
    ///
    /// | `max` por patch | p50 mediano | p95 mediano |
    /// |---|---|---|
    /// | ⛔ nós, orelha `29,3°` | `0,479°` | `2,52°` |
    /// | ⭐ **oráculo, orelha `18,6°`** | **`0,470°`** | **`3,19°`** |
    /// | ⛔ nós, gancho `44,1°` | `0,892°` | `3,70°` |
    /// | ⭐ **oráculo, gancho `38,4°`** | **`0,726°`** | **`3,63°`** |
    ///
    /// ⇒ a leitura foi *«a referência tem o mesmo, logo a barra não separa nada»*, e
    /// a barra caiu. **A conclusão sobre a BARRA está certa e fica de pé.**
    ///
    /// # ⭐⭐⭐ O que estava errado é mais fundo: a GRANDEZA
    ///
    /// ⛔ **O raciocínio nunca foi refutado — ele foi testado sobre um número que não
    /// consegue exprimir aquilo que ele procurava.** O que a coluna `max` mede é o
    /// resto depois de virar cada braço para o quarto de volta **mais próximo**, e
    /// esse resto é **`≤ 45°` por construção**. *A «alternativa topológica a 90°» que
    /// o raciocínio invocava não é um valor grande desta coluna — ela não é um valor
    /// desta coluna de todo.*
    ///
    /// ⚠️ E a assinatura estava à vista: **`29°` e `44°` são o TECTO da grandeza**.
    /// Um número encostado ao máximo que ele pode imprimir é um instrumento saturado,
    /// não um defeito medido — irmão de
    /// [[feedback_an_unlabelled_probe_column_gets_read_backwards]].
    ///
    /// ⛔⛔ **E o pior não era o tecto, era o ramo que faltava:** na aresta que fecha
    /// ciclo — a única onde a holonomia se pode ler — a versão antiga comparava o
    /// braço **cru** do vizinho, nunca o que já lhe tinha sido atribuído. *O teste de
    /// fecho não estava lá para ser saturado; ele não existia.*
    ///
    /// ⇒ Esta grandeza mudou de nome para [`Self::rough_max`], e a holonomia a sério
    /// vive em [`Self::defects`], com controlo positivo e negativo em `comb_tests.rs`.
    ///
    /// # ⛔ O que continua sem barra
    ///
    /// A rugosidade **continua sem barra**, e agora pela razão certa: a tabela acima
    /// mostra a referência com os mesmos números, logo ela mede a discretização e não
    /// o estado da região. *Quem quiser voltar a pôr uma barra aqui tem de a derivar
    /// do oráculo com este mesmo código.* A holonomia, essa, **não precisa de barra
    /// nenhuma** — ela é inteira.
    const _A_ROUGHNESS_RULER_CANNOT_SEE_A_SINGULARITY: () = ();
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
    // ⭐⭐⭐ **AS ARESTAS DA ÁRVORE, guardadas — é o que separa as duas grandezas.**
    //
    // ⛔ Numa aresta de árvore o braço do filho foi *definido* como o quarto de volta
    // mais próximo do pai: o desacordo é zero por construção, e medir holonomia lá é
    // medir a própria definição. A holonomia só tem sentido nas arestas que a árvore
    // **não** usou — as que fecham ciclo.
    let mut tree: std::collections::BTreeSet<(usize, usize)> = std::collections::BTreeSet::new();
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
                if combed[u].is_some() {
                    continue;
                }
                let Some(r) = tangent(x, normal[u]) else {
                    continue;
                };
                let d = raw[u];
                let (c, s) = (dot(d, r), dot(cross(normal[u], d), r));
                let k = (s.atan2(c) / std::f32::consts::FRAC_PI_2).round();
                #[allow(clippy::cast_possible_truncation)]
                let turned = turn(d, normal[u], k as i32);
                combed[u] = Some(turned);
                tree.insert((t.min(u), t.max(u)));
                queue.push_back(u);
            }
        }
    }
    let out: Vec<[f32; 3]> = combed.into_iter().collect::<Option<_>>()?;

    // ── ⭐⭐⭐ A MEDIÇÃO, sobre cada aresta dual UMA vez, com o campo já penteado.
    //
    // ⚠️ **Separada da travessia de propósito.** Medir lá dentro obriga a comparar o
    // braço CRU do vizinho (o penteado ainda não existe quando a aresta é de árvore,
    // e quando ela fecha ciclo o laço já não passa por lá) — foi assim que a versão
    // antiga ficou sem o único ramo que interessa.
    let mut left: Vec<f32> = Vec::with_capacity(adj.iter().map(Vec::len).sum::<usize>() / 2);
    let (mut cycles, mut defects, mut turn_max) = (0usize, 0usize, 0i32);
    let mut seen: std::collections::BTreeSet<(usize, usize)> = std::collections::BTreeSet::new();
    for (t, ns) in adj.iter().enumerate() {
        for &u in ns {
            let key = (t.min(u), t.max(u));
            if !seen.insert(key) {
                continue;
            }
            let Some(r) = tangent(out[t], normal[u]) else {
                continue;
            };
            let d = out[u];
            let (c, s) = (dot(d, r), dot(cross(normal[u], d), r));
            let ang = s.atan2(c);
            let k = (ang / std::f32::consts::FRAC_PI_2).round();
            #[allow(clippy::cast_possible_truncation)]
            let k = k as i32;
            let turned = turn(d, normal[u], k);
            left.push(dot(turned, r).clamp(-1.0, 1.0).acos().to_degrees());
            if !tree.contains(&key) {
                cycles += 1;
                // ⭐ A distância à identidade em quartos de volta: `3` num sentido é
                // `1` no outro, e o que interessa é *quanto* o campo voltou rodado.
                let q = k.rem_euclid(4);
                let q = q.min(4 - q);
                if q != 0 {
                    defects += 1;
                    turn_max = turn_max.max(q);
                }
            }
        }
    }

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
            rough_p50: pct(0.50),
            rough_p95: pct(0.95),
            rough_max: left.last().copied().unwrap_or(0.0),
            cycles,
            defects,
            turn_max,
        },
    ))
}

/// **Só o resíduo** — ver [`comb`], de que esta é a metade que a maioria dos
/// chamadores quer.
#[must_use]
pub fn holonomy(mesh: &Mesh, faces: &[u32], dirs: &[[f32; 3]]) -> Option<Holonomy> {
    comb(mesh, faces, dirs).map(|(_, h)| h)
}

#[cfg(test)]
#[path = "comb_tests.rs"]
mod tests;
