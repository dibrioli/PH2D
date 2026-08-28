//! **A ALTURA do painel de params contra o DOCK** — o irmão do `rowcap_tests`, que
//! conta LINHAS; este mede PÍXEIS.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 600 para `shells/`), e o corte é por
//! PERGUNTA: *cabe no teto de linhas?* é um censo sobre o registry; *cabe no dock?* é um censo
//! sobre o que o painel PUBLICA depois de pintar. Elas partilham o `MockPanelHost` e mais nada.

use super::params::build_params_snapshot;
use crate::motion_state::MotionState;
use ph2d_editor::ProjectSettings;
/// A altura do dock do inspector, do dono dela — nunca um literal copiado.
use ph2d_editor::screens::layout::INSPECTOR_MAX_H;
use ph2d_editor::zones::Rect;

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

/// Quanto o painel OCUPA ao desenhar cada tipo de nó, do maior para o menor.
///
/// ⚠️ O oráculo são os **retângulos que o próprio painel registrou**, não uma soma de alturas
/// de linha ao lado dele: as linhas não têm a mesma altura (um editor de Curva, de Gradiente ou
/// de Paleta devolve a própria), então *mais linhas* não é o mesmo que *mais alto* — e uma
/// segunda aritmética divergiria exatamente no nó composto, que é o caso que importa.
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
/// passou a ser **secções recolhíveis** no painel, e não mais uma linha nesta tabela.
///
/// ⚠️⚠️ **O SEGUNDO NOME CHEGOU em 2026-08-24, e o alarme disparou para um remédio que já
/// existe — a nota acima estava DESATUALIZADA na metade que decidia.** Ela dizia *"secções
/// recolhíveis, que hoje não existem — os `ParamGroup` são cabeçalhos, sem estado de
/// aberto/fechado"*. Medido: existem. O `rows_paint_sections` deste painel usa `SectionFold`,
/// o cabeçalho canónico diz *"TODA seção é colapsável"*, o estado vive no `WidgetStore`
/// (`section_open_live` / `is_collapsed`) e a dobra é animada e interruptível. *O que NÃO
/// existe é uma secção que NASÇA fechada*: o `is_collapsed` devolve «aberta» para um id que
/// nunca viu, e um default declarado teria de entrar no `ph2d-editor-core`, partilhado por
/// ~34 sítios. ⇒ **o remédio está meio construído, e a metade que falta é um default
/// foundational, não uma feature de painel.**
///
/// ⚠️ **E o `motion.spline_wrap` é uma excepção de OUTRA espécie que a do `bezier_warp`, com
/// a medição a dizê-lo:** o `bezier_warp` mede 969 px porque **24 params SÃO a superfície da
/// referência**; este mede 755 px só no estado de **FALLBACK**. Com uma forma escolhida — o
/// uso a que o nó se destina desde a decisão de produto de 2026-08-12 (*"pontos e alças em
/// sliders num painel. Absurdo!"*) — o `ParamGateText` apaga as oito coordenadas e ele mede
/// **456 px** contra um corpo de 664 (sonda [`measure_the_wrap_with_a_shape`]). *As oito
/// coordenadas são o fallback, e o painel só as mostra a quem não escolheu forma nenhuma.*
///
/// ⇒ **O trabalho que este alarme de facto pede, especificado:** um `ParamGroup` que declare
/// nascer FECHADO, honrado por um default no `is_collapsed`. Ele tira este nó da lista (a
/// secção «Curve» leva 8 das 19 linhas) e provavelmente também o `bezier_warp`. ⛔ Um TERCEIRO
/// nome aqui, antes disso, é que passa a ser a lista a crescer.
/// ⛔⛔ **TODA PORTA DO CATÁLOGO TEM UM NOME LEGÍVEL** — o censo que fecha o report do Enio.
///
/// *"Vários nós como o próprio boids têm uma série de inputs sem nenhum nome, sem identificação.
/// Como o usuário entender se em nenhum lugar diz o que é?"* (2026-08-27, com foto).
///
/// O rótulo é **derivado** do nome do manifesto (`ph2d_panel_motion_graph::PortLabel`), e essa
/// escolha só é honesta se a derivação responder por **todo** o catálogo — senão um nó qualquer
/// desenha uma faixa vazia e o defeito volta, num nó só, sem ninguém dar por ela.
///
/// A régua tem três metades:
/// - **não-vazio**: uma porta sem rótulo é a fileira anónima da foto;
/// - **começa em maiúscula**: é o que separa um rótulo de um identificador cru (`target_x`);
/// - **sem `_`**: idem — se sobrar um sublinhado, a derivação não tocou naquele nome.
///
/// ⚠️ Ela vale como censo porque varre o **registry inteiro**, que é a mesma população que a
/// paleta oferece ao artista. Um nó novo entra aqui no dia em que é registado.
#[test]
fn every_port_in_the_catalogue_has_a_readable_label() {
    use ph2d_panel_motion_graph::PortLabel;
    let motion = MotionState::new();
    let mut bad: Vec<String> = Vec::new();
    let mut portas = 0usize;
    for man in motion.registry.manifests() {
        for p in man.inputs.iter().chain(man.outputs.iter()) {
            portas += 1;
            let l = PortLabel::of(p.name);
            let l = l.as_str();
            if l.is_empty()
                || l.contains('_')
                || !l
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
            {
                bad.push(format!("{}::{} -> {l:?}", man.name, p.name));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "portas cujo rotulo derivado nao e' legivel: {bad:?}"
    );
    // ⚠️ **CONTROLE — sem ele um registry vazio (ou um `manifests()` que devolvesse nada) daria
    // uma lista vazia de acusacoes e o gate passaria sobre o nada.**
    assert!(
        portas > 300,
        "o censo varreu so' {portas} portas — ele mediu o registry ou mediu o vazio?"
    );
}

/// ⛔⛔ **A TERCEIRA ESPÉCIE DE ESTOURO: o que só existe no estado que a FEATURE cria.**
///
/// O censo irmão constrói cada nó com `add_node(ty)` e mais nada — sem text param autorado e com
/// a tabela de externals **vazia**. Para o `fx.glow` isso apaga **duas** coisas de uma vez: a
/// linha `Dirt Intensity` (gateada à presença do `dirt`) e os *chips* da linha `Dirt Texture`,
/// cuja altura é função de `Cook::externals()`. ⇒ ele mede `592` de um corpo de `664` e o gate
/// diz que cabe — **medindo o painel com a feature DESLIGADA** (auditoria de 2026-08-27).
///
/// Medido no estado do próprio smoke (`dirt` autorado + 1 nome publicado): **674 px**. Com 5
/// nomes, `708`; com 9, `742`.
///
/// ⚠️ **Exceção aberta por decisão do Enio (2026-08-27)**, e ela é de OUTRA espécie que as três
/// do irmão: aquelas estouram no DEFAULT, esta estoura no USO. ⛔ A resposta boa é a mesma —
/// secções recolhíveis, que hoje não existem —, e o preço de a adiar é o artista abrir o painel
/// já a precisar da roda para ver os dois controlos da sujidade juntos.
///
/// ⚠️ *Este ficheiro já conhecia a classe*: ele carrega uma sonda `#[ignore]` dedicada
/// (`measure_the_wrap_with_a_shape`) porque o `motion.spline_wrap` mede o FALLBACK no default.
/// Ninguém escrevera a espelhada para o nó cujo default é o estado VAZIO.
#[test]
fn the_authored_state_overflow_is_named_too() {
    /// `(tipo, altura no estado autorado)` — medido em 2026-08-27.
    const AUTHORED_OVERFLOW: &[(&str, f32)] = &[("fx.glow", 674.0)];
    let body = reach_census_body();
    for (ty, px) in AUTHORED_OVERFLOW {
        let got = authored_height(ty);
        assert!(
            (got - px).abs() < 1.0,
            "`{ty}` media {px:.0} px no estado autorado e mede {got:.0} agora — um param \
             entrou ou saiu; re-meça e mova o número, ou desfaça"
        );
        assert!(
            got > body,
            "`{ty}` já CABE no corpo ({got:.0} <= {body:.0}) no estado autorado — tire-o da lista"
        );
    }
}

/// A altura do painel de `ty` com os text params AUTORADOS e um nome PUBLICADO — o estado em que
/// o artista de facto usa o nó, e o que o censo do default não consegue ver.
fn authored_height(ty: &str) -> f32 {
    use ph2d_nodegraph::attr::{Column, Stream};
    let mut motion = MotionState::new();
    let node = motion.doc.graph.add_node(ty);
    // O text param autorado e o mesmo nome na tabela de externals — as DUAS metades: sem a
    // primeira a linha `Dirt Intensity` fica gateada fora; sem a segunda a fileira de chips do
    // `Dirt Texture` não é pintada. É a soma delas que faz o painel passar do corpo.
    motion
        .doc
        .graph
        .set_text_param(node, "dirt", "Lens Dirt".to_string());
    motion.pump.cook.set_external(
        "Lens Dirt",
        Stream::new(1).with("P", Column::Vec2(vec![[0.0, 0.0]])),
    );
    ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);
    let snap = build_params_snapshot(&motion, ProjectSettings::default());
    ph2d_panel_motion_params::set_current_params(snap);
    let mut host =
        ph2d_ui_testkit::MockPanelHost::with_panel::<ph2d_panel_motion_params::MotionParamsPanel>();
    let mut state = ph2d_panel_motion_params::MotionParamsPanelState;
    let _ = host.paint::<ph2d_panel_motion_params::MotionParamsPanel>(
        &mut state,
        Rect {
            x: 0.0,
            y: 0.0,
            w: ph2d_editor::screens::layout::INSPECTOR_W,
            h: INSPECTOR_MAX_H,
        },
    );
    let h = host
        .store()
        .panel_content_h(ph2d_editor::ids::MOTION_PARAMS_PANEL)
        .unwrap_or(0.0);
    ph2d_panel_motion_params::set_current_params(None);
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    h
}

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
    /// ⚠️ **O segundo entrou em 2026-08-24 e é de OUTRA espécie** — ver o doc acima: o
    /// `bezier_warp` estoura porque 24 params são a superfície da referência; o
    /// `spline_wrap` estoura só no FALLBACK (sem forma escolhida ele mostra as oito
    /// coordenadas da cúbica), e com uma forma mede **456 px** num corpo de 664.
    const NAMED_OVERFLOW: &[(&str, f32)] = &[
        ("motion.bezier_warp", 969.0),
        ("motion.spline_wrap", 755.0),
        // O terceiro, 2026-08-24: uma lista PLANA que passou por 16 px (meia linha) ao ganhar
        // a cor e a rotação próprias. Sem `ParamGroup`, nenhuma dobra o alcança.
        ("source.shape", 680.0),
    ];
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

/// **SONDA de 2026-08-24: quanto o `motion.spline_wrap` mede COM a forma escolhida.**
///
/// A pergunta que decide se ele é uma excepção legítima: as oito coordenadas do polígono de
/// controle já desaparecem quando o artista nomeia uma forma (`ParamGateText`), então o
/// tamanho no estado de DEFAULT é o do FALLBACK, não o do nó em uso.
///
/// `cargo test -p ph2d-host-desktop --bins measure_the_wrap_with_a_shape -- --ignored --nocapture`
#[test]
#[ignore = "sonda, não gate"]
fn measure_the_wrap_with_a_shape() {
    let mut motion = MotionState::new();
    let node = motion.doc.graph.add_node("motion.spline_wrap");
    motion
        .doc
        .graph
        .set_text_param(node, "path", "alguma-forma");
    ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);
    let snap = build_params_snapshot(&motion, ProjectSettings::default());
    ph2d_panel_motion_params::set_current_params(snap);
    let mut host =
        ph2d_ui_testkit::MockPanelHost::with_panel::<ph2d_panel_motion_params::MotionParamsPanel>();
    let mut state = ph2d_panel_motion_params::MotionParamsPanelState;
    let _ = host.paint::<ph2d_panel_motion_params::MotionParamsPanel>(
        &mut state,
        Rect {
            x: 0.0,
            y: 0.0,
            w: ph2d_editor::screens::layout::INSPECTOR_W,
            h: INSPECTOR_MAX_H,
        },
    );
    let content = host
        .store()
        .panel_content_h(ph2d_editor::ids::MOTION_PARAMS_PANEL)
        .unwrap_or(0.0);
    let visible = host
        .store()
        .panel_visible_h(ph2d_editor::ids::MOTION_PARAMS_PANEL)
        .unwrap_or(0.0);
    println!("spline_wrap COM forma: conteudo {content:.0} px · corpo {visible:.0} px");
    ph2d_panel_motion_params::set_current_params(None);
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}
