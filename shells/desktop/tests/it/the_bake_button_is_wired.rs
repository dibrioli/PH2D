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

use crate::sculpt_source;
use sculpt_source::{arm_with, braced_block, family, function_body, sculpt_src};

/// **O botão e o atalho armam o MESMO pedido.**
///
/// ⚠️ A asserção é sobre o CAMPO, e não sobre quem o escreve: se o painel
/// ganhasse um segundo campo, cada porta passaria a ter o seu ciclo de vida — e
/// o dia em que uma delas fosse consumida num ponto diferente do frame, o botão
/// e o atalho fariam coisas diferentes com o mesmo nome.
///
/// ⚠️ O laço de frame é o QUADRO pela ordem em que corre (`frame_text::render_frame`), sem comentários como o
/// `sculpt_source::source` que este gate usava: desde a OBRA 2 da `line/render-loop` (2026-09-13) o retorno da ponte
/// do painel mora na `fase_world_panel_bridges`.
#[test]
fn the_button_and_the_shortcut_arm_the_same_request() {
    let cluster = sculpt_src();
    // ⚠️ **O FICHEIRO da fase, e não o texto do quadro** (2026-09-21): o retorno da ponte passou a
    // ser consumido numa função-filha — o tecto de LOC por FUNÇÃO obrigou o corte —, e o emendador
    // do quadro só substitui as fases que o QUADRO chama.
    let loop_src = sculpt_source::source("render_loop/fase_world_panel_bridges.rs");

    // O atalho, no roteador de teclado do cluster.
    assert!(
        cluster.contains("req.bake_request = true;"),
        "o `Shift+B` tem de armar o pedido — sem isso o atalho nao pede nada"
    );
    // E o botão, no laço de frame, a partir do RETORNO do bridge.
    // ⚠️ **A âncora é o NOME da porta e não a lista de argumentos**, e isto foi reescrito quando a
    // ponte ganhou a LEI do objecto assado (2026-09-21): um censo ancorado na chamada inteira
    // reprova sobre produto correcto a cada argumento novo — *ele mede a assinatura, não a fiação*.
    let armed = braced_block(&loop_src, "ph2d_app_sculpt3d::panel_bridge::dispatch(");
    // ⚠️ **A asserção é sobre o CAMPO e não sobre quem o segura**, e é o que o doc deste gate já
    // dizia: o atalho escreve-o por `self.sculpt3d_req`, a fase-filha por um `&mut` com outro nome,
    // e o que os torna a MESMA porta é o campo ser um só.
    assert!(
        armed.contains(".bake_request = true;"),
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
    // ⚠️ **Era `return true` e virou um PEDIDO NOMEADO**, e este gate reprovou
    // produto correto quando o alpha por imagem chegou: um `bool` que significa
    // *"quer bake"* não carrega um segundo pedido. O que se afirma é a
    // propriedade — *o braço SOBE em vez de fazer* —, não a forma da resposta.
    assert!(
        arm.contains("return Some(Sculpt3dFrameRequest::Bake)"),
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
    let bridge = family("panel_bridge.rs");
    let dispatch = function_body(&bridge, "dispatch");
    assert!(
        dispatch.contains("hero.gizmo.iter_selected()"),
        "o fato do alvo tem de sair da SELECAO do canvas: {dispatch}"
    );
    assert!(
        dispatch.contains("panel_snapshot(has_bake_target"),
        "…e chegar ao retrato que o painel pinta"
    );
}

/// ⭐⭐ **TROCAR A LEI ESCREVE NO DOCUMENTO *E* ESQUECE O CARIMBO** — as duas metades.
///
/// ⚠️ **A segunda não é higiene, é a porta única da re-acendida:** o
/// [`ph2d_form_donation::baked_form::relight_stale`] pergunta *«estes pixels foram acesos pelo rig
/// de agora?»*, e só o `lit_with` responde. Sem o esquecer, o artista carrega no chip, **nada muda
/// na tela**, e a lei nova só apareceria no dia em que ele mexesse numa lâmpada — *um controlo que
/// escreve certo e não se vê lê-se exactamente como um controlo morto*.
///
/// ⚠️ **Por que a FONTE e não o comportamento:** o braço vive no laço de frame, que exige janela —
/// o mesmo motivo dos dois gates acima.
#[test]
fn trocar_a_lei_escreve_no_objecto_e_esquece_o_carimbo() {
    // ⚠️ **O FICHEIRO e não o texto do quadro:** o braço vive numa função-filha que a `dispatch`
    // da fase chama, e o emendador do quadro só substitui as fases que o QUADRO chama.
    let fonte = sculpt_source::source("render_loop/fase_world_panel_bridges.rs");
    let arm = arm_with(&fonte, "Sculpt3dFrameRequest::LeiDoAlvo(lei)");
    assert!(
        arm.contains("bake.lei = lei;"),
        "o braço tem de escrever a lei no objecto assado: {arm}"
    );
    assert!(
        arm.contains("bake.lit_with = None;"),
        "…e ESQUECER o carimbo, senão a lei nova não chega ao pixel até a lâmpada se mexer: {arm}"
    );
    // ⭐ **O CONTROLO da régua:** uma agulha que *não* está lá tem de falhar, senão este gate
    // passaria por vácuo sobre um braço que mudou de nome.
    assert!(
        !arm.contains("bake.rig = "),
        "controlo: trocar a lei NÃO toca no rig — ele é a outra metade do documento"
    );
}

/// ⛔⛔ **A LEI QUE O PAINEL PINTA É A DO OBJECTO SELECCIONADO** — e não uma constante.
///
/// ⚠️ **Escrito por uma MUTAÇÃO SOBREVIVENTE:** com `.map(|_| 0)` no lugar da leitura, a fileira
/// continua a ser pintada e a despachar — ela mostra *«Paint»* marcado em toda peça, inclusive nas
/// que acendem pela FORMA. *Um selector que mostra sempre a mesma escolha é indistinguível de um
/// selector que funciona, até o artista reparar que o chip marcado não é o que ele vê.*
///
/// ⚠️ E a leitura tem de entrar pelo MAPA dos assados com os bits da SELECÇÃO: é isso que faz a
/// fileira desaparecer num sprite por assar (sem canais não há lei para escolher).
#[test]
fn a_lei_publicada_e_a_do_objecto_seleccionado() {
    let loop_src = sculpt_source::source("render_loop/fase_world_panel_bridges.rs");
    let i = loop_src
        .find("let lei_do_alvo")
        .expect("controlo: o shell tem de derivar a lei que publica");
    let bloco = &loop_src[i..i + 400.min(loop_src.len() - i)];
    for (agulha, porque) in [
        (
            "iter_selected()",
            "a lei é a do objecto que o artista tem na mão",
        ),
        (
            "baked_forms.get(",
            "…lida do mapa dos ASSADOS, que é quem a guarda",
        ),
        (
            "b.lei.index()",
            "…e é a lei DELE, nunca uma constante — senão o chip marcado mente",
        ),
    ] {
        assert!(
            bloco.contains(agulha),
            "falta `{agulha}` na derivação da lei publicada ({porque}): {bloco}"
        );
    }
}
