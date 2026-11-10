//! ⭐ **O ciclo de cor na cena é o do editor** (P13 U-Paleta): o canvas do editor e o Pixel Sprite
//! na cena desenham pela MESMA [`SpriteImage`] com o quadro que mostram (`quadro_a_desenhar`), e o
//! export pelo `compose_picked`; os três rodam os ciclos no [`instante_do_quadro`]. Mesmo quadro ⇒
//! mesmo índice em todos.

use ph2d_pixel_core::frames::{self, NewFrame};
use ph2d_pixel_core::{
    Background, ColorMode, Cycle, CycleDirection, DocOp, NewSprite, Onion, Op, Patch, Sprite,
    apply, ciclo_em, compose_picked, instante_do_quadro,
};
use ph2d_pixel_render::SpriteImage;

/// 6×1 Indexed: índices 0..6 (0 transparente), rampa `[1, 2, 3, 4]` com ciclo de 400 ms, 4 quadros
/// de 100 ms ⇒ cada quadro está num degrau diferente do ciclo.
fn sprite() -> Sprite {
    let mut s = Sprite::new(NewSprite {
        width: 6,
        height: 1,
        mode: ColorMode::Indexed,
        background: Background::Transparent,
    })
    .expect("sprite");
    let (layer, frame) = (s.layers()[0].id, s.frames()[0].id);
    let r = s.bounds();
    let patch = Patch {
        layer,
        frame,
        bbox: Some(r),
        rect: r,
        pixels: (0..6u8).collect(),
    };
    apply(&mut s, Op::Patch(patch)).expect("patch");
    for _ in 0..3 {
        let last = s.frames().last().expect("f").id;
        let (op, _) = frames::new_frame(&mut s, last, NewFrame::Copy).expect("frame");
        apply(&mut s, op).expect("frame");
    }
    let ids: Vec<_> = s.frames().iter().map(|f| f.id).collect();
    // `None` = já são 100 ms (a duração de fábrica).
    if let Some(op) = frames::set_duration(&s, &ids, 100) {
        apply(&mut s, op).expect("duração");
    }
    let add = DocOp::AddRamp {
        name: "water".into(),
        indices: vec![1, 2, 3, 4],
    };
    apply(&mut s, Op::Doc(Box::new(add))).expect("rampa");
    let ramp = s.ramps().ramps[0].id;
    let cycle = Some(Cycle {
        ramp,
        periodo_ms: 400,
        direcao: CycleDirection::Forward,
    });
    apply(&mut s, Op::Doc(Box::new(DocOp::SetCycle { ramp, cycle }))).expect("ciclo");
    s
}

#[test]
fn o_ciclo_na_cena_e_o_do_editor() {
    let s = sprite();
    let mut vistos = Vec::new();
    for (k, f) in s.frames().iter().map(|f| f.id).enumerate() {
        let t = instante_do_quadro(&s, f);
        assert_eq!(t, 100 * k as u64, "o instante = a soma das durações antes");
        let perm = ciclo_em(s.ramps(), t);
        let esperado: Vec<u8> = (0..6u32)
            .flat_map(|i| {
                let mostra = if i == 0 {
                    0
                } else {
                    u32::from(perm[i as usize])
                };
                s.rgba_of_value(mostra)
            })
            .collect();
        // Editor e cena: a mesma imagem, com e sem papel vegetal (o quadro da frente roda igual).
        for onion in [None, Some(Onion::default())] {
            let mut img = SpriteImage::default();
            img.refresh_with(&s, f, None, 1, onion.as_ref());
            let px: Vec<u8> = (0..6)
                .flat_map(|x| img.rgba_at(6, x, 0).expect("px"))
                .collect();
            if onion.is_none() {
                assert_eq!(px, esperado, "quadro {k}: a vista mostra o índice do ciclo");
            } else {
                assert_eq!(px[4..20], esperado[4..20], "quadro {k}: com papel vegetal");
            }
        }
        // O export assa o mesmo instante.
        assert_eq!(
            compose_picked(&s, f, &|_| true),
            esperado,
            "quadro {k}: export"
        );
        vistos.push(esperado);
    }
    vistos.dedup();
    assert_eq!(vistos.len(), 4, "cada quadro num degrau diferente do ciclo");
}
