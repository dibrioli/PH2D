//! Os gates do `Style: Solid` no PRODUTO ([`super::solid_deposit`]) — o motor tem os seus em
//! `ph2d_painter_brush::solid`; aqui pergunta-se o que o **gesto** faz.

use super::measure_shape_system::{cp, tool};
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::Tool;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};

/// Quantos texels do canvas deixaram de ser o branco de fundo.
fn inked(t: &crate::tool::PainterTool) -> usize {
    t.canvas_rgba.chunks_exact(4).filter(|p| p[0] < 250).count()
}

/// Desenha um laço quadrado de lado `s` centrado em `c`, e devolve os texels entintados.
fn loop_gesture(t: &mut crate::tool::PainterTool, c: f32, s: f32) -> usize {
    let pts = [
        [c - s, c - s],
        [c + s, c - s],
        [c + s, c + s],
        [c - s, c + s],
        [c - s, c - s],
    ];
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
    // ⚠️ Vários passos por aresta, e por `windows(2)`: o primeiro ponto REPETE no fim (o laço fecha),
    // então procurar o índice de um ponto na lista devolve o do começo — a fixture nasceu com esse
    // defeito e o gate estourou nele antes de medir coisa nenhuma.
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        for k in 1..=8 {
            #[allow(clippy::cast_precision_loss)]
            let f = k as f32 / 8.0;
            t.on_canvas_pointer(cp(
                [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f],
                PointerPhase::Move,
            ));
        }
    }
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Up));
    inked(t)
}

/// **UM LAÇO FINO E SOLTO PINTA UMA MANCHA GORDA** — o `Style` do Alchemy, e o pedido do Enio
/// (*"para forma sólida a espessura da linha passa a não ser considerada"*) na sua forma medível.
///
/// ⚠️ O oráculo é a RAZÃO contra o mesmo gesto em Line, não um número absoluto: o que a feature
/// promete é que a tinta deixa de ser o rastro do pincel e passa a ser a área cercada.
#[test]
fn a_thin_loop_in_solid_paints_the_region_it_encircles() {
    let side = 256u32;
    let mut line = tool(side, PaintMedia::Digital, 4.0);
    let as_line = loop_gesture(&mut line, 128.0, 60.0);

    let mut solid = tool(side, PaintMedia::Digital, 4.0);
    solid.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
    let as_solid = loop_gesture(&mut solid, 128.0, 60.0);

    // A área cercada é 120×120 = 14 400; o rastro de um pincel de raio 4 sobre o perímetro é ~4 000.
    assert!(
        as_solid > as_line * 2,
        "Solid nao encheu a regiao: {as_solid} texels contra {as_line} do Line"
    );
    assert!(
        as_solid > 13_000,
        "a mancha deveria cobrir a area cercada (~14 400): {as_solid}"
    );
}

/// **A ESPESSURA ENTRA — e a região continua cheia** (W7, ordem do Enio 2026-08-15: *"Solid deve
/// usar o pincel com o falloff e espessura do traço como no modo flip"*).
///
/// ⚠️ **Este gate SUBSTITUI o que afirmava o contrário** (`in_solid_the_brush_width_does_not_enter…`,
/// a §1.1 do plano 38). A decisão é do mesmo autor e o modelo novo é o do Flip: uma figura é o
/// preenchimento **mais** o traço que a cerca. As DUAS metades são load-bearing e nenhuma basta
/// sozinha — só a primeira passaria num produto que voltou a pintar só o contorno; só a segunda,
/// num que voltou a esconder o pincel.
#[test]
fn in_solid_the_brush_paints_the_rim_and_the_region_stays_filled() {
    let side = 256u32;
    let mut a = tool(side, PaintMedia::Digital, 3.0);
    a.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
    let thin = loop_gesture(&mut a, 128.0, 60.0);

    let mut b = tool(side, PaintMedia::Digital, 24.0);
    b.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
    let thick = loop_gesture(&mut b, 128.0, 60.0);
    assert!(
        thick > thin + 5_000,
        "o pincel nao entrou na forma solida ({thin} texels com raio 3 contra {thick} com raio 24)"
    );

    // …e a REGIÃO continua cheia: a mancha de 120×120 (~14 400) tem de estar lá com o pincel fino.
    assert!(
        thin > 13_000,
        "a mancha deixou de cobrir a area cercada (~14 400): {thin}"
    );
}

/// **UM GESTO QUE NÃO CERCA NADA AINDA DEIXA O PINCEL** — o gate que isola a metade nova, e o que
/// prova que a mancha não come o traço.
///
/// ⚠️ **A DIAGONAL é a fixture, e ela foi escolhida por conter DOIS fenômenos que uma reta não
/// contém.** (1) Ela fecha numa *sliver* de área ~zero (§5.3 do plano), então tudo o que sobra na
/// tela é o traço — se ele voltar a ser suprimido, a tela fica **vazia**. (2) A caixa do
/// preenchimento é o QUADRADO inteiro que ela atravessa, e o traço corre pelo **meio** dela: é ali
/// que um snapshot velho apaga tinta, e uma reta horizontal (caixa de 1 px de altura) deixaria o
/// traço todo do lado de fora e passaria nos dois casos.
///
/// **Mutações que sangram:** reinstalar a supressão de dabs em `stamp_dabs`; e tirar o
/// `peel_drag_preview()` do bracket (o restore do quadro seguinte desfaz o lote que acabou de cair
/// dentro da caixa).
#[test]
fn in_solid_a_diagonal_gesture_still_lays_the_brush() {
    let side = 256u32;
    let stroke = |solid: bool| {
        let mut t = tool(side, PaintMedia::Digital, 6.0);
        if solid {
            t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
        }
        t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Down));
        for k in 1..=16 {
            #[allow(clippy::cast_precision_loss)]
            let d = 40.0 + (k as f32 / 16.0) * 176.0;
            t.on_canvas_pointer(cp([d, d], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([216.0, 216.0], PointerPhase::Up));
        inked(&t)
    };
    let as_line = stroke(false);
    let as_solid = stroke(true);
    assert!(as_line > 800, "controle: a diagonal tem de pintar");
    assert!(
        as_solid * 10 >= as_line * 9,
        "o traço sumiu sob o Solid: {as_solid} texels contra {as_line} sem ele"
    );
}

/// **DESMARCADO É BYTE-IDÊNTICO** — a rede que torna toda esta wave reversível.
#[test]
fn the_unticked_checkbox_leaves_the_canvas_byte_identical() {
    let side = 128u32;
    let mut a = tool(side, PaintMedia::Digital, 6.0);
    let _ = loop_gesture(&mut a, 64.0, 30.0);
    let before: Vec<u8> = a.canvas_rgba.to_vec();

    // Ligar e DESLIGAR devolve o estado — o toggle não pode deixar resíduo.
    let mut b = tool(side, PaintMedia::Digital, 6.0);
    b.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
    b.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
    let _ = loop_gesture(&mut b, 64.0, 30.0);
    assert_eq!(
        before, *b.canvas_rgba,
        "o traço mudou com o checkbox desligado"
    );
}

/// Desenha uma ELIPSE pelo shape editor (Down no centro, arrasta o raio, solta) e devolve os texels
/// entintados. ⚠️ O editor fica ABERTO depois disto — é assim que um shape editor funciona, e é
/// nesse estado que a mancha do Solid tem de estar na tela.
fn ellipse_gesture(t: &mut crate::tool::PainterTool, c: f32, r: f32) -> usize {
    t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
    t.on_canvas_pointer(cp([c, c], PointerPhase::Down));
    t.on_canvas_pointer(cp([c + r, c], PointerPhase::Move));
    t.on_canvas_pointer(cp([c + r, c], PointerPhase::Up));
    inked(t)
}

/// **UMA ELIPSE EM SOLID É UM DISCO, NÃO UM ANEL** — o `Style` alcança a família dos shape editors
/// (plano 38 §5.1, decisão do Enio: *"para todos que forem possíveis"*).
///
/// ⚠️ O oráculo é a ÁREA CERCADA contra o rastro do contorno: um anel de raio 70 e pincel 4 tem
/// ~1.800 texels, o disco tem ~15.400. A razão é o que a feature promete.
///
/// ⚠️ **O teto subiu com a W7 e o motivo está no número:** o disco agora veste o traço, então o raio
/// visível é `70 + raio do pincel`, e `π·74² ≈ 17 200`. O piso ficou onde estava — a mancha não
/// encolheu, o contorno é que passou a somar.
#[test]
fn an_ellipse_shape_in_solid_is_a_disc_not_a_ring() {
    let side = 256u32;
    let mut line = tool(side, PaintMedia::Digital, 4.0);
    let as_ring = ellipse_gesture(&mut line, 128.0, 70.0);

    let mut solid = tool(side, PaintMedia::Digital, 4.0);
    solid.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
    let as_disc = ellipse_gesture(&mut solid, 128.0, 70.0);

    assert!(
        as_disc > as_ring * 3,
        "a elipse nao foi preenchida: {as_disc} texels contra {as_ring} do contorno"
    );
    // pi*70*70 = 15 394 de mancha, mais o aro do pincel (pi*74*74 = 17 203).
    assert!(
        (14_000..=17_600).contains(&as_disc),
        "a area preenchida nao e a do disco vestindo o pincel (~17 200): {as_disc}"
    );
}

/// **UMA FORMA SÓLIDA VESTE O PINCEL TAMBÉM** — o irmão do gate da mão livre, na família dos shape
/// editors, e a metade que prova que a wave alcançou os dois caminhos de depósito (a transação de
/// re-carimbo é OUTRA — ver `super::solid_deposit`).
///
/// **Mutação que sangra:** devolver o `return` do `restamp_shapes_preview` sob Solid (os dois raios
/// voltam a pintar o mesmo número).
#[test]
fn in_solid_a_shape_wears_the_brush_too() {
    let side = 256u32;
    let mut a = tool(side, PaintMedia::Digital, 3.0);
    a.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
    let thin = ellipse_gesture(&mut a, 128.0, 70.0);

    let mut b = tool(side, PaintMedia::Digital, 24.0);
    b.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
    let thick = ellipse_gesture(&mut b, 128.0, 70.0);
    assert!(
        thick > thin + 3_000,
        "o pincel nao entrou na forma solida ({thin} com raio 3 contra {thick} com raio 24)"
    );
    // …e a mancha continua lá com o pincel fino (o disco de raio 70 mede ~15 394).
    assert!(
        thin > 14_000,
        "a forma deixou de ser preenchida: {thin} texels"
    );
}

/// **UM `Remove` ABRE UM BURACO** — a Operação booleana entra pelo SENTIDO do laço, e o `nonzero` do
/// preenchimento faz o resto (`super::solid_shapes`). Sem a orientação por op, a forma de Remove
/// seria pintada como mais uma mancha e o buraco não existiria.
///
/// ⚠️ **A fixture monta as duas formas como estado parqueado** — o padrão que os gates booleanos
/// desta crate já usam (`two_overlapping_add_ellipses_union_their_outline`), e não por dois gestos:
/// um Down dentro de uma figura que já existe **REATIVA** aquela figura em vez de começar outra
/// (`super::stroke_router`), então a 1ª versão desta fixture media UMA forma achando que media duas
/// — e passava a impressão de estar exercitando o winding sem nunca o exercitar.
#[test]
fn a_remove_shape_punches_a_hole_in_the_solid() {
    let side = 256u32;
    let mut t = tool(side, PaintMedia::Digital, 4.0);
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
    let disc = |c: f32, r: f32, op| super::stroke_multi::StrokeShape {
        state: crate::undo::ShapeEditState::Ellipse(crate::undo::EllipseState {
            center: [c, c],
            u: [1.0, 0.0],
            rx: r,
            ry: r,
            editing: true,
            seed: 1,
        }),
        op,
    };
    t.paint
        .parked_shapes
        .push(disc(128.0, 70.0, super::stroke_multi::StrokeOp::Add));
    t.paint
        .parked_shapes
        .push(disc(128.0, 30.0, super::stroke_multi::StrokeOp::Remove));
    t.restamp_shapes_preview(&[]);

    let px = |x: usize, y: usize| t.canvas_rgba[(y * side as usize + x) * 4];
    assert!(px(128, 128) >= 250, "o Remove nao abriu buraco no centro");
    assert!(px(128, 78) < 250, "a coroa entre os dois raios ficou vazia");
}

/// **TODO TIPO CONTINUA DONO DO GESTO SOB SOLID** (W7, ordem do Enio: *"todos os types devem ser
/// compatíveis com solid"*).
///
/// ⚠️ **A pergunta é feita ao ENUM, não a uma lista escrita aqui** (`sews_threads` é a porta que
/// sabe quem costura), então um tipo novo entra neste gate sozinho — e a cláusula que o Solid
/// mantinha (`!solid_owns_the_gesture()`) fica impossível de reinstalar em silêncio.
///
/// **Mutação que sangra:** devolver `&& !self.solid_owns_the_gesture()` ao
/// `threads_own_the_gesture`.
#[test]
fn every_sewing_type_still_owns_its_gesture_under_solid() {
    use ph2d_painter_brush::line_kind::LineKind;
    for kind in [
        LineKind::None,
        LineKind::Speed,
        LineKind::Sketchy,
        LineKind::Wire,
        LineKind::Ribbon,
        LineKind::Rough,
    ] {
        let mut t = tool(128, PaintMedia::Digital, 6.0);
        t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
        t.paint.brush.line_kind = kind;
        if !t.paint.brush.sews_threads() {
            continue; // este tipo não costura fio nenhum — nada a exigir
        }
        assert!(
            t.threads_own_the_gesture(),
            "{kind:?} deixou de costurar sob o Style: Solid"
        );
    }
}

/// **A SIMETRIA ALCANÇA A MANCHA** — desenhar um laço de um lado do eixo tem de pintar o outro.
///
/// ⚠️ **O oráculo é o TEXEL espelhado**, não uma contagem: uma contagem maior passaria num produto
/// que apenas engordou a mancha, e o que a feature promete é que a cópia cai no lugar refletido.
///
/// **Mutação que sangra:** o `solid_fill_loops` devolver os laços sem passar pelo `symmetric_loops`.
#[test]
fn the_solid_fill_is_mirrored_by_symmetry() {
    let side = 256u32;
    let mut t = tool(side, PaintMedia::Digital, 4.0);
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
    t.toggle_symmetry_enabled(); // eixo X pelo centro do canvas (128), o default do bind
    let _ = loop_gesture(&mut t, 70.0, 30.0); // um quadrado 40..100, bem à esquerda do eixo

    let px = |x: usize, y: usize| t.canvas_rgba[(y * side as usize + x) * 4];
    assert!(px(70, 70) < 250, "a mancha original nao foi pintada");
    // O espelho de x=70 em torno de 128 é 186.
    assert!(
        px(186, 70) < 250,
        "a mancha nao foi espelhada: o texel em (186,70) ficou branco"
    );
}

/// **O TILING ALCANÇA A MANCHA** — um laço que cruza a costura pinta o outro lado da tile.
///
/// **Mutação que sangra:** o `solid_fill_loops` pular o `tiled_loops`.
#[test]
fn the_solid_fill_wraps_across_the_tiled_seam() {
    let side = 128u32;
    let mut t = tool(side, PaintMedia::Digital, 3.0);
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
    t.toggle_brush_tiling(0); // só o eixo X
    // Um quadrado centrado em x=118: metade dele cai FORA da borda direita (128).
    let _ = loop_gesture(&mut t, 118.0, 20.0);

    let px = |x: usize, y: usize| t.canvas_rgba[(y * side as usize + x) * 4];
    assert!(px(120, 118) < 250, "a mancha original nao foi pintada");
    // O que passou de 128 reaparece em x−128: o quadrado vai até 138 ⇒ 10.
    assert!(
        px(5, 118) < 250,
        "a mancha nao envolveu a costura: o texel em (5,118) ficou branco"
    );
}

/// **O CHECKBOX ALCANÇA A FERRAMENTA** — o clique do painel flipa o estado publicado, nos dois
/// sentidos, e ele chega aos três slots de relevo (Deposit / Faca / Sculpt são UM assunto).
#[test]
fn the_panel_click_reaches_the_tool_and_every_relief_slot() {
    let mut t = tool(64, PaintMedia::Digital, 8.0);
    assert!(!t.brush_settings().style_solid, "o default tem de ser Line");
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
    assert!(
        t.brush_settings().style_solid,
        "o clique nao chegou ao tool"
    );
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
    assert!(!t.brush_settings().style_solid, "o clique nao volta");
}

/// O mesmo laço, com um **TIQUE** entre eventos — sem eles o gesto não tem velocidade e o arremesso
/// do `Speed` nasce **estruturalmente inerte** (é a razão de ele não existir num shape editor: cada
/// forma constrói uma `Stroke` fresca, e a mão nunca correu por ela).
fn loop_gesture_ticked(t: &mut crate::tool::PainterTool, c: f32, s: f32, steps: usize) {
    let pts = [
        [c - s, c - s],
        [c + s, c - s],
        [c + s, c + s],
        [c - s, c + s],
        [c - s, c - s],
    ];
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        for k in 1..=steps {
            #[allow(clippy::cast_precision_loss)]
            let f = k as f32 / steps as f32;
            t.on_canvas_pointer(cp(
                [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f],
                PointerPhase::Move,
            ));
            <crate::tool::PainterTool as Tool>::on_tick(t, 16.0);
        }
    }
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Up));
}

/// Os texels entintados, como máscara.
fn ink_mask(t: &crate::tool::PainterTool) -> Vec<bool> {
    t.canvas_rgba.chunks_exact(4).map(|p| p[0] < 250).collect()
}

/// **A MANCHA SEGUE A TINTA, NÃO O PONTEIRO** (W8; report do Enio 2026-08-15, com a foto: o contorno
/// do Speed desenhado FORA de uma mancha menor).
///
/// Metade dos tipos de linha existe justamente para pôr a tinta longe da mão — o `Speed` a arremessa
/// `v · T` à frente. Enquanto a mancha era o polígono de `ev.pos`, ela e o traço descreviam curvas
/// diferentes, e o artista via as duas.
///
/// ⚠️ **O oráculo cruza DUAS fontes independentes:** onde a tinta caiu vem dos PIXELS de um gesto
/// igual com a mancha desligada (o produto, sem modelo nenhum), e a região preenchida vem da porta
/// que a produz. A pergunta é *a mancha cobre a tinta?* — que é exactamente a foto do report ao
/// contrário.
///
/// ⚠️ **E a fração é comparada com o CONTROLE, nunca com um número escolhido:** um traço tem RAIO, e
/// metade da tinta de qualquer gesto cai por fora da fronteira que ela desenha — mesmo quando as duas
/// coincidem perfeitamente. O que separa *"a mancha segue a tinta"* de *"a mancha ficou noutro
/// sítio"* é a fração do `Speed` medida contra a do gesto sem efeito, onde tinta e ponteiro são a
/// mesma curva por construção.
///
/// **Mutação que sangra:** alimentar `solid_path` com `ev.pos` outra vez.
#[test]
fn the_solid_fill_follows_the_ink_not_the_pointer() {
    use ph2d_painter_brush::line_kind::LineKind;
    const N: u32 = 192;
    let n = N as usize;
    // Que fração da tinta deste tipo cai DENTRO da mancha que ele preenche.
    let covered_fraction = |kind: LineKind| -> f32 {
        // A tinta, medida nos pixels de um gesto com a mancha DESLIGADA.
        let mut a = tool(N, PaintMedia::Digital, 5.0);
        a.paint.brush.line_kind = kind;
        loop_gesture_ticked(&mut a, 96.0, 34.0, 6);
        let ink = ink_mask(&a);
        // A região que o MESMO gesto preenche, perguntada à porta que a produz.
        let mut b = tool(N, PaintMedia::Digital, 5.0);
        b.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
        b.paint.brush.line_kind = kind;
        loop_gesture_ticked(&mut b, 96.0, 34.0, 6);
        let cov = ph2d_painter_brush::solid::fill_coverage(&b.solid_fill_loops(), n, n, [0.0, 0.0]);
        let total = ink.iter().filter(|i| **i).count();
        assert!(total > 300, "{kind:?}: a fixture nao entintou nada ({total})");
        #[allow(clippy::cast_precision_loss)]
        let hit = ink
            .iter()
            .enumerate()
            .filter(|(i, on)| **on && cov[*i] > 0)
            .count() as f32;
        #[allow(clippy::cast_precision_loss)]
        {
            hit / total as f32
        }
    };
    let plain = covered_fraction(LineKind::None);
    let speed = covered_fraction(LineKind::Speed);
    assert!(
        plain > 0.3,
        "o CONTROLE nao cobre a propria tinta ({plain:.3}): o oraculo esta a medir outra coisa"
    );
    assert!(
        speed >= 0.8 * plain,
        "a tinta do Speed cai FORA da mancha ({speed:.3} contra {plain:.3} do gesto sem efeito): \
         a mancha esta a seguir o ponteiro, e a tinta foi arremessada para outro sitio"
    );
}

/// **A CORDA DE FECHAMENTO LEVA O PINCEL** (W8; report do Enio 2026-08-15: *"a linha reta da área
/// não fechada de Solid deve levar o falloff também"*).
///
/// Um gesto aberto fecha sozinho — o preenchimento liga o último ponto ao primeiro —, e essa aresta
/// era a única da fronteira que o pincel nunca caminhava: tudo em volta tinha a borda macia do
/// falloff e ela tinha o corte do rasterizador.
///
/// ⚠️ **O oráculo é a OUTRA borda da mesma varredura**, não um número escolhido: a linha `y = c`
/// atravessa a corda (à esquerda) e uma aresta que o pincel de facto percorreu (à direita). As duas
/// têm de ter uma banda de meio-tom comparável — é o que *"levar o falloff também"* significa em
/// pixels, e é imune ao raio, ao falloff e à opacidade que a fixture escolher.
///
/// **Mutação que sangra:** devolver `Vec::new()` de `closing_chord_dabs`.
#[test]
fn the_closing_chord_wears_the_brush_like_the_rest_of_the_rim() {
    const N: u32 = 160;
    let n = N as usize;
    let (c, s) = (80.0f32, 40.0f32);
    let mut t = tool(N, PaintMedia::Digital, 7.0);
    t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
    // Um "C": três arestas desenhadas, a quarta (a ESQUERDA) é a corda que o fecho inventa.
    let pts = [
        [c - s, c - s],
        [c + s, c - s],
        [c + s, c + s],
        [c - s, c + s],
    ];
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        for k in 1..=10 {
            #[allow(clippy::cast_precision_loss)]
            let f = k as f32 / 10.0;
            t.on_canvas_pointer(cp(
                [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f],
                PointerPhase::Move,
            ));
        }
    }
    t.on_canvas_pointer(cp(pts[3], PointerPhase::Up));
    // A banda de meio-tom (nem fundo, nem miolo cheio) atravessando `x0` na linha `y`.
    let band = |x0: usize, y: usize| -> usize {
        (x0.saturating_sub(14)..=(x0 + 14).min(n - 1))
            .filter(|x| {
                let v = t.canvas_rgba[(y * n + x) * 4];
                // ⚠️ A janela é LARGA de propósito: a rampa medida do falloff é `249,173,71,17` —
                // um limiar apertado conta 2 dos DOIS lados e o gate perde os dentes sem ninguém ver.
                (3..252).contains(&v)
            })
            .count()
    };
    let mid = c as usize;
    let chord = band((c - s) as usize, mid);
    let walked = band((c + s) as usize, mid);
    assert!(
        walked >= 3,
        "a fixture nao tem borda macia nem onde o pincel passou ({walked}): o oraculo nao mede nada"
    );
    assert!(
        chord * 2 >= walked,
        "a corda de fechamento tem borda DURA ({chord} texels de meio-tom) contra os {walked} da \
         aresta que o pincel percorreu — ela nao levou o falloff"
    );
}
