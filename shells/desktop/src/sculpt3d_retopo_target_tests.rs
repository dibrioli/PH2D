//! ⭐⭐ **OS GATES DO ALVO E DA GRADUAÇÃO** — irmão de [`super::tests`] por ASSUNTO: eles
//! provam o que o [`super::target`] faz (o campo de passo, o alisamento em log, a
//! convergência), e não o que o botão escolhe.
//!
//! ⛔ Nasceu do tecto de LOC do shell (HR-18, 600 — o irmão chegou a `659` em 2026-09-01), e
//! a fronteira é a mesma que já separa o produto: *o alvo tem um ficheiro, as réguas outro, a
//! escolha outro — e cada um leva os seus gates.*

/// ⭐⭐⭐ **GATE — a densidade SEGUE A FORMA, e a contagem não se mexe.**
///
/// ⛔ Report do artista (2026-08-28): *«as pontas finas, que deveriam ser relativamente mais
/// densas que as áreas lisas, têm menos densidade de faces e perdem detalhes»*. A régua é o
/// campo de passo que [`super::sizing_field`] entrega.
///
/// ⚠️ **As três metades do contrato, e a do meio é a que se esquece:** (1) com o knob a zero
/// o campo é **vazio** — a saída é a de sempre, e não «quase»; (2) com o knob a um o passo é
/// **menor onde a curvatura é maior**; (3) a **contagem prevista não muda**, senão o slider
/// que passou a pedir uma contagem volta a mentir.
#[test]
fn a_densidade_segue_a_curvatura_sem_mudar_a_contagem() {
    // ⚠️ Um toro: o tubo aperta e o buraco interior é chato. ⛔ Uma esfera tem curvatura
    // CONSTANTE — a fixtura não conteria o fenómeno.
    let work = ph2d_mesh::shapes::torus(64, 24, 1.0, 0.22);
    let target = ph2d_quadflow::edge_for_detail_by_count(&work, 0.5);

    // (1) O knob a zero é a AUSÊNCIA do campo, não um campo constante.
    assert!(
        super::sizing_field(&work, target, 0.0).is_empty(),
        "⛔ com `Follow Curvature` a zero o campo tem de ser VAZIO -- e' isso que faz o \
         passo ser o escalar de sempre"
    );

    let field = super::sizing_field(&work, target, 1.0);
    assert_eq!(field.len(), work.vert_count());

    // (2) Menor onde aperta: correlaciona o passo com a curvatura, por bandas.
    let curv = work.curvatures();
    let mut rows: Vec<(f32, f32)> = (0..work.vert_count())
        .map(|v| (curv[v].abs(), field[v]))
        .collect();
    rows.sort_by(|a, b| a.0.total_cmp(&b.0));
    let n = rows.len();
    let flat: f32 = rows[..n / 4].iter().map(|r| r.1).sum::<f32>() / (n / 4) as f32;
    let tight: f32 = rows[3 * n / 4..].iter().map(|r| r.1).sum::<f32>() / (n - 3 * n / 4) as f32;
    eprintln!(
        "[retopo] passo medio: chapado {flat:.5} · apertado {tight:.5} ({:.2}x) | alvo {target:.5}",
        flat / tight
    );
    assert!(
        tight < flat,
        "⛔ o passo tem de ser MENOR onde a curvatura e' maior (apertado {tight:.5}, \
         chapado {flat:.5})"
    );

    // (3) A contagem prevista não se mexe — a adaptação MOVE os quads, não os cria.
    let count = |h: &dyn Fn(usize) -> f32| -> f64 {
        let pos = work.positions();
        let mut acc = 0.0f64;
        for f in work.faces() {
            let v = f.verts();
            for k in 1..v.len() - 1 {
                let (a, b, c) = (
                    pos[v[0] as usize],
                    pos[v[k] as usize],
                    pos[v[k + 1] as usize],
                );
                let (u, w) = (
                    [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
                    [c[0] - a[0], c[1] - a[1], c[2] - a[2]],
                );
                let nn = [
                    u[1].mul_add(w[2], -(u[2] * w[1])),
                    u[2].mul_add(w[0], -(u[0] * w[2])),
                    u[0].mul_add(w[1], -(u[1] * w[0])),
                ];
                let tri = f64::from(
                    nn[0]
                        .mul_add(nn[0], nn[1].mul_add(nn[1], nn[2] * nn[2]))
                        .sqrt(),
                ) * 0.5;
                let hh =
                    f64::from((h(v[0] as usize) + h(v[k] as usize) + h(v[k + 1] as usize)) / 3.0);
                acc += tri / (hh * hh);
            }
        }
        acc
    };
    let uniform = count(&|_| target);
    let adapted = count(&|v| field[v]);
    eprintln!("[retopo] contagem prevista: uniforme {uniform:.0} · adaptada {adapted:.0}");
    assert!(
        (adapted / uniform - 1.0).abs() <= 0.02,
        "⛔ a adaptacao mudou a CONTAGEM em {:.1} % (uniforme {uniform:.0}, adaptada \
         {adapted:.0}) -- ela move os quads, nao os cria, senao o slider volta a mentir",
        100.0 * (adapted / uniform - 1.0)
    );
}

/// ⭐⭐⭐ **O ALISAMENTO DO PEDIDO É EM LOG, e o gate é a MÉDIA GEOMÉTRICA.**
///
/// ⛔⛔ **É a única asserção que distingue as duas leis.** Um campo de duas metades
/// `{1, 4}` difundido até assentar converge para `2` se a média for **geométrica**
/// (log) e para `2,5` se for **aritmética** (linear) — e as duas passam em qualquer
/// gate que só olhe «o campo ficou mais uniforme».
///
/// ⚠️ **Por que log é a lei certa:** a grandeza que a cadeia consome é uma *razão*
/// de tamanhos (*«a ponta é METADE do corpo»*), não uma diferença. Alisar em linear
/// enviesa para o maior — a ponta subiria mais depressa do que o corpo desce, que é
/// o contrário do que o report do artista pede.
#[test]
fn o_alisamento_do_pedido_e_geometrico_e_nao_aritmetico() {
    // Um quadrado de 4 vértices: dois valem `1` e dois valem `4`.
    let mesh = ph2d_mesh::Mesh::from_parts(
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        vec![ph2d_mesh::Face::quad(0, 1, 2, 3)],
    )
    .expect("a fixtura e' construida aqui");

    let mut h = vec![1.0f32, 1.0, 4.0, 4.0];
    super::smooth_in_log(&mesh, &mut h, 200);

    let geo = 2.0f32; // √(1×4)
    let arit = 2.5f32; // (1+4)/2
    for v in &h {
        assert!(
            (v - geo).abs() < 0.05,
            "o campo assentou em {v:.3}; a media GEOMETRICA e' {geo} e a ARITMETICA {arit} \
             -- se ele assentar na aritmetica, o alisamento deixou de ser em log"
        );
    }
    assert!(
        (h[0] - arit).abs() > 0.4,
        "⛔ o CONTROLO: o valor tem de ficar LONGE da media aritmetica, senao este gate \
         nao distingue as duas leis"
    );
}

/// ⚠️ **Zero rondas é um no-op BYTE-IDÊNTICO** — a metade que mantém
/// `PH2D_SIZING_SMOOTH=0` a ser uma bissecção honesta.
#[test]
fn zero_rondas_de_alisamento_nao_mexe_no_pedido() {
    let mesh = ph2d_mesh::shapes::uv_sphere(12, 8, 1.0);
    let antes: Vec<f32> = (0..mesh.vert_count())
        .map(|i| 0.01f32.mul_add(i as f32, 0.05))
        .collect();
    let mut depois = antes.clone();
    super::smooth_in_log(&mesh, &mut depois, 0);
    assert_eq!(
        antes.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        depois.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        "zero rondas tem de devolver os MESMOS BITS"
    );
}

/// ⭐⭐⭐ **O MEIO-PASSO É O QUE FAZ O ALISAMENTO CONVERGIR** — e este gate nasceu de
/// uma mutação que sobreviveu.
///
/// ⛔⛔ **A fixtura tem de ALTERNAR com a bipartição do grafo.** O anel de vértices
/// de um quad é um ciclo de comprimento `4`, que é **bipartido**: `{0, 2}` de um
/// lado, `{1, 3}` do outro. Um passo INTEIRO de Jacobi (`v ← média dos vizinhos`)
/// troca os dois lados a cada ronda e **oscila para sempre**; o meio-passo
/// (`v ← v + ½(média − v)`) contrai.
///
/// ⚠️ **O gate irmão, com `{1, 1, 4, 4}`, NÃO distingue os dois** — ali a partição
/// dos valores não coincide com a do grafo e o passo inteiro também converge. *Foi
/// exactamente essa mutação que sobreviveu, e a cura é a fixtura, não a lei.*
#[test]
fn o_alisamento_converge_mesmo_quando_o_pedido_alterna() {
    let mesh = ph2d_mesh::Mesh::from_parts(
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        vec![ph2d_mesh::Face::quad(0, 1, 2, 3)],
    )
    .expect("a fixtura e' construida aqui");

    // ⚠️ Os valores alternam AO LONGO DO ANEL — é isso que arma a oscilação.
    let mut h = vec![1.0f32, 4.0, 1.0, 4.0];
    super::smooth_in_log(&mesh, &mut h, 200);

    let span =
        h.iter().copied().fold(f32::MIN, f32::max) / h.iter().copied().fold(f32::MAX, f32::min);
    assert!(
        span < 1.01,
        "⛔ o campo NAO assentou: ele ainda varia {span:.3}x depois de 200 rondas ({h:?}) \
         -- um passo INTEIRO sobre um grafo bipartido troca os dois lados para sempre"
    );
    for v in &h {
        assert!(
            (v - 2.0).abs() < 0.05,
            "assentou em {v:.3} e a media geometrica de 1 e 4 e' 2,0"
        );
    }
}
