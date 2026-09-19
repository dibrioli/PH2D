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
};

fn pente(tem: i64, cheio: i64) -> Municao {
    Municao {
        tem,
        cheio,
        existe: true,
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
