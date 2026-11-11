use super::*;
use ph2d_timeline::ContainerView;

fn lab(c: Color) -> (f64, f64, f64) {
    let u = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    let k = c.components;
    let (l, ch, h) = ph2d_tokens::srgb_to_oklch(u(k[0]), u(k[1]), u(k[2]));
    let h = h.to_radians();
    (l, ch * h.cos(), ch * h.sin())
}

fn dist(a: Color, b: Color) -> f64 {
    let (x, y) = (lab(a), lab(b));
    ((x.0 - y.0).powi(2) + (x.1 - y.1).powi(2) + (x.2 - y.2).powi(2)).sqrt()
}

/// `c` (com o alfa dela) sobre o fundo opaco `f`.
fn sobre(c: Color, f: Color) -> Color {
    let a = c.components[3];
    let k = |i: usize| c.components[i] * a + f.components[i] * (1.0 - a);
    Color::new([k(0), k(1), k(2), 1.0])
}

/// ⭐ **O fantasma lê-se em todo tema** (`Theme::ALL`, OKLab; ver o `eprintln!`): o contorno tracejado
/// contra o fundo da faixa (o mais perto de `Bg1`, `Bg2`, `BgElev`), o corpo a 22 % contra a faixa vazia,
/// e a recusa contra o fantasma que aceita. *Mutação: o fantasma na cor da faixa ⇒ vermelho.*
#[test]
fn o_fantasma_da_largada_le_se_em_todo_tema() {
    let st = TimelinePanelState {
        source_container: Some(0),
        ..TimelinePanelState::default()
    };
    let snap = TimelineViewSnapshot {
        containers: vec![ContainerView {
            name: "Run & jump".into(),
            length: 2.0,
            uses: 1,
        }],
        ..TimelineViewSnapshot::default()
    };
    let mut pior = [(f64::MAX, Theme::Forge); 3];
    for theme in Theme::ALL {
        let [fundo, borda] = cores_do_fantasma(&st, &snap, false, theme);
        let [_, recusa] = cores_do_fantasma(&st, &snap, true, theme);
        let faixas = [ColorToken::Bg1, ColorToken::Bg2, ColorToken::BgElev]
            .map(|f| ph2d_editor_core::paint::resolve(f, theme));
        let perto = |c: Color| {
            faixas
                .iter()
                .map(|&f| dist(sobre(c, f), f))
                .fold(f64::MAX, f64::min)
        };
        let d = [perto(borda), perto(fundo), dist(recusa, borda)];
        for (p, d) in pior.iter_mut().zip(d) {
            if d < p.0 {
                *p = (d, theme);
            }
        }
    }
    eprintln!(
        "contorno × faixa {:?} · corpo × faixa {:?} · recusa × aceita {:?}",
        pior[0], pior[1], pior[2]
    );
    for (p, piso) in pior.iter().zip([PISO_CONTORNO, PISO_CORPO, PISO_RECUSA]) {
        assert!(p.0 >= piso, "{:.3} < {piso} em {:?}", p.0, p.1);
    }
}

const PISO_CONTORNO: f64 = 0.24; // medido 0,275 (PlumberRed), 10/11
const PISO_CORPO: f64 = 0.045; // medido 0,052 (PlumberGreen), 10/11 — o tracejado é que desenha a borda
const PISO_RECUSA: f64 = 0.08; // medido 0,096 (Dark), 10/11

/// ⭐ Dentro de «Walk» aberto, o chip «Walk» é recusado (ciclo): o fantasma diz não; «Jump» aceita.
/// O documento recusa os ciclos fundos (`try_nest`); o painel só vê o caminho aberto.
#[test]
fn o_container_nao_se_larga_dentro_de_si() {
    use ph2d_timeline::EnterStep;
    crate::state::enter_container(EnterStep {
        container: 0,
        lane: 0,
        strip: None,
    });
    let snap = TimelineViewSnapshot {
        crumbs: vec![(0, "Walk".into())],
        containers: ["Walk", "Jump"]
            .map(|n| ContainerView {
                name: n.into(),
                length: 2.0,
                uses: 1,
            })
            .to_vec(),
        fps: 60.0,
        ..TimelineViewSnapshot::default()
    };
    let rows = Rect::new(100.0, 100.0, 800.0, 300.0);
    let g = geom::Geom {
        region: rows,
        label_w: 0.0,
        time_area: rows,
        rows,
        scrollbar: rows,
    };
    let view = TimeView {
        time_x: 100.0,
        right: 900.0,
        view_start: 0.0,
        px_per_s: 100.0,
    };
    let recusa = |c: usize| {
        let st = TimelinePanelState {
            tab: crate::tab::Tab::Containers,
            source_container: Some(c),
            ..TimelinePanelState::default()
        };
        alvo(&st, &g, view, &snap, (300.0, 110.0)).map(|a| a.recusa)
    };
    let (dentro, outro) = (recusa(0), recusa(1));
    crate::state::pop_to_depth(0);
    assert_eq!(dentro, Some(true), "Walk dentro de Walk");
    assert_eq!(outro, Some(false), "Jump dentro de Walk");
}
