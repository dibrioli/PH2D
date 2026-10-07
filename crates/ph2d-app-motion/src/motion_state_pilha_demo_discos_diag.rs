//! ⭐⭐ **OS DISCOS COM `Rolling` BAIXO: avalanche ou a trava a alternar?** (doc 121 §9.23) — a
//! pilha da `=114` em discos, por disco e por tique, em várias REALIZAÇÕES (a pilha é caótica: uma
//! queda só não mede — cinco esconderam a que desabava).

use ph2d_nodegraph::attr::Column as C;

/// O raio de um disco da `=114` (o `size` do cartão; a geometria nasce em raio `1`).
const R: f32 = super::LADO;
/// A janela da queda que o defeito mede.
const JANELA: (usize, usize) = (120, 180);

/// O estado da taça da direita num tique.
pub(super) struct Quadro {
    pub p: Vec<[f32; 2]>,
    pub rot: Vec<f32>,
}

/// ⭐ **A marcha da `=114`** com a `duration` da zona alongada e os params escritos nas formas —
/// a MESMA de [`super::giro_diag::perfil_com`] (que a consome).
pub(super) fn marcha(rolar: Option<f32>, ate: u64, extra: &[(&'static str, f32)]) -> Vec<Quadro> {
    marcha_eps(0.0, rolar, ate, extra)
}

/// [`marcha`] numa REALIZAÇÃO: a grelha deslocada `eps` (a pilha é caótica — uma queda só não mede).
pub(super) fn marcha_eps(
    eps: f32,
    rolar: Option<f32>,
    ate: u64,
    extra: &[(&'static str, f32)],
) -> Vec<Quadro> {
    let sub = {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "uma contagem de sub-passos pequena"
        )]
        let s = super::SUBSTEPS as u32;
        s
    };
    let (mut state, sink) = super::obra::com_substeps(eps, sub);
    let de_tipo = |state: &crate::motion_state::MotionState, t: &str| -> Vec<_> {
        state
            .doc
            .graph
            .nodes()
            .iter()
            .filter(|n| n.type_name == t)
            .map(|n| n.id)
            .collect()
    };
    // A zona deixa de reiniciar dentro da janela — ⚠️ as DUAS, que é a lição do `substeps`.
    let zonas = de_tipo(&state, "sim.zone");
    assert!(!zonas.is_empty(), "a cena tem de ter zonas");
    for z in zonas {
        state.doc.graph.set_param(z, "duration", 12.0);
    }
    if rolar.is_some() || !extra.is_empty() {
        let formas = de_tipo(&state, "source.shape");
        assert!(!formas.is_empty(), "a cena tem de ter formas");
        for f in formas {
            if let Some(r) = rolar {
                state
                    .doc
                    .graph
                    .set_param(f, ph2d_node_motion_shape::param::ROLLING, r);
            }
            for (nome, v) in extra {
                state.doc.graph.set_param(f, *nome, *v);
            }
        }
        crate::motion_shape_gen::publish(&mut state, 0.0);
    }
    let escopos = ph2d_nodegraph::cook::TimeScopes::new();
    let mut fora = Vec::new();
    for k in 0..=ate {
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
        fora.push(Quadro {
            p: match saida.get("P") {
                Some(C::Vec2(v)) => v.clone(),
                _ => Vec::new(),
            },
            rot: match saida.get("rot") {
                Some(C::Scalar(v)) => v.clone(),
                _ => Vec::new(),
            },
        });
    }
    fora
}

/// A inclinação do topo do monte: a recta pelo disco mais alto de cada coluna de largura `2R`,
/// de cada lado do centro da taça — `(graus à esquerda, graus à direita, o perfil)`.
fn inclinacao(p: &[[f32; 2]]) -> (f32, f32, String) {
    let mut topo: std::collections::BTreeMap<i32, f32> = std::collections::BTreeMap::new();
    for q in p {
        #[expect(clippy::cast_possible_truncation, reason = "uma coluna da grelha")]
        let c = ((q[0] - super::VAO) / (2.0 * R)).round() as i32;
        let y = topo.entry(c).or_insert(f32::MIN);
        *y = y.max(q[1]);
    }
    let recta = |lado: &[(f32, f32)]| -> f32 {
        if lado.len() < 2 {
            return 0.0;
        }
        #[expect(clippy::cast_precision_loss, reason = "uma contagem pequena")]
        let n = lado.len() as f32;
        let (mx, my) = (
            lado.iter().map(|x| x.0).sum::<f32>() / n,
            lado.iter().map(|x| x.1).sum::<f32>() / n,
        );
        let (sxy, sxx) = lado.iter().fold((0.0, 0.0), |(a, b), (x, y)| {
            (a + (x - mx) * (y - my), b + (x - mx) * (x - mx))
        });
        (sxy / sxx.max(1e-9)).atan().to_degrees()
    };
    #[expect(clippy::cast_precision_loss, reason = "uma coluna da grelha")]
    let pontos: Vec<(f32, f32)> = topo
        .iter()
        .map(|(c, y)| (*c as f32 * 2.0 * R, *y))
        .collect();
    let esq: Vec<_> = pontos.iter().copied().filter(|x| x.0 <= 0.0).collect();
    let dir: Vec<_> = pontos.iter().copied().filter(|x| x.0 >= 0.0).collect();
    let perfil = pontos
        .iter()
        .map(|(x, y)| format!("{x:+.2}:{y:.2}"))
        .collect::<Vec<_>>()
        .join(" ");
    (recta(&esq), recta(&dir), perfil)
}

/// ⭐ **As réguas do mecanismo** na janela da queda, impressas.
fn imprime_mecanismo(titulo: &str, q: &[Quadro]) {
    imprime_mecanismo_em(titulo, q, JANELA);
}

/// [`imprime_mecanismo`] numa janela `(de, ate)` qualquer.
fn imprime_mecanismo_em(titulo: &str, q: &[Quadro], (de, ate): (usize, usize)) {
    let n = q[ate].rot.len();
    let mut linhas = Vec::new();
    let (mut no_lugar, mut rolado) = (0.0_f32, 0.0_f32);
    let (mut trocas_tot, mut presos_tot) = (0_u32, 0_u32);
    for i in 0..n {
        let (mut liquido, mut giro_r, mut caminho, mut trocas, mut presos) =
            (0.0_f32, 0.0_f32, 0.0_f32, 0_u32, 0_u32);
        for k in (de + 1)..=ate {
            let dth = (q[k].rot[i] - q[k - 1].rot[i]).to_radians();
            let (a, b) = (q[k].p[i], q[k - 1].p[i]);
            let ds = (a[0] - b[0]).hypot(a[1] - b[1]);
            liquido += dth;
            giro_r += dth.abs() * R;
            caminho += ds;
            rolado += dth.abs() * R;
            if ds < 0.25 * dth.abs() * R {
                no_lugar += dth.abs() * R;
            }
            // ⚠️ Trancado = o passo NÃO rodou a peça (o `rot` sai igual ao bit). O `spin == 0` à
            // saída era CEGO: quem solta, roda um passo e volta a trancar sai com `spin` `0`.
            let (agora, antes) = (
                q[k].rot[i].to_bits() == q[k - 1].rot[i].to_bits(),
                q[k - 1].rot[i].to_bits() == q[k - 2].rot[i].to_bits(),
            );
            trocas += u32::from(agora != antes);
            presos += u32::from(agora);
        }
        trocas_tot += trocas;
        presos_tot += presos;
        let (a, b) = (q[ate].p[i], q[de].p[i]);
        let desloc = (a[0] - b[0]).hypot(a[1] - b[1]);
        linhas.push((
            liquido.to_degrees(),
            giro_r,
            caminho,
            desloc,
            trocas,
            presos,
        ));
    }
    linhas.sort_by(|a, b| b.0.abs().total_cmp(&a.0.abs()));
    #[expect(clippy::cast_precision_loss, reason = "contagens pequenas")]
    let (por_s, frac) = (
        trocas_tot as f32 / n as f32 / ((ate - de) as f32 / 60.0),
        presos_tot as f32 / (n * (ate - de)) as f32,
    );
    eprintln!(
        "\n  {titulo} — {n} discos; trocas trancar/destrancar {por_s:.1} por disco por s; \
         tiques trancados {:.0} %; giro NO LUGAR (|ΔP| < ¼·|Δθ|R) {:.0} % do Σ|Δθ|R",
        frac * 100.0,
        100.0 * no_lugar / rolado.max(1e-9)
    );
    eprintln!(
        "    giro líq (°) | Σ|Δθ|·R | caminho Σ|ΔP| | deslocado | caminho/(Σ|Δθ|R) | trocas | trancado"
    );
    for (liq, gr, cam, des, tr, pr) in linhas.iter().take(5) {
        eprintln!(
            "    {liq:>12.1} | {gr:>7.3} | {cam:>13.3} | {des:>9.3} | {:>16.2} | {tr:>6} | {pr:>8}",
            cam / gr.max(1e-9)
        );
    }
    for k in [de, (de + ate) / 2, ate] {
        let (e, d, perfil) = inclinacao(&q[k].p);
        eprintln!("    tique {k}: topo à esquerda {e:+.1}° · à direita {d:+.1}° | {perfil}");
    }
}

/// O perfil `(início, rodopio, tremor)` de uma marcha, nas janelas de `60` a partir do `120` — a
/// conta de [`super::giro_diag::perfil_com`].
pub(super) fn perfil_de(q: &[Quadro]) -> Vec<(usize, f32, f32)> {
    let rots: Vec<&Vec<f32>> = q.iter().map(|x| &x.rot).collect();
    let n = rots.last().map_or(0, |v| v.len());
    assert!(
        n >= 20,
        "piso de populacao: a cena tem de entregar peças ({n})"
    );
    let mut fora = Vec::new();
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
        fora.push((de, rodopio, tremor));
        janela += 60;
    }
    fora
}

/// O resumo de uma marcha depois do `240` (a pilha assente): o maior rodopio e o maior deslize por
/// janela, e as trocas trancado/livre por disco por segundo.
fn assente(q: &[Quadro]) -> (f32, f32, f32) {
    let (mut rod, mut des) = (0.0_f32, 0.0_f32);
    let n = q.last().map_or(0, |x| x.p.len());
    for de in (240..=420).step_by(60) {
        for i in 0..n {
            let (a, b) = (q[de + 60].p[i], q[de].p[i]);
            des = des.max((a[0] - b[0]).hypot(a[1] - b[1]));
            if !q[de].rot.is_empty() {
                rod = rod.max((q[de + 60].rot[i] - q[de].rot[i]).abs());
            }
        }
    }
    let mut trocas = 0_u32;
    if !q[240].rot.is_empty() {
        for i in 0..n {
            for k in 242..=480 {
                let livre = |k: usize| q[k].rot[i].to_bits() != q[k - 1].rot[i].to_bits();
                trocas += u32::from(livre(k) != livre(k - 1));
            }
        }
    }
    #[expect(clippy::cast_precision_loss, reason = "contagens pequenas")]
    let por_s = trocas as f32 / n.max(1) as f32 / 4.0;
    (rod, des, por_s)
}

/// As realizações dos gates e das sondas: a grelha deslocada em `±0,003` u, onze passos.
pub(super) fn realizacoes() -> impl Iterator<Item = f32> {
    (-5..=5_i8).map(|k| f32::from(k) * 0.0006)
}

/// Por `Rolling`, sobre as [`realizacoes`]: a queda (o rodopio de `120..180`, mediana · pior) e,
/// depois do `240`, o pior rodopio, o pior deslize e as trocas trancado/livre por peça por segundo.
pub(super) fn rodada(rolar: f32, extra: &[(&'static str, f32)]) -> (f32, f32, f32, f32, f32) {
    let (mut queda, mut rod, mut des, mut tr) = (Vec::new(), 0.0_f32, 0.0_f32, 0.0_f32);
    for e in realizacoes() {
        let q = marcha_eps(e, Some(rolar), 490, extra);
        if !q[120].rot.is_empty() {
            queda.push(perfil_de(&q)[0].1);
        }
        let (r, d, t) = assente(&q);
        (rod, des, tr) = (rod.max(r), des.max(d), tr.max(t));
    }
    queda.sort_by(f32::total_cmp);
    let (med, pior) = queda
        .get(queda.len() / 2)
        .zip(queda.last())
        .map_or((0.0, 0.0), |(a, b)| (*a, *b));
    (med, pior, rod, des, tr)
}

/// ⭐⭐ **SONDA — avalanche ou trava** (doc 121 §9.23): as réguas do mecanismo nos discos (`Rolling
/// 0 · 0,1 · 0,25`, a realização central) e a tabela das onze realizações, caixas e discos.
///
/// ```text
/// cargo test -p ph2d-app-motion --release probe_avalanche_ou_trava -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_avalanche_ou_trava() {
    use ph2d_node_motion_shape::param;
    let discos = [(param::COLLIDER_SHAPE, 1.0)];
    for rolar in [0.0_f32, 0.1, 0.25] {
        let q = marcha(Some(rolar), 200, &discos);
        imprime_mecanismo(&format!("discos · Rolling {rolar}"), &q);
    }
    eprintln!("\n  célula | queda med · pior | assente: rodopio · deslize · trocas/s");
    let referencia = [(param::COLLIDER_SHAPE, 1.0), (param::LOCK_ROTATION, 1.0)];
    for (nome, rolar, extra) in [
        ("discos Rolling 0", 0.0_f32, &discos[..]),
        ("discos Lock Rotation", 0.0, &referencia[..]),
    ] {
        let (m, p, r, d, t) = rodada(rolar, extra);
        eprintln!("  {nome} | {m:.1} · {p:.1} | {r:.1} · {d:.3} · {t:.2}");
    }
    for (forma, extra) in [("caixas", &[][..]), ("discos", &discos[..])] {
        for rolar in [0.05_f32, 0.1, 0.15, 0.25, 0.75] {
            let (m, p, r, d, t) = rodada(rolar, extra);
            eprintln!("  {forma} {rolar} | {m:.1} · {p:.1} | {r:.1} · {d:.3} · {t:.2}");
        }
    }
}

/// ⭐ **SONDA — o mecanismo por `Rolling`** (doc 121 §9.23): a realização MEDIANA da queda dos
/// discos, as réguas do mecanismo na queda e na primeira janela assente.
#[test]
#[ignore = "sonda de medicao"]
fn probe_o_mecanismo_por_rolamento() {
    let discos = [(ph2d_node_motion_shape::param::COLLIDER_SHAPE, 1.0)];
    for rolar in [0.1_f32, 0.15, 0.2] {
        let mut quedas: Vec<(f32, f32)> = realizacoes()
            .map(|e| (perfil_de(&marcha_eps(e, Some(rolar), 200, &discos))[0].1, e))
            .collect();
        quedas.sort_by(|a, b| a.0.total_cmp(&b.0));
        let (r, e) = quedas[quedas.len() / 2];
        let q = marcha_eps(e, Some(rolar), 310, &discos);
        imprime_mecanismo(&format!("discos {rolar} · eps {e} (mediana {r:.1}°)"), &q);
        imprime_mecanismo_em(
            &format!("discos {rolar} · eps {e} · 240..300"),
            &q,
            (240, 300),
        );
    }
}
