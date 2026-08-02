//! **Arch-gates do gesto de escultura** (ADR-0150 W2).
//!
//! ⚠️ Por que a fonte e não o comportamento: a cena 3D só existe com um
//! `wgpu::Device` vivo (`AppGfx.sculpt3d`), então nenhum teste headless a
//! constrói e nenhum teste de unidade das crates alcança esta fiação. É o mesmo
//! motivo pelo qual o gizmo de âncora da `line/physics` e a decisão de upload do
//! Painter têm arch-gate: *um gate de unidade é cego à fiação do shell*.
//!
//! Cada asserção abaixo tem uma mutação que a derruba, listada no handoff.

use std::fs;

/// A fonte **sem comentários**.
///
/// ⚠️ Não é higiene: um arch-gate que varre o arquivo cru afirma coisas sobre a
/// PROSA. Este mesmo gate nasceu vermelho porque o doc-comment do `undo_stroke`
/// explica *por que* ele não usa `refresh_region` — a explicação continha a
/// palavra que a asserção proibia. Um gate que dispara em documentação ensina a
/// não documentar.
fn source(name: &str) -> String {
    let raw = fs::read_to_string(format!("{}/src/{name}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("não consegui ler src/{name}: {e}"));
    raw.lines()
        .map(|l| match l.find("//") {
            Some(at) => &l[..at],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// O corpo de `fn <name>` até a chave que o fecha, contando profundidade.
fn function_body(src: &str, name: &str) -> String {
    let at = src
        .find(&format!("fn {name}"))
        .unwrap_or_else(|| panic!("não achei `fn {name}`"));
    let open = src[at..].find('{').expect("corpo") + at;
    let mut depth = 0i32;
    for (i, c) in src[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return src[open..open + i + 1].to_string();
                }
            }
            _ => {}
        }
    }
    panic!("`fn {name}` não fecha");
}

/// O bloco `{...}` que começa logo depois de `anchor`, balanceado.
///
/// ⚠️ Existe para afirmar **em que bloco** uma linha mora — que é uma pergunta
/// estrutural — em vez de *a quantos bytes* ela está de outra. A segunda forma é
/// um proxy que expira: a `line/Vector` teve dois arch-gates vermelhos por
/// medirem distância em bytes num arquivo que cresceu.
fn braced_block(src: &str, anchor: &str) -> String {
    let at = src
        .find(anchor)
        .unwrap_or_else(|| panic!("não achei `{anchor}`"));
    let open = src[at..].find('{').expect("bloco") + at;
    let mut depth = 0i32;
    for (i, c) in src[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return src[open..open + i + 1].to_string();
                }
            }
            _ => {}
        }
    }
    panic!("`{anchor}` não fecha");
}

/// O corpo de um braço de `match`: o bloco `{...}` se houver, senão o resto da
/// linha.
///
/// ⚠️ **Existe porque [`braced_block`] ATRAVESSA um braço de uma linha só.** Ele
/// procura a próxima `{` a partir da âncora, e num braço como
/// `Grip::Hold => scene.grab_at(x, y),` essa chave é a do braço **seguinte** —
/// então uma asserção de ausência (*"este braço não chama `walk`"*) sai lendo o
/// braço que chama, e um gate que passa por olhar o lugar errado é pior que
/// gate nenhum.
fn match_arm(src: &str, anchor: &str) -> String {
    let at = src
        .find(anchor)
        .unwrap_or_else(|| panic!("não achei o braço `{anchor}`"));
    let rest = &src[at + anchor.len()..];
    if rest.trim_start().starts_with('{') {
        braced_block(src, anchor)
    } else {
        rest.lines().next().unwrap_or_default().to_string()
    }
}

/// A fiação do módulo 3D no shell, **os dois arquivos como um**.
///
/// ⚠️ O corte entre *a cena* (`sculpt3d.rs`) e *o gesto* (`sculpt3d_input.rs`) é
/// de responsabilidade e já se moveu uma vez (o teto de LOC). Um gate que
/// nomeia o ARQUIVO de cada função vira vermelho no próximo split, sobre
/// produto correto — a `line/Vector` pagou isso duas vezes. As asserções aqui
/// são sobre o que a fiação FAZ, então elas leem o par.
fn sculpt_src() -> String {
    format!("{}\n{}", source("sculpt3d.rs"), source("sculpt3d_input.rs"))
}

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
fn the_gpu_is_handed_the_window_that_answers_for_every_channel() {
    // ⚠️ Este gate existe porque o defeito É este, e um gate de GPU o pegou: o
    // conjunto que muda de NORMAL é maior que o que se MOVE — um vizinho parado
    // ao lado de uma face que girou tem a normal mudada. Subir só os movidos
    // deixa a malha iluminada por normais velhas numa faixa de um anel de
    // largura, bem na BORDA do pincel.
    //
    // ⚠️ **E a pergunta certa mudou de nome na W4.2, sobre o mesmo defeito.**
    // `last_refreshed` responde *de quem eu recomputei a normal* — que é VAZIO
    // num traço de máscara, porque máscara não move geometria. A janela que o
    // upload quer é *o que a GPU precisa RE-LER, em qualquer canal*, e ela tem
    // porta própria. Perguntar a antiga deixaria a máscara invisível no device
    // com todos os gates de CPU verdes.
    let body = function_body(&sculpt_src(), "sculpt_at");
    assert!(
        body.contains("last_gpu_dirty()"),
        "a janela do upload é a que responde por TODOS os canais"
    );
    assert!(
        !body.contains("last_moved()"),
        "`last_moved` é a janela do UNDO, não a da GPU"
    );
    assert!(
        !body.contains("last_refreshed()"),
        "`last_refreshed` é só o canal das NORMAIS: ele é vazio num traço de máscara"
    );
}

#[test]
fn releasing_the_button_turns_the_stroke_into_an_undo_entry() {
    let src = sculpt_src();
    let up = function_body(&src, "sculpt3d_pointer_up");
    assert!(
        up.contains("Drag::Sculpt") && up.contains("close_stroke()"),
        "soltar um traço tem de fechá-lo; sem isso o Ctrl+Z não tem o que desfazer"
    );
    let close = function_body(&src, "close_stroke");
    // O undo NÃO é um segundo sistema: a lei do traço já congela o `pre` por
    // vértice tocado, e a lista de tocados É a janela.
    assert!(
        close.contains("stroke.touched()") && close.contains("stroke.base_positions()"),
        "a entrada de undo sai do traço, não de uma segunda captura"
    );
}

#[test]
fn undoing_rebuilds_the_index_instead_of_refitting_the_way_back() {
    // Um refit sobre a VOLTA só cresce as caixas frouxas (ele nunca as encolhe
    // abaixo do que já viu), então cada Ctrl+Z deixaria a árvore um pouco mais
    // gorda e a consulta um pouco mais lenta, para sempre. Um undo é
    // user-paced — é o lugar certo para pagar a resposta exata.
    let body = function_body(&sculpt_src(), "undo_stroke");
    assert!(
        body.contains("self.mesh.rebuild()"),
        "desfazer tem de reconstruir o índice"
    );
    assert!(
        !body.contains("refresh_region"),
        "o caminho incremental não serve para a volta"
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
    let body = function_body(&sculpt_src(), "new(");
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
fn the_four_mask_operations_have_a_gesture_and_an_undo() {
    // ⚠️ Uma operação construída que o artista não alcança é a mesma coisa que
    // ela não existir — e é literalmente o defeito que a W4.2 fecha (a máscara
    // existia desde a W2 e era invisível).
    let src = sculpt_src();
    let key = function_body(&src, "sculpt3d_key");
    for (code, op) in [
        ("KeyC", "MaskOp::Clear"),
        ("KeyI", "MaskOp::Invert"),
        ("KeyB", "MaskOp::Blur"),
        ("KeyN", "MaskOp::Sharpen"),
    ] {
        assert!(
            key.contains(&format!("K::{code} => Some({op})")),
            "`{op}` precisa de uma tecla"
        );
    }
    assert!(
        key.contains("scene.mask_op(op)"),
        "e a tecla tem de chegar à porta"
    );

    // ⚠️ **As quatro NÃO são verbos.** Um verbo é uma ferramenta que fica na mão;
    // estas executam e acabam. Enfiá-las na lista de números faria "escolher a
    // máscara" e "borrar a máscara" parecerem o mesmo tipo de gesto.
    // ⚠️ O `braced_block` conta CHAVES e a lista de verbos é um ARRAY, então
    // apontá-lo aqui lê o bloco seguinte — foi o que esta asserção fez na
    // primeira versão, e ela falhou descrevendo outra coisa.
    let at = src.find("const BY_NUMBER").expect("a lista de verbos");
    let by_number = &src[at..at + src[at..].find("];").expect("a lista fecha")];
    assert!(
        !by_number.contains("MaskOp"),
        "as operações de máscara não são verbos: elas executam e acabam"
    );
    assert_eq!(
        by_number.matches("Verb::").count(),
        10,
        "a lista de números é dos DEZ verbos"
    );

    let undo = function_body(&src, "undo_stroke");
    assert!(
        undo.contains("entry.whole_mask"),
        "desfazer tem de distinguir a janela de um traço da máscara INTEIRA"
    );
    // ⚠️ E `None` quer dizer *não havia máscara*, que se desfaz REMOVENDO o
    // plano. Zerá-lo deixaria a malha pagando 4 B/vértice por um estado que ela
    // não tinha — e desfazer um `Invert` sobre malha virgem deixaria tudo
    // protegido para sempre.
    assert!(
        undo.contains("take_masks()"),
        "o caso `None` tem de REMOVER o plano, não zerá-lo"
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
/// predicados. Um quarto grip não pode cair no `else` do último `if` e nascer se
/// comportando como um carimbo — ele tem de deixar de compilar até alguém dizer
/// o que significa aqui.
#[test]
fn the_drag_asks_the_grip_and_answers_every_one_of_them() {
    let src = sculpt_src();
    let mv = function_body(&src, "sculpt3d_pointer_move");
    let sculpting = braced_block(&mv, "Drag::Sculpt => match scene.brush.verb.grip()");
    for arm in ["Grip::Hold", "Grip::Hook", "Grip::Stamp"] {
        assert!(
            sculpting.contains(arm),
            "o arrasto tem de responder a {arm} explicitamente"
        );
    }
    assert!(
        !sculpting.contains(" _ =>"),
        "um braço curinga faria o quarto grip nascer se comportando como carimbo, em silêncio"
    );
}
