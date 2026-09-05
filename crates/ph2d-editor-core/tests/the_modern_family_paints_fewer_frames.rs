//! ⭐⭐⭐ **A família moderna EMITE MENOS GEOMETRIA — a moldura que a pele plana apagou não está
//! na cena.**
//!
//! O gate irmão (`every_frame_goes_through_the_theme_door`) lê o FONTE; este carrega no PIXEL:
//! pinta a galeria inteira de widgets duas vezes — no `forge` (clássico) e no `dark` (Godot) — e
//! conta os caminhos que o Vello recebeu. O clássico traça uma moldura por controlo; o moderno
//! não. ⇒ o moderno tem de emitir **estritamente menos** caminhos.
//!
//! ⚠️ **Conta CAMINHOS (`n_paths`), não segmentos:** um rectângulo arredondado de raio 4 e um de
//! raio 12 têm o mesmo número de caminhos e segmentos diferentes — e o que se mede aqui é a
//! moldura a mais, não o raio.
//!
//! ⚠️ **E é a galeria, não um widget:** uma régua sobre um controlo só mede esse controlo; a
//! galeria pinta os 42, e é o conjunto que o dono vê.

use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::widget::showcase::paint_showcase_body;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

fn paths_of_the_gallery(theme: Theme) -> u32 {
    ph2d_editor_core::test_support::ensure_panel_registry();
    let hero = HeroScreen::new(NodeId(1));
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let mut hit = HitIndex::new();
    paint_showcase_body(
        Rect::new(0.0, 0.0, 420.0, 2400.0),
        &mut scene,
        &mut text,
        theme,
        &mut hit,
        &hero.store,
    );
    scene.inner().encoding().n_paths
}

/// **O `dark` emite menos caminhos do que o `forge`, na mesma galeria.**
///
/// **Mutação que deve sangrar:** fazer `visuals::frame` devolver `Frame::Classic` para todo tema
/// — os pintores voltam a traçar no moderno e as duas contagens igualam.
#[test]
fn the_modern_family_paints_fewer_frames_than_the_classic() {
    let classic = paths_of_the_gallery(Theme::Forge);
    let modern = paths_of_the_gallery(Theme::Dark);
    assert!(
        classic > 0 && modern > 0,
        "a galeria nao pintou nada ({classic} / {modern}) — a regua esta' partida"
    );
    assert!(
        modern < classic,
        "o tema moderno emitiu {modern} caminhos contra {classic} do classico: a pele plana nao \
         apagou moldura nenhuma"
    );
}

/// ⭐⭐ **A PORTA sozinha tira EXACTAMENTE uma moldura** — medido num controlo cuja única porta é
/// o `stroke_frame`: um segmento em repouso é *fundo + contorno* no clássico (2 caminhos) e *só
/// fundo* no moderno (1). O texto vai por glifos e não entra na contagem.
///
/// ⛔⛔ **Existe porque a régua da galeria deixou uma mutação SOBREVIVER**: com `visuals::frame` a
/// devolver `Classic` para todo tema, a galeria continuava a emitir *menos* no `dark` — o painel
/// e o botão entram na tabela por OUTRAS portas (`Chrome.panel_border`, `Widgets.bg_stroke`), e
/// um «menos» qualquer passava por pele plana. *Um gate de desigualdade não vê a porta que não
/// é a única.* Este mede a igualdade `classic − 1`.
///
/// **Mutação que deve sangrar:** `visuals::frame` a devolver `Frame::Classic` para todo tema.
#[test]
fn the_door_alone_removes_exactly_one_frame() {
    use ph2d_editor_core::widget::ButtonState;
    use ph2d_editor_core::widget::panel_chrome::paint_segmented_button;
    let paths = |theme: Theme| {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        paint_segmented_button(
            Rect::new(10.0, 10.0, 80.0, 24.0),
            "Butt",
            false,
            (ButtonState::Normal, 1.0),
            &mut scene,
            &mut text,
            theme,
        );
        scene.inner().encoding().n_paths
    };
    let classic = paths(Theme::Forge);
    let modern = paths(Theme::Dark);
    assert_eq!(classic, 2, "o segmento classico e' fundo + contorno");
    assert_eq!(
        modern,
        classic - 1,
        "o segmento moderno devia ser so' o fundo: {modern} caminhos contra {classic}"
    );
    // E o OLED — *Draw Extra Borders* — continua a traçar: é a excepção declarada, não um buraco.
    assert_eq!(
        paths(Theme::Oled),
        classic,
        "o OLED traca bordas extra, como no Godot"
    );
}

/// ⭐⭐ **A WAVE 3 fechou a dívida, e três dos vinte pintores convertidos provam-no ao PIXEL** —
/// cada um é *fundo + moldura* no clássico (2 caminhos; o texto vai por glifos) e *só fundo* num
/// tema moderno (1): o avatar (a moldura de um chip), a barra de estado vazia (a pílula que virou
/// rectângulo) e o menu de contexto vazio (um cartão flutuante).
///
/// ⛔ Existe pela mesma razão do gate acima: o censo do FONTE vê que o ficheiro *conhece* a porta,
/// não que a *atravessa* — um pintor que chame `stroke_frame` com `Feel::Error` em repouso passa
/// no censo e traça na mesma. Só a contagem exacta `classic − 1` o apanha.
///
/// **Mutação que deve sangrar:** trocar o `Feel::Rest` de qualquer um dos três por `Feel::Error`,
/// ou `visuals::frame` a devolver `Frame::Classic` para todo tema.
#[test]
fn the_wave_three_painters_lose_exactly_their_frame() {
    use ph2d_editor_core::widget::{
        Avatar, ContextMenu, StatusBar, paint_avatar, paint_context_menu, paint_status_bar,
    };
    fn rect() -> Rect {
        Rect::new(10.0, 10.0, 120.0, 32.0)
    }
    type Painter = fn(Theme) -> u32;
    let painters: [(&str, Painter); 3] = [
        ("avatar", |theme| {
            let mut scene = VectorScene::new();
            let mut text = TextSystem::without_system_fonts();
            let avatar = Avatar::new(NodeId(7), "Enio", 'E');
            paint_avatar(&avatar, rect(), &mut scene, &mut text, theme);
            scene.inner().encoding().n_paths
        }),
        ("status bar", |theme| {
            let mut scene = VectorScene::new();
            let mut text = TextSystem::without_system_fonts();
            let bar = StatusBar::new(NodeId(8), "Status", Vec::new());
            paint_status_bar(&bar, rect(), &mut scene, &mut text, theme);
            scene.inner().encoding().n_paths
        }),
        ("context menu", |theme| {
            let mut scene = VectorScene::new();
            let mut text = TextSystem::without_system_fonts();
            let menu = ContextMenu::new(NodeId(9), "Menu", Vec::new());
            paint_context_menu(&menu, rect(), &mut scene, &mut text, theme, 22.0);
            scene.inner().encoding().n_paths
        }),
    ];
    for (name, paths) in painters {
        let classic = paths(Theme::Forge);
        let modern = paths(Theme::Dark);
        assert_eq!(classic, 2, "{name}: o classico e' fundo + moldura");
        assert_eq!(
            modern,
            classic - 1,
            "{name}: o moderno devia ser so' o fundo — {modern} caminhos contra {classic}"
        );
        assert_eq!(
            paths(Theme::Oled),
            classic,
            "{name}: o OLED traca bordas extra, como no Godot"
        );
    }
}

/// ⛔ **O controlo: dois temas da MESMA família emitem a MESMA geometria** — o que muda entre o
/// `forge` e o `workshop` é cor, nunca contorno. Sem isto, uma contagem qualquer que desse
/// «menos» passaria por pele plana.
#[test]
fn two_themes_of_one_family_paint_the_same_geometry() {
    assert_eq!(
        paths_of_the_gallery(Theme::Forge),
        paths_of_the_gallery(Theme::Workshop)
    );
    assert_eq!(
        paths_of_the_gallery(Theme::Dark),
        paths_of_the_gallery(Theme::Gray)
    );
}
