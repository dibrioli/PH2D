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
//! [`super::oraculo::le_pfm`], o [`super::oraculo::razao_rb`], a [`super::oraculo::casa_a_populacao`] e o [`super::quadro`]
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

use super::oraculo::{casa_a_populacao, le_pfm, razao_rb};
use super::{Quadro, quadro};
use ph2d_field_render::Orbit;

/// As seis células da varredura — as mesmas da [`super::oraculo::sonda_a_varredura_da_cor`].
///
/// ⭐ As três `g*` (raio IGUAL nos três canais) são **o controlo da wave inteira**: é ali que se
/// separa *«a cor muda com a PROFUNDIDADE»* de *«a cor muda porque cada canal viaja o seu»*. Nós
/// lemos `1,41` nas três — o mesmo número — e o Cycles leu `1,64 · 1,02 · 1,00`.
const CELULAS: [(&str, f32, [f32; 3]); 8] = [
    ("r003", 0.03, [1.0, 0.5, 0.25]),
    ("r010", 0.1, [1.0, 0.5, 0.25]),
    ("r030", 0.3, [1.0, 0.5, 0.25]),
    ("r100", 1.0, [1.0, 0.5, 0.25]),
    ("g003", 0.03, [1.0, 1.0, 1.0]),
    ("g010", 0.1, [1.0, 1.0, 1.0]),
    ("g030", 0.3, [1.0, 1.0, 1.0]),
    ("g100", 1.0, [1.0, 1.0, 1.0]),
];

/// Os quatro contendores, na ordem em que a tabela os imprime.
const COLUNAS: [&str; 4] = ["NÓS", "UNREAL t.real", "UNREAL traçado", "VERDADE"];

/// ⭐⭐⭐ **DUAS varreduras, e a segunda existe por uma medição do alvo.**
///
/// A **publicada** é a da §14.4 e fica por continuidade — é contra ela que os números já escritos
/// se conferem. ⛔ Mas a janela E mediu que o parâmetro de espalhamento da Unreal tem **TECTO**, e
/// que ele está no campo `≈ 100` **nas duas escalas de unidade** (a de fábrica e a outra): ali
/// `r100` e `g100` saem **idênticos ao bit**, e os canais colapsam. ⇒ *a célula de `1,00 m` é
/// inalcançável naquele motor, e um balanço que a inclua apoia-se num ponto que não é uma medição.*
///
/// A **viva** desce a varredura uma casa, para a janela onde as QUATRO colunas respondem.
/// ⚠️ Ela não é *«a mesma comparação com outro ponto»*: as quatro colunas descem **juntas**, senão
/// seria comparar profundidades diferentes em colunas diferentes — a mesma doença noutra forma.
const VARREDURAS: [(&str, [&str; 3], [&str; 3]); 2] = [
    (
        "PUBLICADA · 0,10 · 0,30 · 1,00",
        ["r010", "r030", "r100"],
        ["g010", "g030", "g100"],
    ),
    (
        "VIVA      · 0,03 · 0,10 · 0,30",
        ["r003", "r010", "r030"],
        ["g003", "g010", "g030"],
    ),
];

/// **Quão perto dois números têm de estar para este instrumento não os saber distinguir.**
///
/// ⛔ Não é escolhido: a janela E mediu que o `R/B` do traçado da Unreal move **`0,117 %`** entre
/// `64` e `256` amostras, sobre a mesma máscara — *essa é a reprodutibilidade do próprio número*.
/// Duas células mais próximas do que isto não são dois pontos, e um balanço calculado sobre elas é
/// um **PISO** e não um valor.
pub(super) const INDISTINGUIVEL: f32 = 0.002;

/// O balanço de uma família: `maior/menor`, e se algum par é **indistinguível**.
///
/// ⚠️ ⛔ *Uma lei surda e uma varredura que não varreu leem o MESMO `1,00`* — e as curas são
/// opostas (uma é um achado sobre a lei, a outra é um defeito do arnês). É por isso que a segunda
/// metade existe, e é por isso que ela é impressa ao lado do número e não em vez dele.
fn balanco_de(v: &[f32]) -> Option<(f32, bool)> {
    let [a, b, c] = v else { return None };
    let (lo, hi) = (a.min(*b).min(*c), a.max(*b).max(*c));
    let mesmo = |x: f32, y: f32| (x - y).abs() <= INDISTINGUIVEL * x.abs().max(1e-9);
    Some((
        hi / lo.max(1e-9),
        mesmo(*a, *b) || mesmo(*b, *c) || mesmo(*a, *c),
    ))
}

/// Os dois motores da Unreal, com o sufixo com que a janela E grava cada um.
const MOTORES: [(&str, &str); 2] = [("realtime", "TEMPO REAL"), ("pathtracer", "TRAÇADO")];

/// Lê um quadro de oráculo e põe-no no NOSSO olhar com a população iluminada casada com a nossa.
///
/// ⚠️ Devolve `None` **em voz alta** quando o ficheiro não existe ou não fecha — o [`super::oraculo::le_pfm`]
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
                println!(
                    "    UNREAL {rotulo:<11} ⛔ sem quadro em {dir_u}/ctrl_opaco_{sufixo}.pfm"
                );
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
    println!(
        "    célula · {:>13} · {:>13} · {:>13} · {:>13}",
        COLUNAS[0], COLUNAS[1], COLUNAS[2], COLUNAS[3]
    );
    let mut lido: Vec<(&str, [Option<f32>; 4])> = Vec::new();
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
        // ⛔ A coluna da VERDADE entra aqui, ao lado das outras três — a 1.ª redacção calculava-a,
        // IMPRIMIA-a, e não a somava ao balanço: o veredito saía **sem o lado aprovado**, que é a
        // única linha contra a qual as outras três significam alguma coisa.
        let mut col: [Option<f32>; 4] = [Some(nosso_rb), None, None, None];
        for (i, (sufixo, _)) in MOTORES.into_iter().enumerate() {
            col[i + 1] =
                oraculo(&format!("{dir_u}/jade_{tag}_{sufixo}.pfm"), nosso_n).map(|(rb, _, _)| rb);
        }
        col[3] = dir_v
            .as_ref()
            .and_then(|dv| oraculo(&format!("{dv}/ref_jade_{tag}_e5.pfm"), nosso_n))
            .map(|(rb, _, _)| rb);
        let c = |o: Option<f32>| o.map_or_else(|| format!("{:>13}", "—"), |v| format!("{v:>13.2}"));
        println!(
            "    {tag:<6} · {} · {} · {} · {}",
            c(col[0]),
            c(col[1]),
            c(col[2]),
            c(col[3])
        );
        lido.push((tag, col));
    }

    // ⭐⭐⭐ O NÚMERO DA WAVE: quanto cada lei BALANÇA com a profundidade. Uma lei surda lê `1,00`.
    for (rotulo, por_canal, iguais) in VARREDURAS {
        println!("\n  ── BALANÇO · varredura {rotulo} — uma lei SURDA lê 1,00 ──");
        for (k, nome) in COLUNAS.into_iter().enumerate() {
            for (fam, tags) in [("por canal", por_canal), ("IGUAIS   ", iguais)] {
                let v: Vec<f32> = tags
                    .iter()
                    .filter_map(|t| lido.iter().find(|(n, _)| n == t).and_then(|(_, c)| c[k]))
                    .collect();
                match balanco_de(&v) {
                    // ⚠️⚠️ **A mensagem NÃO afirma a causa, porque este instrumento não a sabe.**
                    // Duas células indistinguíveis leem-se igual em dois mundos opostos: a LEI é
                    // surda (a nossa `IGUAIS` lê `1,00` porque `sss = cor × integrate_burley` devolve
                    // o mesmo nos três canais quando o `mfp` é partilhado — isso é um ACHADO), ou o
                    // PARÂMETRO saturou (a Unreal emite o mesmo ficheiro ao bit acima do tecto —
                    // isso é um defeito do arnês). *As duas curas são opostas, e um flag que
                    // escolhesse uma mandaria metade das leituras para a errada.*
                    Some((b, true)) => println!(
                        "    {nome:<15} {fam} · balanço ≥ {b:.2}×  ⛔ PISO — duas células \
                         INDISTINGUÍVEIS (lei surda? ou parâmetro saturado?)"
                    ),
                    Some((b, false)) => println!("    {nome:<15} {fam} · balanço {b:.2}×"),
                    None => println!("    {nome:<15} {fam} · — (faltam células)"),
                }
            }
        }
    }
}
