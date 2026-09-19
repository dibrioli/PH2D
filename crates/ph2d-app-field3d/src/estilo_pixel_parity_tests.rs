//! ⭐⭐⭐ **A CAMADA DE ESTILO NO PIXEL, pelo caminho do PRODUTO** — a metade que ATRIBUI.
//!
//! # ⚠️ Porque ela é irmã do [`super::estilo_lei_parity_tests`] e não o mesmo ficheiro
//!
//! Aquele mede a **LEI** com as entradas entregues à mão: ele responde *«a aritmética é a mesma?»*.
//! Este mede o **PIXEL** que os dois motores pintam da mesma peça, e responde *«e no produto, onde
//! é que isso aparece?»*. ⛔ **As duas perguntas têm curas opostas** (ver a tabela do irmão), e o
//! corte entre elas foi forçado pelo tecto de LOC — *que escolheu a fronteira certa*.
//!
//! # ⛔⛔⛔ O que esta metade DESCOBRIU, e que corrige a leitura publicada
//!
//! A dose-resposta do `estilo_tests` (`168` píxeis, pior `45` bytes) lia-se como *«a divergência da
//! curvatura chega ao pixel»*. Classificados pelo **ALFA** (a cobertura), os `86` píxeis que de
//! facto divergem com a tinta por curvatura são **`84` de BORDA anti-serrilhada e `2` de miolo** —
//! e a contagem é a **MESMA** a `nitidez 2` e a `8`, só a magnitude cresce.
//!
//! ⇒ *o clamp da tinta ABSORVE a divergência em todo pixel de cobertura cheia* (a
//! [`super::curvatura_parity_tests`] mede `ΔH ≈ 9,8e-4` e a `nitidez ≥ 1` nenhum pixel desta peça
//! está dentro da banda), e o que sobra vive onde a cobertura é parcial.

use ph2d_style::{Curvature, Style};

/// As mesmas baterias do irmão — ⚠️ **lidas de lá e não copiadas**: duas listas de estilos
/// divergiriam no dia em que alguém acrescentasse um botão a uma delas.
use super::estilo_lei_parity_tests::baterias;

/// ⭐⭐⭐ **A TABELA KNOB A KNOB, no PIXEL** — a que o `estilo_tests` publicou, agora com a lei
/// ilibada pelo gate acima.
///
/// ```text
/// bash scripts/ph2d-run.sh env PH2D_GPU=1 cargo test -p ph2d-app-field3d \
///   estilo_lei_parity_tests::a_tabela_knob_a_knob_no_pixel -- --ignored --nocapture
/// ```
#[test]
#[ignore = "precisa de adaptador de GPU"]
fn a_tabela_knob_a_knob_no_pixel() {
    let doc = crate::gpu_frame::estilo_tests::peca_com_aresta_e_cova();
    let (_, _, mats) = crate::gpu_frame::paint_parity_tests::fixtura();
    let materiais = vec![mats[0]];
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::paint_parity_tests::lampada(&cam)];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let Some((cru, _, _)) =
        crate::gpu_frame::paint_parity_tests::dois_caminhos(&surfaces, &doc, &luz)
    else {
        println!("sem adaptador — saltado");
        return;
    };
    println!("\n  BATERIA                              movidos   fora(>0)  fora(>1)   pior");
    for (rot, style) in baterias() {
        let Some((cpu, gpu, _)) = crate::gpu_frame::paint_parity_tests::dois_caminhos_vestidos(
            &surfaces,
            &doc,
            &luz,
            None,
            style,
            ph2d_field_render::Bloom::default(),
        ) else {
            continue;
        };
        // ⚠️⚠️ **O CONTROLO por bateria**: uma que não movesse nada bateria trivialmente, e a linha
        // dela leria «0 fora» como se fosse uma vitória.
        let movidos = cpu.iter().zip(&cru).filter(|(a, b)| a != b).count();
        let fora0 = cpu.iter().zip(&gpu).filter(|(a, b)| a != b).count();
        let fora1 = cpu
            .iter()
            .zip(&gpu)
            .filter(|(a, b)| a.abs_diff(**b) > 1)
            .count();
        let pior = cpu
            .iter()
            .zip(&gpu)
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap_or(0);
        println!("  {rot:<36} {movidos:>7}   {fora0:>7}  {fora1:>7}   {pior:>4}");
    }
    println!();
}

/// ⭐⭐ **A IMAGEM LADO A LADO** — a referência, o dispositivo e a DIFERENÇA amplificada.
///
/// ```text
/// env PH2D_GPU=1 PH2D_ESTILO_DUMP=<dir> bash scripts/ph2d-run.sh cargo test -p ph2d-app-field3d \
///   estilo_lei_parity_tests::a_imagem_lado_a_lado -- --ignored --nocapture
/// ```
///
/// ⚠️ **O destino vem de uma env e a sonda SALTA sem ela** — um teste que escreve num caminho fixo
/// escreve na máquina de quem o correr por acaso.
///
/// ⚠️ **A terceira faixa é `|Δ| × 16`**, e a razão é medida: a divergência da tinta por curvatura
/// chega a `45` bytes num pixel e a `0` na maioria — a `1×` ela é invisível ao olho e a tabela
/// numérica passaria a ser a única testemunha. *Uma imagem de diferença sem ganho declarado é uma
/// imagem preta.*
#[test]
#[ignore = "precisa de adaptador de GPU"]
fn a_imagem_lado_a_lado() {
    let Ok(dir) = std::env::var("PH2D_ESTILO_DUMP") else {
        println!("sem PH2D_ESTILO_DUMP — saltado");
        return;
    };
    let doc = crate::gpu_frame::estilo_tests::peca_com_aresta_e_cova();
    let (_, _, mats) = crate::gpu_frame::paint_parity_tests::fixtura();
    let materiais = vec![mats[0]];
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::paint_parity_tests::lampada(&cam)];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let w = crate::gpu_frame::paint_parity_tests::W as usize;
    let h = crate::gpu_frame::paint_parity_tests::H as usize;

    for (nome, style) in [
        ("fabrica", Style::default()),
        ("vestido", crate::gpu_frame::estilo_tests::vestido()),
        (
            "curvatura8",
            Style {
                curvature: Curvature {
                    edge_sharpness: 8.0,
                    cavity_sharpness: 8.0,
                    ..crate::gpu_frame::estilo_tests::vestido().curvature
                },
                ..Style::default()
            },
        ),
    ] {
        let Some((cpu, gpu, _)) = crate::gpu_frame::paint_parity_tests::dois_caminhos_vestidos(
            &surfaces,
            &doc,
            &luz,
            None,
            style.sanitized(),
            ph2d_field_render::Bloom::default(),
        ) else {
            println!("sem adaptador — saltado");
            return;
        };
        // Três faixas lado a lado: referência · dispositivo · |Δ| × GANHO.
        const GANHO: u16 = 16;
        let mut ppm = format!("P6\n{} {h}\n255\n", w * 3).into_bytes();
        for y in 0..h {
            for faixa in 0..3 {
                for x in 0..w {
                    let i = (y * w + x) * 4;
                    for c in 0..3 {
                        let (a, b) = (cpu[i + c], gpu[i + c]);
                        ppm.push(match faixa {
                            0 => a,
                            1 => b,
                            _ => u8::try_from((u16::from(a.abs_diff(b)) * GANHO).min(255))
                                .unwrap_or(255),
                        });
                    }
                }
            }
        }
        let caminho = format!("{dir}/estilo_{nome}.ppm");
        std::fs::write(&caminho, &ppm).expect("escrever o ppm");
        let fora = cpu.iter().zip(&gpu).filter(|(a, b)| a != b).count();
        let pior = cpu
            .iter()
            .zip(&gpu)
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap_or(0);
        println!(
            "  {nome:<12} {caminho}   fora {fora}  pior {pior}  (ganho da 3.ª faixa: {GANHO}×)"
        );
    }
}

/// ⭐⭐⭐ **ONDE ESTÃO OS PÍXEIS QUE DIVERGEM — e a resposta mudou a leitura da tabela do `§5`.**
///
/// # ⛔⛔ A imagem da diferença é um ANEL na silhueta, e eu tinha lido a tabela ao contrário
///
/// A dose-resposta do `estilo_tests` (`168` píxeis, pior `45` bytes a `nitidez 8`) lia-se como *«a
/// divergência da curvatura chega ao pixel»*. ⚠️ **Mas a [`super::curvatura_parity_tests`] mede a
/// divergência de `H` em `9,8e-4`, e a tinta é `clamp(H · raio · nitidez, ±1)`** — a `nitidez 8`
/// **zero** píxeis desta peça estão dentro da banda (medido no
/// `a_borda_dura_e_igualmente_dura_nos_dois_motores`), logo *o clamp ABSORVE a divergência em todo
/// pixel de cobertura cheia*. ⇒ os `168` têm de estar noutro sítio, e a única população que sobra é
/// a **BORDA ANTI-SERRILHADA**.
///
/// ⭐ **A classificação é o ALFA**, que é a cobertura: `255` é miolo, `0` é fundo, e o que está no
/// meio é a borda — *a régua não precisa do G-buffer, ela está na imagem*.
///
/// # ⛔⛔⛔ Este gate AFIRMA desde 2026-09-19, e antes disso só IMPRIMIA
///
/// O corpo dizia por escrito *«sem asserção de veredito sobre o miolo … a afirmação fica no gate
/// irmão, que é quem tem a barra»* — e **o irmão também não tinha barra nenhuma**: os três testes
/// deste módulo eram instrumentos, os três com o veredito escrito no NOME. ⚠️ Medido por mutação:
/// devolver o arnês da paridade ao estado montado à mão (o defeito real de 19/09) põe **`4 677`
/// píxeis a divergir, `4 519` no MIOLO**, e os três fechavam **VERDES**.
///
/// ⇒ *o que apanhou aquela regressão foi eu ler uma tabela impressa, e ninguém lê uma tabela que
/// passa.* A barra vive aqui porque é aqui que a população está classificada.
///
/// | estado | divergentes | borda | **miolo** |
/// |---|---:|---:|---:|
/// | o produto de hoje (12 baterias somadas) | — | — | **`15`** |
/// | pior bateria sozinha (`só a tinta por curvatura`) | `85` | `84` | `1` |
/// | o arnês montado à mão (a mutação) | `4 677` | `158` | **`4 519`** |
///
/// ⚠️ **A barra é `4×` a medição e não um número redondo escolhido:** o mecanismo diz que o miolo
/// tende a zero (o clamp absorve `ΔH ≈ 9,8e-4` em toda cobertura cheia) e o que fica são os píxeis
/// exactamente na banda — uma população que muda com a peça e com a câmera, nunca com a LEI. Entre
/// `60` e os `4 519` da mutação há `75×`, logo a folga não compra silêncio nenhum.
const MIOLO_MAX: usize = 60;

/// Ver [`MIOLO_MAX`] — a tabela e o porquê da folga vivem lá.
#[test]
#[ignore = "precisa de adaptador de GPU"]
fn os_pixeis_que_divergem_sao_os_da_borda_e_nao_os_do_miolo() {
    let doc = crate::gpu_frame::estilo_tests::peca_com_aresta_e_cova();
    let (_, _, mats) = crate::gpu_frame::paint_parity_tests::fixtura();
    let materiais = vec![mats[0]];
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::paint_parity_tests::lampada(&cam)];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    println!(
        "\n  BATERIA                              divergentes   na BORDA   no MIOLO   no FUNDO"
    );
    let mut miolo_total = 0usize;
    let mut movidos_total = 0usize;
    let Some((cru, _, _)) =
        crate::gpu_frame::paint_parity_tests::dois_caminhos(&surfaces, &doc, &luz)
    else {
        println!("sem adaptador — saltado");
        return;
    };
    for (rot, style) in baterias() {
        let Some((cpu, gpu, _)) = crate::gpu_frame::paint_parity_tests::dois_caminhos_vestidos(
            &surfaces,
            &doc,
            &luz,
            None,
            style,
            ph2d_field_render::Bloom::default(),
        ) else {
            println!("sem adaptador — saltado");
            return;
        };
        let (mut borda, mut miolo, mut fundo) = (0usize, 0usize, 0usize);
        for px in 0..cpu.len() / 4 {
            let i = px * 4;
            if cpu[i..i + 4] == gpu[i..i + 4] {
                continue;
            }
            // ⚠️ O alfa da REFERÊNCIA: é ele que diz a cobertura do pixel.
            match cpu[i + 3] {
                255 => miolo += 1,
                0 => fundo += 1,
                _ => borda += 1,
            }
        }
        println!(
            "  {rot:<36} {:>11}   {borda:>8}   {miolo:>8}   {fundo:>8}",
            borda + miolo + fundo
        );
        miolo_total += miolo;
        movidos_total += cpu.iter().zip(&cru).filter(|(a, b)| a != b).count();
    }
    println!();
    println!("  míolo divergente somado em todas as baterias: {miolo_total}");

    // ⚠️⚠️ **O CONTROLO vem PRIMEIRO, e não é decoração:** se nenhuma bateria mexesse um pixel
    // contra a peça CRUA, todas as linhas liriam `0` divergentes e a barra abaixo passaria sobre
    // uma camada de estilo INERTE. *Uma régua que não vê o fenómeno acontecer não prova que ele
    // não aconteceu.*
    assert!(
        movidos_total > 0,
        "nenhuma das {} baterias moveu um pixel contra a peça crua — o gate mediria o nada",
        baterias().len()
    );
    assert!(
        miolo_total <= MIOLO_MAX,
        "o miolo divergente subiu para {miolo_total} (barra {MIOLO_MAX}): a divergência deixou de \
         viver na BORDA anti-serrilhada, que é o que o nome deste gate afirma"
    );
}
