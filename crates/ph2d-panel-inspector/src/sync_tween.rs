//! **A semente dos campos da secção TWEEN** (suplente #22).
//!
//! ⛔⛔⛔ **Esta linha existe porque a wave ANTERIOR desta linha a esqueceu, e quem a apanhou foi uma
//! FOTO** (19/09, a secção do raio): o painel mostrava `Direction 0 / −1` e `Reach 1 m` — os
//! valores de **fábrica** do `populate` — sobre um objecto autorado a `(1, 0)` com alcance `6`.
//!
//! ⚠️⚠️ *Números plausíveis são a pior forma deste defeito:* eles não parecem partidos, e o artista
//! que escreva por cima de um deles **grava o default nos outros**. É a mesma família que a secção
//! do ÁUDIO e a da CÂMERA pagaram em 10/09 e a da VIGIA em 16/09 — e esta é a **quarta** vez.
//!
//! # ⭐ E aqui só os NÚMEROS precisam de semente
//!
//! Os quatro grupos de chips (canal · curva · modo · fim) leem a selecção do **SNAPSHOT** dentro do
//! pintor, não do store — é a lei que a §11 pagou com um report (*«ler o store fazia o primeiro
//! clique depois de trocar de objecto mandar o valor do objecto anterior»*). ⇒ eles não têm estado
//! a semear, e um `set_button_state` aqui seria uma segunda resposta a *«qual está aceso?»*.
//!
//! # ⚠️ De ARESTA, nunca por quadro
//!
//! Reescrever sempre apagaria o que o artista está a digitar antes de o commit da shell chegar. A
//! aresta é a **assinatura** do instantâneo, e o foco e o arrasto ganham-lhe: *a mão do artista
//! ganha ao instantâneo*.

use std::hash::{Hash, Hasher};

use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::tween_edits::InspectorTweenInfo;

use crate::state::InspectorState;

/// ⚠️ **Só o que SEMEIA um widget entra** — e, como a lista escolhe qual tween se edita, a
/// assinatura tem de conter **o índice aberto**: sem ele, mudar de linha não re-semeava os campos
/// e o artista via os números do tween ANTERIOR.
fn assinatura(info: &InspectorTweenInfo, aberto: usize) -> u64 {
    let mut h = std::hash::DefaultHasher::new();
    info.entity_bits.hash(&mut h);
    aberto.hash(&mut h);
    info.rows.len().hash(&mut h);
    if let Some(r) = info.rows.get(aberto) {
        r.canal.hash(&mut h);
        for v in r.de.iter().chain(r.para.iter()) {
            v.to_bits().hash(&mut h);
        }
        // ⚠️ **A DURAÇÃO entra na assinatura** — sem ela, carregar num preset (que a reescreve)
        // não re-semeava o campo, e o artista via o número ANTERIOR sobre um relógio já mudado.
        r.duracao_us.hash(&mut h);
    }
    h.finish()
}

/// **A semente da secção** — chamada pelo [`crate::sync_sections`].
pub(crate) fn sync(
    host: &mut dyn PanelHostInternal,
    inspector_state: &mut InspectorState,
    entity_changed: bool,
) {
    let Some(info) = crate::state_components::current_inspector_tween() else {
        inspector_state.last_tween_sig = None;
        return;
    };
    let aberto = inspector_state
        .tween_selected
        .min(info.rows.len().saturating_sub(1));
    let sig = assinatura(&info, aberto);
    if !entity_changed && inspector_state.last_tween_sig == Some(sig) {
        return;
    }
    inspector_state.last_tween_sig = Some(sig);
    let Some(row) = info.rows.get(aberto) else {
        return;
    };
    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    // ⚠️ **A ORDEM é a dos arrays de ids**, e é assim que ela não pode divergir: as quatro
    // componentes de cada extremo, na mesma ordem em que o pintor as desenha.
    for (ids_, valores) in [
        (&crate::ids::INSP_TWEEN_DE, &row.de),
        (&crate::ids::INSP_TWEEN_PARA, &row.para),
    ] {
        for (&id, &v) in ids_.iter().zip(valores.iter()) {
            if focus == Some(id) || drag == Some(id) {
                continue; // a mão do artista ganha ao instantâneo
            }
            host.store_mut().set_number_value(id, f64::from(v));
        }
    }
    // ⭐⭐⭐ **A DURAÇÃO do relógio, em SEGUNDOS** — a quinta vez que esta casa escreve esta linha,
    // e a razão é sempre a mesma: *números plausíveis são a pior forma deste defeito*, porque não
    // parecem partidos e quem escreve por cima de um grava o default nos outros.
    //
    // ⛔ **As duas CAIXAS não entram aqui:** elas leem o valor do SNAPSHOT dentro do pintor, como
    // os chips — semeá-las seria a segunda resposta a *«está marcada?»*.
    if let Some(us) = row.duracao_us {
        let id = crate::ids::INSP_TWEEN_DURACAO;
        if focus != Some(id) && drag != Some(id) {
            #[allow(clippy::cast_precision_loss)]
            host.store_mut().set_number_value(id, us as f64 / 1e6);
        }
    }
}
