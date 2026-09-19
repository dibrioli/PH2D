//! ⭐⭐⭐ **OS GATES DO RÓTULO DO CHIP `Auto`** — filho do [`super`] por `#[path]`.
//!
//! ⛔⛔⛔ **Ele vive NUM FICHEIRO PRÓPRIO por uma razão MEDIDA, e não por arrumação:** o gate do
//! elo lê o pintor por `include_str!("section.rs")`, e enquanto ele morava lá dentro **a agulha
//! que ele procura estava escrita nele próprio** — apagar a chamada do pintor deixava-o VERDE,
//! porque o texto continuava no ficheiro. Foi uma mutação que o mostrou.
//!
//! ⚠️ *Um gate `include_str!` que procura uma string escrita nele mesmo não afirma nada.*

use super::rotulo_do_auto;

/// ⭐⭐⭐ **O CHIP `Auto` DIZ QUE LADO ESTÁ A DERIVAR** (report do dono, 2026-09-18).
///
/// ⚠️ **As DUAS metades, porque as curas são opostas:** um rótulo que dissesse sempre um lado
/// **mentiria** onde ele não é derivável (sem âncora, `Keep`, ou `Mixed`, que não tem um lado
/// só), e um que nunca o dissesse deixa o report de pé.
#[test]
fn o_auto_nomeia_o_lado_que_deriva_e_so_quando_ele_existe() {
    let ccw = rotulo_do_auto(Some(1));
    let cw = rotulo_do_auto(Some(2));
    assert_ne!(
        ccw, cw,
        "o rotulo e' o mesmo para os dois lados: o artista nao consegue saber qual e'"
    );
    for (t, esperado) in [(&ccw, "CCW"), (&cw, "CW")] {
        assert!(
            t.to_uppercase().contains(esperado),
            "o rotulo «{t}» nao nomeia {esperado}: o artista clica no Auto e no {esperado}, \
             nao ve' diferenca nenhuma, e conclui que os dois estao partidos"
        );
    }
    // ⛔ E os três casos em que NÃO há um lado a nomear ficam com o rótulo nu.
    let nu = rotulo_do_auto(None);
    for i in [None, Some(0), Some(3)] {
        assert_eq!(
            rotulo_do_auto(i),
            nu,
            "o rotulo inventou um lado para {i:?}: o `Keep` seria circular e o `Mixed` nao tem \
             um lado so'"
        );
    }
    assert!(
        !nu.contains('(') && !nu.is_empty(),
        "o rotulo nu ganhou um parentesis: le-se «{nu}»"
    );
}

/// ⛔⛔ **E O PINTOR CHAMA A PORTA** — sem isto a lei fica certa e o pixel não muda.
///
/// ⚠️ **É `include_str!` e não um teste de pixel**, porque o testkit desta casa não tem leitor
/// de texto pintado: *uma lei pura sem este elo é uma lei que o artista não vê*. A dívida está
/// nomeada no doc da porta.
#[test]
fn o_pintor_chama_a_porta_do_rotulo() {
    let fonte = include_str!("section.rs");
    assert!(
        fonte.contains("rotulo_do_auto(state::current_bone_ik_auto_side())"),
        "a fileira do lado da dobra deixou de pedir o rotulo a' porta: ela volta a pintar \
         «Auto» nu, e o report do dono volta com ele"
    );
}
