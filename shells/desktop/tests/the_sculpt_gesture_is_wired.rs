//! **Arch-gates do gesto de escultura** (ADR-0150 W2).
//!
//! ⚠️ Por que a fonte e não o comportamento: a cena 3D só existe com um
//! `wgpu::Device` vivo (`AppGfx.sculpt3d`), então nenhum teste headless a
//! constrói e nenhum teste de unidade das crates alcança esta fiação. É o mesmo
//! motivo pelo qual o gizmo de âncora da `line/physics` e a decisão de upload do
//! Painter têm arch-gate: *um gate de unidade é cego à fiação do shell*.
//!
//! Cada asserção abaixo tem uma mutação que a derruba, listada no handoff.
mod sculpt_source;
use sculpt_source::{braced_block, function_body, match_arm, sculpt_src, source};

#[test]
fn the_left_button_sculpts_where_it_hits_and_orbits_where_it_misses() {
    let body = function_body(&sculpt_src(), "sculpt3d_pointer_down");
    assert!(
        body.contains("stroke.begin("),
        "o pen-down tem de CONGELAR o `pre` — sem isso a lei do traço não começa"
    );
    // ⚠️ A agulha é o `if took`, e não `if scene.sculpt_at(`: com o Grab a
    // decisão passou a ter DUAS portas de pick (quem puxa PEGA, quem carimba
    // CARIMBA), e um gate ancorado numa delas ficou vermelho sobre produto
    // correto — a terceira vez nesta sessão que um proxy expirou.
    assert!(
        body.contains("scene.take_hold(pos.0, pos.1)")
            && body.contains("scene.sculpt_at(pos.0, pos.1)"),
        "as duas portas de pick têm de ser tentadas conforme o verbo PUXA ou não"
    );
    let hit = body
        .find("if took {")
        .expect("o Down decide pelo RESULTADO do pick, seja qual for a porta");
    let sculpt = body[hit..]
        .find("Drag::Sculpt")
        .expect("o ramo que ACERTA abre um traço");
    let orbit = body[hit..]
        .find("Drag::Orbit")
        .expect("o ramo que ERRA cai na órbita");
    // Arrastar no vazio é o gesto mais comum do mundo; se ele não fizesse nada,
    // o artista concluiria que a cena travou. É o que o SculptGL faz.
    assert!(
        sculpt < orbit,
        "os ramos do pick estão trocados: quem acerta é que esculpe"
    );
}

#[test]
fn the_modifiers_are_read_once_at_pen_down_and_hold_for_the_whole_stroke() {
    // Soltar o Shift no meio de uma pincelada faria METADE dela ser outra
    // ferramenta — e a lei do traço congela um `pre` só, então não há como
    // representar isso. Nenhum app de escultura permite, e aqui a garantia é
    // estrutural: quem lê os modificadores é o Down, e o Move não os consulta.
    let src = sculpt_src();
    let down = function_body(&src, "sculpt3d_pointer_down");
    assert!(
        down.contains("scene.brush.invert = ctrl"),
        "o Ctrl (o `inverte` de todo app de escultura) tem de ser lido no Down"
    );
    assert!(
        down.contains("Verb::Smooth"),
        "o Shift tem de virar Smooth enquanto segurar — o atalho universal"
    );
    let mv = function_body(&src, "sculpt3d_pointer_move");
    assert!(
        !mv.contains("modifiers") && !mv.contains("shift") && !mv.contains("ctrl"),
        "o Move não pode reler modificador: o traço mudaria de ferramenta no meio"
    );
}

#[test]
fn every_3d_port_is_inert_without_a_scene() {
    // A promessa de removibilidade do `docs/3D/02.3` no nível do FRAME: num run
    // normal `sculpt3d` é `None`, cada porta devolve `false` no primeiro `if`, e
    // o dispatch 2D segue como se o módulo não existisse.
    let src = sculpt_src();
    for port in [
        "sculpt3d_pointer_down",
        "sculpt3d_pointer_up",
        "sculpt3d_pointer_move",
        "sculpt3d_wheel",
        "sculpt3d_key",
    ] {
        let body = function_body(&src, port);
        assert!(
            body.contains("sculpt3d_scene_mut()") && body.contains("return false"),
            "`{port}` tem de recusar sem cena armada"
        );
    }
}

#[test]
fn the_shell_takes_the_3d_keys_before_the_widget_store_sees_them() {
    let body = function_body(&source("input_dispatch/keyboard.rs"), "key_input");
    let hook = body
        .find("sculpt3d_key(")
        .expect("as teclas da cena 3D têm de estar costuradas");
    let store = body
        .find("self.handler.on_key(")
        .expect("o store recebe as teclas");
    assert!(
        hook < store,
        "a cena 3D tem de ver a tecla ANTES do store, senão `1..9` viram outra coisa"
    );
}

#[test]
fn the_model_follows_the_hand() {
    // ⚠️ **Proxy deliberado.** O FATO — *arrastar para a direita vira o modelo
    // para a direita* — é definido e medido na crate, em
    // `dragging_right_turns_the_model_right_and_dragging_down_shows_its_top`,
    // que projeta um ponto do modelo NA TELA. Aqui só se afirma que a shell
    // entrega os sinais que aquele fato exige; dirigir a câmera de verdade
    // precisaria de um device.
    //
    // Os dois sinais estavam TROCADOS e o smoke os pegou: `yaw` positivo leva o
    // OLHO para `+X`, e a câmera indo para a direita faz o modelo parecer ir
    // para a esquerda.
    let body = function_body(&sculpt_src(), "sculpt3d_pointer_move");
    assert!(
        body.contains(".orbit(-dx * ORBIT_RAD_PER_PX, dy * ORBIT_RAD_PER_PX)"),
        "a órbita da shell tem de negar o `dx` e NÃO o `dy`"
    );
}

#[test]
fn a_click_on_a_panel_is_not_a_click_on_the_model() {
    // Sem esta pergunta a cena 3D engolia TODO botão do app, inclusive os do
    // rail — ela devolvia `true` incondicionalmente e o dispatch 2D nunca via o
    // evento.
    let src = sculpt_src();
    let down = function_body(&src, "sculpt3d_pointer_down");
    assert!(
        down.contains("cursor_over_hero_panel("),
        "o Down tem de recusar um clique sobre painel"
    );
    // ⚠️ E o Move/Up NÃO podem fazer a mesma pergunta: um arrasto em curso
    // continua sendo do gesto que o abriu, mesmo que o cursor passeie sobre um
    // painel. É a regra de captura que todo gizmo deste shell segue, e gateá-la
    // aqui impede que alguém "complete" a correção e quebre o traço longo.
    for port in ["sculpt3d_pointer_move", "sculpt3d_pointer_up"] {
        assert!(
            !function_body(&src, port).contains("cursor_over_hero_panel("),
            "`{port}` não pode largar um arrasto em curso ao cruzar um painel"
        );
    }
}

#[test]
fn the_mirror_is_off_until_the_artist_asks_for_it() {
    // Um default que só se descobre por acidente é pior que um default menos
    // ambicioso: com o espelho ligado o artista clicava de um lado e via uma
    // segunda protuberância do outro, sem nada na tela explicando por quê.
    // ⚠️ A âncora é a ASSINATURA, não `new(`: desde a W8.1 o cluster tem um
    // segundo construtor (`SceneObject::new`, o objeto da lista) e ele vem
    // ANTES no arquivo — o gate lia a função errada e ficava vermelho sobre
    // produto correto.
    let body = function_body(&sculpt_src(), "new(device: &wgpu::Device");
    assert!(
        body.contains("symmetry: Symmetry::default()"),
        "a simetria tem de nascer desligada; o `X` a liga"
    );
}

#[test]
fn the_brush_radius_is_screen_pixels_converted_against_the_camera() {
    // A entrega do item 6b: o pincel mede pixels de TELA, e o raio de mundo é
    // derivado por dab. Ancorá-lo no modelo fazia o pincel crescer junto com a
    // imagem ao aproximar, o que é o oposto de como se alcança detalhe fino.
    let src = sculpt_src();
    let armed = function_body(&src, "armed_brush");
    assert!(
        armed.contains("world_radius_for_screen_px("),
        "o raio de mundo tem de vir da CÂMERA"
    );
    assert!(
        armed.contains("self.radius_px()"),
        "e do raio já clampado contra a tela, não do campo cru"
    );
    // O teto é do VIEWPORT: um número fixo de pixels muda de significado com a
    // resolução (medido: 160 px = 91% do modelo a 720p e 45% a 1440p).
    let port = function_body(&src, "radius_px(&self)");
    assert!(
        port.contains("self.viewport.1"),
        "o teto do raio tem de ser fração da ALTURA da janela"
    );
    // E nada mais pode responder "de que tamanho é o pincel": um segundo sítio
    // é como o cursor e a tinta passam a discordar.
    assert_eq!(
        src.matches("world_radius_for_screen_px(").count(),
        1,
        "a conversão tela→mundo tem de ter UM sítio no shell"
    );
}

#[test]
fn a_pointer_event_is_walked_at_the_brushes_spacing_and_stops_where_the_ray_misses() {
    // A entrega do item 6c. Um evento de ponteiro não é um dab: o caminho é
    // percorrido a passos do espaçamento, e um passo que erra a malha PARA o
    // gesto (o `break` do `SculptBase.js:151`) em vez de carimbar através do vão.
    // ⚠️ **O braço do CARIMBO, e não o do arrasto inteiro.** Desde o Snake Hook
    // o `Drag::Sculpt` é um `match` sobre o `Grip` com três braços, e dois deles
    // percorrem o caminho — a asserção de ausência abaixo (*a âncora não avança
    // fora do ramo que carimbou*) lia os TRÊS e falhava sobre produto correto.
    let arm = braced_block(
        &function_body(&sculpt_src(), "sculpt3d_pointer_move"),
        "Grip::Stamp =>",
    );
    assert!(
        arm.contains("ph2d_sculpt3d::walk(") && arm.contains("min_spacing("),
        "o arrasto tem de percorrer o caminho no espaçamento do pincel"
    );
    assert!(
        arm.contains("break"),
        "um passo fora do modelo encerra o gesto"
    );

    // ⚠️ **O CARRY, e ele é a metade que se perde distraído:** a âncora só anda
    // quando o `walk` de fato carimbou. Movê-la sempre faria um gesto lento
    // depositar dez vezes mais dabs pelo mesmo caminho — e nada na tela diria
    // por quê. A afirmação é sobre em que BLOCO a atribuição mora.
    let deposited = braced_block(&arm, "if let Some(steps)");
    assert!(
        deposited.contains("stroke_anchor = [x, y]"),
        "a âncora avança dentro do ramo que carimbou"
    );
    assert!(
        !arm.replace(&deposited, "").contains("stroke_anchor ="),
        "a âncora NÃO pode avançar fora dele: é ali que o resíduo se acumula"
    );
}

#[test]
fn the_stroke_anchor_is_armed_at_pen_down() {
    // Sem isto o primeiro arrasto de um traço mede a distância até a âncora do
    // traço ANTERIOR — no outro canto da tela, o que carimba uma fileira de
    // dabs atravessando o modelo.
    assert!(
        function_body(&sculpt_src(), "sculpt3d_pointer_down")
            .contains("stroke_anchor = [pos.0, pos.1]"),
        "o pen-down tem de armar a âncora do espaçamento"
    );
}

#[test]
fn the_grab_holds_its_footprint_instead_of_re_picking() {
    // ⚠️ **A diferença entre Grab e Snake Hook é ONDE a pegada mora**, e ela
    // mora aqui: o Grab prende o ponto do pen-down e arrasta os mesmos vértices;
    // re-picar por evento arrastaria a pegada atrás do cursor, que é o outro
    // verbo. Nenhum gate de unidade vê isto — quem escolhe o centro é a shell.
    let src = sculpt_src();
    let grab = function_body(&src, "grab_at");
    assert!(
        grab.contains("let Some((at, from)) = self.grab"),
        "a pegada tem de ser LIDA do estado, não re-picada"
    );
    assert!(
        grab.contains("finger_world("),
        "o gesto é o delta de TELA convertido pela câmera, senão o barro escapa do cursor ao aproximar"
    );
    assert!(
        grab.contains("Dab::pulling("),
        "e ele chega ao dab pelo construtor que PEDE o gesto"
    );
    assert!(
        !grab.contains("raycast("),
        "nenhum evento de arrasto re-pica: isso arrastaria a pegada, que é o outro verbo"
    );

    // E o arrasto de quem SEGURA não passa pelo walk do espaçamento: um Grab
    // não carimba, então percorrer o caminho daria N dabs idênticos no mesmo
    // lugar.
    let mv = function_body(&src, "sculpt3d_pointer_move");
    let holding = match_arm(&mv, "Grip::Hold =>");
    assert!(
        holding.contains("grab_at(") && !holding.contains("walk("),
        "quem segura arrasta a pegada, não percorre um caminho"
    );
}

/// ⚠️ **O Snake Hook PERCORRE, e é o walk que o torna um fato do caminho.**
///
/// A lei dele é uma soma sobre a lista de dabs (`Grip::Hook`), então sem o passo
/// fixo na geometria arrastar devagar esticaria mais que arrastar rápido pelo
/// mesmo traçado — a doença que este módulo inteiro existe para não ter. Nenhum
/// gate de unidade vê isto: quem decide percorrer é a shell.
#[test]
fn the_hook_walks_the_path_and_hands_each_step_its_own_increment() {
    let src = sculpt_src();
    let mv = function_body(&src, "sculpt3d_pointer_move");
    let hooking = match_arm(&mv, "Grip::Hook =>");
    assert!(
        hooking.contains("walk(") && hooking.contains("hook_step("),
        "quem arrasta percorre o caminho, um passo de cada vez"
    );
    // ⚠️ **O predecessor de cada passo é o passo anterior**, não a âncora do
    // traço: é a diferença entre entregar N incrementos e entregar N vezes o
    // total, e a segunda forma esticaria N² vezes mais.
    assert!(
        hooking.contains("prev = step"),
        "cada passo tem de virar o predecessor do seguinte"
    );
    // O CARRY vale igual aqui: a âncora só anda quando o walk de fato carimbou.
    let deposited = braced_block(&hooking, "if let Some(steps)");
    assert!(
        deposited.contains("stroke_anchor = [x, y]"),
        "a âncora avança dentro do ramo que carimbou"
    );
    assert!(
        !hooking.replace(&deposited, "").contains("stroke_anchor ="),
        "e não fora dele: é ali que o resíduo se acumula"
    );

    // ⚠️ **Os dois centros saem da MESMA porta que o Grab usa.** Duas
    // aritméticas para *onde o dedo está em mundo* divergiriam no dia em que
    // uma ganhasse a perspectiva e a outra não.
    let step = function_body(&src, "hook_step");
    assert_eq!(
        step.matches("finger_world(").count(),
        2,
        "o centro anterior e o novo saem os dois da porta única"
    );
    assert!(
        step.contains("Dab::hooking("),
        "e chegam ao dab pelo construtor que declara ser um INCREMENTO"
    );
    assert!(
        !step.contains("raycast("),
        "o Hook arrasta uma ESFERA pelo espaço: sair do modelo não interrompe um espinho"
    );
}

/// ⚠️ **O `match` do arrasto é EXAUSTIVO sobre o [`Grip`]**, e não uma cascata de
/// predicados. Um grip novo não pode cair no `else` do último `if` e nascer se
/// comportando como um carimbo — ele tem de deixar de compilar até alguém dizer
/// o que significa aqui.
#[test]
fn the_drag_asks_the_grip_and_answers_every_one_of_them() {
    let src = sculpt_src();
    let mv = function_body(&src, "sculpt3d_pointer_move");
    let sculpting = braced_block(&mv, "Drag::Sculpt => match scene.brush.verb.grip()");
    for arm in ["Grip::Hold", "Grip::Hook", "Grip::Turn", "Grip::Stamp"] {
        assert!(
            sculpting.contains(arm),
            "o arrasto tem de responder a {arm} explicitamente"
        );
    }
    assert!(
        !sculpting.contains(" _ =>"),
        "um braço curinga faria o grip novo nascer se comportando como carimbo, em silêncio"
    );
}

/// ⚠️ **Quem GIRA não percorre o caminho, e o eixo é o raio do PEN-DOWN.**
///
/// Os dois são invisíveis a todo gate de unidade — quem escolhe percorrer e quem
/// escolhe o eixo é a shell —, e os dois são defeitos silenciosos: o walk daria
/// N dabs com o mesmo total no mesmo lugar, e um eixo re-derivado do cursor
/// bambolearia alguns graus ao longo da varredura, porque o cursor está
/// justamente andando em círculo.
#[test]
fn the_turn_takes_its_axis_from_the_ray_that_grabbed_the_clay() {
    let src = sculpt_src();
    let mv = function_body(&src, "sculpt3d_pointer_move");
    let turning = match_arm(&mv, "Grip::Turn(kind) =>");
    assert!(
        turning.contains("turn_at(kind"),
        "o arrasto tem de entregar o Amount que o grip carrega, e não re-derivá-lo do verbo"
    );
    assert!(
        !turning.contains("walk("),
        "um gesto de giro não é um caminho a percorrer: o alvo é função do TOTAL varrido"
    );

    let turn = function_body(&src, "turn_at");
    assert!(
        turn.contains("let Some((at, from)) = self.grab"),
        "a âncora tem de ser LIDA do estado, como a do Grab"
    );
    // ⚠️ **`from`, e não `x, y`** — é a linha inteira deste gate.
    assert!(
        turn.contains("self.ray_at(from.0, from.1).dir()"),
        "o eixo sai do pixel do PEN-DOWN; derivá-lo do cursor faz o eixo bambolear"
    );
    assert!(
        !turn.contains("raycast("),
        "girar não re-pica: sair do modelo no meio da varredura não interrompe a torção"
    );
    // Os dois construtores que NOMEIAM a unidade do gesto.
    assert!(
        turn.contains("Dab::turning(") && turn.contains("Dab::scaling("),
        "cada Amount chega ao dab pelo construtor que declara a unidade dele"
    );
}

/// ⚠️ **O ângulo varrido é ACUMULADO, e sem isso a torção inverte sozinha a 180°.**
///
/// Um ângulo com sinal satura em `±π`: medido da direção inicial à atual, a 181°
/// ele volta a `−179°` e o barro **desanda** no meio do gesto. Somando os deltas
/// (pequenos por construção) o total cresce sem teto — e a soma é exata, então o
/// gesto continua sendo um fato do que a mão varreu.
#[test]
fn the_swept_angle_accumulates_instead_of_saturating_at_half_a_turn() {
    let src = sculpt_src();
    let body = function_body(&src, "swept_angle");
    assert!(
        body.contains("g.total +=") && body.contains("atan2("),
        "o total tem de SOMAR o delta de cada evento"
    );
    // A zona morta: perto da âncora a direção é ruído, e o que se perde ali é a
    // REFERÊNCIA, nunca o que já foi varrido.
    assert!(
        body.contains("TWIST_DEADZONE_PX"),
        "a zona morta tem de existir: a um pixel da âncora um tremor vale meio radiano"
    );
    let dead = braced_block(&body, "if len < TWIST_DEADZONE_PX");
    assert!(
        dead.contains("g.last = None"),
        "dentro dela a referência de direção morre, senão a saída seguinte soma um salto"
    );
    assert!(
        !dead.contains("g.total = 0"),
        "e o que já foi varrido FICA: o barro não desfaz a torção porque o dedo passou perto do pivô"
    );
    // O gesto morre com o gesto: um traço novo não começa torcido.
    assert!(
        function_body(&src, "sculpt3d_pointer_down").contains("twist = None"),
        "o pen-down tem de zerar o ângulo varrido"
    );
}

/// ⚠️ **TODO verbo tem de ser alcançável pelo teclado**, e este gate nasceu
/// vermelho: o **Magnify** não tinha tecla nenhuma. Onze verbos de carimbo
/// queriam dez dígitos, ele foi o que transbordou, e nada disse — existia no
/// enum, tinha braço de alvo, era varrido por todos os gates de verbo, e o
/// artista simplesmente não conseguia pegá-lo.
///
/// ⚠️ **A lista sai do PRODUTO, nunca do gate.** Ele lê o `Verb::ALL` da fonte do
/// kernel: uma lista escrita à mão aqui ficaria velha exatamente no commit em
/// que o verbo dezessete entrasse, que é o único momento em que este gate tem
/// alguma coisa a dizer.
#[test]
fn every_verb_is_reachable_from_the_keyboard() {
    let brush = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../crates/ph2d-sculpt3d/src/brush.rs"
    ))
    .expect("a fonte do kernel");
    // ⚠️ **Ancorado no `impl Verb`, e não no primeiro `pub const ALL:`** — o
    // `Falloff` tem um do mesmo nome e vem ANTES no arquivo. O controle abaixo
    // pegou isso na primeira corrida, lendo `["Smooth", "Sphere", "Sharper", …]`.
    let imp = brush.find("impl Verb {").expect("o `impl Verb`");
    let at = imp + brush[imp..].find("pub const ALL:").expect("o Verb::ALL");
    let body = &brush[at..][..brush[at..].find("];").expect("o fim do ALL")];
    let verbs: Vec<&str> = body
        .split("Self::")
        .skip(1)
        .map(|s| s.split(&[',', '\n'][..]).next().unwrap_or("").trim())
        .filter(|s| !s.is_empty())
        .collect();
    // O controle: sem ele um `ALL` que o parser não entendesse deixaria o gate
    // verde sobre uma lista VAZIA.
    assert!(
        verbs.len() >= 16,
        "não consegui ler o Verb::ALL (achei {verbs:?})"
    );

    let keys = function_body(&sculpt_src(), "sculpt3d_key");
    for v in verbs {
        assert!(
            keys.contains(&format!("Verb::{v}")),
            "o verbo {v} não tem tecla: ele existe, tem alvo, e o artista não consegue pegá-lo"
        );
    }
}

/// **A CAVIDADE TEM UMA TECLA, E ELA CHEGA AO DISPOSITIVO** (W10.1).
///
/// Duas metades, e nenhuma implica a outra: um `cycle_cavity` que ninguém chama
/// é uma capacidade sem porta, e uma porta que não alcança o `render` é um
/// número autorado que o pixel nunca vê. ⚠️ **O segundo é o que nenhum teste de
/// unidade pega** — o `render` exige device e janela —, e é por isso que ele é
/// arch-gate sobre a fonte.
#[test]
fn the_cavity_has_a_key_and_the_number_reaches_the_device() {
    let src = sculpt_src();
    let key = function_body(&src, "sculpt3d_key");
    let block = braced_block(&key, "code == K::KeyC");
    assert!(
        block.contains("scene.cycle_cavity()"),
        "o Shift+C tem de chamar a porta unica da cavidade"
    );
    assert!(
        block.contains("DESLIGADA") && block.contains("cavidade: {amount"),
        "o log tem de dizer o NUMERO: este canal nao muda a silhueta, entao sem ele o \
         artista aperta, ve quase a mesma imagem e conclui que a tecla morreu"
    );
    // E o número autorado chega ao passe. ⚠️ `self.cavity` e não um literal: um
    // `0.0` cravado deixaria os gates de GPU verdes (eles chamam a porta direto)
    // com a tecla inerte no produto — a forma exata do defeito que a `line/anim`
    // mediu no overlay da trajetória.
    let render = function_body(&src, "render");
    assert!(
        render.contains("self.cavity,"),
        "o `render` tem de receber a cavidade AUTORADA, nao um literal"
    );
}

/// **A CAVIDADE NASCE DESLIGADA.**
///
/// ⚠️ Um canal de sombreamento que se arma sozinho muda a arte de todo mundo que
/// já esculpiu — e o gate NÃO menciona o passo do ciclo, de propósito: um
/// default só é testado por um teste que não o nomeia. Ele lê o valor com que a
/// cena nasce e afirma que ele é o neutro do `ph2d-mesh-render`.
#[test]
fn the_cavity_is_born_off() {
    let src = sculpt_src();
    assert!(
        src.contains("cavity: ph2d_mesh_render::DEFAULT_CAVITY,"),
        "a cena tem de nascer no neutro da crate de render, e nao num literal proprio"
    );
    assert_eq!(
        ph2d_mesh_render::DEFAULT_CAVITY,
        0.0,
        "o neutro E' zero: com ele o barro e' o da W3, ao byte"
    );
}
