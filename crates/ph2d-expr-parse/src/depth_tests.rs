//! ⛔⛔⛔ **UMA EXPRESSÃO FUNDA DEVOLVE `Err`, NUNCA ABORTA O PROCESSO.**
//!
//! # O que estes gates defendem
//!
//! Auditoria de seis lentes, 2026-08-31 (doc 96 §3.1). Este parser é descida recursiva e o
//! `Expr` é uma cadeia de `Box`; sem teto, `~7 000` parênteses davam
//! `fatal runtime error: stack overflow, aborting` — **exit 134**, sem desenrolar e sem
//! hipótese de recuperar.
//!
//! ⚠️⚠️ **`14 KB` de texto num text param cabem num paste e cabem num `.ph2dproj`** — e um
//! ficheiro assim **matava o editor a cada tentativa de o abrir**. O parser é partilhado (o
//! `motion.expression`, a timeline e o `source.lsystem` entram todos por aqui), então o teto
//! serve os três.
//!
//! ⚠️ **A régua tem de ser executada, não deduzida:** um gate que só afirmasse `MAX_DEPTH == 256`
//! ficaria verde com a guarda apagada. Estes correm o input patológico e exigem que a função
//! **volte**.

use super::{MAX_DEPTH, parse};

/// `n` parênteses à volta de um `1`.
fn nested(n: usize) -> String {
    format!("{}1{}", "(".repeat(n), ")".repeat(n))
}

#[test]
fn nesting_past_the_ceiling_is_an_error_and_not_an_abort() {
    // ⚠️ Se a guarda desaparecer, este teste não FALHA — ele **aborta o processo de teste**, e
    // é essa a assinatura do defeito que ele cobre.
    let fundo = parse(&nested(MAX_DEPTH as usize + 1));
    assert!(fundo.is_err(), "aninhamento acima do tecto tem de recusar");
    let msg = fundo.unwrap_err();
    assert!(
        msg.contains(&MAX_DEPTH.to_string()),
        "a mensagem tem de dizer o tecto, senão quem a lê não sabe o que corrigir: {msg}"
    );

    // E MUITO acima dele: o número que abortava (`~7 000`) e o que cabe num paste.
    for n in [1_000, 7_000, 20_000] {
        assert!(
            parse(&nested(n)).is_err(),
            "{n} parênteses tinham de recusar em vez de derrubar o processo"
        );
    }
}

#[test]
fn a_chain_of_unary_minus_is_bounded_too() {
    // ⚠️ O `-` encadeado recursa **sem parênteses** — outra porta para a mesma pilha, e a 1.ª
    // redacção desta cura só tinha guardado a dos parênteses.
    assert!(parse(&format!("{}1", "-".repeat(MAX_DEPTH as usize + 2))).is_err());
    assert!(parse(&format!("{}1", "-".repeat(50_000))).is_err());
}

#[test]
fn the_ceiling_never_rejects_an_expression_anyone_would_write() {
    // ⚠️ **A metade que impede a cura de ser pior que a doença.** As expressões do ABOP que este
    // repo de facto usa aninham menos de cinco; um tecto que recusasse produto correcto seria
    // uma regressão com cara de segurança.
    for src in [
        "s*0.7",
        "n < 6",
        "sin(t) * (1 + cos(t * 2))",
        // ⚠️ `clamp` não existe neste parser (a lista é `sin cos abs sqrt floor fract min
        // max mix noise smoothnoise select wiggle`) — a 1.ª redacção inventou-a, e o gate
        // acusou a FIXTURA, não o tecto.
        "min(max((a + b) / (c - d), 0), 1)",
        "((((1 + 2) * 3) - 4) / 5)",
    ] {
        assert!(parse(src).is_ok(), "`{src}` tem de continuar a compilar");
    }
    // ⚠️ **A FRONTEIRA EXACTA, e ela tem um off-by-one que é um FACTO e não um descuido:** o
    // `MAX_DEPTH` conta **níveis de descida abertos**, e a expressão de topo já é um deles. Logo
    // o máximo de parênteses aninhados é `MAX_DEPTH − 1`. Afirmar os dois lados é o que impede
    // alguém de "arredondar" a constante e mover a fronteira sem reparar.
    assert!(
        parse(&nested(MAX_DEPTH as usize - 1)).is_ok(),
        "`MAX_DEPTH − 1` parênteses tê^m de ser aceites"
    );
    assert!(
        parse(&nested(MAX_DEPTH as usize)).is_err(),
        "e `MAX_DEPTH` parênteses já não, porque o topo conta como um nível"
    );
}
