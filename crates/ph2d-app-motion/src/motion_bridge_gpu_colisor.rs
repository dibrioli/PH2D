//! ⭐⭐⭐ **AS CERCAS DO COLISOR** — as três perguntas que decidem se um documento com contacto
//! entre peças pode cozer no DISPOSITIVO (docs 109 W2 e 115 W1/W4).
//!
//! # Porque elas são um ASSUNTO e vivem num ficheiro só
//!
//! As três respondem à MESMA razão de motor — *o kernel de WGSL separa DISCOS e a CPU honra a
//! CAIXA declarada*, logo a mesma cena daria uma pilha num lado e um borrão no outro, sem erro
//! nenhum — por **rotas diferentes**: o nome escrito num text param, a coluna que atravessa pela
//! tabela de externos, e o nó que a LÊ. Ler uma sem as outras duas dá sempre a conclusão errada
//! sobre o que o produto recusa.
//!
//! ⚠️ **O DESPACHO fica no [`super`]**, de propósito: o que este ficheiro tem são as PERGUNTAS, e
//! quem decide a ordem do cozimento é o `cook_gpu`. O gate
//! `a_cerca_dos_externos_esta_de_facto_ligada_ao_cozimento` lê aquele ficheiro por `include_str!`
//! justamente porque *um gate que chama a função em vez de percorrer a rota afirma que a lei
//! existe, nunca que o produto a usa*.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::Graph;
use ph2d_nodegraph::node::NodeTypeId;

/// ⭐⭐ **Um documento que escreve o colisor PELO NOME cozinha na CPU** (doc 109 W2).
///
/// O contacto entre peças mora no `sim.step` de CPU e o dispositivo ainda **não** o resolve (W5, sem
/// cliente). Deixar a coluna `collider` chegar a um `sim.step` no dispositivo daria a MESMA cena com
/// uma pilha na CPU e um borrão na placa — sem erro nenhum.
///
/// ⚠️ **Quem a escreve:** o `source.shape` com `Collide` — já recusado pela porta da forma viva, logo
/// acima — e **qualquer nó que escreva uma coluna pelo NOME** (o canal `Custom…` do `motion.drive`
/// escreve a que o artista digitar). Esse recua para a CPU sozinho, mas numa rota HÍBRIDA a coluna
/// que ele escreveu antes da fronteira atravessa-a e chega ao dispositivo.
///
/// ⚠️ **A pergunta é sobre o NOME e não sobre o nó**, de propósito: uma lista de «nós que escrevem por
/// nome» envelheceria no dia do próximo. O preço de um falso positivo (um texto que diga
/// exactamente `collider` noutro sentido) é cozinhar na CPU, nunca uma cena errada.
/// A frase da recusa — uma constante, para o gate que prova a LIGAÇÃO ler a frase do produto e não
/// uma cópia dela.
pub(super) const RECUSA_COLISOR: &str =
    "CPU: uma peca declara colisor pelo nome -- o dispositivo ainda nao resolve contatos (doc 109)";

/// A frase da recusa da porta de EXTERNOS — irmã da acima, e separada de propósito: as duas
/// recusam pela mesma razão de motor e por **rotas diferentes**, e um smoke que leia *«pelo
/// nome»* sobre um objecto da cena procuraria o defeito no sítio errado.
pub(super) const RECUSA_COLISOR_EXTERNO: &str =
    "CPU: um objecto da cena traz colisor -- o dispositivo ainda nao resolve contatos (doc 115)";

/// ⭐⭐⭐ **A METADE QUE A DECLARAÇÃO PELO NOME NÃO ALCANÇA: um EXTERNO que traz colisor**
/// (doc 115 W1).
///
/// # O buraco, e porque ele é LATENTE e não teórico
///
/// A irmã acima varre `graph.node_text_params()` — ela vê o artista a **escrever** o nome de uma
/// coluna. ⛔ Um objecto da cena publicado pela membrana não escreve texto nenhum: ele entra pela
/// tabela de externos do cozedor, que aquela varredura **não olha**. ⇒ no dia em que o Sprite, o
/// vector e o Flip nascerem com o colisor deles (doc 115 W3/W4), a coluna atravessa a fronteira
/// **sem cerca nenhuma** — e o modo de falha é o que o doc da irmã já nomeia: *a MESMA cena com
/// uma pilha na CPU e um borrão na placa, sem erro nenhum*.
///
/// ⚠️⚠️ **Ela nasce INERTE, e isso é a ordem certa e não um descuido.** Medido em 2026-09-17: a
/// membrana publica `(P, size, tint, uv_rect, texture_id)` e mais nada, logo hoje nenhum externo
/// traz estas colunas e esta porta responde `false` em toda cena do produto. *Escrever a cerca
/// ANTES de abrir a rota é o que impede que cada wave a seguir torne mais cenas silenciosamente
/// erradas* — a mesma razão pela qual a W1 das lanes vem primeiro no doc 102.
///
/// ⚠️ **O molde é a [`cook_publishes_live_geometry`]**, não a irmã de texto: a membrana publica os
/// externos ANTES de o cozimento correr (pós-dreno, pré-cook), logo uma varredura por quadro
/// responde à pergunta REAL. Custo: um punhado de externos, três sondas de coluna cada.
pub(super) fn cook_publishes_collider(cook: &ph2d_nodegraph::cook::Cook) -> bool {
    use ph2d_nodegraph::attr::{COLLIDER_BOX_COLUMN, COLLIDER_COLUMN, COLLIDER_OFFSET_COLUMN};
    cook.externals().values().any(|e| {
        [COLLIDER_COLUMN, COLLIDER_BOX_COLUMN, COLLIDER_OFFSET_COLUMN]
            .iter()
            .any(|c| e.value.get(c).is_some())
    })
}

/// ⭐⭐⭐ **A METADE DO CONSUMIDOR** (doc 115 W4) — *a declaração chega a alguém que a LEIA?*
///
/// # Porque ela é obrigatória, e não um refinamento
///
/// Desde a W4 **todo** objecto da cena declara a caixa dele, e a membrana publica TODO sprite com
/// nome — inclusive os que o grafo nunca nomeia. ⇒ a metade de cima sozinha responderia `true` em
/// qualquer cena com um objecto, e o preço não é o `0,02 %` de um quadro que a §11 do doc 115
/// mediu à população do dono: é o `50,9×` do doc 98 numa cena que carimbe um objecto aos milhares,
/// que é exactamente o caso que a §9.5 daquele doc deixou **nomeado a vigiar**.
///
/// ⚠️ **O CONTROLO desta lei já estava escrito, por mim, no gate da W1**: *«um objecto SEM colisor
/// tem de continuar a cozer no dispositivo — senão esta cerca derruba toda cena com um Sprite e o
/// §0.0 deixa o caminho lento definir o produto»*. A W4 tornou a premissa dele falsa (hoje todo
/// objecto traz colisor) e a frase continua verdadeira: o que mudou de sítio foi **qual** das duas
/// metades a garante.
///
/// # A bandeira, e não uma lista de nomes na shell
///
/// A pergunta é *«este tipo de nó lê o colisor DECLARADO?»*, e a resposta é side-metadata do
/// registo ([`NodeRegistry::reads_declared_collider`]), registada por cada nó que chama
/// `ph2d_contact::colisores`. ⛔ Uma lista escrita aqui envelheceria no dia do quarto leitor, em
/// silêncio e do lado errado — o lado que deixa a divergência passar. O censo
/// `todo_leitor_do_colisor_declarado_se_regista` é quem impede a bandeira de ficar por pôr.
pub(super) fn graph_reads_declared_collider(graph: &Graph, reg: &NodeRegistry) -> bool {
    graph
        .nodes()
        .iter()
        .any(|n| reg.reads_declared_collider(NodeTypeId::of(n.type_name.as_str())))
}

pub(super) fn graph_declares_collider(graph: &Graph) -> bool {
    use ph2d_nodegraph::attr::{COLLIDER_BOX_COLUMN, COLLIDER_COLUMN, COLLIDER_OFFSET_COLUMN};
    // As TRÊS colunas da declaração (doc 109 §5): a caixa e o centro também só a CPU resolve.
    graph.node_text_params().values().any(|params| {
        params.values().any(|v| {
            matches!(
                v.trim(),
                COLLIDER_COLUMN | COLLIDER_BOX_COLUMN | COLLIDER_OFFSET_COLUMN
            )
        })
    })
}

#[cfg(test)]
#[path = "motion_bridge_gpu_collider_tests.rs"]
mod collider_tests;
