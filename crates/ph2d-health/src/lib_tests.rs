//! Os gates das DIVERGÊNCIAS declaradas e das QUEIXAS da pesquisa (doc 27 §2.2). Cada divergência
//! tem as duas metades: a CASA faz o que o produto quer **e** as regras do ALVO reproduzem o que o
//! oráculo mediu — sem a segunda, uma regra que não mudasse nada passaria verde.

use super::*;

const DT_MS: f64 = 1000.0 / 60.0;

fn nunca_esquiva() -> impl FnMut() -> f64 {
    || 0.5
}

fn com(maximo: f64) -> (Config, Vida) {
    let cfg = Config {
        maximo,
        ..Config::default()
    };
    let v = Vida::nasce(maximo, &cfg);
    (cfg, v)
}

/// ⛔ **A vida nunca sai de `[0, máx]` na casa** — no alvo ela desce a negativo (`a_dano_basico`).
#[test]
fn a_vida_para_em_zero_na_casa_e_desce_no_alvo() {
    let (cfg, mut casa) = com(100.0);
    let mut alvo = casa;
    casa.golpe(
        &cfg,
        Regras::CASA,
        130.0,
        false,
        false,
        &mut nunca_esquiva(),
    );
    alvo.golpe(
        &cfg,
        Regras::GDEVELOP,
        130.0,
        false,
        false,
        &mut nunca_esquiva(),
    );
    assert_eq!(casa.pontos, 0.0);
    assert!(casa.morta());
    assert_eq!(alvo.pontos, -30.0, "o controlo: o alvo desce a negativo");
}

/// ⛔ **Um morto é FINAL na casa** — nem a cura o ressuscita nem um golpe lhe tira mais; só o
/// [`Vida::reviver`] explícito. No alvo `−30 + 50 = 20`.
#[test]
fn um_morto_so_volta_por_reviver() {
    let (cfg, mut v) = com(100.0);
    v.golpe(
        &cfg,
        Regras::CASA,
        100.0,
        false,
        false,
        &mut nunca_esquiva(),
    );
    assert!(v.morta());
    v.cura(&cfg, Regras::CASA, 50.0);
    assert_eq!(v.pontos, 0.0, "a cura ressuscitou um morto");
    assert!(!v.acabou_de_ser_curada, "a cura recusada acendeu a marca");
    let mut sorteios = 0;
    v.golpe(&cfg, Regras::CASA, 10.0, false, false, &mut || {
        sorteios += 1;
        0.5
    });
    assert_eq!(sorteios, 0, "um golpe num morto sorteou");
    v.reviver(&cfg, Regras::CASA, 40.0);
    assert_eq!(v.pontos, 40.0);
    // O controlo: no alvo a mesma cura ressuscita.
    let (cfg, mut alvo) = com(100.0);
    alvo.golpe(
        &cfg,
        Regras::GDEVELOP,
        130.0,
        false,
        false,
        &mut nunca_esquiva(),
    );
    alvo.cura(&cfg, Regras::GDEVELOP, 50.0);
    assert_eq!(alvo.pontos, 20.0);
}

/// ⛔ **Um pedido negativo ou não-finito não faz NADA na casa** — nem sorteia, nem grava.
#[test]
fn os_pedidos_negativos_nao_fazem_nada_na_casa() {
    let (cfg, mut v) = com(100.0);
    let antes = v;
    let mut sorteios = 0;
    for d in [-10.0, f64::NAN, f64::NEG_INFINITY] {
        v.golpe(&cfg, Regras::CASA, d, true, true, &mut || {
            sorteios += 1;
            0.5
        });
        v.cura(&cfg, Regras::CASA, d);
    }
    assert_eq!(sorteios, 0);
    assert_eq!(v, antes, "um pedido recusado mudou o estado");
    // O controlo: no alvo `Heal(−20)` tira e `Hit(−10)` grava.
    let mut alvo = antes;
    alvo.cura(&cfg, Regras::GDEVELOP, -20.0);
    alvo.golpe(
        &cfg,
        Regras::GDEVELOP,
        -10.0,
        false,
        false,
        &mut nunca_esquiva(),
    );
    assert_eq!((alvo.pontos, alvo.dano_anterior), (80.0, -10.0));
}

/// ⭐⭐ **Com a sobre-cura, a casa cura o que se PEDE** — o alvo aplica a quantidade da cura
/// ANTERIOR (o defeito que o oráculo apanhou: `c_cura_overheal`, passo 5, `Heal(50)` sobe `30`).
#[test]
fn a_sobre_cura_cura_o_que_se_pede() {
    for (regras, esperado) in [(Regras::CASA, 150.0), (Regras::GDEVELOP, 130.0)] {
        let (mut cfg, mut v) = com(100.0);
        v.golpe(&cfg, regras, 30.0, false, false, &mut nunca_esquiva());
        v.cura(&cfg, regras, 50.0); // sem sobre-cura: aplica 30 e pára no máximo
        assert_eq!(v.pontos, 100.0);
        cfg.sobre_cura = true;
        v.cura(&cfg, regras, 50.0);
        assert_eq!(v.pontos, esperado, "{regras:?}");
    }
}

/// ⛔ **A queixa nº 1 — dano a dobrar num golpe:** com invencibilidade, o segundo golpe do MESMO
/// quadro é ignorado por inteiro, e não sorteia.
#[test]
fn dois_golpes_no_mesmo_quadro_contam_um() {
    let cfg = Config {
        invencivel_s: 0.5,
        ..Config::default()
    };
    let mut v = Vida::nasce(100.0, &cfg);
    v.anda(DT_MS);
    v.pre_quadro(&cfg, Regras::CASA, DT_MS / 1000.0);
    let mut sorteios = 0;
    for _ in 0..2 {
        v.golpe(&cfg, Regras::CASA, 10.0, false, false, &mut || {
            sorteios += 1;
            0.5
        });
    }
    assert_eq!(v.pontos, 90.0);
    assert_eq!(sorteios, 1, "o golpe ignorado sorteou");
    // O controlo: sem invencibilidade os dois contam.
    let cfg0 = Config::default();
    let mut w = Vida::nasce(100.0, &cfg0);
    for _ in 0..2 {
        w.golpe(
            &cfg0,
            Regras::CASA,
            10.0,
            false,
            false,
            &mut nunca_esquiva(),
        );
    }
    assert_eq!(w.pontos, 80.0);
}

/// ⭐ **A invencibilidade acaba a tempo, e a fronteira é estrita** — `TimeSinceLastHit < cooldown`.
#[test]
fn a_invencibilidade_acaba_no_tempo_dela() {
    let cfg = Config {
        invencivel_s: 0.5,
        ..Config::default()
    };
    let mut v = Vida::nasce(100.0, &cfg);
    v.golpe(&cfg, Regras::CASA, 10.0, false, false, &mut nunca_esquiva());
    let mut quadros = 0;
    while v.invencivel(&cfg) {
        v.anda(DT_MS);
        v.pre_quadro(&cfg, Regras::CASA, DT_MS / 1000.0);
        quadros += 1;
        assert!(quadros < 1000, "a invencibilidade nunca acabou");
    }
    // 30 quadros a 1/60 somam `0,5000000000000002` s em ms acumulados — já não é `< 0,5`.
    assert_eq!(quadros, 30);
}
