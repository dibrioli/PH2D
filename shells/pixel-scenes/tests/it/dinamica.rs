//! **A dinâmica no app** (P7b): a pressão e o relógio de cada evento chegam pela porta da caneta
//! (`ph2d_pen::porta`), a velocidade e a corda medem-se em px de ECRÃ, e o traço livre carimba ponto
//! a ponto.

use ph2d_app_pixel::compose::Mods;
use ph2d_app_pixel::input::{self, Button, Press};
use ph2d_editor_core::pixel_panel::{
    PixelEditAction, PixelRequest, PixelSelectAction, PixelSensor, PixelThreshold,
};
use ph2d_host::{PenInfo, PointerButton, PointerKind, PointerSource};

use crate::comum::{at, canvas};
use crate::selecao::Palco;

const MS: u128 = 1_000_000;

/// A caneta com a pressão `p` no instante `t` (o que a shell põe na porta antes de despachar).
fn caneta(p: f32, t: u128) {
    let mut e = ph2d_pen::ponteiro::do_rato(0.0, 0.0, PointerKind::Move, PointerButton::Primary, t);
    e.pressure = p;
    e.source = PointerSource::Pen(PenInfo::default());
    ph2d_pen::porta::poe_tinta(&e);
}

fn ecra(p: &Palco, c: (i32, i32)) -> (f32, f32) {
    at(&p.app, &p.sim, &p.cam, p.bits, c.0, c.1)
}

fn premir(p: &mut Palco, s: (f32, f32)) {
    let c = canvas(&p.sim, &p.cam, p.bits);
    let r = input::press(&mut p.app, &c, s, Button::Primary, Mods::default());
    assert_eq!(r, Press::Done);
}

fn mover(p: &mut Palco, s: (f32, f32)) {
    let c = canvas(&p.sim, &p.cam, p.bits);
    assert!(input::moved(&mut p.app, &c, s, Mods::default()));
}

/// A altura pintada na coluna `x` entre as linhas `y0..y1`.
fn altura(p: &Palco, x: i32, ys: std::ops::Range<i32>) -> usize {
    ys.filter(|&y| p.rgba(x, y)[3] != 0).count()
}

fn palco(f: impl FnOnce(&mut ph2d_editor_core::pixel_panel::PixelOptions)) -> Palco {
    let mut p = Palco::limpo();
    // Um sprite vazio: a cena 9 tem desenho nestas linhas.
    p.pede(PixelRequest::Select(PixelSelectAction::All));
    p.pede(PixelRequest::Edit(PixelEditAction::Clear));
    p.pede(PixelRequest::Select(PixelSelectAction::Deselect));
    assert!(
        (0..120).all(|x| (0..80).all(|y| p.rgba(x, y)[3] == 0)),
        "vazio"
    );
    p.ferramenta(ph2d_app_pixel::Tool::Pencil);
    p.opcoes(|o| {
        o.size = 9;
        f(o);
    });
    p
}

#[test]
fn pressure_sets_the_size_through_the_ramp_and_max_is_the_bars_size() {
    let mut p = palco(|o| o.dynamics.size.sensor = PixelSensor::Pressure);
    for (y, pressao, quer) in [(44, 0.0, 1), (54, 1.0, 9)] {
        caneta(pressao, 0);
        let (a, b) = (ecra(&p, (40, y)), ecra(&p, (70, y)));
        premir(&mut p, a);
        mover(&mut p, b);
        p.release();
        assert_eq!(altura(&p, 60, y - 6..y + 6), quer, "pressão {pressao}");
    }
    assert!(altura(&p, 60, 30..40) == 0, "nada fora das duas linhas");
}

#[test]
fn velocity_grows_the_size_with_screen_steps_and_starts_at_min() {
    let mut p = palco(|o| {
        o.dynamics.size.sensor = PixelSensor::Velocity;
        o.dynamics.velocity = PixelThreshold { lo: 0, hi: 150 };
    });
    let (a, b) = (ecra(&p, (10, 50)), ecra(&p, (11, 50)));
    let cel = b.0 - a.0;
    // Passos de 12 px de ecrã a cada 8 ms (pegas 0 / 0,15 = −1 / 8,9 px): depois de 15 eventos o
    // filtro passa de 8,9 px e o tamanho chega ao Max (9).
    caneta(1.0, 0);
    premir(&mut p, a);
    let mut x = a.0;
    for k in 1..=15u128 {
        x += 12.0;
        caneta(1.0, k * 8 * MS);
        mover(&mut p, (x, a.1));
    }
    p.release();
    let col = |s: f32| ((s - a.0) / cel).round() as i32 + 10;
    assert_eq!(altura(&p, 10, 40..60), 1, "o premir carimba o Min");
    let fim = col(x - 12.0 * 2.0);
    assert_eq!(altura(&p, fim, 40..60), 9, "rápido: o Max");
}

#[test]
fn the_stabilizer_string_lags_the_hand_by_eight_screen_px_per_unit() {
    let mut p = palco(|o| {
        o.size = 1;
        o.dynamics.stabilizer = 16;
    });
    let a = ecra(&p, (10, 50));
    let cel = ecra(&p, (11, 50)).0 - a.0;
    caneta(1.0, 0);
    premir(&mut p, a);
    // A mão anda 100 px: dentro da corda de 128 px, nada se mexe.
    mover(&mut p, (a.0 + 100.0, a.1));
    // Mais 156 px (256 no total): a ponta fica 128 px atrás da mão.
    mover(&mut p, (a.0 + 256.0, a.1));
    p.release();
    let ponta = ((128.0 / cel).round()) as i32 + 10;
    assert!(p.rgba(ponta, 50)[3] != 0, "a ponta chegou a mão − 128 px");
    assert_eq!(p.rgba(ponta + 2, 50)[3], 0, "e não passou dela");
    let pintadas = (0..120).filter(|&x| p.rgba(x, 50)[3] != 0).count();
    assert_eq!(
        pintadas as i32,
        ponta - 10 + 1,
        "uma linha do premir até à ponta"
    );
}

/// O Gradient da dinâmica tem a SUA matriz (a do popup *Dynamics*), não a da ferramenta Gradient
/// (o bug 1.3.0 do Aseprite): com a da dinâmica em Bayer2 e a da ferramenta em Bayer8, o carimbo
/// de pressão 0,4 sai com 1 degrau de 4 em fg e os outros em bg.
#[test]
fn the_dynamics_gradient_uses_the_dynamics_own_matrix_not_the_gradient_tools() {
    use ph2d_editor_core::pixel_panel::PixelGradientDither as D;
    use ph2d_pixel_core::dither::Dither;
    let mut p = palco(|o| {
        o.size = 8;
        o.dynamics.gradient.sensor = PixelSensor::Pressure;
        o.dynamics.dither = D::Bayer2;
        o.gradient_dither = D::Bayer8;
    });
    let (fg, bg) = ([255, 0, 0, 255], [0, 0, 255, 255]);
    p.app.fg = fg;
    p.app.bg = bg;
    caneta(0.4, 0);
    // O carimbo do premir leva a fracção cheia (só fg); a matriz mede-se nos pontos seguintes.
    let (a, b) = (ecra(&p, (30, 40)), ecra(&p, (70, 40)));
    premir(&mut p, a);
    mover(&mut p, b);
    p.release();
    let celulas: Vec<(i32, i32)> = (40..120)
        .flat_map(|x| (0..80).map(move |y| (x, y)))
        .filter(|&(x, y)| p.rgba(x, y)[3] != 0)
        .collect();
    assert!(celulas.len() >= 32, "o carimbo pintou: {}", celulas.len());
    let cor = |c: (i32, i32)| p.rgba(c.0, c.1);
    assert!(celulas.iter().any(|&c| cor(c) == fg), "há fg");
    assert!(celulas.iter().any(|&c| cor(c) == bg), "há bg");
    for &(x, y) in &celulas {
        // Fracção 0,4 em Bayer2: 1 degrau aceso (fg), os outros 3 em bg.
        let quer = if Dither::Bayer2.level(x, y) < 1 {
            fg
        } else {
            bg
        };
        assert_eq!(cor((x, y)), quer, "({x}, {y})");
    }
}
