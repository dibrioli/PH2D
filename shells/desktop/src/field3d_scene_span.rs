//! ⭐⭐⭐ **O ALCANCE DOS CONTROLOS** — de quem ele é, e o que o pode mexer.
//!
//! # Por que um módulo próprio
//!
//! Ele é a resposta a **dois reports do Enio em dois dias**, e cada um deixou uma lei que só se
//! entende por extenso: o alcance vem da **peça** e não da câmera (2026-08-30), e fica **travado**
//! enquanto a mão está no controlo, senão o slider realimenta-se a si próprio. Com as duas escritas
//! o `field3d_scene_panel.rs` passava dos **600** do HR-18.
//! ⚠️ **A cura é partir por assunto, nunca uma excepção declarada.**
//!
//! O irmão [`super::panel`] responde *«o que o painel mostra»*; este responde *«até onde cada
//! controlo vai»*.

/// ⭐ **O raio da peça do quadro ANTERIOR** — o insumo do [`gesture_span`].
///
/// ⚠️ **Do quadro anterior de propósito:** a peça deste ainda não foi cozida quando o painel é
/// publicado. Um quadro de atraso num ALCANCE é invisível; ler a câmera era visível a cada roda.
///
/// `0` quando não há peça — o piso do [`gesture_span`] responde por isso.
pub(crate) fn piece_radius() -> f32 {
    crate::field3d_smoke::with_smoke(|s| {
        s.doc.as_ref().and_then(|d| {
            ph2d_field_eval::bounds::bounding_ball(d, &ph2d_field_eval::hybrid::Registry::new())
                .map(|b| b.radius)
        })
    })
    .flatten()
    .unwrap_or(0.0)
}

thread_local! {
    /// ⭐⭐⭐ **O alcance TRAVADO, e a seleção para quem ele foi calculado.**
    ///
    /// Ver [`latched_span`]. `None` = ainda não há nenhum.
    static ALCANCE: std::cell::Cell<Option<(u64, f32)>> = const { std::cell::Cell::new(None) };
}

/// ⭐⭐⭐ **O ALCANCE NÃO SE MEXE ENQUANTO A MÃO ESTÁ NO CONTROLO** — e foi um report que o obrigou.
///
/// # ⛔⛔ A 1.ª versão desta wave era o defeito ESPELHADO
///
/// Ela tirou o alcance da câmera (report do zoom) e pô-lo na **peça**, em oitavas — e a oitava
/// resolvia o caso comum: arrastar uma largura dentro de uma não mexe no alcance. ⛔ **Mas quando
/// ela vira, vira para o DOBRO**, a meio do arrasto: a `scale` do mapeamento cursor→valor
/// **metade**, e o número salta. Report do Enio, no mesmo dia: *«arrastar os sliders ficou bizarro
/// mudando valores aos pulos»*.
///
/// ⚠️ **Trocar um incómodo contínuo por um salto discreto é pior**, e é o que a oitava fez sozinha:
/// com a câmera, o alcance ao menos era **constante durante o arrasto** (a roda não gira enquanto o
/// dedo arrasta).
///
/// # A lei que fica
///
/// O alcance é calculado **uma vez por seleção** e travado. Enquanto o artista mexe naquele objeto,
/// ele é uma constante — venha o que vier à peça. Escolher outro objeto (ou o mesmo outra vez)
/// re-ajusta.
///
/// ⚠️ **O preço, nomeado:** quem arrastar uma largura muito para além do que ela era chega ao fim do
/// curso do slider. ⭐ O campo numérico ao lado **não tem esse teto** — e voltar a clicar no objeto
/// re-ajusta o slider à peça nova. *Um alcance que persegue o valor que ele próprio escreve é a
/// definição de um controlo não idempotente.*
pub(crate) fn latched_span(selected: Option<bevy_ecs::entity::Entity>) -> f32 {
    let chave = selected.map_or(u64::MAX, bevy_ecs::entity::Entity::to_bits);
    latched_span_for(chave, piece_radius())
}

/// A lei da [`latched_span`] com os dois insumos à vista — a porta que o gate atravessa.
///
/// ⚠️ **Uma função-irmã que o teste chama é a única forma de medir a trava**: pela porta de cima o
/// raio vem do traçado, que num teste não existe, e as duas chamadas dariam o mesmo por acidente —
/// o gate passaria sem nada a defender.
pub(crate) fn latched_span_for(chave: u64, raio: f32) -> f32 {
    ALCANCE.with(|c| match c.get() {
        Some((k, v)) if k == chave => v,
        _ => {
            let v = gesture_span(raio);
            c.set(Some((chave, v)));
            v
        }
    })
}

/// ⭐⭐⭐ **O ALCANCE DO GESTO É DA PEÇA, E EM OITAVAS** — nunca da câmera.
///
/// # ⛔⛔ O report que a obrigou (Enio, 2026-08-30)
///
/// *«o ZOOM muda os parâmetros do objeto no painel»* — e mudava mesmo. O alcance dos sliders saía de
/// `cam.half_extent * 2.0`, com a nota ao lado a explicá-lo: *«uma dimensão maior do que o quadro é
/// uma cujo efeito não se vê»*. A razão é boa e a consequência é inaceitável: **aproximar a câmera —
/// um gesto que não toca no objeto — move todos os controlos dele**, e quem estiver a arrastar um
/// deles vê o número mudar de escala debaixo do dedo.
///
/// ⚠️ **E é o mesmo defeito do outro report do mesmo dia** (*«Bend não funcionou e esticou a
/// peça»*): a banda da dobra (`from`/`to`) é uma posição, e uma posição é `Span::Free` — a faixa
/// dela vinha inteira da câmera. Ajustar a banda com um enquadramento e voltar com outro dá dois
/// resultados para o mesmo gesto.
///
/// # A lei que fica
///
/// O alcance é `4×` o raio da peça, **arredondado para cima até à potência de dois**. As duas
/// metades são precisas:
///
/// - **da PEÇA**: a câmera deixa de ter voto, e é isso que o report pede;
/// - **em OITAVAS**: um alcance contínuo na peça teria o defeito simétrico — arrastar uma largura
///   mudaria o alcance, e o botão fugiria do dedo
///   ([`ph2d_field::Span`] não sabe disto; quem sabe é este sítio). Com a oitava, uma largura pode
///   dobrar antes de o alcance se mexer, e quando se mexe é **uma vez**, entre gestos.
///
/// ⚠️ **O piso existe porque uma peça pode ser minúscula** — sem ele, uma esfera de raio `0,001`
/// daria um slider cujo curso inteiro é invisível.
pub(crate) fn gesture_span(piece_radius: f32) -> f32 {
    const PISO: f32 = 1.0;
    const ALCANCE: f32 = 4.0;
    let alvo = (piece_radius * ALCANCE).max(PISO);
    // `exp2(ceil(log2 x))` — a menor potência de dois que cobre o alvo.
    let oitava = alvo.log2().ceil();
    oitava.exp2().max(PISO)
}
