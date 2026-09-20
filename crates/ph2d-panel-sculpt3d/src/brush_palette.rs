//! ⭐⭐⭐ **O SELECTOR DE PINCÉIS, fora do painel** — a metade 2 da **D2** para a escultura.
//!
//! # ⛔⛔ O defeito que isto cura, MEDIDO
//!
//! O painel da escultura mede **`2 373 px`** num encaixe de **`880`**, e o artista vê **2 das 7
//! secções**. Da parte visível, a secção `Tool` come **`614 px` — `70 %`** —, e os botões que ele
//! de facto gira enquanto esculpe (raio, força, queda) começam em `y = 722`, ou seja **onde a tela
//! acaba**. O censo está em
//! `ph2d-panel-registry-init/tests/it/quantas_entradas_tem_cada_painel.rs`, e o controlo dele é que
//! o `3D Model` — o **único** painel que já recebeu esta triagem — é o **único painel de ferramenta
//! que cabe**.
//!
//! # ⭐ A decisão do dono, e a nota que ela reconfere
//!
//! Enio, 2026-09-20: *«sim»* — a secção `Tool` inteira sai do painel para a barra.
//!
//! ⚠️⚠️ **Isto overturna uma decisão ESCRITA do módulo**, e a razão é o `CLAUDE.md` §0.0. O doc do
//! `paint_tool` argumenta pela faixa que reflui e **contra** um dropdown (*«esconde todas menos uma
//! atrás de um clique»*) — e a frase seguinte dele diz que a contagem cresceu sem ninguém
//! reconferir: ela foi escrita para **~10** verbos e hoje são **38**. *Quem move o número que
//! tornava algo inalcançável tem de reconferir a nota* — e a `276 px` de fichas dentro de um painel
//! que estoura `1 493`, a premissa dissolveu-se.
//!
//! # ⛔⛔ Porque uma PALETA e não um pulldown, com as duas recusas pelo caminho
//!
//! 1. **Um pulldown de área** é o que o `3D Model` usa, e o doc do [`AreaMenu`] recusa juntar
//!    perguntas diferentes numa face só (*«vista + gizmo dá 14 linhas com uma face só — o depósito
//!    mudado de sítio»*). ⭐ Os 38 verbos são **UMA** pergunta, logo essa recusa não os alcança —
//!    o que os alcança é a **altura**: `38 × ~22 px ≈ 836`, e o alvo declarado desta casa é um
//!    tablet de `1 024`. *Poupar altura no painel gastando-a num menu não poupa nada.*
//! 2. **A paleta é o contentor que esta casa tem para um catálogo**, e o precedente é do próprio
//!    dono: as `68` formas do 3D são **um** chip que a abre (W100). Ela é um modal centrado, com
//!    rolagem própria e **busca**.
//!
//! # ⚠️ UM grupo, e a razão é que a alternativa seria AUTORIA
//!
//! A paleta das formas agrupa por `Family`, que é um campo do catálogo delas. ⛔ O [`Verb`] **não
//! tem categoria de UI** — os predicados dele (`grip`, as famílias de leitura) descrevem o
//! MECANISMO, e traduzi-los em títulos de grupo seria escrever vocabulário de artista para um
//! módulo que não é o meu. ⇒ **um grupo, e a busca faz o trabalho** — que é exactamente para o que
//! o campo de busca da paleta existe.
//!
//! ⭐ E o título do grupo é a chave da **própria secção que se mudou**
//! (`panel.sculpt3d.section.tool`): a paleta é a secção `Tool` noutro sítio, logo herda o nome
//! dela. *Zero rótulo autorado por mim.*
//!
//! # ⚠️⚠️ OS IDS SÃO OS MESMOS, e é isso que faz o clique continuar a chegar
//!
//! Cada item carrega o `SCULPT3D_VERB[i]` que o painel já cunhava. É a lei que o `area_bar` do
//! `3D Model` escreve por extenso: *«um comando com dois ids tem dois sítios a apodrecer em
//! separado»*. ⛔ Um id novo aqui obrigaria a uma segunda rota de despacho.

use crate::ids::SCULPT3D_VERB;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::widget::command_palette::{
    PaletteGroup, PaletteItem, PaletteModel, PaletteSub,
};
use ph2d_sculpt3d::Verb;

/// ⭐ **O modelo da paleta** — sobre o catálogo do produto ([`Verb::ALL`]).
#[must_use]
pub fn build() -> PaletteModel {
    build_from(&Verb::ALL)
}

/// ⭐⭐ **O mesmo modelo, sobre um catálogo DADO** — e o argumento existe por causa de um gate.
///
/// ⚠️ É o molde do `shape_palette::build_from`, e pela mesma razão: *um gate cujo sujeito é o
/// estado de hoje perde-o no dia em que o produto muda*. Com o catálogo por argumento, o sujeito
/// **constrói-se**.
///
/// ⛔⛔ **Um verbo sem id é SALTADO, e isso não é tolerância — é o único comportamento honesto.**
/// O `Verb::ALL` e o [`SCULPT3D_VERB`] são emparelhados por ÍNDICE (é a convenção que o `seg` do
/// painel já usa), logo um verbo acrescentado sem o id correspondente não tem por onde despachar.
/// ⚠️ Quem o apanha é o [`o_catalogo_e_a_lista_de_ids_tem_o_mesmo_tamanho`], **não** este `filter`:
/// saltá-lo aqui em silêncio seria um pincel que existe e que o artista não alcança.
#[must_use]
pub fn build_from(verbos: &[Verb]) -> PaletteModel {
    let items: Vec<PaletteItem> = verbos
        .iter()
        .enumerate()
        .filter_map(|(i, v)| {
            SCULPT3D_VERB.get(i).map(|&id| PaletteItem {
                label: ph2d_i18n::tr(v.label_key()).to_string(),
                id,
            })
        })
        .collect();

    PaletteModel {
        title: ph2d_i18n::tr("panel.sculpt3d.section.tool").to_string(),
        groups: vec![PaletteGroup {
            title: ph2d_i18n::tr("panel.sculpt3d.section.tool").to_string(),
            // ⚠️ **Um token que EXISTE, nunca um hex novo** (HR-15 / §7): escolher cor é decisão de
            // design, e com um grupo só ela não distingue nada de nada — o que ela faz é não ser
            // uma excepção na paleta.
            color: ph2d_tokens::ColorToken::NodeCatSource,
            subs: vec![PaletteSub { title: None, items }],
        }],
        // ⚠️ **Sem caixa da banda**: ela existe na paleta de componentes para revelar o
        // inaplicável, e aqui todo pincel é aplicável. Não há segunda metade para mostrar.
        toggle: None,
    }
}

/// ⭐ **O VERBO que este id nomeia**, se algum — o inverso do emparelhamento por índice.
///
/// ⚠️ **Varre o catálogo, nunca uma segunda lista**: uma lista à mão aqui envelheceria no primeiro
/// pincel novo, e o sintoma seria *«o item aparece na paleta e não faz nada»* — a espécie de
/// controlo morto que o `CLAUDE.md` §5.0 nomeia.
#[must_use]
pub fn verb_at(id: NodeId) -> Option<Verb> {
    SCULPT3D_VERB
        .iter()
        .position(|&x| x == id)
        .and_then(|i| Verb::ALL.get(i).copied())
}

#[cfg(test)]
#[path = "brush_palette_tests.rs"]
mod tests;
