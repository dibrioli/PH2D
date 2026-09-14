//! ⭐⭐⭐ **O PAINEL DEIXA DE SER O DEPÓSITO** — cada comando vai ao sítio que a **D2** lhe dá.
//!
//! # ⛔⛔ O defeito, com número
//!
//! O painel `3D Model` tem **74 entradas** e só **8** são propriedades do objecto
//! (`docs/UI_New_and_Simple/00_DECISOES_DO_ENIO.md` §D2) — as outras **66 têm outro dono**. O
//! sintoma não é uma opinião: aquele painel precisou de **barra de rolagem** (report do Enio,
//! 2026-08-27), porque não cabia.
//!
//! # ⭐ E o corte da D2 é por ÂMBITO, o que dá TRÊS destinos e não um
//!
//! | fileiras | nº | vai para | porquê |
//! |---|---:|---|---|
//! | vistas + câmera | 9 | o pulldown de área (*View*) | é sobre **olhar** |
//! | verbos do gizmo + referencial | 5 | os chips `MOVE`/`ROT`/`SCALE`/`SPACE` **que já existiam** | é sobre **mover com a mão** |
//! | níveis de exportação | 3 | menu **File** | escrever um arquivo vale em **todo o app** |
//!
//! ⛔ **Os destinos não são gosto:** a D2 diz *«se o comando vale em todo o app vai à barra; se vale
//! só naquele editor vai ao cabeçalho dele»*, e a tabela de destino dela nomeia o *Arquivo* para o
//! `export.*`.
//!
//! # ⛔⛔ E o GIZMO não ganhou controlo novo — os dele estavam MORTOS
//!
//! > Enio, 2026-09-01 (com foto): *«esses botões de mover, rot e scale já existiam. só não estavam
//! > ligados a cada modo.»*
//!
//! Um 2.º pulldown *Gizmo* foi construído e **apagado no mesmo dia**. Os chips do trilho eram a 2.ª
//! espécie de controlo morto do `CLAUDE.md` §5.0: o clique chegava, a luz acendia, e o valor não
//! alcançava consumidor nenhum. *Um controlo morto e um controlo ausente dão o mesmo report, e as
//! curas são opostas.*
//!
//! # ⚠️ E a fila não cresce
//!
//! > Enio, 2026-08-31: *«esse app tem tablets e iPad como alvo. Não podemos ir perdendo espaço.»*
//!
//! A faixa própria (o *cabeçalho de área* da D2) foi construída e revertida no mesmo dia: `28 px`
//! permanentes. E **nove chips crus** na fila levam-na a **2 linhas até no iPad 12,9"** — o maior
//! dos três alvos —, com `2` chips a transbordar (medido pela mutação 6).
//! *Poupar altura gastando largura não poupa nada.*
//!
//! ⭐ **O orçamento medido é `3` chips de área** (sonda de 2026-09-01: com `4` o iPad 11 e o mini
//! passam a duas linhas), e usa-se `1`.
//!
//! # ⚠️ E o gate carrega num PIXEL, duas vezes
//!
//! Um `apply_event(Click(id))` sintético passaria com o chip **morto sob o dedo** — foi assim que o
//! `⋯` nasceu (sem `InteractiveState`) e assim que os quatro chips do vetor morreram.
//!
//! ⛔⛔ **E a segunda costura — *servir é fechar* — NÃO vive neste ficheiro nem no handler do
//! chip.** Medido por mutação em 2026-08-31: apagar o fecho do `chrome::tool_bar_overflow` deixa
//! todos os gates de gesto verdes. Quem fecha, sob o ponteiro, é a regra genérica do store
//! (`dispatch::pointer_down` fecha todo menu aberto num Down primário que não lhe pertença), e não
//! existe `Click` sem `Down` — a única excepção é a **paleta de comandos**, que tem gate próprio no
//! fim deste ficheiro.

use ph2d_editor_core::interaction::ContextMenuKind;
use ph2d_editor_core::screens::hero::{HeroScreen, tool_bar};
use ph2d_editor_core::widget::RailButtonSize;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind};
use ph2d_panel_model3d::{ModeChip, ModelIntent, ModelSnapshot};
use ph2d_text::TextSystem;

const TABLETS: [(&str, f32, f32); 3] = [
    ("iPad 12.9", 1366.0, 1024.0),
    ("iPad 11", 1194.0, 834.0),
    ("iPad mini", 1133.0, 744.0),
];

/// O retrato completo: as duas fileiras da vista (que vão ao pulldown), as duas do gizmo (que vão
/// aos chips do trilho) e a saída para arquivo (que vai ao menu *File*).
fn snapshot_with_area_commands() -> ModelSnapshot {
    let chip = |key, active| ModeChip { key, active };
    ModelSnapshot {
        views: vec![
            chip("panel.model3d.view.front", false),
            chip("panel.model3d.view.back", false),
            chip("panel.model3d.view.right", true),
            chip("panel.model3d.view.left", false),
            chip("panel.model3d.view.top", false),
            chip("panel.model3d.view.bottom", false),
        ],
        camera: vec![
            chip("panel.model3d.camera.ortho", false),
            chip("panel.model3d.camera.frame", false),
            chip("panel.model3d.camera.quad", false),
        ],
        modes: vec![
            chip("panel.model3d.mode.move", false),
            chip("panel.model3d.mode.rotate", true),
            chip("panel.model3d.mode.scale", false),
        ],
        frames: vec![
            chip("panel.model3d.frame.global", true),
            chip("panel.model3d.frame.local", false),
        ],
        exports: vec![
            chip("panel.model3d.export.draft", false),
            chip("panel.model3d.export.fine", false),
            chip("panel.model3d.export.max", false),
        ],
        view_label: "viewport.model3d.view.right",
        ..ModelSnapshot::default()
    }
}

fn hero(w: f32, h: f32, armed: bool) -> (HeroScreen, Rect) {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    hero.view.legacy_chrome = false;
    ph2d_panel_model3d::publish(snapshot_with_area_commands());
    ph2d_panel_model3d::publish_area_bar(&mut hero.store, armed);
    (hero, Rect::new(0.0, 0.0, w, h))
}

fn paint(h: &mut HeroScreen, vp: Rect) {
    let _ = paint_counting(h, vp);
}

/// [`paint`], devolvendo **quantos segmentos de caminho a cena de facto recebeu**.
///
/// ⛔⛔ **Ela existe porque um gate que lê o `ButtonState` mede a PUBLICAÇÃO, não a PINTURA** — é a
/// lição do §4.2 da auditoria do `source.lsystem`, onde o gate que prometia medir *«a queixa chega
/// a PIXEL»* media a linha reservada e ficava verde com a pintura apagada.
///
/// ⚠️ **O número absoluto não significa nada** (o ecrã inteiro entra, e o laço pinta duas vezes) —
/// o que significa é a DIFERENÇA entre dois retratos que só discordam no que se está a medir.
fn paint_counting(h: &mut HeroScreen, vp: Rect) -> u32 {
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    for _ in 0..2 {
        ph2d_editor_core::screens::hero::paint_hero_screen(h, vp, &mut scene, &mut text);
    }
    scene.inner().encoding().n_path_segments
}

fn pointer(kind: PointerKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: ph2d_host::PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    }
}

/// ⚠️ Down **e** Up pelo `dispatch_pointer`, como o `⋯` — um `Click` sintético salta exactamente a
/// metade que falha (o `HitIndex` resolver o ponto).
fn click_at(h: &mut HeroScreen, r: Rect) {
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let arena = bumpalo::Bump::new();
    for ev in [
        pointer(PointerKind::Down, x, y),
        pointer(PointerKind::Up, x, y),
    ] {
        let events =
            ph2d_editor_core::interaction::dispatch_pointer(&mut h.store, &h.hit_index, ev, &arena);
        let evs: Vec<_> = events.to_vec();
        for e in evs {
            h.apply_event(e);
        }
    }
}

/// Abre o chip do pulldown `slot` e devolve o rect dele — com o dedo, não com um `Click` fabricado.
fn open_area_menu(h: &mut HeroScreen, vp: Rect, slot: u32) {
    let chip = h
        .hit_index
        .rect_for(ph2d_editor_core::ids::area_menu_button(slot))
        .unwrap_or_else(|| {
            panic!("o pulldown {slot} da area nao foi registado no indice — nasceu invisivel")
        });
    click_at(h, chip);
    paint(h, vp);
}

/// ⭐⭐⭐ **O pulldown da VISTA abre, serve, e fecha-se ao servir.**
#[test]
fn the_view_pulldown_opens_serves_a_view_and_closes() {
    let (mut h, vp) = hero(1194.0, 834.0, true);
    let _ = ph2d_panel_model3d::drain_intents();
    paint(&mut h, vp);

    // ⛔ **Antes de abrir, os comandos NÃO estão em lado nenhum** — é isto que prova que eles
    // saíram do painel e não que ganharam um segundo sítio.
    assert!(
        h.hit_index
            .rect_for(ph2d_panel_model3d::ids::model3d_view_button(0))
            .is_none(),
        "a vista continua registada com o menu fechado — ela nao saiu do painel, ganhou um 2.o sitio"
    );

    open_area_menu(&mut h, vp, 0);
    assert!(
        matches!(
            h.store.context_menu().map(|r| r.kind),
            Some(ContextMenuKind::AreaCommands { slot: 0 })
        ),
        "o chip nao abriu o pulldown 0 — ele esta' morto sob o dedo"
    );

    let row = h
        .hit_index
        .rect_for(ph2d_panel_model3d::ids::model3d_view_button(0))
        .expect("a 1.a vista nao foi pintada no menu aberto");
    click_at(&mut h, row);

    let intents = ph2d_panel_model3d::drain_intents();
    assert!(
        intents
            .iter()
            .any(|i| matches!(i, ModelIntent::SetView { slot: 0 })),
        "carregar na vista nao pediu nada ao shell: {intents:?}"
    );
    assert!(
        h.store.context_menu().is_none(),
        "o menu ficou aberto POR CIMA da coisa que o clique acabou de fazer"
    );
}

/// ⭐⭐⭐ **OS CHIPS DO TRILHO CONDUZEM O GIZMO** — os que já existiam, com o dedo.
///
/// > Enio, 2026-09-01 (com foto): *«esses botões de mover, rot e scale já existiam. só não estavam
/// > ligados a cada modo.»*
///
/// ⛔⛔ **Eles eram a 2.ª espécie de controlo morto do `CLAUDE.md` §5.0:** o clique chegava, a luz
/// acendia, e o valor não alcançava consumidor nenhum — medido, `TOOL_TRANSLATE`/`ROTATE`/`SCALE` e
/// o `tool_space_local` do `SPACE` não tinham um único leitor na árvore.
#[test]
fn the_rail_verbs_drive_the_gizmo_and_so_does_the_space_chip() {
    let (mut h, vp) = hero(1194.0, 834.0, true);
    let _ = ph2d_panel_model3d::drain_intents();
    paint(&mut h, vp);

    let rot = h
        .hit_index
        .rect_for(ph2d_editor_core::ids::TOOL_ROTATE)
        .expect("o `ROT` do trilho nao esta' no indice de acerto");
    click_at(&mut h, rot);
    let intents = ph2d_panel_model3d::drain_intents();
    assert!(
        intents
            .iter()
            .any(|i| matches!(i, ModelIntent::SetGizmoMode { slot: 1 })),
        "carregar no `ROT` nao pediu o verbo ao shell — o chip voltou a ser morto: {intents:?}"
    );

    let space = h
        .hit_index
        .rect_for(ph2d_editor_core::ids::TOOL_SPACE)
        .expect("o `SPACE` do trilho nao esta' no indice de acerto");
    click_at(&mut h, space);
    let intents = ph2d_panel_model3d::drain_intents();
    assert!(
        intents
            .iter()
            .any(|i| matches!(i, ModelIntent::SetGizmoFrame { slot: 1 })),
        "carregar no `SPACE` nao pediu o referencial — com `Global` activo ele pede o SEGUINTE: \
         {intents:?}"
    );
}

/// ⭐⭐ **A LUZ do chip vem da CENA, e a FACE do `SPACE` também.**
///
/// ⚠️ Com o módulo armado o `chrome::rail_tools` **nunca corre** para estes ids (o painel consome-os
/// antes), logo o rádio que os acendia está desligado — quem os acende é o retrato.
/// *Fiar o clique não é fiar o ESTADO.*
#[test]
fn the_rail_verb_lights_up_from_the_scene_not_from_the_click() {
    let (h, _vp) = hero(1194.0, 834.0, true);
    // O retrato diz `rotate` activo (ver `snapshot_with_area_commands`).
    assert_eq!(
        h.store.button_state(ph2d_editor_core::ids::TOOL_ROTATE),
        Some(ph2d_editor_core::widget::ButtonState::Pressed),
        "o `ROT` nao acendeu com o gizmo em rotacao"
    );
    assert_eq!(
        h.store.button_state(ph2d_editor_core::ids::TOOL_TRANSLATE),
        Some(ph2d_editor_core::widget::ButtonState::Normal),
        "o `MOVE` ficou aceso com o gizmo em rotacao — os chips deixaram de ser exclusivos"
    );
    // E a face do `SPACE` lê o referencial: o retrato diz `Global`.
    assert!(
        !h.store.tool_space_local(),
        "a face do `SPACE` diz `Local` com o gizmo em `Global`"
    );
}

/// ⛔⛔⛔ **O `PIVOT` APAGA-SE quando o 3D toma o trilho — e ele NÃO é um chip morto.**
///
/// ⚠️ **Correcção de 2026-09-01:** a entrega 36 afirmou que o `TOOL_PIVOT` não tinha leitor nenhum.
/// **Tem dois**, e o `grep` que os devia ter mostrado foi cortado por um `head -20`:
/// `input_dispatch.rs` lê o `ButtonState` dele para armar o arrasto **MovePivot**, e o
/// `render_loop/snapshots.rs` para realçar o ponto do pivô.
///
/// ⛔ **E daí sai um defeito que a entrega 36 CRIOU:** com o módulo armado o painel consome os
/// verbos **antes** do `chrome::rail_tools`, logo o rádio exclusivo dele não corre — e um `PIVOT`
/// aceso de antes **fica aceso**. Dois chips acesos ao mesmo tempo, e pior: a ferramenta de pivô do
/// editor 2D continua **armada** por baixo de um canvas que é do 3D.
///
/// ⇒ quem publica o estado do trilho apaga-o. *O gizmo deste módulo tem três verbos; o pivô não é
/// um deles, logo ele não pode ser o activo.*
#[test]
fn the_pivot_chip_goes_dark_when_the_module_takes_the_rail() {
    let (mut h, _vp) = hero(1194.0, 834.0, true);
    // Alguém escolheu `PIVOT` no editor 2D antes de abrir o módulo.
    if let Some(ph2d_editor_core::interaction::InteractiveState::Button { state }) =
        h.store.get_mut(ph2d_editor_core::ids::TOOL_PIVOT)
    {
        *state = ph2d_editor_core::widget::ButtonState::Pressed;
    }
    ph2d_panel_model3d::publish_area_bar(&mut h.store, true);

    assert_eq!(
        h.store.button_state(ph2d_editor_core::ids::TOOL_PIVOT),
        Some(ph2d_editor_core::widget::ButtonState::Normal),
        "o `PIVOT` ficou aceso com o 3D a conduzir o trilho — dois chips acesos, e a ferramenta de \
         pivo do 2D armada por baixo de um canvas que nao e' dela"
    );
    assert_eq!(
        h.store.button_state(ph2d_editor_core::ids::TOOL_ROTATE),
        Some(ph2d_editor_core::widget::ButtonState::Pressed),
        "controlo: o verbo do retrato tem de continuar aceso"
    );
}

/// ⭐⭐⭐ **E a ÁREA não oferece uma segunda porta para o gizmo.**
///
/// ⛔⛔ Este gate existe porque a 1.ª versão desta obra **construiu** um pulldown *Gizmo* ao lado dos
/// chips que já faziam a pergunta. *Um controlo morto e um controlo ausente dão o mesmo report, e as
/// curas são opostas* — quem reconstruir a segunda porta parte aqui.
#[test]
fn the_area_offers_no_second_door_for_the_gizmo() {
    let (h, _vp) = hero(1194.0, 834.0, true);
    let faces: Vec<&str> = h.store.area_menus().iter().map(|m| &*m.face).collect();
    assert_eq!(
        h.store.area_menus().len(),
        1,
        "a area publicou mais do que o pulldown da VISTA ({faces:?}) — o gizmo tem chips no trilho"
    );
    let verbs = ["Move", "Rotate", "Size", "Global", "Local"];
    for menu in h.store.area_menus() {
        for row in &menu.rows {
            let label = row.label().unwrap_or("");
            assert!(
                !verbs.contains(&label),
                "`{label}` esta' num pulldown da area E no trilho — dois sitios para o mesmo verbo"
            );
        }
    }
}

/// ⭐⭐ **A FACE de cada pulldown é a LEITURA do estado** — e as duas são leituras DIFERENTES.
///
/// ⛔ É isto que justifica dois chips em vez de um: um chip cuja face não distingue nada custa a
/// mesma largura e não informa.
#[test]
fn each_pulldown_wears_its_own_reading() {
    let (h, _vp) = hero(1194.0, 834.0, true);
    let faces: Vec<&str> = h.store.area_menus().iter().map(|m| &*m.face).collect();
    assert_eq!(
        faces,
        vec!["Right"],
        "a face nao e' a leitura do retrato (a vista `right`)"
    );
    let labels: Vec<&str> = h.store.area_menus().iter().map(|m| &*m.label).collect();
    assert_eq!(
        labels,
        vec!["View"],
        "o rotulo nao nomeia o grupo que o chip abre"
    );
}

/// Quantos chips a ÁREA acrescenta à fila, e em quantas linhas ela cabe — a medição que o orçamento
/// dos `3` chips pede, num sítio só.
fn area_chips_and_lines(h: &HeroScreen, vp: Rect) -> (usize, usize) {
    let bands = ph2d_editor_core::screens::layout::ChromeBands {
        rail_w: 0.0,
        top_bar_h: ph2d_editor_core::screens::hero::menu_bar::MENU_BAR_H,
        tool_bar_h: tool_bar::tool_bar_h(RailButtonSize::Small, 1),
        ..ph2d_editor_core::screens::layout::ChromeBands::DEFAULT
    };
    let area_w = ph2d_editor_core::screens::layout::HeroLayout::for_viewport_bands(
        vp,
        false,
        bands,
        ph2d_editor_core::screens::layout::CenterSplit::None,
        ph2d_editor_core::screens::layout::DockSides::BOTH,
    )
    .draw_area
    .w;
    let (rail, over) = tool_bar::bar_split(&h.store, false, false, area_w);
    let area_ids: Vec<_> = (0..ph2d_editor_core::ids::MAX_AREA_MENUS)
        .map(ph2d_editor_core::ids::area_menu_button)
        .collect();
    let added = rail
        .entries
        .iter()
        .chain(over.iter())
        .filter(|e| e.node_id().is_some_and(|id| area_ids.contains(&id)))
        .count();
    let lines =
        ph2d_editor_core::widget::horizontal_lines(&rail, area_w - 16.0, RailButtonSize::Small);
    (added, lines)
}

/// ⭐⭐⭐ **O 2.º PULLDOWN — o SOMBREAMENTO** (`docs/Render3d/05`): ele aparece **quando há o que
/// oferecer**, a face dele é a leitura do modo do viewport activo, e servir uma linha pede o modo ao
/// shell — com o dedo, não com um `Click` fabricado.
///
/// ⚠️ **E a fila continua numa linha nos três tablets, agora com DOIS chips.** O orçamento medido é
/// `3` (2026-09-01); este gate mede o que o desenho de facto gasta, e é ele que reprova no dia em que
/// alguém acrescentar o terceiro sem medir.
#[test]
fn the_shading_pulldown_serves_the_render_mode_and_the_bar_is_still_one_line() {
    let chip = |key, active| ModeChip { key, active };
    let with_shading = ModelSnapshot {
        shadings: vec![
            chip("panel.model3d.shading.matcap", true),
            chip("panel.model3d.shading.render", false),
        ],
        looks: vec![
            chip("panel.model3d.look.standard", true),
            chip("panel.model3d.look.neutral", false),
        ],
        exposures: vec![
            chip("panel.model3d.exposure.minus2", false),
            chip("panel.model3d.exposure.minus1", false),
            chip("panel.model3d.exposure.zero", true),
            chip("panel.model3d.exposure.plus1", false),
            chip("panel.model3d.exposure.plus2", false),
        ],
        shading_label: "panel.model3d.shading.matcap",
        ..snapshot_with_area_commands()
    };
    let (mut h, vp) = hero(1194.0, 834.0, true);
    ph2d_panel_model3d::publish(with_shading.clone());
    ph2d_panel_model3d::publish_area_bar(&mut h.store, true);
    let _ = ph2d_panel_model3d::drain_intents();
    paint(&mut h, vp);

    let labels: Vec<&str> = h.store.area_menus().iter().map(|m| &*m.label).collect();
    let faces: Vec<&str> = h.store.area_menus().iter().map(|m| &*m.face).collect();
    assert_eq!(
        labels,
        vec!["View", "Shading"],
        "a area nao publicou o 2.o pulldown"
    );
    assert_eq!(
        faces,
        vec!["Right", "Matcap"],
        "a face do 2.o pulldown nao e' a leitura do modo do viewport"
    );

    open_area_menu(&mut h, vp, 1);
    let row = h
        .hit_index
        .rect_for(ph2d_panel_model3d::ids::model3d_shading_button(1))
        .expect("o `Render` nao foi pintado no menu aberto");
    click_at(&mut h, row);
    let intents = ph2d_panel_model3d::drain_intents();
    assert!(
        intents
            .iter()
            .any(|i| matches!(i, ModelIntent::SetShading { slot: 1 })),
        "carregar no `Render` nao pediu o modo ao shell: {intents:?}"
    );

    for (name, w, ht) in TABLETS {
        let (mut h, vp) = hero(w, ht, true);
        ph2d_panel_model3d::publish(with_shading.clone());
        ph2d_panel_model3d::publish_area_bar(&mut h.store, true);
        let (added, lines) = area_chips_and_lines(&h, vp);
        println!("{name:11} a area acrescenta {added} chip(s) e a fila cabe em {lines} linha(s)");
        assert_eq!(added, 2, "{name}: o desenho usa DOIS chips de area");
        assert_eq!(lines, 1, "{name}: a fila precisa de {lines} linhas");
    }
}

/// O retrato com o pulldown do sombreamento, dizendo **qual chip de cada fileira está aceso**.
fn shading_snapshot(
    modo: Option<usize>,
    vista: Option<usize>,
    exposicao: Option<usize>,
) -> ModelSnapshot {
    let fileira = |chaves: &[&'static str], aceso: Option<usize>| -> Vec<ModeChip> {
        chaves
            .iter()
            .enumerate()
            .map(|(i, key)| ModeChip {
                key,
                active: aceso == Some(i),
            })
            .collect()
    };
    ModelSnapshot {
        shadings: fileira(
            &[
                "panel.model3d.shading.matcap",
                "panel.model3d.shading.render",
            ],
            modo,
        ),
        looks: fileira(
            &["panel.model3d.look.standard", "panel.model3d.look.neutral"],
            vista,
        ),
        exposures: fileira(
            &[
                "panel.model3d.exposure.minus2",
                "panel.model3d.exposure.minus1",
                "panel.model3d.exposure.zero",
                "panel.model3d.exposure.plus1",
                "panel.model3d.exposure.plus2",
            ],
            exposicao,
        ),
        shading_label: "panel.model3d.shading.matcap",
        ..snapshot_with_area_commands()
    }
}

/// Abre o pulldown do sombreamento com este retrato e devolve o `HeroScreen` **com o menu aberto**.
fn hero_with_shading_menu_open(
    modo: Option<usize>,
    vista: Option<usize>,
    exposicao: Option<usize>,
) -> (HeroScreen, Rect) {
    let (mut h, vp) = hero(1194.0, 834.0, true);
    ph2d_panel_model3d::publish(shading_snapshot(modo, vista, exposicao));
    ph2d_panel_model3d::publish_area_bar(&mut h.store, true);
    let _ = ph2d_panel_model3d::drain_intents();
    paint(&mut h, vp);
    open_area_menu(&mut h, vp, 1);
    // ⚠️ **Re-publicar não é cerimónia:** o `publish_area_bar` corre em TODO quadro no app, e o
    // `Down` que abriu o menu escreveu `Pressed` no chip sob o dedo. Sem isto o retrato do gate
    // discordaria do que o produto teria no quadro seguinte.
    ph2d_panel_model3d::publish(shading_snapshot(modo, vista, exposicao));
    ph2d_panel_model3d::publish_area_bar(&mut h.store, true);
    let _ = ph2d_panel_model3d::drain_intents();
    (h, vp)
}

/// ⭐⭐⭐ **A MARCA CHEGA AO PIXEL, e um ponto é um ESTADO** (report do dono, 2026-09-13: *«coloque a
/// marca de seleção no render selecionado»*).
///
/// ⛔⛔ **Um gate que lesse o `ButtonState` mediria a PUBLICAÇÃO, e a publicação já estava certa** —
/// o `area_bar::entries` escreve `Pressed` no chip aceso desde que o pulldown existe. O que faltava
/// era o pintor LER isso, e é por isso que este gate conta geometria: ele reprova exactamente no
/// estado em que o dono abriu o menu e viu nove linhas iguais.
///
/// ⚠️ **A régua é a DIFERENÇA entre três retratos que só discordam no `active`** — o número
/// absoluto conta o ecrã inteiro e não significa nada. E a 2.ª asserção é a que separa *«pintou
/// alguma coisa»* de *«pintou UM ponto por fileira acesa»*: três estados têm de custar exactamente
/// o triplo de um.
#[test]
fn the_mark_reaches_the_pixel_and_one_bullet_is_one_state() {
    let conta = |modo, vista, exposicao| {
        let (mut h, vp) = hero_with_shading_menu_open(modo, vista, exposicao);
        paint_counting(&mut h, vp)
    };
    let nenhuma = conta(None, None, None);
    let uma = conta(Some(1), None, None);
    let tres = conta(Some(1), Some(1), Some(4));
    println!("segmentos: 0 acesas {nenhuma} · 1 acesa {uma} · 3 acesas {tres}");
    assert!(
        uma > nenhuma,
        "a linha ACESA nao pintou nada a mais do que uma apagada ({uma} contra {nenhuma}) — o menu \
         abre com as nove linhas iguais, que e' o report do dono"
    );
    assert_eq!(
        tres - nenhuma,
        (uma - nenhuma) * 3,
        "tres fileiras acesas nao custaram tres pontos ({tres} - {nenhuma} contra 3 x ({uma} - \
         {nenhuma})) — a marca nao e' por FILEIRA"
    );
}

/// ⭐⭐⭐ **UM RISCO ENTRE AS FILEIRAS** — a prova de que as nove linhas são **três perguntas**.
///
/// ⚠️ **A régua é o índice de acerto e não a cena**, e de propósito: ela mede *onde as linhas
/// ficaram*, que é o que responde *«há alguma coisa entre elas?»*. Duas linhas da mesma fileira
/// ficam a `ROW_H` uma da outra; atravessar uma fronteira custa `ROW_H` **mais a banda do risco** —
/// e os dois riscos têm de medir o mesmo, senão não é uma lei, são dois números.
///
/// **Mutação que deve sangrar:** apagar o `push(ToolRailEntry::Divider)` do `area_bar::entries`, ou
/// o braço `MenuRow::Divider` do pintor — nos dois casos as três distâncias colapsam em `ROW_H`.
#[test]
fn a_rule_between_the_rows_says_they_are_three_questions() {
    let (h, _vp) = hero_with_shading_menu_open(Some(0), Some(0), Some(2));
    let y = |id| {
        h.hit_index
            .rect_for(id)
            .unwrap_or_else(|| panic!("a linha {id:?} nao foi pintada no menu aberto"))
            .y
    };
    let matcap = y(ph2d_panel_model3d::ids::model3d_shading_button(0));
    let render = y(ph2d_panel_model3d::ids::model3d_shading_button(1));
    let standard = y(ph2d_panel_model3d::ids::model3d_look_button(0));
    let neutral = y(ph2d_panel_model3d::ids::model3d_look_button(1));
    let menos2 = y(ph2d_panel_model3d::ids::model3d_exposure_button(0));

    let dentro = render - matcap;
    let primeiro = standard - render - dentro;
    let segundo = menos2 - neutral - dentro;
    println!("passo dentro da fileira {dentro} · riscos {primeiro} e {segundo}");
    assert!(
        primeiro > 0.0,
        "nao ha nada entre o modo e a vista da cena: as duas fileiras leem-se como uma lista so"
    );
    assert!(
        (primeiro - segundo).abs() < 0.01,
        "os dois riscos medem diferente ({primeiro} contra {segundo}) — nao e' uma lei, sao dois \
         numeros"
    );
}

/// ⭐⭐⭐ **A SAÍDA vive no menu do APP** — e as linhas dele aparecem DEPOIS das que já lá estavam.
#[test]
fn the_file_menu_serves_an_export_and_the_module_owns_it() {
    let (mut h, vp) = hero(1194.0, 834.0, true);
    let _ = ph2d_panel_model3d::drain_intents();
    paint(&mut h, vp);

    let title = h
        .hit_index
        .rect_for(ph2d_editor_core::ids::MENUBAR_FILE)
        .expect("o titulo `File` nao esta' no indice de acerto");
    click_at(&mut h, title);
    paint(&mut h, vp);

    // ⚠️ As linhas do APP continuam lá — a contribuição SOMA, nunca substitui.
    assert!(
        h.hit_index
            .rect_for(ph2d_editor_core::ids::CTX_MENU_SAVE)
            .is_some(),
        "as linhas que o menu `File` ja' tinha desapareceram — a contribuicao substituiu em vez de somar"
    );

    let row = h
        .hit_index
        .rect_for(ph2d_panel_model3d::ids::model3d_export_button(1))
        .expect("`Export Fine` nao foi pintado no menu `File`");
    click_at(&mut h, row);
    let intents = ph2d_panel_model3d::drain_intents();
    assert!(
        intents
            .iter()
            .any(|i| matches!(i, ModelIntent::Export { slot: 1 })),
        "carregar em `Export Fine` nao pediu nada ao shell: {intents:?}"
    );
}

/// ⭐⭐ **Fechar o módulo tira os chips da fila E as linhas do menu do app, no MESMO quadro.**
///
/// ⚠️ É a lei do `⋯`: publicado em todo quadro, vazio incluído. Sem o ramo vazio o chip ficava lá,
/// a despachar para um painel que já não está — e o *File* ficava com três linhas que não exportam.
#[test]
fn closing_the_module_takes_the_commands_off_the_bar_and_off_the_file_menu() {
    let (mut h, vp) = hero(1194.0, 834.0, true);
    paint(&mut h, vp);
    assert!(
        h.hit_index
            .rect_for(ph2d_editor_core::ids::area_menu_button(0))
            .is_some(),
        "controlo: com o modulo armado o pulldown tem de estar la'"
    );

    ph2d_panel_model3d::publish_area_bar(&mut h.store, false);
    paint(&mut h, vp);
    for slot in 0..ph2d_editor_core::ids::MAX_AREA_MENUS {
        assert!(
            h.hit_index
                .rect_for(ph2d_editor_core::ids::area_menu_button(slot))
                .is_none(),
            "o pulldown {slot} sobreviveu ao fecho do modulo"
        );
    }
    assert!(
        h.store.area_menus().is_empty(),
        "os comandos da area sobreviveram ao fecho do modulo"
    );
    assert!(
        h.store
            .menu_contrib(ContextMenuKind::MenuBarFile)
            .is_empty(),
        "a exportacao sobreviveu no menu `File` depois de o modulo fechar"
    );

    // E o menu do app abre, sem as linhas do módulo.
    let title = h
        .hit_index
        .rect_for(ph2d_editor_core::ids::MENUBAR_FILE)
        .expect("o titulo `File` nao esta' no indice de acerto");
    click_at(&mut h, title);
    paint(&mut h, vp);
    assert!(
        h.hit_index
            .rect_for(ph2d_editor_core::ids::CTX_MENU_SAVE)
            .is_some(),
        "controlo: o menu `File` tem de abrir com as linhas dele"
    );
    assert!(
        h.hit_index
            .rect_for(ph2d_panel_model3d::ids::model3d_export_button(0))
            .is_none(),
        "`Export Draft` continua no menu `File` com o modulo 3D fechado"
    );
}

/// ⭐⭐⭐ **A faixa continua a ser UMA linha nos três tablets, com o módulo armado.**
///
/// ⚠️ E o gate imprime **quantos** chips a área acrescenta: se um dia forem nove em vez de um,
/// esta é a linha que o diz antes de a largura o dizer. ⭐ O orçamento medido em 2026-09-01 é `3`.
#[test]
fn the_area_costs_one_chip_and_the_bar_is_still_one_line() {
    for (name, w, ht) in TABLETS {
        let (h, vp) = hero(w, ht, true);
        let bands = ph2d_editor_core::screens::layout::ChromeBands {
            rail_w: 0.0,
            top_bar_h: ph2d_editor_core::screens::hero::menu_bar::MENU_BAR_H,
            tool_bar_h: tool_bar::tool_bar_h(RailButtonSize::Small, 1),
            ..ph2d_editor_core::screens::layout::ChromeBands::DEFAULT
        };
        let area_w = ph2d_editor_core::screens::layout::HeroLayout::for_viewport_bands(
            vp,
            false,
            bands,
            ph2d_editor_core::screens::layout::CenterSplit::None,
            ph2d_editor_core::screens::layout::DockSides::BOTH,
        )
        .draw_area
        .w;
        let (rail, over) = tool_bar::bar_split(&h.store, false, false, area_w);
        let area_ids: Vec<_> = (0..ph2d_editor_core::ids::MAX_AREA_MENUS)
            .map(ph2d_editor_core::ids::area_menu_button)
            .collect();
        let added = rail
            .entries
            .iter()
            .chain(over.iter())
            .filter(|e| e.node_id().is_some_and(|id| area_ids.contains(&id)))
            .count();
        println!(
            "{name:11} a area acrescenta {added} chip(s); fila {:2} + {:2} atras do dots",
            rail.entries.len(),
            over.len()
        );
        assert_eq!(
            added, 1,
            "{name}: a area pos {added} chips na fila — o orcamento MEDIDO e' 3, e o desenho usa 1 \
             (o gizmo vive nos chips do trilho)"
        );
        let lines =
            ph2d_editor_core::widget::horizontal_lines(&rail, area_w - 16.0, RailButtonSize::Small);
        assert_eq!(lines, 1, "{name}: a fila precisa de {lines} linhas");
    }
}

/// ⭐⭐⭐ **O CAMINHO SEM DEDO** — a paleta de comandos, que não tem `Down` nenhum.
///
/// ⛔⛔ **Este gate existe porque uma MUTAÇÃO sobreviveu.** Sob o ponteiro, quem fecha um menu ao
/// servir é a regra genérica do store (`pointer_down` + `click_belongs_to_the_open_menu`), e não o
/// handler do chip: apagar o interruptor de lá deixava o gate de gesto verde.
///
/// ⚠️ Mas a **paleta de comandos global** levanta `apply_event(Click(id))` sem ponteiro nenhum — e
/// ela projecta a lista que a fila pinta. Por esse caminho o fecho genérico nunca corre, e sem o
/// interruptor escolher o chip **re-abriria** um menu já aberto.
#[test]
fn the_palette_path_toggles_the_menu_because_it_has_no_pointer_down() {
    let (mut h, vp) = hero(1194.0, 834.0, true);
    paint(&mut h, vp);
    let click = ph2d_editor_core::interaction::WidgetEvent::Click(
        ph2d_editor_core::ids::area_menu_button(0),
    );
    assert!(
        h.apply_event(click),
        "controlo: o chip nao respondeu ao clique sintetico"
    );
    assert!(
        matches!(
            h.store.context_menu().map(|r| r.kind),
            Some(ContextMenuKind::AreaCommands { slot: 0 })
        ),
        "a paleta nao abriu o menu"
    );
    assert!(h.apply_event(click));
    assert!(
        h.store.context_menu().is_none(),
        "pela paleta o 2.o pick RE-ABRIU o menu — o interruptor do chip e' a unica coisa que o fecha \
         quando nao ha' Down"
    );
}
