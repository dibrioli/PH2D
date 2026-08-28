//! ⛔⛔ **O VETO DA CADEIA** — ela corre, e só troca a malha se a troca for uma melhoria.
//!
//! ⚠️ **A fixtura é o ponto todo deste gate.** As três peças pedem **três vereditos diferentes**, e
//! uma fixtura de uma peça só não distingue *"a regra funciona"* de *"a regra devolve sempre a mesma
//! coisa"*.

use ph2d_mesh::{Face, Mesh};
use ph2d_quadchain::{Verdict, quads_or_keep};

/// Uma esfera em quads, pela parametrização UV — o caso **orgânico**, onde a cadeia ganha.
fn uv_sphere(nu: usize, nv: usize, r: f32) -> Mesh {
    let mut pos = Vec::new();
    for j in 0..=nv {
        let v = std::f32::consts::PI * j as f32 / nv as f32;
        for i in 0..nu {
            let u = std::f32::consts::TAU * i as f32 / nu as f32;
            pos.push([r * v.sin() * u.cos(), r * v.cos(), r * v.sin() * u.sin()]);
        }
    }
    let idx = |i: usize, j: usize| (j * nu + i % nu) as u32;
    let mut faces = Vec::new();
    for j in 0..nv {
        for i in 0..nu {
            faces.push(Face([
                idx(i, j),
                idx(i + 1, j),
                idx(i + 1, j + 1),
                idx(i, j + 1),
            ]));
        }
    }
    Mesh::from_parts(pos, faces).expect("a esfera")
}

/// Um cubo em quads, subdividido — o caso **duro**, onde a grade já é a resposta certa.
fn subdivided_cube(n: usize, half: f32) -> Mesh {
    let mut pos: Vec<[f32; 3]> = Vec::new();
    let mut faces = Vec::new();
    let mut index = std::collections::BTreeMap::new();
    let key = |p: [f32; 3]| {
        (
            (p[0] * 1.0e5) as i64,
            (p[1] * 1.0e5) as i64,
            (p[2] * 1.0e5) as i64,
        )
    };
    let mut vid = |p: [f32; 3], pos: &mut Vec<[f32; 3]>| -> u32 {
        *index.entry(key(p)).or_insert_with(|| {
            pos.push(p);
            (pos.len() - 1) as u32
        })
    };
    for axis in 0..3usize {
        for side in [-1.0f32, 1.0] {
            for a in 0..n {
                for b in 0..n {
                    let mut quad = [0u32; 4];
                    for (k, (da, db)) in [(0, 0), (1, 0), (1, 1), (0, 1)].into_iter().enumerate() {
                        let (ua, ub) = (
                            -half + 2.0 * half * (a + da) as f32 / n as f32,
                            -half + 2.0 * half * (b + db) as f32 / n as f32,
                        );
                        let mut p = [0.0f32; 3];
                        p[axis] = side * half;
                        p[(axis + 1) % 3] = ua;
                        p[(axis + 2) % 3] = ub;
                        quad[k] = vid(p, &mut pos);
                    }
                    faces.push(Face(quad));
                }
            }
        }
    }
    Mesh::from_parts(pos, faces).expect("o cubo")
}

/// ⛔⛔ **UM ESTOURO A JUSANTE NÃO DERRUBA QUEM PEDIU UMA MELHORIA** (W61b).
///
/// ⛔ **Medido:** um cubo subdividido — **fechado, manifold, 100 % quads** — faz o `ph2d-gridmap`
/// entrar em `index out of bounds: the len is 129 but the index is 157` (`solve.rs:336`). ⚠️ Não é
/// uma pré-condição que se possa conferir à porta: a malha satisfaz tudo o que se sabe exigir.
///
/// ⭐ Esta porta oferece uma **melhoria opcional**, e o veto já diz *"fica com a entrada a menos que
/// a saída seja melhor"* — um estouro é só mais uma forma de não ser melhor. ⛔ **Isto não é a
/// cura**: o defeito é do `ph2d-gridmap`, e a linha dele está viva sobre aquele arquivo.
#[test]
fn a_panic_downstream_does_not_take_down_the_caller() {
    let cube = subdivided_cube(12, 0.35);
    let target = ph2d_remesh_iso::target_edge(&cube, ph2d_remesh_iso::ALPHA);
    // ⚠️ O `panic` a jusante imprime o rasto dele; o que este gate afirma é que ele **volta como
    // veredito**.
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let (kept, v) = quads_or_keep(&cube, target);
    std::panic::set_hook(hook);
    assert!(
        !matches!(v, Verdict::Adopted(_)),
        "a cadeia foi adoptada numa peça de faces PLANAS — {v:?}"
    );
    assert_eq!(
        kept.face_count(),
        cube.face_count(),
        "a malha de entrada não voltou intacta depois do estouro"
    );
}

/// ⭐⭐ **O VETO DURO É O BURACO, e ele vem primeiro.**
///
/// ⚠️ Nenhum ganho de forma paga uma peça aberta — e é por isso que este ramo é testado **sozinho**:
/// numa fixtura só, «rejeitado por buraco» e «rejeitado por não melhorar» leem-se igual.
#[test]
fn a_hole_vetoes_before_any_shape_gain_is_considered() {
    // Uma calote: a esfera **sem** as últimas fileiras — ela já entra com bordo.
    let full = uv_sphere(40, 24, 0.4);
    let keep: Vec<Face> = full
        .faces()
        .iter()
        .copied()
        .take(full.face_count() * 3 / 4)
        .collect();
    let open = Mesh::from_parts(full.positions().to_vec(), keep).expect("a calote");
    let before = ph2d_quadchain::boundary_edges(&open);
    assert!(
        before > 0,
        "a fixtura não tem bordo — ela não contém o caso"
    );
    let target = ph2d_remesh_iso::target_edge(&open, ph2d_remesh_iso::ALPHA);
    let (kept, v) = quads_or_keep(&open, target);
    if let Verdict::Adopted(r) = &v {
        assert!(
            r.boundary_edges <= before,
            "a cadeia foi adoptada tendo AUMENTADO o bordo de {before} para {} — o veto duro não \
             mordeu",
            r.boundary_edges
        );
    } else {
        // Rejeitada ou sem ganho: a malha tem de voltar intacta.
        assert_eq!(kept.face_count(), open.face_count());
    }
}

/// ⭐⭐ **A FASE ZERO REMALHA PARA O ALVO QUE LHE DERAM** — e este gate existe porque ela não o
/// fazia.
///
/// Até 2026-08-25 ela passava `ph2d_remesh_iso::ALPHA` **fixo** enquanto o resto da cadeia
/// quantizava para o `target_edge` do argumento. ⚠️ Com o único chamador de então os dois números
/// coincidiam **por acidente** — ele passava exactamente `target_edge(mesh, ALPHA)` —, e por isso
/// nenhum gate o via: *um parâmetro que metade da função ignora só mente para o SEGUNDO chamador.*
///
/// ⭐ A régua é a **área**: com o dobro do lado cabem ~4× menos triângulos na mesma superfície.
///
/// ⚠️ **Ela mede a FASE ZERO e não a saída da cadeia, e isso é uma correcção.** A primeira redacção
/// media os quads finais — e reprovava a **estourar dentro do `ph2d-gridmap`**
/// (`solve.rs:336`, *"index out of bounds: the len is 74 but the index is 130"*, com um alvo grosso
/// sobre esta mesma esfera). ⛔ Uma régua que atravessa um estouro de outra crate não mede a lei que
/// se quer: mede a travessia inteira. Defeito nomeado no handoff, com este reprodutor.
#[test]
fn the_phase_zero_remeshes_to_the_target_it_was_given() {
    let mesh = uv_sphere(48, 32, 0.45);
    let base = ph2d_remesh_iso::target_edge(&mesh, ph2d_remesh_iso::ALPHA);
    let fine = ph2d_quadchain::phase_zero(&mesh, base);
    let coarse = ph2d_quadchain::phase_zero(&mesh, base * 2.0);
    let (nf, nc) = (fine.faces().len(), coarse.faces().len());
    assert!(
        nf > 2 * nc,
        "dobrar o alvo tem de dar bem menos triângulos: {nf} contra {nc} — se forem parecidos, a \
         fase zero está a remalhar para uma escala própria e a ignorar o argumento"
    );
    // ⚠️ **A metade JUSTA**: sem ela, uma fase zero que devolvesse a malha vazia para todo alvo
    // grosso passaria. As duas saídas têm de continuar a ser malhas da mesma peça.
    assert!(
        nc > 100,
        "o alvo grosso ainda tem de dar uma peça, não um resto: {nc} triângulos"
    );
    for m in [&fine, &coarse] {
        let r = m
            .positions()
            .iter()
            .map(|p| (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt())
            .fold(0.0f32, f32::max);
        assert!(
            (r - 0.45).abs() < 0.05,
            "a remalha tem de ficar sobre a esfera de raio 0,45; o vértice mais longe está a {r}"
        );
    }
}

/// ⭐⭐⭐ **QUANDO A CADEIA PERDE, VOLTA A MALHA QUE O ARTISTA PEDIU — não a que a cadeia comeu.**
///
/// ⚠️ As duas não são a mesma desde 2026-08-25: a cadeia é alimentada por uma grade grossa (medido:
/// a fina custa 107× e dá a mesma resposta), enquanto quem sai no arquivo é a malha do nível que o
/// artista escolheu. ⛔ **A mutação que este gate mata é devolver o `feed`** — ela compila, passa
/// todo o resto, e entrega uma malha 16× mais grossa a quem pediu detalhe. *Um veto que devolve a
/// entrada errada é pior que um veto que não existe: ele parece ter protegido alguma coisa.*
#[test]
fn when_the_chain_loses_the_mesh_the_artist_asked_for_comes_back() {
    // O cubo é o caso onde a cadeia perde por medição — a grade dual já é a resposta certa.
    let feed = subdivided_cube(6, 0.4);
    let keep = subdivided_cube(24, 0.4);
    assert!(
        keep.faces().len() > feed.faces().len() * 4,
        "a fixtura só prova alguma coisa se as duas malhas forem distinguíveis: {} contra {}",
        keep.faces().len(),
        feed.faces().len()
    );
    let target = ph2d_remesh_iso::target_edge(&feed, ph2d_remesh_iso::ALPHA);
    let (out, verdict) = ph2d_quadchain::quads_or_keep_from(&feed, &keep, target);
    assert!(
        !matches!(verdict, Verdict::Adopted(_)),
        "o cubo é o caso em que a cadeia perde; se ela passou a ganhar, esta fixtura deixou de \
         medir o que mede: {verdict:?}"
    );
    assert_eq!(
        out.faces().len(),
        keep.faces().len(),
        "com a cadeia recusada tem de voltar a malha do NÍVEL ({} faces), e voltou uma de {}",
        keep.faces().len(),
        out.faces().len()
    );
}

/// ⭐⭐⭐ **O ACABAMENTO NÃO PODE MUDAR O CENSO DE ARESTAS** — é isto que torna legítimo
/// decidir o veto de topologia **antes** de o pagar.
///
/// ⛔⛔ **Sem esta propriedade a reordenação seria uma aposta.** Medido em 2026-08-28: no
/// cubo subdividido — o caso em que a cadeia perde por medição — a saída abre arestas de
/// bordo e o veto recusa; com o acabamento à frente, ele corria até ao tecto **duas vezes**
/// (a lei alinhada e a cega) sobre uma malha que ninguém ia usar.
///
/// ⚠️ **A fixtura tem de ser uma peça que a cadeia ACEITA** — num cubo o veto dispara antes
/// e o gate mediria o caminho que não interessa.
#[test]
fn the_finishing_cannot_change_the_edge_census() {
    let piece = ph2d_mesh::shapes::uv_sphere(24, 36, 0.45);
    let target = ph2d_remesh_iso::target_edge(&piece, ph2d_remesh_iso::ALPHA);
    let Ok((raw, _)) = ph2d_quadchain::quads_from_mesh_raw(&piece, target) else {
        panic!("a cadeia crua recusou a esfera — a fixtura deixou de medir o que mede");
    };
    let Ok((done, _)) = ph2d_quadchain::quads_from_mesh(&piece, target) else {
        panic!("a cadeia inteira recusou a esfera");
    };
    assert_eq!(
        (raw.vert_count(), raw.face_count()),
        (done.vert_count(), done.face_count()),
        "o acabamento mudou a CONTAGEM — ele so' pode mover vertices"
    );
    let census = |m: &ph2d_mesh::Mesh| {
        let mut count: std::collections::BTreeMap<(u32, u32), usize> =
            std::collections::BTreeMap::new();
        for f in m.faces() {
            let v = f.verts();
            for k in 0..v.len() {
                let (a, b) = (v[k], v[(k + 1) % v.len()]);
                *count.entry((a.min(b), a.max(b))).or_default() += 1;
            }
        }
        (
            count.values().filter(|&&c| c == 1).count(),
            count.values().filter(|&&c| c > 2).count(),
        )
    };
    assert_eq!(
        census(&raw),
        census(&done),
        "o censo de arestas mudou com o acabamento — o veto de topologia NAO pode correr \
         antes dele"
    );
}
