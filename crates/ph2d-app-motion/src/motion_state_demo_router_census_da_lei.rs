//! **AS SONDAS DA LEI DA APARÊNCIA E DO GIZMO** — irmãs por RESPONSABILIDADE (HR-18) das do
//! [`super::census`], de onde saíram quando o ficheiro passou o tecto de LOC.
//!
//! ⚠️ **O corte é o ASSUNTO e não o tamanho:** o irmão mede o que as cenas AUTORAM (a disposição,
//! o movimento, as portas de realimentação, o que o roteador monta); estas medem o que a ordem do
//! dono de 2026-09-19 abriu — *quem desenha sem forma*, o que chega ao sink, o que a tomada do
//! gizmo custa, e se o `gap_y` ainda espaça.
//!
//! ⚠️ Um censo **não é um gate**: ele devolve o NÚMERO com que se escolhe a lei seguinte.
//! Corra-os com `-- --ignored --nocapture`.

use super::tests::shape_origin;
use super::*;

/// **A coluna `size` da corrente, como multiplicador por elemento** — `None` quando ela não a traz.
///
/// ⚠️ Estas leituras viviam no gizmo de posições, que a ordem do dono de 2026-09-19 RETIROU
/// (*«retire tudo relacionado a gizmos desses nós»*). Elas ficam aqui porque o que as sondas abaixo
/// medem é a **CORRENTE** — um facto sobre o que as cenas autoram —, e isso sobrevive ao desenho.
///
/// ⚠️ Uma coluna `Vec2` colapsa na MÉDIA dos eixos: as sondas perguntam *«que números o `size`
/// tem»*, e um par não cabe num histograma de um eixo.
fn escalas_da_corrente(s: &ph2d_nodegraph::attr::Stream) -> Option<Vec<f32>> {
    use ph2d_nodegraph::attr::Column;
    match s.get("size") {
        Some(Column::Scalar(v)) => Some(v.to_vec()),
        Some(Column::Vec2(v)) => Some(v.iter().map(|e| (e[0] + e[1]) * 0.5).collect()),
        _ => None,
    }
}

/// **A coluna `rot` da corrente, em graus** — `None` quando ela não a traz. Irmã da acima.
fn rotacoes_da_corrente(s: &ph2d_nodegraph::attr::Stream) -> Option<Vec<f32>> {
    use ph2d_nodegraph::attr::Column;
    match s.get("rot") {
        Some(Column::Scalar(v)) => Some(v.to_vec()),
        _ => None,
    }
}

/// ⭐⭐⭐ **QUEM DESENHA PIXELS SEM NUNCA TER RECEBIDO UMA FORMA** — a sonda que a ordem do dono
/// de 2026-09-19 obriga a correr ANTES de qualquer cura (§5.0, e o [doc 115] §14.2 por escrito).
///
/// > *«Não deveriam renderizar nada na tela, mas deveriam apenas disponibilizarem a posição e
/// > direção […] e deveriam ser dependentes de Duplicator e Shape (e demais objetos) para
/// > aparecer na tela.»*
///
/// ⚠️⚠️ **O NÚMERO desta sonda decidiu uma wave e depois mudou de significado.** Ela mediu
/// `111` de `123` cenas a desenhar sem forma, e esse número foi a razão de a lei shipar
/// DESLIGADA. Em 2026-09-19 o dono reabriu o report (*«o grid continua desenhando quadrados»*) e
/// deu a saída na mesma mensagem (*«coloque apenas pontos nas posições»*): a lei passou a
/// desenhar uma MARCA em vez de nada, e o custo que a segurava desapareceu. ⇒ *o que esta sonda
/// conta hoje não é «quantas cenas seriam apagadas», é «quantas cenas mostram um ARRANJO», que é
/// uma resposta sobre o catálogo e não sobre o risco.*
///
/// ⚠️ **A pergunta é por SINK e não por nó**, porque quem gera pixels é o lowering: um
/// `rig.skeleton` ligado direito ao `motion.output` desenha um quadrado por junta, com o
/// `default_uv_rect` da shell — e o cabeçalho do próprio nó diz que isso é o desenho
/// (*«a bare skeleton already renders: its joints are elements like any other»*).
///
/// ⚠️ **Os produtores de aparência são DOIS e o censo deriva-os**, nunca os escreve à mão: são
/// as únicas crates de nó que escrevem `uv_rect`/`texture_id`/`geometry_id` como fonte
/// (`source.object` e `source.shape` — ⚠️ a crate chama-se `ph2d-node-motion-shape` e o NÓ
/// chama-se `source.shape`, e a 1.ª redacção desta sonda leu `0 de 123` por causa disso);
/// `duplicator`/`mixer`/`morph`/`trail` **propagam** o que
/// receberam, logo não contam como origem.
///
/// `cargo test -p ph2d-app-motion --lib -- --ignored --nocapture quem_desenha_sem_forma`
#[test]
#[ignore = "sonda, nao um gate"]
fn quem_desenha_sem_forma() {
    /// As origens de aparência. ⚠️ Derivadas da varredura das crates de nó (as únicas que
    /// escrevem uma coluna de aparência sem a receber de uma entrada).
    const ORIGENS: [&str; 2] = ["source.object", "source.shape"];

    let (mut com, mut sem) = (Vec::new(), Vec::new());
    let mut sem_por_fonte: std::collections::BTreeMap<String, Vec<u32>> =
        std::collections::BTreeMap::new();
    for level in 1..=MAX_DEMO_LEVEL {
        let mut state = MotionState::new();
        let sinks =
            crate::motion_demo_legend::monta(&level.to_string(), &mut state.doc, &state.registry).0;
        if sinks.is_empty() {
            continue;
        }
        let g = &state.doc.graph;
        let nome = |id: ph2d_nodegraph::graph::NodeId| {
            g.nodes()
                .iter()
                .find(|n| n.id == id)
                .map(|n| n.type_name.clone())
                .unwrap_or_default()
        };
        // Sobe a montante de cada sink. Uma aresta `delayed` também carrega dados (é o tique
        // anterior), logo entra: um laço de simulação não deixa de receber a forma por isso.
        let mut vistos: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
        let mut pilha: Vec<_> = sinks.clone();
        let mut origens_achadas: std::collections::BTreeSet<String> =
            std::collections::BTreeSet::new();
        let mut folhas: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        while let Some(id) = pilha.pop() {
            if !vistos.insert(id.0) {
                continue;
            }
            let t = nome(id);
            if ORIGENS.contains(&t.as_str()) {
                origens_achadas.insert(t.clone());
            }
            let mut tem_entrada = false;
            for e in g.edges().iter().filter(|e| e.to.0 == id) {
                tem_entrada = true;
                pilha.push(e.from.0);
            }
            if !tem_entrada && !t.is_empty() {
                folhas.insert(t);
            }
        }
        if origens_achadas.is_empty() {
            sem.push(level);
            for f in &folhas {
                let e = sem_por_fonte.entry(f.clone()).or_default();
                if !e.contains(&level) {
                    e.push(level);
                }
            }
        } else {
            com.push(level);
        }
    }
    let total = com.len() + sem.len();
    eprintln!("\n=== QUEM DESENHA SEM FORMA · {total} cenas com sink, de 1..={MAX_DEMO_LEVEL} ===");
    eprintln!(
        "  recebem aparencia de `source.object`/`motion.shape` │ {:>3} ({:>4.1}%)",
        com.len(),
        com.len() as f64 * 100.0 / total as f64
    );
    eprintln!(
        "  desenham SO' POSICOES (ficariam em branco)          │ {:>3} ({:>4.1}%)",
        sem.len(),
        sem.len() as f64 * 100.0 / total as f64
    );
    eprintln!("\n  as cenas so'-posicoes: {sem:?}\n");
    eprintln!("  quem as ALIMENTA (no' de raiz -> cenas):\n");
    for (fonte, cenas) in &sem_por_fonte {
        eprintln!("  {fonte:<26} em {:>3} cena(s): {cenas:?}", cenas.len());
    }
    eprintln!();
}

/// ⭐⭐ **AS COLUNAS QUE CHEGAM AO SINK** — o CONTROLO da [`quem_desenha_sem_forma`], que responde
/// pelo grafo. Esta coze e pergunta ao stream, que é o que o lowering de facto lê.
///
/// ⚠️ **Ela existe porque as duas perguntas podem discordar**: um grafo pode atravessar um
/// `source.shape` e a coluna não chegar ao sink (um nó a jusante que a deixe cair), e um grafo sem
/// origem nenhuma pode ter `uv_rect` por outra via. *O discriminador da cura tem de ser o que o
/// lowering vê, não o que o grafo promete.*
///
/// ⛔⛔ **ARMADILHA MEDIDA: as cenas de `source.shape` cozem a ZERO neste arnês.** A geometria
/// delas é publicada pelo `motion_shape_gen`, que corre no QUADRO (precisa do `shape_store`), e um
/// arnês sem shell não o corre ⇒ a `=110`, a `=114`, a `=115` e a `=119` leem `0 linhas` aqui e
/// desenham dezenas de peças no app. *Esta sonda é o controlo do que APARECE nas colunas, nunca um
/// censo de população* — para «quem tem origem de aparência» a resposta é a [`quem_desenha_sem_forma`],
/// que pergunta ao GRAFO e não precisa de shell.
///
/// `cargo test -p ph2d-app-motion --lib -- --ignored --nocapture colunas_que_chegam_ao_sink`
#[test]
#[ignore = "sonda, nao um gate"]
fn colunas_que_chegam_ao_sink() {
    // As cenas de CICLO (as que o dono segue) mais a `=110`, que é a do carimbo.
    let alvo: [u32; 12] = [110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121];
    eprintln!("\n=== COLUNAS NO SINK · as cenas que o dono segue ===\n");
    for level in alvo {
        let mut state = MotionState::new();
        let sinks =
            crate::motion_demo_legend::monta(&level.to_string(), &mut state.doc, &state.registry).0;
        if sinks.is_empty() {
            eprintln!("  =[{level}] (sem sink)");
            continue;
        }
        for (k, &sink) in sinks.iter().enumerate() {
            let Ok(out) = state
                .pump
                .cook
                .cook(&state.doc.graph, &state.registry, sink, 0.0)
            else {
                eprintln!("  =[{level}] sink {k}: (nao coze)");
                continue;
            };
            let s = out[0].as_stream();
            let tem = |n: &str| if s.get(n).is_some() { "sim" } else { " - " };
            let geo_vivo = match s.get("geometry_id") {
                Some(ph2d_nodegraph::attr::Column::Scalar(v)) => v.iter().any(|&x| x > 0.5),
                _ => false,
            };
            eprintln!(
                "  =[{level}] sink {k}: {:>7} linhas │ uv_rect {} │ texture_id {} │ geometry_id {} (vivo: {}) │ APARENCIA: {}",
                s.count(),
                tem("uv_rect"),
                tem("texture_id"),
                tem("geometry_id"),
                if geo_vivo { "sim" } else { "nao" },
                if s.get("uv_rect").is_some() || geo_vivo {
                    "SIM"
                } else {
                    "NAO — isto desenha MARCAS (a lei do dono, desde 2026-09-19)"
                }
            );
        }
    }
    eprintln!();
}

/// ⭐⭐⭐ **QUEM É «COMO O GRID»** — a população da ordem do dono de 2026-09-19 (*«para nós como Grid
/// e outros similares vamos criar uma seção para tamanho absoluto do gizmo…»*), **DERIVADA do
/// manifesto** e nunca de uma lista escrita à mão.
///
/// A regra: um nó é FONTE DE POSIÇÕES quando **não recebe** uma corrente de instâncias e **emite**
/// uma. É esse o nó que, sem um Duplicator, não tem como virar pixel — e é a ele que a secção do
/// gizmo pertence.
///
/// `cargo test -p ph2d-app-motion --lib -- --ignored --nocapture quem_e_como_o_grid`
#[test]
#[ignore = "sonda, nao um gate"]
fn quem_e_como_o_grid() {
    use ph2d_nodegraph::port::Domain;
    let reg = MotionState::new().registry;
    let inst = |t: &ph2d_nodegraph::port::PortType| t.domain == Domain::Instances;
    let mut fontes: Vec<&str> = Vec::new();
    let mut passagens = 0usize;
    for m in reg.manifests() {
        let emite = m.outputs.iter().any(|p| inst(&p.ty));
        let recebe = m.inputs.iter().any(|p| inst(&p.ty));
        if emite && !recebe {
            fontes.push(m.name);
        } else if emite {
            passagens += 1;
        }
    }
    fontes.sort_unstable();
    eprintln!(
        "\n=== FONTES DE POSICOES · {} de {} nos que emitem instancias ===\n",
        fontes.len(),
        fontes.len() + passagens
    );
    for f in &fontes {
        eprintln!("  {f}");
    }
    eprintln!("\n  ({passagens} sao de PASSAGEM: recebem instancias e devolvem-nas)\n");
}

/// ⭐⭐⭐ **UMA COLUNA NOVA SOBREVIVE À CADEIA?** — a premissa que decide a arquitectura da secção
/// do gizmo (ordem do dono, 2026-09-19). Se uma coluna escrita pela FONTE não chega ao sink, então
/// a secção **não pode** viver no nó de origem e tem de viver no sink.
///
/// ⚠️ Ela é medida com uma coluna INVENTADA (`zz_sonda`) posta num external, atravessando a cadeia
/// que as cenas de facto usam.
///
/// `cargo test -p ph2d-app-motion --lib -- --ignored --nocapture uma_coluna_nova_sobrevive`
#[test]
#[ignore = "sonda, nao um gate"]
fn uma_coluna_nova_sobrevive_a_cadeia() {
    use ph2d_nodegraph::attr::Column;
    use ph2d_nodegraph::graph::{Edge, Graph};
    let cadeias: [&[&str]; 6] = [
        &["motion.move"],
        &["motion.scale"],
        &["motion.rotate"],
        &["motion.move", "motion.scale", "motion.rotate"],
        &["motion.clone"],
        &["motion.cull"],
    ];
    eprintln!("\n=== UMA COLUNA NOVA SOBREVIVE A' CADEIA? ===\n");
    for cadeia in cadeias {
        let mut m = MotionState::new();
        let mut g = Graph::new();
        // A fonte é um `source.object` porque ele lê um EXTERNAL, que é onde a sonda põe a coluna.
        let fonte = g.add_node("source.object");
        g.set_text_param(fonte, "object", "Sonda");
        let mut cur = fonte;
        for t in cadeia {
            let n = g.add_node(*t);
            g.connect(Edge {
                from: (cur, 0),
                to: (n, 0),
                delayed: false,
            })
            .expect("liga");
            cur = n;
        }
        let saida = g.add_node("motion.output");
        g.connect(Edge {
            from: (cur, 0),
            to: (saida, 0),
            delayed: false,
        })
        .expect("liga");
        m.doc.graph = g;
        let com_sonda = crate::motion_bridge::appearance_tile(
            [1.0, 1.0],
            [1.0, 1.0, 1.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
            0,
            false,
        )
        .with("zz_sonda", Column::Scalar(vec![7.0]));
        m.pump.cook.set_external("Sonda".to_string(), com_sonda);
        let Ok(out) = m.pump.cook.cook(&m.doc.graph, &m.registry, saida, 0.0) else {
            eprintln!("  {cadeia:?} │ NAO COZE");
            continue;
        };
        let s = out[0].as_stream();
        let chegou =
            matches!(s.get("zz_sonda"), Some(Column::Scalar(v)) if v.first() == Some(&7.0));
        eprintln!(
            "  {:<52} │ {} linhas │ a coluna {}",
            format!("{cadeia:?}"),
            s.count(),
            if chegou { "CHEGOU" } else { "SUMIU" }
        );
    }
    eprintln!();
}

/// ⛔⛔ **O QUE A TOMADA DO GIZMO CUSTA NUMA CENA DE DISPOSITIVO** — o número que decide se o gizmo
/// é utilizável nas cenas grandes (ordem do dono, 2026-09-19).
///
/// Na rota do device a bomba **não marcha**: uma tomada obriga o `cook_taps_only` a cozinhar aquele
/// sink **na CPU**, e é esse o preço por quadro que o gizmo cobra quando a lei está ligada.
///
/// `cargo test -p ph2d-app-motion --release --lib -- --ignored --nocapture o_que_a_tomada_custa`
#[test]
#[ignore = "sonda de relogio, nao um gate"]
fn o_que_a_tomada_do_gizmo_custa() {
    eprintln!("\n=== O PRECO DA TOMADA DO GIZMO (cozimento de CPU do sink) ===\n");
    for level in [111u32, 116, 117, 120] {
        let mut state = MotionState::new();
        let sinks =
            crate::motion_demo_legend::monta(&level.to_string(), &mut state.doc, &state.registry).0;
        let Some(&sink) = sinks.first() else { continue };
        // Aquece (o memo do cook) e depois mede a mediana de cinco.
        let _ = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sink, 0.0);
        let mut ms: Vec<f64> = (0..5)
            .map(|k| {
                let t = f64::from(k) / 60.0;
                let i = std::time::Instant::now();
                let _ = state
                    .pump
                    .cook
                    .cook(&state.doc.graph, &state.registry, sink, t);
                i.elapsed().as_secs_f64() * 1e3
            })
            .collect();
        ms.sort_by(f64::total_cmp);
        let n = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sink, 0.0)
            .map_or(0, |o| o[0].as_stream().count());
        eprintln!(
            "  ={level:<4} │ {n:>7} linhas │ {:>8.3} ms │ {:>6.1} % de um quadro de 16,7",
            ms[2],
            ms[2] * 100.0 / 16.67
        );
    }
    eprintln!();
}

/// ⛔⛔⛔ **O `gap_y` DA GRELHA AINDA ESPAÇA?** — report do dono, 2026-09-19: *«Gap y quebrou e
/// movimenta tudo em vez de criar espaço»*, depois de a secção do gizmo ter entrado no manifesto
/// daquele nó.
///
/// A régua é a **extensão** da nuvem contra o **centro** dela: espaçar cresce a extensão e deixa o
/// centro quieto; mover desloca o centro e deixa a extensão quieta.
///
/// `cargo test -p ph2d-app-motion --lib -- --ignored --nocapture o_gap_y_ainda_espaca`
#[test]
#[ignore = "sonda, nao um gate"]
fn o_gap_y_ainda_espaca() {
    use ph2d_nodegraph::attr::Column;
    use ph2d_nodegraph::graph::{Edge, Graph};
    eprintln!("\n=== O `gap_y` DA GRELHA ===\n");
    eprintln!("  gap_y │ extensao Y │  centro Y  │ extensao X │  centro X");
    for gy in [0.5f32, 1.0, 2.0, 4.0] {
        let mut m = MotionState::new();
        let mut g = Graph::new();
        let grelha = g.add_node("motion.grid");
        g.set_param(grelha, "rows", 4.0);
        g.set_param(grelha, "cols", 4.0);
        g.set_param(grelha, "gap_x", 1.0);
        g.set_param(grelha, "gap_y", gy);
        let saida = g.add_node("motion.output");
        g.connect(Edge {
            from: (grelha, 0),
            to: (saida, 0),
            delayed: false,
        })
        .expect("liga");
        m.doc.graph = g;
        let Ok(out) = m.pump.cook.cook(&m.doc.graph, &m.registry, saida, 0.0) else {
            eprintln!("  {gy} │ NAO COZE");
            continue;
        };
        let s = out[0].as_stream();
        let Some(Column::Vec2(p)) = s.get("P") else {
            continue;
        };
        let (mut lox, mut hix, mut loy, mut hiy) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
        for q in p {
            lox = lox.min(q[0]);
            hix = hix.max(q[0]);
            loy = loy.min(q[1]);
            hiy = hiy.max(q[1]);
        }
        eprintln!(
            "  {gy:>5} │ {:>10.3} │ {:>10.3} │ {:>10.3} │ {:>10.3}",
            hiy - loy,
            (hiy + loy) / 2.0,
            hix - lox,
            (hix + lox) / 2.0
        );
    }
    eprintln!();
}

/// ⛔⛔ **O QUE O CARTÃO DO GRID PINTA, NA ORDEM** — o instrumento que o report do dono de
/// 2026-09-19 (*«Gap y quebrou e movimenta tudo em vez de criar espaço»*) obriga a ter.
///
/// ⚠️ A secção do gizmo entrou naquele cartão na mesma wave, e **uma tabela de grupos PARCIAL** é a
/// hipótese que esta sonda serve para confirmar ou matar: se a ordem ou o dono de cada linha
/// mudou, o artista arrasta uma linha e escreve noutro param.
///
/// `cargo test -p ph2d-app-motion --lib -- --ignored --nocapture o_que_o_cartao_do_grid_pinta`
#[test]
#[ignore = "sonda, nao um gate"]
fn o_que_o_cartao_do_grid_pinta() {
    let m = MotionState::new();
    let tid = ph2d_nodegraph::node::NodeTypeId::of("motion.grid");
    use ph2d_nodegraph::cook::OpResolver;
    let op = m.registry.resolve(tid).expect("o Grid existe");
    eprintln!("\n=== O CARTAO DO `motion.grid` ===\n");
    eprintln!("  # │ param          │ rotulo          │ seccao");
    for (i, p) in op.manifest().params.iter().enumerate() {
        let hint = m
            .registry
            .param_ui(tid)
            .into_iter()
            .flat_map(|t| t.iter())
            .find(|h| h.param == p.name);
        let grupo = m
            .registry
            .param_groups(tid)
            .iter()
            .find(|g| g.param == p.name)
            .map_or("—", |g| g.group);
        eprintln!(
            "  {i} │ {:<14} │ {:<15} │ {grupo}",
            p.name,
            hint.map_or("(SEM DICA)", |h| h.label)
        );
    }
    eprintln!();
}

/// ⭐⭐⭐ **O `gap_y` NA GRELHA DO DONO — a grelha estava CERTA, e este é o número que o prova.**
///
/// Report de 2026-09-19: *«Gap y quebrou e movimenta tudo em vez de criar espaço»*, sobre a
/// *«grade do segundo exemplo»* — a cena **`=2`**, que é uma `motion.grid` de **360 × 360 =
/// 129 600** elementos.
///
/// ⚠️⚠️ **A causa NÃO era o nó, e esta sonda é o que o mostra:** a nuvem inteira ESPAÇA (o centro
/// fica parado e a extensão cresce) em todas as posições do knob. Quem mentia era o **desenho** —
/// o gizmo de posições, que segurava um PREFIXO da nuvem: uma faixa na borda de baixo, que voava.
///
/// ⚠️ **A sonda imprime as DUAS colunas de propósito** (a nuvem e o que o gizmo segura), porque a
/// cura é que elas passem a concordar: com a amostra por passo o centro do que se desenha fica
/// parado, como o da nuvem. Sem a segunda coluna, ela não pode acusar uma recaída.
///
/// ⚠️ **A sonda irmã [`o_gap_y_ainda_espaca`] mede uma grelha de `4 × 4`**, e é por isso que ela
/// não podia ver nada: *uma fixtura pequena não testa o que só aparece na escala do dono.*
///
/// `cargo test -p ph2d-app-motion --lib -- --ignored --nocapture o_gap_y_na_grelha_do_dono`
#[test]
#[ignore = "sonda, nao um gate"]
fn o_gap_y_na_grelha_do_dono() {
    use ph2d_nodegraph::attr::Column;
    use ph2d_nodegraph::graph::{Edge, Graph};
    // A geometria da cena `=2`, lida dela: `motion_state_gpu_demos.rs`.
    const ROWS: f32 = 360.0;
    const COLS: f32 = 360.0;
    eprintln!("\n=== O `gap_y` NA GRELHA DE {ROWS:.0}x{COLS:.0} (a cena `=2`) ===\n");
    eprintln!("  gap_y │  extensao Y │   centro Y  │ pontos ║ GIZMO: ext │  centro Y  │ pontos");
    let mut antes: Option<(f32, f32)> = None;
    for gy in [1.0f32, 1.5, 2.0, 3.0] {
        let mut m = MotionState::new();
        let mut g = Graph::new();
        let grelha = g.add_node("motion.grid");
        g.set_param(grelha, "rows", ROWS);
        g.set_param(grelha, "cols", COLS);
        g.set_param(grelha, "gap_x", 1.0);
        g.set_param(grelha, "gap_y", gy);
        let saida = g.add_node("motion.output");
        g.connect(Edge {
            from: (grelha, 0),
            to: (saida, 0),
            delayed: false,
        })
        .expect("liga");
        m.doc.graph = g;
        let Ok(out) = m.pump.cook.cook(&m.doc.graph, &m.registry, saida, 0.0) else {
            eprintln!("  {gy} │ NAO COZE");
            continue;
        };
        let s = out[0].as_stream();
        let Some(Column::Vec2(p)) = s.get("P") else {
            continue;
        };
        let medir = |v: &[[f32; 2]]| {
            let (mut lo, mut hi) = (f32::MAX, f32::MIN);
            for q in v {
                lo = lo.min(q[1]);
                hi = hi.max(q[1]);
            }
            (hi - lo, (hi + lo) / 2.0)
        };
        let (ext_t, cen_t) = medir(p);
        // ⭐ O que o GIZMO segura, pela porta do produto — ver `ponto_gizmo::posicoes_amostradas`.
        // ⛔ Reimplementar o corte aqui mediria a cópia, e não a lei.
        let seg = crate::ponto_gizmo::posicoes_amostradas(s);
        let (ext_g, cen_g) = medir(&seg);
        eprintln!(
            "  {gy:>5} │ {ext_t:>11.3} │ {cen_t:>11.3} │ {:>6} ║ {ext_g:>11.3} │ {cen_g:>11.3} │ {}",
            p.len(),
            seg.len()
        );
        if let Some((e0, c0)) = antes {
            eprintln!(
                "        │  Δextensao {:>+8.3} │ Δcentro {:>+8.3}  ⇒ mover/espacar = {:.2}x",
                ext_t - e0,
                cen_t - c0,
                (cen_t - c0).abs() / (ext_t - e0).abs().max(f32::EPSILON)
            );
        }
        antes = Some((ext_t, cen_t));
    }
    eprintln!(
        "\n  ⇒ o `gap_y` ESPACA: o centro fica em 0,000 e a extensao acompanha, em todo o curso\n              do knob. O report do dono era o DESENHO — o gizmo de posicoes segurava as primeiras\n              fileiras da nuvem (uma faixa na borda) e ela VOAVA; esse gizmo foi RETIRADO.\n"
    );
}
