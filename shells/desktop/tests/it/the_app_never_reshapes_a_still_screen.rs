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
    // ⚠️ **O chrome legado (`F9`) também, e a razão é o controlo de vacuidade abaixo** (2026-10-02):
    //    quando as legendas da fila deixaram de ser cortadas (`M…`, `S…`), a elisão deixou de moldar
    //    os prefixos de cada uma e o app caiu de >`1 024` para `1 021` textos — abaixo do tecto que
    //    esta fixtura existe para passar. O chrome legado é conteúdo REAL do app (a barra do topo e
    //    o trilho), não enchimento.
    h.view.legacy_chrome = true;
    // ⚠️ **E o menu Add aberto, pela mesma razão** (2026-10-04): a F3 do Vector tirou o pill VECTOR
    //    e a fileira do painel passou a mostrar só as ferramentas do modo — `1 014` textos. A paleta
    //    do `+` é conteúdo real do app, e é a que o artista abre para criar.
    h.store
        .open_command_palette(ph2d_editor_core::object_add::build(
            &[
                ph2d_editor_core::object_add::CORE,
                ph2d_app_vec::object_add::ENTRIES,
                ph2d_app_flip::object_add::ENTRIES,
                ph2d_app_components::object_add::ENTRIES,
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
