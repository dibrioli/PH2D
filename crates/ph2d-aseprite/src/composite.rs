//! **Das camadas ao quadro** — o passo que transforma cels espalhados numa imagem RGBA8.
//!
//! ⚠️ **A ordem do ficheiro É a ordem de baixo para cima.** As camadas chegam pela ordem em que
//! foram declaradas, e essa ordem é a pilha do Aseprite; compor por qualquer outra dá um desenho
//! plausível e errado.
//!
//! ⚠️ **Um cel LIGADO não é um cel vazio.** O Aseprite guarda um quadro repetido como uma
//! referência ao quadro onde ele foi desenhado (tipo 1); tratá-lo como ausente faz a animação
//! piscar exactamente nos quadros que o artista não redesenhou — que são a maioria deles.

use crate::blend::{BlendMode, blend};
use crate::read::Reader;
use crate::{AseDoc, AseError, AseFrame, AseTag};

const CHUNK_LAYER: u16 = 0x2004;
const CHUNK_CEL: u16 = 0x2005;
const CHUNK_PALETTE: u16 = 0x2019;
const CHUNK_TAGS: u16 = 0x2018;

/// Uma camada declarada no ficheiro.
struct Layer {
    name: String,
    visible: bool,
    /// `true` = grupo (não tem cels; manda na visibilidade dos filhos).
    group: bool,
    /// `true` = tilemap, que este leitor não sabe compor.
    tilemap: bool,
    child_level: u16,
    blend: BlendMode,
    opacity: u8,
}

/// Um cel pronto a compor.
struct Cel {
    layer: usize,
    x: i16,
    y: i16,
    opacity: u8,
    w: u16,
    h: u16,
    /// RGBA8 já convertido da profundidade do ficheiro.
    rgba: Vec<u8>,
}

/// O estado do decodificador enquanto ele atravessa o ficheiro.
pub(crate) struct Build {
    width: u16,
    height: u16,
    depth: u16,
    transparent_index: u8,
    palette: Vec<[u8; 4]>,
    layers: Vec<Layer>,
    /// Os cels do quadro corrente.
    cels: Vec<Cel>,
    /// Para os cels LIGADOS: `(layer, frame) -> índice em `linkable``.
    linkable: Vec<(usize, usize, Cel)>,
    duration_ms: u16,
    frames: Vec<AseFrame>,
    tags: Vec<AseTag>,
    notes: Vec<String>,
}

impl Build {
    pub(crate) fn new(width: u16, height: u16, depth: u16, transparent_index: u8) -> Self {
        Self {
            width,
            height,
            depth,
            transparent_index,
            palette: Vec::new(),
            layers: Vec::new(),
            cels: Vec::new(),
            linkable: Vec::new(),
            duration_ms: 100,
            frames: Vec::new(),
            tags: Vec::new(),
            notes: Vec::new(),
        }
    }

    pub(crate) fn begin_frame(&mut self, duration_ms: u16) {
        self.duration_ms = duration_ms;
        self.cels.clear();
    }

    /// Uma nota só entra UMA vez: um ficheiro com trinta cels de tilemap não deve produzir trinta
    /// linhas iguais para o artista ler.
    fn note(&mut self, line: String) {
        if !self.notes.contains(&line) {
            self.notes.push(line);
        }
    }

    pub(crate) fn chunk(
        &mut self,
        ctype: u16,
        r: &mut Reader<'_>,
        frame: usize,
    ) -> Result<(), AseError> {
        match ctype {
            CHUNK_LAYER => self.layer(r),
            CHUNK_CEL => self.cel(r, frame)?,
            CHUNK_PALETTE => self.palette(r),
            CHUNK_TAGS => self.tags(r),
            _ => {}
        }
        Ok(())
    }

    fn layer(&mut self, r: &mut Reader<'_>) {
        let Some(flags) = r.u16() else { return };
        let Some(kind) = r.u16() else { return };
        let Some(child_level) = r.u16() else { return };
        let _ = r.skip(4); // largura/altura default, que a spec manda ignorar
        let Some(blend_raw) = r.u16() else { return };
        let Some(opacity) = r.u8() else { return };
        let _ = r.skip(3);
        let name = r.string().unwrap_or_default();
        let blend = match BlendMode::from_file(blend_raw) {
            Some(b) => b,
            None => {
                self.note(format!(
                    "layer \"{name}\" uses a blend mode this version does not know ({blend_raw}) — composited as Normal"
                ));
                BlendMode::Normal
            }
        };
        self.layers.push(Layer {
            name,
            visible: flags & 1 != 0,
            group: kind == 1,
            tilemap: kind == 2,
            child_level,
            blend,
            opacity,
        });
    }

    fn palette(&mut self, r: &mut Reader<'_>) {
        let Some(size) = r.u32() else { return };
        let Some(first) = r.u32() else { return };
        let Some(last) = r.u32() else { return };
        let _ = r.skip(8);
        if self.palette.len() < size as usize {
            self.palette.resize(size as usize, [0, 0, 0, 0]);
        }
        for i in first..=last.max(first) {
            let Some(flags) = r.u16() else { return };
            let (Some(red), Some(green), Some(blue), Some(alpha)) =
                (r.u8(), r.u8(), r.u8(), r.u8())
            else {
                return;
            };
            if flags & 1 != 0 {
                let _ = r.string(); // o nome da cor, que não usamos
            }
            if let Some(slot) = self.palette.get_mut(i as usize) {
                *slot = [red, green, blue, alpha];
            }
        }
    }

    fn tags(&mut self, r: &mut Reader<'_>) {
        let Some(n) = r.u16() else { return };
        let _ = r.skip(8);
        for _ in 0..n {
            let (Some(from), Some(to), Some(direction)) = (r.u16(), r.u16(), r.u8()) else {
                return;
            };
            let repeat = r.u16().unwrap_or(0);
            let _ = r.skip(6);
            let _ = r.skip(3); // a cor da tag, obsoleta
            let _ = r.u8();
            let name = r.string().unwrap_or_default();
            self.tags.push(AseTag {
                name,
                from,
                to,
                direction,
                repeat,
            });
        }
    }

    fn cel(&mut self, r: &mut Reader<'_>, frame: usize) -> Result<(), AseError> {
        let Some(layer) = r.u16().map(usize::from) else {
            return Ok(());
        };
        let (Some(x), Some(y), Some(opacity), Some(kind)) = (r.i16(), r.i16(), r.u8(), r.u16())
        else {
            return Ok(());
        };
        let _ = r.skip(7); // z-index (1.3) + reservados
        match kind {
            // Cru e comprimido só diferem no zlib.
            0 | 2 => {
                let (Some(w), Some(h)) = (r.u16(), r.u16()) else {
                    return Ok(());
                };
                let need = usize::from(w) * usize::from(h) * usize::from(self.depth / 8);
                let raw = if kind == 0 {
                    r.rest().to_vec()
                } else {
                    miniz_oxide::inflate::decompress_to_vec_zlib(r.rest())
                        .map_err(|_| AseError::BadZlib(frame))?
                };
                if raw.len() < need {
                    return Err(AseError::Truncated("cel pixels"));
                }
                let rgba = self.to_rgba(&raw[..need]);
                self.push_cel(Cel {
                    layer,
                    x,
                    y,
                    opacity,
                    w,
                    h,
                    rgba,
                });
            }
            // LIGADO: aponta para o cel que a mesma camada tem noutro quadro.
            1 => {
                let Some(src_frame) = r.u16().map(usize::from) else {
                    return Ok(());
                };
                if let Some((_, _, c)) = self
                    .linkable
                    .iter()
                    .find(|(l, f, _)| *l == layer && *f == src_frame)
                {
                    let copy = Cel {
                        layer,
                        x: c.x,
                        y: c.y,
                        opacity,
                        w: c.w,
                        h: c.h,
                        rgba: c.rgba.clone(),
                    };
                    self.cels.push(copy);
                }
            }
            3 => {
                let name = self
                    .layers
                    .get(layer)
                    .map_or_else(|| "?".to_owned(), |l| l.name.clone());
                self.note(format!(
                    "layer \"{name}\" is a tilemap — tilemaps are not imported, that layer is blank"
                ));
            }
            _ => {}
        }
        Ok(())
    }

    /// Guarda o cel no quadro E na tabela dos que podem ser ligados. ⚠️ Um cel só pode ser ligado
    /// depois de existir, e a spec garante que o quadro-fonte vem antes — mas quem escreve o
    /// ficheiro é outro programa, e um alvo em falta aqui é um quadro em branco, não um pânico.
    fn push_cel(&mut self, cel: Cel) {
        let frame = self.frames.len();
        self.linkable.push((
            cel.layer,
            frame,
            Cel {
                layer: cel.layer,
                x: cel.x,
                y: cel.y,
                opacity: cel.opacity,
                w: cel.w,
                h: cel.h,
                rgba: cel.rgba.clone(),
            },
        ));
        self.cels.push(cel);
    }

    /// Converte a profundidade do ficheiro para RGBA8 de alfa reto.
    fn to_rgba(&self, raw: &[u8]) -> Vec<u8> {
        match self.depth {
            32 => raw.to_vec(),
            // Escala de cinza + alfa: o cinza vai aos três canais.
            16 => raw
                .as_chunks::<2>()
                .0
                .iter()
                .flat_map(|p| [p[0], p[0], p[0], p[1]])
                .collect(),
            // Indexado: a paleta manda, e o índice transparente é buraco.
            _ => raw
                .iter()
                .flat_map(|&i| {
                    if i == self.transparent_index {
                        return [0, 0, 0, 0];
                    }
                    match self.palette.get(usize::from(i)) {
                        Some(c) => *c,
                        // Sem paleta (ficheiro antigo, chunk 0x0004): uma rampa de cinza é
                        // legível e óbvia como falha — melhor que preto sólido.
                        None => [i, i, i, 255],
                    }
                })
                .collect(),
        }
    }

    /// Compõe o quadro corrente e arquiva-o.
    pub(crate) fn end_frame(&mut self, _index: usize) -> Result<(), AseError> {
        let (w, h) = (usize::from(self.width), usize::from(self.height));
        let mut out = vec![0_u8; w * h * 4];
        // A visibilidade de um grupo cai sobre os filhos dele: um nível de aninhamento por entrada.
        let mut hidden_at: Option<u16> = None;
        let mut visible = vec![false; self.layers.len()];
        for (i, l) in self.layers.iter().enumerate() {
            if let Some(level) = hidden_at {
                if l.child_level > level {
                    continue; // ainda dentro do grupo escondido
                }
                hidden_at = None;
            }
            if !l.visible {
                if l.group {
                    hidden_at = Some(l.child_level);
                }
                continue;
            }
            visible[i] = !l.group && !l.tilemap;
        }
        // Os cels chegam na ordem das camadas dentro de cada quadro, e as camadas estão de baixo
        // para cima — compor pela ordem de chegada é compor a pilha.
        for cel in &self.cels {
            let Some(layer) = self.layers.get(cel.layer) else {
                continue;
            };
            if !visible.get(cel.layer).copied().unwrap_or(false) {
                continue;
            }
            let opacity = ((u32::from(layer.opacity) * u32::from(cel.opacity) + 127) / 255) as u8;
            for row in 0..usize::from(cel.h) {
                let dy = i32::from(cel.y) + row as i32;
                if dy < 0 || dy as usize >= h {
                    continue;
                }
                for col in 0..usize::from(cel.w) {
                    let dx = i32::from(cel.x) + col as i32;
                    if dx < 0 || dx as usize >= w {
                        continue;
                    }
                    let si = (row * usize::from(cel.w) + col) * 4;
                    let Some(src) = cel.rgba.get(si..si + 4) else {
                        continue;
                    };
                    let di = (dy as usize * w + dx as usize) * 4;
                    let back = [out[di], out[di + 1], out[di + 2], out[di + 3]];
                    let px = blend(layer.blend, back, [src[0], src[1], src[2], src[3]], opacity);
                    out[di..di + 4].copy_from_slice(&px);
                }
            }
        }
        self.frames.push(AseFrame {
            duration_ms: self.duration_ms,
            rgba: out,
        });
        Ok(())
    }

    pub(crate) fn finish(self) -> AseDoc {
        AseDoc {
            width: self.width,
            height: self.height,
            frames: self.frames,
            tags: self.tags,
            notes: self.notes,
        }
    }
}
