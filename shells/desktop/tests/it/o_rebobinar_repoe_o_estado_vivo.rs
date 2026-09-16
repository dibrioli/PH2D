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

/// **Mutação que deve sangrar:** apagar a chamada do `fase_fabrica_e_morte.rs`.
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
    // E ela mora DENTRO da guarda do invariante, ao lado da varredura que já lá estava.
    let guarda = codigo
        .find("self.playhead.time() <= 0.0")
        .expect("o invariante do rebobinar tem de existir neste ficheiro");
    let chamada = codigo
        .find("rewind_runtime_state(")
        .expect("ja' afirmado acima");
    assert!(
        chamada > guarda,
        "a reposicao tem de correr DENTRO do invariante do rebobinar, e nao em todo quadro: \
         repor o vivo com o relogio a andar apagaria a corrida a 60 Hz"
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
    let guarda = codigo
        .find("self.playhead.time() <= 0.0")
        .expect("o invariante do rebobinar tem de existir neste ficheiro");
    let chamada = codigo.find("script_bridge::rewind(").expect(
        "os scripts nao renascem ao rebobinar: o `self` da corrida anterior continuaria, e a pose \
         que a corrida escreveu nao voltaria a do artista",
    );
    assert!(
        chamada > guarda,
        "o renascer dos scripts tem de correr DENTRO do invariante do rebobinar"
    );
}
