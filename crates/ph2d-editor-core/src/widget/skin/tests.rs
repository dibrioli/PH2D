//! Gates da **pele por-widget** (plano UI/UX W6.2).
//!
//! ⚠️ O gate central deste arquivo é o de **BYTES**: a pele do canvas e o painel nativo têm de
//! emitir a mesma cena. Ele é o que impede a única falha silenciosa desta wave — alguém
//! "melhorar" a prévia redesenhando o widget à mão, e a divergência só aparecer numa screenshot.

use super::*;

// ⚠️ **Irmão, e não um `*_tests.rs` solto em `src/widget/`**: o `ph2d-widget-sync` varre os `*.rs`
// do TOPO daquela pasta para gerar o bloco de `mod`, e um arquivo solto ali viraria um "widget"
// sem showcase e sem a11y — a cicatriz do `command_palette`. Aqui dentro de `skin/` ele está fora
// da varredura, como este `tests.rs` já está.
#[path = "frame_tests.rs"]
mod frame;

#[path = "kind_tests.rs"]
mod kind;

// ⚠️ **O corte é de ASSUNTO, e as duas metades falham por motivos diferentes:** aqui mora *a pele
// é a que o painel nativo pinta* (o gate de bytes, o rótulo, o estado vivo, a moldura); ali, *o
// canal de parâmetro por-tipo chega ao braço certo e é inerte em todos os outros*. Uma cresce com
// o catálogo do design system, a outra com o número de campos do `SkinParam`.
#[path = "param_tests.rs"]
mod param;

#[path = "axis_tests.rs"]
mod axis;

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
        paint_widget_skin(
            kind,
            "Save",
            SkinParam::default(),
            rect(),
            &mut scene,
            &mut ts,
            Theme::Forge,
        );
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
    paint_widget_skin(
        WidgetKind::Button,
        "Save",
        SkinParam::default(),
        r,
        &mut b,
        &mut ts,
        Theme::Forge,
    );
    assert_eq!(print(&a), print(&b), "a pele do Button divergiu do pintor");

    let mut a = ph2d_vector::VectorScene::new();
    paint_toggle(&Toggle::new(NodeId(0), "On"), r, &mut a, Theme::Forge);
    let mut b = ph2d_vector::VectorScene::new();
    paint_widget_skin(
        WidgetKind::Toggle,
        "On",
        SkinParam::default(),
        r,
        &mut b,
        &mut ts,
        Theme::Forge,
    );
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
    paint_widget_skin(
        WidgetKind::Tag,
        "Beta",
        SkinParam::default(),
        r,
        &mut b,
        &mut ts,
        Theme::Forge,
    );
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
        SkinParam::default(),
        r,
        &mut forge,
        &mut ts,
        Theme::Forge,
    );
    let mut light = ph2d_vector::VectorScene::new();
    paint_widget_skin(
        WidgetKind::Button,
        "Save",
        SkinParam::default(),
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
    paint_widget_skin(
        WidgetKind::Button,
        "Save",
        SkinParam::default(),
        r,
        &mut a,
        &mut ts,
        Theme::Forge,
    );
    let mut b = ph2d_vector::VectorScene::new();
    paint_widget_skin(
        WidgetKind::Button,
        "Cancel",
        SkinParam::default(),
        r,
        &mut b,
        &mut ts,
        Theme::Forge,
    );

    let (ga, gb) = (print(&a).3, print(&b).3);
    assert!(!ga.is_empty(), "o rotulo nao produziu glifo nenhum");
    assert_ne!(ga, gb, "dois rotulos diferentes pintaram os MESMOS glifos");
}

/// **O ESTADO VIVO chega à pintura** (plano UI/UX W8b.2) — o mesmo tipo, dois valores, duas cenas.
///
/// ⚠️ Este é o gate da porta VIVA, e ele nasceu de uma mutação que sobreviveu: passar `None` em
/// vez do estado do store deixava **todos** os gates do painel autorado verdes — cada slider
/// pintaria a meio para sempre enquanto o `WidgetStore` carrega o valor real, e o único lugar
/// onde isso aparece é uma screenshot. A afirmação é sobre a CENA, e não sobre um argumento: uma
/// pele que aceitasse o estado e o ignorasse cai aqui do mesmo jeito.
#[test]
fn the_live_state_reaches_the_paint() {
    let mut ts = text();
    let mut low = ph2d_vector::VectorScene::new();
    let mut high = ph2d_vector::VectorScene::new();
    for (scene, value) in [(&mut low, 0.0_f32), (&mut high, 1.0_f32)] {
        let live = crate::interaction::InteractiveState::Slider {
            state: crate::widget::SliderState::Normal,
            value,
            orientation: crate::widget::SliderOrientation::Horizontal,
        };
        paint_widget_skin_with(
            WidgetKind::Slider,
            "Opacity",
            NodeId(7),
            // ⚠️ `SETTLED` reproduz o mundo pré-par ao BYTE — estes dois gates medem outra coisa.
            Some((&live, crate::motion::SETTLED)),
            SkinParam::default(),
            rect(),
            scene,
            &mut ts,
            Theme::Forge,
        );
    }
    assert_ne!(
        print(&low),
        print(&high),
        "o valor do store nao chega a' pintura — a pele viva desenha a previa estatica"
    );
}

/// **E a PRÉVIA é esta função sem estado**, ao byte.
///
/// ⚠️ A metade oposta, e ela é o que impede um segundo `match`: se `paint_widget_skin` deixasse de
/// delegar, as duas respostas a *"que aparência tem um Slider?"* divergiriam no único lugar onde
/// ninguém lê número — uma screenshot.
#[test]
fn the_preview_is_this_function_without_state() {
    let mut ts = text();
    for kind in WidgetKind::ALL {
        let mut a = ph2d_vector::VectorScene::new();
        paint_widget_skin(
            kind,
            "Save",
            SkinParam::default(),
            rect(),
            &mut a,
            &mut ts,
            Theme::Forge,
        );
        let mut b = ph2d_vector::VectorScene::new();
        paint_widget_skin_with(
            kind,
            "Save",
            PREVIEW_ID,
            None,
            SkinParam::default(),
            rect(),
            &mut b,
            &mut ts,
            Theme::Forge,
        );
        assert_eq!(
            print(&a),
            print(&b),
            "{kind:?}: a previa deixou de ser esta funcao sem estado"
        );
    }
}

/// **O que está sendo DIGITADO chega à pintura, e não o valor commitado.**
///
/// ⚠️ O `NumberInput` é o único tipo do catálogo cujo estado vivo tem **duas** grandezas para o
/// mesmo campo: o `value` (o número commitado) e o `buffer` (o texto em edição). Enquanto o
/// artista digita `12`, o valor ainda é o antigo — e uma pele que formatasse o `value` mostraria
/// o número velho **debaixo do cursor**, com o campo focado e o texto certo em lugar nenhum.
///
/// O oráculo isola o buffer de propósito: os dois estados carregam o **mesmo `value`**, então
/// qualquer diferença de bytes só pode ter vindo do texto em edição. Deixar de passar o `buffer`
/// (ou passar `None`) colapsa as duas impressões.
#[test]
fn the_number_input_paints_what_is_being_typed_not_the_committed_value() {
    let mut ts = text();
    let mut a = ph2d_vector::VectorScene::new();
    let mut b = ph2d_vector::VectorScene::new();
    for (scene, buffer) in [(&mut a, "7"), (&mut b, "1234")] {
        let live = crate::interaction::InteractiveState::NumberInput {
            state: crate::widget::TextInputState::Focused,
            value: 0.0,
            buffer: buffer.to_string(),
            caret: buffer.len(),
            last_committed: 0.0,
            selection_anchor: None,
        };
        paint_widget_skin_with(
            WidgetKind::NumberInput,
            "Radius",
            NodeId(11),
            // ⚠️ `SETTLED` reproduz o mundo pré-par ao BYTE — estes dois gates medem outra coisa.
            Some((&live, crate::motion::SETTLED)),
            SkinParam::default(),
            rect(),
            scene,
            &mut ts,
            Theme::Forge,
        );
    }
    assert_ne!(
        print(&a),
        print(&b),
        "o buffer em edicao nao chega a' pintura — o campo mostraria o valor commitado"
    );
}

/// **A prévia do medidor mostra o marcador de pico, e não uma barra lisa.**
///
/// ⚠️ Um `LevelMeter` tem TRÊS partes (preenchimento, marca de pico, folga) e a marca só se lê
/// **acima** do preenchimento. Um pico igual ao RMS a desenha dentro da barra, onde ela some — e
/// o artista, olhando a prévia, julgaria um widget de duas partes.
///
/// O oráculo compara a prévia contra um medidor cujo pico foi colapsado no RMS: se
/// [`PREVIEW_PEAK`] deixar de estar acima de [`PREVIEW_VALUE`], as duas impressões passam a ser a
/// mesma. É o irmão exato do argumento que o `PREVIEW_VALUE` faz para o slider.
#[test]
fn the_level_meter_preview_shows_its_peak_marker() {
    let mut ts = text();
    let mut preview = ph2d_vector::VectorScene::new();
    paint_widget_skin(
        WidgetKind::LevelMeter,
        "Master",
        SkinParam::default(),
        rect(),
        &mut preview,
        &mut ts,
        Theme::Forge,
    );

    let mut flat = ph2d_vector::VectorScene::new();
    let mut m = crate::widget::LevelMeter::new(NodeId(0), "Master");
    m.rms = [PREVIEW_VALUE; 2];
    m.peak_hold = [PREVIEW_VALUE; 2];
    crate::widget::paint_level_meter(&m, rect(), &mut flat, Theme::Forge);

    assert_ne!(
        print(&preview),
        print(&flat),
        "o pico da previa nao esta' acima do RMS — a marca desaparece dentro da barra"
    );
}
