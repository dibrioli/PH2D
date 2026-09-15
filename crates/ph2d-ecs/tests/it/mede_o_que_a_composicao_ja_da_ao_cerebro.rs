//! ⭐⭐⭐ **A PERGUNTA DE ANTES DE QUALQUER LINHA do TOP-20 #15 (`StateMachine`)**:
//! *a composição de hoje — `Signal` + `SignalActions` + `Timer` + `Tags` — já exprime um cérebro
//! autorável?*
//!
//! `CLAUDE.md` §5.0: **antes de construir um item de lista aberta, MEÇA se a composição já o
//! exprime** — e na wave anterior (#14) esta mesma pergunta **reescreveu a entrega** (o ricochete
//! já era exacto; o componente existe por outra razão).
//!
//! ⚠️ **Sonda, não gate.** Corre com `--ignored` e IMPRIME; o que ela decide é *o quê* do
//! componente, não se os números batem uma barra.
//!
//! ```text
//! cargo test -p ph2d-ecs --test it mede_o_que_a_composicao_ja_da_ao_cerebro -- --ignored --nocapture
//! ```
//!
//! # As perguntas, uma por bloco
//!
//! A) **O MESMO sinal pode fazer coisas DIFERENTES?** — é a definição de um estado.
//! B) **Uma acção pode DEPENDER de alguma coisa?** — condição, guarda, o que for.
//! C) **Uma acção pode EMITIR um sinal?** — sem isso não há cadeia de transições.
//! D) **A máquina de estados que a casa JÁ TEM** (`ph2d-ui-state`) alcança a cena?

use ph2d_ecs::{Name, SignalAction, SignalActions, SignalVerb, StableId, World};
use ph2d_tags::TagTree;

/// Uma cena mínima: um botão que emite `botao`, e uma porta com a tabela que reage.
fn cena(linhas: Vec<SignalAction>) -> (World, TagTree) {
    let mut w = World::new();
    w.spawn((Name::new("Parede"), StableId(1)));
    w.spawn((Name::new("Porta"), StableId(2), SignalActions(linhas)));
    (w, TagTree::default())
}

fn linha(on: &str, target: &str, verb: SignalVerb) -> SignalAction {
    SignalAction {
        on: on.to_string(),
        target: target.to_string(),
        verb,
        arg: String::new(),
        target_by: ph2d_ecs::SignalTarget::default(),
    }
}

#[test]
#[ignore = "sonda: imprime a medição do §5.0"]
fn mede_o_que_a_composicao_ja_da_ao_cerebro() {
    println!("\n══ TOP-20 #15 — o que a composição de HOJE já dá a um cérebro ══\n");

    // ── A) O MESMO sinal, duas linhas ────────────────────────────────────────
    // Se um estado existisse, `botao` faria UMA coisa de cada vez. Mede-se o contrário.
    let (mut w, t) = cena(vec![
        linha("botao", "Parede", SignalVerb::Show),
        linha("botao", "Parede", SignalVerb::Hide),
    ]);
    let efeitos = ph2d_ecs::signal_actions::resolve(&mut w, &t, &["botao"]);
    println!("A) O MESMO sinal com duas linhas contraditorias (Show + Hide na mesma parede)");
    println!("   efeitos disparados: {}", efeitos.len());
    for e in &efeitos {
        println!("      · {:?}", e.verb);
    }
    println!(
        "   ⇒ {}\n",
        if efeitos.len() == 2 {
            "AS DUAS disparam. Nao ha' nada que escolha UMA — que e' o que um ESTADO e'."
        } else {
            "alguma coisa escolheu — reconferir a nota"
        }
    );

    // ── B) Uma acção pode ser CONDICIONAL? ───────────────────────────────────
    // A `SignalAction` é uma struct com campos fixos: conte-os e diga quais são condição.
    println!("B) Os campos de uma linha da tabela (a unica autoria de logica que existe):");
    println!("      on · target · verb · arg · target_by   ⇒ 5 campos");
    println!("   destes, QUANTOS sao uma condicao (guarda, if, estado)? **ZERO**");
    println!("   ⇒ uma linha dispara SEMPRE que o nome casa. Nao ha' onde escrever «se».\n");

    // ── C) Uma acção pode EMITIR um sinal? ───────────────────────────────────
    let emite: Vec<&SignalVerb> = SignalVerb::ALL
        .iter()
        .filter(|v| format!("{v:?}").contains("Signal") || format!("{v:?}").contains("Emit"))
        .collect();
    println!("C) Os verbos com sink real hoje: {}", SignalVerb::ALL.len());
    for v in SignalVerb::ALL {
        println!("      · {}", v.label());
    }
    println!("   verbos que EMITEM um sinal: {}", emite.len());
    println!(
        "   ⇒ {}\n",
        if emite.is_empty() {
            "NENHUM — e o doc do modulo declara a ausencia por escrito, com o motivo (a classe \
             dos lacos). Sem emitir, uma transicao nao pode acordar a seguinte."
        } else {
            "existe — reconferir"
        }
    );

    // ── D) A máquina que a casa JÁ TEM ───────────────────────────────────────
    println!("D) A casa TEM uma maquina de estados: `ph2d-ui-state::Machine`.");
    println!("   O doc dela declara o que ela NAO e':");
    println!("      «um grafo de estados com transicoes autoradas (a state machine do Rive)»");
    println!("      «aquilo tem condicoes, entradas nomeadas e um editor proprio»");
    println!("   Mais: os estados dela sao PAPEIS fixos (Default/Hover/Pressed/Disabled), o");
    println!("   gatilho e' DERIVADO do rato, e ela nao e' componente registado ⇒ nao viaja no");
    println!("   ficheiro do projecto nem aparece no Inspector de um objecto de CENA.\n");

    println!("══ LEITURA ══");
    println!("O que falta nao e' «accoes»: e' a MEMORIA de em que estado se esta', e o lugar");
    println!("onde se escreve «SE estou em X E acontecer Y, va' para Z».");
}
