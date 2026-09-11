//! ⛔⛔ **O recuo interno de uma linha da Hierarquia é 2 px, por decisão do dono — e o modelo diz 4.**
//!
//! Perguntado em 2026-09-07 com a divergência na mão, o dono respondeu **«2 px»**. O Godot Modern
//! dá `Tree.inner_item_margin_left = base_margin` = 4.
//!
//! ⚠️ **É o terceiro veredito dele na mesma direcção**, e é isso que o torna uma preferência e não
//! um acaso: em 2026-05-24 mandou colar a **seta ao ícone** (`Xs` → `Xxs`) e o **nome ao ícone**
//! (`Md` → `Xs`, que a wave 25 estendeu a todo o app por ordem dele).
//!
//! Este gate existe para que ninguém «corrija» a casa de volta para o modelo achando que foi
//! deriva — *uma divergência deliberada sem gate lê-se como um esquecimento*.

/// O valor é o `Spacing::Xxs`, e é o dono quem o escolhe.
#[test]
fn the_hierarchy_row_inset_is_two_and_not_the_models_four() {
    let inset = ph2d_tokens::Spacing::Xxs.px();
    assert_eq!(
        inset, 2.0,
        "o recuo interno de uma linha da Hierarquia saiu dos 2 px que o dono decidiu em 2026-09-07"
    );
    assert_ne!(
        inset, 4.0,
        "alguem repos o `inner_item_margin_left` = 4 do Godot Modern: a divergencia e' DELIBERADA \
         e tem dono e data"
    );
}
