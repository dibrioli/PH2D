//! **B08c.M2** ([medição](../../../../../../docs/Painter/missao_4k/medicoes/ANTES_DEPOIS.md) §M2): a dívida da luz
//! `Cheia` do repouso (B14.2) sai por faixas. A sonda mede o pior tique depois de soltar, A (de uma vez, o caminho
//! antigo) × B (por faixas) no MESMO processo, intercalados; as duas acabam na composição de casa AO BYTE.

use super::super::agua_sobre_oleo_tests::preset;
use super::super::offthread::otica::forca_no_teste;
use super::super::otica_no_tique::faixa_de_cima;
use super::{ponto, tique};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool as _, PointerPhase};
use std::time::{Duration, Instant};

#[test]
fn a_faixa_de_cima_parte_a_caixa_sem_perder_linha() {
    assert_eq!(
        faixa_de_cima((3, 5, 12, 40), 25),
        ((3, 5, 12, 6), Some((3, 7, 12, 40)))
    );
    assert_eq!(
        faixa_de_cima((3, 5, 12, 40), 1),
        ((3, 5, 12, 5), Some((3, 6, 12, 40)))
    );
    assert_eq!(
        faixa_de_cima((3, 5, 12, 40), 10_000),
        ((3, 5, 12, 40), None)
    );
    assert_eq!(
        faixa_de_cima((3, 5, 12, 6), 10),
        ((3, 5, 12, 5), Some((3, 6, 12, 6)))
    );
}

fn blocos(w: u32, h: u32) -> Vec<u8> {
    let cores = [
        [200u8, 60, 50],
        [60, 150, 210],
        [240, 200, 60],
        [90, 170, 90],
    ];
    let mut v = vec![255u8; (w * h * 4) as usize];
    for (i, p) in v.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        let (x, y) = (i as u32 % w, i as u32 / w);
        p[..3].copy_from_slice(&cores[usize::from(x * 2 >= w) + 2 * usize::from(y * 2 >= h)]);
    }
    v
}

/// Um traço de Dabber com Real Oil (o pen-up do Filbert: a mesma óptica), o compositor ligado; devolve o pior
/// tique (ms) do 1 s depois de soltar, se a tela assentada é a composição de casa ao byte e se a dívida acabou nesse
/// 1 s com a água a correr (o motor no worker entre tiques, como no app).
fn solta(w: u32, h: u32, faixa: Option<usize>) -> (f64, bool, bool) {
    solta_com("Oil/Dabber/Dabber.ph2d-brush.png", w, h, faixa, false)
}

fn solta_com(
    pincel: &str,
    w: u32,
    h: u32,
    faixa: Option<usize>,
    um_por_tique: bool,
) -> (f64, bool, bool) {
    // O um-por-tique é o app (o Filbert compõe em casa: `[otica] 0 imagens`): sem compositor.
    forca_no_teste(Some(!um_por_tique));
    let mut t = PainterTool::default();
    t.abre_a_sprite(blocos(w, h), w, h);
    t.set_paint_tool_mode("wetpaint");
    t.preset_aplicar(&preset(pincel))
        .expect("o preset");
    t.set_wet_oleo_real_ligado(true);
    // A luz rasante (a da prancha B14b): a Simples difere da Cheia, senão a régua do repouso não vê a dívida.
    t.paint.wetpaint.oleo_real_valores.altitude = 12f64.to_radians();
    let s = h as f32 / 1080.0;
    t.set_brush_size_px(56.0 * s);
    t.set_brush_color_srgb8([40, 90, 220]);
    let (cx, cy, l) = (w as f32 / 2.0, h as f32 / 2.0, h as f32 / 3.0);
    let passos = 30;
    let p = |k: usize| {
        let f = k as f32 / passos as f32;
        [
            cx - l + 2.0 * l * f,
            cy + 0.6 * l * (std::f32::consts::TAU * f).sin(),
        ]
    };
    let divida;
    let [x, y] = p(0);
    let _ = t.on_canvas_pointer(ponto(x, y, PointerPhase::Down));
    for k in 1..=passos {
        let [x, y] = p(k);
        let _ = t.on_canvas_pointer(ponto(x, y, PointerPhase::Move));
        tique(&mut t);
        std::thread::sleep(Duration::from_millis(16));
    }
    {
        let sess = t.paint.wetpaint.session.as_mut().expect("sessão");
        eprintln!("compositor de pé: {}", sess.tem_compositor());
        assert!(
            sess.otica.simples.is_some(),
            "premissa: o gesto compôs Simples"
        );
        sess.otica.faixa_cheia_px = faixa;
        sess.otica.orcamento_cheia = um_por_tique.then_some(Duration::ZERO);
        divida = sess.otica.simples.expect("a dívida");
        assert_eq!(sess.ratio, 1, "premissa: a célula é o píxel");
    }
    let [x, y] = p(passos);
    let _ = t.on_canvas_pointer(ponto(x, y, PointerPhase::Up));
    let t0 = Instant::now();
    let mut pior = 0f64;
    let mut serie = Vec::new();
    while t0.elapsed() < Duration::from_secs(1) {
        let q = Instant::now();
        tique(&mut t);
        let ms = q.elapsed().as_secs_f64() * 1000.0;
        serie.push(ms.round());
        pior = pior.max(ms);
        std::thread::sleep(Duration::from_millis(16));
    }
    let resta = t
        .paint
        .wetpaint
        .session
        .as_ref()
        .and_then(|s| s.otica.simples);
    if std::env::var_os("SONDA_SERIE").is_some() {
        eprintln!("faixa {faixa:?}: resta {resta:?} tiques {serie:?}");
    }
    // Assenta: a água pára e a tela mostra tudo.
    {
        let s = t.paint.wetpaint.session.as_mut().expect("sessão");
        s.bring_home();
        s.engine.pointer_down(1.0, 1.0, None);
    }
    let t1 = Instant::now();
    loop {
        tique(&mut t);
        let s = t.paint.wetpaint.session.as_ref().expect("sessão");
        if !s.otica.pendente()
            && s.sim_steps() == s.seen_steps
            && t1.elapsed() > Duration::from_millis(64)
        {
            break;
        }
        assert!(t1.elapsed() < Duration::from_secs(20), "não assentou");
        std::thread::sleep(Duration::from_millis(4));
    }
    let vista = t.canvas_rgba.to_vec();
    let s = t.paint.wetpaint.session.as_mut().expect("sessão");
    s.bring_home();
    s.engine.mark_dirty_full();
    t.wetpaint_composite();
    // Só a caixa da dívida (ratio 1: célula c = píxel c − 1, folga de 2): fora dela a luz rasante nunca foi recomposta.
    let casa = t.canvas_rgba.as_slice();
    let (x0, y0, x1, y1) = divida;
    let mut n = 0usize;
    for y in y0 + 1..y1.saturating_sub(3) {
        for x in x0 + 1..x1.saturating_sub(3) {
            let i = (y * w as usize + x) * 4;
            n += usize::from(vista[i..i + 4] != casa[i..i + 4]);
        }
    }
    if n > 0 {
        eprintln!("faixa {faixa:?}: {n} píxeis da caixa da dívida diferem da casa");
    }
    let igual = n == 0;
    forca_no_teste(None);
    (pior, igual, resta.is_none())
}

/// O portão: por faixas a tela do repouso é a de casa ao byte (o A também, a régua do caminho antigo).
#[test]
fn a_luz_cheia_por_faixas_acaba_na_tela_de_casa_ao_byte() {
    for faixa in [Some(usize::MAX), Some(64 * 64)] {
        let (_, igual, _) = solta(640, 360, faixa);
        assert!(
            igual,
            "faixa {faixa:?}: o repouso difere da composição de casa"
        );
    }
}

/// C3 (B08c.M2): a dívida sai inteira em 1 s SEM ninguém trazer o motor para casa — no app o motor vai ao worker
/// em cada tique e a faixa seguinte esperava por um tique que o encontrasse em casa (nunca: o resto ficou Simples).
#[test]
fn a_divida_cheia_acaba_com_a_agua_a_correr() {
    let (_, igual, drenou) =
        solta_com("Oil/Flat/Filbert.ph2d-brush.png", 640, 360, Some(64 * 64), true);
    assert!(igual, "o repouso difere da composição de casa");
    assert!(drenou, "a dívida Cheia não acabou em 1 s com a água a correr");
}

#[test]
#[ignore = "sonda B08c.M2: o pior tique depois de soltar, 4K, A×B intercalados (--profile smoke, sozinha)"]
fn sonda_o_pior_tique_depois_de_soltar() {
    let (w, h) = std::env::var("SONDA_TELA")
        .ok()
        .and_then(|v| {
            v.split_once('x')
                .map(|(a, b)| (a.parse().unwrap(), b.parse().unwrap()))
        })
        .unwrap_or((3840, 2160));
    let mut a = Vec::new();
    let mut b = Vec::new();
    for r in 0..3 {
        for de_uma_vez in if r % 2 == 0 {
            [true, false]
        } else {
            [false, true]
        } {
            let fx = std::env::var("SONDA_FAIXA")
                .ok()
                .and_then(|v| v.parse().ok());
            let (ms, igual, _) = solta(w, h, if de_uma_vez { Some(usize::MAX) } else { fx });
            assert!(igual, "o repouso difere da casa");
            if de_uma_vez { a.push(ms) } else { b.push(ms) }
        }
    }
    let min = |v: &[f64]| v.iter().copied().fold(f64::MAX, f64::min);
    eprintln!(
        "SONDA B08c.M2 {w}x{h}: A de uma vez {a:.1?} (mín {:.1}) × B por faixas {b:.1?} (mín {:.1}) ms",
        min(&a),
        min(&b)
    );
}
