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
use ph2d_core::{Playhead, Vec2};
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

    /// **A arte como ela APARECE**: cada chave de cada camada, com a POSE dela — o par
    /// `(offset, desenho)`. Um desenho instanciado por duas chaves em poses diferentes
    /// sai **duas vezes**, em lugares diferentes, que é exatamente o que ele é na tela.
    ///
    /// [`Self::geometry_bbox`] responde *"onde estão os pontos"* (é geometria, e é o que
    /// o pivô/settle usa); isto responde *"onde está a arte"* (é aparência, e é o que o
    /// gizmo e o marquee precisam). Confundir as duas põe a caixa do gizmo longe do
    /// desenho no instante em que alguém move uma instância.
    pub fn posed_drawings(&self) -> impl Iterator<Item = (Vec2, &FlipDrawing)> {
        self.layers.iter().flat_map(move |l| {
            l.frames().values().filter_map(move |f| {
                let d = self.drawing(f.drawing?)?;
                Some((f.offset, d))
            })
        })
    }

    /// A caixa envolvente da arte **como ela aparece** (com as poses das chaves) — a que
    /// o gizmo desenha e o marquee testa. `None` se o objeto não tem ponto nenhum.
    #[must_use]
    pub fn posed_bbox(&self) -> Option<([f32; 2], [f32; 2])> {
        let mut acc: Option<([f32; 2], [f32; 2])> = None;
        for (off, d) in self.posed_drawings() {
            for s in &d.strokes {
                for p in s.positions() {
                    let (x, y) = (p.x + off.x, p.y + off.y);
                    acc = Some(match acc {
                        None => ([x, y], [x, y]),
                        Some((lo, hi)) => {
                            ([lo[0].min(x), lo[1].min(y)], [hi[0].max(x), hi[1].max(y)])
                        }
                    });
                }
            }
        }
        acc
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
        // A POSE da origem viaja junto: a chave nova nasce ONDE a arte está agora. Sem
        // isto, duplicar uma chave deslocada faria a cópia saltar para a origem do
        // objeto — o desenho pularia no quadro seguinte.
        let src_offset = self.layers[li].frame_offset(src);
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
        self.layers[li].set_frame_offset(dst, src_offset);
        true
    }

    /// **A pose da chave `key`** da camada — o deslocamento da arte naquele quadro.
    #[must_use]
    pub fn frame_offset(&self, layer_id: LayerId, key: Frame) -> Vec2 {
        self.layer(layer_id)
            .map_or(Vec2::ZERO, |l| l.frame_offset(key))
    }

    /// **Abre espaço em `at`** para uma chave nova, empurrando à frente **só o bloco
    /// contíguo** de quadros que começa em `at` (até o primeiro buraco). Do fim para o
    /// começo, senão a 1ª mudança cairia sobre a vizinha ainda parada — a mesma disciplina
    /// do ripple de [`Self::set_exposure`]. A POSE de cada chave viaja junto
    /// (`relocate_frame` move o `FlipFrame` inteiro).
    ///
    /// É o que deixa CRIAR uma chave no meio da tira, e não só depois da última: sem isto
    /// o alvo `chave + duração` caía sobre a próxima chave real e o insert falhava em
    /// silêncio (smoke do Enio, 2026-07-14). Empurra o bloco contíguo, não tudo à frente —
    /// quadros separados por um buraco não têm o ritmo mexido à toa.
    ///
    /// Devolve `true` se moveu algo. No-op (`false`) se `at` já está livre.
    pub fn open_gap_at(&mut self, layer_id: LayerId, at: Frame) -> bool {
        let Some(li) = self.layer_index(layer_id) else {
            return false;
        };
        // O bloco contíguo a partir de `at`, até o primeiro quadro livre.
        let mut run: Vec<Frame> = Vec::new();
        let mut f = at;
        while self.layers[li].frames().contains_key(&f) {
            run.push(f);
            f += 1;
        }
        let mut moved = false;
        for &f in run.iter().rev() {
            moved |= self.layers[li].relocate_frame(f, f + 1);
        }
        moved
    }

    /// **Desloca a chave `key`** (a POSE, não a arte): só este quadro se move, mesmo que
    /// o desenho seja compartilhado por outras chaves. É a outra metade da instância —
    /// a arte é uma só, o lugar é de cada quadro. `false` se a camada/chave não existe.
    pub fn translate_frame(&mut self, layer_id: LayerId, key: Frame, delta: Vec2) -> bool {
        let Some(li) = self.layer_index(layer_id) else {
            return false;
        };
        let now = self.layers[li].frame_offset(key);
        self.layers[li].set_frame_offset(key, now + delta)
    }

    /// **Quebra o vínculo** da chave `key` com a arte compartilhada (o *make single
    /// user* do Blender): a chave passa a ter um desenho SÓ dela, cópia do que
    /// compartilhava, e os demais quadros seguem com o original intacto.
    ///
    /// É a saída de emergência da instância. Sem ela, instanciar seria irreversível: a
    /// única forma de divergir a arte de um quadro seria apagar a chave e redesenhar.
    ///
    /// `false` se a camada/chave não existe, se a chave é sentinela, ou se o desenho
    /// **já é exclusivo** dela (não há vínculo a quebrar — no-op honesto).
    pub fn make_single_user(&mut self, layer_id: LayerId, key: Frame) -> bool {
        let Some(li) = self.layer_index(layer_id) else {
            return false;
        };
        let Some(src_id) = self.layers[li].frames().get(&key).and_then(|f| f.drawing) else {
            return false; // sentinela ou chave inexistente
        };
        if !self.drawings[src_id.0 as usize].is_instanced() {
            return false; // já é só dela
        }
        let new_id = DrawingId(self.drawings.len() as u32);
        let mut clone = self.drawings[src_id.0 as usize].clone();
        clone.set_users(1);
        self.drawings.push(clone);
        self.drawings[src_id.0 as usize].remove_user();
        self.layers[li].set_frame_drawing(key, new_id)
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
#[path = "object_tests.rs"]
mod tests;
