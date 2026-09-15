//! **ARCH-GATE do fio da entrada do player** (W3).
//!
//! A política de teclas é pura e tem gates próprios (`ph2d_app_physics::player_input`), e a
//! lei tem os dela na `ph2d-platformer`. O que **nenhum dos dois alcança** é o
//! FIO: um `winit::KeyEvent` não pode ser construído fora do winit (a parede que
//! fez o corpo do `on_keyboard_input` virar `key_input`), e o `render_loop` exige
//! janela. Com as duas pontas certas e o meio desligado, tudo fica verde e o
//! personagem não anda.
//!
//! Três afirmações, e cada uma é uma forma diferente de o fio nascer morto.

use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// **(1) A tecla é OBSERVADA, e o bloco que a observa não CONSOME.**
///
/// A seta já tem dono (o nudge de nó do Vector), então roubá-la aqui regrediria
/// uma ferramenta que ninguém pediu para mexer. *"Observa"* e *"observa e
/// engole"* são indistinguíveis em qualquer teste que não tenha uma janela, então
/// o oráculo é a **ausência de `return`** dentro do bloco.
///
/// ⚠️ **A versão original pinava as TRÊS LINHAS do bloco, verbatim**, e nasceu
/// vermelha no dia em que a observação ganhou a guarda de acorde do gate
/// seguinte — o fio estava intacto e o gate reprovava uma correção. *Uma cópia
/// literal do corpo é um proxy que expira*; o que este gate afirma é a
/// PROPRIEDADE.
#[test]
fn the_walk_keys_are_observed_without_being_consumed() {
    let src = crate::input_text::keyboard();
    let (open, call) = observation_block(&src);
    assert!(
        !src[open..call].contains("return"),
        "o bloco que observa as teclas de caminhada NAO pode consumir a tecla — \
         um `return` ali dentro rouba a seta do nudge do Vector"
    );
}

/// **⚠️ (1b) Um ACORDE nunca é entrada de jogo.**
///
/// Report do Enio (cena 112): *"os players pulam e se movem sozinhos"*. As seis
/// teclas que o dedo do jogador reclama moram debaixo de atalhos que o artista
/// aperta o tempo todo — **Ctrl+Z** punha `jump`, **Ctrl+A** `left`, **Ctrl+D**
/// `right`, **Ctrl+S** `down`.
///
/// ⚠️ **A varredura de conflito do `player_input` foi feita e ainda assim
/// errou**, porque mediu a tecla NUA: o doc do `KeyS` afirma que *"um botão de
/// player nunca vê aquele caminho"* porque o Ctrl+S corre dentro de um braço
/// guardado por modificador. Ele vê — esta observação corre **antes** de toda
/// guarda de modificador do arquivo.
///
/// ⚠️ **E o RELEASE tem de passar SEMPRE.** Um guard simétrico troca um pulo
/// espúrio por um personagem que **anda sozinho para sempre**: a tecla apertada
/// sem modificador e solta com o Ctrl preso nunca seria desarmada.
#[test]
fn a_modifier_chord_never_reaches_the_player() {
    let src = crate::input_text::keyboard();
    let (open, call) = observation_block(&src);
    let block = &src[open..call];
    for probe in ["control_key()", "alt_key()", "super_key()"] {
        assert!(
            block.contains(probe),
            "a observacao tem de recusar um acorde: falta `{probe}` no bloco"
        );
    }
    assert!(
        block.contains("if !pressed || !chord {"),
        "o RELEASE tem de passar SEMPRE (`!pressed ||`), senao um Ctrl preso na hora \
         de soltar deixa a tecla armada para sempre"
    );
}

/// O bloco que observa a tecla física: `(início do `if let`, a chamada)`.
fn observation_block(src: &str) -> (usize, usize) {
    // ⚠️ **RE-ANCORADO em 2026-08-24 (plano 30 W5).** A observação deixou de escrever num
    // `PlayerKeys` da shell e passa a alimentar o retrato de dispositivos que o Input Map resolve.
    // A LEI é a mesma (observar sem consumir, recusar acordes); o que mudou foi o endereço.
    let call = src
        .find("crate::keymap::winit_to_input_keycode(code)")
        .expect("o dedo do jogador tem de OBSERVAR a tecla fisica");
    let open = src[..call]
        .rfind("if let PhysicalKey::Code(code) = physical_key {")
        .expect("a observacao mora dentro de um `if let` sobre a tecla fisica");
    (open, call)
}

/// **(2) O dedo INTEIRO chega ao dispatch da física.**
///
/// Sem esta linha o `PlayerKeys` seria estado que ninguém lê: os gates dele
/// continuariam verdes e o personagem ficaria parado.
///
/// ⚠️ A âncora é a **porta única** (`input()`), não `drive()` — e a diferença não
/// é cosmética: com dois getters, um dispatch que entrega só metade do dedo é
/// um build que compila, e o pulo seria a metade esquecida.
#[test]
fn the_whole_finger_is_handed_to_the_physics_dispatch() {
    // ⚠️ **O QUADRO lê-se no texto EMENDADO** (OBRA 2, 2026-09-12): a resolução mudou-se para a fase
    // `fase_pointer_subjects.rs`, que continua a ser o topo do quadro — é o que a agulha afirma.
    let src = crate::frame_text::render_frame();
    // ⚠️ **RE-ANCORADO DUAS vezes.** (W5) a porta passou a ser `App::resolve_player_input`;
    // (W2/L2 Fase C, 2026-09-12) ela deixou de ser um método e passou a ser
    // `ph2d_app_physics::player_input::resolve_player_input`, uma função livre sobre os três tipos
    // que a `App` por acaso segurava. ⭐ **A agulha nova é MELHOR**, e é a §2.13 do HOWTO a
    // funcionar: ela nomeia a PORTA e a propriedade — *um binding, um valor, no topo do quadro* —
    // em vez de nomear quem a chama. Um `self.` na agulha mede o receptor, e o receptor é
    // exactamente o que uma fronteira nova muda por construção.
    assert!(
        src.contains("let player_input = ph2d_app_physics::player_input::resolve_player_input("),
        "o `render_loop` tem de RESOLVER o dedo do jogador a partir do Input Map, no topo do quadro"
    );
}

/// **(3) A entrega acontece ANTES da decisão de HOLD, não depois.**
///
/// ⚠️ Esta é a que não se adivinha. Com a simulação desarmada o mundo é
/// **segurado** (`PhysicsBridge::hold`) e o `dispatch` retorna cedo; entregar a
/// entrada depois desse `return` faria a tecla que o artista já estava segurando
/// ser **engolida** no instante em que ele arma o Physics — o personagem só
/// andaria depois de soltar e apertar de novo.
///
/// A afirmação é POSICIONAL sobre duas âncoras que descrevem o que o código faz
/// (a entrega e o early-out), nunca uma distância em bytes.
///
/// ⚠️ **E a âncora é a CHAMADA, não a lista de argumentos** — a versão original
/// pinava `hand_input_to_players(bridge, sim, drive);` inteiro e nasceu VERMELHA
/// na W4, quando o argumento virou `input`: o fio estava intacto e o gate
/// reprovava um rename. *Uma lista de argumentos é um proxy que expira*, e o que
/// este gate afirma é ORDEM.
///
/// ⚠️ Procurar só o NOME também não serve: ele casa com a **definição** do `fn`,
/// e no dia em que ela subisse para cima do `dispatch` o gate compararia a
/// posição errada e ficaria verde sobre o fio invertido.
#[test]
fn the_input_is_handed_over_before_the_hold_early_out() {
    let src = read("../../crates/ph2d-app-physics/src/bridge/dispatch.rs");
    let hand = src
        .match_indices("hand_input_to_players(")
        .find(|(at, _)| !src[..*at].ends_with("fn "))
        .map(|(at, _)| at)
        .expect("o dispatch tem de CHAMAR a entrega da entrada aos players");
    let hold = src
        .find("if !simulate {")
        .expect("o dispatch tem de ter o early-out do hold");
    assert!(
        hand < hold,
        "a entrada tem de ser entregue ANTES do early-out do hold: \
         entrega em {hand}, early-out em {hold}"
    );
}

/// **Controle positivo** — as três âncoras existem no arquivo que o gate lê.
///
/// Sem ele, renomear um arquivo (ou um gate que procura no lugar errado) deixaria
/// as três asserções acima verdes por vácuo, que é o modo de falha canônico de um
/// gate que lê fonte.
#[test]
fn the_files_the_gate_reads_are_the_ones_that_carry_the_wire() {
    assert!(crate::input_text::keyboard().contains("winit_to_input_keycode"));
    assert!(
        crate::frame_text::render_frame().contains("ph2d_app_physics::bridge::dispatch::dispatch(")
    );
    assert!(
        read("../../crates/ph2d-app-physics/src/bridge/dispatch.rs")
            .contains("fn hand_input_to_players")
    );
}

/// ⭐⭐⭐ **(4) E A CENA QUE O DONO CORRE DE FACTO ANDA** — o fio inteiro, sem agulha textual.
///
/// # ⛔⛔ Por que este teste existe (report do dono, 2026-09-15: *«nada se move»*)
///
/// Os três gates acima são **textuais**, e nenhum deles pode reprovar sobre o defeito que o dono
/// encontrou: eles afirmam que o quadro **chama** o `resolve_player_input` e que ele **entrega** o
/// dedo ao dispatch, e as duas coisas eram verdade. O que estava partido vivia **dentro** do
/// dispatch — a pergunta *«quem são os players?»* varria só o [`PlatformPlayer`], logo o dedo
/// chegava e não tinha a quem ser entregue.
///
/// ⚠️ *Uma agulha que nomeia a CHAMADA é cega ao corpo dela.* ⇒ este gate monta a **cena do smoke**
/// (a mesma que o dono corre), segura uma seta, e corre a **porta do produto** — e só está verde se
/// o boneco ANDAR.
///
/// ⛔ Ele não pode viver na crate da cena nem na da ponte: a cena vive na `ph2d-app-components` e o
/// dispatch na `ph2d-app-physics`, e **nenhuma depende da outra**. A shell é quem as vê às duas —
/// que é exactamente o que «a shell é composição» quer dizer.
#[test]
fn the_top_down_smoke_scene_actually_walks() {
    use ph2d_core::Playhead;
    use ph2d_ecs::SimWorld;

    const DT: f64 = 1.0 / 60.0;
    for nivel in 1..=2u32 {
        let mut sim = SimWorld::new();
        ph2d_app_components::topdown_smoke::montar(sim.world_mut(), nivel);

        // A pose de partida de cada boneco, por NOME — a cena `=2` tem dois.
        let antes = poses(&sim);
        assert!(
            !antes.is_empty(),
            "a cena =${nivel} nao tem boneco nenhum para medir"
        );

        let mut bridge = ph2d_physics_ecs::PhysicsBridge::new();
        let mut doc = ph2d_timeline::TimelineDoc::new();
        let mut playhead = Playhead::new(DT);
        let mut tape = ph2d_physics_ecs::InputTape::new();
        let mut drive = ph2d_preview_drive::PreviewDrive::default();
        playhead.play();
        // ⚠️ **A seta DIREITA segurada**, que é o passo 3 da instrução que o dono recebe.
        let dedo = ph2d_physics_ecs::PlayerInput {
            drive: 1.0,
            ..ph2d_physics_ecs::PlayerInput::default()
        };
        for _ in 0..30 {
            playhead.advance();
            ph2d_app_physics::bridge::dispatch::dispatch(
                &mut bridge,
                &mut sim,
                &playhead,
                DT,
                &mut doc,
                // ⚠️ Armado — é o que o prólogo da cena faz (`components_scenes::topdown_smoke`).
                true,
                dedo,
                &mut tape,
                &mut drive,
            );
        }

        let depois = poses(&sim);
        for (nome, p0) in &antes {
            let p1 = depois.get(nome).expect("o boneco continua na cena");
            let andou = ((p1.x - p0.x).powi(2) + (p1.y - p0.y).powi(2)).sqrt();
            assert!(
                andou > 0.5,
                "na cena =${nivel} o `{nome}` andou {andou:.4} m em meio segundo com a seta \
                 SEGURADA — o dedo nao chega ao mover.\n\
                 ⚠️ Se isto e' ZERO, a pergunta «quem le o teclado?» voltou a ter duas respostas."
            );
        }
    }
}

/// A pose de cada mover de vista de cima, por nome.
fn poses(sim: &ph2d_ecs::SimWorld) -> std::collections::BTreeMap<String, ph2d_core::Vec2> {
    let mut out = std::collections::BTreeMap::new();
    let Some(mut q) = sim.world().try_query::<(
        &ph2d_ecs::Name,
        &ph2d_ecs::Transform,
        &ph2d_physics_ecs::TopDownPlayer,
    )>() else {
        return out;
    };
    for (n, t, _) in q.iter(sim.world()) {
        out.insert(n.as_str().to_string(), t.translation);
    }
    out
}
