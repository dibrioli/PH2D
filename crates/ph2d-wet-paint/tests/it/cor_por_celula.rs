//! ⭐ **A COR POR CÉLULA no depósito** (prompt 41, a Shape Color do hospedeiro no Wet Paint): o dab
//! pousa a cor de cada célula na janela, misturada pela massa; o transfer pousa-a com a sujidade da
//! ponta (`limpo · tinta + sujidade`); o alfa pesa o depósito — e as duas rotas pousam a mesma cor.

use ph2d_wet_paint::brush::ColorFn;
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::par::Rows;
use ph2d_wet_paint::trail::{Dab, Trail, TrailMode};
use ph2d_wet_paint::tuning::Knob;

const BASE: [f64; 3] = [40.0, 90.0, 200.0];
const VERMELHO: [f32; 4] = [220.0, 30.0, 20.0, 1.0];
const AZUL: [f32; 4] = [20.0, 40.0, 230.0, 1.0];

fn dab(x: f64, y: f64, r: f64) -> Dab {
    Dab {
        x,
        y,
        r,
        hardness: 0.5,
        intensity: 40.0,
        water_amount: 0.5,
        dry_gate: 0.2,
        reach: 1.0,
        window_reach: 1.0,
        opacity: 1.0,
    }
}

/// Uma silhueta de disco suave de raio `r` em `(cx, cy)`.
fn disco(cx: f64, cy: f64, r: f64) -> impl Fn(i32, i32) -> f64 + Sync {
    move |x, y| {
        let (dx, dy) = (f64::from(x) - cx, f64::from(y) - cy);
        let d = (dx * dx + dy * dy).sqrt() / r;
        if d >= 1.0 { 0.0 } else { 1.0 - d * d }
    }
}

/// O motor seco e o rasto aberto, com a janela a encher a cada DOIS dabs.
fn bancada(clean: f64) -> (Engine, Trail) {
    let mut e = Engine::new(200, 200);
    e.tuning.set(Knob::Pickup, 0.0);
    e.tuning.set(Knob::TipClean, clean);
    let mut t = Trail::default();
    t.start_stroke(100.0, 100.0, BASE, TrailMode::Paint);
    t.on_segment(12.0, 12.0);
    (e, t)
}

/// O dab de raio 20 em `(100, 100)` com a cor constante `c`; devolve se a janela encheu.
fn pousa(e: &mut Engine, t: &mut Trail, c: [f32; 4], rota: Option<Rows>) -> bool {
    let p = e.sim.gather_params(&e.tuning);
    let tex = e.bristle_texture_for_measure();
    let sil = disco(100.0, 100.0, 20.0);
    let cor = move |_: i32, _: i32, _: f64| c;
    let g = e.active_grid_mut();
    let d = dab(100.0, 100.0, 20.0);
    t.accumulate_paint_shaped_cor(g, &p, &tex, &d, false, &sil, None, None, Some(&cor), rota)
        .window_full
}

/// A tinta da janela e o pigmento no centro do dab.
fn centro(t: &Trail) -> ([f32; 3], f32) {
    let (ax, ay) = t.anchor_for_measure();
    let half = t.window_half_for_measure();
    let (lx, ly) = (100 - ax + half, 100 - ay + half);
    let size = (half * 2 + 1) as usize;
    let pig = t.planes_for_measure().0[lx as usize + ly as usize * size];
    (
        t.tinta_rgb_for_measure(lx, ly).expect("tinta na janela"),
        pig,
    )
}

/// **A 1.ª tinta pousa inteira na janela; a 2.ª mistura-se pela massa** (`soma ÷ (antes + soma)`,
/// pela mistura do motor) — dois dabs da mesma janela no mesmo sítio, vermelho e azul.
///
/// **Mutação que sangra:** `pousa_a_tinta` sempre a sobrescrever (a tinta sai AZUL).
#[test]
fn a_primeira_tinta_carrega_a_ponta_e_a_segunda_mistura_pela_massa() {
    let (mut e, mut t) = bancada(0.0);
    assert!(
        !pousa(&mut e, &mut t, VERMELHO, None),
        "a janela leva dois dabs"
    );
    let (c1, pig1) = centro(&t);
    assert_eq!(c1, [VERMELHO[0], VERMELHO[1], VERMELHO[2]]);
    pousa(&mut e, &mut t, AZUL, None);
    let (c2, pig2) = centro(&t);
    let w = f64::from(pig2 - pig1) / f64::from(pig2);
    assert!(
        (0.3..0.7).contains(&w),
        "os dois dabs pesam parecido: w {w:.3}"
    );
    let mut esperado = [0.0f64; 3];
    let v = VERMELHO.map(f64::from);
    let a = AZUL.map(f64::from);
    e.sim
        .gather_params(&e.tuning)
        .mix
        .mix(v[0], v[1], v[2], a[0], a[1], a[2], w, &mut esperado);
    for k in 0..3 {
        assert!(
            (f64::from(c2[k]) - esperado[k]).abs() < 0.5,
            "canal {k}: a ponta {c2:?}, a mistura pela massa {esperado:?}"
        );
    }
}

/// **A tinta por célula leva a sujidade da ponta, exacta** — a limpeza e a recolha são interpolações
/// para alvos fixos, então uma cor por célula CONSTANTE igual à de um pincel sem ela pousa a MESMA tinta
/// suja, por duas janelas (a sujidade da 1.ª viaja na ponta para a 2.ª), sobre tinta azul suspensa.
///
/// **Mutações que sangram:** pousar a tinta limpa sem a sujidade; não actualizar o `limpo` na
/// recolha; não o actualizar na limpeza.
#[test]
fn a_tinta_por_celula_leva_a_sujidade_exacta_da_ponta() {
    let corre = |com_cor: bool| {
        let mut e = Engine::new(200, 200);
        e.tuning.set(Knob::Pickup, 0.5);
        e.tuning.set(Knob::TipClean, 0.1);
        let g = e.active_grid_mut();
        for i in 0..g.susp.len() {
            g.susp[i] = 400.0;
            g.susp_rgb[i] = [20.0, 30.0, 230.0];
        }
        let base = if com_cor {
            BASE
        } else {
            VERMELHO.map(f64::from)[..3].try_into().unwrap()
        };
        let mut t = Trail::default();
        t.start_stroke(90.0, 100.0, base, TrailMode::Paint);
        t.on_segment(12.0, 12.0);
        let p = e.sim.gather_params(&e.tuning);
        let tex = e.bristle_texture_for_measure();
        let cor = |_: i32, _: i32, _: f64| VERMELHO;
        let mut pousos = 0;
        for x in [90.0, 94.0, 98.0, 102.0] {
            let sil = disco(x, 100.0, 20.0);
            let d = dab(x, 100.0, 20.0);
            let g = e.active_grid_mut();
            let a = t.accumulate_paint_shaped_cor(
                g,
                &p,
                &tex,
                &d,
                false,
                &sil,
                None,
                None,
                com_cor.then_some(&cor as ColorFn<'_>),
                None,
            );
            if a.window_full {
                t.transfer_paint(g, &p);
                pousos += 1;
            }
        }
        assert_eq!(pousos, 2, "duas janelas");
        let g = e.active_grid();
        g.susp_rgb[100 + 100 * g.s]
    };
    let (sem, com) = (corre(false), corre(true));
    let v = [VERMELHO[0], VERMELHO[1], VERMELHO[2]];
    assert!(
        sem.iter().zip(v.iter()).any(|(a, b)| (a - b).abs() > 5.0),
        "a fixtura suja o pincel: {sem:?}"
    );
    for k in 0..3 {
        assert!(
            (sem[k] - com[k]).abs() < 1e-3,
            "canal {k}: sem cor por célula {sem:?}, com ela {com:?}"
        );
    }
}

/// **O alfa da tinta pesa o depósito** — a cor de alfa `0,5` pousa metade do pigmento da de alfa `1`.
///
/// **Mutação que sangra:** tirar o `deposit *= c[3]` do depósito.
#[test]
fn o_alfa_da_tinta_pesa_o_deposito() {
    let pig = |a: f32| {
        let (mut e, mut t) = bancada(0.0);
        pousa(
            &mut e,
            &mut t,
            [VERMELHO[0], VERMELHO[1], VERMELHO[2], a],
            None,
        );
        centro(&t).1
    };
    let (cheio, meio) = (pig(1.0), pig(0.5));
    assert!(cheio > 0.0);
    assert!(
        (f64::from(meio) / f64::from(cheio) - 0.5).abs() < 1e-5,
        "alfa 0,5: {meio} contra {cheio}"
    );
}

/// **As duas rotas do depósito pousam a mesma cor ao bit** (ADR-0175: cada célula da ponta é escrita
/// só na própria linha) — um dab acima do piso, com a cor a variar por célula, e o transfer.
#[test]
fn as_duas_rotas_pousam_a_mesma_cor_ao_bit() {
    let corre = |rota: Rows| {
        let mut e = Engine::new(700, 700);
        e.tuning.set(Knob::Pickup, 0.2);
        e.tuning.set(Knob::TipClean, 0.02);
        let mut t = Trail::default();
        t.start_stroke(350.0, 350.0, BASE, TrailMode::Paint);
        t.on_segment(12.0, 12.0);
        let p = e.sim.gather_params(&e.tuning);
        let tex = e.bristle_texture_for_measure();
        let g = e.active_grid_mut();
        let cor = |x: i32, y: i32, q: f64| -> [f32; 4] {
            [(x % 256) as f32, (y % 256) as f32, (q * 255.0) as f32, 1.0]
        };
        let cor: ColorFn<'_> = &cor;
        for (cx, cy) in [(340.0, 350.0), (360.0, 352.0)] {
            let sil = disco(cx, cy, 230.0);
            let d = dab(cx, cy, 230.0);
            let a = t.accumulate_paint_shaped_cor(
                g,
                &p,
                &tex,
                &d,
                false,
                &sil,
                None,
                None,
                Some(cor),
                Some(rota),
            );
            if a.window_full {
                t.transfer_paint_rows(g, &p, rota);
            }
        }
        let (r, gg, b) = t.tip_planes_for_measure();
        let bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
        let rgb: Vec<u32> = g.susp_rgb.iter().flatten().map(|x| x.to_bits()).collect();
        (bits(r), bits(gg), bits(b), rgb, bits(&g.susp))
    };
    let (s, p) = (corre(Rows::Serial), corre(Rows::Parallel));
    assert!(s.4.iter().any(|&x| x != 0), "a tinta pousou");
    assert!(s == p, "a rota paralela pousou outra cor");
}

/// **Um traço SEM cor depois de um COM cor pousa a ponta, não a tinta velha** — a marca da cor por célula
/// é da janela (o artista desliga a Shape Color entre dois traços).
///
/// **Mutação que sangra:** não apagar `tinta_na_janela` no `roll_window`.
#[test]
fn o_traco_sem_cor_depois_de_um_com_cor_pousa_a_ponta() {
    let (mut e, mut t) = bancada(0.0);
    pousa(&mut e, &mut t, VERMELHO, None);
    assert!(pousa(&mut e, &mut t, VERMELHO, None));
    let p = e.sim.gather_params(&e.tuning);
    t.transfer_paint(e.active_grid_mut(), &p);
    t.drop_remainder();
    let mut e = Engine::new(200, 200);
    e.tuning.set(Knob::Pickup, 0.0);
    e.tuning.set(Knob::TipClean, 0.0);
    t.start_stroke(100.0, 100.0, BASE, TrailMode::Paint);
    t.on_segment(12.0, 12.0);
    let p = e.sim.gather_params(&e.tuning);
    let tex = e.bristle_texture_for_measure();
    let sil = disco(100.0, 100.0, 20.0);
    for _ in 0..2 {
        let d = dab(100.0, 100.0, 20.0);
        let g = e.active_grid_mut();
        if t.accumulate_paint_shaped_cor(g, &p, &tex, &d, false, &sil, None, None, None, None)
            .window_full
        {
            t.transfer_paint(g, &p);
        }
    }
    let g = e.active_grid();
    let i = 100 + 100 * g.s;
    assert!(g.susp[i] > 0.0);
    assert_eq!(
        g.susp_rgb[i],
        BASE.map(|c| c as f32),
        "pousou a tinta do traço anterior"
    );
}
