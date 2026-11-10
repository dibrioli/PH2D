//! P13 C: o efeito de camada Outline e o color cycling na COMPOSIÇÃO (`docs/PixelArt/P13_plano.md`
//! §4, §10, §12). Uma lei, dois chamadores: o efeito vivo = o filtro destrutivo, byte a byte.

use ph2d_pixel_core::filter::outline::{Background as Fundo, CIRCLE, Outline, Place, SQUARE};
use ph2d_pixel_core::filter::{self, Channels};
use ph2d_pixel_core::{
    Background, ColorMode, Cycle, CycleDirection, DocOp, EfeitoDeCamada, LayerId, NewSprite, Op,
    Patch, Purpose, Rect, Sprite, aplicar_efeitos, apply, ciclo_em, compose_window,
    compose_window_em, frames, layers,
};

const W: u32 = 12;
const H: u32 = 10;

fn doc(s: &mut Sprite, d: DocOp) {
    apply(s, Op::Doc(Box::new(d))).expect("doc op");
}

fn cor(s: &Sprite, rgba: [u8; 4]) -> Vec<u8> {
    let v = s.value_of_rgba(rgba);
    v.to_le_bytes()[..s.mode().bpp()].to_vec()
}

fn pinta(s: &mut Sprite, layer: LayerId, frame: usize, r: Rect, rgba: [u8; 4]) {
    let frame = s.frames()[frame].id;
    let px = cor(s, rgba).repeat(r.area());
    apply(
        s,
        Op::Patch(Patch {
            layer,
            frame,
            bbox: Some(r),
            rect: r,
            pixels: px,
        }),
    )
    .expect("patch");
}

/// Fundo opaco por baixo, um bloco pequeno por cima num cel MENOR que o sprite, dois quadros.
fn cena(mode: ColorMode) -> (Sprite, LayerId) {
    let mut s = Sprite::new(NewSprite {
        width: W,
        height: H,
        mode,
        background: Background::Transparent,
    })
    .expect("sprite");
    let fundo = s.layers()[0].id;
    let (op, topo) = layers::new_layer(&mut s, fundo).expect("camada");
    apply(&mut s, op).expect("nova camada");
    let f0 = s.frames()[0].id;
    let (op, _) = frames::new_frame(&mut s, f0, frames::NewFrame::Empty).expect("quadro");
    apply(&mut s, op).expect("quadro");
    let (pele, agua) = (s.palette()[4], s.palette()[9]);
    pinta(&mut s, fundo, 0, Rect::new(0, 6, W, 4), agua);
    pinta(&mut s, topo, 0, Rect::new(4, 3, 3, 2), pele);
    pinta(&mut s, topo, 1, Rect::new(7, 2, 2, 4), pele);
    (s, topo)
}

fn contorno(place: Place) -> Outline {
    Outline {
        color: [255, 255, 255, 255],
        background: Fundo::Mask,
        matrix: if place == Place::Outside { CIRCLE } else { SQUARE },
        place,
        tiled: false,
    }
}

fn vista(s: &Sprite, frame: usize, purpose: Purpose) -> Vec<u8> {
    compose_window(s, s.frames()[frame].id, None, s.bounds(), purpose)
}

#[test]
fn o_efeito_de_contorno_e_o_filtro() {
    for mode in [ColorMode::Rgba, ColorMode::Indexed] {
        for place in [Place::Outside, Place::Inside] {
            let (base, topo) = cena(mode);
            let o = contorno(place);
            let mut vivo = base.clone();
            let efeitos = vec![EfeitoDeCamada::Outline(o)];
            doc(&mut vivo, DocOp::SetLayerEffects { layer: topo, efeitos });
            let mut assado = base.clone();
            let keys: Vec<_> = assado.frames().iter().map(|f| (topo, f.id)).collect();
            let op = filter::apply(&assado, &keys, Channels::ALL, &o).expect("o filtro muda");
            apply(&mut assado, op).expect("filtro");
            for frame in 0..2 {
                for purpose in [Purpose::View, Purpose::Export] {
                    assert_eq!(
                        vista(&vivo, frame, purpose),
                        vista(&assado, frame, purpose),
                        "{mode:?} {place:?} quadro {frame} {purpose:?}"
                    );
                }
            }
            if place == Place::Outside {
                // O contorno exterior sai do bbox do cel (4,3 3×2): (5,2) é contorno.
                let v = vista(&vivo, 0, Purpose::View);
                let i = (2 * W as usize + 5) * 4;
                assert_eq!(&v[i..i + 4], &[255, 255, 255, 255], "{mode:?}: fora do bbox");
            }
        }
    }
}

#[test]
fn sem_efeito_e_a_composicao_de_hoje() {
    for mode in [ColorMode::Rgba, ColorMode::Indexed] {
        let (mut s, topo) = cena(mode);
        // A referência independente: Normal opaco, o de cima ganha onde não é transparente.
        let esperado = |s: &Sprite, frame: usize| -> Vec<u8> {
            let f = s.frames()[frame].id;
            let mut out = Vec::new();
            for y in 0..H as i32 {
                for x in 0..W as i32 {
                    let c = s
                        .layers()
                        .iter()
                        .rev()
                        .map(|l| s.rgba_of_value(s.value_at((l.id, f), x, y)))
                        .find(|c| c[3] != 0)
                        .unwrap_or([0; 4]);
                    out.extend_from_slice(&c);
                }
            }
            out
        };
        for frame in 0..2 {
            assert_eq!(vista(&s, frame, Purpose::View), esperado(&s, frame), "{mode:?}");
        }
        let efeitos = vec![EfeitoDeCamada::Outline(contorno(Place::Outside))];
        doc(&mut s, DocOp::SetLayerEffects { layer: topo, efeitos });
        assert_ne!(vista(&s, 0, Purpose::View), esperado(&s, 0), "o efeito tem de se ver");
        doc(&mut s, DocOp::SetLayerEffects { layer: topo, efeitos: Vec::new() });
        for frame in 0..2 {
            assert_eq!(vista(&s, frame, Purpose::View), esperado(&s, frame), "{mode:?}");
        }
    }
}

#[test]
fn aplicar_os_efeitos_assa_todos_os_quadros_num_passo_com_inverso_exacto() {
    for mode in [ColorMode::Rgba, ColorMode::Indexed] {
        let (mut s, topo) = cena(mode);
        let efeitos = vec![EfeitoDeCamada::Outline(contorno(Place::Outside))];
        doc(&mut s, DocOp::SetLayerEffects { layer: topo, efeitos });
        let antes = s.clone();
        let vivas: Vec<_> = (0..2).map(|f| vista(&s, f, Purpose::Export)).collect();
        let op = aplicar_efeitos(&s, topo).expect("tem efeitos");
        assert!(matches!(op, Op::Batch(_)), "um passo");
        let inverso = apply(&mut s, op).expect("apply");
        assert!(s.layer(topo).unwrap().efeitos.is_empty(), "a lista esvazia");
        for f in 0..2 {
            assert_eq!(vista(&s, f, Purpose::Export), vivas[f], "{mode:?} quadro {f}");
        }
        assert!(aplicar_efeitos(&s, topo).is_none(), "sem efeitos não há passo");
        apply(&mut s, inverso).expect("inverso");
        assert_eq!(s, antes, "{mode:?}: o inverso devolve o documento");
    }
}

/// Indexed com a rampa 1..=4 a rodar e as células com os índices 0..=5 numa linha.
fn com_ciclo(mode: ColorMode, direcao: CycleDirection) -> Sprite {
    let mut s = Sprite::new(NewSprite {
        width: 6,
        height: 1,
        mode,
        background: Background::Transparent,
    })
    .expect("sprite");
    let (layer, frame) = (s.layers()[0].id, s.frames()[0].id);
    let pal = s.palette().to_vec();
    let pixels = (0..6u8)
        .flat_map(|i| cor(&s, if i == 0 { [0; 4] } else { pal[usize::from(i)] }))
        .collect();
    let r = s.bounds();
    apply(
        &mut s,
        Op::Patch(Patch {
            layer,
            frame,
            bbox: Some(r),
            rect: r,
            pixels,
        }),
    )
    .expect("patch");
    doc(
        &mut s,
        DocOp::AddRamp {
            name: "water".into(),
            indices: vec![1, 2, 3, 4],
        },
    );
    let ramp = s.ramps().ramps[0].id;
    let cycle = Some(Cycle {
        ramp,
        periodo_ms: 400,
        direcao,
    });
    doc(&mut s, DocOp::SetCycle { ramp, cycle });
    s
}

#[test]
fn o_ciclo_na_composicao_e_o_do_ciclo_em() {
    for direcao in [
        CycleDirection::Forward,
        CycleDirection::Reverse,
        CycleDirection::PingPong,
    ] {
        let s = com_ciclo(ColorMode::Indexed, direcao);
        let f = s.frames()[0].id;
        let hoje = compose_window(&s, f, None, s.bounds(), Purpose::View);
        assert_eq!(compose_window_em(&s, f, None, s.bounds(), Purpose::View, 0), hoje);
        let mut mudou = false;
        for t in (0..1200).step_by(50) {
            let perm = ciclo_em(s.ramps(), t);
            let v = compose_window_em(&s, f, None, s.bounds(), Purpose::View, t);
            let esperado: Vec<u8> = (0..6u32)
                .flat_map(|x| {
                    let i = s.value_at((s.layers()[0].id, f), x as i32, 0);
                    let mostra = if i == u32::from(s.transparent_index()) {
                        i
                    } else {
                        u32::from(perm[i as usize])
                    };
                    s.rgba_of_value(mostra)
                })
                .collect();
            assert_eq!(v, esperado, "{direcao:?} t={t}");
            assert_eq!(v[20..24], hoje[20..24], "o índice 5 está fora da rampa");
            mudou |= v != hoje;
            let volta = compose_window_em(&s, f, None, s.bounds(), Purpose::View, t + 400);
            assert_eq!(v, volta, "{direcao:?}: volta no período");
        }
        assert!(mudou, "{direcao:?}: o ciclo tem de se ver");
    }
}

#[test]
fn em_rgb_o_ciclo_nao_muda_nada() {
    let mut s = com_ciclo(ColorMode::Rgba, CycleDirection::Forward);
    let f = s.frames()[0].id;
    // Em RGB um canal com o número de um índice da rampa NÃO é um índice.
    let layer = s.layers()[0].id;
    let pixels = (0..6u8).flat_map(|i| [i, i, i, 255]).collect();
    let r = s.bounds();
    let p = Patch {
        layer,
        frame: f,
        bbox: Some(r),
        rect: r,
        pixels,
    };
    apply(&mut s, Op::Patch(p)).expect("patch");
    let hoje = compose_window(&s, f, None, s.bounds(), Purpose::View);
    for t in (0..800).step_by(50) {
        assert_eq!(
            compose_window_em(&s, f, None, s.bounds(), Purpose::View, t),
            hoje,
            "t={t}"
        );
    }
}
