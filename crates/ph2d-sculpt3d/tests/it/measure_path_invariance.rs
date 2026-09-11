//! **A MEDIÇÃO QUE DECIDE A LEI** — quanto o resultado de um traço depende de
//! em quantos eventos de ponteiro o MESMO caminho foi entregue.
//!
//! # A pergunta, e por que ela vem antes de uma linha da troca de lei
//!
//! O [`ph2d_sculpt3d::SculptStroke`] promete três propriedades (o cabeçalho do
//! `stroke.rs`), e a primeira é **independência de amostragem**: devagar ou
//! rápido dá o mesmo resultado. Ela é entregue pelo ENVELOPE (`max`), que é
//! idempotente — carimbar duas vezes o mesmo lugar não intensifica nada.
//!
//! A referência compõe sobre o estado **VIVO** (`vAr[ind] = vx + anx*fallOff`),
//! e composição **não é idempotente**. Trocar a lei por paridade bit-idêntica
//! entrega a lei da referência e **pode** entregar a dependência de amostragem
//! junto. *Pode*, e não *entrega*, porque isso depende de uma segunda coisa:
//! **se a lista de dabs é ou não função pura do caminho.**
//!
//! ⚠️ **Eu quase afirmei que é, e a leitura do [`ph2d_sculpt3d::walk`]
//! me derrubou:** o carry dele só cobre `dist <= min_spacing` (nada carimbado, a
//! âncora fica). Acima disso a âncora **salta para o ponteiro** e o último dab é
//! **exatamente** o ponteiro ⇒ o resto acima de um passo é DESCARTADO, e a lista
//! passa a depender de onde os eventos caíram. É a lei do original
//! (`SculptBase.js:126-151`), portada de propósito.
//!
//! Então esta sonda mede as DUAS metades separadas, porque elas têm curas
//! diferentes:
//!
//! 1. **quantos dabs** o mesmo caminho produz em cada granularidade — pura
//!    aritmética do `walk`, sem malha e sem lei;
//! 2. **quanto o barro diverge** sob cada lei, com o kernel REAL da referência
//!    sobre uma esfera real.
//!
//! Rodar:
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --release --test measure_path_invariance \
//!     -- --ignored --nocapture
//! ```

use ph2d_mesh::Mesh;
use ph2d_sculpt3d::{Dab, MIN_SPACING_FRACTION, ref_kernels as rk, walk};

/// A malha das duas medições — a mesma do `measure_reference_divergence`, para
/// os números das duas sondas serem comparáveis.
fn sphere() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(64, 128, 1.0)
}

/// As posições achatadas em `xyz`, que é a forma que o porte fala.
fn flat(mesh: &Mesh) -> Vec<f32> {
    mesh.positions().iter().flat_map(|p| *p).collect()
}

/// O caminho do gesto, em coordenadas de TELA: uma reta de `LEN_PX`.
///
/// ⚠️ **Reta e não curva, de propósito:** o que está sob medição é o
/// ESPAÇAMENTO, e numa curva o comprimento de arco entre dois eventos deixaria
/// de ser o que o `walk` mede (ele mede a CORDA). Uma reta torna as duas
/// grandezas a mesma, e o número fala só sobre a granularidade.
const LEN_PX: f32 = 200.0;

/// O raio do pincel em px de tela — o espaçamento é uma FRAÇÃO dele.
const RADIUS_PX: f32 = 40.0;

/// Em quantos eventos de ponteiro o mesmo caminho é entregue.
const GRANULARITIES: [usize; 7] = [1, 2, 3, 5, 8, 20, 100];

/// As fronteiras dos `events` pedaços em que o caminho é entregue, em px.
///
/// ⚠️ **Elas são IRREGULARES de propósito, e isso é o CONTROLE da medição:** com
/// eventos do mesmo tamanho um espaçamento exato é trivialmente invariante — a
/// grade dos dabs e a dos eventos se alinham por construção, e a coluna do walk
/// exato sairia `0,000%` sem provar nada. Um ponteiro real entrega pedaços
/// desiguais; a jitter determinística abaixo é o que torna a coluna um achado.
fn event_bounds(events: usize) -> Vec<f64> {
    let len = f64::from(LEN_PX);
    // Um LCG minúsculo: determinístico, e sem dependência nova para uma sonda.
    let mut st: u64 = 0x2545_F491_4F6C_DD1D;
    let mut w: Vec<f64> = (0..events)
        .map(|_| {
            st = st
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            // Pesos em [0,5 , 1,5]: nenhum evento nulo, e até 3x entre vizinhos.
            0.5 + (st >> 40) as f64 / f64::from(1u32 << 24)
        })
        .collect();
    let total: f64 = w.iter().sum();
    let mut acc = 0.0;
    for x in &mut w {
        acc += *x / total * len;
        *x = acc;
    }
    // A última fronteira é o fim EXATO do caminho.
    if let Some(l) = w.last_mut() {
        *l = len;
    }
    w
}

/// Os `t ∈ [0,1]` dos dabs sob a lei **ANTIGA**, congelada aqui.
///
/// ⚠️ **Ela é o "antes" da comparação, e o nome diz isso porque o produto
/// deixou de ser assim.** Antes da wave do passo exato o `walk` espaçava por
/// `dist/n` e o chamador re-ancorava no PONTEIRO; hoje ele espaça por `ms` e o
/// chamador pergunta a [`ph2d_sculpt3d::Walk::anchor`]. Uma sonda que
/// continuasse chamando isto de "o produto" mediria um driver que não existe —
/// que é a armadilha que esta linha já pagou no `measure_wetpaint_tick`.
fn dab_ts_old(events: usize) -> Vec<f64> {
    let ms = RADIUS_PX * MIN_SPACING_FRACTION;
    let mut out = Vec::new();
    // A âncora começa no pen-down, que já carimbou `t = 0`.
    let mut anchor = [0.0f32, 0.0];
    for b in event_bounds(events) {
        let to = [b as f32, 0.0];
        // ⚠️ `None` é o CARRY: nada é carimbado e a âncora TEM de ficar. Escrever
        // `anchor = to` aqui apagaria exatamente o mecanismo sob medição.
        // A lei antiga, escrita à mão porque o `walk` não a fala mais: `n` dabs
        // espaçados por `dist/n`, e a âncora salta para o ponteiro.
        let dist = (to[0] - anchor[0]).hypot(to[1] - anchor[1]);
        if dist > ms {
            let n = (dist / ms).floor().max(1.0) as u32;
            for k in 1..=n {
                let t = f64::from(k) / f64::from(n);
                let x = f64::from(anchor[0]) + (f64::from(to[0]) - f64::from(anchor[0])) * t;
                out.push(x / f64::from(LEN_PX));
            }
            anchor = to;
        }
    }
    out
}

/// Os `t` dos dabs sob a lei que o produto SHIPA — o `walk` real e a porta da
/// âncora, sem uma linha de re-expressão.
fn dab_ts(events: usize) -> Vec<f64> {
    let ms = RADIUS_PX * MIN_SPACING_FRACTION;
    let mut out = Vec::new();
    let mut anchor = [0.0f32, 0.0];
    for b in event_bounds(events) {
        let to = [b as f32, 0.0];
        if let Some(w) = walk(anchor, to, ms) {
            for p in w {
                out.push(f64::from(p[0]) / f64::from(LEN_PX));
            }
            anchor = w.anchor();
        }
    }
    out
}

/// Onde o dab de parâmetro `t` pousa na esfera: um arco no equador.
///
/// O caminho de tela vira caminho de MUNDO pela mesma régua em toda
/// granularidade, então a única coisa que varia entre as colunas é **quantos**
/// dabs há e **onde** eles caem — que é a pergunta.
fn dab_on_sphere(t: f64) -> Dab {
    let a = 0.7 + t * 0.9;
    let c = [a.cos() as f32, 0.0, a.sin() as f32];
    let eye = [-c[0], 0.0, -c[2]];
    Dab::at(c, 0.30, eye)
}

/// A lei da REFERÊNCIA: cada dab soma sobre a posição VIVA.
fn compose_over_live(mesh: &Mesh, ts: &[f64]) -> Vec<f32> {
    let mut pos = flat(mesh);
    let free = vec![1.0f32; mesh.vert_count()];
    let all: Vec<u32> = (0..mesh.vert_count() as u32).collect();
    for &t in ts {
        let d = dab_on_sphere(t);
        let r2 = f64::from(d.radius) * f64::from(d.radius);
        let c = [
            f64::from(d.center[0]),
            f64::from(d.center[1]),
            f64::from(d.center[2]),
        ];
        // A normal de área é a do centro do dab — a esfera é unitária, então ela
        // é o próprio centro normalizado. O que a sonda mede é a COMPOSIÇÃO, não
        // a normal, e mantê-la fixa tira uma variável do número.
        let n = {
            let l = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
            [c[0] / l, c[1] / l, c[2] / l]
        };
        rk::brush(&mut pos, &free, &all, None, n, c, r2, 0.5, false);
    }
    pos
}

/// **O PRODUTO** — o [`SculptStroke`] de verdade, dirigido pela porta que o
/// artista usa.
///
/// ⚠️ **Ele existe porque a coluna de cima é uma RE-EXPRESSÃO, e a lição desta
/// casa é que um número que vira decisão de produto tem de sair da porta do
/// produto.** O `SculptStroke` carrega o `ACCUM_PER_DAB`, a máscara, o octree e
/// a captura preguiçosa do `pre` — nada disso está na minha versão mínima, e
/// qualquer um deles pode mover o número.
fn product_stroke(ts: &[f64]) -> (Mesh, Vec<f32>) {
    let mut mesh = sphere();
    let before = flat(&mesh);
    let brush = ph2d_sculpt3d::Brush {
        verb: ph2d_sculpt3d::Verb::Draw,
        radius: 0.30,
        ..ph2d_sculpt3d::Brush::default()
    };
    let mut stroke = ph2d_sculpt3d::SculptStroke::default();
    stroke.begin(&mesh);
    for &t in ts {
        let d = dab_on_sphere(t);
        stroke.dab(&mut mesh, &brush, &d, ph2d_sculpt3d::Symmetry::default());
    }
    let after = flat(&mesh);
    let _ = before;
    (mesh, after)
}

/// O maior deslocamento por componente entre dois campos.
fn max_diff(a: &[f32], b: &[f32]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(&x, &y)| (f64::from(x) - f64::from(y)).abs())
        .fold(0.0, f64::max)
}

/// A excursão do próprio traço — a régua contra a qual a divergência é lida.
fn reach(a: &[f32], base: &[f32]) -> f64 {
    max_diff(a, base)
}

#[test]
#[ignore = "medição, não gate: roda com --ignored --nocapture"]
fn how_much_of_the_result_is_the_path_and_how_much_is_the_polling() {
    let mesh = sphere();
    let base = flat(&mesh);

    println!("\n=== 1. A LISTA DE DABS é função do caminho? ===");
    println!(
        "caminho de {LEN_PX} px, pincel r={RADIUS_PX} px, espaçamento mínimo {:.1} px\n",
        RADIUS_PX * MIN_SPACING_FRACTION
    );
    println!(
        "{:>8}  {:>16}  {:>16}",
        "eventos", "dabs (lei ANTIGA)", "dabs (o PRODUTO)"
    );
    let mut counts = Vec::new();
    let mut counts_x = Vec::new();
    for g in GRANULARITIES {
        let (n, nx) = (dab_ts_old(g).len(), dab_ts(g).len());
        println!("{g:>8}  {n:>16}  {nx:>16}");
        counts.push(n);
        counts_x.push(nx);
    }
    let f = |c: &[usize]| {
        let (lo, hi) = (
            *c.iter().min().expect("granularidades"),
            *c.iter().max().expect("granularidades"),
        );
        (lo, hi, hi as f64 / lo as f64)
    };
    let (lo, hi, var) = f(&counts);
    let (lox, hix, varx) = f(&counts_x);
    println!("\n  lei ANTIGA:  {lo}..{hi} dabs  =>  variação de {var:.2}x pelo MESMO caminho");
    println!("  o PRODUTO:   {lox}..{hix} dabs  =>  variação de {varx:.2}x");

    println!("\n=== 2. Quanto o BARRO diverge, por lei ===");
    println!(
        "  (divergência = maior deslocamento por componente contra a entrega em 1 evento,\n   \
           lida contra a excursão do próprio traço)\n"
    );
    println!(
        "{:>8}  {:>16}  {:>18}  {:>14}  {:>18}",
        "eventos", "PRODUTO antes", "PRODUTO agora", "COMPOR antes", "COMPOR agora"
    );

    let ref_ts = dab_ts_old(1);
    let com_ref = compose_over_live(&mesh, &ref_ts);
    let (_, prod_ref) = product_stroke(&ref_ts);
    let com_reach = reach(&com_ref, &base);
    let prod_reach = reach(&prod_ref, &base);

    let (_, prod_xref) = product_stroke(&dab_ts(1));
    let xref = compose_over_live(&mesh, &dab_ts(1));
    let x_reach = reach(&xref, &base);

    let (mut env_worst, mut com_worst, mut prod_worst, mut x_worst) =
        (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for g in GRANULARITIES {
        let ts = dab_ts_old(g);
        let com = compose_over_live(&mesh, &ts);
        let (_, prod) = product_stroke(&ts);
        let (_, prod_x) = product_stroke(&dab_ts(g));
        let x = compose_over_live(&mesh, &dab_ts(g));
        let e = max_diff(&prod_x, &prod_xref) / prod_reach * 100.0;
        let c = max_diff(&com, &com_ref) / com_reach * 100.0;
        let p = max_diff(&prod, &prod_ref) / prod_reach * 100.0;
        let xd = max_diff(&x, &xref) / x_reach * 100.0;
        env_worst = env_worst.max(e);
        com_worst = com_worst.max(c);
        prod_worst = prod_worst.max(p);
        x_worst = x_worst.max(xd);
        println!("{g:>8}  {p:>15.3}%  {e:>17.3}%  {c:>13.3}%  {xd:>17.3}%");
    }

    println!("\n  pior divergência de amostragem (% da excursão do próprio traço):");
    println!("    PRODUTO  ANTES da wave   {prod_worst:>8.3}%");
    println!("    PRODUTO  AGORA           {env_worst:>8.3}%   <= o kill-criterion pede <= 0,500%");
    println!("    COMPOR   antes da wave   {com_worst:>8.3}%");
    println!("    COMPOR   agora           {x_worst:>8.3}%   <= a metade 2 fica segura");
    println!("\n  excursão: produto {prod_reach:.6}  compor {com_reach:.6}\n");
}
