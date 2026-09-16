//! ⭐⭐⭐ **AS SONDAS DA VELOCIDADE ANGULAR** — o 8.º report do dono (*«umas caixas rodam, outras
//! parecem não rotacionar»*), medido no estado e não no resultado.
//!
//! Irmã da [`super::salto_diag`] pelo tecto de LOC (HR-18) e por ASSUNTO: ali medem-se o SALTO e os
//! apoios; aqui mede-se o que a pilha faz com a rotação que PERSISTE.
//!
//! ⚠️ **Todas elas medem a lei que o `sim.step` tem em vigor AGORA** (o `contact::LEIS`), e a
//! varredura corre-as por MUTAÇÃO daquele `const` — ver [`super::obra::probe_a_linha_das_leis`],
//! que explica porque a varredura não é por parâmetro.

/// ⭐⭐⭐ **SONDA — O GIRO QUE SOBRA: a pilha assente ainda tem VELOCIDADE ANGULAR?**
///
/// O rodopio (`Σ Δrot` na janela) diz que a pilha roda e **não diz porquê**: pode ser energia que
/// nunca se dissipa, ou uma deriva lenta que nada trava. Esta sonda olha para o ESTADO — o `spin` de
/// cada peça, tique a tique, na janela assente — em vez de olhar para o resultado dele.
///
/// ⚠️ **Ela mede a lei que o `sim.step` tem em vigor AGORA** (o `contact::LEIS`); a varredura corre-a
/// por mutação daquele `const`, como a [`super::obra::probe_a_linha_das_leis`].
///
/// ```text
/// cargo test -p ph2d-app-motion --release probe_o_giro_que_sobra -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_o_giro_que_sobra() {
    use ph2d_nodegraph::attr::Column as C;
    let sub = {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "uma contagem de sub-passos pequena"
        )]
        let s = super::SUBSTEPS as u32;
        s
    };
    let (mut state, sink) = super::obra::com_substeps(0.0, sub);
    let escopos = ph2d_nodegraph::cook::TimeScopes::new();
    eprintln!("\n  tique | spin p50 | spin max | |rot| p50 | |rot| max | spin!=0");
    eprintln!("  ------|----------|----------|-----------|-----------|--------");
    for k in 0..=174_u64 {
        state.pump.mark_dirty();
        state.pump.advance_or_scrub_to_nodes_scoped(
            &state.doc.graph,
            &state.registry,
            &[sink],
            k,
            |t| {
                #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
                let s = t as f64 / 60.0;
                s
            },
            &escopos,
        );
        if k % 20 != 0 && k != 174 {
            continue;
        }
        let Some((_, saida)) = state
            .pump
            .boundary_streams()
            .iter()
            .find(|(no, _)| *no == sink)
        else {
            continue;
        };
        let col = |nome: &str| match saida.get(nome) {
            Some(C::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let (spin, rot) = (col("spin"), col("rot"));
        // ⚠️ **Piso de população**: uma coluna ausente devolve um vector VAZIO, e toda estatística
        // sobre ele lê `0,000` — que é exactamente como se lê «a cura não faz nada».
        if spin.is_empty() && rot.is_empty() {
            eprintln!("  {k:>5} | (sem colunas `spin`/`rot` — a cena nao gira)");
            continue;
        }
        let resumo = |v: &[f32]| -> (f32, f32) {
            if v.is_empty() {
                return (0.0, 0.0);
            }
            let mut a: Vec<f32> = v.iter().map(|x| x.abs()).collect();
            a.sort_by(f32::total_cmp);
            (a[a.len() / 2], a[a.len() - 1])
        };
        let (sp50, spmax) = resumo(&spin);
        let (rp50, rmax) = resumo(&rot);
        let vivos = spin.iter().filter(|x| x.abs() > 1e-4).count();
        eprintln!("  {k:>5} | {sp50:>8.3} | {spmax:>8.2} | {rp50:>9.2} | {rmax:>9.2} | {vivos:>7}");
    }
}

/// ⭐⭐⭐ **SONDA — QUEM leva o safanão, e QUANDO.** A [`probe_o_giro_que_sobra`] mostra que a pilha
/// assenta (o giro mediano cai) e que **uma** peça faz uma excursão tardia; esta nomeia-a.
///
/// ⚠️ **É a 5.ª vez que esta casa paga a mesma forma:** o `edge_max` global, o `χ` cego à almofada,
/// a `ENTREGA` cega à ponta que engrossou, as três réguas de ponta que deitavam fora o índice — e
/// agora o `rodopio`, que é um MÁXIMO sobre as peças e não diz sobre qual. *Uma régua que agrega
/// sobre uma população tem de saber devolver o membro.*
#[test]
#[ignore = "sonda de medicao"]
fn probe_quem_leva_o_safanao() {
    use ph2d_nodegraph::attr::Column as C;
    let sub = {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "uma contagem de sub-passos pequena"
        )]
        let s = super::SUBSTEPS as u32;
        s
    };
    let (mut state, sink) = super::obra::com_substeps(0.0, sub);
    let escopos = ph2d_nodegraph::cook::TimeScopes::new();
    let (mut rots, mut spins, mut ps) = (Vec::new(), Vec::new(), Vec::new());
    for k in 0..=174_u64 {
        state.pump.mark_dirty();
        state.pump.advance_or_scrub_to_nodes_scoped(
            &state.doc.graph,
            &state.registry,
            &[sink],
            k,
            |t| {
                #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
                let s = t as f64 / 60.0;
                s
            },
            &escopos,
        );
        let Some((_, saida)) = state
            .pump
            .boundary_streams()
            .iter()
            .find(|(no, _)| *no == sink)
        else {
            continue;
        };
        rots.push(match saida.get("rot") {
            Some(C::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        });
        spins.push(match saida.get("spin") {
            Some(C::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        });
        ps.push(match saida.get("P") {
            Some(C::Vec2(v)) => v.clone(),
            _ => Vec::new(),
        });
    }
    let n = rots.last().map_or(0, Vec::len);
    assert!(
        n >= 20,
        "piso de populacao: a cena tem de entregar peças ({n})"
    );
    // O giro LÍQUIDO de cada peça na janela assente, e o tique do pior passo dela.
    let mut fila: Vec<(f32, usize, u64, f32)> = (0..n)
        .map(|i| {
            let (mut liquido, mut pior, mut quando) = (0.0_f32, 0.0_f32, 0_u64);
            for k in 121..rots.len() {
                let (a, b) = (rots[k].get(i), rots[k - 1].get(i));
                let (Some(a), Some(b)) = (a, b) else { continue };
                liquido += a - b;
                if (a - b).abs() > pior {
                    pior = (a - b).abs();
                    quando = u64::try_from(k).unwrap_or_default();
                }
            }
            (liquido.abs(), i, quando, pior)
        })
        .collect();
    fila.sort_by(|a, b| b.0.total_cmp(&a.0));
    eprintln!("\n  as 5 peças de maior giro LÍQUIDO na janela assente (tiques 121..174):");
    eprintln!("  peça | liquido | pior passo | no tique | rot final | y final");
    eprintln!("  -----|---------|------------|----------|-----------|--------");
    for (liquido, i, quando, pior) in fila.iter().take(5) {
        let rot = rots.last().and_then(|v| v.get(*i)).copied().unwrap_or(0.0);
        let y = ps.last().and_then(|v| v.get(*i)).map_or(0.0, |q| q[1]);
        eprintln!("  {i:>4} | {liquido:>7.1} | {pior:>10.2} | {quando:>8} | {rot:>9.1} | {y:>7.2}");
    }
    // E o retrato da PIOR ao longo do tempo — o `spin` dela ao lado do `rot`.
    let Some(&(_, alvo, _, _)) = fila.first() else {
        return;
    };
    eprintln!("\n  a pior é a peça {alvo} — tique a tique (de 10 em 10):");
    eprintln!("  tique |    rot |   spin |      y | vizinhos a < 0,32");
    for k in (100..rots.len()).step_by(10) {
        let rot = rots[k].get(alvo).copied().unwrap_or(0.0);
        let spin = spins[k].get(alvo).copied().unwrap_or(0.0);
        let (y, vizinhos) = ps[k].get(alvo).map_or((0.0, 0), |q| {
            (
                q[1],
                ps[k]
                    .iter()
                    .enumerate()
                    .filter(|(j, o)| *j != alvo && (o[0] - q[0]).hypot(o[1] - q[1]) < 0.32)
                    .count(),
            )
        });
        eprintln!("  {k:>5} | {rot:>6.1} | {spin:>6.1} | {y:>6.2} | {vizinhos:>17}");
    }
}

/// ⭐⭐⭐ **SONDA — A PILHA CHEGA A PARAR?** A pergunta que separa *«a pilha zumbe»* de *«a pilha
/// ainda está a assentar»*, e que nenhuma régua desta linha sabia fazer.
///
/// ⛔⛔ **O `rodopio` e o `tremor` foram calibrados num produto em que a rotação NÃO PERSISTIA**, e
/// numa janela que acaba `1,9 s` depois de a pilha aterrar. Com velocidade angular, um giro líquido
/// naquela janela tanto pode ser *zumbido* (defeito) como *uma caixa a tombar para o sítio*
/// (física). ⇒ esta sonda alonga a `duration` da zona e imprime as réguas **janela a janela**: se
/// elas decaem para zero, a pilha assenta e a janela de hoje é que era cedo demais; se estabilizam
/// num patamar, há energia a entrar e é defeito.
///
/// ⚠️ **Ela mexe na `duration` da cena** — logo mede uma VARIANTE dela, e é de propósito: a `=114`
/// reinicia aos `3,0 s` e por construção nunca mostra uma pilha velha.
#[test]
#[ignore = "sonda de medicao"]
fn probe_a_pilha_chega_a_parar() {
    use ph2d_nodegraph::attr::Column as C;
    const ATE: u64 = 560;
    let sub = {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "uma contagem de sub-passos pequena"
        )]
        let s = super::SUBSTEPS as u32;
        s
    };
    let (mut state, sink) = super::obra::com_substeps(0.0, sub);
    // A zona deixa de reiniciar dentro da janela — ⚠️ as DUAS, que é a lição do `substeps`.
    let zonas: Vec<_> = state
        .doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == "sim.zone")
        .map(|n| n.id)
        .collect();
    assert!(!zonas.is_empty(), "a cena tem de ter zonas");
    for z in zonas {
        state.doc.graph.set_param(z, "duration", 12.0);
    }
    let escopos = ph2d_nodegraph::cook::TimeScopes::new();
    let mut rots: Vec<Vec<f32>> = Vec::new();
    let mut spins: Vec<Vec<f32>> = Vec::new();
    for k in 0..=ATE {
        state.pump.mark_dirty();
        state.pump.advance_or_scrub_to_nodes_scoped(
            &state.doc.graph,
            &state.registry,
            &[sink],
            k,
            |t| {
                #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
                let s = t as f64 / 60.0;
                s
            },
            &escopos,
        );
        let Some((_, saida)) = state
            .pump
            .boundary_streams()
            .iter()
            .find(|(no, _)| *no == sink)
        else {
            continue;
        };
        rots.push(match saida.get("rot") {
            Some(C::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        });
        spins.push(match saida.get("spin") {
            Some(C::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        });
    }
    let n = rots.last().map_or(0, Vec::len);
    assert!(
        n >= 20,
        "piso de populacao: a cena tem de entregar peças ({n})"
    );
    eprintln!("\n  janela (tiques) | rodopio | tremor | spin max | spin p50");
    eprintln!("  ----------------|---------|--------|----------|---------");
    let mut janela = 120_usize;
    while janela + 60 < rots.len() {
        let (de, ate) = (janela, janela + 60);
        let (mut liquido, mut passos) = (vec![0.0_f32; n], vec![Vec::new(); n]);
        for k in (de + 1)..ate {
            for i in 0..n {
                let (Some(a), Some(b)) = (rots[k].get(i), rots[k - 1].get(i)) else {
                    continue;
                };
                liquido[i] += a - b;
                passos[i].push((a - b).abs());
            }
        }
        let tremor = (0..n)
            .map(|i| super::tremor::mediana(&passos[i]))
            .fold(0.0_f32, f32::max);
        let rodopio = liquido.iter().fold(0.0_f32, |a, v| a.max(v.abs()));
        let fim = &spins[ate - 1];
        let mut abs: Vec<f32> = fim.iter().map(|x| x.abs()).collect();
        abs.sort_by(f32::total_cmp);
        let (p50, max) = if abs.is_empty() {
            (0.0, 0.0)
        } else {
            (abs[abs.len() / 2], abs[abs.len() - 1])
        };
        eprintln!("  {de:>7}..{ate:<7} | {rodopio:>7.2} | {tremor:>6.3} | {max:>8.2} | {p50:>8.3}");
        janela += 60;
    }
}

/// ⭐⭐⭐ **SONDA — O ARRASTO ANGULAR**, o knob que o `sim.step` já tem e que esta cena nunca usou.
///
/// A [`probe_a_pilha_chega_a_parar`] mostrou que com velocidade angular a pilha **assenta** e
/// demora `~2 s` a mais — e a `=114` reinicia aos `3,0 s`, logo o artista vê o transiente inteiro.
/// O que falta é DISSIPAÇÃO angular, e o `angular_damping` é a que já existe (`1` = nenhuma).
///
/// ⚠️ **Ela varre um PARÂMETRO, logo cabe numa corrida só** — ao contrário das [`Leis`], que são um
/// `const` e pedem mutação. *Medir o que é barato medir antes de construir o que é caro.*
#[test]
#[ignore = "sonda de medicao"]
fn probe_o_arrasto_angular() {
    use ph2d_nodegraph::attr::Column as C;
    let sub = {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "uma contagem de sub-passos pequena"
        )]
        let s = super::SUBSTEPS as u32;
        s
    };
    eprintln!("\n  ang.damping | rodopio | tremor | spin max | y final");
    eprintln!("  ------------|---------|--------|----------|--------");
    for arrasto in [1.0_f32, 0.95, 0.9, 0.8, 0.6, 0.3] {
        let (mut state, sink) = super::obra::com_substeps(0.0, sub);
        let passos: Vec<_> = state
            .doc
            .graph
            .nodes()
            .iter()
            .filter(|n| n.type_name == "sim.step")
            .map(|n| n.id)
            .collect();
        assert!(!passos.is_empty(), "a cena tem de ter um `sim.step`");
        for no in passos {
            state.doc.graph.set_param(no, "angular_damping", arrasto);
        }
        let escopos = ph2d_nodegraph::cook::TimeScopes::new();
        let (mut rots, mut spins, mut ys) = (Vec::new(), Vec::new(), Vec::new());
        for k in 0..=174_u64 {
            state.pump.mark_dirty();
            state.pump.advance_or_scrub_to_nodes_scoped(
                &state.doc.graph,
                &state.registry,
                &[sink],
                k,
                |t| {
                    #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
                    let s = t as f64 / 60.0;
                    s
                },
                &escopos,
            );
            let Some((_, saida)) = state
                .pump
                .boundary_streams()
                .iter()
                .find(|(no, _)| *no == sink)
            else {
                continue;
            };
            rots.push(match saida.get("rot") {
                Some(C::Scalar(v)) => v.clone(),
                _ => Vec::new(),
            });
            spins.push(match saida.get("spin") {
                Some(C::Scalar(v)) => v.clone(),
                _ => Vec::new(),
            });
            if let Some(C::Vec2(v)) = saida.get("P") {
                ys = v.iter().map(|q| q[1]).collect();
            }
        }
        let n = rots.last().map_or(0, Vec::len);
        assert!(
            n >= 20,
            "piso de populacao: a cena tem de entregar peças ({n})"
        );
        let (mut liquido, mut passos_rot) = (vec![0.0_f32; n], vec![Vec::new(); n]);
        for k in 121..rots.len() {
            for i in 0..n {
                let (Some(a), Some(b)) = (rots[k].get(i), rots[k - 1].get(i)) else {
                    continue;
                };
                liquido[i] += a - b;
                passos_rot[i].push((a - b).abs());
            }
        }
        let tremor = (0..n)
            .map(|i| super::tremor::mediana(&passos_rot[i]))
            .fold(0.0_f32, f32::max);
        let rodopio = liquido.iter().fold(0.0_f32, |a, v| a.max(v.abs()));
        let spin_max = spins
            .last()
            .map_or(0.0, |v| v.iter().fold(0.0_f32, |a, x| a.max(x.abs())));
        let y = super::tremor::mediana(&ys);
        eprintln!(
            "  {arrasto:>11.2} | {rodopio:>7.2} | {tremor:>6.3} | {spin_max:>8.2} | {y:>7.2}"
        );
    }
}

/// ⭐⭐⭐ **SONDA — O VALE DA DERIVA.** Quanto é que o `pior salto` muda quando o critério de
/// «a peça estava assentada» deixa de ser o pior passo por tique e passa a incluir a DERIVA.
///
/// ⚠️ **A barra sai daqui, e de um VALE com os dois lados** — a lei em vigor e a que se quer pôr.
/// Uma barra escolhida num lado só mede os defeitos desse lado (a lição que o pincel de tecido
/// pagou com duas barras retiradas).
#[test]
#[ignore = "sonda de medicao"]
fn probe_o_vale_da_deriva() {
    // O tecto de hoje — a varredura abaixo mostra o que acontece a toda a volta dele.
    const QUIETO: f32 = super::salto::QUIETO;
    let sub = {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "uma contagem de sub-passos pequena"
        )]
        let s = super::SUBSTEPS as u32;
        s
    };
    let realizacoes = [-0.003_f32, -0.0015, 0.0, 0.0015, 0.003];
    let todos: Vec<Vec<super::salto::Salto>> = realizacoes
        .iter()
        .map(|eps| super::salto::saltos(sub, *eps, super::salto::SALTO))
        .collect();
    for (k, t) in todos.iter().enumerate() {
        assert!(
            t.len() > 500,
            "piso de populacao: a realizacao {k} tem de produzir eventos ({})",
            t.len()
        );
    }
    eprintln!("\n  os 6 maiores «saltos» que hoje passam o filtro (realizacao central):");
    eprintln!("  peça | tique |  grau | antes | depois | deriva | desloca");
    eprintln!("  -----|-------|-------|-------|--------|--------|--------");
    for s in todos[2]
        .iter()
        .filter(|s| s.antes <= QUIETO && s.depois <= QUIETO)
        .take(6)
    {
        eprintln!(
            "  {:>4} | {:>5} | {:>5.2} | {:>5.3} | {:>6.3} | {:>6.2} | {:>7.2}",
            s.peca, s.tique, s.grau, s.antes, s.depois, s.deriva, s.desloca
        );
    }
    // ⚠️ O tecto de «assentada» é um ARGUMENTO do [`super::salto::pior_salto`] de propósito, e o
    // doc dele manda varrê-lo: *é a varredura que mostra que o achado não depende dele*.
    eprintln!("\n  a MEDIANA do pior salto sobre as 5 realizacoes — QUIETO x DERIVA:");
    eprintln!("  quieto \\ deriva |     inf |    1.00 |    0.50 |    0.25 |  eventos(inf)");
    eprintln!("  ----------------|---------|---------|---------|---------|--------------");
    for quieto in [0.2_f32, 0.1, 0.05, 0.02, 0.01] {
        let mut linha = String::new();
        for tecto in [f32::INFINITY, 1.0, 0.5, 0.25] {
            let mut piores: Vec<f32> = todos
                .iter()
                .map(|t| {
                    t.iter()
                        .find(|s| s.antes <= quieto && s.depois <= quieto && s.deriva <= tecto)
                        .map_or(0.0, |s| s.grau)
                })
                .collect();
            piores.sort_by(f32::total_cmp);
            let v = piores[piores.len() / 2];
            linha.push_str(&format!(" | {v:>7.2}"));
        }
        let quantos = todos[2]
            .iter()
            .filter(|s| s.antes <= quieto && s.depois <= quieto)
            .count();
        eprintln!("  {quieto:>15.3}{linha} | {quantos:>13}");
    }
}
