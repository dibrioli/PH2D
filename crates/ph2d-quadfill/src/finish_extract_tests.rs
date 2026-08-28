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
        for k in 0..3 {
            pos[top][k] *= 3.0;
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

/// ⭐⭐⭐ **A COMPARAÇÃO É DE PARETO, e testa-se sem malha nenhuma.**
///
/// ⛔ **Ela não é gateável pela fixtura, e a prova de mutação disse-o** (2026-08-28): trocar
/// a lei por «só a mediana» deixava os gates de malha verdes, porque naquelas peças as duas
/// concordam. *Uma lei que a fixtura não separa testa-se onde ela é declarada.*
#[test]
fn a_round_that_trades_one_column_for_another_is_refused() {
    let base = crate::shape::QuadShape {
        skew_over_60: 2,
        skew_p50: 8.0,
        aspect_p50: 1.20,
        ..crate::shape::QuadShape::default()
    };
    let mut win = base;
    win.skew_p50 = 4.0;
    assert!(
        super::better(&win, &base),
        "melhor na mediana e igual no resto tem de ganhar"
    );
    // ⛔ Troca: ganha muito na mediana e perde UMA face péssima.
    let mut trade = base;
    trade.skew_p50 = 0.5;
    trade.skew_over_60 = 3;
    assert!(
        !super::better(&trade, &base),
        "uma ronda que compra mediana com uma face pessima tem de ser recusada"
    );
    // ⛔ E o mesmo do outro lado: ganha faces péssimas e perde aspecto.
    let mut trade2 = base;
    trade2.skew_over_60 = 0;
    trade2.aspect_p50 = 1.40;
    assert!(
        !super::better(&trade2, &base),
        "uma ronda que compra faces pessimas com aspecto tem de ser recusada"
    );
    // ⭐ E o empate dentro do ruído não é melhoria — senão a corrida nunca desiste.
    let mut noise = base;
    noise.skew_p50 = 8.0 - super::SAME * 0.5;
    assert!(
        !super::better(&noise, &base),
        "uma diferenca abaixo do ruido nao e' melhoria"
    );
}

/// ⭐⭐⭐ **O ALINHAMENTO AO RELEVO PAGA-SE, e a fixtura é um TORO.**
///
/// ⚠️ **Uma esfera não pode medir isto:** as duas curvaturas dela são iguais, a anisotropia
/// é `0` e a lei degenera no quadrado puro — foi exactamente por isso que uma mutação que
/// punha o `pull` a zero sobreviveu à 1.ª redacção destes gates. Um toro tem as duas
/// curvaturas muito diferentes e a grade dele **nasce alinhada**, então o relevo só pode
/// piorar: é a forma em que a diferença entre as duas leis é máxima.
#[test]
fn the_relief_pull_keeps_the_grid_on_the_form() {
    let quads = ph2d_mesh::shapes::torus(32, 16, 1.0, 0.35);
    let surface = {
        let mut s = quads.clone();
        s.triangulate();
        s
    };
    let relief = |m: &Mesh| crate::quality::follows_relief(&surface, m).0;
    let (mut blind, mut aligned) = (quads.clone(), quads);
    super::finish_extracted_with(&mut blind, &surface, 0.0, super::EXTRACT_SETTLE);
    // ⚠️ **O lado alinhado passa pela PORTA DO PRODUTO**, não pela forma aberta: uma
    // mutação que pusesse o `pull` da porta a zero **sobreviveu** enquanto este gate a
    // contornava (medido, 2026-08-28). *Gatear a lei não é gatear quem a usa.*
    super::finish_extracted(&mut aligned, &surface);
    let (rb, ra) = (relief(&blind), relief(&aligned));
    assert!(
        ra < rb,
        "o alinhamento devia guardar o relevo e mediu {ra:.2}° contra os {rb:.2}° da lei cega"
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

/// ⭐⭐⭐ **A PACIÊNCIA CONTA DA MELHOR RONDA, NÃO DO INÍCIO** — a lei, sem malha nenhuma.
///
/// ⛔ **Uma fixtura de malha não a separa de forma robusta, e foi medido**: para distinguir
/// as duas leituras é preciso uma peça cuja última melhoria caia **depois** da janela, e
/// isso depende do abanão que se lhe dá (num toro sacudido a `0,12` a melhor ronda foi a
/// `124`, a `0,20` foi a `10`). *Perseguir uma fixtura que passe raspando seria afinar a
/// fixtura até o gate passar* — a lei testa-se onde ela é declarada.
#[test]
fn the_patience_window_starts_at_the_best_round() {
    let p = super::EXTRACT_PATIENCE;
    // Acabou de melhorar: nunca desiste, por mais tarde que seja a ronda.
    assert!(!super::give_up(10_000, 10_001), "desistiu logo apos uma melhoria");
    // Uma janela inteira sem melhoria desde a ronda ZERO: desiste.
    assert!(
        super::give_up(p - 1, 0),
        "nao desistiu depois de {p} rondas sem bater a ronda zero"
    );
    assert!(!super::give_up(p - 2, 0), "desistiu antes da janela fechar");
    // ⭐ E o caso que separa as duas leituras: ronda tardia, melhoria recente.
    assert!(
        !super::give_up(p * 5, p * 5 - 2),
        "um contador que arrancasse do INICIO teria desistido aqui -- a janela conta da \
         melhor ronda"
    );
    assert!(
        super::give_up(p * 5, p * 4),
        "uma janela inteira sem melhoria tem de desistir, por tardia que seja"
    );
}
