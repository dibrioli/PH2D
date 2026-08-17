//! Painel → shell. Todo braço de knob é derivado de [`crate::rows`], então uma
//! row que existe é uma row que despacha.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal, seam_reset_button};
use ph2d_sculpt3d::{Alpha, Falloff, RefMode, TransformKind, Verb};

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
        // **Carimba a referência corrente em TODAS as ferramentas** — um GESTO
        // sobre o estado por-verbo, nunca um segundo seletor global.
        //
        // ⚠️ **Ele só alcança quem DECLARA o modo, e a metade que faltava era a
        // única porta que podia pôr um modo onde ele não tem lei.** Enquanto os
        // três modos respondiam por todo verbo isto era um `fill` e ninguém
        // notava; com o `L` declarando só o Smooth (W4), carimbá-lo em todos
        // deixaria quinze verbos rodando uma `KernelLaw` de literatura que não
        // fala deles, **com o chip a mostrar `S`** — porque o painel pinta os
        // OFERECIDOS e o `L` não estaria entre eles. O chip que mente, pela
        // porta de trás.
        //
        // ⚠️ E onde ele não alcança, ele **PRESERVA** em vez de repor um
        // default: o artista carimbou uma escolha, não pediu um reset das que
        // não cabem.
        WidgetEvent::Click(id) if id == ids::SCULPT3D_REF_MODE_ALL => {
            seam_reset_button(host, id);
            let mut ui = snapshot.ui;
            let stamp = ui.brush.mode;
            for verb in Verb::ALL {
                if stamp.declares(verb) {
                    ui.set_mode_of(verb, stamp);
                }
            }
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
        // ⚠️ **Gateado na LEI, como a caixa que o pinta.** Sem o guard um
        // clique sintético (ou um id que sobreviveu a uma troca de modo no
        // mesmo frame) armaria um flag que nenhum dab consulta — o braço
        // `Ignored` do kernel nem o lê —, e o painel o mostraria marcado no
        // próximo modo que oferece a lei.
        WidgetEvent::Click(id)
            if id == ids::SCULPT3D_FRONT_FACES && snapshot.ui.brush.offers_front_faces() =>
        {
            seam_reset_button(host, id);
            let mut ui = snapshot.ui;
            ui.brush.front_faces_only = !ui.brush.front_faces_only;
            state::push_intent(Sculpt3dIntent::SetUi(ui));
            true
        }
        // ⚠️ **Gateado no VERBO, como a row que o pinta** — a mesma razão do
        // vizinho acima: um clique sintético (ou um id que sobreviveu a uma
        // troca de verbo no mesmo frame) armaria um modo que nenhum dab lê, e o
        // painel voltaria a mostrá-lo marcado na próxima lâmina.
        WidgetEvent::Click(id)
            if id == ids::SCULPT3D_SCRAPE_DYNAMIC
                && snapshot.ui.brush.verb == Verb::MultiplaneScrape =>
        {
            seam_reset_button(host, id);
            let mut ui = snapshot.ui;
            ui.brush.scrape_dynamic = !ui.brush.scrape_dynamic;
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
        // ⚠️ **SEIS grupos, UMA porta** — o irmão do `table_intent` logo abaixo,
        // e o corte é a mesma frase: os seis respondem *achou o índice `i`, que
        // ESTADO AUTORADO isso significa?*, e escritos como seis braços eles
        // divergiam num `seam_reset_button` a menos sem ninguém ver. O que fica
        // aqui é o DESPACHO; o que saiu é a DECISÃO.
        WidgetEvent::Click(id) if group_chip_ui(&snapshot, id).is_some() => {
            seam_reset_button(host, id);
            state::push_intent(Sculpt3dIntent::SetUi(
                group_chip_ui(&snapshot, id).expect("guard casou"),
            ));
            true
        }
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

/// **O que um chip de GRUPO significa como estado autorado** — extraído do
/// `apply_event`, no molde do [`arm_alpha_chip`].
///
/// ⚠️ **Ele COMPÕE em vez de empurrar uma constante**, e é por isso que não cabe
/// no [`table_intent`]: aquele devolve uma entrada de tabela, este lê o retrato
/// vivo e devolve o estado inteiro com um campo trocado. Juntá-los obrigaria a
/// tabela a receber o snapshot, e aí ela deixaria de ser uma tabela.
fn group_chip_ui(
    snapshot: &crate::state::Sculpt3dSnapshot,
    id: ph2d_a11y::NodeId,
) -> Option<crate::state::Sculpt3dUi> {
    let mut ui = snapshot.ui.clone();
    if let Some(i) = index_of(&ids::SCULPT3D_VERB, id) {
        // ⚠️ **TROCAR DE FERRAMENTA é guardar e carregar, e nada mais** — o
        // pincel vivo vai para o slot do verbo que sai e o do verbo que entra
        // toma o lugar dele. Nenhum knob é re-armado, porque a força que o
        // artista afinou no Smooth **não é** a força do Clay.
        crate::state::switch_verb(&mut ui, Verb::ALL[i]);
    } else if let Some(i) = index_of(&ids::SCULPT3D_REF_MODE, id) {
        // ⚠️ **A REFERÊNCIA não é a ferramenta:** ela muda a LEI do kernel
        // dentro do verbo que já está em mãos, então quem re-resolve é a porta
        // do MODO — e ela só toca o que depende dele (a curva), preservando
        // toda escolha deliberada.
        crate::state::arm_mode_defaults(&mut ui, RefMode::ALL[i]);
    } else if let Some(i) = index_of(&ids::SCULPT3D_ELASTIC_SCALES, id) {
        // ⚠️ **Sem re-armar nada, e a razão é a do vizinho de baixo:** a largura
        // do campo é uma escolha DO ARTISTA sobre o modo que ele já escolheu,
        // não a escolha de uma ferramenta. Re-resolver aqui devolveria a família
        // que a medição elegeu, apagando o gesto no instante em que ele acontece.
        ui.brush.elastic_scales = ph2d_sculpt3d::kelvinlet::Scales::ALL[i];
    } else if let Some(i) = index_of(&ids::SCULPT3D_UI_LEVEL, id) {
        // ⚠️ **Sem trocar de slot:** mudar o nível não é escolher uma
        // ferramenta, é escolher quanto dela ver — passar pela porta de troca
        // aqui recarregaria o pincel no gesto que o artista fez só para OLHAR.
        ui.ui_level = state::UiLevel::ALL[i];
    } else if let Some(i) = index_of(&ids::SCULPT3D_FALLOFF, id) {
        ui.brush.falloff = Falloff::ALL[i];
    } else if let Some(i) = index_of(&ids::SCULPT3D_MATCAP, id) {
        // A opção `0` é o rig do artista e as seguintes são os matcaps, o mesmo
        // deslocamento que o pintor usa. `checked_sub` e não `- 1`: a opção zero
        // não é o material `-1`, é a AUSÊNCIA de matcap.
        ui.matcap = i.checked_sub(1).map(|k| u8::try_from(k).unwrap_or(u8::MAX));
    } else if let Some(i) = index_of(&ids::SCULPT3D_DETAIL, id) {
        ui.detail = u8::try_from(i).unwrap_or(0);
    } else {
        return None;
    }
    Some(ui)
}
