//! P13 U-Paleta: um sprite de UM quadro também cicla — o relógio do Play (voltas inteiras da
//! animação) soma-se ao instante do quadro (`SpriteImage::avanca_ciclo`).

use ph2d_pixel_core::{
    Background, ColorMode, Cycle, CycleDirection, DocOp, NewSprite, Op, Patch, Sprite, apply,
    ciclo_em,
};
use ph2d_pixel_render::SpriteImage;

/// 6×1 Indexed de UM quadro (100 ms): índices 0..6, rampa `[1, 2, 3, 4]` com ciclo de 400 ms.
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
    let pixels = (0..6u8).collect();
    let patch = Patch {
        layer,
        frame,
        bbox: Some(r),
        rect: r,
        pixels,
    };
    apply(&mut s, Op::Patch(patch)).expect("patch");
    let add = DocOp::AddRamp {
        name: String::new(),
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
fn o_sprite_de_um_quadro_muda_de_indice_com_o_tempo_do_play() {
    let s = sprite();
    let f = s.frames()[0].id;
    let mut img = SpriteImage::default();
    let mut vistos = Vec::new();
    for k in 0..4u64 {
        if k > 0 {
            img.avanca_ciclo(&s, 100);
        }
        img.refresh(&s, f, None, 1);
        let quer = s.palette()[usize::from(ciclo_em(s.ramps(), k * 100)[1])];
        let c = img.rgba_at(6, 1, 0).expect("célula");
        assert_eq!(
            c,
            quer,
            "volta {k}: a célula 1 mostra o degrau do ciclo em t = {}",
            k * 100
        );
        vistos.push(c);
    }
    vistos.sort_unstable();
    vistos.dedup();
    assert_eq!(
        vistos.len(),
        4,
        "os 4 degraus da rampa passaram pela célula"
    );
}
