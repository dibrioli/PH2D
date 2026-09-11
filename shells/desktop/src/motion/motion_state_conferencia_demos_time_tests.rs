//! Gates da cena `=42` — **o ruído e o relógio**.
//!
//! ⚠️ A cena do grupo A julgava-se numa foto; esta tem uma leitura que só o PLAY
//! responde, e por isso os gates aqui fazem uma coisa que os do irmão não faziam:
//! **cozem a MESMA cena em instantes diferentes**. Um laço que fecha e um que não
//! fecha produzem exatamente a mesma imagem parada.

use super::super::conferencia_demos_time;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::NodeId;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).unwrap();
    reg
}

/// Coza a cena num instante e devolva o `Y` de cada fileira, na ordem da tela.
fn rows_at(t: f64) -> Vec<Vec<f32>> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = conferencia_demos_time::build_time_demo_document(&mut doc, &reg)
        .expect("a cena tem de montar");
    assert_eq!(
        sinks.len(),
        conferencia_demos_time::ROWS,
        "uma fileira por sink"
    );
    let mut cook = Cook::new();
    sinks
        .iter()
        .map(|&s| ys(&mut cook, &doc, &reg, s, t))
        .collect()
}

fn ys(cook: &mut Cook, doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId, t: f64) -> Vec<f32> {
    let out = cook.cook(&doc.graph, reg, sink, t).expect("cook");
    match out[0].as_stream().get("P") {
        Some(Column::Vec2(v)) => v.iter().map(|p| p[1]).collect(),
        _ => panic!("sem coluna P"),
    }
}

/// O desvio máximo entre dois perfis — o oráculo de *"é a mesma forma?"*.
fn worst(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).abs())
        .fold(0.0, f32::max)
}

/// O perfil **sem a colocação da fileira** — cada fileira é levantada por um
/// `offset_y` próprio, e comparar duas fileiras cruas mede o LAYOUT em vez do
/// campo. ⚠️ Foi exactamente assim que o gate do lock-step nasceu vermelho por
/// `1,15`, que é o `ROW_GAP` da cena e não uma divergência de régua.
fn shape(v: &[f32]) -> Vec<f32> {
    let mean = v.iter().sum::<f32>() / v.len() as f32;
    v.iter().map(|x| x - mean).collect()
}

/// A amplitude de um perfil — a régua contra a qual todo desvio é lido, e a prova
/// de que a fileira não é uma barra chata.
fn swing(v: &[f32]) -> f32 {
    v.iter().fold(f32::NEG_INFINITY, |m, x| m.max(*x))
        - v.iter().fold(f32::INFINITY, |m, x| m.min(*x))
}

#[test]
fn the_time_scene_builds_every_row() {
    let rows = rows_at(0.0);
    for (i, r) in rows.iter().enumerate() {
        assert_eq!(
            r.len(),
            conferencia_demos_time::COLS as usize,
            "fileira {} com contagem errada",
            i + 1
        );
    }
}

/// **Toda fileira DESENHA alguma coisa** — um perfil chato concordaria com
/// qualquer par e faria os gates abaixo verdes por vácuo.
#[test]
fn every_row_draws_a_profile() {
    for (i, r) in rows_at(0.31).iter().enumerate() {
        // O Y carrega a posição da grade (constante na fileira) MAIS o valor, e é
        // o valor que tem de variar. A grade é de uma linha ⇒ a base é a mesma.
        let s = swing(r);
        assert!(s > 0.05, "fileira {} é chata ({s:e})", i + 1);
    }
}

/// **A ESTRELA: o campo com laço VOLTA, e o sem laço NÃO.**
///
/// ⚠️ As duas metades são o gate. Sem a segunda, um campo CONGELADO fecharia
/// qualquer costura e o gate ficaria verde sobre um ruído morto; sem a primeira,
/// nada distingue o laço de um param ignorado.
#[test]
fn the_looped_row_returns_and_the_open_one_does_not() {
    let l = f64::from(conferencia_demos_time::loop_seconds());
    let (a, b) = (rows_at(0.0), rows_at(l));
    // Fileira 2 (índice 1) é a do laço; fileira 1 (índice 0) o controle.
    let seam = worst(&a[1], &b[1]);
    let drift = worst(&a[0], &b[0]);
    assert!(seam < 1e-4, "a fileira do laço NÃO voltou: {seam:e}");
    assert!(
        drift > 0.05,
        "o controle mal se moveu em {l}s ({drift:e}) — o fecho acima é vácuo"
    );
    // E meia volta tem de ser DIFERENTE do começo: um laço que congela também
    // "fecha".
    let mid = rows_at(l * 0.5);
    let inside = worst(&a[1], &mid[1]);
    assert!(inside > 0.05, "o laço congelou o campo ({inside:e})");
}

/// **A lacunarity acrescenta DETALHE, e o detalhe mede-se como aspereza.**
///
/// O oráculo não é *"os dois perfis diferem"* (dois campos diferentes diferem por
/// qualquer motivo) — é a soma das diferenças entre peças VIZINHAS, que é o que
/// "mais fino" significa num gráfico: mais subidas e descidas no mesmo espaço.
#[test]
fn the_higher_lacunarity_draws_a_rougher_profile() {
    let rows = rows_at(0.0);
    let roughness = |v: &[f32]| -> f32 { v.windows(2).map(|w| (w[1] - w[0]).abs()).sum() };
    let (two, four) = (roughness(&rows[2]), roughness(&rows[3]));
    assert!(
        four > two * 1.2,
        "lacunarity 4 não é mais áspera que 2: {four:.4} contra {two:.4}"
    );
}

/// **O PAN é um VETOR: `pan_y` DESLIZA a fileira, `pan_x` troca a FATIA.**
///
/// ⚠️ **A afirmação que este gate fazia antes era falsa, e quem a derrubou foi
/// outro gate meu.** Ele dizia *"o pan desliza e o seed re-sorteia"*; o
/// `a_pan_of_one_is_a_seed_of_one` prova byte a byte que `pan_y` e `seed` são o
/// MESMO eixo, então essa comparação nunca poderia distinguir nada. O que separa
/// os dois é o GESTO (o passo do widget), que não se mede num cook.
///
/// O que se afirma agora é a propriedade verdadeira: o eixo Y **desliza** (o
/// perfil reaparece adiante, com correlação alta) e o eixo X — onde não existe
/// `seed` — escolhe **outra fatia** (nenhum deslocamento ao longo da fila o
/// alinha com o controle). É essa metade que nenhum param anterior alcançava.
#[test]
fn the_pan_slides_along_the_row_and_changes_the_slice_across_it() {
    let rows = rows_at(0.0);
    let (control, slid_row, sliced) = (shape(&rows[4]), shape(&rows[5]), shape(&rows[6]));
    // Os três são campos distintos — a premissa.
    assert!(worst(&control, &slid_row) > 0.02, "o pan_y não moveu nada");
    assert!(worst(&control, &sliced) > 0.02, "o pan_x não moveu nada");

    // O melhor alinhamento de Pearson sobre os deslocamentos que a fileira
    // permite. ⚠️ Uma célula do reticulado vale `1/frequency ≈ 7,7` peças, então
    // 0,4 de célula é ~3 peças — a janela cobre-o com folga.
    let best_corr = |v: &[f32]| -> f32 {
        (0..12)
            .map(|s| corr(&control[s..], &v[..v.len() - s]))
            .fold(f32::NEG_INFINITY, f32::max)
    };
    let (slid, other) = (best_corr(&slid_row), best_corr(&sliced));
    assert!(
        slid > 0.9,
        "o perfil com pan_y não é o controle deslizado (corr {slid:.3})"
    );
    assert!(
        other < slid - 0.15,
        "a fatia de pan_x parece um deslize (corr {other:.3} contra {slid:.3})"
    );
}

/// Pearson entre duas fatias de igual comprimento.
fn corr(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    let (a, b) = (&a[..n], &b[..n]);
    let mean = |v: &[f32]| v.iter().sum::<f32>() / n as f32;
    let (ma, mb) = (mean(a), mean(b));
    let (mut num, mut da, mut db) = (0.0f32, 0.0f32, 0.0f32);
    for i in 0..n {
        let (x, y) = (a[i] - ma, b[i] - mb);
        num += x * y;
        da += x * x;
        db += y * y;
    }
    let den = (da * db).sqrt();
    if den == 0.0 { 0.0 } else { num / den }
}

/// **As duas réguas andam em LOCK-STEP** — 0,5 s por ciclo e 120 BPM são o mesmo
/// número, então as fileiras 8 e 9 têm de ser o MESMO perfil em todo instante.
///
/// ⚠️ E a fileira 10 (180 BPM) é o CONTROLE: sem ela, um `time_mode` ignorado
/// deixaria as três iguais e as duas primeiras casariam por acidente.
///
/// ⚠️ **A varredura NÃO começa em zero, e o motivo é o próprio modelo:** a fase é
/// `t/period + phase + i·stagger`, então em `t = 0` o termo da régua **desaparece**
/// e as três fileiras coincidem por construção — o instante em que este gate não
/// pode medir o que afirma. O caso degenerado fica afirmado à parte, em vez de
/// escondido: ele é uma propriedade, não uma falha.
#[test]
fn at_the_clock_origin_every_ruler_agrees() {
    let rows = rows_at(0.0);
    let (a, b, c) = (shape(&rows[7]), shape(&rows[8]), shape(&rows[9]));
    assert!(worst(&a, &b) < 1e-5, "t=0: as reguas divergiram");
    assert!(
        worst(&a, &c) < 1e-5,
        "t=0: 180 BPM difere — o termo da regua nao desapareceu"
    );
}

#[test]
fn the_two_rulers_move_in_lock_step_and_the_third_does_not() {
    for t in [0.17f64, 0.43, 1.1] {
        let rows = rows_at(t);
        // ⚠️ SHAPE e não Y cru: cada fileira é levantada pelo seu `offset_y`.
        let (seconds, bpm120, bpm180) = (shape(&rows[7]), shape(&rows[8]), shape(&rows[9]));
        let same = worst(&seconds, &bpm120);
        assert!(
            same < 1e-5,
            "t {t}: 0,5 s e 120 BPM divergiram ({same:e}) — não é a mesma régua"
        );
        let faster = worst(&seconds, &bpm180);
        assert!(
            faster > 0.05,
            "t {t}: 180 BPM não se distingue de 120 ({faster:e})"
        );
    }
}

/// **Nenhuma fileira sobe para dentro da vizinha** — se subissem, os dez gráficos
/// virariam um borrão e o smoke não teria o que ler.
#[test]
fn no_row_climbs_into_its_neighbour() {
    let rows = rows_at(0.29);
    for k in 0..rows.len() - 1 {
        let lo_of_upper = rows[k].iter().fold(f32::INFINITY, |m, v| m.min(*v));
        let hi_of_lower = rows[k + 1].iter().fold(f32::NEG_INFINITY, |m, v| m.max(*v));
        assert!(
            lo_of_upper > hi_of_lower,
            "fileira {} encosta na {}: {lo_of_upper:.3} contra {hi_of_lower:.3}",
            k + 1,
            k + 2
        );
    }
}

/// **Toda fileira é reivindicada pelo DEVICE de ponta a ponta.**
///
/// ⚠️ Isto é headless: `ph2d_gpu_cook::plan` não precisa de adapter. Um param
/// dirigido por fio excluiria o nó da GPU (`eligible` recusa), e é por isso que a
/// cena mostra o pan ESTATICAMENTE em vez de o animar — a demonstração animada
/// teria posto três fileiras na CPU.
#[test]
fn every_row_is_claimed_end_to_end_by_the_device() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = conferencia_demos_time::build_time_demo_document(&mut doc, &reg).expect("monta");
    for (i, &s) in sinks.iter().enumerate() {
        let plan = ph2d_gpu_cook::plan(&doc.graph, &reg, &reg, s);
        assert!(
            plan.is_fully_gpu(),
            "fileira {} não é reivindicada pelo device",
            i + 1
        );
    }
}

/// **O anúncio nomeia todas as fileiras que a cena constrói** — uma lista escrita
/// à mão driftaria da tela.
#[test]
fn the_announcement_names_every_row_that_is_built() {
    let labels: Vec<_> = conferencia_demos_time::row_labels().collect();
    assert_eq!(labels.len(), conferencia_demos_time::ROWS);
    for (i, l) in labels {
        assert!(!l.is_empty(), "fileira {} sem rótulo", i + 1);
    }
}
