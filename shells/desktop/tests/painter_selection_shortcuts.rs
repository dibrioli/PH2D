//! **Os atalhos da seleção do Painter têm um dono, e ele é o MODO** — arch-gate sobre o fonte.
//!
//! Enio, 2026-08-07: a seleção não tinha atalho nenhum; o painel era a única porta. Ctrl+X/C/V/A/D e
//! Ctrl+Shift+I agora existem — e cada um deles **já tem outro dono no app**: Ctrl+A é *selecionar
//! todos os nós* do vetor, Ctrl+C/V são o clipboard do grafo de nós e da timeline.
//!
//! ⚠️ **O que torna isso seguro é a guarda `is_selection_mode`**, e ela não é observável por um teste
//! de unidade: a cadeia mora no `App`, que exige janela. Sem este gate, apagar a guarda deixaria a
//! workspace inteira VERDE e roubaria o Ctrl+A do vetor em silêncio.

use std::fs;

const CHAIN: &str = "shells/desktop/src/input_dispatch/keyboard_painter.rs";
const CALLER: &str = "shells/desktop/src/input_dispatch/keyboard.rs";
/// Onde mora a ORDEM entre quem consome Enter/Esc — o irmão que o teto de LOC criou, e cujo
/// doc-header declara que essas teclas *"viajam juntas"* exatamente para a ordem não se perder.
const ESCAPES: &str = "shells/desktop/src/input_dispatch/keyboard_escapes.rs";

fn read(rel: &str) -> String {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("nao consegui ler {}: {e}", p.display()))
}

/// **A cadeia recusa antes de tocar em qualquer verbo quando o modo não é Selection.**
///
/// O oráculo é POSICIONAL, e é o que importa: a guarda tem de estar ANTES do `match` que despacha —
/// uma guarda depois dele já teria cortado a seleção do vetor.
///
/// **Mutação que sangra:** tirar o `if !painter.is_selection_mode()`.
#[test]
fn the_clipboard_chain_is_gated_on_the_selection_mode() {
    let src = read(CHAIN);
    let start = src
        .find("fn painter_selection_clipboard_chain")
        .expect("a cadeia de clipboard existe");
    let body = &src[start..];
    let guard = body
        .find("is_selection_mode()")
        .expect("a cadeia PERGUNTA pelo modo antes de agir");
    let dispatch = body
        .find("KeyCode::KeyX")
        .expect("a cadeia despacha o Ctrl+X");
    assert!(
        guard < dispatch,
        "a guarda de modo tem de correr ANTES do despacho, senao o Ctrl+A do vetor ja foi roubado"
    );
}

/// **Ctrl exigido, e Ctrl+I só com Shift.** Sem a primeira, um `C` nu cortaria; sem a segunda, o
/// Ctrl+I (que outros donos usam) seria engolido.
///
/// **Mutação que sangra:** trocar `if !ctrl { return false; }` por nada, ou tirar o `if shift`.
#[test]
fn the_chain_demands_ctrl_and_reserves_plain_ctrl_i() {
    let src = read(CHAIN);
    let start = src.find("fn painter_selection_clipboard_chain").unwrap();
    let body = &src[start..];
    assert!(
        body.contains("if !ctrl {"),
        "sem modificador a tecla nua tem de cair fora"
    );
    assert!(
        body.contains("KeyCode::KeyI if shift"),
        "o inverter e Ctrl+SHIFT+I; um Ctrl+I nu nao e nosso"
    );
}

/// **A cadeia é CHAMADA, e depois da cadeia do Delete.** Uma função que ninguém chama é um atalho que
/// não existe — o modo de falha exato que o `keyboard_painter` já documenta para o Delete.
///
/// **Mutação que sangra:** apagar a chamada, ou movê-la para antes do `painter_delete_chain`.
#[test]
fn the_chain_is_called_after_the_delete_chain() {
    let src = read(CALLER);
    let del = src
        .find("self.painter_delete_chain(")
        .expect("a cadeia do Delete e chamada");
    let clip = src
        .find("self.painter_selection_clipboard_chain(")
        .expect("a cadeia de clipboard e CHAMADA (senao o atalho nao existe)");
    assert!(
        del < clip,
        "a ordem declarada e Delete primeiro; e ela que impede a proxima tecla de nascer ambigua"
    );
}

/// **A peça colada decide o Enter/Esc antes de quem DESTRUIRIA trabalho com eles.**
///
/// ⚠️ **A afirmação foi corrigida depois de o gate reprovar a minha primeira versão**, que dizia
/// *"antes de todo mundo"* — e não era verdade nem desejável: o `timeline_key` corre antes e deve
/// continuar correndo, porque o transporte é outra superfície. O que importa é a precedência sobre os
/// donos que agiriam sobre a ARTE: o cancel do gesto de joint e, mais adiante na cadeia, o Enter que
/// faz *Apply* da figura em mãos — esse assaria o traço e deixaria a peça pendurada sobre uma tela
/// que mudou debaixo dela.
///
/// ⚠️ **E ele lia o ARQUIVO ERRADO depois da integração de 2026-08-08**, com a falha exata que
/// esta família já produziu quatro vezes no repo: o `main` PARTIU o `keyboard.rs` pelo teto de LOC
/// e a cadeia de encerramento mudou-se inteira para o irmão `keyboard_escapes.rs`. Os dois donos
/// deixaram de existir no `CALLER`, o `if let Some(...)` não achava nenhum, e o gate ficava
/// **VERDE sem afirmar nada** — sobre um produto correto, o que é pior: ele voltaria a passar no
/// dia em que a ordem quebrasse. *Afirme a PROPRIEDADE, nunca o endereço* — e o `expect` abaixo é
/// o **controle positivo** que transforma "o dono mudou-se de arquivo" numa falha alta em vez de
/// numa varredura vazia.
///
/// **Mutação que sangra:** mover a chamada para depois do `joint_draw_cancel_key`, ou apagá-la.
#[test]
fn the_floating_patch_decides_enter_and_escape_before_the_owners_that_touch_the_art() {
    let src = read(ESCAPES);
    let patch = src
        .find("self.painter_paste_patch_key(")
        .expect("a tecla da peca colada e CHAMADA (senao Enter/Esc nao a alcancam)");
    for (owner, what) in [
        ("self.joint_draw_cancel_key(", "o cancel do gesto de joint"),
        ("self.painter_shape_commit(", "o Apply da figura do Painter"),
    ] {
        let other = src.find(owner).unwrap_or_else(|| {
            panic!(
                "{what} nao esta em {ESCAPES}: ou ele mudou-se de arquivo, ou a cadeia foi \
                 partida -- e um gate que nao acha o vizinho nao esta comparando ORDEM nenhuma"
            )
        });
        assert!(
            patch < other,
            "a peca colada tem de decidir antes de {what}: enquanto ela flutua, Enter/Esc sao dela"
        );
    }
}

/// **E a cadeia inteira é alcançada por UMA chamada do `keyboard.rs`.**
///
/// Sem isto, o gate acima passa a descrever a ordem interna de um arquivo que ninguém chama — o
/// modo de falha que sobra depois de a comparação de ordem mudar de casa.
///
/// **Mutação que sangra:** apagar o `self.escape_key(` do despachante.
#[test]
fn the_dispatcher_still_runs_the_whole_ending_chain() {
    let src = read(CALLER);
    assert!(
        src.contains("self.escape_key("),
        "o despachante chama a cadeia de encerramento (senao Enter/Esc nao alcancam ninguem)"
    );
}

/// **E ela recusa quando não há peça** — senão o Enter pararia de aplicar a figura do artista.
///
/// **Mutação que sangra:** `paste_commit`/`paste_cancel` devolvendo `true` incondicionalmente.
#[test]
fn the_patch_key_consumes_only_when_a_patch_is_live() {
    let src = read(CHAIN);
    let start = src.find("fn painter_paste_patch_key").unwrap();
    let body = &src[start..start + 900.min(src.len() - start)];
    assert!(
        body.contains("painter.paste_commit()") && body.contains("painter.paste_cancel()"),
        "as duas saidas devolvem o bool das portas, e sao ELAS que sabem se havia peca"
    );
    assert!(
        body.contains("_ => false"),
        "toda outra tecla cai fora, sem consumir"
    );
}
