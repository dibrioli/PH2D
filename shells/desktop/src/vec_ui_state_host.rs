//! **QUEM é o hospedeiro desta seleção, e o que o painel vê dele** — irmão do
//! [`super`](crate::vec_ui_state_edit) pelo teto de 600 LOC do HR-18, cortado por
//! RESPONSABILIDADE: ali ficam os VERBOS (capturar, instalar, gravar, deslocar); aqui, a pergunta
//! que todos eles fazem primeiro e a projeção que o painel recebe.
//!
//! ⚠️ **Uma pergunta, uma resposta.** Antes da auditoria de 2026-08-23 cada gesto da seção
//! respondia por si a *"quem é o hospedeiro?"* — cinco `if let [host] = selected_paths()`
//! espalhados pelo `render_loop`, e nenhum deles era o que o `publish` usava para PINTAR. Desde
//! que o hospedeiro passou a ser DERIVADO da seleção, isso seria discordância garantida: o painel
//! mostrava as poses de uma forma e o knob escrevia noutra.

use super::*;

/// **O HOSPEDEIRO desta seleção** — a forma cujas poses a seção STATES mostra e o `Rec` grava.
///
/// # A lei, numa frase
///
/// *O hospedeiro é a forma que GOVERNA o que está selecionado.*
///
/// - uma forma só ⇒ **ela** (o caso de sempre, e o degenerado desta lei);
/// - várias ⇒ a **forma-ancestral mais próxima cuja sub-árvore contém todas**.
///
/// # ⚠️ Por que a segunda linha teve de existir (auditoria de 2026-08-23)
///
/// A regra era *"exatamente UMA forma, senão nada"*, e ela tornava a booleana viva
/// **inanimável pelo produto**: tocar um operando **seleciona o grupo inteiro** (lei deliberada do
/// `object_selection_for`), então a seção STATES — e o interruptor **Preview** com ela — não era
/// sequer PINTADA. Não dimmed, não vazia: ausente, sem uma palavra a dizer o que faltava.
///
/// ⚠️ Isso contradizia a lei escrita da própria seção (*"oferecida para qualquer forma ÚNICA, com
/// estados ou sem — uma seção que só existisse onde já há estados tornaria a feature alcançável
/// apenas onde ela já foi usada"*): a face vazia existia para *sem poses* e **não** para *a
/// seleção é um grupo*.
///
/// ⭐ E a resposta não é inventar um hospedeiro: é **derivá-lo**. Um grupo não tem `VecPathId`, mas
/// a forma que o contém tem — e é exatamente ela que [`members`] já governa. Escolher em silêncio
/// continua proibido: o painel **nomeia** o hospedeiro que esta porta devolveu.
///
/// ⚠️ Sem ancestral-forma comum (duas formas soltas, ou um grupo na raiz) continua a ser `None` —
/// e aí a seção pinta a **face vazia com a dica**, em vez de desaparecer.
pub(crate) fn host_of_selection(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    selected: &[VecPathId],
) -> Option<VecPathId> {
    match selected {
        [] => None,
        [only] => Some(*only),
        _ => covering_shape(sim, scene, map, selected),
    }
}

/// A forma-ancestral mais próxima cuja sub-árvore contém **todos** os selecionados.
///
/// ⚠️ A pertença é medida pelo [`members`], que é a mesma porta que o `Rec` usa para saber o que
/// gravar. Uma travessia própria aqui daria duas respostas a *"quem este hospedeiro governa"*, e a
/// seção mostraria as poses de um objeto enquanto o `Rec` gravaria as de outro.
fn covering_shape(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    selected: &[VecPathId],
) -> Option<VecPathId> {
    let mut cur = entity_of(map, *selected.first()?)?;
    for _ in 0..crate::vec_entities::MAX_DEPTH {
        // ⚠️ **A própria forma é o primeiro candidato**, e não só os ancestrais dela: um botão com
        // um rótulo dentro tem o botão como hospedeiro, e uma varredura que começasse no PAI
        // pularia justamente o caso mais comum — a seleção do widget inteiro.
        if let Some(vp) = sim.world().get::<ph2d_ecs::VecPathRef>(cur) {
            let governed = members(sim, scene, map, vp.0);
            if selected.iter().all(|id| governed.contains(id)) {
                return Some(vp.0);
            }
        }
        cur = sim.world().get::<ph2d_ecs::ChildOf>(cur)?.parent();
    }
    None
}

/// **O que a seção STATES mostra para esta seleção** — `None` = não oferecer a seção **de todo**,
/// o que só acontece com a seleção VAZIA.
///
/// ⚠️ **A face VAZIA é a que torna a feature alcançável**, e ela deixou de existir sem querer: com
/// uma booleana selecionada (que é sempre uma seleção MÚLTIPLA) o `publish` devolvia `None` e a
/// seção não era sequer pintada — nem o cabeçalho, nem o interruptor de preview, nem uma palavra a
/// dizer o que faltava. Hoje uma seleção sem hospedeiro publica um estado com `host: None`, e o
/// painel pinta o cabeçalho mais a dica. É a mesma lei da seção de física, cuja face vazia é a
/// importante.
#[must_use]
// ⚠️ **O trio `(sim, scene, map)` é a CONVENÇÃO DA CASA**, e não um descuido: as portas irmãs
// (`capture`, `install`, `apply`, `members`) recebem os três exactamente assim. Embrulhá-los num
// tipo só para esta função criaria uma **segunda** convenção ao lado da que quatro portas já
// seguem — e duas convenções para o mesmo trio custam mais que o lint. O dia em que o trio virar
// um tipo, ele vira para as cinco de uma vez.
#[allow(clippy::too_many_arguments)]
pub(crate) fn publish(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    selected: &[VecPathId],
    states: &StateSets,
    live: Option<usize>,
    preview_on: bool,
    move_all: bool,
) -> Option<ph2d_panel_vector::state::UiStatesState> {
    if selected.is_empty() {
        return None;
    }
    let Some(h) = host_of_selection(sim, scene, map, selected) else {
        // ⚠️ **A seleção não tem forma que a governe** (duas formas soltas, ou um grupo na raiz).
        // A seção existe e diz o que falta; tudo o que precisa de hospedeiro fica desligado.
        //
        // ⛔ **Escrito campo a campo, e NÃO com `..Default::default()`.** Um default deixaria um
        // campo novo entrar na face vazia sem ninguém decidir o que ela mostra — e uma face vazia
        // é uma decisão de produto, não um resto. (Foi por isso que o `Default` derivado saiu: ele
        // ainda obrigaria o painel a inventar uma curva, que divergiria do `DEFAULT_EASING`.)
        return Some(ph2d_panel_vector::state::UiStatesState {
            host: None,
            recorded: [false; 4],
            role_labels: StateRole::ALL.map(|r| ph2d_i18n::tr(r.i18n_key()).to_string()),
            live: None,
            #[allow(clippy::cast_possible_truncation)]
            duration_s: ph2d_ui_state::DEFAULT_DURATION_S as f32,
            spring: None,
            // ⚠️ A PREVIEW fica de fora: ela entrega o rato a todos os hospedeiros, e oferecê-la
            // aqui seria um botão a agir sobre coisa nenhuma que o artista tem em mãos.
            preview: None,
            move_all: None,
            easing: ph2d_ui_state::DEFAULT_EASING,
            bindings: Vec::new(),
        });
    };
    let (duration, easing) = states.timing(h);
    Some(ph2d_panel_vector::state::UiStatesState {
        // ⭐ **O painel NOMEIA o hospedeiro**, e isso não é enfeite: desde que ele passou a ser
        // DERIVADO da seleção, o artista tem de poder ver de quem são as poses que ele está a
        // gravar. *Escolher em silêncio é como um estado nasce pendurado no objeto errado.*
        host: Some(
            entity_of(map, h)
                .and_then(|e| sim.world().get::<ph2d_ecs::Name>(e))
                .map_or_else(String::new, |n| n.as_str().to_owned()),
        ),
        recorded: StateRole::ALL.map(|r| states.role(h, r).is_some()),
        // ⚠️ Os rótulos saem do CATÁLOGO, não de uma lista no painel: um papel novo aparece
        // nomeado sem ninguém tocar na UI, e nenhuma segunda lista pode envelhecer ao lado
        // desta.
        role_labels: StateRole::ALL.map(|r| ph2d_i18n::tr(r.i18n_key()).to_string()),
        live,
        #[allow(clippy::cast_possible_truncation)]
        duration_s: duration as f32,
        // **A MOLA** — pela MESMA porta que o motor pergunta (`StateSets::spring`), e não por uma
        // leitura ao lado: se o painel derivasse a resposta noutro lugar, ele pintaria as linhas
        // de mola sobre uma cena a andar por curva.
        #[allow(clippy::cast_possible_truncation)]
        spring: states
            .spring(h)
            .map(|s| (s.stiffness as f32, s.damping as f32)),
        // **O interruptor da PREVIEW** só é oferecido quando existe pose autorada em ALGUM
        // hospedeiro — que é exatamente a condição em que [`UiPreview::enter`] liga.
        //
        // ⚠️ **A pergunta é a MESMA que o modelo faz**, e não uma segunda cópia da regra: um
        // botão pintado sobre uma cena sem poses seria um clique que não faz nada, e o artista
        // não teria como saber que o que falta é gravar um estado.
        preview: (!states.is_empty()).then_some(preview_on),
        // **Mover carregando todos os estados** só faz sentido onde ESTE hospedeiro tem pose
        // gravada — a pergunta é sobre o widget que se vai arrastar, não sobre a cena (é o
        // oposto do `preview`, que entrega o rato a todos).
        move_all: StateRole::ALL
            .iter()
            .any(|&r| states.role(h, r).is_some())
            .then_some(move_all),
        // **A CURVA** — publicada SEMPRE, e não só onde há pose gravada.
        //
        // ⚠️ É a mesma regra da duração, que está uma linha acima: as duas descrevem *como este
        // hospedeiro transita*, e afiná-las antes de gravar o primeiro estado é a ordem natural
        // (escolho o feel, depois poso). Escondê-las até haver pose seria uma seção que muda de
        // tamanho enquanto o artista trabalha, pelo motivo que ele não vê.
        easing,
        // ⭐ **AS LIGAÇÕES** — o que faz este hospedeiro mudar de pose sozinho.
        //
        // ⚠️ **O papel viaja como ÍNDICE em `StateRole::ALL`**, a mesma régua dos `role_labels`
        // acima: o painel não alcança a `ph2d-ui-state`, então ele não pode nomear um papel nem
        // reconhecer um enum — e uma segunda tabela de nomes ali envelheceria no dia em que um
        // papel nascesse.
        bindings: states
            .bindings(h)
            .iter()
            .map(|b| {
                (
                    b.name.clone(),
                    StateRole::ALL
                        .iter()
                        .position(|&r| r == b.role)
                        .unwrap_or(0),
                )
            })
            .collect(),
    })
}

#[cfg(test)]
#[path = "vec_ui_state_host_tests.rs"]
mod tests;
