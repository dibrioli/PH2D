//! ⭐⭐⭐ **TODA FRASE QUE DIZ POR ONDE UMA CENA CORRE É CONFERIDA CONTRA A ROTA MEDIDA** — os anúncios
//! do terminal e os tutoriais em PDF.
//!
//! ⛔⛔ Medido em 16/10: a `=113` dizia *«Tudo isto corre no dispositivo»* e o terminal dela diz
//! `[motion-route] CPU: fronteira sem estagio de GPU que despache`; a `=111` e a `=112` diziam que só o
//! desenho do quadradinho saía do processador, e é o `Duplicator` com o pano inteiro; a `=56` dizia
//! que recusava a placa e corre inteira nela. Escritas à mão, sem régua — e a régua que existia
//! (`sonda_rota_das_cenas_de_ciclo`) lia só o PLANO, que não é o que o produto decide (as cercas do
//! `cook_gpu` e o `gpu_route` vêm depois dele: a `=113` planeia híbrida e corre no processador).
//!
//! ⭐ **A régua é a do PRODUTO:** cada cena coze pela porta do quadro (`o_quadro`, com a placa) e o
//! veredito é o `route_said` — o texto EXACTO que `PH2D_MOTION_ROUTE_LOG=1` imprime — mais o cone das
//! fronteiras do plano (os cartões que correm no processador) e a decisão do passe de formas.
//!
//! ⭐ **A frase escolhe a classe** ([`classifica`]): citar o terminal (`[motion-route] device: HIBRIDO`,
//! `… o plano inteiro`, `… CPU`, `pela PLACA`) ou dizer que um cartão `X` corre no processador / na
//! placa. Um anúncio fala da cena dele; um tutorial fala da cena dele quando diz *«nesta cena»*, e
//! senão da LEI (o cartão em todas as cenas do roteador). As formas que já mentiram e não têm classe
//! (*«tudo volta para o processador»*, *«sai do processador»*…) reprovam por existir.

use crate::motion_state::MotionState;
use crate::motion_state::demo_router::MAX_DEMO_LEVEL;
use std::collections::BTreeMap;

#[path = "motion_state_demo_carimbo_escala_tests.rs"]
mod carimbo_escala;
#[path = "motion_state_demo_curas_custo_fixo_tests.rs"]
mod curas_do_custo_fixo;
#[path = "motion_state_demo_rota_frases_tests.rs"]
mod frases_de_rota;
#[path = "motion_state_demo_rota_medida_tests.rs"]
mod medida;

use frases_de_rota::*;
use medida::*;

// ───────────────────────────── O CONFRONTO ─────────────────────────────

/// ⭐ **Uma frase contra a medição** — `Err` com o porquê. `cena` é a medição da cena da frase (na
/// variante dela); `todas` são as cenas do roteador, para a LEI.
fn confere(
    classe: &Classe,
    cena: Option<&Medida>,
    todas: &BTreeMap<Cena, Medida>,
) -> Result<(), String> {
    if let Some(c) = classe.crua {
        return Err(format!(
            "«{c}» afirma uma rota sem a medir — cite o terminal ou diga que cartão corre onde"
        ));
    }
    let mut erros = Vec::new();
    let nega = classe.nega;
    let nao = |erros: &mut Vec<String>, ok: bool, porque: String| {
        if ok == nega {
            erros.push(if nega {
                format!("a frase nega e a medição confirma: {porque}")
            } else {
                porque
            });
        }
    };
    let Some(m) = cena else {
        // A LEI: cada cartão em todas as cenas que o têm.
        for a in &classe.afirma {
            let Afirma::Cartoes(lugar, nomes) = a else {
                return Err("uma frase sobre a LEI não cita o terminal (diga «nesta cena»)".into());
            };
            if nomes.is_empty() {
                return Err(format!(
                    "{lugar:?} sem cartão numa frase sobre a LEI — diga qual"
                ));
            }
            for n in nomes {
                let com: Vec<(&Cena, &Medida)> =
                    todas.iter().filter(|(_, m)| tem(&m.titulos, n)).collect();
                if com.is_empty() {
                    erros.push(format!("nenhuma cena do roteador tem o cartão `{n}`"));
                    continue;
                }
                let no_proc: Vec<&Cena> = com
                    .iter()
                    .filter(|(_, m)| tem(&m.no_processador, n))
                    .map(|(c, _)| *c)
                    .collect();
                match lugar {
                    Lugar::Processador => nao(
                        &mut erros,
                        no_proc.len() == com.len(),
                        format!(
                            "`{n}` corre na placa em {:?}",
                            com.iter()
                                .filter(|(c, _)| !no_proc.contains(c))
                                .map(|(c, _)| *c)
                                .collect::<Vec<_>>()
                        ),
                    ),
                    Lugar::Placa | Lugar::Dispositivo => nao(
                        &mut erros,
                        no_proc.len() < com.len(),
                        format!("`{n}` corre no processador em todas as cenas que o têm"),
                    ),
                    Lugar::Toda => {
                        let fora: Vec<&Cena> = com
                            .iter()
                            .filter(|(_, m)| m.rota != Rota::Processador)
                            .map(|(c, _)| *c)
                            .collect();
                        nao(
                            &mut erros,
                            fora.is_empty(),
                            format!("com `{n}` a cena vai à placa em {fora:?}"),
                        );
                    }
                }
            }
        }
        return if erros.is_empty() {
            Ok(())
        } else {
            Err(erros.join(" · "))
        };
    };
    let rota = format!("{:?}", m.razoes);
    for a in &classe.afirma {
        match a {
            Afirma::Terminal(Citacao::Inteira) => {
                nao(
                    &mut erros,
                    m.rota == Rota::Inteira,
                    format!("o terminal diz {rota}"),
                );
            }
            Afirma::Terminal(Citacao::Hibrida) => {
                nao(
                    &mut erros,
                    m.rota == Rota::Hibrida,
                    format!("o terminal diz {rota}"),
                );
            }
            Afirma::Terminal(Citacao::Placa) => {
                nao(
                    &mut erros,
                    m.rota != Rota::Processador,
                    format!("o terminal diz {rota}"),
                );
            }
            Afirma::Terminal(Citacao::Processador) => {
                nao(
                    &mut erros,
                    m.rota == Rota::Processador,
                    format!("o terminal diz {rota}"),
                );
            }
            Afirma::FormasNaPlaca => nao(
                &mut erros,
                m.formas == Some(true),
                format!("o passe de formas: {:?}", m.formas),
            ),
            Afirma::Cartoes(lugar, nomes) => {
                for n in nomes {
                    if !tem(&m.titulos, n) {
                        erros.push(format!("a cena não tem o cartão `{n}`"));
                    }
                }
                let dentro = |n: &String| tem(&m.no_processador, n);
                match (lugar, nomes.is_empty()) {
                    (Lugar::Processador, true) => nao(
                        &mut erros,
                        m.rota == Rota::Processador,
                        format!("a cena não corre toda no processador: {rota}"),
                    ),
                    (Lugar::Processador, false) => nao(
                        &mut erros,
                        nomes.iter().all(dentro),
                        format!(
                            "na placa: {:?} (no processador: {:?})",
                            nomes.iter().filter(|n| !dentro(n)).collect::<Vec<_>>(),
                            m.no_processador
                        ),
                    ),
                    (Lugar::Placa, true) => nao(
                        &mut erros,
                        m.rota != Rota::Processador,
                        format!("a cena corre toda no processador: {rota}"),
                    ),
                    (Lugar::Placa | Lugar::Dispositivo, false) => nao(
                        &mut erros,
                        m.rota != Rota::Processador && !nomes.iter().any(dentro),
                        format!(
                            "no processador: {:?} ({rota})",
                            nomes.iter().filter(|n| dentro(n)).collect::<Vec<_>>()
                        ),
                    ),
                    (Lugar::Dispositivo, true) => nao(
                        &mut erros,
                        m.rota == Rota::Inteira,
                        format!("a cena não corre inteira na placa: {rota}"),
                    ),
                    (Lugar::Toda, _) => nao(
                        &mut erros,
                        m.rota == Rota::Processador,
                        format!("a cena não corre toda no processador: {rota}"),
                    ),
                }
            }
        }
    }
    // ⚠️ **Uma frase HÍBRIDA que diz o que corre no processador di-lo TODO** — toda fronteira do
    // plano está entre os cartões dela (senão *«só o X»* esconde o resto).
    if !classe.nega && classe.afirma.contains(&Afirma::Terminal(Citacao::Hibrida)) {
        let ditos: Vec<&String> = classe
            .afirma
            .iter()
            .filter_map(|a| match a {
                Afirma::Cartoes(Lugar::Processador, n) => Some(n),
                _ => None,
            })
            .flatten()
            .collect();
        if !ditos.is_empty() {
            let faltam: Vec<&String> = m
                .fronteiras
                .iter()
                .filter(|f| !ditos.iter().any(|d| casa(f, d)))
                .collect();
            if !faltam.is_empty() {
                erros.push(format!(
                    "a frase cala fronteiras que correm no processador: {faltam:?}"
                ));
            }
        }
    }
    if erros.is_empty() {
        Ok(())
    } else {
        Err(erros.join(" · "))
    }
}

/// A medição com a variante que a frase manda pôr à frente do comando.
fn ajuste(variante: Option<&str>) -> Result<fn(&mut MotionState), String> {
    match variante {
        None => Ok(|_| {}),
        Some("PH2D_MOTION_DRIVEN_GPU=0") => Ok(|m| m.driven_gpu = false),
        Some("PH2D_GPU_COOK=0") => Ok(|m| m.gpu_enabled = false),
        Some(v) => Err(format!("a variante `{v}` não tem medição neste gate")),
    }
}

// ───────────────────────────── OS GATES ─────────────────────────────

/// ⭐⭐ **Toda frase de rota tem CLASSE e CENA** (sem placa: corre na suíte). As formas cruas reprovam;
/// uma frase de rota num anúncio sem linha na [`ANUNCIOS`] reprova.
///
/// ⚠️ Pisos: as frases de cada lado (contadas em 16/10, depois da cura) e uma de cada classe. ⚠️ O
/// CONTROLO: a frase da `=113` de antes de 16/10 é apanhada como crua, e uma frase híbrida construída
/// dá os cartões a cada lugar.
#[test]
fn toda_frase_de_rota_tem_classe_e_cena() {
    let (anuncios, sem_cena) = frases_dos_anuncios();
    let tuts = frases_dos_tutoriais();
    assert!(
        sem_cena.is_empty(),
        "frase de rota num anúncio sem cena na tabela ANUNCIOS: {sem_cena:#?}"
    );
    let cruas: Vec<_> = anuncios
        .iter()
        .chain(&tuts)
        .filter(|a| a.classe.crua.is_some())
        .map(|a| format!("{}: «{}»", a.onde, a.frase))
        .collect();
    assert!(cruas.is_empty(), "frases de rota sem classe: {cruas:#?}");
    assert!(
        anuncios.len() >= 16 && tuts.len() >= 10,
        "piso: {} frases nos anúncios e {} nos tutoriais",
        anuncios.len(),
        tuts.len()
    );
    let todas: Vec<&Afirma> = anuncios
        .iter()
        .chain(&tuts)
        .flat_map(|a| &a.classe.afirma)
        .collect();
    for (nome, viu) in [
        (
            "híbrida",
            todas.contains(&&Afirma::Terminal(Citacao::Hibrida)),
        ),
        (
            "inteira",
            todas.contains(&&Afirma::Terminal(Citacao::Inteira)),
        ),
        (
            "processador",
            todas.contains(&&Afirma::Terminal(Citacao::Processador)),
        ),
        ("formas", todas.contains(&&Afirma::FormasNaPlaca)),
        (
            "cartão",
            todas
                .iter()
                .any(|a| matches!(a, Afirma::Cartoes(_, n) if !n.is_empty())),
        ),
    ] {
        assert!(viu, "piso: nenhuma frase da classe {nome}");
    }
    // O CONTROLO.
    let antes = classifica(
        "Tudo isto corre no dispositivo -- so' o desenho do quadradinho (UM) sai do processador.",
    );
    assert!(antes.crua.is_some(), "a frase de antes é crua: {antes:?}");
    let h = classifica(
        "O terminal diz `[motion-route] device: HIBRIDO`: o `Shape`, o `Grid` e o `Duplicator` correm no processador; o `Twist` corre na placa.",
    );
    assert_eq!(
        h.afirma,
        vec![
            Afirma::Terminal(Citacao::Hibrida),
            Afirma::Cartoes(
                Lugar::Processador,
                vec!["Shape".into(), "Grid".into(), "Duplicator".into()]
            ),
            Afirma::Cartoes(Lugar::Placa, vec!["Twist".into()]),
        ]
    );
    let n = classifica("DEU ERRADO se o terminal disser `[motion-route] CPU: ...`.");
    assert!(n.nega && n.afirma == vec![Afirma::Terminal(Citacao::Processador)]);
}

/// ⭐⭐⭐ **Toda frase de rota confere com a rota MEDIDA** — cada cena do roteador pela porta do quadro,
/// na placa.
///
/// ⚠️ O CONTROLO: frases FALSAS construídas sobre cenas medidas reprovam — a `=113` inteira na placa,
/// o `Duplicator` da `=111` na placa, o `Delay` na placa como LEI, e a híbrida que cala uma fronteira.
#[test]
#[ignore = "placa: precisa de GPU"]
fn toda_frase_de_rota_confere_com_a_rota_medida() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        panic!("sem adapter — este gate mede a rota do device");
    };
    let mut todas: BTreeMap<Cena, Medida> = BTreeMap::new();
    for n in 1..=MAX_DEMO_LEVEL {
        let c = Cena::Demo(n.to_string());
        if let Some(m) = mede(&gpu, &c, |_| {}) {
            todas.insert(c, m);
        }
    }
    assert!(todas.len() >= 140, "piso: {} cenas medidas", todas.len());
    let (anuncios, _) = frases_dos_anuncios();
    let mut erros = Vec::new();
    let mut conferidas = 0usize;
    for a in anuncios.iter().chain(&frases_dos_tutoriais()) {
        let medida = match (&a.cena, ajuste(a.classe.variante.as_deref())) {
            (_, Err(e)) => {
                erros.push(format!("{}: {e}", a.onde));
                continue;
            }
            (None, Ok(_)) => None,
            (Some(c), Ok(f)) => match (a.classe.variante.is_none(), todas.get(c)) {
                (true, Some(m)) => Some(m.clone()),
                _ => mede(&gpu, c, f),
            },
        };
        if a.cena.is_some() && medida.is_none() {
            erros.push(format!("{}: a cena {:?} não se mede", a.onde, a.cena));
            continue;
        }
        conferidas += 1;
        if let Err(e) = confere(&a.classe, medida.as_ref(), &todas) {
            erros.push(format!("{} ({:?}): «{}» → {e}", a.onde, a.cena, a.frase));
        }
    }
    assert!(
        erros.is_empty(),
        "frases de rota que a medição desmente:\n{}",
        erros.join("\n")
    );
    assert!(conferidas >= 26, "piso: {conferidas} frases conferidas");
    // O CONTROLO — frases falsas construídas.
    let cena = |n: &str| todas.get(&Cena::Demo(n.into())).expect("medida");
    for (n, falsa) in [
        (
            "113",
            "O terminal diz `[motion-route] device: o plano inteiro (fully-GPU)`.",
        ),
        ("111", "O `Duplicator` corre no processador."),
        ("113", "Tudo isto corre no dispositivo."),
        (
            "116",
            "DEU ERRADO se o terminal disser `[motion-route] device: HIBRIDO`.",
        ),
        (
            "112",
            "O terminal diz `[motion-route] device: HIBRIDO`: o `Falloff` corre no processador.",
        ),
    ] {
        assert!(
            confere(&classifica(falsa), Some(cena(n)), &todas).is_err(),
            "o controlo «{falsa}» sobre a ={n} passou"
        );
    }
    assert!(
        confere(&classifica("O `Delay` corre na placa."), None, &todas).is_err(),
        "o controlo da LEI passou"
    );
}

/// **SONDA** — a rota de cada cena do roteador, como o gate a mede.
#[test]
#[ignore = "sonda: precisa de GPU"]
fn sonda_rota_de_cada_cena_do_roteador() {
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("sem adapter");
    let mut cenas: Vec<Cena> = (1..=MAX_DEMO_LEVEL)
        .map(|n| Cena::Demo(n.to_string()))
        .collect();
    cenas.extend([Cena::Forma(1), Cena::Forma(2)]);
    for c in cenas {
        if let Some(m) = mede(&gpu, &c, |_| {}) {
            eprintln!(
                "{c:?}: {:?} · fronteiras {:?} · no processador {:?} · formas {:?}\n    cartões {:?}",
                m.razoes, m.fronteiras, m.no_processador, m.formas, m.titulos
            );
        }
    }
}

/// **SONDA** — toda frase de rota achada, a classe dela e a cena.
#[test]
#[ignore = "sonda: corra à mão"]
fn sonda_frases_de_rota() {
    let (anuncios, sem) = frases_dos_anuncios();
    for a in anuncios.iter().chain(&frases_dos_tutoriais()) {
        eprintln!(
            "{} {:?}: {:?} nega={} var={:?}\n    «{}»",
            a.onde, a.cena, a.classe.afirma, a.classe.nega, a.classe.variante, a.frase
        );
    }
    eprintln!("sem cena: {sem:?}");
}
