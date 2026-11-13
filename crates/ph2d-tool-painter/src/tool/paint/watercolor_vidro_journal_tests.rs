//! **O vidro no journal por tile** (ADR-0204): o passo do traço captura os tiles do vidro que a região
//! escrita toca, e o antes reconstruído (`journal.get(i).unwrap_or(vivo[i])`) é o plano de antes ao bit.

use crate::compositor::vidro::Vidro;
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use crate::undo::journal::TILE;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

/// 4×3 tiles de 128, a última coluna (8 px) e a última linha (6 px) parciais.
const W: u32 = 520;
const H: u32 = 390;

fn ev(t: &mut PainterTool, pos: [f32; 2], phase: PointerPhase) {
    t.on_canvas_pointer(CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    });
    t.on_tick(1000.0 / 60.0);
}

fn plano(t: &PainterTool) -> Vec<Vidro> {
    let c = t.layers.active().expect("camada activa");
    t.vidros.get(&c).map(|p| p.to_vec()).unwrap_or_default()
}

/// Um traço de `a` a `b`; antes do pen-up confere o journal contra o plano de antes do traço.
fn traco_conferido(t: &mut PainterTool, a: [f32; 2], b: [f32; 2], nome: &str) {
    let antes = plano(t);
    ev(t, a, PointerPhase::Down);
    for k in 1..=10 {
        let s = k as f32 / 10.0;
        ev(
            t,
            [a[0] + (b[0] - a[0]) * s, a[1] + (b[1] - a[1]) * s],
            PointerPhase::Move,
        );
    }
    let vivo = plano(t);
    let ws = &t.undo.write_state;
    let (w, h) = (W as usize, H as usize);
    assert_eq!(antes.len(), w * h, "{nome}: o vidro existe antes do traço");
    let reconstruido: Vec<Vidro> = (0..w * h)
        .map(|i| ws.vidro_before(i).unwrap_or(vivo[i]))
        .collect();
    assert!(
        reconstruido == antes,
        "{nome}: o antes reconstruído do journal difere do plano de antes"
    );
    assert!(vivo != antes, "{nome}: o traço escreveu o vidro");
    // Os bytes = a soma dos tiles da caixa declarada (cada tile pelo tamanho real), nunca o plano inteiro.
    let r = ws.get().declarada().expect("a aguada declara a região");
    let (tx0, ty0) = (r.x as usize / TILE, r.y as usize / TILE);
    let (tx1, ty1) = (
        (r.x + r.w - 1) as usize / TILE,
        (r.y + r.h - 1) as usize / TILE,
    );
    let tile = |tx: usize, ty: usize| {
        (((tx + 1) * TILE).min(w) - tx * TILE) * (((ty + 1) * TILE).min(h) - ty * TILE)
    };
    let tecto: usize = (ty0..=ty1)
        .flat_map(|ty| (tx0..=tx1).map(move |tx| (tx, ty)))
        .map(|(tx, ty)| tile(tx, ty) * size_of::<Vidro>())
        .sum();
    let bytes = ws.vidro_journal_bytes();
    assert!(
        bytes <= tecto && bytes < w * h * size_of::<Vidro>(),
        "{nome}: o journal retém {bytes} B, a caixa declarada pede no máximo {tecto} B"
    );
    // Todo tile onde o vidro mudou foi capturado (senão o antes reconstruído já teria falhado acima).
    assert!(bytes > 0, "{nome}: o journal capturou o vidro");
    ev(t, b, PointerPhase::Up);
    for _ in 0..3 {
        t.on_tick(1000.0 / 60.0);
    }
}

#[test]
fn o_traco_que_cruza_tiles_guarda_so_os_tiles_tocados() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::Watercolor);
    crate::tool::paint::tests::aquarela_neutra(&mut t);
    crate::tool::paint::tests::watercolor_basic_de_antes(&mut t);
    t.set_brush_size_px(10.0);
    t.set_brush_color_srgb8([30, 60, 220]);
    // O 1.º traço faz nascer o vidro (sem antes a descrever).
    ev(&mut t, [40.0, 40.0], PointerPhase::Down);
    for k in 1..=8 {
        ev(
            &mut t,
            [40.0 + 50.0 * k as f32, 40.0 + 35.0 * k as f32],
            PointerPhase::Move,
        );
    }
    let ws = &t.undo.write_state;
    assert!(
        ws.vidro_absent() && ws.vidro_journal_bytes() == 0,
        "o vidro que nasce no traço é `absent` (sem antes), não capturado: absent={} bytes={} plano={}",
        ws.vidro_absent(),
        ws.vidro_journal_bytes(),
        plano(&t).len()
    );
    ev(&mut t, [440.0, 320.0], PointerPhase::Up);
    for _ in 0..3 {
        t.on_tick(1000.0 / 60.0);
    }
    traco_conferido(&mut t, [60.0, 20.0], [400.0, 300.0], "diagonal");
    traco_conferido(
        &mut t,
        [127.5, 30.0],
        [128.5, 200.0],
        "colado a x = 127|128",
    );
    traco_conferido(&mut t, [505.0, 375.0], [517.0, 387.0], "canto parcial");
}

/// **ADR-0204 passo 0 — quem segura o plano do vidro no 2.º pen-down** (a contagem não depende do tamanho da
/// tela). Imprime, por dono, quem segura o plano vivo antes e depois do 1.º quadro do 2.º traço, e as cópias.
/// Asserta só que a conta fecha: nenhum dono sem nome.
#[test]
fn quem_segura_o_vidro_no_segundo_pen_down() {
    use super::watercolor_vidro::COPIAS_DO_VIDRO;
    use std::sync::Arc;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::Watercolor);
    crate::tool::paint::tests::aquarela_neutra(&mut t);
    crate::tool::paint::tests::watercolor_basic_de_antes(&mut t);
    t.set_brush_size_px(10.0);
    let traco = |t: &mut PainterTool, y: f32| {
        ev(t, [40.0, y], PointerPhase::Down);
        for k in 1..=6 {
            ev(t, [40.0 + 20.0 * k as f32, y], PointerPhase::Move);
        }
        ev(t, [160.0, y], PointerPhase::Up);
        for _ in 0..3 {
            t.on_tick(1000.0 / 60.0);
        }
    };
    traco(&mut t, 60.0);
    let c = t.layers.active().expect("camada");
    let donos = |t: &PainterTool, quando: &str| {
        let p = t.vidros.get(&c).expect("o vidro existe").clone();
        let total = Arc::strong_count(&p) - 1; // menos o clone desta linha
        let antes = t
            .paint
            .stroke_undo
            .as_ref()
            .and_then(|s| s.vidros.get(&c))
            .is_some_and(|v| Arc::ptr_eq(v, &p));
        let cursor = t.undo.cursor_segura_o_vidro(c, &p);
        let congelado = t
            .vidro_congelado
            .aguada
            .as_ref()
            .is_some_and(|b| Arc::ptr_eq(&b.2, &p));
        let nomeados = 1 + usize::from(antes) + usize::from(cursor) + usize::from(congelado);
        println!(
            "{quando}: strong={total} · mapa=1 · before={antes} · cursor={cursor} · congelado={congelado} · sem nome={}",
            total as isize - nomeados as isize
        );
        (total, nomeados)
    };
    let repouso = donos(&t, "repouso depois do 1.º traço");
    COPIAS_DO_VIDRO.set(0);
    traco(&mut t, 110.0);
    println!("cópias do plano no 2.º traço: {}", COPIAS_DO_VIDRO.get());
    let repouso_3 = donos(&t, "repouso depois do 2.º traço");
    COPIAS_DO_VIDRO.set(0);
    traco(&mut t, 140.0);
    println!("cópias do plano no 3.º traço: {}", COPIAS_DO_VIDRO.get());
    // O 1.º traço CRIOU o plano: a entrada dele guarda o `after` (OnlyAfter) e segura-o até ao 2.º traço.
    assert_eq!(
        repouso.0,
        repouso.1 + 1,
        "depois do 1.º traço: mapa + cursor + a entrada que o criou"
    );
    assert_eq!(
        repouso_3.0, repouso_3.1,
        "depois do 2.º traço: só mapa + cursor"
    );
}
