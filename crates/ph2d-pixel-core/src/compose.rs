//! A vista derivada: o RGBA de um frame, camadas visíveis de baixo para cima.

use crate::blend::{BlendMode, blend};
use crate::sprite::{ColorMode, FrameId, Layer, LayerId, LayerKind, Rect, Sprite};
use crate::stroke::Stroke;

/// Para quem é a imagem composta: a VISTA mostra as camadas de referência; a EXPORTAÇÃO e as
/// fusões (Merge Down, Flatten) não as levam (Aseprite: a referência não entra no ficheiro).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Purpose {
    View,
    Export,
    /// P14: só as camadas Normal Map (as de referência não entram, como no Export).
    Normal,
}

/// A normal de frente (`n_xy = 0`), em RGB: o texel sem normal pintada.
pub const NORMAL_FLAT: [u8; 3] = [128, 128, 255];

/// P14: a normal do quadro `frame`, `w * h * 4` bytes. RGB = só as camadas Normal Map compostas
/// sobre [`NORMAL_FLAT`] (cobertura parcial mistura com a de frente); A = o alfa da COR composta
/// (Export), para que a banda normal cubra só onde o sprite tem pixels.
#[must_use]
pub fn compose_normal(sprite: &Sprite, frame: FrameId) -> Vec<u8> {
    let region = sprite.bounds();
    let mut n = compose_window(sprite, frame, None, region, Purpose::Normal);
    let cor = compose_window(sprite, frame, None, region, Purpose::Export);
    for (t, c) in n
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(cor.as_chunks::<4>().0)
    {
        let a = u32::from(t[3]);
        for (v, f) in t.iter_mut().zip(NORMAL_FLAT) {
            *v = ((u32::from(*v) * a + u32::from(f) * (255 - a) + 127) / 255) as u8;
        }
        t[3] = c[3];
    }
    n
}

/// Compõe o frame `frame` em `out` (`w * h * 4` bytes, RGBA de alfa reto). `pending` é o traço em
/// curso, visto por cima da camada dele sem tocar o documento.
pub fn compose_rgba(sprite: &Sprite, frame: FrameId, pending: Option<&Stroke>, out: &mut Vec<u8>) {
    let (w, h) = (sprite.width() as usize, sprite.height() as usize);
    out.clear();
    out.resize(w * h * 4, 0);
    compose_region(sprite, frame, pending, sprite.bounds(), Purpose::View, out);
}

/// Compõe só `region` (cortada ao sprite) dentro do `out` de tamanho cheio, que tem de vir já com
/// `w * h * 4` bytes; fora de `region` nada se toca. É o mesmo laço da composição cheia (uma só
/// resposta para «o que está por baixo deste pixel»).
pub fn compose_region(
    sprite: &Sprite,
    frame: FrameId,
    pending: Option<&Stroke>,
    region: Rect,
    purpose: Purpose,
    out: &mut [u8],
) {
    let region = region.intersect(sprite.bounds());
    if region.is_empty() {
        return;
    }
    let w = sprite.width() as usize;
    let buf = compose_window(sprite, frame, pending, region, purpose);
    for y in 0..region.h as usize {
        let src = &buf[y * region.w as usize * 4..(y + 1) * region.w as usize * 4];
        let o = ((region.y as usize + y) * w + region.x as usize) * 4;
        out[o..o + src.len()].copy_from_slice(src);
    }
}

/// A janela `region` (já dentro do sprite) composta num buffer só dela, `region.area() * 4` bytes.
#[must_use]
pub fn compose_window(
    sprite: &Sprite,
    frame: FrameId,
    pending: Option<&Stroke>,
    region: Rect,
    purpose: Purpose,
) -> Vec<u8> {
    let mut buf = vec![0u8; region.area() * 4];
    Ctx {
        sprite,
        frame,
        pending,
        region,
        purpose,
    }
    .draw(&mut buf);
    buf
}

/// O quadro inteiro com as camadas de imagem que `pick` escolher, e só elas (a resposta dele manda sobre
/// visibilidade e referência): a porta do Export com «uma camada» ou «todas» (P9).
#[must_use]
pub fn compose_picked(sprite: &Sprite, frame: FrameId, pick: &dyn Fn(&Layer) -> bool) -> Vec<u8> {
    let region = sprite.bounds();
    let mut buf = vec![0u8; region.area() * 4];
    Ctx {
        sprite,
        frame,
        pending: None,
        region,
        purpose: Purpose::Export,
    }
    .draw_picked(pick, &mut buf);
    buf
}

/// O papel vegetal (Aseprite: *Onion Skin*). Por omissão o do Trial 1.3.18: um antes e um depois,
/// opacidade 68 a descer 28 por frame, por TRÁS do frame, sem tinta (O21 mostra-os em %: 27 e 11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Onion {
    pub prev: u32,
    pub next: u32,
    pub base: u8,
    pub step: u8,
    /// A tinta vermelha (antes) / azul (depois).
    pub tint: bool,
    /// Por cima do frame em vez de por trás.
    pub front: bool,
    /// P12 (O21 `Loop through tag frames`, ligado): dentro de uma tag os fantasmas dão a volta
    /// (o de antes do primeiro frame da tag é o último dela).
    pub loop_tag: bool,
    /// P12 (O21 `Current layer only`, desligado): os fantasmas só da camada activa
    /// ([`compose_with_onion_of`]).
    pub current_layer_only: bool,
}

impl Default for Onion {
    fn default() -> Self {
        Self {
            prev: 1,
            next: 1,
            base: 68,
            step: 28,
            tint: false,
            front: false,
            loop_tag: true,
            current_layer_only: false,
        }
    }
}

/// Um fantasma: o frame, a opacidade e se é dos de antes (tinta vermelha) ou de depois (azul).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ghost {
    pub frame: FrameId,
    pub opacity: u8,
    pub before: bool,
}

/// Os fantasmas do frame `frame`, do mais longe para o mais perto: a opacidade do frame à
/// distância `d` é `max(0, base − step·(d − 1))` (Trial `on_*.ase`); os de opacidade 0 não entram.
#[must_use]
pub fn ghosts(sprite: &Sprite, frame: FrameId, onion: &Onion) -> Vec<Ghost> {
    let Some(i) = sprite.frame_index(frame) else {
        return Vec::new();
    };
    let op = |d: u32| {
        (i32::from(onion.base) - i32::from(onion.step) * (d as i32 - 1)).clamp(0, 255) as u8
    };
    // `Loop through tag frames`: a primeira tag que contém o frame dá a volta dentro dela.
    let tag = onion
        .loop_tag
        .then(|| {
            sprite
                .tags()
                .iter()
                .find(|t| (t.from as usize..=t.to as usize).contains(&i))
        })
        .flatten()
        .map(|t| (t.from as usize, (t.to - t.from) as usize + 1));
    let mut out = Vec::new();
    for (side, reach) in [(true, onion.prev), (false, onion.next)] {
        for d in (1..=reach).rev() {
            let j = match tag {
                Some((from, len)) => {
                    let k = (i - from) as i64 + if side { -i64::from(d) } else { i64::from(d) };
                    Some(from + k.rem_euclid(len as i64) as usize).filter(|&j| j != i)
                }
                None if side => i.checked_sub(d as usize),
                None => Some(i + d as usize),
            };
            let Some(f) = j.and_then(|j| sprite.frames().get(j)) else {
                continue;
            };
            let opacity = op(d);
            if opacity > 0 {
                out.push(Ghost {
                    frame: f.id,
                    opacity,
                    before: side,
                });
            }
        }
    }
    out
}

/// A tinta do papel vegetal: o cinza pelo brilho Rec.709 `v` e depois `((255+v)/2, v/2, v/2)`
/// para os de antes, o espelho azul para os de depois (Trial, 6 144/6 144).
#[must_use]
pub fn onion_tint(c: [u8; 4], before: bool) -> [u8; 4] {
    let v = (u32::from(c[0]) * 2126 + u32::from(c[1]) * 7152 + u32::from(c[2]) * 722) / 10000;
    let (hi, lo) = (((255 + v) / 2) as u8, (v / 2) as u8);
    if before {
        [hi, lo, lo, c[3]]
    } else {
        [lo, lo, hi, c[3]]
    }
}

/// A vista de um frame com o papel vegetal: a janela `region` sobre `under` (os bytes já lá; `None`
/// = transparente), os fantasmas por trás ou por cima do frame, misturados em Normal.
#[must_use]
pub fn compose_with_onion(
    sprite: &Sprite,
    frame: FrameId,
    pending: Option<&Stroke>,
    region: Rect,
    onion: &Onion,
    under: Option<&[u8]>,
) -> Vec<u8> {
    compose_with_onion_of(sprite, frame, pending, region, onion, under, None)
}

/// O [`compose_with_onion`] com a camada activa `layer`: com `onion.current_layer_only` os
/// fantasmas são só dela (o frame continua inteiro).
#[must_use]
pub fn compose_with_onion_of(
    sprite: &Sprite,
    frame: FrameId,
    pending: Option<&Stroke>,
    region: Rect,
    onion: &Onion,
    under: Option<&[u8]>,
    layer: Option<LayerId>,
) -> Vec<u8> {
    let only = layer.filter(|_| onion.current_layer_only);
    let mut buf = under.map_or_else(|| vec![0u8; region.area() * 4], <[u8]>::to_vec);
    let ctx = Ctx {
        sprite,
        frame,
        pending,
        region,
        purpose: Purpose::View,
    };
    if onion.front {
        ctx.draw(&mut buf);
    }
    for g in ghosts(sprite, frame, onion) {
        let px = match only {
            Some(id) => {
                let mut px = vec![0u8; region.area() * 4];
                Ctx {
                    frame: g.frame,
                    ..ctx
                }
                .draw_only(id, &mut px);
                px
            }
            None => compose_window(sprite, g.frame, None, region, Purpose::View),
        };
        for (d, s) in buf
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(px.as_chunks::<4>().0)
        {
            if s[3] == 0 {
                continue;
            }
            let mut src = [s[0], s[1], s[2], s[3]];
            if onion.tint {
                src = onion_tint(src, g.before);
            }
            *d = blend(BlendMode::Normal, *d, src, g.opacity);
        }
    }
    if !onion.front {
        ctx.draw(&mut buf);
    }
    buf
}

#[derive(Clone, Copy)]
struct Ctx<'a> {
    sprite: &'a Sprite,
    frame: FrameId,
    pending: Option<&'a Stroke>,
    region: Rect,
    purpose: Purpose,
}

impl Ctx<'_> {
    /// As camadas de imagem de baixo para cima pela ordem dos cels deste frame.
    fn draw(&self, buf: &mut [u8]) {
        let mut flat = Vec::new();
        self.flatten(None, true, &mut flat);
        self.draw_flat(flat, buf);
    }

    fn draw_picked(&self, pick: &dyn Fn(&Layer) -> bool, buf: &mut [u8]) {
        let mut flat = Vec::new();
        self.flatten(None, true, &mut flat);
        let flat = flat
            .into_iter()
            .map(|(l, _)| (l, pick(l) && !l.normal))
            .collect();
        self.draw_flat(flat, buf);
    }

    /// Só a camada `id`, se visível.
    fn draw_only(&self, id: LayerId, buf: &mut [u8]) {
        let mut flat = Vec::new();
        self.flatten(None, true, &mut flat);
        let flat = flat
            .into_iter()
            .map(|(l, v)| (l, v && l.id == id))
            .collect();
        self.draw_flat(flat, buf);
    }

    fn draw_flat<'a>(&'a self, flat: Vec<(&'a Layer, bool)>, buf: &mut [u8]) {
        let z = |l: &Layer| {
            self.sprite
                .entry((l.id, self.frame))
                .map_or(0, |c| i32::from(c.z))
        };
        let mut keyed: Vec<_> = flat
            .into_iter()
            .enumerate()
            .map(|(i, (l, shown))| ((i as i32 + z(l), z(l)), l, shown))
            .collect();
        // Chave `(índice + z, z)`, estável (Trial `z_*.ase`, 10/10 exactos).
        keyed.sort_by_key(|(k, _, _)| *k);
        for (_, layer, shown) in keyed {
            if shown {
                self.image(layer, buf);
            }
        }
    }

    /// As camadas de imagem da árvore em ordem de desenho, com «vê-se». O grupo é PASS-THROUGH: os
    /// filhos misturam-se direto com o que está por baixo, e o modo do grupo não entra (Trial,
    /// g*/gm*.ase: 6 144/6 144 por modo). O índice conta as escondidas (esconder não reordena).
    fn flatten<'a>(
        &'a self,
        parent: Option<LayerId>,
        shown: bool,
        out: &mut Vec<(&'a Layer, bool)>,
    ) {
        for layer in self.sprite.layers().iter().filter(|l| l.parent == parent) {
            let me = shown && layer.visible && !(layer.reference && self.purpose != Purpose::View);
            let normal = self.purpose == Purpose::Normal;
            match layer.kind {
                LayerKind::Image | LayerKind::Tilemap { .. } => {
                    out.push((layer, me && layer.normal == normal));
                }
                LayerKind::Group { .. } => self.flatten(Some(layer.id), me, out),
            }
        }
    }

    fn image(&self, layer: &Layer, buf: &mut [u8]) {
        let key = (layer.id, self.frame);
        let stroke = self
            .pending
            .filter(|s| s.layer == layer.id && s.frame == self.frame);
        let cel = self.sprite.cel(key);
        let area = match (cel, stroke) {
            (Some(c), Some(s)) => c.bbox.union(s.rect()),
            (Some(c), None) => c.bbox,
            (None, Some(s)) => s.rect(),
            (None, None) => return,
        }
        .intersect(self.region);
        let s = self.sprite;
        let rw = self.region.w as usize;
        let bpp = s.mode().bpp();
        // A cor de cada valor: em Indexed uma tabela de 256 (a LUT), senão a conversão directa.
        let lut: Option<Vec<[u8; 4]>> = (s.mode() == ColorMode::Indexed)
            .then(|| (0..=255u32).map(|v| s.rgba_of_value(v)).collect());
        let srect = stroke.map(Stroke::rect);
        // A opacidade efectiva é `mul(camada, cel)` (Trial `co_*.ase`).
        let opacity = cel.map_or(layer.opacity, |c| {
            crate::blend::mul(u32::from(layer.opacity), u32::from(c.opacity)) as u8
        });
        // O caminho rápido (medido: o Normal opaco é a maioria das células) copia em vez de misturar.
        let copia = layer.blend == BlendMode::Normal && opacity == 255;
        for y in area.y..area.y + area.h as i32 {
            let linha = cel.filter(|c| y >= c.bbox.y && y < c.bbox.y + c.bbox.h as i32);
            for x in area.x..area.x + area.w as i32 {
                let traco = srect
                    .filter(|r| r.contains(x, y))
                    .and_then(|_| stroke?.value_at(s, x, y));
                let v = match (traco, linha) {
                    (Some(v), _) => v,
                    (None, Some(c)) if x >= c.bbox.x && x < c.bbox.x + c.bbox.w as i32 => {
                        let i = ((y - c.bbox.y) as usize * c.bbox.w as usize
                            + (x - c.bbox.x) as usize)
                            * bpp;
                        s.read_value(&c.data[i..i + bpp])
                    }
                    _ => continue,
                };
                let src = match &lut {
                    Some(t) => t[(v & 0xff) as usize],
                    None => s.rgba_of_value(v),
                };
                if src[3] == 0 {
                    continue;
                }
                let i = ((y - self.region.y) as usize * rw + (x - self.region.x) as usize) * 4;
                if copia && src[3] == 255 {
                    buf[i..i + 4].copy_from_slice(&src);
                    continue;
                }
                let dst = [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]];
                buf[i..i + 4].copy_from_slice(&blend(layer.blend, dst, src, opacity));
            }
        }
    }
}

/// Source-over de alfa reto, arredondado (`floor(x + 0.5)`).
pub(crate) fn over(dst: [u8; 4], src: [u8; 4]) -> [u8; 4] {
    let sa = u32::from(src[3]);
    if sa == 255 || dst[3] == 0 {
        return src;
    }
    let da = u32::from(dst[3]) * (255 - sa);
    // Alfas na escala 255²: oa = sa·255 + da·(255−sa).
    let oa = sa * 255 + da;
    let mut o = [0u8; 4];
    for k in 0..3 {
        let num = u32::from(src[k]) * sa * 255 + u32::from(dst[k]) * da;
        o[k] = ((num + oa / 2) / oa) as u8;
    }
    o[3] = ((oa + 127) / 255) as u8;
    o
}

/// O que o conta-gotas lê (Aseprite: *Sample*).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sample {
    /// A imagem composta (RGBA/Gray: a cor misturada; Indexed: o índice da camada visível mais
    /// alta que não é transparente ali).
    AllLayers,
    /// Só a camada dada.
    Layer(LayerId),
}

/// O valor (no modo do sprite) na célula `(x, y)` do frame; `None` fora do sprite.
#[must_use]
pub fn pick(sprite: &Sprite, frame: FrameId, x: i32, y: i32, sample: Sample) -> Option<u32> {
    if !sprite.bounds().contains(x, y) {
        return None;
    }
    match sample {
        Sample::Layer(l) => Some(sprite.value_at((l, frame), x, y)),
        Sample::AllLayers => match sprite.mode() {
            ColorMode::Indexed => Some(
                sprite
                    .layers()
                    .iter()
                    .rev()
                    .filter(|l| !l.is_group() && !l.normal && sprite.is_shown(l.id))
                    .map(|l| sprite.value_at((l.id, frame), x, y))
                    .find(|&v| v != sprite.transparent_value())
                    .unwrap_or_else(|| sprite.transparent_value()),
            ),
            _ => {
                let px = compose_window(sprite, frame, None, Rect::new(x, y, 1, 1), Purpose::View);
                Some(sprite.value_of_rgba([px[0], px[1], px[2], px[3]]))
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::over;

    #[test]
    fn over_is_identity_on_opaque_source_and_on_empty_destination() {
        assert_eq!(over([1, 2, 3, 4], [9, 8, 7, 255]), [9, 8, 7, 255]);
        assert_eq!(over([1, 2, 3, 0], [9, 8, 7, 100]), [9, 8, 7, 100]);
    }

    #[test]
    fn half_over_opaque_lands_halfway() {
        assert_eq!(
            over([0, 0, 0, 255], [255, 255, 255, 128]),
            [128, 128, 128, 255]
        );
    }
}
