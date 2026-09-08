//! ⭐⭐⭐ **O CUSTO DO FILTRO DE TECIDO** — a sonda que o report do dono
//! (*«péssima performance, impossíveis de usar»*, 2026-09-07) obrigou a existir.
//!
//! ⚠️ **Ela mede o EXPOENTE, não um relógio.** Um número de milissegundos desta
//! workstation não sobrevive à carga (`CLAUDE.md` §5.0: *nenhuma leitura de
//! relógio vale acima de `load ~5`*), mas **a forma da curva sobrevive**: se
//! dobrar os vértices multiplica o custo por `~4`, o custo é quadrático, e
//! nenhuma máquina o salva. É essa a pergunta que o report faz.
//!
//! ⚠️ **A malha do smoke tem `98 306` vértices** (`sculpt_sphere`), então um
//! expoente `2` ali é `~10¹⁰` operações — *não é lentidão, é uma paragem*.
//!
//! # ⛔⛔ O TECTO QUE SOBRA É A LEI, e está medido por ablação
//!
//! Depois das duas curas de 07/09 o passo custa **`47,5 ms`** na malha do smoke.
//! Com `VARREDURAS = 1` em vez de `5` ele custa **`12,8 ms`** ⇒ cada varredura
//! vale `~8,7 ms` e **`~91 %` do passo é a relaxação das restrições**; tudo o
//! resto (as travessias, as alocações, a integração, a escrita na malha) são
//! `~4 ms`.
//!
//! ⇒ **atacar as alocações compraria `8 %`**, e a relaxação **não se
//! paraleliza**: ela é Gauss-Seidel numa ordem que é metade da lei (espec
//! §3.1-bis), e a contagem de varreduras saiu das seis fixtures de um passo do
//! oráculo. *O tecto é o do modelo, não o da implementação* — e quem quiser mais
//! quadro por segundo reduz a MALHA (o botão de retopologia existe), não o
//! solver.

use ph2d_sculpt3d::{Brush, ClothFilterKind, ClothFilterStep, SculptStroke, Verb};
use std::time::Instant;

fn passo() -> ClothFilterStep {
    ClothFilterStep {
        s: 1.0,
        gravity_axis: [0.0, 0.0, -1.0],
        frame: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        axes: [true; 3],
        eye: [0.0, 0.0, 1.0],
    }
}

fn pincel() -> Brush {
    Brush {
        verb: Verb::Cloth,
        radius: 0.35,
        strength: 1.0,
        ..Brush::default()
    }
}

/// `(vértices, ms do pen-down, ms de um passo)`.
fn medir(nu: usize, nv: usize) -> (usize, f64, f64) {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(nu, nv, 1.0);
    let n = mesh.vert_count();
    let b = pincel();
    let mut st = SculptStroke::default();
    let t0 = Instant::now();
    st.cloth_filter_begin(&mesh, &b, ClothFilterKind::Gravity, [0.0; 3]);
    let abertura = t0.elapsed().as_secs_f64() * 1e3;
    let t1 = Instant::now();
    st.cloth_filter_step(&mut mesh, ClothFilterKind::Gravity, &passo());
    let um_passo = t1.elapsed().as_secs_f64() * 1e3;
    (n, abertura, um_passo)
}

/// **O expoente de `custo ∼ vértices^p`**, por mínimos quadrados sobre os logs.
fn expoente(amostras: &[(usize, f64)]) -> f64 {
    let n = amostras.len() as f64;
    let (mut sx, mut sy, mut sxy, mut sxx) = (0.0, 0.0, 0.0, 0.0);
    for (v, t) in amostras {
        let (x, y) = ((*v as f64).ln(), t.max(1e-9).ln());
        sx += x;
        sy += y;
        sxy += x * y;
        sxx += x * x;
    }
    (n * sxy - sx * sy) / (n * sxx - sx * sx)
}

/// ⭐⭐⭐ **O PEN-DOWN DO FILTRO É LINEAR NOS VÉRTICES.**
///
/// ⛔⛔ **Ele NÃO era, e é esse o report:** a primeira redacção do adaptador
/// escreveu um anel-1 próprio que **varria TODAS as faces por cada vértice** —
/// `O(V·F)` — quando o pincel, no ficheiro irmão, já lia a tabela pronta de
/// `mesh.adjacency()` em `O(1)`. *Uma segunda resposta a uma pergunta que a casa
/// já tinha respondido, e o preço dela é quadrático.*
///
/// ⚠️ **A barra é o EXPOENTE e não um relógio**, porque é o expoente que decide
/// se a ferramenta é usável na malha do smoke (`98 306` vértices): a `1,3` ela
/// abre; a `2,0` ela pára o app. A folga até `1,3` é para a variação de cache e
/// para o custo de construção das restrições, que é linear com constante maior.
#[test]
fn o_pen_down_do_filtro_e_linear_nos_vertices() {
    let mut abertura = Vec::new();
    let mut passos = Vec::new();
    println!("vertices     pen-down (ms)   um passo (ms)");
    for (nu, nv) in [(24, 36), (34, 51), (48, 72), (68, 102)] {
        let (n, a, p) = medir(nu, nv);
        println!("{n:>8}   {a:>13.2}   {p:>13.2}");
        abertura.push((n, a));
        passos.push((n, p));
    }
    // ⭐ **A malha do SMOKE**, medida e não extrapolada: é ela que o dono toca.
    // ⛔ Fora do ajuste do expoente de propósito — ela é outra família de malha
    // (`sculpt_sphere`, subdividida) e misturá-la com as esferas UV mediria duas
    // topologias numa recta só.
    {
        let mut mesh = ph2d_mesh::shapes::sculpt_sphere(1.0);
        let n = mesh.vert_count();
        let b = pincel();
        let mut st = SculptStroke::default();
        let t0 = Instant::now();
        st.cloth_filter_begin(&mesh, &b, ClothFilterKind::Gravity, [0.0; 3]);
        let a = t0.elapsed().as_secs_f64() * 1e3;
        let t1 = Instant::now();
        st.cloth_filter_step(&mut mesh, ClothFilterKind::Gravity, &passo());
        let p = t1.elapsed().as_secs_f64() * 1e3;
        println!("{n:>8}   {a:>13.2}   {p:>13.2}   <- a malha do smoke (=37)");
    }
    let pa = expoente(&abertura);
    let pp = expoente(&passos);
    println!("expoente do pen-down: {pa:.2}");
    println!("expoente de um passo: {pp:.2}");
    assert!(
        pa < 1.3,
        "o pen-down do filtro cresce com vertices^{pa:.2} -- na malha do smoke (98 306 vertices) \
         isso nao e' lentidao, e' uma paragem. O anel-1 tem de vir de `mesh.adjacency()`, como o \
         pincel ja' faz"
    );
    assert!(
        pp < 1.3,
        "um passo do filtro cresce com vertices^{pp:.2} -- ver o irmao acima"
    );
}
