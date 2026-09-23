//! **§6 do plano 24 — as MEDIÇÕES que a W5 exige antes de abrir.** Duas delas vivem aqui; a
//! terceira (o valor de `z₀`) é decisão do dono e está no handoff.
//!
//! ⚠️ **Elas correm como gate e ficam versionadas**, porque um número que decide uma wave e não é
//! gateado envelhece com o produto — e o `§0.0` exige a medição ANTES de qualquer tecto.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, ScrollFactor, SimWorld, Transform};
use ph2d_preview_drive::PreviewDrive;

use super::drive_parallax;

/// ⭐⭐⭐ **O CUSTO POR QUADRO com N camadas** (§6.2 do plano) — e ele nomeia o recurso.
///
/// O passe é `O(objectos com o componente)` e nunca `O(cena)`: a consulta é filtrada e o laço
/// visita só quem tem `ScrollFactor`. ⚠️ **É a diferença que a `Factory` já pagou** — a cópia dela
/// varria o mundo duas vezes por nascimento, e a porta em lote tornou-a plana.
///
/// ⚠️ **Sem tecto e sem `MAX_*`, de propósito:** um fundo de paralaxe tem `3` a `8` camadas num jogo
/// real, e o que esta medição existe para dizer é que **nenhum `MAX_*` é preciso** — a tabela vive no
/// [`custo_por_quadro_imprime_a_tabela`], que é o que o `§0.0` pede em vez de um limite escolhido.
///
/// ⛔⛔ **Este gate CONTA TRABALHO e não mede relógio** (auditoria 26, §2.9). A 1.ª redacção
/// comparava dois relógios de dezenas de µs e reprovou `1` de `25` a `load 26–36`, com zero linhas
/// de diff no passe — *a família de flakes de fan-out que o §5.0 do roteador nomeia*. O custo aqui
/// é **estrutural** (uma consulta filtrada custa `O(arquétipos + linhas que casam)`, e a cena sem o
/// componente é UM arquétipo), logo a régua honesta tem duas metades:
///
/// 1. **a ESTRUTURA** — o passe lê o mundo por uma consulta que exige `&ScrollFactor` e nunca por
///    uma varredura de entidades (`iter_entities`), que é a forma exacta do defeito da `Factory`;
/// 2. **o TRABALHO** — com vinte mil objectos sem o componente, o passe conduz exactamente as dez
///    camadas; com mil camadas conduz mil.
///
/// ⚠️ O relógio fica no irmão `#[ignore]`, que imprime e não afirma.
#[test]
fn o_passe_e_uma_consulta_filtrada_e_conta_so_as_camadas() {
    // (1) A ESTRUTURA, lida do produto. ⚠️ A agulha monta-se em runtime, senão este ficheiro — que
    // a contém — seria a prova de si mesmo.
    let fonte = include_str!("parallax_bridge.rs");
    let corpo = fonte
        .split("pub fn drive_parallax(")
        .nth(1)
        .expect("o passe existe com este nome");
    let varredura = ["iter", "_entities"].concat();
    assert!(
        !corpo.contains(&varredura),
        "o passe voltou a VARRER o mundo — e' o defeito que a `Factory` pagou (O(cena) por quadro)"
    );
    let consulta = ["query_filtered::<", "("].concat();
    let ancora = corpo
        .find(&consulta)
        .expect("o passe le por uma consulta filtrada");
    let tuplo = &corpo[ancora..ancora + 400];
    assert!(
        tuplo.contains("&ScrollFactor,"),
        "a consulta do passe deixou de EXIGIR o `ScrollFactor` — um `Option<&ScrollFactor>` casaria          com toda a cena"
    );

    // (2) O TRABALHO, contado.
    fn conduz(camadas: usize, resto_da_cena: usize) -> (usize, usize) {
        let mut sim = SimWorld::default();
        for i in 0..camadas {
            #[allow(clippy::cast_precision_loss)]
            let x = i as f32;
            sim.world_mut().spawn((
                ScrollFactor { k: [0.5, 0.5] },
                Transform {
                    translation: Vec2::new(x, 0.0),
                    ..Transform::default()
                },
            ));
        }
        // ⚠️ O resto da cena NÃO tem o componente — é ele que prova que o passe não a conta.
        for _ in 0..resto_da_cena {
            sim.world_mut().spawn((Transform::default(),));
        }
        let mut drive = PreviewDrive::default();
        let n = drive_parallax(&mut sim, Some(([7.0, 0.0], [10.0, 10.0])), 0.0, &mut drive);
        let conduzidos = drive
            .driven_by(ph2d_preview_drive::Driver::ParallaxPose)
            .len();
        (n, conduzidos)
    }
    assert_eq!(conduz(10, 0), (10, 10));
    assert_eq!(
        conduz(10, 20_000),
        (10, 10),
        "vinte mil objectos SEM o componente mudaram o trabalho do passe"
    );
    assert_eq!(conduz(1_000, 0), (1_000, 1_000));
}

/// ⭐⭐⭐ **A composição com o `Transform` de um PAI** (§6.3 do plano) — e a resposta é DECLARADA.
///
/// # ⚠️ A pose conduzida é LOCAL, logo o deslocamento herda a moldura do pai
///
/// Esta ponte escreve o `Transform` do objecto, que é **local ao pai**; a propagação da casa aplica
/// a transformada do pai por cima. ⇒ um pai **rodado** roda o deslocamento da paralaxe, e um pai
/// **escalado** escala-o.
///
/// ⛔ **Isso é uma decisão e não um acidente, e as duas saídas foram pesadas:**
///
/// * **herdar** (o que ship) — um fundo dentro de um grupo inclinado desloca-se no eixo do grupo, e
///   toda a hierarquia da casa (o undo, o reparentar, o gizmo) continua a valer sem uma linha;
/// * **desfazer o pai** — pediria a transformada de MUNDO aqui, que é `O(profundidade)` por objecto
///   e por quadro, e poria esta fase a depender da propagação, que corre **depois** dela.
///
/// ⚠️ **O caso do artista é um pai IDENTIDADE** (um fundo é quase sempre raiz, ou filho de um grupo
/// sem pose), e ali as duas saídas coincidem **ao bit**. É por isso que a decisão pode ficar
/// declarada em vez de resolvida — e é por isso que ela tem gate: *uma decisão sem régua lê-se como
/// um acidente no dia em que alguém a encontrar*.
#[test]
fn a_pose_conduzida_e_local_e_o_pai_compoe_por_cima() {
    let mut sim = SimWorld::default();
    let pai = sim
        .world_mut()
        .spawn((Transform {
            translation: Vec2::new(1000.0, 0.0),
            rotation: 1.2,
            scale: Vec2::new(2.0, 2.0),
            ..Transform::default()
        },))
        .id();
    let filho = sim
        .world_mut()
        .spawn((
            ScrollFactor { k: [0.5, 1.0] },
            Transform::default(),
            ChildOf(pai),
        ))
        .id();
    let mut drive = PreviewDrive::default();
    drive_parallax(
        &mut sim,
        Some(([400.0, 0.0], [10.0, 10.0])),
        0.0,
        &mut drive,
    );
    let t = *sim.world().get::<Transform>(filho).expect("pose");
    assert!(
        (t.translation.x - 200.0).abs() < 1e-3,
        "a pose escrita deixou de ser LOCAL: {} — se ela passar a descontar o pai, esta fase \
         precisa da transformada de MUNDO, que a propagacao so' calcula DEPOIS dela",
        t.translation.x
    );
    // ⭐ E o CONTROLO que torna a decisão barata: com o pai na identidade as duas saídas coincidem.
    let mut sim2 = SimWorld::default();
    let raiz = sim2.world_mut().spawn((Transform::default(),)).id();
    let filho2 = sim2
        .world_mut()
        .spawn((
            ScrollFactor { k: [0.5, 1.0] },
            Transform::default(),
            ChildOf(raiz),
        ))
        .id();
    let mut drive2 = PreviewDrive::default();
    drive_parallax(
        &mut sim2,
        Some(([400.0, 0.0], [10.0, 10.0])),
        0.0,
        &mut drive2,
    );
    assert_eq!(
        sim2.world()
            .get::<Transform>(filho2)
            .expect("pose")
            .translation,
        t.translation,
        "com o pai na identidade as duas leituras divergiram"
    );
}

/// **A TABELA do custo, para o handoff** — ela imprime e não afirma; quem afirma é o gate acima.
///
/// ⚠️ `#[ignore]` porque um relógio de parede sob fan-out é a família de flakes que este repo já
/// conta uma a uma. Corre-se à mão:
/// `cargo test -p ph2d-app-components --release -- --ignored custo_por_quadro --nocapture`
#[test]
#[ignore = "relogio de parede: corre-se a` mao, com a maquina calma"]
fn custo_por_quadro_imprime_a_tabela() {
    println!("camadas | resto da cena |   por quadro");
    for (camadas, resto) in [
        (1_usize, 0_usize),
        (8, 0),
        (8, 20_000),
        (100, 0),
        (1_000, 0),
        (10_000, 0),
    ] {
        let mut sim = SimWorld::default();
        for _ in 0..camadas {
            sim.world_mut()
                .spawn((ScrollFactor { k: [0.5, 0.5] }, Transform::default()));
        }
        for _ in 0..resto {
            sim.world_mut().spawn((Transform::default(),));
        }
        let mut drive = PreviewDrive::default();
        drive_parallax(&mut sim, Some(([1.0, 0.0], [10.0, 10.0])), 0.0, &mut drive);
        let n = 200;
        let t0 = std::time::Instant::now();
        for i in 1..=n {
            #[allow(clippy::cast_precision_loss)]
            let c = i as f32;
            drive_parallax(&mut sim, Some(([c, 0.0], [10.0, 10.0])), 0.0, &mut drive);
        }
        let por_quadro = t0.elapsed().as_secs_f64() / f64::from(n) * 1e6;
        println!("{camadas:>7} | {resto:>13} | {por_quadro:>9.2} us");
    }
}
