//! Os gates do domínio — a caixa e a densidade que os dois amostradores partilham.
//!
//! ⛔ **Sete gates saíram daqui em 2026-09-19** (o sorteio no disco · o buraco do anel · o corte ·
//! a casca · as duas bordas · a elipse · a tabela de rótulos): eles mediam o param `Shape`, que o
//! dono mandou retirar dos quatro cartões. *Um gate cujo sujeito deixou de existir não afirma
//! nada* — ele não fica a passar por vácuo, sai.

use super::*;

/// Um par de uniformes decorrelacionados, sem depender do hash de nó nenhum.
fn draws(n: usize) -> impl Iterator<Item = (f32, f32)> {
    (0..n).map(|i| {
        let u = ((i as u32).wrapping_mul(2_654_435_761) >> 8) as f32 / (1u32 << 24) as f32;
        let v = ((i as u32).wrapping_mul(40_503) >> 8) as f32 / (1u32 << 24) as f32;
        (u, v)
    })
}

/// ⭐ **O SORTEIO É O DE SEMPRE, AO BIT** — a expressão que os nós tinham escrita à mão, byte a
/// byte.
///
/// ⚠️ **Ele sobreviveu à retirada do `Shape` e é isso que ele afirma agora:** o caminho que toda
/// cena salva percorre é o mesmo, com um ramo a menos por baixo. Se a colapsagem tivesse movido um
/// ULP, toda cena com um `motion.scatter` mudaria de layout.
#[test]
fn the_rect_draw_is_bit_for_bit_the_hand_written_one() {
    let (w, h) = (4.0_f32, 2.5_f32);
    let d = Region::rect(w, h);
    for (u, v) in draws(512) {
        let got = d.sample(u, v);
        let want = [(u - 0.5) * w, (v - 0.5) * h];
        assert_eq!(
            got.map(f32::to_bits),
            want.map(f32::to_bits),
            "u={u} v={v}: {got:?} contra {want:?}"
        );
    }
}

/// E a caixa aceita-se inteira — um reticulado não perde um ponto.
#[test]
fn the_rect_cut_removes_nothing() {
    let d = Region::rect(4.0, 4.0);
    for r in 0..9 {
        for c in 0..9 {
            let p = [(c as f32 - 4.0) * 0.5, (r as f32 - 4.0) * 0.5];
            assert!(d.contains(p), "{p:?} caiu fora da propria caixa");
        }
    }
}

/// ⚠️ **A régua é CHEBYSHEV normalizada por eixo** — `1` nas quatro arestas de uma caixa não
/// quadrada, e a esquina fica fora só por um triz (`radial` lá vale exactamente `1`).
///
/// FALSIFICADO por trocar o `max` por uma hipotenusa: a aresta curta passaria a ler `< 1`.
#[test]
fn the_ruler_is_one_on_every_edge_of_a_wide_box() {
    let d = Region::rect(8.0, 2.0);
    assert!((d.radial([4.0, 0.0]) - 1.0).abs() < 1e-5, "a aresta x");
    assert!((d.radial([0.0, 1.0]) - 1.0).abs() < 1e-5, "a aresta y");
    assert!(d.contains([4.0, 1.0]), "a esquina da caixa e' da caixa");
    assert!(!d.contains([4.1, 0.0]), "um passo para fora sai");
}

/// ⚠️ **`falloff = 0` devolve `1,0` AO BIT** — o default reduz ao nó que shipava.
#[test]
fn a_zero_falloff_is_exactly_one_everywhere() {
    let d = Region::rect(3.0, 5.0);
    for (u, v) in draws(1_000) {
        let p = d.sample(u, v);
        assert_eq!(d.density(p, 0.0).to_bits(), 1.0_f32.to_bits());
        // E fora da região também — um consumidor pode perguntar em qualquer sítio.
        assert_eq!(
            d.density([u * 9.0, v * 9.0], 0.0).to_bits(),
            1.0_f32.to_bits()
        );
    }
}

/// E com `falloff = 1` a densidade desce do coração até ao piso, sem o furar.
#[test]
fn a_full_falloff_grades_from_the_core_to_the_floor() {
    let d = Region::rect(4.0, 4.0);
    assert!(
        (d.density([0.0, 0.0], 1.0) - 1.0).abs() < 1e-6,
        "o coracao vale 1"
    );
    let edge = d.density([2.0, 0.0], 1.0);
    assert!(
        (edge - MIN_DENSITY).abs() < 1e-5,
        "a borda tinha de encostar no piso: {edge}"
    );
    // Monotónica pelo caminho, e nunca abaixo do piso.
    let mut prev = f32::MAX;
    for k in 0..=40 {
        let v = d.density([k as f32 * 0.05, 0.0], 1.0);
        assert!(v >= MIN_DENSITY - 1e-6, "furou o piso em {k}: {v}");
        assert!(v <= prev + 1e-6, "subiu ao afastar-se em {k}: {v} > {prev}");
        prev = v;
    }
}

/// ⚠️ **Extensões doentes não entram em pânico** — elas chegam de um fio.
#[test]
fn a_driven_extent_can_be_anything_and_the_answer_is_finite() {
    for bad in [f32::NAN, f32::INFINITY, -7.0] {
        let d = Region::rect(bad, 4.0);
        let p = d.sample(0.3, 0.7);
        assert!(p[0].is_finite() && p[1].is_finite(), "bad={bad} deu {p:?}");
        assert!(d.density(p, 0.5).is_finite(), "bad={bad}");
    }
    // Extensão zero: a caixa não tem interior, e a resposta é a linha — não o vazio.
    let flat = Region::rect(4.0, 0.0);
    assert!(
        flat.contains([1.0, 0.0]),
        "uma caixa achatada ainda tem a linha"
    );
}

/// ⭐ **A caixa que o consumidor indexa é a mesma que a régua mede** — a grelha de vizinhança do
/// `motion.scatter` lê `half_extents` e o sorteio lê os mesmos dois números.
///
/// FALSIFICADO por `half_extents` devolver a extensão INTEIRA: a grelha passaria a cobrir o dobro
/// do espaço e a busca do vizinho mais próximo mediria células vazias.
#[test]
fn the_box_the_consumer_indexes_is_the_box_the_ruler_measures() {
    let d = Region::rect(6.0, 2.0);
    let [hw, hh] = d.half_extents();
    assert_eq!((hw, hh), (3.0, 1.0));
    assert_eq!(
        d.sample(1.0, 1.0).map(f32::to_bits),
        [hw, hh].map(f32::to_bits)
    );
    assert_eq!(
        d.sample(0.0, 0.0).map(f32::to_bits),
        [-hw, -hh].map(f32::to_bits)
    );
}
