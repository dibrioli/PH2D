//! **Sobre que chave estamos falando?** — os resolvedores da tira, módulo-irmão do
//! `flip_strip` pelo cap de LOC.
//!
//! Todo evento da tira começa pela mesma pergunta: qual objeto, qual camada, que quadro
//! (o CRU do playhead ou o FONTE, o do ciclo). São quatro funções sem opinião sobre o que
//! o evento faz — e é por isso que elas cabem juntas: quem as separa é o assunto
//! (*resolver* × *aplicar*), não o tamanho do arquivo.

use ph2d_core::Playhead;
use ph2d_flip::{FlipDoc, FlipObjectId, Frame, LayerId};

/// O objeto e a camada que a tira edita: o 1º objeto (igual ao `bake_stroke`) e a
/// camada ativa (fallback: o topo).
pub(crate) fn target(
    flip: &FlipDoc,
    active_layer: Option<LayerId>,
) -> Option<(FlipObjectId, LayerId)> {
    let obj = flip.objects().first()?;
    let lid = active_layer
        .filter(|id| obj.layer(*id).is_some())
        .or_else(|| obj.layers().last().map(|l| l.id))?;
    Some((obj.id, lid))
}

/// Leva o playhead ao quadro `f` (no FPS do objeto) e PAUSA — quem clica numa
/// célula quer ver aquele desenho, não continuar tocando a partir dele.
pub(crate) fn seek(playhead: &mut Playhead, fps: f32, f: Frame) {
    playhead.pause();
    playhead.seek_frame(i64::from(f.max(0)), f64::from(fps));
}

/// **O intervalo de tween AGORA** — `(objeto, camada, chave A, chave B)` em torno do
/// quadro-fonte atual. `None` se não há dois keyframes para interpolar entre.
///
/// **Porta única:** o botão Add Tween e o construtor da sessão de correção de pares
/// ([`crate::flip_tween_correct::build`]) chamam ESTA função — o plano corrigido tem de ser
/// commitado no MESMO intervalo em que foi construído, e duas resoluções divergiriam (a
/// sessão pinada a um intervalo, o Add commitando noutro, e as correções seriam ignoradas
/// em silêncio).
pub(crate) fn current_tween_interval(
    flip: &FlipDoc,
    active_layer: Option<LayerId>,
    playhead: &Playhead,
) -> Option<(FlipObjectId, LayerId, Frame, Frame)> {
    let (oid, lid) = target(flip, active_layer)?;
    let frame = source_frame(flip, oid, lid, playhead);
    let layer = flip.object(oid)?.layer(lid)?;
    let from = layer.keyframe_at_or_before(frame)?;
    let to = layer.next_keyframe_key(frame)?;
    (from != to).then_some((oid, lid, from, to))
}

/// O **quadro-fonte** da camada agora: sob um ciclo, o quadro do vão que está sendo
/// exibido. Toda op de chave age NELE (a célula que se vê é a que se edita).
pub(crate) fn source_frame(
    flip: &FlipDoc,
    oid: FlipObjectId,
    lid: LayerId,
    playhead: &Playhead,
) -> Frame {
    let Some(obj) = flip.object(oid) else {
        return 0;
    };
    let raw = obj.frame_at(playhead);
    obj.layer(lid).map_or(raw, |l| l.source_frame(raw))
}

/// **Ligar um ciclo dá uma exposição REAL à última célula.**
///
/// O hold implícito da última chave é infinito — sem sentinela, o vão fecha em
/// `última + 1` e ela expõe UM quadro. Num Loop isso é um piscar: as outras células
/// seguram 8 quadros e a última passa num frame. Então, ao ligar Loop/Ping-Pong,
/// materializamos a exposição da última chave **igual à da anterior** (o ritmo que o
/// animador já estabeleceu; `1` se não há anterior).
///
/// Não é mágica escondida: a exposição vira uma sentinela VISÍVEL (a célula alarga na
/// tira) e editável (a caixa **Hold**). Idempotente — se a última chave já tem
/// exposição fixa, nada muda.
pub(crate) fn ensure_cycle_span(flip: &mut FlipDoc, oid: FlipObjectId, lid: LayerId) -> bool {
    let Some(layer) = flip.object(oid).and_then(|o| o.layer(lid)) else {
        return false;
    };
    let cells = layer.cells();
    let Some(&(last, _, _)) = cells.last() else {
        return false;
    };
    if layer.duration_at(last) != 0 {
        return false; // já tem sentinela: a exposição é explícita, respeita
    }
    let prev = cells
        .len()
        .checked_sub(2)
        .and_then(|i| cells.get(i))
        .map_or(1, |&(_, _, e)| e);
    flip.object_mut(oid)
        .is_some_and(|o| o.set_exposure(lid, last, prev))
}
