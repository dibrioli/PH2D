//! ⭐⭐⭐ **GATE Nº7 da [`SPEC_restricoes_por_eliminacao.md`] §5** — *as restrições ficam
//! ESPARSAS, e a contagem de singularidades vai ao lado*.
//!
//! ⛔ **A espec é explícita sobre por que a segunda metade existe:** *«marcar feição a mais
//! aparece como singularidades a mais, nunca como uma feição feia»*. Um gate que só medisse
//! a percentagem de arestas marcadas aprovaria uma lei que marca pouco **e errado**.
//!
//! ⚠️ **Ele vive aqui, e não na `ph2d-mesh`, porque a lei tem de ser medida sobre a malha
//! que o pipeline alimenta** — a remalhada pelo F1. O mesmo motivo do `feature_sweep`.

use ph2d_crossfield::{Dual, energy, singularities, solve_miq};
use ph2d_mesh::{
    FEATURE_EDGE_MIN_COS, FeatureEdge, FeatureOptions, Mesh, feature_dirs, feature_edges, shapes,
};

/// ⭐⭐ **Uma esfera com UM vinco no equador** — quase toda lisa, e uma linha aguda.
///
/// ⛔⛔ **A 1.ª redacção usava um CILINDRO, e ele não serve para medir esparsidade:** a
/// parede inteira dele é uma região **parabólica** (uma curvatura nula, a outra não), então
/// `9,02 %` das arestas serem feição ali **está certo** — é a peça que é assim. ⚠️ *Uma
/// fixtura em que a resposta certa é «quase tudo» não consegue reprovar «marcou de mais».*
///
/// ⇒ Esta tem as duas metades ao mesmo tempo: uma superfície elíptica onde a lei tem de
/// **calar-se**, e uma linha onde ela tem de **falar**.
fn piece() -> Mesh {
    let mut mesh = shapes::uv_sphere(48, 72, 1.0);
    for p in mesh.positions_mut() {
        // Uma TENDA em `y = 0`: o raio cresce linearmente até ao equador e volta. A derivada
        // salta ali, que é o que faz um vinco — ⛔ uma gaussiana daria um ressalto macio, e a
        // lei da feição recusá-lo-ia com razão.
        let bump = 0.12 * (1.0 - (p[1].abs() / 0.10)).max(0.0);
        let k = 1.0 + bump;
        *p = [p[0] * k, p[1], p[2] * k];
    }
    mesh.rebuild();
    mesh.triangulate();
    ph2d_remesh_iso::remesh_isotropic(&mut mesh, ph2d_remesh_iso::ALPHA);
    mesh.triangulate();
    mesh
}

fn edge_mean(m: &Mesh) -> f32 {
    let pos = m.positions();
    let (mut s, mut n) = (0.0f32, 0usize);
    for f in m.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            s += d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
            n += 1;
        }
    }
    s / n.max(1) as f32
}

fn detect(mesh: &Mesh) -> (Vec<FeatureEdge>, f64) {
    let (dirs, _) = feature_dirs(mesh, edge_mean(mesh), FeatureOptions::default());
    let (edges, rep) = feature_edges(mesh, &dirs, FEATURE_EDGE_MIN_COS);
    (edges, rep.sparsity_pct())
}

/// ⭐⭐ **A esparsidade E o alarme, na mesma corrida.**
#[test]
fn the_constraints_stay_sparse_and_do_not_plant_singularities() {
    let mesh = piece();
    let (edges, pct) = detect(&mesh);
    let free = Dual::build(&mesh);
    let (base, _) = solve_miq(&free);
    let base_sing = singularities(&mesh, &free, &base).0;

    let mut held = free.clone();
    let rep = held.constrain(&mesh, &edges);
    let (field, _) = solve_miq(&held);
    let sing = singularities(&mesh, &held, &field).0;

    eprintln!(
        "esfera com vinco: {} arestas de feicao ({pct:.2}% da peca) ⇒ {} faces fixas, \
         {} conflitos | singularidades {base_sing} sem restricao, {sing} com",
        edges.len(),
        rep.faces,
        rep.conflicts
    );
    assert!(
        !edges.is_empty(),
        "⛔ a fixtura tem de conter o fenomeno: o rebordo do cilindro e' um vinco"
    );
    assert!(
        pct < 5.0,
        "⛔ {pct:.2}% das arestas viraram restricao — a espec pede ESPARSO e conservador"
    );
    // ⚠️ A barra é **relativa ao campo livre**, e não um literal: a contagem certa depende da
    // peça, e só a comparação com a mesma peça sem restrição diz alguma coisa.
    assert!(
        sing <= base_sing * 2,
        "⛔ ALARME do gate no7: {base_sing} singularidades sem restricao e {sing} com — \
         marcar feicao a mais aparece exactamente assim"
    );
}

/// ⭐⭐⭐ **O CONTROLO QUE APANHOU O DEFEITO — e por isso é gate, não sonda.**
///
/// Restringe as mesmas faces à direcção que o campo **livre** já tinha ali. A resposta certa
/// é conhecida: *forçar alguém a fazer o que ele já ia fazer não pode mudar nada.*
///
/// # ⚠️ A régua é a ENERGIA, e a contagem de singularidades **não serve** aqui
///
/// ⛔ A 1.ª redacção exigia a **mesma contagem**, e essa barra é falsa: a restrição acrescenta
/// inteiros livres, o arredondamento é **guloso**, e um caminho diferente pelo mesmo problema
/// muda a contagem em `±2` sem nada estar errado (medido nesta fixtura: `14` contra `16`).
///
/// ⭐⭐ **A energia é o próprio objectivo que o solver minimiza**, e ela separa as duas coisas
/// com folga de uma ordem de grandeza: uma re-trajectória legítima custa **`+1,3 %`**
/// (`8,96` ⇒ `9,08`, peça do artista), e o defeito custava **`15×`** (`8,96` ⇒ `137,54`).
///
/// ⛔⛔ **Ele nasceu vermelho e nomeou duas causas** (2026-08-25) — a árvore geradora dava
/// pai a faces restringidas (que não podem absorver um salto de período), e o gauge do `θ`
/// era escrito duas vezes na mesma componente. Com as duas de pé, este controlo dava **111**
/// onde tem de dar o número do campo livre.
///
/// ⚠️ **Nenhum dos outros gates o via:** o campo obedecia à feição (gate nº6 media `0,00°`),
/// as arestas eram esparsas, os conflitos eram zero. *Uma restrição pode estar perfeitamente
/// honrada e o resto do campo estar destruído.*
#[test]
fn pinning_the_field_to_what_it_already_did_changes_nothing() {
    let mesh = piece();
    let (edges, _) = detect(&mesh);
    let free = Dual::build(&mesh);
    let (base, _) = solve_miq(&free);
    let base_sing = singularities(&mesh, &free, &base).0;

    // A direcção que o campo livre já tem nas faces de cada aresta marcada.
    //
    // ⛔⛔ **Só entram as arestas cujas DUAS faces já concordam a menos de 1°**, e a razão é
    // que a restrição é **por aresta**: ela escreve UM valor nas duas. ⚠️ *Onde as duas
    // discordam, «o que o campo já fazia» não é um número só — e o controlo deixaria de ter
    // resposta conhecida, que é a única coisa que o torna um controlo.* Medido: sem este
    // filtro ele dava `18` contra `16`, e a diferença era a discordância, não um defeito.
    let mut same: Vec<FeatureEdge> = Vec::new();
    for e in &edges {
        let mut who: Vec<usize> = Vec::new();
        for (fi, f) in mesh.faces().iter().enumerate() {
            let v = f.verts();
            if (0..v.len()).any(|k| {
                [
                    v[k].min(v[(k + 1) % v.len()]),
                    v[k].max(v[(k + 1) % v.len()]),
                ] == e.verts
            }) {
                who.push(fi);
            }
        }
        let [a, b] = who[..] else { continue };
        let (da, db) = (base.direction(&free, a), base.direction(&free, b));
        let c = (da[0].mul_add(db[0], da[1].mul_add(db[1], da[2] * db[2])))
            .abs()
            .clamp(0.0, 1.0);
        if c.acos() <= 1.0_f32.to_radians() {
            same.push(FeatureEdge {
                verts: e.verts,
                dir: da,
            });
        }
    }
    assert!(
        same.len() >= 4,
        "⛔ a fixtura nao deixou arestas com resposta conhecida: {} de {}",
        same.len(),
        edges.len()
    );

    let mut held = free.clone();
    let rep = held.constrain(&mesh, &same);
    let (field, _) = solve_miq(&held);
    let sing = singularities(&mesh, &held, &field).0;
    let (e0, e1) = (energy(&free, &base), energy(&held, &field));
    let ratio = e1 / e0.max(1.0e-12);
    eprintln!(
        "{} de {} arestas com resposta conhecida ⇒ {} faces fixas ao proprio campo: \
         energia {e0:.4} para {e1:.4} ({ratio:.3}x) · singularidades {base_sing} para {sing}",
        same.len(),
        edges.len(),
        rep.faces
    );
    assert!(
        ratio <= 1.25,
        "⛔ restringir o campo AO QUE ELE JA' FAZIA multiplicou a energia por {ratio:.3} — \
         o defeito esta' na ELIMINACAO, nunca na lei da feicao (o defeito de 2026-08-25 dava 15x)"
    );
    assert!(
        sing <= base_sing * 2,
        "⛔ a contagem foi de {base_sing} para {sing} sobre uma restricao que nao pede nada"
    );
}
