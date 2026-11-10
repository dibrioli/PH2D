//! ⭐ **Onde a pintura DESENHOU** — a caixa VISÍVEL de cada primitivo de uma cena Vello: o caminho
//! (no transform dele) cortado pelos recortes abertos, e a origem de cada glifo. É a régua de «nada
//! se pinta fora do cartão» (dono 10/11, a ⟷ da Timeline fora da borda): a hit-list não a vê.
//!
//! ⚠️ Caixa do caminho = a dos pontos de controlo (cobre a curva); o traço conta pela linha central.
//! O desenho de cada glifo é a origem dele (o texto começa no `x` que lhe deram).

use ph2d_editor_core::zones::Rect;
use ph2d_vector::VectorScene;

const TAG_PATH: u8 = 0x10;
const TAG_TRANSFORM: u8 = 0x20;
const TAG_STYLE: u8 = 0x40;
const TAG_F32: u8 = 0x8;
const TAG_SUBPATH_END: u8 = 0x4;
const TAG_SEG_MASK: u8 = 0x3;
const DRAW_BEGIN_CLIP: u32 = 0x49;
const DRAW_END_CLIP: u32 = 0x21;

type Caixa = [f32; 4];

fn junta(c: Option<Caixa>, (x, y): (f32, f32)) -> Option<Caixa> {
    Some(c.map_or([x, y, x, y], |c| {
        [c[0].min(x), c[1].min(y), c[2].max(x), c[3].max(y)]
    }))
}

fn corta(a: Caixa, b: Caixa) -> Option<Caixa> {
    let c = [
        a[0].max(b[0]),
        a[1].max(b[1]),
        a[2].min(b[2]),
        a[3].min(b[3]),
    ];
    (c[0] <= c[2] && c[1] <= c[3]).then_some(c)
}

/// A caixa de cada caminho, pela ordem dos `PATH` (vazio = `None`).
fn caminhos(scene: &VectorScene) -> Vec<Option<Caixa>> {
    let e = scene.inner().encoding();
    let dados: Vec<f32> = e.path_data.iter().map(|&b| f32::from_bits(b)).collect();
    let (mut pd, mut xf, mut inicio) = (0usize, None::<usize>, true);
    let (mut atual, mut out) = (None, Vec::new());
    for tag in &e.path_tags {
        let t = tag.0;
        if t == TAG_PATH {
            out.push(atual.take());
            inicio = true;
        } else if t == TAG_TRANSFORM {
            xf = Some(xf.map_or(0, |i| i + 1));
        } else if t != TAG_STYLE && t & TAG_SEG_MASK != 0 {
            assert!(t & TAG_F32 != 0, "caminho em i16: a régua só lê f32");
            let n = usize::from(t & TAG_SEG_MASK) + usize::from(inicio);
            let m = xf.map(|i| e.transforms[i]);
            for p in dados[pd..pd + n * 2].as_chunks::<2>().0 {
                let (x, y) = m.map_or((p[0], p[1]), |m| {
                    (
                        m.matrix[0] * p[0] + m.matrix[2] * p[1] + m.translation[0],
                        m.matrix[1] * p[0] + m.matrix[3] * p[1] + m.translation[1],
                    )
                });
                atual = junta(atual, (x, y));
            }
            pd += n * 2;
            inicio = t & TAG_SUBPATH_END != 0;
        }
    }
    out
}

/// ⭐ A caixa visível de cada primitivo de `scene` (recortes aplicados; glifos pela origem).
#[must_use]
pub fn caixas_pintadas(scene: &VectorScene) -> Vec<Rect> {
    let e = scene.inner().encoding();
    let res = &e.resources;
    let glifos: std::collections::BTreeMap<usize, &_> = res
        .glyph_runs
        .iter()
        .map(|r| (r.stream_offsets.draw_tags, r))
        .collect();
    let paths = caminhos(scene);
    assert_eq!(
        e.draw_tags.len(),
        paths.len() + res.glyph_runs.len(),
        "cada desenho tem um caminho ou é um glifo"
    );
    let (mut it, mut clips, mut out) = (paths.into_iter(), Vec::<Option<Caixa>>::new(), Vec::new());
    let tudo = [f32::MIN, f32::MIN, f32::MAX, f32::MAX];
    for (i, tag) in e.draw_tags.iter().enumerate() {
        let topo = clips.last().copied().unwrap_or(Some(tudo));
        let caixa = if let Some(run) = glifos.get(&i) {
            let m = run.transform;
            res.glyphs[run.glyphs.clone()].iter().fold(None, |c, g| {
                let x = m.matrix[0] * g.x + m.matrix[2] * g.y + m.translation[0];
                let y = m.matrix[1] * g.x + m.matrix[3] * g.y + m.translation[1];
                junta(c, (x, y))
            })
        } else {
            it.next().flatten()
        };
        match tag.0 {
            DRAW_BEGIN_CLIP => clips.push(topo.zip(caixa).and_then(|(a, b)| corta(a, b))),
            DRAW_END_CLIP => {
                clips.pop();
            }
            _ => {
                if let Some(c) = topo.zip(caixa).and_then(|(a, b)| corta(a, b)) {
                    out.push(Rect::new(c[0], c[1], c[2] - c[0], c[3] - c[1]));
                }
            }
        }
    }
    out
}
