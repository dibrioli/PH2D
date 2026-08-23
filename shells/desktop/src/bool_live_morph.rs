//! **O QUE A TROCA DE VERBO ACRESCENTA AO COZIMENTO** — irmão do [`super`] pelo teto de 600 LOC
//! do HR-18, e o corte é por RESPONSABILIDADE: ali fica *o que um grupo booleano desenha*; aqui,
//! *o que ele desenha enquanto a operação está a mudar*.
//!
//! ⚠️ **A ponta de PARTIDA e a de CHEGADA correm pelo MESMO [`cook_side`]**, e é isso que impede
//! as duas de divergirem: duas cópias separar-se-iam no dia em que uma delas ganhasse um caso, e o
//! defeito apareceria só a meio de uma transição — o sítio mais caro de o descobrir.

use ph2d_ecs::SimWorld;
use ph2d_vec_scene::{VecPath, VecPathId};

use super::{group_above, op_of_code, operand_verb};
use crate::vec_entities::VecEntityMap;

/// **O VERBO DE CADA PATH DA ENTRADA**, de um dos dois lados.
///
/// Um operando pode contribuir com VÁRIOS paths (um offset vivo, um composto), e o verbo é da
/// FORMA — então ele repete-se por quantos paths aquela forma trouxe. Uma lista por operando não
/// serviria: quem dobra é cada path, um de cada vez.
///
/// `arriving` é o lado de CHEGADA de uma transição: para as formas que ele nomeia o verbo sai dele,
/// e para as outras sai do componente — que é justamente a afirmação de que elas não mudaram.
/// `None` ⇒ o lado de partida, lido inteiro do mundo.
pub(super) fn verbs_of(
    sim: &SimWorld,
    map: &VecEntityMap,
    operands: &[(VecPathId, Vec<VecPath>)],
    pf: ph2d_vec_boolean::PathfinderOp,
    arriving: Option<&[&ph2d_ui_state::BoolMorph]>,
) -> Vec<ph2d_vec_boolean::BoolOp> {
    pf.as_bool().map_or_else(Vec::new, |group| {
        operands
            .iter()
            .flat_map(|(id, v)| {
                let verb = match arriving.and_then(|ms| ms.iter().find(|m| m.id == *id)) {
                    // `None` na pose é *herda*, a MESMA lei da ausência do componente.
                    Some(m) => {
                        m.op.and_then(op_of_code)
                            .and_then(ph2d_vec_boolean::PathfinderOp::as_bool)
                            .unwrap_or(group)
                    }
                    None => operand_verb(sim, map, *id, group),
                };
                std::iter::repeat_n(verb, v.len())
            })
            .collect()
    })
}

/// **A PONTA DE CHEGADA deste grupo**, se alguma forma dele está a meio de uma troca de verbo —
/// *(a operação do grupo lá, o verbo de cada path lá, onde no caminho)*.
///
/// ⚠️ **Ela devolve `None` quando a chegada DESENHA O MESMO**, e isso não é economia: um estado
/// pode trocar o override de uma forma por `None` sem mudar o verbo efetivo dela (o grupo já era
/// aquele). Sem esta conferência o grupo pagaria dois cozimentos e um casamento por quadro para
/// desenhar exatamente o que um cozimento desenha.
///
/// ⚠️ **O `t` vem do PRIMEIRO recado**, e a escolha não é arbitrária: os operandos de um grupo
/// vivem na sub-árvore de um hospedeiro só, então os recados dele saem todos da mesma máquina e
/// carregam o mesmo `t`. Escolher um é escolher o único que existe.
///
/// # ⚠️ O ANINHAMENTO separa as duas leituras, e ler as duas do mesmo sítio é um defeito
///
/// A base de um grupo INTERNO é também operando do EXTERNO — e o `bool_group_op` da pose dela fala
/// do grupo **dela**. Lido pelo externo, ele mandaria o grupo de fora adotar a operação do de
/// dentro, em silêncio e **só em documentos aninhados**, que são precisamente os que ninguém smoka.
///
/// ⇒ o **verbo próprio** de um recado vale para quem quer que o consuma (é o `VecBoolOp` da forma,
/// que o grupo externo lê na mesma, porque a base dele dobra na cadeia de fora); a **operação do
/// grupo** só vale para o grupo de que a forma é filha — e quem responde isso é a porta única,
/// [`group_above`].
pub(super) fn arriving_side(
    sim: &SimWorld,
    map: &VecEntityMap,
    morphs: &[ph2d_ui_state::BoolMorph],
    group: ph2d_ecs::Entity,
    operands: &[(VecPathId, Vec<VecPath>)],
    op: u8,
    verbs: &[ph2d_vec_boolean::BoolOp],
) -> Option<(
    ph2d_vec_boolean::PathfinderOp,
    Vec<ph2d_vec_boolean::BoolOp>,
    f64,
)> {
    let mine: Vec<&ph2d_ui_state::BoolMorph> = morphs
        .iter()
        .filter(|m| operands.iter().any(|(id, _)| *id == m.id))
        .collect();
    let t = mine.first()?.t;
    let op_to = mine
        .iter()
        .filter(|m| group_above(sim, map, m.id).is_some_and(|(g, _)| g == group))
        .find_map(|m| m.group_op)
        .unwrap_or(op);
    let pf_to = op_of_code(op_to)?;
    let verbs_to = verbs_of(sim, map, operands, pf_to, Some(&mine));
    (op_to != op || verbs_to != verbs).then_some((pf_to, verbs_to, t))
}

/// **UM lado da booleana**, cozido. `None` = o motor recusou.
///
/// ⚠️ Ela é uma função e não duas cópias porque a partida e a chegada têm de correr pelo MESMO
/// caminho: duas cópias divergiriam no dia em que uma delas ganhasse um caso, e o defeito
/// apareceria só a meio de uma transição — o sítio mais caro de o descobrir.
pub(super) fn cook_side(
    pf: ph2d_vec_boolean::PathfinderOp,
    verbs: &[ph2d_vec_boolean::BoolOp],
    input: &[VecPath],
) -> Option<Vec<VecPath>> {
    match (pf.as_bool(), input.split_first()) {
        // **As quatro de CONJUNTO: a cadeia com um verbo por passo.** Sem nenhum override isto é
        // byte-idêntico à porta N-ária de sempre — todos os verbos são o do grupo, e a cadeia
        // uniforme *é* o `apply_many_checked` (há gate no motor).
        (Some(_), Some((first, rest))) => {
            let folds: Vec<(&VecPath, ph2d_vec_boolean::BoolOp)> = rest
                .iter()
                .zip(verbs.iter().skip(1))
                .map(|(p, &v)| (p, v))
                .collect();
            ph2d_vec_boolean::apply_chain_checked(first, &folds).ok()
        }
        // ⛔ **As quatro RECEITAS são verbos da PILHA INTEIRA**, e por isso o verbo por forma não
        // se aplica a elas: *"cada forma menos a união do que está acima dela"* não é uma relação
        // entre duas. Elas correm exatamente como sempre correram — e é a UI que tem de não
        // oferecer o seletor por forma quando o grupo está numa receita, senão ele é um controlo
        // inerte.
        _ => {
            let refs: Vec<&VecPath> = input.iter().collect();
            ph2d_vec_boolean::pathfinder(&refs, pf).ok()
        }
    }
}

/// ⭐ **O DESENHO ENTRE DOIS RESULTADOS booleanos** — o que a troca de verbo mostra no meio.
///
/// ⚠️ **Pelo `Plan`, que é o motor de morph desta casa** (o Blend Object, o Morph do artista e o
/// Smart Animate já são ele). Um segundo motor aqui divergiria daquele, e a divergência só
/// apareceria numa screenshot — *dois motores e um estado é pior que um motor lento*.
///
/// ⚠️ **Sem plano, fica-se na PARTIDA**: a mesma lei do par degenerado do `Transition::at`.
/// Inventar um caminho que o motor não sabe traçar seria um salto no primeiro quadro.
///
/// # ⛔ O que foi MEDIDO e recusado
///
/// - **Saltar o verbo** (Blender/AE/Rive): a troca move **64,0** de tinta num quadro mesmo com a
///   peça parada, contra **3,1** do morph.
/// - **Perseguir a partir do vivo** (morfar do que está na tela para a chegada, a fração do que
///   falta): cura o único quadro em que o morph salta, e paga com o desenho a **ficar para trás
///   do movimento** — numa peça que viaja 5× a própria largura ele afasta-se **793,0** de tinta
///   do par fresco, e salta **379,7** contra os 94,0 do par fresco (que é o próprio movimento).
///
/// ⚠️ **O que fica por curar, medido e nomeado:** quando o MOVIMENTO dos operandos muda a
/// topologia de uma das duas pontas a meio da transição (um operando a atravessar a parede da
/// peça), o desenho dá **um** passo de 38,3 de tinta nesse quadro — ainda 38% menor que o salto da
/// indústria, que acontece em todos os casos.
pub(super) fn morph_results(from: &[VecPath], to: &[VecPath], t: f64) -> Vec<VecPath> {
    match (as_one(from), as_one(to)) {
        (Some(a), Some(b)) => {
            ph2d_vec_blend::Plan::new(&a, &b).map_or_else(|| from.to_vec(), |plan| vec![plan.at(t)])
        }
        _ => from.to_vec(),
    }
}

/// **Uma lista de resultados vira UMA forma composta.**
///
/// Uma booleana pode devolver vários grupos disjuntos (uma dentada que parte a peça em duas), e o
/// [`ph2d_vec_blend::Plan`] casa DUAS formas. Juntar os contornos todos numa forma composta é o que
/// deixa o casamento decidir contorno a contorno — escolher por ÍNDICE qual grupo vira qual seria
/// uma segunda regra de correspondência ao lado da que o `Plan` já tem, e a pior das duas.
pub(super) fn as_one(items: &[VecPath]) -> Option<VecPath> {
    let mut it = items.iter();
    let mut out = it.next()?.clone();
    for extra in it {
        for c in 0..extra.contour_count() {
            if let Some((verts, closed)) = extra.contour(c) {
                out.subpaths.push(ph2d_vec_scene::Contour {
                    verts: verts.to_vec(),
                    closed,
                });
            }
        }
    }
    Some(out)
}
