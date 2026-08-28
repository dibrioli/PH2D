//! **OS GATES DO ACABAMENTO** — ver [`super`].
//!
//! ⚠️ **A fixtura tem uma PONTA, e é ela que faz os gates falarem.** Numa esfera lisa a
//! relaxação melhora monotonamente e a última ronda **é** a melhor — um gate «guardar a
//! melhor não é pior» ficaria verde ali mesmo sem a guarda, que é a definição de
//! tautologia. A regressão medida em 2026-08-28 vive na peça com ponta
//! (`sculpt_hooked`), e esta fixtura reproduz o mecanismo dela.

use ph2d_mesh::Mesh;

use super::{EXTRACT_MAX_ROUNDS, finish_extracted};
use crate::shape::quad_shape;

/// ⭐ **UM TORO SACUDIDO** — grade limpa (sem o leque de pólo de uma esfera-UV), curvaturas
/// muito diferentes (anisotropia a sério) e um abanão determinístico que dá à relaxação o
/// que endireitar. *Sem o abanão a malha já é o ponto fixo e nenhuma lei se mexe.*
fn jittered_torus() -> (Mesh, Mesh) {
    let surface = {
        let mut t = ph2d_mesh::shapes::torus(32, 16, 1.0, 0.35);
        t.triangulate();
        t
    };
    let mut quads = ph2d_mesh::shapes::torus(32, 16, 1.0, 0.35);
    {
        // Deslocamento determinístico — nada de RNG num gate.
        let pos = quads.positions_mut();
        for (i, p) in pos.iter_mut().enumerate() {
            #[allow(clippy::cast_precision_loss)]
            let t = (i as f32).mul_add(12.9898, 4.1414).sin() * 43758.547;
            let f = [t.fract(), (t * 1.37).fract(), (t * 2.71).fract()];
            for k in 0..3 {
                p[k] = f[k].mul_add(0.120, p[k] - 0.060);
            }
        }
    }
    quads.rebuild();
    (quads, surface)
}

/// Uma esfera de quads com **um bico**: o pólo puxado para fora, que é a forma em que o
/// pedido de cada face é mais contraditório.
fn spiked() -> (Mesh, Mesh) {
    let mut quads = ph2d_mesh::shapes::uv_sphere(16, 24, 1.0);
    let top = {
        let pos = quads.positions();
        let mut best = (f32::NEG_INFINITY, 0usize);
        for (i, p) in pos.iter().enumerate() {
            if p[1] > best.0 {
                best = (p[1], i);
            }
        }
        best.1
    };
    {
        let pos = quads.positions_mut();
        for c in &mut pos[top] {
            *c *= 3.0;
        }
    }
    quads.rebuild();
    let mut surface = quads.clone();
    surface.triangulate();
    (quads, surface)
}

/// ⭐⭐⭐ **O ACABAMENTO NUNCA ENTREGA PIOR QUE O QUE SHIPAVA.**
///
/// A ronda zero é o Laplaciano de sempre; a saída só muda se alguma ronda for
/// **estritamente** melhor na ordem declarada. ⚠️ *Isto não é uma promessa sobre o
/// algoritmo — é a razão de a saída ser a melhor ronda e não a última.*
#[test]
fn the_finish_never_ships_worse_than_the_round_it_starts_from() {
    let (mut quads, surface) = spiked();
    let rep = finish_extracted(&mut quads, &surface);
    assert!(
        (rep.after.skew_over_60, rep.after.skew_p50)
            <= (rep.before.skew_over_60, rep.before.skew_p50),
        "o acabamento piorou: {} faces pessimas e {:.2}° de mediana contra {} e {:.2}°",
        rep.after.skew_over_60,
        rep.after.skew_p50,
        rep.before.skew_over_60,
        rep.before.skew_p50
    );
    // ⚠️ **E o relatório tem de descrever a malha que saiu**, não uma que ficou pelo
    // caminho: sem isto, um `best_pos` que não fosse reposto passaria no gate acima.
    let out = quad_shape(&quads);
    assert!(
        (out.skew_over_60, out.skew_p50) == (rep.after.skew_over_60, rep.after.skew_p50),
        "o relatorio diz {} / {:.3}° e a malha entregue mede {} / {:.3}°",
        rep.after.skew_over_60,
        rep.after.skew_p50,
        out.skew_over_60,
        out.skew_p50
    );
}

/// ⭐⭐ **A RELAXAÇÃO DEGRADA-SE DE FACTO NESTA FIXTURA** — o controle que impede o gate
/// acima de ser uma tautologia.
///
/// ⛔ **Sem ele, «guardar a melhor ronda» seria uma afirmação sobre nada:** numa peça em
/// que a última ronda é sempre a melhor, a guarda pode ser removida e todos os gates ficam
/// verdes. Este mede o que a guarda evita — *a última ronda é pior que a melhor.*
#[test]
fn the_last_round_is_worse_than_the_best_one_here() {
    let (mut quads, surface) = spiked();
    let rep = finish_extracted(&mut quads, &surface);
    assert!(rep.rounds > 1, "a relaxacao correu {} rondas", rep.rounds);
    assert!(
        rep.kept < rep.rounds,
        "a melhor ronda foi a ULTIMA ({} de {}) -- esta fixtura nao contem a regressao \
         que a guarda existe para evitar, e o gate irmao e' uma tautologia",
        rep.kept,
        rep.rounds
    );
}

/// ⭐ **A TOPOLOGIA NÃO SE MEXE** — uma relaxação move vértices e mais nada.
#[test]
fn the_finish_moves_vertices_and_nothing_else() {
    let (mut quads, surface) = spiked();
    let (v, f) = (quads.vert_count(), quads.face_count());
    let rep = finish_extracted(&mut quads, &surface);
    assert_eq!(quads.vert_count(), v, "o acabamento mudou a contagem de vertices");
    assert_eq!(quads.face_count(), f, "o acabamento mudou a contagem de faces");
    assert!(
        rep.rounds <= EXTRACT_MAX_ROUNDS,
        "correu {} rondas, mais que a rede de {EXTRACT_MAX_ROUNDS}",
        rep.rounds
    );
}

/// ⭐⭐⭐ **A ACEITAÇÃO É CONTRA A RONDA ZERO, e a escolha é a mediana** — as duas leis,
/// sem malha nenhuma.
///
/// ⛔ **Elas não são gateáveis pela fixtura, e a prova de mutação disse-o** (2026-08-28):
/// nas peças de teste as várias ordens concordam. *Uma lei que a fixtura não separa
/// testa-se onde ela é declarada.*
#[test]
fn a_round_that_worsens_any_column_against_round_zero_is_refused() {
    let base = crate::shape::QuadShape {
        skew_over_60: 2,
        skew_p50: 8.0,
        skew_p99: 40.0,
        aspect_p50: 1.20,
        aspect_p99: 1.90,
        ..crate::shape::QuadShape::default()
    };
    let mut win = base;
    win.skew_p50 = 4.0;
    assert!(super::acceptable(&win, &base), "melhor na mediana tem de ser aceite");
    // ⛔ Compra mediana com uma face péssima a mais.
    let mut trade = base;
    trade.skew_p50 = 0.5;
    trade.skew_over_60 = 3;
    assert!(
        !super::acceptable(&trade, &base),
        "uma ronda que ganha uma face pessima tem de ser recusada"
    );
    // ⛔ E o mesmo do outro lado: ganha faces péssimas e perde aspecto.
    let mut trade2 = base;
    trade2.skew_over_60 = 0;
    trade2.aspect_p50 = 1.40;
    assert!(
        !super::acceptable(&trade2, &base),
        "uma ronda que compra faces pessimas com aspecto tem de ser recusada"
    );
    // ⛔ E a CAUDA conta: comprar mediana com `p99` é a troca que a `sculpt_eared` fina
    // fazia antes desta coluna entrar (`27,2° → 28,2°`).
    let mut trade3 = base;
    trade3.skew_p50 = 1.0;
    trade3.skew_p99 = base.skew_p99 + 1.0;
    assert!(
        !super::acceptable(&trade3, &base),
        "uma ronda que compra mediana com a CAUDA tem de ser recusada"
    );
    let mut trade4 = base;
    trade4.skew_p50 = 1.0;
    trade4.aspect_p99 = base.aspect_p99 + 0.1;
    assert!(
        !super::acceptable(&trade4, &base),
        "uma ronda que compra mediana com a cauda do ASPECTO tem de ser recusada"
    );
    // ⭐ E a escolha ENTRE aceitáveis é a mediana, com o aspecto a desempatar.
    let mut a = base;
    a.skew_p50 = 3.0;
    let mut b = base;
    b.skew_p50 = 4.0;
    assert!(super::better(&a, &b), "a mediana menor tem de ganhar");
    let mut c = base;
    c.skew_p50 = 3.0;
    c.aspect_p50 = 1.05;
    assert!(super::better(&c, &a), "empatada a mediana, decide o aspecto");
    // ⭐ E o empate dentro do ruído não é melhoria — senão a corrida nunca desiste.
    let mut noise = base;
    noise.skew_p50 = 8.0 - super::SAME * 0.5;
    assert!(!super::better(&noise, &base), "uma diferenca abaixo do ruido nao e' melhoria");
}

/// ⭐⭐ **O ALINHAMENTO AO RELEVO NÃO É INERTE** — a lei muda o resultado.
///
/// ⚠️ **É só isto que uma fixtura sintética pode afirmar, e a tentativa de afirmar mais
/// falhou honestamente** (2026-08-28): num toro a relaxação **cega** restaura a grade
/// perfeita, que *já é* a grade alinhada, então ali o alinhamento só pode acrescentar erro
/// (`1,64°` contra `1,03°`). ⭐ *A afirmação «o alinhamento guarda o relevo» é sobre peças
/// irregulares, onde a relaxação cega DERIVA* — e é uma medição de corpus, com tabela:
/// `sculpt_wrinkled` grossa, relevo `11,9°` (cru) → `18,8°` (cega) → `11,2°` (alinhada).
/// ⛔ Escolher uma fixtura sintética que passasse seria afinar a fixtura até o gate passar.
#[test]
fn the_relief_pull_is_not_inert() {
    let (quads, surface) = jittered_torus();
    let (mut blind, mut aligned) = (quads.clone(), quads);
    super::finish_extracted_with(&mut blind, &surface, 0.0, super::EXTRACT_SETTLE);
    super::finish_extracted_with(
        &mut aligned,
        &surface,
        super::EXTRACT_RELIEF_PULL,
        super::EXTRACT_SETTLE,
    );
    let moved = blind
        .positions()
        .iter()
        .zip(aligned.positions())
        .map(|(a, b)| {
            (a[0] - b[0])
                .hypot(a[1] - b[1])
                .hypot(a[2] - b[2])
        })
        .fold(0.0f32, f32::max);
    assert!(
        moved > 1.0e-4,
        "a lei alinhada e a cega entregaram a MESMA malha (maior desvio {moved:.2e}) -- o \
         alinhamento esta' inerte"
    );
}

/// ⭐⭐⭐ **A LEI CEGA SÓ TEM A VEZ QUANDO A ALINHADA NÃO SE MEXEU** — a lei da escolha.
///
/// ⛔⛔ **Nenhuma fixtura sintética a separa, e as duas foram tentadas** (2026-08-28): na
/// esfera com bico **nenhuma** das leis se mexe (a queda dispara e não acha nada); no toro
/// sacudido **mexem-se as duas** (a queda nunca dispara). *Afinar uma fixtura até ela
/// separar seria escolher a resposta em vez de a medir.*
#[test]
fn the_blind_law_only_gets_its_turn_when_the_aligned_one_did_not_move() {
    let r = |kept: usize| super::FinishReport {
        kept,
        ..super::FinishReport::default()
    };
    assert!(
        super::use_blind(&r(0), &r(7)),
        "a alinhada nao se mexeu e a cega mexeu-se: a cega tem de entrar"
    );
    assert!(
        !super::use_blind(&r(3), &r(7)),
        "a alinhada mexeu-se: a cega NAO pode entrar, senao o relevo perde-se onde ele \
         estava em jogo"
    );
    assert!(
        !super::use_blind(&r(0), &r(0)),
        "nenhuma se mexeu: fica a ronda zero"
    );
}

/// ⭐⭐⭐ **A PACIÊNCIA DESISTE, e não é a rede que a salva.**
///
/// Com `settle_frac = 0` a corrida nunca assenta: o único fim possível é a
/// [`super::EXTRACT_PATIENCE`] ou a rede de [`EXTRACT_MAX_ROUNDS`]. ⛔ **Sem a paciência,
/// medido na `sculpt_hooked` fina, a corrida gastava as `1 200` rondas e `8,3 s` para
/// entregar exactamente a malha com que começou** — a ordem de comparação recusa toda ronda
/// numa peça em que a relaxação sobe as faces péssimas de `1` para `2` logo à primeira.
#[test]
fn the_patience_gives_up_instead_of_spending_the_whole_net() {
    let (mut quads, surface) = spiked();
    let rep = super::finish_extracted_with(&mut quads, &surface, super::EXTRACT_RELIEF_PULL, 0.0);
    assert!(
        rep.rounds < EXTRACT_MAX_ROUNDS,
        "sem assentamento a corrida gastou a rede inteira ({} rondas) -- a paciencia nao \
         esta' a agir",
        rep.rounds
    );
    assert!(
        rep.rounds - rep.kept <= super::EXTRACT_PATIENCE,
        "correu {} rondas depois da melhor ({}), mais que a paciencia de {}",
        rep.rounds - rep.kept,
        rep.kept,
        super::EXTRACT_PATIENCE
    );
}

/// ⭐⭐⭐ **A PACIÊNCIA SÓ CORRE ENQUANTO NADA FOI ACEITE** — a lei, sem malha nenhuma.
///
/// ⛔⛔ **A 1.ª redacção media «rondas desde a MELHOR» e cortava trabalho real** (medido
/// 2026-08-28): na `sculpt_hooked` fina a primeira ronda aceite é a `312` e a melhor é a
/// `830`; com uma janela de `128` **desde a melhor**, a corrida morria à ronda `128` e a
/// peça saía intocada (`1,11 / 6,5° / p99 33,0`), quando ela chega a `1,04 / 2,0° / p99
/// 22,8` com **zero** faces péssimas. ⚠️ *Desistir enquanto nada foi aceite é barato;
/// desistir depois corta trabalho real.*
///
/// ⚠️ **Testa-se aqui e não numa fixtura:** para a separar seria preciso uma peça cuja
/// primeira aceitação caísse dentro da janela e a melhor fora dela, e afinar uma fixtura até
/// ela separar seria escolher a resposta.
#[test]
fn the_patience_only_runs_while_nothing_has_been_accepted() {
    let p = super::EXTRACT_PATIENCE;
    // Nada aceite e a janela ainda aberta: continua.
    assert!(!super::give_up(p - 2, 0), "desistiu antes da janela fechar");
    // Nada aceite e a janela fechou: desiste.
    assert!(
        super::give_up(p - 1, 0),
        "nao desistiu depois de {p} rondas sem aceitar nada"
    );
    // ⭐ O caso que separa as duas leituras: uma aceitação MUITO cedo, e a corrida continua
    // muito depois da janela.
    assert!(
        !super::give_up(p * 9, 1),
        "desistiu com uma ronda ja' aceite -- a paciencia so' corre enquanto nada foi aceite"
    );
    assert!(
        !super::give_up(p * 9, p / 2),
        "desistiu com uma aceitacao dentro da janela"
    );
}
