use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

/// Um recto horizontal a `passo` px por evento, um quadro de 8 ms por evento (o `sonda_gestos::gesto`);
/// devolve a altura da faixa pintada na coluna do meio.
fn altura(m: PaintMedia, passo: f32, linha: Option<(LinhaDaMao, f64)>) -> usize {
    alturas(m, passo, linha, &[400])[0]
}

/// [`altura`] nas colunas `xs`.
fn alturas(
    m: PaintMedia,
    passo: f32,
    linha: Option<(LinhaDaMao, f64)>,
    xs: &[usize],
) -> Vec<usize> {
    alturas_com(m, passo, linha.as_slice(), xs)
}

/// [`alturas`] com várias linhas da mão.
fn alturas_com(
    m: PaintMedia,
    passo: f32,
    linhas: &[(LinhaDaMao, f64)],
    xs: &[usize],
) -> Vec<usize> {
    let mut t = super::super::super::PainterTool::default();
    t.set_source(vec![255u8; 600 * 120 * 4], 600, 120);
    t.set_paint_media(m);
    t.set_brush_size_px(16.0);
    for &(l, v) in linhas {
        t.set_mao(l, v);
    }
    let cp = |x: f32, phase| CanvasPointer {
        pos: [x, 60.0],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp(20.0, PointerPhase::Down));
    let mut x = 20.0;
    while x < 580.0 {
        x += passo;
        t.on_canvas_pointer(cp(x.min(580.0), PointerPhase::Move));
        t.paint_tick(0.008);
    }
    t.on_canvas_pointer(cp(580.0, PointerPhase::Up));
    xs.iter()
        .map(|&x| {
            (0..120)
                .filter(|y| t.canvas_rgba[(y * 600 + x) * 4] < 200)
                .count()
        })
        .collect()
}

/// ⭐ **A mão lenta endurece a queda e a rápida amacia** (J4 efeitos 2.ª ronda, `exames/efeitos.md`; o
/// `hardness ← speed1` do MyPaint `experimental/glow`): com `speedHardness` e o atraso, o traço a 300 px/s
/// NASCE mais largo (a queda mais dura onde a caneta pousa — o caroço do néon) e fica a largura de sempre no meio
/// (a curva é zero a 300 px/s); o rápido (2 500 px/s) sai mais fino que sem a linha. Sem ela, o início é o meio.
#[test]
fn a_mao_lenta_endurece_a_queda() {
    let xs = [24, 400];
    let lag = (LinhaDaMao::SpeedLag, 0.04);
    let dura = (LinhaDaMao::SpeedHardness, 0.3);
    let sem = alturas_com(PaintMedia::Digital, 2.4, &[lag], &xs);
    let com = alturas_com(PaintMedia::Digital, 2.4, &[lag, dura], &xs);
    assert!(
        sem[0].abs_diff(sem[1]) <= 1,
        "sem a linha o início é o meio: {sem:?}"
    );
    assert!(
        com[0] >= com[1] + 2,
        "com a linha o traço lento nasce mais largo: {com:?}"
    );
    assert!(
        com[1].abs_diff(sem[1]) <= 1,
        "a 300 px/s a dureza é a do pincel: {com:?} × {sem:?}"
    );
    // O rápido amacia (a dureza desce): o pincel de omissão já tem a dureza 0, o tecto de baixo — a lei vê-se
    // na própria curva (`ph2d_painter_brush::dynamics`, `a_dureza_da_velocidade_e_a_curva_do_mypaint`).
}

/// ⭐ **A velocidade vale desde o 1.º dab** (J3, `docs/Painter/Pinceis/exames/canetas.md` 2.ª onda): o traço só
/// mede a velocidade no TIQUE, e os dabs do 1.º quadro saíam a `v = 0` — a largura cheia — e a rampa ainda
/// subia de 0 no 2.º: uma BOLHA no início do traço rápido (o Technical Pen contra o Fineliner do Krita, que
/// começa fino). Com o Marker (`speedSize 0,45`) a 50 px por evento, o início (a 2 px do pen-down e a 30 px)
/// tem a largura do meio (± 1 px); sem a lei, idem (o neutro não muda).
#[test]
fn a_velocidade_vale_desde_o_primeiro_dab() {
    let xs = [22, 50, 75, 400];
    for m in [PaintMedia::Digital, PaintMedia::Impasto] {
        let h = alturas(m, 50.0, Some((LinhaDaMao::SpeedSize, 0.45)), &xs);
        for (x, a) in xs.iter().zip(&h) {
            assert!(
                a.abs_diff(h[3]) <= 1,
                "{m:?}: a {x} px a largura é {a}, a do meio {} — {h:?}",
                h[3]
            );
        }
    }
}

/// ⭐ **A velocidade com ATRASO** (J4 efeitos, `docs/Painter/Pinceis/exames/efeitos.md`; o `speed1_slowness` do
/// MyPaint): com `speedLag` 0,04 s o traço rápido NASCE largo (o filtro ainda não viu a mão: `v = 0`) e afina à
/// medida que o filtro a apanha — a cabeça do traço rápido do `experimental/glow`; sem o atraso, o início tem a
/// largura do meio (a lei da J3, intacta). O 1.º dab sai à largura de SEM velocidade, e um atraso maior afina
/// mais tarde (o filtro, não só o arranque a zero).
#[test]
fn a_velocidade_com_atraso_nasce_larga() {
    let xs = [22, 200, 560];
    let marker = (LinhaDaMao::SpeedSize, 0.45);
    let lag = |s: f64| (LinhaDaMao::SpeedLag, s);
    let plena = alturas_com(PaintMedia::Digital, 50.0, &[], &xs);
    let curto = alturas_com(PaintMedia::Digital, 50.0, &[marker, lag(0.04)], &xs);
    let longo = alturas_com(PaintMedia::Digital, 50.0, &[marker, lag(0.2)], &xs);
    let sem = alturas_com(PaintMedia::Digital, 50.0, &[marker], &xs);
    assert!(
        sem[0].abs_diff(sem[2]) <= 1,
        "sem atraso o início é o meio: {sem:?}"
    );
    assert!(
        curto[0] + 1 >= plena[0],
        "com atraso o 1.º dab sai à largura cheia: {curto:?} × {plena:?}"
    );
    assert!(
        curto[2] + 3 <= curto[0],
        "com atraso o traço afina: {curto:?}"
    );
    assert!(
        longo[1] >= curto[1] + 2,
        "o atraso maior afina mais tarde: {longo:?} × {curto:?}"
    );
}

/// A velocidade→opacidade chega ao Wet Paint (a mão dele herda a `Dynamics` do pincel): no recto rápido a
/// tinta muda (no lento do censo a tinta molhada enche até cheia — `o_censo_so_encolhe`).
#[test]
fn a_velocidade_chega_ao_wet() {
    let tela = |v: f64| {
        let mut t = super::super::super::PainterTool::default();
        t.set_source(vec![255u8; 600 * 120 * 4], 600, 120);
        t.set_paint_media(PaintMedia::WetPaint);
        t.set_mao(LinhaDaMao::SpeedOpacity, v);
        let cp = |x: f32, phase| CanvasPointer {
            pos: [x, 60.0],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        };
        t.on_canvas_pointer(cp(20.0, PointerPhase::Down));
        let mut x: f32 = 20.0;
        while x < 580.0 {
            x += 40.0;
            t.on_canvas_pointer(cp(x.min(580.0), PointerPhase::Move));
            t.paint_tick(0.008);
        }
        t.on_canvas_pointer(cp(580.0, PointerPhase::Up));
        for _ in 0..30 {
            t.paint_tick(1.0 / 60.0);
        }
        t.canvas_rgba.to_vec()
    };
    let (a, b) = (tela(0.0), tela(1.0));
    let n = a.iter().zip(&b).filter(|(x, y)| x != y).count();
    assert!(n > 1000, "a velocidade não chegou ao Wet: {n} bytes");
}

/// ⭐ **`recto lento × rápido`**: com a lei do Marker do MyPaint (`speedSize 0,45`) o recto a 50 px por
/// evento (6 250 px/s) sai mais FINO que o de 3 px (375 px/s) — perto da potência medida, `(6250/375)^−0,45`
/// = 0,28 da lenta ± 2 px —, e sem a lei (o Rebelle: 221 presets neutros) os dois têm a mesma largura.
#[test]
fn a_velocidade_estreita_o_traco() {
    for m in [PaintMedia::Digital, PaintMedia::Impasto] {
        let (lento, rapido) = (altura(m, 3.0, None), altura(m, 50.0, None));
        assert!(
            lento > 8 && lento.abs_diff(rapido) <= 1,
            "{m:?} neutro: {lento} × {rapido}"
        );
        let marker = Some((LinhaDaMao::SpeedSize, 0.45));
        let (l, r) = (altura(m, 3.0, marker), altura(m, 50.0, marker));
        let esperado = l as f32 * (6250.0f32 / 375.0).powf(-0.45);
        assert!(
            (r as f32 - esperado).abs() <= 2.0,
            "{m:?}: rápido {r} px × lento {l} px (a lei: {esperado:.1})"
        );
    }
}
