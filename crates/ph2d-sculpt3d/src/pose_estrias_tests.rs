//! **AS ESTRIAS DA POSE** — o report de 2026-09-17 (*«está quase bom! mas
//! surgem estrias»*, foto: arcos paralelos na casca à volta da região).
//!
//! ⚠️ **A régua é o ÂNGULO ENTRE FACES VIZINHAS**, e não a suavidade do
//! deslocamento: o que o olho lê é a NORMAL, e uma estria é uma
//! descontinuidade da *derivada* do campo, que um desvio de posição minúsculo
//! produz à vontade.
//!
//! ⚠️ **A população é a FAIXA**, e as outras duas entram como controlo — a
//! casca por deformar (o facetado próprio da esfera) e o NÚCLEO (onde a
//! rotação é rígida). *Sem elas, o facetado da esfera lê-se como o defeito.*
//!
//! ⛔⛔ **O corte para o vizinho [`super::pose_fronteira_tests`] é o SUJEITO:**
//! lá a **dobra**, que é binária (uma face está do avesso ou não está); aqui a
//! **suavidade**, que é contínua. *As duas medem a mesma faixa e nenhuma vê o
//! que a outra vê* — a dobra já lia `0` no dia em que o dono fotografou as
//! estrias.

use ph2d_mesh::Mesh;
use std::collections::BTreeMap;

use crate::{Brush, Dab, PoseControlos, SculptStroke, Symmetry, Verb};

fn esfera_do_report() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(256, 384, 1.0)
}

fn pincel(transicao: f32) -> Brush {
    Brush {
        verb: Verb::Pose,
        radius: 0.8,
        strength: 1.0,
        pose: PoseControlos {
            transicao,
            ..PoseControlos::default()
        },
        ..Brush::default()
    }
}

fn puxao(centro: [f32; 3], raio: f32, puxao: [f32; 3]) -> Dab {
    let l = (centro[0] * centro[0] + centro[1] * centro[1] + centro[2] * centro[2]).sqrt();
    let olho = [-centro[0] / l, -centro[1] / l, -centro[2] / l];
    Dab::pulling(centro, raio, olho, puxao)
}

fn normal(p: &[[f32; 3]], anel: &[u32]) -> [f64; 3] {
    let (a, b, c) = (
        p[anel[0] as usize],
        p[anel[1] as usize],
        p[anel[2] as usize],
    );
    let u = [
        f64::from(b[0] - a[0]),
        f64::from(b[1] - a[1]),
        f64::from(b[2] - a[2]),
    ];
    let w = [
        f64::from(c[0] - a[0]),
        f64::from(c[1] - a[1]),
        f64::from(c[2] - a[2]),
    ];
    let n = [
        u[1] * w[2] - u[2] * w[1],
        u[2] * w[0] - u[0] * w[2],
        u[0] * w[1] - u[1] * w[0],
    ];
    let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if l > 0.0 {
        [n[0] / l, n[1] / l, n[2] / l]
    } else {
        [0.0, 0.0, 0.0]
    }
}

/// As arestas interiores: `(face_a, face_b)`.
fn pares_de_faces(m: &Mesh) -> Vec<(usize, usize)> {
    let mut mapa: BTreeMap<(u32, u32), Vec<usize>> = BTreeMap::new();
    for (i, f) in m.faces().iter().enumerate() {
        let vs = f.verts();
        for k in 0..vs.len() {
            let (a, b) = (vs[k], vs[(k + 1) % vs.len()]);
            mapa.entry((a.min(b), a.max(b))).or_default().push(i);
        }
    }
    mapa.into_values()
        .filter(|v| v.len() == 2)
        .map(|v| (v[0], v[1]))
        .collect()
}

fn quantis(entrada: &[f64]) -> (f64, f64, f64, f64) {
    let mut v = entrada.to_vec();
    v.sort_by(f64::total_cmp);
    let q = |f: f64| v[((v.len() - 1) as f64 * f) as usize];
    (q(0.5), q(0.9), q(0.99), *v.last().unwrap())
}

/// O ângulo entre as normais de duas faces, em graus — o que o sombreamento lê.
fn angulo(p: &[[f32; 3]], a: &[u32], b: &[u32]) -> f64 {
    let (na, nb) = (normal(p, a), normal(p, b));
    (na[0] * nb[0] + na[1] * nb[1] + na[2] * nb[2])
        .clamp(-1.0, 1.0)
        .acos()
        .to_degrees()
}

/// A distância assinada à fronteira, recuperada do peso — a inversa da lei
/// `w = suave(0,5 − d/banda)`, por bissecção.
///
/// ⚠️ **É isto que torna a medição uma régua sobre o PRODUTO:** o campo de
/// distância não é público, e reconstruí-lo à parte mediria outro programa.
fn desfaz_suave(w: f64, banda: f64) -> f64 {
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    for _ in 0..40 {
        let m = 0.5 * (lo + hi);
        if m * m * (3.0 - 2.0 * m) < w {
            lo = m;
        } else {
            hi = m;
        }
    }
    (0.5 - 0.5 * (lo + hi)) * banda
}

/// ⭐⭐⭐ **A FAIXA NÃO ESTRIA** — o gate do report.
///
/// Medido na esfera do report (`97 922` vértices, arrasto `0,6`), o ângulo
/// entre faces vizinhas cujas DUAS faces tocam a faixa:
///
/// | transição | o que shipou (Dijkstra) | marcha por FACE | + interface sub-aresta |
/// |---|---|---|---|
/// | `0,60` p90 | `20,27°` | `7,84°` | **`6,42°`** |
/// | `1,00` p50 | `1,460°` | `1,184°` | **`1,162°`** |
/// | `1,00` p90 | `13,457°` | `4,077°` | **`3,551°`** |
/// | `1,00` p99 | `42,01°` | `16,73°` | **`11,81°`** |
/// | `2,00` p90 | `4,84°` | `1,52°` | **`1,52°`** |
///
/// ⚠️ A casca **por deformar** lê `p90 0,902°` e o **núcleo** depois do gesto
/// lê `p50 0,747°` — *a rotação ali é rígida, e é isso que prova que o resto é
/// da faixa e não da esfera.*
///
/// ⛔ **A metade (2) é o controlo positivo:** uma cura que simplesmente
/// alargasse a transição até nada dobrar passaria a (3) sem fazer o que o
/// artista pediu, e a (4) apanha a outra saída barata (diluir o núcleo).
#[test]
fn a_faixa_da_pose_nao_estria() {
    let malha = esfera_do_report();
    let pares = pares_de_faces(&malha);
    let antes = malha.positions().to_vec();

    let mut m = malha.clone();
    let b = pincel(PoseControlos::TRANSICAO_DE_FABRICA);
    let mut s = SculptStroke::default();
    s.begin(&m);
    s.dab(
        &mut m,
        &b,
        &puxao([0.0, 0.0, 1.0], b.radius, [0.6, 0.0, 0.0]),
        Symmetry::default(),
    );
    let sessao = s.pose_sessao().expect("a sessao vive durante o traco");
    let w: Vec<f32> = (0..antes.len())
        .map(|v| sessao.cadeia().peso_total(v))
        .collect();
    let depois = m.positions().to_vec();

    let na_faixa = |i: usize| {
        malha.faces()[i]
            .verts()
            .iter()
            .any(|&v| w[v as usize] > 1e-4 && w[v as usize] < 1.0 - 1e-4)
    };
    let no_nucleo = |i: usize| {
        malha.faces()[i]
            .verts()
            .iter()
            .all(|&v| w[v as usize] >= 1.0 - 1e-4)
    };
    let (mut faixa_antes, mut faixa_depois, mut nucleo) = (Vec::new(), Vec::new(), Vec::new());
    for &(i, j) in &pares {
        let ang = |p: &[[f32; 3]]| angulo(p, malha.faces()[i].verts(), malha.faces()[j].verts());
        if na_faixa(i) && na_faixa(j) {
            faixa_antes.push(ang(&antes));
            faixa_depois.push(ang(&depois));
        } else if no_nucleo(i) && no_nucleo(j) {
            nucleo.push(ang(&depois));
        }
    }

    // (1) — **a régua e a população são sãs:** na casca por deformar o mesmo
    // conjunto de arestas lê o facetado da própria esfera.
    assert!(
        faixa_depois.len() > 20_000 && nucleo.len() > 1_000,
        "a populacao encolheu ({} na faixa, {} no nucleo) — o arranjo deixou \
         de conter o que este gate mede",
        faixa_depois.len(),
        nucleo.len()
    );
    let (_, base90, _, _) = quantis(&faixa_antes);
    assert!(
        base90 <= 1.0,
        "a casca POR DEFORMAR ja' lê p90 {base90:.3}° nas mesmas arestas \
         (medido 0,902) — a regua ou a populacao mudaram, e as metades \
         seguintes deixam de ser sobre a pose"
    );

    let (p50, p90, p99, _) = quantis(&faixa_depois);

    // (2) — **o controlo positivo:** o gesto de facto dobra a faixa.
    assert!(
        p90 >= 1.5,
        "a faixa lê p90 {p90:.3}° (medido 3,551) — o gesto deixou de dobrar \
         nada, e a metade (3) passa a afirmar o vazio"
    );

    // (3) — e não estria. A barra está entre o que a cura entrega (`3,55`) e o
    // que a lei por ARESTAS entregava (`13,46`).
    assert!(
        p90 <= 6.0,
        "a faixa lê p90 {p90:.3}° e p99 {p99:.3}° (medido 3,551 e 11,81; a lei \
         por arestas lia 13,46 e 42,0) — a frente da distancia voltou a \
         caminhar por ARESTAS em vez de atravessar FACES"
    );

    // (4) — e o núcleo não paga por isso: ali a rotação é rígida.
    let (n50, _, _, _) = quantis(&nucleo);
    assert!(
        n50 <= 1.5,
        "o nucleo lê p50 {n50:.3}° (medido 0,747, e a esfera lisa 0,703) — a \
         faixa ficou lisa a' custa de diluir o miolo, que e' outro produto \
         (faixa p50 {p50:.3})"
    );
}

/// ⭐⭐⭐ **A DISTÂNCIA DA TRANSIÇÃO É A DO BARRO, E É ISOTRÓPICA.**
///
/// A causa das estrias, medida em isolado: numa esfera unitária a geodésica de
/// um vértice à fronteira de uma calota **calcula-se à mão** (`θ_v − θ_r`),
/// logo a lei tem um oráculo exacto e de graça.
///
/// | `d_lei / d_exacto` | p50 | p90 | p99 |
/// |---|---|---|---|
/// | por ARESTAS (o que shipou) | `1,069` | **`1,316`** | `1,368` |
/// | a atravessar FACES | `0,997` | `1,007` | `1,024` |
/// | + interface sub-aresta | **`1,002`** | **`1,012`** | `1,032` |
///
/// ⚠️⚠️ **O `1,316` é o defeito inteiro:** um caminho por arestas só toma as
/// direcções que a malha tem, logo `d` fica quase constante em cada *anel* do
/// grafo — as curvas de nível deixam de ser círculos e passam a ser os
/// **losangos** da malha, o peso fica em patamares, e cada degrau é uma dobra.
///
/// ⚠️ **A cauda de baixo NÃO se aperta e é declarada:** junto da fronteira a
/// razão desce a `~0,92`, porque a pertença é binária e o corte só se conhece
/// **dentro** da aresta que o atravessa. É o chão da informação que existe, e
/// não um defeito da marcha.
#[test]
fn a_distancia_da_transicao_e_a_do_barro() {
    let malha = esfera_do_report();
    let pos: Vec<ph2d_pose::V3> = malha.positions().to_vec();
    let escondido = vec![false; pos.len()];
    let viz = ph2d_pose::Vizinhanca::construir(
        pos.len(),
        malha.faces().iter().map(|f| f.verts()),
        &escondido,
    );

    let cursor = [0.0f64, 0.0, 1.0];
    let raio_da_calota = 0.5f64;
    let banda = 0.8f64;
    let angulo_ao_cursor = |v: usize| -> f64 {
        let p = pos[v];
        (f64::from(p[0]) * cursor[0] + f64::from(p[1]) * cursor[1] + f64::from(p[2]) * cursor[2])
            .clamp(-1.0, 1.0)
            .acos()
    };

    let mut w: Vec<f32> = (0..pos.len())
        .map(|v| f32::from(u8::from(angulo_ao_cursor(v) <= raio_da_calota)))
        .collect();
    ph2d_pose::pesos::por_distancia(&viz, &pos, &mut w, banda as f32);

    let mut razao = Vec::new();
    for (v, peso) in w.iter().copied().enumerate() {
        let exacta = angulo_ao_cursor(v) - raio_da_calota;
        // ⚠️ Perto da fronteira a razão é dominada pela meia aresta da semente,
        // que é o chão da informação; a lei mede-se onde ela é observável.
        if !(0.05..=banda * 0.5).contains(&exacta.abs()) || peso <= 1e-3 || peso >= 1.0 - 1e-3 {
            continue;
        }
        razao.push(desfaz_suave(f64::from(peso), banda) / exacta);
    }

    assert!(
        razao.len() > 5_000,
        "so' {} vertices caem na faixa observavel — o arranjo deixou de conter \
         o que este gate mede",
        razao.len()
    );
    let (p50, p90, p99, _) = quantis(&razao);
    assert!(
        (0.98..1.02).contains(&p50),
        "a distancia da lei lê p50 {p50:.4} da exacta (medido 1,002) — ela \
         deixou de ser a distancia sobre a superficie"
    );
    assert!(
        p90 <= 1.05 && p99 <= 1.10,
        "a distancia da lei lê p90 {p90:.4} e p99 {p99:.4} da exacta (medido \
         1,012 e 1,032; a lei por arestas lia 1,316 e 1,368) — a frente voltou \
         a caminhar por ARESTAS, e as curvas de nivel voltam a ser os losangos \
         da malha"
    );
}

/// ⭐⭐ **A FRONTEIRA DA TRANSIÇÃO É LISA** — a metade que a distância não vê.
///
/// A régua é a **rugosidade do campo**: `w(v) − média dos vizinhos`, em
/// unidades do degrau que uma aresta vale numa rampa perfeita
/// (`aresta / banda`). Uma rampa linear lê `0`; uma dobra lê `O(1)`.
///
/// ⚠️⚠️ **Ela existe porque o gate da distância NÃO apanha isto:** com o corte
/// cravado a meia aresta a razão `d_lei/d_exacto` mal se move (`p90 1,007`
/// contra `1,012`) e o sombreamento fica dentro da barra — *o erro da semente é
/// sub-aresta, logo invisível a uma régua de distância, e mesmo assim é o que
/// serrilha a fronteira.* Medido na esfera do report:
///
/// | `|d|` em arestas | corte a meia aresta | corte sub-aresta |
/// |---|---|---|
/// | **`0`–`4`** p90 | **`0,266`** | **`0,141`** |
/// | `4`–`8` p90 | `0,072` | `0,045` |
/// | `8`–`12` p90 | `0,038` | `0,029` |
/// | `20`+ p90 | `0,035` | `0,035` |
///
/// ⭐ **A última linha é o controlo:** longe da fronteira as duas leis leem o
/// mesmo, que é o chão da discretização — *é isso que prova que a barra da
/// primeira linha mede a SEMENTE e não a malha.*
#[test]
fn a_fronteira_da_transicao_e_lisa() {
    let malha = esfera_do_report();
    let mut m = malha.clone();
    let b = pincel(PoseControlos::TRANSICAO_DE_FABRICA);
    let mut s = SculptStroke::default();
    s.begin(&m);
    s.dab(
        &mut m,
        &b,
        &puxao([0.0, 0.0, 1.0], b.radius, [0.05, 0.0, 0.0]),
        Symmetry::default(),
    );
    let sessao = s.pose_sessao().expect("a sessao vive durante o traco");
    let pos = malha.positions();
    let w: Vec<f64> = (0..pos.len())
        .map(|v| f64::from(sessao.cadeia().peso_total(v)))
        .collect();

    let mut viz: Vec<Vec<u32>> = vec![Vec::new(); pos.len()];
    let (mut soma, mut n_ar) = (0.0f64, 0u64);
    for f in malha.faces() {
        let vs = f.verts();
        for k in 0..vs.len() {
            let (a, c) = (vs[k], vs[(k + 1) % vs.len()]);
            viz[a as usize].push(c);
            viz[c as usize].push(a);
            let (pa, pc) = (pos[a as usize], pos[c as usize]);
            soma += f64::from(
                ((pa[0] - pc[0]).powi(2) + (pa[1] - pc[1]).powi(2) + (pa[2] - pc[2]).powi(2))
                    .sqrt(),
            );
            n_ar += 1;
        }
    }
    let aresta = soma / n_ar as f64;
    let banda = f64::from(b.radius) * f64::from(PoseControlos::TRANSICAO_DE_FABRICA);
    let degrau = aresta / banda;
    let (mut junto, mut longe) = (Vec::new(), Vec::new());
    for (peso, anel) in w.iter().copied().zip(viz.iter()) {
        if peso <= 1e-4 || peso >= 1.0 - 1e-4 || anel.is_empty() {
            continue;
        }
        let media: f64 = anel.iter().map(|&u| w[u as usize]).sum::<f64>() / anel.len() as f64;
        let rug = (peso - media).abs() / degrau;
        if (desfaz_suave(peso, banda) / aresta).abs() <= 4.0 {
            junto.push(rug);
        } else {
            longe.push(rug);
        }
    }

    assert!(
        junto.len() > 1_000 && longe.len() > 5_000,
        "a populacao encolheu ({} junto da fronteira, {} longe) — o arranjo \
         deixou de conter o que este gate mede",
        junto.len(),
        longe.len()
    );

    // (1) — **o controlo:** longe da fronteira o campo já é liso, e é esse o
    // chão da discretização com que a primeira metade se compara.
    let (_, longe90, _, _) = quantis(&longe);
    assert!(
        longe90 <= 0.06,
        "longe da fronteira o campo lê p90 {longe90:.4} degraus (medido 0,035) \
         — a marcha perdeu suavidade, e a metade (2) deixa de ser sobre a \
         SEMENTE"
    );

    // (2) — e junto dela também: a fronteira fica DENTRO da aresta que a
    // atravessa, e não cravada a meio dela.
    let (_, junto90, junto99, _) = quantis(&junto);
    assert!(
        junto90 <= 0.20,
        "junto da fronteira o campo lê p90 {junto90:.4} e p99 {junto99:.4} \
         degraus (medido 0,141 e 0,342; com o corte cravado a meia aresta lia \
         0,266 e 0,510) — a semente voltou a cravar a fronteira a meio da \
         aresta, e a transicao sai serrilhada"
    );
}

/// ⭐ **Onde a dobra morre AGORA** — a cura das estrias mexeu no joelho, e o
/// controlo do gate vizinho deixou de conter o fenómeno.
#[test]
#[ignore = "sonda"]
fn diag_o_joelho_da_dobra() {
    let malha = esfera_do_report();
    let viradas = |t: f32, arrasto: f32| -> usize {
        let mut m = malha.clone();
        let antes = malha.positions().to_vec();
        let b = pincel(t);
        let mut s = SculptStroke::default();
        s.begin(&m);
        s.dab(
            &mut m,
            &b,
            &puxao([0.0, 0.0, 1.0], b.radius, [arrasto, 0.0, 0.0]),
            Symmetry::default(),
        );
        let depois = m.positions().to_vec();
        m.faces()
            .iter()
            .filter(|f| {
                let (a, b) = (normal(&antes, f.verts()), normal(&depois, f.verts()));
                a[0] * b[0] + a[1] * b[1] + a[2] * b[2] < 0.0
            })
            .count()
    };
    print!("{:>9}", "transicao");
    for arrasto in [0.6f32, 0.9, 1.2] {
        print!("  arrasto {arrasto:.2}");
    }
    println!();
    for t in [0.1f32, 0.2, 0.3, 0.4, 0.5, 0.667, 0.8, 1.0, 1.2] {
        print!("{t:>9.3}");
        for arrasto in [0.6f32, 0.9, 1.2] {
            print!("{:>14}", viradas(t, arrasto));
        }
        println!();
    }
}

/// ⭐ **O relógio do pen-down** — a marcha atravessa faces (mais cara por
/// vértice) mas PÁRA na meia-banda (a Dijkstra que ela substitui varria a malha
/// inteira). O saldo mede-se, não se supõe.
#[test]
#[ignore = "sonda"]
fn diag_o_relogio_do_pen_down() {
    for (nome, malha) in [
        ("media  24 386", ph2d_mesh::shapes::uv_sphere(128, 192, 1.0)),
        ("fina   97 922", esfera_do_report()),
    ] {
        for t in [0.0f32, 1.0, 2.0] {
            let mut melhor = f64::INFINITY;
            for _ in 0..5 {
                let mut m = malha.clone();
                let b = pincel(t);
                let mut s = SculptStroke::default();
                s.begin(&m);
                let t0 = std::time::Instant::now();
                s.dab(
                    &mut m,
                    &b,
                    &puxao([0.0, 0.0, 1.0], b.radius, [0.3, 0.0, 0.0]),
                    Symmetry::default(),
                );
                melhor = melhor.min(t0.elapsed().as_secs_f64() * 1e3);
            }
            println!(
                "{nome} verts · transicao {t:.1} · pen-down {melhor:.3} ms  (load {})",
                std::fs::read_to_string("/proc/loadavg")
                    .unwrap_or_default()
                    .split_whitespace()
                    .next()
                    .unwrap_or("?")
            );
        }
    }
}

/// Sonda de uma corrida só: a impressão digital do campo de peso, para provar
/// que o corte da marcha na meia-banda **não muda um bit**.
#[test]
#[ignore = "sonda"]
fn diag_a_impressao_do_campo() {
    let malha = esfera_do_report();
    let mut m = malha.clone();
    let b = pincel(PoseControlos::TRANSICAO_DE_FABRICA);
    let mut s = SculptStroke::default();
    s.begin(&m);
    s.dab(
        &mut m,
        &b,
        &puxao([0.0, 0.0, 1.0], b.radius, [0.3, 0.0, 0.0]),
        Symmetry::default(),
    );
    let sessao = s.pose_sessao().expect("a sessao vive durante o traco");
    let mut h: u64 = 1469598103934665603;
    for v in 0..malha.positions().len() {
        for by in sessao.cadeia().peso_total(v).to_bits().to_le_bytes() {
            h ^= u64::from(by);
            h = h.wrapping_mul(1099511628211);
        }
    }
    println!("impressao do campo: {h:016x}");
}
