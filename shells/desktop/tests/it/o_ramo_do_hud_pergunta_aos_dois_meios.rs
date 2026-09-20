//! ⭐⭐⭐ **O ramo do clique do HUD pergunta a QUEM DESENHA, nos DOIS meios** (TOP-20 #20).
//!
//! # ⛔ Porque este gate é de TEXTO, e não de comportamento
//!
//! `hud_button_at` precisa de uma `wgpu::Surface`, de uma cena vectorial construída e do mundo de
//! apresentação — *não é alcançável de um teste*. A LEI da escolha vive numa porta pura
//! (`ph2d_app_components::hud_bridge::botao_sob_o_cursor`, com quatro gates lá); o que fica sem
//! régua é o ELO: **quem colhe os candidatos**.
//!
//! ⚠️⚠️ *Um motor com a lei certa e a shell a não a ligar lê-se como um motor sem a lei* — é a
//! quinta ocorrência desta forma nesta casa, e o `include_str!` é a resposta dela (ele deixa de
//! **compilar** se o ficheiro mudar de sítio, ao contrário de um `read_to_string`).

const RAMO: &str = include_str!("../../src/input_dispatch/despacho_clique_hud.rs");

/// ⭐ **Os dois meios são consultados**, e o vectorial primeiro.
///
/// ⚠️ **A ordem faz parte da afirmação:** ela é a decisão declarada no ramo (o vectorial é o meio
/// próprio do HUD), e um gate que só contasse as duas presenças passaria com elas trocadas.
///
/// **Mutações que devem sangrar:** apagar a linha do `pick_sprite_at_world` · apagar a do
/// `path_at` · trocá-las de ordem.
#[test]
fn o_ramo_do_hud_colhe_o_vectorial_e_o_sprite_por_essa_ordem() {
    let corpo = RAMO
        .split_once("fn hud_button_at")
        .expect("o ramo tem de declarar quem esta' sob o cursor")
        .1;
    let fim = corpo.find("\n    }").expect("a funcao fecha");
    let corpo = &corpo[..fim];
    let vec_ = corpo.find(".path_at(");
    let spr = corpo.find("pick_sprite_at_world");
    assert!(
        vec_.is_some(),
        "o ramo deixou de perguntar a` cena VECTORIAL — um rotulo de HUD e' texto vectorial"
    );
    assert!(
        spr.is_some(),
        "o ramo deixou de perguntar aos SPRITES — um botao feito de arte em pixels fica MUDO, e \
         nada na tela o distingue de um botao sem nome"
    );
    assert!(
        vec_ < spr,
        "o vectorial e' o meio PROPRIO do HUD e vem primeiro; a ordem e' a decisao declarada la'"
    );
}

/// ⚠️ **Quem ESCOLHE é a lei, e não este ramo.**
///
/// ⛔ Uma segunda cópia da regra aqui divergiria no dia em que uma das duas mudasse — a frase que o
/// próprio ramo já escreve sobre o `ph2d_hud::clique`, aplicada à outra metade.
#[test]
fn quem_escolhe_o_botao_e_a_porta_e_nao_o_ramo() {
    let corpo = RAMO
        .split_once("fn hud_button_at")
        .expect("o ramo tem de declarar quem esta' sob o cursor")
        .1;
    let fim = corpo.find("\n    }").expect("a funcao fecha");
    let corpo = &corpo[..fim];
    assert!(
        corpo.contains("botao_sob_o_cursor"),
        "o ramo tem de delegar a escolha na porta com gates"
    );
    // ⚠️ **A metade que impede a recaída:** a elegibilidade mora na porta, logo o ramo não a pode
    // voltar a escrever — *`disabled` e o nome em branco são as duas maneiras de um botão não ser
    // um botão, e separá-las de quem o encontra dá dois lugares para decidir*.
    assert!(
        !corpo.contains("disabled"),
        "a elegibilidade e' da PORTA; escreve^-la aqui da' dois sitios para decidir"
    );
}
