//! Os gates da PONTE da arma — a metade que a lei pura não pode afirmar: que o pente é lido e
//! escrito na entidade certa, que os sinais saem, e que a cerca do relógio existe.

use super::{WeaponTick, frame};
use ph2d_ecs::{Counter, CounterRuntime, Entity, Name, SimWorld, StableId, WeaponFire};

const DT: u64 = 1_000_000 / 60;

/// Uma arma com pente de `n` balas na mesma entidade.
fn arma_com_pente(sim: &mut SimWorld, id: u64, n: i64, cadencia: u64) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new("Arma"),
            StableId(id),
            WeaponFire {
                on_signal: "fire".to_owned(),
                cooldown_ms: cadencia,
                ammo_counter: "ammo".to_owned(),
                reload_ms: 100,
                reload_on: "reload".to_owned(),
                on_fire: "shot".to_owned(),
                on_empty: "click".to_owned(),
                on_reloaded: "ready".to_owned(),
                reserve_counter: String::new(),
            },
            Counter {
                name: "ammo".to_owned(),
                start: n,
                keep_on_restart: false,
            },
            CounterRuntime { value: n },
        ))
        .id()
}

fn municao(sim: &SimWorld, e: Entity) -> i64 {
    sim.world()
        .get::<CounterRuntime>(e)
        .map_or(-999, |r| r.value)
}

/// **O tiro GASTA uma bala do contador DESTA arma, e publica o sinal.**
///
/// É a metade que a lei pura não afirma: ela devolve um número, e quem o escreve é isto.
#[test]
fn o_tiro_gasta_uma_bala_e_publica() {
    let mut sim = SimWorld::new();
    let e = arma_com_pente(&mut sim, 1, 6, 0);
    let t = frame(&mut sim, true, DT, &["fire"]);
    assert_eq!(t.disparos, vec![(e, "shot".to_owned())]);
    assert_eq!(municao(&sim, e), 5, "o pente desceu uma");
}

/// **Duas armas, dois pentes — cada uma gasta o SEU.**
///
/// ⚠️ Sem esta metade, uma ponte que usasse o `counter::soma` (que soma TODOS os contadores com o
/// nome) passaria o gate de cima e faria as duas armas partilharem a munição.
#[test]
fn cada_arma_gasta_o_proprio_pente() {
    let mut sim = SimWorld::new();
    let a = arma_com_pente(&mut sim, 1, 6, 0);
    let b = arma_com_pente(&mut sim, 2, 3, 0);
    let t = frame(&mut sim, true, DT, &["fire"]);
    assert_eq!(t.disparos.len(), 2, "as duas ouvem o mesmo sinal");
    assert_eq!(municao(&sim, a), 5);
    assert_eq!(municao(&sim, b), 2, "o pente de b é o de b");
}

/// **A ORDEM dos factos é a da IDENTIDADE**, e não a da iteração do mundo.
#[test]
fn a_ordem_dos_disparos_e_a_da_identidade() {
    let mut sim = SimWorld::new();
    // nasce primeiro a de identidade MAIOR, para a ordem do mundo e a da identidade discordarem
    let tarde = arma_com_pente(&mut sim, 9, 6, 0);
    let cedo = arma_com_pente(&mut sim, 2, 6, 0);
    let t = frame(&mut sim, true, DT, &["fire"]);
    assert_eq!(
        t.disparos.iter().map(|(e, _)| *e).collect::<Vec<_>>(),
        vec![cedo, tarde],
        "a identidade manda, senão duas máquinas divergem"
    );
}

/// **Com o relógio PARADO nada acontece** — uma arma é da corrida.
///
/// Sem esta cerca, escrever num campo do editor gastaria munição.
#[test]
fn com_o_relogio_parado_a_arma_nao_dispara() {
    let mut sim = SimWorld::new();
    let e = arma_com_pente(&mut sim, 1, 6, 0);
    let t = frame(&mut sim, false, DT, &["fire"]);
    assert_eq!(t, WeaponTick::default());
    assert_eq!(municao(&sim, e), 6, "e não gasta uma bala");
}

/// **O pente acaba: a arma fica SECA, diz-o, e recarrega.**
#[test]
fn o_pente_acaba_a_arma_diz_e_recarrega() {
    let mut sim = SimWorld::new();
    let e = arma_com_pente(&mut sim, 1, 2, 0);
    for _ in 0..2 {
        let _ = frame(&mut sim, true, DT, &["fire"]);
    }
    assert_eq!(municao(&sim, e), 0);

    let seca = frame(&mut sim, true, DT, &["fire"]);
    assert_eq!(seca.secas, vec![(e, "click".to_owned())]);
    assert!(seca.disparos.is_empty(), "sem balas não sai tiro");

    let mut disse = false;
    for _ in 0..12 {
        disse |= !frame(&mut sim, true, DT, &[]).recarregadas.is_empty();
    }
    assert!(disse, "a recarga automática anuncia-se");
    assert_eq!(municao(&sim, e), 2, "e o pente fica CHEIO");
}

/// **Um `ammo_counter` que aponta para um contador que não está NESTA entidade dá munição
/// infinita**, e não uma arma inerte.
///
/// ⚠️ É a lei do alvo que não existe, da tabela de acções: *uma configuração a meio não é um erro*.
/// Quem o diz ao artista é o painel.
#[test]
fn um_pente_noutra_entidade_nao_trava_a_arma() {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Name::new("Arma"),
            StableId(1),
            WeaponFire {
                on_signal: "fire".to_owned(),
                ammo_counter: "ammo".to_owned(),
                on_fire: "shot".to_owned(),
                ..WeaponFire::default()
            },
        ))
        .id();
    // o contador existe na CENA, mas noutro objecto
    sim.world_mut().spawn((
        Name::new("Placar"),
        StableId(2),
        Counter {
            name: "ammo".to_owned(),
            start: 0,
            keep_on_restart: false,
        },
        CounterRuntime { value: 0 },
    ));
    let t = frame(&mut sim, true, DT, &["fire"]);
    assert_eq!(t.disparos, vec![(e, "shot".to_owned())]);
}

/// **Sinais vazios ficam CALADOS** — a lei da casa, e a metade negativa dos três canais.
#[test]
fn os_sinais_vazios_ficam_calados() {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Arma"),
        StableId(1),
        WeaponFire {
            on_signal: "fire".to_owned(),
            ..WeaponFire::default()
        },
    ));
    let t = frame(&mut sim, true, DT, &["fire"]);
    assert_eq!(t, WeaponTick::default(), "ela dispara e não diz nada");
}

// ── O DEPÓSITO (`reserve_counter`) ────────────────────────────────────────────────────────────

/// Põe uma caixa com `n` balas chamada `nome`, numa entidade PRÓPRIA.
fn caixa(sim: &mut SimWorld, nome: &str, n: i64) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new("Caixa"),
            Counter {
                name: nome.to_owned(),
                start: n,
                keep_on_restart: false,
            },
            CounterRuntime { value: n },
        ))
        .id()
}

/// Liga a arma a um depósito.
fn aponta_ao_deposito(sim: &mut SimWorld, arma: Entity, nome: &str) {
    sim.world_mut()
        .get_mut::<WeaponFire>(arma)
        .unwrap()
        .reserve_counter = nome.to_owned();
}

/// ⭐⭐⭐ **A recarga TIRA do depósito, e a ponte escreve nele** — a metade que a lei pura não pode
/// afirmar: ela devolve dois números e quem os põe nos dois donos é isto.
///
/// **Mutações que devem sangrar:** não escrever o depósito · escrevê-lo na arma · ler a soma da
/// cena em vez do dono.
#[test]
fn a_recarga_tira_do_deposito_e_a_ponte_escreve_nele() {
    let mut sim = SimWorld::new();
    let arma = arma_com_pente(&mut sim, 1, 3, 0);
    let cx = caixa(&mut sim, "box", 2);
    aponta_ao_deposito(&mut sim, arma, "box");
    // ⚠️ **QUATRO puxões e não três:** as três gastam o pente e é o QUARTO — o clique SECO — que
    // arranca a recarga automática. *Um arnês que pára no pente vazio mede uma arma que nunca
    // recarrega*, e foi assim que os três gates desta secção nasceram vermelhos.
    for _ in 0..4 {
        let _ = frame(&mut sim, true, DT, &["fire"]);
    }
    assert_eq!(municao(&sim, arma), 0, "o pente esvaziou");
    for _ in 0..12 {
        let _ = frame(&mut sim, true, DT, &[]);
    }
    assert_eq!(municao(&sim, arma), 2, "a recarga leva o que HAVIA, nao 3");
    assert_eq!(
        municao(&sim, cx),
        0,
        "e a ponte escreve no DONO do deposito"
    );
}

/// ⛔⛔ **Um nome que DOIS objectos carregam é RECUSADO** — a arma cai em reserva infinita, e o
/// painel é quem o diz.
///
/// ⚠️ **O CONTROLO é o mesmo mundo com UMA caixa:** sem ele este gate ficaria verde sobre uma
/// ponte que nunca lê depósito nenhum.
#[test]
fn um_deposito_ambiguo_e_recusado_e_a_arma_fica_infinita() {
    for (n_caixas, esperado) in [(1_usize, 2_i64), (2, 3)] {
        let mut sim = SimWorld::new();
        let arma = arma_com_pente(&mut sim, 1, 3, 0);
        for _ in 0..n_caixas {
            caixa(&mut sim, "box", 2);
        }
        aponta_ao_deposito(&mut sim, arma, "box");
        for _ in 0..4 {
            let _ = frame(&mut sim, true, DT, &["fire"]);
        }
        for _ in 0..12 {
            let _ = frame(&mut sim, true, DT, &[]);
        }
        assert_eq!(
            municao(&sim, arma),
            esperado,
            "com {n_caixas} caixa(s) o pente tinha de ficar em {esperado} \
             (uma => leva as 2 que ha'; duas => AMBIGUO, reserva infinita, enche os 3)"
        );
    }
}

/// ⭐ **Um depósito por ESTREAR já vale o `start`** — e a ponte cria-lhe o vivo.
///
/// ⚠️ Sem isto, uma caixa acabada de pôr na cena daria **reserva zero** no primeiro quadro e a arma
/// ficaria seca para sempre — *duas leituras diferentes do «por estrear» dariam uma arma que come
/// a primeira recarga*.
#[test]
fn um_deposito_por_estrear_ja_vale_o_start() {
    let mut sim = SimWorld::new();
    let arma = arma_com_pente(&mut sim, 1, 3, 0);
    let cx = sim
        .world_mut()
        .spawn((
            Name::new("Caixa crua"),
            Counter {
                name: "box".to_owned(),
                start: 5,
                keep_on_restart: false,
            },
        ))
        .id();
    aponta_ao_deposito(&mut sim, arma, "box");
    for _ in 0..4 {
        let _ = frame(&mut sim, true, DT, &["fire"]);
    }
    for _ in 0..12 {
        let _ = frame(&mut sim, true, DT, &[]);
    }
    assert_eq!(
        municao(&sim, arma),
        3,
        "o pente encheu do deposito por estrear"
    );
    assert_eq!(municao(&sim, cx), 2, "e o vivo dele NASCEU com o resto");
}
