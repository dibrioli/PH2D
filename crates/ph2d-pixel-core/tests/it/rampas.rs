//! P13 R1: as rampas, os ciclos, as guias, o som e os efeitos de camada como DADO do documento
//! (`docs/PixelArt/P13_plano.md` §4, §5, §7).

use std::collections::BTreeMap;

use ph2d_pixel_core::codec::{read_library, write_library};
use ph2d_pixel_core::filter::outline::{Background as Fundo, CIRCLE, Outline, Place};
use ph2d_pixel_core::{
    Background, ColorMode, Cycle, CycleDirection, DocOp, EfeitoDeCamada, FaixaDeAudio, Guias,
    Iso2x1, NewSprite, Op, OpError, RampId, Sprite, apply, ciclo_em, rampa_de,
};

fn doc(s: &mut Sprite, d: DocOp) -> Result<Op, OpError> {
    apply(s, Op::Doc(Box::new(d)))
}

/// Um Indexed com a DB32, duas rampas (uma com ciclo), guias, som e um contorno vivo.
fn com_tudo() -> Sprite {
    let mut s = Sprite::new(NewSprite {
        width: 16,
        height: 16,
        mode: ColorMode::Indexed,
        background: Background::Transparent,
    })
    .expect("sprite");
    doc(
        &mut s,
        DocOp::AddRamp {
            name: "skin".into(),
            indices: vec![1, 2, 3, 4, 5],
        },
    )
    .unwrap();
    doc(
        &mut s,
        DocOp::AddRamp {
            name: "water".into(),
            indices: vec![9, 10, 11, 12],
        },
    )
    .unwrap();
    let agua = s.ramps().ramps[1].id;
    let cycle = Cycle {
        ramp: agua,
        periodo_ms: 800,
        direcao: CycleDirection::PingPong,
    };
    doc(
        &mut s,
        DocOp::SetCycle {
            ramp: agua,
            cycle: Some(cycle),
        },
    )
    .unwrap();
    let guias = Guias {
        pontos_de_fuga: [Some([-40.5, 7.25]), Some([60.0, 7.25]), None],
        horizonte: Some(7.25),
        iso: Some(Iso2x1 {
            origem: [8.0, 0.5],
            passo: 5,
        }),
    };
    doc(&mut s, DocOp::SetGuias(guias)).unwrap();
    let audio = vec![
        FaixaDeAudio {
            asset: "sfx/step.ogg".into(),
            inicio_ms: -120,
            ganho: 0.75,
            mudo: false,
        },
        FaixaDeAudio {
            asset: "music/loop.ogg".into(),
            inicio_ms: 400,
            ganho: 1.0,
            mudo: true,
        },
    ];
    doc(&mut s, DocOp::SetAudio(audio)).unwrap();
    let outline = Outline {
        color: [20, 12, 28, 255],
        background: Fundo::Color([0, 0, 0, 0]),
        matrix: CIRCLE,
        place: Place::Inside,
        tiled: true,
    };
    let layer = s.layers()[0].id;
    let efeitos = vec![EfeitoDeCamada::Outline(outline)];
    doc(&mut s, DocOp::SetLayerEffects { layer, efeitos }).unwrap();
    s
}

#[test]
fn o_blob_pixel_volta_igual() {
    let s = com_tudo();
    assert_eq!(s.ramps().ramps.len(), 2);
    assert_eq!(s.ramps().cycles.len(), 1);
    assert_ne!(*s.guias(), Guias::default());
    assert_eq!(s.audio().len(), 2);
    assert_eq!(s.layers()[0].efeitos.len(), 1);
    let lib = BTreeMap::from([(5_u64, s)]);
    let bytes = write_library(&lib);
    assert_eq!(&bytes[11..13], &9_u16.to_le_bytes(), "MINOR 9");
    let back = read_library(&bytes).expect("lê");
    assert_eq!(back, lib, "grava ⇒ lê ⇒ igual");
    assert_eq!(write_library(&back), bytes, "regravar dá os mesmos bytes");
}

#[test]
fn cada_op_do_dado_tem_inverso_exacto() {
    let mut s = com_tudo();
    let agua = s.ramps().ramps[1].id;
    let layer = s.layers()[0].id;
    let ops = [
        DocOp::AddRamp {
            name: "leaf".into(),
            indices: vec![14, 15, 16],
        },
        DocOp::RemoveRamp(agua),
        DocOp::SetCycle {
            ramp: agua,
            cycle: None,
        },
        DocOp::SetGuias(Guias::default()),
        DocOp::SetAudio(Vec::new()),
        DocOp::SetLayerEffects {
            layer,
            efeitos: Vec::new(),
        },
    ];
    for d in ops {
        let add = matches!(d, DocOp::AddRamp { .. });
        let mut t = s.clone();
        let inv = doc(&mut t, d).expect("aplica");
        assert_ne!(t, s);
        apply(&mut t, inv).expect("desfaz");
        assert_eq!(t.ramps(), s.ramps());
        // O AddRamp gasta um id do contador (como uma camada nova); o resto volta byte a byte.
        if !add {
            assert_eq!(t, s);
        }
    }
    // As leis: índice fora da paleta (DB32 = 32 cores), índice já noutra rampa, rampa vazia,
    // ciclo numa rampa que não existe.
    for (indices, why) in [
        (vec![40], "ramp.index"),
        (vec![3], "ramp.index"),
        (vec![], "ramp.empty"),
    ] {
        let r = doc(
            &mut s.clone(),
            DocOp::AddRamp {
                name: "x".into(),
                indices,
            },
        );
        assert_eq!(r, Err(OpError::BadRamps(why)));
    }
    let fantasma = RampId(9999);
    let c = Cycle {
        ramp: fantasma,
        periodo_ms: 100,
        direcao: CycleDirection::Forward,
    };
    assert!(
        doc(
            &mut s,
            DocOp::SetCycle {
                ramp: fantasma,
                cycle: Some(c)
            }
        )
        .is_err()
    );
    assert_eq!(rampa_de(s.ramps(), 3), Some((s.ramps().ramps[0].id, 2)));
    assert_eq!(rampa_de(s.ramps(), 0), None);
}

#[test]
fn o_remap_da_paleta_leva_as_rampas() {
    let mut s = com_tudo();
    let n = s.palette().len();
    let perm: Vec<usize> = (0..n).rev().collect();
    let inv = apply(&mut s, Op::Remap { perm }).expect("remap");
    let pele = &s.ramps().ramps[0].indices;
    let esperado: Vec<u8> = [1, 2, 3, 4, 5].iter().map(|&i| (n - 1 - i) as u8).collect();
    assert_eq!(*pele, esperado);
    apply(&mut s, inv).expect("desfaz");
    assert_eq!(s, com_tudo());
}

#[test]
fn o_ciclo_roda_so_a_rampa_e_volta_no_periodo() {
    let s = com_tudo();
    let p = s.ramps();
    let id: [u8; 256] = std::array::from_fn(|i| i as u8);
    assert_eq!(ciclo_em(p, 0), id);
    assert_eq!(ciclo_em(p, 800), id);
    let meio = ciclo_em(p, 300);
    for i in 0..=255u8 {
        if !(9..=12).contains(&i) {
            assert_eq!(meio[usize::from(i)], i, "fora da rampa fica");
        }
    }
    let mut vista: Vec<u8> = (9..=12).map(|i| meio[i]).collect();
    vista.sort_unstable();
    assert_eq!(vista, vec![9, 10, 11, 12], "permutação");
}
