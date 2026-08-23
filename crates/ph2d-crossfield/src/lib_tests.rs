//! **OS GATES DO CAMPO CRUZADO** — a invariante topológica é o oráculo.

use ph2d_mesh::{Mesh, shapes};

use super::{
    Dual, cycle_count, energy, singularities, solve_alternating, solve_miq, to_vertex_dirs,
    vertex_index,
};

fn euler(mesh: &Mesh) -> i64 {
    use std::collections::BTreeSet;
    let mut e: BTreeSet<(u32, u32)> = BTreeSet::new();
    for f in mesh.faces() {
        let v = f.verts();
        for i in 0..v.len() {
            let (a, b) = (v[i], v[(i + 1) % v.len()]);
            e.insert(if a < b { (a, b) } else { (b, a) });
        }
    }
    mesh.vert_count() as i64 - e.len() as i64 + mesh.faces().len() as i64
}

fn tri(mut m: Mesh) -> Mesh {
    m.triangulate();
    m
}

/// ⭐ **POINCARÉ–HOPF: `Σ índice = 4·χ`.**
///
/// ⚠️ **É a única asserção desta folha que não depende de nada que eu escrevi.**
/// Ela é topologia: vale para qualquer campo 4-RoSy, em qualquer malha, com
/// qualquer solver. Um campo que a viole está errado — e nenhuma inspeção visual
/// diria isso, porque um campo errado ainda *parece* um campo.
///
/// ⚠️ **E ele passou dois meses verde sobre uma fórmula errada** (corrigido em
/// 2026-08-21). Faltava o defeito angular `K_v`, e as quatro fixturas acima são
/// todas **bem distribuídas** — nelas `K_v ≈ 4π/N` é minúsculo e o erro passa por
/// ruído numérico. Numa malha com triângulos de tamanhos muito diferentes a soma
/// saía **`−147`** onde a topologia exige `+8`.
/// ⇒ As duas fixturas ⭐ de baixo são as que **contêm o fenómeno**, e sem elas
/// este gate continua a ser verde sobre qualquer coisa.
#[test]
fn the_index_sum_is_four_times_the_euler_characteristic() {
    for (name, mesh) in [
        ("esfera 24x36", tri(shapes::uv_sphere(24, 36, 1.0))),
        ("esfera 48x64", tri(shapes::uv_sphere(48, 64, 1.0))),
        ("toro", tri(shapes::torus(64, 32, 1.0, 0.35))),
        (
            "cubo subdividido",
            tri(shapes::sphere_with_triangles(4000, 1.0)),
        ),
        // ⭐ Distribuição torta (jitter tangencial, forma exacta) e forma rugosa —
        // as duas em que `K_v` deixa de ser desprezável.
        (
            "esfera SACUDIDA",
            tri(shapes::uv_sphere_shuffled(32, 48, 1.0)),
        ),
        (
            "esfera RUIDOSA",
            tri(shapes::uv_sphere_noisy(32, 48, 1.0, 0.02)),
        ),
    ] {
        let dual = Dual::build(&mesh);
        let (field, report) = solve_miq(&dual);
        let (idx, ir) = crate::vertex_index_with_report(&mesh, &dual, &field);
        let count = idx.iter().filter(|k| **k != 0).count();
        let sum: i32 = idx.iter().sum();
        let want = 4 * euler(&mesh);
        eprintln!(
            "[xfield] {name}: {count} singularidades, soma {sum} (esperado {want}), \
             {} resolucoes, {} inteiros livres, energia {:.4}, pior residuo {:.4}",
            report.solves,
            report.free_integers,
            energy(&dual, &field),
            ir.worst_residual
        );
        assert_eq!(
            i64::from(sum),
            want,
            "{name}: a soma dos indices e' {sum} e a topologia exige {want} -- o campo nao fecha"
        );
        // ⚠️ **A soma sozinha não prova a FÓRMULA**, e é por isso que esta segunda
        // asserção existe. Ela pode fechar por sorte quando os arredondamentos
        // errados se cancelam; o resíduo diz se cada vértice, individualmente,
        // caiu num inteiro. `0,5` é um empate — a régua a decidir por sorteio.
        assert!(
            ir.worst_residual < 0.01,
            "{name}: o pior residuo do arredondamento e' {:.4} -- o indice nao esta' a cair num \
             inteiro, e a soma acima pode ter fechado por cancelamento ({} vertices ambiguos)",
            ir.worst_residual,
            ir.ambiguous
        );
        // E a régua não pode ter desistido de nenhum vértice: cada desistência
        // entra na soma como se fosse um vértice regular.
        assert_eq!(
            (ir.gave_up(), ir.key_collisions),
            (0, 0),
            "{name}: a regua desistiu de {} vertices e teve {} colisoes de chave",
            ir.gave_up(),
            ir.key_collisions
        );
    }
}

/// ⭐ **NUM PLANO NÃO HÁ SINGULARIDADE NENHUMA** — o controle.
///
/// ⚠️ **É a metade que a invariante acima não cobre.** Numa esfera `Σ = 8` tanto
/// com oito singularidades quanto com duzentas em pares que se cancelam; num
/// retalho plano o número certo é **zero**, e ali a contagem tem de o dizer.
#[test]
fn a_flat_patch_has_no_singularities() {
    let mesh = flat_grid(24);
    let dual = Dual::build(&mesh);
    let (field, _) = solve_miq(&dual);
    let idx = vertex_index(&mesh, &dual, &field);
    let inner: usize = idx.iter().filter(|k| **k != 0).count();
    eprintln!("[xfield] plano 24x24: {inner} singularidades interiores");
    assert_eq!(
        inner, 0,
        "{inner} singularidades num retalho PLANO -- num plano a cruz nao tem por que virar"
    );
}

/// ⭐ **DETERMINÍSTICO** (HR-5) — duas corridas, o mesmo campo ao bit.
#[test]
fn the_field_is_bit_reproducible() {
    let mesh = tri(shapes::torus(48, 24, 1.0, 0.35));
    let dual = Dual::build(&mesh);
    let (a, _) = solve_miq(&dual);
    let (b, _) = solve_miq(&dual);
    assert_eq!(a, b, "duas corridas devolveram campos diferentes");
}

/// ⭐ **O SOLVER CONVERGE** — os saltos de período param de mudar.
#[test]
fn the_alternation_converges() {
    for (name, mesh) in [
        ("esfera", tri(shapes::uv_sphere(24, 36, 1.0))),
        ("toro", tri(shapes::torus(48, 24, 1.0, 0.35))),
    ] {
        let dual = Dual::build(&mesh);
        let (_, r) = solve_miq(&dual);
        eprintln!(
            "[xfield] {name}: {} resolucoes para {} inteiros livres ({} ciclos), {} periodos != 0",
            r.solves,
            r.free_integers,
            cycle_count(&dual),
            r.nonzero_periods
        );
        assert_eq!(
            r.free_integers,
            cycle_count(&dual),
            "{name}: o gauge da arvore deixou {} inteiros livres e o grafo tem {} ciclos -- se \
             sobram mais, a arvore nao e' geradora; se menos, ela comeu ciclo",
            r.free_integers,
            cycle_count(&dual)
        );
    }
}

/// ⭐ **A CONVERSÃO PARA POR-VÉRTICE devolve tangentes unitárias** — a ponte para
/// o extrator que já existe.
#[test]
fn the_vertex_conversion_stays_tangent_and_unit() {
    let mesh = tri(shapes::uv_sphere(24, 36, 1.0));
    let dual = Dual::build(&mesh);
    let (field, _) = solve_miq(&dual);
    let dirs = to_vertex_dirs(&mesh, &dual, &field);
    assert_eq!(
        dirs.len(),
        mesh.vert_count(),
        "a conversao nao cobre a malha"
    );
    let normals = mesh.normals();
    for (v, d) in dirs.iter().enumerate() {
        let len = super::dot(*d, *d).sqrt();
        assert!(
            (len - 1.0).abs() < 1.0e-3,
            "o vertice {v} recebeu uma direcao de comprimento {len:.4}"
        );
        let off = super::dot(*d, normals[v]).abs();
        assert!(
            off < 1.0e-3,
            "o vertice {v} recebeu uma direcao {off:.4} fora do plano tangente"
        );
    }
}

/// Um retalho PLANO `n×n` — a fixtura do controle.
///
/// ⚠️ **Construída aqui e não em `shapes`**: a engine só tem sólidos fechados e
/// as três malhas abertas patológicas (`open_disc`, `pillow`, …). Um plano
/// saudável é fixtura DESTA folha, e pô-lo em `shapes` seria acrescentar à
/// superfície pública da malha por conveniência de um teste.
fn flat_grid(n: usize) -> Mesh {
    let mut pos = Vec::with_capacity((n + 1) * (n + 1));
    for j in 0..=n {
        for i in 0..=n {
            let (x, y) = (i as f32 / n as f32 - 0.5, j as f32 / n as f32 - 0.5);
            pos.push([x * 2.0, y * 2.0, 0.0]);
        }
    }
    let mut faces = Vec::with_capacity(n * n * 2);
    let at = |i: usize, j: usize| (j * (n + 1) + i) as u32;
    for j in 0..n {
        for i in 0..n {
            faces.push(ph2d_mesh::Face::tri(
                at(i, j),
                at(i + 1, j),
                at(i + 1, j + 1),
            ));
            faces.push(ph2d_mesh::Face::tri(
                at(i, j),
                at(i + 1, j + 1),
                at(i, j + 1),
            ));
        }
    }
    Mesh::from_parts(pos, faces).expect("o retalho plano e' bem formado")
}

/// ⛔ **A ALTERNÂNCIA INGÊNUA, MEDIDA CONTRA O MIQ** — a recusa, executável.
///
/// ⚠️ **Ela PASSA no gate topológico e é inútil**, e é por isso que este gate
/// existe: `Σ índice = 8` nas duas, e uma delas põe as oito voltas em **dois**
/// pontos de índice `+4` — um ponto onde a cruz dá uma volta inteira, que
/// nenhuma grade de quads contorna. *A invariante prova que o campo fecha, não
/// que ele presta.*
///
/// ⛔ Se um dia a alternância empatar com o MIQ, é o MIQ que regrediu.
#[test]
fn the_naive_alternation_is_measurably_worse() {
    for (name, mesh) in [
        ("esfera 24x36", tri(shapes::uv_sphere(24, 36, 1.0))),
        (
            "cubo subdividido",
            tri(shapes::sphere_with_triangles(4000, 1.0)),
        ),
    ] {
        let dual = Dual::build(&mesh);
        let (alt, rounds) = solve_alternating(&dual, 40);
        let (miq, _) = solve_miq(&dual);
        let (n_alt, s_alt) = singularities(&mesh, &dual, &alt);
        let (n_miq, s_miq) = singularities(&mesh, &dual, &miq);
        eprintln!(
            "[xfield] {name}: alternancia {n_alt} sing (soma {s_alt}, {rounds} rodadas, \
             energia {:.3}) | MIQ {n_miq} sing (soma {s_miq}, energia {:.3})",
            energy(&dual, &alt),
            energy(&dual, &miq)
        );
        assert_eq!(
            s_alt, s_miq,
            "{name}: os dois campos fecham a MESMA topologia"
        );
        assert!(
            energy(&dual, &miq) < energy(&dual, &alt),
            "{name}: o MIQ deixou de ganhar em energia da alternancia -- se isso e' verdade, o \
             gauge da arvore parou de fazer efeito"
        );
        assert!(
            n_miq > n_alt,
            "{name}: a alternancia tem {n_alt} singularidades e o MIQ {n_miq} -- MAIS \
             singularidades de indice 1 e' MELHOR que poucas de indice 4, e este gate existe \
             porque a contagem sozinha enganaria na direcao contraria"
        );
    }
}

/// ⭐⭐⭐ **O CAMPO SOBREVIVE À IDA E VOLTA POR DIREÇÕES CRUAS.**
///
/// ⛔ **Sem este gate, o [`CrossField::from_directions`] é uma régua por validar** — e
/// ela existe precisamente para medir um campo que **não é nosso** (o `*_rem.rosy` do
/// oráculo GPL, fora da árvore), onde não há segunda opinião nenhuma. *Uma régua que
/// só se usa onde não há controlo tem de ser validada onde há.*
///
/// ⚠️ **O `theta` NÃO é comparado, de propósito.** Uma direção grava um dos quatro
/// braços da cruz, logo o `theta` reconstruído difere do original por um múltiplo de
/// 90° — e é o `period` da aresta ao lado que absorve a diferença. *O que tem de
/// sobreviver é o que as réguas lêem*: o índice de cada vértice, e com ele a contagem
/// e a soma das singularidades.
#[test]
fn a_field_survives_the_round_trip_through_raw_directions() {
    for (name, mesh) in [
        ("esfera", tri(shapes::uv_sphere(16, 24, 1.0))),
        ("toro", tri(shapes::torus(24, 12, 1.0, 0.35))),
        ("cubo", tri(shapes::cube(1.0))),
    ] {
        let dual = Dual::build(&mesh);
        let (field, _) = solve_miq(&dual);
        let dirs: Vec<[f32; 3]> = (0..field.len()).map(|f| field.direction(&dual, f)).collect();
        let back = super::CrossField::from_directions(&dual, &dirs)
            .unwrap_or_else(|| panic!("{name}: a reconstrucao recusou o proprio campo"));

        let a = vertex_index(&mesh, &dual, &field);
        let b = vertex_index(&mesh, &dual, &back);
        assert_eq!(
            a, b,
            "{name}: o indice de algum vertice mudou na ida e volta -- a reconstrucao nao \
             preserva o que as reguas leem, e o numero do oraculo medido com ela nao vale"
        );
        assert_eq!(
            singularities(&mesh, &dual, &field),
            singularities(&mesh, &dual, &back),
            "{name}: a contagem/soma de singularidades mudou na ida e volta"
        );
        // ⚠️ **O controlo NEGATIVO**: um campo de outra malha tem de ser RECUSADO, e
        // não medido. *Sem ele, `from_directions` aceitaria qualquer vector e daria
        // índices plausíveis e errados* — que é exactamente a assinatura do defeito de
        // 2026-08-21.
        assert!(
            super::CrossField::from_directions(&dual, &dirs[..dirs.len() - 1]).is_none(),
            "{name}: uma contagem de direcoes ERRADA foi aceite"
        );
    }
}
