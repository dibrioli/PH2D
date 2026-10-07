//! A13 (sonda) — o MECANISMO dos dentes da silhueta a cada densidade do bake: o braço da
//! dobra forte (o dos gates `o_braco_dobrado_de_volta_nao_deixa_dentes`), passo a passo pela
//! [`ph2d_vec_boolean::silhueta_da_pele`] (gancho → união → gancho → esporão → fecho → abertura →
//! fecho), com a viragem máxima depois de cada passo e as estatísticas da entrada.

use super::super::super::super::{
    AMOSTRAS_POR_FORMA, TOLERANCIA_DA_DIAGONAL, amostras_no_orcamento,
    diagonal as diagonal_do_bake, lida, os_nos_servem, quinas_do_artista, segmentos,
};
use super::braco_em;
use ph2d_vec_boolean::overlap::{PAREDE_MINIMA, RAIO_DO_VINCO, SOLDA_DA_QUINA};
use ph2d_vec_scene::{VecPath, VecVertex};
use ph2d_vec_skin::curva::{Bake, CampoIndexado};
use std::fmt::Write as _;

/// O bake do braço a `(primeira, segunda)` com `por_forma` amostras por forma, e as quinas do artista.
pub(super) fn entrada(
    primeira: f32,
    segunda: f32,
    por_forma: usize,
) -> (VecPath, Vec<([f64; 2], f64)>) {
    let (mut sim, _scene, map, id, ossos) =
        crate::barra_da_cena_tests_support::braco_da_dobra_forte(0.3);
    for (k, g) in [(1, primeira), (2, -segunda)] {
        sim.world_mut()
            .get_mut::<ph2d_ecs::Transform>(ossos[k])
            .expect("Transform")
            .rotation += g.to_radians();
    }
    assa_na(&sim, ph2d_ecs::Entity::from_bits(map[&id]), por_forma)
}

/// A [`entrada`] com a pele na lei do ângulo `lei` — a porta dos CONTROLOS que precisam do fenómeno
/// da média em círculo (o meio-ângulo do produto espalha a volta e o gancho e a cunha da união não
/// se formam).
pub(super) fn entrada_na_lei(
    primeira: f32,
    segunda: f32,
    por_forma: usize,
    lei: ph2d_skeleton::MisturaDoAngulo,
) -> (VecPath, Vec<([f64; 2], f64)>) {
    let (mut sim, _scene, map, id, ossos) =
        crate::barra_da_cena_tests_support::braco_da_dobra_forte(0.3);
    for (k, g) in [(1, primeira), (2, -segunda)] {
        sim.world_mut()
            .get_mut::<ph2d_ecs::Transform>(ossos[k])
            .expect("Transform")
            .rotation += g.to_radians();
    }
    assa_na_com(
        &sim,
        ph2d_ecs::Entity::from_bits(map[&id]),
        por_forma,
        Some(lei),
    )
}

/// O bake da forma presa `e` (a lei do produto, com `por_forma` amostras por forma) e as quinas.
pub(super) fn assa_na(
    sim: &ph2d_ecs::SimWorld,
    e: ph2d_ecs::Entity,
    por_forma: usize,
) -> (VecPath, Vec<([f64; 2], f64)>) {
    assa_na_com(sim, e, por_forma, None)
}

/// [`assa_na`] com a lei do ângulo trocada (`None` = a do produto).
fn assa_na_com(
    sim: &ph2d_ecs::SimWorld,
    e: ph2d_ecs::Entity,
    por_forma: usize,
    lei: Option<ph2d_skeleton::MisturaDoAngulo>,
) -> (VecPath, Vec<([f64; 2], f64)>) {
    let skin = sim
        .world()
        .get::<ph2d_skeleton_ecs::SkinBind>(e)
        .expect("bind")
        .clone();
    let pele = crate::skin_live::skin_of(sim, e).expect("pele");
    let pele = match lei {
        Some(l) => ph2d_skeleton::Skin::com_mistura(pele.bones().to_vec(), l).expect("pele"),
        None => pele,
    };
    let prep = lida(e.to_bits(), &skin).expect("fonte");
    let g = &prep.guardado;
    let pesos = skin.pesos_do_quadro(if g.valida() { &g.pesos } else { &[] });
    let (f, t) = if os_nos_servem(&g.path) {
        (g.path.clone(), pesos.to_vec())
    } else {
        let (c, t) = prep.cozido.as_ref().expect("cozido");
        (c.clone(), skin.pesos_do_quadro(t).to_vec())
    };
    let (d, nos) = ph2d_vec_skin::curva::assa_a_pele_com_nos(
        &pele,
        &f,
        &t,
        &skin.correcoes_resolvidas(),
        true,
        CampoIndexado {
            campo: g.campo.as_ref(),
            indice: prep.indice.as_ref(),
            suave: None,
        },
        Bake {
            amostras: amostras_no_orcamento(segmentos(&f), por_forma),
            tolerancia: TOLERANCIA_DA_DIAGONAL * diagonal_do_bake(&f),
        },
    );
    (d, quinas_do_artista(&f, nos))
}

/// A caixa JUSTA das cúbicas (os extremos pelas raízes da derivada), como a `bounding_box` da
/// `kurbo` que a [`ph2d_vec_boolean::silhueta_da_pele`] lê: `[x0, y0, x1, y1]`.
fn caixa(p: &VecPath) -> [f64; 4] {
    let v = &p.verts;
    let n = v.len();
    let mut c = [f64::MAX, f64::MAX, f64::MIN, f64::MIN];
    for k in 0..n {
        let (a, b) = (v[k], v[(k + 1) % n]);
        let q = [a.anchor, a.out_handle, b.in_handle, b.anchor];
        let em = |t: f64, j: usize| {
            let s = 1.0 - t;
            s * s * s * q[0][j]
                + 3.0 * s * s * t * q[1][j]
                + 3.0 * s * t * t * q[2][j]
                + t * t * t * q[3][j]
        };
        for j in 0..2 {
            let mut ts = vec![0.0, 1.0];
            // B'(t)/3 = A t² + B t + C.
            let (p0, p1, p2, p3) = (q[0][j], q[1][j], q[2][j], q[3][j]);
            let (aa, bb, cc) = (
                -p0 + 3.0 * p1 - 3.0 * p2 + p3,
                2.0 * (p0 - 2.0 * p1 + p2),
                p1 - p0,
            );
            if aa.abs() < 1e-14 {
                if bb.abs() > 1e-14 {
                    ts.push(-cc / bb);
                }
            } else {
                let disc = bb * bb - 4.0 * aa * cc;
                if disc >= 0.0 {
                    ts.push((-bb + disc.sqrt()) / (2.0 * aa));
                    ts.push((-bb - disc.sqrt()) / (2.0 * aa));
                }
            }
            for t in ts.into_iter().filter(|t| (0.0..=1.0).contains(t)) {
                let x = em(t, j);
                c[j] = c[j].min(x);
                c[j + 2] = c[j + 2].max(x);
            }
        }
    }
    c
}

pub(super) fn caixa_diag(p: &VecPath) -> f64 {
    let c = caixa(p);
    (c[2] - c[0]).hypot(c[3] - c[1])
}

/// O pior nó: `(viragem, índice, posição)`.
fn pior_no(v: &[VecVertex]) -> (f64, usize, [f64; 2]) {
    (0..v.len())
        .filter_map(|i| {
            ph2d_vec_boolean::overlap::viragem_do_vertice(v, i).map(|a| (a, i, v[i].anchor))
        })
        .fold((0.0, 0, [0.0; 2]), |m, x| if x.0 > m.0 { x } else { m })
}

/// A polilinha de um contorno fechado (16 por cúbica).
fn polilinha(v: &[VecVertex]) -> Vec<[f64; 2]> {
    let n = v.len();
    let mut out = Vec::new();
    for k in 0..n {
        let (a, b) = (v[k], v[(k + 1) % n]);
        let c = [a.anchor, a.out_handle, b.in_handle, b.anchor];
        for i in 0..16 {
            let t = f64::from(i) / 16.0;
            let s = 1.0 - t;
            out.push([0, 1].map(|j| {
                s * s * s * c[0][j]
                    + 3.0 * s * s * t * c[1][j]
                    + 3.0 * s * t * t * c[2][j]
                    + t * t * t * c[3][j]
            }));
        }
    }
    out.push(out[0]);
    out
}

/// Os auto-cruzamentos da polilinha (troços não vizinhos).
fn cruzamentos(pl: &[[f64; 2]]) -> usize {
    let n = pl.len() - 1;
    let mut c = 0;
    for i in 0..n {
        for j in i + 2..n {
            if i == 0 && j == n - 1 {
                continue;
            }
            let (a0, a1, b0, b1) = (pl[i], pl[i + 1], pl[j], pl[j + 1]);
            let (r, s) = (
                [a1[0] - a0[0], a1[1] - a0[1]],
                [b1[0] - b0[0], b1[1] - b0[1]],
            );
            let den = r[0] * s[1] - r[1] * s[0];
            if den.abs() < 1e-18 {
                continue;
            }
            let q = [b0[0] - a0[0], b0[1] - a0[1]];
            let (t, u) = (
                (q[0] * s[1] - q[1] * s[0]) / den,
                (q[0] * r[1] - q[1] * r[0]) / den,
            );
            if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
                c += 1;
            }
        }
    }
    c
}

/// As estatísticas de uma entrada: nós, troços curtos, pares quase coincidentes não vizinhos,
/// auto-cruzamentos, área com sinal.
fn estatisticas(v: &[VecVertex], solda: f64) -> String {
    let n = v.len();
    let d = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).hypot(a[1] - b[1]);
    let curtos = |tol: f64| {
        (0..n)
            .filter(|&i| d(v[i].anchor, v[(i + 1) % n].anchor) < tol)
            .count()
    };
    let mut coincidentes = 0;
    for i in 0..n {
        for j in i + 2..n {
            if (i == 0 && j == n - 1) || d(v[i].anchor, v[j].anchor) >= solda {
                continue;
            }
            coincidentes += 1;
        }
    }
    let pl = polilinha(v);
    let area: f64 = pl
        .windows(2)
        .map(|w| w[0][0] * w[1][1] - w[1][0] * w[0][1])
        .sum::<f64>()
        / 2.0;
    format!(
        "{n} nós · troços < solda {} · < 1e-6 {} · < 1e-9 {} · pares a < solda {coincidentes} · \
         auto-cruzamentos {} · área {area:.4}",
        curtos(solda),
        curtos(1e-6),
        curtos(1e-9),
        cruzamentos(&pl)
    )
}

fn svg(nome: &str, camadas: &[(&[VecVertex], &str, f64)], centro: [f64; 2], lado: f64) {
    let mut s = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='{} {} {lado} {lado}' width='800' height='800'>\
         <rect x='{}' y='{}' width='{lado}' height='{lado}' fill='white'/>",
        centro[0] - lado / 2.0,
        -centro[1] - lado / 2.0,
        centro[0] - lado / 2.0,
        -centro[1] - lado / 2.0
    );
    for (v, cor, w) in camadas {
        let pts: Vec<String> = polilinha(v)
            .iter()
            .map(|p| format!("{:.6},{:.6}", p[0], -p[1]))
            .collect();
        let _ = write!(
            s,
            "<polyline points='{}' fill='none' stroke='{cor}' stroke-width='{w}'/>",
            pts.join(" ")
        );
        for x in v.iter() {
            let _ = write!(
                s,
                "<circle cx='{:.6}' cy='{:.6}' r='{}' fill='{cor}'/>",
                x.anchor[0],
                -x.anchor[1],
                w * 1.5
            );
        }
    }
    s.push_str("</svg>");
    let dir = "/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/c6232662-5952-412a-8b96-880a517c6741/scratchpad/bug_dentes";
    std::fs::create_dir_all(dir).expect("pasta");
    std::fs::write(format!("{dir}/{nome}.svg"), s).expect("svg");
}

/// ⭐ **SONDA — os dentes, passo a passo.**
#[test]
#[ignore = "sonda: imprime e escreve SVG"]
fn diag_os_dentes_passo_a_passo() {
    let p = AMOSTRAS_POR_FORMA;
    let casos: [(&str, f32, f32, usize); 7] = [
        ("x4_36_118", 36.0, 118.0, 2 * p),
        ("x2_22_130", 22.0, 130.0, p),
        ("x2_24_130", 24.0, 130.0, p),
        ("x2_160_134", 160.0, 134.0, p),
        ("x2_176_92", 176.0, 92.0, p),
        ("x1_176_92", 176.0, 92.0, p / 2),
        ("x4_160_134", 160.0, 134.0, 2 * p),
    ];
    for (nome, p1, p2, por_forma) in casos {
        let (d, quinas) = entrada(p1, p2, por_forma);
        if por_forma == AMOSTRAS_POR_FORMA {
            let (sem, _) = braco_em(p1, p2);
            assert_eq!(sem.verts, d.verts, "a sonda refaz o bake do produto");
        }
        let diag = caixa_diag(&d);
        let (solda, raio) = (SOLDA_DA_QUINA * diag, RAIO_DO_VINCO * diag);
        println!(
            "== {nome} ({p1}, {p2}) · solda {solda:.5} raio {raio:.5} · quinas {}",
            quinas.len()
        );
        let s0 = d.verts.clone();
        let s1 = ph2d_vec_boolean::gancho::desfaz_os_ganchos(s0.clone(), &quinas, solda);
        let p1v = VecPath {
            verts: s1.clone(),
            ..d.clone()
        };
        let u = ph2d_vec_boolean::overlap::resolve_overlap(&p1v);
        let s2 = u.as_ref().map_or_else(|| s1.clone(), |u| u.verts.clone());
        let s3 = ph2d_vec_boolean::gancho::desfaz_os_ganchos(s2.clone(), &quinas, solda);
        let s4 = ph2d_vec_boolean::esporao::tira_os_esporoes(s3.clone(), &quinas, solda);
        let s5 = ph2d_vec_boolean::bola::rola_a_bola(s4.clone(), &quinas, raio, solda);
        let paredes: Vec<([f64; 2], f64)> = quinas.iter().map(|&(p, _)| (p, 180.0)).collect();
        let s6 = ph2d_vec_boolean::bola::rola_a_bola_por_dentro(s5.clone(), &paredes, raio, solda);
        let s7 = if s6 == s5 {
            s5.clone()
        } else {
            ph2d_vec_boolean::bola::rola_a_bola(s6.clone(), &quinas, raio, solda)
        };
        let passos: [(&str, &[VecVertex]); 8] = [
            ("0 assado", &s0),
            ("1 gancho", &s1),
            ("2 união", &s2),
            ("3 gancho", &s3),
            ("4 esporão", &s4),
            ("5 fecho", &s5),
            ("6 abertura", &s6),
            ("7 fecho", &s7),
        ];
        {
            let (_, i, _) = pior_no(&s7);
            let n = s7.len();
            for k in [(i + n - 1) % n, i, (i + 1) % n] {
                let v = s7[k];
                println!(
                    "   s7 nó {k}: âncora ({:.6},{:.6}) entra ({:+.2e},{:+.2e}) sai ({:+.2e},{:+.2e}) vira {:.1}°",
                    v.anchor[0],
                    v.anchor[1],
                    v.in_handle[0] - v.anchor[0],
                    v.in_handle[1] - v.anchor[1],
                    v.out_handle[0] - v.anchor[0],
                    v.out_handle[1] - v.anchor[1],
                    ph2d_vec_boolean::overlap::viragem_do_vertice(&s7, k).unwrap_or(0.0)
                );
            }
        }
        if let Ok(dir) = std::env::var("SONDA_DESPEJO") {
            // A entrada do fecho (depois do esporão), em literal Rust, para uma fixtura.
            let mut t = format!(
                "// {nome} solda {solda:e} raio {raio:e}\nconst QUINAS: &[([f64; 2], f64)] = &{quinas:?};\nconst VERTS: &[[[f64; 2]; 3]] = &[\n"
            );
            for v in &s4 {
                let _ = writeln!(
                    t,
                    "    [{:?}, {:?}, {:?}],",
                    v.anchor, v.in_handle, v.out_handle
                );
            }
            t.push_str("];\n");
            std::fs::write(format!("{dir}/{nome}.rs"), t).expect("despejo");
            for (rot, vv) in [("s2", &s2), ("s3", &s3)] {
                let mut t = String::new();
                for v in vv {
                    let _ = writeln!(
                        t,
                        "    [{:?}, {:?}, {:?}],",
                        v.anchor, v.in_handle, v.out_handle
                    );
                }
                std::fs::write(format!("{dir}/{nome}_{rot}.txt"), t).expect("despejo");
            }
        }
        let mut primeiro: Option<usize> = None;
        for (k, (rot, v)) in passos.iter().enumerate() {
            let (a, i, p) = pior_no(v);
            let qd = quinas
                .iter()
                .map(|(q, _)| (q[0] - p[0]).hypot(q[1] - p[1]))
                .fold(f64::MAX, f64::min);
            println!(
                "   {rot:<11} {:>4} nós · pior viragem {a:>6.1}° no nó {i} ({:.4},{:.4}) a {:.3} solda(s) da quina mais perto",
                v.len(),
                p[0],
                p[1],
                qd / solda
            );
            if a >= PAREDE_MINIMA && primeiro.is_none() && k > 0 {
                primeiro = Some(k);
            }
        }
        if let Some(k) = primeiro {
            let (entra, sai) = (passos[k - 1].1, passos[k].1);
            let (_, _, p) = pior_no(sai);
            println!(
                "   ⇒ o dente nasce no passo «{}» · a entrada: {}",
                passos[k].0,
                estatisticas(entra, solda)
            );
            // A entrada perto do dente: os nós a menos de 3 raios.
            let perto: Vec<String> = entra
                .iter()
                .enumerate()
                .filter(|(_, x)| (x.anchor[0] - p[0]).hypot(x.anchor[1] - p[1]) < 3.0 * raio)
                .map(|(i, x)| {
                    let n = entra.len();
                    let seg = (entra[(i + 1) % n].anchor[0] - x.anchor[0])
                        .hypot(entra[(i + 1) % n].anchor[1] - x.anchor[1]);
                    let vira =
                        ph2d_vec_boolean::overlap::viragem_do_vertice(entra, i).unwrap_or(0.0);
                    format!(
                        "#{i}({:.4},{:.4}) seg {:.2}s vira {vira:.1}°",
                        x.anchor[0],
                        x.anchor[1],
                        seg / solda
                    )
                })
                .collect();
            println!("   entrada a < 3 raios do dente: {}", perto.join(" · "));
            // A cadeia inteira junto do dente: a entrada do fecho (azul), o fecho (vermelho) e o
            // fim (verde).
            let (_, _, pf) = pior_no(&s7);
            svg(
                &format!("{nome}_fecho"),
                &[
                    (&s4, "#1e66ff", raio * 0.05),
                    (&s5, "#d00000", raio * 0.035),
                    (&s7, "#2a9d2a", raio * 0.02),
                ],
                pf,
                12.0 * raio,
            );
            let lado = |v: &[VecVertex]| -> String {
                let (a, i, _) = pior_no(v);
                let n = v.len();
                let ar: f64 = (0..n)
                    .map(|k| {
                        v[k].anchor[0] * v[(k + 1) % n].anchor[1]
                            - v[(k + 1) % n].anchor[0] * v[k].anchor[1]
                    })
                    .sum();
                let (p0, p1, p2) = (
                    v[(i + n - 1) % n].anchor,
                    v[i].anchor,
                    v[(i + 1) % n].anchor,
                );
                let cr = (p1[0] - p0[0]) * (p2[1] - p1[1]) - (p1[1] - p0[1]) * (p2[0] - p1[0]);
                let d = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).hypot(a[1] - b[1]) / solda;
                format!(
                    "{a:.1}° no nó {i} · {} · troços vizinhos {:.1}/{:.1} soldas",
                    if cr * ar > 0.0 { "CONVEXO" } else { "côncavo" },
                    d(p0, p1),
                    d(p1, p2)
                )
            };
            println!(
                "   o pior nó: entrada do fecho {} · depois do fecho {} · no fim {}",
                lado(&s4),
                lado(&s5),
                lado(&s7)
            );
            svg(
                &format!("{nome}_passo{k}"),
                &[
                    (entra, "#1e66ff", raio * 0.05),
                    (sai, "#d00000", raio * 0.03),
                ],
                p,
                12.0 * raio,
            );
        }
        svg(
            &format!("{nome}_inteiro"),
            &[
                (&s0, "#1e66ff", diag * 0.0015),
                (&s7, "#d00000", diag * 0.001),
            ],
            {
                let c = caixa(&d);
                [0.5 * (c[0] + c[2]), 0.5 * (c[1] + c[3])]
            },
            diag,
        );
    }
}

/// ⭐ **SONDA — a identidade fora do contacto** (`numa_dobra_forte_o_desenho_nao_se_cruza`): a `45°`
/// e `60°` em C, o que a silhueta muda no desenho sem contacto (com a amostragem do PRODUTO).
#[test]
#[ignore = "sonda: imprime e escreve SVG"]
fn diag_a_identidade_fora_do_contacto() {
    for g in [45.0_f32, 60.0] {
        let (sem, com) = super::super::super::com_e_sem_contacto(g);
        let diag = caixa_diag(&sem);
        let (solda, raio) = (SOLDA_DA_QUINA * diag, RAIO_DO_VINCO * diag);
        let cruza = ph2d_vec_boolean::overlap::resolve_overlap(&sem).is_some();
        // O menor raio de curvatura de TODO o assado (dentro dos segmentos).
        let raio_min = |v: &[VecVertex]| {
            let nn = v.len();
            let mut menor = (f64::MAX, 0, 0.0);
            for k in 0..nn {
                let (a, b) = (v[k], v[(k + 1) % nn]);
                let q = [a.anchor, a.out_handle, b.in_handle, b.anchor];
                for i in 1..200 {
                    let t = f64::from(i) / 200.0;
                    let s1 = 1.0 - t;
                    let d1 = [0, 1].map(|j| {
                        3.0 * (s1 * s1 * (q[1][j] - q[0][j])
                            + 2.0 * s1 * t * (q[2][j] - q[1][j])
                            + t * t * (q[3][j] - q[2][j]))
                    });
                    let d2 = [0, 1].map(|j| {
                        6.0 * (s1 * (q[2][j] - 2.0 * q[1][j] + q[0][j])
                            + t * (q[3][j] - 2.0 * q[2][j] + q[1][j]))
                    });
                    let cr = (d1[0] * d2[1] - d1[1] * d2[0]).abs();
                    let v3 = d1[0].hypot(d1[1]).powi(3);
                    if cr > 1e-300 && v3 / cr < menor.0 {
                        menor = (v3 / cr, k, t);
                    }
                }
            }
            menor
        };
        // As corridas que a BOLA toca: arestas côncavas (lado de fora) entre amostras (32 por
        // segmento, como ela) mais apertadas que `0,99 r`, com a viragem somada.
        {
            let v = &sem.verts;
            let nn = v.len();
            let mut am: Vec<([f64; 2], [f64; 2], usize, f64)> = Vec::new();
            for k in 0..nn {
                let (a, b) = (v[k], v[(k + 1) % nn]);
                let q = [a.anchor, a.out_handle, b.in_handle, b.anchor];
                for i in 0..=32 {
                    let t = f64::from(i) / 32.0;
                    let s1 = 1.0 - t;
                    let p = [0, 1].map(|j| {
                        s1 * s1 * s1 * q[0][j]
                            + 3.0 * s1 * s1 * t * q[1][j]
                            + 3.0 * s1 * t * t * q[2][j]
                            + t * t * t * q[3][j]
                    });
                    let d = [0, 1].map(|j| {
                        3.0 * (s1 * s1 * (q[1][j] - q[0][j])
                            + 2.0 * s1 * t * (q[2][j] - q[1][j])
                            + t * t * (q[3][j] - q[2][j]))
                    });
                    let l = d[0].hypot(d[1]).max(1e-300);
                    am.push((p, [d[0] / l, d[1] / l], k, t));
                }
            }
            let m = am.len();
            let area2: f64 = (0..m)
                .map(|i| am[i].0[0] * am[(i + 1) % m].0[1] - am[(i + 1) % m].0[0] * am[i].0[1])
                .sum();
            let sinal = area2.signum();
            let mut corrida: Vec<(usize, f64, f64)> = Vec::new();
            let fecha_corrida = |c: &mut Vec<(usize, f64, f64)>| {
                if !c.is_empty() {
                    let giro: f64 = c.iter().map(|x| x.1).sum();
                    if 2.0 * raio * (0.5 * giro.min(3.1)).tan() >= solda {
                        let (e0, e1) = (c[0].0, c[c.len() - 1].0);
                        println!(
                            "     corrida da bola: {} arestas · viragem {:.1}° · de seg {} t={:.3} a seg {} t={:.3} · ({:.4},{:.4}) · menor raio da corda {:.4} ({:.2} r)",
                            c.len(),
                            giro.to_degrees(),
                            am[e0].2,
                            am[e0].3,
                            am[e1].2,
                            am[e1].3,
                            am[e0].0[0],
                            am[e0].0[1],
                            c.iter().map(|x| x.2).fold(f64::MAX, f64::min),
                            c.iter().map(|x| x.2).fold(f64::MAX, f64::min) / raio
                        );
                    }
                    c.clear();
                }
            };
            for e in 0..m {
                let (a, b) = (am[e], am[(e + 1) % m]);
                let dth =
                    (a.1[0] * b.1[1] - a.1[1] * b.1[0]).atan2(a.1[0] * b.1[0] + a.1[1] * b.1[1]);
                let corda = (b.0[0] - a.0[0]).hypot(b.0[1] - a.0[1]);
                if dth * sinal < -1e-9 && corda < 0.99 * raio * dth.abs() {
                    corrida.push((e, dth.abs(), corda / dth.abs().max(1e-300)));
                } else {
                    fecha_corrida(&mut corrida);
                }
            }
            fecha_corrida(&mut corrida);
        }
        let rm = raio_min(&sem.verts);
        println!(
            "     o menor raio de curvatura de todo o assado {:.4} ({:.2} r) no segmento {} t={:.3}",
            rm.0,
            rm.0 / raio,
            rm.1,
            rm.2
        );
        let (a, i, p) = pior_no(&com.verts);
        println!(
            "  {g}°: igual {} · cruza {cruza} · nós {} → {} · pior viragem com {a:.1}° no nó {i} ({:.4},{:.4}) · sem {:.1}° · solda {solda:.5} raio {raio:.5}",
            sem.verts == com.verts,
            sem.verts.len(),
            com.verts.len(),
            p[0],
            p[1],
            pior_no(&sem.verts).0
        );
        if sem.verts != com.verts {
            // Onde mudou: o primeiro nó de `com` que não está em `sem`.
            let novo = com
                .verts
                .iter()
                .find(|v| !sem.verts.iter().any(|w| w.anchor == v.anchor))
                .map(|v| v.anchor)
                .unwrap_or(p);
            svg(
                &format!("identidade_{g}"),
                &[
                    (&sem.verts, "#1e66ff", raio * 0.05),
                    (&com.verts, "#d00000", raio * 0.03),
                ],
                novo,
                12.0 * raio,
            );
            // O menor raio de curvatura do assado a menos de 3 raios da mudança, e onde (que nó, t).
            let v = &sem.verts;
            let nn = v.len();
            let mut menor = (f64::MAX, 0, 0.0);
            for k in 0..nn {
                let (a, b) = (v[k], v[(k + 1) % nn]);
                let q = [a.anchor, a.out_handle, b.in_handle, b.anchor];
                for i in 0..=200 {
                    let t = f64::from(i) / 200.0;
                    let s1 = 1.0 - t;
                    let pt = [0, 1].map(|j| {
                        s1 * s1 * s1 * q[0][j]
                            + 3.0 * s1 * s1 * t * q[1][j]
                            + 3.0 * s1 * t * t * q[2][j]
                            + t * t * t * q[3][j]
                    });
                    if (pt[0] - novo[0]).hypot(pt[1] - novo[1]) > 3.0 * raio {
                        continue;
                    }
                    let d1 = [0, 1].map(|j| {
                        3.0 * (s1 * s1 * (q[1][j] - q[0][j])
                            + 2.0 * s1 * t * (q[2][j] - q[1][j])
                            + t * t * (q[3][j] - q[2][j]))
                    });
                    let d2 = [0, 1].map(|j| {
                        6.0 * (s1 * (q[2][j] - 2.0 * q[1][j] + q[0][j])
                            + t * (q[3][j] - 2.0 * q[2][j] + q[1][j]))
                    });
                    let cr = (d1[0] * d2[1] - d1[1] * d2[0]).abs();
                    let v3 = d1[0].hypot(d1[1]).powi(3);
                    if cr > 1e-300 && v3 / cr < menor.0 {
                        menor = (v3 / cr, k, t);
                    }
                }
            }
            println!(
                "     a mudança perto de ({:.4},{:.4}) · o menor raio de curvatura do assado ali {:.4} ({:.2} r) no segmento {} t={:.3}",
                novo[0],
                novo[1],
                menor.0,
                menor.0 / raio,
                menor.1,
                menor.2
            );
        }
    }
}
