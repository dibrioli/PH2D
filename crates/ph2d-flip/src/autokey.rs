//! Autokey **por tool** (W3.T3.4) — port de `grease_pencil_frames.cc:344-378`.
//!
//! A regra que separa um app de animação de um app de desenho: **a ferramenta
//! decide o que nasce**.
//!
//! - **Desenhar** no rabo de um hold cria uma chave **EM BRANCO** (o artista está
//!   fazendo o próximo desenho; a pose anterior fica onde estava). Com *Additive*
//!   ligado, nasce como duplicata (desenhar POR CIMA do anterior).
//! - **Apagar / esculpir / tintar** SEMPRE nascem como **DUPLICATA** do desenho
//!   que estava na tela. Se nascessem em branco, a borracha "apagaria" um quadro
//!   que o usuário nem sabia que existia — ele acharia que apagou o desenho que
//!   via, e apagou um quadro novo, vazio, deixando o original intacto lá atrás.
//!   Este é o erro caro que o GP documenta.
//! - **Em cima de uma chave real, ninguém cria nada** — edita o que está lá.
//!
//! A chave nasce 1× por GESTO (o chamador chama no *down*), e o undo global
//! agrupa "criar a chave + o traço" num passo só, porque o registro é por diff
//! no fim do frame e o gesto inteiro fica dentro dele.

use crate::frame::{Hold, KeyKind};
use crate::ids::{DrawingId, Frame, LayerId};
use crate::object::{DupMode, FlipObject};
use crate::pose::Pose;

/// O que a ferramenta quer quando cai num quadro sem chave própria.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum AutokeyPolicy {
    /// Chave nova **em branco** (o desenho seguinte). Ferramentas que ADICIONAM
    /// tinta do zero: a caneta.
    #[default]
    Blank,
    /// Chave nova como **cópia** do desenho que estava na tela. Ferramentas que
    /// MODIFICAM o que existe: borracha, escultura, tint — e a caneta em modo
    /// *Additive*.
    Duplicate,
}

impl FlipObject {
    /// **Garante uma chave editável em `frame`** na camada, conforme a política da
    /// ferramenta, e devolve o desenho a editar.
    ///
    /// - já há chave REAL exatamente em `frame` → devolve o desenho dela (não cria
    ///   nada: desenhar sobre um keyframe não gera keyframe);
    /// - senão (rabo de hold, sentinela, antes da 1ª chave) → cria segundo a
    ///   política. Sem nada na tela para copiar, `Duplicate` degrada a `Blank`.
    ///
    /// `None` = a camada não existe ou está travada.
    pub fn ensure_key(
        &mut self,
        layer_id: LayerId,
        frame: Frame,
        policy: AutokeyPolicy,
    ) -> Option<DrawingId> {
        let layer = self.layer(layer_id)?;
        if layer.locked {
            return None;
        }
        // Chave real EXATAMENTE aqui? Então é ela que se edita.
        if let Some(f) = layer.frames().get(&frame)
            && let Some(d) = f.drawing
        {
            return Some(d);
        }
        // O molde da duplicata é a CHAVE em curso (não o quadro): é o desenho que
        // está na tela agora. Sentinela de fim não é molde (não há nada na tela).
        let source = if policy == AutokeyPolicy::Duplicate {
            layer
                .active_key(frame)
                .filter(|k| layer.frames().get(k).is_some_and(|f| f.drawing.is_some()))
        } else {
            None
        };

        // **A POSE da chave que está NA TELA** — a nova a herda, seja duplicata ou
        // branca. É o mesmo motivo nos dois casos: o gesto foi feito sobre a arte no
        // lugar em que ela aparece, e a mão do usuário converte tela→local pela pose que
        // ele está VENDO. Se a chave nova nascesse na pose neutra, o traço cairia
        // deslocado pelo tanto que a anterior estava (o seed teria de casar com o sample
        // — `feedback_derived_coordinate_seed_must_match_sample`).
        let shown_pose = layer
            .active_key(frame)
            .map_or(Pose::IDENTITY, |k| layer.frame_pose(k));

        let created = match source {
            // Duplicata PROFUNDA: instanciar faria a borracha comer o quadro de
            // origem junto (é o mesmo desenho).
            Some(src) if self.duplicate_frame(layer_id, src, frame, DupMode::Deep) => {
                self.layer(layer_id)?.drawing_at(frame)
            }
            Some(_) => None, // colisão: só se houvesse chave real aqui — já tratado
            None => self.insert_frame(layer_id, frame, Hold::Implicit, KeyKind::Keyframe),
        }?;
        // (A duplicata já herdou a pose em `duplicate_frame`; a chave BRANCA não — e é
        // ela que este passe cobre. Idempotente para as duas.)
        self.layer_mut(layer_id)?.set_frame_pose(frame, shown_pose);
        Some(created)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::FlipObjectId;
    use crate::stroke::FlipStroke;
    use ph2d_core::Vec2;

    /// Objeto com uma camada e uma chave em 0, cujo desenho tem 1 traço.
    fn obj() -> (FlipObject, LayerId, DrawingId) {
        let mut o = FlipObject::new(FlipObjectId(1), "O");
        let l = o.add_layer("L");
        let d = o
            .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
            .unwrap();
        let mut s = FlipStroke::new();
        s.push_default(Vec2::new(0.0, 0.0));
        s.push_default(Vec2::new(1.0, 1.0));
        o.drawing_mut(d).unwrap().strokes.push(s);
        (o, l, d)
    }

    /// Em cima de uma chave real: NINGUÉM cria nada (as duas políticas).
    #[test]
    fn on_an_existing_key_nothing_is_created() {
        for policy in [AutokeyPolicy::Blank, AutokeyPolicy::Duplicate] {
            let (mut o, l, d0) = obj();
            assert_eq!(o.ensure_key(l, 0, policy), Some(d0));
            assert_eq!(o.drawings().len(), 1, "{policy:?} criou desenho à toa");
        }
    }

    /// Desenhar no rabo do hold cria uma chave EM BRANCO — o próximo desenho.
    #[test]
    fn drawing_in_a_hold_tail_creates_a_blank_key() {
        let (mut o, l, d0) = obj();
        let d = o.ensure_key(l, 5, AutokeyPolicy::Blank).unwrap();
        assert_ne!(d, d0);
        assert!(o.drawing(d).unwrap().strokes.is_empty(), "nasce em branco");
        assert_eq!(o.drawing_at(l, 5), Some(d));
        assert_eq!(o.drawing_at(l, 4), Some(d0), "o anterior segue lá");
    }

    /// **A regra cara:** a borracha no rabo do hold DUPLICA — senão ela "apagaria"
    /// um quadro invisível e o desenho da tela ficaria intacto.
    #[test]
    fn erasing_in_a_hold_tail_duplicates_the_drawing_on_screen() {
        let (mut o, l, d0) = obj();
        let d = o.ensure_key(l, 5, AutokeyPolicy::Duplicate).unwrap();
        assert_ne!(d, d0, "desenho NOVO (cópia profunda)");
        assert_eq!(o.drawing(d).unwrap().strokes.len(), 1, "a arte veio junto");
        // E apagar nele NÃO toca o original (o quadro 0 continua com o traço).
        o.drawing_mut(d).unwrap().strokes.clear();
        assert_eq!(
            o.drawing(d0).unwrap().strokes.len(),
            1,
            "o quadro 0 intacto"
        );
    }

    /// Sem nada na tela para copiar (antes da 1ª chave), `Duplicate` degrada a
    /// branco em vez de falhar.
    #[test]
    fn duplicate_degrades_to_blank_with_nothing_to_copy() {
        let mut o = FlipObject::new(FlipObjectId(1), "O");
        let l = o.add_layer("L");
        let d = o.ensure_key(l, 7, AutokeyPolicy::Duplicate).unwrap();
        assert!(o.drawing(d).unwrap().strokes.is_empty());
        assert_eq!(o.drawing_at(l, 7), Some(d));
    }

    #[test]
    fn a_locked_layer_refuses_every_key() {
        let (mut o, l, _) = obj();
        o.layer_mut(l).unwrap().locked = true;
        assert_eq!(o.ensure_key(l, 5, AutokeyPolicy::Blank), None);
        assert_eq!(o.ensure_key(l, 0, AutokeyPolicy::Duplicate), None);
    }
}
