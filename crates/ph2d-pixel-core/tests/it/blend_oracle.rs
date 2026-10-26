//! **As camadas contra o Aseprite Trial observado** — a fixtura `oracle_trial_blend.json`
//! (`docs/PixelArt/ferramentas/trial_ase_camadas.py` escreve os `.ase`, `trial_camadas.py` lê a tela,
//! `trial_fixture_camadas.py` grava). Cada caso compõe-se pelo NOSSO documento e tem de dar a tela
//! célula a célula: os 19 modos a 255/128/77, o grupo (pass-through; modo e opacidade do grupo não
//! entram), e o Indexed (mistura em RGB, sem voltar à paleta).

use ph2d_pixel_core::blend::BlendMode;
use ph2d_pixel_core::layers::{self, Place};
use ph2d_pixel_core::{
    Background, ColorMode, LayerId, LayerKind, NewSprite, Op, Patch, Rect, Sprite, apply,
    compose_rgba,
};
use serde_json::Value;

fn fx() -> Value {
    serde_json::from_str(include_str!("../fixtures/oracle_trial_blend.json")).expect("json")
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("hex"))
        .collect()
}

struct Caso {
    w: u32,
    h: u32,
    mode: ColorMode,
    s: Sprite,
}

impl Caso {
    fn new(w: u32, h: u32, mode: ColorMode) -> Self {
        let s = Sprite::new(NewSprite {
            width: w,
            height: h,
            mode,
            background: Background::Transparent,
        })
        .expect("sprite");
        Self { w, h, mode, s }
    }

    /// Uma camada nova acima de `above` (ou a primeira, já existente) com estes bytes.
    fn camada(&mut self, above: Option<LayerId>, bytes: &[u8]) -> LayerId {
        let id = match above {
            None => self.s.layers()[0].id,
            Some(a) => {
                let (op, id) = layers::new_layer(&mut self.s, a).expect("nova");
                apply(&mut self.s, op).expect("nova");
                id
            }
        };
        let r = Rect::new(0, 0, self.w, self.h);
        let frame = self.s.frames()[0].id;
        let pixels = bytes.to_vec();
        assert_eq!(pixels.len(), r.area() * self.mode.bpp());
        apply(
            &mut self.s,
            Op::Patch(Patch {
                layer: id,
                frame,
                bbox: Some(r),
                rect: r,
                pixels,
            }),
        )
        .expect("patch");
        id
    }

    fn grupo(&mut self, above: LayerId) -> LayerId {
        let (op, g) = layers::new_group(&mut self.s, above).expect("grupo");
        apply(&mut self.s, op).expect("grupo");
        g
    }

    fn props(&mut self, id: LayerId, mode: u8, opacity: u8) {
        if let Some(op) = layers::edit(&self.s, id, |l| {
            l.blend = BlendMode::from_code(mode).expect("modo");
            l.opacity = opacity;
        }) {
            apply(&mut self.s, op).expect("props");
        }
    }

    fn entra(&mut self, id: LayerId, g: LayerId) {
        let op = layers::move_to(&self.s, id, Place::Into(g)).expect("entra");
        apply(&mut self.s, op).expect("entra");
    }

    fn rgb(&self) -> Vec<u8> {
        let mut out = Vec::new();
        compose_rgba(&self.s, self.s.frames()[0].id, None, &mut out);
        out.as_chunks::<4>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2]])
            .collect()
    }
}

#[test]
fn every_blend_group_and_indexed_case_matches_the_trial_screen() {
    let f = fx();
    let n = f["n"].as_u64().expect("n") as usize;
    let w = f["w"].as_u64().expect("w") as u32;
    let h = (n as u32) / w;
    let (back, src, mid) = (
        unhex(f["back"].as_str().expect("b")),
        unhex(f["src"].as_str().expect("s")),
        unhex(f["mid"].as_str().expect("m")),
    );
    let casos = f["cases"].as_array().expect("cases");
    assert_eq!(casos.len(), 103, "a fixtura mudou de tamanho");
    let num = |s: &str| s.parse::<u8>().expect("num");
    let mut grupos = 0;
    for c in casos {
        let nome = c["file"].as_str().expect("file");
        let want = unhex(c["screen_rgb"].as_str().expect("rgb"));
        let stem = nome.trim_end_matches(".ase");
        let got = if let Some(rest) = stem.strip_prefix("gm") {
            // gm{MM}_o{OOO}: o grupo com modo e opacidade, a fonte Normal dentro
            let (m, o) = (num(&rest[0..2]), num(&rest[4..7]));
            let mut k = Caso::new(w, h, ColorMode::Rgba);
            let b = k.camada(None, &back);
            let g = k.grupo(b);
            k.props(g, m, o);
            let s = k.camada(Some(g), &src);
            k.entra(s, g);
            grupos += 1;
            assert!(matches!(
                k.s.layer(g).map(|l| l.kind),
                Some(LayerKind::Group { .. })
            ));
            k.rgb()
        } else if let Some(rest) = stem.strip_prefix('g') {
            // g{MM}: o grupo Normal com [meio Normal, fonte MM]
            let m = num(&rest[0..2]);
            let mut k = Caso::new(w, h, ColorMode::Rgba);
            let b = k.camada(None, &back);
            let g = k.grupo(b);
            let mi = k.camada(Some(g), &mid);
            k.entra(mi, g);
            let s = k.camada(Some(mi), &src);
            k.props(s, m, 255);
            grupos += 1;
            k.rgb()
        } else if let Some(rest) = stem.strip_prefix('m') {
            let (m, o) = (num(&rest[0..2]), num(&rest[4..7]));
            let mut k = Caso::new(w, h, ColorMode::Rgba);
            let b = k.camada(None, &back);
            let s = k.camada(Some(b), &src);
            k.props(s, m, o);
            k.rgb()
        } else if let Some(rest) = stem.strip_prefix("cel_o") {
            // a opacidade do CEL é a mesma lei que a da camada (P4 traz o campo do cel)
            let o = num(rest);
            let mut k = Caso::new(w, h, ColorMode::Rgba);
            let b = k.camada(None, &back);
            let s = k.camada(Some(b), &src);
            k.props(s, 0, o);
            k.rgb()
        } else if let Some(rest) = stem.strip_prefix("ix_o") {
            let o = num(rest);
            let mut k = Caso::new(w, h, ColorMode::Indexed);
            let pal = unhex(f["pal"].as_str().expect("pal"));
            let colors: Vec<[u8; 4]> = pal.as_chunks::<4>().0.to_vec();
            apply(
                &mut k.s,
                Op::Palette {
                    colors,
                    transparent: 0,
                },
            )
            .expect("paleta");
            let b = k.camada(None, &unhex(f["ib"].as_str().expect("ib")));
            let s = k.camada(Some(b), &unhex(f["is"].as_str().expect("is")));
            k.props(s, 0, o);
            k.rgb()
        } else {
            panic!("caso desconhecido {nome}");
        };
        let erros = got
            .as_chunks::<3>()
            .0
            .iter()
            .zip(want.as_chunks::<3>().0)
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(
            erros, 0,
            "{nome}: {erros} de {} células diferem da tela do Trial",
            n
        );
    }
    assert_eq!(grupos, 19 + 18 + 3, "os casos de grupo");
}
