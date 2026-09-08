//! O registro dos widgets da seção **SKELETON** — irmão do [`super`] pelo teto de 600 LOC do
//! painel, e par natural do `paint_bone` (que os PINTA).
//!
//! Registar é o que os torna clicáveis: pintar + hit-rect não basta — é a classe de bug que já
//! matou botões do vetor duas vezes (o mais recente é o [bug #29], em que TRÊS rotas morreram ao
//! mesmo tempo com o gate de registo **verde**). O gate `architecture_panel_wiring_parity` cobra a
//! correspondência entre os dois ficheiros.
//!
//! Os cinco são registados **incondicionalmente**, mesmo os que só são PINTADOS com uma forma presa
//! ou um osso em foco: o store é agnóstico de estado, e quem decide se o clique é possível é a
//! PINTURA (sem hit-rect não há `Click`).
//!
//! [bug #29]: ../../../docs/Vector%20Module/BUGS_vector.md

use super::{button, world_number_field};
use crate::ids;
use ph2d_editor_core::interaction::WidgetStore;

/// Os widgets do esqueleto: prender, as duas saídas, e os dois números do osso.
pub(super) fn populate_bone(store: &mut WidgetStore) {
    // ⭐⭐⭐ **Os dois segmentos de CRIAR × TRANSFORMAR** (Enio, 2026-09-07). ⚠️ Um segmento
    // registado por LOOP a partir da lista de ids é o mesmo idioma que o gate irmão
    // `table_driven_chips_are_registered_too` existe para apanhar — aqui a lista é a fonte, então
    // acrescentar um terceiro estado ao vocabulário regista-o sozinho.
    for id in ids::VECTOR_BONE_ACTION_IDS {
        button(store, id);
    }
    // ⭐⭐⭐ **OS VERBOS, pela TABELA** ([`ids::VECTOR_BONE_VERBS`]) — a mesma que decide o que
    // atravessa para a shell. Registá-los à mão aqui e encaminhá-los à mão ali são duas respostas à
    // mesma pergunta, e foi assim que o *Add IK* nasceu pintado, aceso e MUDO.
    //
    // ⚠️ **Todos são registados, mesmo sendo pintados um de cada vez** (o *Add IK* e o *Remove IK*
    // excluem-se): o registo diz *«este id existe»* ao índice de acerto e ao AccessKit, e a pintura
    // é que decide qual deles o artista vê.
    for id in ids::VECTOR_BONE_VERBS {
        button(store, id);
    }
    // ⭐⭐⭐ **A fileira do LADO DA DOBRA, pela tabela dela** — e o laço é obrigatório mesmo que os
    // ids coubessem noutra lista: o gate `table_driven_chips_are_registered_too` lê o FONTE e exige
    // que o `populate` itere a mesma tabela que o `paint`. É a regra que impede a próxima fileira
    // de nascer morta sob o dedo.
    for id in ids::VECTOR_BONE_BEND_IDS {
        button(store, id);
    }
    // ⭐⭐⭐ **O SELECTOR DE ACÇÃO do osso inteligente** — o chip que responde *«qual animação?»*.
    //
    // ⚠️ **`Dropdown` no store, botão na tela**: é o `InteractiveState::Dropdown` que faz o dispatch
    // genérico alternar o `open` (e fechar o dos outros). Registá-lo como `Button` faria o clique
    // acender e **nunca abrir lista nenhuma** — a cicatriz da swatch dos tokens, a dos dois números
    // do Input Map, e a da tecla de forma do Morph.
    store.register(
        ids::VECTOR_BONE_SMART_CLIP,
        ph2d_editor_core::interaction::InteractiveState::Dropdown {
            state: ph2d_editor_core::widget::DropdownState::Normal,
            open: false,
            selected_index: None,
        },
    );
    // As opções, pela MESMA tabela que o `paint` percorre e que o encaminhamento deixa passar.
    for id in ids::VECTOR_BONE_SMART_CLIP_IDS {
        button(store, id);
    }
    // ⚠️ **Pela porta do MUNDO** (`world_number_field`, sem `set_number_range`): o comprimento de um
    // osso vive nas unidades do documento, e emprestar-lhe a faixa de outro recurso é exactamente
    // o defeito que o `CLAUDE.md` §0.0 nomeia — a v21 já o pagou com a largura de traço a limitar
    // um deslocamento.
    // ⚠️ **Pela porta do MUNDO** (`world_number_field`, sem `set_number_range`) e pela TABELA que o
    // encaminhamento também lê: o comprimento de um osso vive nas unidades do documento, e
    // emprestar-lhe a faixa de outro recurso é exactamente o defeito que o `CLAUDE.md` §0.0 nomeia
    // — a v21 já o pagou com a largura de traço a limitar um deslocamento. `Mix` e `Softness` são
    // adimensionais e `Chain` conta ossos: nenhum é medida de desenho.
    for id in ids::VECTOR_BONE_FIELDS {
        world_number_field(store, id, 0.0);
    }
    // ⭐⭐⭐ A ÂNCORA DE IK — os dois verbos e os três números dela.
    //
    // ⚠️ **Os dois botões são registados SEMPRE**, mesmo sendo pintados um de cada vez: o registo é
    // o que diz *«este id existe»* ao índice de acerto e ao AccessKit, e a pintura é que decide
    // qual dos dois o artista vê. Registar só o pintado faria o outro nascer **morto sob o dedo** —
    // o defeito dos quatro chips da booleana, que só o gesto real apanhou.
}
