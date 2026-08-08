//! **Arch-gates do BOTÃO de assar no sprite** (ADR-0150, o objetivo 2).
//!
//! ⚠️ **Por que a fonte e não o comportamento:** os dois lados desta fiação
//! exigem coisas que nenhum teste headless tem. O `apply_panel_intent` é método
//! de uma `Sculpt3dScene`, que só existe com um `wgpu::Device` vivo; e o
//! consumo do pedido mora no laço de frame, que exige janela. É o mesmo motivo
//! do `the_sculpt_gesture_is_wired` ao lado — *um gate de unidade é cego à
//! fiação do shell*.
//!
//! ## O que estes gates protegem
//!
//! O gesto de assar tinha **uma porta só, e ela era um atalho** (`Shift+B`) —
//! nada na tela o mencionava, então na prática ele existia para quem o
//! escreveu. O botão é a segunda forma de PEDIR, e a única coisa que impede
//! duas formas de pedir de virarem duas ferramentas com um nome só é elas
//! armarem o **mesmo** campo.

mod sculpt_source;
use sculpt_source::{arm_with, braced_block, function_body, sculpt_src, source};

/// **O botão e o atalho armam o MESMO pedido.**
///
/// ⚠️ A asserção é sobre o CAMPO, e não sobre quem o escreve: se o painel
/// ganhasse um segundo campo, cada porta passaria a ter o seu ciclo de vida — e
/// o dia em que uma delas fosse consumida num ponto diferente do frame, o botão
/// e o atalho fariam coisas diferentes com o mesmo nome.
#[test]
fn the_button_and_the_shortcut_arm_the_same_request() {
    let cluster = sculpt_src();
    let loop_src = source("render_loop/mod.rs");

    // O atalho, no roteador de teclado do cluster.
    assert!(
        cluster.contains("self.sculpt3d_bake_request = true;"),
        "o `Shift+B` tem de armar o pedido — sem isso o atalho nao pede nada"
    );
    // E o botão, no laço de frame, a partir do RETORNO do bridge.
    let armed = braced_block(
        &loop_src,
        "sculpt3d_panel_bridge::dispatch(hero, sculpt3d.as_mut())",
    );
    assert!(
        armed.contains("self.sculpt3d_bake_request = true;"),
        "o retorno do bridge do painel tem de armar o MESMO campo que o atalho, \
         e ele arma: {armed}"
    );
}

/// **O intent do botão NÃO é traduzido para a cena — ele sobe.**
///
/// ⚠️ É o que separa este verbo dos outros vinte do painel: assar precisa do
/// mundo, do renderizador e do mapa de atlas, e os três só existem dentro do
/// frame. Um braço que tentasse fazer o trabalho ali teria de inventar um
/// segundo caminho para o bake — e dois caminhos para *"o que a forma escreve
/// num sprite"* divergem no primeiro canal novo.
#[test]
fn the_bake_intent_leaves_the_scene_untouched_and_travels_up() {
    let cluster = sculpt_src();
    let arm = arm_with(&cluster, "Sculpt3dIntent::BakeToSprite");
    assert!(
        arm.contains("return true"),
        "o braco do bake tem de SUBIR o pedido: {arm}"
    );
    assert!(
        !arm.contains("self."),
        "o braco do bake nao pode tocar a cena — ele nao tem com que assar: {arm}"
    );
}

/// **O alvo do bake é lido da SELEÇÃO da cena 2D, no bridge.**
///
/// ⚠️ E não da escultura, que não sabe — nem deve saber — quem está selecionado
/// no canvas. Sem esta linha o retrato diria *"há alvo"* para sempre e a dica do
/// painel nunca apareceria: um aviso que não aparece é o mesmo que aviso
/// nenhum, e o artista descobriria a condição pelo toast, depois do clique.
#[test]
fn the_panel_learns_about_the_selection_from_the_bridge() {
    let bridge = source("render_loop/sculpt3d_panel_bridge.rs");
    let dispatch = function_body(&bridge, "dispatch");
    assert!(
        dispatch.contains("hero.gizmo.iter_selected()"),
        "o fato do alvo tem de sair da SELECAO do canvas: {dispatch}"
    );
    assert!(
        dispatch.contains("panel_snapshot(has_bake_target)"),
        "…e chegar ao retrato que o painel pinta"
    );
}
