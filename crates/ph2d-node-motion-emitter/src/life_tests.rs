//! Os gates da **VARIÂNCIA DE VIDA** (doc 89, folha 01) — e a medição que FECHA a
//! célula irmã, a da emissão por DISTÂNCIA.

use super::tests::spec;
use super::*;

/// ⭐ **`life_random = 0` DEVOLVE O NÓ DE SEMPRE, COLUNA A COLUNA, AO BIT.**
#[test]
fn a_zero_life_random_reproduces_every_column_bit_for_bit() {
    let mut a = spec();
    a.life = 2.0;
    a.max = 64;
    let s0 = emit(&a, 1.7);
    let mut b = a;
    b.life_random = 0.0;
    let s1 = emit(&b, 1.7);
    assert!(s0.count() > 10, "CONTROLE: ha' particulas ({})", s0.count());
    assert_eq!(s0.count(), s1.count());
    for col in ["P", "vel", "size"] {
        let (Some(Column::Vec2(x)), Some(Column::Vec2(y))) = (s0.get(col), s1.get(col)) else {
            panic!("{col}")
        };
        for (i, (u, v)) in x.iter().zip(y).enumerate() {
            assert_eq!(u.map(f32::to_bits), v.map(f32::to_bits), "{col}[{i}]");
        }
    }
    for col in ["id", "age", "life", "Index", "Count"] {
        let (Some(Column::Scalar(x)), Some(Column::Scalar(y))) = (s0.get(col), s1.get(col)) else {
            panic!("{col}")
        };
        for (i, (u, v)) in x.iter().zip(y).enumerate() {
            assert_eq!(u.to_bits(), v.to_bits(), "{col}[{i}]");
        }
    }
}

/// ⭐⭐ **A VARIÂNCIA SÓ ENCURTA — `life` é o TETO da janela.**
///
/// ⚠️ É a metade da célula que decide a forma da cura: uma partícula que pudesse viver
/// MAIS que `life` sairia da janela `[t−life, t]` que a `window` calcula por aritmética
/// sobre `life`, e o nó nunca a veria. *A direção não é uma escolha de gosto: é a única
/// que não contradiz a lei da contagem.*
#[test]
fn the_variance_only_shortens_and_never_outlives_the_window() {
    let mut s = spec();
    s.life = 2.0;
    s.max = 256;
    s.life_random = 0.6;
    let out = emit(&s, 1.9);
    let Some(Column::Scalar(lives)) = out.get("life") else {
        panic!("life")
    };
    let Some(Column::Scalar(ages)) = out.get("age") else {
        panic!("age")
    };
    assert!(lives.len() > 10, "ha' particulas: {}", lives.len());
    let (lo, hi) = lives
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), v| (a.min(*v), b.max(*v)));
    assert!(hi <= s.life + 1e-6, "alguem viveu mais que o teto: {hi}");
    assert!(
        lo >= s.life * (1.0 - 0.6) - 1e-6,
        "alguem viveu menos que o piso: {lo}"
    );
    assert!(
        hi - lo > s.life * 0.3,
        "a faixa esta' colapsada: {lo}..{hi}"
    );
    // E ninguém emitido está morto — a coluna `life` é a MESMA que decidiu quem fica.
    for (i, (a, l)) in ages.iter().zip(lives).enumerate() {
        assert!(
            a <= l,
            "particula {i} emitida ja' morta: idade {a}, vida {l}"
        );
    }
}

/// ⭐ **E ela MATA:** com a mesma janela, a lista encurta.
#[test]
fn the_variance_thins_the_alive_set() {
    let mut s = spec();
    s.life = 2.0;
    s.max = 256;
    let full = emit(&s, 1.9).count();
    s.life_random = 0.8;
    let thin = emit(&s, 1.9).count();
    assert!(
        thin < full && thin > full / 4,
        "a variancia rala sem apagar: {full} -> {thin}"
    );
}

/// ⚠️ **A vida de uma partícula é dela, e não muda ao longo do tempo** — a mesma
/// propriedade que faz o scrub deste nó ser grátis. Uma vida que dependesse da posição
/// na lista faria uma partícula morrer duas vezes ao arrastar o playhead.
#[test]
fn a_particles_life_is_the_same_at_every_playhead() {
    let mut s = spec();
    s.life = 2.0;
    s.max = 256;
    s.life_random = 0.5;
    let at = |t: f32| -> Vec<(u32, f32)> {
        let out = emit(&s, t);
        let (Some(Column::Scalar(ids)), Some(Column::Scalar(l))) = (out.get("id"), out.get("life"))
        else {
            panic!()
        };
        ids.iter().zip(l).map(|(i, v)| (*i as u32, *v)).collect()
    };
    let (a, b) = (at(1.5), at(1.7));
    let mut shared = 0;
    for (id, life) in &a {
        if let Some((_, other)) = b.iter().find(|(j, _)| j == id) {
            assert_eq!(
                life.to_bits(),
                other.to_bits(),
                "a particula {id} mudou de vida"
            );
            shared += 1;
        }
    }
    assert!(
        shared > 5,
        "CONTROLE: as duas fotos partilham particulas ({shared})"
    );
}

/// ⛔ A fronteira do device NOMEIA-A, pelo mesmo mecanismo do `probability`.
#[test]
fn the_life_variance_falls_off_the_device() {
    let applicable = GPU_KERNEL.applicable.expect("a fronteira e' declarada");
    let with = |lr: f32| {
        applicable(&move |name: &str| match name {
            "probability" => 1.0,
            n if n == LIFE_RANDOM => lr,
            _ => 0.0,
        })
    };
    assert!(with(0.0), "o default continua no device");
    assert!(!with(0.3), "a variancia cai para a CPU");
}

/// ⛔⛔ **A EMISSÃO POR DISTÂNCIA É UMA RECUSA MEDIDA — e a nota que a mantinha aberta
/// estava ERRADA sobre o substrato.**
///
/// A célula dizia: *"a origem é função do playhead, logo o comprimento de arco é uma
/// integral que temos em forma fechada"*. **Não temos.** A origem é conduzida por uma
/// sub-árvore arbitrária; o que existe dela é o LEQUE DE TEMPO (ADR-0163), que devolve
/// AMOSTRAS. E o que a emissão por distância precisa é do arco **absoluto** `S(t)`,
/// porque a identidade de uma partícula é `floor(S(τ)·por_unidade)` no instante `τ` em
/// que ela nasceu — e a identidade é o que todas as pistas de hash deste nó indexam.
///
/// ⭐ **As duas saídas foram medidas, e elas trocam uma pela outra:**
///
/// 1. **Resolução fixa** (a de hoje, `HISTORY_HZ`): a identidade é estável, e o leque
///    cobre `MAX_HISTORY / HISTORY_HZ` segundos — o número que este gate imprime. Uma
///    linha do tempo não tem esse teto, e o custo cresce **sem limite** com `t`.
/// 2. **Orçamento fixo** (N amostras esticadas sobre `[0,t]`): o custo fica limitado e a
///    CONTAGEM até fica boa — mas as amostras mudam de sítio a cada quadro, o arco de um
///    instante PASSADO é reestimado, e `floor(S(τ)·k)` **muda**. Uma renumeração
///    re-sorteia ângulo, velocidade e tamanho de todo o penacho: é a doença do §0.2
///    desta folha, a mesma que já matou `value.instance_field(Random)` aqui.
///
/// ⇒ **contar as partículas é fácil e NOMEÁ-LAS é que não** — e um nó que sabe quantas
/// são mas não sabe quem são não as consegue fazer. A cura seria um acumulador monótono
/// que atravessa tiques e que o replay reproduz, que é exactamente o ring de
/// checkpoints do `sim.zone` — e o que define este nó é não ter nenhum.
#[test]
fn the_distance_mode_needs_an_identity_the_time_fan_cannot_give() {
    // (1) Quanto tempo um leque de resolução FIXA cobre, antes de o tecto morder.
    let covered = {
        let mut lo = 0.0_f32;
        let mut hi = 4096.0_f32;
        for _ in 0..40 {
            let mid = f32::midpoint(lo, hi);
            if history_samples(mid) < 1024 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        lo
    };
    println!("leque de resolucao fixa cobre {covered:.2} s antes do tecto");
    assert!(
        (4.0..5.0).contains(&covered),
        "o tecto medido cobre ~4,27 s: {covered:.3}"
    );

    // (2) Com ORÇAMENTO fixo, o arco de um instante PASSADO é reestimado a cada quadro.
    // Um emissor a orbitar: `p(t) = (cos, sin)` de raio 1, a uma volta por segundo.
    let arc_to = |tau: f32, horizon: f32, budget: usize| -> f32 {
        let step = horizon / (budget - 1) as f32;
        let mut s = 0.0;
        let mut prev = [1.0_f32, 0.0];
        let mut k = 1;
        while (k as f32) * step <= tau {
            let t = k as f32 * step;
            let (c, sn) = (
                (t * std::f32::consts::TAU).cos(),
                (t * std::f32::consts::TAU).sin(),
            );
            s += (c - prev[0]).hypot(sn - prev[1]);
            prev = [c, sn];
            k += 1;
        }
        s
    };
    const BUDGET: usize = 1024;
    const PER_UNIT: f32 = 40.0;
    let tau = 3.0_f32;
    let ids: Vec<i64> = [8.0_f32, 12.0, 20.0, 40.0]
        .iter()
        .map(|hz| (arc_to(tau, *hz, BUDGET) * PER_UNIT) as i64)
        .collect();
    println!("o id da particula nascida em t={tau}s, por horizonte: {ids:?}");
    assert!(
        ids.iter().any(|v| *v != ids[0]),
        "CONTROLE FALHOU: a identidade tinha de derivar com o horizonte ({ids:?}) -- \
         sem essa deriva a recusa nao teria mecanismo"
    );
}
