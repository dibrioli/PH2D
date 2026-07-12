//! ADR-0114 W3.T3.4 — **o autokey por-tool**: qual desenho um gesto edita, e o que
//! nasce se não houver nenhum ali.
//!
//! Um único ponto de resolução para TODA autoria do Flip (caneta e borracha hoje;
//! escultura amanhã) — porque a regra é sutil e errá-la em UM caminho já estraga o
//! documento do usuário:
//!
//! - **Caneta** (`FlipEdit::Draw`): desenhar depois do hold de uma chave cria uma
//!   chave **em branco** — é o próximo desenho da animação. Com *Additive*, nasce
//!   como cópia (desenhar POR CIMA do anterior).
//! - **Borracha** (`FlipEdit::Modify`): SEMPRE **duplicata**. Se nascesse em branco,
//!   o usuário apagaria um quadro novo e vazio — enquanto o desenho que ele estava
//!   vendo continuaria intacto num quadro anterior. É o erro que o Grease Pencil
//!   documenta, e é irrecuperável sem undo.
//! - Em cima de uma chave real, ninguém cria nada.
//! - Com o **autokey desligado**, o gesto edita o desenho que está NA TELA (mesmo
//!   que a chave dele tenha começado 12 quadros atrás) — o comportamento de app de
//!   desenho.

use ph2d_core::Playhead;
use ph2d_flip::{AutokeyPolicy, DrawingId, FlipDoc, FlipObjectId, LayerId};

/// O que o gesto vai fazer com o desenho.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum FlipEdit {
    /// Acrescenta tinta (caneta).
    Draw,
    /// Modifica o que já existe (borracha, escultura, tint).
    Modify,
}

/// O desenho que este gesto deve editar — **criando a chave se o autokey mandar**.
///
/// `None` = não há onde desenhar (sem objeto/camada, camada travada) ou, no caso de
/// `Modify`, não há nada na tela para modificar (apagar o vazio não pode inventar
/// um quadro).
pub(crate) fn target_drawing(
    flip: &mut FlipDoc,
    playhead: &Playhead,
    active_layer: Option<LayerId>,
    strip: &crate::flip_strip::FlipStrip,
    edit: FlipEdit,
) -> Option<(FlipObjectId, LayerId, DrawingId)> {
    let oid = flip.objects().first().map(|o| o.id)?;
    let obj = flip.object_mut(oid)?;
    let lid = active_layer
        .filter(|id| obj.layer(*id).is_some())
        .or_else(|| obj.layers().last().map(|l| l.id))?;
    if obj.layer(lid).is_some_and(|l| l.locked) {
        return None; // camada travada recusa qualquer gesto (regra do GP)
    }
    // **O quadro de AUTORIA** (`authoring_frame`), que não é sempre o quadro cru nem
    // sempre o quadro-fonte: sob um Loop/Ping-Pong o tempo fora do vão é o vão DE NOVO,
    // então o gesto edita o desenho que está na tela (e a edição aparece em todas as
    // voltas, que é o que um ciclo significa). Sob Hold/None — os defaults — o tempo
    // depois do vão é tempo NOVO, e é ali que a chave nova nasce; mapear de volta
    // mataria o autokey. Detalhe em `FlipLayer::authoring_frame`.
    let frame = obj.layer(lid).map_or_else(
        || obj.frame_at(playhead),
        |l| l.authoring_frame(obj.frame_at(playhead)),
    );
    // O que está na tela AGORA (o desenho do hold em curso), no quadro de autoria.
    let on_screen = obj.layer(lid).and_then(|l| l.drawing_at(frame));

    if !strip.autokey {
        // Sem autokey: edita o que está na tela. Se não há nada (antes da 1ª chave),
        // a caneta ainda precisa de um lugar — cria a primeira chave.
        return match (on_screen, edit) {
            (Some(d), _) => Some((oid, lid, d)),
            (None, FlipEdit::Draw) => obj
                .ensure_key(lid, frame, AutokeyPolicy::Blank)
                .map(|d| (oid, lid, d)),
            (None, FlipEdit::Modify) => None,
        };
    }
    // Com autokey: a política é da FERRAMENTA. A borracha nunca nasce em branco.
    let policy = match edit {
        FlipEdit::Draw if strip.additive => AutokeyPolicy::Duplicate,
        FlipEdit::Draw => AutokeyPolicy::Blank,
        FlipEdit::Modify => AutokeyPolicy::Duplicate,
    };
    if edit == FlipEdit::Modify && on_screen.is_none() {
        return None; // nada na tela: a borracha não inventa quadro
    }
    obj.ensure_key(lid, frame, policy).map(|d| (oid, lid, d))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flip_strip::FlipStrip;
    use ph2d_core::Vec2;
    use ph2d_flip::{FlipStroke, Hold, KeyKind};

    /// Um doc com um objeto, uma camada e uma chave em 0 com um traço.
    fn doc() -> (FlipDoc, LayerId) {
        let mut d = FlipDoc::new();
        let oid = d.push_object("O");
        let o = d.object_mut(oid).unwrap();
        o.fps = 12.0;
        let l = o.add_layer("L");
        let did = o
            .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
            .unwrap();
        let mut s = FlipStroke::new();
        s.push_default(Vec2::new(0.0, 0.0));
        s.push_default(Vec2::new(1.0, 1.0));
        o.drawing_mut(did).unwrap().strokes.push(s);
        (d, l)
    }

    /// O playhead no quadro `f` (12 fps).
    fn at(f: i32) -> Playhead {
        let mut p = Playhead::new(1.0 / 12.0);
        p.pause();
        p.seek(f64::from(f) / 12.0);
        p
    }

    /// A caneta no rabo do hold cria uma chave EM BRANCO — o desenho seguinte.
    #[test]
    fn drawing_past_the_hold_creates_a_blank_key() {
        let (mut d, l) = doc();
        let strip = FlipStrip::default(); // autokey ON, additive OFF
        let (oid, _, did) =
            target_drawing(&mut d, &at(5), Some(l), &strip, FlipEdit::Draw).unwrap();
        let o = d.object(oid).unwrap();
        assert!(
            o.drawing(did).unwrap().strokes.is_empty(),
            "nasce em branco"
        );
        assert_eq!(o.layer(l).unwrap().cells().len(), 2, "chave nova em 5");
    }

    /// **A borracha SEMPRE duplica** — senão ela apagaria um quadro novo e vazio, e
    /// o desenho que o usuário vê ficaria intacto lá atrás.
    #[test]
    fn erasing_past_the_hold_duplicates_the_drawing_on_screen() {
        let (mut d, l) = doc();
        let strip = FlipStrip::default();
        let (oid, _, did) =
            target_drawing(&mut d, &at(5), Some(l), &strip, FlipEdit::Modify).unwrap();
        let o = d.object(oid).unwrap();
        assert_eq!(
            o.drawing(did).unwrap().strokes.len(),
            1,
            "a arte veio junto (é ELA que se apaga)"
        );
        // E é uma CÓPIA: o quadro 0 não é tocado.
        let d0 = o.drawing_at(l, 0).unwrap();
        assert_ne!(d0, did);
    }

    /// Additive: a caneta também duplica (desenhar por cima do anterior).
    #[test]
    fn additive_makes_the_pen_duplicate_too() {
        let (mut d, l) = doc();
        let strip = FlipStrip {
            additive: true,
            ..Default::default()
        };
        let (oid, _, did) =
            target_drawing(&mut d, &at(5), Some(l), &strip, FlipEdit::Draw).unwrap();
        assert_eq!(
            d.object(oid).unwrap().drawing(did).unwrap().strokes.len(),
            1
        );
    }

    /// Sem autokey, o gesto edita o desenho QUE ESTÁ NA TELA (a chave de trás) —
    /// nenhuma chave nova nasce.
    #[test]
    fn without_autokey_the_gesture_edits_what_is_on_screen() {
        let (mut d, l) = doc();
        let strip = FlipStrip {
            autokey: false,
            ..Default::default()
        };
        let (oid, _, did) =
            target_drawing(&mut d, &at(5), Some(l), &strip, FlipEdit::Draw).unwrap();
        let o = d.object(oid).unwrap();
        assert_eq!(did, o.drawing_at(l, 0).unwrap(), "é o desenho do quadro 0");
        assert_eq!(o.layer(l).unwrap().cells().len(), 1, "nenhuma chave nova");
    }

    /// A borracha no vazio não inventa quadro nenhum.
    #[test]
    fn erasing_nothing_creates_nothing() {
        let mut d = FlipDoc::new();
        let oid = d.push_object("O");
        let l = d.object_mut(oid).unwrap().add_layer("L");
        let strip = FlipStrip::default();
        assert!(target_drawing(&mut d, &at(3), Some(l), &strip, FlipEdit::Modify).is_none());
        assert_eq!(d.object(oid).unwrap().layer(l).unwrap().cells().len(), 0);
    }

    /// Camada travada recusa tudo.
    #[test]
    fn a_locked_layer_refuses_the_gesture() {
        let (mut d, l) = doc();
        d.object_mut(FlipObjectId(0))
            .unwrap()
            .layer_mut(l)
            .unwrap()
            .locked = true;
        let strip = FlipStrip::default();
        assert!(target_drawing(&mut d, &at(0), Some(l), &strip, FlipEdit::Draw).is_none());
    }
}
