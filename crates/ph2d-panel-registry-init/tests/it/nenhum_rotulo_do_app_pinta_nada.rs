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
//! | cortados depois da cura da caixa de número | **12** | **14** — `CORTADOS_HOJE` |
//! | cortados no idioma de teste | ~130, em 16 painéis | — |
//!
//! ⚠️ **Um painel pintado com o estado de FÁBRICA mostra o estado VAZIO dele** (o Inspector sem
//! selecção mede **um** rótulo). A varredura mede o que um painel pinta **sozinho**; o piso é
//! GLOBAL de propósito, porque um piso por painel seria uma lista de números escritos à mão sobre
//! populações que dependem do documento.

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
/// mas **estes catorze são todos texto de INTERFACE** — rótulos, valores e frases de estado vazio,
/// que a casa escreve e a casa dimensiona. ⇒ a lista é dívida, nunca licença.
const CORTADOS_HOJE: &[(&str, &str)] = &[
    // ⛔ O nome da faixa mestra no Audio Mixer com o dock estreito — **declarado** em 18/09: a
    //    tira do master mede menos do que a palavra pede, e alargá-la é decisão de produto.
    ("audio_mixer", "Master"),
    // ✅ **Os três VALORES cortados (`+0.00 EV`, `0.500`, `1.500`) SAÍRAM desta lista em 18/09** —
    //    a caixa de número contava a borda do stepper DUAS vezes (`56 → 24 px` de orçamento) e hoje
    //    conta uma (`32`). Quem os apagou daqui foi este gate: o censo de obsolescência acusou as
    //    três linhas como já não descrevendo corte nenhum. Mecanismo:
    //    `ph2d-editor-core/tests/it/a_caixa_de_numero_nao_corta_o_numero.rs`.
    // ⛔⛔ **Estes dois só existem quando a build liga TODOS os painéis** — ver a nota do
    //    `PISO_DE_MEDICOES`. Eles são a prova de que a lista capturada com `-p <crate>` sozinho
    //    está incompleta por CONSTRUÇÃO.
    ("flip_frames", "Linear"),
    ("flip_frames", "No Cycle"),
    // ⛔ Opções e rótulos que não cabem no chip deles.
    ("grid_snap", "Line / Neighbors"),
    ("timeline", "PingPong"),
    ("widget_gallery", "Float"),
    ("widget_gallery", "Color"),
    ("widget_gallery", "filter"),
    ("widget_lab", "Geometry Offset"),
    // ⚠️ FRASES de estado vazio elididas a UMA linha. A cura provável não é largura: é elas
    //    QUEBRAREM (o `Lines::Wrap` já existe), que é decisão de desenho.
    (
        "inspector",
        "Select an entity in the Hierarchy to inspect its properties.",
    ),
    ("tags", "No tags yet. Press + New to make the first one."),
    (
        "widget_gallery",
        "Canonical widget showcase \u{b7} reference for peripheral agents",
    ),
    (
        "widget_lab",
        "Bar \u{b7} the fill is the whole background \u{b7} reads at a glance \u{b7} competes with \
         the number",
    ),
    (
        "widget_lab",
        "268 = today's Inspector \u{b7} 184 = the app's MINIMUM column \u{b7} 140 and 110 = tablet",
    ),
];

/// Uma medição, com o painel que a fez e o viewport em que ela aconteceu.
struct Achado {
    painel: &'static str,
    viewport_w: f32,
    m: Medido,
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
                        m,
                    });
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
                "{} @ {:.0}px: {:?} não cabe em {:.1} px — nem a reticência",
                a.painel, a.viewport_w, a.m.texto, a.m.largura
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
                "{} @ {:.0}px: {:?} vira {deformado:?} e some — a caixa tem {:.1} px",
                a.painel, a.viewport_w, a.m.texto, a.m.largura
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
        .map(|a| {
            format!(
                "{} @ {:.0}px: {:?} -> {:?} em {:.1} px",
                a.painel, a.viewport_w, a.m.texto, a.m.pintado, a.m.largura
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
    let presentes: std::collections::BTreeSet<&str> = tudo.iter().map(|a| a.painel).collect();
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
