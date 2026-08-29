//! Os gates dos **FIOS no PRODUTO** ([`super::thread_deposit`]) — o motor tem os dele em
//! `ph2d_painter_brush::{stroke::threads, thread_raster}`; aqui pergunta-se o que o **gesto** faz.

use super::measure_shape_system::{cp, tool};
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_brush::StrokeMethod;
use ph2d_painter_brush::line_kind::LineKind;

/// Quantos texels do canvas deixaram de ser o branco de fundo.
fn inked(t: &PainterTool) -> usize {
    t.canvas_rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[0] < 250)
        .count()
}

/// Arma o Sketchy com números que costuram de facto.
fn arm(t: &mut PainterTool, density: f32) {
    t.paint.brush.line_kind = LineKind::Sketchy;
    t.paint.brush.sketchy_reach = 3.0;
    t.paint.brush.sketchy_density = density;
    t.paint.brush.thread_width_px = 1.0;
    t.paint.brush.thread_opacity = 0.5;
}

/// Um zigue-zague — o gesto que volta para perto de si mesmo, onde há vizinhos a costurar.
///
/// ⚠️ **As pernas ficam MAIS LONGE que a largura do rastro e mais perto que o alcance**, e a fixture
/// nasceu sem isso: com o alcance de UM diâmetro os fios caem todos DENTRO do rastro do pincel, o
/// desenho fica mais escuro e **nenhum texel novo é entintado** — a primeira versão do gate mediu
/// `3366 contra 3366` e reprovou um produto correto. É o vão entre as pernas que a teia atravessa.
fn zigzag(t: &mut PainterTool, c: f32) {
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.on_canvas_pointer(cp([c - 40.0, c], PointerPhase::Down));
    for leg in 0..6 {
        #[allow(clippy::cast_precision_loss)]
        let x = c - 40.0 + (leg as f32) * 16.0;
        let up = if leg % 2 == 0 { 1.0 } else { -1.0 };
        for k in 1..=8 {
            #[allow(clippy::cast_precision_loss)]
            let y = c + up * (k as f32) * 4.0;
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        }
    }
    t.on_canvas_pointer(cp([c + 32.0, c], PointerPhase::Up));
}

/// **O TRAÇO COSTURA-SE, E A TEIA APARECE NA TELA** — a frase da feature no único oráculo que conta,
/// que é a tinta: o mesmo gesto com o tipo armado entinta mais texels que sem ele.
///
/// ⚠️ O oráculo é a RAZÃO contra o MESMO gesto em `None`, nunca um número absoluto: o que a feature
/// promete é tinta ALÉM do rastro do pincel, e é isso que a razão mede.
#[test]
fn the_sketchy_lays_ink_beyond_the_brushs_own_trail() {
    let side = 256u32;
    let mut plain = tool(side, PaintMedia::Digital, 4.0);
    plain.paint.brush.line_kind = LineKind::None;
    zigzag(&mut plain, 128.0);
    let bare = inked(&plain);

    let mut sewn = tool(side, PaintMedia::Digital, 4.0);
    arm(&mut sewn, 0.4);
    zigzag(&mut sewn, 128.0);
    let web = inked(&sewn);

    assert!(bare > 200, "controle: o traço nu tem de pintar ({bare})");
    assert!(
        web > bare + bare / 10,
        "a teia não chegou à tela: {web} texels contra {bare} do traço nu"
    );
}

/// **O NEUTRO É BYTE-IDÊNTICO** — as duas portas: o tipo `None` e a densidade zero.
///
/// ⚠️ **Mutação que sangra:** o depósito ignorar o [`super::thread_deposit::PainterTool::threads_own_the_gesture`]
/// e escrever sempre que houver fios.
#[test]
fn the_neutral_is_byte_identical() {
    let side = 128u32;
    let mut plain = tool(side, PaintMedia::Digital, 4.0);
    plain.paint.brush.line_kind = LineKind::None;
    zigzag(&mut plain, 64.0);

    let mut zero = tool(side, PaintMedia::Digital, 4.0);
    arm(&mut zero, 0.0);
    zigzag(&mut zero, 64.0);
    assert_eq!(
        plain.canvas_rgba, zero.canvas_rgba,
        "Density 0 moveu a tinta"
    );

    // ⚠️ E o CONTROLE: a MESMA fixture com densidade viva DIVERGE, senão a igualdade acima é
    // verdadeira de graça (um gesto que não pinta nada satisfaz os dois lados).
    let mut armed = tool(side, PaintMedia::Digital, 4.0);
    arm(&mut armed, 0.4);
    zigzag(&mut armed, 64.0);
    assert_ne!(
        plain.canvas_rgba, armed.canvas_rgba,
        "controle: com densidade viva a tinta TEM de mudar"
    );
}

/// **UM MÉTODO DE RE-CARIMBO NÃO COSTURA** — e o motivo está no doc do `threads_own_the_gesture`: a
/// figura é re-emitida INTEIRA a cada quadro, então a memória do traço cresceria por quadro e a teia
/// adensaria enquanto o artista apenas olha.
///
/// ⚠️ Isto é o ESCOPO da wave escrito como gate, não uma limitação escondida: o dia em que os shape
/// editors ganharem o Sketchy, é este gate que muda de sentido — e alguém tem de o reescrever de
/// propósito em vez de descobrir a teia engordando parada.
#[test]
fn a_re_stamp_method_does_not_sew() {
    let side = 128u32;
    let mut t = tool(side, PaintMedia::Digital, 4.0);
    arm(&mut t, 0.4);
    assert!(
        t.threads_own_the_gesture(),
        "controle: em Space o gesto TEM de costurar"
    );
    t.paint.brush.stroke_method = StrokeMethod::DragDot;
    assert!(
        !t.threads_own_the_gesture(),
        "um método de re-carimbo entrou no Sketchy"
    );

    // ⚠️ **E a metade que prova que o DEPÓSITO consulta a porta**, e não só que a porta responde
    // certo: o mesmo gesto com o tipo armado tem de pintar **byte-idêntico** ao sem ele. Sem isto a
    // asserção acima testaria a função com a própria função.
    //
    // ⚠️ **O método é o `DragDot`, e ele foi ESCOLHIDO por medição, não por ser o primeiro da lista.**
    // A primeira versão usou `Anchored` e a mutação (o depósito ignorar a porta) **SOBREVIVEU**: com
    // ele — e com `FreeHand` e `Line` — o gesto nem chega ao `park_stroke` (o roteador de figuras o
    // intercepta antes), então a tinta sai igual pelo motivo ERRADO e o gate era vácuo. Medido: sob a
    // mutação o `DragDot` vai de 44 para 147 texels; os outros três não movem um byte.
    let dragdot_gesture = |kind: LineKind| {
        let mut t = tool(side, PaintMedia::Digital, 4.0);
        arm(&mut t, 0.4);
        t.paint.brush.line_kind = kind;
        t.paint.brush.stroke_method = StrokeMethod::DragDot;
        t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
        for k in 1..=10 {
            #[allow(clippy::cast_precision_loss)]
            let f = k as f32;
            t.on_canvas_pointer(cp([20.0 + f * 8.0, 20.0 + f * 8.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([100.0, 100.0], PointerPhase::Up));
        (*t.canvas_rgba).clone()
    };
    let bare = dragdot_gesture(LineKind::None);
    assert!(
        bare.as_chunks::<4>().0.iter().any(|p| p[0] < 250),
        "controle: o gesto Drag Dot tem de pintar"
    );
    assert_eq!(
        bare,
        dragdot_gesture(LineKind::Sketchy),
        "o Sketchy costurou num método de re-carimbo"
    );
}

/// **A TEIA ATRAVESSA O MESMO GATE QUE OS DABS** — a seleção é um fator por texel aplicado sobre a
/// tinta livre, e um canal novo de tinta que a ignorasse seria a segunda semântica de proteção do
/// módulo.
///
/// ⚠️ **O oráculo é o LADO DE FORA da seleção, com CONTROLE dos dois lados:** sem seleção o mesmo
/// gesto entinta à direita da fronteira (senão o gate seria verdadeiro de graça), e com ela nada
/// sobrevive lá.
///
/// **Mutação que sangra:** o `stamp_threads` chamar o `write_threads` direto, sem o `gate_scoped`.
#[test]
fn the_web_goes_through_the_selection_gate() {
    let side = 128u32;
    // Quantos texels entintados caem à DIREITA de `x`.
    let right_of = |t: &PainterTool, x: usize| {
        (0..side as usize)
            .flat_map(|y| (x..side as usize).map(move |cx| (cx, y)))
            .filter(|(cx, y)| t.canvas_rgba[(y * side as usize + cx) * 4] < 250)
            .count()
    };

    let mut open = tool(side, PaintMedia::Digital, 4.0);
    arm(&mut open, 0.4);
    zigzag(&mut open, 64.0);
    let spill = right_of(&open, 70);
    assert!(
        spill > 20,
        "controle: sem seleção o gesto TEM de entintar à direita de x=70 ({spill})"
    );

    let mut held = tool(side, PaintMedia::Digital, 4.0);
    arm(&mut held, 0.4);
    held.set_rect_selection(0, 0, 70, side); // só a metade esquerda é pintável
    assert!(held.selection_active(), "controle: a seleção tem de viver");
    zigzag(&mut held, 64.0);
    assert_eq!(
        right_of(&held, 70),
        0,
        "a teia passou por cima da fronteira da seleção"
    );
}

/// **O `Connection Line` DESLIGADO tira o TRAÇO e deixa o ARAME** — as duas metades, porque só a
/// primeira seria *"o pincel parou de pintar"*.
///
/// ⚠️ **A sonda fica FORA do arco, e a 1ª versão a pôs SOBRE ele — reprovando um produto correto.**
/// Toda corda começa no dab ATUAL, que está sobre o caminho; conforme o traço anda, esse extremo
/// varre o caminho inteiro ⇒ **o eixo é entintado pelas próprias cordas**, com ou sem o traço. O que
/// só o depósito alcança é a LARGURA do rastro: uma corda é fina e mora *dentro* da curva (a corda
/// corta a quina), então um texel a meio raio para FORA do arco é tinta de dab e de mais nada.
///
/// **Mutação que sangra:** a supressão ignorar o flag (⇒ a borda externa continua entintada).
#[test]
fn switching_off_the_connection_line_drops_the_stroke_and_keeps_the_wire() {
    let side = 256u32;
    let (cx, cy, r) = (40.0f32, 40.0f32, 80.0f32); // um quarto de círculo bem aberto
    let brush = 8.0f32;
    let ang = |i: usize| {
        #[allow(clippy::cast_precision_loss)]
        let u = i as f32 / 60.0;
        u * std::f32::consts::FRAC_PI_2
    };
    let arc = |t: &mut PainterTool| {
        t.paint.brush.stroke_method = StrokeMethod::Space;
        t.paint.brush.line_kind = LineKind::Wire;
        t.paint.brush.wire_history = 4.0;
        t.paint.brush.thread_width_px = 1.0;
        t.paint.brush.thread_opacity = 0.8;
        let pt = |i: usize| [cx + r * ang(i).cos(), cy + r * ang(i).sin()];
        t.on_canvas_pointer(cp(pt(0), PointerPhase::Down));
        for i in 1..=60 {
            t.on_canvas_pointer(cp(pt(i), PointerPhase::Move));
        }
        t.on_canvas_pointer(cp(pt(60), PointerPhase::Up));
    };
    // Meio raio de pincel para FORA do arco: só o depósito de dabs alcança ali.
    let a = ang(30);
    let outside = [
        cx + (r + brush * 0.5) * a.cos(),
        cy + (r + brush * 0.5) * a.sin(),
    ];
    let painted = |t: &PainterTool, p: [f32; 2]| {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let i = ((p[1] as usize) * side as usize + (p[0] as usize)) * 4;
        t.canvas_rgba[i] < 250
    };

    let mut on = tool(side, PaintMedia::Digital, brush);
    on.paint.brush.wire_connection_line = true;
    arc(&mut on);
    assert!(
        painted(&on, outside),
        "controle: com a Connection Line LIGADA a largura do rastro tem de estar pintada"
    );

    let mut off = tool(side, PaintMedia::Digital, brush);
    off.paint.brush.wire_connection_line = false;
    arc(&mut off);
    assert!(
        !painted(&off, outside),
        "a Connection Line desligada e a borda do rastro continua pintada: a supressão não pegou"
    );
    // E o arame CONTINUA — senão isto seria só o pincel a parar de pintar.
    assert!(
        inked(&off) > 200,
        "com a Connection Line desligada sobraram {} texels: o arame sumiu junto",
        inked(&off)
    );
}
