//! ⭐⭐ **Os arcos atravessam a decimação** — a lei do [`super::decimate_arcs_by_turn`], sem o
//! cozedor (que vive noutra crate) nem a app: uma decomposição escrita à mão.

use crate::profile::{ArcVertex, FillRule, Profile};

/// `tan(22,5°)`: o *bulge* de um arco de `90°`.
const B90: f32 = 0.414_213_57;

/// Uma lua: o topo é uma parábola DENSA (achatada, como o cozedor a deixaria), e a base são dois
/// arcos de `90°` do círculo unitário. As duas vistas, no mesmo par.
fn lua(densidade: usize) -> Profile {
    let topo: Vec<[f32; 2]> = (0..densidade)
        .map(|i| {
            let x = 1.0 - 2.0 * i as f32 / densidade as f32;
            [x, 0.5 * (1.0 - x * x)]
        })
        .collect();
    let mut arcos: Vec<ArcVertex> = topo.iter().map(|&p| (p, 0.0)).collect();
    arcos.push(([-1.0, 0.0], -B90));
    arcos.push(([0.0, -1.0], -B90));
    let mut poli = topo;
    poli.push([-1.0, 0.0]);
    for k in 1..64 {
        let a = std::f32::consts::PI * (1.0 + k as f32 / 64.0);
        poli.push([a.cos(), a.sin()]);
    }
    Profile::with_arcs(vec![(poli, arcos)], FillRule::NonZero, 1e-4).expect("a lua é um perfil")
}

fn arcos_com_pontas(p: &Profile) -> Vec<([f32; 2], f32, [f32; 2])> {
    let a = &p.arcs()[0];
    (0..a.len())
        .filter(|&i| a[i].1 != 0.0)
        .map(|i| (a[i].0, a[i].1, a[(i + 1) % a.len()].0))
        .collect()
}

/// As três metades: os arcos ficam iguais (pontas e *bulge*), o troço denso encolhe, e a POLILINHA
/// sai byte a byte como sairia sem decomposição nenhuma — a vista nova não mexe na figura.
#[test]
fn engrossar_guarda_os_arcos_e_so_decima_o_tesselado() {
    // ⚠️ Densidades de um achatamento a sério: com `16` pontos cada vértice da parábola vira `~7°`,
    // acima de todo orçamento, e não há nada a decimar (a 1.ª redacção leu isso como defeito).
    for densidade in [256, 1024, 4096] {
        let fino = lua(densidade);
        let sem_arcos = Profile::new(fino.contours().to_vec(), fino.fill(), fino.tolerance())
            .expect("a mesma polilinha");
        for graus in [0.5f32, 1.0, 4.0] {
            let grosso = crate::coarsen_to_normal_error(&fino, graus.to_radians());
            assert_eq!(
                arcos_com_pontas(&grosso),
                arcos_com_pontas(&fino),
                "[{densidade} · {graus}°] um arco mudou ao engrossar"
            );
            assert!(
                grosso.prim_count() < fino.prim_count(),
                "[{densidade} · {graus}°] o troço denso não encolheu: {} → {}",
                fino.prim_count(),
                grosso.prim_count()
            );
            assert_eq!(
                grosso.contours(),
                crate::coarsen_to_normal_error(&sem_arcos, graus.to_radians()).contours(),
                "[{densidade} · {graus}°] a polilinha deixou de ser a de sempre"
            );
        }
    }
}

/// ⚠️ Um contorno de arcos e rectas EXACTAS não tem nada tesselado: a decomposição sai igual, e o
/// `coarse_doc` (que só aceita o que fica mais barato) deixa a peça como está.
#[test]
fn arcos_e_rectas_exactas_nao_tem_o_que_engrossar() {
    let mut poli: Vec<[f32; 2]> = Vec::new();
    for k in 0..=64 {
        let a = std::f32::consts::FRAC_PI_2 * k as f32 / 64.0;
        poli.push([1.0 + 0.2 * a.cos(), 0.2 * a.sin()]);
    }
    poli.extend([[1.0, 0.2], [-1.0, 0.2], [-1.0, -0.2], [1.2, -0.2]]);
    poli.dedup();
    let arcos: Vec<ArcVertex> = vec![
        ([1.2, 0.0], B90),
        ([1.0, 0.2], 0.0),
        ([-1.0, 0.2], 0.0),
        ([-1.0, -0.2], 0.0),
        ([1.2, -0.2], 0.0),
    ];
    let fino = Profile::with_arcs(vec![(poli, arcos)], FillRule::NonZero, 1e-4).expect("perfil");
    let grosso = crate::coarsen_to_normal_error(&fino, 1.0f32.to_radians());
    assert_eq!(grosso.arcs(), fino.arcs(), "sem troço tesselado, nada muda");
}

/// ⚠️⚠️ **Um arco RASO não perde as pontas** — o caso em que a cláusula das pontas é quem decide.
///
/// Na lua as pontas dos arcos viram `90°` e o orçamento mantê-las-ia de qualquer forma; um arco de
/// `6°` entre duas rectas tangentes vira só `3°` em cada ponta, abaixo do orçamento de um erro de
/// `4°`. Sem a cláusula a ponta sai por giro, e com ela sai o *bulge*: o arco desaparece do perfil.
#[test]
fn um_arco_raso_nao_perde_as_pontas() {
    let th = 6.0f32.to_radians();
    let (p, q) = ([0.0f32, 0.0], [th.sin(), 1.0 - th.cos()]);
    let dir = [th.cos(), th.sin()];
    let mut arcos: Vec<ArcVertex> = (0..64)
        .map(|k| ([-2.0 + 2.0 * k as f32 / 64.0, 0.0], 0.0))
        .collect();
    arcos.push((p, (th * 0.25).tan()));
    for k in 0..=64 {
        let s = 2.0 * k as f32 / 64.0;
        arcos.push(([q[0] + dir[0] * s, q[1] + dir[1] * s], 0.0));
    }
    let fim = arcos.last().expect("a corrida tem pontos").0;
    arcos.push(([fim[0], 3.0], 0.0));
    arcos.push(([-2.0, 3.0], 0.0));
    let mut poli: Vec<[f32; 2]> = Vec::new();
    for &(v, b) in &arcos {
        poli.push(v);
        if b != 0.0 {
            for k in 1..8 {
                let a = th * k as f32 / 8.0;
                poli.push([a.sin(), 1.0 - a.cos()]);
            }
        }
    }
    let fino = Profile::with_arcs(vec![(poli, arcos)], FillRule::NonZero, 1e-4).expect("perfil");
    for graus in [0.5f32, 1.0, 4.0] {
        let grosso = crate::coarsen_to_normal_error(&fino, graus.to_radians());
        assert_eq!(
            arcos_com_pontas(&grosso),
            arcos_com_pontas(&fino),
            "[{graus}°] o arco raso perdeu uma ponta ao engrossar"
        );
        assert!(
            grosso.prim_count() < fino.prim_count(),
            "[{graus}°] as corridas colineares não encolheram"
        );
    }
}
