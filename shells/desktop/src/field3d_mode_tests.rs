//! Os gates de **quem é dono do canvas** (W40).

use super::{Owner, forget_owner, note_owner, took_the_canvas};
use ph2d_editor::ToolId;

fn tool(name: &str) -> Owner {
    Owner {
        tool: Some(ToolId::new(name)),
        clay: false,
    }
}

fn clay() -> Owner {
    Owner {
        tool: None,
        clay: true,
    }
}

/// ⭐ **O GATE-MÃE, e ele é o report do Enio escrito em transições.**
///
/// *"Se eu entro no modo sculpt ou vector ou qualquer outro, o Modelagem deve ceder."*
#[test]
fn entering_another_mode_takes_the_canvas() {
    let idle = Owner::default();
    assert!(
        took_the_canvas(&idle, &tool("vector")),
        "pegar o Vector é tomar o canvas — foi o caso que o Enio nomeou"
    );
    assert!(
        took_the_canvas(&idle, &clay()),
        "pôr o barro na tela é tomar o canvas — o outro caso que ele nomeou"
    );
    assert!(
        took_the_canvas(&tool("vector"), &tool("painter")),
        "trocar de ferramenta também: a nova quer o canvas tanto quanto a primeira"
    );
}

/// ⚠️ **A metade que impede o painel de fechar a cada quadro.** Sem ela, a lei seria *"enquanto
/// houver ferramenta em mãos"* — e uma ferramenta pegada **fica** em mãos, então o modelador nunca
/// mais abriria. *Não é uma otimização: é o que separa ceder de nunca voltar.*
#[test]
fn nothing_changing_is_not_taking() {
    assert!(
        !took_the_canvas(&tool("vector"), &tool("vector")),
        "a MESMA ferramenta em mãos dois quadros seguidos não tomou nada"
    );
    assert!(
        !took_the_canvas(&clay(), &clay()),
        "o barro que já estava na tela também não"
    );
    assert!(
        !took_the_canvas(&Owner::default(), &Owner::default()),
        "e um app parado não fecha painel nenhum"
    );
}

/// ⚠️ **LARGAR não é tomar.** Sem esta metade, soltar a ferramenta fecharia o painel de modelagem —
/// o oposto do que o gesto pede.
#[test]
fn dropping_a_tool_is_not_taking_the_canvas() {
    assert!(!took_the_canvas(&tool("vector"), &Owner::default()));
    assert!(!took_the_canvas(&clay(), &Owner::default()));
}

/// ⭐ **A BORDA é uma borda**: o mesmo dono, dois quadros seguidos, cede **uma vez**.
#[test]
fn the_edge_fires_once_and_then_stays_quiet() {
    forget_owner();
    assert!(note_owner(tool("vector")), "o primeiro quadro cede");
    assert!(
        !note_owner(tool("vector")),
        "o segundo NÃO — senão o painel fecharia a cada quadro e o pill piscaria para sempre"
    );
    assert!(note_owner(tool("flip")), "e trocar volta a ceder");
    forget_owner();
}

/// ⭐ **A metade SIMÉTRICA: abrir o MODEL é uma borda própria.**
///
/// ⚠️ Sem ela, o artista que entrou na escultura (e viu o MODEL ceder) e voltou ao MODEL teria os
/// **dois** a desenhar — a mesma interferência, ao contrário.
#[test]
fn opening_the_model_panel_is_its_own_edge() {
    forget_owner();
    assert!(!super::model_just_opened(false), "fechado não é abrir");
    assert!(super::model_just_opened(true), "fechado → aberto é a borda");
    assert!(
        !super::model_just_opened(true),
        "…e ela dispara UMA vez: senão o barro cederia a cada quadro com o MODEL aberto"
    );
    assert!(!super::model_just_opened(false), "fechar não é abrir");
    assert!(super::model_just_opened(true), "e reabrir volta a ceder");
    forget_owner();
}

/// ⚠️ **As duas bordas são INDEPENDENTES.** No quadro em que o artista pega uma ferramenta *e* o
/// MODEL está a abrir, juntá-las num estado só faria uma mascarar a outra — e o resultado seria um
/// dos dois modos a ficar de pé sem ninguém ter decidido qual.
#[test]
fn the_two_edges_do_not_mask_each_other() {
    forget_owner();
    assert!(super::model_just_opened(true), "o MODEL abriu");
    assert!(
        note_owner(tool("vector")),
        "…e no MESMO quadro alguém pegou o Vector: as duas bordas têm de disparar"
    );
    forget_owner();
}

/// ⭐ **O LOOP CONSOME A LEI** — e sem este gate as funções acima podiam estar perfeitas e não ser
/// chamadas por ninguém.
///
/// ⚠️ É a lição da W34 outra vez: *provar o cálculo não prova a alcançabilidade dele*. O caminho
/// real precisa de janela, então o que se mede é a **costura no fonte** — as três metades que, em
/// falta, devolvem exactamente o report do Enio.
#[test]
fn the_render_loop_actually_makes_the_modes_cede() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/render_loop/mod.rs"),
    )
    .expect("o loop existe");
    // ⚠️ Comentários fora: a prosa que EXPLICA a lei cita os mesmos nomes que ela usa.
    let code: String = src
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");

    for needed in [
        // 1. o loop pergunta quem tomou o canvas…
        "crate::field3d_mode::note_owner(owner)",
        // 2. …e FECHA o painel quando alguém tomou (não basta desarmar em silêncio)
        "insert(ph2d_panel_model3d::PANEL_ID, false)",
        // 3. …e a metade simétrica: abrir o MODEL tira o barro da tela
        "crate::field3d_mode::model_just_opened(model_open)",
        "scene.toggle_clay()",
    ] {
        assert!(
            code.contains(needed),
            "a lei de quem cede não está ligada ao quadro — falta `{needed}`. \
             Sem ela o report do Enio volta: *\"não consigo esculpir nada pois o modo de \
             modelagem permanece interferindo\"*"
        );
    }
}

/// ⛔ **A CENA DE ESCULTURA VAZIA NÃO PODE RECEBER UM GESTO** — sem esta guarda o app **crasha**.
///
/// Enio, 2026-08-22, no smoke desta wave (a lei de ceder levou-o à escultura, e lá ele apagou a
/// única peça e clicou):
///
/// ```text
/// [sculpt3d] APAGOU: sobram 0 pecas -- Ctrl+Z a devolve INTEIRA
/// PH2D PANIC ... sculpt3d_input.rs:173 "index out of bounds: the len is 0 but the index is 0"
/// ```
///
/// ⭐ **A cena vazia é um estado LEGÍTIMO** — o `delete_active` produ-lo e promete o Ctrl+Z de
/// volta. O que faltava era a outra metade: os caminhos de gesto indexam `objects[active]` direto.
/// *Um estado que o módulo declara legal e um caminho que o supõe impossível é um pânico à espera
/// do primeiro clique.*
///
/// ⚠️ **Por que um gate de FONTE e não comportamental:** a `Sculpt3dScene` precisa de um `Device`
/// de GPU para existir (`Sculpt3dScene::new(&device, …)`), então nenhum gate deste repositório a
/// constrói — a suíte inteira do módulo irmão testa as funções **puras** à volta dela, e diz isso
/// no cabeçalho. Medir o fonte é o que sobra, e é honesto sobre o que mede.
///
/// ⚠️ **E ele vive AQUI, no módulo de modelagem**, porque a guarda é desta linha: a cura completa
/// são **42** indexações sem guarda em 9 arquivos da `line/sculpt3d`, e reescrevê-las não é nossa.
/// Este gate defende a porta que o artista bateu; o resto está nomeado no handoff.
#[test]
fn the_sculpt_pointer_refuses_an_empty_scene_before_it_indexes_one() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/sculpt3d_input.rs"),
    )
    .expect("o irmão da escultura existe");
    let code: Vec<&str> = src
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect();

    let guard = code
        .iter()
        .position(|l| l.contains("scene.objects.is_empty()"))
        .expect(
            "o `sculpt3d_pointer_down` tem de RECUSAR uma cena vazia — sem isso, apagar a última \
             peça e clicar derruba o app",
        );
    let first_index = code
        .iter()
        .position(|l| l.contains("scene.objects[scene.active]"))
        .expect("…e o gate só prova alguma coisa se a indexação que ele defende existir");
    assert!(
        guard < first_index,
        "a guarda (linha {guard} do código sem comentários) tem de vir ANTES da primeira \
         indexação (linha {first_index}) — depois dela é tarde"
    );
}

/// ⭐⭐ **DESARMAR TEM DE DESARMAR** — o defeito que a W40 não alcançou.
///
/// Enio, depois do smoke da W40: *"ainda não consigo usar outros modos como vector"*.
///
/// # ⚠️ O doc do `with_smoke` prometia isto e o código não o fazia
///
/// > *"Devolve `None` quando o smoke não está armado — e é isso que faz cada gancho de entrada ser
/// > **inerte** (e portanto invisível) fora dele."*
///
/// Só que o `armed_scene()` era consultado **apenas dentro do `boot()`**, isto é, **só enquanto o
/// smoke ainda não existia**. Nascido uma vez, ele vivia para sempre: fechar o painel punha a
/// bandeira a `false` e **ninguém a voltava a ler**. É, à letra, a frase original do report — *"o
/// modo Modelagem nunca é desativado"*.
///
/// ⭐ **E é por isso que esculpir funcionava e o Vector não:** no `input_dispatch` a escultura toma
/// o ponteiro na linha 3174 e a modelagem na 3186 — quem vem depois do 3186 (o Vector, o gizmo, a
/// seleção) nunca via o clique. *A ordem do despacho transformou um bug em dois sintomas
/// diferentes, e o segundo parecia outra coisa.*
///
/// *Um comentário que descreve uma lei que o código deixou de cumprir é pior do que nenhum: ele
/// responde a pergunta e impede que alguém a verifique.*
#[test]
fn disarming_the_module_actually_disarms_it() {
    use crate::field3d_smoke::{set_armed_by_panel, with_smoke};

    set_armed_by_panel(true);
    assert!(
        with_smoke(|_| ()).is_some(),
        "armado, o módulo existe — senão o gate abaixo passaria por não haver nada"
    );

    set_armed_by_panel(false);
    assert!(
        with_smoke(|_| ()).is_none(),
        "DESARMADO, todo gancho de entrada tem de ser inerte — é o que o doc do `with_smoke` \
         promete, e era o que ele não fazia: o ponteiro continuava a ser da modelagem e o Vector \
         nunca via o clique"
    );

    // ⭐ E rearmar volta a acender: o pill é uma porta de ida E volta.
    set_armed_by_panel(true);
    assert!(
        with_smoke(|_| ()).is_some(),
        "reabrir o painel traz o módulo de volta"
    );
    set_armed_by_panel(false);
}

/// ⭐ **REARMAR NÃO REPLANTA O DEMO POR CIMA DA PEÇA DO ARTISTA.**
///
/// ⚠️ **Este gate existe porque uma NOTA o afirmava sem prova**, e a afirmação custou o app: o
/// `set_armed_by_panel` travava ligado *"para o artista não perder a peça"*. A peça não estava em
/// risco — desde a W5 ela é uma **árvore de entidades ECS** —, mas o medo, sem gate, bastou para
/// prender a bandeira, e a bandeira presa deixou o módulo a comer o ponteiro do app inteiro.
///
/// *Uma cerca cuja razão dissolveu continua a cobrar o preço dela. Um gate no lugar da nota teria
/// mostrado, no dia, que já não havia o que proteger.*
#[test]
fn rearming_does_not_replant_the_demo_over_the_artists_piece() {
    use crate::field3d_smoke::set_armed_by_panel;

    set_armed_by_panel(true);
    let mut sim = ph2d_ecs::SimWorld::new();
    // Quadro 1: a semente planta a peça.
    crate::field3d_scene::ecs_bridge(&mut sim, None, &[], &crate::field3d_scene::no_drawing());
    let before = {
        let world = sim.world_mut();
        let mut q = world.query::<(bevy_ecs::entity::Entity, &ph2d_field_ecs::FieldObject)>();
        q.iter(world).count()
    };
    assert_eq!(before, 1, "a peça nasceu");

    // O artista fecha o painel (agora isto DESARMA) e volta a abri-lo.
    set_armed_by_panel(false);
    assert!(
        crate::field3d_scene::ecs_bridge(&mut sim, None, &[], &crate::field3d_scene::no_drawing())
            .is_none(),
        "desarmado, a ponte é inerte — não coze, não semeia, não pede seleção"
    );
    set_armed_by_panel(true);
    crate::field3d_scene::ecs_bridge(&mut sim, None, &[], &crate::field3d_scene::no_drawing());

    let after = {
        let world = sim.world_mut();
        let mut q = world.query::<(bevy_ecs::entity::Entity, &ph2d_field_ecs::FieldObject)>();
        q.iter(world).count()
    };
    assert_eq!(
        after, 1,
        "reabrir o painel NÃO pode plantar uma segunda peça por cima da que o artista tem — \
         era exactamente este o medo que travava a bandeira, e ele não se sustenta: a ponte \
         encontra a raiz que já existe e ignora a semente"
    );
    set_armed_by_panel(false);
}
