//! ⭐⭐⭐ **A PORTA de *«quem falou?»*** (suplente #24, 2026-09-19) — os gates de
//! [`ph2d_runtime::SignalOrigin::quem`] e [`ph2d_runtime::SignalOrigin::outro`].
//!
//! # ⛔⛔ Porque ela é uma porta e não um `match` no consumidor
//!
//! São **catorze** origens e **onze** carregam `source`. Um `match` escrito no consumidor esquece a
//! décima quinta **em silêncio**: ela cai no braço `_`, o sinal dela passa a nascer sem sujeito, e
//! isso lê-se exactamente como *«esta origem não tem sujeito»*. O `match` da porta é EXAUSTIVO ⇒ uma
//! variante nova **não compila** até alguém responder.
//!
//! ⚠️ **Este ficheiro é a metade que falha ALTO.** A outra — a que fica verde — seria um gate que
//! contasse as variantes: ele passaria com uma origem nova a responder `None` por engano.

use ph2d_runtime::{EntityBits, Signal, SignalOrigin};

/// **UMA de cada origem** — construídas pelos construtores PÚBLICOS, que é como o produto as faz.
///
/// ⚠️ ⛔ **Nunca escreva as variantes à mão aqui:** um construtor que se esqueça de passar o
/// `source` (o defeito real) continuaria invisível se o corpus fosse literal.
fn todas() -> Vec<(&'static str, Signal)> {
    vec![
        ("timeline", Signal::from_timeline("a", 1.0)),
        ("contact", Signal::from_contact("a", 11, 22)),
        ("control", Signal::from_control("a")),
        ("motion", Signal::from_motion("a", 3, 4)),
        ("animation", Signal::from_animation("a", 33, 1)),
        ("timer", Signal::from_timer("a", 44, 1)),
        ("state_machine", Signal::from_state_machine("a", 55)),
        ("particles", Signal::from_particles("a", 66)),
        ("counter_watch", Signal::from_counter_watch("a", 77, 0)),
        ("action", Signal::from_action("a", 88, 0)),
        ("ui_button", Signal::from_ui_button("a", 99)),
        ("script", Signal::from_script("a", 111)),
        ("spawn", Signal::from_spawn("a", 122, 2)),
        ("death", Signal::from_death("a", 133)),
    ]
}

/// ⭐⭐⭐ **As TRÊS que não têm sujeito são exactamente estas, e as outras onze têm.**
///
/// ⚠️ **A lista negativa é NOMEADA e não derivada**: derivá-la do próprio `quem()` faria o gate
/// concordar com qualquer resposta, que é a forma clássica de um espelho.
///
/// **Mutação que deve sangrar:** o braço do `Timer` a devolver `None`.
#[test]
fn as_tres_origens_sem_sujeito_sao_estas_e_as_outras_onze_tem_um() {
    const SEM: [&str; 3] = ["timeline", "control", "motion"];
    let corpus = todas();
    assert_eq!(corpus.len(), 14, "o censo das origens mudou de tamanho");
    let mut com = 0;
    for (nome, sig) in &corpus {
        let q = sig.origin.quem();
        if SEM.contains(nome) {
            assert!(q.is_none(), "{nome}: uma origem SEM sujeito devolveu um");
        } else {
            assert!(q.is_some(), "{nome}: uma origem COM sujeito devolveu nada");
            com += 1;
        }
    }
    assert_eq!(com, 11, "a populacao das origens com sujeito mudou");
}

/// ⭐⭐ **O contacto é o ÚNICO com dois lados, e eles não se trocam.**
///
/// ⚠️ Os dois valores da fixtura são DIFERENTES (`11` e `22`) de propósito: iguais, a troca seria
/// invisível — a mesma cerca que o gate do `targets_of` põe do outro lado da fronteira.
///
/// **Mutação que deve sangrar:** o `outro()` a devolver `source`.
#[test]
fn so_o_contacto_tem_o_outro_lado_e_os_dois_nao_se_trocam() {
    for (nome, sig) in todas() {
        let o = sig.origin.outro();
        if nome == "contact" {
            assert_eq!(sig.origin.quem(), Some(EntityBits(11)), "quem gritou");
            assert_eq!(o, Some(EntityBits(22)), "o outro lado");
        } else {
            assert!(
                o.is_none(),
                "{nome}: uma origem sem segundo lado devolveu um"
            );
        }
    }
}

/// ⚠️ **Um sujeito de valor ZERO continua a ser um sujeito** — a porta não inventa uma cerca.
///
/// Quem decide se aqueles bits correspondem a uma entidade viva é a shell (`try_from_bits`) e
/// depois o mundo (`get_entity`). *Uma porta que filtrasse aqui poria a mesma decisão em três
/// sítios, e os três divergiriam.*
///
/// **Mutação que deve sangrar:** `Some(*source).filter(|b| b.0 != 0)`.
#[test]
fn a_porta_nao_julga_os_bits_que_devolve() {
    let s = Signal::from_timer("a", 0, 1);
    assert_eq!(s.origin.quem(), Some(EntityBits(0)));
}

/// **E ela é `const`** — um `const fn` não pode ler estado nem alocar, que é o que a torna segura
/// de chamar dentro do laço do quadro.
#[test]
fn a_porta_e_const() {
    const S: SignalOrigin = SignalOrigin::Control;
    const Q: Option<EntityBits> = S.quem();
    assert!(Q.is_none());
}
