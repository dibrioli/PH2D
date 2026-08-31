//! **O ASPECTO DE UMA FOLHA** — o tamanho final e os dois sorteios.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 600 para `shells/`), e o corte é por
//! responsabilidade: o irmão mede ONDE a folha nasce e em que ordem se desenha, e este mede COM
//! QUE CARA.
//!
//! ⛔⛔ **A lição que os dois gates daqui pagaram:** o primeiro mede a LEI (pura) e o segundo o
//! CONSUMO dela pela membrana — e a mutação que apagava o consumo **sobreviveu** enquanto só
//! havia o primeiro. *Gatear a lei e não o consumidor é gatear metade.*

use crate::render_loop::motion_lsystem_gen::publish;
use crate::render_loop::motion_lsystem_testkit::*;
use ph2d_node_source_lsystem as ls;

/// ⭐⭐ **O TAMANHO FINAL E OS DOIS SORTEIOS** — report do Enio (2026-08-30): *"não temos
/// parâmetros para o tamanho final da folha nem jitter de scale e posição"*.
///
/// ⚠️ **A primeira afirmação protege as outras duas:** os três nascem NEUTROS e o neutro tem de
/// ser exacto — uma feature nova não pode mexer um bit no que já shipou.
#[test]
fn the_leaf_has_a_final_size_and_two_jitters() {
    use crate::render_loop::motion_lsystem_leaves::LeafLook;
    let neutro = LeafLook {
        front: 0.0,
        keep_own_colour: true,
        size: 1.0,
        size_jitter: 0.0,
        pos_jitter: 0.0,
    };
    // 1. ⛔ **O neutro é a identidade AO BIT.**
    for i in 0..64 {
        let (s, d) = neutro.at(i);
        assert_eq!(
            s.to_bits(),
            1.0f32.to_bits(),
            "o tamanho neutro tem de ser 1 exacto"
        );
        assert_eq!(d, [0.0, 0.0], "o empurrao neutro tem de ser zero exacto");
    }
    // 2. O `Leaf Size` multiplica, e sozinho não sorteia nada.
    let dobro = LeafLook {
        size: 2.0,
        ..neutro
    };
    for i in 0..64 {
        assert_eq!(
            dobro.at(i).0,
            2.0,
            "sem jitter todas as folhas tem o mesmo tamanho"
        );
    }
    // 3. O `Size Jitter` varia ENTRE folhas, dentro da faixa, e é determinístico.
    let variado = LeafLook {
        size: 1.0,
        size_jitter: 0.4,
        ..neutro
    };
    let tamanhos: Vec<f32> = (0..64).map(|i| variado.at(i).0).collect();
    let (mn, mx) = tamanhos
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), s| (a.min(*s), b.max(*s)));
    assert!(
        mn >= 0.8 - 1e-5 && mx <= 1.2 + 1e-5,
        "fora da faixa +-20%: {mn}..{mx}"
    );
    assert!(mx - mn > 0.2, "as folhas nao variaram entre si: {mn}..{mx}");
    for (i, t) in tamanhos.iter().enumerate() {
        assert_eq!(variado.at(i).0.to_bits(), t.to_bits(), "o sorteio reproduz");
    }
    // 4. O `Position Jitter` empurra, dentro de meia folha, nos DOIS eixos.
    let empurrado = LeafLook {
        pos_jitter: 1.0,
        ..neutro
    };
    let ds: Vec<[f32; 2]> = (0..64).map(|i| empurrado.at(i).1).collect();
    assert!(
        ds.iter()
            .all(|d| d[0].abs() <= 0.5 + 1e-5 && d[1].abs() <= 0.5 + 1e-5),
        "o empurrao saiu de meia folha"
    );
    let span = |k: usize| {
        let (a, b) = ds
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), d| (a.min(d[k]), b.max(d[k])));
        b - a
    };
    assert!(span(0) > 0.5 && span(1) > 0.5, "um dos eixos nao se mexeu");
    // ⛔⛔ **E OS DOIS SORTEIOS SÃO INDEPENDENTES** — com uma LANE só, a folha maior seria
    // sempre a mais empurrada, e isso lê-se como um padrão, não como acaso.
    let ambos = LeafLook {
        size: 1.0,
        size_jitter: 1.0,
        pos_jitter: 1.0,
        ..neutro
    };
    let (xs, ys): (Vec<f32>, Vec<f32>) =
        (0..256).map(|i| (ambos.at(i).0, ambos.at(i).1[0])).unzip();
    let (mx_, my) = (
        xs.iter().sum::<f32>() / xs.len() as f32,
        ys.iter().sum::<f32>() / ys.len() as f32,
    );
    let cov: f32 = xs
        .iter()
        .zip(&ys)
        .map(|(a, b)| (a - mx_) * (b - my))
        .sum::<f32>()
        / xs.len() as f32;
    let (sx, sy) = (
        (xs.iter().map(|a| (a - mx_).powi(2)).sum::<f32>() / xs.len() as f32).sqrt(),
        (ys.iter().map(|b| (b - my).powi(2)).sum::<f32>() / ys.len() as f32).sqrt(),
    );
    let r = cov / (sx * sy);
    assert!(
        r.abs() < 0.25,
        "o tamanho e o empurrao estao correlacionados (r = {r:.3}) — uma LANE so'"
    );
}
/// ⛔⛔ **E A MEMBRANA TEM DE OS CONSUMIR** — a lei acima é pura, e uma lei que ninguém aplica
/// passa em todos os gates dela.
///
/// Medido: a mutação que apagava o `scale` do tamanho da linha **SOBREVIVEU** à suíte inteira.
/// *Gatear a lei e não o consumidor é gatear metade.*
#[test]
fn the_membrane_applies_the_size_and_the_jitters() {
    let publicar = |size: f32, size_jitter: f32, pos_jitter: f32| -> (Vec<f32>, Vec<[f32; 2]>) {
        let (mut state, n) = factory_plant_with_leaf(5.0, false);
        state.doc.graph.set_param(n, ls::param::LEAF_SIZE, size);
        state
            .doc
            .graph
            .set_param(n, ls::param::LEAF_SIZE_JITTER, size_jitter);
        state
            .doc
            .graph
            .set_param(n, ls::param::LEAF_POS_JITTER, pos_jitter);
        let key = key_of(&mut state, n);
        publish(&mut state, 0.0);
        let inst = instances_of(&state, &key);
        (
            inst.iter().map(|i| i.size[0]).collect(),
            inst.iter().map(|i| i.world_pos).collect(),
        )
    };
    let (base, pos_base) = publicar(1.0, 0.0, 0.0);
    assert!(base.len() > 8, "so' {} folhas", base.len());
    // 1. O `Leaf Size` chega à instância.
    let (dobro, _) = publicar(2.0, 0.0, 0.0);
    for (a, b) in base.iter().zip(&dobro) {
        assert!(
            (b - a * 2.0).abs() < 1e-4,
            "o Leaf Size nao chegou: {a} -> {b}"
        );
    }
    // 2. O `Size Jitter` faz as folhas diferirem UMAS DAS OUTRAS.
    let (variado, _) = publicar(1.0, 0.6, 0.0);
    let distintos = {
        let mut v: Vec<i64> = variado.iter().map(|s| (s * 1e4) as i64).collect();
        v.sort_unstable();
        v.dedup();
        v.len()
    };
    assert!(
        distintos > base.len() / 2,
        "so' {distintos} tamanhos distintos em {} folhas — o jitter nao chegou",
        variado.len()
    );
    // 3. O `Position Jitter` desencosta a folha da âncora, sem a perder de vista.
    let (_, pos) = publicar(1.0, 0.0, 1.0);
    let mut mexidas = 0;
    for (a, b) in pos_base.iter().zip(&pos) {
        let d = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
        if d > 1e-5 {
            mexidas += 1;
        }
        // A folha publicada mede `2 × 3`; meia folha é `1` no maior eixo.
        assert!(d < 2.0, "a folha fugiu do ramo: {d}");
    }
    assert!(
        mexidas * 4 >= pos_base.len() * 3,
        "so' {mexidas} de {} folhas se mexeram",
        pos_base.len()
    );
}
