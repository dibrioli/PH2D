//! Gates do **ATRASO POR CÓPIA** (doc 89, folha 08 — o *Shape Time Offset* do Cavalry).
//!
//! ⚠️ Arquivo próprio por assunto: o `lib_tests.rs` responde *"onde as cópias ficam"*, e este
//! responde *"em que INSTANTE cada uma lê"*.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};

/// Uma fatia com `n` peças, todas em `x`.
fn slice(n: usize, x: f32) -> Stream {
    Stream::new(n).with("P", Column::Vec2(vec![[x, 0.0]; n]))
}

fn pos(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// Uma colocação inerte, para os gates medirem a CONCATENAÇÃO e não a geometria.
fn still() -> impl Fn(usize) -> radial::Placement {
    |_| radial::Placement::of(false, 0.0, 0.0, 0.0, 0.0, [0.0, 0.0])
}

/// ⭐⭐ **A LEI DO LEQUE CONCORDA COM A DE SEMPRE onde as duas se sobrepõem.**
///
/// É o gate que justifica existirem duas: com todas as fatias iguais, concatenar `k` cópias da
/// mesma coisa **é** replicá-la `k` vezes. Sem ele, duas leis para a mesma pergunta divergiriam
/// no primeiro dia em que alguém tocasse numa só.
#[test]
fn the_fanned_law_agrees_with_the_plain_one_when_every_slice_is_the_same() {
    let one = Stream::new(3)
        .with("P", Column::Vec2(vec![[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]]))
        .with("size", Column::Vec2(vec![[2.0, 2.0]; 3]))
        .with("rot", Column::Scalar(vec![10.0, 20.0, 30.0]));
    for (st, rt) in [(1.0f32, 0.0f32), (0.4, 0.0), (1.0, 90.0), (0.5, 45.0)] {
        let plain = clone_stream(&one, 4, &still(), st, rt);
        let slices: Vec<&Stream> = (0..4).map(|_| &one).collect();
        let fanned = fan::clone_fanned(&slices, &still(), st, rt);
        assert_eq!(plain.count(), fanned.count(), "taper ({st}, {rt})");
        for (name, col) in plain.columns() {
            assert_eq!(
                Some(col),
                fanned.get(name),
                "a coluna `{name}` diverge com taper ({st}, {rt})"
            );
        }
    }
}

/// ⭐ **CONTAGENS DIFERENTES SOMAM.** É o caso que a célula nomeia: um `sim.spawn` a montante
/// nasce e mata entre os instantes, então as fatias não têm o mesmo tamanho.
#[test]
fn slices_of_different_lengths_are_summed_not_truncated() {
    let (a, b, c) = (slice(2, 0.0), slice(5, 1.0), slice(1, 2.0));
    let out = fan::clone_fanned(&[&a, &b, &c], &still(), 1.0, 0.0);
    assert_eq!(out.count(), 8, "2 + 5 + 1");
    let p = pos(&out);
    assert_eq!(p.len(), 8, "e a coluna acompanha a contagem");
    // E a ORDEM é a das fatias — a cópia 0 primeiro, inteira.
    assert!(p[0][0] == 0.0 && p[1][0] == 0.0, "a 1a fatia: {p:?}");
    assert!(p[2][0] == 1.0 && p[6][0] == 1.0, "a 2a fatia: {p:?}");
    assert!(p[7][0] == 2.0, "a 3a fatia: {p:?}");
}

/// ⚠️ **UMA COLUNA QUE SÓ ALGUMAS FATIAS TRAZEM É PREENCHIDA, NUNCA SALTADA.** As fatias são
/// concatenadas por POSIÇÃO, então uma coluna mais curta desalinharia todas as outras a
/// partir dali — e em silêncio, porque um `Stream` não valida comprimentos entre colunas.
#[test]
fn a_column_only_some_slices_carry_is_filled_not_skipped() {
    let with = Stream::new(2)
        .with("P", Column::Vec2(vec![[0.0, 0.0]; 2]))
        .with("tint", Column::Vec4(vec![[1.0, 0.0, 0.0, 1.0]; 2]));
    let without = slice(3, 1.0);
    let out = fan::clone_fanned(&[&with, &without], &still(), 1.0, 0.0);
    assert_eq!(out.count(), 5);
    match out.get("tint") {
        Some(Column::Vec4(v)) => {
            assert_eq!(v.len(), 5, "a coluna cobre as CINCO linhas, nao duas");
            assert_eq!(v[0], [1.0, 0.0, 0.0, 1.0], "a fatia que a traz");
            assert_eq!(v[2], [0.0; 4], "a que nao a traz recebe a identidade");
        }
        other => panic!("tint: {other:?}"),
    }
    // ⚠️ E a ordem da união é estável — a `P` (da 1.ª fatia) continua alinhada.
    assert_eq!(pos(&out).len(), 5);
}

/// **O índice global cobre o conjunto INTEIRO**, como no caminho de sempre — uma rampa de cor
/// a jusante tem de ler `0..total` sem interrupção, seja qual for o tamanho de cada fatia.
#[test]
fn the_global_index_runs_over_the_whole_concatenated_set() {
    // ⚠️ As fatias TÊM de trazer as duas colunas: a lei re-escreve-as, não as inventa — a
    // mesma escolha da `clone_stream`, e uma fixtura sem elas mediria a ausência.
    let a = slice(2, 0.0)
        .with("Index", Column::Scalar(vec![0.0, 1.0]))
        .with("Count", Column::Scalar(vec![2.0, 2.0]));
    let b = slice(3, 1.0)
        .with("Index", Column::Scalar(vec![0.0, 1.0, 2.0]))
        .with("Count", Column::Scalar(vec![3.0; 3]));
    let out = fan::clone_fanned(&[&a, &b], &still(), 1.0, 0.0);
    match out.get("Index") {
        Some(Column::Scalar(v)) => assert_eq!(v, &vec![0.0, 1.0, 2.0, 3.0, 4.0]),
        other => panic!("Index: {other:?}"),
    }
    match out.get("Count") {
        Some(Column::Scalar(v)) => assert!(v.iter().all(|c| *c == 5.0)),
        other => panic!("Count: {other:?}"),
    }
}

/// **O taper conta as fatias, não as peças** — a cópia `c` recebe o factor de `c`, seja qual
/// for o tamanho dela.
#[test]
fn the_taper_indexes_copies_not_elements() {
    let (a, b) = (slice(1, 0.0), slice(4, 0.0));
    let out = fan::clone_fanned(&[&a, &b], &still(), 0.0, 0.0);
    match out.get("size") {
        Some(Column::Vec2(v)) => {
            assert_eq!(v.len(), 5);
            // Cópia 0 → factor 1 (o taper ainda não mordeu); cópia 1 (a última) → factor 0.
            assert!(
                (v[0][0] - SIZE_IDENTITY[0]).abs() < 1e-6,
                "a 1a copia: {:?}",
                v[0]
            );
            assert!(v[4][0].abs() < 1e-6, "a ultima copia: {:?}", v[4]);
        }
        other => panic!("size: {other:?}"),
    }
}

/// ⚠️ **`time_offset = 0` NÃO monta leque** — é o que faz todo documento de hoje cozinhar
/// exactamente como antes. O gate mede o par, senão um leque vazio passaria por vácuo.
#[test]
fn a_zero_offset_builds_no_fan_at_all() {
    use ph2d_nodegraph::graph::Graph;
    let mut reg = ph2d_node_registry::NodeRegistry::new();
    register(&mut reg).expect("regista");
    let mut g = Graph::new();
    let n = g.add_node(MANIFEST.name);
    g.set_param(n, "count", 4.0);
    assert!(
        fan::time_fans(&g, &reg, 1.0 / 60.0).is_empty(),
        "sem `time_offset` nao ha' leque"
    );
    // O CONTROLE: com atraso, o leque tem uma fatia por cópia.
    g.set_param(n, fan::TIME_OFFSET, 0.25);
    let fans = fan::time_fans(&g, &reg, 1.0 / 60.0);
    let maps = fans.get(&n).expect("o leque deste no'");
    assert_eq!(maps.len(), 4, "uma fatia por copia");
    // ⭐ A cópia 0 é o AGORA, exactamente — senão ligar o knob moveria o desenho inteiro.
    assert!(
        (maps[0].offset - 0.0).abs() < 1e-12,
        "a copia 0: {}",
        maps[0].offset
    );
    assert!(
        (maps[1].offset + 0.25).abs() < 1e-12,
        "a copia 1 recua: {}",
        maps[1].offset
    );
    assert!(
        (maps[3].offset + 0.75).abs() < 1e-12,
        "a copia 3: {}",
        maps[3].offset
    );
}

/// **Um atraso NEGATIVO lê o futuro** — é uma antecipação, não um erro, e o slider é simétrico
/// por isso.
#[test]
fn a_negative_offset_reads_forward_in_time() {
    use ph2d_nodegraph::graph::Graph;
    let mut reg = ph2d_node_registry::NodeRegistry::new();
    register(&mut reg).expect("regista");
    let mut g = Graph::new();
    let n = g.add_node(MANIFEST.name);
    g.set_param(n, "count", 3.0);
    g.set_param(n, fan::TIME_OFFSET, -0.5);
    let fans = fan::time_fans(&g, &reg, 1.0 / 60.0);
    let maps = fans.get(&n).expect("o leque");
    assert!(maps[1].offset > 0.0, "a copia 1 avanca: {}", maps[1].offset);
}

/// ⚠️ **Um atraso ABAIXO DO PISO não monta leque** — ele custaria `k` cozeduras que devolvem o
/// MESMO instante: o preço inteiro do leque pelo desenho de sempre.
#[test]
fn an_offset_under_the_floor_is_not_worth_a_fan() {
    use ph2d_nodegraph::graph::Graph;
    let mut reg = ph2d_node_registry::NodeRegistry::new();
    register(&mut reg).expect("regista");
    let mut g = Graph::new();
    let n = g.add_node(MANIFEST.name);
    g.set_param(n, fan::TIME_OFFSET, fan::MIN_OFFSET * 0.5);
    assert!(fan::time_fans(&g, &reg, 1.0 / 60.0).is_empty());
    // O CONTROLE: no piso, ele monta.
    g.set_param(n, fan::TIME_OFFSET, fan::MIN_OFFSET);
    assert!(!fan::time_fans(&g, &reg, 1.0 / 60.0).is_empty());
}
