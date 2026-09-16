//! Gates da fronteira do corte.

use super::*;
use ph2d_mesh::shapes;

/// Uma lâmina: o cubo da casa, deslocado em `x`.
fn lamina(cx: f32, meia: f32) -> Mesh {
    let mut m = shapes::cube(meia * 2.0);
    for p in m.positions_mut() {
        p[0] += cx;
    }
    m
}

/// **O corte tira volume — e a peça continua a ENCERRAR volume.**
///
/// ⚠️ A segunda metade não é decoração: um corte que abre a peça deixa-a sem
/// dentro, e a operação seguinte (outra booleana, um remesh) passa a recusar.
/// *Um corte que fecha é a diferença entre uma ferramenta e uma armadilha.*
#[test]
fn um_corte_tira_volume_e_a_peca_continua_fechada() {
    let bola = shapes::uv_sphere(24, 32, 1.0);
    assert!(bola.is_closed(), "a fixtura tem de entrar fechada");
    let antes = bola.bounds().max[0];

    let out = corta(&bola, &lamina(1.2, 0.6), Op::Subtrair).expect("o corte");

    assert!(
        out.bounds().max[0] < antes - 0.05,
        "a lâmina estava em x ∈ [0,6; 1,8] e o `x` máximo não recuou: {} contra {antes}",
        out.bounds().max[0]
    );
    assert_eq!(
        ph2d_mesh::border_edges(&out),
        0,
        "o corte deixou a peça ABERTA — ela deixa de ter dentro"
    );
}

/// ⭐⭐⭐ **A PROPRIEDADE QUE DECIDE A ARQUITECTURA: longe do corte, nem um bit.**
///
/// ⚠️ **Este é o gate que separa um CORTE de um corte-mais-remalhamento.** Uma
/// escultura tem densidade **autorada** — fino onde o artista trabalhou, grosso
/// onde não —, e a rota barata desta casa (`malha → campo → malha`) devolve-a
/// uniforme: medido, `0` vértices preservados em `6` de `6` células, contra
/// `26 533` de `26 533` aqui (`SPEC_trim_gesture.md` §1.1–§1.2, e a sonda
/// `ph2d_sdf::remesh::tests::diag_o_preco_da_volta_por_campo` do nosso lado).
///
/// ⇒ *se alguém trocar este motor por um que re-tessela tudo, este gate é o que
/// reprova* — e nenhuma contagem agregada o faria, porque a contagem de saída
/// de uma rota por voxel até pode ser parecida.
#[test]
fn longe_do_corte_nenhum_vertice_se_move_um_bit() {
    let bola = shapes::uv_sphere(24, 32, 1.0);
    let longe: std::collections::BTreeSet<[u32; 3]> = bola
        .positions()
        .iter()
        .filter(|p| p[0] < -0.5)
        .map(|p| [p[0].to_bits(), p[1].to_bits(), p[2].to_bits()])
        .collect();
    assert!(
        longe.len() > 50,
        "a fixtura tem de ter população longe do corte, e tem {}",
        longe.len()
    );

    let out = corta(&bola, &lamina(1.2, 0.6), Op::Subtrair).expect("o corte");
    let saida: std::collections::BTreeSet<[u32; 3]> = out
        .positions()
        .iter()
        .map(|p| [p[0].to_bits(), p[1].to_bits(), p[2].to_bits()])
        .collect();

    let perdidos = longe.difference(&saida).count();
    assert_eq!(
        perdidos,
        0,
        "{perdidos} de {} vértices do lado OPOSTO ao corte mudaram — este motor \
         re-tessela longe de onde a lâmina passou, e isso destrói a densidade \
         que o artista autorou",
        longe.len()
    );
}

/// ⛔⛔⛔ **A RECUSA ACONTECE ANTES DE OPERAR — e a metade que importa é que ela
/// NÃO é `Ok` sobre uma malha vazia.**
///
/// Medido no motor cru, fora do repo: com peça aberta o estado da ENTRADA diz
/// `NotManifold` e o do RESULTADO diz **«sem erro»**, com `0` vértices. ⇒ quem
/// verificar o resultado devolve `Ok(vazio)` e o artista perde a escultura.
///
/// ⚠️ **O CONTROLO está dentro do gate:** a mesma lâmina, na mesma posição,
/// sobre a peça FECHADA, tem de cortar. Sem ele este teste passaria com uma
/// porta que recusa **sempre**.
#[test]
fn uma_peca_aberta_e_recusada_e_nunca_devolvida_vazia() {
    let bola = shapes::uv_sphere(24, 32, 1.0);
    let mut aberta = Mesh::from_parts(
        bola.positions().to_vec(),
        bola.faces()[..bola.face_count() - 1].to_vec(),
    )
    .expect("a peça com uma face a menos");
    assert!(
        !aberta.is_closed(),
        "a fixtura tem de entrar ABERTA, senão este gate não vê o fenómeno"
    );
    let _ = &mut aberta;

    match corta(&aberta, &lamina(1.2, 0.6), Op::Subtrair) {
        Err(Recusa::PecaAberta) => {}
        Err(outra) => panic!("recusou pela razão errada: {outra:?}"),
        Ok(m) => panic!(
            "DEVOLVEU {} vértices sobre uma peça aberta — se for `0`, é a \
             armadilha do motor a chegar ao artista",
            m.vert_count()
        ),
    }

    // ⭐ O CONTROLO: fechada, a mesma lâmina corta.
    assert!(
        corta(&bola, &lamina(1.2, 0.6), Op::Subtrair).is_ok(),
        "a porta recusa SEMPRE — o gate acima não estaria a medir a abertura"
    );
}

/// **A lâmina aberta é nomeada À PARTE.**
///
/// ⚠️ *«Alguma coisa está aberta» manda o artista procurar nos dois sítios.* As
/// duas recusas existem porque as curas são diferentes: fechar a peça é um
/// remesh; uma lâmina aberta é um gesto degenerado, e a cura é repetir o gesto.
#[test]
fn a_lamina_aberta_e_nomeada_a_parte() {
    let bola = shapes::uv_sphere(24, 32, 1.0);
    let l = lamina(1.2, 0.6);
    let rota = Mesh::from_parts(
        l.positions().to_vec(),
        l.faces()[..l.face_count() - 1].to_vec(),
    )
    .expect("a lâmina com uma face a menos");

    assert_eq!(
        corta(&bola, &rota, Op::Subtrair).err(),
        Some(Recusa::LaminaAberta),
        "a recusa tem de nomear a LÂMINA — a peça está fechada"
    );
}

/// **Um corte que apagaria a peça inteira é RECUSA, não resultado.**
///
/// ⚠️ É a única recusa que é facto sobre a SAÍDA, e ela existe pelo mesmo
/// argumento do cabeçalho um nível acima: devolver `Ok` sobre o nada é o que
/// esta porta existe para impedir.
#[test]
fn um_corte_que_apagaria_tudo_e_recusado() {
    let bola = shapes::uv_sphere(24, 32, 1.0);
    assert_eq!(
        corta(&bola, &lamina(0.0, 4.0), Op::Subtrair).err(),
        Some(Recusa::ResultadoVazio),
        "uma lâmina que engole a peça tem de recusar"
    );
}

/// **Cada recusa diz o que FALTA, não que falhou.**
#[test]
fn cada_recusa_diz_a_cura() {
    for r in [
        Recusa::PecaAberta,
        Recusa::LaminaAberta,
        Recusa::ResultadoVazio,
    ] {
        let t = r.porque();
        assert!(t.len() > 40, "{r:?} tem frase curta demais: {t:?}");
        assert!(
            t.contains("--"),
            "{r:?} não diz a CURA (a frase tem duas metades: o facto e o que fazer): {t:?}"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SONDA (2026-09-15) — **a face que o corte deixa**, medida.
//
// Report do dono, com foto: *«o remesh da face que você cortou fica ruim
// demais»*. A foto mostra um LEQUE de triângulos finos a irradiar de um ponto.
// Esta sonda põe número nisso: ela mede a tampa do corte **contra a densidade da
// própria peça**, que é a régua que o olho usa.
// ─────────────────────────────────────────────────────────────────────────────

/// Aresta mediana da malha (a régua com que a tampa é comparada).
fn arestas(m: &Mesh) -> Vec<f32> {
    let mut tris = Vec::new();
    for f in m.faces() {
        f.triangles(&mut tris);
    }
    let p = m.positions();
    let d = |a: u32, b: u32| {
        let (a, b) = (p[a as usize], p[b as usize]);
        ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
    };
    let mut v: Vec<f32> = tris
        .iter()
        .flat_map(|t| [d(t[0], t[1]), d(t[1], t[2]), d(t[2], t[0])])
        .collect();
    v.sort_by(f32::total_cmp);
    v
}

fn pct(v: &[f32], q: f64) -> f32 {
    if v.is_empty() {
        return f32::NAN;
    }
    v[((v.len() - 1) as f64 * q) as usize]
}

/// Aspecto de um triângulo: aresta mais longa / (2 × raio inscrito).
/// `1` é equilátero; quanto maior, mais lasca.
fn aspecto(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let e = |u: [f32; 3], v: [f32; 3]| {
        ((u[0] - v[0]).powi(2) + (u[1] - v[1]).powi(2) + (u[2] - v[2]).powi(2)).sqrt()
    };
    let (l0, l1, l2) = (e(a, b), e(b, c), e(c, a));
    let s = (l0 + l1 + l2) * 0.5;
    let area2 = (s * (s - l0) * (s - l1) * (s - l2)).max(0.0).sqrt();
    if area2 <= 0.0 {
        return f32::INFINITY;
    }
    l0.max(l1).max(l2) * s / (2.0 * area2)
}

#[test]
#[ignore = "sonda de diagnóstico — imprime a tabela da tampa do corte"]
fn diag_a_tampa_do_corte() {
    for (nome, bola) in [
        ("uv_sphere(24,32)", shapes::uv_sphere(24, 32, 1.0)),
        ("uv_sphere(48,64)", shapes::uv_sphere(48, 64, 1.0)),
        ("tri≈20k", shapes::sphere_with_triangles(20_000, 1.0)),
    ] {
        let ent = arestas(&bola);
        let e50 = pct(&ent, 0.5);
        let out = corta(&bola, &lamina(1.0, 0.6), Op::Subtrair).expect("o corte");

        // A tampa: faces cujos três vértices estão no plano do corte (x = 0,4).
        let p = out.positions();
        let mut tris = Vec::new();
        for f in out.faces() {
            f.triangles(&mut tris);
        }
        let no_plano = |i: u32| (p[i as usize][0] - 0.4).abs() < 1e-4;
        let tampa: Vec<[u32; 3]> = tris
            .iter()
            .copied()
            .filter(|t| t.iter().all(|&i| no_plano(i)))
            .collect();
        let mut asp: Vec<f32> = tampa
            .iter()
            .map(|t| aspecto(p[t[0] as usize], p[t[1] as usize], p[t[2] as usize]))
            .collect();
        asp.sort_by(f32::total_cmp);
        let mut le: Vec<f32> = tampa
            .iter()
            .flat_map(|t| {
                let d = |a: u32, b: u32| {
                    let (a, b) = (p[a as usize], p[b as usize]);
                    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
                };
                [d(t[0], t[1]), d(t[1], t[2]), d(t[2], t[0])]
            })
            .collect();
        le.sort_by(f32::total_cmp);

        // Quantos triângulos a tampa TERIA à densidade da peça: área / área de
        // um equilátero de lado `e50`.
        let mut area = 0.0f32;
        for t in &tampa {
            let (a, b, c) = (p[t[0] as usize], p[t[1] as usize], p[t[2] as usize]);
            let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let n = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            area += 0.5 * (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        }
        let area_eq = 3.0f32.sqrt() / 4.0 * e50 * e50;

        println!(
            "\n── {nome}\n\
             peça  V={:6} T={:6}  aresta p50={e50:.4} p99={:.4}\n\
             saída V={:6} T={:6}\n\
             TAMPA T={:4}  área={area:.4}  T à densidade da peça ≈ {:.0}  ⇒ {:.2}× MENOS\n\
             tampa aresta  min={:.4}  p50={:.4}  max={:.4}   (peça p50={e50:.4})\n\
             tampa aspecto p50={:.2}  p90={:.2}  p99={:.2}  MAX={:.1}",
            bola.positions().len(),
            bola.faces().len(),
            pct(&ent, 0.99),
            out.positions().len(),
            out.faces().len(),
            tampa.len(),
            area / area_eq,
            (area / area_eq) / tampa.len() as f32,
            le.first().copied().unwrap_or(f32::NAN),
            pct(&le, 0.5),
            le.last().copied().unwrap_or(f32::NAN),
            pct(&asp, 0.5),
            pct(&asp, 0.9),
            pct(&asp, 0.99),
            asp.last().copied().unwrap_or(f32::NAN),
        );
    }
}

/// Uma lâmina TESSELADA: o mesmo cubo, com nenhuma aresta acima de `alvo`.
fn lamina_densa(cx: f32, meia: f32, alvo: f32) -> Mesh {
    let n = ((meia * 2.0 / alvo).ceil() as usize).max(1);
    let mut pos: Vec<[f32; 3]> = Vec::new();
    let mut faces: Vec<Face> = Vec::new();
    // seis caras, cada uma uma grelha (n+1)² — vértices duplicados nas arestas,
    // que é o que uma sonda pode fazer (o motor solda).
    let eixos: [([f32; 3], [f32; 3], [f32; 3]); 6] = [
        ([1., 0., 0.], [0., 1., 0.], [0., 0., 1.]),
        ([-1., 0., 0.], [0., 0., 1.], [0., 1., 0.]),
        ([0., 1., 0.], [0., 0., 1.], [1., 0., 0.]),
        ([0., -1., 0.], [1., 0., 0.], [0., 0., 1.]),
        ([0., 0., 1.], [1., 0., 0.], [0., 1., 0.]),
        ([0., 0., -1.], [0., 1., 0.], [1., 0., 0.]),
    ];
    for (nrm, u, v) in eixos {
        let base = pos.len() as u32;
        for iv in 0..=n {
            for iu in 0..=n {
                let (a, b) = (
                    (iu as f32 / n as f32) * 2.0 - 1.0,
                    (iv as f32 / n as f32) * 2.0 - 1.0,
                );
                pos.push([
                    cx + meia * (nrm[0] + u[0] * a + v[0] * b),
                    meia * (nrm[1] + u[1] * a + v[1] * b),
                    meia * (nrm[2] + u[2] * a + v[2] * b),
                ]);
            }
        }
        for iv in 0..n {
            for iu in 0..n {
                let i = base + (iv * (n + 1) + iu) as u32;
                let (r, s, t, w) = (i, i + 1, i + 1 + (n + 1) as u32, i + (n + 1) as u32);
                faces.push(Face::tri(r, s, t));
                faces.push(Face::tri(r, t, w));
            }
        }
    }
    // ⛔ SOLDA: seis grelhas com vértices duplicados entram no motor como seis
    // superfícies ABERTAS (medido: `LaminaAberta`). Uma lâmina tesselada tem de
    // partilhar os vértices das arestas — é o achado que o produto herda.
    // ⚠️ `BTreeMap` e não `HashMap`: a workspace proíbe o segundo por lint
    // estrutural (o determinismo é a espinha deste repo — `CLAUDE.md` §5.1).
    let mut mapa = std::collections::BTreeMap::new();
    let mut novos: Vec<[f32; 3]> = Vec::new();
    let mut remap = vec![0u32; pos.len()];
    for (i, q) in pos.iter().enumerate() {
        let k = [q[0].to_bits(), q[1].to_bits(), q[2].to_bits()];
        let e = *mapa.entry(k).or_insert_with(|| {
            novos.push(*q);
            (novos.len() - 1) as u32
        });
        remap[i] = e;
    }
    let faces: Vec<Face> = faces
        .iter()
        .map(|f| {
            let v = f.verts();
            Face::tri(
                remap[v[0] as usize],
                remap[v[1] as usize],
                remap[v[2] as usize],
            )
        })
        .collect();
    Mesh::from_parts(novos, faces).expect("a grelha")
}

#[test]
#[ignore = "sonda de diagnóstico — o PREÇO de uma lâmina tesselada"]
fn diag_o_preco_da_lamina_densa() {
    let alvo_peca: usize = std::env::var("PH2D_DIAG_TRIS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20_000);
    let bola = shapes::sphere_with_triangles(alvo_peca, 1.0);
    let e50 = pct(&arestas(&bola), 0.5);
    println!(
        "\npeça: V={} T={} aresta p50={e50:.4}",
        bola.positions().len(),
        bola.faces().len()
    );
    for mult in [f32::INFINITY, 8.0, 4.0, 2.0, 1.0, 0.5, 0.25, 0.125] {
        let lam = if mult.is_finite() {
            lamina_densa(1.0, 0.6, e50 * mult)
        } else {
            lamina(1.0, 0.6)
        };
        let t0 = std::time::Instant::now();
        let out = corta(&bola, &lam, Op::Subtrair).expect("o corte");
        let ms = t0.elapsed().as_secs_f64() * 1e3;

        let p = out.positions();
        let mut tris = Vec::new();
        for f in out.faces() {
            f.triangles(&mut tris);
        }
        let tampa: Vec<[u32; 3]> = tris
            .iter()
            .copied()
            .filter(|t| t.iter().all(|&i| (p[i as usize][0] - 0.4).abs() < 1e-4))
            .collect();
        let mut asp: Vec<f32> = tampa
            .iter()
            .map(|t| aspecto(p[t[0] as usize], p[t[1] as usize], p[t[2] as usize]))
            .collect();
        asp.sort_by(f32::total_cmp);
        println!(
            "lâmina aresta={:>8}  V={:6} T={:6}  →  saída V={:6} T={:6}  TAMPA T={:5}  \
             aspecto p50={:.2} p99={:.2}  bordo={}  {ms:7.1} ms",
            if mult.is_finite() {
                format!("{:.4}", e50 * mult)
            } else {
                "—".into()
            },
            lam.positions().len(),
            lam.faces().len(),
            out.positions().len(),
            out.faces().len(),
            tampa.len(),
            pct(&asp, 0.5),
            pct(&asp, 0.99),
            ph2d_mesh::border_edges(&out),
        );
    }
}
