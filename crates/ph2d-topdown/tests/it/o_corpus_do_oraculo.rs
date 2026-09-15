//! **O CORPUS DO ORÁCULO vira gates** (§0.9: *cada corrida vira um gate*).
//!
//! A fixtura é a saída do Godot 4.7.2 (MIT) corrido sem interface — 19 linhas de
//! varredura e 16 de limiar, com cabeçalho e proveniência. Aqui ela é lida e
//! comparada com o que a **nossa** lei manda fazer.
//!
//! # ⚠️ Como se compara uma LEI sem mundo com um motor que tem mundo
//!
//! O corpus mede *quanto o corpo andou num tique*, e a nossa lei só sabe dizer
//! *«anda tanto nesta direcção»*. A ponte entre as duas é um **mundo de mentira**
//! com a única geometria do corpus: **uma parede plana infinita, com o corpo já
//! encostado**. Nesse mundo, *quanto coube* é aritmética e não simulação:
//!
//! - um passo que aponta **para dentro** da parede anda **zero**;
//! - um passo **tangente** anda o orçamento todo.
//!
//! ⚠️ **É um sucedâneo, e por isso ele é NOMEADO** — a wave da fábrica desta linha
//! pagou a lição de medir o produto na porta errada. O que este gate afirma é a
//! **LEI**; que a ponte a honre contra a `rapier` é outro gate, na ponte.
//!
//! # As duas barras, e de onde cada uma saiu
//!
//! - **acima do limiar**, `0,5 %` do orçamento: o oráculo lê `3,9990`–`4,0006`
//!   para um orçamento de `4,0000`, ou seja `±0,03 %`; a folga de `0,5 %` é
//!   dezasseis vezes o ruído dele e continua muito abaixo do sinal (`1,41×`).
//! - **abaixo do limiar**, `1 %` do orçamento: ali o oráculo lê `0,0037`–`0,0172`,
//!   que é a **des-penetração** da margem dele (`0,08 px`) e não movimento — o
//!   nosso plano devolve exactamente zero, e a barra existe para dizer *«o alvo
//!   também não anda»* sem fingir que os dois números são o mesmo.

use ph2d_topdown::slide::{SlideStep, first_step, next_step};

/// O orçamento do corpus, em unidades dele (píxeis por tique).
const ORCAMENTO: f32 = 4.0;
/// O knob do corpus.
const LIMIAR_DEG: f32 = 15.0;
const MAX_SLIDES: u8 = 4;

/// A normal da parede do corpus — a face está em `x = −100` e o corpo à esquerda
/// dela, logo a normal **que se opõe ao movimento** aponta para `−x`.
const NORMAL: [f32; 2] = [-1.0, 0.0];

/// **O mundo de mentira**: uma parede plana infinita, o corpo encostado.
///
/// Devolve quanto o corpo andaria num tique, corrido pelo plano inteiro da lei.
fn quanto_anda(ang_deg: f32) -> f32 {
    let a = ang_deg.to_radians();
    // A direcção pedida: `0` é de cabeça contra a parede (`+x`), `90` é rasante.
    let v = [a.cos() * ORCAMENTO * 60.0, a.sin() * ORCAMENTO * 60.0];
    let mut total = 0.0;
    let mut passo: Option<SlideStep> = first_step(v, 1.0 / 60.0, MAX_SLIDES);
    while let Some(s) = passo {
        // Quanto cabe nesta direcção, contra uma parede plana já encostada:
        // qualquer componente para dentro da parede é recusada por inteiro.
        let para_dentro = -(s.dir[0] * NORMAL[0] + s.dir[1] * NORMAL[1]);
        let andou = if para_dentro > 1.0e-6 { 0.0 } else { s.budget };
        total += andou;
        passo = next_step(s, andou, NORMAL, LIMIAR_DEG);
    }
    total
}

struct Linha {
    ang: f32,
    desloc: f32,
}

fn corpus(marca: &str) -> Vec<Linha> {
    let texto = include_str!("../fixtures/godot_slide.txt");
    texto
        .lines()
        .filter(|l| l.starts_with(marca))
        .map(|l| {
            let mut it = l.split_whitespace().skip(1);
            let ang: f32 = it.next().expect("angulo").parse().expect("angulo f32");
            let desloc: f32 = it.next().expect("desloc").parse().expect("desloc f32");
            Linha { ang, desloc }
        })
        .collect()
}

#[test]
fn o_orcamento_sobrevive_ao_deslize() {
    let linhas = corpus("VARREDURA ");
    // ⛔ **Piso de população**: uma fixtura que deixasse de ser lida (um `git mv`,
    // um filtro partido) faria este gate passar a medir NADA, em silêncio.
    assert_eq!(
        linhas.len(),
        19,
        "o corpus da varredura mudou de tamanho — re-colha-o e re-leia as barras"
    );
    let mut piores: Vec<String> = Vec::new();
    for l in &linhas {
        let nosso = quanto_anda(l.ang);
        let acima_do_limiar = l.ang > LIMIAR_DEG;
        if acima_do_limiar {
            let erro = (nosso - l.desloc).abs() / ORCAMENTO;
            if erro > 0.005 {
                piores.push(format!(
                    "{:.0}°: nos {nosso:.6} · oraculo {:.6} · erro {:.3} % do orcamento",
                    l.ang,
                    l.desloc,
                    erro * 100.0
                ));
            }
        } else if nosso / ORCAMENTO > 0.01 {
            piores.push(format!(
                "{:.0}°: nos {nosso:.6} e devia PARAR (o oraculo anda {:.6}, que e' a margem dele)",
                l.ang, l.desloc
            ));
        }
    }
    assert!(
        piores.is_empty(),
        "a nossa lei de deslize discorda do oraculo em {} de {} angulos:\n  {}\n\n\
         ⚠️ O sinal que este gate defende e' GRANDE: a 45° o orcamento inteiro anda \
         `4,000` e a projeccao andaria `2,828`. Uma lei que falhe aqui e' a lei do \
         PLATFORMER a correr numa vista de cima — o corpo rasteja na parede.",
        piores.len(),
        linhas.len(),
        piores.join("\n  ")
    );
}

#[test]
fn abaixo_do_limiar_ele_para_e_o_controlo_e_o_knob_a_zero() {
    let texto = include_str!("../fixtures/godot_slide.txt");
    let linhas: Vec<(f32, f32, f32)> = texto
        .lines()
        .filter(|l| l.starts_with("LIMIAR "))
        .map(|l| {
            let mut it = l.split_whitespace().skip(1);
            let a: f32 = it.next().unwrap().parse().unwrap();
            let com: f32 = it.next().unwrap().parse().unwrap();
            let sem: f32 = it.next().unwrap().parse().unwrap();
            (a, com, sem)
        })
        .collect();
    assert_eq!(linhas.len(), 16, "o corpus do limiar mudou de tamanho");

    // ⭐⭐ **O CONTROLO primeiro**: com o knob a zero, o oráculo anda em TODOS os
    // ângulos. Se isto falhasse, o penhasco não seria do knob — seria da
    // geometria, e o nosso `min_slide_angle` estaria a imitar a coisa errada.
    for (a, _, sem) in &linhas {
        assert!(
            sem / ORCAMENTO > 0.99,
            "controlo do oraculo partido a {a:.0}°: com o knob a ZERO ele anda {sem:.6} de {ORCAMENTO:.1}"
        );
    }

    for (a, com, _) in &linhas {
        let nosso = quanto_anda(*a);
        let oraculo_parou = com / ORCAMENTO < 0.01;
        let nos_paramos = nosso / ORCAMENTO < 0.01;
        assert_eq!(
            nos_paramos,
            oraculo_parou,
            "a {a:.0}° o oraculo {} (anda {com:.6}) e nos {} (andamos {nosso:.6}).\n\
             ⚠️ O penhasco esta' no knob e o `<=` e' MEDIDO: 15° ainda para, 16° desliza.",
            if oraculo_parou { "PARA" } else { "desliza" },
            if nos_paramos { "paramos" } else { "deslizamos" },
        );
    }
}

#[test]
fn o_tecto_de_deslizes_fecha_o_plano() {
    // Uma parede que nunca deixa andar: a tangente é re-emitida, e sem tecto o
    // laço da ponte nunca terminava.
    let v = [4.0 * 60.0, 0.0];
    let mut n = 0;
    let mut passo = first_step(v, 1.0 / 60.0, 2);
    while let Some(s) = passo {
        n += 1;
        // Uma normal SEMPRE oposta ao passo de agora ⇒ incidência frontal a cada
        // volta… que a cláusula do limiar já corta. Aqui a normal é obliqua de
        // propósito, para exercitar o TECTO e não o limiar.
        let obliqua = [
            -s.dir[0] * 0.5 - s.dir[1] * 0.866,
            -s.dir[1] * 0.5 + s.dir[0] * 0.866,
        ];
        passo = next_step(s, 0.0, obliqua, 15.0);
        assert!(
            n < 50,
            "o plano nao terminou — o tecto de deslizes nao esta' a fechar"
        );
    }
    assert_eq!(
        n, 3,
        "com `max_slides = 2` o plano tem de ter 3 passos (o primeiro + dois deslizes)"
    );
}
