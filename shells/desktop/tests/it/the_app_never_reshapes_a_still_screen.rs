//! ⭐⭐⭐ **O APP PARADO NÃO VOLTA A MOLDAR TEXTO** — a metade de PRODUTO do penhasco de 2026-09-10.
//!
//! A metade de MECANISMO vive em `ph2d-text/tests/a_still_screen_never_reshapes_its_text.rs` (a
//! cache roda em vez de se deitar fora). Esta afirma o que ela compra **no ecrã que o artista
//! abre**: com todos os painéis abertos, um quadro sem input não paga moldagem nenhuma.
//!
//! # A medição que o obrigou
//!
//! | painéis | textos distintos | moldagens em REGIME | ms/quadro (debug) |
//! |---|---|---|---|
//! | 24 | `775` | `0` | `4,87` |
//! | 26 | `1 110` | `1 109` | `157,91` |
//! | 26, depois da cura | `1 106` | **`0`** | **`7,81`** |
//!
//! ⚠️ **`+43 %` de conteúdo dava `29×` de relógio, e a saída desenhada crescia `1,5×`** — *um
//! custo que não aparece no que se vê não é volume: é trabalho deitado fora.*
//!
//! # ⛔ Por que este gate mora no SHELL
//!
//! O conjunto de trabalho que produz o fenómeno é o do **app inteiro** — a `ph2d-text` sozinha
//! não o tem, e a `ph2d-panel-registry-init` liga 22 dos 26 painéis (as features pobres do
//! `CLAUDE.md` §2). Aqui os 26 existem.

use ph2d_editor_core::screens::hero::{HeroScreen, paint_hero_screen};
use ph2d_editor_core::zones::Rect;
use ph2d_text::{LAYOUT_CACHE_CAP, TextSystem};

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1366.0,
    h: 1024.0,
};

#[test]
fn a_still_frame_with_every_panel_open_shapes_no_text() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut h = HeroScreen::new(ph2d_editor_core::NodeId(1));
    // ⚠️ **O chrome legado (`F9`) SAIU da fixtura em 2026-10-07 (D10)** — entrou a 02/10 só para
    //    passar o tecto (`1 021` textos), e a fila de ferramentas, que é o ecrã REAL, nem era pintada
    //    com ele ligado. Hoje a fila leva o produtor do vetor (abaixo) e o app molda `1 043`.
    // ⚠️ **E as BARRAS do vetor publicadas** (D10, 2026-10-07): seis secções do painel Vector (o
    //    gesto) mudaram-se para a fila ② — sem elas a fixtura caía para `1 007` textos. O texto não
    //    sumiu, mudou de sítio: a fila do lápis com a simetria radial é o que o artista vê no lugar.
    ph2d_panel_vector::set_current_vector_style(Some(ph2d_tool_vector::VectorStyleSnapshot {
        mode: ph2d_tool_vector::DrawMode::Pencil,
        symmetry: ph2d_vec_scene::symmetry::SymmetryStyle {
            on: true,
            kind: ph2d_vec_scene::symmetry::SymmetryKind::Radial,
            ..ph2d_vec_scene::symmetry::SymmetryStyle::default()
        },
        ..ph2d_tool_vector::VectorStyleSnapshot::default()
    }));
    // F16 (15/10): uma forma selecionada — a barra de controle inteira (nome, Profile ▾, Type ▾,
    //    Opacity), que é o ecrã que o artista vê ao trabalhar.
    ph2d_panel_vector::set_current_selection_count(1);
    ph2d_panel_vector::state::set_current_selection_name(Some("Star 1".to_owned()));
    ph2d_panel_vector::set_stroke_paint_kind(Some(
        ph2d_panel_vector::state::StrokePaintKind::Solid,
    ));
    ph2d_panel_vector::state::set_current_appearance(Some(ph2d_panel_vector::state::Appearance {
        opacity: 1.0,
        blend: ph2d_vec_scene::BlendMode::Normal,
        layers: Vec::new(),
    }));
    let barras = ph2d_panel_vector::bars::tool_bars(&h.store, None, Vec::new());
    h.store.set_tool_bars(Some(barras));
    // F16 (15/10): o vetor já não publica pulldowns de área — a ② é a barra de controle dele. Os
    //    botões genéricos saíram da fila e o app caiu para `1 005` textos; ⇒ as secções Snap e
    //    Boolean do painel ABERTAS (nascem recolhidas; abri-las é um clique do artista). ⚠️ Abrir o
    //    painel do traço da barra NÃO serve: ele molda os MESMOS textos da secção Stroke (`+1`).
    h.store.set_area_commands(Vec::new(), Vec::new());
    // ⚠️ **E o grupo ▸ Advanced ABERTO** (F17, 16/10): a Boolean mudou-se para dentro dele com
    //    Expand, Blend, Morph, Envelope, Pattern on Path e Contour, e o grupo nasce dobrado — sem
    //    abri-lo a Boolean aberta acima não pintava, nem as outras seis, e a fixtura caiu para `1 024`
    //    textos (o tecto); aberto, `1 130`. O texto não sumiu do app: está a um clique, como o do
    //    Snap. Prova (16/10): com a cache a `clear` em vez de rodar, esta fixtura molda `1 105`
    //    textos no quadro parado (vermelho); sem o grupo, `1 024` cabem numa geração, a cache
    //    partida nunca roda e só o controlo de vacuidade abaixo o diz.
    for s in [
        ph2d_tool_vector::ids::VECTOR_SECTION_SNAP,
        ph2d_panel_vector::ids::VECTOR_SECTION_ADVANCED,
        ph2d_tool_vector::ids::VECTOR_SECTION_BOOLEAN,
    ] {
        h.store.set_collapsed(s, false);
    }

    // ⚠️ **E o menu Add aberto, pela mesma razão** (2026-10-04): a F3 do Vector tirou o pill VECTOR
    //    e a fileira do painel passou a mostrar só as ferramentas do modo — `1 014` textos. A paleta
    //    do `+` é conteúdo real do app, e é a que o artista abre para criar.
    // ⚠️ **Com as SETE famílias do menu real** (`render_loop/fase_object_add.rs::FAMILIES`, 2026-10-13): a
    //    fixtura montava quatro; oito linhas saíram do Tuning do Wet Paint para o Painter (as cerdas para o
    //    Shape, a física da água para o cartão, fora do ecrã desta fixtura) e ela caiu para `1 017` textos.
    h.store
        .open_command_palette(ph2d_editor_core::object_add::build(
            &[
                ph2d_editor_core::object_add::CORE,
                ph2d_app_vec::object_add::ENTRIES,
                ph2d_app_flip::object_add::ENTRIES,
                ph2d_app_skeleton::object_add::ENTRIES,
                ph2d_app_motion::motion_objeto::object_add::ENTRIES,
                ph2d_app_components::object_add::ENTRIES,
                ph2d_app_light2d::object_add::ENTRIES,
            ],
            &|_| None,
        ));
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        for p in reg.panels() {
            h.panel_visibility.insert(p.manifest.id, true);
        }
    });
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();

    // ⚠️ **A cena RESETA por quadro, como no produto** — sem isso mede-se uma cena a crescer.
    scene.reset();
    paint_hero_screen(&mut h, VIEWPORT, &mut scene, &mut text);
    let primeiro = text.shapes();

    // ⛔ **O CONTROLO DE VACUIDADE, e é ele que dá dentes a este gate:** o fenómeno só existe com
    //    o conjunto de trabalho ACIMA do tecto de uma geração. Se um dia o app tiver menos texto
    //    que isso, esta fixtura deixa de o produzir — e é melhor saber por uma reprovação do que
    //    por um verde que não mede nada.
    assert!(
        primeiro > LAYOUT_CACHE_CAP as u64,
        "o app inteiro moldou {primeiro} textos, que não passa o tecto de uma geração \
         ({LAYOUT_CACHE_CAP}) — esta fixtura já não produz o transbordo que ela existe para medir"
    );

    for _ in 0..3 {
        scene.reset();
        paint_hero_screen(&mut h, VIEWPORT, &mut scene, &mut text);
    }
    let antes = text.shapes();
    scene.reset();
    paint_hero_screen(&mut h, VIEWPORT, &mut scene, &mut text);
    let no_quadro = text.shapes() - antes;

    assert_eq!(
        no_quadro, 0,
        "um quadro SEM INPUT moldou {no_quadro} textos com todos os painéis abertos — a cache \
         está a perder o conjunto de trabalho, e o preço medido em 2026-09-10 foi `157,91 ms` \
         por quadro contra `7,81`"
    );
}
