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

/// **O TEXTO da ponte** — a porta única do assunto, partilhada com o
/// `the_law_asks_footing_not_the_controller` por `#[path]`.
#[path = "player_bridge_source.rs"]
mod src_of;
use src_of::{player_family, read};

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
    let b = player_family();
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
        "src/bridge/player_kinmove.rs",
        "src/bridge/player_push.rs",
    ] {
        assert!(
            !read(f).contains("solve_character_collision_impulses("),
            "{f} chama o solver aproximado do rapier — a lei desta wave e' a \
             `push_transfer`, e a conversao passa pela porta da reacao"
        );
    }
}

/// **O empurrão sai pela porta CENTRAL, e a lei que o dimensiona é a PURA.**
///
/// # ⚠️ Esta cerca foi DERRUBADA com medição, não apagada (§8.2)
///
/// A versão anterior exigia a porta da reação VERTICAL, argumentando que *"uma
/// segunda porta seria uma segunda resposta para o que este player deve àquele
/// corpo"*. O argumento estava certo sobre a CONVERSÃO e errado sobre a
/// pergunta: as duas metades **não perguntam a mesma coisa**.
///
/// - A vertical pergunta **ONDE** — o ponto dela é a razão de a jangada
///   INCLINAR sob o personagem (cena `=103`, aprovada).
/// - A lateral pergunta **QUANTO** — e o mesmo ponto ali produzia um rodopio de
///   **12 voltas em 3 s** (74,29 rad contra 0,3175 do dinâmico), que ainda
///   realimentava a linha (21,36 m contra 7,13).
///
/// Com a porta central o giro fica residual e o deslocamento passa a bater com
/// o dinâmico (7,27 contra 7,13). O que o gate protege agora é a separação, e a
/// metade do argumento antigo que sobrevive: **as duas convertem com a massa do
/// PLAYER**, então a conversão não pode divergir num lado só.
#[test]
fn the_push_goes_through_the_central_door_with_the_pure_law() {
    let push = read("src/bridge/player_push.rs");
    assert!(
        push.contains("push_transfer(react, blocked, h.normal)"),
        "quanto atravessa e' decidido pela lei PURA, com a normal do contato"
    );
    let bridge = player_family();
    let call = bridge
        .find("push::push_from_hits(")
        .expect("a dedup tem de ser chamada");
    let tail = &bridge[call..];
    let apply = tail
        .find("apply_player_push(")
        .expect("o empurrao lateral tem de sair pela porta CENTRAL");
    assert!(
        apply < 1600,
        "a aplicacao tem de ficar junto da dedup que a produziu (achei {apply} bytes depois)"
    );
    // ⚠️ **E a porta central NÃO pode tomar um ponto** — é isso que a distingue
    // da irmã, e um `apply_impulse_at_point` ali reinstalaria o rodopio inteiro
    // com todos os outros numeros certos.
    let world = read("../ph2d-physics/src/world/player.rs");
    let door = world
        .find("pub fn apply_player_push(")
        .expect("a porta central tem de existir");
    let body = &world[door..door + 1200];
    assert!(
        !body.contains("apply_impulse_at_point("),
        "o empurrao lateral entra no CENTRO de massa -- um ponto aqui e' o rodopio de volta"
    );
    // A metade do argumento antigo que sobrevive: as duas convertem com a massa
    // de QUEM EMPURRA.
    assert!(
        body.contains(".get(player)"),
        "a conversao massa->impulso tem de ler a massa do PLAYER, como a irma vertical"
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
