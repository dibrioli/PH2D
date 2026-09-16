//! ⭐⭐ **AS SONDAS DE DIAGNÓSTICO DO SALTO** — as que nomeiam a CAUSA, e não as que a medem.
//!
//! Irmã da [`super::salto`] pelo tecto de LOC (HR-18) e por ASSUNTO: ali moram a régua e o GATE
//! (*«quanto saltou uma peça que tinha assentado?»*); aqui moram as três que responderam *«e
//! PORQUÊ?»* — o dossiê do evento, o censo dos apoios face-a-face ao longo do tempo, e o arrasto.
//!
//! ⚠️ **Elas ficam porque cada uma matou uma hipótese**, e a próxima janela que suspeitar do mesmo
//! tem de as poder correr em vez de as reconstruir (doc 111 §5.10 e §5.12).

use super::salto::{ATE, SALTO, marcha, pior_salto, pior_salto_com};
use super::*;
use ph2d_nodegraph::attr::Column;

/// **SONDA — o DOSSIÊ do pior salto: o que a peça estava a fazer nos tiques à volta.**
///
/// ⭐ Ela discrimina as duas hipóteses abertas do doc 111 §5.9.5 **sem tocar no produto**:
/// - se a peça está **encostada à taça** quando salta ⇒ a suspeita dos DOIS solvers a disputá-la;
/// - se o **vão às vizinhas encolhe** tique a tique e depois abre de repente ⇒ a suspeita da
///   correcção que se acumula sem nada a travar.
///
/// ⚠️ A taça guarda por DENTRO: uma peça toca a parede quando `|p − centro| > raio − r` (o
/// `inner` do [`ph2d_node_sim_collide`]). Com o colisor da caixa, `inner ≈ 1,8 − 0,11 = 1,69`.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_o_dossie_do_salto -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_o_dossie_do_salto() {
    /// O tecto de `|Δrot|` por tique que ainda conta como «a peça tinha assentado».
    const QUIETO: f32 = 0.2;
    let (sub, eps) = (8_u32, 0.0_f32);
    let Some(alvo) = pior_salto(sub, eps, QUIETO, SALTO) else {
        eprintln!("nenhum salto legitimo — nada a dissecar");
        return;
    };
    let (mut state, sink) = super::obra::com_substeps(eps, sub);
    let serie = marcha(&mut state, sink, ATE);
    let n = serie.first().map_or(0, |(r, _)| r.len());
    let i = alvo.peca;
    #[expect(clippy::cast_possible_truncation, reason = "um indice de tique")]
    let t0 = alvo.tique as usize;
    eprintln!(
        "\n=== peca {i} · salto de {:.2}° no tique {t0} (substeps {sub}) ===",
        alvo.grau
    );
    eprintln!(
        "  tique |    rot | |Δrot| | dist a' taca | encostada? |    vao min | viz. | desalinho"
    );
    eprintln!(
        "  ------|--------|-------|--------------|------------|------------|------|----------"
    );
    // O centro da taça da DIREITA, e o raio interior que a peça de facto vê.
    let centro = [VAO, TACA_Y];
    let inner = TACA_R - LADO;
    for t in t0.saturating_sub(20)..(t0 + SALTO + 10).min(serie.len()) {
        let (rot, p) = (&serie[t].0, &serie[t].1);
        let drot = if t == 0 {
            0.0
        } else {
            (rot[i] - serie[t - 1].0[i]).abs()
        };
        let dist = (p[i][0] - centro[0]).hypot(p[i][1] - centro[1]);
        let vao = (0..n)
            .filter(|j| *j != i)
            .map(|j| (p[i][0] - p[j][0]).hypot(p[i][1] - p[j][1]))
            .fold(f32::INFINITY, f32::min);
        let marca = if dist > inner { "SIM" } else { "-" };
        // ⭐ E QUEM e' a vizinha mais proxima, com o desalinho dos eixos modulo 90° — a pergunta
        // que decide se o apoio e' FACE-COM-FACE (0°) ou QUINA-contra-face (45°).
        let viz = (0..n)
            .filter(|j| *j != i)
            .min_by(|a, b| {
                let da = (p[i][0] - p[*a][0]).hypot(p[i][1] - p[*a][1]);
                let db = (p[i][0] - p[*b][0]).hypot(p[i][1] - p[*b][1]);
                da.total_cmp(&db)
            })
            .unwrap_or(i);
        let d = (rot[i] - rot[viz]).abs() % 90.0;
        let desalinho = if d > 45.0 { 90.0 - d } else { d };
        eprintln!(
            "  {t:>5} | {:>6.2}° | {drot:>5.2}° | {dist:>12.4} | {marca:>10} | {vao:>12.4} | {viz:>4} | {desalinho:>9.1}°",
            rot[i]
        );
    }
    eprintln!(
        "  ⚠️ a taca guarda por dentro: encostada = dist > {inner:.4} (raio {TACA_R} − {LADO})"
    );
    eprintln!(
        "  ⚠️ duas caixas encostadas face a face tem vao {:.4}; o tipico da cena e' 0,2390",
        2.0 * LADO
    );
}

/// **O STREAM final de uma marcha pelo PUMP** — para quem precisa das COLUNAS (colisores,
/// materiais) e não só de `rot`/`P`.
///
/// ⛔⛔ Ela existe porque a sonda do manifesto de dois pontos (doc 109 §8.7) chamava
/// `pump.cook.cook(..)` **directamente** e por isso media a cena **sem sub-passos** — a mesma porta
/// errada que invalidou uma tabela inteira no §5.8.1 do doc 111. *Uma conclusão medida pela porta
/// errada não é uma conclusão.*
pub(super) fn stream_final(
    substeps: u32,
    eps: f32,
    ate: u64,
) -> Option<ph2d_nodegraph::attr::Stream> {
    let (mut state, sink) = super::obra::com_substeps(eps, substeps);
    let escopos = ph2d_nodegraph::cook::TimeScopes::new();
    let mut fora = None;
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
        if let Some((_, saida)) = state
            .pump
            .boundary_streams()
            .iter()
            .find(|(no, _)| *no == sink)
        {
            fora = Some(saida.clone());
        }
    }
    fora
}

/// **SONDA — quantos apoios FACE-A-FACE existem ao longo do tempo, e não só no fim.**
///
/// ⛔⛔⛔ **A suspeita que esta sonda testa é sobre a RÉGUA, não sobre o produto:** o censo do doc
/// 109 §8.7 conta os contactos do monte **ASSENTE** — isto é, *depois* de cada apoio face-a-face já
/// ter tombado para os `45°`. Ele conta **SOBREVIVENTES**, e leu `0 %` como *«esta cena não tem o
/// caso»* quando o que ele podia estar a medir é *«esta cena DESTRÓI o caso»*.
///
/// ⚠️ *Um censo tirado depois do evento mede o resultado do defeito e lê-se como a ausência da
/// precondição dele.* Se a contagem ao longo do tempo for muito maior que a do fim, a recusa que
/// matou o manifesto de dois pontos estava a medir a própria consequência do que recusava.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_os_apoios_face_a_face_no_tempo -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_os_apoios_face_a_face_no_tempo() {
    for sub in [1_u32, 8] {
        let (mut state, sink) = super::obra::com_substeps(0.0, sub);
        let escopos = ph2d_nodegraph::cook::TimeScopes::new();
        let (mut pico, mut soma, mut amostras, mut ultimo) = (0_usize, 0_usize, 0_usize, 0_usize);
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
            // Só a metade assente interessa: antes de `60` a pilha ainda está a cair.
            if k < 60 {
                continue;
            }
            let Some((_, s)) = state
                .pump
                .boundary_streams()
                .iter()
                .find(|(no, _)| *no == sink)
            else {
                continue;
            };
            let (Some(cols), Some(Column::Vec2(p)), Some(Column::Scalar(rot))) = (
                ph2d_contact::colisores(s),
                s.get("P").cloned(),
                s.get("rot").cloned(),
            ) else {
                continue;
            };
            let mut flush = 0;
            for i in 0..p.len() {
                for j in (i + 1)..p.len() {
                    let (Some(a), Some(b)) = (cols[i], cols[j]) else {
                        continue;
                    };
                    if ph2d_contact::contato(&a, p[i], &b, p[j], (i + j) % 2 == 0).is_none() {
                        continue;
                    }
                    let d = (rot[i] - rot[j]).abs() % 90.0;
                    if if d > 45.0 { 90.0 - d } else { d } < 5.0 {
                        flush += 1;
                    }
                }
            }
            pico = pico.max(flush);
            soma += flush;
            amostras += 1;
            ultimo = flush;
        }
        #[expect(clippy::cast_precision_loss, reason = "contagens pequenas")]
        let media = soma as f32 / amostras.max(1) as f32;
        eprintln!(
            "\n  substeps {sub}: apoios face-a-face — PICO {pico} · media {media:.1} · NO FIM {ultimo}"
        );
    }
    eprintln!("\n  ⚠️ o censo do doc 109 §8.7 le' a coluna «NO FIM». Se o PICO for muito maior,");
    eprintln!("     ele mede os SOBREVIVENTES de um defeito e le'-se como a ausencia do caso.");
}

/// **SONDA — o ARRASTO, que a cena nunca usou** (doc 111 §5.12).
///
/// ⭐⭐⭐ Enquanto a resposta de velocidade matava a velocidade **ABSOLUTA** de cada peça, o monte
/// assentava por **sobre-amortecimento** e ninguém reparou que a cena não declarava travão nenhum
/// (`damping` do `sim.step` no default `1,0` = SEM arrasto). Com o impulso do par a física ficou
/// certa — e o momento passou a conservar-se, que é o que faz o monte continuar a mexer-se.
///
/// ⚠️ *A lei do `CLAUDE.md` §5.0: antes de construir, MEÇA se a composição já o exprime.* O arrasto
/// existe, é um controlo do artista, e nunca foi ligado.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_o_arrasto_da_cena -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_o_arrasto_da_cena() {
    use ph2d_nodegraph::attr::Column as C;
    eprintln!("\n  substeps = 8 · damping 1,0 = SEM arrasto");
    eprintln!("  damping |    balanço    |    giro      |    salto");
    eprintln!("  --------|---------------|--------------|-------------");
    for d in [1.0_f32, 0.98, 0.97, 0.95] {
        // ⚠️ **CINCO realizações por célula** (doc 109 §8.8): 25 quadrados a cair são caóticos, e uma
        // célula de uma realização só não distingue cura de sorteio.
        let (mut bal, mut gir, mut slt) = (Vec::new(), Vec::new(), Vec::new());
        for eps in [-0.003_f32, -0.0015, 0.0, 0.0015, 0.003] {
            let (mut state, sink) = super::obra::com_substeps_e_arrasto(eps, 8, Some(d));
            let serie = marcha(&mut state, sink, 174);
            // ⛔⛔ **O `n` sai do ÚLTIMO tique, nunca do primeiro** — no tique `0` a pilha ainda não
            // nasceu e `rot` vem VAZIO. A 1.ª redacção lia `n = 0`, os dois laços não corriam, e a
            // sonda imprimia `0,000` em todas as células: *um censo que mede nada lê-se como perfeito.*
            let n = serie.last().map_or(0, |(r, _)| r.len());
            assert!(
                n >= 20,
                "piso de populacao: a metade da direita tem 25 pecas, leu {n}"
            );
            // ⛔⛔ **A janela conta-se a partir do FIM, nunca por `skip(120)`.** A [`marcha`] só guarda
            // o tique em que as DUAS colunas existem, e a `rot` só nasce quando há contacto — a série
            // tem MENOS entradas que tiques, e um `skip` fixo caía para lá do fim, devolvendo uma
            // fatia VAZIA cuja mediana é `0`. *A sonda imprimia `0,000` em todas as células, que é a
            // mesma assinatura de «medi e está perfeito».*
            let janela: Vec<_> = serie.windows(2).collect();
            assert!(
                janela.len() >= 54,
                "piso: a janela assente tem de ter 0,9 s ({} janelas)",
                janela.len()
            );
            let assente = &janela[janela.len() - 54..];
            let (mut balanco, mut giro) = (0.0_f32, 0.0_f32);
            for i in 0..n {
                let passos: Vec<f32> = assente
                    .iter()
                    .map(|w| (w[1].0[i] - w[0].0[i]).abs())
                    .collect();
                balanco = balanco.max(super::tremor::mediana(&passos));
                let liq: f32 = assente.iter().map(|w| w[1].0[i] - w[0].0[i]).sum();
                giro = giro.max(liq.abs());
            }
            let (mut ys, mut vao) = (Vec::new(), 0.0);
            if let Some((_, p)) = serie.last() {
                ys = p.iter().map(|q| q[1]).collect();
                vao = super::tests::vizinho_mediano(p);
            }
            ys.sort_by(f32::total_cmp);
            let y = ys.get(ys.len() / 2).copied().unwrap_or(0.0);
            // ⚠️ E o salto TAMBEM tem de correr com o arrasto — a 1.ª redacção chamava o
            // `pior_salto` sem ele e imprimia `1,65°` nas cinco linhas, que é a mesma assinatura.
            let sl = pior_salto_com(8, eps, 0.2, SALTO, Some(d)).map_or(0.0, |s| s.grau);
            let _ = (C::Scalar(vec![]), y, vao);
            bal.push(balanco);
            gir.push(giro);
            slt.push(sl);
        }
        let faixa = |v: &mut Vec<f32>| {
            v.sort_by(f32::total_cmp);
            (v[0], v[v.len() - 1])
        };
        let (b0, b1) = faixa(&mut bal);
        let (g0, g1) = faixa(&mut gir);
        let (s0, s1) = faixa(&mut slt);
        eprintln!(
            "  {d:>7.2} | {b0:>5.3}..{b1:<5.3} | {g0:>5.1}..{g1:<5.1} | {s0:>4.2}..{s1:<4.2}°"
        );
    }
    eprintln!("\n  ⚠️ barra dos gates: balanço ≤ 1,0 °/tique · giro ≤ 29° · salto ≤ 2,0°");
}

/// **SONDA — AUDITORIA DO ATRITO NA CENA** (7.º report do dono: *«o atrito ficou estranhamente
/// reduzido mesmo no máximo como se estivesse desligado»*).
///
/// ⚠️ As bancadas do `sim.step` dizem que o atrito peça×peça está CERTO (uma caixa a deslizar trava
/// a `μ·g`, e numa torre o peso propaga). Esta mede o que o DONO de facto faz: mexer o **Friction**
/// do cartão e olhar o monte.
///
/// A grandeza é o que o atrito de facto governa: **quanto as peças ainda deslizam** na janela
/// assente, e **quão espalhado** o monte acaba.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_auditoria_do_atrito_na_cena -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_auditoria_do_atrito_na_cena() {
    eprintln!("\n  substeps = 8 · o Friction do cartao da forma, de 0 a 1");
    eprintln!("  atrito |      desliza / tique       |    largura do monte");
    eprintln!("  -------|----------------------------|---------------------");
    for f in [0.0_f32, 0.1, 0.25, 0.5, 0.75, 1.0] {
        // ⚠️ CINCO realizações por célula (doc 109 §8.8) — sem elas esta curva é um sorteio.
        let (mut ds, mut ls) = (Vec::new(), Vec::new());
        for eps in [-0.003_f32, -0.0015, 0.0, 0.0015, 0.003] {
            let (mut state, sink) = super::obra::com_substeps_arrasto_atrito(eps, 8, None, Some(f));
            let serie = marcha(&mut state, sink, 174);
            let n = serie.last().map_or(0, |(r, _)| r.len());
            assert!(n >= 20, "piso de populacao: leu {n}");
            let janela: Vec<_> = serie.windows(2).collect();
            assert!(janela.len() >= 54, "piso: {} janelas", janela.len());
            let assente = &janela[janela.len() - 54..];
            // Quanto cada peça ainda ESCORREGA por tique (em fracção do lado dela), mediano.
            let mut desliza = 0.0_f32;
            let mut giro = 0.0_f32;
            for i in 0..n {
                let d: Vec<f32> = assente
                    .iter()
                    .map(|w| {
                        (w[1].1[i][0] - w[0].1[i][0]).hypot(w[1].1[i][1] - w[0].1[i][1])
                            / super::LADO
                    })
                    .collect();
                desliza = desliza.max(super::tremor::mediana(&d));
                let liq: f32 = assente.iter().map(|w| w[1].0[i] - w[0].0[i]).sum();
                giro = giro.max(liq.abs());
            }
            let largura = serie.last().map_or(0.0, |(_, p)| {
                let (lo, hi) = p.iter().fold((f32::MAX, f32::MIN), |(lo, hi), q| {
                    (lo.min(q[0]), hi.max(q[0]))
                });
                hi - lo
            });
            let _ = giro;
            ds.push(desliza);
            ls.push(largura);
        }
        let faixa = |v: &mut Vec<f32>| {
            v.sort_by(f32::total_cmp);
            (v[0], v[v.len() / 2], v[v.len() - 1])
        };
        let (d0, dm, d1) = faixa(&mut ds);
        let (l0, lm, l1) = faixa(&mut ls);
        eprintln!(
            "  {f:>6.2} | {d0:>6.4}..{d1:<6.4} (p50 {dm:.4}) | {l0:>5.2}..{l1:<5.2} (p50 {lm:.2})"
        );
    }
    eprintln!("\n  ⚠️ se as quatro linhas forem iguais, o botao NAO CHEGA ao motor.");
}
