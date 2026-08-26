//! **OS GATES DO REMESH ISOTRÓPICO** — a propriedade que o passe existe para dar.

use ph2d_mesh::shapes;

use super::{ALPHA, Report, remesh_isotropic, target_edge};

fn mean_edge(mesh: &ph2d_mesh::Mesh) -> f32 {
    let p = mesh.positions();
    let (mut sum, mut n) = (0.0f64, 0usize);
    for f in mesh.faces() {
        let v = f.verts();
        for i in 0..v.len() {
            let (a, b) = (p[v[i] as usize], p[v[(i + 1) % v.len()] as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            sum += f64::from(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
            n += 1;
        }
    }
    (sum / n.max(1) as f64) as f32
}

fn volume(mesh: &ph2d_mesh::Mesh) -> f64 {
    let p = mesh.positions();
    let mut vol = 0.0f64;
    for f in mesh.faces() {
        let v = f.verts();
        for k in 1..v.len() - 1 {
            let (a, b, c) = (p[v[0] as usize], p[v[k] as usize], p[v[k + 1] as usize]);
            vol += f64::from(a[0].mul_add(
                b[1].mul_add(c[2], -(b[2] * c[1])),
                a[1].mul_add(
                    b[2].mul_add(c[0], -(b[0] * c[2])),
                    a[2] * b[0].mul_add(c[1], -(b[1] * c[0])),
                ),
            )) / 6.0;
        }
    }
    vol
}

/// ⭐ **A PROPRIEDADE INTEIRA: a saída não herda a densidade da entrada.**
///
/// ⚠️ **É o gate que a medição do oráculo pediu.** Oito vértices e treze mil têm
/// de sair no mesmo lugar — é isso que faz o pipeline a jusante deixar de
/// depender de como o artista deixou a malha. Sem este passe, o cubo devolvia
/// **malha vazia** no remesher.
///
/// ⚠️ **A RÉGUA é a razão para o alvo DA PRÓPRIA MALHA, e a primeira versão
/// deste gate errava nisso** — ela comparava arestas médias ABSOLUTAS entre
/// fixturas de caixas diferentes (o cubo tem diagonal `1,732` e a esfera
/// `3,464`), e acusava `2,01×` de dispersão sobre um passe que estava correto:
/// os dois estavam a **9 %** do respectivo alvo. *Uma régua que compara dois
/// números de unidades diferentes acusa o algoritmo pelo erro de quem mede.*
#[test]
fn the_output_density_does_not_depend_on_the_input_density() {
    let mut ratios: Vec<(String, f32, usize)> = Vec::new();
    for (name, mut mesh) in [
        ("cubo (8 v)", shapes::cube(1.0)),
        ("esfera 24x36 (830 v)", shapes::uv_sphere(24, 36, 1.0)),
        ("esfera 96x144 (13 682 v)", shapes::uv_sphere(96, 144, 1.0)),
    ] {
        let before = mesh.vert_count();
        let want = target_edge(&mesh, ALPHA);
        let r: Report = remesh_isotropic(&mut mesh, ALPHA);
        let got = mean_edge(&mesh);
        eprintln!(
            "[iso] {name}: {before} -> {} v em {} rodadas, aresta {got:.4} / alvo {want:.4} = {:.3}x",
            r.verts_after,
            r.rounds,
            got / want
        );
        assert!(
            r.verts_after > 0 && !mesh.faces().is_empty(),
            "{name}: o passe devolveu malha vazia"
        );
        ratios.push((name.to_string(), got / want, r.verts_after));
    }

    // (1) cada malha chega ao SEU alvo.
    for (name, ratio, _) in &ratios {
        assert!(
            (0.75..=1.35).contains(ratio),
            "{name}: a aresta media saiu {ratio:.3}x o alvo -- o passe nao converge para o que \
             lhe foi pedido"
        );
    }
    // (2) e as três chegam ao MESMO múltiplo do alvo — é isso que quer dizer
    //     "a densidade da saída não depende da entrada".
    let (lo, hi) = ratios
        .iter()
        .fold((f32::MAX, 0.0f32), |(l, h), e| (l.min(e.1), h.max(e.1)));
    assert!(
        hi / lo < 1.15,
        "as razoes ao alvo saem entre {lo:.3}x e {hi:.3}x ({:.2}x de dispersao) -- a saida ainda \
         HERDA a densidade da entrada: {ratios:?}",
        hi / lo
    );
    // (3) ⭐ E o caso DIRETO: duas esferas da MESMA caixa, com 16x de diferenca
    //     na entrada, têm de sair com a mesma contagem.
    let (small, big) = (ratios[1].2 as f32, ratios[2].2 as f32);
    let spread = small.max(big) / small.min(big);
    assert!(
        spread < 1.15,
        "830 vertices deram {small} e 13 682 deram {big} ({spread:.2}x) -- a entrada ainda manda"
    );
}

/// ⭐ **O ALVO É DA CAIXA, e a escala prova-o.**
///
/// ⚠️ Duas esferas de raios diferentes têm de sair com arestas na mesma razão dos
/// raios. Um alvo derivado da entrada não teria esta propriedade.
#[test]
fn the_target_scales_with_the_bounding_box() {
    let small = target_edge(&shapes::uv_sphere(24, 36, 1.0), ALPHA);
    let big = target_edge(&shapes::uv_sphere(24, 36, 4.0), ALPHA);
    assert!(
        (big / small - 4.0).abs() < 1.0e-3,
        "o alvo nao escalou com a caixa: {small:.5} e {big:.5} ({:.3}x, esperado 4)",
        big / small
    );
}

/// ⭐ **A FORMA SOBREVIVE** — a reprojeção é o que separa remalhar de alisar.
///
/// ⚠️ **Sem a projeção de volta à superfície ORIGINAL o Laplaciano encolhe**, e o
/// mecanismo já está medido e registrado (recusa 13 do ADR-0160). A barra é o
/// volume, que é a grandeza que um encolhimento move primeiro.
#[test]
fn the_shape_survives_the_remesh() {
    for (name, mut mesh) in [
        ("esfera", shapes::uv_sphere(48, 64, 1.0)),
        ("toro", shapes::torus(64, 32, 1.0, 0.35)),
    ] {
        let before = volume(&mesh);
        remesh_isotropic(&mut mesh, ALPHA);
        let after = volume(&mesh);
        eprintln!("[iso] {name}: volume {before:.4} -> {after:.4}");
        assert!(
            (after - before).abs() < 0.05 * before.abs(),
            "{name}: o volume andou {:.1}% ({before:.4} -> {after:.4}) -- a reprojecao nao esta' \
             a segurar o Laplaciano",
            100.0 * (after - before).abs() / before.abs()
        );
    }
}

/// ⭐ **DETERMINÍSTICO** (HR-5) — duas corridas, a mesma malha ao bit.
#[test]
fn the_remesh_is_bit_reproducible() {
    let (mut a, mut b) = (
        shapes::uv_sphere(24, 36, 1.0),
        shapes::uv_sphere(24, 36, 1.0),
    );
    remesh_isotropic(&mut a, ALPHA);
    remesh_isotropic(&mut b, ALPHA);
    assert_eq!(a.positions(), b.positions(), "duas corridas divergiram");
}

/// ⭐⭐ **A SAÍDA É UMA VARIEDADE FECHADA** — a promessa que TODO o resto do
/// pipeline consome sem a verificar.
///
/// ⚠️ **Este gate nasceu de um defeito que passou por três fases sem ser visto**
/// (2026-08-21). O `cube` saía deste passe com **18 anéis abertos**; o traçado a
/// jusante lia 33 singularidades falsas e uma soma de índices de `−1` onde a
/// topologia exige `+8`, e *nada no campo estava errado*. A causa era uma
/// colisão de diagonal no flip da `ph2d-mesh` (ver a recusa 4 no cabeçalho de
/// `dyntopo_flip`), e ela também atingia a esfera EMBARALHADA e a RUIDOSA.
///
/// ⚠️ **Por que o `the_genus_survives` não o apanhava:** ele conta as arestas num
/// `BTreeSet`, que **funde** as duas arestas do par duplicado — a característica
/// de Euler saía certa sobre uma malha que já não era variedade. *Uma régua que
/// deduplica não pode denunciar duplicação.* Aqui a contagem é por OCORRÊNCIA.
///
/// | fixture | anéis abertos ANTES da cura | depois |
/// |---|---|---|
/// | `cube` | **18** | 0 |
/// | `sphere_shuffled` | 2 | 0 |
/// | `sphere_uv` · `torus` | 0 | 0 |
#[test]
fn the_remesh_returns_a_closed_manifold() {
    use std::collections::BTreeMap;
    for (name, mut mesh) in [
        ("cubo", shapes::cube(1.0)),
        ("esfera", shapes::uv_sphere(24, 36, 1.0)),
        (
            "esfera embaralhada",
            shapes::uv_sphere_shuffled(24, 36, 1.0),
        ),
        ("esfera ruidosa", shapes::uv_sphere_noisy(24, 36, 1.0, 0.02)),
        ("toro", shapes::torus(32, 16, 1.0, 0.35)),
    ] {
        remesh_isotropic(&mut mesh, ALPHA);
        let mut undirected: BTreeMap<(u32, u32), usize> = BTreeMap::new();
        let mut directed: BTreeMap<(u32, u32), usize> = BTreeMap::new();
        for f in mesh.faces() {
            let v = f.verts();
            for i in 0..v.len() {
                let (a, b) = (v[i], v[(i + 1) % v.len()]);
                *undirected.entry((a.min(b), a.max(b))).or_default() += 1;
                *directed.entry((a, b)).or_default() += 1;
            }
        }
        let border = undirected.values().filter(|&&n| n == 1).count();
        let nonmanifold = undirected.values().filter(|&&n| n > 2).count();
        // ⚠️ **A terceira pergunta, e ela é DIFERENTE das outras duas.** Uma
        // aresta dirigida usada duas vezes é orientação inconsistente ou face
        // duplicada — nenhuma das quais move a contagem por aresta não-dirigida.
        let repeated = directed.values().filter(|&&n| n > 1).count();
        assert_eq!(
            (border, nonmanifold, repeated),
            (0, 0, 0),
            "{name}: o passe devolveu uma malha que nao e' variedade fechada \
             (borda {border}, nao-variedade {nonmanifold}, dirigida-repetida {repeated})"
        );
    }
}

/// ⭐ **A TOPOLOGIA ATRAVESSA** — o gênero da entrada é o da saída.
///
/// ⚠️ **O `cubo` está aqui desde 2026-08-21**, e é a fixtura que faltava: ele era
/// a única do corpus que saía deste passe não-variedade, e este gate passava
/// mesmo assim (ver [`the_remesh_returns_a_closed_manifold`]).
#[test]
fn the_genus_survives() {
    use std::collections::BTreeSet;
    for (name, mut mesh, want) in [
        ("cubo", shapes::cube(1.0), 2i64),
        ("esfera", shapes::uv_sphere(24, 36, 1.0), 2),
        ("toro", shapes::torus(64, 32, 1.0, 0.35), 0),
    ] {
        remesh_isotropic(&mut mesh, ALPHA);
        let mut edges: BTreeSet<(u32, u32)> = BTreeSet::new();
        for f in mesh.faces() {
            let v = f.verts();
            for i in 0..v.len() {
                let (a, b) = (v[i], v[(i + 1) % v.len()]);
                edges.insert(if a < b { (a, b) } else { (b, a) });
            }
        }
        let chi = mesh.vert_count() as i64 - edges.len() as i64 + mesh.faces().len() as i64;
        eprintln!("[iso] {name}: chi={chi} (esperado {want})");
        assert_eq!(chi, want, "{name}: o remesh mudou o GENERO da superficie");
    }
}

/// Os laços de bordo e o perímetro de uma malha — a régua honesta do buraco.
///
/// ⛔⛔ **NÃO a contagem de arestas de bordo**: ela é função do passo, e um remalhe que
/// reamostra a mesma curva mais fino sobe-a sem tocar no buraco. Medido 2026-08-26 na
/// `sculpt_punctured`: `38 → 104` arestas com o perímetro **exacto**.
fn border(mesh: &ph2d_mesh::Mesh) -> (usize, f32) {
    let mut n: std::collections::BTreeMap<(u32, u32), usize> = std::collections::BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            *n.entry(if a < b { (a, b) } else { (b, a) }).or_default() += 1;
        }
    }
    let pos = mesh.positions();
    let edges: Vec<(u32, u32)> = n
        .into_iter()
        .filter(|(_, c)| *c == 1)
        .map(|(e, _)| e)
        .collect();
    let mut length = 0.0f32;
    let mut nxt: std::collections::BTreeMap<u32, Vec<u32>> = std::collections::BTreeMap::new();
    for &(a, b) in &edges {
        let (p, q) = (pos[a as usize], pos[b as usize]);
        let d = [p[0] - q[0], p[1] - q[1], p[2] - q[2]];
        length += d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
        nxt.entry(a).or_default().push(b);
        nxt.entry(b).or_default().push(a);
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut loops = 0usize;
    for &v in nxt.keys() {
        if !seen.insert(v) {
            continue;
        }
        loops += 1;
        let mut stack = vec![v];
        while let Some(u) = stack.pop() {
            for &w in nxt.get(&u).into_iter().flatten() {
                if seen.insert(w) {
                    stack.push(w);
                }
            }
        }
    }
    (loops, length)
}

/// **Quanto o rebordo VIRA** de aresta para aresta, em graus — a medida do serrilhado.
///
/// Um círculo de `n` lados vira `360/n` (10° com 36 lados); um rebordo aberto a pincel vira
/// muito mais. ⚠️ *É esta grandeza que decide se uma fixtura de bordo tem o que medir.*
fn mean_rim_turn(mesh: &ph2d_mesh::Mesh) -> f32 {
    let mut n: std::collections::BTreeMap<(u32, u32), usize> = std::collections::BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            *n.entry(if a < b { (a, b) } else { (b, a) }).or_default() += 1;
        }
    }
    let mut nbr: std::collections::BTreeMap<u32, Vec<u32>> = std::collections::BTreeMap::new();
    for ((a, b), c) in n {
        if c == 1 {
            nbr.entry(a).or_default().push(b);
            nbr.entry(b).or_default().push(a);
        }
    }
    let pos = mesh.positions();
    let (mut sum, mut count) = (0.0f32, 0usize);
    for (&v, ns) in &nbr {
        if ns.len() != 2 {
            continue;
        }
        let p = pos[v as usize];
        let dir = |w: u32| {
            let q = pos[w as usize];
            let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
            let l = d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt().max(1.0e-20);
            [d[0] / l, d[1] / l, d[2] / l]
        };
        let (a, b) = (dir(ns[0]), dir(ns[1]));
        let c = a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2])).clamp(-1.0, 1.0);
        // O ângulo INTERNO é `acos(c)`; a viragem é o que falta para a recta.
        sum += 180.0 - c.acos().to_degrees();
        count += 1;
    }
    sum / count.max(1) as f32
}

/// ⭐⭐⭐ **UMA ESFERA COM UM BURACO PEQUENO NO POLO** — e a fixtura anterior era um
/// **tubo**, que não continha o fenómeno.
///
/// ⛔⛔ **As três provas de mutação sobreviveram ao tubo** (2026-08-26). O rebordo dele é um
/// **círculo plano** numa superfície que passa exactamente por ele: alisar ao longo de um
/// polígono regular de 48 lados encolhe-o `0,2 %` por passo, e projectar na superfície
/// devolve-o ao mesmo sítio. *As três leis do rebordo eram indistinguíveis ali, e o gate
/// passava com qualquer uma delas.*
///
/// ⇒ Esta é a forma da queixa do artista: **um buraco pequeno numa superfície CURVA**, onde
/// os vizinhos de um vértice do rebordo são quase todos **interiores** (a Laplaciana puxa-o
/// para dentro) e a superfície de referência **continua para lá do rebordo** (a projecção
/// deixa-o deslizar). É a `sculpt_t002`, em miniatura.
fn sphere_with_a_small_hole(rings: usize, segments: usize) -> ph2d_mesh::Mesh {
    let mut full = ph2d_mesh::shapes::uv_sphere(rings, segments, 1.0);
    full.triangulate();
    // O vértice mais alto — o polo. Tirar o leque dele abre um buraco de `segments` lados.
    let pole = full
        .positions()
        .iter()
        .enumerate()
        .max_by(|a, b| a.1[1].total_cmp(&b.1[1]))
        .map(|(i, _)| u32::try_from(i).unwrap_or(0))
        .unwrap_or(0);
    let faces: Vec<ph2d_mesh::Face> = full
        .faces()
        .iter()
        .filter(|f| !f.verts().contains(&pole))
        .copied()
        .collect();
    // ⭐⭐ **O rebordo é IRREGULAR, e isso é metade do fenómeno.** Um buraco aberto por um
    // pincel não tem rebordo de compasso; e ⚠️ **num rebordo liso o alisamento não tem o que
    // encolher**, então uma fixtura de rebordo circular deixa a lei do `λ` indistinguível de
    // qualquer outra. *Foi o que aconteceu com a primeira fixtura, um tubo.*
    let rim: Vec<u32> = {
        let mut n: std::collections::BTreeMap<(u32, u32), usize> =
            std::collections::BTreeMap::new();
        for f in &faces {
            let v = f.verts();
            for k in 0..v.len() {
                let (a, b) = (v[k], v[(k + 1) % v.len()]);
                *n.entry(if a < b { (a, b) } else { (b, a) }).or_default() += 1;
            }
        }
        let mut out: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
        for ((a, b), c) in n {
            if c == 1 {
                out.insert(a);
                out.insert(b);
            }
        }
        out.into_iter().collect()
    };
    let mut positions = full.positions().to_vec();
    for (k, &v) in rim.iter().enumerate() {
        // Alternado ao longo da esfera: para o polo e de volta. Fica no raio 1.
        let s = if k % 2 == 0 { 1.9f32 } else { 1.0 };
        let p = positions[v as usize];
        let r = p[0].mul_add(p[0], p[2] * p[2]).sqrt().max(1.0e-9);
        let (nr, ny) = ((r / s).min(1.0), 0.0f32);
        let _ = ny;
        let y = (1.0 - nr * nr).max(0.0).sqrt() * p[1].signum();
        positions[v as usize] = [p[0] / r * nr, y, p[2] / r * nr];
    }
    ph2d_mesh::Mesh::from_parts(positions, faces).expect("a fixtura e' construida aqui")
}

/// ⭐⭐ **A LEI DO REBORDO ESTÁ DESLIGADA — e a régua dela VIVE.**
///
/// ⛔⛔ Ela foi construída, medida e **rejeitada** (ver [`super::BORDER_LAW`]): entrega o
/// perímetro exacto e faz o produto **pior em todas as colunas**, porque um rebordo esculpido
/// serrilha e o bordo é uma linha de feição. ⚠️ *Um gate que só afirmasse «está desligada»
/// aprovaria a lei ter sido apagada; este mede-a com ela ligada.*
#[test]
fn the_rim_law_is_off_and_the_ruler_is_alive() {
    // ⚠️ `const { }` como a casa faz nos manifestos: a asserção **é** sobre uma constante,
    // e é isso que se quer afirmar. ⛔ Ligar a `BORDER_LAW` passa a ser erro de COMPILAÇÃO
    // até alguém apagar esta linha e ler a tabela dela.
    const {
        assert!(!super::BORDER_LAW);
    };
    let mut mesh = sphere_with_a_small_hole(24, 36);
    let (loops0, len0) = border(&mesh);
    remesh_isotropic(&mut mesh, ALPHA);
    let (loops1, len1) = border(&mesh);
    eprintln!("desligada: {loops0} lacos / {len0:.4} ⇒ {loops1} lacos / {len1:.4}");
    assert_eq!(loops1, 1, "⛔ o buraco tem de continuar a ser UM buraco");
    assert!(
        (len1 - len0).abs() / len0 > 0.05,
        "⛔ com a lei DESLIGADA o rebordo tem de ANDAR -- se nao anda, a regua morreu"
    );
}

/// ⭐⭐⭐ **A LEI, quando ligada, PRESERVA O REBORDO — os laços E o perímetro.**
///
/// ⛔⛔ Até 2026-08-26 este passe não sabia o que era um bordo: alisava o rebordo na direcção
/// dos vizinhos **interiores** e projectava-o na **superfície** de referência. Medido na
/// `sculpt_t002` do artista, o perímetro do buraco crescia **30 %** — e nenhuma régua o via,
/// porque a contagem de arestas de bordo é função do passo.
///
/// ⚠️ **As DUAS metades são precisas.** Sem a do perímetro, um rebordo que encolhe passa;
/// sem a dos laços, um rebordo que se parte em dois passa.
#[test]
fn the_remesh_keeps_the_rim_where_the_artist_put_it() {
    let mut mesh = sphere_with_a_small_hole(24, 36);
    let (loops0, len0) = border(&mesh);
    assert_eq!(loops0, 1, "a fixtura tem de ter UM laco");
    let step = target_edge(&mesh, ALPHA);
    // ⭐⭐⭐ **A FIXTURA PROVA QUE CONTÉM O FENÓMENO** — e a régua é o quanto o rebordo
    // SERRILHA, porque é isso que o alisamento tem para encolher.
    //
    // ⛔⛔ A primeira fixtura desta linha era um **tubo**: rebordo circular, plano, sobre uma
    // superfície que passa por ele. **As três provas de mutação sobreviveram** — com ou sem
    // lei, o perímetro andava `0,04 %`. *Uma fixtura sem o fenómeno aprova qualquer lei.*
    let turn = mean_rim_turn(&mesh);
    eprintln!(
        "fixtura: perimetro {len0:.4} · passo {step:.4} · razao {:.1} · viragem media {turn:.1}°",
        len0 / step
    );
    assert!(
        turn > 60.0,
        "⛔ a fixtura tem de ter rebordo SERRILHADO, senao a lei do alisamento e' \
         indistinguivel: viragem media {turn:.1}°"
    );

    super::remesh_with(&mut mesh, ALPHA, true);
    let (loops1, len1) = border(&mesh);
    eprintln!("rebordo: {loops0} lacos / {len0:.4} ⇒ {loops1} lacos / {len1:.4}");
    assert_eq!(loops1, 1, "⛔ o remalhe partiu ou fechou o laco de bordo");
    // ⚠️ **A barra é a que a lei ENTREGA, não uma folga escolhida à mão.** Com a lei, esta
    // fixtura sai a `0,000 %` e a `sculpt_t002` do artista sai **exacta** (`0,6046` ⇒
    // `0,6046`). Uma barra de `1 %` — a primeira que escrevi — é cem vezes mais frouxa que
    // o código, e deixaria passar uma regressão inteira.
    let drift = (len1 - len0).abs() / len0;
    assert!(
        drift < 0.001,
        "⛔ o perimetro do rebordo andou {:.1}% ({len0:.4} ⇒ {len1:.4})",
        drift * 100.0
    );
}

/// ⭐⭐ **INÉRCIA: numa peça fechada a lei do rebordo não existe.**
#[test]
fn a_closed_piece_has_no_rim_to_keep() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(16, 24, 1.0);
    mesh.triangulate();
    assert_eq!(border(&mesh), (0, 0.0), "a esfera nao tem bordo");
    remesh_isotropic(&mut mesh, ALPHA);
    assert_eq!(
        border(&mesh),
        (0, 0.0),
        "⛔ o remalhe ABRIU uma peca fechada"
    );
}
