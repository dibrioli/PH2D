//! **A Shape Color no pincel salvo** (prompt 45, B13 do dono): as rampas viajam no preset por NOME
//! ([`EstadoDaShapeColor::para_json`], a mesma lei da macro), e um pincel sem elas TIRA a do anterior.

use super::super::shape_ramp::EstadoDaShapeColor;
use super::tests::{ferramenta, pelo_ficheiro, pinta};
use super::*;
use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
use ph2d_painter_brush::BrushBlend;

/// Cada campo da Shape Color mexido do de fábrica, sobre um Shape com textura (a Texture Color pinta).
fn com_a_shape_color(t: &mut PainterTool) {
    t.set_brush_shape_kind(TextureKind::Voronoi.to_u8());
    t.set_shape_color_ramp(ColorRamp::new(
        vec![
            RampStop::new(0.0, [0.0, 1.0, 0.0, 1.0]),
            RampStop::new(1.0, [1.0, 0.0, 0.0, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    ));
    t.set_shape_falloff_ramp(ColorRamp::new(
        vec![
            RampStop::new(0.0, [1.0, 1.0, 0.0, 1.0]),
            RampStop::new(0.6, [0.0, 0.0, 1.0, 1.0]),
        ],
        RampColorMode::Hsv,
        RampInterp::Constant,
    ));
    t.set_shape_ramp_enabled(true);
    t.toggle_shape_ramp_bw();
    t.toggle_shape_texture_color();
    t.toggle_shape_falloff_color();
    t.swap_shape_color_layers();
    t.set_shape_color_blend(BrushBlend::Multiply as u8);
    t.set_shape_texture_opacity(0.7);
    t.set_shape_falloff_opacity(0.6);
}

/// O preset de `t`, gravado em ficheiro e relido.
fn salvo(t: &PainterTool) -> Preset {
    let (prm, texturas) =
        t.wet_preset_capturar(&Parametros::novo(MEIO_DO_PRESET, "Teste", "Rampa"));
    pelo_ficheiro(prm, texturas)
}

/// ⭐ A Shape Color vai e volta pelo JSON — e CADA campo dela difere do de fábrica (o controlo: uma
/// conversão em falta passaria calada pelo valor de fábrica).
#[test]
fn a_shape_color_vai_e_volta_pelo_json() {
    let mut t = ferramenta();
    com_a_shape_color(&mut t);
    let v = t.shape_color_estado().para_json();
    let fabrica = EstadoDaShapeColor::default().para_json();
    for (k, x) in v.as_object().unwrap() {
        assert_ne!(
            Some(x),
            fabrica.get(k),
            "o campo {k} não foi mexido pela fixtura"
        );
    }
    let mut e = EstadoDaShapeColor::default();
    e.ler_json(&v);
    assert_eq!(e.para_json(), v);
}

/// ⭐ **O pincel gravado pinta o mesmo traço COM a rampa** — o de outro tool com o preset aplicado, ao byte.
/// O controlo: o mesmo preset sem a chave pinta outro traço.
#[test]
fn o_pincel_gravado_pinta_a_shape_color() {
    let mut a = ferramenta();
    com_a_shape_color(&mut a);
    // A fixtura desliga as duas camadas (cada campo mexido); a Falloff Color volta para a rampa pintar.
    a.toggle_shape_falloff_color();
    let p = salvo(&a);
    assert!(
        p.parametros.shape_color.is_some(),
        "o preset não levou a rampa"
    );
    let mut b = ferramenta();
    assert!(b.wet_preset_aplicar(&p).is_empty());
    let mut sem = p.clone();
    sem.parametros.shape_color = None;
    let mut c = ferramenta();
    assert!(c.wet_preset_aplicar(&sem).is_empty());
    let (pa, pb, pc) = (pinta(&mut a), pinta(&mut b), pinta(&mut c));
    assert!(pa == pb, "o pincel gravado não pinta a rampa que tinha");
    assert!(pa != pc, "controlo: sem a rampa o traço tem de mudar");
}

/// ⭐ **Um pincel SEM rampa tira a do anterior** (a regra dos campos ausentes do P44): o estado volta ao de
/// fábrica e o traço é o de uma ferramenta nova com o mesmo pincel.
#[test]
fn um_pincel_sem_rampa_tira_a_do_anterior() {
    let p = salvo(&ferramenta());
    assert_eq!(
        p.parametros.shape_color, None,
        "um pincel sem rampa grava a chave"
    );
    let mut a = ferramenta();
    com_a_shape_color(&mut a);
    assert!(a.wet_preset_aplicar(&p).is_empty());
    assert_eq!(
        a.shape_color_estado().para_json(),
        EstadoDaShapeColor::default().para_json()
    );
    let mut b = ferramenta();
    assert!(b.wet_preset_aplicar(&p).is_empty());
    assert!(pinta(&mut a) == pinta(&mut b));
}

/// Uma macro gravada ANTES do prompt 45 trazia a Shape Color FORA do preset (`shape_color` no topo): ela
/// continua a chegar.
#[test]
fn uma_macro_de_antes_do_45_traz_a_shape_color() {
    let mut a = ferramenta();
    com_a_shape_color(&mut a);
    let (texto, texturas) = a.macro_estado();
    let mut v: serde_json::Value = serde_json::from_str(&texto).unwrap();
    let sc = v["preset"]
        .as_object_mut()
        .unwrap()
        .remove("shape_color")
        .expect("o preset da macro leva a rampa");
    v["shape_color"] = sc;
    let mut b = ferramenta();
    b.macro_estado_aplica(&v.to_string(), texturas).unwrap();
    assert_eq!(
        b.shape_color_estado().para_json(),
        a.shape_color_estado().para_json()
    );
}
