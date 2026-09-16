//! Gates da BORDA do corte — a costura que a curva de interseção deixa.
//!
//! Filho (`#[path]`) do [`super`], e os ajudantes de fixtura são os dele.

use super::*;
use ph2d_mesh::{Mesh, shapes};

/// ⭐⭐⭐ **A COSTURA DO CORTE É UTILIZÁVEL — no caminho do PRODUTO.**
///
/// Report do dono (2026-09-15): *«o algoritmo remesh produz bordas mais corretas
/// que o algoritmo da Box Trim; melhore a topologia das bordas do corte»*.
///
/// ⚠️ **Este é o gate do caminho REAL, e por isso vive aqui:** a `ph2d-mesh-bool`
/// só consegue montar um cubo de seis faces, e ali o mesmo corte mede `256`;
/// a lâmina que o produto entrega é **tesselada à densidade da peça**, e com ela
/// o pior triângulo mede **`33`**. *A régua de uma lei mora onde a entrada real
/// dela é construída.*
#[test]
fn a_costura_do_corte_e_utilizavel() {
    let bola = shapes::sphere_with_triangles(50_000, 1.0);
    let tris: usize = bola
        .faces()
        .iter()
        .map(|f| f.verts().len().saturating_sub(2))
        .sum();
    let alvo = ph2d_mesh::edge_for_tri_count(bola.surface_area(), tris as f32);
    let n = 200usize;
    let anel: Vec<[f32; 2]> = (0..n)
        .map(|i| {
            let t = i as f32 / n as f32 * std::f32::consts::TAU;
            [0.6 * t.cos(), 0.6 * t.sin()]
        })
        .collect();
    let lamina = prisma(
        &anel,
        &raios_orto(&anel),
        &plano(),
        &bola,
        Profundidade::DaPeca,
        Paredes::Fixas,
        Resolucao::Ate(alvo),
    )
    .expect("o prisma");
    let out = ph2d_mesh_bool::corta(&bola, &lamina, ph2d_mesh_bool::Op::Subtrair).expect("o corte");

    let p = out.positions();
    let mut t3 = Vec::new();
    for f in out.faces() {
        f.triangles(&mut t3);
    }
    let mut asp: Vec<f32> = t3
        .iter()
        .map(|x| {
            let (a, b, c) = (p[x[0] as usize], p[x[1] as usize], p[x[2] as usize]);
            let e = |u: [f32; 3], v: [f32; 3]| {
                ((u[0] - v[0]).powi(2) + (u[1] - v[1]).powi(2) + (u[2] - v[2]).powi(2)).sqrt()
            };
            let (l0, l1, l2) = (e(a, b), e(b, c), e(c, a));
            let s = (l0 + l1 + l2) * 0.5;
            let area = (s * (s - l0) * (s - l1) * (s - l2)).max(0.0).sqrt();
            if area > 1e-14 {
                l0.max(l1).max(l2) * s / (2.0 * area)
            } else {
                f32::INFINITY
            }
        })
        .collect();
    asp.sort_by(f32::total_cmp);
    let pior = asp.last().copied().unwrap_or(f32::INFINITY);
    assert!(
        pior < 60.0,
        "o pior triângulo do corte mede {pior:.0} de aspecto (medido `33`) —          antes da limpeza da costura ele media `2 573 809`"
    );
    // ⚠️ **A mediana é a outra metade:** um `MAX` bom com a mediana podre
    // significaria que a limpeza trocou um defeito raro por um geral.
    assert!(
        asp[asp.len() / 2] < 4.0,
        "a mediana do aspecto subiu para {:.2}",
        asp[asp.len() / 2]
    );
    assert_eq!(ph2d_mesh::border_edges(&out), 0, "o corte abriu a peça");
    assert_eq!(
        ph2d_mesh::non_manifold_edges(&out),
        0,
        "o corte deixou aresta com três faces"
    );
}

#[test]
#[ignore = "sonda: o PICO na borda do corte"]
fn diag_o_pico_na_borda() {
    let bola = shapes::sphere_with_triangles(50_000, 1.0);
    let tris: usize = bola
        .faces()
        .iter()
        .map(|f| f.verts().len().saturating_sub(2))
        .sum();
    let alvo = ph2d_mesh::edge_for_tri_count(bola.surface_area(), tris as f32);
    // ⭐⭐ **O CENTRO É DESLOCADO de propósito, e é isso que muda tudo:** com o
    // círculo centrado a borda do corte vive a `|z| = 0,8` e **nunca encontra a
    // silhueta** da peça (que é o equador). A foto do dono mostra o pico
    // exactamente onde o corte SAI pela beira — logo a fixtura centrada não
    // contém o fenómeno.
    let (n, raio) = (200usize, 0.6f32);
    let centro_x: f32 = std::env::var("PH2D_PICO_CX")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.8);
    let anel: Vec<[f32; 2]> = (0..n)
        .map(|i| {
            let t = i as f32 / n as f32 * std::f32::consts::TAU;
            [centro_x + raio * t.cos(), raio * t.sin()]
        })
        .collect();
    let lamina = prisma(
        &anel,
        &raios_orto(&anel),
        &plano(),
        &bola,
        Profundidade::DaPeca,
        Paredes::Fixas,
        Resolucao::Ate(alvo),
    )
    .expect("o prisma");

    // ⭐ A borda do corte é, por construção, o CÍRCULO de raio `raio` no plano
    // `xy` — logo o desvio de cada vértice dela é exacto e não precisa de
    // ordenar curva nenhuma.
    let retrato = |m: &Mesh, rot: &str| {
        let p = m.positions();
        let mut t3 = Vec::new();
        for f in m.faces() {
            f.triangles(&mut t3);
        }
        let na_parede =
            |i: u32| ((p[i as usize][0] - centro_x).hypot(p[i as usize][1]) - raio).abs() < 1e-3;
        // A BORDA: vértices de uma face que tem uns na parede e outros fora.
        let mut borda = std::collections::BTreeSet::new();
        for t in &t3 {
            let c = t.iter().filter(|&&i| na_parede(i)).count();
            if c > 0 && c < 3 {
                for &i in t {
                    if na_parede(i) {
                        borda.insert(i);
                    }
                }
            }
        }
        // O que um pico é: um vértice de borda cujo raio foge do círculo.
        let mut d: Vec<f32> = borda
            .iter()
            .map(|&i| ((p[i as usize][0] - centro_x).hypot(p[i as usize][1]) - raio).abs())
            .collect();
        d.sort_by(f32::total_cmp);
        // ⭐⭐ **A BORDA TEM POSIÇÃO EXACTA CONHECIDA:** o cilindro `x²+y²=r²`
        // sobre a esfera unitária intersecta em `|z| = √(1−r²)` — DOIS círculos
        // planos. ⇒ o desvio de `|z|` a esse valor é o pico, em unidades de
        // cena, sem ordenar curva nenhuma.
        let z_certo = (1.0 - raio * raio).sqrt();
        let mut dz: Vec<f32> = borda
            .iter()
            .map(|&i| (p[i as usize][2].abs() - z_certo).abs())
            .collect();
        dz.sort_by(f32::total_cmp);
        println!(
            "{rot:10} borda V={:4} | raio: p50={:.2e} MAX={:.2e} ({:.3} do alvo) \
             | |z|−{z_certo:.3}: p50={:.2e} p99={:.2e} MAX={:.2e} ({:.2} do alvo)",
            borda.len(),
            d[d.len() / 2],
            d.last().copied().unwrap_or(f32::NAN),
            d.last().copied().unwrap_or(f32::NAN) / alvo,
            dz[dz.len() / 2],
            dz[(dz.len() - 1) * 99 / 100],
            dz.last().copied().unwrap_or(f32::NAN),
            dz.last().copied().unwrap_or(f32::NAN) / alvo,
        );
    };
    // ⭐ O RETRATO DAS CUNHAS: o que é, de facto, um triângulo mau na borda.
    let cunhas = |m: &Mesh| {
        let p = m.positions();
        let mut t3 = Vec::new();
        for f in m.faces() {
            f.triangles(&mut t3);
        }
        let antigos: std::collections::BTreeSet<[u32; 3]> = bola
            .positions()
            .iter()
            .map(|v| [v[0].to_bits(), v[1].to_bits(), v[2].to_bits()])
            .collect();
        let e_antigo = |i: u32| {
            antigos.contains(&{
                let v = p[i as usize];
                [v[0].to_bits(), v[1].to_bits(), v[2].to_bits()]
            })
        };
        let mut piores: Vec<(f32, [f32; 3], usize, f32)> = Vec::new();
        for t in &t3 {
            let (a, b, c) = (p[t[0] as usize], p[t[1] as usize], p[t[2] as usize]);
            let e = |u: [f32; 3], v: [f32; 3]| {
                ((u[0] - v[0]).powi(2) + (u[1] - v[1]).powi(2) + (u[2] - v[2]).powi(2)).sqrt()
            };
            let (l0, l1, l2) = (e(a, b), e(b, c), e(c, a));
            let s = (l0 + l1 + l2) * 0.5;
            let area = (s * (s - l0) * (s - l1) * (s - l2)).max(0.0).sqrt();
            if area <= 0.0 {
                continue;
            }
            let asp = l0.max(l1).max(l2) * s / (2.0 * area);
            if asp > 20.0 {
                piores.push((
                    asp,
                    [l0, l1, l2],
                    t.iter().filter(|&&i| e_antigo(i)).count(),
                    // a ALTURA do triângulo: a área sobre a aresta mais longa.
                    2.0 * area / l0.max(l1).max(l2),
                ));
            }
        }
        piores.sort_by(|a, b| b.0.total_cmp(&a.0));
        println!("  cunhas com aspecto > 20: {}", piores.len());
        for (asp, l, antigos, h) in piores.iter().take(6) {
            println!(
                "    aspecto {asp:8.1}  arestas [{:.2e} {:.2e} {:.2e}]  altura {h:.2e} \
                 ({:.4} do alvo)  vértices ANTIGOS: {antigos}/3",
                l[0],
                l[1],
                l[2],
                h / alvo
            );
        }
    };
    println!("\nalvo de aresta = {alvo:.4}");
    let curado =
        ph2d_mesh_bool::corta(&bola, &lamina, ph2d_mesh_bool::Op::Subtrair).expect("corte");
    {
        // ⛔ A pergunta decisiva: alguma coisa SAI da esfera? Os vértices da peça
        // estão a raio `1` e todo ponto novo nasce sobre uma face dela (raio
        // `≤ 1`) — logo um raio acima de `1` é geometria a espetar.
        let p = curado.positions();
        let mut r: Vec<f32> = p
            .iter()
            .map(|v| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt())
            .collect();
        r.sort_by(f32::total_cmp);
        // E as faces DUPLICADAS (a «almofada» que a linha do quad remesh pagou).
        let mut t3 = Vec::new();
        for f in curado.faces() {
            f.triangles(&mut t3);
        }
        let mut chaves = std::collections::BTreeMap::new();
        for t in &t3 {
            let mut k = *t;
            k.sort_unstable();
            *chaves.entry(k).or_insert(0usize) += 1;
        }
        println!(
            "  raio MAX={:.6} (acima de 1 por {:.2e})  ·  faces repetidas: {}  ·  \
             componentes ligados: {}",
            r.last().copied().unwrap_or(f32::NAN),
            r.last().copied().unwrap_or(1.0) - 1.0,
            {
                // Que ESPÉCIE de repetição: o mesmo enrolamento, ou o espelho?
                let mut iguais = 0usize;
                let mut espelhos = 0usize;
                let mut vistos: std::collections::BTreeMap<[u32; 3], Vec<[u32; 3]>> =
                    std::collections::BTreeMap::new();
                for t in &t3 {
                    let mut k = *t;
                    k.sort_unstable();
                    vistos.entry(k).or_default().push(*t);
                }
                for (_, v) in vistos.iter().filter(|(_, v)| v.len() > 1) {
                    let ciclo = |a: &[u32; 3], b: &[u32; 3]| {
                        (0..3).any(|r| (0..3).all(|i| a[i] == b[(i + r) % 3]))
                    };
                    if ciclo(&v[0], &v[1]) {
                        iguais += 1;
                    } else {
                        espelhos += 1;
                    }
                    eprintln!("    REPETIDA {:?} e {:?}", v[0], v[1]);
                    for t in v {
                        let q = |i: u32| p[i as usize];
                        eprintln!(
                            "      {:?} · raios {:.4} {:.4} {:.4}",
                            t,
                            q(t[0])[0].hypot(q(t[0])[1]),
                            q(t[1])[0].hypot(q(t[1])[1]),
                            q(t[2])[0].hypot(q(t[2])[1])
                        );
                    }
                }
                format!(
                    "{} (iguais {iguais}, espelhos {espelhos})",
                    iguais + espelhos
                )
            },
            componentes(&curado),
        );
        println!(
            "  bordo={}  nao-manifold={}",
            ph2d_mesh::border_edges(&curado),
            ph2d_mesh::non_manifold_edges(&curado),
        );
    }
    cunhas(&curado);
    retrato(&curado, "CURADO");
    retrato(
        &ph2d_mesh_bool::limpa_a_costura(&curado, &bola),
        "2x CURADO",
    );
}

/// Quantos pedaços desligados a malha tem — a régua que apanha uma «almofada».
fn componentes(m: &Mesh) -> usize {
    let n = m.positions().len();
    let mut pai: Vec<usize> = (0..n).collect();
    fn raiz(p: &mut [usize], mut i: usize) -> usize {
        while p[i] != i {
            p[i] = p[p[i]];
            i = p[i];
        }
        i
    }
    let mut tris = Vec::new();
    for f in m.faces() {
        f.triangles(&mut tris);
    }
    for t in &tris {
        for w in [(t[0], t[1]), (t[1], t[2])] {
            let (a, b) = (raiz(&mut pai, w.0 as usize), raiz(&mut pai, w.1 as usize));
            if a != b {
                pai[a] = b;
            }
        }
    }
    (0..n)
        .map(|i| raiz(&mut pai, i))
        .collect::<std::collections::BTreeSet<_>>()
        .len()
}

/// ⭐⭐⭐ **A BORDA DO CORTE NÃO TEM ALMOFADAS — e a fixtura é a que as CONTÉM.**
///
/// > Report do dono (2026-09-15, com foto): *«Borda melhorou mas não está
/// > perfeita»* — um **espigão** a sair da silhueta da peça.
///
/// Com o corte a **SAIR pela beira** da peça aparecem pares **espelhados** — o
/// mesmo triângulo duas vezes, um virado ao contrário. Juntos encerram volume
/// **ZERO**: são uma aba infinitamente fina, e é isso que o sombreamento desenha
/// como uma farpa.
///
/// ⛔⛔⛔ **E quem os cria é a limpeza da wave anterior, não o motor.** A 1.ª
/// redacção deste gate afirmava o contrário e **reprovou no próprio controlo**:
/// a saída crua traz `0` almofadas em todas as posições varridas. *Fundir dois
/// vértices faz dois triângulos distintos passarem a ter o mesmo trio* ⇒ o
/// controlo certo não é o corte CRU, é a limpeza a DIZER quantas descartou.
///
/// ⚠️⚠️ **A fixtura CENTRADA não contém o fenómeno:** com o círculo no meio da
/// peça a borda vive a `|z| = 0,8` e **nunca encontra a silhueta** (que é o
/// equador). ⛔ E a lâmina GROSSA (o cubo de seis faces) também não as produz em
/// posição nenhuma — só o cilindro tesselado a sair pela beira.
///
/// ⭐ **As três metades:** a saída não tem nenhuma · a limpeza descarta pelo
/// menos uma em alguma posição (senão o gate mede o nada) · e a peça continua
/// **fechada e manifold** depois de os DOIS lados de cada par saírem.
#[test]
fn a_borda_do_corte_nao_tem_almofadas() {
    let bola = shapes::sphere_with_triangles(50_000, 1.0);
    let tris: usize = bola
        .faces()
        .iter()
        .map(|f| f.verts().len().saturating_sub(2))
        .sum();
    let alvo = ph2d_mesh::edge_for_tri_count(bola.surface_area(), tris as f32);
    let almofadas = |m: &Mesh| {
        let mut t3 = Vec::new();
        for f in m.faces() {
            f.triangles(&mut t3);
        }
        let mut por_chave: std::collections::BTreeMap<[u32; 3], Vec<[u32; 3]>> =
            std::collections::BTreeMap::new();
        for t in &t3 {
            let mut k = *t;
            k.sort_unstable();
            por_chave.entry(k).or_default().push(*t);
        }
        por_chave
            .values()
            .filter(|v| {
                v.len() > 1 && !(0..3).any(|r| (0..3).all(|i| v[0][i] == v[1][(i + r) % 3]))
            })
            .count()
    };

    let mut com_fenomeno = 0usize;
    for centro_x in [0.0f32, 0.5, 0.8, 0.95] {
        let (n, raio) = (200usize, 0.6f32);
        let anel: Vec<[f32; 2]> = (0..n)
            .map(|i| {
                let t = i as f32 / n as f32 * std::f32::consts::TAU;
                [centro_x + raio * t.cos(), raio * t.sin()]
            })
            .collect();
        let lamina = prisma(
            &anel,
            &raios_orto(&anel),
            &plano(),
            &bola,
            Profundidade::DaPeca,
            Paredes::Fixas,
            Resolucao::Ate(alvo),
        )
        .expect("o prisma");
        let cru = ph2d_mesh_bool::corta_cru(&bola, &lamina, ph2d_mesh_bool::Op::Subtrair)
            .expect("o corte cru");
        assert_eq!(
            almofadas(&cru),
            0,
            "centro em x={centro_x}: o MOTOR passou a emitir almofadas — a \
             explicação deste gate mudou, e o doc dele tem de mudar com ela"
        );
        let (limpo, descartadas) = ph2d_mesh_bool::costura::limpa_a_costura_relatando(&cru, &bola);
        com_fenomeno += usize::from(descartadas > 0);
        assert_eq!(
            almofadas(&limpo),
            0,
            "centro em x={centro_x}: sobrou uma almofada — é o espigão do report"
        );
        assert_eq!(
            ph2d_mesh::border_edges(&limpo),
            0,
            "centro em x={centro_x}: descartar a almofada ABRIU a peça"
        );
        assert_eq!(
            ph2d_mesh::non_manifold_edges(&limpo),
            0,
            "centro em x={centro_x}: descartar a almofada deixou aresta com três faces"
        );
    }
    // ⛔⛔ **O CONTROLO, e ele é o que este gate tem de mais caro:** sem uma
    // posição em que o CRU traga almofadas, tudo acima passa por vácuo — que é
    // exactamente o estado em que a régua estava antes do report.
    assert!(
        com_fenomeno > 0,
        "nenhuma das posições produziu almofada no corte CRU — a fixtura deixou          de conter o fenómeno, e este gate passou a medir o nada"
    );
}
