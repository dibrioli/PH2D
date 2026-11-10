//! **P13 no `.ase`** (`docs/PixelArt/P13_plano.md` §5 e §12 «C + R4»): o efeito de camada sai
//! assado (outros apps vêem o contorno) e volta vivo, o cel volta cru; rampas e ciclos voltam.

use ph2d_pixel_core::filter::outline::{Background, CIRCLE, Outline, Place};
use ph2d_pixel_core::{
    ColorMode, Cycle, CycleDirection, DocOp, EfeitoDeCamada, NewSprite, Op, Patch, Rect, Sprite,
    apply, compose_picked,
};
use ph2d_pixel_io::ase::{escrever, ler};

const VERMELHO: [u8; 4] = [255, 0, 0, 255];
const AZUL: [u8; 4] = [0, 0, 255, 255];

fn sprite_p13() -> Sprite {
    let mut s = Sprite::new(NewSprite {
        width: 8,
        height: 8,
        mode: ColorMode::Rgba,
        ..NewSprite::default()
    })
    .expect("sprite");
    let (layer, frame) = (s.layers()[0].id, s.frames()[0].id);
    let rect = Rect {
        x: 3,
        y: 3,
        w: 2,
        h: 2,
    };
    let pixels = AZUL.repeat(4);
    let bbox = Some(rect);
    apply(
        &mut s,
        Op::Patch(Patch {
            layer,
            frame,
            bbox,
            rect,
            pixels,
        }),
    )
    .expect("cel");
    let efeitos = vec![EfeitoDeCamada::Outline(Outline {
        color: VERMELHO,
        background: Background::Mask,
        matrix: CIRCLE,
        place: Place::Outside,
        tiled: false,
    })];
    apply(
        &mut s,
        Op::Doc(Box::new(DocOp::SetLayerEffects { layer, efeitos })),
    )
    .expect("fx");
    for (name, indices) in [("skin", vec![1, 2, 3]), ("water", vec![5, 6, 7, 8])] {
        let name = name.into();
        apply(&mut s, Op::Doc(Box::new(DocOp::AddRamp { name, indices }))).expect("ramp");
    }
    let ramp = s.ramps().ramps[1].id;
    let cycle = Some(Cycle {
        ramp,
        periodo_ms: 400,
        direcao: CycleDirection::PingPong,
    });
    apply(&mut s, Op::Doc(Box::new(DocOp::SetCycle { ramp, cycle }))).expect("cycle");
    s
}

fn quadro(s: &Sprite) -> Vec<u8> {
    compose_picked(s, s.frames()[0].id, &|l| s.is_shown(l.id))
}

#[test]
fn o_ase_leva_rampas_ciclos_e_efeitos() {
    let s0 = sprite_p13();
    let (bytes, avisos) = escrever(&s0).expect("escreve");
    assert!(avisos.is_empty(), "{avisos:?}");

    // Outro programa (só o leitor do formato): o cel do ficheiro já tem o contorno.
    let file = ph2d_aseprite::parse_file(&bytes).expect("parse");
    let c = &file.frames[0].cels[0];
    let ph2d_aseprite::AseCelBody::Image { w, pixels, .. } = &c.body else {
        panic!("cel de imagem");
    };
    let px = |x: i32, y: i32| {
        let i = ((y - i32::from(c.y)) * i32::from(*w) + (x - i32::from(c.x))) as usize * 4;
        [pixels[i], pixels[i + 1], pixels[i + 2], pixels[i + 3]]
    };
    assert_eq!(
        px(2, 3),
        VERMELHO,
        "o contorno exterior está no cel gravado"
    );
    assert_eq!(px(3, 3), AZUL);
    assert_eq!(px(2, 2), [0; 4], "a diagonal fica fora (matriz Circle)");

    let (a, _) = ler(&bytes).expect("lê");
    let (l0, la) = (&s0.layers()[0], &a.layers()[0]);
    assert_eq!(la.efeitos, l0.efeitos, "o efeito volta vivo");
    assert_eq!(
        la.user, l0.user,
        "a propriedade ph2d.efeitos sai do user data"
    );
    assert_eq!(
        a.user(),
        s0.user(),
        "ph2d.ramps/ph2d.cycles saem do user data"
    );
    let k0 = (l0.id, s0.frames()[0].id);
    let ka = (la.id, a.frames()[0].id);
    let (c0, ca) = (s0.cel(k0).expect("cel"), a.cel(ka).expect("cel"));
    assert_eq!((ca.bbox, &ca.data), (c0.bbox, &c0.data), "o cel volta cru");
    assert_eq!(ca.user, c0.user, "ph2d.cru sai do user data");
    let rampas = |s: &Sprite| {
        let p = s.ramps();
        let pos = |id| p.ramps.iter().position(|r| r.id == id);
        let r: Vec<_> = p
            .ramps
            .iter()
            .map(|r| (r.name.clone(), r.indices.clone()))
            .collect();
        let c: Vec<_> = p
            .cycles
            .iter()
            .map(|c| (pos(c.ramp), c.periodo_ms, c.direcao))
            .collect();
        (r, c)
    };
    assert_eq!(rampas(&a), rampas(&s0), "rampas e ciclos voltam");
    assert_eq!(quadro(&a), quadro(&s0), "a composição é a mesma");

    let (bytes_a, _) = escrever(&a).expect("escreve");
    let (b, _) = ler(&bytes_a).expect("lê");
    assert_eq!(a, b, "ida-e-volta exacta");
    assert_eq!(escrever(&b).expect("escreve").0, bytes_a, "determinístico");

    if let Some(dir) = std::env::var_os("PH2D_ORACULO_DIR") {
        let dir = std::path::PathBuf::from(dir);
        std::fs::write(dir.join("p13_efeitos.ase"), &bytes).expect("grava");
        std::fs::write(dir.join("p13_efeitos.rgba"), quadro(&s0)).expect("grava");
    }
}
