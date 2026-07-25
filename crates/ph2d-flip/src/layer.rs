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

use crate::cycle::LayerCycle;
use crate::frame::{FlipFrame, Hold, KeyKind};
use crate::ids::{DrawingId, Frame, LayerId};
use crate::pose::Pose;
use ph2d_painter_effects::BlendMode;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::Bound;

/// Profundidade multiplano default: `1.0` = no plano da câmera (segue 100%, o
/// comportamento flat de sempre). Ver [`FlipLayer::depth`].
pub const DEFAULT_LAYER_DEPTH: f32 = 1.0;

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
    /// Ciclo (pre/post behavior) — o wrap-mode do amostrador FORA do vão da
    /// camada. O default reproduz o comportamento pré-W3 (nada antes, segura
    /// depois): ver [`crate::CycleMode`].
    pub cycle: LayerCycle,
    /// Esta camada participa dos Ghost Frames (W3.T3.3). Ligada por padrão — o
    /// artista desliga nas camadas de fundo/referência, que só poluiriam.
    pub use_onion: bool,
    /// **Profundidade multiplano** (2.5D, ADR-0114 §Decisão 3) — a FRAÇÃO com que
    /// esta camada acompanha a câmera. `1.0` (default) = no plano da câmera, segue
    /// 100% (o comportamento flat de sempre); `0.0` = infinitamente longe, travada
    /// na cena (estática enquanto a câmera panha). O deslocamento de tela por
    /// unidade de pan é exatamente `depth`, então camadas de FUNDO recebem valores
    /// baixos e rastejam, dando a paralaxe da câmera multiplano da Disney.
    ///
    /// Ancorada na ORIGEM do objeto Flip: enquadrado de frente, todos os planos
    /// coincidem; o render lerpa o centro da câmera da fatia entre a origem do
    /// objeto e o centro real por `depth` (ver o `parallax_camera` do shell).
    pub depth: f32,
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
            cycle: LayerCycle::default(),
            use_onion: true,
            depth: DEFAULT_LAYER_DEPTH,
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

    /// **A pose da chave `key`** — o afim da arte daquele quadro ([`FlipFrame::pose`]).
    /// Identidade se a chave não existe.
    #[must_use]
    pub fn frame_pose(&self, key: Frame) -> Pose {
        self.frames.get(&key).map_or(Pose::IDENTITY, |f| f.pose)
    }

    /// Aponta a chave `key` para outro desenho (o refcount é do
    /// [`crate::FlipObject`] — quem chama acerta os `users`). `false` se a chave não
    /// existe ou é sentinela.
    pub fn set_frame_drawing(&mut self, key: Frame, drawing: DrawingId) -> bool {
        match self.frames.get_mut(&key) {
            Some(f) if f.drawing.is_some() => {
                f.drawing = Some(drawing);
                true
            }
            _ => false,
        }
    }

    /// Escreve a pose da chave `key`. `false` se a chave não existe.
    pub fn set_frame_pose(&mut self, key: Frame, pose: Pose) -> bool {
        match self.frames.get_mut(&key) {
            Some(f) => {
                f.pose = pose;
                true
            }
            None => false,
        }
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
                // Pose neutra. Quem cria uma chave a partir de OUTRA (duplicar,
                // instanciar, autokey, tween) reescreve o offset depois — senão a arte
                // saltaria de volta para a origem no quadro novo.
                pose: Pose::IDENTITY,
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

    /// Fixa o fim do vão da camada em `end`: move (ou cria) a **sentinela de fim**
    /// que vem depois de `key`, e marca a chave como duração FIXA. É como a última
    /// chave ganha exposição (`crate::FlipObject::set_exposure`).
    ///
    /// `false` se `key` não é uma chave real, se `end <= key`, ou se há uma chave
    /// REAL em `end` (aí o vão já está delimitado por ela).
    pub(crate) fn set_end_sentinel(&mut self, key: Frame, end: Frame) -> bool {
        if end <= key || self.frames.get(&key).is_none_or(|f| f.drawing.is_none()) {
            return false;
        }
        if self.frames.get(&end).is_some_and(|f| !f.is_end()) {
            return false;
        }
        // A sentinela antiga (se houver) some — só uma fecha o vão.
        let old: Vec<Frame> = self
            .frames
            .range((Bound::Excluded(key), Bound::Unbounded))
            .filter(|(_, f)| f.is_end())
            .map(|(&k, _)| k)
            .collect();
        for k in old {
            self.frames.remove(&k);
        }
        if let Some(f) = self.frames.get_mut(&key) {
            f.implicit_hold = false; // duração fixa: a sentinela é quem manda
        }
        self.frames.insert(end, FlipFrame::end());
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
    use crate::cycle::CycleMode;

    fn d(n: u32) -> Option<DrawingId> {
        Some(DrawingId(n))
    }

    fn layer() -> FlipLayer {
        FlipLayer::new(LayerId(0), "L")
    }

    /// Camada com chaves em `keys` (desenhos 0..n), holds implícitos.
    fn keyed(keys: &[Frame]) -> FlipLayer {
        let mut l = layer();
        for (i, &k) in keys.iter().enumerate() {
            l.add_frame(k, d(i as u32), KeyKind::Keyframe, Hold::Implicit);
        }
        l
    }

    /// 🔴 **A profundidade multiplano nasce em `1.0` (flat) e sobrevive ao round-trip**
    /// (2.5D, ADR-0114 §Decisão 3). Default `1.0` = a camada segue a câmera 100% ⇒ arte
    /// antiga byte-idêntica. Uma camada de fundo com `depth` baixo tem de voltar da
    /// serialização com o valor — senão a paralaxe se perde ao salvar/abrir.
    #[test]
    fn depth_defaults_to_flat_and_survives_the_round_trip() {
        assert_eq!(layer().depth, DEFAULT_LAYER_DEPTH);
        assert_eq!(DEFAULT_LAYER_DEPTH, 1.0, "flat = segue a camera 100%");
        let mut l = layer();
        l.depth = 0.2; // camada de fundo: rasteja a 20% do pan
        let bytes = postcard::to_allocvec(&l).unwrap();
        let back: FlipLayer = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(back.depth, 0.2, "o depth sobrevive ao postcard");
        assert_eq!(back, l);
    }

    /// O vão: da 1ª chave real ao fim. Sem sentinela, a última chave expõe UM
    /// quadro (a tira mostra 1 e deixa esticar — o hold implícito é infinito e não
    /// pode definir sozinho o fim do ciclo).
    #[test]
    fn span_ends_at_the_sentinel_or_one_past_the_last_key() {
        assert_eq!(layer().span(), None, "camada vazia não tem vão");
        assert_eq!(keyed(&[0, 4, 8]).span(), Some((0, 9)));
        let mut l = keyed(&[0, 4]);
        l.add_frame(8, None, KeyKind::Keyframe, Hold::Implicit); // sentinela
        assert_eq!(l.span(), Some((0, 8)), "a sentinela FECHA o vão");
    }

    /// O ciclo é o wrap-mode do amostrador — e o DEFAULT não muda nada (nada antes
    /// da 1ª chave, o último desenho segura depois).
    #[test]
    fn cycles_wrap_the_sampler_without_duplicating_frames() {
        let mut l = keyed(&[0, 4]); // vão [0, 5)
        assert_eq!(l.drawing_at_cycled(-1), None, "default: nada antes");
        assert_eq!(l.drawing_at_cycled(99), d(1), "default: segura depois");

        l.cycle = LayerCycle {
            pre: CycleMode::None,
            post: CycleMode::Loop,
        };
        assert_eq!(l.drawing_at_cycled(5), d(0), "5 → 0 (o vão tem 5 quadros)");
        assert_eq!(l.drawing_at_cycled(9), d(1), "9 → 4");
        assert_eq!(l.drawing_at_cycled(-1), None, "o outro lado não ciclou");
    }

    /// **Amostrar e autorar divergem — e a divergência não é onde parece.**
    ///
    /// Sob `Hold` (o default), o quadro depois do vão MOSTRA o último desenho, mas
    /// autorar ali cria uma chave NOVA: o tempo depois do vão é tempo novo, e é assim
    /// que a animação cresce. Sob `Loop`, o mesmo quadro mostra o desenho do vão E
    /// autorar ali edita ESSE desenho: o tempo é o vão de novo, e escrever no quadro
    /// cru quebraria o ciclo.
    #[test]
    fn authoring_follows_the_cycle_only_where_time_repeats() {
        let mut l = keyed(&[0, 4]); // vão [0, 5) — as duas chaves e mais nada

        // Hold (default): o quadro 20 MOSTRA o desenho de 4 …
        assert_eq!(l.source_frame(20), 4);
        assert_eq!(l.drawing_at_cycled(20), d(1));
        // … mas autorar nele é autorar EM 20 (a chave nova nasce ali).
        assert_eq!(l.authoring_frame(20), 20, "Hold: o tempo lá fora é NOVO");

        // Loop: o quadro 20 é o quadro 0 de novo — mostrar E autorar caem no 0.
        l.cycle = LayerCycle {
            pre: CycleMode::Loop,
            post: CycleMode::Loop,
        };
        assert_eq!(l.source_frame(20), 0, "20 % 5 = 0");
        assert_eq!(
            l.authoring_frame(20),
            0,
            "Loop: editar a 2ª volta edita o vão"
        );
        // Dentro do vão é sempre identidade, em qualquer ciclo.
        assert_eq!(l.authoring_frame(3), 3);
        assert_eq!(l.source_frame(3), 3);
    }

    /// Navegação por DESENHO (o flip do animador): pula os holds, e a partir do
    /// meio de um hold "anterior" cai no início da exposição atual.
    #[test]
    fn drawing_navigation_skips_holds() {
        let l = keyed(&[0, 12, 24]);
        assert_eq!(l.next_drawing_key(5), Some(12));
        assert_eq!(
            l.prev_drawing_key(18),
            Some(12),
            "início da exposição atual"
        );
        assert_eq!(l.prev_drawing_key(12), Some(0), "e daí, o desenho anterior");
        assert_eq!(l.next_drawing_key(24), None);
        assert_eq!(l.active_key(18), Some(12));
    }

    /// As células da tira: cada chave real com sua EXPOSIÇÃO. A última mostra 1
    /// (nada a delimita ainda).
    #[test]
    fn cells_carry_the_exposure_of_each_key() {
        let l = keyed(&[0, 4, 6]);
        assert_eq!(
            l.cells(),
            vec![
                (0, DrawingId(0), 4),
                (4, DrawingId(1), 2),
                (6, DrawingId(2), 1),
            ]
        );
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
                pose: Pose::IDENTITY,
            },
        );
        l.frames.insert(
            5,
            FlipFrame {
                drawing: d(1),
                implicit_hold: true,
                kind: KeyKind::Keyframe,
                pose: Pose::IDENTITY,
            },
        );
        l.frames.insert(10, FlipFrame::end());
        l.frames.insert(
            12,
            FlipFrame {
                drawing: d(2),
                implicit_hold: true,
                kind: KeyKind::Keyframe,
                pose: Pose::IDENTITY,
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
