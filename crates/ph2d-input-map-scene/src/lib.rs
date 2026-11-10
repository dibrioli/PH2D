//! ⭐ **As cenas da aba Input Map** (`PH2D_GAME_SMOKE`, `docs/InputMap/01_plano.md` §8). O roteiro do jogo
//! (`ph2d_app_game::smoke`) numera-as e monta o nível 2; aqui moram o mapa do dono, os dois leitores que ele pede
//! (um Trigger cuja Action é `fire`, um script Luau com `ph2d.action_pressed("laser")`; o `reload` fica sem leitor,
//! o aviso âmbar da B4) e o estado da aba de cada cena, posto pelas portas `InputMapDoc::smoke_*`. A shell compõe:
//! acha a aba, chama [`arma`], instala o mapa e pede a aba (`request_input_map_tab`, o caminho do menu).
//!
//! **Numeração** (contada no roteiro, `CENAS`): A1..A4 = 6..9 · B1..B4 = 10..13. A rodada seguinte toma
//! C1..C4 = 14..17 (receitas, ao vivo, manche, controles) e só DEPOIS P1 = 18 (o Play com a tela aberta): conta-se
//! de novo `CENAS` e acrescenta-se ao fim de [`CODIGOS`] por esta ordem, nunca com cena falsa a guardar lugar.

use ph2d_core::Vec2;
use ph2d_ecs::{ActionEdge, ActionTriggerRow, Entity, Name, SignalOnAction, SimWorld, Transform};
use ph2d_input::{Binding, GamepadAxis, InputMap, Key};
use ph2d_input_map_editor::view_keyboard::state::Sub;
use ph2d_input_map_editor::{InputMapDoc, View};
use ph2d_script::{LuauScript, ProjectScripts};

/// A primeira cena da aba no roteiro `PH2D_GAME_SMOKE`.
pub const PRIMEIRA: u32 = 6;

/// O código de cada cena, pela ordem do roteiro.
pub const CODIGOS: [&str; 8] = ["A1", "A2", "A3", "A4", "B1", "B2", "B3", "B4"];

/// O que cada cena mostra (o terminal do smoke).
const O_QUE: [&str; 8] = [
    "A · List: as 11 acções do dono (fábrica + fire J, reload sem tecla, laser H)",
    "A · List: a escutar no reload (o chip tracejado espera a próxima tecla)",
    "A · List: busca pela seta ↑ (só jump e move_up; o pé diz quem as lê)",
    "A · List: move_left com a manche esquerda (Dead 0.15, Press 0.50 e a frase do pé)",
    "B · Keyboard: jump aberto (↑ e Z acesas, lido pelo Platform Player)",
    "B · Keyboard: o ponteiro sobre a ↑ (o pé: jump e move_up)",
    "B · Keyboard: o comando com o move_left aberto (a barra 0 · Dead · Press · 1)",
    "B · Keyboard: reload aberto (sem teclas; ninguém o lê)",
];

/// A zona da manche do move_left nas cenas com comando (maquetes A4 e B3).
pub const DEAD: f32 = 0.15;
pub const PRESS: f32 = 0.5;

pub const TRIGGER_NAME: &str = "Fire Trigger";
pub const LASER_NAME: &str = "laser.luau";
/// O script que dá ao `laser` um leitor (o Input Map lê-o pela biblioteca do projecto).
pub const LASER_LUAU: &str = r#"-- laser.luau — reads the «laser» action of the Input Map (H by default).
function update(self, dt)
  if ph2d.action_pressed("laser") then
    print("laser!")
  end
end
"#;

/// O código da cena `nivel`, se é uma cena da aba.
#[must_use]
pub fn codigo(nivel: u32) -> Option<&'static str> {
    let i = nivel.checked_sub(PRIMEIRA)?;
    CODIGOS.get(usize::try_from(i).ok()?).copied()
}

/// O mapa do dono: as 8 de fábrica + fire `J`, reload sem tecla, laser `H`; `manche` liga a manche esquerda (←) ao
/// move_left com Dead 0.15 e Press 0.50.
#[must_use]
pub fn owner_map(manche: bool) -> InputMap {
    let mut map = InputMap::with_player_defaults();
    for (name, key) in [("fire", Some('J')), ("reload", None), ("laser", Some('H'))] {
        let a = map.create(name);
        if let (Some(k), Some(act)) = (key, map.get_mut(a)) {
            act.bindings.push(Binding::Key(Key(u32::from(k))));
        }
    }
    let ml = map.id(ph2d_input::PLAYER_MOVE_LEFT);
    if let Some(a) = ml.and_then(|id| map.get_mut(id)).filter(|_| manche) {
        a.bindings.push(Binding::PadAxis {
            axis: GamepadAxis::LeftStickX,
            positive: false,
        });
        a.set_zone(DEAD, PRESS);
    }
    map
}

/// Monta o nível (`monta`, o nível 2 do roteiro) e os dois leitores; devolve o jogador.
pub fn leitores(
    sim: &mut SimWorld,
    nivel: u32,
    monta: impl FnOnce(&mut SimWorld) -> Entity,
) -> Entity {
    let jogador = monta(sim);
    let w = sim.world_mut();
    let row = ActionTriggerRow {
        action: "fire".to_owned(),
        edge: ActionEdge::Press,
        signal: "shoot".to_owned(),
        ..Default::default()
    };
    w.spawn((
        Name::new(TRIGGER_NAME),
        Transform::from_translation(Vec2::new(-6.0, 1.0)),
        SignalOnAction(vec![row]),
    ));
    let laser = w
        .get_resource_or_init::<ProjectScripts>()
        .insert(LASER_NAME, LASER_LUAU);
    w.spawn((
        Name::new("Laser"),
        Transform::from_translation(Vec2::new(-7.0, 1.0)),
        LuauScript::library(laser),
    ));
    ph2d_ecs::assign_missing_stable_ids(w);
    if let Some(i) = CODIGOS.iter().position(|c| Some(*c) == codigo(nivel)) {
        println!("[game-smoke] ={nivel} ({}) {}", CODIGOS[i], O_QUE[i]);
    }
    jogador
}

/// A tecla ↑ (a da busca A3 e do ponteiro B2).
#[must_use]
pub fn up_arrow() -> Option<Key> {
    Key::named_in_script("up").and_then(|v| v.first().copied())
}

/// Põe a aba no estado da cena `cena` e devolve o mapa que a shell instala no `hero.input_map`. `None` = código
/// desconhecido.
#[must_use]
pub fn arma(doc: &mut InputMapDoc, cena: &str) -> Option<InputMap> {
    if !CODIGOS.contains(&cena) {
        return None;
    }
    let map = owner_map(matches!(cena, "A4" | "B3"));
    let id = |n: &str| map.id(n);
    let (jump, move_left, reload) = (id("jump"), id(ph2d_input::PLAYER_MOVE_LEFT), id("reload"));
    let lista = cena.starts_with('A');
    doc.smoke_view(if lista { View::List } else { View::Keyboard });
    match cena {
        "A2" => reload.into_iter().for_each(|r| doc.smoke_listen(r)),
        "A3" => doc.smoke_search_key(up_arrow()),
        "B1" => doc.smoke_open(jump),
        "B2" => {
            doc.smoke_open(jump);
            doc.smoke_hover_key(up_arrow());
        }
        "B3" => {
            doc.smoke_open(move_left);
            doc.smoke_sub(Sub::Gamepad);
        }
        "B4" => doc.smoke_open(reload),
        _ => {}
    }
    Some(map)
}
