#![forbid(unsafe_code)]
//! ph2d-i18n — internationalization.
//!
//! **M13 stub.** This crate currently ships a **static English string
//! table** keyed by Fluent-style identifiers (`tool.trim_transparency.label`,
//! `tool.make_square.label`, …). The full Fluent/ICU MessageFormat
//! implementation is deferred per the milestone plan; this stub
//! gives consumers a stable call-site shape (`tr("key")`) that the
//! eventual Fluent bundle replaces transparently.
//!
//! # Why a stub, not Fluent right now
//!
//! Fluent requires runtime locale state + per-locale bundles +
//! plural-form resolution + a bundler-source story. None of those are
//! load-bearing for the current milestone (single-locale en-US editor).
//! Shipping the table now centralizes the **strings** so the Fluent
//! migration is a one-touch impl swap, not a 100-callsite rename.
//!
//! # Usage
//!
//! ```
//! use ph2d_i18n::tr;
//! // Image-tool labels are abreviados (cap 5 chars) pra caber no chip;
//! // o tooltip mantém o nome legível por extenso.
//! assert_eq!(tr("tool.trim_transparency.label"), "TRIM");
//! assert_eq!(tr("tool.trim_transparency.tooltip"), "Trim Transparency");
//! assert_eq!(tr("tool.unknown.key"), "tool.unknown.key"); // missing-key passthrough
//! ```

/// O que a família das INSTÂNCIAS diz (avisos dos verbos de prefab, a paleta de componentes).
mod app_components;
/// O que a família do MODELADOR de campo diz (peças, paleta de formas, importar/exportar).
mod app_field3d;
/// O que a família do FLIP diz (os avisos de pintar, preencher, colorir, esculpir).
mod app_flip;
/// O que a família do MOTION diz (as recusas de ligar, os conselhos do grafo, o cartão).
mod app_motion;
/// O que a família do PAINTER diz (texturas de pincel, a pré-visualização na GPU).
mod app_painter;
/// O que a família da FÍSICA diz (desenhar uma junta, o leitor de carga, o cartão do corpo).
mod app_physics;
/// O que a família da ESCULTURA diz (as recusas da retopologia, importar/exportar malhas).
mod app_sculpt3d;
/// O que a família do VETOR diz (os selos da booleana, importar SVG).
mod app_vec;
/// As strings do navegador de assets.
mod asset_browser;
/// As strings dos dois painéis de áudio (editor + mixer).
mod audio;
/// As palavras que os MOTORES de áudio publicam (codec, plataforma de entrega, lei de variação).
mod audio_engines;
/// As strings do RACK do Audio Editor (efeitos, parâmetros, presets, unidades).
mod audio_fx;
/// ⭐⭐ **Os nomes das MISTURAS** que o motor `ph2d-blend-mode` publica — pintados por três painéis.
mod blend_modes;
/// As strings dos MENUS da moldura (barra de menus, menus de contexto, paleta de comandos).
mod chrome_menus;
/// As strings do RESTO da moldura (barra do topo, HUD, diálogos, seletor de cor, cartão de instância).
mod chrome_panes;
/// As strings da BARRA DE FERRAMENTAS (o rail esquerdo e a fila horizontal).
mod chrome_rail;
/// ⭐⭐ **As palavras do MOTOR DA CENA** (`ph2d-ecs`) — a 7.ª fatia da fronteira dos motores, e a
/// primeira achada por um instrumento.
mod ecs_scene;
/// As strings dos dois painéis do Flip (o painel e a tira de quadros).
mod flip;
/// As strings do painel Grid Settings.
mod grid_snap;
/// As strings dos cinco painéis das ferramentas de imagem.
mod image_tools;
/// ⭐ **A JANELA do Input Map** — irmã por ASSUNTO, cortada do pai pelo tecto de LOC em 2026-09-17.
mod input_map;
/// As strings dos painéis do Motion (grafo, params) e dos editores ricos partilhados.
mod motion_panels;
/// As strings da SHELL (avisos, diálogos, nomes por omissão).
mod shell;
/// As strings da SHELL sobre mídia (imagem, folhas, importar/exportar, áudio).
mod shell_media;
/// As strings do painel TIMELINE — cortadas daqui pelo tecto de LOC (ver o cabeçalho delas).
mod timeline;
mod vector;
/// As strings que o MOTOR publica para o painel de Vector (efeitos, filtros, misturas).
mod vector_engine;

mod chrome;
/// As palavras do catálogo de componentes — o 1.º MOTOR a falar pela tabela.
mod component_catalog;
// ⚠️ **O doc que aqui estava era o do [`tags`]** e descrevia o painel errado — um `///` colado ao
//    item seguinte é o mesmo acidente que esta fatia quase repetiu duas declarações abaixo. Ele
//    voltou para o irmão a que pertence; este módulo fica sem doc, que é honesto, em vez de com um
//    que descreve outra coisa.
mod factory;
mod inspector;
/// ⭐ **As secções de JOGO do Inspector** (TOP-20 #9..#18 — tags, fábrica, ciclo de vida, mover de
/// vista de cima, projéctil, máquina de estados, script, partículas). ⚠️ Tabela IRMÃ e não o fim
/// da `inspector.rs`: com os 151 braços destes lá dentro ela passava o tecto de 700 LOC do HR-18,
/// e um corte por ASSUNTO é o que esse tecto pede.
mod inspector_game;
mod inspector_player;
mod model3d;
/// Os nomes dos nós — a 3.ª fatia da fronteira dos motores.
mod node_catalog;
/// ⭐⭐ **Os nomes das SECÇÕES** do painel de params de um nó — 38 palavras sobre 229 sítios.
mod node_groups;
/// As OPCOES de cada selector de nó — a 5.ª fatia da fronteira dos motores.
mod node_options;
/// Os rótulos de PARAMETRO dos nós — a 4.ª fatia, e a maior.
mod node_params;
/// A metade `motion` dos rotulos de parametro — o corte de LOC por familia.
mod node_params_motion;
/// As palavras dos motores do pincel e dos efeitos — a 2.ª fatia da fronteira.
mod paint_engines;
/// ⭐ **As RAZÕES de uma fileira apagada** (18/09) — irmão de ASSUNTO do [`model3d`], e não de
/// painel: o corte foi imposto pelo tecto de LOC dele e separa *nomes de coisas* de *frases para o
/// artista*.
mod model3d_inert;
/// ⭐⭐⭐ **O vocabulário do BRILHO da cena 3D** (`docs/Render3d/12`, a `W7`) — irmão do
/// [`model3d_render`] por RESPONSABILIDADE e para o não deixar chegar ao tecto de LOC.
mod model3d_bloom;
/// ⭐⭐⭐ **A APRESENTAÇÃO da cena 3D** — o olhar, a exposição e a camada de ESTILO
/// (`docs/Render3d/03` e `05`). ⚠️ Ele NÃO entra na cadeia do [`tr`]: quem o alcança é o braço
/// final do [`model3d`], que lhe delega — *a tabela do documento aponta para a da apresentação.*
mod model3d_render;
mod painter_layers;
/// ⭐⭐ **A COR, a CURVA, o COMANDO e as RECUSAS da timeline** — quatro motores numa fatia.
mod quatro_motores;
mod sculpt3d;
/// ⭐⭐ **As palavras do MOTOR da escultura** — a 6.ª fatia da fronteira dos motores.
mod sculpt_engine;
/// ⭐ **As strings do painel TAGS** (TOP-20 #9) — irmão de tabela, por assunto.
mod tags;
/// ⭐⭐ **As cinco recusas da ÁRVORE de tags**, publicadas pela folha `ph2d-tags`.
///
/// ⚠️ Tabela IRMÃ da [`tags`] de propósito: aquela são as palavras que o PAINEL escreve, estas são
/// as que a LEI produz. Juntá-las esconderia qual das duas uma fatia futura está a mexer.
mod tags_engine;
/// ⭐⭐ **As palavras do DESIGN SYSTEM** — temas, desenhos de slider, recusas de fórmula.
mod tokens;
/// ⭐⭐ **As palavras dos MOTORES DAS FERRAMENTAS** — a 8.ª fatia da fronteira dos motores.
mod tool_engines;
/// ⭐ **O vocabulário do mover de VISTA DE CIMA** (TOP-20 #13) — os três selectores que o
/// `topdown_edits` declara e o Inspector pinta. Tabela irmã pela lei do assunto (ver [`tags`]).
mod topdown;

/// ⭐⭐ **A substituição de MARCADORES** — `tr_with`, a lei que lê um modelo.
pub mod formato;
/// ⭐⭐⭐ **QUAL IDIOMA ESTÁ A FALAR** — o estado que esta crate não teve durante nove fatias.
pub mod idioma;
/// ⭐⭐⭐ **A lei do IDIOMA DE TESTE** — o que deforma cada palavra que sai desta tabela.
pub mod pseudo;

pub use formato::{tr_with, tr_with_em};
pub use idioma::{Idioma, idioma};

/// A palavra que uma chave diz, **no idioma desta corrida**.
///
/// Uma chave desconhecida volta CRUA (*missing-key passthrough*), para a entrada que falta se ver
/// no ecrã em vez de pintar vazio — e é sobre isso que meio repo pergunta *«esta chave existe?»*
/// com `tr(k) != k` (ver [`TextKey::key`]).
///
/// ⭐⭐⭐ **O idioma sai de [`idioma()`]** (`PH2D_LANG=teste`), e essa é a única maneira que este
/// repo tem de perguntar se uma palavra do ecrã veio mesmo daqui: os **30** censos do HR-15 lêem o
/// FONTE de uma crate, e um rótulo esquecido no pintor pinta-se igual ao que veio da tabela.
/// ⚠️ **O caminho de omissão é byte a byte o de sempre** — o [`Idioma::Ingles`] devolve a tabela
/// crua, sem uma comparação a mais no laço de desenho além do `match`.
#[must_use]
pub fn tr(key: &str) -> &'static str {
    tr_em(idioma(), key)
}

/// O mesmo, com o idioma DADO — a porta pela qual um gate mede o idioma de teste sem mexer no
/// ambiente do processo.
///
/// ⚠️ **Um teste que escrevesse `PH2D_LANG` mudaria o processo INTEIRO**, e a suíte deste repo corre
/// em paralelo: seria mais um membro da família de flakes de fan-out, e dos caros — um gate a medir
/// inglês enquanto o vizinho pediu o idioma de teste.
#[must_use]
pub fn tr_em(idioma: Idioma, key: &str) -> &'static str {
    let ingles = tr_ingles(key);
    match idioma {
        Idioma::Ingles => ingles,
        Idioma::Teste => pseudo::traduz(key, ingles),
    }
}

/// A tabela, tal como está escrita.
fn tr_ingles(key: &str) -> &'static str {
    match key {
        // ── Vector panel (ADR-0108/0112) — section headers, tool modes and the
        // shape catalogue. The panel is a 17-section stack; every section title
        // and every chrome word routes through here (the shape NAMES themselves
        // stay in the `ph2d-tool-vector` catalogue, which is their single source).
        // O Falloff modula a FORÇA do deformador abaixo dele na pilha; o card diz para onde ele
        // aponta, para que um Falloff sozinho (sem deformador abaixo) não pareça quebrado.
        // A SIMETRIA de desenho (W6.3). ⚠️ Os rótulos dos TIPOS não estão aqui: eles moram em
        // `ph2d_symmetry::SymmetryKind::label`, ao lado do enum, porque uma segunda lista
        // divergiria da primeira no dia em que o vocabulário ganhasse o quinto tipo.
        // A MOLDURA (plano UI/UX W0) — o contêiner.
        // **Os TOKENS** (plano UI/UX W4): a row que diz de que token a propriedade segue.
        // A linha que SOLTA a propriedade — ela volta ao literal do documento.
        // O interruptor do painel autorado (plano UI/UX W8b.2) — a moldura descreve um painel, e
        // este chip o mostra ao lado, docado.
        // **AS ÂNCORAS** (plano UI/UX W3) — a regra do filho que NÃO está num fluxo.
        // ⚠️ A vertical é nomeada pelo que se VÊ ("Top"/"Bottom"), e não pelo sinal: o documento é
        // Y-up, então "Top" é a âncora 1. A tradução mora numa tabela só, na shell.
        // **OS COMPONENTES** (plano UI/UX W5) — o prefab: mestre, instância, override.
        // ⚠️ "Main" e não "Master": é a palavra que o Figma passou a usar, e é a que aparece no
        // readout de órfã — os dois lados têm de falar a mesma.
        // **AS DIFERENÇAS** (W5b) — a lista de peças, o absorver e a troca de mestre.
        // ⚠️ Rótulo PRÓPRIO para a cor que esta cópia autorou: sem ele, *"esta peça está
        // diferente"* só se descobre carregando em Reset e vendo o que muda.
        // **OS VARIANTS** (W5c) — que versão do componente esta cópia é.
        // ⚠️ Este rótulo só é usado no modo de NOMES CRUS: quando os mestres irmãos declaram
        // propriedades no nome (`Size=Small`), o rótulo de cada fileira é a propriedade, que é
        // palavra do ARTISTA e nunca passa por aqui.
        // **A PELE POR-WIDGET** (plano UI/UX W6.2) — a forma veste um widget do catálogo, e o
        // pintor REAL desenha no lugar dela.
        // ⚠️ Os nomes dos tipos são os do catálogo (`ph2d_editor_core::widget`) e passam por aqui
        // como qualquer outro rótulo: eles aparecem na tela, e a lei do repo não abre exceção para
        // substantivo próprio de design system.
        // **OS ESTADOS de UI** (plano UI/UX W7) — os quatro papéis e os três verbos.
        // ⚠️ Os papéis são NOMES DE PAPEL, não de estado livre: "Hover" descreve o que aconteceu
        // com o rato, e é isso que torna o gatilho derivável em vez de autorado.
        // O readout da pré-visualização: sem ele uma cena parada num hover parece o repouso, e a
        // gravação seguinte do Default o sobrescreve com a pose errada.
        // **O MODO DE PREVIEW** (W7r). O segundo rótulo diz como SAIR, e não é cortesia: um modo
        // que toma o rato e não anuncia a porta de saída é um modo em que o artista fica preso.
        // ⚠️ O rótulo diz o que ACONTECE, não o que a caixa é: *"Move All States"* descreve o
        // efeito do próximo arrasto, e é isso que o artista precisa de decidir antes de arrastar.
        // **O SELETOR DE CURVA** (W7). *"Curve"* e não *"Easing"*: o artista escolhe a FORMA do
        // movimento, e *easing* é o nome que a implementação lhe dá. Os rótulos dos chips não
        // estão aqui — vêm do `EasingFamily::label()`, porque são o vocabulário do catálogo e não
        // texto deste painel; uma segunda lista aqui divergiria do menu da timeline.
        // ⚠️ O rótulo do widget é o NOME da entidade (a Hierarquia), nunca um campo próprio — esta
        // linha é o que torna essa lei visível ao artista em vez de descoberta por acidente.
        // **O AUTO LAYOUT** (plano UI/UX W2, ADR-0153) — a moldura que empilha os filhos.
        // ⚠️ Os rótulos de direção incluem o "Off" porque *"esta moldura flui?"* e *"em que
        // direção?"* são a MESMA pergunta (o `display` do CSS) — ver `VECTOR_LAYOUT_DIR_OFF`.
        // A FONTE da largura de um traço de lápis (W1d). "Pen" e não "Pressure": o rótulo diz o
        // DISPOSITIVO, e é ele que hoje não existe nesta shell — o artista escolhe e não vê
        // diferença nenhuma, o que é a resposta honesta enquanto o caminho do tablet não chega.
        // **O catálogo de perfis de largura** (W2b) — os rótulos da tabela
        // `ph2d_stroke_width::PRESETS`, na ordem em que ela os lista. São VERBOS sobre a curva
        // ("afina", "engrossa"), não números: um nome só serve se descrever a forma, e a tabela
        // foi medida para que descreva (o doc dela traz o multiplicador em cinco pontos do arco).
        // **O Z-INDEX** — o lugar da forma na pilha dos IRMÃOS, maior = mais à frente
        // (a convenção do Godot/Unity). Readout: o numero e' derivado da arvore.
        // A seção do CONECTOR — só aparece com um conector na seleção. Os RÓTULOS dos três
        // campos (Route / Jetty / Spread) vêm do catálogo em `ph2d-tool-vector::connector`,
        // que é a fonte única deles (a mesma regra do catálogo de formas).

        // ── Physics world panel (ADR-0131 D8 / W2b) — the WORLD half of physics
        // authoring. The per-BODY half is the Inspector's "Physics Body" section.
        // Labels say what the number DOES — but ⚠️ that cuts both ways: calling
        // the UNIFORM damping "Air Drag" is exactly what made the first smoke
        // fail (Enio: "todos os objetos grandes e pequenos caem na mesma
        // velocidade"). A label has to promise what the model can deliver.
        // Wet Tuning side panel (doc 22) — labels are the model app's own.
        // ⛔⛔ **Este valor era "Wet Tuning" e a ABA dizia "Wet Paint"** — duas superfícies do MESMO
        // painel, no ecrã ao mesmo tempo, com nomes diferentes (medido 2026-09-17: cinco painéis
        // assim). A partir da migração do `Panel::TITLE` para `TextKey` há **uma** fonte: esta
        // chave é o que a aba lê E o que o cabeçalho do painel pinta. Ganhou a palavra da ABA,
        // porque é a que o menu *Window* também diz e há gate a atá-las
        // (`the_tab_and_the_menu_call_a_panel_the_same_thing`).
        "panel.wet_tuning.title" => "Wet Paint",
        "panel.wet_tuning.group.paint" => "Paint",
        "panel.wet_tuning.group.water" => "Water",
        "panel.wet_tuning.group.physics" => "Physics",
        "panel.wet_tuning.group.tools" => "Tools",
        "panel.wet_tuning.group.paper" => "Paper",
        "panel.wet_tuning.group.experimental" => "Experimental",
        "panel.wet_tuning.km_mixing" => "Pigment mixing (K-M)",
        "panel.wet_tuning.km_glaze" => "Glaze layering (K-M)",
        "panel.wet_tuning.note" => {
            "Kubelka-Munk subtractive color. Further gated extensions (diffusion, backrun, fingering, dry-brush, render extras) ship neutral; see the tuning registry's hidden group."
        }
        "panel.wet_tuning.knob.pigmentPerDab" => "Pigment per dab",
        "panel.wet_tuning.knob.paperGate" => "Paper gate",
        "panel.wet_tuning.knob.felt" => "Felt (pores)",
        "panel.wet_tuning.knob.bristleCount" => "Bristle count",
        "panel.wet_tuning.knob.drag" => "Drag",
        "panel.wet_tuning.knob.pickup" => "Pickup",
        "panel.wet_tuning.knob.intensity" => "Intensity",
        "panel.wet_tuning.knob.bristleStrength" => "Bristle strength",
        "panel.wet_tuning.knob.bristleSize" => "Bristle size",
        "panel.wet_tuning.knob.spacing" => "Spacing",
        "panel.wet_tuning.knob.tipClean" => "Tip clean",
        "panel.wet_tuning.knob.blendForce" => "Blend force",
        "panel.wet_tuning.knob.gateSaturation" => "Gate saturation",
        "panel.wet_tuning.knob.waterPerDab" => "Water per dab",
        "panel.wet_tuning.knob.waterCap" => "Water cap",
        "panel.wet_tuning.knob.evaporation" => "Evaporation",
        "panel.wet_tuning.knob.rewet" => "Re-wet",
        "panel.wet_tuning.knob.retention" => "Retention",
        "panel.wet_tuning.knob.edgeDarkening" => "Edge darkening",
        "panel.wet_tuning.knob.baseEvaporation" => "Base evaporation",
        "panel.wet_tuning.knob.leveling" => "Leveling",
        "panel.wet_tuning.knob.capillary" => "Capillary",
        "panel.wet_tuning.knob.brake" => "Brake",
        "panel.wet_tuning.knob.gravity" => "Gravity",
        "panel.wet_tuning.knob.levelClamp" => "Level clamp",
        "panel.wet_tuning.knob.viscosity" => "Viscosity",
        "panel.wet_tuning.knob.maxVelocity" => "Max velocity",
        "panel.wet_tuning.knob.projection" => "Projection",
        "panel.wet_tuning.knob.brakeReach" => "Brake reach",
        "panel.wet_tuning.knob.capillaryGate" => "Capillary gate",
        "panel.wet_tuning.knob.eraser" => "Eraser",
        "panel.wet_tuning.knob.dryer" => "Dryer",
        "panel.wet_tuning.knob.blow" => "Blow",
        "panel.wet_tuning.knob.smear" => "Smear",
        "panel.wet_tuning.knob.wetLift" => "Rewet lift",
        "panel.wet_tuning.knob.extStaining" => "Staining",
        "panel.wet_tuning.knob.paperContrast" => "Contrast",
        "panel.wet_tuning.knob.paperFibres" => "Fibres",
        "panel.wet_tuning.knob.paperGrooves" => "Grooves",
        "panel.wet_tuning.knob.visualGrain" => "Visual grain",
        "panel.wet_tuning.knob.emboss" => "Emboss",
        "panel.wet_tuning.knob.paperVisibility" => "Paper visibility",
        // **O painel de TOKENS** (plano UI/UX W6) — a tabela de cor do design system, autorável.
        // ⚠️ Os NOMES dos tokens (`bg-0`, `accent`, …) NÃO passam por aqui: eles são as chaves do
        // `tokens.json`, o endereço que o artista digita no picker de binding e que o arquivo
        // guarda — traduzi-los partiria o endereço.
        // ⛔ Era "Tokens" e a aba dizia "Design Tokens" — ver a nota do `panel.wet_tuning.title`.
        "panel.tokens.title" => "Design Tokens",
        // ⭐⭐ **OS QUATRO PAINÉIS QUE NÃO TINHAM CHAVE NENHUMA** (2026-09-17) — eles não pintam o
        // cabeçalho por esta tabela, logo nunca precisaram de uma; a migração do `Panel::TITLE`
        // para `TextKey` deu-lhes a primeira. ⚠️ Moram aqui e não numa tabela de família porque
        // nenhum deles TEM família: os OSSOS pedem emprestado o vocabulário do vetor
        // (`panel.vector.*`), e os outros três são bancadas.
        "panel.skeleton.title" => "Bones",
        "panel.authored.title" => "Authored UI",
        "panel.widget_gallery.title" => "Widget Gallery",
        "panel.widget_lab.title" => "Widget Lab",
        // A HIERARQUIA (2026-09-13) — o painel que o artista tem aberto o dia inteiro, e que não
        // tinha uma única chave. As duas frases do contador moram INTEIRAS aqui, com os marcadores
        // (`ph2d_i18n::tr_with`): colar o número no código fixaria a ordem das palavras.
        "panel.hierarchy.title" => "Hierarchy",
        "panel.hierarchy.add" => "Add",
        // o selo de TIPO de uma linha sem selo próprio — palavra gritada que a 1.ª régua não via
        "panel.hierarchy.badge.entity" => "ENT",
        "panel.hierarchy.search" => "Search\u{2026}",
        "panel.hierarchy.count.entities" => "{entities} entities",
        "panel.hierarchy.count.entities_components" => {
            "{entities} entities \u{00b7} {components} components"
        }
        // ⚠️ A FORMA da linha de cabeçalho (o travessão e a ordem das peças) estava no `format!`
        //    do pintor — só as duas palavras vinham da tabela, e a gramática ficava no fonte.
        //    ⛔ A `panel.tokens.authored` que morava aqui ficou ÓRFÃ com esta entrada e foi
        //    apagada: a palavra dela vive dentro do modelo.
        "panel.tokens.header" => "{tema}  \u{2014}  {n} authored",
        "panel.tokens.reset" => "Reset",
        "panel.tokens.reset_all" => "Reset This Mode",
        // O readout de CONTRASTE (plano UI/UX W4b). ⚠️ O nome do CRITÉRIO ("WCAG 2.2 AA 1.4.3")
        // não passa por aqui: ele é o endereço de uma norma, e traduzi-lo tornaria a coisa que o
        // artista precisa de PROCURAR impossível de procurar — a mesma lei que mantém as chaves
        // dos tokens fora desta tabela.
        "panel.tokens.contrast.title" => "Contrast below WCAG",
        // ⭐ A LINHA inteira é uma frase (2026-09-16): colar `on` entre dois pedaços de código
        //    fixava a ordem das palavras. O critério (`WCAG 2.2 AA 1.4.3`) entra como PEÇA — é o
        //    endereço de uma norma, não se traduz.
        "panel.tokens.contrast.line" => "{fg} on {bg} - {ratio}:1, need {min} ({criterion})",
        "panel.tokens.swatch" => "Design token colour",
        // A família NUMÉRICA (plano UI/UX W4c.1) — a escala que se mede em px.
        // ⚠️ O cabeçalho diz a UNIDADE, e é o que separa esta lista da de cima: as duas listam
        // "tokens", e sem a unidade um chip com `8` ao lado de uma swatch não diz de que grandeza
        // se está a falar. A unidade é a razão de as três escalas serem UMA família.
        "panel.tokens.numeric" => "Scale (px)",
        "panel.tokens.formula.hint" => "e.g. {spacing.md} * 2",
        // O INTEROP DTCG (plano UI/UX W9). ⚠️ **"DTCG" não é traduzido**: é o nome próprio do
        // formato W3C que o Tokens Studio / Style Dictionary / Penpot falam, e é a palavra que o
        // artista procura no menu da OUTRA ferramenta — a mesma lei que mantém "WCAG 2.2 AA" e as
        // chaves dos tokens fora desta tabela.
        //
        // ⚠️ E as reticências são ASCII (`...`), como as do `Import Font...` do painel de vetor —
        // este painel não tem outro botão que abra um diálogo com quem ser consistente, e a fonte
        // agrupada cobre o `\u{2026}` mas o gate de tofu não o vigia.
        "panel.tokens.dtcg.export" => "Export DTCG...",
        "panel.tokens.dtcg.import" => "Import DTCG...",
        // O painel da cena 3D (ADR-0150 W12). Os NOMES dos verbos e das curvas
        // NÃO estão aqui: eles vêm de `Verb::label()` / `Falloff::label()`, que
        // é a mesma porta que o log do teclado usa — duas tabelas de nomes para
        // a mesma lista divergiriam no dia em que um verbo fosse renomeado.
        "panel.physics.title" => "Physics",
        "panel.physics.section.world" => "World",
        "panel.physics.section.solver" => "Solver",
        // Two DIFFERENT models, and the section headers are what keeps them
        // apart: "Air Drag" scales with a body's cross-section and is resisted
        // by its mass (big things fall faster); "Damping" is a uniform velocity
        // decay that mass cannot enter (everything slows equally). Labelling
        // the uniform one "Air Drag" is what made the first smoke fail.
        "panel.physics.section.air" => "Air Drag",
        "panel.physics.section.damping" => "Damping",
        "panel.physics.air_drag" => "Density",
        "panel.physics.section.layers" => "Collision Layers",
        "panel.physics.section.sleep" => "Sleep",
        "panel.physics.section.debug" => "Debug",
        "panel.physics.gravity_x" => "Gravity X",
        "panel.physics.gravity_y" => "Gravity Y",
        "panel.physics.substeps" => "Sub-steps",
        "panel.physics.iterations" => "Iterations",
        "panel.physics.contact_hz" => "Contact Hz",
        "panel.physics.linear_damping" => "Linear",
        "panel.physics.angular_damping" => "Angular",
        // ⚠️ *"Enabled"*, e não *"Spin"*: o campo que este interruptor escreve é o
        // `sleep_angular_threshold`, mas a `rapier` 0.35 lê dele **o sinal**, não a magnitude —
        // `>= 0` = os corpos podem dormir, `< 0` = nunca dormem. O rótulo tem de prometer o que o
        // modelo entrega, e o que ele entrega é dormir, não rodar.
        "panel.physics.sleep_enabled" => "Enabled",
        "panel.physics.sleep_speed" => "Speed",
        "panel.physics.sleep_delay" => "Delay",
        // ── Interaction tool (W-Hand): what the POINTER does to a running scene.
        "panel.physics.section.interact" => "Interaction",
        "panel.physics.tool" => "Tool",
        "panel.physics.tool.hand" => "Hand",
        "panel.physics.tool.explode" => "Blast",
        "panel.physics.tool.attract" => "Pull",
        "panel.physics.hold" => "Hold",
        "panel.physics.hold.spring" => "Spring",
        "panel.physics.hold.rigid" => "Rigid",
        "panel.physics.hold.rope" => "Rope",
        "panel.physics.hold_stiffness" => "Stiffness",
        "panel.physics.hold_damping" => "Damping",
        "panel.physics.hold_slack" => "Slack",
        "panel.physics.blast_radius" => "Radius",
        "panel.physics.blast_force" => "Impulse",
        "panel.physics.pull_radius" => "Radius",
        "panel.physics.pull_force" => "Force",
        "panel.physics.ik_damping" => "Smoothing",
        "panel.physics.ik_angle" => "Tip Angle",
        "panel.physics.ik_angle.free" => "Free",
        "panel.physics.ik_angle.match" => "Match",
        "panel.physics.interact_hint" => "Play + drag on the canvas",
        // ── A seção JOINTS (W-JointTools) ──────────────────────────────────
        // ⚠️ As duas seções de interação existem porque as duas famílias querem
        // estados OPOSTOS do transporte: aquelas três empurram o solver e pedem
        // Play, estas cinco autoram a cena e pedem Pause. A dica de cada modo
        // diz qual — uma dica só mandaria metade dos artistas fazer exatamente
        // o que não funciona.
        "panel.physics.section.joint" => "Joints",
        "panel.physics.joint_tool" => "Drag",
        "panel.physics.joint_tool.body" => "Body",
        "panel.physics.joint_tool.rig" => "Rig",
        "panel.physics.joint_tool.links" => "Links",
        "panel.physics.joint_tool.ik" => "IK",
        "panel.physics.joint_tool.fk" => "FK",
        "panel.physics.joint_hint.body" => "Drag moves only the body you grab",
        "panel.physics.joint_hint.rig" => "Paused: drag carries the whole rig, anchors included",
        "panel.physics.joint_hint.links" => "Paused: drag carries the moving links; anchors stay",
        "panel.physics.joint_hint.ik" => "Paused: drag the tip and the chain bends behind it",
        "panel.physics.joint_hint.fk" => "Paused: drag a link and it swings about its joint",
        "panel.physics.joint_hint.alt" => "Alt while dragging always carries the whole rig",
        "panel.physics.show_colliders" => "Show Colliders",
        "panel.physics.reset_defaults" => "Reset to Defaults",
        "panel.physics.clear_run" => "Clear Recorded Run",
        "panel.physics.restore_run" => "Restore Discarded Run",
        // The world scale is `ProjectSettings::pixels_per_meter` — a PROJECT
        // setting. This panel shows it so the metre-valued knobs above can be
        // read in pixels, and deliberately does not own or duplicate it (D4).
        "panel.physics.scale" => "Scale",
        // ⭐ O leitor da escala do mundo é uma FRASE — ver a nota do `panel.model3d.footer`. ⚠️ O
        // `px/m` é um SÍMBOLO de unidade e fica dentro dela: ele é igual em toda língua, e parti-lo
        // num argumento daria a alguém a ideia de o traduzir.
        "panel.physics.scale_readout" => "{label}: {value} px/m",
        // ⚠️ O MODELO, e não só a palavra: o `": "` vivia no `format!` do pintor, e há
        //    línguas em que o dois-pontos leva espaço antes.
        "panel.physics.bodies_count" => "Bodies: {n}",
        // As oito operações do Pathfinder. As quatro primeiras eram literais no painel até a W5;
        // passam por aqui agora porque a fileira é UMA e metade dela em i18n seria o pior dos dois.
        // A BOOLEANA VIVA (plano UI/UX W1): o modo dos oito acima + o commit.
        // Stroke markers (arrowheads) — the two selectors in the STROKE section.
        // Only the ROW labels live here: the marker NAMES ("Arrow", "Diamond",
        // "Bar"…) come from `ph2d_vec_scene::Marker::label()`, which is their
        // single source — the same rule the shape catalogue follows.
        // ⚠️ **A família de chaves de um painel mora num IRMÃO** — `vector.rs` e `sculpt3d.rs`,
        // os dois cortados do mesmo arquivo pelo mesmo teto de LOC, em linhas paralelas. O corte
        // é de ASSUNTO: aqui ficam as chaves do APP, lá as de UM painel.
        //
        // ⚠️ **Os irmãos são consultados em CADEIA, e o encaminhamento vem ANTES do vazamento** —
        // senão toda chave do painel que ficasse por último cairia no `leak_key` e ele pintaria os
        // próprios identificadores. Um irmão novo entra nesta cadeia, nunca num segundo `match`.
        //
        // Pass-through para chave desconhecida, para a entrada que falta ficar visível na UI
        // (o identificador cru é feio de propósito).
        k => vector::tr(k)
            .or_else(|| timeline::tr(k))
            .or_else(|| sculpt3d::tr(k))
            .or_else(|| model3d::tr(k))
            .or_else(|| model3d_inert::tr(k))
            .or_else(|| model3d_bloom::tr(k))
            .or_else(|| tags::tr(k))
            .or_else(|| factory::tr(k))
            .or_else(|| topdown::tr(k))
            .or_else(|| component_catalog::tr(k))
            .or_else(|| paint_engines::tr(k))
            .or_else(|| sculpt_engine::tr(k))
            .or_else(|| ecs_scene::tr(k))
            .or_else(|| tags_engine::tr(k))
            .or_else(|| tool_engines::tr(k))
            .or_else(|| tokens::tr(k))
            .or_else(|| node_catalog::tr(k))
            .or_else(|| node_groups::tr(k))
            .or_else(|| node_params::tr(k))
            .or_else(|| node_options::tr(k))
            .or_else(|| chrome::tr(k))
            .or_else(|| input_map::tr(k))
            .or_else(|| painter_layers::tr(k))
            .or_else(|| inspector::tr(k))
            .or_else(|| inspector_game::tr(k))
            .or_else(|| inspector_player::tr(k))
            .or_else(|| audio::tr(k))
            .or_else(|| grid_snap::tr(k))
            .or_else(|| flip::tr(k))
            .or_else(|| image_tools::tr(k))
            .or_else(|| motion_panels::tr(k))
            .or_else(|| asset_browser::tr(k))
            .or_else(|| chrome_menus::tr(k))
            .or_else(|| chrome_rail::tr(k))
            .or_else(|| chrome_panes::tr(k))
            .or_else(|| audio_fx::tr(k))
            .or_else(|| audio_engines::tr(k))
            .or_else(|| blend_modes::tr(k))
            .or_else(|| quatro_motores::tr(k))
            .or_else(|| vector_engine::tr(k))
            .or_else(|| shell::tr(k))
            .or_else(|| shell_media::tr(k))
            .or_else(|| app_components::tr(k))
            .or_else(|| app_field3d::tr(k))
            .or_else(|| app_flip::tr(k))
            .or_else(|| app_painter::tr(k))
            .or_else(|| app_physics::tr(k))
            .or_else(|| app_sculpt3d::tr(k))
            .or_else(|| app_motion::tr(k))
            .or_else(|| app_vec::tr(k))
            .unwrap_or_else(|| leak_key(k)),
    }
}

/// ⭐⭐ **Uma CHAVE guardada numa tabela `const`** — o rótulo de um segmentado, a linha de um card.
///
/// O [`tr`] não é `const fn`, logo uma tabela `const [&str; N]` de rótulos não o pode chamar, e a
/// tabela passa a guardar CHAVES. ⛔ Guardadas como `&str`, uma chave e um texto são o MESMO tipo: o
/// consumidor que se esquece de traduzir compila, passa em todo teste que não leia o pixel, e pinta
/// `panel.inspector.joint.pin` no ecrã. Com `TextKey` esse esquecimento é **erro de compilação** — o
/// `seg_row(…, &[&str])` não aceita `[TextKey; N]` até alguém escrever `.map(TextKey::tr)`.
///
/// ⚠️ A tabela continua `const` de propósito: há contagens derivadas dela em tempo de compilação
/// (`PLAYER_ROW_COUNT` do Inspector), que uma função no lugar da tabela partiria.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextKey(&'static str);

impl TextKey {
    /// A chave, tal como está na tabela (`"panel.inspector.joint.pin"`).
    #[must_use]
    pub const fn new(key: &'static str) -> Self {
        Self(key)
    }

    /// O texto que a chave diz — o mesmo [`tr`], com a chave desconhecida a pintar-se crua.
    #[must_use]
    pub fn tr(self) -> &'static str {
        tr(self.0)
    }

    /// ⭐ **A chave CRUA, para quem faz censo dela** — nunca para pintar.
    ///
    /// ⚠️ Ela existe porque um gate precisa de perguntar *«esta chave EXISTE na tabela?»*, e a
    /// resposta é `tr(k) != k` (a chave desconhecida volta crua, com `leak_key`). ⛔ O tipo continua
    /// a ser a cerca: quem quer TEXTO chama [`Self::tr`], e o nome deste método diz em voz alta que
    /// o que sai daqui é um identificador.
    #[must_use]
    pub const fn key(self) -> &'static str {
        self.0
    }
}

/// Stub for the unknown-key path: leak the input into a `&'static`
/// so the return type stays uniform. Only fires on developer typos
/// (unknown keys at runtime), so the small per-typo leak is fine.
/// The Fluent migration replaces this with a `String` return.
fn leak_key(key: &str) -> &'static str {
    Box::leak(key.to_string().into_boxed_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_keys_round_trip_to_english() {
        // Image-tool labels were abbreviated 2026-05-25 to fit the
        // 44-px chip column; tooltips keep the long English form.
        assert_eq!(tr("tool.trim_transparency.label"), "TRIM");
        assert_eq!(tr("tool.trim_transparency.tooltip"), "Trim Transparency");
        assert_eq!(tr("tool.make_square.label"), "SQUAR");
        assert_eq!(tr("edit.undo.label"), "Undo");
        // Timeline panel chrome (W2.E9).
        assert_eq!(tr("panel.timeline.title"), "Timeline");
        assert_eq!(tr("panel.timeline.loop"), "Loop");
        assert_eq!(tr("panel.timeline.prop.translate_x"), "Translate X");
        // Vector panel chrome — section headers + the shape-catalogue words.
        assert_eq!(tr("panel.vector.section.tool"), "Tool");
        assert_eq!(tr("panel.vector.mode.shape"), "Shape");
        assert_eq!(tr("panel.vector.category"), "Category");
        assert_eq!(tr("panel.vector.shape.no_params"), "No parameters");
        assert_eq!(tr("panel.vector.group.iso"), "3D");
    }

    #[test]
    fn a_text_key_says_what_its_key_says() {
        const TITLE: TextKey = TextKey::new("panel.timeline.title");
        assert_eq!(TITLE.tr(), tr("panel.timeline.title"));
        assert_eq!(TITLE.tr(), "Timeline");
        assert_eq!(
            TextKey::new("tool.nonexistent.bar").tr(),
            "tool.nonexistent.bar"
        );
    }

    #[test]
    fn unknown_key_returns_the_key_itself() {
        let leaked = tr("tool.nonexistent.foo");
        assert_eq!(leaked, "tool.nonexistent.foo");
    }
}
