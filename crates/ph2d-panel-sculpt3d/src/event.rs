//! Painel → shell. Todo braço de knob é derivado de [`crate::rows`], então uma
//! row que existe é uma row que despacha.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal, seam_reset_button};
use ph2d_sculpt3d::{Alpha, Falloff, TransformKind, Verb};

use crate::rows;
use crate::state::{self, Sculpt3dIntent};

/// A posição de um id num array de opções — o inverso exato da ordem que o
/// pintor usa, e por isso o chip que acende e o valor que pousa não podem
/// discordar.
fn index_of(group: &[ph2d_a11y::NodeId], id: ph2d_a11y::NodeId) -> Option<usize> {
    group.iter().position(|&g| g == id)
}

/// Todo comando de um só toque: o id e o que ele significa. Uma tabela, e não uma
/// cascata de `if id == …`, porque o `event` e o `populate` têm de concordar sobre
/// a LISTA e uma cascata é o formato que apodrece calado.
pub(crate) const COMMANDS: &[(ph2d_a11y::NodeId, Sculpt3dIntent)] = &[
    (ids::SCULPT3D_DYNTOPO, Sculpt3dIntent::ToggleDyntopo),
    (ids::SCULPT3D_LEVEL_DOWN, Sculpt3dIntent::ChangeLevel(false)),
    (ids::SCULPT3D_LEVEL_UP, Sculpt3dIntent::ChangeLevel(true)),
    (ids::SCULPT3D_SUBDIVIDE, Sculpt3dIntent::Subdivide),
    (ids::SCULPT3D_REVERSE, Sculpt3dIntent::ReverseLevel),
    (ids::SCULPT3D_FLATTEN, Sculpt3dIntent::Flatten),
    (ids::SCULPT3D_REMESH, Sculpt3dIntent::Remesh),
    (ids::SCULPT3D_CLOSE_HOLES, Sculpt3dIntent::CloseHoles),
    (ids::SCULPT3D_BAKE_AO, Sculpt3dIntent::BakeAo),
    (ids::SCULPT3D_BAKE_SPRITE, Sculpt3dIntent::BakeToSprite),
    (ids::SCULPT3D_ALPHA_SPRITE, Sculpt3dIntent::AlphaFromSprite),
    (ids::SCULPT3D_DUPLICATE, Sculpt3dIntent::Duplicate),
    (ids::SCULPT3D_DELETE, Sculpt3dIntent::Delete),
    (ids::SCULPT3D_ISOLATE, Sculpt3dIntent::ToggleIsolate),
    (ids::SCULPT3D_MERGE, Sculpt3dIntent::Merge),
    (ids::SCULPT3D_EXTRACT, Sculpt3dIntent::Extract),
];

/// As quatro primitivas e as quatro operações de máscara, na ordem em que o
/// painel as lista.
/// **O intent de um chip de grupo** — a porta única dos três grupos cujo clique
/// é uma entrada de tabela.
///
/// ⚠️ Os outros grupos (verbo, falloff, alpha, matcap, detalhe) **não** entram
/// aqui de propósito: eles não empurram uma constante, eles COMPÕEM um
/// [`Sculpt3dUi`] a partir do retrato vivo. Enfiá-los nesta tabela obrigaria a
/// porta a receber o estado inteiro, e aí ela deixaria de ser uma tabela.
fn table_intent(id: ph2d_a11y::NodeId) -> Option<Sculpt3dIntent> {
    if let Some(i) = index_of(&ids::SCULPT3D_ADD, id) {
        return Some(ADD_INTENTS[i].clone());
    }
    if let Some(i) = index_of(&ids::SCULPT3D_TRANSFORM, id) {
        // ⚠️ **Ele ARMA, e o painel não decide o que "clicar o aceso" faz.** A
        // cena é quem sabe o que já está armado (`arm_transform`), então mandar
        // o TIPO — e não um `Option` — é o que impede o painel de guardar uma
        // segunda cópia do arm para calcular o desligamento.
        return Some(Sculpt3dIntent::ArmTransform(TransformKind::ALL[i]));
    }
    index_of(&ids::SCULPT3D_MASK_OP, id).map(|i| MASK_INTENTS[i].clone())
}

const ADD_INTENTS: [Sculpt3dIntent; 4] = [
    Sculpt3dIntent::AddSphere,
    Sculpt3dIntent::AddCube,
    Sculpt3dIntent::AddCylinder,
    Sculpt3dIntent::AddTorus,
];
const MASK_INTENTS: [Sculpt3dIntent; 4] = [
    Sculpt3dIntent::MaskClear,
    Sculpt3dIntent::MaskInvert,
    Sculpt3dIntent::MaskBlur,
    Sculpt3dIntent::MaskSharpen,
];

pub(crate) fn apply_event(
    _state: &mut crate::state::Sculpt3dPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    // Sem cena não há o que editar — e um intent emitido agora seria aplicado
    // ao primeiro barro que aparecesse, o que é pior que não fazer nada.
    let Some(snapshot) = state::current() else {
        return EventOutcome::from_bool(false);
    };
    let consumed = match ev {
        // Uma pista moveu: lê a pista que o arrasto deixou, transforma no valor
        // desta row, e emite o estado autorado INTEIRO com aquele campo trocado.
        WidgetEvent::ValueChanged(id) => {
            if let Some(row) = rows::row_for(id) {
                if id == row.chip {
                    // A edição do chip já foi espelhada na pista ligada a ele,
                    // que disparou o próprio `ValueChanged` e foi tratada lá.
                    // Engolir, ou uma edição notifica duas vezes.
                    true
                } else {
                    let track = host.store().slider(id).map_or(0.5, |(_, v)| v);
                    let mut ui = snapshot.ui;
                    (row.set)(&mut ui, row.value_of(track));
                    state::push_intent(Sculpt3dIntent::SetUi(ui));
                    true
                }
            } else {
                false
            }
        }
        // A ferramenta. A lista É `Verb::ALL`, então a ordem dos chips e a do
        // modelo não podem derivar.
        WidgetEvent::Click(id) if index_of(&ids::SCULPT3D_VERB, id).is_some() => {
            seam_reset_button(host, id);
            let i = index_of(&ids::SCULPT3D_VERB, id).expect("guard casou");
            let mut ui = snapshot.ui;
            let verb = Verb::ALL[i];
            crate::state::arm_verb_defaults(&mut ui, verb);
            state::push_intent(Sculpt3dIntent::SetUi(ui));
            true
        }
        WidgetEvent::Click(id) if index_of(&ids::SCULPT3D_FALLOFF, id).is_some() => {
            seam_reset_button(host, id);
            let i = index_of(&ids::SCULPT3D_FALLOFF, id).expect("guard casou");
            let mut ui = snapshot.ui;
            ui.brush.falloff = Falloff::ALL[i];
            state::push_intent(Sculpt3dIntent::SetUi(ui));
            true
        }
        // **O PADRÃO** — a opção `0` é o pincel LISO e as seguintes são os
        // padrões, a mesma aritmética do matcap logo abaixo. `checked_sub` e não
        // `- 1` pelo mesmo motivo: a opção zero não é o padrão `-1`, é a
        // AUSÊNCIA de padrão.
        WidgetEvent::Click(id) if index_of(&ids::SCULPT3D_ALPHA, id).is_some() => {
            seam_reset_button(host, id);
            let i = index_of(&ids::SCULPT3D_ALPHA, id).expect("guard casou");
            arm_alpha_chip(&snapshot, i);
            true
        }
        // **A LUZ** — a opção `0` é o rig do artista e as seguintes são os
        // matcaps, o mesmo deslocamento que o pintor usa. `checked_sub` e não
        // `- 1`: a opção zero não é o material `-1`, é a AUSÊNCIA de matcap.
        WidgetEvent::Click(id) if index_of(&ids::SCULPT3D_MATCAP, id).is_some() => {
            seam_reset_button(host, id);
            let i = index_of(&ids::SCULPT3D_MATCAP, id).expect("guard casou");
            let mut ui = snapshot.ui;
            ui.matcap = i.checked_sub(1).map(|k| u8::try_from(k).unwrap_or(u8::MAX));
            state::push_intent(Sculpt3dIntent::SetUi(ui));
            true
        }
        // ⚠️ **Recusa fora dos verbos de carimbo.** O pintor já não o oferece
        // ali, mas o roteador é a outra metade: um clique sintético (ou um id
        // que sobreviveu a uma troca de verbo no mesmo frame) armaria um flag
        // que nenhum dab lê, e o painel voltaria a mostrá-lo marcado no próximo
        // verbo que o oferece — um estado que o artista não pediu.
        WidgetEvent::Click(id)
            if id == ids::SCULPT3D_ACCUMULATE && snapshot.ui.brush.verb.accumulates() =>
        {
            seam_reset_button(host, id);
            let mut ui = snapshot.ui;
            ui.brush.accumulate = !ui.brush.accumulate;
            state::push_intent(Sculpt3dIntent::SetUi(ui));
            true
        }
        // ⚠️ **Gateado no padrão armado, como a row que o pinta.** Sem o guard
        // o clique chegaria a um interruptor que ninguém desenhou — e o estado
        // dele mudaria pelas costas do artista, que é a forma exata de um
        // controle nascer mentindo sobre o que a tela mostra.
        WidgetEvent::Click(id)
            if id == ids::SCULPT3D_ALPHA_PREVIEW && snapshot.ui.brush.alpha.is_some() =>
        {
            seam_reset_button(host, id);
            let mut ui = snapshot.ui;
            ui.alpha_preview = !ui.alpha_preview;
            state::push_intent(Sculpt3dIntent::SetUi(ui));
            true
        }
        WidgetEvent::Click(id) if id == ids::SCULPT3D_WIREFRAME => {
            seam_reset_button(host, id);
            let mut ui = snapshot.ui;
            ui.wireframe = !ui.wireframe;
            state::push_intent(Sculpt3dIntent::SetUi(ui));
            true
        }
        WidgetEvent::Click(id) if index_of(&ids::SCULPT3D_DETAIL, id).is_some() => {
            seam_reset_button(host, id);
            let i = index_of(&ids::SCULPT3D_DETAIL, id).expect("guard casou");
            let mut ui = snapshot.ui;
            ui.detail = u8::try_from(i).unwrap_or(0);
            state::push_intent(Sculpt3dIntent::SetUi(ui));
            true
        }
        // Os três eixos do espelho. Botões independentes e não um rádio: um
        // segmented é *um de N* por construção, e o ZBrush espelha em dois eixos
        // ao mesmo tempo.
        WidgetEvent::Click(id)
            if id == ids::SCULPT3D_SYM_X
                || id == ids::SCULPT3D_SYM_Y
                || id == ids::SCULPT3D_SYM_Z =>
        {
            seam_reset_button(host, id);
            let mut ui = snapshot.ui;
            let axis = if id == ids::SCULPT3D_SYM_X {
                &mut ui.symmetry.x
            } else if id == ids::SCULPT3D_SYM_Y {
                &mut ui.symmetry.y
            } else {
                &mut ui.symmetry.z
            };
            *axis = !*axis;
            state::push_intent(Sculpt3dIntent::SetUi(ui));
            true
        }
        // ⚠️ **TRÊS grupos, UMA porta.** Os três respondiam a mesma coisa —
        // *achou o índice, empurra a entrada `i` de uma tabela* — em três braços
        // idênticos a menos do nome da tabela. O quarto teria nascido copiando o
        // terceiro, e é assim que um deles ganha um `seam_reset_button` a menos
        // sem ninguém ver.
        WidgetEvent::Click(id) if table_intent(id).is_some() => {
            seam_reset_button(host, id);
            state::push_intent(table_intent(id).expect("guard casou"));
            true
        }
        WidgetEvent::Click(id) if COMMANDS.iter().any(|(k, _)| *k == id) => {
            seam_reset_button(host, id);
            let intent = COMMANDS
                .iter()
                .find(|(k, _)| *k == id)
                .map(|(_, i)| i.clone())
                .expect("guard casou");
            state::push_intent(intent);
            true
        }
        // Os cabeçalhos dobram. Estado de VISTA do painel, então nunca vira
        // intent — o shell não tem opinião sobre que seções estão abertas.
        WidgetEvent::Click(id) if is_section_header(id) => {
            seam_reset_button(host, id);
            // ⚠️ A PORTA ÚNICA — este bloco era `set_collapsed(id, !is_collapsed(id))` escrito à
            //    mão, que é literalmente o corpo do `toggle_collapsed`. Três cópias privadas da
            //    mesma pergunta, e a que sabe que uma dobra tem PARTIDA (o `t` de onde ela vem)
            //    é a porta — pela cópia, a estreia de cada secção destes painéis saltaria.
            host.store_mut().toggle_collapsed(id);
            true
        }
        WidgetEvent::Click(id) if id == ids::SCULPT3D_CLOSE => {
            seam_reset_button(host, id);
            host.set_panel_visible(crate::Sculpt3dPanel::ID, false);
            true
        }
        _ => false,
    };
    EventOutcome::from_bool(consumed)
}

/// As seis seções — as TRÊS com tabela de rows mais as três de botões.
///
/// ⚠️ A Topology mudou de lado quando ganhou a pista da resolução: ela sai da
/// lista à mão porque o laço de `SECTIONS` já a cobre, e mantê-la nas duas seria
/// registrar o mesmo cabeçalho duas vezes.
fn is_section_header(id: ph2d_a11y::NodeId) -> bool {
    rows::SECTIONS.iter().any(|s| s.id == id)
        || id == ids::SCULPT3D_SEC_TOOL
        || id == ids::SCULPT3D_SEC_SYMMETRY
        || id == ids::SCULPT3D_SEC_SCENE
}

/// **O que um clique na fileira de PADRÃO arma** — extraído do `apply_event`.
///
/// ⚠️ **Um corte por RESPONSABILIDADE, não por tamanho.** O braço cresceu quando
/// a fileira ganhou o slot de imagem e cruzou o teto de 200 LOC do `apply_event`,
/// e o que saiu foi a decisão inteira — *qual chip é este, e o que ele significa*
/// —, que é uma pergunta com resposta própria. O que fica lá é o DESPACHO: qual
/// widget foi tocado.
fn arm_alpha_chip(snapshot: &crate::state::Sculpt3dSnapshot, i: usize) {
    // ⚠️ **O ÚLTIMO chip é o slot de IMAGEM, e ele sai por OUTRA porta.** O
    // painel não tem a imagem: quando o artista escolhe um procedural o
    // `Arc<AlphaImage>` deixa o `Sculpt3dUi`, e só a CENA continua a segurá-lo.
    // Um `SetUi` aqui não teria o que armar, e o chip seria um controle que só
    // sabe deixar de estar aceso.
    if i > Alpha::ALL.len() {
        state::push_intent(Sculpt3dIntent::ArmStoredImage);
        return;
    }
    let mut ui = snapshot.ui.clone();
    ui.brush.alpha = i
        .checked_sub(1)
        .map(|k| Alpha::ALL[k.min(Alpha::ALL.len() - 1)].clone());
    // ⚠️ **Armar um padrão SEMEIA a escala do modelo** — e só enquanto o artista
    // não escolheu a dele. Uma escala é absoluta (um poro tem o tamanho de um
    // poro), mas *qual número* depende do tamanho e da densidade da peça, coisas
    // que só o modelo sabe: um literal acerta uma malha e erra todas as outras,
    // que foi o que o smoke reprovou.
    //
    // A mesma lei do `arm_inflate_defaults` do Painter: **arma um default, nunca
    // impõe política**. O sentinela é a constante de fábrica.
    if ui.brush.alpha.is_some()
        && (ui.brush.alpha_scale - ph2d_sculpt3d::DEFAULT_ALPHA_SCALE).abs() < 1e-6
        && snapshot.alpha_seed > 0.0
    {
        ui.brush.alpha_scale = snapshot.alpha_seed;
    }
    state::push_intent(Sculpt3dIntent::SetUi(ui));
}
