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

use ph2d_ecs::{Entity, SimWorld, VecBoolGroup};
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecXforms, bake_xform, xform_of};

use crate::vec_entities::VecEntityMap;

/// Uma entrada do memo: o que ENTROU (a operação + a lista de mundo) e o que SAIU.
struct Memo {
    op: u8,
    /// O verbo de cada path da entrada, já resolvido (override da forma ⟶ senão o do grupo).
    ///
    /// ⚠️ **Ele faz parte da CHAVE, e não é decoração.** Trocar o modo de uma forma não muda nem a
    /// geometria de entrada nem o `op` do grupo — sem este campo o memo daria acerto, a resposta
    /// velha seria re-servida, e o clique no seletor não faria coisa nenhuma na tela.
    verbs: Vec<ph2d_vec_boolean::BoolOp>,
    input: Vec<VecPath>,
    out: Vec<VecPath>,
}

/// **O que um grupo booleano desenhou neste frame** — a resposta que o produtor computou e que o
/// **Apply** consolida.
///
/// ⚠️ Ela existe para que haja **uma porta só**: o Apply não re-pergunta ao motor, ele materializa
/// exatamente o que está na tela. Uma segunda chamada ao `pathfinder` faria a forma SALTAR no
/// clique se qualquer coisa a montante tivesse mudado — o defeito que o ADR-0128 pagou cinco
/// vezes com o Expand do Blend.
pub(crate) struct Cooked {
    /// O operando mais ao FUNDO: quem carrega o resultado no mapa, e onde ele é inserido no bake.
    pub(crate) base: VecPathId,
    /// Todos os operandos consumidos, **na ordem de z** (a base é o primeiro).
    pub(crate) operands: Vec<VecPathId>,
    /// A geometria que o grupo desenha, em MUNDO.
    pub(crate) out: Vec<VecPath>,
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
            let input: Vec<VecPath> = operands.iter().flat_map(|(_, v)| v.clone()).collect();
            let Some(pf) = op_of_code(op) else {
                // Código que este build não conhece: o grupo desenha como grupo comum. É a
                // degradação que não perde arte (o doc do componente a declara).
                continue;
            };
            // **O VERBO DE CADA PATH DA ENTRADA.** Um operando pode contribuir com VÁRIOS paths
            // (um offset vivo, um composto), e o verbo é da FORMA — então ele repete-se por
            // quantos paths aquela forma trouxe. Uma lista por operando não serviria: quem dobra
            // é cada path, um de cada vez.
            let verbs: Vec<ph2d_vec_boolean::BoolOp> = pf.as_bool().map_or_else(Vec::new, |g| {
                operands
                    .iter()
                    .flat_map(|(id, v)| {
                        std::iter::repeat_n(operand_verb(sim, map, *id, g), v.len())
                    })
                    .collect()
            });
            let hit = self
                .memo
                .get(&base)
                .is_some_and(|m| m.op == op && m.verbs == verbs && m.input == input);
            if !hit {
                let out = match (pf.as_bool(), input.split_first()) {
                    // **As quatro de CONJUNTO: a cadeia com um verbo por passo.** Sem nenhum
                    // override isto é byte-idêntico à porta N-ária de sempre — todos os verbos
                    // são o do grupo, e a cadeia uniforme *é* o `apply_many_checked` (há gate no
                    // motor).
                    (Some(_), Some((first, rest))) => {
                        let folds: Vec<(&VecPath, ph2d_vec_boolean::BoolOp)> = rest
                            .iter()
                            .zip(verbs.iter().skip(1))
                            .map(|(p, &v)| (p, v))
                            .collect();
                        let Ok(out) = ph2d_vec_boolean::apply_chain_checked(first, &folds) else {
                            // O motor recusou. O mapa fica INTOCADO — a arte continua como estava.
                            continue;
                        };
                        out
                    }
                    // ⛔ **As quatro RECEITAS são verbos da PILHA INTEIRA**, e por isso o verbo
                    // por forma não se aplica a elas: *"cada forma menos a união do que está
                    // acima dela"* não é uma relação entre duas. Elas correm exatamente como
                    // sempre correram — e é a UI que tem de não oferecer o seletor por forma
                    // quando o grupo está numa receita, senão ele é um controlo inerte.
                    _ => {
                        let refs: Vec<&VecPath> = input.iter().collect();
                        let Ok(out) = ph2d_vec_boolean::pathfinder(&refs, pf) else {
                            continue;
                        };
                        out
                    }
                };
                self.memo.insert(
                    base,
                    Memo {
                        op,
                        verbs,
                        input,
                        out,
                    },
                );
            }
            if let Some(m) = self.memo.get(&base) {
                touched.push(base);
                live.insert(base, m.out.clone());
                for (id, _) in operands.iter().skip(1) {
                    live.insert(*id, Vec::new());
                }
                self.plans.push((
                    group.to_bits(),
                    Cooked {
                        base,
                        operands: operands.iter().map(|(id, _)| *id).collect(),
                        out: m.out.clone(),
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

    /// **O PLANO QUE CONSOME `id`**, e o grupo dele. `None` = ela não é operando de booleana viva
    /// nenhuma neste frame.
    ///
    /// ⚠️ **O mais INTERNO vence**, e não por acaso: com grupos aninhados, a base de um grupo
    /// interno é também operando do externo, e os planos estão ordenados por profundidade
    /// decrescente. O verbo de uma forma vale dentro do grupo *dela* — quem consome o resultado
    /// desse grupo é outra pergunta, com outra resposta.
    #[must_use]
    pub(crate) fn plan_containing(&self, id: VecPathId) -> Option<(Entity, &Cooked)> {
        self.plans
            .iter()
            .find(|(_, c)| c.operands.contains(&id))
            .map(|(bits, c)| (Entity::from_bits(*bits), c))
    }

    /// **QUEM ABSORVEU cada operando neste frame** — o par *(operando, base que carrega a tinta)*,
    /// publicado no `VecViewState` para o hit-test e o marquee.
    ///
    /// A base de cada grupo **não** entra: ela carrega o resultado, logo é alcançável pela própria
    /// entrada do mapa, como qualquer forma desenhada.
    ///
    /// # O ANINHAMENTO é resolvido AQUI, e não no pick
    ///
    /// ⚠️ A base de um grupo interno é ela própria um operando do grupo externo — e portanto o
    /// mapa dela também acaba **VAZIO**. Um par direto `(operando_interno → base_interna)`
    /// apontaria para uma porta sem tinta, e o operando ficaria tão inalcançável quanto antes: o
    /// defeito voltaria, **só nos documentos aninhados**, que são precisamente os que ninguém
    /// smoka. Seguir a cadeia até quem de facto desenha é uma volta por frame, e deixa o pick a
    /// fazer uma pergunta só.
    #[must_use]
    pub(crate) fn absorbed(&self) -> Vec<(VecPathId, VecPathId)> {
        let direct: Vec<(VecPathId, VecPathId)> = self
            .plans
            .iter()
            .flat_map(|(_, c)| c.operands.iter().skip(1).map(|&id| (id, c.base)))
            .collect();
        direct
            .iter()
            .map(|&(operand, base)| (operand, outermost_door(&direct, base)))
            .collect()
    }

    /// Esquece tudo — o load de projeto e o restore de undo trocam a cena inteira debaixo do
    /// memo, e os `VecPathId` são reciclados entre documentos.
    pub(crate) fn forget(&mut self) {
        self.memo.clear();
        self.plans.clear();
    }
}

/// **O verbo com que ESTA forma dobra** — o override dela, quando ele é uma das quatro operações
/// de conjunto; senão o do grupo.
///
/// ⚠️ **Ausência é HERANÇA, e é isso que mantém os oito botões do grupo vivos.** Um seletor de
/// grupo que deixasse de decidir assim que as formas passassem a mandar seria o defeito
/// *"parâmetro que não muda nada"*; aqui ele é o **padrão** de quem não se pronunciou — e é
/// também o que faz todo documento anterior a esta feature desenhar byte-idêntico.
///
/// ⚠️ Na BASE o resultado é ignorado, e não por um `if`: o verbo viaja emparelhado com o path que
/// ENTRA na cadeia, e a base não entra — ela é o acumulador inicial. É a representação a apagar o
/// caso especial.
fn operand_verb(
    sim: &SimWorld,
    map: &VecEntityMap,
    id: VecPathId,
    group: ph2d_vec_boolean::BoolOp,
) -> ph2d_vec_boolean::BoolOp {
    map.get(&id)
        .and_then(|&bits| {
            sim.world()
                .get::<ph2d_ecs::VecBoolOp>(Entity::from_bits(bits))
        })
        .and_then(|v| op_of_code(v.op))
        .and_then(ph2d_vec_boolean::PathfinderOp::as_bool)
        .unwrap_or(group)
}

/// A base que de facto **desenha**, subindo a cadeia de absorções a partir de `base`.
///
/// Uma base absorvida por um grupo mais externo tem o mapa vazio; quem tem tinta é a base do
/// grupo do topo. O teto é o número de pares — a cadeia é estritamente ascendente por
/// profundidade, então não há ciclo a temer, e o limite é defesa contra um estado corrompido.
fn outermost_door(direct: &[(VecPathId, VecPathId)], base: VecPathId) -> VecPathId {
    let mut cur = base;
    for _ in 0..direct.len() {
        match direct.iter().find(|(operand, _)| *operand == cur) {
            Some(&(_, next)) => cur = next,
            None => break,
        }
    }
    cur
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
