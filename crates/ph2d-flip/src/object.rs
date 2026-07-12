//! [`FlipObject`] — um objeto Flip: uma pilha de camadas + o array de desenhos +
//! Ghost Frames + FPS. É a unidade que vira uma **entidade ECS** na Hierarquia
//! (via `ph2d_ecs::FlipObjectRef`), como um sprite ou um path vetorial.
//!
//! O objeto é o dono tanto das camadas (que têm os frames) quanto dos desenhos
//! (o array refcontado). Por isso as ops que criam/removem frames referenciando
//! desenhos vivem AQUI — é onde o refcount é mantido consistente. A mecânica
//! pura do mapa de frames fica na [`crate::FlipLayer`]; o objeto só coordena.
//!
//! Ops portadas do `GreasePencil::{insert_frame, insert_duplicate_frame,
//! remove_frames, remove_drawings_with_no_users}` (`02_referencia §1`),
//! clean-room.

use crate::drawing::FlipDrawing;
use crate::frame::{Hold, KeyKind};
use crate::ids::{DrawingId, FlipObjectId, Frame, LayerId};
use crate::layer::FlipLayer;
use crate::onion::OnionSettings;
use ph2d_core::Playhead;
use serde::{Deserialize, Serialize};

/// FPS default de um objeto Flip — 24 (padrão de cinema; a espessura de hold é
/// função dele: o quadro N aparece em `N/fps` segundos).
pub const DEFAULT_FPS: f32 = 24.0;

/// Como duplicar um frame ([`FlipObject::duplicate_frame`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DupMode {
    /// Compartilha o MESMO desenho (`+1 user`) — ciclos, economia de memória, e
    /// a edição propaga para todos os frames que o instanciam.
    Instance,
    /// Copia o desenho (novo, `users = 1`). É o que o operador "duplicate" do
    /// editor do GP faz por padrão.
    Deep,
}

/// Um objeto Flip.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FlipObject {
    /// Id estável na cena; o que a entidade ECS carrega.
    pub id: FlipObjectId,
    /// Nome (Hierarquia / tira).
    pub name: String,
    /// Camadas, em ordem de z (índice 0 = fundo). O reorder é do painel (W2).
    layers: Vec<FlipLayer>,
    /// O array de desenhos. `DrawingId` é o índice AQUI (posicional).
    drawings: Vec<FlipDrawing>,
    /// Ghost Frames.
    pub onion: OnionSettings,
    /// Quadros por segundo (mapeia número de quadro ↔ tempo).
    pub fps: f32,
    /// Próximo `LayerId` livre (ids de camada são estáveis, não posicionais).
    next_layer_id: u32,
}

impl FlipObject {
    /// Objeto vazio (sem camadas). O caller adiciona camadas com [`Self::add_layer`].
    #[must_use]
    pub fn new(id: FlipObjectId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            layers: Vec::new(),
            drawings: Vec::new(),
            onion: OnionSettings::default(),
            fps: DEFAULT_FPS,
            next_layer_id: 0,
        }
    }

    // ── camadas ──────────────────────────────────────────────────────────────

    /// Todas as camadas (ordem de z: 0 = fundo).
    #[must_use]
    pub fn layers(&self) -> &[FlipLayer] {
        &self.layers
    }

    /// Adiciona uma camada nova no topo; devolve seu id estável.
    pub fn add_layer(&mut self, name: impl Into<String>) -> LayerId {
        let id = LayerId(self.next_layer_id);
        self.next_layer_id += 1;
        self.layers.push(FlipLayer::new(id, name));
        id
    }

    /// Remove a camada `id` (e seus frames). Os desenhos que só ela referenciava
    /// caem a `users = 0` (reclamáveis por [`Self::remove_unused_drawings`]).
    /// Devolve `true` se existia.
    pub fn remove_layer(&mut self, id: LayerId) -> bool {
        let Some(i) = self.layers.iter().position(|l| l.id == id) else {
            return false;
        };
        self.layers[i].clear_frames();
        self.layers.remove(i);
        self.recompute_users();
        true
    }

    /// Sobe a camada `id` um passo na ordem de z (rumo ao topo). Troca com a
    /// vizinha de cima (índice maior); os desenhos/refcount não mudam (só a
    /// ordem do slice). Devolve `false` se `id` não existe ou já é o topo.
    pub fn raise_layer(&mut self, id: LayerId) -> bool {
        let Some(i) = self.layer_index(id) else {
            return false;
        };
        if i + 1 >= self.layers.len() {
            return false;
        }
        self.layers.swap(i, i + 1);
        true
    }

    /// Desce a camada `id` um passo na ordem de z (rumo ao fundo). Troca com a
    /// vizinha de baixo (índice menor). Devolve `false` se `id` não existe ou já
    /// é o fundo.
    pub fn lower_layer(&mut self, id: LayerId) -> bool {
        let Some(i) = self.layer_index(id) else {
            return false;
        };
        if i == 0 {
            return false;
        }
        self.layers.swap(i, i - 1);
        true
    }

    #[must_use]
    pub fn layer(&self, id: LayerId) -> Option<&FlipLayer> {
        self.layers.iter().find(|l| l.id == id)
    }

    pub fn layer_mut(&mut self, id: LayerId) -> Option<&mut FlipLayer> {
        self.layers.iter_mut().find(|l| l.id == id)
    }

    fn layer_index(&self, id: LayerId) -> Option<usize> {
        self.layers.iter().position(|l| l.id == id)
    }

    // ── desenhos ─────────────────────────────────────────────────────────────

    /// Todos os desenhos (indexados por `DrawingId`).
    #[must_use]
    pub fn drawings(&self) -> &[FlipDrawing] {
        &self.drawings
    }

    #[must_use]
    pub fn drawing(&self, id: DrawingId) -> Option<&FlipDrawing> {
        self.drawings.get(id.0 as usize)
    }

    pub fn drawing_mut(&mut self, id: DrawingId) -> Option<&mut FlipDrawing> {
        self.drawings.get_mut(id.0 as usize)
    }

    // ── pose / bounds (ADR-0111 parity: o objeto tem Transform + geometria LOCAL) ─

    /// A caixa envolvente de TODOS os pontos de TODOS os desenhos do objeto, no
    /// espaço em que a geometria está guardada (`(min, max)`), ou `None` se o objeto
    /// não tem nenhum ponto. É a bbox que o gizmo lê (via `flip_gizmo_view`) e o
    /// centro que o `settle` põe como pivô.
    #[must_use]
    pub fn geometry_bbox(&self) -> Option<([f32; 2], [f32; 2])> {
        let mut it = self
            .drawings
            .iter()
            .flat_map(|d| d.strokes.iter())
            .flat_map(|s| s.positions().iter());
        let first = it.next()?;
        let (mut lo, mut hi) = ([first.x, first.y], [first.x, first.y]);
        for p in it {
            lo[0] = lo[0].min(p.x);
            lo[1] = lo[1].min(p.y);
            hi[0] = hi[0].max(p.x);
            hi[1] = hi[1].max(p.y);
        }
        Some((lo, hi))
    }

    /// Aplica o afim 2D `m` (`[a, b, c, d, e, f]`, col-major: `x' = a·x + c·y + e`,
    /// `y' = b·x + d·y + f`) às POSIÇÕES de todos os pontos de todos os desenhos —
    /// deslocando a geometria inteira do objeto de uma vez. Usado pelo `settle`/
    /// "Set Center" (sempre translação pura → largura/opacidade/cor intactas; a
    /// escala do gizmo vive no `Transform` e entra no render, não aqui).
    pub fn bake_affine(&mut self, m: [f64; 6]) {
        let apply = |p: &mut ph2d_core::Vec2| {
            let (x, y) = (f64::from(p.x), f64::from(p.y));
            p.x = (m[0] * x + m[2] * y + m[4]) as f32;
            p.y = (m[1] * x + m[3] * y + m[5]) as f32;
        };
        for d in &mut self.drawings {
            for s in &mut d.strokes {
                for p in s.positions_mut() {
                    apply(p);
                }
                // Os buracos do fill (W4) andam junto — senão um "Set Center" deslocaria
                // o contorno e deixaria os furos para trás.
                for ring in &mut s.holes {
                    for p in ring {
                        apply(p);
                    }
                }
            }
        }
    }

    // ── ops de frame (mantêm o refcount) ─────────────────────────────────────

    /// Insere uma chave em `key` na camada, criando um **desenho novo vazio** que
    /// ela referencia. Devolve o id do desenho (ou `None` se a camada não existe
    /// ou já há uma chave real em `key`). Espelha `GreasePencil::insert_frame`.
    pub fn insert_frame(
        &mut self,
        layer_id: LayerId,
        key: Frame,
        hold: Hold,
        kind: KeyKind,
    ) -> Option<DrawingId> {
        let new_id = DrawingId(self.drawings.len() as u32);
        let li = self.layer_index(layer_id)?;
        if !self.layers[li].add_frame(key, Some(new_id), kind, hold) {
            return None; // colisão — nenhum desenho foi alocado
        }
        self.drawings.push(FlipDrawing::new());
        self.drawings[new_id.0 as usize].add_user();
        Some(new_id)
    }

    /// Duplica a chave em `src` para `dst` (mesma duração). `Instance` compartilha
    /// o desenho (`+1 user`); `Deep` o copia (`users = 1`). Espelha
    /// `GreasePencil::insert_duplicate_frame`. `false` se a camada/`src` não
    /// existe, `src` é sentinela, ou `dst` colide com uma chave real.
    pub fn duplicate_frame(
        &mut self,
        layer_id: LayerId,
        src: Frame,
        dst: Frame,
        mode: DupMode,
    ) -> bool {
        let Some(li) = self.layer_index(layer_id) else {
            return false;
        };
        let (src_id, src_kind, duration) = {
            let layer = &self.layers[li];
            let Some(f) = layer.frames().get(&src) else {
                return false;
            };
            let Some(id) = f.drawing else {
                return false; // sentinela não se duplica
            };
            (id, f.kind, layer.duration_at(src))
        };
        let hold = if duration == 0 {
            Hold::Implicit
        } else {
            Hold::Fixed(duration)
        };
        match mode {
            DupMode::Instance => {
                if !self.layers[li].add_frame(dst, Some(src_id), src_kind, hold) {
                    return false;
                }
                self.drawings[src_id.0 as usize].add_user();
            }
            DupMode::Deep => {
                let new_id = DrawingId(self.drawings.len() as u32);
                if !self.layers[li].add_frame(dst, Some(new_id), src_kind, hold) {
                    return false;
                }
                let mut clone = self.drawings[src_id.0 as usize].clone();
                clone.set_users(1);
                self.drawings.push(clone);
            }
        }
        true
    }

    /// Remove a chave em `key` da camada; decrementa o refcount do desenho que
    /// ela referenciava. **Não** compacta (chame [`Self::remove_unused_drawings`]
    /// para reclamar). Espelha `GreasePencil::remove_frames` sem o compactar
    /// automático. `false` se a camada/chave não existe.
    pub fn remove_frame(&mut self, layer_id: LayerId, key: Frame) -> bool {
        let Some(li) = self.layer_index(layer_id) else {
            return false;
        };
        match self.layers[li].remove_frame(key) {
            None => false,
            Some(unref) => {
                if let Some(DrawingId(i)) = unref {
                    self.drawings[i as usize].remove_user();
                }
                true
            }
        }
    }

    /// Move a chave em `from` para `to` na mesma camada (relocação simples,
    /// preservando desenho/hold/tipo; sem recomputar duração). `false` se `from`
    /// não existe ou `to` já tem uma chave real. Não mexe no refcount (o desenho
    /// continua referenciado, só em outra chave).
    pub fn move_frame(&mut self, layer_id: LayerId, from: Frame, to: Frame) -> bool {
        let Some(layer) = self.layer_mut(layer_id) else {
            return false;
        };
        layer.relocate_frame(from, to)
    }

    // ── refcount / compactação ───────────────────────────────────────────────

    /// Reconstrói `users` de cada desenho a partir dos frames (fonte de verdade).
    /// O(frames). Espelha `update_drawing_users_for_layer`.
    pub fn recompute_users(&mut self) {
        let mut counts = vec![0u32; self.drawings.len()];
        for layer in &self.layers {
            for f in layer.frames().values() {
                if let Some(DrawingId(i)) = f.drawing
                    && let Some(c) = counts.get_mut(i as usize)
                {
                    *c += 1;
                }
            }
        }
        for (d, c) in self.drawings.iter_mut().zip(counts) {
            d.set_users(c);
        }
    }

    /// Reclama desenhos sem usuários e **remapeia** todos os `DrawingId` dos
    /// frames. Recompute `users` primeiro (fonte de verdade), depois compacta
    /// **de forma estável** (preserva a ordem relativa dos mantidos — mais
    /// determinístico que o swap do GP, mesmo contrato observável). Espelha
    /// `remove_drawings_with_no_users`.
    pub fn remove_unused_drawings(&mut self) {
        self.recompute_users();
        // new_index[old] = Some(novo) para os mantidos; None para os reclamados.
        let mut new_index: Vec<Option<u32>> = vec![None; self.drawings.len()];
        let mut kept: Vec<FlipDrawing> = Vec::new();
        for (old, d) in std::mem::take(&mut self.drawings).into_iter().enumerate() {
            if d.has_users() {
                new_index[old] = Some(kept.len() as u32);
                kept.push(d);
            }
        }
        self.drawings = kept;
        // Remapeia os frames. Um desenho referenciado tem users>0 → foi mantido →
        // new_index é Some; o `debug_assert` guarda contra inconsistência.
        for layer in &mut self.layers {
            for f in layer.frames_mut().values_mut() {
                if let Some(DrawingId(old)) = f.drawing {
                    let mapped = new_index[old as usize];
                    debug_assert!(mapped.is_some(), "frame referencia desenho reclamado");
                    f.drawing = mapped.map(DrawingId);
                }
            }
        }
    }

    // ── amostragem por playhead ──────────────────────────────────────────────

    /// O desenho ativo da camada `layer_id` no quadro `frame` — semântica de hold
    /// **e do ciclo da camada** (é o que o render amostra; a autoria usa o caminho
    /// cru `FlipLayer::drawing_at`, W3.T3.2).
    #[must_use]
    pub fn drawing_at(&self, layer_id: LayerId, frame: Frame) -> Option<DrawingId> {
        self.layer(layer_id)?.drawing_at_cycled(frame)
    }

    /// O quadro (número inteiro) em que o playhead está, dado o FPS deste objeto.
    #[must_use]
    pub fn frame_at(&self, playhead: &Playhead) -> Frame {
        let f = playhead.frame(self.fps as f64);
        f.clamp(i32::MIN as i64, i32::MAX as i64) as i32
    }

    /// Amostra todas as camadas no quadro `frame`: `(camada, desenho ativo)` —
    /// honrando o ciclo de cada camada. A visibilidade/lock é decisão de render/
    /// edição, não desta amostragem.
    #[must_use]
    pub fn sample(&self, frame: Frame) -> Vec<(LayerId, Option<DrawingId>)> {
        self.layers
            .iter()
            .map(|l| (l.id, l.drawing_at_cycled(frame)))
            .collect()
    }

    /// Amostra no tempo do playhead (converte via FPS).
    #[must_use]
    pub fn sample_at(&self, playhead: &Playhead) -> Vec<(LayerId, Option<DrawingId>)> {
        self.sample(self.frame_at(playhead))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Um objeto com uma camada e um desenho não-vazio no quadro 0.
    fn object_with_one_frame() -> (FlipObject, LayerId, DrawingId) {
        let mut o = FlipObject::new(FlipObjectId(1), "Obj");
        let l = o.add_layer("Layer 1");
        let d = o
            .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
            .unwrap();
        (o, l, d)
    }

    #[test]
    fn insert_frame_allocates_a_drawing_with_one_user() {
        let (o, l, d) = object_with_one_frame();
        assert_eq!(d, DrawingId(0));
        assert_eq!(o.drawings().len(), 1);
        assert_eq!(o.drawing(d).unwrap().users(), 1);
        assert_eq!(o.drawing_at(l, 0), Some(d));
        assert_eq!(o.drawing_at(l, 5), Some(d), "segura");
    }

    #[test]
    fn raise_and_lower_reorder_layers_without_touching_drawings() {
        let mut o = FlipObject::new(FlipObjectId(1), "Obj");
        let a = o.add_layer("A"); // índice 0 (fundo)
        let b = o.add_layer("B"); // índice 1
        let c = o.add_layer("C"); // índice 2 (topo)
        let ids = |o: &FlipObject| o.layers().iter().map(|l| l.id).collect::<Vec<_>>();
        assert_eq!(ids(&o), vec![a, b, c]);

        assert!(o.raise_layer(a)); // A sobe: [B, A, C]
        assert_eq!(ids(&o), vec![b, a, c]);
        assert!(!o.raise_layer(c), "C já é o topo");
        assert!(o.lower_layer(c)); // C desce: [B, C, A]
        assert_eq!(ids(&o), vec![b, c, a]);
        assert!(!o.lower_layer(b), "B já é o fundo");
        assert!(!o.raise_layer(LayerId(999)), "id inexistente");
    }

    /// Empurra um traço de 2 pontos `a`→`b` no desenho `d`.
    fn push_seg(o: &mut FlipObject, d: DrawingId, a: [f32; 2], b: [f32; 2]) {
        let mut s = crate::stroke::FlipStroke::new();
        s.push_default(ph2d_core::Vec2::new(a[0], a[1]));
        s.push_default(ph2d_core::Vec2::new(b[0], b[1]));
        o.drawing_mut(d).unwrap().strokes.push(s);
    }

    #[test]
    fn geometry_bbox_unions_all_strokes_and_drawings() {
        let (mut o, l, d0) = object_with_one_frame();
        assert_eq!(o.geometry_bbox(), None, "objeto sem pontos: sem bbox");
        push_seg(&mut o, d0, [10.0, 20.0], [30.0, 15.0]);
        // 2º desenho noutro quadro estende a bbox.
        let d1 = o
            .insert_frame(l, 5, Hold::Implicit, KeyKind::Keyframe)
            .unwrap();
        push_seg(&mut o, d1, [-5.0, 40.0], [12.0, 41.0]);
        assert_eq!(
            o.geometry_bbox(),
            Some(([-5.0, 15.0], [30.0, 41.0])),
            "união de todos os pontos de todos os desenhos"
        );
    }

    #[test]
    fn bake_affine_translation_shifts_every_point() {
        let (mut o, _l, d0) = object_with_one_frame();
        push_seg(&mut o, d0, [10.0, 20.0], [30.0, 40.0]);
        // Translação pura (-10, -20): a geometria recua, a bbox acompanha.
        o.bake_affine([1.0, 0.0, 0.0, 1.0, -10.0, -20.0]);
        assert_eq!(o.geometry_bbox(), Some(([0.0, 0.0], [20.0, 20.0])));
        // Largura/opacidade não são tocadas por um bake de translação.
        let s = &o.drawing(d0).unwrap().strokes[0];
        assert_eq!(s.positions()[0], ph2d_core::Vec2::new(0.0, 0.0));
    }

    #[test]
    fn insert_frame_collision_allocates_no_drawing() {
        let (mut o, l, _d) = object_with_one_frame();
        // Segunda chave real em 0 → falha, sem desenho novo.
        assert_eq!(
            o.insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe),
            None
        );
        assert_eq!(o.drawings().len(), 1, "não vazou um desenho órfão");
    }

    /// T0.4 espelhado a nível de objeto: duplicar-como-instância compartilha o
    /// desenho; remover um frame decrementa; compactar remapeia.
    #[test]
    fn instance_shares_drawing_and_removal_decrements() {
        let (mut o, l, d0) = object_with_one_frame();
        assert!(o.duplicate_frame(l, 0, 10, DupMode::Instance));
        assert_eq!(o.drawing(d0).unwrap().users(), 2, "instanciado");
        assert!(o.drawing(d0).unwrap().is_instanced());
        assert_eq!(o.drawing_at(l, 10), Some(d0), "o clone-instância aponta d0");

        // Remove a chave 10 → users volta a 1.
        assert!(o.remove_frame(l, 10));
        assert_eq!(o.drawing(d0).unwrap().users(), 1);
    }

    #[test]
    fn deep_duplicate_copies_the_drawing() {
        let (mut o, l, d0) = object_with_one_frame();
        // Põe conteúdo no d0 para provar a cópia profunda.
        o.drawing_mut(d0).unwrap().strokes.push(Default::default());
        assert!(o.duplicate_frame(l, 0, 10, DupMode::Deep));
        assert_eq!(o.drawings().len(), 2);
        let d1 = o.drawing_at(l, 10).unwrap();
        assert_ne!(d1, d0, "desenho NOVO");
        assert_eq!(o.drawing(d1).unwrap().users(), 1);
        assert_eq!(o.drawing(d0).unwrap().users(), 1, "d0 não ganhou user");
        assert_eq!(o.drawing(d1).unwrap().strokes.len(), 1, "conteúdo copiado");
    }

    /// T0.4: compactação com remap. Cria 3 desenhos, remove o do MEIO, compacta;
    /// os `DrawingId` dos frames sobreviventes são remapeados.
    #[test]
    fn remove_unused_drawings_compacts_and_remaps() {
        let mut o = FlipObject::new(FlipObjectId(1), "Obj");
        let l = o.add_layer("L");
        let d0 = o
            .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
            .unwrap();
        let d1 = o
            .insert_frame(l, 5, Hold::Implicit, KeyKind::Keyframe)
            .unwrap();
        let d2 = o
            .insert_frame(l, 10, Hold::Implicit, KeyKind::Keyframe)
            .unwrap();
        assert_eq!((d0, d1, d2), (DrawingId(0), DrawingId(1), DrawingId(2)));

        // Remove o frame do meio (chave 5, desenho d1). users(d1) → 0.
        assert!(o.remove_frame(l, 5));
        assert_eq!(o.drawing(d1).unwrap().users(), 0);

        o.remove_unused_drawings();
        // d1 sumiu; d2 foi remapeado de 2 → 1.
        assert_eq!(o.drawings().len(), 2);
        // Os frames restantes ainda resolvem para desenhos válidos e distintos.
        let at0 = o.drawing_at(l, 0).unwrap();
        let at10 = o.drawing_at(l, 10).unwrap();
        assert_eq!(at0, DrawingId(0), "d0 ficou no lugar");
        assert_eq!(at10, DrawingId(1), "d2 remapeado para 1");
        assert_ne!(at0, at10);
        // Nada aparece em 5..9 (o frame foi removido; d0 é implicit → estende? NÃO:
        // remove_frame com anterior implicit apaga a chave, então d0 estende).
        assert_eq!(o.drawing_at(l, 7), Some(at0), "d0 estende sobre o buraco");
    }

    #[test]
    fn move_frame_relocates_key() {
        let (mut o, l, d0) = object_with_one_frame();
        assert!(o.move_frame(l, 0, 20));
        assert_eq!(o.drawing_at(l, 0), None, "não há mais nada antes de 20");
        assert_eq!(o.drawing_at(l, 20), Some(d0));
        // Mover para cima de uma chave real falha.
        o.insert_frame(l, 25, Hold::Implicit, KeyKind::Keyframe);
        assert!(!o.move_frame(l, 20, 25));
    }

    #[test]
    fn remove_layer_drops_its_drawings_users() {
        let (mut o, l, d0) = object_with_one_frame();
        assert!(o.remove_layer(l));
        assert_eq!(o.layers().len(), 0);
        assert_eq!(o.drawing(d0).unwrap().users(), 0, "sem camada, sem user");
        o.remove_unused_drawings();
        assert_eq!(o.drawings().len(), 0);
    }

    /// T0.12: amostragem por playhead. FPS 24: t=0.25s → quadro 6.
    #[test]
    fn sample_by_playhead_resolves_active_drawing_per_layer() {
        let mut o = FlipObject::new(FlipObjectId(1), "Obj");
        o.fps = 24.0;
        let l = o.add_layer("L");
        let d0 = o
            .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
            .unwrap();
        let d6 = o
            .insert_frame(l, 6, Hold::Implicit, KeyKind::Keyframe)
            .unwrap();

        let mut ph = Playhead::new(1.0 / 24.0);
        ph.seek(0.0);
        assert_eq!(o.frame_at(&ph), 0);
        assert_eq!(o.sample_at(&ph), vec![(l, Some(d0))]);

        ph.seek(0.25); // 0.25 * 24 = quadro 6
        assert_eq!(o.frame_at(&ph), 6);
        assert_eq!(o.sample_at(&ph), vec![(l, Some(d6))]);
    }
}
