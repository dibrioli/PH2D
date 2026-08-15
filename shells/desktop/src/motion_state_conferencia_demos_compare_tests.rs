//! Gates da cena `=45` — **a COMPARAÇÃO e o NOME que não resolve**.
//!
//! Eles medem a GEOMETRIA que a cena cozinha, não a intenção com que ela foi
//! escrita: se a mensagem diz *"degraus complementares"*, o cozido tem de os ter, e
//! se diz *"quatro vezes mais larga"*, a razão tem de sair da contagem.
//!
//! ⚠️ **A SILHUETA, nunca o Y cru** — cada fileira carrega o próprio `offset_y`, e
//! comparar Y cru entre duas mede o `BAND_GAP` e não a máscara. É a quarta vez que
//! este repo paga a lição (o `1,15` do grupo B, o `offset_y` do `Range` no C, os
//! gêmeos do D).
//!
//! ⚠️ **O gate do BADGE mora no irmão `motion_bridge_unresolved_tests.rs`**, e não
//! aqui: ele precisa do `inert_reaching_output`, que é privado do `render_loop`.
//! Ele existe — e é o que torna verdadeira a instrução da cena de *abrir o painel
//! de grafo* —, mas o alcance decidiu onde.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// Cozinha a cena e devolve o `Y` de cada fileira, na ordem das bandas.
fn rows() -> Vec<Vec<f32>> {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).unwrap();
    let mut doc = MotionDoc::default();
    let sinks = build_compare_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), BANDS, "uma fileira por banda");
    let mut cook = Cook::new();
    sinks
        .iter()
        .map(|&s| {
            let out = cook.cook(&doc.graph, &reg, s, 0.0).expect("cook");
            match out[0].as_stream().get("P") {
                Some(Column::Vec2(v)) => v.iter().map(|p| p[1]).collect(),
                _ => panic!("sem coluna P"),
            }
        })
        .collect()
}

/// Quantas peças de uma fileira estão LEVANTADAS — a máscara é 0/1, então meio
/// caminho separa as duas populações com folga enorme.
fn raised(v: &[f32]) -> usize {
    let lo = v.iter().fold(f32::INFINITY, |m, x| m.min(*x));
    v.iter().filter(|x| **x - lo > VALUE_SCALE * 0.5).count()
}

/// **O OP É LIDO: os dois degraus são COMPLEMENTARES.**
///
/// ⚠️ A metade que faz este gate valer não é *"as duas fileiras diferem"* — é que
/// **a soma das levantadas dá a fileira inteira**. Duas máscaras quaisquer diferem;
/// só um par complementar cobre a fileira exactamente uma vez.
#[test]
fn the_op_is_read_and_the_two_steps_are_complementary() {
    let r = rows();
    let (gt, lt) = (raised(&r[0]), raised(&r[1]));
    let n = r[0].len();
    assert_eq!(n, COLS as usize, "a fileira tem {COLS} peças");
    assert!(
        gt > 0 && lt > 0,
        "nenhuma das duas pode ser plana: {gt}/{lt}"
    );
    assert_eq!(
        gt + lt,
        n,
        "Greater e Less particionam a fileira: {gt} + {lt} contra {n}"
    );
    // E o degrau cai perto do MEIO, que é onde a mensagem diz que ele está.
    let mid = n / 2;
    assert!(
        gt.abs_diff(mid) <= 2,
        "o degrau tem de cair no meio: {gt} levantadas de {n}"
    );
    eprintln!("op lido: Greater {gt} levantadas · Less {lt} · fileira {n}");
}

/// **A TOLERÂNCIA É LIDA: a banda larga é ~4× a estreita** — a razão que a mensagem
/// manda o olho ler, medida em vez de afirmada.
///
/// ⚠️ É o par que um kernel cego a `params.epsilon` não consegue desenhar: as duas
/// fileiras têm o MESMO op, o MESMO limiar e a MESMA entrada.
#[test]
fn the_tolerance_is_read_and_the_wide_band_is_four_times_the_narrow() {
    let r = rows();
    let (narrow, wide) = (raised(&r[2]), raised(&r[3]));
    assert!(narrow > 0, "a banda estreita tem de existir: {narrow}");
    let want = EPS_WIDE / EPS_NARROW; // 4,0 por construção
    #[expect(clippy::cast_precision_loss, reason = "contagens pequenas")]
    let got = wide as f32 / narrow as f32;
    assert!(
        (got - want).abs() < 1.0,
        "a razão das bandas segue a razão dos epsilons: {narrow} e {wide} dão {got:.2}, \
         esperado ~{want:.2}"
    );
    eprintln!(
        "tolerância lida: eps {EPS_NARROW} -> {narrow} peças · {EPS_WIDE} -> {wide} ({got:.2}x)"
    );
}

/// **A fileira do nome que não resolve é PLANA — e é o desenho CERTO.**
///
/// ⚠️ Este gate afirma o oposto do que um gate normal afirma, e de propósito: a
/// escada devolve zeros no comprimento certo, e é essa planura que o badge existe
/// para EXPLICAR. Uma fileira que se mexesse aqui significaria que o nome resolveu,
/// e a cena deixaria de demonstrar a wave.
#[test]
fn the_unresolved_name_draws_a_flat_row_and_that_is_the_point() {
    let r = rows();
    let flat = &r[BANDS - 1];
    let lo = flat.iter().fold(f32::INFINITY, |m, x| m.min(*x));
    let hi = flat.iter().fold(f32::NEG_INFINITY, |m, x| m.max(*x));
    assert!(
        hi - lo < 1e-6,
        "a fileira do nome ausente tem de ser PLANA: [{lo:.6}, {hi:.6}]"
    );
    // E o CONTROLE: ela é plana por não achar o nome, não por a cena estar partida.
    // A banda 1 corre pela MESMA geometria e desenha um degrau.
    assert!(
        raised(&r[0]) > 0,
        "a cena não está partida — a banda 1 desenha"
    );
}
