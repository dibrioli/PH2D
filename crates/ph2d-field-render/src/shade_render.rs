//! **O SOMBREAMENTO RENDER** — do G-buffer aos pixels com MATERIAL, LUZ e CÉU (`docs/Render3d/05`).
//!
//! Irmão do [`super::shade`], com a mesma fronteira: o traçador entrega máscara e normal, e isto é a
//! única coisa que sabe o que é uma cor. O que muda é a pergunta — o matcap responde *«que cor tem
//! esta normal na fotografia?»*; isto responde *«que luz esta superfície devolve ao olho?»*:
//!
//! 1. a lei do material é a da `ph2d-material` (o OpenPBR provado contra o MaterialX corrido);
//! 2. a direcção de vista de cada pixel sai do MESMO raio que o traçou ([`Orbit::ray_at_plane`]) —
//!    com a lente convergente ela muda de pixel para pixel, e uma `(0, 0, 1)` fixa poria o realce
//!    no sítio errado de toda peça vista em perspectiva;
//! 3. o olhar (exposição e vista) é o da `ph2d-view-transform`, aplicado **por amostra, antes** da
//!    média da borda — a média de duas luzes já transformadas é o que o olho vê, e transformar a
//!    média de duas luzes da cena não é.
//!
//! ⚠️ **Tudo em espaço de VISTA**, que é onde o G-buffer guarda a normal: as lâmpadas e o céu chegam
//! já nesse referencial, e quem as converte é quem sabe de onde elas vêm.

use super::*;
use ph2d_material::{Environment, Surface};
use ph2d_view_transform::Look;

/// Uma luz direcional, em espaço de VISTA.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lamp {
    /// A direcção **para** a luz, unitária.
    pub to_light: [f32; 3],
    /// A radiância que chega (`cor × intensidade`, a convenção do MaterialX).
    pub radiance: [f32; 3],
}

/// A luz de uma cena: as lâmpadas e o céu.
pub struct Lighting<'a> {
    pub lamps: &'a [Lamp],
    pub sky: &'a (dyn Environment + Sync),
}

/// A direcção **para o observador** no pixel `(x, y)`, em espaço de vista — o raio do traçado, ao
/// contrário.
///
/// ⚠️ `pub(crate)` para o gate da costura lhe poder chamar: a lei de que ela é a do RAIO, e não uma
/// constante, é exactamente o que aquele gate afirma.
pub(crate) fn view_direction(cam: &Orbit, screen: &Screen, x: usize, y: usize) -> [f32; 3] {
    let (u, v) = screen.plane_at(x as f32 + 0.5, y as f32 + 0.5);
    let (_, d) = cam.ray_at_plane(u, v);
    let (right, up, toward_eye) = cam.basis();
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    [-dot(d, right), -dot(d, up), -dot(d, toward_eye)]
}

/// A luz que a superfície devolve pela direcção `v`, já com o olhar — em linear de ECRÃ.
fn radiance(
    surface: &Surface,
    light: &Lighting<'_>,
    look: Look,
    n: [f32; 3],
    v: [f32; 3],
) -> [f32; 3] {
    let add = |a: [f32; 3], b: [f32; 3]| [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
    let mut rgb = surface.indirect(n, v, light.sky);
    for lamp in light.lamps {
        rgb = add(rgb, surface.direct(n, v, lamp.to_light, lamp.radiance));
    }
    look.apply(add(rgb, surface.emission(n, v)))
}

/// Colore o G-buffer com um material sob uma luz e devolve RGBA8 **pré-multiplicado**.
///
/// ⚠️ As mesmas duas leis do [`super::shade`]: a borda é a média em LINEAR (aqui, linear de ecrã,
/// porque o olhar já foi aplicado a cada amostra), e o pixel de fundo sai com os bytes EXACTOS que o
/// chamador pediu.
///
/// ⚠️ **As linhas correm em paralelo** e as bordas depois, em série: uma borda precisa do fundo e de
/// quatro amostras, e são poucas (`0,5–1,2 %` dos pixels, `docs/3DModeling/05`).
#[must_use]
pub fn shade_render(
    g: &Gbuffer,
    cam: &Orbit,
    surface: &Surface,
    light: &Lighting<'_>,
    look: Look,
    background: [u8; 4],
) -> Vec<u8> {
    let (w, h) = (g.width as usize, g.height as usize);
    let mut out = vec![0u8; w * h * 4];
    if w == 0 || h == 0 {
        return out;
    }
    let screen = Screen::new(g.width, g.height, cam.half_extent);
    let write = |px: &mut [u8], c: [f32; 4]| {
        px[0] = ph2d_color::srgb::linear_to_srgb_byte(c[0]);
        px[1] = ph2d_color::srgb::linear_to_srgb_byte(c[1]);
        px[2] = ph2d_color::srgb::linear_to_srgb_byte(c[2]);
        px[3] = (c[3].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    };

    out.par_chunks_mut(w * 4).enumerate().for_each(|(y, row)| {
        for (x, px) in row.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let i = y * w + x;
            if g.hit[i] {
                let c = radiance(
                    surface,
                    light,
                    look,
                    g.normal[i],
                    view_direction(cam, &screen, x, y),
                );
                write(px, [c[0], c[1], c[2], 1.0]);
            } else {
                // ⚠️ Copiado, e não passado pela conversão — a mesma cerca do `shade`.
                px.copy_from_slice(&background);
            }
        }
    });

    let bg_a = f32::from(background[3]) / 255.0;
    let bg = [
        ph2d_color::srgb::srgb_to_linear_byte(background[0]) * bg_a,
        ph2d_color::srgb::srgb_to_linear_byte(background[1]) * bg_a,
        ph2d_color::srgb::srgb_to_linear_byte(background[2]) * bg_a,
        bg_a,
    ];
    for e in &g.edges {
        let i = e.pixel as usize;
        // ⚠️ **A vista do CENTRO do pixel serve às quatro amostras**: dentro de um pixel a direcção do
        // raio muda menos do que o passo de um byte move a luz, e a borda não guarda as posições.
        let v = view_direction(cam, &screen, i % w, i / w);
        let mut acc = [0.0f32; 4];
        for k in 0..4 {
            let c = if e.hit[k] {
                let rgb = radiance(surface, light, look, e.normal[k], v);
                [rgb[0], rgb[1], rgb[2], 1.0]
            } else {
                bg
            };
            for j in 0..4 {
                acc[j] += c[j] * 0.25;
            }
        }
        write(&mut out[i * 4..i * 4 + 4], acc);
    }
    out
}
