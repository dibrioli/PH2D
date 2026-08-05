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

// ─────────────────────────────────────────────────────────────────────────────────────────────
// A MOLDURA É UMA CAIXA (BUGS_vector #26)
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// A altura natural de uma linha de painel — o único ponto onde a pele e o painel TÊM de
/// coincidir ao bit.
fn row_h() -> f32 {
    ph2d_tokens::ROW_H_PX
}

/// **A PELE PREENCHE A MOLDURA** — a lei da 2ª rodada do #26, afirmada onde ela é decidida.
///
/// ⚠️ O oráculo é **byte-a-byte contra o pintor a quem se pede a moldura inteira**: se a pele
/// pedir 64% (a 1ª rodada), 25% (o mundo antes dela) ou 2×, este gate fica vermelho. É a forma
/// mais estreita de dizer *"o gizmo abraça a tinta"* sem decodificar a geometria da cena.
///
/// ⚠️ E ele é o irmão de `the_skin_paints_exactly_what_the_native_painter_paints`, que faz a
/// mesma afirmação para os dez tipos que nunca precisaram do canal: **os doze passam a ter uma
/// lei só**.
#[test]
fn the_skin_asks_for_the_whole_frame() {
    let tall = Rect::new(10.0, 20.0, 400.0, 140.0);
    let mut ts = text();

    let mut want = ph2d_vector::VectorScene::new();
    let mut c = Checkbox::new(NodeId(0), "Grid");
    c.box_px = Some(tall.h.min(tall.w));
    paint_checkbox(&c, tall, &mut want, &mut ts, Theme::Forge);
    let mut got = ph2d_vector::VectorScene::new();
    paint_widget_skin(
        WidgetKind::Checkbox,
        "Grid",
        tall,
        &mut got,
        &mut ts,
        Theme::Forge,
    );
    assert_eq!(
        print(&want),
        print(&got),
        "a pele do Checkbox nao pediu a moldura inteira — sobra folga vertical, e o gizmo deixa \
         de abracar a tinta"
    );

    let mut want = ph2d_vector::VectorScene::new();
    let mut sl = Slider::new(NodeId(0), "Opacity");
    sl.value = PREVIEW_VALUE;
    sl.track_px = Some(tall.h);
    paint_slider(&sl, tall, &mut want, Theme::Forge);
    let mut got = ph2d_vector::VectorScene::new();
    paint_widget_skin(
        WidgetKind::Slider,
        "Opacity",
        tall,
        &mut got,
        &mut ts,
        Theme::Forge,
    );
    assert_eq!(
        print(&want),
        print(&got),
        "a pele do Slider nao pediu a moldura inteira"
    );

    // **O TOKEN passou a ser o tamanho NATURAL do objeto**: uma moldura da altura dele pinta
    // exactamente a tinta que o app pinta numa linha de painel.
    let natural = Rect::new(0.0, 0.0, 400.0, ph2d_tokens::CHECKBOX_BOX_PX);
    assert_eq!(
        skin_checkbox_box_px(natural),
        ph2d_tokens::CHECKBOX_BOX_PX,
        "uma moldura da altura do token deixou de pintar a caixa do token"
    );
}

/// **Uma caixa QUADRADA não transborda uma moldura estreita.**
///
/// ⚠️ Este gate nasceu de uma consequência da lei nova que a lei antiga não tinha: com a caixa
/// capada em 18 px ela quase nunca era mais larga que a moldura; pedindo a ALTURA, uma moldura
/// alta e estreita a faria derramar para fora do gizmo — o oposto exacto do que a 2ª rodada
/// pede.
#[test]
fn a_square_box_never_spills_out_of_a_narrow_frame() {
    let narrow = Rect::new(0.0, 0.0, 40.0, 300.0);
    assert_eq!(
        skin_checkbox_box_px(narrow),
        narrow.w,
        "a caixa pediu mais do que a moldura tem de largura"
    );
    assert!(
        narrow.w < narrow.h,
        "a fixture nao contem o fenomeno: a moldura nao e' mais estreita que alta"
    );
}

/// **O REPRO.** *"O checkbox sempre fica pequeno"* — a caixa media 18 px em TODA moldura, e como
/// a pele é pintada em px de TELA isso significa que dar zoom crescia o retângulo e não crescia o
/// widget.
///
/// ⚠️ **O oráculo é a TINTA numa moldura FIXA**, e essa fixação é o que o torna um oráculo: com
/// duas molduras diferentes toda cena difere de qualquer maneira (o rótulo é centrado, a caixa é
/// centrada) e a comparação não diria nada. Aqui as duas rotas recebem o MESMO retângulo alto, e
/// sob a lei antiga elas eram byte-IDÊNTICAS — a pele não acrescentava nada ao pintor. É essa
/// igualdade que o gate recusa.
///
/// ⚠️ E a primeira versão deste gate lia o `x` do primeiro glifo, que é **relativo à run**: media
/// `0` nas duas molduras e teria reprovado o produto correto.
#[test]
fn the_checkbox_box_grows_with_the_frame() {
    let tall = Rect::new(0.0, 0.0, 400.0, row_h() * 4.0);
    let mut ts = text();

    let mut token_sized = ph2d_vector::VectorScene::new();
    paint_checkbox(
        &Checkbox::new(NodeId(0), "Snap"),
        tall,
        &mut token_sized,
        &mut ts,
        Theme::Forge,
    );
    let mut grown = ph2d_vector::VectorScene::new();
    paint_widget_skin(
        WidgetKind::Checkbox,
        "Snap",
        tall,
        &mut grown,
        &mut ts,
        Theme::Forge,
    );
    assert_ne!(
        print(&token_sized),
        print(&grown),
        "a pele do Checkbox saiu identica a' do token numa moldura 4x a linha — o canvas herdou \
         a politica de linha do painel, e dar zoom nao cresce o widget"
    );

    // …e o que ela pede é a moldura, não uma fração dela.
    assert_eq!(skin_checkbox_box_px(tall), tall.h);
}

/// **O irmão do slider.** *"O Slider tem sempre altura fixa"* — a trilha pinava no teto de linha
/// (8 px) a partir de 32 px de moldura.
///
/// ⚠️ Oráculo de TINTA outra vez: o slider não pinta rótulo, então a comparação é entre a cena
/// que a espessura nova produz e a que o teto produzia. Sob a lei antiga as duas eram iguais ao
/// bit numa moldura alta.
#[test]
fn the_slider_track_grows_with_the_frame() {
    let tall = Rect::new(0.0, 0.0, 400.0, row_h() * 4.0);
    let mut ts = text();

    let mut capped = ph2d_vector::VectorScene::new();
    let mut s = Slider::new(NodeId(0), "Opacity");
    s.value = PREVIEW_VALUE; // sem `track_px`: a politica de LINHA, com o teto
    paint_slider(&s, tall, &mut capped, Theme::Forge);

    let mut grown = ph2d_vector::VectorScene::new();
    paint_widget_skin(
        WidgetKind::Slider,
        "Opacity",
        tall,
        &mut grown,
        &mut ts,
        Theme::Forge,
    );

    assert_ne!(
        print(&capped),
        print(&grown),
        "a trilha da pele saiu identica a' do teto de linha — o canvas herdou a politica do painel"
    );
    assert_eq!(skin_slider_track_px(tall), tall.h);
}

/// **A moldura continua a ser o TETO** — uma caixa nunca transborda o que a contém.
///
/// ⚠️ Este gate mede o PINTOR, não a pele, e a razão é medida: a razão da pele é `18/28 ≈ 0,64`,
/// sempre menor que 1, então **o valor que a pele pede nunca morde o `.min`**. Um gate que o
/// exercitasse pela pele estaria a afirmar um `.min` que nunca dispara — verde por vácuo. Quem
/// pode tropeçar nele é o próximo chamador do canal.
#[test]
fn the_frame_still_caps_the_box() {
    let squat = Rect::new(0.0, 0.0, 400.0, 6.0);
    let mut ts = text();

    let mut asked_huge = ph2d_vector::VectorScene::new();
    let mut c = Checkbox::new(NodeId(0), "Snap");
    c.box_px = Some(1000.0);
    paint_checkbox(&c, squat, &mut asked_huge, &mut ts, Theme::Forge);

    let mut asked_exactly_the_frame = ph2d_vector::VectorScene::new();
    c.box_px = Some(squat.h);
    paint_checkbox(
        &c,
        squat,
        &mut asked_exactly_the_frame,
        &mut ts,
        Theme::Forge,
    );

    assert_eq!(
        print(&asked_huge),
        print(&asked_exactly_the_frame),
        "uma caixa de 1000 px numa moldura de 6 nao foi limitada pela moldura"
    );
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
            Some(&live),
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
        paint_widget_skin(kind, "Save", rect(), &mut a, &mut ts, Theme::Forge);
        let mut b = ph2d_vector::VectorScene::new();
        paint_widget_skin_with(
            kind,
            "Save",
            PREVIEW_ID,
            None,
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
