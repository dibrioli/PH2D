//! ⭐ **As cenas da aba Input Map** (`PH2D_GAME_SMOKE`, `docs/InputMap/01_plano.md` §8). O roteiro do jogo
//! (`ph2d_app_game::smoke`) numera-as e monta o nível 2; aqui moram o mapa do dono, os dois leitores que ele pede
//! (um Trigger cuja Action é `fire`, um script Luau com `ph2d.action_pressed("laser")`; o `reload` fica sem leitor,
//! o aviso âmbar da B4) e o estado da aba de cada cena, posto pelas portas `InputMapDoc::smoke_*`. A shell compõe:
//! acha a aba, chama [`arma`], instala o mapa e pede a aba (`request_input_map_tab`, o caminho do menu).
//!
//! **Numeração** (contada no roteiro, `CENAS` = 23; uma cena nova conta-o de novo e entra ao fim de [`CODIGOS`]):
//!
//! | `PH2D_GAME_SMOKE` | Cena | Estado |
//! |---|---|---|
//! | 6 | A1 | List: as 11 acções do dono |
//! | 7 | A2 | List: à escuta no reload |
//! | 8 | A3 | List: busca pela ↑ (jump e move_up) |
//! | 9 | A4 | List: move_left com a manche esquerda (Dead 0.15, Press 0.50) |
//! | 10 | B1 | Keyboard: jump aberto |
//! | 11 | B2 | Keyboard: o ponteiro sobre a ↑ |
//! | 12 | B3 | Keyboard: o comando com o move_left aberto |
//! | 13 | B4 | Keyboard: reload aberto (sem teclas, sem leitor) |
//! | 14 | C1 | Try it: as quatro receitas, nenhuma pergunta aberta |
//! | 15 | C2 | Try it: a ↑ em baixo na bancada (jump e move_up a 1.00, o pé nomeia-as) |
//! | 16 | C3 | Try it: a manche esquerda a −0.62 ([`manche`], só no smoke): Pushed 0.62 · Strength 0.55 · Pressed |
//! | 17 | C4 | Try it: «Players can change keys», só [`NA_TELA`] no jogo, um ficheiro do jogador com 1 troca |
//! | 18 | P1 | Play (com `PH2D_GAME_SMOKE_DRIVE=editor`): a tela Controls aberta, Fire, 2.ª tecla à escuta ([`tecla_do_p1`]) |
//! | 19 | B5 | Keyboard: o numérico — fire também no Numpad 0, move_up no Numpad 8; fire aberto, o ponteiro no Numpad 0 |
//! | 20 | P2 | Play (`DRIVE=editor`): a tela Controls aberta, o ponteiro sobre a linha Jump ([`rato_do_p`]) |
//! | 21 | P3 | Play: o «Reset to defaults» premido (Down sem Up) |
//! | 22 | P4 | Play: o «Reset to defaults» premido e arrastado para fora (o premido apaga) |
//! | 23 | P5 | Play (`DRIVE=editor` ou `fullscreen`): a tela aberta, o ponteiro no botão «Controls» da barra ([`rato_do_p5`]) |

use ph2d_core::Vec2;
use ph2d_ecs::{ActionEdge, ActionTriggerRow, Entity, Name, SignalOnAction, SimWorld, Transform};
use ph2d_editor_core::screens::hero::{HeroScreen, transport_bar};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_input::{Binding, GamepadAxis, InputMap, Key, PlayerKeys};
use ph2d_input_map_editor::view_keyboard::state::Sub;
use ph2d_input_map_editor::view_try::state::TryCard;
use ph2d_input_map_editor::{InputMapDoc, View};
use ph2d_player_controls::{PlaySession, host};
use ph2d_script::{LuauScript, ProjectScripts};

/// A primeira cena da aba no roteiro `PH2D_GAME_SMOKE`.
pub const PRIMEIRA: u32 = 6;

/// O código de cada cena, pela ordem do roteiro.
pub const CODIGOS: [&str; 18] = [
    "A1", "A2", "A3", "A4", "B1", "B2", "B3", "B4", "C1", "C2", "C3", "C4", "P1", "B5", "P2", "P3",
    "P4", "P5",
];

/// O que cada cena mostra (o terminal do smoke).
const O_QUE: [&str; 18] = [
    "A · List: as 11 acções do dono (fábrica + fire J, reload sem tecla, laser H)",
    "A · List: a escutar no reload (o chip tracejado espera a próxima tecla)",
    "A · List: busca pela seta ↑ (só jump e move_up; o pé diz quem as lê)",
    "A · List: move_left com a manche esquerda (Dead 0.15, Press 0.50 e a frase do pé)",
    "B · Keyboard: jump aberto (↑ e Z acesas, lido pelo Platform Player)",
    "B · Keyboard: o ponteiro sobre a ↑ (o pé: jump e move_up)",
    "B · Keyboard: o comando com o move_left aberto (a barra 0 · Dead · Press · 1)",
    "B · Keyboard: reload aberto (sem teclas; ninguém o lê)",
    "C · Try it: as quatro receitas, nenhuma pergunta aberta",
    "C · Try it: a ↑ em baixo na bancada (jump e move_up a 1.00; o pé diz quem a lê)",
    "C · Try it: a manche esquerda a 0.62 para a esquerda (Pushed 0.62 · Strength 0.55 · Pressed)",
    "C · Try it: a tela do jogador (Players can change keys; jump, move_left e fire à vista; 1 tecla trocada)",
    "P · Play: a tela Controls aberta no jogo, à escuta na 2.ª tecla do Fire (`PH2D_GAME_SMOKE_DRIVE=editor`)",
    "B · Keyboard: o numérico (fire também no Numpad 0, move_up no Numpad 8; fire aberto, o ponteiro no Numpad 0)",
    "P · Play: a tela Controls aberta, o ponteiro sobre a linha Jump (a linha acende)",
    "P · Play: a tela Controls aberta, o botão esquerdo premido sobre «Reset to defaults» (sem soltar)",
    "P · Play: o «Reset to defaults» premido e o ponteiro arrastado para fora dele (o premido apaga; soltar ali cancela)",
    "P · Play: a tela Controls aberta, o ponteiro no botão Controls da barra (o botão acende, nenhuma linha da tela)",
];

/// A manche esquerda das cenas C3: empurrada a 0.62 para a esquerda (maquete C·3: força (0.62 − 0.15) ÷ 0.85).
pub const MANCHE_C3: f32 = -0.62;
/// O `game_id` do mundo das cenas C4 e P1 (o ficheiro do jogador mora em `~/.ph2d`, por jogo).
pub const GAME_ID: u64 = 0x1A_9E3C;
/// As acções que a tela do jogador mostra nas cenas C4 e P1 (as da maquete C·4), pela ordem do mapa.
pub const NA_TELA: [&str; 3] = [ph2d_input::PLAYER_MOVE_LEFT, "jump", "fire"];
/// A troca do jogador deste computador (C4): o fire ganha a `K` como 2.ª tecla — uma acção trocada.
pub const TECLA_DO_JOGADOR: char = 'K';

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
    if codigo(nivel).is_some_and(|c| c == "C4" || no_play(c)) {
        w.insert_resource(InputMapDoc::smoke_game_id(GAME_ID));
    }
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
    let mut map = owner_map(matches!(cena, "A4" | "B3" | "C3"));
    if cena == "C4" || no_play(cena) {
        tela_do_jogador(&mut map);
    }
    let id = |n: &str| map.id(n);
    let (jump, move_left, reload) = (id("jump"), id(ph2d_input::PLAYER_MOVE_LEFT), id("reload"));
    doc.smoke_view(match cena.as_bytes().first() {
        Some(b'A') => View::List,
        Some(b'B') => View::Keyboard,
        _ => View::TryIt,
    });
    match cena {
        "C2" => doc.smoke_bench_key(up_arrow()),
        "C3" => doc.smoke_reveal(TryCard::Stick),
        "C4" => {
            doc.smoke_player_keys(troca_do_jogador(&map));
            doc.smoke_reveal(TryCard::Controls);
        }
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
        "B5" => {
            let kp = |n: &str| Key::named_in_script(n).map(|v| v[0]);
            for (action, key) in [("fire", kp("kp0")), ("move_up", kp("kp8"))] {
                if let (Some(a), Some(k)) = (map.id(action), key)
                    && let Some(act) = map.get_mut(a)
                {
                    act.bindings.push(Binding::Key(k));
                }
            }
            doc.smoke_open(map.id("fire"));
            doc.smoke_hover_key(kp("kp0"));
        }
        _ => {}
    }
    Some(map)
}

/// «Players can change keys» ligado e só [`NA_TELA`] à vista no jogo (C4, P1).
fn tela_do_jogador(map: &mut InputMap) {
    let vistas: Vec<_> = (map.actions().iter())
        .map(|a| (a.id, NA_TELA.contains(&a.name.as_str())))
        .collect();
    let c = map.controls_mut();
    c.players_can_change_keys = true;
    for (id, vista) in vistas {
        c.set_shown_in_game(id, vista);
    }
}

/// As teclas de um jogador que trocou UMA acção: o fire com [`TECLA_DO_JOGADOR`] a seguir à do jogo.
#[must_use]
pub fn troca_do_jogador(map: &InputMap) -> PlayerKeys {
    let mut keys = PlayerKeys::new();
    if let Some(fire) = map.id("fire").and_then(|id| map.get(id)) {
        let mut b = fire.bindings.clone();
        b.push(Binding::Key(Key(u32::from(TECLA_DO_JOGADOR))));
        keys.set("fire", b);
    }
    keys
}

/// A aba abre-se por cima da vista? Nas do Play (P1..P4) não: o jogo corre na vista da cena.
#[must_use]
pub fn abre_a_aba(cena: &str) -> bool {
    !no_play(cena)
}

/// As cenas do Play (P1..P4): a tela Controls do jogador, no jogo.
fn no_play(cena: &str) -> bool {
    cena.starts_with('P')
}

/// O eixo e o valor que a cena empurra no comando ao vivo (só a C3; o smoke não tem comando de verdade).
#[must_use]
pub fn manche(nivel: u32) -> Option<(GamepadAxis, f32)> {
    (codigo(nivel) == Some("C3")).then_some((GamepadAxis::LeftStickX, MANCHE_C3))
}

/// O quadro de cada tecla do condutor na P1 (o Play entra ao 60): abre a tela, desce ao Fire, passa à 2.ª tecla e
/// põe-na à escuta.
const TECLAS_P1: [(u32, &str); 5] = [
    (150, ""),
    (160, "down"),
    (170, "down"),
    (180, "right"),
    (190, "enter"),
];

/// A tecla que o condutor da P1 prime no `quadro` (`""` = a que abre a tela, a da fábrica); nas P2..P4 só a que abre.
#[must_use]
pub fn tecla_do_p1(nivel: u32, quadro: u32) -> Option<Key> {
    let teclas = match codigo(nivel)? {
        "P1" => &TECLAS_P1[..],
        c if no_play(c) => &TECLAS_P1[..1],
        _ => return None,
    };
    let (_, nome) = teclas.iter().find(|(q, _)| *q == quadro)?;
    if nome.is_empty() {
        return Some(ph2d_input::controls::PlayerControls::default().open_key);
    }
    Key::named_in_script(nome).and_then(|v| v.first().copied())
}

/// O quadro em que o condutor das P2..P4 põe o ponteiro (a tela abriu ao 150 e já foi pintada) e o da P4 o arrasta.
const RATO_P: (u32, u32) = (160, 170);

/// O ponteiro do condutor das P2..P4, pela porta do host (a do rato real, coordenadas lógicas): a P2 pousa-o no
/// centro da linha Jump; a P3 prime o botão principal no «Reset to defaults» (sem soltar); a P4 prime-o lá e, um
/// passo depois, arrasta-o para fora do botão, uma largura dele à direita. Devolve o ponto posto (o log da foto).
pub fn rato_do_p(
    nivel: u32,
    quadro: u32,
    s: &mut PlaySession,
    map: &InputMap,
) -> Option<(f32, f32)> {
    let c = codigo(nivel).filter(|c| matches!(*c, "P2" | "P3" | "P4"))?;
    let lay = s.layout()?;
    let centro = |r: Rect| (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let jump = s.rows(map).iter().position(|r| r.action == "jump");
    let linha = jump.and_then(|i| lay.rows.get(i.checked_sub(lay.first)?));
    let (reset, fora) = (centro(lay.reset), lay.reset.x + lay.reset.w * 1.5);
    let ponto = match (c, quadro) {
        ("P2", q) if q == RATO_P.0 => {
            let at = centro(linha?.band);
            let _ = host::pointer_move(s, Some(at));
            at
        }
        ("P3" | "P4", q) if q == RATO_P.0 => {
            let _ = host::click(s, (true, true), Some((reset, map)), false);
            reset
        }
        ("P4", q) if q == RATO_P.1 => {
            let at = (fora, reset.1);
            let _ = host::pointer_move(s, Some(at));
            at
        }
        _ => return None,
    };
    println!(
        "[game-smoke] ={nivel} ({c}) o ponteiro em ({:.0}, {:.0})",
        ponto.0, ponto.1
    );
    Some(ponto)
}

/// O movimento do condutor da P5 no [`RATO_P`]`.0`: o ponteiro no centro do botão «Controls» da barra (o rect que a
/// barra registou no último quadro — o da barra do Play no editor, o do palco na tela cheia), em píxeis FÍSICOS, para a
/// porta única do movimento da shell (`pointer_moved`: a barra fica com ele, a tela solta o hover e o hero acende-o).
#[must_use]
pub fn rato_do_p5(nivel: u32, quadro: u32, hero: &HeroScreen) -> Option<PointerEvent> {
    if codigo(nivel) != Some("P5") || quadro != RATO_P.0 {
        return None;
    }
    let r = hero.hit_index.rect_for(transport_bar::CONTROLS.0)?;
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    println!("[game-smoke] ={nivel} (P5) o ponteiro no Controls em ({x:.0}, {y:.0})");
    let s = hero.escala().factor();
    Some(PointerEvent {
        x: x * s,
        y: y * s,
        pressure: 1.0,
        kind: PointerKind::Move,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    })
}
