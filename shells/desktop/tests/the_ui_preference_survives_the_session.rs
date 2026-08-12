//! **Arch-gate: a preferência de UI é LIDA no arranque e GRAVADA na mudança.**
//!
//! ## Por que um gate de TEXTO
//!
//! As duas metades vivem em código de shell que nenhum teste de unidade alcança: a leitura mora no
//! `init.rs`, dentro do braço que constrói o `HeroScreen` (precisa de janela + GPU), e a escrita é
//! chamada do `forward_to_hero`, que precisa de um `AppGfx` vivo. Os gates de comportamento existem
//! e são reais — `prefs_tests` prova a lei do ficheiro e do detector, `screens/hero/tests` prova que
//! o menu escreve no `motion` —, mas **nenhum deles vê a fiação**, e uma preferência que ninguém lê
//! nem grava passa em todos eles: viva na suíte, inerte no produto.
//!
//! É a mesma classe do `the_gpu_cook_recusal_placement` e do `every_panel_the_shell_drives_is_in_
//! its_registry`: quando a decisão mora num sítio que só o app alcança, o gate lê a fonte.

const INIT: &str = include_str!("../src/init.rs");
const FORWARDING: &str = include_str!("../src/forwarding.rs");
const PERSIST: &str = include_str!("../src/forwarding_persist.rs");

/// O arranque instala as preferências no dono do facto.
///
/// ⚠️ **Os DOIS eixos, e não só o carácter.** O reduced motion é a metade que um artista com
/// sensibilidade vestibular escolheu — esquecê-la no load faria a garantia dele durar até fechar o
/// app, que é a forma mais cara de a perder.
///
/// **Mutação que deve sangrar:** apagar qualquer uma das duas linhas do `init.rs`.
#[test]
fn the_shell_installs_the_saved_preference_before_the_first_frame() {
    assert!(
        INIT.contains("crate::prefs::load()"),
        "o `init.rs` nunca lê `~/.ph2d/prefs.txt` — o carácter escolhido morre ao fechar o app"
    );
    for call in [
        "motion.set_character(prefs.character)",
        "motion.set_reduced_motion(prefs.reduced_motion)",
    ] {
        assert!(
            INIT.contains(call),
            "o `init.rs` lê as preferências e não instala `{call}` — metade da escolha do artista \
             fica no ficheiro e nunca chega ao produto"
        );
    }
}

/// E o hook de ponteiro grava.
///
/// **Mutação que deve sangrar:** tirar a chamada de `persist::prefs_if_changed` do
/// `forward_to_hero` — a escolha passa a valer até fechar o app, e nenhum outro gate repara.
#[test]
fn the_pointer_hook_persists_the_preference_when_it_changes() {
    assert!(
        FORWARDING.contains("persist::prefs_if_changed(hero)"),
        "o `forward_to_hero` não persiste as preferências — a escolha do artista não sobrevive à \
         sessão. É o hook certo porque a escolha É um clique numa row do pill Settings → Motion."
    );
    assert!(
        PERSIST.contains("crate::prefs::should_save(previous, now)"),
        "o detector de mudança não passa pela porta `should_save` — a lei da primeira observação \
         (semeia sem gravar) deixa de ser executável e volta a ser um comentário"
    );
}

/// ⚠️ **CONTROLE POSITIVO.** Sem ele, mudar o nome de um ficheiro deixaria os dois gates acima a
/// varrer o vazio e a passar por vácuo — a falha silenciosa que o `keyboard.rs` partido já produziu
/// neste repo.
#[test]
fn the_scanned_files_are_the_real_ones() {
    assert!(
        INIT.contains("fn ") && INIT.len() > 1000,
        "o `init.rs` não foi lido — o gate da leitura estaria a varrer o vazio"
    );
    assert!(
        FORWARDING.contains("fn forward_to_hero"),
        "o `forwarding.rs` mudou de dono: `forward_to_hero` já não vive aqui, e o gate da escrita \
         passa a afirmar nada sobre o produto"
    );
    assert!(
        PERSIST.contains("fn prefs_if_changed"),
        "o `forwarding_persist.rs` mudou de dono"
    );
}
