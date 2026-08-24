//! **Todo param de todo nó CHEGA ao painel** — o censo do teto de linhas (doc 88, B3).
//!
//! Irmão do `range_tests` (a ESCALA de um valor) e do `unit_tests` (a UNIDADE dele): este mede
//! se o valor **aparece**. Um param acima do `MAX_PARAM_ROWS` não é desenhado nem registrado —
//! o `.take()` do `paint_rows` o descarta —, então ele existe no modelo, o cook o lê, e o
//! artista não tem gesto nenhum que o alcance. É a falha silenciosa que as quatro condições de
//! UI proíbem, e a única testemunha possível é um censo sobre o registry inteiro: nenhum gate
//! por-nó a veria, porque cada um usa a fixture do seu próprio nó.
//!
//! ⚠️ O teto é um recurso de verdade — o `populate` do painel registra **21 widgets por slot**
//! —, então ele não pode simplesmente sumir; o que ele pode é ser **medido** (§0). A sonda
//! abaixo imprime o censo; o gate o mantém honesto.

use super::params::build_params_snapshot;
use crate::motion_state::MotionState;
use ph2d_editor::ProjectSettings;
use ph2d_panel_motion_params::MAX_PARAM_ROWS;

/// Quantas linhas de painel cada tipo de nó do registry produz, do maior para o menor.
///
/// ⚠️ Conta as linhas do SNAPSHOT, não os `ParamSpec` do manifesto: um nó emite também as
/// linhas de text param (Curve / Gradient / Palette / Text / Source / Channels) **antes** do
/// laço do manifesto, e é a soma que disputa os slots. Contar o manifesto responderia a outra
/// pergunta e reportaria um teto folgado demais.
fn row_census() -> Vec<(&'static str, usize)> {
    let mut motion = MotionState::new();
    let types: Vec<&'static str> = motion.registry.manifests().map(|m| m.name).collect();
    let mut census: Vec<(&'static str, usize)> = types
        .into_iter()
        .map(|ty| {
            let node = motion.doc.graph.add_node(ty);
            ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);
            let n = build_params_snapshot(&motion, ProjectSettings::default())
                .map_or(0, |s| s.rows.len());
            (ty, n)
        })
        .collect();
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    census.sort_by_key(|&(_, n)| std::cmp::Reverse(n));
    census
}

/// **Nenhum param fica fora da tela.**
///
/// Nasceu VERMELHO contra o teto de 8 que shipava: o `field.remap` produz linhas acima dele, e
/// as excedentes eram descartadas em silêncio. A mutação que o prova é baixar
/// `MAX_PARAM_ROWS` de volta — o gate nomeia o nó e a contagem, em vez de dizer só "falhou".
#[test]
fn the_panel_shows_every_param_of_every_node() {
    let census = row_census();
    let over: Vec<String> = census
        .iter()
        .filter(|(_, n)| *n > MAX_PARAM_ROWS)
        .map(|(ty, n)| format!("{ty} ({n} linhas)"))
        .collect();
    assert!(
        over.is_empty(),
        "estes nós têm mais linhas que MAX_PARAM_ROWS ({MAX_PARAM_ROWS}), e o excedente é \
         descartado pelo `.take()` do paint_rows — o param existe e o artista não o alcança: \
         {over:?}"
    );
}

/// **E o teto não é folgado a ponto de não medir nada.**
///
/// A metade oposta, e ela não é cerimônia: sem isto, "conserte o gate acima" tem uma resposta
/// trivial e errada — pôr o teto em 256 e pagar 5376 registros de widget no `populate` por um
/// número que ninguém mediu. O teto é o pior caso medido mais folga de uma família; se o censo
/// cair muito abaixo dele, é sinal de que ele foi escolhido em vez de medido.
#[test]
fn the_row_cap_is_measured_not_guessed() {
    let census = row_census();
    let worst = census.first().copied().expect("o registry não é vazio");
    assert!(
        worst.1 <= MAX_PARAM_ROWS,
        "o pior nó ({} com {} linhas) não cabe no teto {MAX_PARAM_ROWS}",
        worst.0,
        worst.1
    );
    assert!(
        MAX_PARAM_ROWS <= worst.1 * 2,
        "o teto {MAX_PARAM_ROWS} é mais que o dobro do pior nó medido ({} com {} linhas) — \
         cada slot custa 21 registros de widget no populate, então isto é orçamento gasto \
         num número que ninguém mediu",
        worst.0,
        worst.1
    );
}

/// A SONDA: imprime o censo inteiro, para o número do teto sair de uma medição.
/// `cargo test -p ph2d-host-desktop measure_the_param_row_census -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição, não gate"]
fn measure_the_param_row_census() {
    let census = row_census();
    println!("\n=== LINHAS DE PAINEL POR NÓ (teto atual: {MAX_PARAM_ROWS}) ===");
    for (ty, n) in census.iter().take(20) {
        let flag = if *n > MAX_PARAM_ROWS {
            "  <-- CORTADO"
        } else {
            ""
        };
        println!("{n:3}  {ty}{flag}");
    }
    let over = census.iter().filter(|(_, n)| *n > MAX_PARAM_ROWS).count();
    println!(
        "--- {} tipos no total, {} acima do teto\n",
        census.len(),
        over
    );
}

/// **As SEÇÕES agrupam as rows, e a ordem é a que a tabela declara.**
///
/// Um nó com grupos tem de entregar as rows já ORDENADAS (soltas primeiro, depois grupo a
/// grupo) e dizer onde cada seção começa. Sem a ordenação o pintor desenharia o mesmo
/// cabeçalho várias vezes, intercalado — que é pior que lista plana.
#[test]
fn a_grouped_node_delivers_its_rows_sorted_with_the_loose_ones_first() {
    let mut motion = MotionState::new();
    let node = motion.doc.graph.add_node("field.remap");
    ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);
    let snap = build_params_snapshot(&motion, ProjectSettings::default()).expect("o nó existe");

    assert!(
        !snap.sections.is_empty(),
        "o field.remap declara grupos — sem seções a tabela não chegou ao painel"
    );
    // As soltas vêm antes da primeira seção, e são os essenciais (a transferência).
    let first = snap.sections[0].1;
    assert!(
        first > 0,
        "os params SEM grupo têm de vir antes de toda seção — é onde os essenciais moram"
    );
    // Os índices são crescentes e cada seção começa onde uma row começa.
    let mut prev = 0;
    for (title, at) in &snap.sections {
        assert!(*at > prev || *at == first, "{title}: seções fora de ordem");
        assert!(
            *at < snap.rows.len(),
            "{title}: seção depois do fim das rows"
        );
        prev = *at;
    }
    // E nenhuma seção repete: rows do mesmo grupo são contíguas.
    let titles: Vec<&String> = snap.sections.iter().map(|(t, _)| t).collect();
    let mut uniq = titles.clone();
    uniq.sort();
    uniq.dedup();
    assert_eq!(
        titles.len(),
        uniq.len(),
        "um grupo apareceu duas vezes — as rows dele não estão contíguas: {titles:?}"
    );
    // E nenhuma seção fica VAZIA: um cabeçalho sem rows embaixo é a seção-morta, irmã do
    // botão-morto — ele desenha, dobra, e não esconde nada.
    for (k, (title, at)) in snap.sections.iter().enumerate() {
        let end = snap
            .sections
            .get(k + 1)
            .map_or(snap.rows.len(), |(_, next)| *next);
        assert!(
            end > *at,
            "a seção {title} não tem row nenhuma embaixo dela"
        );
    }
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// **Toda entrada de `ParamGroup` nomeia um param que o nó DECLARA.**
///
/// Um nome errado numa tabela de seções não falha em lugar nenhum: o `param_group` não acha a
/// entrada, a row fica solta, e "solta" é exatamente o que uma escolha deliberada também
/// parece. O nó continua compilando, o painel continua pintando, e o param que devia estar
/// numa seção fica na parede — que é o problema que a seção existe para resolver.
///
/// Por isso o censo é sobre o REGISTRY inteiro, e não sobre um nó: cada gate por-nó usa a
/// fixture do próprio nó, e a tabela do sétimo nasce sem testemunha nenhuma.
///
/// O conjunto aceito é *params do manifesto* ∪ *nomes das rows do snapshot* — a união porque um
/// text param (curva, gradiente, paleta, fórmula) produz row e **não** aparece no manifesto,
/// e agrupá-lo é legítimo.
#[test]
fn every_param_group_entry_names_a_param_the_node_declares() {
    let mut motion = MotionState::new();
    let types: Vec<(&'static str, ph2d_nodegraph::node::NodeTypeId)> = motion
        .registry
        .manifests()
        .map(|m| (m.name, m.id))
        .collect();
    let mut bad: Vec<String> = Vec::new();
    for (ty, id) in types {
        let groups = motion.registry.param_groups(id);
        if groups.is_empty() {
            continue;
        }
        let node = motion.doc.graph.add_node(ty);
        ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);
        let mut declared: std::collections::BTreeSet<String> = Default::default();
        if let Some(snap) = build_params_snapshot(&motion, ProjectSettings::default()) {
            for row in &snap.rows {
                declared.extend(row.params().iter().map(|p| (*p).to_string()));
            }
        }
        if let Some(op) = {
            use ph2d_nodegraph::cook::OpResolver;
            motion.registry.resolve(id)
        } {
            declared.extend(op.manifest().params.iter().map(|p| p.name.to_string()));
        }
        for g in groups {
            if !declared.contains(g.param) {
                bad.push(format!("{ty}: a seção {:?} nomeia {:?}", g.group, g.param));
            }
        }
    }
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    assert!(
        bad.is_empty(),
        "estas entradas de ParamGroup nomeiam params que o nó não tem — a row fica SOLTA e \
         nada acusa: {bad:?}"
    );
}

/// **E os nós que a medição nomeou de fato entregam seções.**
///
/// A metade oposta do gate acima: sem ela, "conserte os nomes" tem a resposta trivial de
/// apagar as tabelas. A lista sai da sonda `measure_the_param_row_census` — são os nós de 9+
/// linhas, os que a parede de sliders de fato machuca.
#[test]
fn the_nodes_the_census_named_all_ship_sections() {
    let mut motion = MotionState::new();
    for ty in [
        "field.remap",
        "motion.emitter",
        "motion.boids",
        "field.radial_sweep",
        "value.pattern",
        "motion.spline_wrap",
        "motion.distribute_curve",
    ] {
        let node = motion.doc.graph.add_node(ty);
        ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);
        let snap = build_params_snapshot(&motion, ProjectSettings::default())
            .unwrap_or_else(|| panic!("{ty} existe no registry"));
        assert!(
            !snap.sections.is_empty(),
            "{ty} tem {} linhas e nenhuma seção — é a parede plana que o doc 88 B3 ataca",
            snap.rows.len()
        );
        // E sobra algo SOLTO: uma seção que engole o nó inteiro põe todo controle atrás de um
        // clique, que troca uma parede por uma porta trancada.
        assert!(
            snap.sections[0].1 > 0,
            "{ty} agrupou TODOS os params — nenhum essencial ficou solto na frente",
        );
    }
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// **Só a régua ESCOLHIDA é oferecida** — o `time_mode` do oscilador (doc 88 B3).
///
/// `frequency` e `bpm` são a MESMA grandeza em duas unidades. Mostrar as duas seria pior que
/// um botão morto: seriam dois números na tela discordando sobre um só valor, sem nada dizendo
/// qual manda — e o cook lê exatamente um deles.
///
/// As duas metades (presença E ausência) num gate só, porque cada uma sozinha tem resposta
/// trivial: "sempre mostre os dois" passa na presença, "nunca mostre nenhum" passa na ausência.
#[test]
fn the_oscillator_offers_only_the_time_ruler_it_uses() {
    let mut motion = MotionState::new();
    let node = motion.doc.graph.add_node("motion.oscillator");
    ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);

    let names = |motion: &MotionState| -> Vec<String> {
        build_params_snapshot(motion, ProjectSettings::default())
            .expect("o no existe")
            .rows
            .iter()
            .flat_map(|r| r.params().into_iter().map(|p| p.to_string()))
            .collect()
    };

    // Segundos (o default): o Hz aparece, o BPM não.
    let secs = names(&motion);
    assert!(secs.iter().any(|p| p == "frequency"), "{secs:?}");
    assert!(!secs.iter().any(|p| p == "bpm"), "{secs:?}");

    // BPM: exatamente o inverso.
    motion.doc.graph.set_param(node, "time_mode", 1.0);
    let bpm = names(&motion);
    assert!(bpm.iter().any(|p| p == "bpm"), "{bpm:?}");
    assert!(!bpm.iter().any(|p| p == "frequency"), "{bpm:?}");

    // E o SELETOR está sempre lá — a régua se escolhe, então o controle que a escolhe não
    // pode desaparecer com a escolha (seria a única porta de volta).
    assert!(
        secs.iter().any(|n| n == "time_mode"),
        "o seletor sumiu em Seconds"
    );
    assert!(
        bpm.iter().any(|n| n == "time_mode"),
        "o seletor sumiu em BPM"
    );
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// A altura do dock do inspector, do dono dela — nunca um literal copiado.
use ph2d_editor::screens::layout::INSPECTOR_MAX_H;
use ph2d_editor::zones::Rect;

/// Quanto o painel OCUPA ao desenhar cada tipo de nó, do maior para o menor.
///
/// ⚠️ O oráculo são os **retângulos que o próprio painel registrou**, não uma soma de alturas
/// de linha ao lado dele: as linhas não têm a mesma altura (um editor de Curva, de Gradiente ou
/// de Paleta devolve a própria), então *mais linhas* não é o mesmo que *mais alto* — e uma
/// segunda aritmética divergiria exatamente no nó composto, que é o caso que importa.
/// A altura do CORPO do inspector — a régua contra a qual «transborda?» se
/// pergunta.
///
/// ⚠️ **Não é `INSPECTOR_MAX_H`**, que é a do dock INTEIRO: entre os dois há a
/// faixa do título e o padding, e o número que o `dispatch_wheel` compara é o do
/// corpo. Enquanto a sonda media o fundo do último hit-rect (em coordenadas de
/// ECRÃ) as duas coincidiam por acidente; medindo o conteúdo PUBLICADO — que
/// começa em zero — elas diferem pela altura do cabeçalho, e comparar contra a do
/// dock diria que um nó cabe quando ele já não cabe.
fn inspector_body_h() -> f32 {
    reach_census_body()
}

fn height_census() -> Vec<(&'static str, f32)> {
    // ⚠️ **A altura é a que o painel PUBLICA, e não o fundo do último hit-rect** —
    // e a régua mudou por uma razão que vale registar. Desde que o corpo blinda o
    // `HitIndex` com a mesma banda que recorta o desenho
    // (`nothing_scrolled_above_the_body_can_still_be_clicked_under_the_title`), o
    // fundo do último retângulo **satura na altura do corpo**: o
    // `motion.bezier_warp` passou a ler `802` de um dock de `880` e o gate
    // acusou-o de ter perdido params. Não perdeu — a sonda é que deixou de medir
    // o nó e passou a medir a janela.
    let mut census: Vec<(&'static str, f32)> = reach_census()
        .into_iter()
        .map(|(ty, _, content)| (ty, content))
        .collect();
    census.sort_by(|a, b| b.1.total_cmp(&a.1));
    census
}

thread_local! {
    /// A altura VISÍVEL que o último censo leu — a mesma para todo nó (ela é do
    /// dock, não do conteúdo), publicada aqui para o gate a poder comparar.
    static BODY_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
}

fn reach_census_body() -> f32 {
    let _ = reach_census();
    BODY_H.with(std::cell::Cell::get)
}

/// `(tipo, fundo do último retângulo registado, altura de CONTEÚDO publicada)` por nó.
///
/// ⚠️ **A terceira coluna é a que passou a decidir**, e a razão é uma medição: o corpo do
/// painel **ROLA** desde a wave da rolagem (`lib_scroll_tests`), e o `dispatch_wheel` deriva
/// o `max_scroll` de `content_h`/`visible_h`. A primeira — o fundo do último hit-rect —
/// deixou de medir o nó desde que o corpo blinda o `HitIndex`: ela satura na janela, e o que
/// responde agora é *o que se pode APONTAR*, não *o que se desenhou*.
fn reach_census() -> Vec<(&'static str, f32, f32)> {
    let mut motion = MotionState::new();
    let types: Vec<&'static str> = motion.registry.manifests().map(|m| m.name).collect();
    let census: Vec<(&'static str, f32, f32)> = types
        .into_iter()
        .map(|ty| {
            let node = motion.doc.graph.add_node(ty);
            ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);
            let snap = build_params_snapshot(&motion, ProjectSettings::default());
            ph2d_panel_motion_params::set_current_params(snap);
            let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<
                ph2d_panel_motion_params::MotionParamsPanel,
            >();
            let mut state = ph2d_panel_motion_params::MotionParamsPanelState;
            let rects = host.paint::<ph2d_panel_motion_params::MotionParamsPanel>(
                &mut state,
                Rect {
                    x: 0.0,
                    y: 0.0,
                    w: ph2d_editor::screens::layout::INSPECTOR_W,
                    h: INSPECTOR_MAX_H,
                },
            );
            // O fundo do retângulo mais baixo que o painel registrou: o último pixel que o
            // artista consegue apontar.
            let bottom = rects.iter().map(|(_, r)| r.y + r.h).fold(0.0f32, f32::max);
            let content = host
                .store()
                .panel_content_h(ph2d_editor::ids::MOTION_PARAMS_PANEL)
                .unwrap_or(0.0);
            let visible = host
                .store()
                .panel_visible_h(ph2d_editor::ids::MOTION_PARAMS_PANEL)
                .unwrap_or(0.0);
            BODY_H.with(|b| b.set(visible));
            (ty, bottom, content)
        })
        .collect::<Vec<_>>();
    ph2d_panel_motion_params::set_current_params(None);
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    census
}

/// A SONDA da altura: imprime o censo, para o veredito sair de uma medição.
/// `cargo test -p ph2d-host-desktop --bins measure_the_param_height_census -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição, não gate"]
fn measure_the_param_height_census() {
    let census = height_census();
    println!("\n=== ALTURA OCUPADA POR NÓ (dock: {INSPECTOR_MAX_H} px) ===");
    for (ty, h) in census.iter().take(20) {
        let over = if *h > INSPECTOR_MAX_H {
            "  <-- ESTOURA"
        } else {
            ""
        };
        println!("{h:7.1}  {ty}{over}");
    }
    let over = census.iter().filter(|(_, h)| *h > INSPECTOR_MAX_H).count();
    println!(
        "--- {} tipos no total, {over} acima da altura do dock\n",
        census.len()
    );
}

/// **TODA LINHA REGISTADA CABE NO CONTEÚDO PUBLICADO** — a outra metade do corte silencioso,
/// **recalibrada em 2026-08-21 porque a premissa da anterior dissolveu**.
///
/// O gate irmão prova que nenhum param é descartado pelo `.take()` do `MAX_PARAM_ROWS`. Este
/// vê a segunda porta para a MESMA invisibilidade — e qual é essa porta mudou:
///
/// ⚠️ **Este gate chamava-se `every_node_fits_the_inspector_dock` e o corpo dele afirmava
/// *"o painel **não rola**"*.** Isso era verdade quando ele nasceu e deixou de ser: o corpo
/// do painel de params **rola** desde a wave da rolagem (`ph2d-panel-motion-params::
/// lib_scroll_tests`, cujo cabeçalho diz literalmente *"a altura deixa de ser um limite de
/// produto"*), o `forwarding.rs` intercepta a roda sobre o `MOTION_PARAMS_PANEL`, e o
/// arch-gate `scrollable_panels_intercept_the_wheel` guarda essa ligação. Desenhar abaixo de
/// 880 px deixou de ser inalcançável.
///
/// ⚠️ **O que NÃO mudou é o que este gate passa a medir:** o `dispatch_wheel` deriva o
/// `max_scroll` de `content_h`/`visible_h`, então uma linha registada **além do conteúdo
/// publicado** continua fora de alcance — o rolamento para antes dela, e o modo de falha é o
/// mesmo de sempre (o param existe, o painel o regista, o artista não chega lá).
///
/// ⚠️ **O oráculo é ROLAR de verdade, não uma aritmética ao lado.** A primeira versão desta
/// recalibração comparava `fundo − conteúdo` contra uma constante derivada do censo, e a
/// premissa dela era falsa: o desvio é 114 px em 115 nós e **110 no `motion.color_array`**,
/// porque uma linha de PALETA fecha com folga diferente de uma escalar. Uma segunda
/// aritmética sobre o layout diverge do layout — então aqui se põe o rolamento no máximo que
/// o painel publica e se pergunta ao painel onde a última linha FICOU.
///
/// ⚠️ O `INSPECTOR_MAX_H` continua a ser lido — ele é a altura VISÍVEL, e é o que separa
/// *cabe* de *rola*; a sonda irmã imprime os dois.
/// ⚠️ **A fixture TRANSBORDA de propósito**, e é o que separa este gate de um verde por
/// acidente: no estado de DEFAULT o nó mais alto do catálogo cabe no dock (a sonda irmã
/// imprime o número), então rolar não teria o que provar. Um `source.shape` com traço abre a
/// família inteira do traço — cor, tracejado e os três do Trim — e é o pior caso REAL do
/// catálogo de hoje. *Uma fixture só prova o que contém.*
#[test]
fn the_last_row_of_the_tallest_node_is_reachable_by_scrolling() {
    let tallest = "source.shape";
    let mut motion = MotionState::new();
    let node = motion.doc.graph.add_node(tallest);
    motion
        .doc
        .graph
        .set_param(node, ph2d_node_motion_shape::param::STROKE_WIDTH, 0.2);
    ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);
    ph2d_panel_motion_params::set_current_params(build_params_snapshot(
        &motion,
        ProjectSettings::default(),
    ));
    let mut host =
        ph2d_ui_testkit::MockPanelHost::with_panel::<ph2d_panel_motion_params::MotionParamsPanel>();
    let dock = Rect {
        x: 0.0,
        y: 0.0,
        w: ph2d_editor::screens::layout::INSPECTOR_W,
        h: INSPECTOR_MAX_H,
    };
    let paint = |host: &mut ph2d_ui_testkit::MockPanelHost| -> f32 {
        let mut state = ph2d_panel_motion_params::MotionParamsPanelState;
        host.paint::<ph2d_panel_motion_params::MotionParamsPanel>(&mut state, dock)
            .iter()
            .map(|(_, r)| r.y + r.h)
            .fold(0.0f32, f32::max)
    };
    paint(&mut host);
    let content = host
        .store()
        .panel_content_h(ph2d_editor::ids::MOTION_PARAMS_PANEL)
        .expect("o painel publica a altura do CONTEÚDO");
    let visible = host
        .store()
        .panel_visible_h(ph2d_editor::ids::MOTION_PARAMS_PANEL)
        .expect("...e a VISÍVEL, que é a régua do `dispatch_wheel`");
    // ⚠️ **A régua é `content` contra `visible`, e não o fundo do último hit-rect
    // contra a altura do dock.** Desde a blindagem do `HitIndex` aquele fundo
    // satura na janela — ele mede o que se pode APONTAR, que é precisamente o que
    // este gate não quer saber aqui.
    assert!(
        content > visible,
        "a fixture tem de TRANSBORDAR, senao rolar nao prova nada: {content:.0} px de \
         {visible:.0}"
    );
    // O rolamento máximo que o `dispatch_wheel` deixa o artista pedir — derivado do painel,
    // nunca de uma segunda conta.
    let max_scroll = (content - visible).max(0.0);
    use ph2d_editor::panel::PanelHostInternal as _;
    host.store_mut()
        .set_panel_scroll(ph2d_editor::ids::MOTION_PARAMS_PANEL, max_scroll);
    let rolled = paint(&mut host);
    ph2d_panel_motion_params::set_current_params(None);
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    assert!(
        rolled <= INSPECTOR_MAX_H + 0.5,
        "o nó mais alto ({tallest}) tem {content:.0} px de linhas e o painel só deixa rolar \
         {max_scroll:.0} px (conteúdo {content:.0}, visível {visible:.0}) — no fim do \
         rolamento a última linha ainda termina em {rolled:.0}, além do dock de \
         {INSPECTOR_MAX_H} px, e nenhum gesto a alcança"
    );
}

/// **Quem passa do dock é NOMEADO aqui** — a sonda do gate acima, agora que caber deixou de
/// ser lei.
///
/// Um nó mais alto que o dock não é um defeito (ele rola), mas é um FATO de produto: abrir o
/// inspector e já precisar da roda é pior UX que caber. O número fica escrito aqui, e a
/// mutação que o prova é esconder um param — a altura desce.
///
/// ⚠️ **Ele deixou de exigir a lista VAZIA em 2026-08-23, e a mudança é sobre o que ele
/// sempre disse ser:** o doc acima chama-se *"o número fica escrito aqui"*, e a asserção
/// `is_empty()` fazia dele *"ninguém pode passar"*. O `motion.bezier_warp` passa — 24 params
/// são a superfície do *Bezier Warp* da referência (4 cantos + 8 tangentes × `(x, y)`), e um
/// lado sem as duas tangentes deixa de ser uma cúbica: cortar a superfície para caber num
/// dock seria deixar o dock desenhar o produto. ⇒ a exceção é **NOMEADA, com a altura
/// medida**, e qualquer nó que ela não nomeie continua a deixar a suíte vermelha.
///
/// ⛔ E ela não é um allowlist a crescer: um segundo nome aqui é sinal de que a resposta certa
/// passou a ser **secções recolhíveis** no painel (que hoje não existem — os `ParamGroup` são
/// cabeçalhos, sem estado de aberto/fechado), e não mais uma linha nesta tabela.
#[test]
fn the_dock_overflow_is_named_not_discovered() {
    /// Os nós que passam do dock **de propósito**, com a altura medida em 2026-08-23.
    ///
    /// ⚠️ **A primeira leitura deste número foi `920`, e ela estava ERRADA por um motivo que
    /// vale registar:** ela foi tirada enquanto o `MAX_PARAM_ROWS` ainda era `20`, ou seja
    /// com o painel a **cortar quatro das 24 linhas**. `920` era a altura do TETO, não a do
    /// nó. Com o teto no lugar certo ele mede `1083`. *Uma altura medida sob um limite que
    /// está a cortar é a altura do limite.*
    ///
    /// ⚠️ **E a segunda leitura foi `1083` medida pela régua ERRADA.** Ela vinha do
    /// fundo do último hit-rect, em coordenadas de ECRÃ — logo incluía a faixa do
    /// título. Desde que o corpo blinda o `HitIndex`, esse fundo satura na altura
    /// da janela e deixou de medir o nó; a régua passou a ser a altura de CONTEÚDO
    /// que o painel publica, que começa em zero. O mesmo nó, o mesmo painel:
    /// **969**.
    const NAMED_OVERFLOW: &[(&str, f32)] = &[("motion.bezier_warp", 969.0)];
    let body = inspector_body_h();
    let census = height_census();
    let (worst_ty, worst_h) = census.first().copied().expect("o registry não é vazio");
    let over: Vec<String> = census
        .iter()
        .filter(|(ty, h)| *h > body && !NAMED_OVERFLOW.iter().any(|(n, _)| n == ty))
        .map(|(ty, h)| format!("{ty} ({h:.0} px)"))
        .collect();
    assert!(
        over.is_empty(),
        "estes nós desenham além da altura do CORPO ({body:.0} px) no estado de \
         DEFAULT e NÃO estão nomeados — alcançáveis pela roda, mas o inspector abre já a \
         precisar dela: {over:?}. O pior é {worst_ty} com {worst_h:.0} px."
    );
    // ⚠️ E o nomeado tem de continuar a ser o que se mediu: uma altura que ANDE (para cima
    // ou para baixo) é um param que entrou ou saiu sem ninguém reparar. Ela não é uma barra,
    // é um retrato.
    for (ty, px) in NAMED_OVERFLOW {
        let got = census
            .iter()
            .find(|(t, _)| t == ty)
            .unwrap_or_else(|| panic!("o nó nomeado `{ty}` sumiu do registry"))
            .1;
        assert!(
            (got - px).abs() < 1.0,
            "`{ty}` media {px:.0} px quando foi nomeado e mede {got:.0} agora — um param \
             entrou ou saiu; re-meça e mova o número, ou desfaça"
        );
        assert!(
            got > body,
            "`{ty}` já CABE no corpo ({got:.0} ≤ {body:.0}) — tire-o da lista"
        );
    }
}
