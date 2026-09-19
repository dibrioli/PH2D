//! ⭐⭐⭐ **A BORDA QUE FERVE — as sondas** (`W7c`, report do dono de 2026-09-19: *«a peça ferve na
//! borda enquanto orbito»*).
//!
//! ⚠️ **Ela mede duas coisas que se leem iguais num número só** e que decidem waves diferentes:
//!
//! 1. **O FENÓMENO** — quanto da silhueta é uma escada **dura** (cobertura `0` ou `255`, sem meio
//!    termo) e quanto um passo de câmara **sub-pixel** faz saltar. *Uma borda dura sob movimento é
//!    exactamente o que o olho lê como fervura*: o pixel não escurece um pouco, ele **apaga-se**.
//! 2. **O PREÇO** — quanto custa, no DISPOSITIVO e a `1920×1080`, a segunda passagem que produz a
//!    cobertura parcial ([`ph2d_field_gpu::trace::TraceSetup::antialias`]).
//!
//! # ⛔⛔ Porque o preço tem de ser MEDIDO OUTRA VEZ, e não lido da tabela que existe
//!
//! O [`ph2d_field_render::trace_cancellable`] traz a tabela que pôs esta passagem fora do quadro de
//! movimento (`1,30×`–`1,40×` do quadro) — e ela foi lida **a `640×360`, na CPU, antes de o quadro
//! inteiro ir para a placa**. *Quem move o número que tornava algo inalcançável tem de reconferir a
//! nota* (`CLAUDE.md` §0.0): o brilho acabou de atravessar para o dispositivo por essa mesma razão,
//! e esta passagem é a vizinha dele.
//!
//! # ⛔⛔ DUAS sondas de preço, e só UMA decide
//!
//! - [`quanto_custa_a_borda_no_pintor`] é a que decide: ela corre o **caminho do produto**, onde só
//!   a imagem volta (`8,3 MB`), e isola a passagem desligando os outros dois passageiros da bandeira
//!   pela [`crate::gpu_frame::Sonda`].
//! - [`quanto_custa_a_borda_no_dispositivo`] mede a marcha **sozinha** pelo
//!   [`crate::gpu_frame::march`]. ⚠️ Ali o G-buffer inteiro (`49,8 MB`) atravessa o barramento **e a
//!   lista de bordas com ele** — uma travessia que o pintor não paga —, e a diferença **afoga-se no
//!   ruído** (`0,93×`–`1,03×`, com `±6 ms` de dispersão). *Ela fica pela coluna que a carga da
//!   máquina não alcança: a OCUPAÇÃO* — quantos pixels são re-amostrados é uma contagem, e ela é a
//!   mesma numa máquina a `load 50` e numa parada.

use super::borda_tests::{banda, camara, luz, parcial, quadro};
use super::device_tests::{LH, LW, contexto};

/// As cenas em que a fervura se lê — silhuetas curvas, que é onde a escada é mais grosseira.
///
/// ⚠️ **A `36` é a do brilho e entra de propósito:** é a cena que o dono acabou de aprovar, logo é
/// a que ele tem à frente quando orbita.
const CENAS: &[u32] = &[0, 5, 11, 33, 36];

/// ⭐⭐⭐ **A RÉGUA DO FERVILHAR** — o fenómeno, nos três quadros que o produto sabe pintar.
///
/// | coluna | o que diz |
/// |---|---|
/// | `parcial` | que fracção da banda da silhueta tem cobertura **entre** `0` e `255` |
/// | `médio`·`p99`·`pior` | quanto um pixel da banda muda quando a câmara roda `d`, em `/255` |
///
/// ⚠️ **As três colunas do salto e não só o máximo:** um máximo é UM pixel, e ele cai no ponto de
/// maior contraste da imagem (um reflexo na silhueta) em vez de descrever a borda. *A média é o
/// resumo honesto; o `p99` é o que o olho apanha como cintilação.*
///
/// ⚠️ **O `d` é sub-pixel de propósito.** Com a câmara a rodar meio grau por quadro toda a banda é
/// outra e a diferença entre as duas leis desaparece na aritmética; é no regime em que a silhueta
/// se desloca uma **fracção** de pixel que uma imagem honesta muda pouco e uma escada dura **vira
/// pixels inteiros**. *É essa a assinatura da fervura.*
#[test]
#[ignore = "medição — precisa de GPU"]
fn a_regua_do_fervilhar() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let (w, h) = (LW as usize, LH as usize);
    println!(
        "\n  cena · quadro · banda · parcial · d(rad) · médio · p99 · pior · {}",
        contexto()
    );
    for &n in CENAS {
        let real = crate::smoke::scene(n);
        let reg = crate::smoke::sampled_registry();
        // Os três quadros que o produto sabe pintar, com a bandeira de cada um.
        let sem_borda = crate::gpu_frame::Sonda {
            bordas: false,
            ..crate::gpu_frame::Sonda::default()
        };
        let cheia = crate::gpu_frame::Sonda::default();
        // ⚠️ **O «hoje» é o produto de ONTEM** — desde a `W7c` a segunda passagem corre em todo
        // quadro, logo a coluna que mede a fervura antiga tem de a desligar pela sonda.
        let arranjos: [(&str, bool, bool, crate::gpu_frame::Sonda); 3] = [
            ("mexia (19/09)", true, false, sem_borda),
            ("mexe agora", true, false, cheia),
            ("assente", false, true, cheia),
        ];
        for (nome, movimento, assente, sonda) in arranjos {
            let doc = crate::preview::coarse_doc(&real, movimento).unwrap_or_else(|| real.clone());
            let Some(p0) = quadro(t, &doc, &reg, &camara(0.0), assente, sonda) else {
                println!("  {n:>4} · {nome:<14} · (a placa recusa)");
                continue;
            };
            let b = banda(&p0.rgba, w, h);
            if b.is_empty() {
                println!("  {n:>4} · {nome:<14} · (sem silhueta)");
                continue;
            }
            let cobertura = parcial(&p0.rgba, &b);
            for d in [0.0005f32, 0.002] {
                let Some(p1) = quadro(t, &doc, &reg, &camara(d), assente, sonda) else {
                    continue;
                };
                let mut saltos: Vec<f32> = b
                    .iter()
                    .map(|&i| (luz(&p0.rgba[i * 4..]) - luz(&p1.rgba[i * 4..])).abs())
                    .collect();
                saltos.sort_by(f32::total_cmp);
                #[allow(clippy::cast_precision_loss)]
                let medio = saltos.iter().sum::<f32>() / saltos.len() as f32;
                #[allow(clippy::cast_precision_loss, clippy::cast_sign_loss)]
                let p99 = saltos[((saltos.len() as f32 * 0.99) as usize).min(saltos.len() - 1)];
                let pior = *saltos.last().expect("a banda não é vazia");
                println!(
                    "  {n:>4} · {nome:<14} · {:>7} · {:>6.1} % · {d:>6.4} · {medio:>6.1} · {p99:>5.1} · {pior:>5.1}",
                    b.len(),
                    cobertura * 100.0
                );
            }
        }
    }
}

/// ⭐⭐⭐ **O PREÇO DA SEGUNDA PASSAGEM, no dispositivo e a `1920×1080`** — ver a nota do módulo
/// sobre porque ele é medido pelo [`crate::gpu_frame::march`] e porque isso é um TECTO.
///
/// ⚠️ **A/B intercalado no MESMO processo** — entre duas corridas desta máquina o mesmo passe já
/// deu `11,36` e `5,50 ms` (`CLAUDE.md` §5.0), e um A medido antes de um B noutra corrida mede a
/// carga, não a lei.
#[test]
#[ignore = "medição — precisa de GPU"]
fn quanto_custa_a_borda_no_dispositivo() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    println!(
        "\n  cena · bordas · ocupação · sem borda · com borda · delta · razão · {}",
        contexto()
    );
    for &n in CENAS {
        let real = crate::smoke::scene(n);
        let reg = crate::smoke::sampled_registry();
        let doc = crate::preview::coarse_doc(&real, true).unwrap_or_else(|| real.clone());
        let cam = camara(0.0);
        let luzes = [crate::gpu_frame::tests_lampada(&cam).world];
        let marcha = |antialias: bool| {
            crate::gpu_frame::march(t, &doc, &reg, &cam, &luzes, None, LW, LH, antialias)
        };
        let Some((g, _)) = marcha(true) else {
            println!("  {n:>4} · (a placa recusa)");
            continue;
        };
        // ⭐⭐⭐ **A OCUPAÇÃO é a metade da medição que a CARGA DA MÁQUINA não alcança** — quantos
        // pixels a passagem re-amostra é uma CONTAGEM, e ela é a mesma numa máquina a `load 50` e
        // numa parada. *É ela que diz se há alguma hipótese de o relógio ser grande.*
        let bordas = g.edges.len();
        #[allow(clippy::cast_precision_loss)]
        let ocupacao = bordas as f32 / (LW as f32 * LH as f32) * 100.0;
        let (mut sem, mut com) = (f32::INFINITY, f32::INFINITY);
        for _ in 0..4 {
            for antialias in [false, true] {
                let t0 = std::time::Instant::now();
                let r = marcha(antialias);
                #[allow(clippy::cast_possible_truncation)]
                let ms = t0.elapsed().as_secs_f32() * 1e3;
                std::hint::black_box(r.map(|(g, _)| g.edges.len()));
                if antialias {
                    com = com.min(ms);
                } else {
                    sem = sem.min(ms);
                }
            }
        }
        println!(
            "  {n:>4} · {bordas:>7} · {ocupacao:>5.2} % · {sem:>9.2} · {com:>9.2} · {:>+6.2} · {:>5.2}×",
            com - sem,
            com / sem
        );
    }
}

/// ⭐⭐⭐ **O PREÇO NO CAMINHO DO PRODUTO** — o PINTOR, a `1920×1080`, com os outros dois passageiros
/// da bandeira desligados pela [`crate::gpu_frame::Sonda`].
///
/// ⚠️ **É esta a coluna que decide a wave**, e não a do [`quanto_custa_a_borda_no_dispositivo`]: ali
/// o G-buffer inteiro (`49,8 MB`) atravessa o barramento e afoga a diferença no ruído; aqui só a
/// imagem volta (`8,3 MB`), que é o que o produto faz.
///
/// | coluna | o quadro |
/// |---|---|
/// | `hoje` | o de MOVIMENTO como ele ship: sem borda, sem ricochete, sem campo do chão |
/// | `+borda` | o mesmo, **só** com a segunda passagem da silhueta ligada |
/// | `assente` | o de parar: tudo ligado — o que o artista já paga ao largar o rato |
#[test]
#[ignore = "medição — precisa de GPU"]
fn quanto_custa_a_borda_no_pintor() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    // ⚠️ **`bordas: true` com os outros dois em baixo** — é assim que a segunda passagem se mede
    // SOZINHA, que foi a pergunta da `W7c`.
    let so_a_borda = crate::gpu_frame::Sonda {
        chao_recebe_cor: false,
        ricochete: false,
        ..crate::gpu_frame::Sonda::default()
    };
    println!(
        "\n  cena · hoje · +borda · delta · razão · assente · {}",
        contexto()
    );
    for &n in CENAS {
        let real = crate::smoke::scene(n);
        let reg = crate::smoke::sampled_registry();
        let doc = crate::preview::coarse_doc(&real, true).unwrap_or_else(|| real.clone());
        let cam = camara(0.0);
        let arranjos = [
            (
                false,
                crate::gpu_frame::Sonda {
                    bordas: false,
                    ..crate::gpu_frame::Sonda::default()
                },
            ),
            (false, so_a_borda),
            (true, crate::gpu_frame::Sonda::default()),
        ];
        // Aquecimento fora da conta: a primeira corrida de cada arranjo compila o pipeline dele.
        for (assente, sonda) in arranjos {
            if quadro(t, &doc, &reg, &cam, assente, sonda).is_none() {
                println!("  {n:>4} · (a placa recusa)");
                return;
            }
        }
        let mut ms = [f32::INFINITY; 3];
        // ⚠️ **INTERCALADO** — os três arranjos em cada repetição, e não três séries seguidas.
        for _ in 0..5 {
            for (k, (assente, sonda)) in arranjos.into_iter().enumerate() {
                let t0 = std::time::Instant::now();
                let p = quadro(t, &doc, &reg, &cam, assente, sonda);
                #[allow(clippy::cast_possible_truncation)]
                let dt = t0.elapsed().as_secs_f32() * 1e3;
                std::hint::black_box(p.map(|p| p.edges));
                ms[k] = ms[k].min(dt);
            }
        }
        println!(
            "  {n:>4} · {:>7.2} · {:>7.2} · {:>+6.2} · {:>5.2}× · {:>7.2}",
            ms[0],
            ms[1],
            ms[1] - ms[0],
            ms[1] / ms[0],
            ms[2]
        );
    }
}

/// ⭐⭐ **O OUTRO passageiro da mesma lei: o contorno ENGROSSADO muda a silhueta?**
///
/// ⚠️ **A pergunta existe porque a régua do fervilhar leu `mexe c/ borda` ≡ `assente hoje` ao
/// BIT em cinco cenas** — e isso só pode ser verdade se o [`crate::preview::coarse_doc`] estiver a
/// devolver o MESMO documento nos dois quadros. *Se ele mudasse a silhueta nalguma cena, «pôr a
/// borda no quadro de movimento» não bastaria para os dois quadros terem a mesma borda ali.*
#[test]
fn o_contorno_grosso_muda_alguma_coisa() {
    let mut mexe = 0;
    let mut difere = 0;
    for n in 0..=crate::smoke::scenes::CENAS {
        if crate::smoke::scenes::PODADAS.contains(&n) {
            continue;
        }
        let real = crate::smoke::scene(n);
        let (a, b) = (
            crate::preview::coarse_doc(&real, true),
            crate::preview::coarse_doc(&real, false),
        );
        if a.is_some() || b.is_some() {
            mexe += 1;
            let prim = |d: &Option<ph2d_field::FieldDoc>| {
                d.as_ref().map_or_else(|| contorno(&real), contorno)
            };
            let (pa, pb) = (prim(&a), prim(&b));
            if pa != pb {
                difere += 1;
            }
            println!("  cena {n:>3} · a mexer {pa:>5} prims · assente {pb:>5} prims");
        }
    }
    println!(
        "  ⇒ {mexe} cenas em que o engrossamento morde, {difere} em que os dois quadros DIFEREM"
    );
}

/// Quantas primitivas de contorno o documento tem, somadas — a grandeza que o
/// [`crate::preview::coarse_doc`] compara para decidir se vale a pena engrossar.
fn contorno(doc: &ph2d_field::FieldDoc) -> usize {
    doc.nodes()
        .iter()
        .filter_map(|node| match &node.kind {
            ph2d_field::NodeKind::Leaf(
                ph2d_field::Primitive::Extrude { profile, .. }
                | ph2d_field::Primitive::Revolve { profile },
            ) => Some(profile.prim_count()),
            _ => None,
        })
        .sum()
}
