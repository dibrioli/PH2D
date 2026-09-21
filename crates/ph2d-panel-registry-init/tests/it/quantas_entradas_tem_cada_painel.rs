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
pub(super) const DOBRA: f32 = 880.0;

/// ⛔⛔ **A viewport é ALTA de propósito: `4000 px`.**
///
/// A pergunta do degrau `G` é *«quantas coisas este painel põe à frente do artista?»*, e um painel
/// que não cabe **rola** — ele não deixa de ter as entradas. Numa viewport de ecrã o índice de
/// acerto só ficaria com as linhas visíveis, e o censo leria *«este painel é pequeno»* sobre
/// exactamente o painel que precisou de barra de rolagem (que foi o report do dono que abriu
/// este degrau, em 2026-08-27). ⇒ mede-se com ecrã a sobrar, e o número é o do painel, não o da
/// janela.
pub(super) const VIEWPORT: Rect = Rect {
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
    /// ⭐ **Células de composto que foram COLAPSADAS** — ver [`conta`]. Elas não entram no
    /// [`Contagem::total`] (um selector é UM controlo), e ficam guardadas porque a triagem que o
    /// dono fez em 2026-09-01 contou **alvos de toque**: sem este número a régua nova não
    /// consegue reproduzir o dele, e o controlo dela morre.
    celulas: usize,
    /// Quantos grupos de composto foram contados — o par do [`Contagem::celulas`].
    grupos: usize,
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

    /// ⭐ **A grandeza da triagem de 2026-09-01: ALVOS DE TOQUE.** Cada célula de um composto
    /// conta por si, que é o que o dono contou quando triou o `3D Model` em `74`.
    ///
    /// ⚠️ Ela existe **só** para o controlo da régua: a `D2` pergunta *«esta coisa no ecrã é um
    /// comando ou uma propriedade?»*, e um selector é UMA coisa. *Duas grandezas com o mesmo nome
    /// é como uma régua deixa de ser comparável consigo própria.*
    fn alvos(&self) -> usize {
        self.total() - self.grupos + self.celulas
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
fn conta(
    store: &ph2d_editor_core::interaction::WidgetStore,
    pintados: &[(NodeId, Rect)],
    grupos: &[Vec<NodeId>],
) -> Contagem {
    let mut c = Contagem::default();

    // ⭐⭐⭐ **UM CONTROLO COMPOSTO É UM, e isto desmentiu o número que ordenava esta lista.**
    //
    // Um selector de N opções e uma máscara de 32 bits são pintados como N `Button`, logo a
    // classificação por SUBSTRATO contava-os como N **comandos**. Medido em 2026-09-21 no
    // Inspector: dos `60` do bloco base, `32` eram as camadas de colisão (UMA grelha), `9` os
    // tipos de junta (UMA escolha) e `~14` outros selectores — **`5` eram comandos a sério**.
    // ⇒ *o painel que mais usa selectores liderava a dívida por causa disso*, e a `D2` teria
    // mandado uma máscara de bits para a barra do topo.
    //
    // ⚠️ **Quem sabe é QUEM PINTA**, e não a fonte: a regra barata (*«array = célula, escalar =
    // comando»*) foi medida e falha nos DOIS sentidos — `INSP_ORDER_SP_*` são três escalares que
    // formam um selector, e `INSP_INSTANCE_DROP_ORPHAN` é um array que é uma LISTA. ⇒ os pintores
    // canónicos declaram o grupo ([`ph2d_editor_core::widget::composto`]).
    //
    // ⛔ E ele conta como **VALOR**: escolher uma opção é dizer QUANTO, não FAZER.
    let mut celula: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();
    for g in grupos {
        // ⚠️ Só conta o grupo que de facto foi PINTADO neste estado — um selector registado e
        //    não desenhado não está no ecrã, e esta régua mede o ecrã.
        if g.iter().any(|id| pintados.iter().any(|(p, _)| p == id)) {
            c.valores += 1;
            c.grupos += 1;
        }
        for id in g {
            celula.insert(id.0);
        }
    }

    for (id, r) in pintados {
        if celula.contains(&id.0) {
            // A altura de uma célula continua a contar — o controlo ocupa o ecrã.
            c.altura = c.altura.max(r.y + r.h);
            c.celulas += 1;
            continue;
        }
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
            // ⭐ A pintura corre DENTRO do censo dos compostos — ver [`conta`].
            let (_, grupos) = ph2d_editor_core::widget::composto::medindo(|| {
                let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
            });
            let pintados = host.registos_da_ultima_pintura();
            let vazio = conta(host.store(), &pintados, &grupos);

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
                    let (_, grupos) = ph2d_editor_core::widget::composto::medindo(|| {
                        let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
                    });
                    let pintados = host.registos_da_ultima_pintura();
                    let c = conta(host.store(), &pintados, &grupos);
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
            let (_, grupos_curta) = ph2d_editor_core::widget::composto::medindo(|| {
                let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT_CURTA);
            });
            let curta = conta(
                host.store(),
                &host.registos_da_ultima_pintura(),
                &grupos_curta,
            )
            .altura;

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
            "  {:<26} {:>8}  {:>7}  {:>5}  {:>6}  {:>5}  {:>7.0}  {}{}\n",
            l.painel,
            c.comandos,
            c.valores,
            c.outros,
            c.orfaos,
            c.total(),
            c.altura,
            if l.armado.is_some() {
                // ⛔⛔ **As duas leituras, nunca o `max` sozinho.** A armação de um painel é o
                //    estado MÁXIMO dele e não o do dia a dia: medido em 2026-09-20, a fixtura da
                //    escultura arma o FILTRO, e as catorze fichas dele são condicionais
                //    (`if !snap.filter_armed { return }`) — escondê-las atrás de um `max` daria
                //    `+230 px` a um painel que o artista raramente vê assim. *Uma coluna que
                //    colapsa dois estados num número descreve um app que ninguém usa.*
                format!(
                    "vazio {:.0} · armado {:.0}  ",
                    l.vazio.altura,
                    l.armado.map(|a| a.altura).unwrap_or(0.0)
                )
            } else {
                String::new()
            },
            if !altura_e_do_conteudo(l.vazio.altura, l.altura_curta) {
                // ⛔ Ele ancora no fundo: a leitura é da JANELA. Ver [`VIEWPORT_CURTA`].
                format!(
                    "(ancora — segue a janela: {:.0} vs {:.0})",
                    l.vazio.altura, l.altura_curta
                )
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
    let c = m3d.cheia();
    // ⚠️⚠️ **A PREMISSA deste gate mudou em 2026-09-21, e as duas metades dizem porquê.**
    //
    // Ele comparava o `total()` com os `~57` da triagem de 2026-09-01 e passou a ler `21`, porque
    // a régua aprendeu que **um controlo COMPOSTO é um** (as células de um selector deixaram de
    // contar uma a uma). ⛔ Alargar a banda seria matar o controlo: ele existe para dizer que esta
    // régua ainda vê o painel.
    //
    // ⇒ a metade velha fica INTACTA sobre a grandeza que a triagem usou — **alvos de toque** — e
    // a metade nova afirma a diferença, que é a razão de a wave existir. *Uma régua que troca de
    // grandeza tem de conseguir reproduzir a antiga, senão ninguém consegue saber se ela melhorou
    // ou se se partiu.*
    let alvos = c.alvos();
    assert!(
        (40..=90).contains(&alvos),
        "o controlo da régua falhou: o `3D Model` lê {alvos} ALVOS DE TOQUE, e a triagem de \
         2026-09-01 deixou-o em ~57 (74 − 17). Ou o painel mudou, ou esta régua deixou de medir o \
         que mede.{}",
        tabela(&linhas),
    );
    assert!(
        c.total() < alvos,
        "o `3D Model` lê o mesmo nas duas grandezas ({} controlos contra {alvos} alvos) — então \
         ou ele deixou de ter um único selector, ou os pintores canónicos deixaram de declarar o \
         grupo e a régua voltou a contar célula a célula.{}",
        c.total(),
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

/// ⭐⭐⭐ **ONDE CAI CADA SECÇÃO DO PAINEL DA ESCULTURA** — a sonda que dimensiona a wave do `G`.
///
/// ⛔⛔ **Ela existe porque a `line/sculpt3d` já mediu esta tabela e eu não posso usar o número
/// dela.** O `scenes_pente.rs` daquela crate tem os `y` de cada secção contra a [`DOBRA`], medidos
/// **na app a correr**; este censo mede num arnês. *Misturar dois instrumentos numa conta é a
/// forma exacta de fabricar uma medição* — e a diferença entre os dois é a resposta à pergunta
/// *«quanto é que esta wave devolve?»*, que é o que decide se ela vale a pena.
///
/// ⚠️ Ela **não reprova**: é uma sonda. Quem a lê é quem for fazer a triagem.
#[test]
fn diag_onde_caem_as_seccoes_da_escultura() {
    use ph2d_panel_sculpt3d::ids as sid;

    // ⭐ As sete secções, pelo nome que o artista vê. ⛔ A ordem aqui é a da TABELA e não a do
    //   ecrã — é exactamente isso que a sonda vai desmentir, e a `line/sculpt3d` já pagou essa
    //   leitura uma vez (*«a ordem da tela lê-se do `y`, nunca da tabela `SECTIONS`»*).
    let seccoes: [(&str, NodeId); 7] = [
        ("Tool", sid::SCULPT3D_SEC_TOOL),
        ("Brush", sid::SCULPT3D_SEC_BRUSH),
        ("Symmetry", sid::SCULPT3D_SEC_SYMMETRY),
        ("Topology", sid::SCULPT3D_SEC_TOPOLOGY),
        ("Shading", sid::SCULPT3D_SEC_SHADING),
        ("Scene", sid::SCULPT3D_SEC_SCENE),
        ("Bake", sid::SCULPT3D_SEC_BAKE),
    ];

    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_editor_core::panel::with_registry(|reg| {
        let painel = reg
            .panels_mut()
            .iter_mut()
            .find(|p| p.manifest.id == "sculpt3d")
            .expect("o painel da escultura tem de estar no registo");
        let arm = super::paineis_armados::TABELA
            .iter()
            .find(|a| a.painel == "sculpt3d")
            .expect("a escultura tem armação");

        // ⛔⛔ **DOIS estados, e não um.** A armação pinta o estado MÁXIMO de propósito (nível
        //    `Pro`, filtro ARMADO) — é o que faz uma régua de LARGURA ver todos os rótulos. Medir a
        //    ALTURA ali lê o **pior caso**: as catorze fichas do filtro valem `~221 px` e só são
        //    pintadas depois de o artista armar o filtro. *Uma coluna que colapsa dois estados num
        //    número descreve um app que ninguém usa.*
        for (nome_do_estado, arma) in [
            ("PIOR CASO (Pro, filtro armado)", arm.arma),
            (
                "DIA A DIA (Basic, filtro desarmado)",
                (|store: &mut ph2d_editor_core::interaction::WidgetStore| {
                    let _ = store;
                    super::o_sculpt3d_armado::arma_o_dia_a_dia();
                }) as fn(&mut ph2d_editor_core::interaction::WidgetStore),
            ),
        ] {
            let mut host = MockPanelHost::new();
            (arma)(host.store_mut());
            painel.populate(host.store_mut());
            let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
            let pintados = host.registos_da_ultima_pintura();
            (arm.desarma)();

            let mut linhas: Vec<(f32, &str)> = seccoes
                .iter()
                .filter_map(|(nome, id)| {
                    pintados
                        .iter()
                        .find(|(pid, _)| pid == id)
                        .map(|(_, r)| (r.y, *nome))
                })
                .collect();
            linhas.sort_by(|a, b| a.0.total_cmp(&b.0));

            let fundo = conta(host.store(), &pintados, &[]).altura;
            println!(
                "\n  === o painel da ESCULTURA — {nome_do_estado} (dobra = {DOBRA:.0} px) ==="
            );
            let mut anterior: Option<(f32, &str)> = None;
            for (y, nome) in &linhas {
                if let Some((ya, na)) = anterior {
                    println!("      {na:<12} ocupa {:>6.0} px", y - ya);
                }
                println!(
                    "  {:>6.0}  {nome:<12} {}",
                    y,
                    if *y > DOBRA {
                        "⛔ fora do ecrã"
                    } else {
                        "visível"
                    }
                );
                anterior = Some((*y, nome));
            }
            if let Some((ya, na)) = anterior {
                println!("      {na:<12} ocupa {:>6.0} px", fundo - ya);
            }
            println!("  {fundo:>6.0}  (fim do conteúdo)\n");
        }
    });
}

/// ⭐⭐⭐ **O QUE COME OS `614 px` DA SECÇÃO `Tool`** — a sonda que impede a cura errada.
///
/// ⛔⛔ **Ela existe porque eu quase propus cortar a coisa errada.** A secção `Tool` mede `614 px`
/// e tem `38` chips de VERBO, e a conta de cabeça («38 chips ⇒ 614 px») **não fecha**: `38` chips
/// numa coluna de `~300 px` são `~9` fileiras, `~216 px`. ⇒ o resto são as **outras 24 famílias de
/// chips** daquele painel (o modo da pose, os oito do pano, os seis do contorno, os nove do
/// filtro…), que aparecem conforme o verbo na mão.
///
/// *Uma cura desenhada sobre a família que eu já tinha na cabeça teria devolvido um terço do que
/// promete* — e a régua que a impede é esta.
#[test]
fn diag_o_que_come_a_seccao_tool_da_escultura() {
    use ph2d_panel_sculpt3d::ids as sid;

    // ⚠️ **As famílias são NOMEADAS, e a lista é a do ficheiro de ids** — não um `grep` meu. Uma
    //    família nova que não esteja aqui aparece na linha `(sem família nomeada)`, que é o que
    //    impede esta sonda de mentir por omissão.
    let familias: &[(&str, &[NodeId])] = &[
        ("VERB", &sid::SCULPT3D_VERB),
        ("FALLOFF", &sid::SCULPT3D_FALLOFF),
        ("ALPHA", &sid::SCULPT3D_ALPHA),
        ("MATCAP", &sid::SCULPT3D_MATCAP),
        ("FILTER_KIND", &sid::SCULPT3D_FILTER_KIND),
        ("CLOTH_MODE", &sid::SCULPT3D_CLOTH_MODE),
        ("BOUNDARY_MODE", &sid::SCULPT3D_BOUNDARY_MODE),
        ("POSE_MODE", &sid::SCULPT3D_POSE_MODE),
        ("CLOTH_FILTER_KIND", &sid::SCULPT3D_CLOTH_FILTER_KIND),
        ("REF_MODE", &sid::SCULPT3D_REF_MODE),
        ("ADD", &sid::SCULPT3D_ADD),
        ("MASK_OP", &sid::SCULPT3D_MASK_OP),
        ("TRANSFORM", &sid::SCULPT3D_TRANSFORM),
        ("BOUNDARY_FALLOFF", &sid::SCULPT3D_BOUNDARY_FALLOFF),
        ("ELASTIC_SCALES", &sid::SCULPT3D_ELASTIC_SCALES),
        ("CLOTH_AREA", &sid::SCULPT3D_CLOTH_AREA),
        ("CFILTER_AXIS", &sid::SCULPT3D_CFILTER_AXIS),
        ("TRIM_FORMA", &sid::SCULPT3D_TRIM_FORMA),
        ("SMEAR_MODE", &sid::SCULPT3D_SMEAR_MODE),
        ("UI_LEVEL", &sid::SCULPT3D_UI_LEVEL),
        ("RETOPO_MODE", &sid::SCULPT3D_RETOPO_MODE),
        ("PROJECT_MODE", &sid::SCULPT3D_PROJECT_MODE),
        ("PLANO_INVERSAO", &sid::SCULPT3D_PLANO_INVERSAO),
        ("CLOTH_FORCE_FALLOFF", &sid::SCULPT3D_CLOTH_FORCE_FALLOFF),
        ("CLOTH_FILTER_ORIENT", &sid::SCULPT3D_CLOTH_FILTER_ORIENT),
    ];

    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_editor_core::panel::with_registry(|reg| {
        let painel = reg
            .panels_mut()
            .iter_mut()
            .find(|p| p.manifest.id == "sculpt3d")
            .expect("o painel da escultura tem de estar no registo");
        let arm = super::paineis_armados::TABELA
            .iter()
            .find(|a| a.painel == "sculpt3d")
            .expect("a escultura tem armação");

        let mut host = MockPanelHost::new();
        (arm.arma)(host.store_mut());
        painel.populate(host.store_mut());
        let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
        let pintados = host.registos_da_ultima_pintura();
        (arm.desarma)();

        // Só o que cai DENTRO da secção `Tool` (entre o cabeçalho dela e o da `Brush`).
        let y_de = |id: NodeId| pintados.iter().find(|(p, _)| *p == id).map(|(_, r)| r.y);
        let (topo, fundo) = (
            y_de(sid::SCULPT3D_SEC_TOOL).unwrap_or(0.0),
            y_de(sid::SCULPT3D_SEC_BRUSH).unwrap_or(f32::MAX),
        );

        println!("\n  === a secção `Tool` ({topo:.0}..{fundo:.0}) por família de chips ===");
        let mut somado = 0usize;
        let mut linhas: Vec<(f32, String)> = Vec::new();
        for (nome, ids) in familias {
            let dentro: Vec<f32> = ids
                .iter()
                .filter_map(|id| y_de(*id))
                .filter(|y| *y >= topo && *y < fundo)
                .collect();
            if dentro.is_empty() {
                continue;
            }
            somado += dentro.len();
            let lo = dentro.iter().copied().fold(f32::MAX, f32::min);
            let hi = dentro.iter().copied().fold(f32::MIN, f32::max);
            linhas.push((
                lo,
                format!(
                    "  {lo:>6.0}..{hi:<6.0} {nome:<22} {:>3} chips",
                    dentro.len()
                ),
            ));
        }
        linhas.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (_, l) in &linhas {
            println!("{l}");
        }
        let total_na_seccao = pintados
            .iter()
            .filter(|(_, r)| r.y >= topo && r.y < fundo)
            .count();
        println!(
            "  → {somado} chips em famílias nomeadas, de {total_na_seccao} rectângulos na secção\n"
        );
    });
}

/// SONDA TEMPORÁRIA — de que SECÇÃO são as 707 entradas do Inspector.
#[test]
#[ignore]
fn diag_de_quem_sao_as_entradas_do_inspector() {
    use ph2d_tool_registry::hash_node_id_runtime;
    use std::collections::BTreeMap;
    let h = |s: &str| hash_node_id_runtime(s).0;

    // ⭐ O mapa `NodeId -> secção` é DERIVADO: cada secção do Inspector tem o SEU ficheiro de ids,
    //   e cada id nasce de `hash_node_id("<literal>")`. Nenhuma lista escrita à mão.
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../ph2d-panel-inspector/src/ids"
    );
    let mut nome: BTreeMap<u64, (String, String)> = BTreeMap::new();
    for e in std::fs::read_dir(dir).expect("os ids do inspector") {
        let p = e.expect("entrada").path();
        let Some(f) = p.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let seccao = f
            .trim_start_matches("inspector_")
            .trim_start_matches("inspector")
            .to_string();
        let seccao = if seccao.is_empty() {
            "(base)".to_string()
        } else {
            seccao
        };
        let src = std::fs::read_to_string(&p).expect("ler ids");
        for pedaco in src.split("hash_node_id(\"").skip(1) {
            if let Some(lit) = pedaco.split('"').next() {
                nome.insert(h(lit), (seccao.clone(), lit.to_string()));
            }
        }
    }
    println!("ids declarados por literal: {}", nome.len());

    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_editor_core::panel::with_registry(|reg| {
        let painel = reg
            .panels_mut()
            .iter_mut()
            .find(|p| p.manifest.id == "inspector")
            .expect("o inspector tem de estar no registo");
        let arm = super::paineis_armados::TABELA
            .iter()
            .find(|a| a.painel == "inspector")
            .expect("o inspector tem armação");
        let mut host = MockPanelHost::new();
        (arm.arma)(host.store_mut());
        painel.populate(host.store_mut());
        let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
        let pintados = host.registos_da_ultima_pintura();
        let store = host.store();

        let mut por_seccao: BTreeMap<String, Contagem> = BTreeMap::new();
        let mut sem_nome = 0usize;
        let mut orfaos: Vec<(String, String)> = Vec::new();
        for (id, _r) in &pintados {
            let (seccao, lit) = match nome.get(&id.0) {
                Some(v) => v.clone(),
                None => {
                    sem_nome += 1;
                    ("(id sem literal)".to_string(), String::new())
                }
            };
            let c = por_seccao.entry(seccao.clone()).or_default();
            match store.get(*id) {
                Some(s) => classifica(c, s),
                None => {
                    c.orfaos += 1;
                    orfaos.push((seccao, lit));
                }
            }
        }
        (arm.desarma)();

        // ⭐⭐ A ALTURA por secção — a grandeza que o dono SENTE (o degrau `G` abriu com um
        //    report dele sobre barra de rolagem, não com uma contagem).
        let mut faixa: BTreeMap<String, (f32, f32)> = BTreeMap::new();
        for (id, r) in &pintados {
            let sec = nome
                .get(&id.0)
                .map_or_else(|| "(id sem literal)".to_string(), |v| v.0.clone());
            let e = faixa.entry(sec).or_insert((f32::MAX, f32::MIN));
            e.0 = e.0.min(r.y);
            e.1 = e.1.max(r.y + r.h);
        }
        let altura = |s: &str| faixa.get(s).map_or(0.0, |f| f.1 - f.0);
        let mut linhas: Vec<(&String, &Contagem)> = por_seccao.iter().collect();
        linhas.sort_by(|a, b| altura(b.0).total_cmp(&altura(a.0)));
        println!(
            "\n  secção                comandos  valores  cromo  órfãos  total   altura  dobras"
        );
        for (s, c) in &linhas {
            let h = altura(s);
            println!(
                "  {:22} {:8} {:8} {:6} {:7} {:6} {:8.0} {:6.1}",
                s,
                c.comandos,
                c.valores,
                c.outros,
                c.orfaos,
                c.total(),
                h,
                h / DOBRA
            );
        }
        println!("\n  pintados sem literal conhecido: {sem_nome}");

        // ⭐ O QUE são os comandos das secções pesadas — a D2 corta por ÂMBITO, e o âmbito
        //   lê-se no nome do gesto, nunca na contagem.
        let mut cmds: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (id, _r) in &pintados {
            if let (
                Some((sec, lit)),
                Some(ph2d_editor_core::interaction::InteractiveState::Button { .. }),
            ) = (nome.get(&id.0), store.get(*id))
            {
                cmds.entry(sec.clone()).or_default().push(lit.clone());
            }
        }
        for sec in [
            "(base)",
            "physics_body",
            "tween",
            "joint",
            "path_follow",
            "anim",
            "sampling",
        ] {
            if let Some(v) = cmds.get(sec) {
                let mut v = v.clone();
                v.sort();
                println!(
                    "\n  [{sec}] {} comandos:\n    {}",
                    v.len(),
                    v.join("\n    ")
                );
            }
        }
        println!("\n  ÓRFÃOS (pintado + hit-indexado, SEM estado no store):");
        let mut por_sec: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (s, l) in orfaos {
            por_sec.entry(s).or_default().push(l);
        }
        for (s, mut ls) in por_sec {
            ls.sort();
            println!("   {s}: {} -> {}", ls.len(), ls.join(", "));
        }
    });
}

/// ⭐⭐⭐ **OS PINTORES CANÓNICOS DECLARAM O GRUPO — e desarmado não custa nada.**
///
/// ⛔⛔ Sem isto, um selector de N opções entra na dívida N vezes e uma máscara de 32 bits entra
/// 32 — foi assim que o Inspector chegou a `314` comandos quando tem `150`, e que o `3D Model`
/// leu `39` quando tem **`1`**. *A `D2` teria mandado uma máscara de bits para a barra do topo.*
///
/// ⚠️ **As duas metades são obrigatórias.** A primeira mede que o censo VÊ; a segunda que ele é
/// **mudo** quando ninguém o arma — sem ela, o caminho do produto passaria a alocar um `Vec` por
/// composto **por quadro**, que é o vazamento que o `leak_key` do `ph2d-i18n` já custou a esta casa.
#[test]
fn os_pintores_de_composto_declaram_o_grupo() {
    use ph2d_editor_core::widget::composto;

    let _ = ph2d_panel_registry_init::register_all_panels();
    let (grupos, soltos, contagem): (Vec<Vec<NodeId>>, usize, Contagem) =
        ph2d_editor_core::panel::with_registry(|reg| {
            let painel = reg
                .panels_mut()
                .iter_mut()
                .find(|p| p.manifest.id == "inspector")
                .expect("o inspector tem de estar no registo");
            let arm = super::paineis_armados::TABELA
                .iter()
                .find(|a| a.painel == "inspector")
                .expect("o inspector tem armação");

            let mut host = MockPanelHost::new();
            (arm.arma)(host.store_mut());
            painel.populate(host.store_mut());
            let (_, grupos) = composto::medindo(|| {
                let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
            });

            // ⭐ O CONTROLO: a MESMA pintura, com o censo DESARMADO.
            //
            // ⚠️⚠️ **A pergunta é «ACRESCENTOU?», não «está vazio?»** — e a 1.ª redacção perguntou a
            //    segunda e reprovou sobre produto correcto. O `grupos()` devolve *o que foi registado
            //    desde o `arma`*, e o `medindo` só esvazia ao ARMAR: depois dele a lista ainda tem os
            //    39 grupos da corrida armada, e lê-la a seguir a uma pintura desarmada mede a corrida
            //    anterior. *Um censo com memória mede-se pelo DELTA, nunca pelo valor.*
            // ⭐⭐ E o que o CENSO fez com eles — sem isto, uma mutação que declare o grupo e depois
            //    não o conte passa despercebida (medido: ela SOBREVIVEU à 1.ª redacção deste gate).
            let pintados = host.registos_da_ultima_pintura();
            let contagem = conta(host.store(), &pintados, &grupos);

            let antes = composto::grupos().len();
            let mut host2 = MockPanelHost::new();
            (arm.arma)(host2.store_mut());
            painel.populate(host2.store_mut());
            let _ = host2.medindo_a_pintura_do_registo(painel, VIEWPORT);
            let depois = composto::grupos().len();
            (arm.desarma)();
            (grupos, depois - antes, contagem)
        });

    assert_eq!(
        soltos, 0,
        "o censo registou {soltos} grupos com o produto DESARMADO — o caminho do produto está a \
         pagar uma alocação por composto, por quadro"
    );

    // ⭐⭐⭐ **E um grupo conta como UM VALOR.** ⛔ Sem esta metade, um censo que declare os
    //    grupos e depois os conte como ZERO passa — e o painel some da dívida em vez de aparecer
    //    com o número certo. *Declarar e CONTAR são duas coisas, e a mutação provou-o.*
    let pintados_em_grupo = grupos.iter().filter(|g| !g.is_empty()).count();
    assert!(
        contagem.grupos > 0 && contagem.grupos <= pintados_em_grupo,
        "o censo contou {} grupos de {pintados_em_grupo} declarados — ou ele deixou de os contar, \
         ou está a contar grupos que ninguém declarou",
        contagem.grupos
    );
    assert!(
        contagem.valores >= contagem.grupos,
        "o censo contou {} grupos e só {} valores — um selector é um VALOR, e se ele não entrar \
         ali o painel encolhe na dívida sem uma linha ter mudado",
        contagem.grupos,
        contagem.valores
    );

    // ⭐ A MÁSCARA: as 32 camadas de colisão são UM controlo. ⛔ O número é o do modelo
    //   (`BitmaskGrid32` tem 32 células por construção), nunca um limiar escolhido.
    assert!(
        grupos.iter().any(|g| g.len() == 32),
        "nenhum grupo de 32 células — a grelha de bits das camadas de colisão deixou de se \
         declarar, e ela sozinha põe 32 comandos na dívida deste painel. Grupos: {:?}",
        grupos.iter().map(Vec::len).collect::<Vec<_>>()
    );
    // ⭐ E as FILEIRAS SEGMENTADAS: o Inspector tem-nas às dúzias (tipo de junta, recorte,
    //   máscara, ordenação, formato…). ⛔ O piso é `2` porque um selector de UMA opção não existe.
    let fileiras = grupos.iter().filter(|g| (2..32).contains(&g.len())).count();
    assert!(
        fileiras >= 10,
        "só {fileiras} fileiras segmentadas declaradas — o pintor canónico
         (`paint_segmented_group_adaptive`) deixou de declarar o grupo, e o censo volta a contar \
         cada opção como um comando. Grupos: {:?}",
        grupos.iter().map(Vec::len).collect::<Vec<_>>()
    );
}
