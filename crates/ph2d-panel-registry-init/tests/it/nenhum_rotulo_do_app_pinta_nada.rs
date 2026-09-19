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
//! | curados no mesmo dia | **16** |
//! | por curar, nomeados | **8** — `A_PASSAGEM_ARMADA_AINDA_CORTA` |
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

/// ⚠️ **Três, e não uma:** a largura do encaixe sai do viewport, e um rótulo que cabe a `1920`
/// pode não caber a `1280`. ⛔ Elas **não** cobrem a coluna que o artista aperta à mão (isso vive
/// no `~/.ph2d/layout.txt`, fora do repo) — essa metade é do gate de coluna de cada painel.
const VIEWPORTS: &[(f32, f32)] = &[(1366.0, 1024.0), (1920.0, 1080.0), (1280.0, 800.0)];

/// ⛔ **Piso de população.** *Sem ele, uma varredura que deixasse de pintar leria zero cortes e
/// passaria por aprovação* — a forma exacta que este repo já pagou com o censo por prefixo de nome.
///
/// ⛔⛔⛔ **A POPULAÇÃO DEPENDE DAS FEATURES, e isso apanhou este gate no dia em que ele nasceu.**
/// Quatro painéis (`flip`, `flip_frames`, `painter_layers`, `wet_tuning`) **não** estão no `default`
/// desta crate — eles chegam pelo shell. ⇒ `cargo test -p ph2d-panel-registry-init` mede **24**
/// painéis e `3 146` rótulos; a árvore inteira, com a unificação de features, mede **28** e
/// **`4 022`** — e traz **dois cortes a mais**, que a primeira lista não tinha.
///
/// ⚠️ *Uma crate testada sozinha é testada num mundo que o produto não habita* (`CLAUDE.md` §2), e
/// aqui isso lê-se como uma catraca completa. O piso cobre os dois mundos de propósito, e o censo
/// de obsolescência **salta** as linhas cujo painel esta build não liga — senão a corrida com as
/// features pobres acusaria de obsoletas duas linhas vivas.
const PISO_DE_MEDICOES: usize = 2_800;

/// ⛔ E o piso de PAINÉIS: uma varredura que registasse zero painéis passaria os quatro gates.
const PISO_DE_PAINEIS: usize = 24;

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
/// ⛔⛔ **Os `8` que ficam são UMA família e por isso não se curam um a um:** todos são fileiras
/// cuja coluna de nome é a da SECÇÃO ([`ph2d_editor_core::widget::property_box::Seccao`]), medida
/// para caber ao lado de um CONTROLO de campos — e uma fileira de **marcar** tem um controlo de
/// `18 px`, uma de **lista** tem o nome do artista. *Alargar a coluna de uma delas parte o
/// alinhamento da secção inteira, que é a doença que aquela porta existe para curar.* ⇒ a wave
/// seguinte é da lei da linha de propriedade, não deste censo.
///
/// ⚠️ **Cada linha diz o NÚMERO e o MECANISMO**, e a lista **só encolhe** — o censo de
/// obsolescência abaixo reprova quem deixar de descrever um corte.
const A_PASSAGEM_ARMADA_AINDA_CORTA: &[(&str, &str, &str)] = &[
    // Fileira de MARCAR: o controlo precisa de `18 px` e a coluna do nome fica com `174`, porque
    // ela é medida para as fileiras de CAMPOS da mesma secção. O nome pede `~190`.
    (
        "inspector",
        "Center (makes it a 9-slice Region)",
        "marcar · 174 px",
    ),
    (
        "inspector",
        "Show anchors at runtime (no game runtime yet)",
        "marcar · 174 px",
    ),
    // Fileira de LISTA: o texto é do DOCUMENTO (o nome que o artista deu à âncora / ao sinal / à
    // propriedade do script / à peça). ⚠️ Um corte aqui **não é sempre defeito** — o que é defeito
    // é a caixa: `26 px` para um nome não é uma caixa, é um resto.
    ("inspector", "hand_right", "lista · 26 px · nome do artista"),
    (
        "inspector",
        "2.50s · repeats · → respawn_done",
        "lista · 128 px · resumo com sinal do artista",
    ),
    (
        "inspector",
        "legacy_speed = 1 — not in the script",
        "lista · 189 px · nome do artista",
    ),
    (
        "inspector",
        "• AudioSource2D — was on \u{201c}footsteps\u{201d}",
        "lista · 229 px · nome da peça",
    ),
    // Rótulo de fileira numa secção cuja coluna é estreita por ter muitos campos.
    ("inspector", "Mounted On", "campos · 60 px"),
    // Botão dentro de uma fileira de lista: a legenda cresceu com o verbo e a caixa não.
    ("inspector", "x Remove Transition", "botão · 118 px"),
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
];

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
    viewport_w: f32,
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
        format!("{}{estado} @ {:.0}px", self.painel, self.viewport_w)
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
    for &(w, h) in VIEWPORTS {
        let viewport = Rect {
            x: 0.0,
            y: 0.0,
            w,
            h,
        };
        ph2d_editor_core::panel::with_registry(|reg| {
            visitados = reg.panels().len();
            for painel in reg.panels_mut() {
                let id = painel.manifest.id;
                let mut host = MockPanelHost::new();
                painel.populate(host.store_mut());
                for m in host.medindo_a_pintura_do_registo(painel, viewport) {
                    tudo.push(Achado {
                        painel: id,
                        viewport_w: w,
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
                    for m in host.medindo_a_pintura_do_registo(painel, viewport) {
                        tudo.push(Achado {
                            painel: id,
                            viewport_w: w,
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
         reticência. Uma varredura que lê pouco devolve ZERO cortes e lê-se como aprovação.",
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
                "{}: {:?} não cabe em {:.1} px — nem a reticência",
                a.onde(),
                a.m.texto,
                a.m.largura
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
    let mut mudos = Vec::new();
    for a in varre() {
        if !a.m.texto.chars().any(char::is_alphabetic) {
            continue;
        }
        let deformado = pseudo::deforma(&a.m.texto);
        let cabe = ts.prefix_width_weighted(&deformado, a.m.fonte, a.m.peso) <= a.m.largura;
        if !cabe && largura_da_reticencia(&mut ts, a.m.fonte, a.m.peso) > a.m.largura {
            mudos.push(format!(
                "{}: {:?} vira {deformado:?} e some — a caixa tem {:.1} px",
                a.onde(),
                a.m.texto,
                a.m.largura
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
        .filter(|a| !CORTADOS_HOJE.contains(&(a.painel, a.m.texto.as_str())))
        // ⭐ E a dívida que o PONTO CEGO escondia — por PAINEL e por TEXTO, como a irmã de cima:
        //    o mesmo rótulo pode caber num painel e não caber noutro.
        .filter(|a| {
            !a.armado
                || !A_PASSAGEM_ARMADA_AINDA_CORTA
                    .iter()
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
const PAINEIS_MEDIDOS_VAZIOS: &[(&str, &str)] = &[
    (
        "model3d",
        "pinta a arvore de um documento de campo implicito (`FieldDoc`), que e' COZIDO da \
         hierarquia da cena a cada quadro; sem mundo ECS ele nao tem uma peca para listar.",
    ),
    (
        "motion_graph",
        "pinta um GRAFO de nos vivo (`ph2d-nodegraph`), com o cartao e os pinos derivados do \
         manifesto de cada no; o arnes nao monta um grafo.",
    ),
    (
        "motion_params",
        "irmao do de cima: as fileiras dele sao os params do no ESCOLHIDO no grafo, logo ele e' \
         vazio enquanto nao houver grafo nem escolha.",
    ),
    (
        "sculpt3d",
        "o painel da escultura pinta o que a `AppGfx.sculpt3d` publica, e essa cena segura uma \
         surface de wgpu — o arnes corre sem dispositivo, logo sem peca.",
    ),
];

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
    let mut por_quadro: std::collections::BTreeMap<(&str, i32, bool), usize> =
        std::collections::BTreeMap::new();
    for a in &tudo {
        *por_quadro
            .entry((a.painel, a.viewport_w as i32, a.armado))
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
    let armadas_mortas: Vec<String> = A_PASSAGEM_ARMADA_AINDA_CORTA
        .iter()
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
