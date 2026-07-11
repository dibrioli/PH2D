//! [`FlipLayer`] — uma camada de um objeto Flip: um mapa de frames + as
//! propriedades no idioma do Painter (blend/opacity/visibility/lock/mask).
//!
//! A camada é dona do **mapa de frames** e da mecânica pura dele (inserir/
//! remover chave, resolver o desenho ativo por hold). Ela **não** conhece os
//! desenhos (isso é do [`crate::FlipObject`], que coordena o refcount) — os
//! métodos daqui manipulam só `Option<DrawingId>` e a topologia de chaves.
//!
//! A mecânica de `add_frame`/`remove_frame`/`remove_leading_end_frames` é
//! portada 1:1 do Grease Pencil 5.2 (`blenkernel/intern/grease_pencil.cc`, ver
//! `02_referencia §1`), clean-room.

use crate::frame::{FlipFrame, Hold, KeyKind};
use crate::ids::{DrawingId, Frame, LayerId};
use ph2d_painter_effects::BlendMode;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::Bound;

/// Uma máscara de camada: a camada `source` mascara esta. Espelha o
/// `LayerMask` do GP (referência por camada), com o flag de inversão.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerMask {
    /// A camada que fornece a máscara.
    pub source: LayerId,
    /// Inverte a máscara (o de fora vira o visível).
    pub invert: bool,
}

/// Uma camada: mapa de frames + propriedades de composição.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FlipLayer {
    /// Id estável (referências de máscara apontam para cá).
    pub id: LayerId,
    /// Nome exibido na Hierarquia / tira.
    pub name: String,
    /// As chaves: `frame de início → conteúdo`. `BTreeMap` = `sorted_keys` de
    /// graça, e a amostragem por hold é `range(..=frame).next_back()`.
    frames: BTreeMap<Frame, FlipFrame>,
    /// Opacidade da camada `[0,1]` (default `1.0`).
    pub opacity: f32,
    /// Modo de blend — o mesmo enum de 22 modos do compositor do Painter.
    pub blend: BlendMode,
    /// Visível (o olho da Hierarquia).
    pub visible: bool,
    /// Travada para edição (o cadeado).
    pub locked: bool,
    /// Máscaras que agem sobre esta camada (vazio = sem máscara).
    pub masks: Vec<LayerMask>,
}

impl FlipLayer {
    /// Camada vazia com defaults do idioma Painter.
    #[must_use]
    pub fn new(id: LayerId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            frames: BTreeMap::new(),
            opacity: 1.0,
            blend: BlendMode::default(),
            visible: true,
            locked: false,
            masks: Vec::new(),
        }
    }

    /// O mapa de frames (chaves em ordem crescente).
    #[must_use]
    pub fn frames(&self) -> &BTreeMap<Frame, FlipFrame> {
        &self.frames
    }

    /// Acesso mutável ao mapa (para o `FlipObject` remapear na compactação). As
    /// chaves não devem mudar por aqui — só os valores.
    pub(crate) fn frames_mut(&mut self) -> &mut BTreeMap<Frame, FlipFrame> {
        &mut self.frames
    }

    /// Sem nenhuma chave.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// **O desenho ativo no quadro `frame`** (semântica de HOLD). A maior chave
    /// `≤ frame`; se for uma sentinela de fim, nada aparece (`None`).
    ///
    /// `range(..=frame).next_back()` = `upper_bound`+recua-um do GP. O `.drawing`
    /// já é `None` para end-frames, então o hold sai de graça.
    #[must_use]
    pub fn drawing_at(&self, frame: Frame) -> Option<DrawingId> {
        self.frames
            .range(..=frame)
            .next_back()
            .and_then(|(_, f)| f.drawing)
    }

    /// A duração (em quadros) da chave em `key`: distância até a próxima chave, ou
    /// `0` se é a última (segura indefinidamente). Espelha `get_frame_duration_at`.
    #[must_use]
    pub fn duration_at(&self, key: Frame) -> u32 {
        if !self.frames.contains_key(&key) {
            return 0;
        }
        match self.next_key(key) {
            Some(next) => (next - key).max(0) as u32,
            None => 0,
        }
    }

    /// A menor chave estritamente maior que `key`.
    fn next_key(&self, key: Frame) -> Option<Frame> {
        self.frames
            .range((Bound::Excluded(key), Bound::Unbounded))
            .next()
            .map(|(&k, _)| k)
    }

    /// Remove as end-frames CONSECUTIVAS logo após `after` (na ordem de chaves).
    /// Espelha `remove_leading_end_frames_in_range` — para quando acha uma chave
    /// que não é sentinela.
    fn remove_leading_end_frames(&mut self, after: Frame) {
        let keys: Vec<Frame> = self
            .frames
            .range((Bound::Excluded(after), Bound::Unbounded))
            .map(|(&k, _)| k)
            .collect();
        for k in keys {
            match self.frames.get(&k) {
                Some(f) if f.is_end() => {
                    self.frames.remove(&k);
                }
                _ => break,
            }
        }
    }

    /// Insere uma chave em `key` apontando `drawing`, com o hold dado. Espelha
    /// `Layer::add_frame` (`grease_pencil.cc:1535`).
    ///
    /// Regras (clean-room do GP):
    /// - `key` livre → insere; `key` já é sentinela → sobrescreve; `key` já é
    ///   chave real → **falha** (`false`, nada muda).
    /// - `Hold::Fixed(dur)`: se a próxima chave já está exatamente em `key+dur`,
    ///   nada mais; senão remove sentinelas líderes e, se a próxima está além de
    ///   `key+dur` (ou não há), cria uma sentinela em `key+dur`.
    /// - `Hold::Implicit` (ou `Fixed(0)`): marca `implicit_hold`, sem sentinela.
    ///
    /// Devolve `true` se a chave foi inserida.
    pub(crate) fn add_frame(
        &mut self,
        key: Frame,
        drawing: Option<DrawingId>,
        kind: KeyKind,
        hold: Hold,
    ) -> bool {
        // add_frame_internal: chave real presente = falha; sentinela = sobrescreve.
        if let Some(f) = self.frames.get(&key)
            && !f.is_end()
        {
            return false;
        }
        let implicit_hold = hold.is_implicit();
        self.frames.insert(
            key,
            FlipFrame {
                drawing,
                implicit_hold,
                kind,
            },
        );
        let duration = hold.duration();
        let end_key = key.saturating_add(i32::try_from(duration).unwrap_or(i32::MAX));

        // Se a próxima chave (original) coincide com o fim, já está delimitado.
        if self.next_key(key) == Some(end_key) {
            return true;
        }
        self.remove_leading_end_frames(key);
        if duration == 0 {
            return true; // implicit hold
        }
        // Duração fixa: cria sentinela se a próxima (pós-limpeza) está além do fim.
        match self.next_key(key) {
            Some(next) if next <= end_key => {}
            _ => {
                self.frames.insert(end_key, FlipFrame::end());
            }
        }
        true
    }

    /// Remove a chave em `key`. Espelha `Layer::remove_frame`
    /// (`grease_pencil.cc:1565`).
    ///
    /// Devolve:
    /// - `None` se `key` não existe (no-op);
    /// - `Some(unref)` se removeu/converteu — `unref` é o desenho que perdeu a
    ///   referência (a sentinela de fim tem `None`). O chamador
    ///   ([`crate::FlipObject`]) decrementa o refcount desse desenho.
    ///
    /// A sutileza do GP: se o quadro anterior tem duração fixa (não implicit e
    /// não sentinela), não dá pra só apagar — o slot vira uma sentinela de fim,
    /// para o anterior não vazar.
    pub(crate) fn remove_frame(&mut self, key: Frame) -> Option<Option<DrawingId>> {
        let removed = self.frames.get(&key)?.drawing; // None → key ausente

        if self.frames.len() == 1 {
            self.frames.remove(&key);
            return Some(removed);
        }
        // Limpa sentinelas líderes logo após a chave removida (se houver próxima).
        if self.next_key(key).is_some() {
            self.remove_leading_end_frames(key);
        }
        // Anterior com duração fixa → converte o slot em sentinela de fim.
        if let Some((_, prev)) = self
            .frames
            .range((Bound::Unbounded, Bound::Excluded(key)))
            .next_back()
            && !prev.implicit_hold
            && !prev.is_end()
        {
            self.frames.insert(key, FlipFrame::end());
            return Some(removed);
        }
        self.frames.remove(&key);
        Some(removed)
    }

    /// Move o conteúdo da chave `from` para `to`, preservando desenho/hold/tipo.
    /// Relocação simples (sem recomputar duração nem mexer em sentinelas): recusa
    /// (`false`) se `from` não existe ou `to` já é uma chave REAL. Uma sentinela
    /// em `to` é sobrescrita. O refcount não muda (é o mesmo desenho).
    pub(crate) fn relocate_frame(&mut self, from: Frame, to: Frame) -> bool {
        if from == to || !self.frames.contains_key(&from) {
            return from == to && self.frames.contains_key(&from);
        }
        if let Some(f) = self.frames.get(&to)
            && !f.is_end()
        {
            return false; // não sobrescreve chave real
        }
        let Some(frame) = self.frames.remove(&from) else {
            return false;
        };
        self.frames.insert(to, frame);
        true
    }

    /// Mata todas as chaves (usado ao remover a camada — o objeto zera o refcount
    /// via `recompute_users` depois).
    pub(crate) fn clear_frames(&mut self) {
        self.frames.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(n: u32) -> Option<DrawingId> {
        Some(DrawingId(n))
    }

    fn layer() -> FlipLayer {
        FlipLayer::new(LayerId(0), "L")
    }

    /// T0.3: a tabela canônica do GP `{0:d0, 5:d1, 10:end, 12:d2}` — d1 aparece
    /// 5..9, nada 10..11, d2 de 12 em diante; nada antes de 0.
    #[test]
    fn drawing_at_follows_hold_and_end_sentinel() {
        let mut l = layer();
        // Monta o mapa diretamente (Implicit em todas, mais uma sentinela).
        l.frames.insert(
            0,
            FlipFrame {
                drawing: d(0),
                implicit_hold: true,
                kind: KeyKind::Keyframe,
            },
        );
        l.frames.insert(
            5,
            FlipFrame {
                drawing: d(1),
                implicit_hold: true,
                kind: KeyKind::Keyframe,
            },
        );
        l.frames.insert(10, FlipFrame::end());
        l.frames.insert(
            12,
            FlipFrame {
                drawing: d(2),
                implicit_hold: true,
                kind: KeyKind::Keyframe,
            },
        );

        assert_eq!(l.drawing_at(-1), None, "nada antes da 1ª chave");
        assert_eq!(l.drawing_at(0), d(0));
        assert_eq!(l.drawing_at(4), d(0));
        for f in 5..=9 {
            assert_eq!(l.drawing_at(f), d(1), "d1 aparece em {f}");
        }
        assert_eq!(l.drawing_at(10), None, "sentinela: nada");
        assert_eq!(l.drawing_at(11), None);
        assert_eq!(l.drawing_at(12), d(2));
        assert_eq!(l.drawing_at(999), d(2), "segura indefinidamente");
    }

    /// add_frame implícito: insere; chave real presente = falha.
    #[test]
    fn add_frame_implicit_and_collision() {
        let mut l = layer();
        assert!(l.add_frame(0, d(0), KeyKind::Keyframe, Hold::Implicit));
        assert!(l.frames[&0].implicit_hold);
        assert_eq!(l.frames.len(), 1);
        // Colisão com chave real → falha, nada muda.
        assert!(!l.add_frame(0, d(9), KeyKind::Keyframe, Hold::Implicit));
        assert_eq!(l.drawing_at(0), d(0));
    }

    /// add_frame fixo cria uma sentinela em key+dur.
    #[test]
    fn add_frame_fixed_creates_end_sentinel() {
        let mut l = layer();
        assert!(l.add_frame(0, d(0), KeyKind::Keyframe, Hold::Fixed(5)));
        // Chave real em 0 + sentinela em 5.
        assert_eq!(l.frames.len(), 2);
        assert!(!l.frames[&0].implicit_hold);
        assert!(l.frames[&5].is_end());
        assert_eq!(l.drawing_at(4), d(0));
        assert_eq!(l.drawing_at(5), None, "sentinela em 5");
    }

    /// Sobrescrever uma sentinela com uma chave real.
    #[test]
    fn add_frame_overwrites_end_sentinel() {
        let mut l = layer();
        l.add_frame(0, d(0), KeyKind::Keyframe, Hold::Fixed(5)); // sentinela em 5
        assert!(l.frames[&5].is_end());
        assert!(l.add_frame(5, d(1), KeyKind::Keyframe, Hold::Implicit));
        assert_eq!(l.drawing_at(5), d(1), "a sentinela virou chave real");
    }

    /// add_frame fixo cujo fim cai numa chave já existente → sem sentinela nova.
    #[test]
    fn add_frame_fixed_ending_on_existing_key_adds_no_sentinel() {
        let mut l = layer();
        l.add_frame(0, d(0), KeyKind::Keyframe, Hold::Implicit);
        l.add_frame(5, d(1), KeyKind::Keyframe, Hold::Implicit);
        let before = l.frames.len();
        // Insere em 2 com duração 3 → fim em 5, que já é chave → nada novo.
        assert!(l.add_frame(2, d(2), KeyKind::Keyframe, Hold::Fixed(3)));
        assert_eq!(l.frames.len(), before + 1, "só a chave em 2, sem sentinela");
        assert!(!l.frames.contains_key(&6));
    }

    /// remove_frame com anterior implicit → apaga de fato (o anterior estende).
    #[test]
    fn remove_frame_with_implicit_prev_deletes() {
        let mut l = layer();
        l.add_frame(0, d(0), KeyKind::Keyframe, Hold::Implicit);
        l.add_frame(5, d(1), KeyKind::Keyframe, Hold::Implicit);
        let unref = l.remove_frame(5);
        assert_eq!(unref, Some(d(1)), "d1 perdeu a referência");
        assert!(!l.frames.contains_key(&5));
        assert_eq!(l.drawing_at(6), d(0), "o anterior estende");
    }

    /// remove_frame com anterior FIXO → converte o slot em sentinela de fim.
    #[test]
    fn remove_frame_with_fixed_prev_becomes_end() {
        let mut l = layer();
        // 0 fixo dur 5 (sentinela em 5). Insere real em 5.
        l.add_frame(0, d(0), KeyKind::Keyframe, Hold::Fixed(5));
        l.add_frame(5, d(1), KeyKind::Keyframe, Hold::Implicit);
        // Remover a chave 5: o anterior (0) é fixo → 5 vira sentinela.
        let unref = l.remove_frame(5);
        assert_eq!(unref, Some(d(1)));
        assert!(
            l.frames.contains_key(&5),
            "o slot continua (como sentinela)"
        );
        assert!(l.frames[&5].is_end());
        assert_eq!(l.drawing_at(5), None, "o fixo anterior não vaza");
    }

    /// remove_frame da última chave apaga direto; end-frame devolve unref None.
    #[test]
    fn remove_last_frame_and_end_frame() {
        let mut l = layer();
        l.add_frame(0, d(0), KeyKind::Keyframe, Hold::Implicit);
        assert_eq!(l.remove_frame(0), Some(d(0)));
        assert!(l.frames.is_empty());
        // Chave ausente = no-op.
        assert_eq!(l.remove_frame(42), None);
    }

    #[test]
    fn duration_at_measures_to_next_key() {
        let mut l = layer();
        l.add_frame(0, d(0), KeyKind::Keyframe, Hold::Implicit);
        l.add_frame(5, d(1), KeyKind::Keyframe, Hold::Implicit);
        assert_eq!(l.duration_at(0), 5);
        assert_eq!(l.duration_at(5), 0, "última segura indefinidamente");
        assert_eq!(l.duration_at(99), 0, "chave ausente");
    }
}
