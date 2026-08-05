//! Gates da **pele por-widget** (plano UI/UX W6.2).
//!
//! ⚠️ O gate central deste arquivo é o de **BYTES**: a pele do canvas e o painel nativo têm de
//! emitir a mesma cena. Ele é o que impede a única falha silenciosa desta wave — alguém
//! "melhorar" a prévia redesenhando o widget à mão, e a divergência só aparecer numa screenshot.

use super::*;

/// A impressão digital de uma cena: os caminhos, os bytes de geometria e de tinta, **e os
/// glifos**.
///
/// ⚠️ `n_paths` sozinho não serve de oráculo — dois desenhos completamente diferentes podem ter a
/// mesma contagem. E os `Vec<u32>` sozinhos também não: **texto não vira caminho**, ele vira
/// `glyph_run`, então uma pele que pintasse o rótulo ERRADO passaria por eles sem tocar num byte.
/// Os glifos entram pelo `(id, x, y)` — a identidade e o lugar de cada um.
type Print = (u32, Vec<u32>, Vec<u32>, Vec<(u32, u32, u32)>);

fn print(scene: &ph2d_vector::VectorScene) -> Print {
    let e = scene.inner().encoding();
    let glyphs = e
        .resources
        .glyphs
        .iter()
        .map(|g| (g.id, g.x.to_bits(), g.y.to_bits()))
        .collect();
    (e.n_paths, e.path_data.clone(), e.draw_data.clone(), glyphs)
}

/// **Esta cena desenhou alguma coisa?** Caminhos **ou** glifos.
///
/// ⚠️ Escrito depois de o gate `every_kind_paints_something` nascer VERMELHO sobre produto
/// CORRETO: um `ListItem` em repouso e não-selecionado não tem preenchimento nenhum — ele é o
/// rótulo e mais nada —, e `n_paths` para ele é legitimamente **zero**. O oráculo dizia *"emitiu
/// caminho"* enquanto a asserção dizia *"pintou"*, e as duas frases só coincidem para widgets com
/// fundo.
fn drew_anything(scene: &ph2d_vector::VectorScene) -> bool {
    let e = scene.inner().encoding();
    e.n_paths > 0 || !e.resources.glyph_runs.is_empty()
}

fn rect() -> Rect {
    Rect::new(10.0, 20.0, 160.0, 36.0)
}

fn text() -> TextSystem {
    TextSystem::without_system_fonts()
}

/// **Todo tipo PINTA alguma coisa.**
///
/// ⚠️ O modo de falha que este gate existe para pegar: um braço do `match` que não termina num
/// `paint_*` (esquecido, ou apagado num refactor) faz a forma **desaparecer** do canvas — o
/// desenho foi substituído por nada, e nada na tela diz porquê.
#[test]
fn every_kind_paints_something() {
    for kind in WidgetKind::ALL {
        let mut scene = ph2d_vector::VectorScene::new();
        let mut ts = text();
        paint_widget_skin(kind, "Save", rect(), &mut scene, &mut ts, Theme::Forge);
        assert!(
            drew_anything(&scene),
            "{kind:?} nao emitiu nem caminho nem glifo — a forma sumiria do canvas"
        );
    }
}

/// **A pele emite EXACTAMENTE o que o pintor nativo emite** — o gate de bytes.
///
/// Ele percorre as duas rotas com a MESMA entrada (mesmo retângulo, mesmo rótulo, mesmo tema,
/// mesmo `TextSystem`) e compara a cena inteira. Uma prévia com desenho próprio o quebra na
/// primeira divergência, por mais sutil que ela seja.
#[test]
fn the_skin_paints_exactly_what_the_native_painter_paints() {
    let r = rect();

    let mut a = ph2d_vector::VectorScene::new();
    let mut ts = text();
    paint_button(
        &Button::new(NodeId(0), "Save"),
        r,
        &mut a,
        &mut ts,
        Theme::Forge,
    );
    let mut b = ph2d_vector::VectorScene::new();
    paint_widget_skin(WidgetKind::Button, "Save", r, &mut b, &mut ts, Theme::Forge);
    assert_eq!(print(&a), print(&b), "a pele do Button divergiu do pintor");

    let mut a = ph2d_vector::VectorScene::new();
    paint_toggle(&Toggle::new(NodeId(0), "On"), r, &mut a, Theme::Forge);
    let mut b = ph2d_vector::VectorScene::new();
    paint_widget_skin(WidgetKind::Toggle, "On", r, &mut b, &mut ts, Theme::Forge);
    assert_eq!(print(&a), print(&b), "a pele do Toggle divergiu do pintor");

    let mut a = ph2d_vector::VectorScene::new();
    paint_tag(
        &Tag::new(NodeId(0), "Beta"),
        r,
        &mut a,
        &mut ts,
        Theme::Forge,
    );
    let mut b = ph2d_vector::VectorScene::new();
    paint_widget_skin(WidgetKind::Tag, "Beta", r, &mut b, &mut ts, Theme::Forge);
    assert_eq!(print(&a), print(&b), "a pele do Tag divergiu do pintor");
}

/// **Trocar um token move os DOIS lados na mesma direção** (o 3º gate que o plano pede).
///
/// ⚠️ Ele não afirma uma cor: afirma que a pele **responde ao tema** — se a pele tivesse cor
/// própria, o mesmo desenho sairia dos dois temas e este gate ficaria vermelho. É a metade que
/// prova que a ponte token→widget atravessa o canvas.
#[test]
fn a_token_change_moves_the_canvas_too() {
    let r = rect();
    let mut ts = text();
    let mut forge = ph2d_vector::VectorScene::new();
    paint_widget_skin(
        WidgetKind::Button,
        "Save",
        r,
        &mut forge,
        &mut ts,
        Theme::Forge,
    );
    let mut light = ph2d_vector::VectorScene::new();
    paint_widget_skin(
        WidgetKind::Button,
        "Save",
        r,
        &mut light,
        &mut ts,
        Theme::Sunstone,
    );
    assert_ne!(
        print(&forge).2,
        print(&light).2,
        "a pele pintou a MESMA tinta nos dois temas — ela nao esta' a ler os tokens"
    );
}

/// **O RÓTULO chega à tinta** — a metade que o `n_paths` é cego a ver.
///
/// ⚠️ O rótulo é o `Name` da entidade, e ele atravessa quatro camadas até aqui. Se alguma delas o
/// perder, o widget pinta a moldura certa com o texto errado (ou vazio) — e **todos os outros
/// gates deste arquivo ficam verdes**, porque a geometria não muda um byte.
#[test]
fn the_label_reaches_the_paint() {
    let r = rect();
    let mut ts = text();
    let mut a = ph2d_vector::VectorScene::new();
    paint_widget_skin(WidgetKind::Button, "Save", r, &mut a, &mut ts, Theme::Forge);
    let mut b = ph2d_vector::VectorScene::new();
    paint_widget_skin(
        WidgetKind::Button,
        "Cancel",
        r,
        &mut b,
        &mut ts,
        Theme::Forge,
    );

    let (ga, gb) = (print(&a).3, print(&b).3);
    assert!(!ga.is_empty(), "o rotulo nao produziu glifo nenhum");
    assert_ne!(ga, gb, "dois rotulos diferentes pintaram os MESMOS glifos");
}

/// **Os códigos são literais PINADOS.** Reordenar o enum não pode mover um número que já viaja
/// em arquivos salvos.
#[test]
fn the_codes_are_pinned_and_unique() {
    assert_eq!(WidgetKind::Button.code(), 1);
    assert_eq!(WidgetKind::Toggle.code(), 2);
    assert_eq!(WidgetKind::Checkbox.code(), 3);
    assert_eq!(WidgetKind::Slider.code(), 4);
    assert_eq!(WidgetKind::ProgressBar.code(), 5);
    assert_eq!(WidgetKind::Tag.code(), 6);
    assert_eq!(WidgetKind::TextInput.code(), 7);
    assert_eq!(WidgetKind::Card.code(), 8);
    assert_eq!(WidgetKind::SectionHeader.code(), 9);
    assert_eq!(WidgetKind::ListItem.code(), 10);
    assert_eq!(WidgetKind::Spinner.code(), 11);
    assert_eq!(WidgetKind::Divider.code(), 12);

    let mut seen = std::collections::BTreeSet::new();
    for kind in WidgetKind::ALL {
        assert!(seen.insert(kind.code()), "codigo repetido em {kind:?}");
    }
    assert_eq!(seen.len(), WidgetKind::ALL.len(), "a lista perdeu um tipo");
}

/// **Ida e volta é total**, e um código desconhecido devolve `None`.
///
/// ⚠️ O `None` é a metade que o plano pede como gate (*"um `kind` desconhecido degrada para o
/// desenho, nunca para um painel vazio"*): ele é o que um documento autorado por um build mais
/// novo produz, e recusá-lo seria recusar o arquivo.
#[test]
fn the_round_trip_is_total_and_the_unknown_degrades() {
    for kind in WidgetKind::ALL {
        assert_eq!(WidgetKind::from_code(kind.code()), Some(kind));
    }
    assert_eq!(WidgetKind::from_code(0), None, "0 nao e' tipo nenhum");
    assert_eq!(WidgetKind::from_code(9999), None, "um tipo do futuro");
}

/// **Cada tipo tem chave i18n PRÓPRIA** — duas iguais fariam dois chips com o mesmo nome, e o
/// artista não teria como distinguir o que está a escolher.
#[test]
fn every_kind_has_its_own_i18n_key() {
    let mut seen = std::collections::BTreeSet::new();
    for kind in WidgetKind::ALL {
        let key = kind.i18n_key();
        assert!(
            key.starts_with("panel.vector.widget.kind."),
            "{kind:?} tem chave fora da familia: {key}"
        );
        assert!(seen.insert(key), "chave repetida: {key}");
        assert_ne!(
            ph2d_i18n::tr(key),
            key,
            "a chave {key} nao esta' na tabela de i18n — o chip mostraria a chave crua"
        );
    }
}
