//! **O TETO DE CONTAGEM de um param se MEDE** (doc 88 A1 · CLAUDE.md §0).
//!
//! O slider dual separa duas perguntas que estavam colapsadas num número só: o **soft** (`ParamUiHint.max`)
//! é a faixa confortável que o arrasto cobre, e o **hard** (`ParamHardMax`) é **onde o disfuncional
//! começa** — o número que a caixa de texto ainda aceita. O §0 é explícito sobre o segundo: *antes de
//! escrever qualquer teto, MEÇA — e escreva o número que a medição deu, com a tabela ao lado dele.*
//!
//! ⚠️ **E o spread de hoje é o próprio sintoma que o §0 descreve:** `motion.voronoi` traz **165.000**
//! (um número que a linha da GPU mediu) enquanto `motion.grid` traz **20** por lado e `motion.clone`
//! **32** — não porque alguém mediu a grade e a achou 8.000× mais cara que o voronoi, mas porque
//! ninguém mediu. *O caminho mais lento definiu o teto do mais rápido.*
//!
//! Esta sonda existe para trocar esses palpites por números. Ela cozinha o nó pela porta REAL (`Cook`
//! sobre o registry do produto) numa varredura de contagens e reporta o custo por cook — **e a tabela
//! que ela imprime é o que tem de aparecer ao lado de cada teto que esta wave escrever**.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins measure_the_count_ceiling -- --ignored --nocapture
//! ```

#[cfg(test)]
mod tests {
    use crate::motion_state::MotionState;
    use ph2d_nodegraph::cook::Cook;
    use ph2d_nodegraph::effect::Effect;
    use ph2d_nodegraph::graph::{Edge, Graph};

    /// Um cook do nó `type_name` com o param de contagem em `count`, ligado a um `motion.output`.
    /// Devolve `(ms do cook MÍNIMO de cinco, instâncias que saíram)` — ou `None` se o nó não cozinha
    /// sozinho (precisa de entrada que esta sonda não monta).
    ///
    /// ⚠️ **Um `Cook` NOVO por amostra, e a 1ª corrida é a que conta** — o oposto do redutor que
    /// um laço quente pede, e a 1ª versão desta sonda errou exatamente aqui: o `Cook` **MEMOIZA**,
    /// então descartar a 1ª corrida (a cautela de *first-touch* que o pen-up do Painter ensinou)
    /// deixava a mediana medir **quatro acertos de memo**. A tabela saía `0,00 ms` em toda célula
    /// enquanto o teste levava **1402 s** — *números que não reconciliam com o relógio de parede
    /// são a assinatura desta doença*, e é o relógio que denuncia, não a tabela.
    ///
    /// ⚠️ **`source` alimenta a porta 0 com uma grade de N instâncias**, para os nós que
    /// TRANSFORMAM um stream em vez de o criarem (`motion.clone`, `motion.kaleidoscope`,
    /// `motion.pin_constraint`). Nesses o eixo medido **MULTIPLICA** a fonte, então a coluna
    /// `inst` é o que vale ler: um teto sobre `count` ali é um teto sobre **CÓPIAS**, não
    /// sobre instâncias — quem clona um stream de 10 mil paga 10 mil por cópia.
    ///
    /// ⚠️ **Um nó COM ESTADO precisa da realimentação no GRAFO, e a fixture não a tinha.**
    /// Boids, verlet e wave declaram uma porta `state` que o editor **auto-liga**
    /// (`out --pre--> state`) quando o artista os coloca; `Graph::add_node` não faz isso.
    /// Sem a aresta o `state` chega vazio, o `eval` cai no braço de **SEMEADURA** e nunca
    /// dá um passo — medido, boids a 8.000 saía **0,026 ms**, ou 3,2 ns por agente, com uma
    /// busca de vizinhança na cara do laço. *Aquecer no TEMPO não bastava: o que faltava era
    /// no GRAFO.* E os dois fatos — ter estado, ser temporal — são **DERIVADOS do manifesto
    /// do próprio nó**, nunca de uma tabela paralela que a próxima simulação nasce fora
    /// (o padrão do `ph2d-motion-diagnose`).
    fn cook_cost(
        type_name: &str,
        count_param: &str,
        count: f32,
        fixed: &[(&str, f32)],
        source: Option<f32>,
    ) -> Option<(f64, usize)> {
        let st = MotionState::new();
        let mut g = Graph::new();
        let node = g.add_node(type_name);
        let out = g.add_node("motion.output");
        let manifest = st.registry.manifests().find(|m| m.name == type_name)?;
        let temporal = matches!(manifest.effect, Effect::Temporal);
        if let Some(port) = manifest.inputs.iter().position(|p| p.name == "state") {
            g.connect(Edge {
                from: (node, 0),
                to: (node, port as u16),
                delayed: true,
            })
            .ok()?;
        }
        if let Some(rows) = source {
            let src = g.add_node("motion.grid");
            g.set_param(src, "rows", rows);
            g.set_param(src, "cols", 1.0);
            g.connect(Edge {
                from: (src, 0),
                to: (node, 0),
                delayed: false,
            })
            .ok()?;
        }
        // ⚠️ Os params FIXOS existem para que o eixo medido seja o pedido: a grade multiplica
        // `rows` por `cols`, então sem `cols = 1` a coluna "1000" da tabela mede 3000 instâncias
        // e o número deixa de ser sobre o param que a linha nomeia.
        for (k, v) in fixed {
            g.set_param(node, *k, *v);
        }
        g.set_param(node, count_param, count);
        g.connect(Edge {
            from: (node, 0),
            to: (out, 0),
            delayed: false,
        })
        .ok()?;
        g.validate(&st.registry).ok()?;
        // ⚠️ **O redutor é o MÍNIMO, e a escolha tem mecanismo:** as cinco amostras fazem
        // trabalho IDÊNTICO (um `Cook` novo cada), então a única fonte de espalhamento é
        // contenção — e uma máquina disputada só sabe deixar mais LENTO. Medido nesta
        // sessão: o MESMO binário deu `motion.voronoi` a 5,49 ms com a máquina calma e
        // **100,86 ms com seis `rustc` de outras linhas vivos** (18×). A mediana não
        // defende disso; o mínimo, sim. *(O §5.12 do Painter alerta que o mínimo é o
        // redutor ERRADO quando uma amostra é estruturalmente diferente — aqui não é.)*
        let mut ms = Vec::new();
        let mut n = 0usize;
        for _ in 0..5 {
            let mut cook = Cook::new(); // memo VAZIO: este cook faz o trabalho inteiro.
            if temporal {
                // ⚠️ **`advance_tick`, não `cook`** — é ele que PUBLICA `prev_outputs`, e uma
                // aresta delayed lê exatamente isso. Um segundo `cook` num tempo diferente
                // não avança tick nenhum: o `state` continua chegando vazio e o nó segue
                // semeando, com a tabela reportando o custo da semeadura como se fosse o passo.
                cook.advance_tick(&g, &st.registry, 0.0).ok()?;
            }
            let t = if temporal { 1.0 / 60.0 } else { 0.5 };
            let t0 = std::time::Instant::now();
            let set = cook.cook(&g, &st.registry, out, t).ok()?;
            ms.push(t0.elapsed().as_secs_f64() * 1000.0);
            n = set.iter().next().map_or(0, |s| s.as_stream().count());
        }
        ms.sort_by(f64::total_cmp);
        Some((ms[0], n))
    }

    /// **A TABELA** — o custo de um cook contra a contagem pedida, por nó de contagem.
    ///
    /// O que ler nela: a coluna `inst` diz quantas instâncias de fato saíram (um nó pode recusar,
    /// clampar ou multiplicar o número pedido), e a coluna `ms` diz o que um quadro paga. O
    /// orçamento de referência é **16,6 ms** (um quadro de 60 fps) — o teto HARD honesto de um param
    /// de contagem é a última potência que fica confortavelmente abaixo dele, e é ESSE número que
    /// deve ser escrito no `ParamHardMax`, com esta tabela ao lado.
    #[test]
    #[ignore = "sonda de medição, não gate"]
    fn measure_the_count_ceiling() {
        // (nó, param de contagem, soft max de hoje) — os nós cuja contagem carrega um RECURSO.
        // ⚠️ `motion.path` fica FORA: cozinha sozinho e devolve **0 instâncias** (a curva dele vem
        // de um text param que esta sonda não semeia) — medir zero seria medir a fixture.
        // ⚠️ **Cada nó leva a PRÓPRIA varredura**, e não é conveniência: varrer o `scatter` até
        // 400k custou **208 SEGUNDOS** num número que ninguém vai usar, e varrer a grade só até
        // 10k esconderia que ela é barata. A faixa de cada linha é a faixa em que a resposta
        // daquele nó de fato muda.
        /// Um sujeito: `(nó, param, fixos, faixa, fonte)`.
        ///
        /// ⚠️ **O soft NÃO está aqui** — ele é lido do registry na hora de imprimir. A versão
        /// anterior o carregava como literal por sujeito, e DOIS envelheceram sem ninguém ver
        /// (o `distribute_curve` anunciava `soft=2000` sobre um slider de **320**, o
        /// `distribute_radial` o mesmo sobre **600**): uma sonda de TETO imprimindo o teto
        /// errado. Um número copiado à mão para um readout é um número que vai mentir.
        type Subject = (
            &'static str,
            &'static str,
            &'static [(&'static str, f32)],
            &'static [f32],
            Option<f32>,
        );
        const DEEP: &[f32] = &[100_000.0, 400_000.0, 1_000_000.0];
        let subjects: &[Subject] = &[
            // Os LINEARES e baratos — a varredura vai fundo porque eles aguentam.
            ("motion.grid", "rows", &[("cols", 1.0)], DEEP, None),
            ("motion.fibonacci", "count", &[], DEEP, None),
            (
                "motion.distribute_radial",
                "count",
                &[("rings", 1.0)],
                DEEP,
                None,
            ),
            ("motion.distribute_curve", "count", &[], DEEP, None),
            // Os DEFORMADORES de grade — `rows × cols`, então `cols = 1` mantém o eixo medido
            // sendo o pedido (a mesma cautela da grade).
            ("motion.lattice", "rows", &[("cols", 1.0)], DEEP, None),
            // ⚠️ O `motion.wave` é `Effect::Temporal` — um cook num playhead fixo mede UM passo,
            // que é o que um quadro paga; a faixa é menor porque ele carrega estado por célula.
            (
                "motion.wave",
                "rows",
                &[("cols", 2.0)],
                &[10_000.0, 100_000.0, 400_000.0],
                None,
            ),
            // As SIMULAÇÕES — cada uma com o próprio mecanismo, e por isso a própria faixa.
            // Boids faz vizinhança; o verlet é Gauss-Seidel por aresta (sequencial por semântica).
            (
                "motion.boids",
                "count",
                &[],
                &[500.0, 2_000.0, 8_000.0],
                None,
            ),
            (
                "motion.verlet_rope",
                "count",
                &[],
                &[10_000.0, 100_000.0, 400_000.0],
                None,
            ),
            // O QUADRÁTICO — best-candidate blue noise (Mitchell): `nearest_sq` varre todos os
            // pontos já postos, logo O(count² × CANDIDATES). A varredura cerca o quadro de 60 fps.
            (
                "motion.scatter",
                "count",
                &[],
                &[2_000.0, 3_000.0, 4_000.0, 6_000.0],
                None,
            ),
            // Os MULTIPLICADORES — leem um stream e o repetem. A fonte é declarada (100
            // instâncias), então `inst` = count × 100 e o teto é sobre CÓPIAS.
            (
                "motion.clone",
                "count",
                &[],
                &[100.0, 1_000.0, 10_000.0],
                Some(100.0),
            ),
            (
                "motion.kaleidoscope",
                "segments",
                &[],
                &[100.0, 1_000.0, 10_000.0],
                Some(100.0),
            ),
            // O que INDEXA a fonte: os pins apontam para linhas do stream de entrada, então a
            // fonte tem de ser grande o bastante para o eixo medido não saturar nela.
            (
                "motion.pin_constraint",
                "count",
                &[],
                &[4_096.0, 40_000.0, 200_000.0],
                Some(200_000.0),
            ),
            // O da GPU — o cook de CPU aqui é o caminho de REFERÊNCIA, não o que manda no teto
            // (§0). A varredura fica pequena de propósito: é para NOMEAR a distância, não para
            // derivar um cap dela.
            (
                "motion.voronoi",
                "count",
                &[],
                &[1_000.0, 4_000.0, 8_000.0],
                None,
            ),
        ];
        // ⚠️ **A tabela carrega o próprio detector.** Esta worktree divide a máquina com as
        // outras linhas, e sob carga os números desta sonda não falam sobre o código —
        // eles falam sobre quantos `rustc` estavam vivos. Com a carga impressa ao lado,
        // uma tabela colada num doc meses depois ainda diz se podia ser acreditada.
        let load = std::fs::read_to_string("/proc/loadavg")
            .ok()
            .and_then(|s| s.split_whitespace().next().map(str::to_owned))
            .unwrap_or_else(|| "?".into());
        println!("\n=== custo de UM cook (ms) x contagem pedida — orçamento 16,6 ms/quadro ===");
        println!("    (load average {load} — acima de ~5 a tabela mede a MÁQUINA, não o código)");
        let reg = &MotionState::new().registry;
        let soft_of = |ty: &str, param: &str| -> f32 {
            reg.manifests()
                .find(|m| m.name == ty)
                .and_then(|m| reg.param_ui(m.id))
                .and_then(|hints| hints.iter().find(|h| h.param == param))
                .map_or(f32::NAN, |h| h.max)
        };
        for (ty, param, fixed, counts, source) in subjects {
            let soft = soft_of(ty, param);
            let mut row = format!("{ty:26} soft={soft:<9.0} ");
            let mut first = None;
            let mut last = None;
            for &c in counts.iter() {
                match cook_cost(ty, param, c, fixed, *source) {
                    Some((ms, n)) => {
                        row.push_str(&format!("{c:.0}:{ms:.3}ms/{n} "));
                        first.get_or_insert(ms);
                        last = Some(ms);
                    }
                    None => row.push_str(&format!("{c:.0}:-- ")),
                }
            }
            // ⚠️ O CONTROLE. Um custo que NÃO sobe com a contagem significa que a sonda não está
            // medindo o cook — foi exatamente assim que a 1ª versão saiu `0,00 ms` em toda célula
            // sobre um teste de 1402 s. A tabela só vale se esta razão for maior que 1.
            if let (Some(a), Some(b)) = (first, last) {
                row.push_str(&format!("| x{:.0}", if a > 0.0 { b / a } else { 0.0 }));
            }
            println!("{row}");
        }
    }
}
