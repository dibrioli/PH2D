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
use ph2d_editor_core::ids;
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

const SRC: &str = include_str!("../src/paint_sections.rs");

/// **As ferramentas, DERIVADAS do vocabulário** ([`DrawMode::ALL`]).
///
/// ⚠️⚠️ **Era uma lista escrita à mão de 14, com um `assert_eq!(len, 14)` ao lado — e ela deixou
/// passar o 15.º modo (o Trim) em SILÊNCIO.** A asserção media o comprimento da própria lista, logo
/// concordava consigo mesma para sempre. *Um censo que se verifica contra a sua própria cópia não é
/// um censo*: a população tem de vir de quem a define.
const TODAS: &[DrawMode] = DrawMode::ALL;

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
        Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H),
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

/// ⭐ **O REPORT, medido nas 14 ferramentas**: a grade de tipos de forma sobe na Forma e em mais
/// nenhuma. O caminho de volta é o pill *Shape* da fileira TOOL, que nunca sai.
#[test]
fn the_shape_catalog_belongs_to_the_shape_tool_and_to_no_other() {
    let mut hero = hero_with_vector_panel();
    for &m in TODAS {
        paint_in(&mut hero, snap_of(m));
        assert_eq!(
            painted(&hero, ids::VECTOR_SECTION_SHAPE),
            m == DrawMode::Shape,
            "a grade de formas na ferramenta {m:?}"
        );
    }
}

/// Os knobs do Lápis: a única seção que já se escondia, e a guarda mudou-se para a tabela para
/// haver **uma** resposta à pergunta.
#[test]
fn the_pencil_knobs_belong_to_the_pencil() {
    let mut hero = hero_with_vector_panel();
    for &m in TODAS {
        paint_in(&mut hero, snap_of(m));
        assert_eq!(
            painted(&hero, ids::VECTOR_SECTION_PENCIL),
            m == DrawMode::Pencil,
            "os knobs do lapis na ferramenta {m:?}"
        );
    }
}

/// ⭐⭐ **A SIMETRIA é a excepção que a regra precisa**: opção de desenho, logo pertence aos modos
/// que autoram geometria — **mas o efeito dela sobrevive à troca de ferramenta**, e o interruptor
/// que a desliga é o único que existe.
///
/// ⛔ Escondê-la por modo com ela LIGADA deixaria o artista a ver o eixo na tela e sem nada para o
/// apagar. *Um controlo cujo efeito sobrevive ao modo não pode ter o único interruptor escondido
/// pelo modo.*
#[test]
fn the_symmetry_hides_outside_drawing_but_never_while_it_is_on() {
    let mut hero = hero_with_vector_panel();
    let autoram = [
        DrawMode::Pen,
        DrawMode::Pencil,
        DrawMode::Shape,
        DrawMode::Frame,
    ];
    for &m in TODAS {
        paint_in(&mut hero, snap_of(m));
        assert_eq!(
            painted(&hero, ids::VECTOR_SECTION_SYMMETRY),
            autoram.contains(&m),
            "a simetria DESLIGADA na ferramenta {m:?}"
        );
    }
    // …e ligada, ela aparece em TODAS — inclusive nas que não desenham nada.
    for &m in TODAS {
        let mut snap = snap_of(m);
        snap.symmetry.on = true;
        paint_in(&mut hero, snap);
        assert!(
            painted(&hero, ids::VECTOR_SECTION_SYMMETRY),
            "a simetria LIGADA sumiu na ferramenta {m:?} — e e' o unico interruptor que a desliga"
        );
    }
}

/// ⚠️ **A fileira TOOL nunca sai** — ela é o caminho de volta para toda seção que este escopo
/// esconde. Sem ela, esconder a grade de formas trancaria o artista fora da ferramenta.
#[test]
fn the_tool_row_survives_every_scope() {
    let mut hero = hero_with_vector_panel();
    for &m in TODAS {
        paint_in(&mut hero, snap_of(m));
        assert!(
            painted(&hero, ids::VECTOR_SECTION_TOOL),
            "a fileira TOOL sumiu na ferramenta {m:?}"
        );
    }
}

/// ⭐⭐⭐ **O CENSO: toda seção do corpo DECLARA de quem é.**
///
/// ⚠️ **Nem mais** — nenhuma chamada nua de `self.step` sobrevive dentro do `paint_body`; **nem
/// menos** — as duas metades contam a mesma população. *Um `Always` implícito, repetido 38 vezes,
/// foi o que pôs a grade de formas na ferramenta Select*: a tabela só serve se ninguém puder
/// entrar sem passar por ela.
#[test]
fn every_body_section_declares_which_tool_it_belongs_to() {
    let ini = SRC
        .find("pub(crate) fn paint_body")
        .expect("o orquestrador do corpo");
    let fim = SRC[ini..]
        .find("fn close_fold")
        .expect("o fim do orquestrador")
        + ini;
    let corpo = &SRC[ini..fim];

    let declaradas = corpo.matches("self.step_in(y, snap.mode, ").count();
    assert!(
        declaradas >= 39,
        "so' {declaradas} secoes declaram o escopo"
    );

    // ⚠️ A metade "nem mais": um `self.step(` cru no corpo é uma seção que entrou sem dizer de
    // quem é — e ela pintaria em toda ferramenta, que é exactamente o defeito.
    assert_eq!(
        corpo.matches("self.step(").count(),
        0,
        "uma secao do corpo ainda usa o passo CRU: ela pinta em toda ferramenta por omissao"
    );

    // ⚠️ **A ÚNICA excepção, nomeada**: a `path_section` não passa pelo `step` porque fecha a dobra
    // à mão (ela é a última e não leva separador de rodapé). Um segundo nome aqui significa que
    // alguém copiou esse contrato — e aí a excepção deixa de ser uma.
    let fora: Vec<&str> = ["self.path_section(y)"]
        .into_iter()
        .filter(|n| corpo.contains(n))
        .collect();
    assert_eq!(
        fora.len(),
        1,
        "a lista de secoes fora do passo mudou: {fora:?}"
    );
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
    ph2d_panel_vector::set_current_effects(false, &["Blur"], Vec::new());
    paint_in(&mut hero, snap_of(DrawMode::Select));
    assert!(
        !painted(&hero, ids::VECTOR_SECTION_EFFECTS),
        "sem caminho selecionado, o titulo EFFECTS sobe sozinho — cabecalho sem corpo"
    );

    // Controle: COM alvo ela aparece — senão este gate passaria por a seção estar morta.
    ph2d_panel_vector::set_current_effects(true, &["Blur"], Vec::new());
    paint_in(&mut hero, snap_of(DrawMode::Select));
    assert!(painted(&hero, ids::VECTOR_SECTION_EFFECTS));
    ph2d_panel_vector::set_current_effects(false, &[], Vec::new());
}

/// ⭐⭐ **As CINCO seções que são comandos sobre a seleção somem quando não há o que comandar** —
/// Boolean · Expand · Envelope · Arrange · Path.
///
/// ⚠️ Cada controle delas foi seguido do `paint` até ao consumidor: todos param numa guarda de
/// seleção (`input_dispatch.rs:124/191/350/416/447/478/1142`, `vec_expand.rs:100`,
/// `envelope_live.rs:147`, `node_ops.rs:145`). Com a seleção vazia elas eram cabeçalho e botões que
/// só sabiam recusar.
#[test]
fn the_selection_commands_vanish_when_there_is_nothing_to_command() {
    let mut hero = hero_with_vector_panel();
    let comandos = [
        ("Boolean", ids::VECTOR_SECTION_BOOLEAN),
        ("Expand", ids::VECTOR_SECTION_EXPAND),
        ("Envelope", ids::VECTOR_SECTION_ENVELOPE),
        ("Arrange", ids::VECTOR_SECTION_ARRANGE),
        ("Path", ids::VECTOR_SECTION_PATH),
    ];
    ph2d_panel_vector::set_current_selection_count(0);
    paint_in(&mut hero, snap_of(DrawMode::Select));
    for (nome, id) in comandos {
        assert!(
            !painted(&hero, id),
            "a secao {nome} sobe com a selecao vazia — todo botao dela so' sabe recusar"
        );
    }
    // Controle: com UM caminho selecionado as cinco voltam — senão este gate passaria por elas
    // estarem mortas, e não pela lei.
    ph2d_panel_vector::set_current_selection_count(1);
    paint_in(&mut hero, snap_of(DrawMode::Select));
    for (nome, id) in comandos {
        assert!(
            painted(&hero, id),
            "a secao {nome} nao voltou com uma selecao"
        );
    }
    ph2d_panel_vector::set_current_selection_count(0);
}

/// ⛔⛔ **AS DUAS QUE NÃO PODEM SUMIR, e a razão é uma SEGUNDA SELEÇÃO INVISÍVEL.**
///
/// *Blend* e *Morph* parecem irmãs das cinco acima e não são: o `Pick Shapes` troca o `DrawMode`
/// **sem olhar a seleção** (`tool_panel_event.rs:160`), e os dois botões correm sobre o
/// `vec_blend_picks` — uma lista que o `blend_pick_at` coleta e que **nunca toca no `PenTool`**
/// (`input_dispatch.rs:1362`).
///
/// *Uma regra de "esconde com a seleção vazia" aplicada a olho teria escondido justamente os dois
/// controles que ainda funcionam.*
#[test]
fn the_blend_and_morph_survive_an_empty_selection_because_picks_are_a_second_selection() {
    let mut hero = hero_with_vector_panel();
    ph2d_panel_vector::set_current_selection_count(0);
    paint_in(&mut hero, snap_of(DrawMode::PickBlend));
    assert!(
        painted(&hero, ids::VECTOR_SECTION_BLEND),
        "o Blend sumiu — e com ele o Pick Shapes, que e' como se escolhem as formas"
    );
    assert!(painted(&hero, ids::VECTOR_SECTION_MORPH));
    // E também na ferramenta comum: o modo Pick entra POR ali, então esconder a seção trancaria
    // a porta de entrada dela.
    paint_in(&mut hero, snap_of(DrawMode::Select));
    assert!(painted(&hero, ids::VECTOR_SECTION_BLEND));
    assert!(painted(&hero, ids::VECTOR_SECTION_MORPH));
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
        ("paint_brush.rs", include_str!("../src/paint_brush.rs")),
        (
            "paint_texture_pattern.rs",
            include_str!("../src/paint_texture_pattern.rs"),
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
    let porta = include_str!("../src/art_vocabulary.rs");
    for label in ["PICK", "CHANGE", "USE"] {
        let i = porta
            .find(&format!("const {label}: &str = \""))
            .unwrap_or_else(|| panic!("a porta perdeu o rotulo {label}"));
        let valor = &porta[i..porta[i..].find('\n').unwrap() + i];
        assert!(
            !valor.contains("Shape"),
            "o rotulo {label} voltou a prometer uma forma sozinha: {valor}"
        );
    }
}
