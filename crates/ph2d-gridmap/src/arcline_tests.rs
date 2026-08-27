use super::{Signed, measure_arc_lines};
use crate::cut::CutMesh;
use crate::solve::GridMap;
use crate::solve::turn2;

/// ⭐⭐⭐ **A IDENTIDADE QUE A EQUAÇÃO INTEIRA ASSENTA:** `e·(R^rot·v) = turn2(e, −rot)·v`.
///
/// ⚠️ **É aqui que um sinal troca sem que nada deixe de compilar.** O gate não lê as
/// entradas de matriz nenhuma — ele avalia os dois lados e compara, para os quatro `rot`
/// e os dois eixos. *Uma dedução à mão verificada por outra dedução à mão não é uma
/// verificação.*
#[test]
fn the_axis_identity_holds_for_every_turn() {
    let dot = |a: [f32; 2], b: [f32; 2]| a[0].mul_add(b[0], a[1] * b[1]);
    for e in [[1.0, 0.0], [0.0, 1.0]] {
        for rot in 0..4 {
            for v in [[1.0, 0.0], [0.0, 1.0], [3.0, -7.0], [-2.0, 5.0]] {
                let left = dot(e, turn2(v, rot));
                let right = dot(turn2(e, -rot), v);
                assert!(
                    (left - right).abs() < 1e-5,
                    "e={e:?} rot={rot} v={v:?}: {left} != {right}"
                );
            }
        }
    }
}

/// ⭐ E o resultado de `turn2(e, −rot)` é sempre um EIXO com sinal — é isso que faz os
/// coeficientes serem `±1` e a eliminação levar inteiros a inteiros.
#[test]
fn a_quarter_turn_of_an_axis_is_an_axis() {
    for e in [[1.0, 0.0], [0.0, 1.0]] {
        for rot in -4..=4 {
            let v = turn2(e, -rot);
            let zeros = usize::from(v[0].abs() < 1e-6) + usize::from(v[1].abs() < 1e-6);
            assert_eq!(zeros, 1, "e={e:?} rot={rot} ⇒ {v:?} nao e' um eixo");
            assert!((v[0].abs().max(v[1].abs()) - 1.0).abs() < 1e-6);
        }
    }
}

/// A união com sinal compõe: `y_c = σ₂·y_b + δ₂` e `y_b = σ₁·y_a + δ₁` ⇒ o `find` de `c`
/// tem de devolver `σ₁σ₂` e `σ₂δ₁ + δ₂`.
#[test]
fn the_signed_union_composes() {
    let mut uf = Signed::new(3);
    // `y_b = −1·y_a + 4`
    uf.parent[1] = 0;
    uf.sign[1] = -1.0;
    uf.off[1] = 4.0;
    // `y_c = −1·y_b + 3`
    uf.parent[2] = 1;
    uf.sign[2] = -1.0;
    uf.off[2] = 3.0;
    let (root, s, d) = uf.find(2);
    assert_eq!(root, 0);
    assert!((s - 1.0).abs() < 1e-6, "sinal {s}");
    // `y_c = −(−y_a + 4) + 3 = y_a − 1`
    assert!((d + 1.0).abs() < 1e-6, "deslocamento {d}");
}

/// ⚠️ E a compressão de caminho **não pode mudar a resposta** — ela reescreve `sign`/`off`
/// enquanto os lê. *Um `find` que se corrompe a si próprio dá a resposta certa uma vez.*
#[test]
fn path_compression_does_not_move_the_answer() {
    let mut uf = Signed::new(4);
    for (child, parent, s, d) in [
        (1u32, 0u32, -1.0, 4.0),
        (2, 1, -1.0, 3.0),
        (3, 2, -1.0, 1.0),
    ] {
        uf.parent[child as usize] = parent;
        uf.sign[child as usize] = s;
        uf.off[child as usize] = d;
    }
    let first = uf.find(3);
    let second = uf.find(3);
    assert_eq!(first.0, second.0);
    assert!((first.1 - second.1).abs() < 1e-6);
    assert!((first.2 - second.2).abs() < 1e-6);
}

/// Sem costuras não há equação nenhuma, e `0` ali significa **«nada a impor»**.
#[test]
fn no_seams_means_no_equations() {
    let cut = CutMesh::default();
    let w = crate::weld::Weld::default();
    let map = GridMap::default();
    let r = measure_arc_lines(&cut, &w, &map);
    assert_eq!(r.arcs, 0);
    assert_eq!(r.sign_conflicts, 0);
    assert_eq!(r.eliminated, 0);
}

/// ⭐⭐⭐ **O INTERRUPTOR DESLIGADO É INERTE, BIT A BIT.**
///
/// ⚠️ *É o controlo da wave inteira.* Sem ele, «a saída mudou» e «a wave fez alguma
/// coisa» leem-se igual — e a segunda pode ser falsa com a primeira verdadeira.
#[test]
fn the_ties_switch_is_inert_when_off() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(16, 24, 1.0);
    mesh.triangulate();
    let dual = ph2d_crossfield::Dual::build(&mesh);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let layout = ph2d_trace::trace_patches(&mesh, &dual, &field);
    let (cut, _) = crate::cut::cut_along_patches(&mesh, &layout);
    let (combed, _) = crate::comb::comb_patches(&mesh, &layout, &cut);
    let h = 0.2;
    let (a, _) = crate::weld_solve_driver::solve_welded(&mesh, &cut, &combed, h, 4);
    let (b, _) =
        crate::weld_solve_driver::solve_welded_with(&mesh, &cut, &combed, h, 4, None, None);
    assert_eq!(a.shift, b.shift);
    assert_eq!(a.uv.len(), b.uv.len());
    for (ra, rb) in a.uv.iter().zip(&b.uv) {
        assert_eq!(ra, rb, "o mapa mudou com o interruptor DESLIGADO");
    }
}

/// ⭐⭐ **E LIGADO ELE MEXE** — a saída deixa de ser a mesma.
///
/// ⚠️ Este gate não afirma que ela ficou **melhor**; afirma que a restrição **entrou**.
/// *Um interruptor que não move nada e um que melhora tudo leem igual num gate de
/// igualdade, e só um deles é o que se construiu.*
#[test]
fn the_ties_change_the_map_when_on() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(16, 24, 1.0);
    mesh.triangulate();
    let dual = ph2d_crossfield::Dual::build(&mesh);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let layout = ph2d_trace::trace_patches(&mesh, &dual, &field);
    let (cut, _) = crate::cut::cut_along_patches(&mesh, &layout);
    let (combed, _) = crate::comb::comb_patches(&mesh, &layout, &cut);
    let h = 0.2;
    let (base, _) = crate::weld_solve_driver::solve_welded(&mesh, &cut, &combed, h, 4);
    let (w, _) = crate::weld::weld(&cut, &combed);
    let ties = super::build_arc_ties(&cut, &w, &base);
    assert!(ties.groups() > 0, "a esfera tem de dar grupos de amarra");
    let (tied, rep) =
        crate::weld_solve_driver::solve_welded_with(&mesh, &cut, &combed, h, 4, Some(&ties), None);
    assert!(
        rep.tie_groups > 0,
        "nenhum grupo entrou: {} recusados",
        rep.tie_refused
    );
    let moved = base
        .uv
        .iter()
        .zip(&tied.uv)
        .any(|(ra, rb)| ra.iter().zip(rb).any(|(a, b)| a != b));
    assert!(moved, "as amarras nao moveram o mapa");
}

/// ⭐⭐⭐ **A ÁLGEBRA TEM DE REPRODUZIR A GEOMETRIA** — e é este o gate que decide se a
/// equação está certa.
///
/// O resíduo de [`ArcEquation::residual`] é a **componente atravessada** do arco, lida
/// por dentro (somando termos sobre variáveis). A
/// [`crate::align::measure_arc_quantization`] lê a **mesma** grandeza por fora (a
/// diferença das duas posições). ⚠️ *Dois caminhos independentes até o mesmo número — se
/// discordarem, é a álgebra que está errada, e descobre-se AGORA e não depois da
/// eliminação.*
///
/// ⛔ Sem este gate, um sinal trocado no `off` passaria despercebido até a restrição
/// puxar o mapa para o sítio errado — e ali já haveria duas variáveis a bissecar.
#[test]
fn the_equation_residual_matches_the_geometric_reading() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(16, 24, 1.0);
    mesh.triangulate();
    let dual = ph2d_crossfield::Dual::build(&mesh);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let layout = ph2d_trace::trace_patches(&mesh, &dual, &field);
    let (cut, _) = crate::cut::cut_along_patches(&mesh, &layout);
    let (combed, _) = crate::comb::comb_patches(&mesh, &layout, &cut);
    let (map, _) = crate::weld_solve_driver::solve_welded(&mesh, &cut, &combed, 0.2, 6);
    let (w, _) = crate::weld::weld(&cut, &combed);

    let eqs = super::arc_equations(&cut, &w, &map);
    assert!(!eqs.is_empty(), "a esfera tem de dar equacoes");
    // A leitura geométrica: por arco, o menor componente do deslocamento dos extremos.
    let demand = vec![0u32; cut.seams.len() + 1];
    let geo = crate::align::measure_arc_quantization(&cut, &map, &demand);
    assert_eq!(
        eqs.len(),
        geo.arcs,
        "as duas reguas tem de ver os MESMOS arcos"
    );

    let mut worst = 0.0f32;
    for eq in &eqs {
        worst = worst.max(eq.residual(&w, &map).abs());
    }
    // ⚠️ A barra é o `max` da geométrica, com folga de `f32`: as duas contas somam os
    // mesmos termos por ordens diferentes.
    assert!(
        (worst - geo.across_max).abs() < 1.0e-3,
        "algebra {worst} contra geometria {} — a equacao nao reproduz a leitura",
        geo.across_max
    );
}

/// ⭐ **TODO coeficiente é `±1`** — é isso que faz a eliminação levar inteiros a inteiros.
///
/// ⚠️ *Um coeficiente `2` seria meia célula*, que é exactamente o que o `worst_det` do
/// [`crate::weld_flat`] existe para contar.
#[test]
fn every_arc_coefficient_is_plus_or_minus_one() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(16, 24, 1.0);
    mesh.triangulate();
    let dual = ph2d_crossfield::Dual::build(&mesh);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let layout = ph2d_trace::trace_patches(&mesh, &dual, &field);
    let (cut, _) = crate::cut::cut_along_patches(&mesh, &layout);
    let (combed, _) = crate::comb::comb_patches(&mesh, &layout, &cut);
    let (map, _) = crate::weld_solve_driver::solve_welded(&mesh, &cut, &combed, 0.2, 6);
    let (w, _) = crate::weld::weld(&cut, &combed);
    for eq in super::arc_equations(&cut, &w, &map) {
        for (v, ax, k) in eq.terms {
            assert!(
                (k.abs() - 1.0).abs() < 1e-6,
                "coeficiente {k} em {v:?}[{ax}] — a eliminacao deixaria de ser inteira"
            );
        }
    }
}

/// ⭐⭐⭐ **O DENOMINADOR DE UMA AMARRA NUNCA PODE FICAR ABAIXO DA CURVATURA EFECTIVA.**
///
/// ⛔⛔⛔ É o gate do defeito medido em 2026-08-27: a 1.ª redacção de
/// [`crate::weld_solve::WeldRelaxer::relax_tie`] dividia o gradiente por `Σ den[classe]`
/// — a curvatura de cada membro **em isolamento** —, e os cantos dos arcos **são** os
/// cones, que são incógnitas LIVRES do sistema dos fechos: mexer uma move todas as
/// dependentes a jusante. Nesta fixtura o denominador fingido é `8,1` contra uma
/// curvatura efectiva de `73,8` ⇒ **passo `9×` maior que o mínimo**, e um Gauss–Seidel com
/// `ω > 2` diverge. (Na `sphere_uv` o rácio da Hessiana chega a `81×`.)
///
/// ⚠️ **A barra é uma DESIGUALDADE, e não a igualdade que a 1.ª redacção deste gate
/// exigiu.** Medi-a e ela reprovou: o passo é realizado ao bit (`pedido` = `andado`) e o
/// gradiente ainda assim não vai a zero, porque *o numerador de Poisson de uma cópia
/// depende das VIZINHAS* — mover muitas cópias ao mesmo tempo tem termos cruzados
/// negativos que nenhuma destas Hessianas conta. ⇒ `H` é um **majorante** da curvatura
/// (aqui `1,46×`), o passo fica **curto**, e curto **converge**. *Errar para cima é lento;
/// errar para baixo é `inf`.*
///
/// ⚠️ **Vale igual para a [`crate::weld_solve::WeldRelaxer::relax_free`]**, cujo doc diz
/// «minimização exacta ao longo daquela coordenada»: ela move a livre **e** todas as
/// dependentes de uma vez, então herda o mesmo majorante. *A frase está optimista pela
/// mesma razão, e a propriedade que a salva é esta desigualdade, não a exactidão.*
#[test]
fn the_tie_denominator_never_falls_below_the_effective_curvature() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(16, 24, 1.0);
    mesh.triangulate();
    let dual = ph2d_crossfield::Dual::build(&mesh);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let layout = ph2d_trace::trace_patches(&mesh, &dual, &field);
    let (cut, _) = crate::cut::cut_along_patches(&mesh, &layout);
    let (combed, _) = crate::comb::comb_patches(&mesh, &layout, &cut);
    let h = 0.2;
    let (mut map, _) = crate::weld_solve_driver::solve_welded(&mesh, &cut, &combed, h, 4);
    let (w, _) = crate::weld::weld(&cut, &combed);
    let ties = super::build_arc_ties(&cut, &w, &map);

    let mut rep = crate::solve::SolveReport::default();
    let a = crate::solve::assemble(&mesh, &cut, &combed, h, &mut rep);
    let mut r = crate::weld_solve::WeldRelaxer::new(&a, &w, &cut, &combed);
    r.attach_ties(&ties);
    let groups = r.tie_counts().0;
    assert!(groups > 0, "a esfera tem de dar grupos de amarra");

    // ⚠️ **Duas passagens antes de medir.** A 1.ª ainda ENCAIXA o grupo na relação (o mapa
    // de partida não a satisfaz), e um encaixe não é um passo. *Medir a seguir ao encaixe
    // mediria o desalinhamento inicial, não a lei do denominador.*
    for _ in 0..2 {
        for g in 0..groups {
            r.relax_tie(&mut map, g);
        }
    }
    let (mut checked, mut worst) = (0usize, 0.0f32);
    for g in 0..groups {
        let Some((before, h_true, h_pretend)) = r.tie_normal(&map, g) else {
            continue;
        };
        let root = ties.group(g).map_or(0, |t| t.0) as usize;
        let (rc, rax) = (root / 2, root % 2);
        let root_before = r.read_class(&map, rc)[rax];
        r.relax_tie(&mut map, g);
        let moved = r.read_class(&map, rc)[rax] - root_before;
        let after = r.tie_normal(&map, g).map_or(0.0, |t| t.0);
        if moved.abs() < 1.0e-9 || before.abs() < 1.0e-6 {
            continue;
        }
        // A curvatura EFECTIVA: quanto o gradiente de facto caiu por unidade andada.
        let h_eff = (before - after) / moved;
        if h_eff <= 0.0 {
            continue;
        }
        // ⛔⛔⛔ **O denominador que a [`relax_tie`] DE FACTO dividiu** — derivado do que
        // ela andou (`andado = gradiente / denominador`), nunca do que a
        // [`tie_normal`] devolve. ⚠️ *A 1.ª redacção deste gate lia o `h_true` do
        // relatório e a mutação SOBREVIVEU: ela media a Hessiana calculada, e o defeito
        // está em QUAL das duas o passo usa.*
        let den_used = before / moved;
        assert!(
            den_used >= h_eff * 0.99,
            "grupo {g}: o denominador USADO {den_used:e} ficou ABAIXO da curvatura \
             efectiva {h_eff:e} — o passo sobre-relaxa e a varredura diverge \
             (H {h_true:e}, fingida {h_pretend:e})"
        );
        // ⭐ E o controlo: o denominador FINGIDO viola-a, senão este gate nao prova nada.
        if h_pretend > 0.0 {
            worst = worst.max(h_eff / h_pretend);
        }
        checked += 1;
    }
    assert!(
        checked > 0,
        "nenhum grupo mediu: a fixtura nao contem o fenomeno"
    );
    assert!(
        worst > 2.0,
        "o denominador fingido nunca sobre-relaxou nesta fixtura (pior {worst:.2}x) — \
         o gate passaria com o defeito de volta"
    );
}

/// ⭐⭐ **A FOLGA DE POSTO DE `solve2` É RELATIVA À ESCALA DA MATRIZ.**
///
/// ⚠️ **A hipótese que motivou esta guarda foi REFUTADA** — trocá-la não moveu o `inf` da
/// `sphere_uv` (mesma ronda `6134`, mesmos `3119` não-finitos), e o controlo com as
/// amarras desligadas ficou **byte-idêntico**. Ela FICA pela razão que sobrevive: um
/// limiar **absoluto** (`1,0e-12`) sobre um sistema normal cuja escala é `Σ den·|J|²` não
/// tem significado — e é no ramo `1-D` (o que congelar um eixo escolhe) que ele morde,
/// porque ali o divisor deixa de ser o determinante e passa a ser `h[k][k]` sozinho.
#[test]
fn the_rank_slack_is_relative_to_the_matrix_scale() {
    // Uma matriz bem condicionada na escala dela, com a coluna 1 numericamente nula.
    let h = [[1.0e6, 0.0], [0.0, 1.0e-9]];
    let g = [1.0, 1.0];
    assert!(
        crate::weld_solve_driver::solve2_pub(h, g, [true, false]).is_none(),
        "a coluna nula passou a guarda: o limiar ainda e' absoluto"
    );
    // E a mesma coluna, na escala DELA, é perfeitamente resolúvel.
    let h = [[1.0e-9, 0.0], [0.0, 1.0e-9]];
    assert!(
        crate::weld_solve_driver::solve2_pub(h, g, [true, false]).is_some(),
        "uma matriz pequena mas sa' foi recusada: a guarda deixou de ser relativa"
    );
}

/// ⭐⭐⭐ **NENHUM MEMBRO DE UM GRUPO AMARRADO PODE SER ESCRITO PELA `relax_class` — A
/// RAIZ INCLUÍDA.**
///
/// ⛔⛔⛔ É o gate da causa medida em 2026-08-27 para o `NaN` da `sphere_uv`. Os membros
/// não-raiz saem da [`crate::weld_solve::WeldRelaxer::relax_class`] por `driven`, e os que
/// são incógnitas LIVRES por `freeze_free`. ⚠️ *Uma raiz que seja classe **simples** não é
/// nem uma coisa nem outra* — e a `relax_class` continuava a escrevê-la no eixo amarrado,
/// com o denominador da classe **sozinha**, enquanto a `relax_tie` a escrevia com o do
/// grupo. **Duas leis sobre o mesmo escalar.**
///
/// ⭐ A contagem CASA com o sintoma: `6` raízes simples na `esfera-fina`, `6` pregos com
/// passo não-finito. Marcada a raiz, o contínuo passa de `3 119` não-finitos a **`0`**, a
/// escada de `0 / 28` a **`29 / 0`**, e as visitas de `580 029` a `103 320`.
#[test]
fn no_tied_scalar_is_also_written_by_the_class_relaxation() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(16, 24, 1.0);
    mesh.triangulate();
    let dual = ph2d_crossfield::Dual::build(&mesh);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let layout = ph2d_trace::trace_patches(&mesh, &dual, &field);
    let (cut, _) = crate::cut::cut_along_patches(&mesh, &layout);
    let (combed, _) = crate::comb::comb_patches(&mesh, &layout, &cut);
    let h = 0.2;
    let (mut map, _) = crate::weld_solve_driver::solve_welded(&mesh, &cut, &combed, h, 4);
    let (w, _) = crate::weld::weld(&cut, &combed);
    let ties = super::build_arc_ties(&cut, &w, &map);

    let mut rep = crate::solve::SolveReport::default();
    let a = crate::solve::assemble(&mesh, &cut, &combed, h, &mut rep);
    let mut r = crate::weld_solve::WeldRelaxer::new(&a, &w, &cut, &combed);
    r.attach_ties(&ties);
    let groups = r.tie_counts().0;
    assert!(groups > 0, "a esfera tem de dar grupos de amarra");
    // ⚠️ **Sem raízes simples este gate seria vacuoso** — ele só distingue as duas
    // redacções na população em que `freeze_free` não chega.
    assert!(
        r.plain_roots() > 0,
        "a fixtura nao contem uma raiz de classe simples: o gate nao prova nada"
    );

    for g in 0..groups {
        let Some((_, members)) = ties.group(g) else {
            continue;
        };
        for &x in members {
            let (c, ax) = (x as usize / 2, x as usize % 2);
            let before = r.read_class(&map, c)[ax];
            r.relax_class(&mut map, c);
            let after = r.read_class(&map, c)[ax];
            assert!(
                (after - before).abs() <= 1.0e-9,
                "grupo {g}, escalar {x} (classe {c}, eixo {ax}): a relax_class moveu-o \
                 de {before:e} para {after:e} — e' a segunda lei sobre um escalar amarrado"
            );
        }
    }
}
