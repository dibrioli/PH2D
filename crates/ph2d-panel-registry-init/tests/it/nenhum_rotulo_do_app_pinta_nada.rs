//! ⭐⭐⭐ **A VARREDURA DAS ELISÕES — TODO painel, TODO rótulo, e a pergunta da PRÓXIMA LÍNGUA.**
//!
//! # ⛔⛔ O buraco que ela fecha
//!
//! O HR-15 fechou com **30 censos** e os trinta lêem o **FONTE**: eles respondem *«esta palavra vem
//! da tabela?»*. Nenhum lê o **ECRÃ**, e a pergunta que o ecrã faz é outra: ***ela COUBE?*** Medido
//! em 2026-09-18, a coluna dos nomes do Audio Mixer estava verde nos trinta censos e cortava
//! `Depth` e `Return` **na língua em que o app ship** — e quatro botões de silenciar pintavam
//! **NADA**, com a foto do dono a mostrar uma fileira de `[…]`.
//!
//! ⇒ este ficheiro pinta **cada painel do REGISTO** (a população, nunca uma lista escrita à mão) e
//! lê o [`ph2d_editor_core::text_elide::elisao`], que regista cada rótulo medido com o orçamento,
//! a fonte e o peso.
//!
//! # ⭐⭐ Uma pintura em INGLÊS responde pelas DUAS línguas
//!
//! O idioma de teste é uma **função pura** do inglês ([`ph2d_i18n::pseudo::deforma`]), logo a
//! palavra deformada re-mede-se no mesmo orçamento, na mesma fonte e no mesmo peso — ⛔ **sem
//! mexer no `PH2D_LANG` do processo**, que tornaria esta suíte mais um membro da família de flakes
//! de fan-out (o `tr` lê o idioma de um `OnceLock`, uma vez por processo).
//!
//! ⚠️ **A previsão segura a CAIXA e faz crescer a PALAVRA, logo ela é CONSERVADORA:** onde a caixa
//! **deriva** do que pinta — a coluna do Audio Mixer desde 18/09 —, a resposta real é melhor do que
//! esta. *Uma previsão pessimista é a certa para uma lei que diz «nunca».*
//!
//! # A MEDIÇÃO desta varredura (2026-09-18, três viewports)
//!
//! | grandeza | `-p` sozinho (24 painéis) | árvore inteira (28) |
//! |---|---:|---:|
//! | rótulos medidos | **3 146** | **4 022** |
//! | a pintar **NADA** em inglês | **0** | **0** |
//! | a pintar **NADA** no idioma de teste | **0** | **0** |
//! | cortados (`prefixo…`) em inglês, ANTES da cura | 15 | 17 |
//! | cortados hoje (número · frases · chip · colunas de rótulo) | **8** | **8** — `CORTADOS_HOJE` |
//! | cortados no idioma de teste | ~130, em 16 painéis | — |
//!
//! ⚠️ **Um painel pintado com o estado de FÁBRICA mostra o estado VAZIO dele** (o Inspector sem
//! selecção mede **um** rótulo). A varredura mede o que um painel pinta **sozinho**; o piso é
//! GLOBAL de propósito, porque um piso por painel seria uma lista de números escritos à mão sobre
//! populações que dependem do documento.
//!
//! # ⭐⭐⭐ E desde 2026-09-19 há uma SEGUNDA passagem: o painel com um DOCUMENTO na mão
//!
//! A frase acima era, até esse dia, uma **cegueira declarada**: o Inspector tem **28** secções que
//! só existem com um objecto seleccionado, e nenhuma régua de largura deste repo as via. A fixtura
//! [`super::o_inspector_armado`] arma as 28 e a varredura pinta o painel **duas** vezes por
//! viewport — vazio e armado —, com o `Achado::armado` a dizer qual.
//!
//! | a passagem ARMADA (1.ª corrida, três viewports) | |
//! |---|---:|
//! | rótulos que pintavam **NADA** | **8** (as unidades `px` e `1/s`) |
//! | cortados (`prefixo…`) | **24** |
//! | curados no mesmo dia | **21** (19 por seis portas + 2 por ordem do dono) |
//! | por curar, nomeados | **3** no Inspector + **2** na Hierarquia — `A_PASSAGEM_ARMADA_AINDA_CORTA` |
//!
//! ⚠️ **Os `5` que ficam cortam texto que o ARTISTA escreveu** (o nome de uma âncora, de um sinal,
//! de uma propriedade de script, de dois objectos) — e a lista traz o número de cada caixa para que
//! isso possa ser conferido em vez de acreditado.
//!
//! # ⭐⭐⭐ E em 2026-09-19 a mesma pergunta foi feita ao PAINEL DE PARAMS DO MOTION
//!
//! Ele estava na lista dos medidos vazios com a justificação *«precisa de um grafo»* — e isso era
//! **uma ausência afirmada sem olhar a API**: o painel lê um `ParamsSnapshot` publicado numa porta
//! `thread_local`, e um snapshot são DADOS, não um motor. A fixtura armava uma fileira de **cada
//! uma das 13 espécies**, com os rótulos **derivados** das `828` entradas `node.*.param.*` do
//! catálogo (`444` distintas), ordenadas pelo que PINTAM.
//!
//! ⛔⛔ **E em 2026-09-20 ela SAIU, porque o painel saiu** (`line/motion-value`, ordem do dono de
//! 17/09: os params vivem no CARTÃO). ⚠️ *A medição perdeu a SUPERFÍCIE, não o sujeito* — ela
//! pintava através do painel, e o `set_current_params` era a porta dele. O que fica registado é a
//! forma do achado, que vale para o próximo painel declarado vazio; ⏳ e a dívida é medir os mesmos
//! `828` rótulos no CARTÃO, cuja porta (`ph2d_panel_motion_graph::nome_cabe_na_capsula`) já existe
//! e já é usada sobre os nomes de TIPO pela `ph2d_app_motion::motion_param_reach_tests`.
//!
//! | a 1.ª corrida com o Motion armado | |
//! |---|---:|
//! | rótulos que pintam **NADA** | **0** |
//! | cortados (`prefixo…`) | **16** |
//! | curados no mesmo dia | **4** (a grelha de opções mede as palavras nas DUAS dimensões) |
//! | os `12` que ficam | **NÃO são dívida** — `FORA_POR_DECISAO_DO_DONO` |
//!
//! ⛔⛔⛔ **E a decisão chegou com o smoke desta mesma passagem** (Enio, 2026-09-19): *«as
//! propriedades dos nós não existirão no painel lateral em versões futuras. Apenas nos próprios
//! nós dos grafos.»* ⇒ os doze cortes são reais e medidos, e curá-los seria trabalho sobre uma
//! superfície que vai sair. ⚠️ *Uma linha que ninguém deve pegar tem de o dizer no NOME da lista
//! onde mora* — a armação fica, o censo de obsolescência continua a exigir que cada linha
//! descreva um corte real, e no dia em que o painel sair as doze ficam obsoletas e o gate manda
//! apagá-las.
//!
//! ⛔⛔ **E a terceira família achou uma LEI que a cura óbvia violava:** trocar o pintor da queixa
//! por um que QUEBRA fez reprovar o `the_motion_chrome_never_gives_a_row_label_a_wrap_budget`, cuja
//! razão é um report do dono de 2026-08-30 — no chrome do Motion uma linha de `22 px` que quebra
//! derrama a segunda metade por cima da entrada seguinte. *A lei é mais velha e está medida.*
//!
//! ⚠️ **A passagem armada NÃO alimenta o censo da tabela de strings** — ela põe no painel texto do
//! DOCUMENTO (`Hero`, `Enemy`, `Closed`), que a tabela não sabe produzir e nem devia. A razão está
//! escrita no filtro daquele teste.

use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::text_elide::elisao::Medido;
use ph2d_editor_core::text_elide::largura_da_reticencia;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::pseudo;
use ph2d_text::TextSystem;
use ph2d_ui_testkit::MockPanelHost;

/// ⭐⭐⭐ **A ESCADA: cada degrau e um par (janela, largura das colunas).**
///
/// ⛔⛔ **A redaccao anterior desta constante estava ERRADA sobre o que media, e a frase
/// dela dizia-o:** *"a largura do encaixe sai do viewport, e um rotulo que cabe a 1920 pode nao
/// caber a 1280"*. ⛔ Medido em 2026-09-19, os cinco cortes conhecidos da varredura dao
/// **exactamente o mesmo numero** nas tres janelas — a largura de uma coluna docada vem do
/// TOKEN (`ChromeBands::DEFAULT`: `308` a esquerda, `304` a direita) e a janela so decide ONDE ela
/// fica. As tres janelas compram a largura do encaixe do FUNDO (o timeline, que e o que sobra
/// entre as colunas) e nada mais.
///
/// ⚠️⚠️ **E a largura de fabrica nao e a que o artista tem.** A mesma constante dizia que a
/// coluna apertada a mao *"vive no `~/.ph2d/layout.txt`, fora do repo"* e mandava essa metade para
/// "o gate de coluna de cada painel" — que nao existe para 24 painéis. Lido o ficheiro do
/// dono nesse dia, a coluna da ESQUERDA estava no **minimo (`220`)** em **cinco dos seis** espacos
/// de trabalho, contra os `308` do token: `88 px` que nenhuma regua desta casa olhou.
///
/// ⛔ **O degrau nao SAI do ficheiro dele** — ele vive fora do repositorio e nao
/// existe noutra maquina, logo um gate que o lesse mediria coisas diferentes em cada sitio. O
/// degrau estreito e o `PANEL_MIN_W_PX`, que e a LEI (o piso ate onde a borda encolhe, com o
/// `DOCK_W_MIN` a le-lo); o ficheiro do dono serve para dizer QUAL degrau importa.
///
/// ⚠️ **Uma janela so no degrau estreito, de proposito:** medido acima, a janela nao muda a
/// largura de uma coluna, logo tres janelas la seriam a mesma medicao tres vezes.
const ESCADA: &[(&str, f32, f32, f32)] = &[
    // (nome, largura da janela, altura, largura das DUAS colunas; `0` = as de fabrica)
    ("1366", 1366.0, 1024.0, 0.0),
    ("1920", 1920.0, 1080.0, 0.0),
    ("1280", 1280.0, 800.0, 0.0),
    // ⭐⭐ O degrau em que o dono de facto trabalha.
    (
        "colunas no minimo",
        1366.0,
        1024.0,
        ph2d_tokens::PANEL_MIN_W_PX,
    ),
    // ⛔⛔ **As `14` entradas do `motion_params` saíram em 2026-09-20 — e NÃO por serem curadas:**
    //    o painel de params do Motion saiu do app (`line/motion-value`, ordem do dono de 17/09),
    //    logo elas deixaram de descrever coisa nenhuma. ⭐ Quem as apanhou foi o **censo de
    //    obsolescência** desta catraca (*«declarado 16, mede 0»*), que é exactamente o que ele
    //    existe para fazer — *uma lista de dívida sem censo vira licença.*
    //    ⏳ Os `6` cortes de coluna de nome e o chip de `24 px` para `RGB` eram do PRODUTO, e o
    //    cartão herda-os se pintar as mesmas fileiras: a medição no cartão está na dívida NOMEADA
    //    do [`super::paineis_armados`].
];

/// As bandas de um degrau: `0` quer dizer *as de fabrica*.
fn bandas(col_w: f32) -> ph2d_editor_core::screens::layout::ChromeBands {
    let base = ph2d_editor_core::screens::layout::ChromeBands::DEFAULT;
    if col_w <= 0.0 {
        return base;
    }
    ph2d_editor_core::screens::layout::ChromeBands {
        left_dock_w: col_w,
        right_dock_w: col_w,
        ..base
    }
}

/// ⛔ **Piso de população.** *Sem ele, uma varredura que deixasse de pintar leria zero cortes e
/// passaria por aprovação* — a forma exacta que este repo já pagou com o censo por prefixo de nome.
///
/// ⛔⛔⛔ **A POPULAÇÃO DEPENDE DAS FEATURES, e isso apanhou este gate no dia em que ele nasceu.**
/// Quatro painéis (`flip`, `flip_frames`, `painter_layers`, `wet_tuning`) **não** estão no `default`
/// desta crate — eles chegam pelo shell. ⇒ `cargo test -p ph2d-panel-registry-init` mede **24**
/// painéis e `3 146` rótulos; a árvore inteira, com a unificação de features, mede **28** e
/// **`4 022`** — e traz **dois cortes a mais**, que a primeira lista não tinha.
///
/// ⚠️ *Uma crate testada sozinha é testada num mundo que o produto não habita* (`CLAUDE.md` §2).
///
/// ⛔⛔⛔ **E O PISO COBRIA OS DOIS MUNDOS, O QUE O FAZIA NÃO AFIRMAR NADA SOBRE NENHUM.** A
/// redacção anterior punha-o no número MENOR *«de propósito, para o gate passar das duas
/// maneiras»* — e a frase seguinte, escrita pelo mesmo autor, já dizia porque isso estava errado:
/// *«um piso posto no menor dos dois deixa de afirmar o que acontece no maior — e é lá que o app
/// de facto corre»*. A integração de 2026-09-20 cobrou-a: os três painéis que só a árvore inteira
/// liga (`flip`, `painter_layers`, `wet_tuning`) cortavam **7** rótulos que as duas catracas
/// declaravam a ZERO, e as duas fecharam **verdes** em todas as corridas `-p` da linha.
/// ⇒ *a régua estava calibrada num âmbito e julgada noutro*, que é a forma exacta que
/// [`feedback_an_operation_count_is_not_a_profile_and_the_build_profile_decides_the_number`]
/// descreve uma volta abaixo.
///
/// ⭐ **Hoje o piso é o do âmbito em que o app CORRE, e a corrida pobre reprova ALTO** em vez de
/// medir menos em silêncio. Medido em 2026-09-20, com `--cargo-profile ci-test`:
///
/// | âmbito | painéis | rótulos |
/// |---|---:|---:|
/// | `-p ph2d-panel-registry-init` (features de omissão) | `24` | `11 375` |
/// | `--workspace` (unificação de features — o que o `ship.sh` e o CI correm) | **`28`** | **`12 545`** |
///
/// ⚠️ O piso de medições fica **entre os dois** (`12 000`): abaixo do que a árvore inteira produz,
/// acima do que a pobre produz. *Um piso que ambos os âmbitos passam não é um piso, é um adorno.*
const PISO_DE_MEDICOES: usize = 12_000;

/// ⛔ E o piso de PAINÉIS: uma varredura que registasse zero painéis passaria os quatro gates.
/// ⚠️ Ele é **28** — a população da árvore inteira — pela razão da irmã acima: com `24` a corrida
/// `-p` respondia sobre um app a que faltam três painéis, e respondia VERDE.
/// ⛔⛔ **Desceu de `28` para `27` na integração de 2026-09-20, e NÃO é a catraca a ceder:** o
/// painel de params do Motion **saiu do app** (`line/motion-value`, ordem do dono de 17/09 — os
/// params passaram a viver no cartão). *A população encolheu; o piso descreve-a.* ⚠️ Um piso que
/// ficasse em `28` reprovaria para sempre sobre produto correcto, e subi-lo de volta só é honesto
/// no dia em que um painel NOVO entrar no `default` do registo.
const PISO_DE_PAINEIS: usize = 27;

/// ⭐⭐ **A DÍVIDA NOMEADA — o que sai cortado HOJE, em inglês.** Ela só **encolhe**: uma linha
/// daqui sai quando alguém curar o rótulo, e o censo de obsolescência **reprova** se ela ficar a
/// descrever um corte que já não acontece.
///
/// ⚠️ **Um corte não é sempre um defeito** (o nome de um ficheiro, o nome que o artista escreveu),
/// e desde 19/09 as **duas** que sobram são as duas espécies que NÃO são dívida — uma **decisão de
/// produto** (a tira do master) e uma **DEMONSTRAÇÃO** (a régua de largura do laboratório, que
/// existe para mostrar um rótulo a ser espremido). ⇒ toda linha que aqui entrar a partir de agora
/// diz de que espécie é, com o número; *sem isso a lista deixa de ser dívida e passa a licença.*
const CORTADOS_HOJE: &[(&str, &str)] = &[
    // ⛔ O nome da faixa mestra no Audio Mixer com o dock estreito — **declarado** em 18/09: a
    //    tira do master mede menos do que a palavra pede, e alargá-la é decisão de produto.
    ("audio_mixer", "Master"),
    // ✅ **Os três VALORES cortados (`+0.00 EV`, `0.500`, `1.500`) SAÍRAM desta lista em 18/09** —
    //    a caixa de número contava a borda do stepper DUAS vezes (`56 → 24 px` de orçamento) e hoje
    //    conta uma (`32`). Quem os apagou daqui foi este gate: o censo de obsolescência acusou as
    //    três linhas como já não descrevendo corte nenhum. Mecanismo:
    //    `ph2d-editor-core/tests/it/a_caixa_de_numero_nao_corta_o_numero.rs`.
    // ✅ **Os dois do `flip_frames` (`Linear`, `No Cycle`) SAÍRAM em 18/09** — e eram a prova de que
    //    uma lista capturada com `-p <crate>` sozinho está incompleta por CONSTRUÇÃO (aqueles dois
    //    painéis não estão no `default` desta crate). O chip deles era o literal `84 px`, com `46`
    //    de invólucro; hoje ele DERIVA da lista que oferece. ⚠️ E o censo só via metade do defeito:
    //    `Ping-Pong` (`65,5`) e `Ease In-Out` (`72,9`) vivem nas mesmas listas e nunca foram
    //    pintados por omissão. Mecanismo: `ph2d-panel-flip-frames/src/toolbar_plan.rs::chip_w`.
    // ✅ **O `PingPong` da barra da timeline SAIU em 19/09**, e por DUAS razões que chegaram
    //    juntas: o dono unificou a grafia (`Ping-Pong`, a que os outros quatro sítios do app já
    //    usavam) e a coluna de rótulo dos dez toggles deixou de ser o literal `52 px` — ela mede a
    //    LISTA (`60,45` em inglês, `91,34` no idioma de teste). ⚠️ E o censo só via UM dos dez:
    //    no idioma de teste **seis** daqueles rótulos estouravam a coluna, e nenhuma das duas leis
    //    desta varredura pergunta isso — elas perguntam se algo pinta NADA. Mecanismo:
    //    `ph2d-panel-timeline/tests/it/a_coluna_do_toggle_mede_a_lista.rs`.
    // ✅ **O `Line / Neighbors` do Grid Snap SAIU em 19/09** — a coluna da secção *Inspect* era o
    //    literal `80,0` e aquele rótulo mede `94,48`. Hoje ela mede a LISTA dos quatro
    //    (`ph2d-editor-core/src/grid_snap/inspect.rs`), e as linhas de sonda partilham-na — antes
    //    elas tinham uma **segunda** coluna, de `70`, que nenhuma régua textual desta casa via.
    // ✅ **Os TRÊS chips da galeria saíram em 19/09** (`Float` · `Color` · `filter`), e os dois
    //    mecanismos são os que esta varredura já cobrou noutros painéis:
    //    · o chip de espécie do editor de variantes era **`45 %` do que sobrava da linha** e passou
    //      a medir a FAMÍLIA (`ph2d-editor-core/src/widget/variant_editor.rs`) — e o censo só via
    //      dois dos seis, porque ele mede a opção ESCOLHIDA;
    //    · a pílula `filter` recebia o respiro de uma caixa de rótulo **por cima** do recuo que ela
    //      já tem, logo pagava-o DUAS vezes (`24,26` px de palavra em `12,13` de orçamento). ⚠️ O
    //      mesmo defeito estava vivo em **todo chip da secção *Tags* do Inspector** e em **seis dos
    //      sete selos da Hierarquia**, e esta varredura **não podia vê-los**: um painel de fábrica
    //      não tem objecto seleccionado. Mecanismo: `Tag::label_budget` / `Tag::width_for`.
    // ⛔ **A régua de largura do laboratório — e ela é a DEMONSTRAÇÃO, nunca dívida por curar.**
    //    A §2 daquele painel chama-se *«the chosen design, squeezed»* e desenha a MESMA linha a
    //    `268` · `184` · `140` · `110` px, com um rótulo comprido de propósito. A `110` a coluna do
    //    rótulo fica com `70,00` e `Geometry Offset` mede `95,16` ⇒ ele **tem** de aparecer
    //    cortado: *é isso que a régua existe para mostrar*. Nas outras três larguras ele cabe.
    //    ⚠️ Curar esta linha seria apagar a medição que o painel foi construído para fazer.
    ("widget_lab", "Geometry Offset"),
    // ✅ **As três LEGENDAS de prosa saíram em 19/09** — a do cabeçalho da galeria (`290,76` px numa
    //    coluna de `268`) e as duas da bancada (`399,79` e `386,58` em `384`). Elas eram elididas a
    //    UMA linha; hoje QUEBRAM, que é a mesma cura das duas frases de estado vazio do produto em
    //    18/09. ⚠️ E as três devolvem a ALTURA ao chamador: sem isso a 2.ª linha escreveria por
    //    cima do risco do cabeçalho e da fileira seguinte da bancada.
];

/// ⭐⭐⭐ **O QUE O PONTO CEGO ESCONDIA — a dívida do Inspector com um DOCUMENTO na mão.**
///
/// Esta lista nasceu em 2026-09-19, no dia em que a varredura passou a pintar o Inspector armado
/// ([`super::o_inspector_armado`]). ⚠️ **Ela não é uma tolerância nova: é uma população que nunca
/// tinha sido medida** — o painel de fábrica não tem objecto seleccionado, e as **28** secções
/// condicionais dele estavam, por construção, fora de toda régua de largura deste repo.
///
/// **A primeira corrida acusou `24` cortes; `16` foram curados no mesmo dia**, por três
/// mecanismos e não por 16 remendos:
/// - a UNIDADE de um campo de número passou a ser tudo-ou-nada (`px` e `1/s` pintavam **NADA**);
/// - o grupo segmentado passou a dar a cada peça o que a PALAVRA dela pede (`segmented_row_widths`)
///   e três cópias locais da disposição passaram pela porta;
/// - os avisos das secções — **seis cópias byte a byte** — viraram uma porta que QUEBRA.
///
/// ⭐⭐⭐ **E em 2026-09-19 a dívida do Inspector foi de `6` para `3`, com TRÊS curas estruturais:**
/// - a **fileira de botões** passou a medir as PALAVRAS ([`ph2d_editor_core::widget::segment_rects_for`]):
///   em partes iguais `x Remove Transition` recebia `118 px` e saía `x Remove Transi…` **com a
///   fileira a caber inteira** — *uma média não é um MÁXIMO*, a mesma lei do grupo segmentado, agora
///   na família do `Button` e em **13** sítios;
/// - a coluna de nomes da secção da **roldana** era `font × 5,0` — *cinco alturas de letra*, escrita
///   em TRÊS sítios, com um deles a dizer *«same label column as the Rope row»* — e passou a medir a
///   LISTA das três palavras que pinta (`Mounted On` · `Gear` · `Rope`);
/// - a linha de **órfão** do cartão de instância era MEDIDA a quebrar (`text_h`) e PINTADA a cortar
///   (`paint_text` elide desde 06/09) ⇒ *um contentor medido por uma regra e preenchido por outra*.
///
/// ⛔⛔ **Os `5` que ficam NÃO são dívida da mesma espécie, e cada linha di-lo:** `3` deles cortam
/// **texto que o ARTISTA escreveu** (o nome de uma âncora, o nome de um sinal, o nome de uma
/// propriedade de script) numa caixa cuja largura é a que a lei da linha de propriedade dá, e `2`
/// são nomes de objecto numa linha de árvore. *Um corte não é sempre um defeito* — o que seria
/// defeito é a CAIXA, e cada linha traz o número dela para que isso possa ser conferido.
///
/// ⚠️ **E o número de uma linha destas ENVELHECE:** o `hand_right` estava aqui como `26 px` e mede
/// `57,1`, e o resumo do timer estava como `128` e mede `147,4` depois de a coluna dele passar a
/// sair da lista. *Uma dívida com o número errado é pior do que uma sem número: ela convida a curar
/// o que já mudou.* ⇒ re-meça antes de pegar uma destas linhas.
///
/// ⚠️ **Cada linha diz o NÚMERO e o MECANISMO**, e a lista **só encolhe** — o censo de
/// obsolescência abaixo reprova quem deixar de descrever um corte.
const A_PASSAGEM_ARMADA_AINDA_CORTA: &[(&str, &str, &str)] = &[
    // ⭐⭐⭐ **AS QUATRO DA INTEGRAÇÃO DE 2026-09-20.** Elas apareceram no dia em que as DEZ portas
    //    novas do Inspector (`action_trigger` · `counter_watch` · `emitter` · `hud` ·
    //    `path_follow` · `ray` · `sequence` · `shake` · `tween` · `weapon`) passaram a ser
    //    ARMADAS por esta fixtura — antes delas a varredura media aquelas secções VAZIAS.
    // ⚠️ *Elas não são cortes novos do produto: são cortes que ninguém conseguia ver.*
    // ⭐⭐⭐ **A entrada do HUD SAIU em 2026-09-22, e a razão é a cura que ela própria prescrevia:**
    //    *«a cura é a frase QUEBRAR, como a do áudio — outra wave»*. A wave foi a dos avisos: esta
    //    frase era pintada por uma das **dez cópias** do pintor de aviso, todas a chamar o
    //    `paint_text` (que CORTA) em vez do `paint_text_block` (que QUEBRA). Com a porta única
    //    ela quebra em duas linhas e **deixa de ser um corte**.
    // ⛔ *A dívida não foi silenciada — ela foi PAGA*, e o censo de obsolescência foi quem o disse.
    // ⭐⭐ **O `Authored`/`Counter` do HUD SAÍRAM em 2026-09-23** — o segmentado passou pela porta da
    //    ESCOLHA, que o mediu a não caber numa fileira ao lado do nome e o pôs em PALETA, onde cada
    //    peça leva a largura NATURAL da palavra. Quem os apagou daqui foi o censo de obsolescência.
    // ⭐⭐⭐ **A PROVENIÊNCIA, cortada desde 2026-09-22 — e é o PREÇO de a secção alinhar.**
    //
    // ⛔ A `RENDER SOURCE` punha o nome POR CIMA do valor, logo a ranhura tinha a largura do painel
    //    inteiro; hoje ela é uma linha de propriedade como as vizinhas (report do dono: *«várias
    //    seções muito confusas e desorganizadas»* + *«o alinhamento precisa melhorar em todos os
    //    lugares»*), e o valor vive na coluna do controlo — `112 px` no encaixe dele.
    // ⚠️ **O nome da folha é texto do ARTISTA** (ele escolheu-o no Aseprite): encurtá-lo não é uma
    //    saída, e a lei da casa para isso é o BALÃO — que este pintor passou a declarar.
    (
        "inspector",
        "Hand-packed \u{b7} hero \u{b7} idle_0",
        "ranhura de proveniencia \u{b7} 112,0 px \u{b7} nome de folha que o artista deu (tem balao)",
    ),
    // ⭐ **E a irmã do TIMER saiu pela MESMA cura, no mesmo commit** (*«mesma cura que o irmão do
    //    HUD — quebrar»*). As duas eram avisos DERIVADOS de números, sem versão curta possível; o
    //    que faltava não era encurtá-los, era a porta que os quebra.
    // ✅ **As DUAS fileiras de MARCAR saíram em 2026-09-19, por ordem do dono.** O controlo de uma
    //    caixa precisa de `18 px` e a coluna do nome fica com `174`, porque ela é medida para as
    //    fileiras de CAMPOS da mesma secção — e `Center (makes it a 9-slice Region)` pedia `~190`.
    //    Postas as três saídas à frente dele (deixar cortar · encurtar · a caixa deixa de seguir o
    //    alinhamento do meio), ele escolheu **encurtar**: hoje são `Center` e
    //    `Show anchors at runtime`, com a consequência de cada uma no BALÃO. ⚠️ Quem as apagou
    //    daqui foi o censo de obsolescência deste ficheiro. Metade que PÕE:
    //    `ph2d-panel-inspector/tests/it/as_caixas_que_encurtaram_guardam_a_explicacao.rs`.
    // ✅ **E TRÊS saíram no mesmo dia por CURA ESTRUTURAL** — o preâmbulo acima diz qual foi cada
    //    uma: `x Remove Transition` (a fileira mede as palavras), `Mounted On` (a coluna mede a
    //    lista) e `• AudioSource2D — was on "footsteps"` (a frase passou a QUEBRAR, que é como a
    //    altura dela já era medida).
    //
    // ── O que fica: texto que o ARTISTA escreveu, numa caixa que a lei da linha dá ────────────
    //
    // O chip que escolhe em que âncora do PAI este objecto se monta. A coluna do nome desta secção
    // é medida sobre as TRÊS palavras dela (`Rides Parent Anchor` · `Always show anchors` ·
    // `Show anchors at runtime`) e o chip fica com o resto, menos o recuo e o chevron dele.
    // ⚠️ **Alargar o chip aqui ESTREITA a coluna das duas caixas de marcar**, que o dono acabou de
    //    mandar encurtar — é uma troca entre um rótulo do programa e um nome do artista, e a lei da
    //    casa manda cortar o segundo.
    (
        "inspector",
        "hand_right",
        "chip de escolha · 57,1 px · nome que o artista deu a' ancora",
    ),
    // O resumo de um timer, na coluna da DIREITA da lista. ⭐ Desde 19/09 essa coluna sai da LISTA
    // dos resumos e não de `w/2` (o nome deixou de poder pintar por cima dela), com tecto de
    // `0,55 × w` — e o tecto é sobre o NOME, que é por onde o artista acha a linha.
    // ⚠️ O que fica cortado é o fim: `→ respawn_done`, o nome do SINAL que o artista escreveu.
    (
        "inspector",
        "2.50s \u{b7} repeats \u{b7} \u{2192} respawn_done",
        "lista · 147,4 px · acaba no nome do sinal, que e' do artista",
    ),
    // A linha de uma propriedade que o ficheiro `.luau` deixou de declarar. ⚠️ A frase compõe
    // `<nome do artista> = <valor> — <razão do programa>` numa fileira de altura FIXA (ela tem um
    // botão `Remove` ao lado, logo não pode quebrar).
    // ⛔ **A saída NÃO é elidir o nome em vez da razão:** quando há vários órfãos a razão repete-se
    //    e o NOME é o único discriminador — cortá-lo tornaria a lista ilegível. A saída honesta é a
    //    razão deixar de ser repetida por linha (a nota da secção já a diz), que é produto.
    (
        "inspector",
        "legacy_speed = 1 \u{2014} not in the script",
        "lista · 189,0 px · frase composta com o nome do artista a' cabeca",
    ),
    // ⭐ A HIERARQUIA, armada em 2026-09-19: **zero** rótulos do programa cortados. Os dois que
    //    saem são NOMES QUE O ARTISTA DEU, numa linha de árvore que ja' desconta o recuo e os
    //    selos — a caixa e' honesta (`110`–`133 px`) e elidir um nome comprido e' o que toda
    //    arvore deste feitio faz. ⚠️ *Um corte nao e' sempre um defeito*, e esta e' a especie que
    //    a lista declara desde 18/09.
    (
        "hierarchy",
        "Enemy Spawner \u{b7} left wing",
        "nome do artista · 132,8 px",
    ),
    (
        "hierarchy",
        "Background Parallax Layer",
        "nome do artista · 110,8 px",
    ),
    // ⭐⭐ **A TRIPLA DA SUBSUPERFÍCIE, na integração de 2026-09-20.** A `W10` da `line/3DModeling`
    //    trouxe `"Subsurface Color R/G/B"` (a luz que atravessa a peça, por canal), e nas TRÊS
    //    larguras largas só o **G** é cortado: `G` pinta mais largo que `R` e `B`, logo dois irmãos
    //    cabem nos `110 px` da fileira e o terceiro perde uma letra.
    // ⚠️⚠️ *Nenhum dos dois lados vê isto sozinho:* o rótulo é da linha, a régua que o mede chegou
    //    do `main` na MESMA rodada, e o painel dela só passou a ser medido quando a fixtura deixou
    //    de ler uma tabela de quatro (ver `o_model3d_armado`).
    // ⛔ **E a cura não é renomear:** a lei de 19/09 manda um nome perder a EXPLICAÇÃO antes das
    //    LETRAS, e aqui não há explicação para tirar — `Subsurface Color G` é o nome. Renomear a
    //    tripla é decisão de VOCABULÁRIO do dono, que esta casa já mediu e reverteu uma vez.
    (
        "model3d",
        "Subsurface Color G",
        "fileira de campo · 110,0 px · a tripla R/G/B, e o `G` pinta mais largo que os irmaos",
    ),
];

/// ⛔⛔⛔ **O QUE FICA CORTADO NUMA SUPERFÍCIE QUE O DONO JÁ MANDOU SAIR — e por isso NÃO é
/// dívida.**
///
/// **Enio, 2026-09-19**, depois do smoke desta varredura: *«as propriedades dos nós não existirão
/// no painel lateral em versões futuras. Apenas nos próprios nós dos grafos. Então não vale a pena
/// investir no painel lateral dos motion nodes. Temos outra linha trabalhando nele.»*
///
/// ⚠️ **A diferença entre esta lista e a de cima é o VEREDITO, não o defeito.** Os cortes são
/// reais e os números estão medidos; o que mudou é que curá-los seria trabalho sobre uma
/// superfície que vai deixar de existir — e a `A_PASSAGEM_ARMADA_AINDA_CORTA` lê-se como *«isto
/// está por fazer»*. *Uma linha de dívida que ninguém deve pegar tem de dizer isso no NOME da
/// lista onde mora, senão a próxima janela gasta uma fatia nela.*
///
/// ⭐ **O painel já nasce DESLIGADO desde 2026-09-07** (`PH2D_MOTION_PANEL=1` traz-no de volta) —
/// o que o dono decidiu foi o passo seguinte, e a autoria mudou-se para o CARTÃO do nó.
///
/// ⛔ **A armação FICA, e não é contradição:** enquanto o painel estiver no registo, esta
/// varredura mede-o — e o censo de obsolescência abaixo continua a exigir que cada linha daqui
/// descreva um corte REAL. *No dia em que o painel sair, as doze ficam obsoletas e o gate manda
/// apagá-las*, que é exactamente como esta lista deve morrer.
///
/// ⚠️ **E o que esta passagem já curou NÃO se desfaz:** a grelha de opções de um selector passou a
/// medir as palavras nas duas dimensões, e a porta que o faz
/// ([`ph2d_editor_core::widget::wrapped_cells_for`]) é da CASA — ela serve toda fileira segmentada
/// que quebre, e o cartão do nó é uma delas.
const FORA_POR_DECISAO_DO_DONO: &[(&str, &str, &str)] = &[
    // ⭐⭐⭐ **`16` cortes na primeira corrida, num painel que nenhuma régua de largura tinha
    //    medido** — a declaração que o deixava de fora (*«precisa de um grafo»*) era uma ausência
    //    afirmada sem olhar a API: ele lê um SNAPSHOT, que são dados. Os rótulos eram DERIVADOS
    //    das `828` entradas do catálogo. ⛔ A fixtura saiu em 2026-09-20 com o próprio painel.
    //
    // ⛔⛔ **Eles são TRÊS famílias e nenhuma se cura uma linha de cada vez:**
    //
    // **(a) a coluna do NOME é um literal** — `DEFAULT_LABEL_W = 70` do
    // `slider_with_chip`, que o painel empresta como coluna de param. Os seis rótulos abaixo são
    // do PROGRAMA e o catálogo tem `444` distintos: *uma coluna escrita como número não pergunta
    // pela lista que vai pintar*, e o `79`/`82` ao lado do `70` diz que ela é respondida em mais
    // do que um sítio.
    //
    // **(b) um CHIP de escolha é dimensionado pelo item em mãos** — o doc da
    // [`ph2d_editor_core::widget::dropdown_label_budget`] já escreve a lei que estes violam:
    // *«quem dimensiona um chip de escolha tem de o fazer pela LISTA, nunca pelo item»*.
    //
    // **(c) a QUEIXA de uma fórmula é uma FRASE cortada a meio** — a mesma família que a linha
    // de órfão do cartão de instância e os avisos das secções do Inspector já pagaram, **e aqui a
    // cura delas está PROIBIDA**: o `the_motion_chrome_never_gives_a_row_label_a_wrap_budget`
    // reprovou o `paint_text_block`, e a lei dele é do report do dono de 30/08 — no chrome do
    // Motion uma linha de `22 px` que quebra derrama a segunda metade **por cima da entrada
    // seguinte**. ⚠️ *A lei é mais velha e está medida; a cura óbvia não a vencia, contornava-a.*
    // ⇒ o que esta linha pede é uma FILEIRA que saiba que segura uma frase (altura própria), que
    // é desenho e não uma troca de pintor.
    //
    // ⇒ a wave seguinte é a das DUAS portas que ficam; cada linha aqui traz o número de hoje.
    //
    // (a) a coluna do NOME
    // ✅ **(b) as QUATRO opções de selector SAÍRAM no mesmo dia** — a grelha delas era o
    //    `block_cells` sobre uma contagem fixa de `4` por fileira, e passou a medir as PALAVRAS
    //    nas duas dimensões (`wrapped_cells_for`). ⚠️⚠️ **A 1.ª tentativa curou só a LARGURA e
    //    TROCOU DE VÍTIMA** — as quatro passaram a caber e `Project` · `Linear` · `Linear Mip`
    //    passaram a ser cortadas: *repartir bem uma fileira MAL FORMADA não cura nada*, porque
    //    uma fileira que não cabe encolhe tudo na mesma proporção (que é a lei certa: ali falta
    //    coluna, não disposição). ⇒ a quebra também tem de sair das palavras.
    // ⛔ **O pior da lista, e e' do PRODUTO:** o chip do espaco de cor do editor de gradiente tem
    //    `24 px` para TRES letras.
    // ⚠️ Os tres nomes de canal sao PLAUSIVEIS e nao derivados (um canal e' uma coluna do stream,
    //    que o artista cria) — mas a CAIXA de `44 px` e' a do produto, e nenhuma palavra de oito
    //    letras cabe nela.
    // (c) a QUEIXA de uma fórmula — uma FRASE cortada a meio, e a cura óbvia está PROIBIDA
    // Nome de uma forma que o artista desenhou, num chip de fonte.
];

/// ⭐⭐⭐ **O NOME DO DEGRAU ESTREITO** — o unico que nao esta na largura de fabrica.
pub(crate) const DEGRAU_ESTREITO: &str = "colunas no minimo";

/// ⛔⛔⛔ **A DIVIDA DO DEGRAU ESTREITO, POR PAINEL E POR CONTAGEM.**
///
/// ⛔⛔ **`129` cortes que esta varredura nunca tinha visto, na largura em que o dono de facto
/// trabalha.** Ate 2026-09-19 ela media UMA largura de coluna — a de fabrica (`308` a
/// esquerda, `304` a direita) — e ficava verde sobre todos eles. Lido o
/// `~/.ph2d/layout.txt` desse dia, a coluna da ESQUERDA do dono estava no **minimo (`220`)** em
/// cinco dos seis espacos de trabalho.
///
/// # ⭐⭐⭐ O MECANISMO, e ele e UM SO para `80` dos `129`
///
/// A linha de propriedade tem duas ordens do dono que se cruzam:
/// 1. *"Label acima do campo numerico! Muito ruim!"* (2026-09-14) — o nome fica ao LADO;
/// 2. *"nao permita que a caixa seja redimencionada para menor que isso"* (2026-05-24) — a
///    caixa tem piso de [`NUMBER_INPUT_MIN_W_PX`] (`72`).
///
/// A porta ([`property_label_col_w_for`]) ja faz tudo o que pode: a coluna MEDE os nomes da seccao
/// (`Seccao::medida`, e 32 sitios ja a usam) e EMPRESTA para alem da metade. O que a prende e o
/// **tecto**, que e a ordem 2: `coluna <= util - vao - 72`. A `220` de coluna isso da
/// **`90 px` para o nome**, e o Inspector tem dezenas de rotulos entre `100` e `190`.
///
/// ⛔⛔⛔ **A frase que estava aqui — *«nao ha cura de codigo dentro das duas ordens»* — MORREU em
/// 2026-09-19, e as duas curas cabem dentro delas.** O dono escolheu, das tres saidas que esta
/// nota lhe devolveu, **duas**: *«encurtar · balao ao passar o rato»*.
///
/// 1. **O BALAO** ([`ph2d_editor_core::text_elide::balao`]): toda palavra cortada por este app
///    e legivel ao passar o rato — `128` de `128`, com o gate [`toda_palavra_cortada_tem_balao`]
///    a prova-lo pela MESMA varredura que as achou. ⭐ E ele e a unica resposta possivel para o
///    texto que o ARTISTA escreve (o nome de um objecto, de uma ancora, de uma propriedade de
///    script), que encurtar nunca pode alcançar.
/// 2. **ENCURTAR**, e a parte mecanica dela nao custou um unico nome novo: *um nome perde a
///    EXPLICACAO antes de perder LETRAS* ([`ph2d_editor_core::text_elide::fit_do_nome`]) —
///    `"Acceleration (0 = instant)"` deixa de sair `"Acceleration…"` e sai **`"Acceleration"`**.
///    `18` rotulos, zero chaves novas e zero prosa a envelhecer.
///
/// ⛔⛔ **E QUATRO renomes a mao foram construidos e REVERTIDOS, com o preco medido:** encurtar
/// `"Always show anchors"` para `"Always show"` (e outros tres, todos onde a SECCAO ja diz a
/// palavra que sai) fechou **`2`** rotulos de `80` e deixou **`54`** citacoes do nome antigo na
/// prosa deste repo — tres delas no roteador (`CLAUDE.md` §5), que uma linha nao pode editar.
/// ⇒ *renomear um rotulo custa as citacoes dele*, e e por isso que a lei da casa para encurtar um
/// nome (2026-09-19, tres caixas) passa pelo DONO a escolher o nome e pela explicacao a mudar-se
/// para o balao do widget — nunca por um palpite a mais.
///
/// ⚠️ **O que sobra e uma decisao de VOCABULARIO, e e dele:** `~36` nomes compostos
/// (`Air Acceleration`, `Non-Spatialized Radius`, `Crouch Height`) precisam de **`13`–`15`
/// caracteres** para caber em `78`–`90 px`, e medem `16`–`22`. Encurta-los nao e tirar gordura, e
/// **trocar o nome** — e ate la o balao le-os.
///
/// ⛔ **A terceira saida continua fora:** o nome subir para cima do controlo, que a ordem 1
/// proibe. E ⛔ **alargar a coluna tambem**: ela acaba na METADE da linha por ordem do dono
/// (2026-09-14, *«as labels alinhadas todas a direita, no centro do painel»*).
///
/// # ⚠️ Porque uma CONTAGEM por painel, e nao uma linha por corte
///
/// ⛔ Uma lista de `129` linhas, cada uma com o numero e o mecanismo, seria prosa que
/// ninguem le — e o mecanismo e **partilhado**: a lei desta casa e curar por MECANISMO e
/// nunca por `N` remendos. A catraca tem as duas metades:
/// - nenhum painel corta MAIS do que o numero dele;
/// - nenhum painel corta MENOS (senao o numero ja nao descreve nada e tem de DESCER).
///
/// ⚠️ **O que ela NAO apanha, declarado:** um painel que troque um corte por outro fica com
/// a mesma contagem. A metade que o apanharia e uma linha por texto, que e o que esta nota acabou
/// de recusar — *a catraca conta a POPULACAO, e quem julga um corte novo e a mensagem de
/// falha, que imprime a lista inteira.*
///
/// ⚠️ `motion_params` esta aqui com `5` e **fora de escopo** por decisao do dono (ver
/// [`FORA_POR_DECISAO_DO_DONO`]): o numero fica para a catraca nao mentir sobre a populacao.
const CORTES_NO_DEGRAU_ESTREITO: &[(&str, usize)] = &[
    // ⚠️⚠️ **Este numero NAO desce com a lei de encurtar de 2026-09-19, e isso e a lei.** Um
    //    rotulo que sai `"Acceleration"` de `"Acceleration (0 = instant)"` continua a esconder a
    //    explicacao do artista — o que muda e ele deixar de comer o NOME. Quem conta essa
    //    diferenca e a [`LETRAS_PERDIDAS_NO_DEGRAU_ESTREITO`], e e por isso que sao duas.
    // ⭐ `103 → 102` em 2026-09-21: o `Per-Corner Tint (vertex gradient)` virou
    //    `Per-corner Tint` por ordem do dono, e a explicação foi para o BALÃO. *Um nome
    //    que encolhe tira um corte* — a catraca a DESCER é a lei a funcionar.
    // ⭐⭐ `102 → 90` em 2026-09-21: **treze** rótulos do Inspector perderam a REGRA que
    //    carregavam entre parêntesis (`Acceleration (0 = instant)` → `Acceleration`), e ela foi
    //    para o balão do controlo. *Um nome que encolhe tira um corte* — e aqui tirou doze.
    // ⭐ `90 → 89` em 2026-09-21: as SETE linhas de marcar do Inspector passaram pela porta
    //    `paint_check_row` (report do dono sobre a ALTURA), e com isso entraram na coluna da
    //    SECÇÃO em vez da de omissão. ⚠️ **Ela desceu porque a coluna ficou mais CERTA**, não
    //    porque alguém encurtou um nome — e o censo de obsolescência foi quem o exigiu.
    // ⬇️ `89 → 86` em 2026-09-22: as três frases que os avisos deixaram de CORTAR (elas quebram).
    // ⬇️ `86 → 85` em 2026-09-22: o `"Repeat (0 = forever)"` da §11 Animation virou **`"Repeat"`**
    //    com a explicação a descer para o balão — a lei *«um nome perde a EXPLICAÇÃO antes de
    //    perder LETRAS»* aplicada ao último rótulo que ainda carregava uma regra dentro do nome.
    //    ⚠️⚠️ **O número foi ATRIBUÍDO por A/B e não inferido:** a minha 1.ª redacção desta linha
    //    dizia que o corte tinha saído por a ORDEM das secções ter mudado de família — *um
    //    palpite com cara de medição*, e falso (reordenar não muda a largura de coluna de
    //    ninguém). Repondo o texto longo a catraca lê `86` e nomeia-o na lista; com o curto, `85`.
    // ⬇️ `85 → 79` em 2026-09-23: as escolhas do Inspector passaram pela porta da ESCOLHA
    //    (`property_row::paint_choice_row`), e as que não cabem ao lado do nome viram PALETA com
    //    larguras naturais — os seis segmentados que repartiam a fileira em partes deixaram de
    //    cortar. Medido no âmbito do app (`--workspace`), a catraca a pedir o número.
    ("inspector", 79),
    // ⭐ Era `6`: o `Mute` do Master deixou de ler `…` quando a coluna aperta (report do dono,
    //    19/09). *Uma catraca que desce é a metade justa dela a funcionar.*
    ("audio_mixer", 5),
    ("sculpt3d", 6),
    ("hierarchy", 5),
    // ⬇️ `5 → 2` em 2026-09-23: o *Reset* de cada linha virou ÍCONE (era um botão de texto de
    //    `48 px` que saía do nome de toda linha autorada) e a amostra passou ao quadrado da altura
    //    de uma linha (`22`, era o `SwatchSize::Md` de `32`) — o nome ganhou `10 px` em toda linha.
    ("tokens", 2),
    ("vector", 3),
    ("color_equalization", 2),
    ("tags", 2),
    ("audio_editor", 1),
    ("authored", 1),
    // ⭐⭐ Era `1` (a `"Subsurface Anisotropy"`): a wave da SUBSUPERFÍCIE desta linha (`W10`, 17/09)
    //    trouxe a tripla `"Subsurface Color R/G/B"`, e no degrau estreito **só o G** é cortado —
    //    `G` pinta mais largo que `R` e `B`, logo dois irmãos cabem em `110 px` e o terceiro não.
    //    ⚠️ *Nenhum dos dois lados vê isto sozinho:* o rótulo é da linha e a régua que o mede
    //    chegou do `main` na mesma rodada. ⛔ E a cura NÃO é renomear — a lei de 19/09 manda o nome
    //    perder a EXPLICAÇÃO antes das LETRAS, e aqui não há explicação para tirar: os `~63` nomes
    //    compostos que ficam são decisão de VOCABULÁRIO do dono, medida e revertida uma vez.
    ("model3d", 2),
    ("widget_lab", 1),
    // ⛔⛔⛔ **OS TRÊS QUE SÓ A ÁRVORE INTEIRA VÊ** (integração de 2026-09-20). Eles estão atrás de
    //    features que o `default` desta crate NÃO liga (`panel-flip`, `panel-painter-layers`,
    //    `panel-wet-tuning`) e chegam pelo `shells/desktop` ⇒ numa corrida `-p` não são sequer
    //    REGISTADOS, e a catraca lia `0` como *«este painel não corta»* em vez de *«este painel
    //    não existe aqui»*. *Os dois lêem-se igual num número.* O piso de população passou a
    //    recusar aquele âmbito (ver [`PISO_DE_PAINEIS`]), e estes números são a medição do âmbito
    //    em que o app corre.
    // ⚠️ São **dívida da linha**, não decisão de produto: o §7 do handoff de 20/09 já os nomeia
    //    (*«os ~46 cortes do degrau estreito FORA do Inspector, em sete painéis»*) — o que faltava
    //    era a régua conseguir vê-los.
    // `["Delete", "Duplicate"]`
    ("flip", 2),
    // `["Composite Brush", "Digital Basic", "Sync with other tools", "View Plane"]`
    // ⚠️⚠️ **`3 → 4` em 2026-09-21, e NÃO é regressão — é a POPULAÇÃO a crescer.** Este painel
    //    passou a ser ARMADO (`super::o_painter_armado`), e armado ele pinta as fileiras que só
    //    existem depois de o artista escolher um padrão. O `View Plane` é uma delas, e nunca
    //    tinha sido medido: a varredura via o Painter no estado de FÁBRICA, onde a textura é
    //    `None`. *Um número que sobe porque a régua passou a ver mais não é o mesmo que um número
    //    que sobe porque alguém partiu algo* — foi o que o `o_inspector_armado` fez ao painel ao
    //    lado (`129` cortes invisíveis de uma vez).
    // ⛔ Continua a ser DÍVIDA, e a cura é a do dono (2026-09-20): encurtar o nome, com o balão a
    //    guardar a explicação.
    // ⚠️ `4 → 5` em 2026-09-21: o **`Use Color Ramp`** vive numa secção que este painel semeia
    //    DOBRADA, e esta varredura passou a abrir toda gaveta antes de medir (ver
    //    [`abre_as_gavetas`]). *População nova, não regressão* — o rótulo sempre foi cortado; o
    //    que mudou é que agora há quem o veja.
    ("painter_layers", 5),
    // `["Glaze layering (K-M)", "Pigment mixing (K-M)"]`
    ("wet_tuning", 2),
];

/// ⭐⭐⭐ **QUANTOS ROTULOS PERDEM LETRAS** — a catraca que o DONO ve, e a irmã da de cima.
///
/// ⛔⛔ **Ela existe porque a [`CORTES_NO_DEGRAU_ESTREITO`] deixou de separar duas coisas muito
/// diferentes** no dia em que o `fit_do_nome` chegou: um rotulo que sai `"Acceleration"` e um que
/// sai `"Acceleration…"` contam os DOIS como corte — e contam bem, porque nos dois o artista fica
/// sem a explicacao. Mas so o segundo lhe come o NOME, e e o segundo que ele fotografou.
///
/// ⇒ *duas grandezas estavam a ser lidas como uma*, que e a forma exacta que o `shift_frac_max`
/// do gridmap ja custou a esta casa. Esta catraca conta **so quem acaba em reticencias** (ou em
/// nada), e e sobre ela que «encurtar» se mede.
///
/// ⚠️ **As duas sao precisas.** Sem a de cima, encurtar um nome ate ele caber e esconder um
/// corte tem a mesma leitura; sem esta, o `fit_do_nome` nao teria numero nenhum a mostrar.
const LETRAS_PERDIDAS_NO_DEGRAU_ESTREITO: &[(&str, usize)] = &[
    // ⭐ `63` de `80`: a lei de encurtar tirou as reticencias a **17** rotulos deste painel, sem
    //    um unico nome novo. Os `63` que ficam sao nomes compostos, e encurtar um deles e trocar
    //    o nome — decisao de vocabulario, que e do dono.
    // ⭐ `86 → 85` em 2026-09-21: o `Per-Corner Tint (vertex gradient)` virou
    //    `Per-corner Tint` por ordem do dono, e a explicação foi para o BALÃO. *Um nome
    //    que encolhe tira um corte* — a catraca a DESCER é a lei a funcionar.
    // ⭐ `85 → 84` em 2026-09-21, pela mesma passagem pela porta — ver a irmã acima.
    // ⬇️ `84 → 81` em 2026-09-22, pela mesma cura — as letras deixaram de se perder.
    // ⬇️ `81 → 75` em 2026-09-23, pela mesma passagem pela porta da escolha — ver a irmã acima.
    ("inspector", 75),
    ("audio_mixer", 5),
    ("sculpt3d", 6),
    ("hierarchy", 5),
    // ⬇️ `5 → 2` em 2026-09-23: o *Reset* de cada linha virou ÍCONE (era um botão de texto de
    //    `48 px` que saía do nome de toda linha autorada) e a amostra passou ao quadrado da altura
    //    de uma linha (`22`, era o `SwatchSize::Md` de `32`) — o nome ganhou `10 px` em toda linha.
    ("tokens", 2),
    ("vector", 3),
    ("color_equalization", 2),
    ("tags", 2),
    ("audio_editor", 1),
    ("authored", 1),
    // ⭐ Era `1`, pela MESMA tripla da irmã [`CORTES_NO_DEGRAU_ESTREITO`] — o `"Subsurface Color G"`
    //    acaba em reticência, logo conta nas duas.
    ("model3d", 2),
    ("widget_lab", 1),
    // ⛔ Os dois que só a árvore inteira vê — o mecanismo está na irmã
    //    [`CORTES_NO_DEGRAU_ESTREITO`]. ⚠️ O `wet_tuning` NÃO entra aqui: os dois rótulos dele
    //    são cortes que **não acabam em reticência**, e é isso que separa esta catraca da irmã.
    ("flip", 2),
    // ⚠️ `3 → 4` em 2026-09-21 pelo MESMO motivo da irmã: o Painter passou a ser ARMADO e o
    //    `View Plane` só existe com um padrão escolhido. Ver a nota lá.
    // ⚠️ `4 → 5` no mesmo dia: a varredura passou a ABRIR as gavetas, e o `Use Color Ramp` vive
    //    numa secção que este painel semeia dobrada. *População nova, não regressão.*
    ("painter_layers", 5),
];

/// ⭐⭐⭐ **E NENHUM PAINEL PASSA A COMER MAIS LETRAS** — as duas metades, como a irmã.
/// ⭐⭐⭐ **ABRE TODA GAVETA antes de medir** — esta varredura mede o ECRÃ, e desde 2026-09-21 o
/// ecrã **DOBRA** (o Inspector abre com toda secção com chevron recolhida menos a Transform).
///
/// ⛔⛔ *Sem isto a varredura passa a medir os rótulos VISÍVEIS ao abrir, e um rótulo cortado dentro
/// de uma gaveta fechada lê-se como inexistente* — que é a cegueira que a fixtura
/// [`super::o_inspector_armado`] existe para curar, agora um nível acima.
///
/// ⚠️ Gémea da `abre_tudo` do censo de entradas: as duas passam pela MESMA porta do produto
/// ([`ph2d_editor_core::screens::hero::pre_populate::marca_as_gavetas`]), porque o conjunto das
/// gavetas é semeado por ela e não pelo `Panel::populate`.
fn abre_as_gavetas(store: &mut ph2d_editor_core::interaction::WidgetStore) {
    ph2d_editor_core::screens::hero::pre_populate::marca_as_gavetas(store);
    let gavetas = store.collapsible_ids();
    assert!(
        gavetas.len() >= 38,
        "o arnês vê {} gavetas — sem elas esta varredura mede um painel dobrado",
        gavetas.len()
    );
    for id in gavetas {
        store.set_collapsed(id, false);
    }
}

#[test]
fn as_letras_perdidas_no_degrau_estreito_so_encolhem() {
    let mut por_painel: std::collections::BTreeMap<&str, std::collections::BTreeSet<String>> =
        std::collections::BTreeMap::new();
    for a in varre()
        .iter()
        .filter(|a| a.degrau == DEGRAU_ESTREITO && !a.m.coube())
        // ⭐ **A reticencia e o discriminador** — ela e o que o olho do dono le como «cortado».
        //   Um `pintado` VAZIO e o degrau pior da mesma escada e conta aqui tambem.
        .filter(|a| a.m.pintado.ends_with('\u{2026}') || a.m.pintado.is_empty())
    {
        por_painel
            .entry(a.painel)
            .or_default()
            .insert(a.m.texto.clone());
    }
    let declarado: std::collections::BTreeMap<&str, usize> =
        LETRAS_PERDIDAS_NO_DEGRAU_ESTREITO.iter().copied().collect();
    let piorou: Vec<String> = por_painel
        .iter()
        .filter(|(id, v)| v.len() > declarado.get(*id).copied().unwrap_or(0))
        .map(|(id, v)| {
            let mut nomes: Vec<&str> = v.iter().map(String::as_str).collect();
            nomes.sort_unstable();
            format!(
                "{id}: {} rotulos comidos (declarado {}) — {:?}",
                v.len(),
                declarado.get(id).copied().unwrap_or(0),
                nomes
            )
        })
        .collect();
    assert!(
        piorou.is_empty(),
        "na largura em que o dono trabalha estes paineis passaram a COMER LETRAS de mais \
         rotulos:\n  {}",
        piorou.join("\n  ")
    );
    let obsoletos: Vec<String> = declarado
        .iter()
        .filter(|(id, n)| {
            por_painel
                .get(*id)
                .map_or(0, std::collections::BTreeSet::len)
                < **n
        })
        .map(|(id, n)| {
            format!(
                "{id}: declarado {n}, mede {}",
                por_painel
                    .get(id)
                    .map_or(0, std::collections::BTreeSet::len)
            )
        })
        .collect();
    assert!(
        obsoletos.is_empty(),
        "estes numeros ja nao descrevem quantas letras o painel come — BAIXE-OS:\n  {}",
        obsoletos.join("\n  ")
    );
}

/// ⭐⭐⭐ **A DIVIDA DO DEGRAU ESTREITO SO ENCOLHE** — as duas metades.
#[test]
fn a_divida_do_degrau_estreito_so_encolhe() {
    let mut por_painel: std::collections::BTreeMap<&str, std::collections::BTreeSet<String>> =
        std::collections::BTreeMap::new();
    for a in varre()
        .iter()
        .filter(|a| a.degrau == DEGRAU_ESTREITO)
        .filter(|a| !a.m.coube() && !a.m.nada())
    {
        por_painel
            .entry(a.painel)
            .or_default()
            .insert(a.m.texto.clone());
    }
    let declarado: std::collections::BTreeMap<&str, usize> =
        CORTES_NO_DEGRAU_ESTREITO.iter().copied().collect();
    // ⭐ A metade que sobe: um painel que passe a cortar mais.
    let piorou: Vec<String> = por_painel
        .iter()
        .filter(|(id, cortes)| cortes.len() > declarado.get(*id).copied().unwrap_or(0))
        .map(|(id, cortes)| {
            let mut nomes: Vec<&str> = cortes.iter().map(String::as_str).collect();
            nomes.sort_unstable();
            format!(
                "{id}: {} cortes (declarado {}) — {:?}",
                cortes.len(),
                declarado.get(id).copied().unwrap_or(0),
                nomes
            )
        })
        .collect();
    assert!(
        piorou.is_empty(),
        "na largura em que o dono trabalha estes paineis passaram a cortar MAIS:\n  {}",
        piorou.join("\n  ")
    );
    // ⭐ A metade justa: um numero que ja nao descreve nada DESCE.
    let obsoletos: Vec<String> = declarado
        .iter()
        .filter(|(id, n)| {
            por_painel
                .get(*id)
                .map_or(0, std::collections::BTreeSet::len)
                < **n
        })
        .map(|(id, n)| {
            format!(
                "{id}: declarado {n}, mede {}",
                por_painel
                    .get(id)
                    .map_or(0, std::collections::BTreeSet::len)
            )
        })
        .collect();
    assert!(
        obsoletos.is_empty(),
        "estes numeros ja nao descrevem o que o painel corta — BAIXE-OS:\n  {}",
        obsoletos.join("\n  ")
    );
}

/// ⭐ **Os painéis que ESTA build liga** — lidos do registo, e não do que a pintura produziu.
fn paineis_do_registo() -> std::collections::BTreeSet<&'static str> {
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    })
}

/// Uma medição, com o painel que a fez e o viewport em que ela aconteceu.
struct Achado {
    painel: &'static str,
    /// O degrau da [`ESCADA`] em que esta medição aconteceu.
    degrau: &'static str,
    /// ⭐ **A passagem com o documento na mão** ([`super::o_inspector_armado`]) — hoje só o
    /// Inspector a tem. ⚠️ Ela **não** muda o `painel`, de propósito: o censo de obsolescência
    /// filtra a dívida pelos painéis do REGISTO, e um nome inventado (`"inspector (armado)"`)
    /// nunca lá estaria ⇒ a linha dele passaria a ser saltada **para sempre**, em silêncio. É a
    /// mesma armadilha que a 1.ª redacção daquele censo já pagou.
    armado: bool,
    m: Medido,
}

impl Achado {
    /// O sítio, como uma mensagem de falha o nomeia.
    fn onde(&self) -> String {
        let estado = if self.armado { " (armado)" } else { " (vazio)" };
        format!("{}{estado} @ {}", self.painel, self.degrau)
    }
}

/// ⭐ Pinta **todo** painel do registo nos três viewports e devolve tudo o que o censo viu.
///
/// ⚠️ **Cada painel leva um host NOVO**, com o `populate` dele corrido — é isso que o app faz no
/// arranque, e um store vazio faria metade dos painéis desenhar o estado de um widget que ainda
/// não existe.
fn varre() -> Vec<Achado> {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut tudo = Vec::new();
    // ⚠️ **Contados no REGISTO e não nos achados:** um painel que não pinte um único rótulo
    //    elidível não aparece na lista, e um piso sobre os achados leria menos do que a
    //    população real — acusando uma build correcta.
    let mut visitados = 0usize;
    for &(degrau, w, h, col_w) in ESCADA {
        let viewport = Rect {
            x: 0.0,
            y: 0.0,
            w,
            h,
        };
        let bandas = bandas(col_w);
        ph2d_editor_core::panel::with_registry(|reg| {
            visitados = reg.panels().len();
            for painel in reg.panels_mut() {
                let id = painel.manifest.id;
                let mut host = MockPanelHost::new();
                painel.populate(host.store_mut());
                abre_as_gavetas(host.store_mut());
                for m in host.medindo_a_pintura_do_registo_com_bandas(painel, viewport, bandas) {
                    tudo.push(Achado {
                        painel: id,
                        degrau,
                        armado: false,
                        m,
                    });
                }
                // ⭐⭐⭐ **A SEGUNDA PASSAGEM: o painel com um DOCUMENTO na mão.**
                //
                // ⚠️ **Armar vem ANTES do `populate`**, e não é ordem de conveniência: as
                // `populate_*` das secções condicionais semeiam os widgets a partir da informação
                // publicada, logo um `populate` corrido antes veria o painel vazio e a passagem
                // mediria as mesmas fileiras da primeira.
                //
                // ⭐ A população sai da [`super::paineis_armados::TABELA`], nunca de um `if` por
                //    nome de painel: um painel novo armado entra num sítio só.
                if let Some(arm) = super::paineis_armados::TABELA
                    .iter()
                    .find(|a| a.painel == id)
                {
                    let mut host = MockPanelHost::new();
                    (arm.arma)(host.store_mut());
                    painel.populate(host.store_mut());
                    abre_as_gavetas(host.store_mut());
                    for m in host.medindo_a_pintura_do_registo_com_bandas(painel, viewport, bandas)
                    {
                        tudo.push(Achado {
                            painel: id,
                            degrau,
                            armado: true,
                            m,
                        });
                    }
                    // ⛔ O estado que uma fixtura deixa para trás é o estado que a régua seguinte
                    //    mede — e estas portas são `thread_local`, partilhadas pelo binário todo.
                    (arm.desarma)();
                }
            }
        });
    }
    assert!(
        tudo.len() >= PISO_DE_MEDICOES && visitados >= PISO_DE_PAINEIS,
        "a varredura mediu {} rótulos em {visitados} painéis (piso {PISO_DE_MEDICOES} / \
         {PISO_DE_PAINEIS}) — ou um painel deixou de pintar, ou o censo deixou de ouvir a lei da \
         reticência. Uma varredura que lê pouco devolve ZERO cortes e lê-se como aprovação.\n\
         ⚠️ SE VOCÊ CORREU ISTO COM `-p ph2d-panel-registry-init`, a causa é essa e não o código: \
         `flip`, `flip_frames`, `painter_layers` e `wet_tuning` NÃO estão no `default` desta crate \
         — eles chegam pelo `shells/desktop`, e só a unificação de features de um build de \
         WORKSPACE os acende (medido 2026-09-20: 24 painéis/11 375 rótulos contra 28/12 545). \
         ⇒ corra `cargo nextest run --workspace -E 'test(nenhum_rotulo_do_app_pinta_nada)'`.",
        tudo.len()
    );
    tudo
}

/// ⭐⭐⭐ **NENHUM RÓTULO DESTE APP PINTA NADA.**
///
/// ⛔⛔ É a lei dura, e ela nasce de um defeito medido: o botão de silenciar do mixer tem `25,0 px`,
/// o respiro levava `16,0` e sobravam `9,0` para uma letra que precisa de `10,1` ⇒ **nem a
/// reticência cabia, e o botão saía VAZIO**. *Um controlo sem legenda e um controlo morto dão o
/// mesmo report.*
///
/// ⚠️ A catraca está a **ZERO**, que é a mais apertada que existe: já não há linha onde escrever um
/// rótulo mudo.
#[test]
fn nenhum_rotulo_do_app_pinta_nada() {
    let mudos: Vec<String> = varre()
        .iter()
        .filter(|a| a.m.nada())
        .map(|a| {
            format!(
                "{}: {:?} não cabe em {:.1} px — nem a reticência em {}",
                a.onde(),
                a.m.texto,
                a.m.largura,
                a.m.onde
            )
        })
        .collect();
    assert!(
        mudos.is_empty(),
        "estes controlos pintam NADA:\n  {}",
        mudos.join("\n  ")
    );
}

/// ⭐⭐⭐ **E NENHUM PINTARIA NADA NO DIA EM QUE ALGUÉM TRADUZIR.**
///
/// ⚠️ **A reticência não depende da língua** — é isso que torna esta a régua do defeito duro: uma
/// caixa que hoje mostra `M` mostra-o porque a palavra é curta, não porque a caixa chegue, e no dia
/// em que a palavra crescer ela cai **directamente no vazio**, sem passar pelo `prefixo…`.
///
/// ⛔⛔ **O que NÃO se traduz também não se deforma, e a 1.ª redacção disto acusou um NÚMERO:**
/// `"3"` numa caixa de `9,0 px` da timeline, «deformado» para `"[3]"`. ⚠️ O censo regista o que foi
/// **PINTADO** e não sabe de onde a string veio — da tabela, do documento, de um `format!` —, logo
/// a previsão precisa da lei que separa: **uma palavra tem LETRAS**. É a mesma cerca que o corpus
/// do próprio idioma de teste declara (as `89 832` traduções medidas excluem o que não tem letra),
/// e ela erra para o lado seguro: um rótulo com letras continua a ser medido.
#[test]
fn nenhum_rotulo_pintaria_nada_na_proxima_lingua() {
    let mut ts = TextSystem::without_system_fonts();
    // ⛔⛔⛔ **UM SIMBOLO DE UNIDADE NAO SE TRADUZ, logo tambem nao se deforma.**
    //
    // Medido em 2026-09-19 pela escada: o sufixo `s` (segundos) dentro de uma caixa de numero de
    // `9,5 px` era acusado porque `[s]` nao cabe. ⚠️ Mas `s`, `px`, `deg` e `rad` sao
    // simbolos **SI/tipograficos** escritos a mao em [`Unit::suffix`] — eles nunca passam
    // pela tabela de idiomas, logo nao existe lingua em que cresçam. *Deformar um texto que a
    // tabela nao produz e medir uma lingua que nao existe* — a mesma familia do `"3"` que a
    // 1.ª redaccao deste gate acusou, um nivel acima (aquela cerca era «tem LETRAS» e esta e «e uma
    // UNIDADE»).
    //
    // ⭐ **A lista e DERIVADA do enum** (`Unit::ALL`), nunca escrita aqui: uma unidade nova
    // entra sozinha, e o piso de populacao apanha o dia em que o `ALL` deixar de casar.
    let unidades: std::collections::BTreeSet<&'static str> = ph2d_editor_core::widget::Unit::ALL
        .iter()
        .map(|u| u.suffix().trim())
        .filter(|s| !s.is_empty())
        .collect();
    assert!(
        unidades.len() >= 8,
        "o extractor leu {} simbolos de unidade (piso 8) — o `Unit::ALL` mudou de forma, e uma \
         lista vazia poe este gate a acusar toda unidade do app",
        unidades.len()
    );
    let mut mudos = Vec::new();
    for a in varre() {
        if !a.m.texto.chars().any(char::is_alphabetic) {
            continue;
        }
        if unidades.contains(a.m.texto.trim()) {
            continue;
        }
        let deformado = pseudo::deforma(&a.m.texto);
        let cabe = ts.prefix_width_weighted(&deformado, a.m.fonte, a.m.peso) <= a.m.largura;
        if !cabe && largura_da_reticencia(&mut ts, a.m.fonte, a.m.peso) > a.m.largura {
            mudos.push(format!(
                "{}: {:?} vira {deformado:?} e some — a caixa tem {:.1} px em {}",
                a.onde(),
                a.m.texto,
                a.m.largura,
                a.m.onde
            ));
        }
    }
    assert!(
        mudos.is_empty(),
        "estes controlos ficam MUDOS na primeira tradução:\n  {}",
        mudos.join("\n  ")
    );
}

/// ⭐⭐ **A DÍVIDA DOS CORTES SÓ ENCOLHE.**
#[test]
fn nenhum_corte_novo_entra_sem_ser_nomeado() {
    let novos: Vec<String> = varre()
        .iter()
        .filter(|a| !a.m.coube() && !a.m.nada())
        // ⛔ **O degrau ESTREITO tem catraca propria, por CONTAGEM**
        // ([`a_divida_do_degrau_estreito_so_encolhe`]) — os `111` cortes dele partilham
        // **um** mecanismo, e uma linha de prosa por cada um seria a lista que ninguem le. Aqui
        // ficam os cortes na largura de FABRICA, que sao poucos e cada um com a causa dele.
        .filter(|a| a.degrau != DEGRAU_ESTREITO)
        .filter(|a| !CORTADOS_HOJE.contains(&(a.painel, a.m.texto.as_str())))
        // ⭐ E a dívida que o PONTO CEGO escondia — por PAINEL e por TEXTO, como a irmã de cima:
        //    o mesmo rótulo pode caber num painel e não caber noutro.
        .filter(|a| {
            !a.armado
                || !A_PASSAGEM_ARMADA_AINDA_CORTA
                    .iter()
                    .chain(FORA_POR_DECISAO_DO_DONO)
                    .any(|(p, t, _)| *p == a.painel && *t == a.m.texto)
        })
        .map(|a| {
            format!(
                "{}: {:?} -> {:?} em {:.1} px",
                a.onde(),
                a.m.texto,
                a.m.pintado,
                a.m.largura
            )
        })
        .collect();
    assert!(
        novos.is_empty(),
        "cortes NOVOS — ou a caixa passa a descrever o que pinta, ou a linha entra na dívida com \
         o mecanismo escrito ao lado:\n  {}",
        novos.join("\n  ")
    );
}

/// ⛔⛔⛔ **OS PAINÉIS QUE ESTA VARREDURA NÃO CONSEGUE PINTAR CHEIOS.**
///
/// Medido 2026-09-19, contando os rótulos que cada painel do registo mede de fábrica: **cinco**
/// mediam `0` e três mediam menos de `8`. O piso desta varredura é GLOBAL (`2 800` rótulos), logo
/// ela ficava **verde com oito painéis invisíveis** — *um piso sobre a soma não pergunta por
/// ninguém*.
///
/// ⚠️ Três deles foram ARMADOS ([`super::paineis_armados::TABELA`]). Os que ficam aqui pedem um
/// mundo que o arnês não constrói, e cada linha diz **qual**.
const PAINEIS_MEDIDOS_VAZIOS: &[(&str, &str)] = &[(
    "motion_graph",
    "pinta um GRAFO de nos vivo (`ph2d-nodegraph`), com o cartao e os pinos derivados do \
         manifesto de cada no; o arnes nao monta um grafo.",
)];

/// ⛔ O piso POR PAINEL. ⚠️ **Ele sai da medição, não do gosto:** o painel mais magro que a
/// varredura de facto enche é o do esqueleto, com `7` rótulos; os que ela não enche medem `0`,
/// `2` ou `3`. *O `5` é o vale entre as duas populações* — e um número acima de `7` acusaria um
/// painel honesto no dia em que ele perdesse uma linha.
const PISO_POR_PAINEL: usize = 5;

/// ⭐⭐⭐ **NENHUM PAINEL DO REGISTO É MEDIDO VAZIO SEM O DECLARAR.**
///
/// ⛔⛔ Esta é a régua da própria régua. A varredura afirma coisas fortes — *«nenhum rótulo deste
/// app pinta nada»* — e elas só valem sobre o que ela pintou. Um painel que ela pinta vazio não é
/// aprovado: é **não medido**, e as duas coisas leem-se igual num relatório verde.
///
/// ⚠️ A conta é sobre o MÁXIMO entre as passagens: um painel armado enche na segunda, e é isso que
/// o tira desta lista.
#[test]
fn nenhum_painel_e_medido_vazio_sem_o_declarar() {
    let tudo = varre();
    // ⚠️⚠️ **A conta é POR VIEWPORT, e a 1.ª redacção somava os três.** Uma mutação sobrevivente
    //    disse-o: com a soma, um painel que mede `3` rótulos passa a ler `9` e salta um piso de
    //    `5` sem ter enchido nada. *Um piso aplicado a uma soma de corridas é um piso dividido
    //    pelo número de corridas.* ⇒ contamos `(painel, viewport, armado)` e ficamos com o MELHOR
    //    quadro que aquele painel consegue mostrar.
    let mut por_quadro: std::collections::BTreeMap<(&str, &str, bool), usize> =
        std::collections::BTreeMap::new();
    for a in &tudo {
        *por_quadro
            .entry((a.painel, a.degrau, a.armado))
            .or_insert(0) += 1;
    }
    let mut por_painel: std::collections::BTreeMap<&str, usize> = paineis_do_registo()
        .into_iter()
        .map(|id| (id, 0usize))
        .collect();
    for ((id, _, _), n) in por_quadro {
        let e = por_painel.entry(id).or_insert(0);
        *e = (*e).max(n);
    }
    let declarados: std::collections::BTreeSet<&str> =
        PAINEIS_MEDIDOS_VAZIOS.iter().map(|(id, _)| *id).collect();
    for (id, porque) in PAINEIS_MEDIDOS_VAZIOS {
        assert!(
            porque.len() > 60,
            "a declaração de `{id}` não diz que MUNDO falta ao arnês"
        );
    }
    let mudos: Vec<String> = por_painel
        .iter()
        .filter(|(id, n)| **n < PISO_POR_PAINEL && !declarados.contains(*id))
        .map(|(id, n)| format!("{id}: {n} rótulo(s) medidos (piso {PISO_POR_PAINEL})"))
        .collect();
    assert!(
        mudos.is_empty(),
        "estes painéis são pintados VAZIOS e ninguém o declarou — ou eles ganham uma armação em \
         `paineis_armados::TABELA`, ou entram em `PAINEIS_MEDIDOS_VAZIOS` com o mundo que lhes \
         falta:\n  {}",
        mudos.join("\n  ")
    );
    // ⭐ **A metade justa:** um painel que passou a encher-se sai da lista, senão a declaração
    //    cobre o dia em que ele voltar a esvaziar-se.
    let ressuscitados: Vec<String> = PAINEIS_MEDIDOS_VAZIOS
        .iter()
        .filter(|(id, _)| por_painel.get(id).is_some_and(|n| *n >= PISO_POR_PAINEL))
        .map(|(id, _)| (*id).to_string())
        .collect();
    assert!(
        ressuscitados.is_empty(),
        "estes painéis já são medidos cheios — APAGUE a declaração: {}",
        ressuscitados.join(", ")
    );
}

/// ⭐⭐⭐ **UMA FIXTURA NÃO DEIXA NADA PARA TRÁS.**
///
/// ⛔⛔ **Nasceu de uma mutação SOBREVIVENTE (2026-09-19):** apagar o `desarma` de uma armação não
/// acordava gate nenhum — *o `desarma` era uma promessa escrita num doc-comment*. E o custo dela
/// é da suíte inteira: estas portas são `thread_local` e o binário de teste corre todos os módulos
/// na mesma thread, logo o que uma fixtura deixa é o documento que o gate seguinte mede.
///
/// A régua é a única honesta: pinta VAZIO, arma, desarma, pinta VAZIO outra vez — e as duas leituras
/// do vazio têm de ser **a mesma**.
///
/// ⭐ E ela apanhou logo uma fuga real: desarmar a Hierarquia devolvia a árvore à fixtura
/// (`clear_live_hierarchy`) e **deixava o contador de componentes em `42`** — ele é outra porta, e
/// *uma porta que o `arma` usa e o `desarma` esquece é exactamente o que este gate existe para ver*.
#[test]
fn uma_fixtura_nao_deixa_nada_para_tras() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let viewport = Rect {
        x: 0.0,
        y: 0.0,
        w: 1366.0,
        h: 1024.0,
    };
    let mut sujos = Vec::new();
    let mut visitadas = 0usize;
    ph2d_editor_core::panel::with_registry(|reg| {
        for painel in reg.panels_mut() {
            let id = painel.manifest.id;
            let Some(arm) = super::paineis_armados::TABELA
                .iter()
                .find(|a| a.painel == id)
            else {
                continue;
            };
            visitadas += 1;
            let vazio = |painel: &mut ph2d_editor_core::panel::ErasedPanel| {
                let mut host = MockPanelHost::new();
                painel.populate(host.store_mut());
                abre_as_gavetas(host.store_mut());
                host.medindo_a_pintura_do_registo(painel, viewport)
                    .into_iter()
                    .map(|m| (m.texto, m.pintado, m.largura.to_bits()))
                    .collect::<Vec<_>>()
            };
            let antes = vazio(painel);
            {
                let mut host = MockPanelHost::new();
                (arm.arma)(host.store_mut());
                painel.populate(host.store_mut());
                abre_as_gavetas(host.store_mut());
                let _ = host.medindo_a_pintura_do_registo(painel, viewport);
            }
            (arm.desarma)();
            let depois = vazio(painel);
            if antes != depois {
                sujos.push(format!(
                    "{id}: o vazio mede {} rótulos antes de armar e {} depois de desarmar",
                    antes.len(),
                    depois.len()
                ));
            }
        }
    });
    assert!(
        visitadas == super::paineis_armados::TABELA.len(),
        "o gate visitou {visitadas} das {} armações — uma delas nomeia um painel que o registo \
         não tem, e uma armação sobre um painel ausente nunca corre",
        super::paineis_armados::TABELA.len()
    );
    assert!(
        sujos.is_empty(),
        "estas fixturas deixaram estado para trás — o `desarma` não desfaz tudo o que o `arma` \
         fez:\n  {}",
        sujos.join("\n  ")
    );
}

/// ⛔⛔ **O CENSO DE OBSOLESCÊNCIA — sem ele a catraca vira LICENÇA.**
///
/// Uma entrada que já não descreve corte nenhum é uma linha que deixa passar o corte seguinte com
/// o mesmo texto noutro sítio. ⇒ curar um rótulo **obriga** a apagar a linha dele.
#[test]
fn nenhuma_linha_da_divida_ficou_obsoleta() {
    let tudo = varre();
    // ⚠️ **Uma linha cujo painel esta build não liga não é obsoleta — é INVISÍVEL.** Sem esta
    //    cerca, a corrida com as features pobres (`-p` sozinho, 24 painéis) acusaria as duas
    //    linhas do `flip_frames` de já não descreverem nada, e a cura seria apagá-las.
    //
    // ⛔⛔ **E os presentes são os do REGISTO, nunca os que PINTARAM alguma coisa** — a 1.ª
    //    redacção fazia o segundo, e o furo apareceu na primeira cura: o Inspector mede **um**
    //    rótulo (o painel vazio), e ao fazê-lo QUEBRAR ele deixou de registar seja o que for ⇒
    //    saiu da população e a linha de dívida dele passou a ser saltada **para sempre**, em
    //    silêncio. *Uma catraca cuja população encolhe com a cura vira licença* — a mesma forma
    //    que o piso do `every_host_that_rewrites_verts` já pagou.
    let presentes = paineis_do_registo();
    // ⭐ A metade justa da lista NOVA: uma linha que já não descreve corte nenhum sai.
    // ⚠️ **As DUAS listas, de propósito:** a que espera cura e a que o dono dispensou. Um corte
    //    que deixa de acontecer torna a linha obsoleta em qualquer uma delas — *uma lista isenta
    //    de censo é a catraca a virar licença*, e a de baixo existe precisamente para morrer no
    //    dia em que o painel sair.
    let armadas_mortas: Vec<String> = A_PASSAGEM_ARMADA_AINDA_CORTA
        .iter()
        .chain(FORA_POR_DECISAO_DO_DONO)
        .filter(|(painel, _, _)| presentes.contains(painel))
        .filter(|(painel, texto, _)| {
            !tudo
                .iter()
                .any(|a| a.armado && a.painel == *painel && a.m.texto == *texto && !a.m.coube())
        })
        .map(|(painel, texto, porque)| format!("{painel} (armado): {texto:?} ({porque})"))
        .collect();
    assert!(
        armadas_mortas.is_empty(),
        "estas linhas da dívida das passagens ARMADAS já não descrevem corte nenhum — \
         APAGUE-AS:\n  {}",
        armadas_mortas.join("\n  ")
    );
    let obsoletas: Vec<String> = CORTADOS_HOJE
        .iter()
        .filter(|(painel, _)| presentes.contains(painel))
        .filter(|(painel, texto)| {
            !tudo
                .iter()
                .any(|a| a.painel == *painel && a.m.texto == *texto && !a.m.coube())
        })
        .map(|(painel, texto)| format!("{painel}: {texto:?}"))
        .collect();
    assert!(
        obsoletas.is_empty(),
        "estas linhas da dívida já não descrevem corte nenhum — APAGUE-AS (é o que uma cura \
         deixa para trás):\n  {}",
        obsoletas.join("\n  ")
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// ⭐⭐⭐ **A TABELA SABE PRODUZIR ISTO?** — o censo que só o DONO conseguia correr.
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// ⭐ As TABELAS de string do app — derivadas do directório, nunca uma lista à mão.
fn tabelas() -> Vec<String> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("crates/<crate>/ tem dois pais")
        .join("crates/ph2d-i18n/src");
    let mut v: Vec<String> = std::fs::read_dir(&dir)
        .expect("a pasta da tabela existe")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "rs"))
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect();
    v.sort();
    v
}

/// O que a tabela sabe devolver: os textos EXACTOS e os MODELOS (com `{marcador}`).
struct Tabela {
    exactos: std::collections::BTreeSet<String>,
    /// `(prefixo, pedaços fixos do meio, sufixo)` de um modelo, para casar sem uma regex.
    modelos: Vec<Vec<String>>,
}

impl Tabela {
    fn ler() -> Self {
        let mut exactos = std::collections::BTreeSet::new();
        let mut modelos = Vec::new();
        for src in tabelas() {
            for par in ph2d_label_census::keys::declared_pairs_in(&src) {
                if par.texto.contains('{') {
                    // ⚠️ Um MODELO (`"{n} entities"`): o painel pinta-o PREENCHIDO, logo a
                    // comparação é pelos pedaços FIXOS, na ordem em que eles aparecem.
                    let pedacos: Vec<String> = par
                        .texto
                        .split(['{', '}'])
                        .step_by(2)
                        .filter(|s| !s.is_empty())
                        .map(str::to_string)
                        .collect();
                    if !pedacos.is_empty() {
                        modelos.push(pedacos);
                    }
                }
                exactos.insert(par.texto);
            }
        }
        Self { exactos, modelos }
    }

    /// A tabela consegue produzir este texto?
    fn produz(&self, t: &str) -> bool {
        if self.exactos.contains(t) {
            return true;
        }
        // ⭐⭐ **UMA LINHA DE PARÁGRAFO não é um texto da tabela — é um PEDAÇO dele.**
        //
        // ⛔ Achado na varredura da árvore inteira: o `wet_tuning` pinta prosa com
        // `paint_text_block`, que a QUEBRA em linhas, e o censo das elisões mede **cada linha**.
        // Saíam acusados `"extensions (diffusion, backrun, fingering,"` e `"the tuning registry's
        // hidden group."` — os dois pedaços contíguos da mesma frase, que a tabela declara inteira.
        //
        // ⚠️ **O preço está declarado:** aceitar SUBSTRING afrouxa a régua — um rótulo curto escrito
        // à mão que por acaso caia dentro de uma frase longa da tabela passa. Ele fica do lado
        // BARATO (um falso negativo raro) contra a alternativa, que seria uma lista de isenções
        // sobre pedaços de frase — e esses mudam sempre que uma coluna muda de largura.
        if t.len() >= 8
            && self
                .exactos
                .iter()
                .any(|x| x.len() > t.len() && x.contains(t))
        {
            return true;
        }
        self.modelos.iter().any(|pedacos| {
            let mut resto = t;
            pedacos.iter().all(|p| match resto.find(p.as_str()) {
                Some(i) => {
                    resto = &resto[i + p.len()..];
                    true
                }
                None => false,
            })
        })
    }
}

/// ⭐ Os painéis cujas palavras NÃO são chrome — `(id, porquê)`.
///
/// ⛔ Uma isenção de painel inteiro é grosseira de propósito, e por isso ela tem a metade justa
/// abaixo: um painel que deixe de abrigar acusação nenhuma sai da lista.
const PAINEIS_QUE_NAO_SAO_CHROME: &[(&str, &str)] = &[
    (
        "widget_gallery",
        "e' a BANCADA de widgets: a razao de existir dela e' demonstrar cada controlo com texto de \
         AMOSTRA (`Item A`, `Entity name`, `Rect2Editor`, `\"muzzle\"`). Traduzir uma amostra e' \
         traduzir a regua. Decisao do dono, ja' registada para o `widget_lab` e para o \
         `Geometry Offset`.",
    ),
    (
        "widget_lab",
        "idem — e mais: metade do texto dele NOMEIA a medicao que ele faz (`1 · THE FOUR DESIGNS`, \
         `LIVE BOX — drag it`, `row 22`). O `Geometry Offset` ja' e' isencao NOMEADA no censo das \
         elisoes deste mesmo ficheiro, pela mesma razao.",
    ),
    (
        "authored",
        "as palavras deste painel sao do ARTISTA, nao do programa: ele desenha a arvore e o app \
         ESCREVE o codigo do painel (`src/generated/panel.rs`, com o gate \
         `the_generated_panel_is_what_the_emitter_emits` a compara-lo byte a byte com o que o \
         emissor produz). `Design`/`Preview`/`Code` sao conteudo do documento, como o nome de uma \
         camada — e o proprio gate do painel se chama `the_program_writes_no_word_into_this_panel`.",
    ),
];

/// ⭐⭐ **O texto ja' CORTADO é um artefacto da medição, não um rótulo.**
///
/// O censo das elisões regista o que cada painter MEDIU, e alguns medem de novo a forma já elidida
/// (`"Mas…"` do `audio_mixer`, cujo original `Master` é a dívida NOMEADA em [`CORTADOS_HOJE`]).
/// ⇒ um texto que acaba em reticência e cujo começo é começo de um texto da tabela é a mesma
/// palavra, medida duas vezes. ⛔ Uma linha de isenção por cada um deles seria uma lista que muda
/// sempre que uma coluna muda de largura.
fn e_a_mesma_palavra_ja_cortada(t: &str, tabela: &Tabela) -> bool {
    let Some(prefixo) = t.strip_suffix('\u{2026}') else {
        return false;
    };
    let prefixo = prefixo.trim_end();
    !prefixo.is_empty() && tabela.exactos.iter().any(|x| x.starts_with(prefixo))
}

/// ⭐⭐⭐ **TODA PALAVRA QUE UM PAINEL PINTA, A TABELA SABE PRODUZIR.**
///
/// # ⛔⛔ O buraco: os 30 censos lêem o FONTE de UMA crate, e o ecrã não tem fronteiras
///
/// Um rótulo escrito à mão pinta-se **exactamente igual** ao que veio da tabela — nada nesta árvore
/// os distingue. O `Idioma::Teste` distingue-os, e até hoje era **o DONO** quem o corria, a olho,
/// numa fotografia. Três defeitos desta família foram achados assim, um por report.
///
/// ⇒ este gate faz a mesma pergunta por construção: ele pinta **todo painel do registo** (a mesma
/// varredura do resto do ficheiro) e pergunta, de cada rótulo medido, se a **tabela sabe
/// produzi-lo** — exacto, ou preenchendo um modelo `{marcador}`.
///
/// ⚠️ **Ele é sólido num sentido só, e isso está declarado:** um texto que a tabela NÃO produz é,
/// por construção, escrito no código; um que ela produz **pode** ser uma coincidência (uma palavra
/// escrita à mão igual a uma da tabela). *O erro fica do lado barato.*
///
/// # ⛔⛔ CORRA-O SOBRE A ÁRVORE INTEIRA — com `-p` ele vê MENOS painéis
///
/// Metade dos painéis do registo está atrás de uma **feature opcional** (`panel-wet-tuning`,
/// `panel-…`), e um `cargo test -p ph2d-panel-registry-init` não as acende: a unificação de
/// features de um build de WORKSPACE acende. ⇒ este gate fechou **VERDE** com `-p` e acusou **2**
/// rótulos na varredura da árvore — os dois do `wet_tuning`, que com `-p` nem é registado.
///
/// ⚠️ **A assimetria já estava medida no cabeçalho deste ficheiro** (a tabela de 18/09 tem duas
/// colunas: *«`-p` sozinho (24 painéis)»* e *«árvore inteira (28)»*), e o
/// [`PISO_DE_PAINEIS`] está no número MENOR de propósito, para o gate passar das duas maneiras.
/// *Um piso posto no menor dos dois deixa de afirmar o que acontece no maior* — e é lá que o app
/// de facto corre.
#[test]
fn toda_palavra_que_um_painel_pinta_a_tabela_sabe_produzir() {
    let tabela = Tabela::ler();
    // ⛔ Controlo de vacuidade: uma tabela vazia aprova tudo.
    assert!(
        // ⚠️ `4 155` textos DISTINTOS para `6 706` entradas — muitas chaves partilham a mesma
        //    palavra (`Size`, `Angle`, `Mix`), e o piso é sobre o CONJUNTO, não sobre as entradas.
        //    *Um piso copiado da grandeza vizinha reprova sobre uma régua correcta.*
        // ⚠️ E `415` modelos e não `606`: descodificar o `\u{…}` tirou `191` textos da classe
        //    MODELO — a chaveta do escape disfarçava-os de marcador. *O piso apanhou a mudança, que
        //    é para o que ele existe.*
        tabela.exactos.len() >= 4_000 && tabela.modelos.len() >= 380,
        "a régua leu {} textos e {} modelos — o leitor da tabela partiu-se, e um censo com a \
         tabela vazia acusa TUDO (ou, com ela cheia de nada, aprova tudo)",
        tabela.exactos.len(),
        tabela.modelos.len()
    );
    let mut crus: Vec<String> = varre()
        .iter()
        // ⛔⛔ **A passagem ARMADA fica de fora, e a razão é a PERGUNTA deste censo.**
        //
        // Ele pergunta *«esta palavra está escrita no CÓDIGO?»*, e a fixtura do
        // [`super::o_inspector_armado`] põe na mão do painel um DOCUMENTO — nomes de objecto
        // (`Hero`), de tag (`Enemy` · `Flying`), de estado (`Closed` · `Open`), de acção
        // (`hit → Hide · Door`). ⚠️ **A tabela não os sabe produzir e nem devia**: eles são o que
        // o artista escreveu. Medido em 19/09, incluí-la acusa `12` textos, **os doze do
        // documento**, e a única cura disponível seria uma lista de isenções sobre palavras
        // inventadas por uma fixtura — *uma lista que não descreve o produto*.
        //
        // ⚠️ **O que esta cegueira custa está medido e é PEQUENO:** um literal escrito à mão
        // DENTRO de uma secção condicional não é visto aqui, e continua a ser visto pelos **30
        // censos lexicais**, que lêem o FONTE de `ph2d-panel-inspector` inteiro
        // (`every_word_this_panel_shows_comes_from_the_string_table`). *Este censo é a segunda
        // testemunha, não a única.*
        .filter(|a| !a.armado)
        // ⭐⭐ **O que é um RÓTULO já tem régua nesta casa** — a mesma `is_language` dos 30 censos
        //    lexicais (duas letras ASCII adjacentes, fora de um marcador). Sem ela a lista abre com
        //    `231` acusados e a esmagadora maioria são VALORES: `"0.010"`, `"-9.81"`, `"+0.00"`,
        //    `"0:00.0 / 0:00.0"`, `"▶"`, `"☰"`. *Um número que um painel pinta não é uma palavra, e
        //    uma lista de isenções sobre eles seria uma lista de números escritos à mão.*
        .filter(|a| ph2d_label_census::is_language(&a.m.texto))
        .filter(|a| {
            !PAINEIS_QUE_NAO_SAO_CHROME
                .iter()
                .any(|(id, _)| *id == a.painel)
        })
        .filter(|a| !tabela.produz(&a.m.texto))
        .filter(|a| !e_a_mesma_palavra_ja_cortada(&a.m.texto, &tabela))
        .map(|a| format!("{} · {:?}", a.painel, a.m.texto))
        .collect();
    crus.sort();
    crus.dedup();
    assert!(
        crus.is_empty(),
        "estes {} rótulos são pintados por um painel e a tabela de strings NÃO os sabe produzir — \
         eles estão escritos no código, e nenhum dos 30 censos os vê:\n  {}",
        crus.len(),
        crus.join("\n  ")
    );
}

/// ⭐ **A METADE JUSTA da lista de painéis isentos** — sem ela, um painel que já não escreva uma
/// palavra à mão fica isento para sempre, e a isenção passa a cobrir o que aparecer amanhã.
#[test]
fn nenhum_painel_isento_deixou_de_abrigar_uma_palavra_escrita_a_mao() {
    let tabela = Tabela::ler();
    let todos = varre();
    let mut mortas = Vec::new();
    for (id, porque) in PAINEIS_QUE_NAO_SAO_CHROME {
        assert!(porque.len() > 40, "a isenção `{id}` não diz o mecanismo");
        let abriga = todos.iter().any(|a| {
            a.painel == *id
                && ph2d_label_census::is_language(&a.m.texto)
                && !tabela.produz(&a.m.texto)
                && !e_a_mesma_palavra_ja_cortada(&a.m.texto, &tabela)
        });
        if !abriga {
            mortas.push(format!(
                "`{id}`: já não pinta uma única palavra que a tabela não saiba produzir — apague a \
                 linha, e o painel passa a ser guardado como os outros"
            ));
        }
    }
    assert!(mortas.is_empty(), "{}", mortas.join("\n"));
}

/// ⭐⭐⭐ **TODA PALAVRA CORTADA POR ESTE APP TEM BALAO.**
///
/// ⛔⛔ **A ordem do dono (2026-09-19) foi *«encurtar · balao ao passar o rato»*, e esta e a
/// metade que vale para o texto que ELE escreve.** Encurtar cura os nomes que sao NOSSOS; o nome
/// de um objecto, de uma ancora ou de uma propriedade de script nao tem dono nenhum deste lado, e
/// a unica resposta para esses e poder ler o inteiro.
///
/// ⭐⭐ **A regua e a MESMA varredura que achou os `128` cortes** — o instrumento que descobriu o
/// problema e o que prova a cura. Para cada corte que o censo das elisoes viu, tem de existir um
/// balao com o MESMO texto; quando nao existe, e porque o pintor daquele sitio nao embrulhou a
/// pintura num [`ph2d_editor_core::text_elide::balao::na_area`] e o app nao sabe ONDE por a bolha.
///
/// ⚠️ **A mensagem de falha nomeia o PINTOR** (`Medido::onde`), e nao so o texto: sem isso a cura
/// seria arqueologia, que e a lei que aquele campo existe para pagar.
#[test]
fn toda_palavra_cortada_tem_balao() {
    let (tudo, baloes) = ph2d_editor_core::text_elide::balao::medindo(varre);

    let mut disponiveis: std::collections::BTreeMap<String, usize> = Default::default();
    for (_, texto) in &baloes {
        *disponiveis.entry(texto.clone()).or_default() += 1;
    }

    let mut sem_balao: Vec<String> = Vec::new();
    let mut cortes = 0usize;
    for a in tudo.iter().filter(|a| !a.m.coube()) {
        cortes += 1;
        match disponiveis.get_mut(&a.m.texto) {
            Some(n) if *n > 0 => *n -= 1,
            _ => sem_balao.push(format!(
                "{}: {:?} cortado em {:.1} px e SEM balao — o pintor de {} nao chama \
                 `text_elide::balao::na_area`",
                a.onde(),
                a.m.texto,
                a.m.largura,
                a.m.onde
            )),
        }
    }

    // ⚠️ **PISO DE POPULACAO:** uma varredura que deixasse de cortar seja o que for devolveria
    //    ZERO sem balao e leria-se como aprovacao — a forma exacta que este repo ja pagou num
    //    censo por prefixo. O numero sai da medicao de 2026-09-19 (`128` cortes nos 4 degraus,
    //    e a escada corre cada painel 4 vezes).
    assert!(
        cortes >= 100,
        "a varredura viu {cortes} cortes (piso 100) — ou o app deixou de cortar (verifique a \
         escada), ou este censo deixou de medir. Um censo que varre menos fica verde."
    );
    sem_balao.sort_unstable();
    sem_balao.dedup();
    assert!(
        sem_balao.is_empty(),
        "estas palavras sao cortadas e o artista NAO tem como as ler:\n  {}",
        sem_balao.join("\n  ")
    );
}

/// ⭐ **O CONTROLO: o balao nao inventa.**
///
/// ⚠️ Sem esta metade, um `na_area` que registasse TODO rotulo — cortado ou nao — passaria o gate
/// de cima e encheria a tela de bolhas sobre texto que se le perfeitamente. *Uma regua que so
/// verifica a presenca aprova o excesso.*
#[test]
fn o_balao_so_guarda_o_que_foi_cortado() {
    let (tudo, baloes) = ph2d_editor_core::text_elide::balao::medindo(varre);
    let cortados: std::collections::BTreeSet<&str> = tudo
        .iter()
        .filter(|a| !a.m.coube())
        .map(|a| a.m.texto.as_str())
        .collect();
    let intrusos: Vec<&str> = baloes
        .iter()
        .map(|(_, t)| t.as_str())
        .filter(|t| !cortados.contains(t))
        .collect();
    assert!(
        intrusos.is_empty(),
        "o balao guardou texto que COUBE — ele passaria a aparecer sobre rotulos legiveis:\n  \
         {intrusos:?}"
    );
}

/// ⛔⛔⛔ **NENHUM RÓTULO DO APP PINTA UMA CHAVE** — o report do dono de 2026-09-21, com foto.
///
/// A secção `SHAPE ▸ Texture` do Painter mostrava quatro fileiras assim:
///
/// ```text
/// paint_brush.pattern_param.contrast     0.500
/// paint_brush.pattern_param.brightness   0.500
/// ```
///
/// ⭐ **A tabela sabia traduzi-las** (`tr` devolve `Contrast`, `Brightness`, `Turbulence`,
/// `Rings`) — o que faltava era **alguém chamar `tr`**. O laço que monta estas fileiras estava
/// escrito **três vezes** e duas traduziam; a terceira passava a chave crua.
///
/// # ⛔ Porque nenhum dos 30 censos de texto a via
///
/// Eles perguntam *«este LITERAL vem da tabela?»* e varrem o **fonte**. Ali não há literal nenhum:
/// há um campo (`s.label`) que por acaso é uma chave. ⇒ *um censo de fonte é cego a um rótulo que
/// o programa CALCULA*, e a régua que o apanha tem de ler o **ECRÃ**.
///
/// # ⭐⭐ O discriminador é a PRÓPRIA tabela, e não a forma do texto
///
/// Um texto pintado que a tabela **sabe traduzir** é, por construção, uma chave que alguém
/// esqueceu de traduzir. ⛔ Uma régua de FORMA (*«tem ponto e não tem espaço»*) acusaria `0.5` e
/// nomes de ficheiro; esta não tem falso positivo nenhum.
#[test]
fn nenhum_rotulo_do_app_pinta_uma_chave() {
    let achados = varre();
    let cruas: Vec<String> = achados
        .iter()
        .filter(|a| {
            let t = a.m.texto.as_str();
            // ⭐ A tabela conhece-o ⇒ ele é uma CHAVE, e o pintor esqueceu-se de a traduzir.
            !t.is_empty() && ph2d_i18n::tr(t) != t
        })
        .map(|a| {
            format!(
                "{}: {:?} (a tabela diz {:?})",
                a.onde(),
                a.m.texto,
                ph2d_i18n::tr(&a.m.texto)
            )
        })
        .collect();
    assert!(
        cruas.is_empty(),
        "o app está a pintar {} CHAVE(S) de tradução no lugar do nome:\n  {}\n\n\
         ⇒ quem pinta esse rótulo passou a chave crua em vez de a mandar ao `ph2d_i18n::tr`.",
        cruas.len(),
        cruas.join("\n  ")
    );
}
