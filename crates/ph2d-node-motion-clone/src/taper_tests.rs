//! Os gates do **taper** da fila ([`super::SCALE_TAPER`]) — doc 89, folha 08.
//!
//! ⚠️ **O oráculo separa DUAS leis, e é isso que ele existe para fazer.** Um gate que só
//! afirmasse *"a última cópia é menor"* passaria tanto para o lerp da referência quanto para a
//! potência composta que o título da célula sugeria — as duas encolhem. O que se mede é o
//! valor **do meio**, onde as duas discordam: com `scale_taper = 0,5` e 5 cópias, o lerp dá
//! `0,75` na cópia 2 e a potência daria `0,25`.

use super::*;

/// Uma entrada de duas peças, com as colunas que se quiser.
fn input(cols: Vec<(&str, Column)>) -> Stream {
    let mut s = Stream::new(2);
    for (n, c) in cols {
        s.set(n, c);
    }
    s
}

/// Duas posições quaisquer — o taper não as toca, mas o stream precisa de `P`.
fn ps() -> Column {
    Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0]])
}

fn sizes(s: &Stream) -> Option<Vec<[f32; 2]>> {
    match s.get("size") {
        Some(Column::Vec2(v)) => Some(v.clone()),
        _ => None,
    }
}

fn rots(s: &Stream) -> Option<Vec<f32>> {
    match s.get("rot") {
        Some(Column::Scalar(v)) => Some(v.clone()),
        _ => None,
    }
}

/// O valor da cópia `c` — as peças de uma cópia são contíguas, duas a duas.
fn per_copy<T: Copy>(v: &[T], k: usize) -> Vec<T> {
    (0..k).map(|c| v[c * 2]).collect()
}

/// **O TAPER É UM LERP DA 1ª À ÚLTIMA, e o meio é onde ele se distingue da potência.**
#[test]
fn the_taper_lerps_and_the_middle_copy_is_what_tells_the_two_laws_apart() {
    let out = clone_row(&input(vec![("P", ps())]), 5, 3.0, 0.0, false, 0.5, 0.0);
    let s = sizes(&out).expect("o taper cunha o `size` que faltava");
    let got = per_copy(&s, 5).iter().map(|x| x[0]).collect::<Vec<_>>();
    assert_eq!(
        got,
        vec![1.0, 0.875, 0.75, 0.625, 0.5],
        "lerp de 1 até 0,5 em 5 cópias"
    );
    // ⚠️ O CONTROLE: a lei composta daria 0,25 na cópia do meio, e 0,0625 na última.
    assert!(
        (got[2] - 0.25).abs() > 0.4,
        "a cópia do meio não pode ser a da potência composta"
    );
    // E os dois eixos andam juntos (o taper é uniforme).
    assert!(s.iter().all(|q| (q[0] - q[1]).abs() < 1e-7), "uniforme");
}

/// **A ROTAÇÃO SOMA GRAUS, e a última cópia leva o número inteiro que o artista digitou.**
#[test]
fn the_rotation_taper_hands_the_whole_angle_to_the_last_copy() {
    let out = clone_row(&input(vec![("P", ps())]), 4, 3.0, 0.0, false, 1.0, 90.0);
    let r = rots(&out).expect("o taper cunha o `rot` que faltava");
    assert_eq!(per_copy(&r, 4), vec![0.0, 30.0, 60.0, 90.0]);
}

/// **ELE MULTIPLICA/SOMA AO QUE A PEÇA JÁ TEM** — modula a fonte, nunca a substitui.
///
/// ⚠️ A distinção tem consequência: um `Set` apagaria o `size` que um scatter autorou, e o
/// artista perderia a variação que o trouxe até aqui.
#[test]
fn the_taper_modulates_what_the_piece_already_carries() {
    let inp = input(vec![
        ("P", ps()),
        ("size", Column::Vec2(vec![[2.0, 2.0], [4.0, 4.0]])),
        ("rot", Column::Scalar(vec![10.0, 20.0])),
    ]);
    let out = clone_row(&inp, 3, 3.0, 0.0, false, 0.0, 180.0);
    let s = sizes(&out).expect("size");
    let r = rots(&out).expect("rot");
    // Fatores 1 · 0,5 · 0 — e a primeira peça de cada cópia tinha 2,0.
    assert_eq!(
        per_copy(&s, 3).iter().map(|q| q[0]).collect::<Vec<_>>(),
        vec![2.0, 1.0, 0.0]
    );
    // …e a SEGUNDA peça tinha 4,0, então o mesmo fator dá o dobro.
    assert_eq!(s[1][0], 4.0, "a variação autorada sobrevive");
    assert_eq!(s[3][0], 2.0, "…e é escalada, não substituída");
    // Graus 0 · 90 · 180 SOMADOS aos 10/20 que já lá estavam.
    assert_eq!(per_copy(&r, 3), vec![10.0, 100.0, 190.0]);
    assert_eq!(r[1], 20.0, "a rotação autorada da 2ª peça também soma");
}

/// **NO DEFAULT NENHUMA DAS DUAS COLUNAS É TOCADA** — nem cunhada.
///
/// ⚠️ As duas metades importam. Se ele cunhasse, todo grafo já autorado passaria a carregar
/// duas colunas novas rio abaixo (que viajam, são serializadas e mudam o que um nó a jusante
/// vê); se ele NÃO cunhasse com o knob ligado, o taper seria um botão morto no caso mais comum
/// de todos — uma grelha, que não traz `size` nem `rot`.
#[test]
fn the_default_neither_touches_nor_mints() {
    let out = clone_row(&input(vec![("P", ps())]), 4, 3.0, 0.0, false, 1.0, 0.0);
    assert!(sizes(&out).is_none(), "não cunha `size` no literal");
    assert!(rots(&out).is_none(), "nem `rot`");
    // E o controle: com o knob ligado ele cunha.
    let on = clone_row(&input(vec![("P", ps())]), 4, 3.0, 0.0, false, 0.5, 5.0);
    assert!(sizes(&on).is_some() && rots(&on).is_some(), "ligado, cunha");
}

/// **O `center` NÃO INVERTE O TAPER** — ele muda onde a fila fica, não por onde ela afunila.
#[test]
fn centring_the_queue_does_not_reverse_the_taper() {
    let inp = input(vec![("P", ps())]);
    let off = clone_row(&inp, 5, 3.0, 0.0, false, 0.5, 0.0);
    let on = clone_row(&inp, 5, 3.0, 0.0, true, 0.5, 0.0);
    assert_eq!(
        sizes(&off).expect("size"),
        sizes(&on).expect("size"),
        "os dois controles são ortogonais: o taper corre pela ordinal nos dois casos"
    );
    // E o controle: o `center` de facto mudou alguma coisa (as POSIÇÕES).
    let px = |s: &Stream| match s.get("P") {
        Some(Column::Vec2(v)) => v.iter().map(|q| q[0]).collect::<Vec<_>>(),
        _ => Vec::new(),
    };
    assert_ne!(px(&off), px(&on), "o center tinha de mover a fila");
}

/// **UMA CÓPIA SÓ É A PRIMEIRA CÓPIA** — `t = 0`, e nenhuma divisão por `k − 1 = 0`.
#[test]
fn a_queue_of_one_is_the_head_of_the_taper() {
    let out = clone_row(&input(vec![("P", ps())]), 1, 3.0, 0.0, false, 0.25, 90.0);
    let s = sizes(&out).expect("size");
    assert!(
        s.iter().all(|q| q[0] == 1.0 && q[1] == 1.0),
        "a única cópia é a PRIMEIRA, e a primeira vale 1: {s:?}"
    );
    assert!(
        rots(&out).expect("rot").iter().all(|r| *r == 0.0),
        "e não gira"
    );
}

/// **OS DOIS KNOBS SÃO ALCANÇÁVEIS NO PAINEL** — um param sem `ParamUiHint` existe no
/// cozimento e não existe para o artista.
#[test]
fn both_knobs_are_painted() {
    for p in [SCALE_TAPER, ROT_TAPER] {
        let hint = PARAM_HINTS
            .iter()
            .find(|h| h.param == p)
            .unwrap_or_else(|| panic!("{p} tem de estar pintado"));
        assert!(hint.max > hint.min, "{p}: faixa vazia");
        // O literal tem de CABER na faixa, senão o painel não consegue voltar ao default.
        let default = MANIFEST
            .params
            .iter()
            .find(|s| s.name == p)
            .expect("o param existe")
            .default;
        assert!(
            default >= hint.min && default <= hint.max,
            "{p}: o default {default} está fora do curso [{}, {}]",
            hint.min,
            hint.max
        );
    }
}
