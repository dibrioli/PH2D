//! **O pill SCULPT: entrar, sair, e DIZER onde se está** (ADR-0150).
//!
//! Módulo irmão de teste do [`super`] (`#[path]`, `cfg(test)`), no molde do
//! `sculpt3d_transform_tests`: as travessias de papel exigem uma cena, e uma cena exige um device,
//! então elas são `#[ignore]` + `gpu_or_skip!`. O que NÃO exige device (o sync do pill sem cena)
//! roda sempre.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --release --bins sculpt3d::mode::tests -- --ignored --nocapture
//! ```

use ph2d_editor::interaction::InteractiveState;
use ph2d_editor::screens::hero::{HeroScreen, ids};
use ph2d_editor::widget::ButtonState;
use ph2d_mesh::shapes::uv_sphere;

use super::super::Sculpt3dScene;

/// Abre a GPU, ou diz que não há nada a afirmar. (Cópia local do irmão `transform`: um macro
/// exportado entre módulos de teste seria acoplamento por conveniência.)
macro_rules! gpu_or_skip {
    () => {
        match ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) {
            Ok(g) => g,
            Err(_) => {
                eprintln!("no GPU adapter on this machine — nothing to assert");
                return;
            }
        }
    };
}

fn pill_state(hero: &HeroScreen) -> Option<ButtonState> {
    match hero.store.get(ids::TOPBAR_SCULPT3D) {
        Some(InteractiveState::Button { state }) => Some(*state),
        _ => None,
    }
}

/// **SAIR devolve o PONTEIRO** — a queixa inteira do Enio numa asserção.
///
/// ⚠️ **O oráculo é [`Sculpt3dScene::shows_clay`], a porta que o `sculpt3d_pointer_down` de fato
/// consulta**, e não o `clay_on_screen` que o pill lê. Os dois delegam ao mesmo papel hoje, e é
/// justamente por isso que o gate tem de perguntar ao consumidor REAL: se alguém amanhã der ao pill
/// uma segunda fonte, este gate sangra em vez de concordar com ela.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn leaving_the_mode_hands_the_pointer_back_to_the_canvas() {
    let gpu = gpu_or_skip!();
    let mut scene = Sculpt3dScene::new(&gpu.device, uv_sphere(12, 24, 1.0), 1.0);
    assert!(
        scene.shows_clay(),
        "a cena nasce no barro — sem isso o resto do gate nao afirma nada"
    );

    let label = scene.toggle_clay();
    assert!(
        !scene.shows_clay(),
        "o pill nao tirou o barro da tela: o ponteiro continua sendo da cena 3D e o artista segue \
         sem conseguir selecionar um sprite (o report de 2026-08-09), com o botao dizendo que saiu"
    );
    // ⚠️ **A saída é a LUZ, e o gate a nomeia:** ela é a próxima posição do ciclo do `D`, e não um
    // destino escolhido pelo pill. Trocá-la por `Off` apagaria a doação de quem estava a pintar.
    assert!(
        label.starts_with("LUZ"),
        "a saida deixou de ser a proxima posicao do ciclo: {label}"
    );

    scene.toggle_clay();
    assert!(
        scene.shows_clay(),
        "o pill nao volta para o barro — ele so' sabe sair, e o botao promete as duas coisas"
    );
}

/// **ENTRAR de QUALQUER posição volta ao barro** — inclusive da terceira, que o pill não nomeia.
///
/// ⚠️ Sem este gate a mutação *"o toggle é `role.next()` sempre"* passa: a partir do barro ela dá a
/// mesma resposta do produto, e só de `Off` (onde `next()` já é `Clay`) ela coincide por acidente.
/// A posição que a separa é a do MEIO.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn entering_from_the_middle_position_comes_back_to_the_clay() {
    let gpu = gpu_or_skip!();
    let mut scene = Sculpt3dScene::new(&gpu.device, uv_sphere(12, 24, 1.0), 1.0);
    // Uma volta de `D`: barro → luz. É a posição em que o artista pinta com a forma acesa.
    scene.cycle_role();
    assert!(!scene.shows_clay(), "premissa: o barro saiu da tela");

    scene.toggle_clay();
    assert!(
        scene.shows_clay(),
        "entrar a partir da LUZ nao trouxe o barro de volta — o pill anda no ciclo em vez de ENTRAR"
    );
}

/// **O pill SEGUE a tecla `D`** — a razão de o estado ser escrito por frame.
///
/// ⚠️ **A mutação que este gate mata é a barata:** escrever o *pressed* só no clique (o que o
/// `physics_toggle` faz, e ali está certo, porque lá o clique é a única porta). Aqui o `D` move o
/// papel sem passar pelo pill, e um estado guardado passaria a dizer *dentro* sobre uma cena que
/// saiu da tela — em silêncio.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_pill_follows_the_key_that_the_pill_never_sees() {
    let gpu = gpu_or_skip!();
    // ⚠️ `HeroScreen::new` já popula o store — ao contrário do `MockPanelHost::new`, que o pula e
    // que já deixou um gate verde sobre chips mortos sob o mouse.
    let mut hero = HeroScreen::new(ph2d_editor::NodeId(1));
    let mut scene = Sculpt3dScene::new(&gpu.device, uv_sphere(12, 24, 1.0), 1.0);

    super::sync_pill(&mut hero, Some(&scene));
    assert_eq!(
        pill_state(&hero),
        Some(ButtonState::Pressed),
        "com o barro na tela o pill tem de estar aceso"
    );

    // O artista aperta `D` — o pill nunca soube.
    scene.cycle_role();
    super::sync_pill(&mut hero, Some(&scene));
    assert_eq!(
        pill_state(&hero),
        Some(ButtonState::Normal),
        "o `D` tirou o barro da tela e o pill continuou aceso: ele guarda um bool proprio, e ele \
         ja' discorda da cena"
    );
}

/// **Sem cena o pill fica SOLTO** — o estado honesto de *entrar*.
///
/// Não precisa de device (é o caso `None`), então roda sempre — e é ele que prova que o sync mora
/// ANTES do early-return da ponte.
#[test]
fn with_no_scene_the_pill_is_the_invitation_to_enter() {
    let mut hero = HeroScreen::new(ph2d_editor::NodeId(1));
    super::sync_pill(&mut hero, None);
    assert_eq!(
        pill_state(&hero),
        Some(ButtonState::Normal),
        "sem escultura nenhuma o pill nao pode estar aceso"
    );
}

/// **O pedido é CONSUMIDO mesmo quando não há o que fazer.**
///
/// ⚠️ Guardá-lo faria a escultura nascer sozinha no frame em que a janela aparecesse — muito depois
/// do clique, e sem nada na tela explicando por quê.
#[test]
fn a_request_with_no_gpu_is_spent_not_stored() {
    let mut app = crate::app_state::App::new();
    app.sculpt3d_toggle_request = true;
    app.sculpt3d_apply_toggle();
    assert!(
        !app.sculpt3d_toggle_request,
        "o pedido sobreviveu ao frame: a cena nasceria sozinha quando a janela aparecesse"
    );
}

/// **O painel do modo acompanha o modo — nas BORDAS, e só nelas.**
///
/// As duas metades são os dois defeitos que o Enio reportou juntos (2026-08-09): *"o painel não
/// está reabrindo quando fechado e não fecha quando sai do modo sculpt"*.
///
/// ⚠️ **A regra anterior era `None → Some` da CENA**, e a cena sobrevive à saída de propósito
/// (sair nunca larga a escultura) — então ela não tinha segunda borda: fechar o painel uma vez o
/// custava para o resto da sessão, e sair do modo deixava as ferramentas do modo na tela, sobre o
/// slot do INSPECTOR.
///
/// ⚠️ **A asserção do MEIO é a que impede a cura de matar o `X`:** entre duas bordas o bridge não
/// pode ter opinião nenhuma, senão re-abrir todo frame torna o botão de fechar um controle que não
/// faz nada.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_panel_follows_the_edges_of_the_clay_and_nothing_between_them() {
    let gpu = gpu_or_skip!();
    let mut scene = Sculpt3dScene::new(&gpu.device, uv_sphere(12, 24, 1.0), 1.0);

    assert_eq!(
        scene.take_clay_edge(),
        Some(true),
        "a cena nova não anuncia a entrada: o painel nasce fechado sobre uma escultura na tela"
    );
    assert_eq!(
        scene.take_clay_edge(),
        None,
        "a borda sobrevive ao frame: o bridge re-afirmaria a visibilidade e o `X` viraria um botão \
         morto"
    );

    scene.toggle_clay();
    assert!(
        !scene.shows_clay(),
        "premissa: o pill tirou o barro da tela"
    );
    assert_eq!(
        scene.take_clay_edge(),
        Some(false),
        "sair do modo não fecha o painel: as ferramentas do barro ficam sobre o slot do inspector"
    );

    scene.toggle_clay();
    assert_eq!(
        scene.take_clay_edge(),
        Some(true),
        "voltar ao modo não reabre o painel: um `X` custaria as ferramentas para o resto da sessão"
    );
}
