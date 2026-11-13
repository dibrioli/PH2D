//! **O «+» sem diff** (n.º 7 item 3, chave [`MAIS_SEM_DIFF`]): a entrada do «+» transparente guarda a tela que
//! saiu inteira sem a varrer contra a nova, e não segura a nova (toda neutra). O desfazer e o refazer dão o
//! documento ao bit da rota antiga (a chave desligada), no mesmo processo; o 1.º traço na camada nova escreve
//! a tela no lugar; o tecto conta a entrada pelo tamanho real.

use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use crate::undo_planes::MAIS_SEM_DIFF;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use std::sync::Arc;

const W: u32 = 520;
const H: u32 = 390;
/// Bytes de um plano RGBA da tela.
const PLANO: usize = (W as usize) * (H as usize) * 4;

#[derive(Clone, Copy)]
enum Meio {
    Wb,
    Wet,
}

fn ferramenta(m: Meio) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; PLANO], W, H);
    match m {
        Meio::Wb => {
            t.set_paint_media(PaintMedia::Watercolor);
            crate::tool::paint::tests::aquarela_neutra(&mut t);
            crate::tool::paint::tests::watercolor_basic_de_antes(&mut t);
        }
        Meio::Wet => t.set_paint_tool_mode("wetpaint"),
    }
    t.set_brush_size_px(14.0);
    t.set_brush_color_srgb8([30, 60, 220]);
    t
}

fn ev(t: &mut PainterTool, pos: [f32; 2], phase: PointerPhase) {
    t.on_canvas_pointer(CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    });
    t.on_tick(1000.0 / 60.0);
}

/// O traço `k`; devolve quantos donos seguram a tela no pen-down além da tela viva e do cursor (que o
/// pen-down elide): com um dono a mais o 1.º fork copia a tela inteira.
fn traco(t: &mut PainterTool, k: usize) -> usize {
    let y = 40.0 + 41.0 * (k % 8) as f32;
    let (x0, x1) = if k.is_multiple_of(2) {
        (30.0, 300.0)
    } else {
        (110.0, 480.0)
    };
    let cursor = t
        .undo
        .cursor_canvas_ptr()
        .is_some_and(|p| p == Arc::as_ptr(&t.canvas_rgba));
    let donos = Arc::strong_count(&t.canvas_rgba) - 1 - usize::from(cursor);
    ev(t, [x0, y], PointerPhase::Down);
    ev(t, [x0 + 20.0, y + 2.0], PointerPhase::Move);
    for s in 2..=8 {
        let f = s as f32 / 8.0;
        ev(t, [x0 + (x1 - x0) * f, y + 30.0 * f], PointerPhase::Move);
    }
    ev(t, [x1, y + 30.0], PointerPhase::Up);
    for _ in 0..3 {
        t.on_tick(1000.0 / 60.0);
    }
    t.wet_bring_home();
    donos
}

fn documento(t: &PainterTool) -> Doc {
    let mut telas = vec![t.canvas_rgba.to_vec()];
    telas.extend(t.images.values().map(|i| i.rgba8.to_vec()));
    (t.layers.active(), t.layers.len(), telas)
}

/// `j` (a chave ligada) e `o` (a rota antiga) fazem o mesmo gesto; devolve o que `f` deu em cada um.
fn par<R>(
    f: &mut dyn FnMut(&mut PainterTool) -> R,
    j: &mut PainterTool,
    o: &mut PainterTool,
    quando: &str,
) -> (R, R) {
    MAIS_SEM_DIFF.set(true);
    let a = f(j);
    MAIS_SEM_DIFF.set(false);
    let b = f(o);
    MAIS_SEM_DIFF.set(true);
    // O Wet corre o motor fora do fio: duas ferramentas não dão o mesmo bit entre si (medido com a chave
    // IGUAL nos dois lados), então lá cada rota confere-se contra o próprio histórico gravado.
    if CRUZADO.get() {
        assert!(
            documento(j) == documento(o),
            "{quando}: o documento difere da rota antiga"
        );
    }
    HIST.with_borrow_mut(|h| h.push((documento(j), documento(o))));
    (a, b)
}

type Doc = (Option<crate::layers::LayerId>, usize, Vec<Vec<u8>>);

thread_local! {
    static CRUZADO: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
    static HIST: std::cell::RefCell<Vec<(Doc, Doc)>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Desfaz (ou refaz) um passo nas duas rotas e confere cada uma contra o estado gravado `i`.
fn volta(j: &mut PainterTool, o: &mut PainterTool, desfaz: bool, i: usize, quando: &str) -> bool {
    let anda = |t: &mut PainterTool| {
        let r = if desfaz { t.undo_last() } else { t.redo_last() };
        t.wet_bring_home();
        r
    };
    MAIS_SEM_DIFF.set(true);
    let a = anda(j);
    MAIS_SEM_DIFF.set(false);
    let b = anda(o);
    MAIS_SEM_DIFF.set(true);
    assert_eq!(a, b, "{quando}: um lado parou antes");
    if a {
        let (dj, d_o) = HIST.with_borrow(|h| h[i].clone());
        assert!(
            documento(j) == dj,
            "{quando}: a chave não volta ao estado gravado"
        );
        assert!(
            documento(o) == d_o,
            "{quando}: a rota antiga não volta ao estado gravado"
        );
    }
    a
}

/// O «+» medido: os bytes que a entrada dele soma ao tecto.
fn mais(t: &mut PainterTool) -> usize {
    let b0 = t.undo.retained_bytes();
    t.add_raster_layer("nova").expect("+");
    t.undo.retained_bytes() - b0
}

/// Base branca opaca, dois traços, «+», dois traços na nova, «+» outra vez (sobre a camada rala), um traço;
/// desfaz tudo e refaz tudo ao bit da rota antiga. Controlo: a rota antiga segura a tela nova (o 1.º traço
/// nela copia) e guarda `Whole` dos dois lados no «+» sobre a base opaca.
fn o_mais_da_o_bit(m: Meio, nome: &str) {
    let mut j = ferramenta(m);
    let mut o = ferramenta(m);
    CRUZADO.set(matches!(m, Meio::Wb));
    HIST.with_borrow_mut(|h| {
        h.clear();
        h.push((documento(&j), documento(&o)));
    });
    for k in 0..2 {
        par(
            &mut |t| traco(t, k),
            &mut j,
            &mut o,
            &format!("{nome} base {k}"),
        );
    }
    let (bj, bo) = par(&mut mais, &mut j, &mut o, &format!("{nome} «+» 1"));
    println!("{nome}: «+» sobre a base opaca: {bj} B (rota antiga {bo} B), plano = {PLANO} B");
    assert_eq!(
        bj,
        2 * PLANO,
        "{nome}: a entrada do «+» pesa a tela que saiu (Nova) + a imagem dela (OnlyAfter)"
    );
    assert_eq!(
        bo,
        3 * PLANO,
        "{nome}: controlo: a rota antiga guarda Whole dos dois lados + a imagem"
    );
    let (lj, lo) = par(
        &mut |t| traco(t, 2),
        &mut j,
        &mut o,
        &format!("{nome} nova 2"),
    );
    assert_eq!(
        lj, 0,
        "{nome}: no 1.º traço da camada nova ninguém além do cursor segura a tela"
    );
    assert_eq!(
        lo, 1,
        "{nome}: controlo: na rota antiga a entrada segura a tela nova e o 1.º traço copia-a"
    );
    par(
        &mut |t| traco(t, 3),
        &mut j,
        &mut o,
        &format!("{nome} nova 3"),
    );
    let (bj2, bo2) = par(&mut mais, &mut j, &mut o, &format!("{nome} «+» 2"));
    println!("{nome}: «+» sobre a camada rala: {bj2} B (rota antiga {bo2} B)");
    assert_eq!(bj2, 2 * PLANO, "{nome}: o 2.º «+» pesa o mesmo");
    assert!(
        bo2 < bj2,
        "{nome}: controlo: sobre a camada rala a rota antiga guarda só a janela (Patch)"
    );
    par(
        &mut |t| traco(t, 4),
        &mut j,
        &mut o,
        &format!("{nome} nova 4"),
    );
    let topo = HIST.with_borrow(Vec::len) - 1;
    let mut n = 0;
    while volta(
        &mut j,
        &mut o,
        true,
        (topo - n).saturating_sub(1),
        &format!("{nome} desfeito {n}"),
    ) {
        n += 1;
    }
    assert_eq!(n, topo, "{nome}: desfez-se tudo o que se fez");
    assert!(n >= 7, "{nome}: controlo: só {n} passos desfeitos");
    assert_eq!(j.layers.len(), 1, "{nome}: desfeito tudo, sobra a base");
    for k in 0..n {
        assert!(
            volta(&mut j, &mut o, false, k + 1, &format!("{nome} refeito {k}")),
            "{nome}: refazer {k}"
        );
    }
    CRUZADO.set(true);
}

#[test]
fn o_mais_sem_diff_da_o_bit_no_desfazer_e_no_refazer_wb() {
    o_mais_da_o_bit(Meio::Wb, "WB");
}

#[test]
fn o_mais_sem_diff_da_o_bit_no_desfazer_e_no_refazer_wet() {
    o_mais_da_o_bit(Meio::Wet, "Wet");
}

/// **A contabilidade do tecto:** cada passo pesa o que a entrada dele soma, o «+» pesa `2 × plano` (a tela
/// que saiu, pelo tamanho real, e a imagem dela), e o tecto despeja por baixo até caber. Com o tecto que cabe
/// o histórico inteiro da rota antiga, a nova retém exactamente o que a simulação do despejo prevê — menos.
#[test]
fn o_tecto_conta_o_mais_pelo_tamanho_real() {
    // A pilha: «+» sobre a base opaca, um traço na nova (rala), «+» sobre a rala, seis traços.
    let corre = |chave: bool, tecto: Option<usize>| {
        MAIS_SEM_DIFF.set(chave);
        let mut t = ferramenta(Meio::Wb);
        let mut pesos = vec![mais(&mut t)];
        let passo = |t: &mut PainterTool, f: &mut dyn FnMut(&mut PainterTool)| {
            let b = t.undo.retained_bytes();
            f(t);
            t.undo.retained_bytes() - b
        };
        pesos.push(passo(&mut t, &mut |t| {
            traco(t, 0);
        }));
        pesos.push(mais(&mut t));
        for k in 1..=6 {
            pesos.push(passo(&mut t, &mut |t| {
                traco(t, k);
            }));
        }
        assert_eq!(
            t.undo.undo_depth(),
            pesos.len(),
            "nenhum passo despejado sem tecto"
        );
        if let Some(c) = tecto {
            t.undo.set_max_bytes(c);
        }
        MAIS_SEM_DIFF.set(true);
        (pesos, t.undo.undo_depth())
    };
    // O despejo do `cap`: tira por baixo enquanto a soma passa o tecto (e sobra mais de um passo).
    let simula = |pesos: &[usize], tecto: usize| {
        let mut soma: usize = pesos.iter().sum();
        let mut i = 0;
        while pesos.len() - i > 1 && soma > tecto {
            soma -= pesos[i];
            i += 1;
        }
        pesos.len() - i
    };
    let (antigo, _) = corre(false, None);
    let (novo, _) = corre(true, None);
    println!(
        "pesos por passo: rota antiga {antigo:?}\n                 nova        {novo:?} (plano {PLANO} B)"
    );
    assert_eq!(
        novo[0],
        2 * PLANO,
        "o «+» pesa a tela que saiu + a imagem dela"
    );
    assert_eq!(novo[2], 2 * PLANO, "o «+» sobre a rala pesa o mesmo");
    assert_eq!(
        antigo[0],
        3 * PLANO,
        "controlo: sobre a base opaca a rota antiga guarda Whole dos dois lados"
    );
    assert!(
        antigo[2] < novo[2],
        "controlo: sobre a rala a rota antiga guarda a janela"
    );
    assert_eq!(
        antigo[1..2],
        novo[1..2],
        "o traço pesa o mesmo nas duas rotas"
    );
    assert_eq!(
        antigo[3..],
        novo[3..],
        "os traços pesam o mesmo nas duas rotas"
    );
    // O tecto que cabe tudo menos o 1.º «+» da rota antiga: o «+» sobre a rala, mais pesado na nova, faz
    // despejar mais um passo.
    let tecto = antigo[1..].iter().sum::<usize>();
    let (_, prof_antiga) = corre(false, Some(tecto));
    let (_, prof_nova) = corre(true, Some(tecto));
    println!(
        "tecto {tecto} B: rota antiga retém {prof_antiga} passos, a nova {prof_nova} (previstas {} · {})",
        simula(&antigo, tecto),
        simula(&novo, tecto)
    );
    assert_eq!(
        prof_antiga,
        simula(&antigo, tecto),
        "o tecto despeja o que a conta prevê (antiga)"
    );
    assert_eq!(
        prof_nova,
        simula(&novo, tecto),
        "o tecto despeja o que a conta prevê (nova)"
    );
    assert!(
        prof_nova < prof_antiga,
        "controlo: com o «+» sem diff o desfazer lembra menos passos"
    );
}
