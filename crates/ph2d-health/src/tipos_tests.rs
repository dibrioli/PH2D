//! Os gates dos TIPOS — ver o cabeçalho de [`super`].

use super::*;
use crate::{Config, Regras, Vida};

const DT: f64 = 1.0 / 60.0;

fn vida(pontos: f64, cfg: &Config) -> Vida {
    Vida::nasce(pontos, cfg)
}

fn taxa(mult: f64) -> Taxa {
    Taxa {
        mult,
        absorve: false,
    }
}

fn absorve(mult: f64) -> Taxa {
    Taxa {
        mult,
        absorve: true,
    }
}

/// Um golpe de `dano` com a `taxa`, sem escudo nem armadura, e o sorteio de esquiva a nunca esquivar.
fn golpe(v: &mut Vida, cfg: &Config, dano: f64, t: Taxa) {
    v.golpe_tipado(cfg, Regras::CASA, dano, t, true, true, &mut || 1.0);
}

/// ⭐⭐⭐ **O ORÁCULO decide os casos que ele tem** — `docs/Components/ferramentas/godot_health_tipos/
/// saida.txt`, as linhas cuja resposta não passa pelo arredondamento da vida INTEIRA do alvo (esses
/// são a divergência declarada, gate abaixo).
///
/// **Mutações que devem sangrar:** a taxa negativa a curar · o absorver a passar pela armadura ·
/// a taxa antes de tudo em vez de depois da armadura (o caso da armadura, abaixo).
#[test]
fn o_oraculo_decide_imune_meio_dobro_negativo_e_absorver() {
    let cfg = Config::default();
    // (caso da fixtura, vida inicial, taxa, dano, vida depois)
    let casos: [(&str, f64, Taxa, f64, f64); 8] = [
        ("neutro", 100.0, Taxa::NEUTRA, 10.0, 90.0),
        ("mult_1", 100.0, taxa(1.0), 10.0, 90.0),
        ("imune", 100.0, taxa(0.0), 10.0, 100.0),
        ("meio", 100.0, taxa(0.5), 10.0, 95.0),
        ("dobro", 100.0, taxa(2.0), 10.0, 80.0),
        ("negativo", 50.0, taxa(-1.0), 10.0, 50.0),
        ("absorve", 50.0, absorve(1.0), 10.0, 60.0),
        ("absorve_meio", 50.0, absorve(0.5), 10.0, 55.0),
    ];
    for (nome, antes, t, dano, depois) in casos {
        let mut v = vida(antes, &cfg);
        golpe(&mut v, &cfg, dano, t);
        assert_eq!(v.pontos, depois, "«{nome}»: {antes} → {}", v.pontos);
    }
    // absorve_cheio: nada — nem a cura, e os sinais saem da DIFERENÇA de pontos.
    let mut v = vida(100.0, &cfg);
    golpe(&mut v, &cfg, 10.0, absorve(1.0));
    assert_eq!(v.pontos, 100.0);
    // morte: o dobro de 10 sobre 15 mata, e o golpe seguinte não faz nada.
    let mut v = vida(15.0, &cfg);
    golpe(&mut v, &cfg, 10.0, taxa(2.0));
    assert_eq!(v.pontos, 0.0);
    golpe(&mut v, &cfg, 10.0, taxa(2.0));
    assert_eq!(v.pontos, 0.0);
}

/// ⛔ **A divergência DECLARADA: a casa NÃO arredonda.** O alvo tem vida inteira e dá `3` a
/// `10 × 0,33` e `3` a `10 × 0,25`; a casa, porte do GDevelop com vida `f64`, dá o número.
#[test]
fn a_casa_nao_arredonda_como_o_alvo() {
    let cfg = Config::default();
    let mut v = vida(100.0, &cfg);
    for _ in 0..3 {
        golpe(&mut v, &cfg, 10.0, taxa(0.33));
    }
    // O alvo leu `91` (três golpes de `3`).
    assert!((v.pontos - 90.1).abs() < 1e-9, "{}", v.pontos);
    let mut v = vida(100.0, &cfg);
    golpe(&mut v, &cfg, 10.0, taxa(0.25));
    assert_eq!(v.pontos, 97.5, "o alvo leu 97");
}

/// ⭐ **A taxa entra DEPOIS da armadura** (o plano §3, onde o RPG Maker a põe) — `(10 − 5) × 2 = 10`
/// e não `10 × 2 − 5 = 15`. E ela vale **mesmo quando o golpe atravessa a armadura**: uma
/// resistência não é armadura.
#[test]
fn a_taxa_entra_depois_da_armadura_e_nao_e_armadura() {
    let cfg = Config {
        armadura_fixa: 5.0,
        ..Config::default()
    };
    let mut v = vida(100.0, &cfg);
    golpe(&mut v, &cfg, 10.0, taxa(2.0));
    assert_eq!(v.pontos, 90.0, "a taxa correu antes da armadura");
    let mut v = vida(100.0, &cfg);
    v.golpe_tipado(
        &cfg,
        Regras::CASA,
        10.0,
        taxa(2.0),
        true,
        false,
        &mut || 1.0,
    );
    assert_eq!(v.pontos, 80.0, "sem armadura a taxa continua a valer");
}

/// ⭐ **A [`Taxa::NEUTRA`] é o golpe de antes AO BIT** — sobre uma sequência que passa pela
/// esquiva, a armadura percentual, o escudo e a invencibilidade. (A bancada do oráculo do
/// GDevelop corre pelo `golpe`, que delega aqui — este gate é o espelho dela.)
#[test]
fn a_taxa_neutra_e_o_golpe_de_antes_ao_bit() {
    let cfg = Config {
        armadura_fixa: 1.25,
        armadura_pct: 0.3,
        esquiva: 0.25,
        invencivel_s: 0.1,
        escudo_max: 8.0,
        escudo_duracao_s: 0.0,
        ..Config::default()
    };
    let mut a = vida(100.0, &cfg);
    let mut b = a;
    a.activa_escudo(&cfg, 6.0, true);
    b.activa_escudo(&cfg, 6.0, true);
    let sorteios = [0.9, 0.1, 0.5, 0.2, 0.7, 0.3];
    for (i, dano) in [7.3, 11.0, 2.5, 19.75, 4.0, 13.1].into_iter().enumerate() {
        let s = sorteios[i];
        a.golpe(&cfg, Regras::CASA, dano, true, true, &mut || s);
        b.golpe_tipado(
            &cfg,
            Regras::CASA,
            dano,
            Taxa::NEUTRA,
            true,
            true,
            &mut || s,
        );
        assert_eq!(a, b, "divergiram no golpe {i}");
        a.anda(120.0);
        b.anda(120.0);
    }
}

/// ⭐ **Absorver não é um golpe**: cura dentro da invencibilidade, e não a arma.
#[test]
fn absorver_nao_e_travado_pela_invencibilidade_nem_a_arma() {
    let cfg = Config {
        invencivel_s: 1.0,
        ..Config::default()
    };
    let mut v = vida(50.0, &cfg);
    golpe(&mut v, &cfg, 10.0, Taxa::NEUTRA); // arma a invencibilidade
    assert!(v.invencivel(&cfg));
    golpe(&mut v, &cfg, 10.0, absorve(1.0));
    assert_eq!(v.pontos, 50.0, "a cura foi travada pela invencibilidade");
    let mut v = vida(50.0, &cfg);
    golpe(&mut v, &cfg, 10.0, absorve(1.0));
    assert!(!v.invencivel(&cfg), "absorver armou a invencibilidade");
}

/// ⭐⭐ **Um pulso não é um golpe**: passa DENTRO da invencibilidade, não a arma, não passa pela
/// armadura — e a taxa do tipo continua a valer.
///
/// **Mutação que deve sangrar:** o pulso a armar a invencibilidade (o herói envenenado ficava
/// invencível aos inimigos).
#[test]
fn um_pulso_nao_e_travado_nao_arma_e_nao_passa_pela_armadura() {
    let cfg = Config {
        invencivel_s: 1.0,
        armadura_fixa: 100.0,
        ..Config::default()
    };
    let mut v = vida(100.0, &cfg);
    v.arma_invencibilidade();
    assert!(v.invencivel(&cfg));
    v.pulso(&cfg, Regras::CASA, 3.0, Taxa::NEUTRA, true);
    assert_eq!(
        v.pontos, 97.0,
        "o pulso foi travado (invencibilidade ou armadura)"
    );
    let mut v = vida(100.0, &cfg);
    v.pulso(&cfg, Regras::CASA, 3.0, taxa(2.0), true);
    assert_eq!(v.pontos, 94.0, "a taxa não valeu no pulso");
    assert!(!v.invencivel(&cfg), "o pulso armou a invencibilidade");
    // E um golpe logo a seguir ENTRA.
    golpe(&mut v, &cfg, 150.0, Taxa::NEUTRA);
    assert_eq!(v.pontos, 44.0);
}

/// Corre as aflições `segundos` e devolve a soma dos pulsos e quantos houve.
fn corre(a: &mut Aflicoes, segundos: f64) -> (f64, usize) {
    let passos = (segundos / DT).round() as usize;
    let mut soma = 0.0;
    let mut n = 0;
    for _ in 0..passos {
        for p in a.anda(DT) {
            soma += p.pontos;
            n += 1;
        }
    }
    (soma, n)
}

/// ⭐⭐⭐ **O total é `por_s × dur_s`, SEMPRE** — com um intervalo que divide a duração, com um que
/// não divide (o último pulso leva a fracção) e com intervalo `0` (pulsa a cada tique).
///
/// **Mutação que deve sangrar:** o último pulso a levar o intervalo inteiro em vez do resto.
#[test]
fn o_total_de_uma_aflicao_e_a_taxa_vezes_a_duracao() {
    for (intervalo, pulsos) in [(1.0, 3), (0.7, 5), (0.0, 180)] {
        let mut a = Aflicoes::default();
        a.aplica("veneno", 4.0, 3.0, intervalo);
        let (soma, n) = corre(&mut a, 5.0);
        assert!((soma - 12.0).abs() < 1e-9, "intervalo {intervalo}: {soma}");
        assert_eq!(n, pulsos, "intervalo {intervalo}");
        assert!(a.0.is_empty(), "a aflição não saiu");
    }
}

/// ⭐⭐ **Reaplicar RENOVA e NÃO ADIA o pulso** — com um golpe a cada meio segundo e intervalo de um
/// segundo, o pulso continua a cair de segundo a segundo.
///
/// **Mutação que deve sangrar:** repor a fase do pulso ao reaplicar (o inimigo atingido depressa
/// nunca leva pulso).
#[test]
fn reaplicar_renova_a_duracao_e_nao_adia_o_pulso() {
    let mut a = Aflicoes::default();
    a.aplica("veneno", 2.0, 3.0, 1.0);
    let mut pulsos = 0;
    for tique in 0..(4.0 / DT) as usize {
        if tique % 30 == 0 {
            a.aplica("veneno", 2.0, 3.0, 1.0);
        }
        pulsos += a.anda(DT).len();
    }
    assert_eq!(pulsos, 4, "o pulso foi adiado pela reaplicação");
    // A duração renovou: ainda há veneno aos 4 s (o último golpe foi aos 3,5 s).
    assert!(!a.0.is_empty());
    // E a taxa passa à maior.
    a.aplica("veneno", 5.0, 0.5, 1.0);
    assert_eq!(a.0[0].por_s, 5.0);
    assert!(a.0[0].resta_s > 2.0, "renovar com menos encurtou");
}

/// ⭐ **Um tipo, uma aflição** — dois venenos não se somam; fogo e veneno correm lado a lado.
#[test]
fn um_tipo_uma_aflicao_e_tipos_diferentes_correm_juntos() {
    let mut a = Aflicoes::default();
    a.aplica("veneno", 2.0, 2.0, 1.0);
    a.aplica("veneno", 2.0, 2.0, 1.0);
    a.aplica("fogo", 3.0, 1.0, 1.0);
    assert_eq!(a.0.len(), 2);
    let (soma, _) = corre(&mut a, 3.0);
    assert!((soma - (4.0 + 3.0)).abs() < 1e-9, "{soma}");
}

/// Um pedido sem sentido não faz nada, e a limpeza cura tudo.
#[test]
fn pedidos_sem_sentido_nao_fazem_nada() {
    let mut a = Aflicoes::default();
    for (p, d) in [
        (0.0, 1.0),
        (-1.0, 1.0),
        (1.0, 0.0),
        (f64::NAN, 1.0),
        (1.0, f64::INFINITY),
    ] {
        a.aplica("x", p, d, 1.0);
    }
    assert!(a.0.is_empty());
    a.aplica("x", 1.0, 1.0, 1.0);
    a.limpa();
    assert!(a.anda(DT).is_empty());
}
