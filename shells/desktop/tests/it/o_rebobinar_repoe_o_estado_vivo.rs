//! ⭐⭐⭐ **A porta do rebobinar TEM CHAMADOR** (TOP-20 #15, W0).
//!
//! ⚠️⚠️ **Este gate existe por uma lição que esta casa já pagou em cheio:** o
//! `hero::dock_columns::close` tem a lei certa, está gateado, e foi medido com **ZERO chamadores de
//! produto** depois de o gesto que o accionava ter saído. *Nenhuma sonda deste repo pergunta se uma
//! PORTA tem chamador* — e uma porta sem chamador e uma lei ausente produzem o mesmo app.
//!
//! A porta é `ph2d_ecs::rewind_runtime::rewind_runtime_state`, e o chamador tem de ser o
//! **INVARIANTE** do rebobinar (relógio no início e parado), nunca um gancho num botão: o
//! transporte tem mais de um caminho até ao zero.

use std::fs;

fn fonte(rel: &str) -> String {
    let p = format!("{}/src/{rel}", env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("{p}: {e}"))
}

/// ⚠️⚠️ **A PREMISSA DESTES DOIS GATES MORREU em 2026-09-19, e eles reprovaram a dizê-lo.**
///
/// Eles mediam uma ORDEM POR POSIÇÃO — *«a chamada aparece DEPOIS da guarda no ficheiro»* — e o
/// renascimento virou uma **porta** (`renascer_a_corrida`) quando um verbo passou a poder pedir o
/// mesmo (`SignalVerb::RestartRun`). A porta é declarada no topo do ficheiro, logo as chamadas que
/// eles procuravam passaram a vir ANTES da guarda: `chamada < guarda`, e os dois ficaram vermelhos
/// **sobre produto correcto**.
///
/// ⛔ **Reescrevê-los para «procure em qualquer sítio» seria apagar a lei.** O que eles protegem é
/// real: repor o vivo com o relógio a andar apagaria a corrida a 60 Hz. ⇒ a régua passa a medir a
/// **PROPRIEDADE em duas metades** — *a porta CONTÉM as quatro coisas* e *ela é CHAMADA de dentro
/// do invariante* —, que é mais forte do que a posição era: ela sobrevive ao ficheiro ser
/// reorganizado, e uma metade esquecida dentro da porta passa a ser visível.
///
/// **Mutação que deve sangrar:** apagar a chamada do invariante · tirar uma das metades da porta.
#[test]
fn a_porta_do_rebobinar_e_chamada_pelo_invariante_do_transporte() {
    let src = fonte("render_loop/fase_fabrica_e_morte.rs");
    // ⚠️ Sem comentários: este ficheiro EXPLICA a cura por escrito, e um censo cru leria a
    // explicação como a chamada (a armadilha que a `line/app-physics` mediu ao varrer `\bApp\b`).
    let codigo: String = src
        .lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        codigo.contains("rewind_runtime_state("),
        "a porta que repoe o estado vivo nao tem chamador na shell — um `Timer` corrido ficaria \
         corrido depois de um Reset, e uma `Factory` com `Max Total` gasto recusar-se-ia a produzir"
    );
    // ⭐ E o RENASCIMENTO é chamado DENTRO da guarda do invariante.
    let guarda = codigo
        .find("self.playhead.time() <= 0.0")
        .expect("o invariante do rebobinar tem de existir neste ficheiro");
    let chamada = codigo[guarda..]
        .find("renascer_a_corrida(")
        .map(|i| i + guarda);
    assert!(
        chamada.is_some(),
        "a reposicao tem de correr DENTRO do invariante do rebobinar, e nao em todo quadro: \
         repor o vivo com o relogio a andar apagaria a corrida a 60 Hz"
    );
    // ⭐⭐ **E a porta tem DOIS chamadores — o invariante e o recomeço.** ⚠️ Sem esta contagem o
    // teste acima passaria com a chamada do invariante APAGADA: ele acharia a do recomeço, que
    // também vem depois da guarda no ficheiro. *Uma régua de posição precisa de uma de população
    // ao lado.*
    assert_eq!(
        codigo.matches("renascer_a_corrida(").count(),
        3,
        "a porta do renascimento tem de ser DECLARADA uma vez e CHAMADA duas — pelo invariante do \
         rebobinar e pelo `SignalVerb::RestartRun`"
    );
}

/// ⭐ **E os SCRIPTS do artista renascem no MESMO invariante** (TOP-20 #16). A VM não mora no mundo,
/// então a porta da família `Logic` não os alcança — a irmã dela é a da ponte.
///
/// **Mutação que deve sangrar:** apagar a chamada, ou tirá-la de dentro da guarda.
#[test]
fn os_scripts_renascem_no_invariante_do_transporte() {
    let src = fonte("render_loop/fase_fabrica_e_morte.rs");
    let codigo: String = src
        .lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n");
    // ⭐⭐ **As QUATRO metades vivem na PORTA**, e o gate mede-as lá — ver a nota da irmã acima
    // sobre a premissa que morreu. ⛔ Uma metade esquecida faz a 2.ª corrida nascer com o resto da
    // primeira, **em silêncio**.
    let porta = codigo
        .find("fn renascer_a_corrida(")
        .expect("o renascimento tem de ser uma PORTA: ele tem dois chamadores");
    let corpo = &codigo[porta..];
    for (metade, agulha) in [
        ("varrer quem nasceu", "factory_bridge::sweep_spawned("),
        ("o estado vivo do mundo", "rewind_runtime_state("),
        ("os SCRIPTS", "script_bridge::rewind("),
        ("os EMISSORES", "particles.rewind("),
    ] {
        assert!(
            corpo.contains(agulha),
            "a porta do renascimento perdeu a metade «{metade}» ({agulha}) — a 2.ª corrida nasce \
             com o resto da primeira, e nada o diz"
        );
    }
}
