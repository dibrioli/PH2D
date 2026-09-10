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

// ── ⭐⭐⭐ O REPORT DE 2026-09-09 ────────────────────────────────────────────────

/// A ÁREA da superfície — a régua de *«o pano cresceu?»*. ⚠️ O esticão máximo não
/// serve: ele é um extremo e não diz se a peça inteira ganhou material.
fn area(m: &Mesh) -> f64 {
    let p = m.positions();
    let mut s = 0.0;
    for f in m.faces() {
        let v = f.verts();
        for k in 1..v.len() - 1 {
            let (a, b, c) = (p[v[0] as usize], p[v[k] as usize], p[v[k + 1] as usize]);
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
            let x = [
                u[1] * w[2] - u[2] * w[1],
                u[2] * w[0] - u[0] * w[2],
                u[0] * w[1] - u[1] * w[0],
            ];
            s += 0.5 * (x[0] * x[0] + x[1] * x[1] + x[2] * x[2]).sqrt();
        }
    }
    s
}

/// **UM GESTO sobre um `SculptStroke` que o chamador possui** — e é essa posse
/// que a fixtura precisa de ter.
///
/// ⛔⛔ **A 1.ª redacção criava um `SculptStroke` por gesto e NÃO reproduzia o
/// defeito:** a cena tem UM (`self.stroke`), e é nele que o material persiste.
/// *Uma fixtura que constrói um objecto novo por gesto mede um programa em que
/// nada pode atravessar gestos.*
fn gesto_em(st: &mut SculptStroke, m: &mut Mesh, props: ClothFilterProps) {
    st.cloth_filter_begin(m, props, ClothFilterKind::Gravity, [0.0, 0.9, 0.45]);
    for k in 0..120 {
        let p = ClothFilterStep {
            s: (k as f32 + 1.0) / 120.0,
            gravity_axis: [0.0, -1.0, 0.0],
            frame: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            axes: [true, true, true],
            eye: [0.0, 0.0, 1.0],
        };
        st.cloth_filter_step(m, ClothFilterKind::Gravity, &p);
    }
    st.cloth_filter_end();
}

/// ⭐⭐⭐ **O MATERIAL ATRAVESSA OS GESTOS** — *«se eu fizer mais de uma simulação
/// … o objeto continua esticando»*.
#[test]
fn o_pano_nao_cresce_a_cada_gesto() {
    let rest = esfera();
    let a0 = area(&rest);
    let mut m = esfera();
    mascarar(&mut m);
    let mut st = SculptStroke::default();
    let props = ClothFilterProps {
        volume: 1.0,
        ..ClothFilterProps::default()
    };
    let mut areas = Vec::new();
    for _ in 0..3 {
        gesto_em(&mut st, &mut m, props);
        areas.push(area(&m) / a0);
    }
    println!(
        "area por gesto: {:.4} · {:.4} · {:.4}",
        areas[0], areas[1], areas[2]
    );
    // ⚠️ **A barra é o CRESCIMENTO entre gestos, não o valor absoluto** — o
    // primeiro gesto estica de facto (é o que uma simulação faz), e o defeito era
    // ele voltar a esticar o mesmo tanto a cada repetição. Sem a cura a série era
    // `1,13 → 1,23 → 1,31`.
    assert!(
        areas[2] / areas[0] < 1.05,
        "o pano cresceu {:.1}% entre o 1.o e o 3.o gesto",
        100.0 * (areas[2] / areas[0] - 1.0)
    );
}

/// ⭐⭐ **E ESCULPIR RE-SEMEIA O MATERIAL** — a outra metade, e sem ela a cura
/// seria pior que o defeito: o pano lutaria contra a forma que o artista acabou
/// de esculpir.
///
/// ⛔⛔ **A 1.ª redacção deste gate era VÁCUA e a mutação disse-o.** Ela mexia um
/// vértice e comparava «com a edição no meio» contra «sem a edição» — que diferem
/// **porque a malha difere**, tenha o material sido re-semeado ou não. *Um gate
/// que compara duas árvores diferentes mede a diferença delas, não a lei.*
///
/// ⇒ a fixtura é agora uma edição cuja resposta é **oposta** nos dois mundos:
/// **ampliar a peça `1,2×`**. Com re-semeadura o material é a peça grande e o
/// segundo gesto drapeja-a normalmente; sem ela o material continua a ser a peça
/// PEQUENA, cada aresta nasce a `1,2` de um tecto de `1,10`, e o limitador
/// encolhe a peça de volta — a área cai em vez de subir.
#[test]
fn uma_edicao_alheia_re_semeia_o_material() {
    let mut m = esfera();
    mascarar(&mut m);
    let mut st = SculptStroke::default();
    let props = ClothFilterProps::default();
    gesto_em(&mut st, &mut m, props);
    // Alguém esculpiu: a peça inteira ficou `1,2×` maior.
    for p in m.positions_mut() {
        for c in p.iter_mut() {
            *c *= 1.2;
        }
    }
    let ampliada = area(&m);
    gesto_em(&mut st, &mut m, props);
    let depois = area(&m);
    println!(
        "area: ampliada {ampliada:.4} -> depois do 2.o gesto {depois:.4} ({:+.1}%)",
        100.0 * (depois / ampliada - 1.0)
    );
    // Com o material re-semeado o 2.º gesto ESTICA, como qualquer gesto de
    // gravidade; com o material velho ele ENCOLHE a peça para dentro do tecto.
    assert!(
        depois > ampliada,
        "o material nao foi re-semeado: o pano encolheu a peca recem-esculpida \
         ({ampliada:.4} -> {depois:.4})"
    );
}

/// ⭐⭐⭐ **A LEI DE DOBRA RESISTE A MUDAR A CURVATURA** — *«nunca consigo … pano
/// duro ou couro»*.
#[test]
fn a_rigidez_de_dobra_segura_a_curvatura() {
    let rest = esfera();
    let mut tri = Vec::new();
    for f in rest.faces() {
        let v = f.verts();
        for k in 1..v.len() - 1 {
            tri.push([v[0], v[k], v[k + 1]]);
        }
    }
    let topo = ph2d_cloth::ClothTopology::build(&tri, rest.vert_count());
    let em = |m: &Mesh| -> Vec<[f64; 3]> {
        m.positions()
            .iter()
            .map(|p| [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])])
            .collect()
    };
    let xr = em(&rest);
    let desvio = |m: &Mesh| {
        let xn = em(m);
        let mut d: Vec<f64> = topo
            .dobradicas()
            .iter()
            .map(|h| {
                (ph2d_cloth::dihedral_de(&xn, *h) - ph2d_cloth::dihedral_de(&xr, *h))
                    .abs()
                    .to_degrees()
            })
            .collect();
        d.sort_by(|a, b| a.partial_cmp(b).unwrap());
        d[(d.len() - 1) * 99 / 100]
    };
    let corre = |bend: f32| {
        let mut m = esfera();
        mascarar(&mut m);
        let mut st = SculptStroke::default();
        gesto_em(
            &mut st,
            &mut m,
            ClothFilterProps {
                bend,
                ..ClothFilterProps::default()
            },
        );
        desvio(&m)
    };
    let (solto, duro) = (corre(0.0), corre(1.0));
    println!("desvio de dobra p99: solto {solto:.2} graus | duro {duro:.2} graus");
    assert!(solto > 20.0, "a fixtura tem de dobrar: {solto:.2}");
    assert!(
        duro < solto * 0.85,
        "a rigidez nao segurou a curvatura: {solto:.2} -> {duro:.2}"
    );
}

/// ⛔⛔⛔ **O TAMANHO DA PREGA É DA MALHA, e este gate existe para ninguém tentar
/// afiná-lo com um botão.**
///
/// Medido numa cortina franzida, com a rigidez de dobra no máximo e no mínimo:
///
/// | vértices por lado | aresta | onda (dobra `0`) | onda (dobra `1`) |
/// |---:|---:|---:|---:|
/// | `20` | `0,100` | `0,667` | `0,667` |
/// | `40` | `0,050` | `0,333` | `0,400` |
/// | `80` | `0,025` | `0,250` | `0,333` |
///
/// ⇒ *a onda acompanha a ARESTA* (`~7`–`10` arestas), e a RIGIDEZ move-a `+33 %`.
///
/// ⚠️⚠️ **E a frase que estava aqui — «`~7`–`10` arestas faça o artista o que
/// fizer» — foi REFUTADA em 2026-09-09, na malha FINA.** Esta tabela varreu a
/// rigidez (`k`) com o número de PASSAGENS preso em `1`, e as duas não são a
/// mesma grandeza: `k` diz com que força a restrição de ângulo empurra **ali**,
/// e a passagem diz até onde a resistência VIAJA (uma projecção de Jacobi
/// propaga **uma dobradiça por passagem**). Medido pela
/// [`sonda_da_onda_da_prega`](../../tests/sonda_da_onda_da_prega.rs), a `80`
/// vértices por lado e com a dobra no máximo: `1` passagem dá `10,0` arestas com
/// amplitude `0,106`; `8` dão **`26,7`** com `0,145`; `32` dão `20,0` com
/// **`0,287`** — *a onda cresce `2`–`2,7×` e a amplitude SOBE*, logo não é a peça
/// a achatar.
///
/// ⛔ **Isto NÃO faz dele um botão, e a mesma sonda diz porquê:** na malha
/// GROSSA (`20` por lado) as passagens matam o caimento — a amplitude cai de
/// `0,329` para `0,0016` (`200×`) e a régua passa a contar ruído, que é o modo de
/// falha já registado no doc 11 §2.1. E a régua é grosseira por construção (conta
/// lobos: com `3`–`8` numa fileira, cada passo é um salto grande).
///
/// ⇒ o que fica REFUTADO é a inevitabilidade, não a dificuldade. A cura
/// publicada continua a ser o solver **hierárquico** (Müller, 2008) — mas ele é a
/// via BARATA de ter muitas passagens, **não** um pré-requisito para a feature
/// existir. ⚠️ Uma wave que pegue nisto começa por uma régua melhor (a fileira
/// por FFT ou autocorrelação, não contagem de lobos) e pelo preço das passagens.
///
/// ⭐ **Este gate continua VÁLIDO e não foi afrouxado:** ele compara grossa
/// contra fina nos valores de FÁBRICA (`bend = 0`, `1` passagem), e ali a onda
/// segue mesmo a malha.
#[test]
fn a_onda_de_uma_prega_acompanha_a_aresta_da_malha() {
    // Uma cortina franzida: o material sobra e tem de pregar.
    let cortina = |n: usize| -> Mesh {
        let s = 2.0 / n as f32;
        let mut pos = Vec::new();
        for j in 0..=n {
            for i in 0..=n {
                let z = 0.0005 * ((i * 7 + j * 13) % 5) as f32;
                let x = i as f32 * s - 1.0;
                pos.push([if j == 0 { x * 0.5 } else { x }, 1.0 - j as f32 * s, z]);
            }
        }
        let id = |i: usize, j: usize| u32::try_from(j * (n + 1) + i).unwrap_or(u32::MAX);
        let mut faces = Vec::new();
        for j in 0..n {
            for i in 0..n {
                faces.push(ph2d_mesh::Face::tri(
                    id(i, j),
                    id(i + 1, j),
                    id(i + 1, j + 1),
                ));
                faces.push(ph2d_mesh::Face::tri(
                    id(i, j),
                    id(i + 1, j + 1),
                    id(i, j + 1),
                ));
            }
        }
        let mut m = Mesh::from_parts(pos, faces).unwrap_or_else(|e| panic!("{e:?}"));
        let alto: Vec<bool> = m.positions().iter().map(|p| p[1] > 0.999).collect();
        let mk = m.masks_mut();
        for (i, a) in alto.iter().enumerate() {
            if *a {
                mk[i] = 1.0;
            }
        }
        m
    };
    // Travessias da média ao longo da fileira do meio, com PROEMINÊNCIA — sem o
    // limiar de amplitude ela conta o ruído de `f32` de uma folha esticada.
    let onda = |m: &Mesh, n: usize| -> f64 {
        let p = m.positions();
        let z: Vec<f64> = (0..=n)
            .map(|i| f64::from(p[(n / 2) * (n + 1) + i][2]))
            .collect();
        let media: f64 = z.iter().sum::<f64>() / z.len() as f64;
        let amp = z.iter().fold(0.0f64, |a, v| a.max((v - media).abs()));
        let (mut cruzes, mut lado, mut lobo) = (0usize, (z[0] - media).signum(), 0.0f64);
        for v in &z {
            let d = v - media;
            if d.signum() != lado && lobo > 0.15 * amp {
                cruzes += 1;
                lado = d.signum();
                lobo = 0.0;
            } else {
                lobo = lobo.max(d.abs());
            }
        }
        if cruzes == 0 {
            f64::INFINITY
        } else {
            2.0 / cruzes as f64
        }
    };
    let corre = |n: usize| {
        let mut m = cortina(n);
        let mut st = SculptStroke::default();
        let props = ClothFilterProps::default();
        st.cloth_filter_begin(&m, props, ClothFilterKind::Gravity, [0.0, 1.0, 0.0]);
        for k in 0..200 {
            let p = ClothFilterStep {
                s: (k as f32 + 1.0) / 200.0,
                gravity_axis: [0.0, -1.0, 0.0],
                frame: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                axes: [true, true, true],
                eye: [0.0, 0.0, 1.0],
            };
            st.cloth_filter_step(&mut m, ClothFilterKind::Gravity, &p);
        }
        st.cloth_filter_end();
        onda(&m, n)
    };
    let (grossa, fina) = (corre(20), corre(80));
    println!("onda: malha grossa {grossa:.4} | malha fina {fina:.4}");
    // A aresta muda `4×`; a onda tem de acompanhar, e não ficar parada.
    assert!(
        grossa > fina * 1.5,
        "a onda deixou de seguir a malha ({grossa:.4} contra {fina:.4}) -- se isto \
         reprovar por MELHORIA, a nota do gate e' que envelheceu"
    );
}

/// ⭐⭐⭐ **O `Expand` OBEDECE AO TECTO — e a barra é `tecto²`, derivada.**
///
/// # O defeito que este gate fecha
///
/// O tecto compara `|aresta|` contra `tecto × ℓ`, e o `ℓ` dele é o repouso
/// **corrente** (`ℓ_material + τ`). O *Expand* é o único dos cinco tipos que mexe
/// no `τ` ⇒ *o denominador da régua crescia com a lei que ela devia limitar*.
/// Medido antes da cura, na esfera de fábrica com um arrasto de curso inteiro:
/// `7,743` de esticão contra o material a UM gesto e `14,358` a três, contra
/// `1,1`–`1,4` de todos os outros quatro.
///
/// ⚠️ **E o que aquilo produzia não era uma peça maior, era uma AMARROTADA:** o
/// volume dava `1,067 → 0,245 → 0,761`. Depois da cura ele é monótono —
/// `1,120 → 1,288 → 1,470` —, que é o que a palavra *expandir* promete.
///
/// # A barra NÃO é escolhida
///
/// [`ph2d_cloth`] passou a limitar o repouso a `tecto ×` o material
/// (`Verlet::limitar_o_repouso`), e o tecto de esticão limita a posição a
/// `tecto ×` o repouso. Compondo, o esticão contra o MATERIAL não pode passar de
/// **`tecto²`** — `1,21` na omissão. Medido: `1,210`.
///
/// ⛔ A folga de `2 %` é a convergência do limitador (ele é uma projecção de
/// Jacobi com um número finito de passagens), **não** margem para o defeito
/// voltar: o valor de antes era `6,4×` a barra.
#[test]
fn o_expand_obedece_ao_tecto_como_os_outros_quatro() {
    let repouso = esfera();
    let props = ClothFilterProps::default();
    let tecto = f64::from(props.stretch_max);
    let barra = tecto * tecto * 1.02;

    let m = correr(props, ClothFilterKind::Expand, 1, false);
    let e = estica(&repouso, &m);
    let v = volume(&m) / volume(&repouso);
    println!("[expand] estica contra o material {e:.4} (barra {barra:.4}) | volume {v:.4}");

    assert!(
        e <= barra,
        "o Expand furou o tecto: {e:.4} contra a barra DERIVADA tecto^2 = {barra:.4}. \
         Antes da cura ele dava 7,743 -- se isto reprovar, o `limitar_o_repouso` \
         deixou de ser chamado ou o tecto voltou a medir-se contra `l + tau`"
    );
    // ⚠️ **A METADE QUE DIZ QUE ELE AINDA FAZ ALGUMA COISA.** Sem ela, apagar o
    // Expand inteiro passaria este gate — um tecto obedecido por um no-op.
    assert!(
        v > 1.05,
        "o Expand deixou de EXPANDIR: volume {v:.4}. O gate nao pode ser satisfeito \
         por uma lei que nao faz nada"
    );
}
