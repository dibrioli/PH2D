//! SONDA — a coluna da direita VAZIA contra a banda do grafo que cresce para dentro dela.
//!
//! Report do dono (2026-09-20): *«pisca do lado direito quando escondemos o inspector e
//! aumentamos muito a área do grafo de nós»*.

use ph2d_editor_core::screens::layout::{CenterSplit, ChromeBands, DockSides, HeroLayout};
use ph2d_editor_core::zones::Rect;

fn sobreposicao(r: Rect, col: Rect) -> f32 {
    let w = (r.x + r.w).min(col.x + col.w) - r.x.max(col.x);
    let h = (r.y + r.h).min(col.y + col.h) - r.y.max(col.y);
    if w <= 0.0 || h <= 0.0 { 0.0 } else { w * h }
}

/// Um quadro: recebe o que o quadro anterior PUBLICOU e devolve o que este publica.
fn um_quadro(
    viewport: Rect,
    bands: ChromeBands,
    t: f32,
    timeline: bool,
    publicado: &[Rect],
) -> (DockSides, Vec<Rect>, Rect, Rect) {
    let monta = |docks| {
        let mut l = HeroLayout::for_viewport_bands(
            viewport,
            false,
            bands,
            CenterSplit::Horizontal { t },
            docks,
        );
        if timeline {
            l.dock_timeline_into_motion();
        }
        l
    };
    let sonda = monta(DockSides::BOTH);
    let (esq, dir) = sonda.side_columns();
    let docks = DockSides::from_published(esq, dir, publicado.iter().copied());
    let layout = monta(docks);
    // ⚠️ O Inspector esta ESCONDIDO — e' o estado do report; ninguem publica na coluna direita.
    (
        docks,
        vec![layout.hierarchy, layout.motion_graph],
        layout.motion_graph,
        dir,
    )
}

fn varre(nome: &str, timeline: bool) {
    let viewport = Rect::new(0.0, 0.0, 1930.0, 1012.0);
    let mut bands = ChromeBands::DEFAULT;
    bands.rail_w = 0.0;
    eprintln!("\n=== {nome} ===");
    eprintln!(
        "  {:<6} {:<12} {:<10} {:<10} veredito",
        "t", "8 quadros", "grafo/col", "col/grafo"
    );
    for passo in 0..=20 {
        let t = passo as f32 / 20.0;
        let mut publicado: Vec<Rect> = Vec::new();
        let (mut hist, mut ultimo) = (
            Vec::new(),
            (Rect::new(0.0, 0.0, 0.0, 0.0), Rect::new(0.0, 0.0, 0.0, 0.0)),
        );
        for _ in 0..8 {
            let (docks, proximo, grafo, col) = um_quadro(viewport, bands, t, timeline, &publicado);
            hist.push(u8::from(docks.right));
            publicado = proximo;
            ultimo = (grafo, col);
        }
        let (grafo, col) = ultimo;
        let sob = sobreposicao(grafo, col);
        let (a_col, a_grafo) = (col.w * col.h, grafo.w * grafo.h);
        let cauda = &hist[4..];
        let estavel = cauda.iter().all(|&v| v == cauda[0]);
        let seq: String = hist
            .iter()
            .map(|v| if *v == 1 { 'X' } else { '.' })
            .collect();
        eprintln!(
            "  {t:<6.2} {seq:<12} {:<10.3} {:<10.3} {}",
            if a_col > 0.0 { sob / a_col } else { 0.0 },
            if a_grafo > 0.0 { sob / a_grafo } else { 0.0 },
            if estavel { "estavel" } else { "*** PISCA ***" }
        );
    }
}

#[test]
#[ignore = "sonda de medicao"]
fn diag_a_coluna_vazia_pisca() {
    varre("sem timeline docada", false);
    varre("COM a timeline docada (o estado do Motion)", true);
}

/// Que largura de FAIXA aparece e desaparece, e quanto mede a coluna.
#[test]
#[ignore = "sonda de medicao"]
fn diag_a_largura_da_faixa_que_pisca() {
    let viewport = Rect::new(0.0, 0.0, 1930.0, 1012.0);
    let mut bands = ChromeBands::DEFAULT;
    bands.rail_w = 0.0;
    let t = 0.2;
    let mut publicado: Vec<Rect> = Vec::new();
    eprintln!("\n=== a faixa que aparece e desaparece (t = {t}) ===");
    for q in 0..6 {
        let (docks, proximo, grafo, col) = um_quadro(viewport, bands, t, true, &publicado);
        eprintln!(
            "  quadro {q}: right={:<5} grafo x {:.1}..{:.1} (w {:.1})   coluna x {:.1}..{:.1} (w {:.1})",
            docks.right,
            grafo.x,
            grafo.x + grafo.w,
            grafo.w,
            col.x,
            col.x + col.w,
            col.w
        );
        publicado = proximo;
    }
}

/// COM a timeline docada, a fronteira e' a ALTURA DA JANELA — ela esta' a um fio.
#[test]
#[ignore = "sonda de medicao"]
fn diag_a_altura_da_janela_decide() {
    let mut bands = ChromeBands::DEFAULT;
    bands.rail_w = 0.0;
    eprintln!("\n=== com a timeline docada, t = 0 (o grafo no maximo) ===");
    eprintln!(
        "  {:<8} {:<12} {:<10} veredito",
        "altura", "8 quadros", "grafo/col"
    );
    for h in [900, 950, 1000, 1012, 1050, 1100, 1200, 1400, 1600] {
        let viewport = Rect::new(0.0, 0.0, 1930.0, h as f32);
        let mut publicado: Vec<Rect> = Vec::new();
        let mut hist = Vec::new();
        let mut razao = 0.0;
        for q in 0..8 {
            let (docks, proximo, grafo, col) = um_quadro(viewport, bands, 0.0, true, &publicado);
            hist.push(u8::from(docks.right));
            if q == 0 {
                let a = col.w * col.h;
                razao = if a > 0.0 {
                    sobreposicao(grafo, col) / a
                } else {
                    0.0
                };
            }
            publicado = proximo;
        }
        let cauda = &hist[4..];
        let estavel = cauda.iter().all(|&v| v == cauda[0]);
        let seq: String = hist
            .iter()
            .map(|v| if *v == 1 { 'X' } else { '.' })
            .collect();
        eprintln!(
            "  {h:<8} {seq:<12} {razao:<10.3} {}",
            if estavel { "estavel" } else { "*** PISCA ***" }
        );
    }
}
