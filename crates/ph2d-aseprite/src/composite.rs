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
use crate::file::{AseCel, AseCelBody, AseFile, AseLayer, KIND_GROUP, KIND_TILEMAP, LAYER_VISIBLE};
use crate::{AseDoc, AseError, AseFrame};

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

/// O estado da composição enquanto ela atravessa os quadros.
struct Build<'a> {
    width: u16,
    height: u16,
    depth: u16,
    transparent_index: u8,
    palette: &'a [[u8; 4]],
    layers: Vec<Layer>,
    /// Os cels do quadro corrente.
    cels: Vec<Cel>,
    /// Para os cels LIGADOS: `(layer, frame) -> cel`.
    linkable: Vec<(usize, usize, Cel)>,
    duration_ms: u16,
    frames: Vec<AseFrame>,
    notes: Vec<String>,
}

/// O documento composto: quadros RGBA + autoria.
pub(crate) fn compose(file: &AseFile) -> Result<AseDoc, AseError> {
    let mut b = Build {
        width: file.width,
        height: file.height,
        depth: file.depth,
        transparent_index: file.transparent_index,
        palette: &file.palette,
        layers: Vec::with_capacity(file.layers.len()),
        cels: Vec::new(),
        linkable: Vec::new(),
        duration_ms: 100,
        frames: Vec::with_capacity(file.frames.len()),
        notes: Vec::new(),
    };
    for l in &file.layers {
        b.layer(l);
    }
    for (index, f) in file.frames.iter().enumerate() {
        if let Some(p) = &f.palette {
            b.palette = p;
        }
        b.duration_ms = f.duration_ms;
        b.cels.clear();
        for c in &f.cels {
            b.cel(c);
        }
        b.end_frame(index)?;
    }
    Ok(AseDoc {
        width: file.width,
        height: file.height,
        frames: b.frames,
        tags: file.tags.clone(),
        notes: b.notes,
    })
}

impl Build<'_> {
    /// Uma nota só entra UMA vez: um ficheiro com trinta cels de tilemap não deve produzir trinta
    /// linhas iguais para o artista ler.
    fn note(&mut self, line: String) {
        if !self.notes.contains(&line) {
            self.notes.push(line);
        }
    }

    fn layer(&mut self, l: &AseLayer) {
        let blend = match BlendMode::from_file(l.blend) {
            Some(b) => b,
            None => {
                self.note(format!(
                    "layer \"{}\" uses a blend mode this version does not know ({}) — composited as Normal",
                    l.name, l.blend
                ));
                BlendMode::Normal
            }
        };
        self.layers.push(Layer {
            name: l.name.clone(),
            visible: l.flags & LAYER_VISIBLE != 0,
            group: l.kind == KIND_GROUP,
            tilemap: l.kind == KIND_TILEMAP,
            child_level: l.child_level,
            blend,
            opacity: l.opacity,
        });
    }

    fn cel(&mut self, c: &AseCel) {
        let layer = usize::from(c.layer);
        match &c.body {
            AseCelBody::Image { w, h, pixels } => {
                let rgba = self.to_rgba(pixels);
                self.push_cel(Cel {
                    layer,
                    x: c.x,
                    y: c.y,
                    opacity: c.opacity,
                    w: *w,
                    h: *h,
                    rgba,
                });
            }
            // LIGADO: aponta para o cel que a mesma camada tem noutro quadro.
            AseCelBody::Linked { frame } => {
                let src_frame = usize::from(*frame);
                if let Some((_, _, o)) = self
                    .linkable
                    .iter()
                    .find(|(l, f, _)| *l == layer && *f == src_frame)
                {
                    let copy = Cel {
                        layer,
                        x: o.x,
                        y: o.y,
                        opacity: c.opacity,
                        w: o.w,
                        h: o.h,
                        rgba: o.rgba.clone(),
                    };
                    self.cels.push(copy);
                }
            }
            AseCelBody::Tilemap(_) => {
                let name = self
                    .layers
                    .get(layer)
                    .map_or_else(|| "?".to_owned(), |l| l.name.clone());
                self.note(format!(
                    "layer \"{name}\" is a tilemap — tilemaps are not imported, that layer is blank"
                ));
            }
        }
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
    fn end_frame(&mut self, _index: usize) -> Result<(), AseError> {
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
}
