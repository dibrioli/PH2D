//! **A CALOTA RESOLVIDA** ([`ph2d_remesh_iso::Cap`]) — o contrato da porta que a fase zero do
//! botão usa para dar `≥ 2` células de resolução ao bico de cada espinho afiado.
//!
//! ⚠️ **O que estes gates defendem é a PORTA, e não o veredito do produto**: a tabela ponta a
//! ponta (a escada de singularidades, a grade no bico, as cinco realizações) vive em
//! `docs/3D/quad-remesh/PLANO_a_graduacao_da_ponta.md` §105.

use ph2d_mesh::{Mesh, shapes};
use ph2d_remesh_iso::{ALPHA, Cap, remesh_isotropic_graded, remesh_isotropic_graded_capped};

/// A esfera com **um espinho afiado** em `+x` — a forma que o dono fotografou duas vezes.
///
/// ⚠️ A gaussiana é a mesma receita da fixtura da lei do ápice (`ph2d-quadfill`): altura `2`,
/// `σ = 0,15`. *Duas fixturas com a mesma pergunta e receitas diferentes dão duas respostas.*
fn esfera_com_espinho() -> Mesh {
    let mut m = shapes::uv_sphere(64, 96, 1.0);
    for p in m.positions_mut() {
        let n = p[0].mul_add(p[0], p[1].mul_add(p[1], p[2] * p[2])).sqrt();
        if n <= 1.0e-9 {
            continue;
        }
        let u = [p[0] / n, p[1] / n, p[2] / n];
        let d = ((u[0] - 1.0) * (u[0] - 1.0) + u[1] * u[1] + u[2] * u[2]).sqrt();
        let h = 2.0 * (-(d * d) / (2.0 * 0.15 * 0.15)).exp();
        for k in 0..3 {
            p[k] += u[k] * h;
        }
    }
    m.rebuild();
    m
}

/// O bico: o vértice mais longe do centroide.
fn bico(m: &Mesh) -> [f32; 3] {
    let pos = m.positions();
    let n = pos.len().max(1) as f32;
    let mut mid = [0.0f32; 3];
    for p in pos {
        for k in 0..3 {
            mid[k] += p[k] / n;
        }
    }
    *pos.iter()
        .max_by(|a, b| dist(**a, mid).total_cmp(&dist(**b, mid)))
        .expect("a fixtura tem vertices")
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
}

/// A aresta média entre os vértices a `raio` do ponto — a grandeza de que o pólo `+1` depende.
fn aresta_junto_de(m: &Mesh, at: [f32; 3], raio: f32) -> (f32, usize) {
    let pos = m.positions();
    let mut soma = 0.0f64;
    let mut n = 0usize;
    let mut vistos = 0usize;
    for f in m.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            if dist(a, at) <= raio || dist(b, at) <= raio {
                soma += f64::from(dist(a, b));
                n += 1;
            }
        }
    }
    for p in pos {
        if dist(*p, at) <= raio {
            vistos += 1;
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    let media = if n > 0 { (soma / n as f64) as f32 } else { 0.0 };
    (media, vistos)
}

fn iguais(a: &Mesh, b: &Mesh) -> bool {
    a.vert_count() == b.vert_count()
        && a.face_count() == b.face_count()
        && a.positions() == b.positions()
        && a.faces()
            .iter()
            .map(|f| f.verts().to_vec())
            .collect::<Vec<_>>()
            == b.faces()
                .iter()
                .map(|f| f.verts().to_vec())
                .collect::<Vec<_>>()
}

/// ⭐⭐⭐ **Sem calotas, a porta nova é a antiga AO BIT** — é o que autoriza a fase zero a
/// chamar só esta, e é o que faz a bissecção (`PH2D_TIP_CAP=0`) significar alguma coisa.
#[test]
fn uma_lista_vazia_de_calotas_e_a_porta_graduada_ao_bit() {
    let base = esfera_com_espinho();
    let (mut a, mut b) = (base.clone(), base.clone());
    remesh_isotropic_graded(&mut a, ALPHA);
    remesh_isotropic_graded_capped(&mut b, ALPHA, &[]);
    assert!(
        iguais(&a, &b),
        "a lista vazia mudou a malha: {} v / {} f contra {} v / {} f",
        a.vert_count(),
        a.face_count(),
        b.vert_count(),
        b.face_count()
    );
}

/// ⛔ **Um pedido inválido é um no-op** — e não um estouro nem uma calota infinita. A fase zero
/// deriva estes números de um slider e de uma lei de ápice; um `0` ou um `NaN` chega aqui.
#[test]
fn um_pedido_invalido_nao_muda_nada() {
    let base = esfera_com_espinho();
    let at = bico(&base);
    let t = ph2d_remesh_iso::target_edge(&base, ALPHA);
    let (mut a, mut b) = (base.clone(), base.clone());
    remesh_isotropic_graded(&mut a, ALPHA);
    remesh_isotropic_graded_capped(
        &mut b,
        ALPHA,
        &[
            Cap {
                at,
                radius: 0.0,
                step: t / 3.0,
            },
            Cap {
                at,
                radius: 3.0 * t,
                step: 0.0,
            },
            Cap {
                at,
                radius: f32::NAN,
                step: t / 3.0,
            },
            Cap {
                at,
                radius: 3.0 * t,
                step: f32::NAN,
            },
            Cap {
                at,
                radius: -3.0 * t,
                step: -1.0,
            },
        ],
    );
    assert!(iguais(&a, &b), "um pedido invalido mudou a malha");
}

/// ⭐⭐⭐ **A CALOTA AFINA O BICO — e o orçamento não estoura.**
///
/// ⚠️ **As duas metades são o gate**: afinar sozinho é o `PH2D_F1_TARGET=1` que já foi
/// **refutado** (a malha inteira ao alvo parte a cadeia a jusante). O que esta porta promete é
/// mover o orçamento, não o multiplicar — e é a renormalização por contagem que o paga.
#[test]
fn a_calota_afina_o_bico_e_o_orcamento_nao_estoura() {
    let base = esfera_com_espinho();
    let at = bico(&base);
    let t = ph2d_remesh_iso::target_edge(&base, ALPHA);
    // ⭐⭐⭐ **`t/6` e não `t/3`, e a razão é MEDIDA (2026-09-03):** nesta fixtura a graduação
    // por curvatura **já** entrega o bico a `0,0241` = `0,25 t` — mais fino que um pedido de
    // `t/3`, logo a calota seria **inerte** e o gate ficaria verde sobre uma porta morta.
    // ⚠️ *Um espinho sintético é a única coisa afiada da peça, então ele domina a mediana da
    // curvatura; numa escultura o relevo está em todo o lado e o bico não se destaca* — é por
    // isso que a peça do dono sai com o bico a `1,3`–`2,3 ×` o passo da grade e esta esfera não.
    let passo = t / 6.0;
    let raio = 8.0 * passo;

    let mut sem = base.clone();
    remesh_isotropic_graded(&mut sem, ALPHA);
    let mut com = base.clone();
    remesh_isotropic_graded_capped(
        &mut com,
        ALPHA,
        &[Cap {
            at,
            radius: raio,
            step: passo,
        }],
    );

    let (e_sem, n_sem) = aresta_junto_de(&sem, at, raio);
    let (e_com, n_com) = aresta_junto_de(&com, at, raio);
    assert!(
        n_sem >= 3 && n_com >= 3,
        "a calota tem de ter vertices nos dois lados: {n_sem} contra {n_com}"
    );
    // ⚠️ **A barra absoluta é a lei; a razão é o controlo contra uma porta morta.** Medido
    // nesta fixtura: `0,02226` (sem) → `0,01801` (com), pedido `0,01633` — a calota entrega
    // `1,10 ×` o que pede, e não `1,00 ×`, porque a banda de histerese do remalhador
    // (`SPLIT_FACTOR` / `COLLAPSE_FACTOR`) não fecha para toda aresta. ⛔ *Sem a calota o bico
    // fica a `1,36 ×` o pedido, logo a barra absoluta não é vácua.*
    assert!(
        e_com <= e_sem * 0.85,
        "a calota nao afinou o bico: {e_sem:.5} -> {e_com:.5} (pedido {passo:.5})"
    );
    assert!(
        e_com <= passo * 1.25,
        "a calota ficou longe do pedido: {e_com:.5} contra {passo:.5}"
    );

    #[allow(clippy::cast_precision_loss)]
    let inflacao = com.face_count() as f32 / sem.face_count().max(1) as f32;
    assert!(
        inflacao <= 1.25,
        "o orcamento estourou: {} -> {} faces ({inflacao:.2}x)",
        sem.face_count(),
        com.face_count()
    );
}
