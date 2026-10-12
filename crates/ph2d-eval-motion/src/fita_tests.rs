use super::*;
use ph2d_nodegraph::attr::{Column, Stream};

fn ext(v: f32) -> External {
    let value = Stream::new(1).with("v", Column::Scalar(vec![v]));
    External {
        rev: ph2d_nodegraph::external::fingerprint(&value),
        value,
    }
}

fn pub_(pares: &[(&str, f32)]) -> All {
    pares
        .iter()
        .map(|(n, v)| ((*n).to_string(), ext(*v)))
        .collect()
}

fn v(all: &All, nome: &str) -> Option<f32> {
    match all.get(nome)?.value.get("v")? {
        Column::Scalar(x) => x.first().copied(),
        _ => None,
    }
}

#[test]
fn each_tick_reads_back_what_was_published_and_unchanged_ticks_cost_nothing() {
    let mut f = Fita::new();
    for t in 0..10u64 {
        let g = if t < 5 { 1.0 } else { 2.0 };
        assert_eq!(
            f.grava(t, &pub_(&[("g", g), ("parado", 7.0)])),
            Gravacao::Nova
        );
    }
    assert_eq!(
        f.marcas.len(),
        2,
        "só as mudanças (o tique 0 e o 5) se gravam"
    );
    for t in 0..10u64 {
        let s = f.estado_em(t).unwrap();
        assert_eq!(v(&s, "g"), Some(if t < 5 { 1.0 } else { 2.0 }), "tique {t}");
        assert_eq!(v(&s, "parado"), Some(7.0));
    }
    assert!(
        f.estado_em(10).is_none(),
        "para lá do gravado não se inventa"
    );
}

#[test]
fn a_name_that_stops_being_published_is_absent_at_its_tick() {
    let mut f = Fita::new();
    f.grava(0, &pub_(&[("a", 1.0), ("b", 2.0)]));
    f.grava(1, &pub_(&[("a", 1.0)]));
    assert!(!f.estado_em(1).unwrap().contains_key("b"));
    assert_eq!(v(&f.estado_em(0).unwrap(), "b"), Some(2.0));
}

#[test]
fn walking_tick_by_tick_equals_rebuilding_each_tick() {
    let mut f = Fita::new();
    for t in 0..30u64 {
        f.grava(t, &pub_(&[("g", (t / 4) as f32)]));
    }
    let mut estado = f.estado_em(3).unwrap();
    for t in 4..30u64 {
        f.avanca(t, &mut estado);
        assert!(
            mesmas_revisoes(&estado, &f.estado_em(t).unwrap()),
            "tique {t}"
        );
    }
}

#[test]
fn replaying_over_recorded_ground_is_equal_and_a_different_input_is_a_branch() {
    let mut f = Fita::new();
    for t in 0..20u64 {
        f.grava(t, &pub_(&[("g", t as f32)]));
    }
    assert_eq!(f.grava(7, &pub_(&[("g", 7.0)])), Gravacao::Igual);
    assert_eq!(f.grava(8, &pub_(&[("g", 99.0)])), Gravacao::Ramo);
    assert_eq!(v(&f.estado_em(8).unwrap(), "g"), Some(99.0));
    assert_eq!(
        v(&f.estado_em(7).unwrap(), "g"),
        Some(7.0),
        "o passado do ramo fica"
    );
    assert!(f.estado_em(9).is_none(), "o futuro velho foi esquecido");
    assert_eq!(f.grava(9, &pub_(&[("g", 100.0)])), Gravacao::Nova);
}

#[test]
fn recording_before_the_first_tick_starts_over() {
    let mut f = Fita::new();
    for t in 10..20u64 {
        f.grava(t, &pub_(&[("g", 1.0)]));
    }
    assert!(f.estado_em(5).is_none());
    assert_eq!(f.grava(5, &pub_(&[("g", 3.0)])), Gravacao::Ramo);
    assert!(f.estado_em(12).is_none());
    assert_eq!(v(&f.estado_em(5).unwrap(), "g"), Some(3.0));
}

#[test]
fn over_budget_the_oldest_change_folds_into_the_base_and_the_bytes_stay_bounded() {
    let mut f = Fita::new();
    let um = peso(&vec![("g".to_string(), Some(ext(0.0)))]);
    f.set_orcamento(um * 10);
    for t in 0..100u64 {
        f.grava(t, &pub_(&[("g", t as f32)]));
        assert!(f.bytes() <= um * 10, "tique {t}: {} bytes", f.bytes());
    }
    assert_eq!(v(&f.estado_em(99).unwrap(), "g"), Some(99.0));
    assert_eq!(
        v(&f.estado_em(95).unwrap(), "g"),
        Some(95.0),
        "o recente fica exacto"
    );
    // O limite NOMEADO: antes da base, repete-se a base.
    assert_eq!(
        v(&f.estado_em(3).unwrap(), "g"),
        v(&f.estado_em(f.base_ate.unwrap()).unwrap(), "g")
    );
}

/// O RSS do processo, em bytes (Linux; `0` noutro sítio).
fn rss() -> usize {
    std::fs::read_to_string("/proc/self/statm")
        .ok()
        .and_then(|s| s.split_whitespace().nth(1)?.parse::<usize>().ok())
        .map_or(0, |paginas| paginas * 4096)
}

/// **A RÉGUA DA FITA contra a memória REAL** (doc 131 §4, C.11): `K` entradas a mudar em todo tique,
/// sem orçamento; o que o `bytes()` declara por mudança contra o que o processo ganhou de RSS.
#[test]
#[ignore = "sonda: memória do processo"]
fn sonda_a_fita_declara_o_que_ocupa() {
    use ph2d_nodegraph::attr::{Column, Stream};
    use ph2d_nodegraph::external::External;
    const K: usize = 300;
    const T: u64 = 1_500;
    let nomes: Vec<String> = (0..K).map(|e| format!("entrada_{e}")).collect();
    let mut f = Fita::new();
    f.set_orcamento(usize::MAX);
    let antes = rss();
    let mut agora = All::new();
    let mut tempos = Vec::with_capacity(T as usize);
    for t in 0..T {
        for (e, nome) in nomes.iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "um valor de teste")]
            let v = (t as usize + e) as f32;
            agora.insert(
                nome.clone(),
                External {
                    rev: t * 1_000 + e as u64,
                    value: Stream::new(1).with("v", Column::Scalar(vec![v])),
                },
            );
        }
        let t0 = std::time::Instant::now();
        f.grava(t, &agora);
        tempos.push(t0.elapsed().as_secs_f64() * 1e3);
    }
    let minimo = |v: &[f64]| v.iter().copied().fold(f64::MAX, f64::min);
    eprintln!(
        "fita: gravar um tique — os 100 primeiros {:.4} ms · os 100 últimos {:.4} ms (o mínimo)",
        minimo(&tempos[..100]),
        minimo(&tempos[tempos.len() - 100..])
    );
    let real = rss().saturating_sub(antes);
    #[expect(clippy::cast_precision_loss, reason = "uma razão")]
    let mudancas = (K as u64 * T) as f64;
    #[expect(clippy::cast_precision_loss, reason = "uma razão")]
    let (declarado, ocupado) = (f.bytes() as f64 / mudancas, real as f64 / mudancas);
    eprintln!(
        "fita: {K} entradas × {T} tiques — declara {declarado:.1} B/mudança, o processo ganhou {ocupado:.1} B/mudança ({:.1}×)",
        ocupado / declarado
    );
}

/// ⭐ **O QUE A FITA DEVOLVE É O QUE SE PUBLICOU** (doc 131 §7, C.11) — sobre uma sequência que passa
/// pelos três caminhos da gravação (tique NOVO no fim, que lê a `cauda`; o mesmo tique outra vez; um
/// RAMO para trás) e pela DOBRA (orçamento curto): todo tique gravado e ainda não dobrado lê-se igual
/// ao publicado nele.
#[test]
fn o_que_a_fita_devolve_e_o_que_se_publicou_em_cada_tique() {
    let mut f = Fita::new();
    let mut verdade: BTreeMap<u64, All> = BTreeMap::new();
    let publica = |t: u64, ramo: u64| -> All {
        let mut a = All::new();
        for e in 0..4u64 {
            if (t + e) % 3 != 0 {
                a.insert(format!("e{e}"), ext((t * 7 + e + ramo * 1000) as f32));
            }
        }
        a
    };
    // Para a frente, com tiques repetidos (a tocar sobre chão gravado).
    for t in 0..40 {
        let a = publica(t, 0);
        f.grava(t, &a);
        f.grava(t, &a);
        verdade.insert(t, a);
    }
    // Um RAMO em 25 e de novo para a frente: o futuro velho é esquecido.
    verdade.retain(|t, _| *t < 25);
    for t in 25..60 {
        let a = publica(t, 1);
        f.grava(t, &a);
        verdade.insert(t, a);
    }
    for (t, a) in &verdade {
        let lido = f.estado_em(*t).expect("gravado");
        assert_eq!(
            lido.keys().collect::<Vec<_>>(),
            a.keys().collect::<Vec<_>>(),
            "t={t}: as entradas"
        );
        for (nome, e) in a {
            assert_eq!(lido[nome].rev, e.rev, "t={t} {nome}");
        }
    }
    // E com um orçamento que dobra: o que ficou depois da base continua verdadeiro.
    let um = f.bytes() / 60;
    let mut g = Fita::new();
    g.set_orcamento(um * 10);
    for t in 0..60 {
        g.grava(t, &publica(t, 0));
    }
    for t in 55..60 {
        let lido = g.estado_em(t).expect("gravado");
        for (nome, e) in &publica(t, 0) {
            assert_eq!(lido[nome].rev, e.rev, "dobrada, t={t} {nome}");
        }
    }
}
