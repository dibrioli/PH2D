//! **As SETAS do Morph** — os `NodeId` da seção *Morph States* (plano 32 W4/W7/W8).
//!
//! ⛔⛔ **SEÇÃO PRÓPRIA, e a lição é de produto** (Enio, 2026-08-25: *"os states de morph deveriam
//! ter sessão exclusiva"*). A W4 pendurou estas linhas dentro da seção **States** — a das poses de
//! UI e do Smart Animate — com o argumento de que *"o Inspector mostra o que o objecto TEM"* e de
//! que *"um objecto raramente é as duas coisas"*. O argumento é verdadeiro e **não era a pergunta**:
//! partilhar a seção fez o cabeçalho de uma feature **já entregue** passar a aparecer por causa de
//! outra, e o dono leu isso como contaminação — que é exactamente o que era.
//!
//! ⚠️ *A lei do ADR-0166 diz o que MOSTRAR, nunca ONDE.* Duas features com donos diferentes,
//! histórias diferentes e gates diferentes partilhando um cabeçalho é uma porta a mais na seção de
//! quem chegou primeiro.

use super::{NodeId, fnv_node_id_runtime, hash_node_id};

/// **O cabeçalho da seção MORPH STATES** — a máquina de estados do Morph selecionado.
///
/// Vizinha da [`super::vector::VECTOR_SECTION_MORPH`] e **abaixo dela**, porque a ordem é o
/// assunto: a seção Morph diz o que o objecto **é**, esta diz **como ele decide** em que forma
/// está.
pub const VECTOR_SECTION_MORPH_STATES: NodeId = hash_node_id("vector.section.morph_states");

/// **O botão que faz o conjunto** — escolhe-se N formas no canvas e ele cria o objecto que as
/// governa, com **todas** as transições possíveis já ligadas (plano 32 W8).
pub const VECTOR_MORPH_STATES_MAKE: NodeId = hash_node_id("vector.morph.states.make");

/// ⭐⭐ **O INTERRUPTOR DA PRÉ-VISUALIZAÇÃO** — o modo em que o teclado é da máquina.
///
/// ⚠️ **Ele existe porque a condição de uma transição é uma TECLA** (Enio, 2026-08-25: *"precisamos
/// de um modo preview (com botão) como o de states de animação pois senão temos conflitos de
/// atalhos (como setas do teclado movendo as formas)"*). Sem um modo, carregar em `Z` morfa a forma
/// **e** faz o que o `Z` faz no editor — os dois, sem nada na tela a explicar.
///
/// ⛔ **É a ÚNICA porta, e o playhead deixou de ser uma delas.** A W5 ligava a máquina ao
/// transporte a andar; era exactamente aí que o conflito aparecia, porque o Play não tranca o
/// teclado do editor. *Duas portas para o mesmo modo divergem, e a que o artista encontra primeiro
/// é a que não tranca nada.*
pub const VECTOR_MORPH_PREVIEW: NodeId = hash_node_id("vector.morph.preview");

/// **QUANTAS FORMAS um conjunto de estados aceita** — e o recurso é o **relógio de pintura do
/// painel**, medido.
///
/// ⚠️ **Ele é também o tamanho do POOL de linhas**: desde a W10 a lista tem **uma entrada por
/// forma**, então *quantas formas* e *quantas linhas* passaram a ser o mesmo número. ⛔ A constante
/// `MAX_MORPH_ARROWS` que vivia aqui **morreu** — duas constantes para uma pergunta divergem.
///
/// # ⭐⭐ Este número era **9**, e a W10 dissolveu o que o segurava
///
/// O tecto anterior saiu de a lista ser o **grafo completo**: `n(n-1)` linhas, `0,0104 ms` cada.
/// A regra era *«esta seção sozinha nunca custa mais do que TODO o resto do painel junto»*, e ela
/// cruzava em `n = 9` (72 linhas, `0,752 ms` contra `0,746 ms`).
///
/// A W10 pôs a tecla no **destino** ⇒ a lista passou a ter `n` linhas. A MESMA regra, com a MESMA
/// sonda, mede agora (2026-08-25, `MockPanelHost` a pintar o painel Vector inteiro, release; o
/// painel **sem** esta seção custa `0,726 ms`):
///
/// | formas = linhas | painel | delta da seção | % de um quadro de 16,7 ms |
/// |---:|---:|---:|---:|
/// | 9 | `0,824 ms` | `0,085 ms` | 4,93 % |
/// | 32 | `0,953 ms` | `0,215 ms` | 5,71 % |
/// | 64 | `1,128 ms` | `0,389 ms` | 6,75 % |
/// | 114 | `1,430 ms` | `0,704 ms` | 8,56 % |
/// | **118** | **`1,457 ms`** | **`0,731 ms`** | **8,73 %** |
/// | 122 | `1,492 ms` | `0,765 ms` | 8,93 % |
///
/// ⇒ **118 é o último `n` que a regra aceita** — e o antigo `9` custa hoje `0,085 ms`, **um nono**
/// do que custava. *Quem move o número que tornava algo inalcançável tem de reconferir a nota*
/// (`CLAUDE.md` §0.0), e o que se move aqui não é o tecto: é o **expoente**.
///
/// ⛔ **O pool de ids NÃO é o recurso** — medido: `118 × 25 = 2 950` widgets custam `~0,29 ms`
/// **uma vez**, no `populate`, e nunca por quadro.
///
/// ⛔ **A régua não fica como gate.** Ela divide dois relógios, que é exactamente a família de
/// flakes sob fan-out do `CLAUDE.md` §5.0 — as tabelas acima são o registo, e re-medir é rodar a
/// sonda.
pub const MAX_MORPH_STATES: usize = 118;

/// **Quantas acções o menu da condição oferece.** É o pool de ids do popover.
///
/// ⚠️ Um mapa com mais acções continua a funcionar; o menu mostra as primeiras. O número acompanha
/// o que um projecto real tem (o mapa de fábrica traz **seis**).
pub const MAX_MORPH_ACTIONS: usize = 24;

/// ⭐⭐ **DESFAZER TUDO** — dissolve o conjunto: o objecto some, as formas voltam **soltas e
/// visíveis**, onde estavam.
///
/// ⚠️ **Ele é o inverso EXACTO do [`VECTOR_MORPH_STATES_MAKE`]**, e desde a W11 não precisa de
/// código próprio de desmontagem: a lista são os filhos, então dissolver é reparentar todos para
/// fora. *Um botão que desfaz tem de chegar ao mesmo mundo de onde se partiu, e não a um parecido.*
pub const VECTOR_MORPH_DISSOLVE: NodeId = hash_node_id("vector.morph.dissolve");

/// **PLAY na forma `row`** — vai até ela, animado, para o artista a VER.
///
/// ⚠️ **Era o `Show` das poses de UI, e o Enio renomeou-o** (2026-08-26): ali um estado é uma pose
/// que se **aplica**; aqui é uma forma para onde se **viaja**, e a viagem é o produto.
///
/// ⚠️ **Ele LIGA a pré-visualização se estiver desligada** — a máquina só anda dentro do modo (é
/// ele que tem o relógio), e um Play que não tocasse nada seria um botão morto com nome de verbo.
#[must_use]
pub fn morph_shape_play_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.morph.shape.play.{row}"))
}

/// **DESCONECTAR a forma `row`** — ela sai do conjunto e volta a ser uma forma solta e visível.
///
/// ⚠️ **Era o `Clear`, e o Enio renomeou-o** (2026-08-26) — com razão: `Clear` sugere *apagar*, e
/// aqui não se apaga nada. A forma continua no documento, com o desenho dela; ela só deixa de ser
/// um estado. ⭐ **E a tecla dela FICA guardada**: voltar a arrastá-la para dentro devolve-a.
///
/// ⚠️ Desde a W11 isto é **exactamente** o gesto de arrastar para fora na Hierarquia — o botão é o
/// atalho, nunca uma segunda lei.
#[must_use]
pub fn morph_shape_disconnect_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.morph.shape.disconnect.{row}"))
}

/// **O botão que ABRE o modal dos eventos** para a forma `row`.
///
/// ⚠️ **Era um dropdown, e o Enio trocou-o** (2026-08-26: *"no lugar do dropdown melhor um botão
/// que abre um modal com os eventos"*). O menu vivia dentro do scroll da seção e precisava de um
/// passe diferido só para não ser cortado na borda; um modal não tem esse problema, e mostra a
/// lista inteira com espaço para o nome de cada acção.
#[must_use]
pub fn morph_shape_key_button_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.morph.shape.keybtn.{row}"))
}

/// A opção `action` no menu da condição da seta `row`.
///
/// ⚠️ **O índice `0` é o «—»** (sem condição): ele existe porque *tirar* a condição tem de ser um
/// gesto — e, desde a W8, é **a única maneira de desligar uma transição**. O grafo é completo por
/// construção, então uma seta sem acção é uma passagem que existe e **nunca acontece**.
#[must_use]
pub fn morph_shape_key_option_id(row: usize, action: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.morph.arrow.when.{row}.{action}"))
}

/// O cabeçalho da sub-lista das setas, dentro da seção.
pub const VECTOR_MORPH_SHAPES_LABEL: NodeId = hash_node_id("vector.morph.arrows.label");
