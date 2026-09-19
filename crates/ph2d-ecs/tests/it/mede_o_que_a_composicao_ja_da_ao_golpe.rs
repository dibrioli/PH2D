//! ⭐⭐⭐ **A PERGUNTA DE ANTES DE QUALQUER LINHA do suplente #24 (`Health`)**:
//! *a composição de hoje — `SignalOnHit` + `SignalActions` + `Counter` + `CounterWatch` — já
//! exprime «o inimigo que levou o tiro perde vida e morre»?*
//!
//! `CLAUDE.md` §5.0: **antes de construir um item de lista aberta, MEÇA se a composição já o
//! exprime.** ⚠️ E aqui há uma razão a mais para medir: o cabeçalho do [`ph2d_ecs::counter_watch`]
//! **declara por escrito** que o `Health` é composição —
//! *«`Counter{name:"vidas"}` + `SignalActions[golpe → AddToCounter(-1)]` +
//! `CounterWatch[vidas AtMost 0 → "morri"]`»*. Esta sonda pergunta se essa frase continua
//! verdadeira quando há **mais do que um** sujeito.
//!
//! ⚠️ **Sonda, não gate.** Corre com `--ignored` e IMPRIME; o que ela decide é *o quê* da wave.
//!
//! ```text
//! cargo test -p ph2d-ecs --test it mede_o_que_a_composicao_ja_da_ao_golpe -- --ignored --nocapture
//! ```
//!
//! # As perguntas, uma por bloco
//!
//! A) **Dez inimigos, UM tiro — quantos reagem?**
//! B) **A tabela consegue dizer «quem me bateu»?**
//! C) **Existe verbo que tire uma coisa da cena?**
//! D) **Dois inimigos com a mesma vida — as vidas são duas ou uma?**

use ph2d_ecs::{
    Counter, CounterRuntime, Name, SignalAction, SignalActions, SignalTarget, SignalVerb, StableId,
    World,
};
use ph2d_tags::TagTree;

/// Uma linha de tabela — o mesmo molde da sonda do cérebro.
fn linha(on: &str, target: &str, verb: SignalVerb) -> SignalAction {
    SignalAction {
        on: on.to_string(),
        target: target.to_string(),
        verb,
        arg: String::new(),
        target_by: SignalTarget::default(),
        // ⚠️ **`Anyone`, que é o default e o que a tabela SABIA dizer** — a cerca `Myself` é o que
        // esta medição encomendou, e pô-la aqui apagaria o fenómeno que a sonda existe para mostrar.
        from: ph2d_ecs::SignalFrom::Anyone,
    }
}

/// A resolução com sinais SEM SUJEITO — que é tudo o que a tabela recebia antes desta wave.
fn resolve_nomes(w: &mut World, t: &TagTree, ns: &[&str]) -> Vec<ph2d_ecs::SignalEffect> {
    let disparos: Vec<ph2d_ecs::signal_actions::Disparo<'_>> = ns
        .iter()
        .map(|n| ph2d_ecs::signal_actions::Disparo::anonimo(n))
        .collect();
    ph2d_ecs::signal_actions::resolve(w, t, &disparos)
}

/// `n` inimigos IGUAIS, cada um com a mesma tabela: *«ao ouvir `golpe`, perco uma vida»*.
///
/// ⚠️ **O alvo é VAZIO**, que é *«este objecto»* — o caso que o doc do `SignalAction` chama de
/// comum, *«é ele que faz uma cópia de prefab funcionar sem rewiring»*.
fn arena(n: u64) -> (World, TagTree) {
    let mut w = World::new();
    for i in 0..n {
        w.spawn((
            Name::new(format!("Inimigo {i}")),
            StableId(i + 1),
            SignalActions(vec![linha("golpe", "", SignalVerb::AddToCounter)]),
            Counter {
                name: "vida".to_string(),
                start: 3,
            },
            CounterRuntime { value: 3 },
        ));
    }
    (w, TagTree::default())
}

#[test]
#[ignore = "sonda: imprime a medição do §5.0"]
fn mede_o_que_a_composicao_ja_da_ao_golpe() {
    println!("\n== suplente #24 -- o que a composicao de HOJE ja' da' a um GOLPE ==\n");

    // -- A) Dez inimigos, um tiro --------------------------------------------
    let (mut w, t) = arena(10);
    let efeitos = resolve_nomes(&mut w, &t, &["golpe"]);
    println!("A) DEZ inimigos iguais, UM sinal `golpe` (um tiro que acertou em UM deles)");
    println!("   efeitos resolvidos: {}", efeitos.len());
    println!(
        "   => {}\n",
        if efeitos.len() == 10 {
            "OS DEZ reagem. O sinal e' um NOME global e a tabela nao sabe QUEM levou o tiro."
        } else {
            "alguem filtrou -- reconferir a nota"
        }
    );

    // -- B) Da' para dizer «quem me bateu»? ----------------------------------
    println!("B) As maneiras de a tabela escolher A QUEM a accao se aplica");
    println!("   variantes de `SignalTarget`: Named (nome, ou vazio = este) . Tagged (por tag)");
    println!("   ha' alguma que diga «quem falou» ou «quem bateu»? NAO");
    println!("   => o `SignalOrigin::Contact` CARREGA `source` e `other`, e a shell deita-os fora:");
    println!("      `shells/desktop/src/render_loop/fase_signal_outbox.rs` -- `.map(|s| s.name)`");
    println!("      => `resolve(world, tags, &[&str])` recebe SO' NOMES.\n");

    // -- C) Existe verbo que tire da cena? -----------------------------------
    println!("C) Os verbos que a tabela oferece ({}):", SignalVerb::ALL.len());
    for v in SignalVerb::ALL {
        println!("      . {v:?}");
    }
    let apaga = SignalVerb::ALL
        .iter()
        .any(|v| format!("{v:?}").contains("Destroy"));
    println!(
        "   => {}\n",
        if apaga {
            "ha' um que apaga -- reconferir a nota"
        } else {
            "NENHUM tira da cena. E a fisica nao le' a `Visibility`, logo `Hide` nao e' morte: \
             um inimigo escondido continua a travar balas."
        }
    );

    // -- D) Duas vidas, ou uma so'? ------------------------------------------
    let (w2, _) = arena(2);
    let soma = ph2d_ecs::counter::soma(&w2, "vida");
    println!("D) DOIS inimigos, cada um com `Counter{{name:\"vida\", start:3}}`");
    println!("   o que a porta do contador responde a «quanto vale `vida`?»: {soma:?}");
    println!(
        "   => {}\n",
        if soma == Some(6) {
            "UMA so' -- a porta SOMA por NOME, em todo o mundo (e o doc dela di-lo). \
             Uma vida POR INIMIGO nao e' exprimivel com um nome partilhado."
        } else {
            "a porta separou-os -- reconferir a nota"
        }
    );

    println!("== o que a wave tem de trazer ==");
    println!("   1. o sinal chega a' tabela COM quem falou e com quem bateu");
    println!("   2. uma linha pode exigir que o golpe seja DELA (a cerca)");
    println!("   3. um verbo que TIRE da cena");
}
