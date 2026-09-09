//! ⭐⭐⭐ **O REPORT DE 08/09 DO DONO, EM NÚMEROS** — *«o Cloth não age como pano
//! real, mas como um elástico que estica indefinidamente»* + *«deve haver algum
//! grau de elasticidade mas deve haver a possibilidade de manter volume»*.
//!
//! A cena dele: uma esfera com **três pontos mascarados** e o filtro de
//! **gravidade** arrastado até ao fim. O que ela produzia era uma esfera puxada
//! em três tubos compridos, e uma superfície que a luz mostrava rasgada.
//!
//! # ⚠️ Porque as duas queixas dele são UM defeito
//!
//! Uma restrição de distância com rigidez `0,6` é uma **mola**: sob carga
//! sustentada ela assenta num equilíbrio ESTICADO que cresce com a carga, sem
//! tecto. Medido aqui: `18,6×` o comprimento de repouso na pior aresta, o volume
//! da peça em `44 %` do que era, e ~`250` pares de faces com mais de `60°` entre
//! as normais — que é exactamente o «render danificado» da foto, porque um
//! matcap amostra a direcção da face.
//!
//! # A cura, e o que cada metade compra
//!
//! | | esticão máx | vincos `>60°` | volume |
//! |---|---:|---:|---:|
//! | como estava | `18,58` | `247` | `44 %` |
//! | só o tecto (`1,10`) | `2,82` | `307` | `39 %` |
//! | só o volume | `18,36` | `166` | `99 %` |
//! | **os dois** | **`3,13`** | **`195`** | **`98 %`** |

use ph2d_mesh::Mesh;
use ph2d_sculpt3d::{ClothFilterKind, ClothFilterProps, ClothFilterStep, SculptStroke};

fn esfera() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(32, 64, 1.0)
}

/// Os três pontos que o dono mascarou, perto do topo.
fn mascarar(m: &mut Mesh) {
    let alvos = [[0.0f32, 0.9, 0.45], [-0.75, 0.6, 0.3], [0.75, 0.6, 0.3]];
    let pos: Vec<[f32; 3]> = m.positions().to_vec();
    let mk = m.masks_mut();
    for (i, p) in pos.iter().enumerate() {
        for a in &alvos {
            let d = ((p[0] - a[0]).powi(2) + (p[1] - a[1]).powi(2) + (p[2] - a[2]).powi(2)).sqrt();
            if d < 0.28 {
                mk[i] = 1.0;
            }
        }
    }
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// A maior razão `|aresta| / |aresta de repouso|` da malha.
fn estica(rest: &Mesh, now: &Mesh) -> f64 {
    let (a, b) = (rest.positions(), now.positions());
    let mut mx: f64 = 0.0;
    for f in rest.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (i, j) = (v[k] as usize, v[(k + 1) % v.len()] as usize);
            let l0 = dist(a[i], a[j]);
            if l0 > 1e-9 {
                mx = mx.max(f64::from(dist(b[i], b[j]) / l0));
            }
        }
    }
    mx
}

/// **O QUE A LUZ MOSTRA** — quantos pares de faces vizinhas têm mais de `60°`
/// entre as normais. ⚠️ Ruído de alta frequência sobe isto; uma deformação lisa
/// não — e é por isso que ele é a régua do «render danificado» e o esticão não é.
fn vincos(m: &Mesh) -> usize {
    let n = m.face_normals();
    let mut c = 0;
    for (fi, f) in m.faces().iter().enumerate() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            for &gj in m.adjacency().vert_faces.neighbours(a as usize) {
                if gj as usize <= fi || !m.faces()[gj as usize].verts().contains(&b) {
                    continue;
                }
                let (x, y) = (n[fi], n[gj as usize]);
                let d = f64::from(x[0] * y[0] + x[1] * y[1] + x[2] * y[2]);
                if d.clamp(-1.0, 1.0).acos().to_degrees() > 60.0 {
                    c += 1;
                }
            }
        }
    }
    c
}

/// O volume com sinal da peça.
fn volume(m: &Mesh) -> f64 {
    let mut t = Vec::new();
    for f in m.faces() {
        let v = f.verts();
        for k in 1..v.len() - 1 {
            t.push([v[0], v[k], v[k + 1]]);
        }
    }
    let x: Vec<[f64; 3]> = m
        .positions()
        .iter()
        .map(|p| [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])])
        .collect();
    ph2d_cloth::verlet::volume_de(&x, &t)
}

/// **O GESTO DO REPORT** — `n` arrastos completos, cada um com `s` a crescer de
/// zero até `1,0`, que é o que `FILTER_DRAG_PER_PX = 0,001` dá em mil pixels.
fn correr(props: ClothFilterProps, kind: ClothFilterKind, gestos: usize, mask: bool) -> Mesh {
    let mut m = esfera();
    if mask {
        mascarar(&mut m);
    }
    for _ in 0..gestos {
        let mut st = SculptStroke::default();
        st.cloth_filter_begin(&m, props, kind, [0.0, 0.9, 0.45]);
        for k in 0..120 {
            let passo = ClothFilterStep {
                s: (k as f32 + 1.0) / 120.0,
                gravity_axis: [0.0, -1.0, 0.0],
                frame: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                axes: [true, true, true],
                eye: [0.0, 0.0, 1.0],
            };
            st.cloth_filter_step(&mut m, kind, &passo);
        }
        st.cloth_filter_end();
    }
    m
}

/// **O extremo ELÁSTICO da faixa** — o mais parecido com a lei de antes que o
/// painel oferece.
///
/// ⚠️⚠️ **Ele NÃO é a lei do alvo, e a diferença está medida:** com `∞` a mesma
/// corrida dá esticão máximo `9,83`; com `2,00`, `8,25`. A lei do alvo continua
/// alcançável — pelo [`ph2d_cloth::verlet::Solver`], que é onde as `103` fixtures
/// do oráculo a correm —, e **não** pelo painel, de propósito: ela É o defeito
/// que o dono reportou.
fn frouxo() -> ClothFilterProps {
    ClothFilterProps {
        stretch_max: ClothFilterProps::STRETCH.1,
        ..ClothFilterProps::default()
    }
}

/// ⭐⭐⭐ **O DEFEITO DO REPORT, e a cura.**
#[test]
fn o_tecto_de_esticao_corta_o_elastico_do_report() {
    let rest = esfera();
    let solto = correr(frouxo(), ClothFilterKind::Gravity, 4, true);
    let preso = correr(
        ClothFilterProps::default(),
        ClothFilterKind::Gravity,
        4,
        true,
    );
    let (a, b) = (estica(&rest, &solto), estica(&rest, &preso));
    println!("esticao maximo: frouxo (2,00) {a:.3} | omissao (1,10) {b:.3}");
    // ⚠️ A barra é a do REGIME, não um número afinado: sem tecto o pano passa de
    // dez vezes o repouso (é o «estica indefinidamente»), com tecto fica na casa
    // de um dígito.
    // ⚠️ **As barras são desta MALHA** (`32×64`), e o número muda com ela: na
    // `48×96` a mesma corrida lê `18,58` contra `3,13`. *Uma barra copiada de
    // outra densidade mede outra coisa.*
    assert!(a > 8.0, "a fixtura tem de produzir o defeito: {a:.3}");
    assert!(b < 3.0, "o tecto tem de o cortar: {b:.3}");
    assert!(a / b > 3.0, "razao {:.2}", a / b);
}

/// ⛔⛔ **SEM ÂNCORA A GRAVIDADE NÃO ESTICA NADA — e é por isso que a fixtura tem
/// máscara.** Uma peça inteira em queda livre é uma TRANSLAÇÃO RÍGIDA: nenhuma
/// distância entre vértices muda, e um gate sobre ela aprovaria qualquer lei.
#[test]
fn sem_ancora_a_gravidade_e_uma_translacao_rigida() {
    let rest = esfera();
    let livre = correr(frouxo(), ClothFilterKind::Gravity, 1, false);
    let e = estica(&rest, &livre);
    println!("sem mascara: esticao {e:.6}");
    assert!((e - 1.0).abs() < 1e-3, "{e:.6}");
}

/// ⭐⭐ **A CONSERVAÇÃO DE VOLUME** — o dono pediu *«a possibilidade de manter
/// volume»*, e sem ela a esfera esvazia-se.
#[test]
fn o_volume_conserva_se_e_sem_ele_a_peca_esvazia() {
    let v0 = volume(&esfera());
    let sem = volume(&correr(
        ClothFilterProps::default(),
        ClothFilterKind::Gravity,
        4,
        true,
    ));
    let com = volume(&correr(
        ClothFilterProps {
            volume: 1.0,
            ..ClothFilterProps::default()
        },
        ClothFilterKind::Gravity,
        4,
        true,
    ));
    println!(
        "volume: repouso {v0:.4} | sem {:.1}% | com {:.1}%",
        100.0 * sem / v0,
        100.0 * com / v0
    );
    assert!(
        sem / v0 < 0.75,
        "a fixtura tem de esvaziar: {:.3}",
        sem / v0
    );
    assert!(
        com / v0 > 0.90,
        "e a opcao tem de a encher: {:.3}",
        com / v0
    );
}

/// ⭐⭐⭐ **O APERTO É QUEM MAIS GANHA COM O VOLUME** — e este era um **aberto
/// nomeado** do módulo (*«a decisão do dono sobre o aperto com força alta»*, com
/// a opção (b) construída, medida e REFUTADA).
///
/// ⚠️ Sem volume, o aperto implode a peça a `4 %` do que ela era e deixa
/// ~`2 800` vincos — ele não estica, **colapsa**. É a terceira saída que faltava.
#[test]
fn o_aperto_deixa_de_implodir_quando_o_volume_esta_ligado() {
    let v0 = volume(&esfera());
    let sem = correr(ClothFilterProps::default(), ClothFilterKind::Pinch, 1, true);
    let com = correr(
        ClothFilterProps {
            volume: 1.0,
            ..ClothFilterProps::default()
        },
        ClothFilterKind::Pinch,
        1,
        true,
    );
    let (a, b) = (vincos(&sem), vincos(&com));
    println!(
        "aperto: vincos {a} -> {b} | volume {:.1}% -> {:.1}%",
        100.0 * volume(&sem) / v0,
        100.0 * volume(&com) / v0
    );
    assert!(a > 500, "a fixtura tem de produzir o colapso: {a}");
    assert!(b * 4 < a, "o volume tem de o curar: {a} -> {b}");
    assert!(volume(&com) / v0 > 0.85, "{:.3}", volume(&com) / v0);
}

/// ⛔⛔ **DECLARADO, e com gate para que ninguém o «cure»: o volume CANCELA a
/// Escala e o Inflate.** Os dois gestos existem para mudar o volume da peça; uma
/// restrição que o conserva anula-os por construção — medido, a Escala passa de
/// `1,20×` o volume de repouso para `1,01×` nesta malha (`1,45 → 1,01` na `48×96`).
///
/// ⇒ *é por isso que a conservação de volume nasce DESLIGADA*, e não porque seja
/// cara.
#[test]
fn o_volume_cancela_a_escala_por_construcao() {
    let v0 = volume(&esfera());
    let sem = volume(&correr(
        ClothFilterProps::default(),
        ClothFilterKind::Scale,
        1,
        true,
    )) / v0;
    let com = volume(&correr(
        ClothFilterProps {
            volume: 1.0,
            ..ClothFilterProps::default()
        },
        ClothFilterKind::Scale,
        1,
        true,
    )) / v0;
    println!("escala: volume {sem:.3}x sem conservacao, {com:.3}x com");
    assert!(sem > 1.15, "a escala tem de inchar: {sem:.3}");
    assert!(
        (com - 1.0).abs() < 0.10,
        "e a conservacao tem de a anular: {com:.3}"
    );
}

/// ⚠️ **O topo da faixa É o desligado**, e é a única forma de o artista alcançar
/// a lei do alvo sem um segundo controlo.
#[test]
fn a_porta_prende_o_tecto_na_faixa_e_o_volume_nasce_desligado() {
    // ⚠️ O `∞` da lei é preso pela porta: o painel não tem como o pedir.
    let s = frouxo().clamped();
    assert!(
        (s.stretch_max - ClothFilterProps::STRETCH.1).abs() < 1e-6,
        "{}",
        s.stretch_max
    );
    let d = ClothFilterProps::default().solver();
    assert!(
        d.estica_max.is_finite() && d.estica_max > 1.0,
        "{}",
        d.estica_max
    );
    assert_eq!(d.volume, 0.0, "o volume nasce desligado");
}
