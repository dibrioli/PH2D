//! ⭐⭐⭐ **SONDA — o que prende o TAMANHO DA PREGA: a rigidez, ou o ALCANCE?**
//!
//! Pergunta do dono (2026-09-09): *«o único modo de definir o tamanho da ruga é
//! a densidade da malha?»*
//!
//! # A afirmação que esta sonda existe para desmontar
//!
//! O gate irmão [`mede_o_tecido_que_estica::a_onda_de_uma_prega_acompanha_a_aresta_da_malha`]
//! mede que a onda é `~7`–`10` **arestas** e que a rigidez de dobra a move
//! `+33 %` e nada mais, e a nota conclui que decoupar a onda da malha *«é um
//! solver hierárquico»*. ⚠️ **Mas essa varredura variou a RIGIDEZ com o número de
//! passagens preso em `1`** (o valor de omissão de `PH2D_DOBRA_N`), e as duas
//! grandezas não são a mesma coisa:
//!
//! - a **rigidez** (`k`) diz com que força a restrição de ângulo empurra **ali**;
//! - o **alcance** diz até onde a notícia dessa resistência VIAJA, e uma
//!   projecção local de Jacobi propaga **uma dobradiça por passagem**.
//!
//! ⇒ se o que prende a onda é o alcance, então `n` passagens compram `~n`
//! arestas de comprimento de onda, e o solver hierárquico é a maneira **BARATA**
//! de ter muitas passagens — não um pré-requisito para ter a feature.
//!
//! *É a lei da casa: «isto fica para depois porque PRECISA de Y» — desmonte o Y
//! primeiro.*
//!
//! ⛔ **A sonda é `#[ignore]` e não afirma nada** — ela imprime a tabela. Quem
//! escrever uma barra a partir dela tem de a derivar de um recurso.
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --release --test sonda_da_onda_da_prega -- --ignored
//! --nocapture
//! ```

use ph2d_mesh::Mesh;
use ph2d_sculpt3d::{ClothFilterKind, ClothFilterProps, ClothFilterStep, SculptStroke};

/// Uma cortina franzida — o material sobra e tem de pregar.
///
/// ⚠️ **Cópia local do arnês do irmão, de propósito** (a mesma razão que o
/// `sculpt3d_filter_cloth_tests` escreve): partilhar um gerador de fixtures entre
/// ficheiros de teste é acoplamento por conveniência, e uma sonda tem de poder
/// mudar a fixtura sem mexer num gate.
fn cortina(n: usize) -> Mesh {
    let s = 2.0 / n as f32;
    let mut pos = Vec::new();
    for j in 0..=n {
        for i in 0..=n {
            let z = 0.0005 * ((i * 7 + j * 13) % 5) as f32;
            let x = i as f32 * s - 1.0;
            pos.push([if j == 0 { x * 0.5 } else { x }, 1.0 - j as f32 * s, z]);
        }
    }
    let id = |i: usize, j: usize| u32::try_from(j * (n + 1) + i).unwrap_or(u32::MAX);
    let mut faces = Vec::new();
    for j in 0..n {
        for i in 0..n {
            faces.push(ph2d_mesh::Face::tri(
                id(i, j),
                id(i + 1, j),
                id(i + 1, j + 1),
            ));
            faces.push(ph2d_mesh::Face::tri(
                id(i, j),
                id(i + 1, j + 1),
                id(i, j + 1),
            ));
        }
    }
    let mut m = Mesh::from_parts(pos, faces).unwrap_or_else(|e| panic!("{e:?}"));
    let alto: Vec<bool> = m.positions().iter().map(|p| p[1] > 0.999).collect();
    let mk = m.masks_mut();
    for (i, a) in alto.iter().enumerate() {
        if *a {
            mk[i] = 1.0;
        }
    }
    m
}

/// O comprimento de onda na fileira do meio, por travessias da média **com
/// proeminência**.
///
/// ⚠️ **O limiar de amplitude é load-bearing** — sem ele uma folha esticada conta
/// o ruído de `f32` e a régua lê *mais* pregas quanto mais plana a peça está
/// (registado no doc 11 §2.1 como uma das duas réguas que mentiram).
fn onda(m: &Mesh, n: usize) -> (f64, f64, usize) {
    let p = m.positions();
    let z: Vec<f64> = (0..=n)
        .map(|i| f64::from(p[(n / 2) * (n + 1) + i][2]))
        .collect();
    let media: f64 = z.iter().sum::<f64>() / z.len() as f64;
    let amp = z.iter().fold(0.0f64, |a, v| a.max((v - media).abs()));
    let (mut cruzes, mut lado, mut lobo) = (0usize, (z[0] - media).signum(), 0.0f64);
    for v in &z {
        let d = v - media;
        if d.signum() != lado && lobo > 0.15 * amp {
            cruzes += 1;
            lado = d.signum();
            lobo = 0.0;
        } else {
            lobo = lobo.max(d.abs());
        }
    }
    let l = if cruzes == 0 {
        f64::INFINITY
    } else {
        2.0 / cruzes as f64
    };
    (l, amp, cruzes)
}

/// Uma corrida de gravidade até ao fim, com a rigidez e as passagens pedidas.
fn corre(n: usize, bend: f32, passagens: u32) -> (f64, f64, f64, usize) {
    // ⚠️ Único sítio onde a sonda escreve no ambiente. É seguro aqui porque o
    // teste é de thread única e a variável é lida no mesmo passo, mas ⛔ nenhuma
    // outra sonda deste ficheiro pode correr em paralelo com esta.
    unsafe { std::env::set_var("PH2D_DOBRA_N", passagens.to_string()) };
    let mut m = cortina(n);
    let mut st = SculptStroke::default();
    let props = ClothFilterProps {
        bend,
        ..ClothFilterProps::default()
    };
    st.cloth_filter_begin(&m, props, ClothFilterKind::Gravity, [0.0, 1.0, 0.0]);
    for k in 0..200 {
        let p = ClothFilterStep {
            s: (k as f32 + 1.0) / 200.0,
            gravity_axis: [0.0, -1.0, 0.0],
            frame: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            axes: [true, true, true],
            eye: [0.0, 0.0, 1.0],
        };
        st.cloth_filter_step(&mut m, ClothFilterKind::Gravity, &p);
    }
    st.cloth_filter_end();
    let (l, amp, cruzes) = onda(&m, n);
    // A onda em ARESTAS é a grandeza comparável entre densidades.
    (l, l / (2.0 / n as f64), amp, cruzes)
}

#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn a_onda_contra_a_rigidez_e_contra_o_alcance() {
    println!(
        "\n{:>6} {:>6} {:>6} | {:>8} {:>8} {:>10} {:>7}",
        "lado", "bend", "passes", "onda", "arestas", "amplitude", "lobos"
    );
    println!("{}", "-".repeat(64));
    for n in [20usize, 80] {
        for bend in [0.0f32, 1.0] {
            for passes in [1u32, 2, 4, 8, 16, 32, 64] {
                // Com bend = 0 a lei sai por `return`: uma linha só basta.
                if bend == 0.0 && passes != 1 {
                    continue;
                }
                let (l, arestas, amp, lobos) = corre(n, bend, passes);
                println!(
                    "{n:>6} {bend:>6.1} {passes:>6} | {l:>8.4} {arestas:>8.2} {amp:>10.5} {lobos:>7}"
                );
            }
        }
    }
    println!(
        "\n⚠️ A AMPLITUDE e' a coluna que decide se a onda e' real: uma peca que \
         ACHATA\n   tem amplitude a morrer, e ai' a contagem de lobos mede ruido, \
         nao pregas."
    );
    println!(
        "\n⇒ se a coluna ARESTAS crescer com `passes`, o que prende a onda e' o \
         ALCANCE (e o solver hierarquico e' a via BARATA, nao um pre-requisito).\n  \
         Se ficar parada em ~7-10, a nota do gate irmao esta' certa.\n"
    );
}
