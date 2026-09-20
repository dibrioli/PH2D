//! **AS SONDAS DE MEDIÇÃO DA CORDA** — irmãs dos gates da `=120`.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE e foi imposto pelo tecto de LOC:** o pai afirma LEIS (uma
//! régua com barra, que reprova), e este ficheiro MEDE (imprime uma tabela e sai verde). As duas
//! metades mudam por razões diferentes — uma quando a lei muda, a outra quando alguém quer medir
//! outra coisa.
//!
//! ⛔ Todas são `#[ignore]`: elas correm-se à mão, e os relógios levam o `loadavg` ao lado
//! (`CLAUDE.md` §5.0).

use super::{CORDA, DT, primeiro};
use super::{MotionState, build, mede_a_corda, mede_o_osso};

/// ⭐⭐⭐ **A PEÇA CONTRA O VÃO** — a sonda que nomeia o que a varredura de 2026-09-20 achou
/// (ordem do dono, depois do smoke da unidade: *«veja se erro similar acontece em outros locais
/// do módulo»*).
///
/// ⛔⛔⛔ **O comprimento DESENHADO de uma peça de rig é um número da FORMA; o comprimento
/// VERDADEIRO é a coluna `len` da corrente — e o `len` não tem UM consumidor de desenho em toda a
/// casa.** Ele é escrito pelo `rig.bones` e pelo `source.lsystem`, e lido só pelo `fk::resolve`
/// (que reconstrói `P` com ele) e pelo próprio `rig.bones` (que pergunta se ele já lá está). Quem
/// decide o tamanho na tela é a coluna `size`, que vem da forma.
///
/// ⚠️ **Nas duas cenas isto bate porque o número foi DERIVADO à mão** (`CORDA_PECA` do vão da
/// corda, `OSSO_PECA` do `OSSO_LEN`), e é por isso que nenhum gate o via: eles leem `P`, `rot` e
/// `size`, e os três estão certos. O que nenhum lê é a RELAÇÃO entre `size` e o vão.
///
/// Medido (pela porta do produto, com a `TIQUES` de queda na corda):
///
/// | knob do painel | vão | desenhado | razão | o que se vê |
/// |---|---|---|---|---|
/// | `Count = 10`   | `0,21201` | `0,10000` | **`0,47×`** | um rosário, com buracos entre as contas |
/// | `Count = 20`   | `0,10252` | `0,10000` | `0,98×` | o cordão que o dono aprovou |
/// | `Count = 30`   | `0,06944` | `0,10000` | `1,44×` | as peças montam umas nas outras |
/// | `Count = 40`   | `0,05393` | `0,10000` | **`1,85×`** | uma barra contínua |
/// | `Length = 0,2` | `0,20000` | `0,45000` | **`2,25×`** | o mesmo, na fileira dos ossos |
/// | `Length = 0,45`| `0,45000` | `0,45000` | `1,00×` | a cadeia que ladrilha |
/// | `Length = 0,9` | `0,90000` | `0,45000` | **`0,50×`** | ossos soltos, um vão de cada dois vazio |
///
/// ⏳ **DECISÃO DO DONO** (as duas saídas, com o preço): (a) ficar como está — o artista escreve o
/// tamanho da peça a condizer com a corrente, e o painel não o ajuda; (b) uma peça de rig VESTIR o
/// osso dela (o `size` por elemento sai do `len`), que é o que faz a corda ler-se como um cordão
/// **em qualquer `Count`** e custa o `size` deixar de ser o que o artista escreveu na forma.
#[test]
#[ignore = "sonda de medicao, nao gate"]
fn diag_a_peca_contra_o_vao() {
    eprintln!("\n=== a peca DESENHADA contra o vao que ela atravessa ===");
    for count in [10.0f32, 20.0, 30.0, 40.0] {
        let (vao, desenhado, n) = mede_a_corda(Some(count));
        eprintln!(
            "  Count={count:>5}  pecas={n:>3}  vao={vao:.5}  desenhado={desenhado:.5}  \
             razao={:.2}x",
            desenhado / vao
        );
    }
    for length in [0.2f32, 0.45, 0.9] {
        let (vao, desenhado, n) = mede_o_osso(length);
        eprintln!(
            "  Length={length:>4}  pecas={n:>3}  vao={vao:.5}  desenhado={desenhado:.5}  \
             razao={:.2}x",
            desenhado / vao
        );
    }
}
/// ⭐⭐⭐ **A CORDA SEGMENTO A SEGMENTO** — a sonda do report do dono (2026-09-20: *«porque a corda
/// afina no final?»*).
///
/// ⚠️ **As réguas que existem leem o PRIMEIRO segmento** ([`peca_contra_vao`] devolve `pecas[0]`),
/// e é por isso que nenhuma delas podia ver isto: *uma régua que lê um elemento não vê um
/// GRADIENTE ao longo da lista*. Esta imprime o vão, os dois semi-eixos e a espessura de cada um.
#[test]
#[ignore = "sonda de medicao, nao gate"]
fn diag_a_corda_segmento_a_segmento() {
    eprintln!("\n=== quanto a corda AFINA da 1.a peca a' ULTIMA ===");
    // ⚠️⚠️ **A coluna da ESPESSURA é a que responde ao report, e a 1.ª redacção desta sonda não a
    // tinha.** Ela imprimia só `size[0]` — o COMPRIMENTO —, que **deve** seguir o vão: é isso que
    // faz a peça ir de uma junta à seguinte. Com a cura no sítio a tabela saiu IDÊNTICA, e a
    // leitura ingénua era *«a cura não fez nada»*. *Uma régua que lê um eixo não vê o outro.*
    eprintln!("\n  comprimento = 2·size.x (segue o vao, e DEVE) · espessura = 2·size.y\n");
    eprintln!("  count | tiques |  1.o vao | ultimo vao | compr. u/1 | ESPESS. u/1 | pior vizinho");
    eprintln!("  ------|--------|----------|------------|------------|-------------|-------------");
    for count in [10.0f32, 20.0, 40.0, 80.0] {
        for tiques in [1usize, 5, 20, 40, 120] {
            let mut m = MotionState::new();
            let sinks = build(&mut m.doc, &m.registry).expect("a cena monta");
            crate::motion_shape_gen::publish(&mut m, 0.0);
            let corda = primeiro(&m.doc.graph, "motion.verlet_rope");
            m.doc.graph.set_param(corda, "count", count);
            let sink = sinks[CORDA];
            let mut t = 0.0f64;
            for _ in 0..tiques {
                let _ = m.pump.cook.cook(&m.doc.graph, &m.registry, sink, t);
                let _ = m.pump.cook.advance_tick(&m.doc.graph, &m.registry, t);
                t += DT;
            }
            let saida = m
                .pump
                .cook
                .cook(&m.doc.graph, &m.registry, sink, t)
                .expect("o sink coze");
            let mut pecas = Vec::new();
            ph2d_eval_motion::lower_to_vector_instances_onto(
                saida[0].as_stream(),
                ph2d_render::SinkStyle::PLAIN,
                &mut pecas,
            );
            let Some((p, u)) = pecas.first().zip(pecas.last()) else {
                continue;
            };
            // ⚠️ **O pior par VIZINHO é a segunda coluna de propósito:** um gradiente suave ao
            // longo de vinte peças lê-se como *«a corda afina»*, e um DEGRAU entre duas vizinhas
            // lê-se como *«a corda tem um defeito»*. São leituras diferentes e a razão
            // ponta-a-ponta não as separa.
            let pior = pecas
                .windows(2)
                .map(|w| (w[1].size[0] / w[0].size[0]).min(w[0].size[0] / w[1].size[0]))
                .fold(1.0f32, f32::min);
            eprintln!(
                "  {count:>5.0} | {tiques:>6} | {:>8.5} | {:>10.5} | {:>9.3}x | {:>10.3}x | \
                 {pior:>10.3}x",
                2.0 * p.size[0],
                2.0 * u.size[0],
                u.size[0] / p.size[0],
                u.size[1] / p.size[1],
            );
        }
    }
    eprintln!();
}
/// ⭐⭐⭐ **O QUADRO DO PANO DA CORDA** — report do dono (2026-09-20: *«avalie performance de
/// rope»*).
///
/// ⚠️⚠️ **Pela porta do QUADRO (`pump`), nunca por `cook`:** o memo do cozedor é chaveado pelo
/// TIQUE, e medir com o tique parado lê o quadro anterior — a armadilha que o
/// [`crate::motion_custo_do_quadro_probe`] pagou primeiro (`0,002 ms` para 500 boids).
///
/// ⚠️ **E a `source.shape` lê um EXTERNAL que a SHELL publica** — sem o `publish` por quadro ela
/// emite zero linhas e a tabela mede o vazio. A população vai impressa ao lado do relógio, que é
/// o controlo que separa *«é rápido»* de *«não desenhou nada»*.
///
/// ⚠️ O `loadavg` vai no fim: nenhuma leitura de relógio desta máquina vale acima de `~5`
/// (`CLAUDE.md` §5.0).
#[test]
#[ignore = "sonda de medicao (relogio), nao gate"]
fn diag_o_quadro_da_corda() {
    eprintln!("\n=== o quadro do pano da CORDA (cena =120) ===\n");
    eprintln!("  count | pecas |    solver | pano inteiro | % de 16,67 |  ns/peca | FPS");
    eprintln!("  ------|-------|-----------|--------------|------------|----------|------");
    for count in [
        20.0f32, 80.0, 320.0, 1280.0, 2560.0, 5120.0, 10240.0, 20480.0,
    ] {
        // ⭐ **Duas marchas, duas SAÍDAS** — a do nó da corda sozinho (o solver e o estado dele) e
        // a do pano inteiro (mais o `rig.bones`, o `scale`, o `move`, o carimbo e o lower). *Medir
        // o segundo sozinho responde «quanto custa a corda» com o desenho dela lá dentro.*
        let mede = |so_o_solver: bool| -> (f64, usize) {
            let mut m = MotionState::new();
            let sinks = build(&mut m.doc, &m.registry).expect("a cena monta");
            let corda = primeiro(&m.doc.graph, "motion.verlet_rope");
            m.doc.graph.set_param(corda, "count", count);
            let saida = if so_o_solver { corda } else { sinks[CORDA] };
            let (uv, tam) = ([0.0, 0.0, 1.0, 1.0], [1.0, 1.0]);
            let mut tique = 0u64;
            let marcha = |m: &mut MotionState, t: &mut u64| {
                let ph = f64::from(u32::try_from(*t).unwrap_or(0)) / 60.0;
                crate::motion_shape_gen::publish(m, ph);
                m.pump.mark_dirty();
                let ok = m
                    .pump
                    .pump(&m.doc.graph, &m.registry, &[saida], *t, ph, uv, tam);
                *t += 1;
                ok
            };
            for _ in 0..12 {
                marcha(&mut m, &mut tique);
            }
            let mut melhor = f64::INFINITY;
            for _ in 0..16 {
                let t = std::time::Instant::now();
                assert!(marcha(&mut m, &mut tique), "o quadro tem de cozinhar");
                melhor = melhor.min(t.elapsed().as_secs_f64() * 1e3);
            }
            (
                melhor,
                m.pump.vector_instances.len() + m.pump.instances.len(),
            )
        };
        let (solver, _) = mede(true);
        let (pano, pecas) = mede(false);
        #[expect(clippy::cast_precision_loss, reason = "uma contagem de cena")]
        let por = pano * 1e6 / (pecas.max(1) as f64);
        eprintln!(
            "  {count:>5.0} | {pecas:>5} | {solver:>6.3} ms | {pano:>9.3} ms | {:>9.1}% | \
             {por:>6.0} ns | {:>5.0}",
            pano / 16.67 * 100.0,
            1000.0 / pano.max(1e-9)
        );
    }
    eprintln!(
        "\n  load: {}\n",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .split_whitespace()
            .next()
            .unwrap_or("?")
    );
}
/// ⭐⭐⭐ **QUAL ESTATÍSTICA DOS `len` NÃO RESPIRA?** — a segunda metade da decisão do report
/// *«porque a corda afina no final?»*.
///
/// Se a espessura passar a ser UMA para a cadeia inteira, ela tem de ser uma grandeza que **não
/// oscile por quadro** — senão o defeito muda de nome (a corda deixa de afinar e passa a
/// *respirar*). ⚠️ *A escolha entre mediana, média e mínimo é medição e não gosto*, e o critério é
/// a dispersão TEMPORAL de cada uma ao longo de uma corda que balança.
#[test]
#[ignore = "sonda de medicao, nao gate"]
fn diag_qual_estatistica_do_len_nao_respira() {
    let mut m = MotionState::new();
    let sinks = build(&mut m.doc, &m.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut m, 0.0);
    // ⚠️ Os `len` lêem-se na SAÍDA do `rig.bones` e não na corda: é essa a lista que a lei vê.
    let ossos = primeiro(&m.doc.graph, "rig.bones");
    let sink = sinks[CORDA];
    let mut t = 0.0f64;
    let (mut med, mut avg, mut min) = (Vec::new(), Vec::new(), Vec::new());
    for passo in 0..300 {
        let _ = m.pump.cook.cook(&m.doc.graph, &m.registry, sink, t);
        if let Ok(o) = m.pump.cook.cook(&m.doc.graph, &m.registry, ossos, t)
            && let Some(ph2d_nodegraph::attr::Column::Scalar(len)) = o[0].as_stream().get("len")
            && !len.is_empty()
            && passo >= 20
        {
            let mut ord = len.clone();
            ord.sort_by(f32::total_cmp);
            med.push(ord[ord.len() / 2]);
            min.push(ord[0]);
            #[expect(clippy::cast_precision_loss, reason = "uma contagem de segmentos")]
            avg.push(len.iter().sum::<f32>() / len.len() as f32);
        }
        let _ = m.pump.cook.advance_tick(&m.doc.graph, &m.registry, t);
        t += DT;
    }
    let disp = |v: &[f32]| -> (f32, f32, f32) {
        let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
        for x in v {
            lo = lo.min(*x);
            hi = hi.max(*x);
        }
        (lo, hi, hi / lo.max(f32::MIN_POSITIVE))
    };
    eprintln!("\n=== a espessura candidata, ao longo de 280 tiques de corda a balancar ===\n");
    eprintln!("  estatistica |      min |      max | oscilacao");
    eprintln!("  ------------|----------|----------|----------");
    for (nome, v) in [("mediana", &med), ("media", &avg), ("minimo", &min)] {
        let (lo, hi, r) = disp(v);
        eprintln!(
            "  {nome:<11} | {lo:>8.5} | {hi:>8.5} | {:>7.3}%",
            (r - 1.0) * 100.0
        );
    }
    eprintln!();
}
