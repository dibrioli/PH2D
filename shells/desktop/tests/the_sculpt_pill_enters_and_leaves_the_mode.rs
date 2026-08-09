//! **O pill SCULPT entra e sai do modo — e a fiação que o torna vivo** (ADR-0150).
//!
//! ⚠️ **Nenhum teste de unidade alcança a metade que importa aqui.** O cumprimento do pedido mora
//! no laço de frame (o único ponto em que o `device`, o tamanho da superfície e a escultura estão
//! os três em escopo) e o laço precisa de janela e de GPU. Sobra ler o fonte — e o que se lê é a
//! PROPRIEDADE, nunca um endereço.
//!
//! O comportamento (o papel viaja, o pill segue o `D`) é gateado ao lado da cena, em
//! `src/sculpt3d_mode_tests.rs`.

use std::fs;

const FRAME: &str = "src/render_loop/mod.rs";
const BRIDGE: &str = "src/render_loop/sculpt3d_panel_bridge.rs";
const MODE: &str = "src/sculpt3d_mode.rs";

/// **O pill EXISTE, é registrado, e o clique chega ao barramento** — três das quatro condições de
/// UI numa asserção (a quarta, *a sequência leva a algum lugar*, é dos gates ao lado da cena).
///
/// ⚠️ **A do meio é a que já matou um pill neste repo:** pintado e hit-indexado no fixture mas sem
/// registro no `populate`, ele não tem `InteractiveState`, o `Up` nunca emite `Click`, e o botão
/// nasce morto sob o mouse — com todo o resto verde.
#[test]
fn the_pill_is_painted_registered_and_reaches_the_bus() {
    use ph2d_editor::action_bus::EditorAction;
    use ph2d_editor::interaction::{InteractiveState, WidgetEvent};
    use ph2d_editor::screens::hero::{HeroScreen, chrome, fixture, ids};

    assert!(
        fixture::topbar_clusters()
            .iter()
            .any(|(id, _)| *id == ids::TOPBAR_SCULPT3D),
        "o pill não está entre os clusters que a topbar PINTA"
    );

    let mut hero = HeroScreen::new(ph2d_editor::NodeId(1));
    assert!(
        matches!(
            hero.store.get(ids::TOPBAR_SCULPT3D),
            Some(InteractiveState::Button { .. })
        ),
        "o pill não foi registrado no `populate`: ele desenha e está morto sob o mouse"
    );

    assert!(
        chrome::dispatch_all(&mut hero, WidgetEvent::Click(ids::TOPBAR_SCULPT3D)),
        "o clique no pill não é consumido por handler nenhum"
    );
    assert!(
        hero.bus
            .drain()
            .any(|a| matches!(a, EditorAction::ToggleSculpt3d)),
        "o clique foi consumido e não pediu nada: o pill é um botão que não faz nada"
    );
}

/// **O clique do pill chega ao shell, e o shell o cumpre.**
///
/// ⚠️ **A ORDEM é load-bearing e é ela que o gate afirma:** o toggle escreve o papel da forma, e o
/// papel é exatamente o que decide se a doação roda — cumpri-lo DEPOIS do `donate_form` deixaria a
/// tinta acesa por um estado que o artista já mudou, um frame atrás do que ele vê.
#[test]
fn the_frame_fulfils_the_toggle_before_the_form_donates() {
    let src = fs::read_to_string(FRAME).expect("o laço de frame existe");
    let toggle = src
        .find("self.sculpt3d_apply_toggle()")
        .expect("o laço de frame cumpre o pedido do pill SCULPT");
    let donate = src
        .find("self.sculpt3d_donate_form()")
        .expect("o laço de frame roda a doação");
    assert!(
        toggle < donate,
        "o toggle do pill corre DEPOIS da doação: a forma doa (ou deixa de doar) segundo o papel \
         de antes do clique"
    );
    // O braço do dreno: sem ele o pill é um botão que emite uma ação que ninguém escuta.
    assert!(
        src.contains("EditorAction::ToggleSculpt3d => self.sculpt3d_toggle_request = true"),
        "o dreno de ações não arma o pedido do pill"
    );
}

/// **O pill diz o que a forma É, mesmo quando não há forma.**
///
/// ⚠️ O sync tem de correr **antes** do early-return da ponte: depois dele, o frame em que a cena
/// desaparecesse deixaria o botão preso em *pressed* para sempre — aceso sobre uma escultura que
/// não existe.
#[test]
fn the_pill_is_synced_before_the_bridge_gives_up_on_a_missing_scene() {
    let src = fs::read_to_string(BRIDGE).expect("a ponte do painel 3D existe");
    let sync = src
        .find("sync_pill(hero,")
        .expect("a ponte sincroniza o pill SCULPT");
    let give_up = src
        .find("let Some(scene) = scene else")
        .expect("a ponte desiste sem cena");
    assert!(
        sync < give_up,
        "o sync do pill mora DEPOIS do early-return: sem cena o botão fica com o estado do último \
         frame em que houve uma"
    );
}

/// **O pill não é um botão morto num run normal.**
///
/// ⚠️ **É a metade que tira o módulo de trás da variável de ambiente.** Sem ela, entrar com o app
/// frio não faz nada — e um botão que não faz nada é pior que um botão que falta. A criação usa a
/// MESMA primitiva do verbo de acrescentar peça; uma malha própria aqui seria a segunda resposta a
/// *com que forma uma escultura começa*.
#[test]
fn entering_with_no_scene_creates_one_from_the_one_primitive_door() {
    let src = fs::read_to_string(MODE).expect("o módulo do modo existe");
    assert!(
        src.contains("Sculpt3dScene::new(&device, mesh, aspect)"),
        "entrar sem cena não cria nenhuma: o pill é um botão morto em todo run sem a env var"
    );
    assert!(
        src.contains("Primitive::Sphere.mesh()"),
        "a peça inicial deixou de vir da porta única das primitivas"
    );
    // ⚠️ E SAIR nunca larga a cena: apagar a escultura num botão cujo nome não promete isso é
    // destruir o trabalho do artista. O `D`, que faz a mesma travessia, também não apaga nada.
    assert!(
        !src.contains("sculpt3d = None"),
        "o toggle larga a cena em algum caminho — sair do modo passou a apagar a escultura"
    );
}
