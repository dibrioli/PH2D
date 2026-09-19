//! ⭐⭐⭐ **A UNREAL COMO TERCEIRO CONTENDOR** — e ela senta-se nos DOIS lados da mesma mesa.
//!
//! Ordem do dono, 2026-09-18: *«avance para a Unreal»*. A triagem, a parede e o incidente estão na
//! [`§15`](../../../docs/Render3d/10_a_luz_que_atravessa_a_peca.md); esta sonda é a medição.
//!
//! # ⭐⭐ Porque ela vale mais que outro programa qualquer
//!
//! | motor | o que é | o que responde |
//! |---|---|---|
//! | **tempo real** | a mesma família de aproximação que nós (perfil em espaço de ecrã) | *«somos como a indústria?»* — é o nosso PAR |
//! | **traçado de caminhos** | transporte a sério, com a placa | *«a indústria está certa?»* — é uma **segunda verdade** |
//!
//! ⇒ a mesma corrida confirma ou desmente o oráculo Cycles da §14 **e** mede o nosso par. *Um alvo
//! que responde dos dois lados vale mais que dois alvos.*
//!
//! # ⛔⛔ A régua é a do módulo PAI, e isso é a decisão de desenho desta sonda
//!
//! Este é um módulo-**filho** por `#[path]`, e não um irmão de `lib.rs`: assim ele chama o
//! [`super::le_pfm`], o [`super::razao_rb`], a [`super::casa_a_populacao`] e o [`super::quadro`]
//! **sem que o pai abra visibilidade nenhuma** — e, sobretudo, sem uma segunda cópia de nenhuma
//! delas. *Três colunas medidas por três funções diferentes não são uma comparação.*
//!
//! # ⚠️ O que a janela E mediu em LINEAR, e porque não é a nossa barra
//!
//! O controlo opaco dela lê `R/B = 2,1437`, que é `0,75/0,35 = 2,14286` **a quatro dígitos** — a
//! prova mais forte de que o material está certo (lambertiano, luz branca, sem tingir nada). O
//! nosso controlo lê `1,33` porque é medido **em bytes do nosso olhar**. ⛔ *Comparar um com o
//! outro seria medir a curva de exibição e chamar-lhe material* — e é por isso que esta sonda passa
//! os quadros dela pelo MESMO caminho que passou os do Cycles.

use super::{Quadro, casa_a_populacao, le_pfm, quadro, razao_rb};
use ph2d_field_render::Orbit;

/// As seis células da varredura — as mesmas da [`super::sonda_a_varredura_da_cor`].
///
/// ⭐ As três `g*` (raio IGUAL nos três canais) são **o controlo da wave inteira**: é ali que se
/// separa *«a cor muda com a PROFUNDIDADE»* de *«a cor muda porque cada canal viaja o seu»*. Nós
/// lemos `1,41` nas três — o mesmo número — e o Cycles leu `1,64 · 1,02 · 1,00`.
const CELULAS: [(&str, f32, [f32; 3]); 6] = [
    ("r010", 0.1, [1.0, 0.5, 0.25]),
    ("r030", 0.3, [1.0, 0.5, 0.25]),
    ("r100", 1.0, [1.0, 0.5, 0.25]),
    ("g010", 0.1, [1.0, 1.0, 1.0]),
    ("g030", 0.3, [1.0, 1.0, 1.0]),
    ("g100", 1.0, [1.0, 1.0, 1.0]),
];

/// Os dois motores da Unreal, com o sufixo com que a janela E grava cada um.
const MOTORES: [(&str, &str); 2] = [("realtime", "TEMPO REAL"), ("pathtracer", "TRAÇADO")];

/// Lê um quadro de oráculo e põe-no no NOSSO olhar com a população iluminada casada com a nossa.
///
/// ⚠️ Devolve `None` **em voz alta** quando o ficheiro não existe ou não fecha — o [`super::le_pfm`]
/// recusa magia errada, cabeçalho curto, tamanho que não bate e valores não-finitos. *Um leitor que
/// devolve lixo plausível é pior que um que falha*, e esta linha já pagou isso uma vez.
fn oraculo(caminho: &str, alvo_n: usize) -> Option<(f32, usize, f32)> {
    let (_, _, linear) = le_pfm(caminho)?;
    let (bytes, stops) = casa_a_populacao(&linear, alvo_n);
    let (rb, n) = razao_rb(&bytes);
    Some((rb, n, stops))
}

/// ⏱️⭐⭐⭐ **NÓS · a UNREAL (dois motores) · a VERDADE — a mesma cena, a mesma régua.**
///
/// ```text
/// PH2D_UNREAL=/var/tmp/ph2d-oraculo-unreal/out \
/// PH2D_VERDADE2=<dir do lote 2 do Cycles> \
///   cargo test -p ph2d-app-field3d --lib sonda_a_unreal -- --ignored --nocapture
/// ```
///
/// ⛔⛔ **O CONTROLO OPACO é o portão e vem PRIMEIRO.** Sem ele nenhum número do jade vale nada: a
/// conversão de mão (a Unreal é levógira, X-para-a-frente, em centímetros) produz uma imagem
/// **espelhada** que passa despercebida a olho, e a §14.2 só teve direito ao veredito porque a bola
/// opaca bateu em largura e em cor. *Uma barra calibrada sem o lado aprovado mede os nossos
/// defeitos.*
#[test]
#[ignore = "sonda: precisa dos quadros da Unreal em $PH2D_UNREAL"]
fn sonda_a_unreal_contra_nos_e_contra_a_verdade() {
    let Ok(dir_u) = std::env::var("PH2D_UNREAL") else {
        println!("sem $PH2D_UNREAL — saltado");
        return;
    };
    let dir_v = std::env::var("PH2D_VERDADE2").ok();

    let mut cam = Orbit::default();
    cam.half_extent *= 0.42;
    cam.target = [0.55, 0.0, 0.0];
    let doc = crate::smoke::scenes::edge::cena_33().expect("a cena");
    let (onde, luz) = crate::lights::opening_light(&cam);

    let nosso_quadro = |m: ph2d_material::OpenPbr| -> Vec<u8> {
        let (_, _, px) = quadro(&Quadro {
            doc: &doc,
            m,
            cam: &cam,
            onde,
            luz,
            com_sombra: true,
            chao: None,
            sem_ceu: true,
        });
        px
    };

    // ── O PORTÃO: a bola OPACA ────────────────────────────────────────────────────────────────
    println!("\n  ── CONTROLO OPACO (o portão — se isto não bater, o jade não vale nada) ──");
    let opaco = ph2d_material::OpenPbr {
        subsurface_weight: 0.0,
        base_color: [0.75, 0.35, 0.35],
        specular_weight: 0.0,
        ..ph2d_material::OpenPbr::default()
    };
    let nossos = nosso_quadro(opaco);
    let (nosso_rb, nosso_n) = razao_rb(&nossos);
    println!("    NÓS                 R/B {nosso_rb:>6.3}   ({nosso_n} px iluminados)");

    // ⭐⭐⭐ **A BARRA É O LADO APROVADO, e só ele — nunca um número escolhido.**
    //
    // ⛔⛔ A 1.ª redacção desta sonda punha `0,05`, tirado do `1,32` que a §14.2 publica para o
    // Cycles. **Ela reprovou o próprio Cycles.** O motivo é que aquele `1,32` foi medido por OUTRA
    // régua — a sonda do controlo casa a exposição pelo **perfil de bytes de uma coluna**, esta
    // casa-a pela **população iluminada do quadro** (o método da §14.4, que é o veredito corrigido)
    // —, e ⚠️ **`R/B` em bytes NÃO é invariante à exposição**, porque a transformação de vista é
    // não-linear. *Misturar dois critérios de casamento é comparar dois números que nunca mediram a
    // mesma coisa.*
    //
    // ⇒ a barra passa a ser **medida na mesma corrida**: o desvio que o lado aprovado produz, mais
    // meia folga. Assim ela **não pode** reprovar o aprovado por construção — que é a lei do §0.9
    // levada à letra (*duas barras do tecido foram retiradas por reprovarem a saída do próprio
    // alvo*).
    let aprovado = dir_v
        .as_ref()
        .and_then(|dv| oraculo(&format!("{dv}/ref_opaco_e5.pfm"), nosso_n));
    let Some((v_rb, v_n, v_stops)) = aprovado else {
        println!(
            "    ⛔⛔ SEM O LADO APROVADO (o opaco do Cycles em $PH2D_VERDADE2) o portão NÃO ARMA.\n\
             \x20      Uma barra calibrada sem ele mede os NOSSOS defeitos — os números do jade ficam SELADOS."
        );
        return;
    };
    let folga_do_metodo = (v_rb - nosso_rb).abs();
    println!(
        "    VERDADE (Cycles)    R/B {v_rb:>6.3}   ({v_n} px · {v_stops:+.2} st) · Δ \
         {folga_do_metodo:.3} ⇐ É ESTA A BARRA"
    );
    let barra = folga_do_metodo * 1.5;

    let mut controlo_ok = true;
    for (sufixo, rotulo) in MOTORES {
        match oraculo(&format!("{dir_u}/ctrl_opaco_{sufixo}.pfm"), nosso_n) {
            Some((rb, n, stops)) => {
                let d = (rb - nosso_rb).abs();
                if d > barra {
                    controlo_ok = false;
                }
                let v = if d > barra { "⛔ FORA" } else { "✓" };
                println!(
                    "    UNREAL {rotulo:<11} R/B {rb:>6.3}   ({n} px · {stops:+.2} st) · Δ {d:.3} {v}"
                );
            }
            None => {
                controlo_ok = false;
                println!("    UNREAL {rotulo:<11} ⛔ sem quadro em {dir_u}/ctrl_opaco_{sufixo}.pfm");
            }
        }
    }
    println!("    (barra = Δ do aprovado × 1,5 = {barra:.3})");

    if !controlo_ok {
        println!(
            "\n  ⛔⛔ O CONTROLO NÃO PASSOU — os números do jade ficam SELADOS.\n     A causa é a \
             MONTAGEM (mão, escala, unidades de luz, céu), não a lei, e lê-los agora seria a régua \
             mal calibrada que a §14.3 já pagou uma vez."
        );
        return;
    }

    // ── O JADE ────────────────────────────────────────────────────────────────────────────────
    println!("\n  ── O JADE · `R/B` da região iluminada, os quatro no NOSSO olhar ──");
    println!("    célula ·       NÓS ·  UNREAL t.real ·  UNREAL traçado ·      VERDADE");
    let mut balanco: Vec<(&str, Vec<f32>)> = vec![
        ("NÓS", Vec::new()),
        ("UNREAL t.real", Vec::new()),
        ("UNREAL traçado", Vec::new()),
        ("VERDADE", Vec::new()),
    ];
    for (tag, raio, escala) in CELULAS {
        let m = ph2d_material::OpenPbr {
            subsurface_weight: 1.0,
            geometry_thin_walled: false,
            subsurface_color: [0.75, 0.35, 0.35],
            base_color: [0.75, 0.35, 0.35],
            specular_weight: 0.0,
            subsurface_radius: raio,
            subsurface_radius_scale: escala,
            ..ph2d_material::OpenPbr::default()
        };
        let (nosso_rb, nosso_n) = razao_rb(&nosso_quadro(m));
        let mut linha = format!("    {tag:<6} · {nosso_rb:>9.2}");
        balanco[0].1.push(nosso_rb);
        for (i, (sufixo, _)) in MOTORES.into_iter().enumerate() {
            let r = oraculo(&format!("{dir_u}/jade_{tag}_{sufixo}.pfm"), nosso_n);
            linha += &r.map_or_else(
                || "  ·            —".to_string(),
                |(rb, _, _)| format!("  · {rb:>13.2}"),
            );
            if let Some((rb, _, _)) = r {
                balanco[i + 1].1.push(rb);
            }
        }
        let v = dir_v
            .as_ref()
            .and_then(|dv| oraculo(&format!("{dv}/ref_jade_{tag}_e5.pfm"), nosso_n));
        linha += &v.map_or_else(
            || "  ·            —".to_string(),
            |(rb, _, _)| format!("  · {rb:>11.2}"),
        );
        // ⛔ A 1.ª redacção calculava esta coluna, IMPRIMIA-a, e não a somava ao balanço — a tabela
        // do veredito saía **sem o lado aprovado**, que é a única linha contra a qual as outras
        // três significam alguma coisa. *Uma régua que deixa o aprovado fora da conta compara os
        // contendores uns com os outros e chama-lhe verdade.*
        if let Some((rb, _, _)) = v {
            balanco[3].1.push(rb);
        }
        println!("{linha}");
    }

    // ⭐⭐⭐ O NÚMERO DA WAVE: quanto cada lei BALANÇA com a profundidade. Uma lei surda lê `1,00`.
    println!("\n  ── O BALANÇO (maior/menor sobre as três profundidades) — uma lei SURDA lê 1,00 ──");
    for (nome, vals) in &balanco {
        for (fam, faixa) in [("por canal", 0..3), ("IGUAIS   ", 3..6)] {
            let v: Vec<f32> = faixa.filter_map(|i| vals.get(i).copied()).collect();
            if v.len() == 3 {
                let (lo, hi) = v
                    .iter()
                    .fold((f32::MAX, f32::MIN), |(a, b), &x| (a.min(x), b.max(x)));
                println!("    {nome:<15} {fam} · balanço {:.2}×", hi / lo.max(1e-9));
            }
        }
    }
}
