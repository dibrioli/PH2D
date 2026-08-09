//! **W-KinPush — o empurrão é a NOSSA lei, não a do controlador.**
//!
//! O rapier traz um `solve_character_collision_impulses`, e o doc dele diz o que
//! ele é: *"only approximate as it is not based on a global constraints
//! resolution scheme"*. Ele reconstrói manifolds por conta própria, usa a
//! `translation_remaining` de cada colisão (e não o bloqueio TOTAL do tique) e
//! escala por uma razão de massas que não é a do resto deste módulo.
//!
//! ⚠️ **A wave é sobre QUAL lei empurra, não sobre SE empurra**, e nenhum gate de
//! comportamento distingue as duas: as duas movem o caixote, e uma tabela de
//! *"quanto andou"* aceita qualquer uma. O que as separa é estrutural — a nossa
//! atravessa a mesma porta da reação vertical (`apply_player_reaction`, com a
//! massa do PLAYER) e a lei dela é pura (`push_transfer`, testável sem rapier).
//!
//! ⚠️ **Arch-gate, e não gate de unidade**, pela razão de sempre neste módulo: a
//! troca compila, roda e deixa a suíte verde.

use std::fs;

fn read(rel: &str) -> String {
    let path = format!("{}/{rel}", env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn character() -> String {
    read("../ph2d-physics/src/world/character.rs")
}

/// **O CONTROLE:** o scanner acha os dois arquivos e o que se espera deles.
///
/// ⚠️ Sem isto, um caminho errado deixa as duas ausências abaixo verdes por
/// vácuo — *um gate de ausência precisa de um irmão de presença*.
#[test]
fn the_scanner_reads_the_files_it_claims_to_scan() {
    let c = character();
    assert!(
        c.contains("controller.move_shape("),
        "o controlador tem de estar aqui"
    );
    assert!(
        c.contains("struct CharacterHit"),
        "e o fato cru que ele publica"
    );
    let b = read("src/bridge/player.rs");
    assert!(
        b.contains("push::push_from_hits("),
        "a ponte tem de chamar a dedup"
    );
}

/// **O solver aproximado da biblioteca não é CHAMADO em lugar nenhum.**
///
/// ⚠️ **A âncora é o parêntese, e a 1ª versão deste gate não o tinha:** ele
/// procurava o nome nu e ficou VERMELHO sobre um produto correto, porque o
/// doc-comment do `character.rs` **cita** aquela função para explicar de onde
/// veio a forma de levar a testemunha para o mundo. *Um oráculo que casa com a
/// própria documentação não está a olhar para o produto* — a mesma cicatriz do
/// `{stamp_avg:` do Painter.
#[test]
fn the_libraries_approximate_solver_is_not_the_law() {
    for f in [
        "../ph2d-physics/src/world/character.rs",
        "../ph2d-physics/src/world/player.rs",
        "src/bridge/player.rs",
        "src/bridge/player_push.rs",
    ] {
        assert!(
            !read(f).contains("solve_character_collision_impulses("),
            "{f} chama o solver aproximado do rapier — a lei desta wave e' a \
             `push_transfer`, e a conversao passa pela porta da reacao"
        );
    }
}

/// **O empurrão sai pela MESMA porta da reação vertical**, com a massa do
/// player, e a lei que o dimensiona é a PURA.
///
/// ⚠️ Uma segunda porta aqui seria uma segunda resposta para *"o que este player
/// deve àquele corpo?"* — e as duas divergiriam no dia em que a conversão
/// massa→impulso mudasse num lado só.
#[test]
fn the_push_goes_through_the_reaction_door_with_the_pure_law() {
    let push = read("src/bridge/player_push.rs");
    assert!(
        push.contains("push_transfer(react, blocked, h.normal)"),
        "quanto atravessa e' decidido pela lei PURA, com a normal do contato"
    );
    let bridge = read("src/bridge/player.rs");
    let call = bridge
        .find("push::push_from_hits(")
        .expect("a dedup tem de ser chamada");
    let tail = &bridge[call..];
    let apply = tail
        .find("apply_player_reaction(")
        .expect("o empurrao tem de ser aplicado pela porta da reacao");
    assert!(
        apply < 900,
        "a aplicacao tem de ficar junto da dedup que a produziu (achei {apply} bytes depois)"
    );
}

/// **O bloqueio é o do TIQUE INTEIRO, não o resto de uma iteração.**
///
/// ⚠️ É esta a diferença aritmética entre a nossa lei e a do rapier, e ela é
/// invisível num caixote solto: com um obstáculo só as duas coincidem quase
/// sempre. Elas separam-se num deslize com várias iterações, onde a
/// `translation_remaining` da primeira colisão ainda contém o que o slide vai
/// entregar depois — e devolvê-la seria empurrar com um movimento que de facto
/// aconteceu.
#[test]
fn what_travels_is_the_whole_ticks_blocking() {
    let push = read("src/bridge/player_push.rs");
    assert!(
        push.contains("let blocked = [wanted[0] - effective[0], wanted[1] - effective[1]];"),
        "o bloqueio e' `pedido - efetivo`, o total do tique"
    );
    assert!(
        !push.contains("translation_remaining"),
        "e nunca o resto de uma iteracao do slide"
    );
}
