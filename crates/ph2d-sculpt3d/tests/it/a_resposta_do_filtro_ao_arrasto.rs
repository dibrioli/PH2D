//! ⭐⭐⭐ **A RESPOSTA DO FILTRO DE TECIDO: QUADRÁTICA NO ARRASTO, LINEAR NO
//! *STRENGTH*** — a lei que estava escrita em dois sítios e medida em nenhum.
//!
//! # Porque isto existe
//!
//! O doc do [`ClothFilterProps::strength`] afirma **duas** coisas, e as duas são
//! load-bearing:
//!
//! 1. *«a resposta é quadrática no arrasto»* — é o mecanismo que justifica o
//!    próprio knob existir: sem ele a única maneira de o artista pedir mais era
//!    arrastar mais, e arrastar o dobro dá **muito** mais que o dobro;
//! 2. *«ele multiplica a FORÇA, nunca o arrasto — escalar os dois seria
//!    quadrático no *Strength*, que não é a lei»*.
//!
//! ⛔⛔ **Nenhuma das duas tinha instrumento.** Uma mutação que fizesse o
//! *Strength* escalar também o arrasto — exactamente o erro que aquele parágrafo
//! avisa — passava a suíte inteira em silêncio. *Uma lei escrita em dois sítios
//! ainda não é uma lei.*
//!
//! # O mecanismo, e porque as duas metades têm de ser medidas JUNTAS
//!
//! O arrasto decide **quantos passos de simulação correm**
//! (`stroke_cloth_filter::PASSO_DE_ARRASTO`); a força entra **dentro** de cada
//! passo. Numa simulação que acumula, o deslocamento ao fim de `k` passos é
//! `Σ_j (k−j+1)·S·Δt` — **quadrático em `k`** e **linear em `S`**.
//!
//! ⚠️ **Medir só a metade (1) aprovaria a mutação**: se o *Strength* escalasse o
//! arrasto, a resposta continuaria quadrática no arrasto — e passaria a ser
//! quadrática no *Strength* também. É a segunda metade que separa as duas, e é a
//! que não existia.
//!
//! # ⚠️ As barras saem da MEDIÇÃO, e a sonda que as deu fica ao lado
//!
//! `cargo test -p ph2d-sculpt3d --release --test a_resposta_do_filtro_ao_arrasto
//! -- --ignored --nocapture`

use ph2d_mesh::Mesh;
use ph2d_sculpt3d::{ClothFilterKind, ClothFilterProps, ClothFilterStep, SculptStroke};

fn esfera() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(32, 64, 1.0)
}

/// O deslocamento MÉDIO por vértice — a grandeza que o artista vê como
/// *«quanto o filtro mexeu»*.
///
/// ⚠️ **Média e não máximo:** um extremo global mede um vértice, e o que a lei
/// da §7 governa é a peça.
fn deslocamento(rest: &Mesh, now: &Mesh) -> f64 {
    let (a, b) = (rest.positions(), now.positions());
    let mut soma = 0.0f64;
    for (p, q) in a.iter().zip(b) {
        let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
        soma += f64::from((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt());
    }
    soma / a.len() as f64
}

/// Um gesto de filtro que arrasta de `0` até `s_alvo`, com o `Strength` dado.
///
/// ⚠️ **O arrasto sobe em degraus FINOS e constantes** (`0,005`, metade do
/// `PASSO_DE_ARRASTO`), então o número de passos de simulação é proporcional ao
/// `s_alvo` **por construção** — que é precisamente a variável independente
/// destas duas medidas.
fn correr(s_alvo: f32, strength: f32, kind: ClothFilterKind) -> Mesh {
    const DEGRAU: f32 = 0.005;
    let mut m = esfera();
    let props = ClothFilterProps {
        strength,
        ..ClothFilterProps::default()
    };
    let mut st = SculptStroke::default();
    st.cloth_filter_begin(&m, props, kind, [0.0, 0.9, 0.45]);
    let chamadas = (s_alvo / DEGRAU).round() as usize;
    for k in 1..=chamadas {
        let passo = ClothFilterStep {
            s: k as f32 * DEGRAU,
            gravity_axis: [0.0, -1.0, 0.0],
            frame: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            axes: [true, true, true],
            eye: [0.0, 0.0, 1.0],
        };
        st.cloth_filter_step(&mut m, kind, &passo);
    }
    st.cloth_filter_end();
    m
}

#[test]
#[ignore = "sonda: imprime a tabela de onde as barras saem"]
fn de_onde_saem_as_duas_barras() {
    let repouso = esfera();
    println!("\n=== (A) DOBRAR O ARRASTO, com o Strength parado ===");
    println!("{:>8} | {:>12} | {:>10}", "s", "desloc", "razao 2x");
    let mut ant: Option<f64> = None;
    for s in [0.125f32, 0.25, 0.5, 1.0] {
        let d = deslocamento(&repouso, &correr(s, 1.0, ClothFilterKind::Gravity));
        match ant {
            Some(a) => println!("{s:>8.3} | {d:>12.6} | {:>10.3}", d / a),
            None => println!("{s:>8.3} | {d:>12.6} | {:>10}", "-"),
        }
        ant = Some(d);
    }

    println!("\n=== (B) DOBRAR O STRENGTH, com o arrasto parado em s = 0,5 ===");
    println!("{:>8} | {:>12} | {:>10}", "strength", "desloc", "razao 2x");
    let mut ant: Option<f64> = None;
    for f in [0.25f32, 0.5, 1.0, 2.0] {
        let d = deslocamento(&repouso, &correr(0.5, f, ClothFilterKind::Gravity));
        match ant {
            Some(a) => println!("{f:>8.3} | {d:>12.6} | {:>10.3}", d / a),
            None => println!("{f:>8.3} | {d:>12.6} | {:>10}", "-"),
        }
        ant = Some(d);
    }
}

/// ⭐⭐⭐ **A resposta é CÚBICA no arrasto — e a espec sempre o disse; a palavra
/// ao lado dela é que estava errada.**
///
/// A §7 da espec dá a lei do alvo: `S = força_base · Δpx · 0,001 · escala_UI` e o
/// acumulado `Σ_j (k−j+1)·S_j·Δt`. ⚠️⚠️ **O `S_j` CRESCE com `j`** — ele é o
/// arrasto acumulado, não uma constante —, e `Σ_j (k−j+1)·j = k(k+1)(k+2)/6`, que
/// é **cúbico**. Ler aquela soma como quadrática é supor `S_j` constante, e o
/// próprio exemplo medido da espec (`0,108000` a `k = 8`, `0,000900` num passo
/// só) só fecha com o `S_j` a crescer.
///
/// ⛔⛔ **Dois sítios do repo diziam «quadrática»** — o doc do
/// [`ClothFilterProps::strength`] e o `docs/3D/cloth/12` — e nenhum deles tinha
/// instrumento. *A fórmula estava certa e a palavra escrita ao lado dela não;
/// é a mesma família de erro que já custou uma jornada a esta linha.*
///
/// ⚠️ **A barra é o VALE entre os expoentes vizinhos, e não um número
/// escolhido:** dobrar o arrasto dá `2ⁿ`, logo `4` se fosse quadrática, **`8`**
/// se cúbica e `16` se quártica. Medido: `8,010` e `8,002` (duas dobras
/// independentes). A barra `[6, 11]` exclui as duas vizinhas com folga em ambos
/// os lados.
#[test]
fn a_resposta_do_filtro_e_cubica_no_arrasto_e_nao_quadratica() {
    let repouso = esfera();
    let d = |s: f32| deslocamento(&repouso, &correr(s, 1.0, ClothFilterKind::Gravity));
    let (d_quarto, d_meio, d_inteiro) = (d(0.25), d(0.5), d(1.0));

    // ⚠️ **Controlo de NÃO-VACUIDADE primeiro:** se o filtro não movesse nada, as
    // razões seriam `0/0` e o gate leria-se como aprovado.
    assert!(
        d_quarto > 1e-3,
        "o filtro nao moveu nada a s = 0,25 (desloc {d_quarto:.6}) -- o gate mede um no-op"
    );

    for (a, b, nome) in [
        (d_quarto, d_meio, "0,25 -> 0,50"),
        (d_meio, d_inteiro, "0,50 -> 1,00"),
    ] {
        let razao = b / a;
        assert!(
            (6.0..=11.0).contains(&razao),
            "dobrar o arrasto ({nome}) deu {razao:.3}x -- a lei da espec §7 e' CUBICA \
             (`Sigma_j (k-j+1)*S_j*dt` com `S_j` a crescer) e dobrar da' `8`. \
             `4` seria quadratica, `16` quartica, `2` linear."
        );
    }
}

/// ⭐⭐⭐ **E o *Strength* é LINEAR — a metade que separa as duas e que não
/// existia.**
///
/// ⚠️⚠️ **Medir só a metade de cima APROVARIA a mutação que o doc avisa:** se o
/// *Strength* escalasse também o arrasto, a resposta continuaria cúbica no
/// arrasto **e** passaria a ser cúbica no *Strength*. O doc do
/// [`ClothFilterProps::strength`] escreve exactamente esse perigo —
/// *«ele multiplica a FORÇA, nunca o arrasto»* — e ninguém o media.
///
/// ⚠️ **A barra é o vale entre `2` (linear) e `4` (quadrática).** Medido: `2,000`
/// em **três** dobras seguidas, ao terceiro decimal — o `S` entra como factor
/// comum de cada termo da soma, então a linearidade é exacta e não aproximada.
#[test]
fn o_strength_e_linear_e_nao_escala_o_arrasto() {
    let repouso = esfera();
    let d = |f: f32| deslocamento(&repouso, &correr(0.5, f, ClothFilterKind::Gravity));
    let (a, b, c, e) = (d(0.25), d(0.5), d(1.0), d(2.0));

    assert!(
        a > 1e-3,
        "o filtro nao moveu nada com strength 0,25 (desloc {a:.6}) -- gate vacuo"
    );

    for (x, y, nome) in [
        (a, b, "0,25 -> 0,50"),
        (b, c, "0,50 -> 1,00"),
        (c, e, "1,00 -> 2,00"),
    ] {
        let razao = y / x;
        assert!(
            (1.9..=2.1).contains(&razao),
            "dobrar o Strength ({nome}) deu {razao:.3}x -- ele multiplica a FORCA e \
             so' a forca, logo a resposta e' LINEAR nele (`2`). Um `8` aqui diz que \
             ele passou a escalar tambem o ARRASTO, que decide quantos passos correm."
        );
    }
}
