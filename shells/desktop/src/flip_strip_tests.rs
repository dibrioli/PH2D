//! Gates da tira (`flip_strip`), módulo-irmão pelo cap de LOC.
//!
//! Foco: as duas maneiras de nascer uma chave a partir de outra — **cópia** (Key Dup) e
//! **instância** (Key Instance). A diferença não é cosmética: a instância é o que dá ao
//! multiframe algo para deduplicar (`flip_multiframe::targets`) e ao animador o reuso de
//! arte de um ciclo.

use super::*;
use ph2d_editor::tool::PanelEvent;
use ph2d_flip::{DrawingId, FlipObjectId, Hold, KeyKind};

/// Um doc com UM objeto, UMA camada e a chave 0 (com desenho próprio), playhead em 0.
fn doc_with_key0() -> (FlipDoc, FlipObjectId, LayerId, Playhead) {
    let mut doc = FlipDoc::default();
    let oid = doc.push_object("Flip");
    let obj = doc.object_mut(oid).unwrap();
    let lid = obj.add_layer("Layer 1");
    obj.insert_frame(lid, 0, Hold::Implicit, KeyKind::Keyframe);
    (doc, oid, lid, Playhead::default())
}

fn did_at(doc: &FlipDoc, oid: FlipObjectId, lid: LayerId, f: Frame) -> Option<DrawingId> {
    doc.object(oid)?.layer(lid)?.drawing_at(f)
}

/// Clica um botão da barra pelo caminho REAL (o `PanelEvent` que o painel empurra).
fn click(
    id: ph2d_editor::NodeId,
    doc: &mut FlipDoc,
    lid: LayerId,
    playhead: &mut Playhead,
    strip: &mut FlipStrip,
) -> bool {
    apply_panel_event(
        &PanelEvent::Click(id),
        doc,
        Some(lid),
        playhead,
        strip,
        false,
    )
}

/// 🔴 **Key Instance cria uma chave que COMPARTILHA o desenho.**
///
/// É o `duplicate as instance` do GP (o *linked duplicate* do Blender): a arte é UMA só,
/// referenciada por duas chaves. Editar num quadro edita no outro — é assim que um ciclo
/// reusa desenho sem duplicá-lo.
///
/// Mutação que sangra: trocar `DupMode::Instance` por `Deep` no braço do botão (o `users`
/// cai para 1 e os `DrawingId`s divergem).
#[test]
fn the_instance_button_makes_two_keys_share_one_drawing() {
    let (mut doc, oid, lid, mut ph) = doc_with_key0();
    let mut strip = FlipStrip::default();
    let d0 = did_at(&doc, oid, lid, 0).expect("a chave 0 tem desenho");

    assert!(click(
        ph2d_editor::ids::FLIP_KEY_INSTANCE,
        &mut doc,
        lid,
        &mut ph,
        &mut strip
    ));

    // A chave nova entrou depois da exposição da atual (o mesmo lugar do Key Dup).
    let d1 = did_at(&doc, oid, lid, 1).expect("a chave nova nao existe: o botao nao criou nada");
    assert_eq!(
        d1, d0,
        "a chave nova ganhou um desenho PROPRIO — isso e uma copia, nao uma instancia"
    );

    let drawing = doc.object(oid).unwrap().drawing(d0).unwrap();
    assert_eq!(drawing.users(), 2, "o refcount nao subiu: o elo nao existe");
    assert!(
        drawing.is_instanced(),
        "o desenho nao se declara instanciado — a celula nunca acende o pontinho"
    );
}

/// **O elo é REAL: editar por uma chave aparece na outra.** É a promessa que o usuário
/// enxerga (e a razão de o multiframe deduplicar por `DrawingId`: sem o dedup, um gesto
/// aplicaria o pincel DUAS vezes neste mesmo buffer).
#[test]
fn editing_an_instanced_drawing_shows_up_in_the_other_key() {
    let (mut doc, oid, lid, mut ph) = doc_with_key0();
    let mut strip = FlipStrip::default();
    click(
        ph2d_editor::ids::FLIP_KEY_INSTANCE,
        &mut doc,
        lid,
        &mut ph,
        &mut strip,
    );

    // Desenha PELA chave 0.
    let d0 = did_at(&doc, oid, lid, 0).unwrap();
    doc.object_mut(oid)
        .unwrap()
        .drawing_mut(d0)
        .unwrap()
        .strokes
        .push(ph2d_flip::FlipStroke::new());

    // E lê PELA chave 1 — sem saber que é a mesma arte.
    let d1 = did_at(&doc, oid, lid, 1).unwrap();
    let seen = doc.object(oid).unwrap().drawing(d1).unwrap().strokes.len();
    assert_eq!(
        seen, 1,
        "o traco desenhado no quadro 0 nao apareceu no quadro 1: o elo esta morto"
    );
}

/// **O irmão de presença** (`feedback_absence_gate_needs_a_presence_sibling`): o Key **Dup**
/// continua sendo uma CÓPIA. Sem este gate, o de cima ficaria verde num mundo em que
/// `duplicate_frame` compartilhasse SEMPRE — e aí o Dup teria virado instância em silêncio,
/// que é o oposto do que o animador pede quando clica em "duplicar".
#[test]
fn the_dup_button_still_makes_an_independent_copy() {
    let (mut doc, oid, lid, mut ph) = doc_with_key0();
    let mut strip = FlipStrip::default();
    let d0 = did_at(&doc, oid, lid, 0).unwrap();

    assert!(click(
        ph2d_editor::ids::FLIP_KEY_DUP,
        &mut doc,
        lid,
        &mut ph,
        &mut strip
    ));

    let d1 = did_at(&doc, oid, lid, 1).expect("o Dup nao criou chave");
    assert_ne!(
        d1, d0,
        "o Key Dup passou a COMPARTILHAR a arte — editar um quadro editaria o outro"
    );
    let obj = doc.object(oid).unwrap();
    assert_eq!(obj.drawing(d0).unwrap().users(), 1);
    assert!(!obj.drawing(d0).unwrap().is_instanced());
}

/// **Apagar uma das duas chaves NÃO leva a arte junto.** O refcount é o que segura o
/// desenho: com 2 usuários, remover uma chave decrementa para 1 e o outro quadro continua
/// desenhado. (Sem isso, apagar um quadro de um ciclo apagaria o ciclo inteiro.)
#[test]
fn deleting_one_of_two_instanced_keys_keeps_the_art_alive() {
    let (mut doc, oid, lid, mut ph) = doc_with_key0();
    let mut strip = FlipStrip::default();
    click(
        ph2d_editor::ids::FLIP_KEY_INSTANCE,
        &mut doc,
        lid,
        &mut ph,
        &mut strip,
    );
    let d0 = did_at(&doc, oid, lid, 0).unwrap();
    doc.object_mut(oid)
        .unwrap()
        .drawing_mut(d0)
        .unwrap()
        .strokes
        .push(ph2d_flip::FlipStroke::new());
    // O cenário que este gate diz testar TEM de existir: se o botão parar de instanciar,
    // o que segue seria uma cópia comum — verde por outro motivo (o teste passaria sem
    // nunca ter exercitado o caminho compartilhado).
    assert!(
        doc.object(oid).unwrap().drawing(d0).unwrap().is_instanced(),
        "o fixture nao criou uma instancia: o resto deste teste nao prova nada"
    );

    // O botão de apagar age na chave que o playhead vê — que é a 1 (o click do Instance
    // seekou para lá). Apaga-a.
    assert!(click(
        ph2d_editor::ids::FLIP_KEY_DELETE,
        &mut doc,
        lid,
        &mut ph,
        &mut strip
    ));

    let d0_now = did_at(&doc, oid, lid, 0).expect("a chave 0 sumiu junto com a instancia");
    let obj = doc.object(oid).unwrap();
    let drawing = obj
        .drawing(d0_now)
        .expect("a arte foi reclamada com 1 usuario vivo");
    assert_eq!(drawing.strokes.len(), 1, "o traco do quadro 0 evaporou");
    assert_eq!(drawing.users(), 1);
    assert!(
        !drawing.is_instanced(),
        "o desenho continua se dizendo instanciado com uma chave so"
    );
}

// ── criar chave no MEIO da tira (não só na última) + Unlink encerra multiframe ──

/// 🔴 **Uma chave nova entra no MEIO da tira**, não só depois da última (smoke do Enio
/// 2026-07-14: *"quadros ou instâncias só podem ser criados no fim dos quadros"*).
///
/// Estando numa chave que NÃO é a última, o alvo `chave + duração` cai sobre a próxima
/// chave real — antes o insert colidia e falhava mudo. Agora `open_gap_at` abre a fenda.
///
/// Mutação que sangra: tirar o `open_gap_at` do braço de Key Add/Dup/Instance (as chaves
/// ficam `[0, 5]`, a nova nunca entra).
#[test]
fn a_new_key_is_inserted_in_the_middle_not_only_at_the_end() {
    let (mut doc, oid, lid, mut ph) = doc_with_key0(); // chave 0
    doc.object_mut(oid)
        .unwrap()
        .insert_frame(lid, 5, Hold::Implicit, KeyKind::Keyframe);
    let mut strip = FlipStrip::default(); // playhead em 0 → a chave 0 (não é a última)

    assert!(click(
        ph2d_editor::ids::FLIP_KEY_DUP,
        &mut doc,
        lid,
        &mut ph,
        &mut strip
    ));

    let keys: Vec<Frame> = doc
        .object(oid)
        .unwrap()
        .layer(lid)
        .unwrap()
        .frames()
        .keys()
        .copied()
        .collect();
    assert_eq!(
        keys,
        vec![0, 5, 6],
        "a chave nova nao entrou no meio: o alvo colidiu com a chave 5 e o insert falhou (o bug)"
    );
}

/// **O Unlink ENCERRA a multisseleção** — senão o próximo Sculpt seria multiframe e
/// editaria os dois quadros de novo, e como a cópia nasce idêntica à origem o resultado
/// idêntico PARECE que o vínculo voltou (smoke do Enio 2026-07-14).
#[test]
fn unlink_ends_the_multi_selection() {
    let (mut doc, oid, lid, mut ph) = doc_with_key0();
    let mut strip = FlipStrip::default();
    // Instancia a chave 0 (o playhead vai para a chave nova).
    click(
        ph2d_editor::ids::FLIP_KEY_INSTANCE,
        &mut doc,
        lid,
        &mut ph,
        &mut strip,
    );
    // O usuário multi-seleciona os dois quadros na tira (para multiframe).
    strip.selection = vec![0, 1];

    // Desvincula a chave sob o playhead (a instância).
    assert!(click(
        ph2d_editor::ids::FLIP_KEY_UNLINK,
        &mut doc,
        lid,
        &mut ph,
        &mut strip
    ));
    assert_eq!(
        strip.selection.len(),
        1,
        "o Unlink nao encerrou a multissselecao — o proximo Sculpt religaria os quadros"
    );
    let _ = oid;
}

// ── régua de scrub: move o playhead sem tocar na seleção ────────────────────────

/// 🔴 **A régua de scrub move o playhead SEM tocar na multisseleção** (W7.3, smoke do
/// Enio 2026-07-14: *"não podemos arrastar o playhead sem desselecionar os outros
/// quadros"*).
///
/// É o ponto INTEIRO da régua — o animador scrubba entre os quadros marcados (para ver o
/// falloff / re-ancorar) sem desmontar o multiframe. Clicar numa CÉLULA é que seleciona; a
/// régua só transporta. O painel já resolveu o QUADRO (`FlipStripSnapshot::scrub_frame`);
/// o shell aqui só faz o seek.
///
/// Mutação que sangra: fazer o braço de `FLIP_SCRUB` reescrever `strip.selection` (como o
/// clique de célula faz) — a seleção deixa de ser `[0, 4, 8]`; ou não fazer o seek — o
/// playhead não chega ao 4.
#[test]
fn the_scrub_lane_moves_the_playhead_without_touching_the_selection() {
    let (mut doc, oid, lid, mut ph) = doc_with_key0();
    doc.object_mut(oid)
        .unwrap()
        .insert_frame(lid, 4, Hold::Implicit, KeyKind::Keyframe);
    doc.object_mut(oid)
        .unwrap()
        .insert_frame(lid, 8, Hold::Implicit, KeyKind::Keyframe);
    let mut strip = FlipStrip::default();
    strip.selection = vec![0, 4, 8]; // multiframe armado

    let edited = apply_panel_event(
        &PanelEvent::SetValue(ph2d_editor::ids::FLIP_SCRUB, 4.0),
        &mut doc,
        Some(lid),
        &mut ph,
        &mut strip,
        false,
    );

    assert!(!edited, "scrub e transporte, nao edicao de documento");
    assert_eq!(
        doc.object(oid).unwrap().frame_at(&ph),
        4,
        "a regua nao levou o playhead ao quadro 4"
    );
    assert_eq!(
        strip.selection,
        vec![0, 4, 8],
        "a regua DESMONTOU a multiselecao — o ponto inteiro dela e nao tocar a selecao"
    );
}
