//! **Os gates dos SINAIS da §11** (spec §8.10) — [`super::tick_sprite_animations`].
//!
//! ⚠️ Eles correm o tique REAL, que é o produtor do produto: o que a shell publica no outbox é
//! exactamente o que estas funções devolvem. Um gate que encenasse a produção mediria a encenação.

use super::tick_sprite_animations;
use crate::preview_drive::PreviewDrive;
use ph2d_ecs::{AnimationTag, Entity, SimWorld, SpriteAnimations, SpriteAnimator, Transform};
use ph2d_render::Sprite;

const DT: f64 = 1.0 / 60.0;

/// Uma sprite de 4 células com uma animação a tocar, e os nomes de sinal que se pedir.
fn scene(finish: &str, on_loop: &str, repeat: Option<u32>) -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag {
        // 4 células a 50 ms = 200 ms por volta.
        frame_ms: 50,
        repeat,
        signal_on_finish: finish.to_owned(),
        signal_on_loop: on_loop.to_owned(),
        ..AnimationTag::new("walk", 0, 3)
    })
    .unwrap();
    let mut player = SpriteAnimator::new("walk");
    player.playing = true;
    let e = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            ph2d_ecs::SpriteGrid {
                hframes: 4,
                vframes: 1,
                frame: 0,
            },
            player,
            lib,
        ))
        .id();
    (sim, e)
}

/// Corre `ticks` passos fixos num só tique e devolve os sinais.
fn run(sim: &mut SimWorld, ticks: u32, painted: &[Option<u64>]) -> Vec<(String, u32)> {
    let mut drive = PreviewDrive::default();
    tick_sprite_animations(sim, ticks, DT, painted, &mut drive)
        .into_iter()
        .map(|s| (s.name, s.cycles))
        .collect()
}

/// **UMA ANIMAÇÃO CALADA NÃO FALA** — e é o default de toda animação.
///
/// ⛔ Uma que grita por omissão enche o canal de eventos que ninguém pediu, e o consumidor
/// (hoje um toast, amanhã som) não tem como os distinguir dos que importam.
///
/// **Mutação que deve sangrar:** tirar o `!tag.signal_on_loop.is_empty()`.
#[test]
fn an_animation_with_no_names_says_nothing() {
    let (mut sim, _) = scene("", "", None);
    // 60 tiques = 1 s = cinco voltas. Nada.
    assert!(run(&mut sim, 60, &[]).is_empty());
}

/// **FECHAR UM CICLO GRITA O NOME AUTORADO** — e o nome é o contrato inteiro (ADR-0143).
#[test]
fn closing_a_cycle_shouts_the_authored_name() {
    let (mut sim, _) = scene("", "footstep", None);
    // Uma volta são 200 ms; 15 tiques são 250 ms.
    let out = run(&mut sim, 15, &[]);
    assert_eq!(out.len(), 1, "esperava UM sinal, veio {out:?}");
    assert_eq!(out[0].0, "footstep");
    assert_eq!(out[0].1, 1, "uma volta = um ciclo");
}

/// **UM TIQUE ATRASADO COLAPSA NUM SINAL SÓ, com a contagem dentro.**
///
/// ⚠️ É a lei que o `SignalOrigin::Motion` já escreveu: *o colapso é lossy e o número é o que ele
/// descarta*. Publicar um sinal por ciclo daria uma **rajada de passos que ninguém deu** quando a
/// janela esteve parada — e uma rajada é ruído, não um efeito.
///
/// **Mutação que deve sangrar:** empurrar `outcome.looped` sinais em vez de um.
#[test]
fn a_stalled_tick_collapses_into_one_signal_that_counts() {
    let (mut sim, _) = scene("", "footstep", None);
    // 60 tiques de uma vez = 1 s = CINCO voltas de 200 ms.
    let out = run(&mut sim, 60, &[]);
    assert_eq!(
        out.len(),
        1,
        "cinco voltas tem de dar UM sinal, veio {out:?}"
    );
    assert_eq!(out[0].1, 5, "…com a contagem dentro");
}

/// **ACABAR É OUTRO NOME**, e uma animação de uma volta só o grita uma vez.
///
/// ⚠️ As duas metades: sem a segunda, um `finished` que se repetisse a cada quadro depois do fim
/// daria um evento por quadro para sempre — e o transporte fica parado ali.
///
/// **Mutação que deve sangrar:** publicar o `finish` com o nome do `loop`.
#[test]
fn finishing_is_a_different_name_and_it_fires_once() {
    let (mut sim, _) = scene("attack_done", "footstep", Some(1));
    let out = run(&mut sim, 15, &[]);
    let names: Vec<&str> = out.iter().map(|(n, _)| n.as_str()).collect();
    assert!(
        names.contains(&"attack_done"),
        "a animacao de uma volta tinha de gritar o fim: {out:?}"
    );
    // E depois disso ela está parada: mais tiques não produzem nada.
    assert!(
        run(&mut sim, 60, &[]).is_empty(),
        "o `finished` repetiu-se depois de a animacao ter acabado"
    );
}

/// **A PRÉ-VISUALIZAÇÃO É MUDA.** Uma folha em pintura toca sobre uma **cópia** do animador — ela
/// existe para mostrar o movimento, não para fazer acontecer coisas na cena.
///
/// ⛔ Sem isto, pegar no pincel tocaria um som.
///
/// **Mutação que deve sangrar:** tirar o `if !preview_only`.
#[test]
fn a_painted_preview_is_silent() {
    let (mut sim, e) = scene("", "footstep", None);
    // Pausada: só a pré-visualização a faz andar.
    if let Some(mut a) = sim.world_mut().get_mut::<SpriteAnimator>(e) {
        a.playing = false;
    }
    let painted = [Some(e.to_bits())];
    let out = run(&mut sim, 60, &painted);
    assert!(out.is_empty(), "a pre-visualizacao falou: {out:?}");
    // CONTROLO POSITIVO: ela ANDOU — senão o gate mede o vazio.
    assert!(
        sim.world().get::<SpriteAnimator>(e).unwrap().elapsed_ticks > 0,
        "a pre-visualizacao nem andou — o gate acima nao mediu nada"
    );
}

/// **E o mesmo animador, a tocar de verdade, FALA** — o par do gate acima, e o que impede a cura
/// de ser «nunca publicar».
#[test]
fn the_same_animation_playing_for_real_does_speak() {
    let (mut sim, e) = scene("", "footstep", None);
    let painted = [Some(e.to_bits())];
    // A tocar E sob pintura: a condução é da cena, não da pré-visualização.
    assert_eq!(run(&mut sim, 15, &painted).len(), 1);
}
