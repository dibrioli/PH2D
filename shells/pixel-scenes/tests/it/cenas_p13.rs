//! A sonda das cenas 28–36 (P13 §8): cada cena montada headless pelo roteador, e o que a foto
//! tem de mostrar MEDIDO no documento. Os números vão ao plano (§12 «Cenas 28–36»).

use std::collections::BTreeSet;

use ph2d_app_pixel::{PixelApp, mode, panel};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_pixel_cenas::p13;
use ph2d_pixel_core::Sprite;
use ph2d_render::Camera2d;

use crate::comum::{PPM, WINDOW, spawn_root};

/// Os números vão a `target/sonda_p13.txt` (o roteiro de testes engole o stdout).
macro_rules! println {
    ($($t:tt)*) => {{
        use std::io::Write;
        let p = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/sonda_p13.txt");
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(p).expect("sonda");
        writeln!(f, $($t)*).expect("escreve");
    }};
}

fn monta(n: u32) -> (PixelApp, SimWorld, u32) {
    let mut app = PixelApp::default();
    let mut sim = SimWorld::new();
    assert!(
        ph2d_app_pixel::smoke::stage(&mut app, &mut sim, n, spawn_root),
        "cena {n}"
    );
    let (bits, key) = mode::sprites(sim.world())[0];
    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    hero.gizmo.mode.enter(bits, ObjectMode::Draw);
    let mut cam = Camera2d::default();
    app.frame(&mut sim, &mut cam, WINDOW, PPM, Some(bits), spawn_root);
    let _ = panel::frame(&mut app, sim.world(), &mut hero, &mut ToastQueue::new());
    (app, sim, key)
}

fn sprite(app: &PixelApp, key: u32) -> &Sprite {
    &app.doc(key).expect("doc").sprite
}

fn rgba(s: &Sprite, f: usize) -> Vec<u8> {
    let mut out = Vec::new();
    ph2d_pixel_core::compose_rgba(s, s.frames()[f].id, None, &mut out);
    out
}

fn cores(px: &[u8]) -> BTreeSet<[u8; 4]> {
    px.chunks_exact(4)
        .filter(|c| c[3] > 0)
        .map(|c| [c[0], c[1], c[2], c[3]])
        .collect()
}

#[test]
fn a_sonda_das_cenas_28_a_36() {
    // 28: as três rampas, a água a ciclar.
    let (app, _sim, k) = monta(28);
    let s = sprite(&app, k);
    let r = s.ramps();
    let tamanhos: Vec<usize> = r.ramps.iter().map(|r| r.indices.len()).collect();
    assert_eq!(tamanhos, [5, 4, 6]);
    assert_eq!(r.cycles.len(), 1);
    assert_eq!(s.mode(), ph2d_pixel_core::ColorMode::Indexed);
    let p0 = ph2d_pixel_core::cycle::ciclo_em(r, 0);
    let p1 = ph2d_pixel_core::cycle::ciclo_em(r, u64::from(p13::CICLO_28_MS / 4 + 1));
    let mexem = (0..256).filter(|&i| p0[i] != p1[i]).count();
    println!(
        "28: rampas {tamanhos:?}, ciclo {} ms, índices que trocam num passo: {mexem}",
        r.cycles[0].periodo_ms
    );
    assert_eq!(mexem, 4);

    // 29: Exact RGB × Keep ramps, lado a lado.
    let (a, b, pal) = p13::foto_lado_a_lado().expect("foto");
    let dif = a
        .chunks_exact(4)
        .zip(b.chunks_exact(4))
        .filter(|(x, y)| x != y)
        .count();
    let (ca, cb) = (cores(&a), cores(&b));
    println!(
        "29: paleta {} cores; Exact RGB usa {}, Keep ramps usa {}; {dif} de 4096 px diferem",
        pal.len(),
        ca.len(),
        cb.len()
    );
    assert!(dif > 0);
    let (app, _sim, k) = monta(29);
    assert_eq!(sprite(&app, k).width(), 2 * p13::FOTO_29 + p13::VAO_29);
    assert_eq!(sprite(&app, k).ramps().ramps.len(), 3);

    // 30: os três níveis do dither e o Push na mão.
    let (app, _sim, k) = monta(30);
    let px = rgba(sprite(&app, k), 0);
    let w = sprite(&app, k).width() as usize;
    let h = sprite(&app, k).height();
    let faixa = h / 4;
    let mut fr = Vec::new();
    for i in 0..3u32 {
        let y0 = (faixa / 2 + i * faixa + 2) as usize;
        let n = (y0..y0 + (faixa - 4) as usize)
            .flat_map(|y| px[y * w * 4 + 16..(y * w + w - 4) * 4].chunks_exact(4))
            .filter(|c| c[..3] == ph2d_pixel_core::palette::DEFAULT[1][..3])
            .count();
        fr.push(n);
    }
    println!(
        "30: na linha do meio de cada faixa, células da cor primária por faixa: {fr:?} de {} cada; ferramenta {:?}",
        (w - 8) * (faixa as usize - 4),
        app.tool
    );
    assert!(fr[0] < fr[1] && fr[1] < fr[2]);
    assert_eq!(app.tool, ph2d_app_pixel::Tool::Push);

    assert!(p13::cena(31).is_none(), "a 31 espera rampas no herói");

    // 32: as guias no documento.
    let (app, _sim, k) = monta(32);
    let g = sprite(&app, k).guias();
    println!("32: guias {g:?}");
    assert!(g.pontos_de_fuga[0].is_some() && g.pontos_de_fuga[1].is_some() && g.iso.is_some());

    // 33: os vícios que o lint acha.
    let (app, _sim, k) = monta(33);
    let s = sprite(&app, k);
    let avisos = ph2d_pixel_lint::lint(s, s.frames()[0].id, &ph2d_pixel_lint::Regras::default());
    let grupos: BTreeSet<_> = avisos.iter().map(|a| a.regra).collect();
    println!(
        "33: {} avisos em {} grupos: {grupos:?}",
        avisos.len(),
        grupos.len()
    );
    assert_eq!(grupos.len(), 5, "o 6.º (cores a mais) só com Color limit");

    // 35: o Outline ligado na camada.
    let (app, _sim, k) = monta(35);
    let s = sprite(&app, k);
    let ef = s.layers()[0].efeitos.len();
    let com = cores(&rgba(s, 0));
    println!("35: efeitos na camada {ef}; cores compostas {com:?}");
    assert_eq!(ef, 1);
    assert!(com.contains(&p13::CONTORNO_35));

    // 36: o Tween e o som.
    let (app, _sim, k) = monta(36);
    let s = sprite(&app, k);
    let n = s.frames().len();
    let wav = std::fs::metadata(p13::fala_36())
        .map(|m| m.len())
        .unwrap_or(0);
    let xs: Vec<u32> = (0..n)
        .map(|f| {
            let px = rgba(s, f);
            let xs: Vec<u32> = px
                .chunks_exact(4)
                .enumerate()
                .filter(|(_, c)| c[3] > 0)
                .map(|(i, _)| i as u32 % s.width())
                .collect();
            xs.iter().sum::<u32>() / xs.len().max(1) as u32
        })
        .collect();
    println!(
        "36: {n} quadros, centro x por quadro {xs:?}; faixas de som {}; wav {wav} bytes",
        s.audio().len()
    );
    assert_eq!(n as u32, 2 + p13::TWEEN_36);
    assert_eq!(s.audio().len(), 1);
}
