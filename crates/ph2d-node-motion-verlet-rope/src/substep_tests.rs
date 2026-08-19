//! **OS SUB-PASSOS** — os gates do grupo K2 da folha 03.
//!
//! Assunto próprio: aqui não se pergunta *o que a corda desenha*, pergunta-se
//! *quantas vezes por tique ela volta a perguntar onde está*. As três fixtures
//! (`steady_stretch`, `travel`, `free_fall`) moram no `substep_probe`, que é
//! quem as construiu para MEDIR — uma segunda cópia aqui divergiria da tabela
//! que o doc-comment do param cita.

use super::*;
use crate::substep_probe::{free_fall, steady_stretch, travel};

/// **`substeps = 1` É O MUNDO DE ANTES, BIT A BIT.**
///
/// O caminho de um sub-passo é LITERAL — `prev` verbatim na entrada, a pose de
/// entrada do tique verbatim na saída.
///
/// ⚠️ **Este gate NÃO prova que o caminho literal é necessário, e a medição diz
/// por quê:** no regime da corda `prev` está a um deslocamento de tique de
/// `pos`, onde o lema de Sterbenz torna `a − (a − b)` exactamente `b` (144 de
/// 144 pares realistas medidos). A mutação que troca o ramo pela fórmula
/// **sobrevive**; ela só morde com um `prev` perto da origem e a pose longe
/// dela, ou com um zero negativo. O que este gate prova é o que interessa ao
/// artista: **um documento sem o param novo desenha a corda de sempre, ao
/// bit**.
#[test]
fn one_substep_is_the_old_rope_to_the_bit() {
    let count = 12;
    let seg = 0.5f32;
    let mut p = Params {
        count,
        seg_rest: seg,
        gravity: 9.8,
        iterations: 8,
        damping: 0.02,
        pin_tail: false,
        bend: 0.3,
        substeps: 1,
    };
    let mut pos: Vec<[f32; 2]> = (0..count).map(|i| [i as f32 * seg, 0.0]).collect();
    let mut prev = pos.clone();
    // Dois tiques com âncora a mexer-se, para o estado sair do repouso.
    for k in 0..2 {
        let a = [k as f32 * 0.3, 0.0];
        let (np, pp) = step(pos.clone(), &prev, &[], &[], a, [0.0, 0.0], 1.0 / 60.0, &p);
        pos = np;
        prev = pp;
    }
    let (one_pos, _one_prev) = step(
        pos.clone(),
        &prev,
        &[],
        &[],
        [0.6, 0.0],
        [0.0, 0.0],
        1.0 / 60.0,
        &p,
    );

    // O CONTROLE de que a fixture contém o fenômeno: com DOIS sub-passos o
    // mesmo tique tem de dar OUTROS bits, senão o gate abaixo é satisfeito por
    // o param não fazer nada.
    p.substeps = 2;
    let (two_pos, _) = step(
        pos.clone(),
        &prev,
        &[],
        &[],
        [0.6, 0.0],
        [0.0, 0.0],
        1.0 / 60.0,
        &p,
    );
    let moved = (0..count)
        .flat_map(|i| (0..2).map(move |c| (i, c)))
        .any(|(i, c)| one_pos[i][c].to_bits() != two_pos[i][c].to_bits());
    assert!(
        moved,
        "o CONTROLE: dois sub-passos TÊM de dar outros bits, senão o param é inerte \
         e a byte-identidade abaixo não prova nada"
    );

    // E o que o artista de facto recebe: **um documento que nunca escreveu o
    // param** desenha a corda de sempre. Isto passa pela PORTA DE LEITURA
    // (`ctx.param` → o default do manifesto → o clamp), que é onde o `1` mora —
    // comparar `Params { substeps: 1 }` consigo mesmo seria `f(x) == f(x)`, um
    // `assert_eq!` que não pode falhar.
    let absent = cook_rope_pose(None);
    let explicit_one = cook_rope_pose(Some(1.0));
    assert_eq!(absent.len(), explicit_one.len(), "a mesma corda");
    for (i, (a, b)) in absent.iter().zip(explicit_one.iter()).enumerate() {
        for c in 0..2 {
            assert_eq!(
                a[c].to_bits(),
                b[c].to_bits(),
                "o param AUSENTE resolve a 1, ao bit (ponto {i}, eixo {c})"
            );
        }
    }

    // ...e o mesmo documento com `4` NÃO pode dar os mesmos bits — senão a
    // igualdade acima seria verde porque a porta de leitura ignora o param.
    let four = cook_rope_pose(Some(4.0));
    assert!(
        absent
            .iter()
            .zip(four.iter())
            .any(|(a, b)| a[1].to_bits() != b[1].to_bits()),
        "o CONTROLE da porta: `substeps = 4` tem de mudar a pose"
    );
}

/// Coze uma corda de 20 tiques e devolve a pose final, com o `substeps`
/// **escrito ou AUSENTE** do documento — a distinção que o gate acima precisa e
/// que uma `Params` construída à mão não sabe exprimir.
fn cook_rope_pose(substeps: Option<f32>) -> Vec<[f32; 2]> {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MANIFEST.id).then_some(&MotionVerletRope as &dyn NodeOp)
        }
    }
    let mut g = Graph::new();
    let rope = g.add_node("motion.verlet_rope");
    g.set_param(rope, "count", 12.0);
    g.set_param(rope, "gravity", 9.8);
    if let Some(s) = substeps {
        g.set_param(rope, "solver_substeps", s);
    }
    g.connect(Edge {
        from: (rope, 0),
        to: (rope, 2),
        delayed: true,
    })
    .expect("o self-loop `pre`");

    let mut cook = Cook::new();
    let mut last = Vec::new();
    // ⚠️ **O `advance_tick` é o que faz a aresta `pre` carregar estado.** Sem ele
    // a corda fica em repouso e as duas colunas saem bit a bit iguais por a cena
    // estar MORTA, não por a lei concordar.
    for k in 0..20 {
        let t = k as f64 / 60.0;
        let out = cook.cook(&g, &Ops, rope, t).expect("coze");
        if let Some(Column::Vec2(v)) = out[0].as_stream().get("P") {
            last = v.clone();
        }
        cook.advance_tick(&g, &Ops, t).expect("tique");
    }
    last
}

/// **A ORÇAMENTO IGUAL, OS SUB-PASSOS CONVERGEM MELHOR QUE AS ITERAÇÕES.**
///
/// O achado do *Small Steps in Physics Simulation* (Macklin et al., SCA 2019):
/// para o MESMO número total de correções, muitos sub-passos com poucas
/// iterações batem um passo com muitas — porque uma iteração propaga uma
/// correção já computada, e um sub-passo **volta a perguntar onde a corda
/// está**.
///
/// Medido (24 nós, gravidade 98, orçamento 24): `1 x 24` estica **8,3008%** e
/// `24 x 1` estica **0,5253%** — quinze vezes menos. ⚠️ A barra é uma RAZÃO e
/// não um número absoluto: o esticão de um solver iterativo depende da
/// gravidade, da contagem e do comprimento, e um limiar fixo envelheceria na
/// primeira mudança de fixture.
#[test]
fn at_equal_budget_substeps_converge_better_than_iterations() {
    let one_pass = steady_stretch(24, 98.0, 24, 1);
    let many = steady_stretch(24, 98.0, 1, 24);
    assert!(
        one_pass > many * 5.0,
        "a orçamento igual, 24 sub-passos tem de bater 24 iteracoes por larga \
         margem; um passo esticou {:.4}% e vinte e quatro esticaram {:.4}%",
        one_pass * 100.0,
        many * 100.0
    );
}

/// **AS `iterations` SOZINHAS NÃO ALCANÇAM O QUE OS SUB-PASSOS ALCANÇAM.**
///
/// É este gate que prova que o param novo **não é** um segundo nome para o que
/// já existia. Medido: `1 x 128` (cinco vezes o custo) estica **1,1266%** e
/// ainda perde para `4 x 24`, que estica **0,4772%**. Um solver de posição
/// relaxa em direção às restrições do MESMO estado inicial, então iterar mais
/// converge para a resposta daquele passo — e a resposta daquele passo é que
/// está errada quando o passo é grande.
#[test]
fn iterations_alone_do_not_reach_what_substeps_reach() {
    let iters_128 = steady_stretch(24, 98.0, 128, 1);
    let subs_4 = steady_stretch(24, 98.0, 24, 4);
    assert!(
        subs_4 < iters_128,
        "quatro sub-passos ({:.4}%) tem de bater cento e vinte e oito iteracoes \
         ({:.4}%); se nao baterem, o param novo e' um sinonimo do que ja existia",
        subs_4 * 100.0,
        iters_128 * 100.0
    );
}

/// **A INÉRCIA QUE CHEGA É DE UM TIQUE, E O SUB-PASSO É UMA FRAÇÃO DELE.**
///
/// A metade da ENTRADA: o `prev` que o tique recebe codifica um deslocamento de
/// um tique inteiro, então o primeiro sub-passo levaria `N x` a inércia se
/// ninguém o re-escalasse. Uma corda a viajar sob gravidade zero tem de
/// percorrer a mesma distância por tique, qualquer que seja o número de
/// sub-passos.
#[test]
fn the_inertia_that_arrives_is_a_ticks_worth() {
    let one = travel(1);
    let sixteen = travel(16);
    let rel = (one - sixteen).abs() / one.abs();
    assert!(
        rel < 0.02,
        "a distancia percorrida num tique nao pode depender do numero de \
         sub-passos; um passo andou {one:.4} e dezasseis andaram {sixteen:.4}"
    );
}

/// **E O QUE SAI TEM DE SER A VELOCIDADE DO ÚLTIMO SUB-PASSO, ESTICADA A UM
/// TIQUE.**
///
/// ⚠️ A viagem é CEGA a esta metade — a velocidade constante faz o deslocamento
/// MÉDIO sobre o tique ser igual ao FINAL, então devolver a pose de entrada do
/// tique passa por lá. O oráculo é a queda LIVRE, onde a média é estritamente
/// menor que a final: devolvendo a pose de entrada, cada fronteira de tique
/// come a velocidade que os sub-passos ganharam, e um segundo de queda mede
/// **−2,65 m com dezasseis sub-passos** contra os **−4,97** analíticos.
///
/// `iterations = 0` de propósito: com restrição rígida o solver puxa o ponto
/// atrasado de volta e ESCONDE o erro.
#[test]
fn the_inertia_that_leaves_is_the_last_substeps() {
    let analytic = 0.5 * 9.8; // meio a t^2 com t = 1 s
    for sub in [1usize, 2, 4, 8, 16] {
        let fell = free_fall(sub).abs();
        let err = (fell - analytic).abs() / analytic;
        assert!(
            err < 0.05,
            "a queda livre de um segundo tem de medir ~{analytic:.2} m em \
             qualquer numero de sub-passos; com {sub} ela mediu {fell:.4}"
        );
    }
}
