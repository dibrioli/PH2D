//! Gates do [`super`] — **o valor CHEGA a um consumidor**, que é a pergunta que nenhum gate deste
//! repo fazia sobre estes cinco campos.
//!
//! ⚠️ **Cada gate move o RECT, nunca um `bool` auxiliar.** Um teste que escrevesse «pausado = true»
//! e verificasse que o tique pára mediria o `bool`; o defeito real era o rect nunca ser lido, por
//! isso a única entrada legítima é a geometria que o artista digita na §8.
//!
//! ⚠️ **Controlo NEGATIVO em toda a parte:** cada gate mede primeiro a mesma cena com a entidade
//! DENTRO do rect. Sem essa metade, uma implementação que recusasse tudo passaria.

use super::{hides, processing_paused};
use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, EnableMode, Entity, Name, OnScreenEnabler, SimWorld, Transform, World};

/// Uma entidade em `(x, y)`, sem enabler.
fn spawn_at(w: &mut World, name: &str, x: f32, y: f32) -> Entity {
    w.spawn((
        Transform::from_translation(Vec2::new(x, y)),
        Name::new(name),
    ))
    .id()
}

/// O rect canónico das fixturas: `[0, 0, 10, 10]` — a origem está DENTRO, `(100, 100)` está fora.
const RECT: [f32; 4] = [0.0, 0.0, 10.0, 10.0];

fn enable(w: &mut World, e: Entity, mode: EnableMode) {
    w.entity_mut(e).insert(OnScreenEnabler::new(RECT, mode));
}

/// ⭐⭐ **O gate que o defeito pedia: o RECT decide se a entidade desenha.**
///
/// Duas posições diferentes sobre a MESMA cena dão duas respostas diferentes ao extract — que é o
/// que *«só aparece quando está no ecrã»* quer dizer. Antes desta wave as duas davam `false`,
/// porque ninguém perguntava.
///
/// **Mutação:** apagar o ramo `HideVisible` de [`super::hides`] ⇒ RED (a de fora passa a desenhar).
#[test]
fn leaving_the_authored_rect_stops_the_sprite_from_drawing() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let e = spawn_at(w, "Coin", 5.0, 5.0);
    enable(w, e, EnableMode::HideVisible);

    assert!(
        !hides(w, e, [5.0, 5.0]),
        "dentro do rect autorado a entidade tem de desenhar — sem esta metade um gate que \
         recusasse tudo passaria"
    );
    assert!(
        hides(w, e, [100.0, 100.0]),
        "fora do rect autorado a entidade continuou a desenhar: o `contains` do OnScreenEnabler \
         nao chegou ao extract"
    );
}

/// ⚠️ **Só o `HideVisible` esconde.** Um modo de pausa que também escondesse tornaria os três
/// indistinguíveis para quem olha a tela — e o enum passaria a ter duas opções mortas.
///
/// **Mutação:** trocar a comparação de [`super::hides`] por `.is_some()` ⇒ RED.
#[test]
fn the_pause_modes_never_hide_they_only_stop_the_clock() {
    for mode in [EnableMode::InheritPause, EnableMode::PauseProcessing] {
        let mut sim = SimWorld::new();
        let w = sim.world_mut();
        // ⚠️ **Fora do rect por CONSTRUÇÃO** — o [`super::processing_paused`] lê a pose do mundo
        // sozinho, então uma fixtura DENTRO do rect mediria o modo com o enabler satisfeito.
        let e = spawn_at(w, "Runner", 100.0, 100.0);
        enable(w, e, mode);
        assert!(
            !hides(w, e, [100.0, 100.0]),
            "{mode:?} escondeu a entidade — esse e' o trabalho do HideVisible, e confundi-los \
             deixa duas das tres opcoes sem efeito proprio"
        );
        assert!(
            processing_paused(w, e),
            "{mode:?} fora do rect nao pausou o processamento"
        );
    }
}

/// ⭐ **`HideVisible` desce a subárvore** — está escrito no doc do `EnableMode` (*«the node and
/// subtree»*), e sem isto esconder um grupo deixava os filhos a desenhar no ar.
///
/// **Mutação:** apagar a caminhada de ancestrais de [`super::hides`] ⇒ RED no filho.
#[test]
fn hiding_a_parent_takes_the_whole_subtree_with_it() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let parent = spawn_at(w, "Cart", 100.0, 100.0);
    let child = spawn_at(w, "Wheel", 100.0, 100.0);
    w.entity_mut(child).insert(ChildOf(parent));

    assert!(
        !hides(w, child, [100.0, 100.0]),
        "o filho ja' estava escondido antes de o pai ganhar o enabler — o gate nao mede nada"
    );
    enable(w, parent, EnableMode::HideVisible);
    assert!(
        hides(w, child, [100.0, 100.0]),
        "o pai saiu do rect dele e o filho continuou a desenhar"
    );
}

/// ⭐⭐⭐ **A DIFERENÇA entre os dois modos de pausa é real** — e é a única coisa que impede que um
/// deles seja um controlo morto dentro de um enum de três.
///
/// `InheritPause` desce; `PauseProcessing` pára na entidade que o carrega.
///
/// **Mutação:** trocar o `== Some(InheritPause)` da caminhada de [`super::processing_paused`] por
/// um `matches!` dos dois modos ⇒ RED (o filho do `PauseProcessing` passa a pausar).
#[test]
fn inherit_pause_descends_and_pause_processing_stops_at_its_own_node() {
    for (mode, child_should_pause) in [
        (EnableMode::InheritPause, true),
        (EnableMode::PauseProcessing, false),
    ] {
        let mut sim = SimWorld::new();
        let w = sim.world_mut();
        let parent = spawn_at(w, "Cart", 100.0, 100.0);
        let child = spawn_at(w, "Wheel", 0.0, 0.0); // local (0,0) sobre o pai ⇒ mundo (100,100)
        w.entity_mut(child).insert(ChildOf(parent));
        enable(w, parent, mode);

        assert!(
            processing_paused(w, parent),
            "{mode:?}: o proprio no' nao pausou"
        );
        assert_eq!(
            processing_paused(w, child),
            child_should_pause,
            "{mode:?}: o filho respondeu ao contrario do que o modo promete"
        );
    }
}

/// ⚠️ **O ponto é o do PRÓPRIO nó, e o mundo é o que conta.** Um filho cuja pose LOCAL cai dentro
/// do rect mas cujo MUNDO cai fora tem de ser recusado — ler a pose local faria o enabler mentir em
/// toda hierarquia com um pai deslocado, que é o caso comum.
///
/// **Mutação:** trocar o `world_transform` de [`super::world_pos`] pelo `Transform` local ⇒ RED.
#[test]
fn the_rect_is_measured_against_the_world_pose_not_the_local_one() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let parent = spawn_at(w, "Ship", 100.0, 100.0);
    let child = spawn_at(w, "Flag", 5.0, 5.0); // LOCAL dentro do rect, MUNDO em (105,105)
    w.entity_mut(child).insert(ChildOf(parent));
    enable(w, child, EnableMode::PauseProcessing);

    assert!(
        processing_paused(w, child),
        "o filho foi julgado pela pose LOCAL: (5,5) esta' dentro do rect, mas ele esta' em \
         (105,105) no mundo"
    );
}

/// ⚠️ **Sem enabler nenhum, nada muda** — a metade que prova que esta cura é inerte na cena que
/// não a usa (e que o quadro continua byte-idêntico).
#[test]
fn an_entity_without_the_component_is_never_hidden_nor_paused() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let e = spawn_at(w, "Plain", 999.0, 999.0);
    assert!(!hides(w, e, [999.0, 999.0]));
    assert!(!processing_paused(w, e));
}
