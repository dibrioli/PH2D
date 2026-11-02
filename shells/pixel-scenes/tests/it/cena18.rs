//! **A sonda da cena 18** (P7b): o que a mensagem ao dono promete sai daqui — o desenho já
//! seleccionado, o Ctrl+B, o carimbo com as seis cores, virar, e a largura de um traço lento contra
//! um rápido com o rato (Size ← Velocity, as pegas de fábrica).

use ph2d_app_pixel::compose::Mods;
use ph2d_app_pixel::input::{self, Button, Press};
use ph2d_app_pixel::smoke_pincel::{AT, BRUSH_SIZE, DRAWING};
use ph2d_editor_core::pixel_panel::{
    PixelBrushAction, PixelBrushShape, PixelPatternTurn, PixelRequest, PixelSensor,
};
use ph2d_host::{PointerButton, PointerKind};
use winit::keyboard::KeyCode;

use crate::comum::{at, canvas};
use crate::selecao::Palco;

const MS: u128 = 1_000_000;

/// O relógio do evento do rato (o que a shell põe na porta antes de despachar).
fn rato(t: u128) {
    let e = ph2d_pen::ponteiro::do_rato(0.0, 0.0, PointerKind::Move, PointerButton::Primary, t);
    ph2d_pen::porta::poe_tinta(&e);
}

fn cores_do_desenho() -> Vec<[u8; 4]> {
    DRAWING[0].iter().flatten().copied().collect()
}

/// Um traço horizontal na linha `y`, de `x0` com `n` eventos de `passo` px de ecrã a cada 8 ms.
fn traco(p: &mut Palco, x0: i32, y: i32, passo: f32, n: u128) {
    let a = at(&p.app, &p.sim, &p.cam, p.bits, x0, y);
    let c = canvas(&p.sim, &p.cam, p.bits);
    rato(0);
    assert_eq!(
        input::press(&mut p.app, &c, a, Button::Primary, Mods::default()),
        Press::Done
    );
    for k in 1..=n {
        rato(k * 8 * MS);
        let s = (a.0 + passo * k as f32, a.1);
        assert!(input::moved(&mut p.app, &c, s, Mods::default()));
    }
    p.release();
}

/// Fundo: transparente (o branco da cena é o do fundo, não das células) ou branco.
fn vazia(c: [u8; 4]) -> bool {
    c[3] == 0 || c == [255; 4]
}

/// Um clique (premir e largar) na célula `(x, y)`.
fn clique(p: &mut Palco, x: i32, y: i32) {
    let s = at(&p.app, &p.sim, &p.cam, p.bits, x, y);
    let c = canvas(&p.sim, &p.cam, p.bits);
    rato(0);
    let _ = input::press(&mut p.app, &c, s, Button::Primary, Mods::default());
    p.release();
}

/// A altura pintada (não branca) na coluna `x` em volta da linha `y`.
fn altura(p: &Palco, x: i32, y: i32) -> usize {
    (y - 9..y + 10).filter(|&yy| !vazia(p.rgba(x, yy))).count()
}

#[test]
fn scene_18_selected_drawing_ctrl_b_stamps_flips_and_slow_vs_fast() {
    let mut p = Palco::cena(18);
    assert!(p.mascara().is_some(), "o desenho nasce seleccionado");
    let o = p.app.options;
    assert_eq!(o.size, BRUSH_SIZE);
    assert_eq!(o.dynamics.size.sensor, PixelSensor::Velocity);
    assert_eq!(p.rgba(AT.0, AT.1), cores_do_desenho()[0]);
    assert_eq!(p.rgba(90, 60), [255; 4], "o fundo é branco até ao canto");
    p.tecla(KeyCode::KeyB, true, false);
    assert!(p.mascara().is_none(), "o Ctrl+B larga a selecção");
    let i = p.quadro();
    assert_eq!(i.pattern, Some((6, 4)));
    // Um clique carimba as seis cores (Source: alinhadas à grelha de onde vieram).
    clique(&mut p, 80, 40);
    let vistas: Vec<[u8; 4]> = (74..86)
        .flat_map(|x| (36..44).map(move |y| (x, y)))
        .map(|(x, y)| p.rgba(x, y))
        .collect();
    for cor in cores_do_desenho() {
        assert!(vistas.contains(&cor), "a cor {cor:?} carimbada");
    }
    // Virar: a linha do carimbo inverte-se.
    let linha = |p: &Palco, y: i32| -> Vec<[u8; 4]> {
        (70..92)
            .map(|x| p.rgba(x, y))
            .filter(|c| !vazia(*c))
            .collect()
    };
    p.pede(PixelRequest::Brush(PixelBrushAction::Turn(
        PixelPatternTurn::FlipH,
    )));
    clique(&mut p, 80, 54);
    let mut virada = linha(&p, 54);
    virada.reverse();
    let direita: Vec<_> = linha(&p, 40);
    assert_eq!(virada.len(), 6);
    assert_eq!(
        virada.iter().collect::<std::collections::BTreeSet<_>>(),
        direita.iter().collect::<std::collections::BTreeSet<_>>()
    );
    // De volta ao círculo: lento sai fino, rápido sai grosso.
    p.pede(PixelRequest::Brush(PixelBrushAction::Shape(
        PixelBrushShape::Circle,
    )));
    let cel =
        at(&p.app, &p.sim, &p.cam, p.bits, 11, 0).0 - at(&p.app, &p.sim, &p.cam, p.bits, 10, 0).0;
    traco(&mut p, 4, 20, 2.0, 40);
    traco(&mut p, 4, 50, 24.0, 20);
    let lentas: Vec<usize> = (4..20).map(|x| altura(&p, x, 20)).collect();
    let rapidas: Vec<usize> = (4..72).step_by(4).map(|x| altura(&p, x, 50)).collect();
    eprintln!("[sonda cena 18] lento por coluna {lentas:?} · rápido de 4 em 4 colunas {rapidas:?}");
    let lento = lentas.iter().copied().max().unwrap_or(0);
    let rapido = rapidas.iter().copied().max().unwrap_or(0);
    eprintln!(
        "[sonda cena 18] célula = {cel:.1} px de ecrã · lento (2 px/evento) = {lento} · rápido (24 px/evento) = {rapido}"
    );
    // 2 px por evento com as pegas 0/300: f = 3/19,8 ⇒ 1 + 0,15·15 ≈ 3.
    assert!(lento <= 3, "lento: {lento}");
    assert_eq!(rapido, BRUSH_SIZE as usize, "rápido: o Max");
}
