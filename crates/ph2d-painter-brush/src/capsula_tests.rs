//! A lei do dab-cápsula (doc 31 §5) — as peças, contra referências escritas à parte.

use super::*;

/// `n = ⌈máx|D| / sub⌉`, nunca abaixo de 2 nem acima de [`MAX_SUB`]; `off_k = −D·k/n` (o 0 é o próprio
/// dab) e a caixa cresce só para TRÁS.
#[test]
fn a_capsula_reparte_o_passo_para_tras() {
    assert_eq!(Capsula::new([0.0, 0.0], 1.0), None);
    assert_eq!(
        Capsula::new([0.9, 0.3], 1.0),
        None,
        "um sub-passo: o próprio dab"
    );
    let c = Capsula::new([10.0, -4.0], 1.0).expect("cápsula");
    assert_eq!(c.n(), 10);
    assert_eq!(c.off(0), [0.0, 0.0]);
    assert_eq!(c.off(5), [-5.0, 2.0]);
    assert_eq!(c.caixa(), ([-9.0, 0.0], [0.0, 3.6]));
    assert_eq!(
        Capsula::new([1000.0, 0.0], 1.0).map(Capsula::n),
        Some(MAX_SUB)
    );
    assert_eq!(Capsula::new([f32::NAN, 0.0], 1.0), None);
}

/// A **União** é a composição de `n` passagens da mesma cor: `1 − Π(1 − c·x_k)`, sobre `c`; com `c → 0`
/// a soma (o regime linear). A Média divide por `n`; as leis do prompt 32 lêem a queda do CENTRO.
#[test]
fn as_leis_da_junta() {
    let xs = [(0.5f32, 1.0f32, 0.5f32), (0.2, 0.4, 0.5), (0.0, 0.0, 0.9)];
    let junta = |lei, c| {
        let mut j = Junta::new(lei, c);
        for (x, q, f) in xs {
            j.mais(x, q, f);
        }
        j.fim()
    };
    let c = 0.6f32;
    let uniao = (1.0 - (1.0 - c * 0.5) * (1.0 - c * 0.2) * (1.0 - c * 0.0)) / c;
    assert!((junta(Lei::Uniao, c) - uniao).abs() < 1e-6);
    assert!((junta(Lei::Uniao, 0.0) - 0.7).abs() < 1e-6);
    assert!((junta(Lei::Media, c) - 0.7 / 3.0).abs() < 1e-6);
    assert!((junta(Lei::Centro, c) - 1.0 * (0.5 + 0.5 + 0.9) / 3.0).abs() < 1e-6);
    assert!((junta(Lei::CentroMax, c) - 0.9).abs() < 1e-6);
    assert_eq!(Junta::new(Lei::Uniao, c).fim(), 0.0);
}

/// O produto corre a União, e `com_lei` repõe a lei e o sub-passo ao sair.
#[test]
fn o_produto_corre_a_uniao() {
    assert_eq!(lei(), Lei::Uniao);
    assert_eq!((sub_passo_px(10.0), sub_passo_px(40.0)), (1.0, 2.0));
    com_lei(Lei::Media, Some(0.1), || {
        assert_eq!((lei(), sub_passo_px(40.0)), (Lei::Media, 8.0));
    });
    assert_eq!((lei(), sub_passo_px(40.0)), (Lei::Uniao, 2.0));
}

/// Os sub-dabs são o traço denso: do mais antigo (`k = n − 1`, junto ao centro anterior) ao próprio dab,
/// sem passo; sem cápsula, o dab tal qual.
#[test]
fn os_sub_dabs_vao_do_centro_anterior_ao_dab() {
    let d = crate::Dab {
        center: [50.0, 20.0],
        radius_px: 20.0,
        coverage: 1.0,
        color: [0.0; 3],
        color_shift: [0.0; 3],
        rotation: [1.0, 0.0],
        dir: [1.0, 0.0],
        arc_len: 30.0,
        stroke_radius_px: 20.0,
        step: [4.0, 0.0],
        ante: None,
    };
    let s: Vec<_> = sub_dabs(&d, 1.0)
        .map(|s| (s.center, s.arc_len, s.step))
        .collect();
    assert_eq!(
        s,
        vec![
            ([47.0, 20.0], 27.0, [0.0, 0.0]),
            ([48.0, 20.0], 28.0, [0.0, 0.0]),
            ([49.0, 20.0], 29.0, [0.0, 0.0]),
            ([50.0, 20.0], 30.0, [0.0, 0.0]),
        ]
    );
    let liso = crate::Dab {
        step: [0.0, 0.0],
        ante: None,
        ..d
    };
    assert_eq!(sub_dabs(&liso, 1.0).count(), 1);
}

fn cerdas() -> crate::TextureSettings {
    let mut p = [0.5; crate::texture::MAX_TEX_PARAMS];
    for (i, s) in crate::texture::param_specs(crate::TextureKind::Bristles)
        .iter()
        .enumerate()
    {
        p[i] = s.default;
    }
    crate::TextureSettings {
        kind: crate::TextureKind::Bristles,
        params: p,
        ..crate::TextureSettings::default()
    }
}

/// ⭐ **A silhueta da cápsula é a dos sub-dabs, AO BIT** — `(1 − Π(1 − c·x_k)) / c` com `x_k` a
/// [`crate::dab::silhouette_at`] do dab centrado em `centro + off_k` (a queda, a pegada e o Shape juntos),
/// pela referência escrita aqui; com a redonda e com as cerdas. CONTROLO: a lei do prompt 32 (a queda do
/// CENTRO) muda centenas de píxeis.
#[test]
fn a_silhueta_da_capsula_e_a_dos_sub_dabs() {
    use crate::texture::{ShapeFrame, shape_basis};
    let c = 0.7f32;
    let centro = [100.25f32, 60.5];
    let cap = Capsula::new([9.0, 3.0], 1.0).expect("cápsula");
    for forma in [crate::TextureSettings::default(), cerdas()] {
        let spec = crate::BrushSpec {
            radius_px: 20.0,
            shape: forma,
            ..crate::BrushSpec::default()
        };
        let fp = spec.dab_footprint([1.0, 0.0]);
        let sb = shape_basis(
            &spec.shape,
            &mut 0u64,
            [256.0, 128.0],
            fp,
            ShapeFrame::Stroke {
                arc_len: 0.0,
                unit_px: 20.0,
            },
        );
        let shape = spec.shape.is_active().then_some(crate::ShapeInput {
            basis: &sb,
            image: None,
            ramp_lut: None,
        });
        let (mut vistos, mut centro_difere) = (0, 0);
        for py in 30..92i64 {
            for px in 70..130i64 {
                let mut prod = 1.0f32;
                for k in 0..cap.n() {
                    let o = cap.off(k);
                    let ck = [centro[0] + o[0], centro[1] + o[1]];
                    let (dx, dy) = (px as f32 + 0.5 - centro[0], py as f32 + 0.5 - centro[1]);
                    let t = fp.falloff_t((dx - o[0]) * (1.0 / 20.0), (dy - o[1]) * (1.0 / 20.0));
                    let x = crate::dab::silhouette_at(&spec, shape, t, px, py, ck, 20.0);
                    prod *= 1.0 - (c * x).clamp(0.0, 1.0);
                }
                let referencia = (1.0 - prod) / c;
                let lei = |l| {
                    crate::dab::silhouette_capsula(
                        &spec, shape, fp, cap, l, c, px, py, centro, 20.0,
                    )
                };
                assert_eq!(
                    lei(Lei::Uniao).to_bits(),
                    referencia.to_bits(),
                    "({px}, {py})"
                );
                vistos += usize::from(referencia > 0.0);
                centro_difere += usize::from((lei(Lei::Centro) - referencia).abs() > 0.02);
            }
        }
        assert!(vistos > 1000, "a fixtura não pinta: {vistos}");
        assert!(centro_difere > 200, "o controlo não mede: {centro_difere}");
    }
}

/// O carimbo da cápsula chega ao centro ANTERIOR (a caixa cresce para trás), e sem cápsula é o dab de
/// sempre, ao bit.
#[test]
fn o_carimbo_da_capsula_chega_ao_centro_anterior() {
    let spec = crate::BrushSpec {
        radius_px: 6.0,
        hardness: 1.0,
        color: [0.0; 3],
        ..crate::BrushSpec::default()
    };
    let carimba = |cap: Option<Capsula>| {
        let mut buf = vec![255u8; 96 * 32 * 4];
        let _ = crate::stamp_dab_textured_masked(
            &mut buf,
            96,
            32,
            [70.0, 16.0],
            &spec,
            1.0,
            false,
            None,
            None,
            None,
            None,
            [1.0, 0.0],
            cap,
        );
        buf
    };
    let tinta = |b: &[u8], x: usize| 255 - b[(16 * 96 + x) * 4];
    let liso = carimba(None);
    let cap = carimba(Capsula::new([-0.0 + 30.0, 0.0], 1.0));
    assert_eq!(
        tinta(&liso, 45),
        0,
        "o dab liso não chega ao centro anterior"
    );
    assert!(
        tinta(&cap, 45) > 200,
        "a cápsula não chega: {}",
        tinta(&cap, 45)
    );
    assert_eq!(tinta(&cap, 80), tinta(&liso, 80), "para a FRENTE nada muda");
    let mut sem = vec![255u8; 96 * 32 * 4];
    let _ = crate::stamp_dab_textured(
        &mut sem,
        96,
        32,
        [70.0, 16.0],
        &spec,
        1.0,
        false,
        None,
        None,
        None,
    );
    assert_eq!(sem, liso);
}

/// ⭐ **O passo só existe com a caixa e um método que percorre um caminho** — com QUALQUER ponta (a
/// redonda também): desligada (a omissão), nenhum dab leva passo — o traço de antes, ao bit; ligada, todo
/// dab menos o de abertura (o 1.º do traço denso também é um só); Drag Dot não percorre caminho.
#[test]
fn o_passo_so_existe_com_a_caixa_e_um_metodo_de_caminho() {
    use crate::{Dynamics, Stroke, StrokePoint, stroke_method::StrokeMethod};
    let dabs = |continuo: bool, metodo: StrokeMethod| {
        let spec = crate::BrushSpec {
            radius_px: 20.0,
            spacing: 0.25,
            stabilizer: 0.0,
            continuous_spacing: continuo,
            stroke_method: metodo,
            ..crate::BrushSpec::default()
        };
        let mut stroke = Stroke::new(spec, Dynamics::default(), 1);
        let (mut todos, mut out) = (Vec::new(), Vec::new());
        let p = |x: f32| StrokePoint {
            pos: [x, 48.0],
            pressure: 1.0,
        };
        stroke.begin(p(40.0), &mut out);
        todos.extend_from_slice(&out);
        for k in 1..=30 {
            stroke.extend(p(40.0 + 5.0 * k as f32), &mut out);
            todos.extend_from_slice(&out);
        }
        todos
    };
    assert!(!crate::BrushSpec::default().continuous_spacing);
    let off = dabs(false, StrokeMethod::default());
    assert!(off.len() > 10 && off.iter().all(|d| d.step == [0.0, 0.0]));
    let on = dabs(true, StrokeMethod::default());
    assert_eq!(on[0].step, [0.0, 0.0], "o dab de abertura");
    assert!(
        on[1..]
            .iter()
            .all(|d| d.step[0] > 5.0 && d.step[1].abs() < 1e-3),
        "{:?}",
        on.iter().map(|d| d.step).collect::<Vec<_>>()
    );
    assert_eq!(
        on.iter().map(|d| d.center).collect::<Vec<_>>(),
        off.iter().map(|d| d.center).collect::<Vec<_>>(),
        "a caixa não move os dabs"
    );
    assert!(
        dabs(true, StrokeMethod::DragDot)
            .iter()
            .all(|d| d.step == [0.0, 0.0])
    );
}
