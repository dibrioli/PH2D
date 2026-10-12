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

/// As mesmas entradas com as mesmas revisões.
fn mesmas(a: &All, b: &All) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|((na, ea), (nb, eb))| na == nb && ea.rev == eb.rev)
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
        assert!(mesmas(&estado, &f.estado_em(t).unwrap()), "tique {t}");
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
    f.grava(0, &pub_(&[("g", 0.0)]));
    let fixo = f.bytes();
    f.grava(1, &pub_(&[("g", 1.0)]));
    let um = f.bytes() - fixo;
    let teto = fixo + um * 9;
    f.set_orcamento(teto);
    for t in 2..100u64 {
        f.grava(t, &pub_(&[("g", t as f32)]));
        assert!(f.bytes() <= teto, "tique {t}: {} bytes", f.bytes());
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
    // Tocar de novo sobre o chão gravado (o `Igual`, tique a tique) e ler um tique qualquer (o
    // início de um scrub): antes da âncora e do cursor, os dois refaziam a história desde a base.
    let mut sobre = Vec::new();
    for t in 700..800u64 {
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
        assert_eq!(f.grava(t, &agora), Gravacao::Igual);
        sobre.push(t0.elapsed().as_secs_f64() * 1e3);
    }
    let mut ler = Vec::new();
    for t in (0..T).step_by(97) {
        let t0 = std::time::Instant::now();
        let s = f.estado_em(t).expect("gravado");
        ler.push(t0.elapsed().as_secs_f64() * 1e3);
        assert_eq!(s.len(), K);
    }
    let maximo = |v: &[f64]| v.iter().copied().fold(0.0, f64::max);
    eprintln!(
        "fita: tocar sobre o gravado {:.4} ms/tique (o mínimo) · ler um tique qualquer {:.4}–{:.4} ms",
        minimo(&sobre),
        minimo(&ler),
        maximo(&ler)
    );
    eprintln!(
        "fita: size_of Mudanca {} · Option<Valor> {} B",
        std::mem::size_of::<Mudanca>(),
        std::mem::size_of::<Option<Valor>>()
    );
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
            if !(t + e).is_multiple_of(3) {
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

/// O publicado num tique de uma «história» `h`: seis nomes, cada um muda a seu ritmo, às vezes
/// some, e o `e5` tem duas colunas.
fn publicado(t: u64, h: u64) -> All {
    let mut a = All::new();
    for e in 0..6u64 {
        if (t * 13 + e * 5 + h).is_multiple_of(11) {
            continue;
        }
        let x = (t / (e + 1) + h * 1000) as f32;
        let mut s = Stream::new(1).with("v", Column::Scalar(vec![x]));
        if e == 5 {
            s = s.with("P", Column::Vec2(vec![[x, -x]]));
        }
        a.insert(
            format!("e{e}"),
            External {
                rev: ph2d_nodegraph::external::fingerprint(&s),
                value: s,
            },
        );
    }
    a
}

/// ⭐ **Um passeio de edição contra um MODELO** (doc 131 §7, C.11): tocar para a frente (com
/// tiques saltados), voltar e tocar sobre o gravado (`Igual`, pelo cursor), editar no mesmo tique
/// (`Ramo` no cursor), voltar e divergir, voltar para antes do primeiro — e, com um orçamento curto,
/// a dobra. Depois de cada passo, todo tique gravado (e não dobrado) lê-se como o modelo, por
/// `estado_em` e andando com `avanca` — atravessando várias âncoras.
#[test]
fn um_passeio_de_edicao_le_se_como_o_modelo() {
    for orcamento in [usize::MAX, 24 * 1024] {
        let mut f = Fita::new();
        f.set_orcamento(orcamento);
        let mut modelo: BTreeMap<u64, All> = BTreeMap::new();
        let mut semente = 0x9e37_79b9_7f4a_7c15_u64;
        let mut sorte = |n: u64| {
            semente = semente
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            (semente >> 33) % n
        };
        let mut t = 10u64;
        let mut h = 0u64;
        let mut ancoras = 0;
        let mut dobras = 0;
        for passo in 0..60 {
            let ops = sorte(5);
            let n = 5 + sorte(60);
            for _ in 0..n {
                match ops {
                    // Para a frente, às vezes saltando tiques.
                    0 | 1 => t += 1 + sorte(2) * sorte(3),
                    // Voltar para um tique gravado e tocar sobre ele.
                    2 if !modelo.is_empty() => {
                        let p = *modelo.keys().next().unwrap();
                        let u = *modelo.keys().next_back().unwrap();
                        t = p + sorte(u - p + 1);
                    }
                    // Editar no mesmo tique (a história muda, o tique não).
                    3 => h += 1,
                    // Voltar e divergir.
                    _ => {
                        t = t.saturating_sub(sorte(40));
                        h += 1;
                    }
                }
                let a = publicado(t, h);
                let esperado = match (modelo.keys().next(), modelo.keys().next_back()) {
                    (Some(&p), Some(&u)) if p <= t && t <= u => {
                        let antes = modelo.range(..=t).next_back().unwrap().1;
                        if mesmas(antes, &a) {
                            Gravacao::Igual
                        } else {
                            Gravacao::Ramo
                        }
                    }
                    (Some(&p), _) if t < p => Gravacao::Ramo,
                    _ => Gravacao::Nova,
                };
                let g = f.grava(t, &a);
                if orcamento == usize::MAX {
                    assert_eq!(g, esperado, "passo {passo}, t={t}");
                }
                match g {
                    Gravacao::Igual => {}
                    Gravacao::Nova => {
                        modelo.insert(t, a);
                    }
                    Gravacao::Ramo => {
                        if modelo.keys().next().is_some_and(|&p| t < p) {
                            modelo.clear();
                        }
                        modelo.retain(|k, _| *k < t);
                        modelo.insert(t, a);
                    }
                }
            }
            let (p, u) = (
                *modelo.keys().next().unwrap(),
                *modelo.keys().next_back().unwrap(),
            );
            let desde = f.base_ate.map_or(p, |b| b + 1).max(p);
            let mut andando: Option<All> = None;
            for k in desde..=u {
                let certo = modelo.range(..=k).next_back().unwrap().1;
                let lido = f.estado_em(k).expect("gravado");
                assert!(mesmas(&lido, certo), "passo {passo} ({orcamento}), t={k}");
                match andando.as_mut() {
                    Some(e) => {
                        f.avanca(k, e);
                        assert!(mesmas(e, certo), "avanca: passo {passo}, t={k}");
                    }
                    None => andando = Some(lido),
                }
            }
            assert!(f.estado_em(u + 1).is_none());
            ancoras = ancoras.max(f.ancoras.len());
            dobras += usize::from(f.base_ate.is_some());
        }
        if orcamento != usize::MAX {
            assert!(
                dobras >= 10,
                "o orçamento curto tem de dobrar ({dobras} passos)"
            );
            assert!(f.bytes() <= orcamento);
        }
        assert!(
            ancoras >= 3 || orcamento != usize::MAX,
            "as âncoras foram exercidas ({ancoras})"
        );
        eprintln!(
            "passeio ({orcamento} B): até {ancoras} âncoras, {dobras} passos com a base dobrada"
        );
    }
}
