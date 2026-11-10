//! ⭐ **As cenas da aba Input Map** (`docs/InputMap/01_plano.md` §8): o nível 2 com os dois leitores que o mapa do
//! dono pede — um Trigger cuja Action é `fire` e um script Luau que chama `ph2d.action_pressed("laser")`; o
//! `reload` fica sem leitor (o aviso âmbar da B4). O estado da aba é da família Input Map
//! (`ph2d_app_input_map::smoke::arma`, chamado pela shell com o código da cena): uma família não depende da outra.
//!
//! **Numeração** (contada no roteiro, `CENAS`): A1..A4 = 6..9 · B1..B4 = 10..13. A rodada seguinte toma
//! C1..C4 = 14..17 (receitas, ao vivo, manche, controles) e só DEPOIS P1 = 18 (o Play com a tela aberta): conta-se
//! de novo `CENAS` e acrescenta-se ao fim de [`CODIGOS`] por esta ordem, nunca com cena falsa a guardar lugar.

use ph2d_core::Vec2;
use ph2d_ecs::{ActionEdge, ActionTriggerRow, Name, SignalOnAction, SimWorld, Transform};
use ph2d_script::{LuauScript, ProjectScripts};

/// A primeira cena da aba no roteiro `PH2D_GAME_SMOKE`.
pub const PRIMEIRA: u32 = 6;

/// O código de cada cena, pela ordem do roteiro (o mesmo de `ph2d_app_input_map::smoke::CENAS`).
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

/// Os dois leitores na cena (o Trigger do `fire` e o objecto do script do `laser`) e a mensagem do terminal.
pub fn leitores(sim: &mut SimWorld, nivel: u32) {
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
}
