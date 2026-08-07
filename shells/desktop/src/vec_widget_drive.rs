//! **A ROW DIRIGE A FORMA** — o valor vivo de um controle autorado, projetado no que a cena
//! desenha (plano UI/UX W8b.3).
//!
//! É a metade que fecha o laço do §2: *o desenho é a PELE, o widget é o COMPORTAMENTO*, e aqui o
//! comportamento passa a ter CONSEQUÊNCIA. Sem isto o painel gerado responde ao ponteiro e não
//! muda nada na arte — um controle que se move e não move nada.
//!
//! # A porta única: o que uma row FAZ é derivado do TIPO dela
//!
//! [`drive_of`] é a ÚNICA resposta a *"o que este widget dirige?"*, e a resposta sai do
//! [`WidgetKind`] — nunca de um campo autorado ao lado. Um Slider produz um número e o número que
//! uma forma tem é a **opacidade**; um Toggle produz um sim/não e o sim/não que uma forma tem é
//! **aparecer**. Um segundo controle (*"esta row dirige o quê?"*) teria de ser mantido de acordo
//! com o tipo, e no dia em que discordassem o painel pintaria um slider que apaga a forma.
//!
//! ⚠️ **Quem não RESPONDE não dirige**, e o conjunto é menor que o dos controles: um `Button`
//! responde ao ponteiro e produz um EVENTO, não um valor — dirigir uma forma com ele exigiria um
//! vocabulário de VERBOS (*"faz o quê?"*) que nenhum consumidor pede ainda, e um vocabulário sem
//! consumidor é desenhar no escuro (a lei que o `OverrideSlot` da W5a já escreveu). O painel
//! **não oferece** o vínculo nesses tipos, em vez de o oferecer e o deixar inerte.
//!
//! # VISTA, nunca documento
//!
//! O resultado é publicado no [`VecViewState`] — a projeção por-frame — e **nada aqui escreve a
//! cena**. Três consequências, e as três são o motivo:
//!
//! 1. arrastar um slider **não vira passo de undo** (o undo deste editor é por DIFF do mundo, e
//!    uma pose derivada escrita no documento faria cada gesto encher a fila — a lição que o
//!    ADR-0153 pagou no auto layout);
//! 2. a tinta autorada **sobrevive**: arrastar até zero e voltar devolve a arte AO BIT, porque
//!    ninguém a reescreveu;
//! 3. e o fato tem UM dono — a `VecViewState` é a mesma lista que o olhinho da Hierarquia e os
//!    tokens já alimentam.
//!
//! ⚠️ **O preço honesto, nomeado:** o valor de uma row vive no `WidgetStore`, que é de runtime ⇒
//! ele **não é salvo**. Reabrir o projeto devolve os controles ao default e a arte ao que o
//! artista autorou. Persistir a POSIÇÃO de um controle é a W4b/W8a, não esta.

use ph2d_ecs::{Entity, SimWorld, VecWidget, VecWidgetBind};
use ph2d_editor::interaction::{InteractiveState, WidgetStore};
use ph2d_editor::widget::{CheckboxValue, WidgetKind};
use ph2d_vec_scene::{BoundStyle, VecPathId, VecViewState};

use crate::vec_entities::VecEntityMap;

/// O que uma row faz com a forma que ela dirige.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Drive {
    /// A forma aparece, ou não.
    Visible(bool),
    /// A forma desvanece: `255` = como o artista a pintou.
    Opacity(u8),
}

/// **Este tipo de widget pode dirigir uma forma?** — a porta que o painel pergunta para OFERECER
/// o vínculo, e o resolvedor para o HONRAR.
///
/// Duas metades da mesma pergunta: oferecida sem ser honrada dá um controle que não faz nada;
/// honrada sem ser oferecida dá uma capacidade que ninguém alcança.
#[must_use]
pub(crate) fn bindable(kind: WidgetKind) -> bool {
    matches!(
        kind,
        WidgetKind::Slider | WidgetKind::Toggle | WidgetKind::Checkbox
    )
}

/// O que o estado VIVO deste controle diz à forma. `None` = o estado não é do tipo que dirige.
///
/// ⚠️ O `Indeterminate` de um checkbox conta como **ligado**: ele quer dizer *"parte dos filhos"*,
/// e uma forma não tem partes — colapsá-lo em desligado faria a forma sumir num estado que o
/// artista lê como *"meio marcado"*.
#[must_use]
pub(crate) fn drive_of(st: &InteractiveState) -> Option<Drive> {
    match st {
        InteractiveState::Toggle { on, .. } => Some(Drive::Visible(*on)),
        InteractiveState::Checkbox { value, .. } => {
            Some(Drive::Visible(*value != CheckboxValue::Unchecked))
        }
        InteractiveState::Slider { value, .. } => {
            // O slider do catálogo anda em `0..=1`; o clamp defende contra um estado semeado fora
            // dessa faixa em vez de deixar o `as u8` embrulhar.
            Some(Drive::Opacity((value.clamp(0.0, 1.0) * 255.0).round() as u8))
        }
        _ => None,
    }
}

/// **O que os controles autorados dizem à cena neste frame.**
///
/// Percorre os widgets que têm vínculo, lê o valor VIVO da row no store e devolve o par
/// `(forma dirigida, o que fazer)`. Vazio quando nada está vinculado — que é todo documento que
/// já existe, e é o que faz o desenho deles ficar byte-idêntico ao mundo pré-W8b.3.
///
/// ⚠️ **A row é encontrada pela MESMA chave que o gerador escreve** (`ui_panel_spec::key_of` sobre
/// o `Name`) — uma segunda derivação aqui daria um painel cujas rows respondem e um resolvedor que
/// procura por um id que ninguém registou, e o sintoma seria *"o slider não faz nada"* sem erro
/// nenhum.
#[must_use]
pub(crate) fn resolve(
    sim: &SimWorld,
    map: &VecEntityMap,
    store: &WidgetStore,
) -> Vec<(VecPathId, Drive)> {
    let w = sim.world();
    let mut out = Vec::new();
    for &bits in map.values() {
        let e = Entity::from_bits(bits);
        if w.get_entity(e).is_err() {
            continue;
        }
        let (Some(widget), Some(bind)) = (w.get::<VecWidget>(e), w.get::<VecWidgetBind>(e)) else {
            continue;
        };
        if !WidgetKind::from_code(widget.kind).is_some_and(bindable) {
            continue;
        }
        let Some(name) = w.get::<ph2d_ecs::Name>(e) else {
            continue;
        };
        let row = ph2d_editor::ids::authored_row_id(&crate::ui_panel_spec::key_of(&name.0));
        // Um widget cuja row não está no painel COMMITADO ainda não tem valor vivo — o vínculo
        // fica dormente em vez de inventar um default e mexer na arte sem ninguém ter tocado nada.
        let Some(drive) = store.get(row).and_then(drive_of) else {
            continue;
        };
        out.push((bind.target as VecPathId, drive));
    }
    out
}

/// **Aplica os drives na projeção do frame.**
///
/// ⚠️ A opacidade é FUNDIDA na entrada de tinta que a forma já tenha, nunca acrescentada ao lado:
/// o consumidor lê **uma** entrada por forma (`bound_style(id)` devolve a primeira), então uma
/// segunda entrada para a mesma forma seria silenciosamente descartada — e qual das duas some
/// dependeria da ordem de iteração de um mapa.
pub(crate) fn apply(drives: &[(VecPathId, Drive)], view: &mut VecViewState) {
    for &(id, drive) in drives {
        match drive {
            Drive::Visible(true) => {}
            Drive::Visible(false) => {
                if !view.hidden.contains(&id) {
                    view.hidden.push(id);
                }
            }
            Drive::Opacity(a) => {
                if let Some(b) = view.bound.iter_mut().find(|b| b.path == id) {
                    b.alpha = Some(a);
                } else {
                    view.bound.push(BoundStyle {
                        path: id,
                        alpha: Some(a),
                        ..BoundStyle::default()
                    });
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "vec_widget_drive_tests.rs"]
mod tests;
