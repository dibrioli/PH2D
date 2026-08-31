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

/// **A cena nunca toma um clique que é da MOLDURA — e é isto que faz o pill SAIR.**
///
/// ⚠️ **O defeito era maior que o pill:** a recusa perguntava *"o cursor está sobre um PAINEL?"*,
/// e painel é só uma espécie de UI. A faixa do topo e o rail não publicam `panel_rect`, então com
/// o barro na tela a cena 3D engolia o clique em TODO pill do topo — entrar funcionava (sem barro
/// a recusa por `shows_clay` já devolvia o gesto) e sair não, que é exatamente a assimetria que o
/// Enio reportou.
///
/// ⚠️ **A metade NEGATIVA é a que sangra:** a mutação natural é voltar a chamar o irmão de painel,
/// e ela deixa a metade positiva verde num arquivo que ainda "pergunta alguma coisa".
#[test]
fn the_scene_never_takes_a_click_that_belongs_to_the_chrome() {
    let src = fs::read_to_string("src/sculpt3d_input.rs").expect("o módulo do gesto existe");
    assert!(
        src.contains("fn sculpt3d_pointer_down"),
        "controle positivo: o dono do gesto mudou de arquivo e este gate varreria o vazio"
    );
    assert_eq!(
        src.matches("chrome_hit::pointer_over_chrome").count(),
        2,
        "as DUAS portas da cena (o botão e a roda) têm de perguntar pela moldura inteira"
    );
    assert!(
        !src.contains("cursor_over_hero_panel"),
        "a cena voltou a perguntar só pelos PAINÉIS: os pills do topo morrem sob o mouse com o \
         barro na tela"
    );
    assert!(
        !src.contains("cursor_over_hero_chrome"),
        "a cena voltou à LISTA de fundos escrita à mão — ela apodreceu em 2026-08-30 e levou \
         consigo o menu superior e as abas"
    );
}

/// **Todo fundo de moldura é conhecido por quem contorna a moldura.**
///
/// ⛔⛔ **ESTE GATE JÁ FALHOU EM SILÊNCIO, e a lição é do tamanho do report.** Ele existia
/// exactamente para impedir que um `*_BACKDROP` novo nascesse fora da lista — e o
/// `MENUBAR_BACKDROP` nasceu fora dela na mesma semana, **sem o gate se mexer**, porque a varredura
/// lia **um subdiretório** (`ids/chrome/`) e o id novo foi escrito em `ids/menubar.rs`, uma casa
/// acima. *Um gate que varre um DIRETÓRIO afirma sobre o diretório, não sobre o repo.*
///
/// ⚠️ **E o piso `found >= N` não o salvou**: ele foi satisfeito pelos quatro fundos LEGADOS, que
/// continuam declarados mesmo já não sendo pintados por ninguém. *Um piso contado sobre DECLARAÇÕES
/// não nota que as declarações deixaram de ter consumidor.*
///
/// ⇒ hoje a varredura é da **árvore inteira** de ids, e a lista já não é a porta da cena 3D — é a
/// lista de obstáculos que o gizmo de navegação contorna (a porta é
/// `chrome_hit::pointer_over_chrome`, ver `the_scene_asks_the_one_chrome_door.rs`).
///
/// ⚠️ **E ele lê o BLOCO da constante, nunca o arquivo inteiro** — a 1ª versão casava com o
/// arquivo, e o nome do fundo do topo aparece no doc-comment ao lado da lista: a mutação que o
/// TIRA da lista passava, porque o gate estava reconhecendo a própria prosa. *Um oráculo que casa
/// com a documentação de si mesmo não está olhando para o produto.*
#[test]
fn every_chrome_backdrop_is_known_to_the_scene() {
    let src = fs::read_to_string("src/forwarding.rs").expect("a porta existe");
    let (_, after) = src
        .split_once("pub const CHROME_BACKDROPS")
        .expect("controle positivo: a lista mudou de nome e este gate varreria o vazio");
    let door = after
        .split_once("];")
        .expect("a lista não fecha: o bloco a conferir é o literal, não o arquivo")
        .0;
    let root = "../../crates/ph2d-editor-core/src/ids";
    let mut files = Vec::new();
    let mut stack = vec![std::path::PathBuf::from(root)];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("os ids da moldura existem") {
            let path = entry.expect("entrada legível").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                files.push(path);
            }
        }
    }
    assert!(
        files.len() >= 10,
        "controlo: só {} ficheiros de id varridos — a árvore mudou de casa",
        files.len()
    );
    let mut found = 0usize;
    for path in files {
        let ids = fs::read_to_string(&path).expect("arquivo de ids legível");
        for line in ids.lines() {
            let Some((name, _)) = line
                .trim()
                .strip_prefix("pub const ")
                .and_then(|rest| rest.split_once(':'))
            else {
                continue;
            };
            if !name.ends_with("_BACKDROP") {
                continue;
            }
            found += 1;
            assert!(
                door.contains(name),
                "o fundo de moldura `{name}` não está em `CHROME_BACKDROPS`: o gizmo de navegação \
                 não o contorna e vai esconder-se debaixo dele"
            );
        }
    }
    assert!(
        found >= 5,
        "controle positivo: a varredura achou {found} fundos — os ids mudaram de casa e este gate \
         ficou verde por vácuo"
    );
}

/// **A visibilidade do painel é escrita pela BORDA, não por "a chave já existe".**
///
/// ⚠️ **Nenhum teste de unidade alcança isto** — o bridge quer um `HeroScreen` E uma cena, e a cena
/// quer um device. Sobra ler o fonte, e o que se lê é a propriedade: o único sítio que escreve a
/// visibilidade deste painel é o que a testemunha da borda alimenta.
///
/// A metade NEGATIVA é a que sangra: a regra anterior (`contains_key`) fica verde sobre um bridge
/// que ainda "escreve alguma coisa" — e era ela que custava o painel para o resto da sessão.
#[test]
fn the_panel_visibility_is_written_by_the_edge_of_the_clay() {
    let src = fs::read_to_string(BRIDGE).expect("a ponte do painel existe");
    assert!(
        src.contains("take_clay_edge()"),
        "a ponte deixou de perguntar pela borda: o painel volta a ignorar a saída do modo"
    );
    assert_eq!(
        src.matches("panel_visibility").count(),
        1,
        "há mais de um sítio decidindo a visibilidade deste painel — duas portas divergem, e a que \
         não conhece a borda ganha em silêncio"
    );
    assert!(
        !src.contains("contains_key"),
        "a regra do `abre uma vez` voltou: fechar o painel passa a custá-lo para o resto da sessão"
    );
}

/// **O padrão do pincel vem do que o artista VÊ, não do que o sprite GUARDA.**
///
/// Um sprite cuja aparência nasce do sistema de CAMADAS do Painter (procedurais, ajustes, blend)
/// continua apontando para a imagem de ORIGEM. Ler a origem devolve outra textura — e é literalmente
/// o report do Enio (2026-08-09): *"veja a textura ao lado e veja a textura no preview"*, com
/// *"é preciso que as texturas geradas proceduralmente no sistema de camadas da sprite tb
/// funcionem"* como a outra metade da mesma frase.
///
/// ⚠️ **A porta não é nova: é a MESMA do "Use as Brush Grain"**, cujo doc-comment no Painter já a
/// nomeia como *a fonte para usar o documento vivo como padrão*. Uma segunda resposta a *"como este
/// documento vira um padrão?"* divergiria dela na primeira camada de ajuste.
///
/// ⚠️ E a ORDEM é a asserção: as camadas vivas vêm ANTES da imagem de origem. Invertida, o
/// fallback ganha sempre — que é exatamente o mundo que o Enio fotografou.
#[test]
fn the_brush_pattern_reads_the_live_layers_before_the_stored_image() {
    let src = fs::read_to_string(FRAME).expect("o laço de frame existe");
    let arm = src
        .find("sculpt3d_alpha_request, false")
        .expect("o braço do padrão por imagem existe");
    let tail = &src[arm..];
    assert!(
        tail.contains("composite_to_lum()"),
        "o padrão não pergunta pelas CAMADAS vivas: uma textura procedural do sprite chega como a \
         imagem de origem, que é outra coisa"
    );
    assert!(
        tail.contains("read_sprite_source("),
        "o padrão perdeu o caminho da imagem guardada — um sprite sem documento vivo deixa de \
         servir de padrão"
    );
    // ⚠️ **A ORDEM que importa é a da CONSULTA, não a das definições.** A 1ª versão deste gate
    // comparava onde cada uma APARECE no arquivo, e a mutação que troca o combinador
    // (`baked().or(live)`) passava por ela: as duas continuam escritas na mesma ordem, e só a
    // decisão muda. O sujeito é o escrutínio do `match`, que é onde a escolha de fato acontece.
    let scrutinee = tail
        .split_once("let line = match ")
        .expect("a escolha entre as duas fontes mora num `match`")
        .1;
    assert!(
        scrutinee.starts_with("live"),
        "a imagem GUARDADA é consultada antes das camadas VIVAS: o fallback ganha sempre e o \
         procedural nunca chega ao pincel"
    );
    // ⚠️ E perguntar o que a tela mostra não pode trocar a ferramenta da mão do artista.
    assert!(
        !tail[..tail.find("read_sprite_source(").expect("conferido acima")].contains("set_active("),
        "o padrão ATIVA o Painter para poder perguntar: o artista perde a ferramenta que tinha na \
         mão por causa de uma leitura"
    );
}

/// **E o laço de frame HONRA a borda da lâmpada.**
///
/// ⚠️ **Um gate de unidade é cego à fiação do shell, e este par é a prova:** a testemunha
/// (`take_rig_edge`) tem gate próprio ao lado da cena, e ele fica VERDE com o `follow_live_rig`
/// chamado incondicionalmente — a mutação passou por ele. O que decide a arte do documento não é a
/// testemunha existir, é o laço de frame perguntar a ela.
#[test]
fn the_frame_only_re_authors_the_baked_light_when_the_lamp_moved() {
    let src = fs::read_to_string(FRAME).expect("o laço de frame existe");
    assert!(
        src.contains("if scene.take_rig_edge() {"),
        "o laço re-autora o rig dos objetos assados sem perguntar se a lâmpada MOVEU: uma cena \
         recém-criada re-acende a arte inteira com o rig default"
    );
    let guard = src
        .find("if scene.take_rig_edge() {")
        .expect("o guard existe");
    let call = src
        .find("bake::follow_live_rig(")
        .expect("o laço re-autora o rig dos objetos assados");
    assert!(
        guard < call && call - guard < 200,
        "a chamada saiu de dentro do guard da borda: ela volta a rodar todo frame"
    );
}
