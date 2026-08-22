//! **A BOOLEANA VIVA** — um grupo cujos filhos se combinam, e continuam editáveis.
//!
//! O 7º produtor de [`LiveGeometry`], e o **segundo que TRANSFORMA o mapa** em vez de o estender
//! (o primeiro é o [`crate::align_live`]). A razão é a mesma dele e vale a pena repeti-la: os
//! cinco produtores que o `render_loop` funde com `extend` são mutuamente exclusivos — uma forma
//! é offset OU pattern OU contour, cada um é um componente próprio e o painel oferece um de cada
//! vez. **A booleana não é membro dessa família**: ela é um componente do PAI, então os operandos
//! dela podem ter, cada um, o seu offset vivo. Fundida por `extend` ela apagaria esses offsets —
//! ou seria apagada por eles — em silêncio.
//!
//! # O que entra, e o que sai
//!
//! A entrada de cada operando é **o que o mapa já diz** (o offset/pattern/contour dele, se
//! houver), ou a fonte assada em MUNDO. É a lei do `align_live`, e é ela que faz a booleana
//! operar sobre *o que se DESENHA* em vez de sobre *o que está guardado*.
//!
//! A saída vai no id da **BASE** — o operando mais ao FUNDO —, e cada um dos demais recebe uma
//! lista **VAZIA**, que o `dispatch` desenha como nada (`ph2d-vec-render/src/lib.rs:174`: ele
//! itera os items do mapa, e zero items desenham zero). Não é arranjo: é exatamente a regra que a
//! booleana DESTRUTIVA já segue (*"a base é a de trás: o resultado ocupa a fatia de z dela"*), e
//! é o que faz o **Apply** não mover a arte um pixel.
//!
//! ⚠️ **O grupo NÃO pode ser a chave.** Um grupo é *"uma entidade sem geometria própria que tem
//! filhos"*, então ele não tem `VecPathId` — e a `LiveGeometry` é chaveada por `VecPathId`. Foi
//! por isso que a base virou a portadora do resultado.
//!
//! # O GRAFO: a operação passa a ser da LIGAÇÃO
//!
//! Um grupo que carrega [`VecBoolEdges`] deixa de ter uma operação e passa a ter um **diagrama**:
//! ligações dirigidas, cada uma com a sua operação, o que faz a mesma forma somar com uma vizinha e
//! subtrair de outra. A lei mora em `ph2d_vec_boolean::graph`; aqui mora a costura ([`cook`]).
//!
//! ⚠️ A frase *"a saída vai no id da BASE"* acima passa a ser o caso **particular** em que o grupo
//! não tem grafo. Com grafo, quem carrega o resultado é **cada sumidouro** — quem não opera sobre
//! ninguém —, e um grupo pode ter mais de um. É a mesma regra dita de forma mais geral, e a
//! `derive_star` prova a equivalência (dois gates a prendem).
//!
//! # `Err` e `Ok(vazio)` são coisas DIFERENTES
//!
//! `Ok(vazio)` é uma RESPOSTA: a interseção de duas formas disjuntas é o vazio, e desenhar nada é
//! o certo (a mesma lei que o `offset_live` documenta — *"vazio é a ANIQUILAÇÃO"*). `Err` é o
//! motor a RECUSAR (geometria degenerada no meio de um arrasto), e aí o mapa fica **intocado** —
//! a arte continua como estava em vez de piscar.
//!
//! # Aninhamento: o mais FUNDO primeiro
//!
//! Um grupo booleano dentro de outro compõe naturalmente **se** o de dentro cozinhar antes: o
//! resultado dele pousa no id da base dele, e o de fora encontra esse id no mapa e o consome como
//! operando. Por isso os grupos são processados por profundidade decrescente — a ordem é a única
//! coisa que separa "compõe" de "o de fora lê a fonte crua do de dentro".

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, SimWorld, VecBoolEdge, VecBoolEdges, VecBoolGroup};
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecXforms, bake_xform, xform_of};

use crate::vec_entities::VecEntityMap;

/// Uma entrada do memo: o que ENTROU (a operação, o grafo e a lista de mundo) e o que SAIU.
///
/// ⚠️ O grafo faz parte da CHAVE. Sem ele, ligar duas formas com outra operação deixaria a resposta
/// velha na tela até que alguém MEXESSE numa delas — um memo que não vê metade da sua entrada é
/// cache envenenada, não cache.
struct Memo {
    op: u8,
    edges: Option<Vec<VecBoolEdge>>,
    input: Vec<VecPath>,
    /// O que CADA operando desenha: o resultado, para quem o carrega; vazio, para quem foi
    /// consumido. Uma lista só, porque um grafo pode ter mais de um sumidouro.
    out: Vec<(VecPathId, Vec<VecPath>)>,
}

/// **O que um grupo booleano desenhou neste frame** — a resposta que o produtor computou e que o
/// **Apply** consolida.
///
/// ⚠️ Ela existe para que haja **uma porta só**: o Apply não re-pergunta ao motor, ele materializa
/// exatamente o que está na tela. Uma segunda chamada ao `pathfinder` faria a forma SALTAR no
/// clique se qualquer coisa a montante tivesse mudado — o defeito que o ADR-0128 pagou cinco
/// vezes com o Expand do Blend.
pub(crate) struct Cooked {
    /// Os operandos que desenharam **nada** — a geometria deles vive dentro do resultado de outro,
    /// e o Apply os remove sem repor.
    pub(crate) consumed: Vec<VecPathId>,
    /// Quem carrega um resultado, **na ordem de z**, com a geometria em MUNDO. O Apply remove cada
    /// um e insere o resultado na fatia de z dele.
    ///
    /// ⚠️ É uma LISTA, e não um par, porque um grafo pode ter mais de um sumidouro: duas relações
    /// independentes dentro do mesmo grupo produzem duas formas, cada uma no seu lugar.
    pub(crate) sinks: Vec<(VecPathId, Vec<VecPath>)>,
}

/// A booleana viva de toda a cena, com memo por grupo (chaveado pela BASE).
#[derive(Default)]
pub(crate) struct BoolLive {
    memo: BTreeMap<VecPathId, Memo>,
    /// O que cada grupo desenhou NESTE frame, por bits de entidade. Reconstruído a cada
    /// `recook` — é uma fotografia, não estado.
    plans: Vec<(u64, Cooked)>,
}

impl BoolLive {
    /// **Aplica a booleana viva sobre o mapa do frame.** Chamado DEPOIS dos cinco produtores que
    /// estendem e ANTES do alinhamento (que é um campo do `StrokeSpec` do RESULTADO) e do
    /// `fx_silhouette` (a silhueta é do que se desenha, e o que se desenha já é isto).
    pub(crate) fn recook(
        &mut self,
        scene: &VecScene,
        sim: &SimWorld,
        map: &VecEntityMap,
        xforms: &VecXforms,
        live: &mut LiveGeometry,
    ) {
        let mut touched: Vec<VecPathId> = Vec::new();
        self.plans.clear();
        for (group, op) in groups_deepest_first(scene, sim, map) {
            let ids = crate::vec_entities::subtree_paths(sim, scene, group);
            // Os operandos são as regiões FECHADAS, e só elas — a mesma triagem que a booleana
            // destrutiva faz (`selected_closed_z`). Um caminho ABERTO dentro do grupo não é
            // operando e **passa verbatim**: ele não é suprimido, porque suprimi-lo apagaria arte
            // que a operação nunca consumiu.
            let mut operands: Vec<(VecPathId, Vec<VecPath>)> = Vec::new();
            for id in ids {
                let items = input_of(scene, xforms, live, id);
                if items.iter().any(|p| p.closed) {
                    operands.push((id, items.into_iter().filter(|p| p.closed).collect()));
                }
            }
            if operands.len() < 2 {
                continue;
            }
            let base = operands[0].0;
            // ⚠️ As ligações são filtradas pelos operandos VIVOS. Uma que nomeie uma forma apagada
            // (ou aberta, ou tirada do grupo) faria o resolvedor recusar o grafo INTEIRO, e a
            // booleana de um grupo que o artista nem tocou desapareceria da tela. A limpeza é do
            // lado da autoria; esta é a rede que a torna não-fatal.
            // ⚠️ `Option`, e não a lista vazia: a PRESENÇA do componente é que diz *"este grupo é
            // um grafo"*. Com uma lista vazia a cair de volta na operação única, cortar o último
            // elo do diagrama faria as formas FUNDIREM-SE — o oposto exato do gesto.
            let edges: Option<Vec<VecBoolEdge>> = sim.world().get::<VecBoolEdges>(group).map(|g| {
                g.edges
                    .iter()
                    .copied()
                    .filter(|e| {
                        operands.iter().any(|(id, _)| *id == e.from)
                            && operands.iter().any(|(id, _)| *id == e.to)
                    })
                    .collect()
            });
            let input: Vec<VecPath> = operands.iter().flat_map(|(_, v)| v.clone()).collect();
            let hit = self
                .memo
                .get(&base)
                .is_some_and(|m| m.op == op && m.edges == edges && m.input == input);
            if !hit {
                let Some(out) = cook(op, edges.as_deref(), &operands) else {
                    // Código desconhecido, ciclo, ou o motor recusou. O mapa fica INTOCADO — a arte
                    // continua como estava.
                    continue;
                };
                self.memo.insert(
                    base,
                    Memo {
                        op,
                        edges,
                        input,
                        out,
                    },
                );
            }
            if let Some(m) = self.memo.get(&base) {
                touched.push(base);
                for (id, items) in &m.out {
                    live.insert(*id, items.clone());
                }
                self.plans.push((
                    group.to_bits(),
                    Cooked {
                        consumed: m
                            .out
                            .iter()
                            .filter(|(_, v)| v.is_empty())
                            .map(|(id, _)| *id)
                            .collect(),
                        sinks: m
                            .out
                            .iter()
                            .filter(|(_, v)| !v.is_empty())
                            .cloned()
                            .collect(),
                    },
                ));
            }
        }
        // O memo não sobrevive ao componente: um grupo desfeito (Ungroup, Apply, Ctrl+Z) manteria
        // a resposta velha e a re-serviria se ele voltasse com outra geometria.
        self.memo.retain(|id, _| touched.contains(id));
    }

    /// **O que este grupo desenhou neste frame.** `None` = ele não cozinhou (menos de dois
    /// operandos fechados, código desconhecido, ou o motor recusou).
    #[must_use]
    pub(crate) fn plan(&self, group: Entity) -> Option<&Cooked> {
        let bits = group.to_bits();
        self.plans.iter().find(|(b, _)| *b == bits).map(|(_, c)| c)
    }

    /// Esquece tudo — o load de projeto e o restore de undo trocam a cena inteira debaixo do
    /// memo, e os `VecPathId` são reciclados entre documentos.
    pub(crate) fn forget(&mut self) {
        self.memo.clear();
        self.plans.clear();
    }
}

/// **O que cada operando desenha** — a porta única, e a bifurcação inteira desta feature.
///
/// `None` = não cozinhou (código desconhecido, ciclo, ou o motor recusou), e quem chama deixa o
/// mapa intocado.
///
/// # `None` de ligações é o caminho de HOJE — e ele fica
///
/// ⚠️ Não é hesitação: as quatro **receitas** (`MinusBack`/`Trim`/`Crop`/`Merge`) são afirmações
/// sobre a PILHA inteira e não cabem numa seta — *"cada forma menos a união do que está acima
/// dela"* não é uma relação entre dois. Só as quatro operações de conjunto viram ligação, então um
/// grupo sem grafo continua a ser resolvido pelo `pathfinder` N-ário.
///
/// E o que impede isto de virar **dois motores sobre um estado** é uma igualdade PROVADA: o gate
/// `a_estrela_derivada_desenha_o_que_o_grupo_de_hoje_desenha` (em `ph2d-vec-boolean`) prende os
/// dois caminhos com `assert_eq!` sobre a geometria inteira, para as quatro operações de conjunto.
/// Os dois caminhos existem porque respondem a perguntas diferentes, não porque discordam.
///
/// ⚠️ `Some(&[])` é o **grafo sem ligação nenhuma**, e resolve por aqui: cada forma desenha-se a si
/// própria. Ver o doc do [`VecBoolEdges`].
fn cook(
    op: u8,
    edges: Option<&[VecBoolEdge]>,
    operands: &[(VecPathId, Vec<VecPath>)],
) -> Option<Vec<(VecPathId, Vec<VecPath>)>> {
    let Some(edges) = edges else {
        let input: Vec<&VecPath> = operands.iter().flat_map(|(_, v)| v.iter()).collect();
        // Código que este build não conhece: o grupo desenha como grupo comum. É a degradação que
        // não perde arte (o doc do componente a declara).
        let out = ph2d_vec_boolean::pathfinder(&input, op_of_code(op)?).ok()?;
        // A saída vai no id da BASE — o operando mais ao FUNDO —, e cada um dos demais recebe uma
        // lista VAZIA. É a mesma regra que a booleana DESTRUTIVA já segue.
        let mut all: Vec<(VecPathId, Vec<VecPath>)> =
            operands.iter().map(|(id, _)| (*id, Vec::new())).collect();
        all[0].1 = out;
        return Some(all);
    };
    let g: Vec<ph2d_vec_boolean::BoolEdge> = edges
        .iter()
        .map(|e| {
            op_of_code(e.op).map(|op| ph2d_vec_boolean::BoolEdge {
                from: e.from,
                to: e.to,
                op,
            })
        })
        .collect::<Option<_>>()?;
    ph2d_vec_boolean::resolve_graph(operands, &g).ok()
}

/// A entrada de um operando: o que o mapa já diz, ou a fonte assada em MUNDO.
///
/// ⚠️ A conversão para mundo é a mesma do `align_live`/`offset_live`, e pelo mesmo motivo: o
/// `dispatch` sobe a geometria DERIVADA pela CÂMERA, não pelo afim do path — aplicar a pose duas
/// vezes foi bug real desta linha.
fn input_of(
    scene: &VecScene,
    xforms: &VecXforms,
    live: &LiveGeometry,
    id: VecPathId,
) -> Vec<VecPath> {
    match live.get(&id) {
        Some(items) => items.clone(),
        None => match scene.path(id) {
            Some(p) => {
                let mut world = p.cooked().into_owned();
                bake_xform(&mut world, &xform_of(xforms, id));
                vec![world]
            }
            None => Vec::new(),
        },
    }
}

/// Os grupos booleanos da cena, **do mais fundo para o mais raso**.
///
/// Descobertos subindo a partir de cada caminho — a shell não tem uma query de mundo à mão aqui, e
/// subir é o que os irmãos já fazem (`top_ancestor`). Um grupo aparece uma vez só.
fn groups_deepest_first(scene: &VecScene, sim: &SimWorld, map: &VecEntityMap) -> Vec<(Entity, u8)> {
    let w = sim.world();
    let mut found: Vec<(Entity, u8, usize)> = Vec::new();
    for path in scene.paths() {
        let Some(&bits) = map.get(&path.id) else {
            continue;
        };
        let mut cur = Entity::from_bits(bits);
        // Sobe a cadeia inteira: um grupo booleano dentro de outro é caso legítimo, e os dois
        // entram na lista (a ordem por profundidade é que os faz compor).
        for _ in 0..MAX_DEPTH {
            let Some(parent) = w.get::<ph2d_ecs::ChildOf>(cur).map(|c| c.parent()) else {
                break;
            };
            if let Some(g) = w.get::<VecBoolGroup>(parent)
                && !found.iter().any(|(e, _, _)| *e == parent)
            {
                found.push((parent, g.op, depth_of(sim, parent)));
            }
            cur = parent;
        }
    }
    // Profundidade DECRESCENTE: o de dentro cozinha antes, e o de fora encontra o resultado dele
    // no mapa. Empate resolve por `Entity` para a ordem ser estável entre frames.
    found.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)));
    found.into_iter().map(|(e, op, _)| (e, op)).collect()
}

/// Quantos ancestrais `entity` tem. É a profundidade do **GRUPO**, não a distância até o caminho
/// onde ele foi encontrado.
///
/// ⚠️ A distinção não é acadêmica e custou o gate do aninhamento: subindo a partir de um caminho,
/// o grupo de FORA fica *mais longe* (número maior) que o de dentro — a ordem sai exatamente
/// invertida, e a composição cozinha ao contrário em silêncio. A profundidade de um grupo é
/// propriedade dele, e é a única que compara dois grupos que foram achados por caminhos
/// diferentes.
fn depth_of(sim: &SimWorld, entity: Entity) -> usize {
    let w = sim.world();
    let mut cur = entity;
    let mut n = 0usize;
    for _ in 0..MAX_DEPTH {
        match w.get::<ph2d_ecs::ChildOf>(cur) {
            Some(c) => {
                cur = c.parent();
                n += 1;
            }
            None => break,
        }
    }
    n
}

/// Teto de profundidade da caminhada de ancestral (defesa, não limite de produto) — o mesmo
/// número, e pelo mesmo motivo, que o `vec_entities::MAX_DEPTH`.
const MAX_DEPTH: usize = 64;

/// O código do painel → a operação do Pathfinder. `None` = código que este build não conhece.
///
/// ⚠️ Esta é a **porta única** da tradução, e ela existe porque o `ph2d-ecs` não depende do vetor:
/// o componente carrega o `u8` cru (como o `VecShape::Param{kind}` faz com o catálogo de formas),
/// e uma segunda tabela em qualquer outro sítio divergiria dela no dia em que a 9ª operação
/// aparecesse.
#[must_use]
pub(crate) fn op_of_code(code: u8) -> Option<ph2d_vec_boolean::PathfinderOp> {
    use ph2d_vec_boolean::PathfinderOp as P;
    Some(match code {
        0 => P::Union,
        1 => P::Subtract,
        2 => P::Intersect,
        3 => P::Exclude,
        4 => P::MinusBack,
        5 => P::Trim,
        6 => P::Crop,
        7 => P::Merge,
        _ => return None,
    })
}

/// A operação do Pathfinder → o código que o componente guarda. A inversa exata de
/// [`op_of_code`], e as duas moram lado a lado porque uma tabela sem a sua inversa é a metade que
/// alguém reescreve com os índices trocados.
#[must_use]
pub(crate) fn code_of_op(op: ph2d_vec_boolean::PathfinderOp) -> u8 {
    use ph2d_vec_boolean::PathfinderOp as P;
    match op {
        P::Union => 0,
        P::Subtract => 1,
        P::Intersect => 2,
        P::Exclude => 3,
        P::MinusBack => 4,
        P::Trim => 5,
        P::Crop => 6,
        P::Merge => 7,
    }
}

#[cfg(test)]
#[path = "bool_live_tests.rs"]
mod tests;
