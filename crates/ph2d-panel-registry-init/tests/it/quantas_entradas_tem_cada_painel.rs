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
    /// ⭐⭐⭐ **Quantos comandos DISTINTOS o painel oferece** — a grandeza da `D2`.
    ///
    /// ⛔⛔ **Um botão pintado uma vez POR LINHA de uma lista é UMA capacidade, não N**, e o
    /// [`Contagem::comandos`] conta-o N vezes. Medido em 2026-09-21: o `tokens` lê **`110`**
    /// comandos e oferece **`4`** — `107` daqueles são o mesmo botão de *elo* repetido em cada
    /// linha da tabela de cor, por decisão escrita no pintor (*«qualquer token pode seguir
    /// qualquer outro»*). ⇒ *o número que ordenava a dívida era o COMPRIMENTO da lista.*
    ///
    /// ⚠️ **É a mesma cegueira do composto, virada 90°:** aquele é um controlo repetido ao
    /// LONGO de uma linha, este é um comando repetido ao LONGO de uma coluna. O primeiro foi
    /// curado por suspeita; o segundo só apareceu porque alguém foi atacar o painel que o
    /// número apontava.
    ///
    /// ⛔ **O discriminador NÃO é geométrico** — a 1.ª redacção agrupava por coluna e colapsava
    /// os `34` botões de largura cheia do Inspector em `1`. Ele é a PROVENIÊNCIA do id: um id
    /// que está escrito no fonte é um comando **nomeado**; um que nasce de
    /// `hash_node_id_runtime(&format!("…{row}"))` é uma **instância** de uma família, e só aí a
    /// coluna decide.
    distintos: usize,
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

/// ⭐⭐⭐ **Os ids que estão ESCRITOS no fonte** — a porta que separa um comando de uma instância.
///
/// ⚠️⚠️ **Há TRÊS formas de declarar um id nesta casa, e a varredura tem de as conhecer às três**
/// — `hash_node_id("<lit>")` · `hash_node_id_runtime("<lit>")` (um literal continua a ser um
/// literal, seja quem for a hashear) · e `NodeId(<n>)` cru, que é como o `grid-snap` declara os
/// dele. ⛔ **Uma forma que falte erra no sentido MAU**: os ids dela caem no balde dos derivados,
/// colapsam por coluna, e o painel lê-se **mais barato do que é** — medido, o `grid-snap` lia
/// `10` em vez de `20`. É por isso que [`todo_painel_com_id_derivado_esta_nomeado`] existe.
fn ids_nomeados() -> &'static std::collections::BTreeSet<u64> {
    static CACHE: std::sync::OnceLock<std::collections::BTreeSet<u64>> = std::sync::OnceLock::new();
    CACHE.get_or_init(|| {
        let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crates/");
        let mut out = std::collections::BTreeSet::new();
        let mut pilha = vec![raiz.to_path_buf()];
        while let Some(d) = pilha.pop() {
            let Ok(rd) = std::fs::read_dir(&d) else {
                continue;
            };
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    pilha.push(p);
                    continue;
                }
                if !p.extension().is_some_and(|x| x == "rs") {
                    continue;
                }
                let Ok(src) = std::fs::read_to_string(&p) else {
                    continue;
                };
                for chave in ["hash_node_id(\"", "hash_node_id_runtime(\""] {
                    for pedaco in src.split(chave).skip(1) {
                        if let Some(lit) = pedaco.split('"').next() {
                            out.insert(ph2d_tool_registry::hash_node_id_runtime(lit).0);
                        }
                    }
                }
                for pedaco in src.split("NodeId(").skip(1) {
                    let Some(n) = pedaco.split(')').next() else {
                        continue;
                    };
                    if let Ok(v) = n.trim().parse::<u64>() {
                        out.insert(v);
                    }
                }
            }
        }
        out
    })
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

    // ⭐⭐⭐ **Os comandos DISTINTOS** — ver [`Contagem::distintos`]. Um botão NOMEADO conta por
    // si; as instâncias derivadas agrupam-se por COLUNA (`x` e largura iguais), e cada coluna é
    // uma capacidade.
    //
    // ⚠️ A geometria só decide DENTRO do balde dos derivados. Aplicá-la a todos colapsaria os
    // `34` botões de largura cheia empilhados do Inspector num só — medido, e é o CONTROLO que
    // matou a 1.ª redacção desta lei.
    let nomeados = ids_nomeados();
    let mut com_nome = 0usize;
    let mut colunas: std::collections::BTreeSet<(i32, i32)> = std::collections::BTreeSet::new();
    for (id, r) in pintados {
        if celula.contains(&id.0)
            || !matches!(store.get(*id), Some(InteractiveState::Button { .. }))
        {
            continue;
        }
        if nomeados.contains(&id.0) {
            com_nome += 1;
        } else {
            colunas.insert((r.x.round() as i32, r.w.round() as i32));
        }
    }
    c.distintos = com_nome + colunas.len();
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

/// ⭐⭐⭐ **ABRE TODA GAVETA antes de medir** — o censo mede o que o painel CONTÉM.
///
/// ⛔⛔ **Ele mede o ECRÃ e não o catálogo** (ver [`conta`]: registar e pintar diferem por `12×`),
/// e desde 2026-09-21 o ecrã **DOBRA**: o Inspector abre com toda secção com chevron recolhida
/// menos a Transform. *Sem esta linha o censo passaria a ler `~10` comandos onde há `88`, e a
/// dívida não teria desaparecido — teria ficado escondida.*
///
/// ⇒ são DUAS perguntas e DUAS réguas: este censo pergunta *quanto este painel TEM*, e o
/// [`o_inspector_abre_dentro_da_dobra`] pergunta *quanto ele MOSTRA ao abrir*.
///
/// ⚠️ **Pela porta [`ph2d_editor_core::interaction::WidgetStore::collapsible_ids`]**, que é o
/// conjunto que o `populate` semeou — uma lista escrita à mão aqui ficaria cega à gaveta que a
/// próxima wave acrescentar, e o modo de falha é o censo a encolher em silêncio.
fn abre_tudo(store: &mut ph2d_editor_core::interaction::WidgetStore) {
    // ⚠️⚠️ **O conjunto das gavetas é semeado pelo `populate_shared` do PRODUTO**, não pelo
    //    `Panel::populate` — sem esta chamada o `collapsible_ids` devolve **vazio** no arnês e o
    //    `abre_tudo` abre nada, em silêncio. *O censo lia `12` comandos no Inspector e a régua
    //    parecia certa.*
    ph2d_editor_core::screens::hero::pre_populate::marca_as_gavetas(store);
    let gavetas = store.collapsible_ids();
    assert!(
        gavetas.len() >= 38,
        "o arnês vê {} gavetas e as secções vivas do Inspector são {} — sem elas este censo mede \
         um painel dobrado e chama-lhe a dívida dele",
        gavetas.len(),
        ph2d_editor_core::ids::LIVE_SECTIONS.len()
    );
    for id in gavetas {
        store.set_collapsed(id, false);
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
            abre_tudo(host.store_mut());
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
                    abre_tudo(host.store_mut());
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
            abre_tudo(host.store_mut());
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
        "\n  painel                     comandos  distintos  valores  cromo  orfaos  total   altura  fora-da-dobra\n",
    );
    for l in linhas {
        let c = l.cheia();
        let fora = c.altura - DOBRA;
        s.push_str(&format!(
            "  {:<26} {:>8}  {:>9}  {:>7}  {:>5}  {:>6}  {:>5}  {:>7.0}  {}{}\n",
            l.painel,
            c.comandos,
            c.distintos,
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
            abre_tudo(host.store_mut());
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
        abre_tudo(host.store_mut());
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
        abre_tudo(host.store_mut());
        // ⭐ A pintura corre DENTRO do censo dos compostos — senão esta sonda volta a contar cada
        //   opção de um selector como um comando, que é o defeito que ela própria revelou.
        let (_, grupos) = ph2d_editor_core::widget::composto::medindo(|| {
            let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
        });
        let pintados = host.registos_da_ultima_pintura();
        let store = host.store();
        let celula: std::collections::BTreeSet<u64> =
            grupos.iter().flatten().map(|id| id.0).collect();

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
            if celula.contains(&id.0) {
                c.celulas += 1;
                continue;
            }
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
        // ⭐ Cada grupo conta como UM valor, na secção do 1.º membro que tem literal.
        for g in &grupos {
            if let Some(sec) = g.iter().find_map(|id| nome.get(&id.0).map(|v| v.0.clone())) {
                let c = por_seccao.entry(sec).or_default();
                c.valores += 1;
                c.grupos += 1;
            }
        }

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
            if celula.contains(&id.0) {
                continue;
            }
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
            abre_tudo(host.store_mut());
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

/// O mapa `NodeId -> literal` dos ids do Inspector, para as sondas nomearem o que acham.
fn nomes_do_inspector() -> std::collections::BTreeMap<u64, String> {
    use ph2d_tool_registry::hash_node_id_runtime;
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../ph2d-panel-inspector/src/ids"
    );
    let mut m = std::collections::BTreeMap::new();
    let Ok(rd) = std::fs::read_dir(dir) else {
        return m;
    };
    for e in rd.flatten() {
        let Ok(src) = std::fs::read_to_string(e.path()) else {
            continue;
        };
        for pedaco in src.split("hash_node_id(\"").skip(1) {
            if let Some(lit) = pedaco.split('"').next() {
                m.insert(hash_node_id_runtime(lit).0, lit.to_string());
            }
        }
    }
    m
}

/// SONDA TEMPORÁRIA — fileiras de botões lado a lado que NINGUÉM declarou como grupo.
#[test]
#[ignore]
fn diag_compostos_por_declarar() {
    use ph2d_editor_core::widget::composto;
    use std::collections::BTreeMap;

    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_editor_core::panel::with_registry(|reg| {
        let mut total = 0usize;
        for painel in reg.panels_mut() {
            let id = painel.manifest.id;
            let mut host = MockPanelHost::new();
            let arm = super::paineis_armados::TABELA
                .iter()
                .find(|a| a.painel == id);
            if let Some(a) = arm {
                (a.arma)(host.store_mut());
            }
            painel.populate(host.store_mut());
            abre_tudo(host.store_mut());
            let (_, grupos) = composto::medindo(|| {
                let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
            });
            let pintados = host.registos_da_ultima_pintura();
            let store = host.store();
            if let Some(a) = arm {
                (a.desarma)();
            }
            let declarado: std::collections::BTreeSet<u64> =
                grupos.iter().flatten().map(|n| n.0).collect();

            // Botões pintados, por FAIXA de linha (mesmo `y`, mesma altura).
            let mut faixas: BTreeMap<(i32, i32), Vec<(f32, u64)>> = BTreeMap::new();
            for (nid, r) in &pintados {
                if declarado.contains(&nid.0) {
                    continue;
                }
                if !matches!(
                    store.get(*nid),
                    Some(ph2d_editor_core::interaction::InteractiveState::Button { .. })
                ) {
                    continue;
                }
                faixas
                    .entry((r.y.round() as i32, r.h.round() as i32))
                    .or_default()
                    .push((r.x, nid.0));
            }
            // ⚠️ Uma FILEIRA é um conjunto de botões ENCOSTADOS (a folga entre dois vizinhos é
            //    pequena). Dois botões longe um do outro na mesma linha são dois comandos.
            let mut suspeitos = 0usize;
            let mut maior = 0usize;
            let mut nomeadas: Vec<Vec<String>> = Vec::new();
            for ((_, h), mut v) in faixas {
                if v.len() < 2 {
                    continue;
                }
                v.sort_by(|a, b| a.0.total_cmp(&b.0));
                suspeitos += v.len();
                maior = maior.max(v.len());
                nomeadas.push(
                    v.iter()
                        .map(|(_, hh)| nomes_do_inspector().get(hh).cloned().unwrap_or_default())
                        .collect(),
                );
                let _ = h;
            }
            if suspeitos > 0 {
                println!(
                    "  {id:22} {suspeitos:4} botões em fileira por declarar (maior fileira: {maior})"
                );
                total += suspeitos;
            }
            if id == "inspector" {
                for v in &nomeadas {
                    println!("      fileira de {}: {}", v.len(), v.join(" · "));
                }
            }
        }
        println!("\n  TOTAL por declarar: {total}");
    });
}

/// ⛔ **A CATRACA DA CARGA DE COMANDOS — ela só ENCOLHE, e mede COMANDOS DISTINTOS.**
///
/// ⚠️⚠️ **Ela existe porque o número que ordena esta lista já foi `3,6×` maior do que a verdade**
/// — e depois **`27×`**. O Inspector leu `314` enquanto a `D2` o media; com os compostos
/// declarados lê `88`. O `tokens` leu `110` e assumiu o topo da lista; ele oferece **`4`**.
///
/// ⛔⛔ **As duas mentiras são a MESMA, viradas 90°:** um controlo repetido ao LONGO DE UMA LINHA
/// (o selector, curado em 2026-09-21 de manhã) e um comando repetido ao LONGO DE UMA COLUNA (o
/// botão de *elo* que o `tokens` pinta em cada uma das `107` linhas da tabela de cor). ⚠️ *A
/// primeira foi achada por suspeita; a segunda só apareceu porque alguém foi ATACAR o painel que
/// o número apontava* — e a nota que aqui esteve dizia, por escrito, *«o `tokens` é hoje o topo da
/// lista e a dívida dele é real»*. **Era falso: `107` dos `110` são um botão só.**
///
/// ⭐ **Por isso a catraca mede [`Contagem::distintos`] e não [`Contagem::comandos`]:** o número
/// bruto de um painel-LISTA é o COMPRIMENTO da lista, logo acrescentar um token de desenho fá-lo
/// subir e a mensagem acusaria *«um composto deixou de se declarar»* — falso, e manda a cura para
/// o sítio errado. O distinto é invariante ao tamanho da lista, que é o que uma dívida de
/// capacidade tem de ser.
///
/// ⚠️ **Ela continua a guardar os compostos:** apagar um `composto::grupo` devolve as células ao
/// balde dos botões, e como elas têm id nomeado o distinto **sobe** na mesma.
///
/// ⛔ **Os números são MEDIDOS, não escolhidos**, e a catraca só desce. Um painel que passe a
/// contar MENOS também reprova, com a outra metade: *ela não é folga, é o sítio onde se escreve o
/// número novo.*
const CARGA_DE_COMANDOS: &[(&str, usize)] = &[
    // ⭐ `314 → 150` (os pintores canónicos) `→ 88` (o helper de 16 sítios + as abas). Todos os
    //    `88` são ids NOMEADOS: aqui o bruto e o distinto coincidem, e a dívida é real.
    ("inspector", 88),
    // ⛔⛔ `110` botões, **`4`** comandos: fechar · importar · exportar · e o *elo*, que é pintado
    //    uma vez por linha por decisão escrita no pintor. O painel é uma LISTA, não uma dívida.
    ("tokens", 4),
    // ⛔ `58` botões, **`17`** comandos: a fábrica derivada dele vive noutra crate
    //    (`ph2d_tool_painter::ids::wet_tuning_reset_id`) — um *Reset* por botão de afinação.
    ("wet_tuning", 17),
    ("physics", 49),
    ("sculpt3d", 36),
    ("vector", 24),
    ("model3d", 1),
];

#[test]
fn a_carga_de_comandos_de_um_painel_so_encolhe() {
    let linhas = censo();
    let medido: std::collections::BTreeMap<&str, usize> = linhas
        .iter()
        .map(|l| (l.painel, l.cheia().distintos))
        .collect();
    let mut subiram = Vec::new();
    let mut desceram = Vec::new();
    for (painel, tecto) in CARGA_DE_COMANDOS {
        let Some(&n) = medido.get(painel) else {
            panic!("`{painel}` saiu do registo — apague a linha da catraca");
        };
        if n > *tecto {
            subiram.push(format!("{painel}: {n} contra {tecto}"));
        } else if n < *tecto {
            desceram.push(format!("{painel}: {n} contra {tecto}"));
        }
    }
    assert!(
        subiram.is_empty(),
        "estes painéis passaram a contar MAIS comandos:\n  {}\n\n\
         ⇒ ou um composto deixou de se declarar (as células dele voltam a contar uma a uma), ou \
         o painel ganhou uma capacidade nova. ⛔ A cura da 1.ª é declarar o grupo, nunca subir o \
         número.{}",
        subiram.join("\n  "),
        tabela(&linhas)
    );
    assert!(
        desceram.is_empty(),
        "estes painéis contam MENOS do que a catraca diz — ESCREVA o número medido:\n  {}{}",
        desceram.join("\n  "),
        tabela(&linhas)
    );
}

/// ⛔⛔⛔ **O CONTROLO da varredura de ids — quem tem botão DERIVADO está NOMEADO.**
///
/// [`ids_nomeados`] conhece **três** formas de declarar um id, e uma forma que falte erra no
/// sentido MAU: os ids dela caem no balde dos derivados, colapsam por coluna, e o painel lê-se
/// **mais barato do que é**. Medido durante a construção: o `grid-snap` declara os dele como
/// `NodeId(1033)` cru e lia **`10`** comandos distintos em vez de **`20`**.
///
/// ⚠️ **Um painel com botões derivados tem de estar nesta lista, com a FÁBRICA escrita ao lado.**
/// Um painel que apareça aqui sem estar na lista é a varredura cega outra vez, e a mensagem
/// manda procurar a forma nova — nunca acrescentar a linha sem a olhar.
///
/// ⛔ **A lista tem a metade da OBSOLESCÊNCIA**, senão ela vira licença: uma entrada cujo painel
/// já não tem botão derivado reprova e sai.
///
/// ⚠️ **A fábrica pode viver noutra crate** — a 1.ª redacção deste controlo exigia-a na crate do
/// painel e o `wet_tuning` desmentiu-a: os *Reset* dele nascem em `ph2d-tool-painter`.
const PAINEIS_COM_ID_DERIVADO: &[(&str, &str)] = &[
    (
        "asset_browser",
        "asset_browser::ids — um botão por ficheiro da lista",
    ),
    (
        "authored",
        "authored.opt.{key}.{index} · authored.row.{key}",
    ),
    (
        "tokens",
        "tokens_link_id(row) — o elo, uma vez por linha da tabela de cor",
    ),
    (
        "wet_tuning",
        "ph2d_tool_painter::ids::wet_tuning_reset_id(key)",
    ),
];

#[test]
fn todo_painel_com_id_derivado_esta_nomeado() {
    let linhas = censo();
    let mut inesperados = Vec::new();
    let mut obsoletos = Vec::new();
    for l in &linhas {
        let c = l.cheia();
        let derivado = c.comandos > c.distintos;
        let listado = PAINEIS_COM_ID_DERIVADO.iter().any(|(p, _)| *p == l.painel);
        if derivado && !listado {
            inesperados.push(format!(
                "{}: {} botões contra {} distintos",
                l.painel, c.comandos, c.distintos
            ));
        }
        if listado && !derivado {
            obsoletos.push(l.painel.to_string());
        }
    }
    assert!(
        inesperados.is_empty(),
        "estes painéis têm botões que a varredura de ids NÃO reconhece:\n  {}\n\n\
         ⇒ ou eles têm uma fábrica derivada por nomear na `PAINEIS_COM_ID_DERIVADO`, ou — e é o \
         caso perigoso — eles declaram os ids numa FORMA que a `ids_nomeados` não conhece, e \
         nesse caso o painel está a ler-se mais barato do que é. ⛔ Olhe a forma ANTES de \
         acrescentar a linha.{}",
        inesperados.join("\n  "),
        tabela(&linhas)
    );
    assert!(
        obsoletos.is_empty(),
        "estas entradas já não descrevem nada — apague-as:\n  {}",
        obsoletos.join("\n  ")
    );
    // ⚠️ O piso: sem ele, uma varredura que passasse a reconhecer TUDO (ou um censo que medisse
    //    zero botões) deixaria as duas metades acima trivialmente verdadeiras.
    let com_derivado = linhas
        .iter()
        .filter(|l| l.cheia().comandos > l.cheia().distintos)
        .count();
    assert!(
        com_derivado >= 2,
        "só {com_derivado} painéis têm id derivado — esta régua mede a partição e uma partição \
         com um lado vazio não afirma nada.{}",
        tabela(&linhas)
    );
}

/// SONDA TEMPORÁRIA — de que são feitas as entradas do painel de TOKENS.
///
/// ⚠️ Os ids deste painel são **DERIVADOS do índice da linha** (`tokens.reset.{row}`), não
/// literais, então a varredura de texto que nomeia o Inspector lê **zero** aqui. O mapa
/// `NodeId -> nome` constrói-se chamando as PRÓPRIAS funções de id, que são a fonte.
#[cfg(feature = "panel-tokens")]
#[test]
#[ignore]
fn diag_de_quem_sao_as_entradas_do_tokens() {
    use ph2d_panel_tokens::ids as tid;
    use std::collections::BTreeMap;

    // ⭐ A família de cada id, derivada de quem o fabrica. O tecto de `row` é folgado de
    //   propósito: ele só tem de cobrir a tabela, e uma linha a mais custa um hash.
    let mut nome: BTreeMap<u64, &'static str> = BTreeMap::new();
    for row in 0..512usize {
        for (id, fam) in [
            (tid::tokens_swatch_id(row), "cor: swatch"),
            (tid::tokens_reset_id(row), "cor: reset"),
            (tid::tokens_link_id(row), "cor: elo"),
            (tid::tokens_num_chip_id(row), "px: campo"),
            (tid::tokens_num_reset_id(row), "px: reset"),
            (tid::tokens_num_link_id(row), "px: elo"),
            (tid::tokens_num_fx_id(row), "px: f(x)"),
            (tid::tokens_num_formula_id(row), "px: fórmula"),
        ] {
            nome.insert(id.0, fam);
        }
    }
    for (id, fam) in [
        (tid::TOKENS_CLOSE, "painel: fechar"),
        (tid::TOKENS_RESET_ALL, "painel: reset all"),
        (tid::TOKENS_DTCG_EXPORT, "painel: export"),
        (tid::TOKENS_DTCG_IMPORT, "painel: import"),
    ] {
        nome.insert(id.0, fam);
    }

    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_editor_core::panel::with_registry(|reg| {
        let painel = reg
            .panels_mut()
            .iter_mut()
            .find(|p| p.manifest.id == "tokens")
            .expect("o painel de tokens tem de estar no registo");
        let mut host = MockPanelHost::new();
        painel.populate(host.store_mut());
        abre_tudo(host.store_mut());
        let (_, grupos) = ph2d_editor_core::widget::composto::medindo(|| {
            let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
        });
        let pintados = host.registos_da_ultima_pintura();
        let store = host.store();
        let celula: std::collections::BTreeSet<u64> =
            grupos.iter().flatten().map(|id| id.0).collect();

        let mut por_familia: BTreeMap<&'static str, Contagem> = BTreeMap::new();
        for (id, _r) in &pintados {
            let fam = nome.get(&id.0).copied().unwrap_or("(id desconhecido)");
            let c = por_familia.entry(fam).or_default();
            if celula.contains(&id.0) {
                c.celulas += 1;
                continue;
            }
            match store.get(*id) {
                Some(s) => classifica(c, s),
                None => c.orfaos += 1,
            }
        }
        println!("\n  família               comandos  valores  cromo  órfãos  total");
        for (f, c) in &por_familia {
            println!(
                "  {:22} {:8} {:8} {:6} {:7} {:6}",
                f,
                c.comandos,
                c.valores,
                c.outros,
                c.orfaos,
                c.total()
            );
        }
        println!("\n  grupos declarados: {}", grupos.len());
    });
}

/// Pinta o Inspector com as portas de [`super::o_inspector_armado::PORTAS`] cujo nome está em
/// `ligadas`, e devolve a altura em píxeis.
#[cfg(test)]
fn altura_do_inspector_com(ligadas: &[&str]) -> f32 {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut h = 0.0;
    ph2d_editor_core::panel::with_registry(|reg| {
        let painel = reg
            .panels_mut()
            .iter_mut()
            .find(|p| p.manifest.id == "inspector")
            .expect("o inspector tem de estar no registo");
        super::o_inspector_armado::arma_tudo();
        for (nome, desarma) in super::o_inspector_armado::PORTAS {
            if !ligadas.contains(nome) {
                desarma();
            }
        }
        let mut host = MockPanelHost::new();
        painel.populate(host.store_mut());
        // ⚠️ **A política de dobra mora no arranque do EDITOR**, não no `Panel::populate` —
        //    esta régua percorre a porta do produto, senão mede um painel todo aberto.
        ph2d_editor_core::screens::hero::pre_populate::marca_as_gavetas(host.store_mut());
        let (_, grupos) = ph2d_editor_core::widget::composto::medindo(|| {
            let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
        });
        h = conta(host.store(), &host.registos_da_ultima_pintura(), &grupos).altura;
        super::o_inspector_armado::desarma_tudo();
    });
    h
}

/// SONDA TEMPORÁRIA — **o preço de cada secção do Inspector, em píxeis e em ecrãs.**
#[test]
#[ignore]
fn diag_o_preco_de_cada_seccao_do_inspector() {
    let todas: Vec<&str> = super::o_inspector_armado::PORTAS
        .iter()
        .map(|(n, _)| *n)
        .collect();
    let cheio = altura_do_inspector_com(&todas);
    println!(
        "\n  TUDO ARMADO (o objecto impossível): {cheio:.0} px = {:.1} ecrãs\n",
        cheio / DOBRA
    );

    let mut precos: Vec<(f32, &str)> = Vec::new();
    for (nome, _) in super::o_inspector_armado::PORTAS {
        let sem: Vec<&str> = todas.iter().copied().filter(|n| n != nome).collect();
        precos.push((cheio - altura_do_inspector_com(&sem), nome));
    }
    precos.sort_by(|a, b| b.0.total_cmp(&a.0));
    println!("  secção                  custo px   ecrãs");
    let mut soma = 0.0;
    for (px, nome) in &precos {
        soma += px;
        println!("  {nome:22} {px:9.0}  {:6.2}", px / DOBRA);
    }
    println!("\n  soma dos custos: {soma:.0} px (o cheio é {cheio:.0})");

    // ⭐ Cenários que EXISTEM.
    for (nome, ligadas) in CENARIOS_REAIS {
        let h = altura_do_inspector_com(ligadas);
        println!(
            "\n  [{nome}] {h:.0} px = {:.1} ecrãs  ({} secções)",
            h / DOBRA,
            ligadas.len()
        );
    }
}

/// ⭐⭐⭐ **Objectos que EXISTEM** — cada um é uma combinação que o app de facto produz.
///
/// ⛔ A [`super::o_inspector_armado::arma_tudo`] monta um objecto impossível, e o cabeçalho dela
/// di-lo por escrito. *Uma régua de ALTURA corrida sobre ele mede um estado que nenhum artista
/// alcança.*
const CENARIOS_REAIS: &[(&str, &[&str])] = &[
    // O mais comum do app: uma imagem na cena.
    (
        "sprite simples",
        &[
            "name",
            "transform",
            "visibility",
            "sprite",
            "sampling",
            "blend",
            "visibility_section",
            "ordering",
        ],
    ),
    // O mesmo, com um corpo — a cena de física mais simples.
    (
        "sprite + corpo físico",
        &[
            "name",
            "transform",
            "visibility",
            "sprite",
            "sampling",
            "blend",
            "visibility_section",
            "ordering",
            "physics",
        ],
    ),
    // O herói de um plataforma: corpo, controlador, câmera a segui-lo.
    (
        "herói de plataforma",
        &[
            "name",
            "transform",
            "visibility",
            "sprite",
            "sampling",
            "blend",
            "visibility_section",
            "ordering",
            "physics",
            "player",
            "camera",
            "tags",
        ],
    ),
];

/// SONDA TEMPORÁRIA — **o CHÃO do Inspector**: quanto ele custa com todas as secções dobradas.
#[test]
#[ignore]
fn diag_o_chao_do_inspector_com_tudo_dobrado() {
    // ⭐ A TABELA, nunca uma varredura de texto: a 1.ª redacção filtrava literais por
    //   «section»/«header» e achou **1** de `38` — os ids das secções vivas vivem na fundação,
    //   e a `nomes_do_inspector` só lê a crate do painel.
    let cabecalhos = ph2d_editor_core::ids::LIVE_SECTIONS;
    println!("\n  secções vivas: {}", cabecalhos.len());

    for (nome, ligadas) in CENARIOS_REAIS {
        let _ = ph2d_panel_registry_init::register_all_panels();
        ph2d_editor_core::panel::with_registry(|reg| {
            let painel = reg
                .panels_mut()
                .iter_mut()
                .find(|p| p.manifest.id == "inspector")
                .expect("o inspector");
            super::o_inspector_armado::arma_tudo();
            for (p, desarma) in super::o_inspector_armado::PORTAS {
                if !ligadas.contains(p) {
                    desarma();
                }
            }
            let mut host = MockPanelHost::new();
            painel.populate(host.store_mut());
            // ⭐ Dobrar TUDO depois do `populate` — ele é quem semeia o estado das secções.
            let mut dobrados = 0usize;
            for (seccao, _cor) in &cabecalhos {
                host.store_mut().set_collapsed(*seccao, true);
                dobrados += 1;
            }
            let (_, grupos) = ph2d_editor_core::widget::composto::medindo(|| {
                let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
            });
            let c = conta(host.store(), &host.registos_da_ultima_pintura(), &grupos);
            super::o_inspector_armado::desarma_tudo();
            println!(
                "  [{nome}] dobrado: {:.0} px = {:.1} ecrãs  ({dobrados} cabeçalhos postos a dobrar, {} controlos ainda pintados)",
                c.altura,
                c.altura / DOBRA,
                c.total()
            );
        });
    }
}

/// Pinta o Inspector no cenário `ligadas`, com as secções vivas dobradas **excepto** `abertas`
/// (que se nomeia pelo miolo do literal: `"name"` ⇒ `insp_live_name_section`), e devolve a altura.
#[cfg(test)]
fn altura_do_inspector_dobrando_tudo_menos(ligadas: &[&str], abertas: &[&str]) -> f32 {
    let deixa_aberta: Vec<NodeId> = abertas
        .iter()
        .map(|n| ph2d_tool_registry::hash_node_id_runtime(&format!("insp_live_{n}_section")))
        .collect();
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut h = 0.0;
    ph2d_editor_core::panel::with_registry(|reg| {
        let painel = reg
            .panels_mut()
            .iter_mut()
            .find(|p| p.manifest.id == "inspector")
            .expect("o inspector");
        super::o_inspector_armado::arma_tudo();
        for (p, desarma) in super::o_inspector_armado::PORTAS {
            if !ligadas.contains(p) {
                desarma();
            }
        }
        let mut host = MockPanelHost::new();
        painel.populate(host.store_mut());
        // ⚠️⚠️ **Pela porta do produto PRIMEIRO**, e a razão é um gate vermelho: ela semeia também
        //    as SUB-secções (a grelha de 32 camadas da visibilidade, a máscara de *cull*), que não
        //    são secções vivas. Sem esta linha o «chão» media `752 px` e a abertura media `673` —
        //    *o painel com a Transform ABERTA lia-se mais baixo do que com tudo fechado*, porque as
        //    duas réguas não estavam a medir o mesmo painel.
        ph2d_editor_core::screens::hero::pre_populate::marca_as_gavetas(host.store_mut());
        // ⚠️ **Escreve as DUAS pontas**, e não só a dobra: a política já semeia, logo uma função
        //    que só fecha não consegue exprimir «tudo aberto» — e sem isso o CONTROLO do gate da
        //    dobra não existe.
        for (seccao, _cor) in &ph2d_editor_core::ids::LIVE_SECTIONS {
            host.store_mut()
                .set_collapsed(*seccao, !deixa_aberta.contains(seccao));
        }
        let (_, grupos) = ph2d_editor_core::widget::composto::medindo(|| {
            let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
        });
        h = conta(host.store(), &host.registos_da_ultima_pintura(), &grupos).altura;
        super::o_inspector_armado::desarma_tudo();
    });
    h
}

/// SONDA TEMPORÁRIA — **as políticas de abertura, lado a lado.**
#[test]
#[ignore]
fn diag_as_politicas_de_abertura_do_inspector() {
    const POLITICAS: &[(&str, &[&str])] = &[
        (
            "identidade (name·transform·visibility·render)",
            &["name", "transform", "visibility", "render"],
        ),
        ("só o transform", &["transform"]),
        ("transform + render", &["transform", "render"]),
        ("tudo dobrado", &[]),
    ];
    let todas: Vec<&str> = SECCOES_POR_NOME.to_vec();
    println!("\n  cenário                  como o painel ABRE hoje");
    for (nome, ligadas) in CENARIOS_REAIS {
        let h = altura_do_inspector_com(ligadas);
        println!("  {nome:24} {h:7.0} px = {:.1} ecrãs", h / DOBRA);
    }
    println!("\n  cenário                  com TUDO forçado aberto (o antes)");
    for (nome, ligadas) in CENARIOS_REAIS {
        let h = altura_do_inspector_dobrando_tudo_menos(ligadas, &todas);
        println!("  {nome:24} {h:7.0} px = {:.1} ecrãs", h / DOBRA);
    }
    for (pol, abertas) in POLITICAS {
        println!("\n  política: {pol}");
        for (nome, ligadas) in CENARIOS_REAIS {
            let h = altura_do_inspector_dobrando_tudo_menos(ligadas, abertas);
            println!(
                "  {nome:24} {h:7.0} px = {:.1} ecrãs  {}",
                h / DOBRA,
                if h <= DOBRA { "✓ cabe" } else { "" }
            );
        }
    }
}

/// SONDA TEMPORÁRIA — **o que fica no chão** quando toda secção viva está dobrada.
#[test]
#[ignore]
fn diag_o_que_sobra_no_chao_do_inspector() {
    let nomes = nomes_do_inspector();
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_editor_core::panel::with_registry(|reg| {
        let painel = reg
            .panels_mut()
            .iter_mut()
            .find(|p| p.manifest.id == "inspector")
            .expect("o inspector");
        super::o_inspector_armado::arma_tudo();
        let ligadas = CENARIOS_REAIS[0].1;
        for (p, desarma) in super::o_inspector_armado::PORTAS {
            if !ligadas.contains(p) {
                desarma();
            }
        }
        let mut host = MockPanelHost::new();
        painel.populate(host.store_mut());
        for (seccao, _cor) in &ph2d_editor_core::ids::LIVE_SECTIONS {
            host.store_mut().set_collapsed(*seccao, true);
        }
        let (_, grupos) = ph2d_editor_core::widget::composto::medindo(|| {
            let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
        });
        let pintados = host.registos_da_ultima_pintura();
        let store = host.store();
        let celula: std::collections::BTreeSet<u64> =
            grupos.iter().flatten().map(|n| n.0).collect();
        let mut v: Vec<(f32, f32, String, String)> = pintados
            .iter()
            .filter(|(id, _)| !celula.contains(&id.0))
            .filter_map(|(id, r)| {
                store.get(*id).map(|s| {
                    (
                        r.y,
                        r.h,
                        format!("{s:?}")
                            .split(' ')
                            .next()
                            .unwrap_or("?")
                            .to_string(),
                        nomes
                            .get(&id.0)
                            .cloned()
                            .unwrap_or_else(|| "(sem literal)".into()),
                    )
                })
            })
            .collect();
        v.sort_by(|a, b| a.0.total_cmp(&b.0));
        println!("\n  y      h    espécie        id");
        for (y, h, esp, lit) in &v {
            println!("  {y:6.0} {h:4.0}  {esp:14} {lit}");
        }
        super::o_inspector_armado::desarma_tudo();
    });
}

/// Quantos controlos o Inspector pinta com `dobrada` dobrada (ou nenhuma, se `None`), com TODAS as
/// portas armadas.
#[cfg(test)]
fn controlos_do_inspector_dobrando(dobrada: Option<NodeId>) -> usize {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut n = 0usize;
    ph2d_editor_core::panel::with_registry(|reg| {
        let painel = reg
            .panels_mut()
            .iter_mut()
            .find(|p| p.manifest.id == "inspector")
            .expect("o inspector");
        super::o_inspector_armado::arma_tudo();
        let mut host = MockPanelHost::new();
        painel.populate(host.store_mut());
        if let Some(id) = dobrada {
            host.store_mut().set_collapsed(id, true);
        }
        let (_, grupos) = ph2d_editor_core::widget::composto::medindo(|| {
            let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
        });
        n = conta(host.store(), &host.registos_da_ultima_pintura(), &grupos).total();
        super::o_inspector_armado::desarma_tudo();
    });
    n
}

/// SONDA TEMPORÁRIA — **quantas secções vivas CUMPREM o chevron que pintam.**
#[test]
#[ignore]
fn diag_que_seccoes_do_inspector_nao_dobram() {
    let base = controlos_do_inspector_dobrando(None);
    println!("\n  tudo aberto: {base} controlos\n");
    let nomes: std::collections::BTreeMap<u64, String> = SECCOES_POR_NOME
        .iter()
        .map(|n| {
            (
                ph2d_tool_registry::hash_node_id_runtime(&format!("insp_live_{n}_section")).0,
                (*n).to_string(),
            )
        })
        .collect();
    let mut mortas = Vec::new();
    for (seccao, _cor) in &ph2d_editor_core::ids::LIVE_SECTIONS {
        let n = controlos_do_inspector_dobrando(Some(*seccao));
        let nome = nomes
            .get(&seccao.0)
            .cloned()
            .unwrap_or_else(|| format!("{:x}", seccao.0));
        let caiu = base - n;
        println!("  {nome:16} dobrada: {n:4} controlos  (−{caiu})");
        if caiu <= 1 {
            mortas.push(nome.clone());
        }
    }
    println!(
        "\n  {} de {} secções pintam o chevron e NÃO dobram: {:?}",
        mortas.len(),
        ph2d_editor_core::ids::LIVE_SECTIONS.len(),
        mortas
    );
}

/// Os miolos dos literais `insp_live_<x>_section`, para as sondas nomearem o que acham.
const SECCOES_POR_NOME: &[&str] = &[
    "action",
    "anchor",
    "anim",
    "audio",
    "blend",
    "camera",
    "color",
    "emitter",
    "factory",
    "hud",
    "joint",
    "lifecycle",
    "name",
    "ordering",
    "particles",
    "pathfollow",
    "physics",
    "player",
    "projectile",
    "ray",
    "render",
    "sampling",
    "script",
    "seq",
    "shake",
    "sheet",
    "slice",
    "sm",
    "tags",
    "timer",
    "topdown",
    "transform",
    "trigger",
    "tween",
    "visibility",
    "watch",
    "weapon",
    "wheel",
];

/// SONDA TEMPORÁRIA — **o artista consegue abrir a máscara de cull da câmera?**
#[test]
#[ignore]
fn diag_a_mascara_de_cull_da_camera_abre() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_editor_core::panel::with_registry(|reg| {
        let painel = reg
            .panels_mut()
            .iter_mut()
            .find(|p| p.manifest.id == "inspector")
            .expect("o inspector");
        super::o_inspector_armado::arma_tudo();
        let mut host = MockPanelHost::new();
        let cull = ph2d_tool_registry::hash_node_id_runtime("insp_camera_cull_header");
        painel.populate(host.store_mut());
        println!(
            "\n  depois do 1.º populate: dobrada = {}  (escolha do artista = {:?})",
            host.store().is_collapsed(cull),
            host.store().collapsed_choice(cull)
        );
        // O artista carrega no cabeçalho.
        host.store_mut().toggle_collapsed(cull);
        println!(
            "  depois do clique:       dobrada = {}",
            host.store().is_collapsed(cull)
        );
        // O quadro seguinte.
        painel.populate(host.store_mut());
        println!(
            "  depois do 2.º populate: dobrada = {}   <-- se voltou a `true`, ela NÃO abre",
            host.store().is_collapsed(cull)
        );
        super::o_inspector_armado::desarma_tudo();
    });
}

/// ⭐⭐⭐ **O INSPECTOR ABRE DENTRO DA DOBRA** — para um objecto que EXISTE.
///
/// ⛔⛔ **A fixtura [`super::o_inspector_armado::arma_tudo`] monta um objecto IMPOSSÍVEL** e o
/// cabeçalho dela di-lo por escrito. Medi-la em altura dá `14 987 px` (`17` ecrãs) e manda uma wave
/// atrás de um estado que nenhum artista alcança — é por isso que este gate corre os
/// [`CENARIOS_REAIS`].
///
/// ⚠️ **As três metades:** o painel cabe · o CONTROLO (com tudo forçado aberto ele **não** cabe,
/// senão isto ficaria verde sobre um painel que já coubesse por acaso) · e a terceira, que é a que
/// impede a cura barata: **alguma coisa continua ABERTA**. *Um painel que abre com tudo dobrado
/// cabe sempre e não serve para nada.*
#[test]
fn o_inspector_abre_dentro_da_dobra() {
    let todas: Vec<&str> = SECCOES_POR_NOME.to_vec();
    let chao = altura_do_inspector_dobrando_tudo_menos(CENARIOS_REAIS[0].1, &[]);
    for (nome, ligadas) in CENARIOS_REAIS {
        let abre = altura_do_inspector_com(ligadas);
        assert!(
            abre <= DOBRA,
            "o Inspector abre com {abre:.0} px para «{nome}», contra uma dobra de {DOBRA:.0} — \
             o artista tem de rolar para ver um objecto acabado de escolher.\n\
             ⇒ a política vive no `populate` do painel: toda secção com chevron nasce dobrada \
             menos a Transform."
        );
        let tudo_aberto = altura_do_inspector_dobrando_tudo_menos(ligadas, &todas);
        assert!(
            tudo_aberto > DOBRA * 2.0,
            "o CONTROLO falhou: com TUDO aberto «{nome}» mede {tudo_aberto:.0} px, e este gate \
             precisa que esse estado NÃO caiba — senão ele fica verde sobre um painel que caberia \
             de qualquer maneira."
        );
        assert!(
            abre > chao,
            "«{nome}» abre com {abre:.0} px e o chão (tudo dobrado) é {chao:.0} — ou seja, NADA \
             está aberto. ⛔ Um painel que abre fechado cabe sempre e não mostra nada."
        );
    }
}

/// ⛔⛔ **TODA SEMEADURA DE DOBRA PASSA PELA PORTA.**
///
/// Um `set_collapsed` cru dentro de um `populate` torna a gaveta **INABRÍVEL** se aquele
/// `populate` voltar a correr — ver [`ph2d_editor_core::interaction::WidgetStore::set_collapsed_if_unchosen`].
/// ⚠️ Hoje o do Inspector corre **uma vez**, logo o perigo é latente; *uma lei que depende de
/// quantas vezes alguém chama uma função é uma lei à espera da chamada seguinte.*
#[test]
fn toda_semeadura_de_dobra_passa_pela_porta() {
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/");
    let mut cruas = Vec::new();
    let mut vistos = 0usize;
    let mut pilha = vec![raiz.to_path_buf()];
    while let Some(d) = pilha.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                pilha.push(p);
                continue;
            }
            let Some(f) = p.file_name().and_then(|s| s.to_str()) else {
                continue;
            };
            // ⚠️ A população é o `populate*`/`pre_populate*`, que é onde a re-corrida é
            //    possível — um `set_collapsed` num braço de CLIQUE é exactamente o que tem
            //    de ser cru. ⛔ O `pre_populate` entrou depois: a semeadura da grelha de
            //    camadas da visibilidade vivia lá e o censo não a via.
            if !(f.starts_with("populate") || f.starts_with("pre_populate")) || !f.ends_with(".rs")
            {
                continue;
            }
            let Ok(src) = std::fs::read_to_string(&p) else {
                continue;
            };
            vistos += 1;
            for (n, linha) in src.lines().enumerate() {
                if linha.contains(".set_collapsed(") && !linha.trim_start().starts_with("//") {
                    cruas.push(format!("{}:{}", p.display(), n + 1));
                }
            }
        }
    }
    // ⚠️ O piso: sem ele uma varredura partida lê zero ficheiros e o gate fica verde a medir nada.
    assert!(
        vistos >= 20,
        "a varredura viu só {vistos} ficheiros `populate*` — ela partiu-se"
    );
    assert!(
        cruas.is_empty(),
        "estas semeaduras de dobra são CRUAS e tornam a gaveta inabrível se o `populate` \
         re-correr:\n  {}\n⇒ use `set_collapsed_if_unchosen`.",
        cruas.join("\n  ")
    );
}

/// SONDA TEMPORÁRIA — **que painéis ABREM fora da dobra**, com a política de dobra em vigor.
///
/// ⚠️ É a régua do [`o_inspector_abre_dentro_da_dobra`] apontada a TODO o registo. ⛔ Ela não é o
/// [`imprime_o_censo`]: aquele abre as gavetas e mede o que o painel TEM; esta mede o que ele
/// MOSTRA ao abrir.
#[test]
#[ignore]
fn diag_que_paineis_abrem_fora_da_dobra() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_editor_core::panel::with_registry(|reg| {
        let mut linhas: Vec<(f32, String, usize, bool, f32, usize)> = Vec::new();
        for painel in reg.panels_mut() {
            let id = painel.manifest.id.to_string();
            let arm = super::paineis_armados::TABELA
                .iter()
                .find(|a| a.painel == id);
            let mut host = MockPanelHost::new();
            if let Some(a) = arm {
                (a.arma)(host.store_mut());
            }
            painel.populate(host.store_mut());
            // A política de dobra do editor, como no arranque.
            ph2d_editor_core::screens::hero::pre_populate::marca_as_gavetas(host.store_mut());
            let (_, grupos) = ph2d_editor_core::widget::composto::medindo(|| {
                let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
            });
            let c = conta(host.store(), &host.registos_da_ultima_pintura(), &grupos);
            // ⭐ O CHÃO: quanto sobra com TODA gaveta que este painel tem fechada. A diferença
            //   para a abertura é o que a política de dobra pode comprar aqui.
            let gavetas = host.store().collapsible_ids();
            for g in &gavetas {
                host.store_mut().set_collapsed(*g, true);
            }
            let (_, gg) = ph2d_editor_core::widget::composto::medindo(|| {
                let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
            });
            let chao = conta(host.store(), &host.registos_da_ultima_pintura(), &gg).altura;
            if let Some(a) = arm {
                (a.desarma)();
            }
            linhas.push((c.altura, id, c.total(), arm.is_some(), chao, gavetas.len()));
        }
        linhas.sort_by(|a, b| b.0.total_cmp(&a.0));
        println!("\n  painel                 abre com  ecrãs  com tudo dobrado  compra  gavetas");
        for (h, id, _n, _armado, chao, gav) in &linhas {
            println!(
                "  {id:22} {h:8.0} {:6.1} {chao:12.0} px {:8.0} px {gav:6}",
                h / DOBRA,
                h - chao
            );
        }
    });
}

/// SONDA TEMPORÁRIA — **o preço de cada secção do painel do PAINTER**, e quais ele de facto pinta.
#[test]
#[ignore]
fn diag_o_preco_de_cada_seccao_do_painter() {
    // ⭐ Os nomes derivam-se das próprias constantes do motor, nunca de uma lista escrita à mão.
    let seccoes: &[(&str, NodeId)] = &[
        ("shape", ph2d_tool_painter::ids::PAINTER_SHAPE_SECTION),
        (
            "shape_ramp",
            ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_SECTION,
        ),
        (
            "brush_texture",
            ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_SECTION,
        ),
        (
            "brush_stroke",
            ph2d_tool_painter::ids::PAINTER_BRUSH_STROKE_SECTION,
        ),
        (
            "brush_randomize",
            ph2d_tool_painter::ids::PAINTER_BRUSH_RANDOMIZE_SECTION,
        ),
        (
            "brush_color_ramp",
            ph2d_tool_painter::ids::PAINTER_BRUSH_COLOR_RAMP_SECTION,
        ),
        (
            "brush_tiling",
            ph2d_tool_painter::ids::PAINTER_BRUSH_TILING_SECTION,
        ),
        (
            "brush_symmetry",
            ph2d_tool_painter::ids::PAINTER_BRUSH_SYMMETRY_SECTION,
        ),
        ("wetpaint", ph2d_tool_painter::ids::PAINTER_WETPAINT_SECTION),
        (
            "watercolor",
            ph2d_tool_painter::ids::PAINTER_WATERCOLOR_SECTION,
        ),
        (
            "watercolor_paper",
            ph2d_tool_painter::ids::PAINTER_WATERCOLOR_PAPER_SECTION,
        ),
        ("impasto", ph2d_tool_painter::ids::PAINTER_IMPASTO_SECTION),
        ("mask", ph2d_tool_painter::ids::PAINTER_MASK_SECTION),
    ];

    let mede = |dobrar: Option<NodeId>, abrir_tudo: bool| -> (f32, bool) {
        let mut h = 0.0;
        let mut pintou = false;
        ph2d_editor_core::panel::with_registry(|reg| {
            let painel = reg
                .panels_mut()
                .iter_mut()
                .find(|p| p.manifest.id == "painter_layers")
                .expect("o painel do painter");
            let arm = super::paineis_armados::TABELA
                .iter()
                .find(|a| a.painel == "painter_layers")
                .expect("o painter tem armação");
            let mut host = MockPanelHost::new();
            (arm.arma)(host.store_mut());
            painel.populate(host.store_mut());
            if abrir_tudo {
                for id in host.store().collapsible_ids() {
                    host.store_mut().set_collapsed(id, false);
                }
            }
            if let Some(id) = dobrar {
                host.store_mut().set_collapsed(id, true);
            }
            let (_, grupos) = ph2d_editor_core::widget::composto::medindo(|| {
                let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
            });
            let pintados = host.registos_da_ultima_pintura();
            pintou = dobrar.is_none_or(|id| pintados.iter().any(|(n, _)| *n == id));
            h = conta(host.store(), &pintados, &grupos).altura;
            (arm.desarma)();
        });
        (h, pintou)
    };

    let _ = ph2d_panel_registry_init::register_all_panels();
    let (como_abre, _) = mede(None, false);
    let (tudo_aberto, _) = mede(None, true);
    println!(
        "\n  como ABRE hoje (a política do dono de 2026-06-24): {como_abre:.0} px = {:.1} ecrãs",
        como_abre / DOBRA
    );
    println!(
        "  com TUDO aberto: {tudo_aberto:.0} px = {:.1} ecrãs\n",
        tudo_aberto / DOBRA
    );
    println!("  secção                  pintada?  custo px   ecrãs");
    let mut v: Vec<(f32, &str, bool)> = Vec::new();
    for (nome, id) in seccoes {
        let (h, pintada) = mede(Some(*id), true);
        v.push((tudo_aberto - h, nome, pintada));
    }
    v.sort_by(|a, b| b.0.total_cmp(&a.0));
    for (px, nome, pintada) in &v {
        println!(
            "  {nome:22} {:8}  {px:9.0}  {:6.2}",
            if *pintada { "sim" } else { "NÃO" },
            px / DOBRA
        );
    }
}

/// Quanto um painel MOSTRA ao abrir — com a armação dele e a política de dobra do editor.
#[cfg(test)]
fn altura_de_abertura(id_painel: &str) -> f32 {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut h = 0.0;
    ph2d_editor_core::panel::with_registry(|reg| {
        let painel = reg
            .panels_mut()
            .iter_mut()
            .find(|p| p.manifest.id == id_painel)
            .unwrap_or_else(|| panic!("`{id_painel}` saiu do registo"));
        let arm = super::paineis_armados::TABELA
            .iter()
            .find(|a| a.painel == id_painel);
        let mut host = MockPanelHost::new();
        if let Some(a) = arm {
            (a.arma)(host.store_mut());
        }
        painel.populate(host.store_mut());
        ph2d_editor_core::screens::hero::pre_populate::marca_as_gavetas(host.store_mut());
        let (_, grupos) = ph2d_editor_core::widget::composto::medindo(|| {
            let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
        });
        h = conta(host.store(), &host.registos_da_ultima_pintura(), &grupos).altura;
        if let Some(a) = arm {
            (a.desarma)();
        }
    });
    h
}

/// ⛔⛔ **A ALTURA DE ABERTURA SÓ ENCOLHE.**
///
/// ⚠️⚠️ **Ela existe porque o modo de falha está DATADO.** Em 2026-06-24 o dono decidiu quais
/// secções do painel do Painter nascem abertas; medido por `git log -S`, o `Shape` nasceu a
/// **`06-25`**, o `Shape Ramp` a `06-25`, o `Symmetry` a `06-29` e o `Watercolor Paper` a `07-05`
/// — **todas ABERTAS, e nenhuma delas reviu a decisão**. O painel foi de caber para `3,1` ecrãs
/// sem que uma linha de política mudasse.
///
/// ⇒ *uma secção nova nascer aberta é o caminho por onde um painel volta a transbordar*, e é
/// silencioso: ninguém mede a altura de abertura de um painel.
///
/// ⛔ **Um painel que passe a mostrar MAIS reprova**; um que mostre MENOS também, com a outra
/// metade — *ela não é folga, é o sítio onde se escreve o número novo*.
const ALTURA_DE_ABERTURA: &[(&str, f32)] = &[
    // ⭐ `14 987 → 918` (o objecto impossível da fixtura) quando a política de dobra nasceu.
    ("inspector", 918.0),
    // ⭐ `2 709 → 1 529`: as quatro secções que chegaram DEPOIS da decisão do dono nascem
    //    recolhidas. ⛔ Ele não cabe, e o que falta é DECISÃO: o `Texture` (`450`) e o `Stroke`
    //    (`368`) são dele, e com tudo recolhido o painel mediria `736`.
    ("painter_layers", 1529.0),
    ("sculpt3d", 2097.0),
    ("tokens", 2866.0),
    ("vector", 1349.0),
    ("physics", 1293.0),
    ("audio_mixer", 1209.0),
];

#[test]
fn a_altura_de_abertura_de_um_painel_so_encolhe() {
    let mut subiram = Vec::new();
    let mut desceram = Vec::new();
    for (painel, tecto) in ALTURA_DE_ABERTURA {
        let h = altura_de_abertura(painel);
        if h > tecto + 0.5 {
            subiram.push(format!("{painel}: {h:.0} px contra {tecto:.0}"));
        } else if h < tecto - 0.5 {
            desceram.push(format!("{painel}: {h:.0} px contra {tecto:.0}"));
        }
    }
    assert!(
        subiram.is_empty(),
        "estes painéis passaram a MOSTRAR MAIS ao abrir:\n  {}\n\n\
         ⇒ quase de certeza uma secção nova nasceu ABERTA. ⛔ A cura é semeá-la recolhida \
         (`set_collapsed_if_unchosen`), nunca subir o número — foi assim que o painel do Painter \
         foi de caber para `3,1` ecrãs entre 06-24 e 07-05 de 2026.",
        subiram.join("\n  ")
    );
    assert!(
        desceram.is_empty(),
        "estes painéis mostram MENOS do que a catraca diz — ESCREVA o número medido:\n  {}",
        desceram.join("\n  ")
    );
}

/// SONDA TEMPORÁRIA — **os selectores de cor do app**: quantos, onde, e com que largura.
///
/// ⚠️ A pergunta do dono (2026-09-21) é *«os selectores de cor de todo o app precisam ser
/// padronizados»*, com um desenho: a amostra deve **encher a coluna do valor**, como a caixa de
/// marcar, em vez de ser um quadrado encostado à direita.
#[test]
#[ignore]
fn diag_os_selectores_de_cor_do_app() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_editor_core::panel::with_registry(|reg| {
        let mut total = 0usize;
        let mut larguras: std::collections::BTreeMap<i32, usize> =
            std::collections::BTreeMap::new();
        println!("\n  painel                  cores  larguras distintas");
        for painel in reg.panels_mut() {
            let id = painel.manifest.id;
            let mut host = MockPanelHost::new();
            let arm = super::paineis_armados::TABELA
                .iter()
                .find(|a| a.painel == id);
            if let Some(a) = arm {
                (a.arma)(host.store_mut());
            }
            painel.populate(host.store_mut());
            abre_tudo(host.store_mut());
            let (_, _g) = ph2d_editor_core::widget::composto::medindo(|| {
                let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
            });
            let pintados = host.registos_da_ultima_pintura();
            let store = host.store();
            if let Some(a) = arm {
                (a.desarma)();
            }
            let mut w_locais: std::collections::BTreeMap<i32, usize> =
                std::collections::BTreeMap::new();
            for (nid, r) in &pintados {
                // ⚠️ **A 1.ª régua lia `is_picker_swatch` + estado de picker e dava ZERO no
                //    Inspector** — que é exactamente o painel de onde veio o report. As swatches
                //    de tint guardam a cor em `widget_color` e não estão naquele conjunto.
                let e_cor = store.is_picker_swatch(*nid)
                    || store.widget_color(*nid).is_some()
                    || matches!(
                        store.get(*nid),
                        Some(ph2d_editor_core::interaction::InteractiveState::ColorPicker { .. })
                            | Some(
                                ph2d_editor_core::interaction::InteractiveState::BlenderPicker { .. }
                            )
                    );
                if e_cor {
                    *w_locais.entry(r.w.round() as i32).or_default() += 1;
                    *larguras.entry(r.w.round() as i32).or_default() += 1;
                    total += 1;
                }
            }
            if !w_locais.is_empty() {
                let n: usize = w_locais.values().sum();
                println!(
                    "  {id:22} {n:6}  {:?}",
                    w_locais
                        .iter()
                        .map(|(w, c)| format!("{w}px×{c}"))
                        .collect::<Vec<_>>()
                );
            }
        }
        println!("\n  TOTAL de selectores de cor pintados: {total}");
        println!("  larguras no app inteiro: {:?}", larguras);
    });
}

/// As larguras distintas com que o app pinta um selector de cor, com quantos há de cada.
#[cfg(test)]
fn larguras_dos_selectores_de_cor() -> std::collections::BTreeMap<i32, usize> {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut out = std::collections::BTreeMap::new();
    ph2d_editor_core::panel::with_registry(|reg| {
        for painel in reg.panels_mut() {
            let id = painel.manifest.id;
            let mut host = MockPanelHost::new();
            let arm = super::paineis_armados::TABELA
                .iter()
                .find(|a| a.painel == id);
            if let Some(a) = arm {
                (a.arma)(host.store_mut());
            }
            painel.populate(host.store_mut());
            abre_tudo(host.store_mut());
            let (_, _g) = ph2d_editor_core::widget::composto::medindo(|| {
                let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
            });
            let pintados = host.registos_da_ultima_pintura();
            let store = host.store();
            if let Some(a) = arm {
                (a.desarma)();
            }
            for (nid, r) in &pintados {
                let e_cor = store.is_picker_swatch(*nid)
                    || store.widget_color(*nid).is_some()
                    || matches!(
                        store.get(*nid),
                        Some(ph2d_editor_core::interaction::InteractiveState::ColorPicker { .. })
                            | Some(
                                ph2d_editor_core::interaction::InteractiveState::BlenderPicker { .. }
                            )
                    );
                if e_cor {
                    *out.entry(r.w.round() as i32).or_default() += 1;
                }
            }
        }
    });
    out
}

/// ⛔⛔ **O NÚMERO DE FORMAS DE UM SELECTOR DE COR SÓ ENCOLHE.**
///
/// ⛔ **Report do dono, 2026-09-21, com um DESENHO:** *«os seletores de cor de todo o app precisam
/// ser padronizados»*. Medido no mesmo dia: **`109`** selectores em **CINCO** larguras — `18`,
/// `24`, `32`, `120` e `268 px`.
///
/// ⭐ **A forma que ele desenhou já existia no app**, num painel só — o `authored`, gerado por
/// TABELA, com a amostra a encher a coluna do valor (`268`). Ela virou a porta
/// [`ph2d_editor_core::property_row::paint_color_row`], e as **seis** linhas de cor do Inspector
/// (tint · self tint · as duas das partículas · as duas do tween) passaram de `24` para a coluna.
///
/// ⚠️ **A catraca conta FORMAS e não sítios:** o alvo é *uma* forma, e cada wave que converte um
/// grupo tira uma linha daqui. ⛔ Uma forma NOVA reprova.
const LARGURAS_DE_COR: &[i32] = &[18, 32, 59, 120, 268];
// ⭐ O `59` é METADE da coluna do valor menos o vão — a célula do per-corner, que é `2 × 2` dentro
//   de uma linha. ⚠️ Ele **substituiu** um `35`: o nome comprido do bloco comia a coluna, e
//   encurtá-lo (ordem do dono) deu `68 %` mais alvo sem mover uma linha de disposição.

#[test]
fn as_formas_de_um_selector_de_cor_so_encolhem() {
    let medido = larguras_dos_selectores_de_cor();
    let vistas: Vec<i32> = medido.keys().copied().collect();
    let novas: Vec<&i32> = vistas
        .iter()
        .filter(|w| !LARGURAS_DE_COR.contains(w))
        .collect();
    assert!(
        novas.is_empty(),
        "formas NOVAS de selector de cor: {novas:?}\n  medido: {medido:?}\n\
         ⇒ um selector de cor novo passa pela porta `paint_color_row`, que o põe na coluna do \
         valor. ⛔ Acrescentar a largura aqui é desfazer a padronização que o dono pediu."
    );
    let mortas: Vec<&i32> = LARGURAS_DE_COR
        .iter()
        .filter(|w| !vistas.contains(w))
        .collect();
    assert!(
        mortas.is_empty(),
        "estas formas já não existem — APAGUE a linha, a catraca desceu: {mortas:?}\n  medido: {medido:?}"
    );
    // ⚠️ O piso: sem ele uma varredura partida lê zero selectores e as duas metades acima ficam
    //    trivialmente verdadeiras.
    let total: usize = medido.values().sum();
    assert!(
        total >= 100,
        "a varredura viu {total} selectores de cor e o app tem ~109 — ela partiu-se"
    );
}

/// ⭐⭐⭐ **A LINHA DE COR ENCHE A COLUNA DO VALOR** — o desenho do dono, medido.
#[test]
fn a_linha_de_cor_enche_a_coluna_do_valor() {
    let medido = larguras_dos_selectores_de_cor();
    // A coluna do valor nesta viewport, pela MESMA porta que desenha — nunca uma segunda conta.
    let interior = 268.0_f32;
    let row = ph2d_editor_core::widget::property_row_columns_for(
        0.0,
        interior,
        0.0,
        ph2d_tokens::ROW_H_PX,
        None,
        None,
    );
    let coluna = row.control.w.round() as i32;
    let n = medido.get(&coluna).copied().unwrap_or(0);
    assert!(
        n >= 6,
        "só {n} selectores de cor medem a coluna do valor ({coluna} px) — as seis linhas do \
         Inspector passam pela `paint_color_row`.\n  medido: {medido:?}"
    );
    // ⛔ **O CONTROLO**: a forma ANTIGA (o quadrado `SwatchSize::Sm`, `24 px`) não pode voltar.
    //    Sem ele este gate ficaria verde num app onde alguém acrescentasse seis barras E deixasse
    //    os quadradinhos onde estavam.
    let antigos = medido
        .get(&(ph2d_editor_core::widget::SwatchSize::Sm.px().round() as i32))
        .copied()
        .unwrap_or(0);
    assert_eq!(
        antigos, 0,
        "voltaram {antigos} selectores com a forma antiga (o quadrado de \
         `SwatchSize::Sm`).\n  medido: {medido:?}"
    );
}

/// ⭐⭐⭐ **A COLUNA DE UM BLOCO DE CORES SAI DE TODOS OS NOMES DELE.**
///
/// ⛔ **Report do dono, 2026-09-21:** *«quanto ao alinhamento precisamos melhorar em todos os
/// lugares»*. Se cada linha medisse a coluna com o próprio nome, duas amostras da mesma secção
/// começavam em `x` diferentes.
///
/// ⚠️⚠️ **Este gate nasceu de uma mutação SOBREVIVENTE** (pôr a coluna a sair de UM nome passava
/// `332` gates do painel) — e a 1.ª redacção dele media o `x` das amostras no painel, com o
/// **CONTROLO a reprovar**: nas larguras que o produto usa a coluna **satura** (os blocos `Tint` e
/// `Particles` caem os dois em `1 760,0`), logo a escolha dos nomes não é observável ali.
/// *Uma lei cuja consequência satura no regime do produto não se gateia pelo produto.*
///
/// ⇒ duas metades: a **medida**, que prova na PORTA que a largura do nome move a coluna (senão a
/// outra afirmaria sobre um parâmetro inerte), e a **textual**, que prova que o bloco recolhe
/// TODOS os rótulos.
#[test]
fn a_coluna_de_um_bloco_de_cores_sai_de_todos_os_nomes() {
    let col = |label_w: f32| {
        ph2d_editor_core::widget::property_row_columns_for(
            0.0,
            268.0,
            0.0,
            ph2d_tokens::ROW_H_PX,
            Some(label_w),
            None,
        )
        .control
        .x
    };
    // ⛔⛔ **O REGIME em que a lei existe está MEDIDO** (`diag_quando_o_nome_move_a_coluna`): a
    //    coluna do nome é `min(50 %, …)`, logo **só um nome mais largo do que METADE do painel a
    //    move**. Num painel de `268` os nomes de `20`, `60` e `120` px dão os TRÊS `x = 134`; o de
    //    `200` dá `182`. ⇒ *com os nomes curtos de hoje esta lei é INERTE, e ela morde no regime
    //    dos «nomes grandes» que o dono nomeou no mesmo report.*
    let curto = col(20.0);
    let comprido = col(200.0);
    assert!(
        (curto - comprido).abs() > 1.0,
        "a largura do nome não move a coluna ({curto:.1} contra {comprido:.1}) — sem isso a \
         metade de baixo afirma sobre um parâmetro inerte"
    );
    // ⚠️ E o CONTROLO do controlo: abaixo de meia largura ela de facto NÃO move, que é o que
    //    explica porque a 1.ª redacção deste gate media o painel e reprovava.
    assert!(
        (col(20.0) - col(120.0)).abs() < 0.5,
        "um nome abaixo de meia largura passou a mover a coluna — o regime medido mudou, e a nota \
         acima deixou de descrever o produto"
    );

    let fonte = include_str!("../../../ph2d-panel-inspector/src/sections/color_tint.rs");
    let agulha = "let rotulos: Vec<&str> = cores.iter().map(|(_, l, _)| *l).collect();";
    assert_eq!(
        fonte.matches(agulha).count(),
        1,
        "o `bloco_de_cores` deixou de recolher TODOS os rótulos do bloco — com um filtro a coluna \
         passa a ser do PRIMEIRO nome, e duas linhas da mesma secção caem em `x` diferentes assim \
         que o painel for estreito o bastante para a coluna deixar de saturar."
    );
}

/// SONDA TEMPORÁRIA — **onde a largura do NOME move a coluna do controlo?**
#[test]
#[ignore]
fn diag_quando_o_nome_move_a_coluna() {
    println!("\n  painel   nome=20  nome=60  nome=120  nome=200");
    for w in [180.0_f32, 220.0, 268.0, 336.0, 420.0, 600.0] {
        let col = |lw: f32| {
            ph2d_editor_core::widget::property_row_columns_for(
                0.0,
                w,
                0.0,
                ph2d_tokens::ROW_H_PX,
                Some(lw),
                None,
            )
            .control
        };
        println!(
            "  {w:6.0}  {:7.1}  {:7.1}  {:8.1}  {:8.1}   (larguras: {:.0}/{:.0}/{:.0}/{:.0})",
            col(20.0).x,
            col(60.0).x,
            col(120.0).x,
            col(200.0).x,
            col(20.0).w,
            col(60.0).w,
            col(120.0).w,
            col(200.0).w
        );
    }
}

/// ⭐⭐⭐ **O PER-CORNER É UMA LINHA DE PROPRIEDADE** — o nome à esquerda, os quatro cantos na
/// coluna do valor, com a ALTURA DE LINHA das outras amostras.
///
/// ⛔⛔ **Ordem do dono, 2026-09-21, depois de ver a versão com a prévia:** *«vamos tirar o preview
/// (rect maior) e no lugar colocar a label. Alinhar ao centro. Os 4 seletores de cor à direita com
/// a altura padrão dos outros seletores de cor»*.
///
/// ⚠️⚠️ **A premissa da versão anterior MORREU com a prévia.** A tolerância do
/// `no_row_paints_its_name_above_its_control` media o grupo em `~144 px` (`4 amostras + a PRÉVIA à
/// direita delas`) contra uma coluna de `~120`; sem a prévia ele cabe, e a entrada do
/// `color_tint.rs` **saiu** daquela lista. *A excepção não foi afrouxada — o que a causava foi
/// retirado.*
#[test]
fn o_per_corner_e_uma_linha_de_propriedade() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_editor_core::panel::with_registry(|reg| {
        let painel = reg
            .panels_mut()
            .iter_mut()
            .find(|p| p.manifest.id == "inspector")
            .expect("o inspector");
        let arm = super::paineis_armados::TABELA
            .iter()
            .find(|a| a.painel == "inspector")
            .expect("o inspector tem armação");
        let mut host = MockPanelHost::new();
        (arm.arma)(host.store_mut());
        painel.populate(host.store_mut());
        abre_tudo(host.store_mut());
        let (_, _g) = ph2d_editor_core::widget::composto::medindo(|| {
            let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
        });
        let pintados = host.registos_da_ultima_pintura();
        (arm.desarma)();
        let r = |id: NodeId| {
            pintados
                .iter()
                .find(|(n, _)| *n == id)
                .map(|(_, r)| *r)
                .unwrap_or_else(|| panic!("{id:?} não foi pintado"))
        };
        use ph2d_panel_inspector::ids as iid;
        let tl = r(iid::INSP_SPRITE_CORNER_TL);
        let tr_ = r(iid::INSP_SPRITE_CORNER_TR);
        let bl = r(iid::INSP_SPRITE_CORNER_BL);
        let br = r(iid::INSP_SPRITE_CORNER_BR);
        let tint = r(iid::INSP_SPRITE_TINT_SWATCH);

        // (1) A grelha continua a ser uma grelha — ela mapeia os cantos do quad.
        assert!(
            (tl.y - tr_.y).abs() < 0.5 && (bl.y - br.y).abs() < 0.5 && bl.y > tl.y,
            "as filas do 2×2 não estão alinhadas: TL {:.1} TR {:.1} · BL {:.1} BR {:.1}",
            tl.y,
            tr_.y,
            bl.y,
            br.y
        );
        assert!(
            (tl.x - bl.x).abs() < 0.5 && (tr_.x - br.x).abs() < 0.5 && tr_.x > tl.x,
            "as colunas do 2×2 não estão alinhadas"
        );

        // (2) ⭐ **A ALTURA É A DE LINHA** — a mesma da `Tint`, que é o que o dono pediu.
        for (nome, c) in [("TL", tl), ("TR", tr_), ("BL", bl), ("BR", br)] {
            assert!(
                (c.h - ph2d_tokens::ROW_H_PX).abs() < 0.5,
                "o canto {nome} mede {:.1} px de altura e a linha padrão é {:.1}",
                c.h,
                ph2d_tokens::ROW_H_PX
            );
        }

        // (3) ⭐ **NA COLUNA DO VALOR** — a grelha começa onde a barra da `Tint` começa.
        assert!(
            (tl.x - tint.x).abs() < 0.5,
            "a grelha começa em {:.1} e a barra da `Tint` em {:.1} — o per-corner deixou de \
             partilhar a coluna do valor com as vizinhas",
            tl.x,
            tint.x
        );
        // (4) E as duas colunas de células enchem a coluna do valor.
        let ocupado = tr_.x + tr_.w - tl.x;
        assert!(
            (ocupado - tint.w).abs() < 1.5,
            "as duas células ocupam {ocupado:.1} px e a coluna do valor tem {:.1}",
            tint.w
        );
        // ⛔ **O CONTROLO**: a célula NÃO é do tamanho de uma barra inteira — se fosse, o «2×2»
        //    seria uma coluna só e as duas metades acima passariam sobre outra disposição.
        assert!(
            tl.w < tint.w * 0.75,
            "a célula mede {:.1} e a barra {:.1} — isto já não é uma grelha de duas colunas",
            tl.w,
            tint.w
        );
    });
}

/// ⭐⭐ **O QUE SAIU DO NOME ESTÁ NO BALÃO** — a regra do dono, 2026-09-21.
///
/// *«Quanto aos nomes grandes precisamos reduzir, as dicas devem ser passadas para o mouse
/// Hover»*. ⛔ Encurtar sem o balão é perder a explicação: o `(vertex gradient)` do per-corner
/// dizia o que os quatro cantos FAZEM.
///
/// ⚠️ **O balão mora nos CONTROLOS e não no rótulo**, por medição: um rótulo de bloco não tem id
/// nem rect registado, logo não há onde o pendurar sem mecanismo novo — e a mão passa é sobre as
/// amostras.
#[test]
fn a_dica_que_saiu_do_nome_do_per_corner_esta_no_balao() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_editor_core::panel::with_registry(|reg| {
        let painel = reg
            .panels_mut()
            .iter_mut()
            .find(|p| p.manifest.id == "inspector")
            .expect("o inspector");
        let mut host = MockPanelHost::new();
        painel.populate(host.store_mut());
        use ph2d_panel_inspector::ids as iid;
        for (nome, id) in [
            ("TL", iid::INSP_SPRITE_CORNER_TL),
            ("TR", iid::INSP_SPRITE_CORNER_TR),
            ("BL", iid::INSP_SPRITE_CORNER_BL),
            ("BR", iid::INSP_SPRITE_CORNER_BR),
        ] {
            let balao = host.store().tooltip_for(id);
            assert!(
                balao.is_some_and(|t| !t.is_empty()),
                "o canto {nome} não tem balão — o nome do bloco foi encurtado e a explicação \
                 que ele carregava não foi para lado nenhum"
            );
        }
        // ⛔ **O CONTROLO**: a `Tint`, cujo nome nunca carregou explicação, NÃO ganha balão —
        //    senão este gate ficaria verde num app que pendura um balão em tudo.
        assert!(
            host.store()
                .tooltip_for(iid::INSP_SPRITE_TINT_SWATCH)
                .is_none(),
            "a `Tint` ganhou um balão — esta régua deixa de distinguir «a dica foi para o hover» \
             de «há balões em todo o lado»"
        );
    });
}

/// SONDA TEMPORÁRIA — **que linhas do Inspector são espremidas pelo próprio NOME.**
///
/// ⚠️ Medido em 2026-09-21 no per-corner: a coluna do nome é `min(50 %, …)`, logo um nome mais
/// largo do que METADE do painel **come a coluna do controlo** — ali isso valia `35` contra
/// `59 px` de amostra (`68 %` de alvo).
///
/// ⛔ **O discriminador NÃO é a LARGURA do controlo** — a 1.ª redacção usou-a e acusou `126`
/// linhas, todas a `72 px`, que é o **PISO do campo**: aquelas são linhas de DOIS campos a
/// partilhar a coluna, e isso é o desenho, não um aperto. O discriminador é **ONDE a coluna
/// COMEÇA**: um nome que não aperta deixa-a no mesmo `x` de todas as outras.
#[test]
#[ignore]
fn diag_que_linhas_o_nome_espreme() {
    let nomes = nomes_do_inspector();
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_editor_core::panel::with_registry(|reg| {
        let painel = reg
            .panels_mut()
            .iter_mut()
            .find(|p| p.manifest.id == "inspector")
            .expect("o inspector");
        let arm = super::paineis_armados::TABELA
            .iter()
            .find(|a| a.painel == "inspector")
            .expect("armação");
        let mut host = MockPanelHost::new();
        (arm.arma)(host.store_mut());
        painel.populate(host.store_mut());
        abre_tudo(host.store_mut());
        let (_, grupos) = ph2d_editor_core::widget::composto::medindo(|| {
            let _ = host.medindo_a_pintura_do_registo(painel, VIEWPORT);
        });
        let pintados = host.registos_da_ultima_pintura();
        let store = host.store();
        let celula: std::collections::BTreeSet<u64> =
            grupos.iter().flatten().map(|n| n.0).collect();

        // ⭐ A coluna SEM aperto lê-se do PRODUTO: a barra da `Tint`, cujo nome é curto.
        let norma_x = pintados
            .iter()
            .find(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_SPRITE_TINT_SWATCH)
            .map(|(_, r)| r.x)
            .expect("a `Tint` é a referência da coluna");

        // ⛔⛔ **Só o controlo MAIS À ESQUERDA de cada faixa**, e a 2.ª redacção não o fazia: o
        //    segundo campo de uma linha `X`/`Y` está `+48 px` à direita **por desenho**, e ele
        //    enchia a lista de falsos. *Um par de componentes é UMA propriedade.*
        let mut por_faixa: std::collections::BTreeMap<i32, (f32, u64)> =
            std::collections::BTreeMap::new();
        let mut espremidas: Vec<(f32, String)> = Vec::new();
        for (id, r) in &pintados {
            if celula.contains(&id.0) || r.h > ph2d_tokens::ROW_H_PX + 1.0 {
                continue;
            }
            if !matches!(
                store.get(*id),
                Some(
                    ph2d_editor_core::interaction::InteractiveState::NumberInput { .. }
                        | ph2d_editor_core::interaction::InteractiveState::TextInput { .. }
                        | ph2d_editor_core::interaction::InteractiveState::Dropdown { .. }
                        | ph2d_editor_core::interaction::InteractiveState::Slider { .. }
                        | ph2d_editor_core::interaction::InteractiveState::Checkbox { .. }
                )
            ) {
                continue;
            }
            let e = por_faixa.entry(r.y.round() as i32).or_insert((r.x, id.0));
            if r.x < e.0 {
                *e = (r.x, id.0);
            }
        }
        for (x, id) in por_faixa.values() {
            if *x > norma_x + 1.0 {
                let lit = nomes.get(id).cloned().unwrap_or_default();
                if !lit.is_empty() {
                    espremidas.push((x - norma_x, lit));
                }
            }
        }
        (arm.desarma)();
        espremidas.sort_by(|a, b| b.0.total_cmp(&a.0));
        espremidas.dedup_by(|a, b| a.1 == b.1);
        println!("\n  a coluna sem aperto começa em x = {norma_x:.0}");
        println!(
            "  linhas empurradas pelo próprio nome: {}\n",
            espremidas.len()
        );
        println!("  empurra  id");
        for (d, lit) in espremidas.iter().take(30) {
            println!("  {d:7.0}  {lit}");
        }
    });
}
