//! **O QUE UM DAB DE PROJECTAR CUSTA** — o item §10.6 da espec, que ela deixou
//! **por MEDIR** e que o `CLAUDE.md` §0.0 proíbe escrever sem a tabela ao lado.
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --release --test it mede_o_projectar -- --ignored --nocapture
//! ```
//!
//! # ⭐⭐⭐ O TECTO, MEDIDO — e ele é MAIS BARATO que um `Draw`
//!
//! | vértices | `Draw` | 1 alvo | 2 alvos | 4 alvos | 1+bidir | alvo 160k | 160k+bidir |
//! |---|---|---|---|---|---|---|---|
//! | `9 902` | `0,028` | `0,021` | `0,019` | `0,025` | `0,019` | `0,051` | `0,046` |
//! | `39 802` | `0,100` | `0,075` | `0,075` | `0,081` | `0,071` | `0,218` | `0,190` |
//! | `159 602` | `0,401` | `0,313` | `0,326` | `0,335` | `0,299` | `0,744` | **`0,697`** |
//!
//! ⇒ **o pior caso é `0,697 ms` contra um orçamento de `8`** — `1,7×` um dab de
//! `Draw`, com um alvo de `160 k` vértices e a procura nos DOIS sentidos.
//!
//! ⛔⛔ **A advertência da espec NÃO transferiu, e é por isso que ela mandava
//! medir:** ela cita os `2,6×`–`6,1×` que a colisão do pincel de tecido paga por
//! **um** obstáculo. Aqui o número é outro porque o trabalho é outro — a colisão
//! corre por sub-passo sobre a região simulada inteira; isto é **um raio por
//! vértice da pegada**, e a pegada tem centenas.
//!
//! ⚠️ **E a contagem de ALVOS quase não move o número** (`0,313` contra `0,335`
//! com quatro): o custo mora na pegada e na travessia, não na lista.
//!
//! # ⛔⛔ E a primeira medição devolveu `15,4 ms` — num defeito de OUTRA crate
//!
//! Antes da cura, o mesmo dab contra um alvo denso **nos dois sentidos** custava
//! `15,423 ms` numa peça de `9 902` vértices — `440×` o mesmo dab num sentido só.
//! A sonda [`diag_acerto_contra_erro`] isolou o mecanismo em dois números: na
//! mesma malha, um raio que **acerta** custava `0,436 µs` e um que **erra**
//! custava **`1 021,9 µs`** — `2 343×`.
//!
//! ⭐ **A causa eram DUAS promessas não cumpridas, uma escondida pela outra:**
//!
//! 1. o [`ph2d_mesh::Aabb::ray_slab`] **documentava** que um eixo com `NaN` não
//!    restringe o intervalo, e o código fazia o contrário (`NaN.min(+∞) = +∞`
//!    ⇒ `t0 = ∞`, que é uma **rejeição**);
//! 2. a varredura do octree empilhava a **RAIZ** sem a testar, e a poda dos
//!    filhos (`t0 <= best`) é inerte enquanto nada foi acertado (`∞ <= ∞`) ⇒
//!    *num raio que erra, cada folha era visitada*.
//!
//! ⇒ o (2) era a **rede** que escondia o (1). Depois de a raiz passar a
//! acreditar no slab, o gate
//! `an_axis_aligned_ray_grazing_a_box_plane_is_not_lost_to_nan` reprovou — e foi
//! ele que apontou o dedo ao (1).
//!
//! ⚠️⚠️ **O consumidor que o expôs foi este pincel** (um raio por vértice, e
//! metade deles erra de propósito), **mas quem já pagava era o PICK**: o cursor
//! fora da peça é um raio que erra. *Um defeito de custo que só aparece quando
//! alguém erra de propósito pode viver anos numa crate que toda a gente usa.*
//!
//! ⚠️ **`--release`, sempre.** Em debug o lançamento de raio lê ordens de
//! grandeza mais lento e o tecto sairia pequeno de mais.

use std::time::Instant;

use ph2d_mesh::{Face, Mesh, Pose};
use ph2d_sculpt3d::{Brush, Dab, RefMode, SculptStroke, Symmetry, Verb};

/// O orçamento de um dab, em milissegundos — o mesmo que o esfregão usa.
const ORCAMENTO_MS: f64 = 8.0;

fn placa(alt: f32) -> (Mesh, Pose) {
    let l = 8.0;
    let m = Mesh::from_parts(
        vec![[-l, -l, alt], [l, -l, alt], [l, l, alt], [-l, l, alt]],
        vec![Face::tri(0, 1, 2), Face::tri(0, 2, 3)],
    )
    .expect("a placa");
    (m, Pose::IDENTITY)
}

fn esfera(alvo_tris: usize) -> Mesh {
    ph2d_mesh::shapes::sphere_with_triangles(alvo_tris, 1.0)
}

fn um_dab(verb: Verb, mesh: &mut Mesh, alvos: Vec<(Mesh, Pose)>, bidir: bool) -> f64 {
    let brush = Brush {
        verb,
        mode: RefMode::B,
        radius: 0.35,
        strength: 1.0,
        project_bidirectional: bidir,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(mesh);
    s.pecas_da_cena = alvos;
    s.pose_activa = Pose::IDENTITY;
    // A pega no topo da esfera, virada para a placa que está por baixo.
    let dab = Dab::at([0.0, 0.0, 1.0], brush.radius, [0.0, 0.0, -1.0]);
    // Uma passagem de aquecimento: o octree da peça e o do alvo nascem aqui.
    s.dab(mesh, &brush, &dab, Symmetry::default());
    let t = Instant::now();
    const N: u32 = 8;
    for _ in 0..N {
        s.dab(mesh, &brush, &dab, Symmetry::default());
    }
    t.elapsed().as_secs_f64() * 1000.0 / f64::from(N)
}

/// ⭐⭐⭐ **O TECTO DE CUSTO, MEDIDO** — e ele diz de que RECURSO é.
#[test]
#[ignore]
fn mede_o_custo_de_um_dab_de_projectar() {
    println!(
        "\n{:>9}  {:>8}  {:>8}  {:>8}  {:>8}  {:>8}  {:>10}  {:>10}",
        "vertices", "Draw", "1 alvo", "2 alvos", "4 alvos", "1+bidir", "alvo 160k", "160k+bidir"
    );
    for tris in [20_000usize, 80_000, 320_000] {
        let base = esfera(tris);
        let n = base.vert_count();
        let d = um_dab(Verb::Draw, &mut base.clone(), Vec::new(), false);
        let um = um_dab(
            Verb::SceneProject,
            &mut base.clone(),
            vec![placa(-1.5)],
            false,
        );
        let dois = um_dab(
            Verb::SceneProject,
            &mut base.clone(),
            vec![placa(-1.5), placa(-2.5)],
            false,
        );
        let quatro = um_dab(
            Verb::SceneProject,
            &mut base.clone(),
            vec![placa(-1.5), placa(-2.5), placa(-3.5), placa(-4.5)],
            false,
        );
        let bidir = um_dab(
            Verb::SceneProject,
            &mut base.clone(),
            vec![placa(-1.5)],
            true,
        );
        // ⭐⭐ **A VARIÁVEL QUE DECIDE É A COMPLEXIDADE DO ALVO, não a contagem
        // deles:** uma placa tem DUAS faces, e um raio contra ela é uma
        // travessia de octree que acaba na raiz. Um alvo a sério é uma peça
        // esculpida — e é essa a coluna que nomeia o recurso.
        let denso = esfera(320_000);
        let mut afastado = Vec::new();
        for p in denso.positions() {
            afastado.push([p[0], p[1], p[2] - 2.5]);
        }
        let denso = Mesh::from_parts(afastado, denso.faces().to_vec()).expect("o alvo denso");
        let d160 = um_dab(
            Verb::SceneProject,
            &mut base.clone(),
            vec![(denso.clone(), Pose::IDENTITY)],
            false,
        );
        let d160b = um_dab(
            Verb::SceneProject,
            &mut base.clone(),
            vec![(denso, Pose::IDENTITY)],
            true,
        );
        println!(
            "{n:>9}  {d:>8.3}  {um:>8.3}  {dois:>8.3}  {quatro:>8.3}  {bidir:>8.3}  \
             {d160:>10.3}  {d160b:>10.3}"
        );
        assert!(
            d160b < ORCAMENTO_MS,
            "um dab contra um alvo DENSO nos dois sentidos custa {d160b:.3} ms \
             numa peça de {n} vértices, contra um orçamento de {ORCAMENTO_MS}"
        );
        assert!(
            um < ORCAMENTO_MS,
            "um dab com UM alvo custa {um:.3} ms numa peça de {n} vértices, \
             contra um orçamento de {ORCAMENTO_MS} — o tecto passou a ser este \
             número, e ele tem de ser escrito com esta tabela ao lado"
        );
    }
}

/// **SONDA** — quanto custa um raio que ACERTA contra um que ERRA, na mesma
/// malha densa. É ela que nomeia o recurso.
#[test]
#[ignore]
fn diag_acerto_contra_erro() {
    let alvo = esfera(320_000);
    println!("alvo: {} vertices", alvo.vert_count());
    let origem = [0.0f32, 0.0, 4.0];
    for (nome, dir) in [
        ("ACERTA (para a esfera)", [0.0f32, 0.0, -1.0]),
        ("ERRA (para fora)", [0.0, 0.0, 1.0]),
    ] {
        let raio = ph2d_mesh::Ray::new(origem, dir);
        let t = Instant::now();
        const N: u32 = 2000;
        let mut acertos = 0u32;
        for _ in 0..N {
            if alvo.raycast(&raio).is_some() {
                acertos += 1;
            }
        }
        let us = t.elapsed().as_secs_f64() * 1e6 / f64::from(N);
        println!("  {nome}: {us:.3} us por raio ({acertos} acertos de {N})");
    }
}
