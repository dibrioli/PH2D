//! Os gates da LEI da arma. Cada um nomeia a célula da medição do §5.0 que ele defende
//! ([plano 22](../../../docs/Components/22_plano_weapon_fire.md) §1).

use super::{Municao, Tiro, WeaponFire, WeaponRuntime, avanca, born};

/// O passo fixo da casa, em microssegundos.
const DT: u64 = 1_000_000 / 60;
/// Um quarto de segundo entre tiros — quatro por segundo.
const CADENCIA: u64 = 250;

fn arma() -> WeaponFire {
    WeaponFire {
        on_signal: "fire".to_owned(),
        cooldown_ms: CADENCIA,
        on_fire: "shot".to_owned(),
        ..WeaponFire::default()
    }
}

/// Munição infinita — o que um `ammo_counter` vazio produz.
const INFINITA: Municao = Municao {
    tem: 0,
    cheio: 0,
    existe: false,
    reserva: 0,
    reserva_existe: false,
};

/// ⚠️ **Reserva INFINITA**, que é o valor de toda cena gravada antes de 2026-09-20 — o corpus
/// inteiro desta bancada mede a lei SEM depósito, e é isso que prova que ela não mudou.
fn pente(tem: i64, cheio: i64) -> Municao {
    Municao {
        tem,
        cheio,
        existe: true,
        reserva: 0,
        reserva_existe: false,
    }
}

/// O mesmo pente, com um DEPÓSITO de `reserva` balas.
fn pente_com_deposito(tem: i64, cheio: i64, reserva: i64) -> Municao {
    Municao {
        reserva,
        reserva_existe: true,
        ..pente(tem, cheio)
    }
}

/// Segura o gatilho `quadros` tiques e devolve quantas vezes ela disparou.
fn segurando(cfg: &WeaponFire, st: &mut WeaponRuntime, mun: Municao, quadros: u32) -> u32 {
    let mut n = 0;
    for _ in 0..quadros {
        if avanca(cfg, st, mun, DT, true, false).disparou {
            n += 1;
        }
    }
    n
}

/// **G-1 — o PRIMEIRO tiro é imediato.**
///
/// É o defeito medido na composição: pelo relógio a 1.ª bala sai `0,250 s` depois de carregar, e
/// *carregar e não acontecer nada durante um quarto de segundo lê-se como um botão que não
/// funciona*.
#[test]
fn o_primeiro_tiro_sai_no_tique_zero() {
    let cfg = arma();
    let mut st = born();
    let t = avanca(&cfg, &mut st, INFINITA, DT, true, false);
    assert!(t.disparou, "a arma tem de disparar no primeiro tique");
}

/// **G-2 — segurar o gatilho um segundo dá a CADÊNCIA, não 60.**
///
/// A composição de hoje devolve `60` (a linha `Hold` fala em todo tique e a fábrica nasce a cada
/// sinal).
#[test]
fn segurar_o_gatilho_um_segundo_da_a_cadencia() {
    let cfg = arma();
    let mut st = born();
    let n = segurando(&cfg, &mut st, INFINITA, 60);
    assert_eq!(
        n, 4,
        "com {CADENCIA} ms entre tiros, um segundo tem de dar 4 e não 60"
    );
}

/// **G-3 — `cooldown_ms = 0` é o NEUTRO e todo pedido dispara.**
///
/// ⚠️ Sem esta metade, uma lei que ignorasse o campo passaria a G-2 ao escolher um número fixo.
#[test]
fn sem_cadencia_todo_pedido_dispara() {
    let cfg = WeaponFire {
        cooldown_ms: 0,
        ..arma()
    };
    let mut st = born();
    let n = segurando(&cfg, &mut st, INFINITA, 60);
    assert_eq!(n, 60, "sem cadência a arma dispara em todo tique");
}

/// **G-4 — o pente esvazia, a arma PÁRA, e ela diz que está seca.**
///
/// A composição dispara `10` balas de um pente de `6` e deixa o contador a `−4`.
#[test]
fn o_pente_acaba_e_a_arma_para() {
    let cfg = WeaponFire {
        cooldown_ms: 0,
        ..arma()
    };
    let mut st = born();
    let mut mun = pente(6, 6);
    let mut saiu = 0;
    let mut secas = 0;
    for _ in 0..10 {
        let t = avanca(&cfg, &mut st, mun, DT, true, false);
        if t.disparou {
            saiu += 1;
        }
        if t.seca {
            secas += 1;
        }
        mun.tem = t.municao;
    }
    assert_eq!(saiu, 6, "só podem sair as balas que o pente tinha");
    assert_eq!(mun.tem, 0, "o contador nunca vai a negativo");
    assert_eq!(secas, 4, "cada gatilho no vazio é um clique seco");
}

/// **G-5 — a recarga REPÕE o cheio, nunca soma.**
#[test]
fn a_recarga_repoe_o_pente_cheio() {
    let cfg = WeaponFire {
        cooldown_ms: 0,
        reload_ms: 100,
        reload_on: "reload".to_owned(),
        ..arma()
    };
    let mut st = born();
    let mut mun = pente(0, 6);
    let inicio = avanca(&cfg, &mut st, mun, DT, false, true);
    assert!(inicio.comecou_a_recarregar);
    assert!(!inicio.recarregou, "ela não acaba no tique em que começa");

    let mut encheu = false;
    for _ in 0..12 {
        let t = avanca(&cfg, &mut st, mun, DT, false, false);
        mun.tem = t.municao;
        encheu |= t.recarregou;
    }
    assert!(encheu, "100 ms cabem em 12 tiques de 16,6 ms");
    assert_eq!(mun.tem, 6, "o pente fica CHEIO");
}

/// **G-6 — recarregar com balas dentro dá o CHEIO, não `start + resto`.**
///
/// É a armadilha exacta do `Add to Counter`, o único verbo que escreve um contador: somar o tamanho
/// do pente só acerta com ele a zero — *recarregar com três dentro daria nove*.
#[test]
fn recarregar_com_balas_dentro_nao_soma() {
    let cfg = WeaponFire {
        cooldown_ms: 0,
        reload_ms: 100,
        reload_on: "reload".to_owned(),
        ..arma()
    };
    let mut st = born();
    let mut mun = pente(3, 6);
    for _ in 0..12 {
        let t = avanca(&cfg, &mut st, mun, DT, false, mun.tem == 3);
        mun.tem = t.municao;
    }
    assert_eq!(mun.tem, 6, "seis, e não nove");
}

/// **G-7 — durante a recarga o gatilho NÃO dispara.**
///
/// Sem isto a recarga é decorativa: a arma continuaria a cuspir balas enquanto o artista vê a
/// animação de recarregar.
#[test]
fn durante_a_recarga_o_gatilho_fica_mudo() {
    let cfg = WeaponFire {
        cooldown_ms: 0,
        reload_ms: 500,
        reload_on: "reload".to_owned(),
        ..arma()
    };
    let mut st = born();
    let mut mun = pente(3, 6);
    assert!(avanca(&cfg, &mut st, mun, DT, false, true).comecou_a_recarregar);
    for _ in 0..10 {
        let t = avanca(&cfg, &mut st, mun, DT, true, false);
        assert!(!t.disparou, "a arma a recarregar não dispara");
        mun.tem = t.municao;
    }
    assert_eq!(mun.tem, 3, "e não gasta munição nenhuma");
}

/// **G-7b — recarregar DURANTE uma recarga não a reinicia.**
///
/// Senão carregar na tecla em pânico faria a arma nunca ficar pronta.
#[test]
fn pedir_recarga_a_meio_nao_reinicia_a_recarga() {
    let cfg = WeaponFire {
        cooldown_ms: 0,
        reload_ms: 100,
        reload_on: "reload".to_owned(),
        ..arma()
    };
    let mut st = born();
    let mun = pente(0, 6);
    let _ = avanca(&cfg, &mut st, mun, DT, false, true);
    let mut encheu = false;
    for _ in 0..12 {
        // o dedo em pânico: pede recarga em TODO tique
        encheu |= avanca(&cfg, &mut st, mun, DT, false, true).recarregou;
    }
    assert!(encheu, "a recarga acaba apesar dos pedidos em cima dela");
}

/// **G-8 — `ammo_counter` vazio é munição INFINITA, e a lei não escreve nada.**
#[test]
fn sem_contador_a_municao_e_infinita() {
    let cfg = WeaponFire {
        cooldown_ms: 0,
        ..arma()
    };
    let mut st = born();
    let n = segurando(&cfg, &mut st, INFINITA, 300);
    assert_eq!(n, 300, "sem pente ela nunca fica seca");
}

/// **G-4b — o gatilho no vazio ARRANCA a recarga automática.**
///
/// ⚠️ Metade NEGATIVA junto: sem `reload_ms` ela fica seca e nenhuma recarga começa — *uma arma que
/// recarregasse sozinha sem o artista o pedir tiraria um estado que ele pode querer*.
#[test]
fn o_clique_seco_arranca_a_recarga_e_sem_reload_fica_seca() {
    let com = WeaponFire {
        cooldown_ms: 0,
        reload_ms: 100,
        ..arma()
    };
    let mut st = born();
    let t = avanca(&com, &mut st, pente(0, 6), DT, true, false);
    assert!(t.seca && t.comecou_a_recarregar);

    let sem = WeaponFire {
        cooldown_ms: 0,
        reload_ms: 0,
        ..arma()
    };
    let mut st2 = born();
    let t2 = avanca(&sem, &mut st2, pente(0, 6), DT, true, false);
    assert!(t2.seca, "ela diz que está seca");
    assert!(
        !t2.comecou_a_recarregar,
        "sem reload_ms nenhuma recarga começa"
    );
}

/// **G-12 (metade da lei) — `born` é «pronta a disparar».**
///
/// *Rebobinar é RENASCER*: a cadência zera e a recarga a meio é cancelada.
#[test]
fn nascer_deixa_a_arma_pronta() {
    let cfg = arma();
    let mut st = born();
    let _ = avanca(&cfg, &mut st, INFINITA, DT, true, false);
    assert!(st.cooldown_left_us > 0, "o controlo: ela ficou em cadência");

    st = born();
    assert_eq!(st, WeaponRuntime::default());
    assert!(
        avanca(&cfg, &mut st, INFINITA, DT, true, false).disparou,
        "depois de renascer o primeiro tiro é outra vez imediato"
    );
}

/// **A ORDEM dentro do tique É a lei** — uma recarga que acaba AGORA deixa a arma disparar no
/// tique seguinte, e o pedido de recarga em cima de um pente já cheio é inerte.
#[test]
fn a_ordem_do_tique_e_a_lei() {
    let cfg = WeaponFire {
        cooldown_ms: 0,
        reload_ms: 16,
        reload_on: "reload".to_owned(),
        ..arma()
    };
    let mut st = born();
    let mut mun = pente(0, 2);
    let _ = avanca(&cfg, &mut st, mun, DT, false, true);
    let t = avanca(&cfg, &mut st, mun, DT, false, false);
    assert!(t.recarregou && t.municao == 2);
    mun.tem = t.municao;

    // ⚠️ o pedido sobre o pente CHEIO não arranca nada: ele lê o que a recarga acabou de escrever.
    let inerte = avanca(&cfg, &mut st, mun, DT, false, true);
    assert_eq!(
        inerte,
        Tiro {
            municao: 2,
            ..Tiro::default()
        },
        "recarregar um pente cheio é inerte"
    );
}

// ── O DEPÓSITO (`reserve_counter`) ────────────────────────────────────────────────────────────

/// Uma arma com pente e recarga, sem cadência.
fn com_recarga(reload_ms: u64) -> WeaponFire {
    WeaponFire {
        on_signal: "fogo".into(),
        ammo_counter: "pente".into(),
        reload_ms,
        ..WeaponFire::default()
    }
}

/// Anda `n` tiques sem tocar em nada, e devolve os factos ACUMULADOS.
///
/// ⚠️⚠️ **Acumulados e não «o último»** — a 1.ª redacção devolvia o último `Tiro` e os gates
/// reprovaram sobre produto CERTO: o `recarregou` é um ACONTECIMENTO, vale num tique só, e ler a
/// última leitura de um acontecimento é lê-lo como se nunca tivesse existido. *A mesma distinção
/// facto/evento que o `projectiles_finished` do #14 pagou.*
fn espera(cfg: &WeaponFire, st: &mut WeaponRuntime, mun: &mut Municao, n: u32) -> Tiro {
    let mut acc = Tiro {
        municao: mun.tem,
        reserva: mun.reserva,
        ..Tiro::default()
    };
    for _ in 0..n {
        let t = avanca(cfg, st, *mun, DT, false, false);
        mun.tem = t.municao;
        mun.reserva = t.reserva;
        acc.municao = t.municao;
        acc.reserva = t.reserva;
        acc.recarregou |= t.recarregou;
        acc.disparou |= t.disparou;
        acc.seca |= t.seca;
        acc.comecou_a_recarregar |= t.comecou_a_recarregar;
    }
    acc
}

/// ⭐⭐⭐ **A recarga leva SÓ o que há no depósito** — a feature inteira num número.
///
/// **Mutações que devem sangrar:** `falta.min(ha)` → `falta` · não descontar da reserva.
#[test]
fn a_recarga_leva_so_o_que_ha_no_deposito() {
    let cfg = com_recarga(100);
    let mut st = born();
    let mut mun = pente_com_deposito(0, 5, 2);
    // O gatilho no pente vazio arranca a recarga automática.
    let t = avanca(&cfg, &mut st, mun, DT, true, false);
    assert!(t.seca && t.comecou_a_recarregar, "a seca arranca a recarga");
    let t = espera(&cfg, &mut st, &mut mun, 10);
    assert!(t.recarregou, "a recarga acabou");
    assert_eq!(
        t.municao, 2,
        "o pente leva o que HAVIA, nao os 5 do `start`"
    );
    assert_eq!(t.reserva, 0, "e o deposito fica vazio");
}

/// ⭐⭐ **A transferência CONSERVA:** o que o pente sobe é exactamente o que o depósito desce.
///
/// ⚠️ **Uma varredura e não uma amostra** — um caso só não distingue *«levou o que falta»* de
/// *«levou o que havia»* quando os dois números por acaso coincidem.
#[test]
fn a_transferencia_conserva() {
    for cheio in [1_i64, 5, 9] {
        for tem in 0..cheio {
            for reserva in [0_i64, 1, 3, 100] {
                let cfg = com_recarga(50);
                let mut st = born();
                let mut mun = pente_com_deposito(tem, cheio, reserva);
                let antes = (mun.tem, mun.reserva);
                let _ = avanca(&cfg, &mut st, mun, DT, false, true);
                let t = espera(&cfg, &mut st, &mut mun, 6);
                if !t.recarregou {
                    assert_eq!(reserva, 0, "so' um deposito VAZIO impede a recarga");
                    continue;
                }
                let sobe = t.municao - antes.0;
                let desce = antes.1 - t.reserva;
                assert_eq!(sobe, desce, "cheio={cheio} tem={tem} reserva={reserva}");
                assert!(t.reserva >= 0, "um deposito nunca fica NEGATIVO");
                assert!(t.municao <= cheio, "o pente nunca passa do `start`");
            }
        }
    }
}

/// ⚠️ **Com o depósito vazio a recarga NÃO arranca** — e o clique seco continua a soar, que é o
/// report certo: *clique, clique, clique* **é** ficar sem munição.
#[test]
fn com_o_deposito_vazio_a_recarga_nao_arranca() {
    let cfg = com_recarga(100);
    let mut st = born();
    let t = avanca(&cfg, &mut st, pente_com_deposito(0, 5, 0), DT, true, false);
    assert!(t.seca, "o clique seco continua a soar");
    assert!(
        !t.comecou_a_recarregar,
        "e ela NAO fica presa num prazo inutil"
    );
    // E um pedido EXPLÍCITO também não a arranca.
    let t = avanca(&cfg, &mut st, pente_com_deposito(0, 5, 0), DT, false, true);
    assert!(!t.comecou_a_recarregar);
}

/// ⭐⭐⭐ **O CONTROLO: sem depósito a lei é a de SEMPRE, campo a campo.**
///
/// ⚠️ Sem esta metade, tudo acima ficaria verde sobre uma lei que tivesse mudado o caminho de
/// omissão — e o caminho de omissão é o de **toda cena já gravada**.
#[test]
fn sem_deposito_a_lei_e_a_de_sempre() {
    let cfg = com_recarga(50);
    for tem in 0..4_i64 {
        let (mut a, mut b) = (born(), born());
        let t_infinita = {
            let mut m = pente(tem, 4);
            let _ = avanca(&cfg, &mut a, m, DT, true, false);
            espera(&cfg, &mut a, &mut m, 5)
        };
        // Um depósito com MAIS do que o pente leva tem de dar o mesmo pente.
        let t_farto = {
            let mut m = pente_com_deposito(tem, 4, 1_000);
            let _ = avanca(&cfg, &mut b, m, DT, true, false);
            espera(&cfg, &mut b, &mut m, 5)
        };
        assert_eq!(
            (t_infinita.municao, t_infinita.recarregou, t_infinita.seca),
            (t_farto.municao, t_farto.recarregou, t_farto.seca),
            "tem={tem}: um deposito FARTO tem de ser indistinguivel de nao ter deposito"
        );
    }
}
