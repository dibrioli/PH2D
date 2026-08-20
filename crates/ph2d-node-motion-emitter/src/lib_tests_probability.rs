//! Os gates do [`super::LANE_PROB`] — a probabilidade de nascer (doc 89, folha 01).
//!
//! ⚠️ **O gate que justifica o param existir é o do CINTILAR**, não o que conta partículas. A
//! rota composta (`field.remap(probability) → motion.cull`) também rala uma lista; o que ela
//! não faz é rala-la de forma ESTÁVEL num emitter, porque hasheia o índice e a janela viva
//! desliza. Um gate que só medisse *"saíram menos"* passaria para as duas.

use super::*;

/// O `Spec` da suíte, com a probabilidade pedida.
fn spec_p(p: f32) -> Spec {
    let mut s = super::tests::spec();
    s.probability = p;
    s
}

/// Os `id` de cada partícula viva em `t`.
fn ids_at(p: f32, t: f32) -> Vec<u32> {
    match emit(&spec_p(p), t).get("id") {
        #[expect(clippy::cast_possible_truncation, reason = "ids inteiros num f32")]
        #[expect(clippy::cast_sign_loss, reason = "um id nunca é negativo")]
        Some(Column::Scalar(v)) => v.iter().map(|x| *x as u32).collect(),
        _ => Vec::new(),
    }
}

/// **`1` MANTÉM TODA A GENTE, E O STREAM É O QUE SEMPRE FOI.**
///
/// ⚠️ A comparação é feita coluna a coluna contra o mesmo cozimento — não contra números
/// escritos à mão: o que se afirma é que o caminho novo **não é tomado**, não que eu sei
/// reproduzir a saída.
#[test]
fn one_keeps_everyone_and_the_stream_is_untouched() {
    let t = 4.5;
    let (a, b) = (emit(&spec_p(1.0), t), emit(&spec_p(1.0), t));
    // A janela desta fixture é `rate 10 × life 1` ⇒ **11** vivas — o limiar é contra o número
    // medido, não contra um redondo que eu escolhi.
    assert!(
        a.count() >= 10,
        "a fixture tem de ter partículas: {}",
        a.count()
    );
    for name in ["id", "age", "Index", "Count"] {
        assert_eq!(
            format!("{:?}", a.get(name)),
            format!("{:?}", b.get(name)),
            "a coluna {name} tem de ser determinística"
        );
    }
    // E o essencial: com `1` ninguém é recusado, então a contagem é a da janela inteira.
    let w = window(spec_p(1.0).spawn, spec_p(1.0).life, spec_p(1.0).max, t);
    assert_eq!(a.count(), w.count, "a janela inteira sobrevive");
}

/// **O PORTÃO RALA, E OS SOBREVIVENTES SÃO UM SUBCONJUNTO** — nunca partículas outras.
#[test]
fn the_gate_thins_the_stream_into_a_subset_of_the_same_particles() {
    let t = 4.5;
    let all = ids_at(1.0, t);
    let half = ids_at(0.5, t);
    assert!(
        half.len() < all.len(),
        "0,5 tem de deixar cair alguém: {} de {}",
        half.len(),
        all.len()
    );
    assert!(!half.is_empty(), "…e não pode levar toda a gente");
    for id in &half {
        assert!(all.contains(id), "o id {id} não estava na janela");
    }
    // A ORDEM (mais velha primeiro) sobrevive ao portão.
    let mut sorted = half.clone();
    sorted.sort_unstable();
    assert_eq!(
        half, sorted,
        "os sobreviventes saem na ordem em que nasceram"
    );
}

/// **UMA PARTÍCULA QUE NASCEU FICA NASCIDA ENQUANTO A JANELA DESLIZA** — o gate que separa
/// esta lei da rota composta.
///
/// ⚠️ O oráculo tem duas metades. A primeira: a janela de facto DESLIZOU (senão o teste não
/// mede nada). A segunda: todo id vivo nos dois instantes tem a MESMA resposta nos dois. Um
/// sorteio pelo índice — que é o do `field.remap` — reprova aqui, porque o índice de uma
/// partícula muda quando a mais velha morre.
#[test]
fn a_particle_that_is_born_stays_born_while_the_window_slides() {
    let p = 0.45;
    let (t1, t2) = (4.0, 4.35);
    let (a, b) = (ids_at(p, t1), ids_at(p, t2));
    let all1: Vec<u32> = ids_at(1.0, t1);
    let all2: Vec<u32> = ids_at(1.0, t2);
    assert_ne!(
        all1, all2,
        "a janela TEM de ter deslizado entre os dois instantes"
    );
    let overlap: Vec<u32> = all1.iter().copied().filter(|k| all2.contains(k)).collect();
    assert!(
        overlap.len() > 5,
        "e tem de haver sobreposição: {}",
        overlap.len()
    );
    for id in overlap {
        assert_eq!(
            a.contains(&id),
            b.contains(&id),
            "o id {id} mudou de resposta entre {t1} e {t2} — isto é o cintilar"
        );
    }
}

/// **O SORTEIO NÃO REMEXE OS OUTROS** — quem sobrevive lança exactamente como lançava.
///
/// ⚠️ **Este gate NÃO SANGRA contra o código de hoje, e fica com a razão escrita em vez de ser
/// apagado.** Nenhuma mutação realista do `emit` o faz vermelho: `rand01(seed, id, lane)` é uma
/// FUNÇÃO da identidade, não uma sequência, então não há estado que uma recusa possa consumir —
/// a propriedade é verdadeira por construção. O que ele defende é uma CLASSE: o dia em que
/// alguém trocar as extracções por um gerador sequencial (o `rng.next()` que toda referência
/// usa), ligar um knob de densidade passará a mudar a direção e a velocidade de TODAS as
/// partículas que ficam, e a chuva inteira saltará. É o mesmo precedente do caminho literal de
/// `0°` no `motion.sort`: documentado, não escondido.
#[test]
fn the_draw_disturbs_nothing_else_about_the_survivors() {
    let t = 4.5;
    let full = emit(&spec_p(1.0), t);
    let thin = emit(&spec_p(0.5), t);
    let col = |s: &Stream, n: &str| match s.get(n) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    };
    let (fid, tid) = (ids_at(1.0, t), ids_at(0.5, t));
    let (fv, tv) = (col(&full, "vel"), col(&thin, "vel"));
    let (fs, ts) = (col(&full, "size"), col(&thin, "size"));
    let mut checked = 0;
    for (j, id) in tid.iter().enumerate() {
        let i = fid
            .iter()
            .position(|k| k == id)
            .expect("está na janela cheia");
        assert_eq!(tv[j], fv[i], "a velocidade do id {id} mudou");
        assert_eq!(ts[j], fs[i], "o tamanho do id {id} mudou");
        checked += 1;
    }
    assert!(
        checked >= 3,
        "poucos sobreviventes para afirmar isto: {checked}"
    );
}

/// **`Index`/`Count` DESCREVEM OS SOBREVIVENTES**, não as candidatas.
///
/// ⚠️ O `motion.tint` em gradiente divide por `Count − 1`: uma contagem que incluísse as
/// recusadas faria o degradê parar antes do fim, e o defeito leria como *"a cor está errada"*.
#[test]
fn index_and_count_describe_the_survivors() {
    let s = emit(&spec_p(0.5), 4.5);
    let n = s.count();
    match (s.get("Index"), s.get("Count")) {
        (Some(Column::Scalar(i)), Some(Column::Scalar(c))) => {
            #[expect(clippy::cast_precision_loss, reason = "uma contagem pequena")]
            let want: Vec<f32> = (0..n).map(|k| k as f32).collect();
            assert_eq!(*i, want, "o Index corre 0..n−1 sobre quem ficou");
            #[expect(clippy::cast_precision_loss, reason = "uma contagem pequena")]
            let total = n as f32;
            assert!(
                c.iter().all(|x| *x == total),
                "o Count é o dos sobreviventes"
            );
        }
        other => panic!("Index/Count: {other:?}"),
    }
}

/// **`0` É UMA RESPOSTA: nenhuma nasce, e nada explode.**
#[test]
fn zero_is_an_empty_stream_and_not_a_panic() {
    let s = emit(&spec_p(0.0), 4.5);
    assert_eq!(s.count(), 0);
    assert!(ids_at(0.0, 4.5).is_empty());
}

/// **O DEVICE É RECUSADO SÓ QUANDO O PORTÃO MORDE** — as duas metades.
#[test]
fn the_device_is_refused_only_when_the_gate_bites() {
    let app = crate::gpu::GPU_KERNEL.applicable.expect("a recusa existe");
    assert!(
        app(&|n| if n == "probability" { 1.0 } else { 0.0 }),
        "em 1 nada recua"
    );
    assert!(
        !app(&|n| if n == "probability" { 0.5 } else { 0.0 }),
        "abaixo de 1 o device tem de sair — a contagem passa a depender de DADOS"
    );
}
