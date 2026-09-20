//! ⭐⭐⭐ **O CENSO DO DEGRAU `G` — de quem são as entradas de cada painel.**
//!
//! # ⛔⛔ Porque ele existe: o degrau mais caro do plano tinha UM painel medido
//!
//! O `README.md` do módulo diz, desde 2026-09-04: *«a maior obra aberta: 1 painel de 25 censado.
//! O `3D Model` perdeu `17` das `74` entradas; ⛔ **nenhum outro foi medido** — o «66 de 74» é só
//! dele.»* ⇒ a obra que devolve mais tela deste app estava a ser escolhida **sem régua**.
//!
//! # ⚠️ A pergunta do degrau `G` não é «o painel é grande?»
//!
//! A decisão `D2` do dono (`docs/UI_New_and_Simple/00_DECISOES_DO_ENIO.md`) corta por **ÂMBITO**:
//! um comando do app vai à barra global, um comando do editor vai a um chip-pulldown da fila, e só
//! uma **propriedade do objecto escolhido** fica. ⇒ o Inspector é grande **por direito** — ele é um
//! painel de propriedades —, e um censo que só medisse TAMANHO poria-o no topo e mandaria a wave
//! para o sítio errado.
//!
//! ⇒ a grandeza é a **CARGA DE COMANDOS**: quantas entradas do painel são coisas que *fazem*, e
//! não coisas que *valem*.
//!
//! # ⭐ A régua sai do SUBSTRATO, não de uma lista escrita à mão
//!
//! [`WidgetStore::focus_order`] dá os ids registados e [`WidgetStore::get`] dá a espécie de cada
//! um. Um [`InteractiveState::Button`] é um **gesto** (não carrega valor nenhum); um `Slider`,
//! `NumberInput`, `Checkbox`, `Toggle`, `Dropdown`, `Combobox`, `TextInput`, `Radio` ou
//! `ColorPicker` é um **valor**. ⛔ Nada aqui é uma lista de nomes de painel.
//!
//! # ⭐⭐ O CONTROLO POSITIVO, e porque ele calibra as DUAS metades
//!
//! O `3D Model` é o único painel com a triagem feita, e os **17** que saíram dele em 2026-09-01
//! (`the_area_hands_its_commands_to_the_bar_and_the_app_menu`) eram **9 vistas + câmera**,
//! **5 verbos de gizmo** e **3 níveis de exportação** — ⭐ *todos botões*. É isso que prova que a
//! espécie `Button` é a proxy certa para «comando», e não uma escolha minha.
//!
//! # ⚠️ O que esta régua NÃO afirma, declarado
//!
//! Ela diz **onde olhar**, nunca dá o veredito de uma entrada. Um botão pode abrir o editor de uma
//! propriedade (o quadrado de cor), e um `Toggle` pode ser um comando disfarçado. ⇒ a triagem de
//! cada painel continua a ser trabalho de wave, com a tabela da `D2` na mão — como foi a do
//! `3D Model`. *Uma contagem que se lesse como veredito mandaria apagar controlos vivos.*
//!
//! # ⛔ E ele ASSERTA, não imprime
//!
//! *Uma tabela impressa que passa ninguém lê* (`CLAUDE.md` §5.0 — a lição das três paridades de
//! pixel que eram impressoras). A mensagem de falha traz a tabela inteira; o veredito é a catraca.

use ph2d_editor_core::NodeId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_ui_testkit::MockPanelHost;

/// ⭐⭐⭐ **A DOBRA — a altura de um encaixe real desta casa, `880 px`.**
///
/// ⛔ Não é escolhida: é o número que a `line/sculpt3d` mediu em 2026-09-19 ao responder ao dono
/// onde o botão do pente caía (`crates/ph2d-app-sculpt3d/src/scenes_pente.rs`, a tabela do `y`),
/// e é contra ele que aquela linha declara *«o painel está sobre o orçamento»*.
const DOBRA: f32 = 880.0;

/// ⛔⛔ **A viewport é ALTA de propósito: `4000 px`.**
///
/// A pergunta do degrau `G` é *«quantas coisas este painel põe à frente do artista?»*, e um painel
/// que não cabe **rola** — ele não deixa de ter as entradas. Numa viewport de ecrã o índice de
/// acerto só ficaria com as linhas visíveis, e o censo leria *«este painel é pequeno»* sobre
/// exactamente o painel que precisou de barra de rolagem (que foi o report do dono que abriu
/// este degrau, em 2026-08-27). ⇒ mede-se com ecrã a sobrar, e o número é o do painel, não o da
/// janela.
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1920.0,
    h: 4000.0,
};

/// ⭐⭐⭐ **A SEGUNDA ALTURA — o controlo que separa CONTEÚDO de CROMO ANCORADO.**
///
/// ⛔⛔ **A 1.ª redacção da coluna da altura mediu a JANELA em sete painéis** (2026-09-20): eles
/// leram `3 992` sobre uma viewport de `4 000`. A causa não é um fundo de altura inteira — isso já
/// estava curado pela espécie — é um painel que **ancora um controlo no FUNDO** do encaixe: ele
/// segue a janela por desenho, e um `max` sobre os rectângulos lê-o como conteúdo.
///
/// ⇒ mede-se **duas vezes**, e quem acompanha a janela declara-se **não medido**. *Uma régua de
/// altura sem um segundo ponto não distingue um painel alto de um painel esticado* — e as duas
/// leituras mandam a wave para sítios opostos.
const VIEWPORT_CURTA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1920.0,
    h: 2000.0,
};

/// A altura é do CONTEÚDO se ela **não** se moveu com a janela.
///
/// ⚠️ A folga de `1 px` é para o arredondamento de uma linha, não para tolerar deriva: as duas
/// viewports diferem `2 000 px`, logo um painel ancorado move-se `2 000`, não `1`.
fn altura_e_do_conteudo(alta: f32, curta: f32) -> bool {
    (alta - curta).abs() <= 1.0
}

/// O que um painel põe à frente do artista, partido por **quem é o dono** da entrada.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Contagem {
    /// ⭐ **Gestos** — o que a `D2` manda triar. Um [`InteractiveState::Button`].
    comandos: usize,
    /// **Valores** — o que um painel de propriedades é feito de. Fica, por definição.
    valores: usize,
    /// Navegação e cromo (abas, linhas de lista, superfícies, `Plain`). Não é nem uma coisa nem
    /// outra, e contá-lo com os comandos inflaria a dívida de todo painel com uma lista.
    outros: usize,
    /// ⚠️ Pintado e hit-indexado, **sem estado no store**. Ver [`conta`].
    orfaos: usize,
    /// ⭐⭐⭐ **Até onde o painel pinta**, em píxeis — o fundo do rectângulo mais baixo.
    ///
    /// É a grandeza que o DONO sente: o degrau `G` abriu com um report dele de 2026-08-27
    /// (*«este painel precisou de barra de rolagem»*), não com uma contagem. Contra a
    /// [`DOBRA`], ela diz quanto do painel está **fora do ecrã**.
    altura: f32,
}

impl Contagem {
    fn total(&self) -> usize {
        self.comandos + self.valores + self.outros + self.orfaos
    }
}

/// A espécie de uma entrada, pelo substrato.
fn classifica(c: &mut Contagem, s: &InteractiveState) {
    match s {
        InteractiveState::Button { .. } => c.comandos += 1,
        InteractiveState::Toggle { .. }
        | InteractiveState::Slider { .. }
        | InteractiveState::Checkbox { .. }
        | InteractiveState::Radio { .. }
        | InteractiveState::Dropdown { .. }
        | InteractiveState::Combobox { .. }
        | InteractiveState::TextInput { .. }
        | InteractiveState::NumberInput { .. }
        | InteractiveState::ColorPicker { .. }
        | InteractiveState::BlenderPicker { .. }
        | InteractiveState::CurvePoint { .. } => c.valores += 1,
        // ⚠️ **Tudo o resto é cromo, e a lista é explícita de propósito**: um `_ =>` mandaria uma
        //    espécie NOVA para «outros» em silêncio, e a pergunta *«ela faz ou vale?»* é a única
        //    coisa que este censo mede.
        InteractiveState::Tag { .. }
        | InteractiveState::Tabs { .. }
        | InteractiveState::ListItem { .. }
        | InteractiveState::TreeView { .. }
        | InteractiveState::BlenderHit { .. }
        | InteractiveState::GraphSurface { .. }
        | InteractiveState::TimelineSurface { .. }
        | InteractiveState::FlipStripSurface { .. }
        | InteractiveState::Modal { .. }
        | InteractiveState::Plain => c.outros += 1,
    }
}

/// ⭐⭐⭐ **O que o painel PÔS NO ECRÃ**, cruzado com a espécie que o store guarda.
///
/// ⛔⛔ **Registar e pintar são grandezas diferentes, e a diferença é `12×`.** O `populate` semeia
/// tudo o que o painel *poderia* mostrar (todos os modos, todas as secções condicionais); o
/// `paint` desenha o subconjunto do estado actual. Medido: o `3D Model` **regista `912`** e a
/// triagem da `D2` contou **`74`** — ⭐ e foi o CONTROLO POSITIVO deste ficheiro que apanhou a 1.ª
/// redacção a medir o catálogo e a chamar-lhe ecrã.
fn conta(store: &ph2d_editor_core::interaction::WidgetStore, pintados: &[(NodeId, Rect)]) -> Contagem {
    let mut c = Contagem::default();
    for (id, r) in pintados {
        match store.get(*id) {
            Some(s) => {
                classifica(&mut c, s);
                // ⛔⛔⛔ **A ALTURA MEDE-SE SÓ ONDE HÁ CONTROLO, e a 1.ª redacção não o fazia.**
                //
                // Medido em 2026-09-20: a versão que somava TODO rectângulo pintado lia `3 992`
                // para sete painéis — que é a **viewport** — e `14 987` para o Inspector. A causa
                // é um painel pintar um FUNDO de altura inteira (a pista de uma barra de rolagem,
                // o fundo de uma lista): ele é hit-indexado como qualquer coisa, e o `max` passa a
                // medir a janela. ⇒ *uma régua de altura sem controlo positivo mede a moldura e
                // chama-lhe conteúdo.*
                //
                // ⚠️ **O discriminador é a espécie**, que este censo já tem: um `Button`, um
                // `Slider` ou um `NumberInput` é conteúdo; um `Plain`, uma superfície ou um órfão
                // é cromo. ⭐ E é a mesma partição que a coluna dos comandos usa — *uma régua com
                // duas perguntas e uma só classificação não pode divergir entre elas.*
                if !matches!(
                    s,
                    InteractiveState::Plain
                        | InteractiveState::GraphSurface { .. }
                        | InteractiveState::TimelineSurface { .. }
                        | InteractiveState::FlipStripSurface { .. }
                        | InteractiveState::Modal { .. }
                        | InteractiveState::TreeView { .. }
                ) {
                    // ⚠️ **O fundo do rectângulo, não o topo** — um controlo alto que começa acima
                    //    da dobra pode acabar abaixo dela.
                    c.altura = c.altura.max(r.y + r.h);
                }
            }
            // ⚠️ Pintado e hit-indexado **sem estado no store** — o *órfão* do `CLAUDE.md` §5.0.
            //    ⛔ Ele não é um controlo morto: as curas são opostas (um apaga-se, o outro liga-se),
            //    e somá-lo aos comandos inflaria a dívida de quem pinta cabeçalhos de secção.
            None => c.orfaos += 1,
        }
    }
    c
}

/// Uma linha do censo.
#[derive(Debug, Clone, Copy)]
struct Linha {
    painel: &'static str,
    /// O painel como ele abre, sem documento nenhum na mão.
    vazio: Contagem,
    /// ⭐ Com um documento na mão, para os que a [`super::paineis_armados::TABELA`] sabe armar.
    armado: Option<Contagem>,
    /// ⭐ A mesma altura, na [`VIEWPORT_CURTA`] — o controlo. Ver [`altura_e_do_conteudo`].
    altura_curta: f32,
}

impl Linha {
    /// ⚠️ **O que o artista vê é o MAIOR dos dois** — um painel que só enche com um objecto
    /// escolhido enche na mesma. Medir só o vazio leria o Inspector como um painel pequeno.
    fn cheia(&self) -> Contagem {
        match self.armado {
            Some(a) if a.total() > self.vazio.total() => a,
            _ => self.vazio,
        }
    }
}

/// Varre **todo** painel do registo e conta o que cada um regista, por espécie.
fn censo() -> Vec<Linha> {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut out = Vec::new();
    ph2d_editor_core::panel::with_registry(|reg| {
        for painel in reg.panels_mut() {
            let id = painel.manifest.id;

            let mut host = MockPanelHost::new();
            painel.populate(host.store_mut());
            let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
            let pintados = host.registos_da_ultima_pintura();
            let vazio = conta(host.store(), &pintados);

            // ⚠️ **Armar vem ANTES do `populate`**, e não é ordem de conveniência: as `populate_*`
            // das secções condicionais semeiam-se da informação publicada, logo um `populate`
            // corrido antes veria o painel vazio (a mesma lei que a varredura das elisões paga).
            let armado = super::paineis_armados::TABELA
                .iter()
                .find(|a| a.painel == id)
                .map(|arm| {
                    let mut host = MockPanelHost::new();
                    (arm.arma)(host.store_mut());
                    painel.populate(host.store_mut());
                    let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
                    let pintados = host.registos_da_ultima_pintura();
                    let c = conta(host.store(), &pintados);
                    // ⛔ O estado que uma fixtura deixa para trás é o estado que a régua seguinte
                    //    mede — estas portas são `thread_local`.
                    (arm.desarma)();
                    c
                });

            // ⭐ **O CONTROLO**: a mesma pintura numa janela `2 000 px` mais baixa. Corre-se sempre
            //   no estado VAZIO — o que se pergunta é se o painel ANCORA, e isso não depende de
            //   haver um documento na mão.
            let mut host = MockPanelHost::new();
            painel.populate(host.store_mut());
            let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT_CURTA);
            let curta = conta(host.store(), &host.registos_da_ultima_pintura()).altura;

            out.push(Linha {
                painel: id,
                vazio,
                armado,
                altura_curta: curta,
            });
        }
    });
    // ⭐ Ordenado pela grandeza da `D2` — a carga de COMANDOS —, não pelo tamanho.
    out.sort_by(|a, b| {
        b.cheia()
            .comandos
            .cmp(&a.cheia().comandos)
            .then(b.cheia().total().cmp(&a.cheia().total()))
            .then(a.painel.cmp(b.painel))
    });
    out
}

fn tabela(linhas: &[Linha]) -> String {
    let mut s = String::from(
        "\n  painel                     comandos  valores  cromo  orfaos  total   altura  fora-da-dobra\n",
    );
    for l in linhas {
        let c = l.cheia();
        let fora = c.altura - DOBRA;
        s.push_str(&format!(
            "  {:<26} {:>8}  {:>7}  {:>5}  {:>6}  {:>5}  {:>7.0}  {}\n",
            l.painel,
            c.comandos,
            c.valores,
            c.outros,
            c.orfaos,
            c.total(),
            c.altura,
            if !altura_e_do_conteudo(l.vazio.altura, l.altura_curta) {
                // ⛔ Ele ancora no fundo: a leitura é da JANELA. Ver [`VIEWPORT_CURTA`].
                format!("(ancora — segue a janela: {:.0} vs {:.0})", l.vazio.altura, l.altura_curta)
            } else if fora > 0.0 {
                format!("+{fora:.0} px  ({:.0}%)", 100.0 * fora / DOBRA)
            } else {
                "cabe".into()
            },
        ));
    }
    s.push_str(&format!(
        "  {:<26} {:>8}  {:>7}  {:>5}  {:>6}  {:>5}   (dobra = {DOBRA:.0} px)\n",
        "TOTAL",
        linhas.iter().map(|l| l.cheia().comandos).sum::<usize>(),
        linhas.iter().map(|l| l.cheia().valores).sum::<usize>(),
        linhas.iter().map(|l| l.cheia().outros).sum::<usize>(),
        linhas.iter().map(|l| l.cheia().orfaos).sum::<usize>(),
        linhas.iter().map(|l| l.cheia().total()).sum::<usize>(),
    ));
    s
}

/// ⭐⭐⭐ **A SONDA** — corra-a para ver a tabela; ela nunca reprova.
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-panel-registry-init \
///   --features panel-painter-layers,panel-flip,panel-flip-frames,panel-wet-tuning \
///   --test it -- quantas_entradas --nocapture
/// ```
///
/// ⛔⛔ **AS QUATRO FEATURES NÃO SÃO DECORAÇÃO.** O `default` desta crate regista **24** painéis e
/// o app corre com **28** (`shells/desktop/Cargo.toml`) — sem elas o censo lê `0` para quatro
/// painéis, que se lê como *«este painel não tem comandos»* em vez de *«este painel não existe
/// aqui»*. *Os dois lêem-se igual num número* — é a mesma armadilha que a varredura das elisões
/// pagou em 2026-09-20, e é por isso que o teste abaixo tem piso de população.
#[test]
fn imprime_o_censo() {
    println!("{}", tabela(&censo()));
}

/// ⭐⭐⭐ **O CONTROLO POSITIVO: o painel cuja triagem já foi feita reproduz o número dela.**
///
/// ⛔⛔ Sem isto a tabela de cima é um número sem unidade. O `3D Model` tinha `74` entradas e
/// perdeu `17` para a fila e para o menu *File* em 2026-09-01 ⇒ ~`57`.
///
/// ⚠️ **A banda é larga de propósito**: o `populate` regista também o que a tabela da `D2` não
/// contava como «entrada» (barras, âncoras de secção). O que se afirma é a ORDEM DE GRANDEZA — se
/// esta régua lesse `9` ou `200`, ela não estaria a medir o mesmo painel.
#[test]
fn o_painel_ja_triado_reproduz_o_numero_da_triagem() {
    let linhas = censo();
    let m3d = linhas
        .iter()
        .find(|l| l.painel == "model3d")
        .expect("o painel `3D Model` tem de estar no registo");
    let n = m3d.cheia().total();
    assert!(
        (40..=90).contains(&n),
        "o controlo da régua falhou: o `3D Model` lê {n} entradas, e a triagem de 2026-09-01 \
         deixou-o em ~57 (74 − 17). Ou o painel mudou, ou esta régua deixou de medir o que \
         mede.{}",
        tabela(&linhas),
    );
}

/// ⛔⛔ **O PISO DE POPULAÇÃO — o censo recusa o âmbito pobre.**
///
/// Ver a nota do [`imprime_o_censo`]: uma corrida `-p` sem as quatro features mede **24** painéis
/// e o app tem **28**. Sem este piso, quatro painéis liam `0` comandos e a wave seguinte ia para o
/// painel errado.
#[test]
fn o_censo_recusa_o_ambito_pobre() {
    let linhas = censo();
    // ⭐⭐ **A régua é o NOME e não a contagem.** A 1.ª redacção deste gate pedia `>= 28` — eu
    //    contara `ls crates/ph2d-panel-*` e incluíra a própria crate do REGISTO, que não é um
    //    painel. ⚠️ E um número não diz QUAIS: `27` painéis com os quatro errados lê-se igual a
    //    `27` com os certos. ⇒ nomeiam-se os que a `default` desta crate **não** liga.
    for esperado in ["painter_layers", "flip", "flip_frames", "wet_tuning"] {
        assert!(
            linhas.iter().any(|l| l.painel == esperado),
            "o censo não viu o painel `{esperado}` — ele chega pelo `shells/desktop` e a \
             `default` desta crate não o liga. Corra-o com \
             `--features panel-painter-layers,panel-flip,panel-flip-frames,panel-wet-tuning`, \
             senão quatro painéis lêem `0` comandos e isso lê-se como *«não tem comandos»* em vez \
             de *«não existe aqui»*.{}",
            tabela(&linhas),
        );
    }
    // ⚠️ O piso da população fica ao lado, a ver o resto: `27` é o mesmo número do
    //    `PISO_DE_PAINEIS` da varredura das elisões, medido no mesmo âmbito.
    assert!(
        linhas.len() >= 27,
        "o censo viu {} painéis e o registo tem 27.{}",
        linhas.len(),
        tabela(&linhas),
    );
}

/// ⛔⛔ **A CEGUEIRA DECLARADA: o `motion_graph` não é medível por este arnês.**
///
/// Ele vive no **split do centro**, e o arnês monta o encaixe com `CenterSplit::None` — ali o
/// painel recebe um rect de área zero e **devolve antes de desenhar** (está escrito no
/// `paint_with_layout` do testkit). ⇒ ele lê `0` em todas as colunas, e `0` de *«não foi medido»*
/// e `0` de *«não tem comandos»* são o mesmo byte.
///
/// ⚠️ Este teste existe para a cegueira ser **consultável** em vez de uma linha a zero que alguém
/// lê como um painel limpo. No dia em que o arnês souber montar o split, ele reprova e a nota
/// morre à vista no diff.
#[test]
fn a_cegueira_do_painel_do_centro_esta_nomeada() {
    let linhas = censo();
    let mg = linhas
        .iter()
        .find(|l| l.painel == "motion_graph")
        .expect("o painel do grafo tem de estar no registo");
    assert_eq!(
        mg.cheia().total(),
        0,
        "o `motion_graph` passou a ser medível — o arnês aprendeu a montar o split do centro. \
         Apague esta cegueira e conte-o com os outros.{}",
        tabela(&linhas),
    );
}
