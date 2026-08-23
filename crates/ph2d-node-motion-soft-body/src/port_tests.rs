//! Gates da porta **`shape`** — o corpo que deixa de ser obrigatoriamente um
//! rectângulo (doc 89, folha 03, o P1).
//!
//! Ficam à parte do `tests.rs` porque ele bateu no tecto de LOC (HR-18), e a
//! costura é a natural: aqui só se pergunta *o que acontece quando a forma vem
//! de fora*. A geometria pura do arranjo (o anel, o pino, as regiões) é
//! falsificada no `layout_tests`; o ajuste de forma, no `shape_tests`.

use super::*;
use crate::tests::params;

/// Um disco de `n` pontos, centrado onde se pedir — a nuvem que nenhuma grelha
/// exprime.
fn disc(n: usize, r: f32, centre: [f32; 2]) -> Vec<[f32; 2]> {
    (0..n)
        .map(|k| {
            let t = k as f32 / n as f32 * core::f32::consts::TAU;
            [centre[0] + r * t.cos(), centre[1] + r * t.sin()]
        })
        .collect()
}

/// A porta GANHA de `rows`/`cols`: o corpo tem a contagem da nuvem, não a da
/// malha — e é semeado na forma dela, transladada para a âncora.
#[test]
fn a_shape_on_the_port_becomes_the_body() {
    let p = params(4, 4, 9.0, 0.5, true); // 16 partículas se a porta perdesse
    let shape = disc(24, 2.0, [7.0, -3.0]);
    let out = simulate([1.0, 5.0], &Stream::new(0), &shape, 0.0, &p);
    let pos = vec2_col(&out, "P");
    assert_eq!(
        pos.len(),
        24,
        "a contagem tem de vir da porta, nao de rows×cols"
    );
    // A forma foi re-centrada e pousada na âncora: cada partícula está onde o
    // disco a punha, menos o centro do disco, mais a âncora.
    for (i, q) in shape.iter().enumerate() {
        let want = [q[0] - 7.0 + 1.0, q[1] + 3.0 + 5.0];
        assert!(
            (pos[i][0] - want[0]).abs() < 1e-4 && (pos[i][1] - want[1]).abs() < 1e-4,
            "particula {i}: {:?} contra {want:?}",
            pos[i]
        );
    }
}

/// ⭐ **A costura inteira, medida pela porta do produto:** entregar a MALHA
/// AUTORADA pela porta tem de simular como o corpo autorado — mesma queda, mesmo
/// pino, mesmo jiggle. É este gate que diz que a generalização é uma reescrita e
/// não uma segunda física.
#[test]
fn the_authored_grid_through_the_port_behaves_like_the_authored_body() {
    for &(rows, cols) in &[(3usize, 5usize), (8, 4), (6, 6)] {
        let p = params(rows, cols, 9.0, 0.5, true);
        let mesh = crate::shape::rest_shape(rows, cols, p.spacing);
        let (mut a, mut b) = (Stream::new(0), Stream::new(0));
        let mut worst = 0.0f32;
        for k in 0..90 {
            let t = k as f32 / 60.0;
            let anchor = [(t * 3.0).sin(), 0.0];
            a = simulate(anchor, &a, &[], t, &p);
            b = simulate(anchor, &b, &mesh, t, &p);
            let (pa, pb) = (vec2_col(&a, "P"), vec2_col(&b, "P"));
            assert_eq!(pa.len(), pb.len(), "{rows}x{cols}: contagens diferentes");
            for (x, y) in pa.iter().zip(&pb) {
                worst = worst.max((x[0] - y[0]).abs().max((x[1] - y[1]).abs()));
            }
        }
        // O resíduo é o do centroide somado (ver `layout_tests`), amplificado
        // por 90 tiques de realimentação — e continua três ordens abaixo de um
        // espaçamento.
        assert!(
            worst < 1e-3,
            "{rows}x{cols}: a porta divergiu do corpo autorado em {worst:.6}"
        );
    }
}

/// O pino segue a FORMA: o topo do disco fica na âncora, e o resto cai.
#[test]
fn the_port_pins_the_top_of_the_shape() {
    // ⚠️ **Mole de propósito.** Com `stiffness = 0,5` este disco assenta a 1,1% da
    // própria altura e fica lá — o corpo está certo, mas a fixtura mal contém o
    // fenómeno que o controle quer medir. Uma barra baixada seria a régua a
    // ceder; o que muda aqui é o CORPO, para o que se mede ser grande.
    let p = params(4, 4, 20.0, 0.12, true);
    let shape = disc(32, 2.0, [0.0, 0.0]);
    let top = shape
        .iter()
        .enumerate()
        .max_by(|a, b| a.1[1].partial_cmp(&b.1[1]).unwrap())
        .map(|(i, _)| i)
        .unwrap();
    let mut state = Stream::new(0);
    for k in 0..150 {
        state = simulate([0.0, 0.0], &state, &shape, k as f32 / 60.0, &p);
    }
    let pos = vec2_col(&state, "P");
    let start = simulate([0.0, 0.0], &Stream::new(0), &shape, 0.0, &p);
    let start = vec2_col(&start, "P");
    let moved_top = (pos[top][1] - start[top][1]).abs();
    let bottom = (0..shape.len())
        .min_by(|&a, &b| start[a][1].partial_cmp(&start[b][1]).unwrap())
        .unwrap();
    let fell = (start[bottom][1] - pos[bottom][1]).abs();
    assert!(
        moved_top < 1e-3,
        "o topo do disco escorregou {moved_top:.4}"
    );
    // ⚠️ A barra é uma fracção da PEÇA (o disco mede `2r` de altura), nunca um
    // literal de mundo: a primeira versão pedia `0,05` absoluto e ficou a
    // `0,0445` quando o pino passou a segurar um ARCO em vez de um ponto — o
    // corpo ficou mais bem preso, e o controle leu isso como regressão.
    // Medido com esta fixtura: o fundo cede **4,7%** da altura do disco enquanto
    // o topo não anda um milésimo — a barra fica a 2%, com folga de 2,3×.
    let height = 2.0 * 2.0;
    assert!(
        fell > height * 0.02,
        "CONTROLE: o resto do disco tinha de cair, e andou {fell:.4} numa peca de {height}"
    );
}

/// ⚠️ Uma nuvem vazia (porta desligada, ou um montante que ainda não cozinhou)
/// **cai para a malha autorada** — um corpo que desaparecesse porque o fio de
/// cima ainda não produziu nada seria um corpo que pisca.
#[test]
fn an_empty_port_falls_back_to_the_authored_mesh() {
    let p = params(3, 4, 9.0, 0.5, true);
    for shape in [vec![], vec![[0.0f32, 0.0]], vec![[0.0f32, 0.0], [1.0, 1.0]]] {
        let out = simulate([0.0, 0.0], &Stream::new(0), &shape, 0.0, &p);
        assert_eq!(
            vec2_col(&out, "P").len(),
            12,
            "porta com {} ponto(s): tinha de cair na malha 3×4",
            shape.len()
        );
    }
}

/// E a forma continua a ser um corpo MOLE: um disco entregue pela porta recupera
/// a forma depois de esmagado, que é a razão de o nó existir.
#[test]
fn a_shape_from_the_port_recovers_its_form() {
    let mut p = params(4, 4, 0.0, 0.6, false);
    p.clusters = 1;
    let shape = disc(40, 2.0, [0.0, 0.0]);
    let mut state = simulate([0.0, 0.0], &Stream::new(0), &shape, 0.0, &p);
    // Esmaga a meio no eixo y, mantendo o estado (a velocidade fica zero).
    let squashed: Vec<[f32; 2]> = vec2_col(&state, "P")
        .iter()
        .map(|q| [q[0], q[1] * 0.4])
        .collect();
    state = state.with("P", Column::Vec2(squashed));
    let width = |s: &Stream| {
        let pos = vec2_col(s, "P");
        let (lo, hi) = pos
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), q| (l.min(q[1]), h.max(q[1])));
        hi - lo
    };
    let before = width(&state);
    for k in 1..120 {
        state = simulate([0.0, 0.0], &state, &shape, k as f32 / 60.0, &p);
    }
    let after = width(&state);
    assert!(
        after > before * 1.8,
        "o disco tinha de voltar a abrir: {before:.3} -> {after:.3}"
    );
}
