//! Os gates da ponte da paralaxe, parte 3: o movimento próprio (W4) e o dolly (W5).
//!
//! ⚠️ **Módulo-FILHO do [`super`] por tecto de LOC** (o ficheiro-pai chegou a `1359` contra `700`),
//! e o corte é por WAVE, que é como ele já se lia. ⭐ Filho e não irmão de propósito: os ajudantes
//! (`cena`, `pose_em`, `com_camera`, as constantes `PARADO`/`SEM_LIMITE`) ficam UMA vez, no pai —
//! duas cópias de um arnês divergem, e um gate que mede o arnês errado fica verde a medir nada.

#![allow(clippy::wildcard_imports)]
use super::*;

// ══ O MOVIMENTO PRÓPRIO (plano 24, W4) ═════════════════════════════════════════════════════════

fn cena_mov(k: [f32; 2], v: [f32; 2], pose: Transform) -> (SimWorld, ph2d_ecs::Entity) {
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((ScrollFactor { k }, ScrollMotion { velocity: v }, pose))
        .id();
    (sim, e)
}

fn pose_em_t(k: [f32; 2], v: [f32; 2], centro: [f32; 2], t: f64) -> Vec2 {
    let (mut sim, e) = cena_mov(k, v, pose_em(0.0, 0.0));
    let mut drive = PreviewDrive::default();
    drive_parallax(&mut sim, Some((centro, SEM_LIMITE)), t, &mut drive);
    sim.world().get::<Transform>(e).expect("pose").translation
}

/// ⭐⭐⭐ **A deriva é `velocidade × playhead`, e ela é PURA** — o mesmo instante dá a mesma pose,
/// e a ordem em que os instantes são pedidos não conta.
///
/// ⚠️ **É esta propriedade que se compra**, e não o efeito: o *autoscroll* do alvo move `+0,000` nos
/// quatro observáveis (medido, §4.6) porque vive no caminho de DESENHO. O nosso é medível, e é isso
/// que faz este gate existir.
#[test]
fn a_deriva_e_velocidade_vezes_o_playhead() {
    let v = [3.0_f32, -1.5];
    for t in [0.0_f64, 0.5, 2.0, 10.0, 3600.0] {
        let p = pose_em_t(ScrollFactor::NEUTRO, v, [0.0, 0.0], t);
        #[allow(clippy::cast_possible_truncation)]
        let esperado = Vec2::new((f64::from(v[0]) * t) as f32, (f64::from(v[1]) * t) as f32);
        assert_eq!(p, esperado, "t = {t}");
    }

    // ⭐⭐⭐ **O PRODUTO é feito em `f64`, e o corpus de cima NÃO o discriminava** — uma mutação
    // que o fizesse em `f32` SOBREVIVEU, porque `3,0 × 3 600` é exacto nas duas larguras.
    //
    // ⚠️ **A deriva é a única grandeza desta família que cresce sem limite com o tempo**, e é ela
    // que paga o `f64`: ao fim de uma hora de relógio o ULP de um `f32` é `2,4e-4`, e arredondar o
    // PRODUTO em vez do resultado é o que o dobra. Os pares abaixo são **medidos** — de `72`
    // células, `10` separam as duas larguras, e a pior é `3,9e-3` (uma nuvem a `12,3 m/s` ao fim de
    // uma hora). *Um corpus de números redondos não distingue duas aritméticas.* ⚠️ Cada esperado
    // está escrito com a MENOR decimal que dá os mesmos bits de `f32` — o valor é exacto, o texto é
    // curto (o clippy recusa a expansão decimal inteira como precisão excessiva).
    let mut discriminantes = 0;
    for (v, t, esperado) in [
        (12.345_f32, 3600.1_f64, 44_443.234_f32),
        (3.0, 3600.1, 10_800.3),
        (12.345, 1_234.567_8, 15_240.74),
        (0.3, 1_234.567_8, 370.370_36),
    ] {
        let p = pose_em_t(ScrollFactor::NEUTRO, [v, 0.0], [0.0, 0.0], t);
        assert_eq!(
            p.x, esperado,
            "v = {v}, t = {t} — o produto deixou de ser feito em `f64`"
        );
        #[allow(clippy::cast_possible_truncation)]
        if v * (t as f32) != esperado {
            discriminantes += 1;
        }
    }
    assert_eq!(
        discriminantes, 4,
        "os pares deixaram de separar as duas larguras: este gate voltou a nao poder ver a \
         diferenca que ele existe para guardar"
    );
}

/// ⭐⭐⭐ **SCRUBBAR PARA TRÁS DESFAZ A DERIVA** — a propriedade que o alvo não tem, e a razão de a
/// lei não ter estado.
///
/// ⚠️ Um acumulador (`pos += v·dt`) daria uma nuvem que **continua a andar** quando o artista puxa a
/// régua para trás, e duas máquinas com quadros diferentes veriam nuvens diferentes. *Este gate é a
/// diferença entre uma deriva e um contador.*
#[test]
fn um_scrub_para_tras_desfaz_a_deriva() {
    let v = [3.0_f32, 0.0];
    let (mut sim, e) = cena_mov(ScrollFactor::NEUTRO, v, pose_em(0.0, 0.0));
    let mut drive = PreviewDrive::default();
    let mut pose_em_instante = |t: f64| {
        drive_parallax(&mut sim, Some(([0.0, 0.0], SEM_LIMITE)), t, &mut drive);
        sim.world().get::<Transform>(e).expect("pose").translation.x
    };
    // ⚠️ A régua percorre os instantes FORA de ordem, de propósito: uma lei com acumulador passa
    // a subir e nunca a descer, e uma varredura monótona não a distinguiria da pura.
    let ida = pose_em_instante(10.0);
    let volta = pose_em_instante(2.0);
    let repete = pose_em_instante(10.0);
    let zero = pose_em_instante(0.0);
    assert!((ida - 30.0).abs() < 1e-3, "t=10 leu {ida}");
    assert!(
        (volta - 6.0).abs() < 1e-3,
        "t=2 depois de t=10 leu {volta} — a deriva ACUMULOU"
    );
    assert!((repete - 30.0).abs() < 1e-3, "voltar a t=10 leu {repete}");
    assert!(
        zero.abs() < 1e-3,
        "rebobinar leu {zero} — a nuvem nao voltou ao principio"
    );
}

/// ⭐⭐ **A deriva SOMA-SE à paralaxe** — ela não é um segundo condutor.
///
/// ⚠️ Medido no [`super::w4_probe`]: dois motores sobre o mesmo `Transform` entram no ledger com
/// chaves diferentes, e esta ponte leria a escrita do outro como um arrasto do artista.
#[test]
fn a_deriva_soma_se_ao_deslocamento_da_camera() {
    let k = [0.5_f32, 0.5];
    let v = [3.0_f32, 0.0];
    let centro = [400.0_f32, 0.0];
    let so_camera = pose_em_t(k, [0.0, 0.0], centro, 10.0);
    let so_deriva = pose_em_t(k, v, [0.0, 0.0], 10.0);
    let ambas = pose_em_t(k, v, centro, 10.0);
    assert!(
        (so_camera.x - 200.0).abs() < 1e-3,
        "so' camera: {}",
        so_camera.x
    );
    assert!(
        (so_deriva.x - 30.0).abs() < 1e-3,
        "so' deriva: {}",
        so_deriva.x
    );
    assert!(
        (ambas.x - 230.0).abs() < 1e-3,
        "as duas juntas leem {} e nao a SOMA (230)",
        ambas.x
    );
}

/// ⛔⛔ **Um objecto com deriva é conduzido MESMO com `k` neutro** — a metade que o salto do neutro
/// esconderia.
///
/// ⚠️ `k = 1` diz *«não guardo nada do movimento da CÂMERA»*, e a deriva não é movimento da câmera.
/// Sem esta metade uma nuvem que anda sozinha num plano normal ficaria parada — e o painel diria
/// que está a andar.
#[test]
fn a_deriva_acorda_um_objecto_de_k_neutro() {
    let (mut sim, e) = cena_mov(ScrollFactor::NEUTRO, [3.0, 0.0], pose_em(0.0, 0.0));
    let mut drive = PreviewDrive::default();
    assert_eq!(
        drive_parallax(&mut sim, Some(([0.0, 0.0], SEM_LIMITE)), 10.0, &mut drive),
        1,
        "um objecto com deriva e `k` neutro nao foi conduzido"
    );
    assert!((sim.world().get::<Transform>(e).expect("pose").translation.x - 30.0).abs() < 1e-3);
    // ⭐ E o CONTROLO da omissão: `k` neutro **sem** deriva continua a ser saltado, byte-idêntico.
    let (mut sim2, e2) = cena_mov(ScrollFactor::NEUTRO, [0.0, 0.0], pose_em(-3.0, 7.0));
    let mut drive2 = PreviewDrive::default();
    assert_eq!(
        drive_parallax(
            &mut sim2,
            Some(([400.0, 0.0], SEM_LIMITE)),
            10.0,
            &mut drive2
        ),
        0
    );
    assert_eq!(
        *sim2.world().get::<Transform>(e2).expect("pose"),
        pose_em(-3.0, 7.0)
    );
    assert!(drive2.is_empty());
}

/// ⭐ **A deriva entra ANTES da repetição** — quem anda para sempre é precisamente quem tem de
/// envolver, e envolver antes de somar deixaria a nuvem a fugir.
#[test]
fn uma_nuvem_que_deriva_e_repete_nao_foge() {
    let tile = 256.0_f32;
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((
            ScrollFactor {
                k: ScrollFactor::NEUTRO,
            },
            ScrollMotion {
                velocity: [3.0, 0.0],
            },
            ScrollRepeat { tile: [tile, 0.0] },
            pose_em(0.0, 0.0),
        ))
        .id();
    let mut drive = PreviewDrive::default();
    for t in [0.0_f64, 60.0, 600.0, 6000.0] {
        drive_parallax(&mut sim, Some(([0.0, 0.0], SEM_LIMITE)), t, &mut drive);
        let x = sim.world().get::<Transform>(e).expect("pose").translation.x;
        assert!(
            x.abs() <= tile / 2.0 + 1e-3,
            "a t = {t} a nuvem fugiu para {x} — a repeticao deixou de a apanhar"
        );
    }
    // O CONTROLO: sem a repetição, aos `6 000 s` ela está a `18 000` do sítio.
    let solta = pose_em_t(ScrollFactor::NEUTRO, [3.0, 0.0], [0.0, 0.0], 6000.0);
    assert!((solta.x - 18_000.0).abs() < 1.0, "solta: {}", solta.x);
}

// ══ O DOLLY — a multiplano (plano 24, W5 · §2) ══════════════════════════════════════════════════

/// Uma cena com câmera de jogo, que é de onde o dolly sai.
fn cena_dolly(k: [f32; 2], dolly: f32, pose: Transform) -> (SimWorld, ph2d_ecs::Entity) {
    let mut sim = SimWorld::default();
    sim.world_mut().spawn((
        ph2d_ecs::GameCamera {
            dolly,
            ..ph2d_ecs::GameCamera::default()
        },
        ph2d_ecs::StableId(1),
        Transform::default(),
    ));
    let e = sim.world_mut().spawn((ScrollFactor { k }, pose)).id();
    (sim, e)
}

/// ⭐⭐⭐ **A ESCALA do dolly é `(1 − δ)/(1 − k·δ)`, e ela foi verificada em aritmética EXACTA.**
///
/// A tabela sai de uma varredura em `Fraction` do Python contra a definição geométrica
/// (`[z/(z−d)] ÷ [z₀/(z₀−d)]`), com `12` células a concordarem ao racional — a fórmula fechada é a
/// definição e não uma aproximação dela.
#[test]
fn a_escala_do_dolly_bate_a_geometria() {
    for (k, delta, esperado) in [
        (1.0_f32, 0.0_f32, 1.0_f32),
        (1.0, 0.5, 1.0),
        (1.0, -1.0, 1.0),
        (0.5, 0.5, 2.0 / 3.0),
        (0.5, -1.0, 4.0 / 3.0),
        (0.25, 0.5, 4.0 / 7.0),
        (0.25, -1.0, 1.6),
        (0.01, 0.5, 0.502_512_6),
    ] {
        let e = ScrollFactor::escala_do_dolly(k, delta).expect("a camera nao atravessa");
        assert!(
            (e - esperado).abs() < 1e-6,
            "k = {k}, delta = {delta}: {e} contra {esperado}"
        );
    }
}

/// ⛔⛔ **A DEGENERESCÊNCIA QUE O PLANO PUBLICA ESTÁ REFUTADA PELA FÓRMULA DELE** — e este gate é a
/// refutação, não uma nota.
///
/// O §2 do plano diz *«`k = 0` é `z = ∞` ⇒ escala ≡ 1 para todo `d` (o que está infinitamente longe
/// nunca muda de tamanho)»*. ⚠️ **O parêntesis é verdade e a conclusão não:** o tamanho ABSOLUTO não
/// muda, e a `escala` desta lei é RELATIVA ao plano focal — que CRESCEU. A fórmula fechada dá
/// `1 − δ`, e o limite numérico confirma-o (`k = 1/10` → `0,5263`; `1/10⁴` → `0,50003`).
///
/// ⭐ **E não há um braço `if k == 0`:** a fórmula fechada já contém a lei, logo ela não pode
/// divergir do limite. *Uma degenerescência escrita num ramo é a segunda resposta à mesma pergunta.*
#[test]
fn o_ceu_encolhe_relativamente_ao_plano_focal_e_o_plano_dizia_o_contrario() {
    let delta = 0.5_f32;
    let ceu = ScrollFactor::escala_do_dolly(0.0, delta).expect("o ceu nao e' atravessado");
    assert!(
        (ceu - (1.0 - delta)).abs() < 1e-6,
        "o ceu leu {ceu} e a formula da' 1 − delta = {}",
        1.0 - delta
    );
    assert!(
        (ceu - 1.0).abs() > 0.4,
        "o ceu leu {ceu}, que e' o que o plano prometia — a refutacao evaporou"
    );
    // ⭐ O LIMITE: a fórmula fechada e o `k` a tender para zero encontram-se.
    for n in [10.0_f32, 100.0, 10_000.0] {
        let quase = ScrollFactor::escala_do_dolly(1.0 / n, delta).expect("nao atravessa");
        assert!(
            (quase - ceu).abs() < 2.0 / n,
            "k = 1/{n}: {quase} nao converge para {ceu}"
        );
    }
}

/// ⭐⭐⭐ **A MULTIPLANO: dois planos, um dolly, e a razão dos tamanhos bate a fórmula** — o gate que
/// o plano encomendou.
#[test]
fn dois_planos_com_um_dolly_dao_a_razao_da_formula() {
    let delta = 0.5_f32;
    let mut sim = SimWorld::default();
    sim.world_mut().spawn((
        ph2d_ecs::GameCamera {
            dolly: delta,
            ..ph2d_ecs::GameCamera::default()
        },
        ph2d_ecs::StableId(1),
        Transform::default(),
    ));
    let perto = sim
        .world_mut()
        .spawn((ScrollFactor { k: [1.0, 1.0] }, pose_em(0.0, 0.0)))
        .id();
    let longe = sim
        .world_mut()
        .spawn((ScrollFactor { k: [0.25, 0.25] }, pose_em(0.0, 0.0)))
        .id();
    let mut drive = PreviewDrive::default();
    drive_parallax(&mut sim, Some(([0.0, 0.0], SEM_LIMITE)), PARADO, &mut drive);
    let sp = sim.world().get::<Transform>(perto).expect("pose").scale.x;
    let sl = sim.world().get::<Transform>(longe).expect("pose").scale.x;
    assert!(
        (sp - 1.0).abs() < 1e-5,
        "o plano focal mudou de tamanho: {sp}"
    );
    assert!((sl - 4.0 / 7.0).abs() < 1e-5, "o fundo leu {sl} contra 4/7");
    // ⭐ **É isto que nenhum motor 2D faz:** um zoom multiplicaria os dois pelo MESMO número.
    assert!(
        (sp / sl - 7.0 / 4.0).abs() < 1e-4,
        "a razao entre os dois planos e' {} e nao 7/4 — isto virou um zoom",
        sp / sl
    );
}

/// ⛔⛔ **Com `dolly = 0` a saída é BYTE-IDÊNTICA à da W1** — a exigência do plano, e a lei desta
/// casa para tudo o que é novo.
#[test]
fn com_dolly_zero_a_saida_e_byte_identica() {
    let autorada = Transform {
        translation: Vec2::new(-3.0, 7.0),
        rotation: 0.35,
        scale: Vec2::new(2.0, 3.0),
        ..Transform::default()
    };
    for k in [[0.3_f32, 0.7], [0.0, 2.0], [-0.5, 1.0]] {
        let (mut a, ea) = cena(k, autorada);
        let mut da = PreviewDrive::default();
        drive_parallax(&mut a, Some(([400.0, -250.0], SEM_LIMITE)), PARADO, &mut da);

        let (mut b, eb) = cena_dolly(k, 0.0, autorada);
        let mut db = PreviewDrive::default();
        drive_parallax(&mut b, Some(([400.0, -250.0], SEM_LIMITE)), PARADO, &mut db);

        assert_eq!(
            *a.world().get::<Transform>(ea).expect("pose"),
            *b.world().get::<Transform>(eb).expect("pose"),
            "com dolly ZERO a pose mudou (k {k:?}) — a omissao da W5 deixou de ser inerte"
        );
    }
}

/// ⛔⛔ **A câmera a ATRAVESSAR a camada RECUSA, e o objecto fica onde o artista o pôs.**
///
/// ⚠️ `1 − k·δ ≤ 0` é `d ≥ z`: a câmera passou para lá do fundo, e não há tamanho aparente nenhum.
/// *Um clamp ali entregaria um número plausível para uma cena impossível.*
#[test]
fn a_camera_a_atravessar_a_camada_recusa() {
    // `k = 0,5` ⇒ `z = 2·z₀`; um dolly de `2,5` passa para lá dela.
    let autorada = pose_em(-3.0, 7.0);
    let (mut sim, e) = cena_dolly([0.5, 0.5], 2.5, autorada);
    let mut drive = PreviewDrive::default();
    assert_eq!(
        drive_parallax(
            &mut sim,
            Some(([400.0, 0.0], SEM_LIMITE)),
            PARADO,
            &mut drive
        ),
        0,
        "a camada atravessada foi conduzida"
    );
    assert_eq!(*sim.world().get::<Transform>(e).expect("pose"), autorada);
    assert!(ScrollFactor::escala_do_dolly(0.5, 2.5).is_none());
    // ⭐ O CONTROLO: um dolly de `1,9` ainda está aquém dela e conduz.
    let (mut sim2, _) = cena_dolly([0.5, 0.5], 1.9, autorada);
    let mut drive2 = PreviewDrive::default();
    assert_eq!(
        drive_parallax(
            &mut sim2,
            Some(([400.0, 0.0], SEM_LIMITE)),
            PARADO,
            &mut drive2
        ),
        1
    );
}

/// ⭐⭐ **A escala multiplica a AUTORADA e nunca a viva** — senão ela COMPÕE a cada quadro e o fundo
/// cresce sem limite.
#[test]
fn a_escala_do_dolly_nao_compoe_entre_quadros() {
    let autorada = Transform {
        scale: Vec2::new(2.0, 2.0),
        ..pose_em(0.0, 0.0)
    };
    let (mut sim, e) = cena_dolly([0.25, 0.25], 0.5, autorada);
    let mut drive = PreviewDrive::default();
    let mut escalas = Vec::new();
    for _ in 0..5 {
        drive_parallax(&mut sim, Some(([0.0, 0.0], SEM_LIMITE)), PARADO, &mut drive);
        escalas.push(sim.world().get::<Transform>(e).expect("pose").scale.x);
    }
    let esperado = 2.0 * (4.0 / 7.0);
    for (i, s) in escalas.iter().enumerate() {
        assert!(
            (s - esperado).abs() < 1e-5,
            "ao quadro {i} a escala leu {s} contra {esperado} — ela COMPOS"
        );
    }
}

/// ⭐⭐⭐ **O DOLLY MUDA A FRACÇÃO, e não só o tamanho** — a metade da multiplano que um zoom também
/// não faz.
///
/// ⚠️ **Este gate nasceu de uma MUTAÇÃO SOBREVIVENTE:** apagar o dolly do deslocamento (usar `k₀` em
/// vez de `k(δ)`) passava por toda a bancada, porque **nenhum gate da tabela do declive tem uma
/// câmera de jogo na cena** — sem ela o dolly é `0` e as duas leituras coincidem. *Um corpus onde o
/// parâmetro está no ponto neutro não testa esse parâmetro*, a forma que este repo já conta.
#[test]
fn o_dolly_muda_a_velocidade_e_nao_so_o_tamanho() {
    let (k, delta) = ([0.25_f32, 0.25], 0.5_f32);
    // `k(δ) = k · (1−δ)/(1−kδ) = 0,25 · (4/7) = 1/7` ⇒ declive `1 − 1/7 = 6/7`.
    let declive = |d: f32| {
        let medir = |cam: f32| {
            let (mut sim, e) = cena_dolly(k, d, pose_em(0.0, 0.0));
            let mut drive = PreviewDrive::default();
            drive_parallax(&mut sim, Some(([cam, 0.0], SEM_LIMITE)), PARADO, &mut drive);
            sim.world().get::<Transform>(e).expect("pose").translation.x
        };
        (medir(200.0) - medir(-200.0)) / 400.0
    };
    let sem = declive(0.0);
    let com = declive(delta);
    assert!(
        (sem - 0.75).abs() < 1e-4,
        "sem dolly o declive e' {sem} contra 1 − 0,25"
    );
    assert!(
        (com - 6.0 / 7.0).abs() < 1e-4,
        "com dolly {delta} o declive leu {com} contra 6/7 — a fraccao deixou de mudar, e o dolly \
         virou um zoom que so' mexe no tamanho"
    );
}

/// ⛔⛔ **O dolly sai da câmera ACTIVA, e não de qualquer uma.**
///
/// ⚠️ **Também nasceu de uma mutação sobrevivente:** todas as cenas do dolly têm UMA câmera, e ali
/// «a activa» e «a primeira» são a mesma. *Uma fixtura com um só candidato não testa uma escolha.*
#[test]
fn o_dolly_sai_da_camera_activa_e_nao_de_qualquer_uma() {
    let mut sim = SimWorld::default();
    // ⚠️⚠️ **A DESLIGADA nasce PRIMEIRO, e a ordem é load-bearing:** com a activa à frente, *«a
    // activa»* e *«a primeira que a consulta devolve»* são a mesma entidade, e a mutação que troca
    // a porta por uma consulta crua **SOBREVIVE**. *Uma fixtura em que duas respostas coincidem
    // não distingue as duas perguntas* — e foi assim que esta prova nasceu.
    sim.world_mut().spawn((
        ph2d_ecs::GameCamera {
            dolly: 0.5,
            priority: 99,
            active: false,
            ..ph2d_ecs::GameCamera::default()
        },
        ph2d_ecs::StableId(2),
        Transform::default(),
    ));
    // A ACTIVA, sem dolly.
    sim.world_mut().spawn((
        ph2d_ecs::GameCamera {
            dolly: 0.0,
            priority: 10,
            ..ph2d_ecs::GameCamera::default()
        },
        ph2d_ecs::StableId(1),
        Transform::default(),
    ));
    let autorada = Transform {
        scale: Vec2::new(2.0, 2.0),
        ..pose_em(0.0, 0.0)
    };
    let e = sim
        .world_mut()
        .spawn((ScrollFactor { k: [0.25, 0.25] }, autorada))
        .id();
    let mut drive = PreviewDrive::default();
    drive_parallax(&mut sim, Some(([0.0, 0.0], SEM_LIMITE)), PARADO, &mut drive);
    assert_eq!(
        sim.world().get::<Transform>(e).expect("pose").scale,
        autorada.scale,
        "uma camera DESLIGADA mandou no dolly: a escolha da activa deixou de passar pela porta"
    );
}
