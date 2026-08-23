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
