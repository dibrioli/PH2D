//! **O TEMPO da camada** — vão, ciclos, navegação por desenho e as células da tira
//! (W3). Módulo-irmão do [`crate::layer`] (que é dono do MAPA de chaves e da mecânica
//! de inserir/remover): aqui não se muda nada, só se PERGUNTA que quadro é qual.
//!
//! A separação não é só do cap de LOC: são duas responsabilidades que erram de jeitos
//! diferentes. O mapa erra em topologia (uma sentinela a mais, um refcount a menos); o
//! tempo erra em SEMÂNTICA — e as três transforms daqui (`drawing_at` cru,
//! `source_frame`, `authoring_frame`) já custaram um bug cada
//! (`docs/Flip/BUGS_flip.md` #7).

use crate::cycle::{CycleMode, map_frame};
use crate::frame::KeyKind;
use crate::ids::{DrawingId, Frame};
use crate::layer::FlipLayer;
use ph2d_core::Vec2;
use std::ops::Bound;

impl FlipLayer {
    // ── vão + ciclos (W3.T3.2) ───────────────────────────────────────────────

    /// O **vão** da camada: `[primeira chave, fim)`, ou `None` se ela não tem
    /// nenhuma chave REAL (só sentinelas / vazia). `fim` = a sentinela de fim, se
    /// a última chave for uma; senão `última chave + 1` (a última chave real expõe
    /// um quadro — esticar esse hold na tira cria a sentinela e fixa a duração).
    ///
    /// É o intervalo que os ciclos repetem/espelham (`crate::cycle::map_frame`).
    #[must_use]
    pub fn span(&self) -> Option<(Frame, Frame)> {
        let (&first, _) = self.frames().iter().find(|(_, f)| !f.is_end())?;
        let (&last, last_f) = self.frames().iter().next_back()?;
        let end = if last_f.is_end() {
            last
        } else {
            last.saturating_add(1)
        };
        (end > first).then_some((first, end))
    }

    /// **O quadro-FONTE que o quadro `frame` mostra** — o quadro do ciclo. Dentro do
    /// vão é a identidade; fora, o pre/post behavior mapeia de volta para dentro
    /// (Loop/PingPong) ou para a borda (Hold). Quando nada aparece (`None` do
    /// [`map_frame`], ou camada sem chave), devolve o próprio `frame`.
    ///
    /// É o que a TIRA destaca e o que os fantasmas usam como "agora".
    #[must_use]
    pub fn source_frame(&self, frame: Frame) -> Frame {
        self.span()
            .and_then(|s| map_frame(self.cycle, s, frame))
            .unwrap_or(frame)
    }

    /// **O quadro em que a AUTORIA age** — e ele NÃO é sempre o quadro-fonte.
    ///
    /// A distinção é o que separa "tempo novo" de "tempo repetido":
    /// - sob uma **repetição** (`Loop`/`PingPong`), o tempo fora do vão **não é tempo
    ///   novo** — é o vão de novo. Desenhar no quadro 30 de um Loop de 12 tem de editar
    ///   o desenho que está NA TELA (o quadro 6), e a edição aparece em todas as
    ///   voltas. Autorar no quadro cru criaria uma chave em 30, esticando o vão e
    ///   QUEBRANDO o ciclo que o usuário acabou de ligar. (Memória
    ///   `feedback_derived_coordinate_seed_must_match_sample`.)
    /// - sob `Hold`/`None` (os defaults), o tempo depois do vão **é tempo novo**: o
    ///   último desenho está só segurando a tela. Desenhar ali cria a chave ALI — é
    ///   assim que uma animação cresce, quadro a quadro. Mapear de volta para a última
    ///   chave mataria o autokey.
    #[must_use]
    pub fn authoring_frame(&self, frame: Frame) -> Frame {
        let Some(span) = self.span() else {
            return frame;
        };
        let side = if frame < span.0 {
            self.cycle.pre
        } else if frame >= span.1 {
            self.cycle.post
        } else {
            return frame; // dentro do vão: identidade, sempre
        };
        match side {
            CycleMode::Loop | CycleMode::PingPong => {
                map_frame(self.cycle, span, frame).unwrap_or(frame)
            }
            CycleMode::None | CycleMode::Hold => frame,
        }
    }

    /// **O desenho ativo no quadro `frame`, honrando o ciclo da camada** — o que o
    /// RENDER amostra. Dentro do vão é idêntico a [`Self::drawing_at`]; fora, o
    /// pre/post behavior decide (nada / segura / repete / vai-e-volta).
    ///
    /// (O ciclo default reproduz o caminho cru EXATAMENTE — mas o mapeamento roda
    /// sempre, porque uma camada com **sentinela de fim** precisa dele: no cru, o
    /// quadro depois da sentinela é vazio, e é o `post` que decide se aquilo é o
    /// fim do desenho ou um hold. Atalhar por "ciclo == default" faria a exposição
    /// fixa da última chave apagar a arte.)
    #[must_use]
    pub fn drawing_at_cycled(&self, frame: Frame) -> Option<DrawingId> {
        let span = self.span()?;
        self.drawing_at(map_frame(self.cycle, span, frame)?)
    }

    /// **A pose do desenho que o quadro `frame` mostra** — o par de
    /// [`Self::drawing_at_cycled`], e pelo MESMO mapa.
    ///
    /// A arte e o lugar dela têm de sair pela mesma porta: amostrar o desenho pelo ciclo
    /// e a pose pelo quadro cru poria a arte da 2ª volta do Loop no lugar do quadro que
    /// não existe (`feedback_derived_coordinate_seed_must_match_sample`).
    #[must_use]
    pub fn offset_at_cycled(&self, frame: Frame) -> Vec2 {
        let Some(span) = self.span() else {
            return Vec2::ZERO;
        };
        let Some(src) = map_frame(self.cycle, span, frame) else {
            return Vec2::ZERO;
        };
        // A chave ATIVA no quadro-fonte (não `frames[src]`: dentro de um hold, o quadro
        // exibido não é a chave — a pose é a da chave que segura a tela).
        self.active_key(src)
            .map_or(Vec2::ZERO, |k| self.frame_offset(k))
    }

    // ── navegação por DESENHO (o "flip" do animador, W3.T3.5) ────────────────

    /// A chave REAL anterior a `frame` (estritamente) — o desenho anterior. Estando
    /// no meio de um hold, cai no INÍCIO da exposição atual (e o toque seguinte vai
    /// ao desenho de verdade anterior); é a semântica de F/G do Harmony.
    #[must_use]
    pub fn prev_drawing_key(&self, frame: Frame) -> Option<Frame> {
        self.frames()
            .range((Bound::Unbounded, Bound::Excluded(frame)))
            .rev()
            .find(|(_, f)| !f.is_end())
            .map(|(&k, _)| k)
    }

    /// A próxima chave REAL depois de `frame` (estritamente) — o desenho seguinte.
    #[must_use]
    pub fn next_drawing_key(&self, frame: Frame) -> Option<Frame> {
        self.frames()
            .range((Bound::Excluded(frame), Bound::Unbounded))
            .find(|(_, f)| !f.is_end())
            .map(|(&k, _)| k)
    }

    /// A chave ATIVA em `frame` (a maior `≤ frame`, sentinela inclusa) — o que a
    /// tira destaca e o autokey consulta.
    #[must_use]
    pub fn active_key(&self, frame: Frame) -> Option<Frame> {
        self.frames().range(..=frame).next_back().map(|(&k, _)| k)
    }

    /// A próxima chave que é um **KEYFRAME** (pula breakdowns E sentinelas) — os
    /// EXTREMOS entre os quais o tween interpola.
    ///
    /// Usar `next_drawing_key` aqui é o bug clássico: depois de gerar 3 inbetweens,
    /// o "desenho seguinte" passa a ser um BREAKDOWN, e re-tweenar interpolaria entre
    /// a chave e o próprio inbetween (gerando lixo entre 0 e 2 em vez de regenerar
    /// entre 0 e 8). O `exclude_breakdowns` do GP existe por isto.
    #[must_use]
    pub fn next_keyframe_key(&self, frame: Frame) -> Option<Frame> {
        self.frames()
            .range((Bound::Excluded(frame), Bound::Unbounded))
            .find(|(_, f)| f.drawing.is_some() && f.kind == KeyKind::Keyframe)
            .map(|(&k, _)| k)
    }

    /// O KEYFRAME em `frame` ou antes dele (pula breakdowns) — o extremo A do tween
    /// quando o playhead parou em cima de um inbetween.
    #[must_use]
    pub fn keyframe_at_or_before(&self, frame: Frame) -> Option<Frame> {
        self.frames()
            .range(..=frame)
            .rev()
            .find(|(_, f)| f.drawing.is_some() && f.kind == KeyKind::Keyframe)
            .map(|(&k, _)| k)
    }

    /// As chaves REAIS em ordem, com a exposição (nº de quadros) de cada uma — a
    /// própria tira de frames. A exposição da última chave real é `1` quando nada a
    /// delimita (o hold implícito é infinito; a tira mostra 1 e o usuário estica).
    #[must_use]
    pub fn cells(&self) -> Vec<(Frame, DrawingId, u32)> {
        self.frames()
            .iter()
            .filter_map(|(&k, f)| f.drawing.map(|d| (k, d)))
            .map(|(k, d)| {
                let dur = self.duration_at(k).max(1);
                (k, d, dur)
            })
            .collect()
    }
}
