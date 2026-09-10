//! Gates da câmera de jogo (TOP-20 #7, W1).
//!
//! # ⭐⭐⭐ O corpus é o ORÁCULO, e ele não foi inventado
//!
//! As **45** fixturas de `tests/fixtures/camera2d_godot/` são corridas do **Godot 4.7.2 (MIT)** sem
//! interface, sobre trajetórias **nossas**. Cada uma vira **dois** gates, e são dois de propósito
//! (a lei do `CLAUDE.md §0.9` — *compare POR PASSO, não só o resultado final*):
//!
//! - **por passo**: um passo a partir do estado que o ORÁCULO tinha. Isola a lei da deriva.
//! - **por trajetória**: a nossa lei corre sozinha do quadro `0` ao `239`. Prova que ela não deriva.
//!
//! ⚠️ **A configuração vem do NOME do ficheiro E é conferida contra o cabeçalho.** Só o nome
//! deixaria um corpus regerado com outros parâmetros a testar, em silêncio, uma lei diferente da
//! que a tabela afirma.

use super::*;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/camera2d_godot");
/// A janela do oráculo: `1152 × 648` ⇒ meia-janela `576 × 324`.
const HALF: [f32; 2] = [576.0, 324.0];
const DT: f32 = 1.0 / 60.0;
/// As trajetórias que cada configuração percorre.
const TRAJS: [&str; 5] = ["degrau", "rampa", "vaivem", "diagonal", "parada"];

/// `(nome, damping, dead_zone, limites)` — a configuração de cada corrida do oráculo.
///
/// ⚠️ **`damping = 0` é o `position_smoothing_enabled = false` dele**, e não «amortecimento lento».
fn corpus() -> Vec<(&'static str, f32, f32, Option<CameraLimits>)> {
    vec![
        ("livre", 0.0, 0.0, None),
        ("suave5", 5.0, 0.0, None),
        ("suave2", 2.0, 0.0, None),
        ("suave12", 12.0, 0.0, None),
        ("zonamorta", 0.0, 0.2, None),
        ("zonamorta_suave", 5.0, 0.2, None),
        (
            "limites",
            0.0,
            0.0,
            Some(CameraLimits {
                min: [-1000.0, -1000.0],
                max: [1000.0, 1000.0],
            }),
        ),
        (
            "tudo",
            5.0,
            0.2,
            Some(CameraLimits {
                min: [-1000.0, -1000.0],
                max: [1000.0, 1000.0],
            }),
        ),
        (
            // ⭐ A caixa mais ESTREITA que a janela — o caso que faz o `f32::clamp` entrar em pânico.
            "caixa_estreita",
            0.0,
            0.0,
            Some(CameraLimits {
                min: [200.0, -100_000.0],
                max: [500.0, 100_000.0],
            }),
        ),
    ]
}

struct Corrida {
    cabecalho: String,
    alvo: Vec<[f32; 2]>,
    cam: Vec<[f32; 2]>,
}

fn ler(nome: &str, traj: &str) -> Corrida {
    let p = format!("{FIXTURES}/{nome}__{traj}.csv");
    let texto = std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("fixtura ausente: {p} ({e}) — regere o corpus com o oraculo"));
    let mut cabecalho = String::new();
    let (mut alvo, mut cam) = (Vec::new(), Vec::new());
    for linha in texto.lines() {
        if let Some(resto) = linha.strip_prefix("# traj=") {
            cabecalho = resto.to_string();
            continue;
        }
        if linha.starts_with('#') || linha.starts_with("frame,") {
            continue;
        }
        let c: Vec<f32> = linha
            .split(',')
            .skip(1)
            .filter_map(|v| v.parse().ok())
            .collect();
        assert_eq!(c.len(), 4, "linha malformada em {p}: {linha}");
        alvo.push([c[0], c[1]]);
        cam.push([c[2], c[3]]);
    }
    assert!(
        alvo.len() >= 200,
        "corpus curto em {p}: {} linhas",
        alvo.len()
    );
    Corrida {
        cabecalho,
        alvo,
        cam,
    }
}

fn seguir(damping: f32, dz: f32) -> CameraFollow {
    CameraFollow {
        target: "alvo".into(),
        damping: [damping, damping],
        dead_zone: [dz, dz],
        lookahead: [0.0, 0.0],
        offset: [0.0, 0.0],
    }
}

/// ⭐⭐⭐ **A nossa lei reproduz o oráculo PASSO A PASSO, nas 45 corridas.**
///
/// A recorrência é `cam[f] = follow_step(cam[f-1], mira = alvo[f])` — e ela também foi MEDIDA, não
/// suposta: no degrau com `dz = 0,2` e `speed = 5` o oráculo devolve `15,400024`, que é
/// `184,8/12` e só sai da ordem *zona morta → amortecimento*.
///
/// ⚠️ **A ÂNCORA não é observável na fixtura** (o oráculo só publica o centro), então ela é
/// acarretada de passo a passo enquanto o CENTRO é reposto do oráculo a cada iteração. É isso que
/// mantém o gate a medir a lei em vez da deriva — e é a razão de existir o irmão de trajetória.
///
/// **Mutação que deve sangrar:** trocar a ordem em [`follow_step`] · fundir âncora e centro · tirar
/// o `min(1.0)` **não** sangra aqui de propósito (todo o corpus corre com `speed·dt < 1`) e tem
/// gate próprio.
#[test]
fn the_law_reproduces_the_oracle_step_by_step() {
    let mut piores: Vec<(String, f32)> = Vec::new();
    for (nome, damping, dz, lim) in corpus() {
        let f = seguir(damping, dz);
        for traj in TRAJS {
            let c = ler(nome, traj);
            let mut pior = 0.0f32;
            let mut ancora = c.cam[0];
            for i in 1..c.cam.len() {
                let (a, p) =
                    follow_step(ancora, c.cam[i - 1], c.alvo[i], HALF, &f, lim.as_ref(), DT);
                ancora = a;
                for (nosso, dele) in p.iter().zip(c.cam[i].iter()) {
                    pior = pior.max((nosso - dele).abs());
                }
            }
            assert!(
                pior < 0.01,
                "{nome}__{traj}: a nossa lei afasta-se do oraculo em {pior:.6} px num passo so'"
            );
            piores.push((format!("{nome}__{traj}"), pior));
        }
    }
    assert_eq!(piores.len(), 45, "o corpus tem de ser 45 corridas");
    let (onde, pior) = piores.iter().fold((String::new(), 0.0f32), |a, (n, v)| {
        if *v > a.1 { (n.clone(), *v) } else { a }
    });
    // ⚠️ **A folga é a do `f32` do oráculo, e o gate IMPRIME-A.** Uma barra sem a medição ao lado
    // não diz se está apertada: se um dia a margem subir de `0,0002` para `0,009`, o gate continua
    // verde e ninguém repara — a linha abaixo é o que torna essa subida legível.
    eprintln!("[camera2d] pior desvio por passo do corpus: {pior:.6} px em `{onde}` (barra 0,01)");
    assert!(
        pior < 0.01,
        "pior desvio por passo do corpus: {pior:.6} em `{onde}`"
    );
}

/// ⭐⭐ **E ela não DERIVA** — a corrida inteira, do quadro `0` ao fim, sem tocar no oráculo.
///
/// ⚠️ **É um gate diferente do irmão, e não uma repetição:** o de cima parte do estado do oráculo a
/// cada passo e responde *«a lei está certa?»*; este parte só do quadro `0` e responde *«ela
/// continua certa depois de 239 passos a acumular `f32`?»*. Uma lei com o sinal trocado num termo
/// pequeno passa no primeiro e reprova neste.
#[test]
fn the_law_does_not_drift_over_the_whole_run() {
    for (nome, damping, dz, lim) in corpus() {
        let f = seguir(damping, dz);
        for traj in TRAJS {
            let c = ler(nome, traj);
            let mut centro = c.cam[0];
            let mut ancora = c.cam[0];
            let mut pior = 0.0f32;
            for i in 1..c.cam.len() {
                let passo = follow_step(ancora, centro, c.alvo[i], HALF, &f, lim.as_ref(), DT);
                ancora = passo.0;
                centro = passo.1;
                for (nosso, dele) in centro.iter().zip(c.cam[i].iter()) {
                    pior = pior.max((nosso - dele).abs());
                }
            }
            assert!(
                pior < 0.5,
                "{nome}__{traj}: deriva de {pior:.6} px ao fim de {} passos",
                c.cam.len()
            );
        }
    }
}

/// ⚠️ **O CABEÇALHO tem de concordar com a tabela** — senão um corpus regerado com outros números
/// testaria, em silêncio, uma lei que a tabela não descreve.
///
/// **Mutação que deve sangrar:** mudar um `speed` da tabela sem regerar o corpus.
#[test]
fn the_fixture_header_agrees_with_the_table() {
    for (nome, damping, dz, lim) in corpus() {
        for traj in TRAJS {
            let c = ler(nome, traj);
            assert!(
                c.cabecalho.starts_with(traj),
                "{nome}__{traj}: o cabecalho diz outra trajetoria: {}",
                c.cabecalho
            );
            let liga = |k: &str| c.cabecalho.contains(k);
            if damping > 0.0 {
                assert!(
                    liga("smoothing=true") && liga(&format!("speed={damping}")),
                    "{nome}__{traj}: cabecalho sem `speed={damping}`: {}",
                    c.cabecalho
                );
            } else {
                assert!(
                    liga("smoothing=false"),
                    "{nome}__{traj}: a tabela diz instantaneo e o cabecalho nao: {}",
                    c.cabecalho
                );
            }
            if dz > 0.0 {
                assert!(
                    liga(&format!("drag={dz}")),
                    "{nome}__{traj}: cabecalho sem `drag={dz}`: {}",
                    c.cabecalho
                );
            }
            assert_eq!(
                lim.is_some(),
                !c.cabecalho.contains("limit= "),
                "{nome}__{traj}: a tabela e o cabecalho discordam sobre haver limites: {}",
                c.cabecalho
            );
        }
    }
}

/// ⭐⭐⭐ **A fixtura PRODUZ o fenómeno** — sem isto, os dois gates de paridade podiam estar verdes
/// sobre um corpus em que a câmera nunca se mexe.
///
/// ⚠️ É a terceira leitura de uma mutação sobrevivente (*a fixtura não produz o fenómeno*), escrita
/// como gate em vez de descoberta: cada configuração tem de exercer **a coisa que ela nomeia**.
#[test]
fn the_corpus_actually_exercises_what_each_row_claims() {
    // A câmera de facto anda, em todas.
    for (nome, _, _, _) in corpus() {
        for traj in TRAJS {
            let c = ler(nome, traj);
            let excursao = c
                .cam
                .iter()
                .fold(0.0f32, |a, p| a.max(p[0].abs() + p[1].abs()));
            assert!(
                excursao > 1.0,
                "{nome}__{traj}: a camera nunca saiu do sitio (excursao {excursao}) — a fixtura \
                 nao produz o fenomeno que os gates de paridade dizem medir"
            );
        }
    }
    // A ZONA MORTA de facto segura a câmera parada enquanto o alvo anda.
    let c = ler("zonamorta", "rampa");
    let parados = c.cam.iter().take(35).filter(|p| p[0] == 0.0).count();
    assert!(
        parados >= 34,
        "a fixtura da zona morta nao mostra a camera PARADA no arranque ({parados} quadros)"
    );
    // Os LIMITES de facto saturam.
    let c = ler("limites", "rampa");
    let topo = c.cam.iter().fold(0.0f32, |a, p| a.max(p[0]));
    assert!(
        (topo - 424.0).abs() < 0.01,
        "a fixtura dos limites satura em {topo}, e o oraculo mediu 424,000000 (1000 − 576)"
    );
}

/// ⛔⛔ **A DIVERGÊNCIA DECLARADA** — nós não oscilamos onde o oráculo oscila.
///
/// Os números do lado dele são medidos nesta máquina, com o degrau de `0 → 300` e `dt = 1/60`:
/// `speed = 120` (`k = 2`) dá `600 → 0 → 600` para sempre; `speed = 200` (`k = 3,33`) dá
/// `1000,000061 → −1333,333740` e continua a abrir.
///
/// **Mutação que deve sangrar:** tirar o `.min(1.0)` de [`damp_axis`].
#[test]
fn we_do_not_diverge_where_the_oracle_does() {
    for speed in [60.0_f32, 120.0, 200.0, 10_000.0] {
        let mut c = 0.0_f32;
        let mut pior = 0.0_f32;
        for _ in 0..240 {
            c = damp_axis(c, 300.0, speed, DT);
            pior = pior.max(c.abs());
        }
        assert!(
            (c - 300.0).abs() < 1e-3,
            "speed={speed}: assentou em {c} em vez de 300 — o oraculo oscila `600 -> 0 -> 600` a \
             120 e diverge para −1333,33 a 200, e e' isso que o `min(1.0)` recusa"
        );
        assert!(
            pior <= 300.0 + 1e-3,
            "speed={speed}: passou do alvo (pico {pior}) — uma interpolacao nao ultrapassa"
        );
    }
}

/// ⭐⭐ **Uma sala mais ESTREITA que o ecrã fixa a câmera no centro dela — e não estoura.**
///
/// ⛔ O `f32::clamp` da biblioteca padrão **entra em pânico** quando `min > max`, que é exactamente
/// o que `lo + meia > hi − meia` dá. *Uma sala pequena derrubaria o jogo.*
#[test]
fn a_room_narrower_than_the_screen_pins_the_camera_and_does_not_panic() {
    // Medido no oráculo: limites [200, 500], meia-janela 576 ⇒ 350,000000 em TODOS os quadros.
    for alvo in [-5_000.0_f32, 0.0, 80.0, 780.0, 5_000.0] {
        let c = clamp_axis_to_limits(alvo, 576.0, 200.0, 500.0);
        assert!(
            (c - 350.0).abs() < 1e-4,
            "alvo {alvo}: deu {c}, e o oraculo mediu 350,000000 = (200+500)/2"
        );
    }
}

/// A meia-janela respeita a proporção — num ecrã largo a meia-largura não é a meia-altura.
#[test]
fn the_half_window_follows_the_aspect() {
    let h = half_extent(10.0, 1152.0 / 648.0);
    assert!((h[1] - 5.0).abs() < 1e-5, "meia-altura: {h:?}");
    assert!(
        (h[0] - 5.0 * (1152.0 / 648.0)).abs() < 1e-4,
        "meia-largura: {h:?}"
    );
    // Degenerados não devolvem lixo.
    assert_eq!(half_extent(-1.0, 1.0), [0.0, 0.0]);
}

/// ⚠️ **A zona morta é cravada em `0..=1`** — acima de `1` o alvo sairia do ecrã sem a câmera reagir.
#[test]
fn the_dead_zone_is_fenced_to_the_window() {
    let centro = [0.0, 0.0];
    // `dz = 2` tem de agir como `dz = 1`: a margem é a meia-janela inteira, nunca o dobro dela.
    let a = dead_zone_goal(centro, [1000.0, 0.0], HALF, [2.0, 2.0]);
    let b = dead_zone_goal(centro, [1000.0, 0.0], HALF, [1.0, 1.0]);
    assert_eq!(a, b, "dz=2 tem de ser tratado como dz=1");
    assert!((a[0] - (1000.0 - 576.0)).abs() < 1e-3, "{a:?}");
    // `dz = 0` cola no alvo.
    let z = dead_zone_goal(centro, [7.0, -3.0], HALF, [0.0, 0.0]);
    assert_eq!(z, [7.0, -3.0]);
}

/// ⭐ **A antecipação não inventa velocidade no primeiro passo.**
///
/// **Mutação que deve sangrar:** trocar o `None` por `Some(target)` — a mira passaria a antecipar
/// `0` sempre, e o gate da velocidade real morreria com a suíte verde.
#[test]
fn lookahead_needs_a_previous_step_to_exist() {
    let f = CameraFollow {
        lookahead: [0.5, 0.0],
        ..CameraFollow::default()
    };
    // Sem passo anterior: a mira é o alvo (mais o offset, que aqui é zero).
    assert_eq!(aim_at([10.0, 0.0], None, &f, DT), [10.0, 0.0]);
    // Com passo anterior: o alvo anda 1 m por passo ⇒ 60 m/s ⇒ meio segundo à frente = +30 m.
    let a = aim_at([10.0, 0.0], Some([9.0, 0.0]), &f, DT);
    assert!((a[0] - 40.0).abs() < 1e-3, "antecipacao: {a:?}");
}

/// O `offset` entra na mira mesmo sem antecipação — é o enquadramento, não um efeito de velocidade.
#[test]
fn the_offset_frames_the_target_even_without_lookahead() {
    let f = CameraFollow {
        offset: [0.0, 2.5],
        ..CameraFollow::default()
    };
    assert_eq!(aim_at([1.0, 1.0], None, &f, DT), [1.0, 3.5]);
}

/// ⭐⭐ **A câmera que manda: maior prioridade, desempate pelo `StableId`, e a desligada não conta.**
///
/// **Mutações que devem sangrar:** tirar o filtro do `active` · trocar o `Reverse` do desempate.
#[test]
fn the_active_camera_is_the_highest_priority_and_ties_break_by_stable_id() {
    let mut w = World::new();
    let baixa = w
        .spawn((
            GameCamera {
                priority: 1,
                ..GameCamera::default()
            },
            crate::StableId(7),
        ))
        .id();
    let alta = w
        .spawn((
            GameCamera {
                priority: 5,
                ..GameCamera::default()
            },
            crate::StableId(99),
        ))
        .id();
    assert_eq!(active_camera_of(&mut w), Some(alta), "a prioridade manda");
    assert_eq!(camera_count(&mut w), 2);

    // Empate: ganha o MENOR StableId.
    let empate = w
        .spawn((
            GameCamera {
                priority: 5,
                ..GameCamera::default()
            },
            crate::StableId(2),
        ))
        .id();
    assert_eq!(
        active_camera_of(&mut w),
        Some(empate),
        "no empate ganha o menor StableId — senao a ordem de arquetipo decide, e ela muda quando \
         alguem insere um componente"
    );

    // Desligar a vencedora devolve o lugar à seguinte, e NÃO à desligada.
    w.get_mut::<GameCamera>(empate).unwrap().active = false;
    w.get_mut::<GameCamera>(alta).unwrap().active = false;
    assert_eq!(
        active_camera_of(&mut w),
        Some(baixa),
        "a desligada nao concorre"
    );
    w.get_mut::<GameCamera>(baixa).unwrap().active = false;
    assert_eq!(
        active_camera_of(&mut w),
        None,
        "todas desligadas = nenhuma manda"
    );
}

/// ⚠️ **Uma cena sem câmera nenhuma responde `None`, e isso é informação** — o enquadramento fica
/// sendo o do editor, que é o comportamento de hoje.
#[test]
fn a_scene_with_no_camera_says_so() {
    let mut w = World::new();
    assert_eq!(active_camera_of(&mut w), None);
    assert_eq!(camera_count(&mut w), 0);
}

/// ⛔⛔ **O estado VIVO não pode ser gravado** — a cerca é do TIPO, e este gate diz porquê.
///
/// ⚠️ Ele não compila nada: afirma a propriedade em prosa executável, porque o que o protege é a
/// **ausência** de `Serialize`, e uma ausência não tem asserção directa. O que se pode afirmar é
/// que o `CameraRuntime` nasce vazio e que o registo NÃO o conhece — ver
/// `registry_tests.rs`.
#[test]
fn the_live_state_starts_empty_and_is_not_a_document() {
    let r = CameraRuntime::default();
    assert_eq!(r.center, [0.0, 0.0]);
    assert_eq!(r.last_target, None);
    assert!(!r.settled, "uma camera por assentar tem de o dizer");
}

/// **Damping `0` é instantâneo**, e é o que reproduz o oráculo sem suavização.
#[test]
fn zero_damping_is_instant_not_frozen() {
    assert_eq!(damp_axis(0.0, 300.0, 0.0, DT), 300.0);
    assert_eq!(damp_axis(0.0, 300.0, -1.0, DT), 300.0);
    assert_eq!(damp_axis(0.0, 300.0, f32::NAN, DT), 300.0);
    // `dt` inválido também não congela a câmera num sítio errado.
    assert_eq!(damp_axis(0.0, 300.0, 5.0, 0.0), 300.0);
}

/// **Os limites entram DEPOIS do amortecimento** — a ordem que evita o tremor na borda.
///
/// **Mutação que deve sangrar:** aplicar os limites antes do `damp_axis` em [`follow_step`].
#[test]
fn the_limits_come_after_the_damping() {
    let f = seguir(5.0, 0.0);
    let lim = CameraLimits {
        min: [-1000.0, -1000.0],
        max: [1000.0, 1000.0],
    };
    // Alvo muito para lá da cerca: o passo amortecido dá 25, que ainda está dentro ⇒ 25.
    let (_, p) = follow_step(
        [0.0, 0.0],
        [0.0, 0.0],
        [300.0, 0.0],
        HALF,
        &f,
        Some(&lim),
        DT,
    );
    assert!((p[0] - 25.0).abs() < 1e-3, "{p:?}");
    // Já saturado: a cerca segura em 1000 − 576 = 424 e o amortecimento não a atravessa.
    let (_, p) = follow_step(
        [424.0, 0.0],
        [424.0, 0.0],
        [100_000.0, 0.0],
        HALF,
        &f,
        Some(&lim),
        DT,
    );
    assert!(
        (p[0] - 424.0).abs() < 1e-3,
        "saturado devia ficar em 424: {p:?}"
    );
}
