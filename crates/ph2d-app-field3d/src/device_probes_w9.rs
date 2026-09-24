//! ⏱️⭐⭐⭐ **AS SONDAS DA `W9`** — o que MEDE as curas da wave, separado do que AFIRMA.
//!
//! ⚠️ **Elas moram aqui e não ao lado dos gates porque o tecto de LOC o obrigou** (2026-09-23,
//! `1 951` contra `700`) — e é a MESMA fronteira que o [`super::super::device_probes`] já pagou em
//! 2026-09-15, com a mesma leitura: *o [`super`] **afirma** propriedades (com barras, controlos e
//! mutações a matá-las) e isto **mede** (imprime tabelas e não afirma quase nada).*
//!
//! ⛔⛔ **E eu tinha-as escrito no ficheiro dos gates**, com aquela fronteira já estabelecida e
//! documentada uma casa acima. *Uma fronteira que existe e que o autor seguinte não vê é uma
//! fronteira que se paga duas vezes.*

use super::*;

/// ⏱️⭐⭐⭐⭐ **A PRIMEIRA CHAMADA DE UMA CENA PAGA A COMPILAÇÃO — e é ela que o gate vermelho mede.**
///
/// O [`com_o_dispositivo_a_maioria_das_cenas_e_nitida_em_movimento`] cronometra **UMA** chamada a
/// [`crate::gpu_frame::paint`] por cena e chama-lhe *«o quadro de movimento»*. ⛔⛔ Mas a primeira
/// chamada de uma cena compila o programa da placa: o cache do pintor tem por chave o **TEXTO** do
/// shader, e o texto leva a FITA da peça — logo *cada cena é um texto novo e um compilador inteiro*.
/// O quadro que o artista arrasta é o **N-ésimo**, com tudo compilado.
///
/// ⚠️⚠️ **E a `W9` já prescreve a régua certa, por escrito**
/// ([`03`](../../../docs/Render3d/03_o_plano.md)): *«`1920×1080`, **mínimo de N**, A/B intercalado
/// no MESMO processo»*. O gate não o faz — *uma régua que o próprio plano do módulo corrige e
/// ninguém emendou*.
///
/// ⇒ esta sonda pinta a MESMA cena três vezes seguidas e põe as três a par. Se a 1.ª for a cara e
/// as outras baratas, o vermelho é da régua.
#[test]
#[ignore = "sonda de diagnóstico: separa a compilação do custo do quadro"]
fn diag_o_primeiro_quadro_de_uma_cena_paga_a_compilacao() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    println!(
        "\n  cena · quadro de MOVIMENTO a {LW}×{LH} · {}",
        contexto()
    );
    println!("  cena ·      1.ª ·      2.ª ·      3.ª ·  1.ª/mín");
    let (mut pior_razao, mut nitidas_pela_1a, mut nitidas_pelo_min) = (0.0f32, 0usize, 0usize);
    for n in 0..crate::smoke::scenes::CENAS {
        if crate::smoke::scenes::PODADAS.contains(&n) {
            continue;
        }
        let doc = crate::smoke::scene(n);
        let reg = crate::smoke::sampled_registry();
        let cam = ph2d_field_render::Orbit::default();
        let luz = [crate::gpu_frame::tests_lampada(&cam)];
        let surfaces = ph2d_field_render::Surfaces {
            all: &materiais,
            owners: None,
        };
        let mut ts = Vec::new();
        for _ in 0..3 {
            let t0 = std::time::Instant::now();
            let saiu = crate::gpu_frame::paint(
                t,
                &doc,
                &reg,
                &cam,
                &luz,
                &surfaces,
                &ph2d_field_render::Presentation::of(olhar),
                BG,
                None,
                LW,
                LH,
                false,
            );
            #[allow(clippy::cast_possible_truncation)]
            let ms = t0.elapsed().as_secs_f32() * 1e3;
            if saiu.is_none() {
                break;
            }
            ts.push(ms);
        }
        if ts.len() < 3 {
            println!("  {n:>4} ·        na CPU");
            continue;
        }
        let minimo = ts.iter().copied().fold(f32::INFINITY, f32::min);
        let razao = ts[0] / minimo;
        pior_razao = pior_razao.max(razao);
        if ts[0] <= PREVIEW_BUDGET_MS {
            nitidas_pela_1a += 1;
        }
        if minimo <= PREVIEW_BUDGET_MS {
            nitidas_pelo_min += 1;
        }
        println!(
            "  {n:>4} · {:>7.2} · {:>7.2} · {:>7.2} · {razao:>7.2}x",
            ts[0], ts[1], ts[2]
        );
    }
    println!(
        "\n  ⇒ pela 1.ª chamada: {nitidas_pela_1a} nítidas · pelo MÍNIMO de 3: {nitidas_pelo_min} \
         · pior razão 1.ª/mín {pior_razao:.2}x\n"
    );
}

/// ⏱️⭐⭐⭐⭐ **O QUE O ARTISTA PAGA: arrastar um número, ou ACRESCENTAR uma forma?**
///
/// A [`diag_o_primeiro_quadro_de_uma_cena_paga_a_compilacao`] mede `1,4`–`4,4 s` na primeira
/// pintura de cada cena e `6`–`130 ms` nas seguintes, e o custo **não é da CPU**: a 2.ª chamada
/// refaz a MESMA fita (a [`crate::gpu_frame::paint`] reconstrói o `DeviceField` a cada chamada) e
/// custa `12 ms`. ⇒ *o segundo e meio é a placa a compilar o programa.*
///
/// ⛔⛔ **E a pergunta que decide se isto é um defeito de PRODUTO ou uma nota de bancada é outra:**
/// o cache do pintor tem por chave o **TEXTO** do shader, e o texto leva a FITA da peça. Um
/// **arrasto de slider** muda os números (o armazém `k`) e o texto fica igual; **acrescentar uma
/// forma** muda a árvore, logo muda a fita, logo muda o texto. *Se for assim, toda mudança de
/// ESTRUTURA custa um segundo e meio, e nenhum doc deste módulo o diz.*
///
/// Esta sonda põe as duas lado a lado: a mesma peça com o RAIO mudado, e uma peça com uma forma a
/// mais.
#[test]
#[ignore = "sonda de diagnóstico: separa mudar um NÚMERO de mudar a ESTRUTURA"]
fn diag_mudar_um_numero_contra_acrescentar_uma_forma() {
    use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::tests_lampada(&cam)];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };

    let uma_bola = |r: f32| {
        FieldDoc::new(
            vec![ph2d_field_eval::leaf(
                Primitive::Sphere { radius: r },
                Xform::at(0.0, 0.0, 0.0),
            )],
            NodeId(0),
        )
        .expect("a bola")
    };
    let pinta = |doc: &FieldDoc| -> f32 {
        let t0 = std::time::Instant::now();
        let saiu = crate::gpu_frame::paint(
            t,
            doc,
            &reg,
            &cam,
            &luz,
            &surfaces,
            &ph2d_field_render::Presentation::of(olhar),
            BG,
            None,
            LW,
            LH,
            false,
        );
        assert!(saiu.is_some(), "o pintor recusou a peça");
        #[allow(clippy::cast_possible_truncation)]
        let ms = t0.elapsed().as_secs_f32() * 1e3;
        ms
    };

    println!("\n  {}", contexto());
    println!("  gesto                              ·       ms");
    println!("  ───────────────────────────────────·─────────");
    println!(
        "  bola r=0,50, 1.ª vez (compila)     · {:>8.2}",
        pinta(&uma_bola(0.50))
    );
    println!(
        "  a MESMA, outra vez                 · {:>8.2}",
        pinta(&uma_bola(0.50))
    );
    println!(
        "  ARRASTAR o raio: r=0,60            · {:>8.2}",
        pinta(&uma_bola(0.60))
    );
    println!(
        "  arrastar outra vez: r=0,70         · {:>8.2}",
        pinta(&uma_bola(0.70))
    );

    // ⚠️ A forma a mais é o que muda a ÁRVORE. Duas folhas unidas: é a estrutura mais barata que
    // deixa de ser a fita de uma bola.
    let duas = FieldDoc::new(
        vec![
            ph2d_field_eval::leaf(Primitive::Sphere { radius: 0.5 }, Xform::at(-0.3, 0.0, 0.0)),
            ph2d_field_eval::leaf(Primitive::Sphere { radius: 0.4 }, Xform::at(0.3, 0.0, 0.0)),
            ph2d_field::Node {
                xform: Xform::IDENTITY,
                kind: ph2d_field::NodeKind::Combine {
                    op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1)],
                },
                mods: Vec::new(),
                verb: None,
            },
        ],
        NodeId(2),
    )
    .expect("as duas");
    println!(
        "  ACRESCENTAR uma forma (1.ª vez)    · {:>8.2}",
        pinta(&duas)
    );
    println!(
        "  a MESMA peça de duas, outra vez    · {:>8.2}",
        pinta(&duas)
    );
    println!(
        "  voltar à bola r=0,50               · {:>8.2}\n",
        pinta(&uma_bola(0.50))
    );
}

/// ⏱️⭐⭐⭐⭐ **A FITA INERTE TAMBÉM PAGA NO RELÓGIO DO QUADRO? — o A/B no MESMO processo.**
///
/// A cura da fita inerte (`docs/Render3d/03` §W9) foi construída para o **COMPILADOR**: ela tira a
/// peça do texto do pintor e o cache passa a acertar (`1 406 → 74 ms` ao acrescentar uma forma).
/// ⭐ **Mas ela também encolhe o kernel que corre**, e o modelo de custo do quadro de movimento diz
/// que o custo segue o TAMANHO da fita (`R² 0,803` com `instruções + transcendentes + raízes`, e
/// `R² 0,165` com os passos da marcha). ⇒ *se o modelo estiver certo, a cura tem um segundo prémio
/// que ninguém pediu.*
///
/// ⚠️ **O A/B corre no MESMO processo e INTERCALADO**, que é a régua que a `W9` prescreve: subtrair
/// dois relógios de corridas separadas dá a soma dos ruídos. E cada lado tira o **MÍNIMO de
/// [`QUADROS_MEDIDOS`]**, senão isto mede a compilação.
///
/// ⛔ **A ordem é `sem` → `com` dentro de cada cena**, e as duas primeiras chamadas de cada lado
/// pagam a compilação daquele texto: é por isso que o mínimo é obrigatório aqui, e não um luxo.
#[test]
#[ignore = "sonda de diagnóstico: mede se a fita inerte paga no relógio do quadro"]
fn diag_a_fita_inerte_no_relogio_do_quadro() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    println!("\n  {}", contexto());
    println!("  cena ·  com a fita ·  sem a fita ·  razão ·  instr · transc");
    let (mut melhor, mut pior) = (f32::INFINITY, 0.0f32);
    for n in 0..crate::smoke::scenes::CENAS {
        if crate::smoke::scenes::PODADAS.contains(&n) {
            continue;
        }
        let doc = crate::smoke::scene(n);
        let reg = crate::smoke::sampled_registry();
        let cam = ph2d_field_render::Orbit::default();
        let luz = [crate::gpu_frame::tests_lampada(&cam)];
        let surfaces = ph2d_field_render::Surfaces {
            all: &materiais,
            owners: None,
        };
        let minimo = |fita_inerte: bool| {
            (0..QUADROS_MEDIDOS)
                .map(|_| {
                    let t0 = std::time::Instant::now();
                    let saiu = crate::gpu_frame::paint_com(
                        t,
                        &doc,
                        &reg,
                        &cam,
                        &luz,
                        &surfaces,
                        &ph2d_field_render::Presentation::of(olhar),
                        BG,
                        None,
                        LW,
                        LH,
                        false,
                        crate::gpu_frame::Sonda {
                            fita_inerte,
                            ..crate::gpu_frame::Sonda::default()
                        },
                    );
                    #[allow(clippy::cast_possible_truncation)]
                    let ms = t0.elapsed().as_secs_f32() * 1e3;
                    saiu.map(|_| ms)
                })
                .collect::<Option<Vec<f32>>>()
                .map(|v| v.into_iter().fold(f32::INFINITY, f32::min))
        };
        let (Some(sem), Some(com)) = (minimo(false), minimo(true)) else {
            println!("  {n:>4} ·        na CPU");
            continue;
        };
        let campo = ph2d_field_eval::device::DeviceField::new(&doc, &reg).expect("a peça");
        let fita = campo.tape_wgsl().expect("a fita");
        let instrs = fita.source.lines().count();
        let transc = fita
            .source
            .lines()
            .filter(|l| {
                [
                    "sin(", "cos(", "tan(", "asin(", "acos(", "atan", "exp(", "log(", "pow(",
                ]
                .iter()
                .any(|f| l.contains(f))
            })
            .count();
        let razao = sem / com;
        melhor = melhor.min(razao);
        pior = pior.max(razao);
        println!(
            "  {n:>4} · {com:8.2} ms · {sem:8.2} ms · {razao:5.2}x · {instrs:>6} · {transc:>6}"
        );
    }
    println!("  ⇒ a razão sem/com vai de {melhor:.2}x a {pior:.2}x\n");
}

/// ⏱️⭐⭐⭐⭐ **A CURA DA FITA INERTE SOBREVIVE AO CAMINHO REAL? — a peça com LEI DO DONO.**
///
/// ⛔⛔ **A medição que decidiu a cura correu com `owners: None`, e o produto põe lei do dono em
/// TODA peça com mais de uma folha** ([`crate::materials::Table::build`]:
/// `(placed.len() > 1).then(...)`). E a lei do dono emite **uma FITA INTEIRA por folha**
/// (`dono_folha_0`, `dono_folha_1`, …, em [`ph2d_field_eval::owners_wgsl`]) ⇒ *o texto do shader do
/// pintor volta a levar a geometria da peça, N vezes, e a cura pode não alcançar o caso do artista.*
///
/// ⇒ esta sonda mede o MESMO gesto — acrescentar uma forma — com a lei do dono montada como o
/// produto a monta, e põe as quatro células a par.
///
/// ⚠️ **A 1.ª pintura de cada peça é a que interessa aqui** (é ela que o artista espera), logo não
/// há mínimo de N: o que se quer medir **é** a compilação.
#[test]
#[ignore = "sonda de diagnóstico: mede a cura da fita inerte no caminho com lei do dono"]
fn diag_a_fita_inerte_com_a_lei_do_dono() {
    use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::tests_lampada(&cam)];

    // Uma peça de `n` bolas unidas — e as folhas POSTAS, que é o que a lei do dono recebe.
    let peca = |n: usize| -> (FieldDoc, Vec<FieldDoc>) {
        #[allow(clippy::cast_precision_loss)]
        let folhas: Vec<ph2d_field::Node> = (0..n)
            .map(|i| {
                ph2d_field_eval::leaf(
                    Primitive::Sphere { radius: 0.35 },
                    Xform::at(i as f32 * 0.4 - 0.4, 0.0, 0.0),
                )
            })
            .collect();
        let mut nos = folhas.clone();
        let mut raiz = NodeId(0);
        for i in 1..n {
            nos.push(ph2d_field::Node {
                xform: Xform::IDENTITY,
                kind: ph2d_field::NodeKind::Combine {
                    op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                    children: vec![raiz, NodeId(i as u32)],
                },
                mods: Vec::new(),
                verb: None,
            });
            raiz = NodeId((nos.len() - 1) as u32);
        }
        let postas = folhas
            .into_iter()
            .map(|f| FieldDoc::new(vec![f], NodeId(0)).expect("a folha"))
            .collect();
        (FieldDoc::new(nos, raiz).expect("a peça"), postas)
    };

    println!("\n  {}", contexto());
    println!("  folhas · lei do dono ·  fita inerte ·   1.ª pintura");
    println!("  ───────·─────────────·──────────────·───────────────");
    for com_dono in [false, true] {
        for fita_inerte in [false, true] {
            for n in [2usize, 3] {
                let (doc, postas) = peca(n);
                let owners = com_dono.then(|| {
                    ph2d_field_eval::owners::Owners::new(
                        &postas,
                        &reg,
                        ph2d_field_render::hit_tolerance(
                            cam.half_extent,
                            f32::from(u16::try_from(LH).unwrap_or(u16::MAX)),
                        ),
                    )
                });
                let mats: Vec<ph2d_material::Surface> = (0..n)
                    .map(|_| ph2d_material::OpenPbr::default().prepare())
                    .collect();
                let surfaces = ph2d_field_render::Surfaces {
                    all: &mats,
                    owners: owners.as_ref(),
                };
                let t0 = std::time::Instant::now();
                let saiu = crate::gpu_frame::paint_com(
                    t,
                    &doc,
                    &reg,
                    &cam,
                    &luz,
                    &surfaces,
                    &ph2d_field_render::Presentation::of(olhar),
                    BG,
                    None,
                    LW,
                    LH,
                    false,
                    crate::gpu_frame::Sonda {
                        fita_inerte,
                        ..crate::gpu_frame::Sonda::default()
                    },
                );
                #[allow(clippy::cast_possible_truncation)]
                let ms = t0.elapsed().as_secs_f32() * 1e3;
                println!(
                    "  {n:>6} · {:>11} · {:>12} · {}",
                    if com_dono { "SIM" } else { "não" },
                    if fita_inerte { "SIM" } else { "não" },
                    if saiu.is_some() {
                        format!("{ms:10.2} ms")
                    } else {
                        "     na CPU".to_string()
                    }
                );
            }
        }
    }
    println!();
}

/// ⏱️ **As sondas que medem uma CENA** — ver o cabeçalho do [`cenas`].
#[path = "device_probes_w9_cenas.rs"]
mod cenas;

/// ⏱️⭐⭐⭐⭐ **QUANTOS QUADROS A RÉGUA PRECISA? — a tabela que o [`super::QUADROS_MEDIDOS`] devia ter.**
///
/// ⛔⛔ **Esta sonda nasceu de uma auditoria (2026-09-23).** O doc daquela constante diz que o `3` é
/// *«o joelho MEDIDO»* e o que existia era uma OBSERVAÇÃO (uma cena leu `176,62` e depois
/// `9,84 ms`) — ela justifica *«mais do que um»* e não diz nada sobre `4`. *O `CLAUDE.md` §0.0 manda
/// escrever o número que a medição deu, com a tabela ao lado; um número sem tabela é um palpite.*
///
/// ⚠️ **A régua corre no MESMO processo e por cena**, com os `N` intercalados: o que se quer saber é
/// se o VEREDITO do gate se move com `N`, e uma corrida por `N` mediria a deriva da máquina.
#[test]
#[ignore = "sonda de diagnóstico: mede o joelho do QUADROS_MEDIDOS"]
fn diag_quantos_quadros_a_regua_precisa() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    const N_MAX: usize = 5;
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    println!("\n  {}", contexto());
    // Por cena, os `N_MAX` relógios seguidos; o mínimo dos primeiros `n` dá a leitura daquele `n`.
    let mut por_cena: Vec<Vec<f32>> = Vec::new();
    for n in 0..crate::smoke::scenes::CENAS {
        if crate::smoke::scenes::PODADAS.contains(&n) {
            continue;
        }
        let doc = crate::smoke::scene(n);
        let reg = crate::smoke::sampled_registry();
        let cam = ph2d_field_render::Orbit::default();
        let luz = [crate::gpu_frame::tests_lampada(&cam)];
        let surfaces = ph2d_field_render::Surfaces {
            all: &materiais,
            owners: None,
        };
        let mut ts = Vec::with_capacity(N_MAX);
        for _ in 0..N_MAX {
            let t0 = std::time::Instant::now();
            let saiu = crate::gpu_frame::paint(
                t,
                &doc,
                &reg,
                &cam,
                &luz,
                &surfaces,
                &ph2d_field_render::Presentation::of(olhar),
                BG,
                None,
                LW,
                LH,
                false,
            );
            if saiu.is_none() {
                break;
            }
            #[allow(clippy::cast_possible_truncation)]
            ts.push(t0.elapsed().as_secs_f32() * 1e3);
        }
        if ts.len() == N_MAX {
            por_cena.push(ts);
        }
    }
    println!(
        "     N ·  nítidas de {} ·  pior cena ·  mediana",
        por_cena.len()
    );
    for n in 1..=N_MAX {
        let mins: Vec<f32> = por_cena
            .iter()
            .map(|ts| ts[..n].iter().copied().fold(f32::INFINITY, f32::min))
            .collect();
        let nitidas = mins.iter().filter(|m| **m <= PREVIEW_BUDGET_MS).count();
        let pior = mins.iter().copied().fold(0.0f32, f32::max);
        let mut ord = mins.clone();
        ord.sort_by(f32::total_cmp);
        println!(
            "  {n:>4} · {nitidas:>14} · {pior:>8.2} ms · {:>7.2} ms",
            ord[ord.len() / 2]
        );
    }
    println!();
}

/// ⏱️⭐⭐⭐⭐ **O tecto da wave do TORNO** — ver o cabeçalho do [`torno`].
#[path = "device_probes_w9_torno.rs"]
mod torno;

/// ⏱️⭐⭐⭐⭐ **A grade assada contra a árvore** — ver o cabeçalho do [`grade`].
#[path = "device_probes_w9_grade.rs"]
mod grade;

/// ⏱️⭐⭐⭐⭐ **Onde os passos da marcha acontecem** — ver o cabeçalho do [`perto`].
#[path = "device_probes_w9_perto.rs"]
mod perto;
