//! **As edições de uma STRIP** — pôr, tirar, mover, aparar, esticar, retimar e fadear
//! (ADR-0115).
//!
//! Split do [`crate::intent_apply`] pelo cap de 700 LOC, na mesma linha de corte que o irmão
//! [`crate::intent_apply_fade`] já usava: aquele arquivo roteia o vocabulário INTEIRO da
//! timeline (transporte, chaves, tracks, clips, containers, markers), e a família da pilha é
//! um assunto próprio — *o que uma strip é, e o que se pode fazer com ela*.
//!
//! ⚠️ O `host` é **recebido**, nunca re-lido: quem o resolve é o `apply_intent`, uma vez por
//! intent (ADR-0133 §5), e uma segunda leitura aqui poderia responder outra coisa se alguém
//! mexesse no estado no meio — a edição cairia no documento enquanto o animador olha um
//! container.

use crate::StackHost;
use crate::intent::TimelineIntent;
use crate::intent_apply::edit_at;
use crate::state::TimelineState;
use crate::strip_edge_edit::{
    MAX_STRIP_SPEED, MIN_STRIP_SPEED, mark_edge, stretch_strip, trim_strip,
};

/// # O que cada edição de fade/retime FAZ
///
/// A prosa abaixo morava nos variants do [`crate::intent`] e mudou-se para cá quando aquele
/// arquivo cruzou o cap de 700 LOC — pela linha que o módulo dele já declarava: *"o
/// vocabulário é lido por todo mundo que dirige o painel, e o roteador por quem muda o que
/// uma edição faz"*. Uma linha de resumo lá, o porquê aqui.
///
/// **`StretchStrip`** — o retime que o `TrimStrip` recusa ser. A fatia da fonte fica FIXA e
/// o span é redimensionado em volta dela, então a taxa cai fora: `speed = slice / span`.
/// Nada é revelado nem escondido; os MESMOS quadros tocam, mais devagar ou mais depressa, e
/// a borda que você NÃO arrasta fica onde está. É o *único* caminho de autoria do `speed` na
/// UI, e existe porque uma taxa é coisa que se sente, não número que se digita (os NLEs a
/// fazem ferramenta separada — Rate Stretch do Premiere, Change Speed do Resolve; não temos
/// paleta de ferramentas dentro de um painel, então ela é o modificador do gesto que
/// modifica).
///
/// **`SetStripEase`** — o fade PRÓPRIO do strip numa borda (ADR-0115 B4), o que a alça de
/// quina autora. É a MESMA curva do crossfade, e esse é o ponto inteiro: onde um vizinho
/// sobrepõe esta borda, a SOBREPOSIÇÃO define a janela e este número é ignorado
/// (`ClipLane::blend_in`/`blend_out`), então os dois nunca podem discordar. O painel RECUSA o
/// arraste ali em vez de escrever um número que ninguém vai ler (a Unity acinzenta o campo e
/// o re-rotula "Blend"). Sem ele, um strip SOZINHO numa lane não podia fadear: entrava e saía
/// duro — os campos existiam e o avaliador já os honrava, ninguém os escrevia.
///
/// **`SetStripLead`** — o fade de FORA, o "lead" que mora no GAP ao lado (Enio, 2026-07-16
/// para a borda inicial, 2026-07-19 para a final). É um blend DIFERENTE do de cima: aquele
/// cruza contra este clipe enquanto ele TOCA; este cruza contra o quadro CONGELADO daquela
/// borda, então o objeto atravessa o vão enquanto o clipe toca limpo. Clampado a `[0, gap]` —
/// o fade de fora vive no vão e não pode invadir o vizinho — e escrevê-lo LIMPA o ease de
/// dentro na MESMA borda: a alça de fade está de um lado da borda ou do outro, nunca dos dois.
///
/// **`SetStripCurve`** — a FORMA do fade numa borda (Enio, 2026-07-31: *"no menu do botão
/// direito sobre o fade de uma strip vamos colocar as mesmas opções de easing que temos nos
/// clips"*), onde os dois de cima autoram o COMPRIMENTO. Uma curva por BORDA, então um strip
/// pode acelerar para fora de um clipe e desacelerar para dentro do próximo. ⚠️ `None` é o
/// `smoothstep` de fábrica, e é um `Option` deliberado em vez de um preset: a rampa de fábrica
/// **não está no catálogo** (a mais próxima, `Quad InOut`, dá `0.125` onde ela dá `0.15625`),
/// então guardar um preset reescreveria a forma de todo fade já autorado. ⚠️ E ela **não
/// alcança um crossfade de SOBREPOSIÇÃO**, a mesma lei que o `ease_in` já obedece: ali os dois
/// pesos precisam somar exatamente 1 (o que vale porque `smoothstep(1−u) == 1−smoothstep(u)`),
/// e uma curva assimétrica de um lado só faria a lane somar menos que 1 no meio do crossfade,
/// com a pose afundando para as lanes de baixo.
///
/// Roteia um intent da família da pilha.
///
/// O `match` é exaustivo sobre ela e cai num `unreachable!` fora dela — o chamador só
/// encaminha os desta família, e o `match` DELE é exaustivo sobre o enum inteiro, então um
/// variant novo é erro de compilação lá, não um braço morto aqui.
pub(crate) fn apply(state: &mut TimelineState, host: StackHost, intent: TimelineIntent) {
    use TimelineIntent as I;
    match intent {
        I::AddStrip {
            lane,
            source,
            t_start,
            t_end,
        } => edit_at(state, host, |doc, host, _| {
            let _ = doc.add_strip_to(host, lane, source, t_start.max(0.0), t_end.max(0.0));
        }),
        I::RemoveStrip { lane, id } => edit_at(state, host, |doc, host, _| {
            doc.remove_strip_in(host, lane, id);
        }),
        I::DuplicateStrip { lane, id } => edit_at(state, host, |doc, host, _| {
            doc.duplicate_strip_in(host, lane, id);
        }),
        I::MoveStrip {
            lane,
            to_lane,
            id,
            t_start,
        } => edit_at(state, host, |doc, host, _| {
            doc.move_strip_in(host, lane, to_lane, id, t_start.max(0.0));
        }),
        I::TrimStrip {
            lane,
            id,
            edge,
            t,
            from,
        } => edit_at(state, host, |doc, host, _| {
            if let Some(s) = doc.strip_in_mut(host, lane, id) {
                trim_strip(s, edge, t);
                mark_edge(s, false, edge, from);
            }
        }),
        I::StretchStrip {
            lane,
            id,
            edge,
            t,
            from,
        } => edit_at(state, host, |doc, host, _| {
            if let Some(s) = doc.strip_in_mut(host, lane, id) {
                stretch_strip(s, edge, t);
                mark_edge(s, true, edge, from);
            }
        }),
        I::SetStripLoop {
            lane,
            id,
            loop_mode,
        } => edit_at(state, host, |doc, host, _| {
            if let Some(s) = doc.strip_in_mut(host, lane, id) {
                s.loop_mode = loop_mode;
            }
        }),
        // Authoring a strip's fades — both live in `intent_apply_fade` (LOC cap). The inward
        // ease and the outward lead, at either edge; each runs inside this `edit_at` bracket.
        I::SetStripEase {
            lane,
            id,
            edge,
            seconds,
        } => edit_at(state, host, |doc, host, _| {
            crate::intent_apply_fade::set_ease(doc, host, lane, id, edge, seconds);
        }),
        I::SetStripLead {
            lane,
            id,
            edge,
            seconds,
        } => edit_at(state, host, |doc, host, _| {
            crate::intent_apply_fade::set_lead(doc, host, lane, id, edge, seconds);
        }),
        I::SetStripCurve {
            lane,
            id,
            edge,
            curve,
        } => edit_at(state, host, |doc, host, _| {
            crate::intent_apply_fade::set_curve(doc, host, lane, id, edge, curve);
        }),
        I::SetStripSpeed { lane, id, speed } => edit_at(state, host, |doc, host, _| {
            if let Some(s) = doc.strip_in_mut(host, lane, id) {
                // The span follows the rate, `t_start` pinned — the same edit
                // `stretch_strip` makes, stated as a number instead of felt as a
                // drag (see the variant's docs).
                s.speed = speed.clamp(MIN_STRIP_SPEED, MAX_STRIP_SPEED); // CLAMP-OK: const bounds
                let slice = s.slice();
                if slice > 0.0 {
                    // Same edit, same change bar: typing a rate moves the END edge,
                    // and a mark that only appeared for the DRAG would make the two
                    // paths to one number look like two different edits.
                    let before = s.t_end;
                    s.t_end = s.t_start + slice / s.speed;
                    mark_edge(s, true, 1, before);
                }
            }
        }),
        other => unreachable!("intent_apply routes only the stack family here: {other:?}"),
    }
}
