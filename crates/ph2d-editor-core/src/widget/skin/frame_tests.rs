//! **QUANTO DA MOLDURA a pele ocupa** — o cluster do BUGS_vector #26.
//!
//! ⚠️ Irmão de [`super`], e o corte entre os dois é de ASSUNTO: ali afirma-se *o que a pele
//! pinta* (o contrato dos códigos, a paridade de bytes com o pintor nativo, o rótulo, o estado
//! vivo); aqui, *onde ela pinta* — que a tinta preenche o retângulo autorado em vez do padding
//! de uma linha de painel.
//!
//! Eles moram em arquivos separados porque o pai cruzou o teto de 500 LOC dos primitivos de
//! widget quando o catálogo ganhou o `NumberInput` e o `LevelMeter`, e porque esta metade tem
//! helper próprio (`row_h`) que a outra nunca usa.

use super::*;

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
