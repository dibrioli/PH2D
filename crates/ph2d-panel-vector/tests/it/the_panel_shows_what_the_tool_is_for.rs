//! ⭐⭐⭐ **O painel mostra o que serve à ferramenta na mão** — o censo da tabela de escopo.
//!
//! Report do Enio (2026-08-31, com foto): *"Mesmo com outras ferramentas selecionadas, as Shapes
//! ficam expostas e as propriedades das shapes também. Melhor deixar no painel apenas o que é útil
//! para a ferramenta em uso. E isso para todas as ferramentas."*
//!
//! ⚠️ **Medido antes de mexer: das 39 seções do corpo, UMA consultava o modo** (a do Lápis).
//! ⭐⭐ E a lei já estava escrita e honrada uma fileira acima — dentro da seção TOOL, a fileira do
//! Marquee só aparece no modo Node e os botões da linha de corte só no modo Cut, com o
//! doc-comment a dizer *"fora do modo Cut os dois seriam controles de uma ferramenta que não está
//! na mão"* (Enio, 2026-07-31). *A regra não atravessava a fronteira da seção porque vivia escrita
//! à mão dentro de cada uma, onde 38 podem esquecê-la.*

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::panel::{ErasedPanel, Panel, PanelRegistry};
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W};
use ph2d_editor_core::screens::paint_hero_screen;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::{VectorPanel, set_current_vector_style};
use ph2d_text::TextSystem;
use ph2d_tool_vector::VectorStyleSnapshot;
use ph2d_tool_vector::params::DrawMode;
use ph2d_vector::VectorScene;
use std::sync::Once;

/// O corpo inteiro do painel cabe — ver o `paint_in`.
const TALL_VIEWPORT_H: f32 = HERO_VIEWPORT_H * 4.0;

fn hero_with_vector_panel() -> HeroScreen {
    static INIT: Once = Once::new();
    ph2d_editor_core::test_support::ensure_panel_registry();
    INIT.call_once(|| {
        let mut reg = PanelRegistry::new_empty();
        reg.push(ErasedPanel::new::<VectorPanel>());
        let _ = ph2d_editor_core::panel::install_panel_registry(reg);
    });
    let mut hero = HeroScreen::new(NodeId(1));
    hero.panel_visibility.insert(VectorPanel::ID, true);
    hero
}

fn paint_in(hero: &mut HeroScreen, snap: VectorStyleSnapshot) {
    set_current_vector_style(Some(snap));
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    paint_hero_screen(
        hero,
        // ⚠️ Alta de propósito: a porta da rolagem (`scroll_area`, spec `04_a_rolagem_unica`)
        // recorta o CLIQUE à banda do corpo, e o `painted` lê o registo — à altura de fábrica uma
        // secção abaixo da dobra lia-se «não oferecida». Aqui a pergunta é a LEI, não a dobra.
        Rect::new(0.0, 0.0, HERO_VIEWPORT_W, TALL_VIEWPORT_H),
        &mut scene,
        &mut text,
    );
}

fn painted(hero: &HeroScreen, section: NodeId) -> bool {
    hero.hit_index.rect_for(section).is_some()
}

fn snap_of(mode: DrawMode) -> VectorStyleSnapshot {
    VectorStyleSnapshot {
        mode,
        ..VectorStyleSnapshot::default()
    }
}

/// ⚠️⚠️ **A OUTRA METADE DO MESMO REPORT, e ela não é sobre a ferramenta: é sobre a SELEÇÃO.**
///
/// A seção *Effects* punha a guarda **depois** do cabeçalho — então sem caminho selecionado o
/// painel pintava o título, o separador de rodapé, e corpo nenhum. Foi a única das 39 a fazê-lo, e
/// o doc-comment dela dizia o contrário (*"devolve o `y` intocado … é o que faz o `step` não emitir
/// separador órfão"*): estava certo sobre a intenção e errado sobre o código.
///
/// *Um cabeçalho que promete um corpo que não existe é pior que uma seção ausente.*
#[test]
fn a_section_header_never_promises_a_body_that_is_not_there() {
    let mut hero = hero_with_vector_panel();
    ph2d_panel_vector::set_current_effects(false, false, &["Blur"], Vec::new());
    paint_in(&mut hero, snap_of(DrawMode::Select));
    assert!(
        !painted(&hero, ph2d_tool_vector::ids::VECTOR_SECTION_EFFECTS),
        "sem caminho selecionado, o titulo EFFECTS sobe sozinho — cabecalho sem corpo"
    );

    // Controle: COM alvo ela aparece — senão este gate passaria por a seção estar morta.
    ph2d_panel_vector::set_current_effects(true, false, &["Blur"], Vec::new());
    paint_in(&mut hero, snap_of(DrawMode::Select));
    assert!(painted(
        &hero,
        ph2d_tool_vector::ids::VECTOR_SECTION_EFFECTS
    ));
    ph2d_panel_vector::set_current_effects(false, false, &[], Vec::new());
}

/// ⛔⛔ **AS DUAS QUE NÃO PODEM SUMIR, e a razão é uma SEGUNDA SELEÇÃO INVISÍVEL.**
///
/// *Blend* e *Morph* parecem irmãs das cinco acima e não são: o `Pick Shapes` troca o `DrawMode`
/// **sem olhar a seleção** (`tool_panel_event.rs:160`), e os dois botões correm sobre o
/// `VecState::blend_picks` — uma lista que o `blend_pick_at` coleta e que **nunca toca no `PenTool`**
/// (`input_dispatch.rs:1362`).
///
/// *Uma regra de "esconde com a seleção vazia" aplicada a olho teria escondido justamente os dois
/// controles que ainda funcionam.*
#[test]
fn the_blend_and_morph_survive_an_empty_selection_because_picks_are_a_second_selection() {
    let mut hero = hero_with_vector_panel();
    // F17: as duas vivem no grupo ▸ Advanced, que nasce dobrado — a pergunta aqui é a SELEÇÃO.
    hero.store
        .set_collapsed(ph2d_panel_vector::ids::VECTOR_SECTION_ADVANCED, false);
    ph2d_panel_vector::set_current_selection_count(0);
    paint_in(&mut hero, snap_of(DrawMode::PickBlend));
    assert!(
        painted(&hero, ph2d_tool_vector::ids::VECTOR_SECTION_BLEND),
        "o Blend sumiu — e com ele o Pick Shapes, que e' como se escolhem as formas"
    );
    assert!(painted(&hero, ph2d_tool_vector::ids::VECTOR_SECTION_MORPH));
    // E também na ferramenta comum: o modo Pick entra POR ali, então esconder a seção trancaria
    // a porta de entrada dela.
    paint_in(&mut hero, snap_of(DrawMode::Select));
    assert!(painted(&hero, ph2d_tool_vector::ids::VECTOR_SECTION_BLEND));
    assert!(painted(&hero, ph2d_tool_vector::ids::VECTOR_SECTION_MORPH));
}

/// ⭐⭐ **OS TRÊS BOTÕES DE ARTE FALAM UMA PALAVRA SÓ, e ela não é "Shape".**
///
/// Os dois modelos aceitam **um grupo** desde 2026-08-30 (`20881b0b0` na estampa, `59a80bd6e` no
/// pincel) e os rótulos continuaram a dizer *"Shape"* — prometendo **menos** do que a porta aceita.
/// ⚠️ Esse defeito **não dá erro**: ele apaga a feature para quem lê o botão.
///
/// ⛔ E eram três literais em dois ficheiros para **um** gesto (clicar numa forma do documento) —
/// dois nomes para um gesto são dois conceitos aos olhos de quem aprende. A porta é
/// `art_vocabulary`.
#[test]
fn the_art_pickers_speak_one_word() {
    // ⚠️⚠️ **DESCASCAR OS COMENTÁRIOS É OBRIGATÓRIO, e a 1.ª redacção deste gate não o fazia** — ele
    // reprovou sobre produto CORRECTO, acusando um doc-comment que **documentava a cura** (*"o
    // rótulo do botão muda para «Pick Shape…»"*). *Um gate textual que não descasca comentários
    // proíbe que a cura seja explicada.*
    let so_codigo = |fonte: &str| -> String {
        fonte
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    for (ficheiro, bruto) in [
        ("paint_brush.rs", include_str!("../../src/paint_brush.rs")),
        (
            "paint_texture_pattern.rs",
            include_str!("../../src/paint_texture_pattern.rs"),
        ),
    ] {
        let fonte = so_codigo(bruto);
        // ⚠️ Só o LITERAL de rótulo, não a palavra: os doc-comments destes ficheiros falam de
        // formas o tempo todo, com razão. O que se proíbe é um botão novo a nascer com nome próprio.
        for proibido in [
            "\"Pick Shape",
            "\"Change Shape",
            "\"Use Shape",
            "\"Pick Art",
            "\"Use Art",
        ] {
            assert!(
                !fonte.contains(proibido),
                "{ficheiro} escreve o rotulo {proibido}…\" a' mao — ele tem de vir do \
                 `art_vocabulary`, senao os dois paineis divergem no primeiro ajuste"
            );
        }
        assert!(
            fonte.contains("crate::art_vocabulary::"),
            "{ficheiro} deixou de consultar a porta do vocabulario"
        );
    }
    // A fixtura contém o fenômeno: a porta de facto NÃO diz "Shape".
    // ⚠️ Desde 2026-09-16 a porta guarda a CHAVE (`TextKey`), e o que se confere é o TEXTO que a
    //    tabela devolve para ela — que é o que o artista lê.
    let porta = include_str!("../../src/art_vocabulary.rs");
    for label in ["PICK", "CHANGE", "USE"] {
        let i = porta
            .find(&format!("const {label}: TextKey = TextKey::new(\""))
            .unwrap_or_else(|| panic!("a porta perdeu o rotulo {label}"));
        let resto = &porta[i..];
        let abre = resto.find("new(\"").expect("a chave abre") + "new(\"".len();
        let fecha = abre + resto[abre..].find('"').expect("a chave fecha");
        let chave = &resto[abre..fecha];
        let valor = ph2d_i18n::tr(chave);
        assert_ne!(
            valor, chave,
            "a chave {chave} do rotulo {label} nao esta na tabela"
        );
        assert!(
            !valor.contains("Shape"),
            "o rotulo {label} voltou a prometer uma forma sozinha: {valor}"
        );
    }
}
