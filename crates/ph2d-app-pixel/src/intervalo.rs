//! **O intervalo da linha do tempo** (P6h): os quadros / cels / filas escolhidos e os comandos que
//! agem sobre eles, pela lei medida no Aseprite Trial
//! (`docs/PixelArt/ferramentas/oraculo_timeline.md`, fixtura `oracle_trial_timeline.json`).
//!
//! O intervalo é estado de VISTA (não é passo do desfazer); os ids que deixam de existir caem na
//! leitura ([`Doc::range_frames`]), nunca ficam a apontar para o vazio.

use ph2d_editor_core::pixel_panel::{PixelRange, PixelRangeKind as K, PixelRequest};
use ph2d_pixel_core::{CelKey, FrameId, LayerId, frames, layers};

use crate::PixelApp;
use crate::state::Doc;

impl Doc {
    /// Os quadros do intervalo que ainda existem, na ordem da linha do tempo (vazio sem intervalo
    /// de quadros/cels; as filas de camadas valem por todos os quadros).
    pub(crate) fn range_frames(&self) -> Vec<FrameId> {
        let all = self.sprite.frames().iter().map(|f| f.id);
        match self.range.kind {
            K::None => Vec::new(),
            K::Layers => all.collect(),
            K::Frames | K::Cels => all.filter(|f| self.range.frames.contains(&f.0)).collect(),
        }
    }

    /// As camadas de imagem do intervalo que ainda existem (todas num intervalo de quadros).
    fn range_layers(&self) -> Vec<LayerId> {
        let all = self.sprite.layers().iter().filter(|l| !l.is_group());
        match self.range.kind {
            K::None => Vec::new(),
            K::Frames => all.map(|l| l.id).collect(),
            K::Cels | K::Layers => all
                .filter(|l| self.range.layers.contains(&l.id.0))
                .map(|l| l.id)
                .collect(),
        }
    }

    /// Os cels (chaves) cobertos pelo intervalo.
    pub(crate) fn range_keys(&self) -> Vec<CelKey> {
        let fs = self.range_frames();
        let ls = self.range_layers();
        ls.iter()
            .flat_map(|&l| fs.iter().map(move |&f| (l, f)))
            .collect()
    }

    /// O intervalo como o painel o pinta (só ids que existem; vazio se nada sobrou).
    pub(crate) fn range_info(&self) -> PixelRange {
        let frames: Vec<u32> = self.range_frames().iter().map(|f| f.0).collect();
        let layers: Vec<u32> = self.range_layers().iter().map(|l| l.0).collect();
        let vazio = match self.range.kind {
            K::None => true,
            K::Frames => frames.is_empty(),
            K::Cels | K::Layers => frames.is_empty() || layers.is_empty(),
        };
        if vazio {
            return PixelRange::default();
        }
        PixelRange {
            kind: self.range.kind,
            layers,
            frames,
        }
    }

    /// Os quadros a que um comando de quadro se aplica: os do intervalo, senão o activo.
    pub(crate) fn alvo_quadros(&self) -> Vec<FrameId> {
        let fs = self.range_frames();
        if fs.is_empty() { vec![self.frame] } else { fs }
    }

    /// As posições `(de, até)` do envelope do intervalo com 2+ quadros (o Reverse Frames: com
    /// quadros soltos inverte do 1.º ao último, os de permeio também — Trial 01/11, `reverse_picks`).
    pub(crate) fn range_span(&self) -> Option<(usize, usize)> {
        let pos: Vec<usize> = self
            .range_frames()
            .iter()
            .filter_map(|&f| self.sprite.frame_index(f))
            .collect();
        let (&a, &b) = (pos.first()?, pos.last()?);
        (a < b).then_some((a, b))
    }
}

impl PixelApp {
    /// O intervalo, a largada do bloco arrastado e a contínua; `false` = não é daqui.
    pub(crate) fn aplica_intervalo(&mut self, r: &PixelRequest) -> bool {
        let Some(key) = self.chave else {
            return false;
        };
        match r {
            PixelRequest::Range(novo) => {
                if let Some(doc) = self.docs.get_mut(&key) {
                    doc.range = novo.clone();
                    doc.range = doc.range_info();
                }
                true
            }
            PixelRequest::RangeMove { before, copy } => {
                let Some(doc) = self.docs.get_mut(&key) else {
                    return true;
                };
                let ids = doc.range_frames();
                let (op, novos) = if *copy {
                    frames::copy_block(&mut doc.sprite, &ids, *before as usize).unzip()
                } else {
                    (
                        frames::move_block(&doc.sprite, &ids, *before as usize),
                        None,
                    )
                };
                if let Some(op) = op
                    && self.commit(key, op).is_ok()
                    && let (Some(novos), Some(doc)) = (novos, self.docs.get_mut(&key))
                {
                    doc.range.frames = novos.iter().map(|f| f.0).collect();
                }
                true
            }
            PixelRequest::CelsMove {
                pairs,
                beyond,
                copy,
            } => {
                let de = |l: u32, f: u32| (LayerId(l), FrameId(f));
                let pares: Vec<_> = pairs
                    .iter()
                    .map(|&((l0, f0), (l1, f1))| (de(l0, f0), de(l1, f1)))
                    .collect();
                let alem: Vec<_> = beyond
                    .iter()
                    .map(|&((l0, f0), (l1, k))| (de(l0, f0), (LayerId(l1), k as usize)))
                    .collect();
                let feito = self
                    .docs
                    .get_mut(&key)
                    .and_then(|d| frames::move_cels_grow(&mut d.sprite, &pares, &alem, *copy));
                if feito.is_none() {
                    crate::tiles::recusou(self, key, None, "pixel.tiles.refuse.cel_drag");
                }
                if let Some((op, novos)) = feito
                    && self.commit(key, op).is_ok()
                    && let Some(doc) = self.docs.get_mut(&key)
                {
                    let pares = pares
                        .into_iter()
                        .chain(alem.iter().map(|&(a, (l, k))| (a, (l, novos[k]))));
                    let mut r = PixelRange {
                        kind: K::Cels,
                        ..PixelRange::default()
                    };
                    for (_, (l, f)) in pares {
                        r.layers.extend((!r.layers.contains(&l.0)).then_some(l.0));
                        r.frames.extend((!r.frames.contains(&f.0)).then_some(f.0));
                    }
                    doc.range = r;
                }
                true
            }
            // O ícone da contínua NÃO é passo do desfazer (medido: um Ctrl+Z depois dele desfaz o
            // passo de antes): aplica-se ao sprite sem passar pela história.
            PixelRequest::LayerContinuous(id) => {
                let stamp = self.stamp();
                if let Some(doc) = self.docs.get_mut(&key)
                    && let Some(op) =
                        layers::edit(&doc.sprite, LayerId(*id), |l| l.continuous = !l.continuous)
                    && ph2d_pixel_core::apply(&mut doc.sprite, op).is_ok()
                {
                    doc.stamp = stamp;
                }
                true
            }
            _ => false,
        }
    }
}
