//! Architecture gate — **o CONTROLO SEM CONSUMIDOR**: um `ids::X` que o artista
//! vê e clica, e cujo valor não chega a ninguém. É a terceira pergunta da
//! costura, e a que nenhum instrumento deste repo fazia.
//!
//! ## As duas metades que já existiam, e o buraco entre elas
//!
//! 1. `architecture_panel_wiring_parity` mede **focalizabilidade**: pintado +
//!    hit-indexado mas sem `InteractiveState` ⇒ `is_focusable()==false` ⇒ morto
//!    sob o dedo. O cabeçalho dele **declara** que a metade complementar — *"o
//!    evento é largado no `_ => false`"* — fica por fazer.
//! 2. Os `seam_*.rs` provam que um clique **chega à ferramenta**. Nunca que a
//!    escrita da ferramenta chega a um EFEITO.
//!
//! Entre as duas cabe o defeito real: o controlo é focalizável, o clique chega,
//! e **ninguém pergunta por aquele id**. Foi assim que a caça de 2026-08-30
//! achou 34 controlos mortos em ~504.
//!
//! ## A lei que este gate impõe
//!
//! *Um id que o produto USA tem de chegar a um **término**.* Término é:
//!
//! | sigla | o que é | porque conta |
//! |---|---|---|
//! | `CMP` | `id == ids::X` / `.contains(&ids::X)` | alguém pergunta pelo id |
//! | `ARM` | `ids::X => ..` (braço de `match`) | idem, como padrão |
//! | `MAP` | `(ids::X, carga)` numa tabela | o id é a CHAVE de um mapa |
//! | `SEAM` | dentro de um `panel_seam! { .. }` | a macro emite populate **e** braço |
//! | `SELF` | registado com um `InteractiveState` que o chrome despacha por KIND (`BlenderHit`, `BlenderPicker`, `TimelineSurface`, …) ou marcado por `mark_collapsible_section` | o despacho genérico responde sem ver o id |
//! | `READ` | `store.slider(ids::X)` **fora** do ficheiro que o pinta | alguém lê o valor |
//! | `OTHER` | qualquer outro uso fora de paint/populate/lista | conservador de propósito |
//!
//! ⛔ **Encaminhar NÃO é responder.** Um braço cujo corpo devolve o mesmo id a
//! outro canal (`bus.push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)))`)
//! é um SALTO, não um término: quem tem de responder é o outro lado. É essa
//! distinção — e só ela — que separa o `FLIP_STRIP_CLOSE` (na lista `BUTTONS`,
//! encaminhado, e sem braço em nenhum dos três drenos) do `FLIP_PREV_DRAWING`
//! (na MESMA lista, e com `*id == ids::FLIP_PREV_DRAWING` no `flip_strip.rs`).
//!
//! ## As duas famílias que a régua junta antes de julgar
//!
//! - **Tabela**: um id membro de uma `const T: [NodeId; N]` herda os usos de
//!   `T` (menos a declaração e a própria lista) — é assim que `VECTOR_SECTIONS`
//!   mantém 40 cabeçalhos vivos por UM `for` no `populate`.
//! - **Par**: dois ids no MESMO grupo de parênteses são UM controlo (slider +
//!   chip: `slider_chip_int(store, SLIDER, CHIP, ..)`), e partilham a sorte.
//!   ⚠️ Só em `(`, nunca em `[` — senão a lista `BUTTONS` daria vida ao X morto
//!   por vizinhança.
//!
//! ## Quem fica FORA da população (não é controlo)
//!
//! - **Decoração / grupo**: `Card::new(id)`, `SegmentedAdaptive::new(id, ..)`,
//!   `seg_row(.., id, ..)` — o clique cai na OPÇÃO, nunca no grupo.
//! - **Corpo de painel**: `set_panel_rect(id, ..)`, `parent: ids::X`, e os
//!   membros de `PANEL_Z_ORDER_FALLBACK` (que é o registo de painéis do repo).
//!
//! ## ⛔ O que este gate NÃO vê (lista honesta — vale mais que um verde falso)
//!
//! - **Ids DINÂMICOS** (`ids::wet_tuning_slider_id(key)`, `flip_cell_id(i)`,
//!   `vector_token_option_id(..)`): a população é só `pub const X: NodeId`.
//!   Painéis inteiros guiados por tabela (o `ph2d-panel-wet-tuning`, auditado
//!   42/42 limpo) ficam quase todos fora — o que este gate diz sobre eles é
//!   apenas o que diz sobre os poucos consts que eles têm.
//! - **Efeito confinado ao PINTOR**: um filtro/dobra/rolagem que o próprio
//!   `paint` lê e aplica (o `HIER_SEARCH`) lê como morto e está vivo. Está na
//!   catraca, com o motivo.
//! - **O término que é uma AUSÊNCIA.** Um id registado no `HitIndex` só para o canvas **não**
//!   receber o clique (o fundo de um cartão flutuante) é consumido por `hit(..).is_none()`, que
//!   não é um uso do id em lado nenhum. Ele lê como morto e está vivo — o `INPUT_MAP_SURFACE` da
//!   catraca é essa espécie, medida em 2026-08-30. ⛔ Ensinar a régua a aceitar *«registado num
//!   `HitIndex` e mais nada»* **branquearia os seis cabeçalhos mortos** que têm a mesma forma: o
//!   que os separa é a INTENÇÃO, e nenhuma varredura de fonte a lê.
//! - **Braço que existe e não faz nada** (`X => {}`): mede alcance, não efeito.
//! - **A cadeia de saltos**: o gate exige que o id seja perguntado em ALGUM
//!   sítio; não segue o `PanelEvent` até ao `FlipDoc`.
//! - **Tabelas com o MESMO nome em crates diferentes** (`SECTIONS` existe na
//!   física, no sculpt3d e no wet-tuning) são fundidas: os membros de uma herdam
//!   os usos da outra. O erro que isso produz é sempre um falso NEGATIVO — um
//!   morto que passa —, que é a direcção segura para um gate que shipa verde.
//! - **Um `_ => false` que consome sem responder**: se o painel tiver um braço
//!   que compara o id e não faz nada, lê como vivo. A pergunta é *alcance*.
//! - **Ids de outros módulos de `ids`** que não `ph2d-editor-core/src/ids/**`.
//!
//! ⚠️ **Controlo POSITIVO dentro do instrumento.** A catraca abaixo tem de
//! continuar a ser DETECTADA (secção *stale*), e os ids de `KNOWN_LIVE`
//! — verificados vivos à mão — têm de continuar a NÃO ser sinalizados. Um
//! instrumento cujo vermelho é garantido pela forma dos dados não mede nada.
//!
//! Dep-free (std only), como os outros gates de arquitectura.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// **Catraca de dívida conhecida — ela só ENCOLHE.** `(id, motivo)`.
/// Uma linha que já não é detectada é uma dívida paga que ninguém apagou.
const NO_CONSUMER_PENDING: &[(&str, &str)] = &[
    // ═══ DÍVIDAS PAGAS em 2026-08-30, na mesma jornada que as descobriu ═══
    //
    // Sete entradas saíram desta lista **porque os controlos foram LIGADOS**, cada um com o seu
    // gate e a sua prova de mutação. Ficam nomeadas aqui, sem célula, para que a próxima leitura
    // não as procure e para que o padrão delas se veja de uma vez:
    //
    //   `VECTOR_SYM_SEGMENTS` + `_NUM` — o slider Radial>Segments passou a ter braço de evento e
    //       a contagem chega à roseta (`seam_symmetry_segments.rs`, 4 mutações).
    //   `PHYSICS_SEC_LAYERS`  — o cabeçalho dobra; a lista de cabeçalhos passou a ser UMA
    //       (`every_painted_section_header_folds`, censo das duas lentes).
    //   `SCULPT3D_SEC_BAKE`   — irmão exacto do anterior, mesma cura, mesmo censo.
    //   `FLIP_STRIP_CLOSE`    — o X da tira FECHA a tira, pela mesma lei que o X da timeline já
    //       escrevia (`close_button_seam.rs`). Ele saiu da lista `BUTTONS`: encaminhar um fecho
    //       pelo barramento pedia um braço no shell que nunca existiu.
    //   `INSP_PLAYER_ADD`     — registo ÓRFÃO de um botão que saiu do produto na F3; apagado.
    //   `WET_TUNING_SCROLL`   — id ÓRFÃO (a barra real é a `WET_TUNING_SCROLLBAR_ID`); apagado.
    //
    // ⚠️ **Os dois últimos são de OUTRA espécie, e a distinção é a cura:** um id declarado que
    // ninguém pinta nem regista é **lixo** — cura-se apagando. Um id pintado e registado cujo
    // valor não chega a consumidor nenhum é um **knob morto** — cura-se ligando o braço. Esta
    // régua vê as duas iguais, e foi por isso que os órfãos apareceram numa caça a mortos.
    //
    // ═══ O QUE FICA ═══
    (
        "MENUBAR_BACKDROP",
        "TERMINA POR AUSENCIA, e essa e' a funcao dele: e' o fundo da barra de menus, e o efeito \
         de o registar no `HitIndex` e' BLOQUEAR — fazer o `chrome_hit::pointer_over_chrome` \
         responder `true` para os 87% da faixa pintada que nao sao titulo. Sem ele, um pen-down \
         entre dois titulos deposita tinta na arte escondida por baixo da barra (medido pela \
         auditoria de 2026-08-30). ⛔ Nenhuma varredura de terminos POSITIVOS o pode ver, e \
         ensinar esta regua a aceitar o padrao branquearia os cabecalhos de seccao genuinamente \
         mortos, que tem a mesma forma (CLAUDE.md §5.0). ⚠️ O irmao `RAIL_BACKDROP` escapa a esta \
         lista so' porque o `left_rail::apply_event` lhe imprime o nome — a mesma especie com \
         disfarce.",
    ),
    (
        "PAINTER_BRUSH_STROKE_SAVE_OBJECT",
        "MORTO POR DECISAO, declarado no fonte: `paint_stroke.rs` diz *\"clicking it is a \
         deliberate no-op (no route in the tool)\"* ate' o formato de objecto existir. Fica aqui \
         para que o dia em que alguem o ligar apague esta linha.",
    ),
    // ⚠️⚠️ **`INPUT_MAP_SURFACE` saiu daqui em 2026-08-30, e a história vale mais que a linha.**
    // Ele entrou como MORTO, foi RECLASSIFICADO como *vivo por um término que esta régua não tem*
    // — o término dele é a **AUSÊNCIA**: todo caminho de canvas do shell só corre com o índice de
    // hit VAZIO sob o cursor, então **estar registado É a resposta** (medido: apagando o
    // `register`, **1189 de 1600** pontos dentro do cartão caem no canvas por baixo) — e saiu de
    // vez quando a superfície ganhou consumidor NOMEADO (`chrome_hit.rs::chrome_claims`), que é
    // um término positivo e que a régua passou a ver sozinha.
    //
    // ⛔ **A lição fica:** a régua só reconhece términos POSITIVOS (`id == X`, braço de `match`,
    // chave de tabela). Um `HitIndex::register` cujo efeito é *bloquear* lê exactamente como um
    // *registado e esquecido* — e ensinar a régua a aceitar o padrão branquearia os cabeçalhos de
    // secção genuinamente mortos, que têm a mesma forma. Foi por isso que ele ficou na catraca em
    // vez de passar a um allowlist enquanto a distinção não existiu.
    // ⚠️ VIVO, mas invisível a ESTA régua — o ponto cego está declarado no
    // cabeçalho (§ *efeito confinado ao pintor*). Fica na catraca em vez de num
    // allowlist separado para que uma segunda ocorrência da mesma forma seja
    // lida como a familia que e', e nao como caso isolado.
    (
        "HIER_SEARCH",
        "VIVO mas invisivel: a caixa de busca da Hierarquia. O proprio `paint.rs` le o texto \
         (`store.get`) e filtra a arvore com `compute_match_filter` — o efeito nunca sai do \
         pintor, e o termino desta regua e' sempre FORA dele.",
    ),
];

/// **Controlo NEGATIVO** — ids verificados VIVOS à mão em 2026-08-30, cada um
/// por uma porta diferente. Se o gate passar a sinalizar um deles, ele ficou
/// grosseiro e a régua tem de ser lida antes do código.
const KNOWN_LIVE: &[(&str, &str)] = &[
    // `ph2d-panel-wet-tuning` — auditado 42/42 limpo. Estes sao os consts dele.
    ("WET_TUNING_PAPER_EYE", "CMP no event.rs do painel"),
    ("WET_TUNING_KM_MIXING", "CMP no event.rs do painel"),
    ("WET_TUNING_KM_GLAZE", "CMP no event.rs do painel"),
    (
        "WET_TUNING_CLOSE",
        "CMP no event.rs — e o braco encaminha OUTRO id, o que nao e' um salto",
    ),
    (
        "WET_TUNING_DRAG_HANDLE",
        "SELF: registado como InteractiveState::BlenderHit",
    ),
    // As outras portas, uma por familia.
    (
        "FLIP_PREV_DRAWING",
        "na MESMA lista BUTTONS do FLIP_STRIP_CLOSE, e com braco em flip_strip.rs",
    ),
    ("PAD_CANCEL", "SEAM: dentro de um panel_seam! {}"),
    (
        "PHYSICS_SEC_DEBUG",
        "CMP no event.rs, pintado em `src/paint/body.rs` — a regua tem de ler o DIRECTORIO",
    ),
    (
        "SCULPT3D_SEC_BRUSH",
        "tabela de STRUCTS: membro de `rows::SECTIONS`, varrida com `.any(|s| s.id == id)`",
    ),
    (
        "VECTOR_SECTION_STROKE",
        "tabela: membro de VECTOR_SECTIONS, marcada colapsavel num for",
    ),
    (
        "PAINTER_SEL_OP_ADD",
        "tabela PAINTER_SEL_OP_IDS + .contains no selection_overlay",
    ),
    (
        "PAINTER_BRUSH_SIZE_CHIP",
        "par: (SLIDER, CHIP, STEP) no mesmo grupo de parênteses",
    ),
    ("INSP_DRAG_HANDLE", "SELF: BlenderHit no pre_populate"),
];

/// A **metade justa**: a sonda tem de VER o que já existe. Baseline medido em
/// 2026-08-30: 2074 ids declarados, 1750 na população, 76 tabelas de `NodeId`.
const MIN_IDS: usize = 1900;
const MIN_POPULATION: usize = 1500;
const MIN_TABLES: usize = 60;

/// O registo de painéis do repo — os membros são CORPOS de painel, não
/// controlos. Derivado (a lista vive no produto), não copiado.
const PANEL_REGISTRY_TABLE: &str = "PANEL_Z_ORDER_FALLBACK";

const SELF_KINDS: &[&str] = &[
    "BlenderHit",
    "BlenderPicker",
    "TimelineSurface",
    "GraphSurface",
    "FlipStripSurface",
    "CurvePoint",
    "Scrollbar",
];
const SELF_CALLS: &[&str] = &[
    "mark_collapsible_section",
    "mark_collapsible",
    "register_picker_swatch",
];
const DECOR_CALLS: &[&str] = &[
    "Card::new",
    "SegmentedAdaptive::new",
    "Segmented::new",
    "RadioGroup::new",
    "Tabs::new",
];
const GROUP_CALLS: &[&str] = &["seg_group", "seg_row", "segmented", "segmented_row"];
const PANEL_CALLS: &[&str] = &[
    "set_panel_rect",
    "clear_panel_rect",
    "panel_rect",
    "panel_resize_delta",
    "blender_picker_offset",
    "set_panel_visible",
    "is_panel_visible",
];
const READ_CALLS: &[&str] = &[
    "slider",
    "toggle",
    "checkbox",
    "text",
    "number_value",
    "get",
    "get_mut",
    "is_collapsed",
    "dropdown",
    "selected_index",
    "button_visual",
    "hover_live",
];
const LINK_CALLS: &[&str] = &[
    "link_slider_number",
    "link_slider_number_mapped",
    "link_slider_number_mapped_integer",
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Decl,
    Use,
    TblDecl,
    Seam,
    Cmp,
    Arm,
    Fwd,
    Map,
    Self_,
    Decor,
    Panel,
    Link,
    Read,
    ReadPaint,
    Paint,
    Pop,
    List,
    Reg,
    Other,
}

impl Kind {
    /// Um término responde à pergunta *"quem lê este controlo?"*.
    fn is_terminus(self) -> bool {
        matches!(
            self,
            Kind::Cmp | Kind::Arm | Kind::Map | Kind::Read | Kind::Self_ | Kind::Other | Kind::Seam
        )
    }
    /// O produto USA o id (por oposição a apenas declarar/importar).
    fn is_wired(self) -> bool {
        !matches!(self, Kind::Decl | Kind::Use | Kind::TblDecl)
    }
}

struct Occ {
    kind: Kind,
    file: usize,
    line: usize,
    text: String,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn is_test_path(rel: &str) -> bool {
    let f = rel.rsplit('/').next().unwrap_or("");
    rel.contains("/tests/")
        || rel.contains("/benches/")
        || rel.contains("/examples/")
        || f == "tests.rs"
        || f.ends_with("_tests.rs")
}

fn is_ids_dir(rel: &str) -> bool {
    rel.contains("/ph2d-editor-core/src/ids/")
}

fn stem(rel: &str) -> &str {
    rel.rsplit('/')
        .next()
        .unwrap_or("")
        .strip_suffix(".rs")
        .unwrap_or("")
}

/// ⚠️ **A pergunta é sobre o CAMINHO, não sobre o nome do ficheiro.** O painel da
/// física pinta em `src/paint/body.rs`: um teste que só olhasse o `stem` leria
/// aquele uso como resposta em vez de pintura, e um cabeçalho morto lá dentro
/// passaria. (Foi exactamente o que a prova de mutação apanhou.)
fn is_paint_file(rel: &str) -> bool {
    let b = stem(rel);
    if b.starts_with("paint") || b.contains("_paint") || b.ends_with("_overlay") || b == "seam" {
        return true;
    }
    rel.split('/')
        .any(|c| c == "paint" || c.starts_with("paint_"))
}

fn is_pop_file(rel: &str) -> bool {
    let b = stem(rel);
    if b.starts_with("populate") || b.contains("_populate") || b.starts_with("toolbar_plan") {
        return true;
    }
    rel.split('/').any(|c| c == "populate")
}

fn collect_rs(dir: &Path, root: &Path, out: &mut Vec<(String, String)>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = rd.flatten().map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        let name = p
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        if p.is_dir() {
            if name != "target" && name != ".git" {
                collect_rs(&p, root, out);
            }
        } else if name.ends_with(".rs") {
            let rel = p
                .strip_prefix(root)
                .unwrap_or(&p)
                .to_string_lossy()
                .replace('\\', "/");
            if is_test_path(&rel) {
                continue;
            }
            if let Ok(s) = std::fs::read_to_string(&p) {
                out.push((rel, s));
            }
        }
    }
}

fn close_of(b: &[u8], open: usize) -> usize {
    let (o, c) = match b[open] {
        b'(' => (b'(', b')'),
        b'[' => (b'[', b']'),
        _ => (b'{', b'}'),
    };
    let mut depth = 0i32;
    let mut i = open;
    while i < b.len() {
        match b[i] {
            b'"' => {
                i += 1;
                while i < b.len() {
                    if b[i] == b'\\' {
                        i += 2;
                        continue;
                    }
                    if b[i] == b'"' {
                        break;
                    }
                    i += 1;
                }
            }
            ch if ch == o => depth += 1,
            ch if ch == c => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
            }
            _ => {}
        }
        i += 1;
    }
    b.len()
}

fn is_ident_byte(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// O caminho de chamada imediatamente à esquerda de um `(` (`Card::new`,
/// `hit_index.register` devolve `register`), ou vazio para um tuplo.
fn callee_before(b: &[u8], paren: usize) -> String {
    let mut k = paren;
    while k > 0 && (b[k - 1] as char).is_whitespace() {
        k -= 1;
    }
    let end = k;
    while k > 0 {
        let prev = b[k - 1];
        if is_ident_byte(prev) {
            k -= 1;
        } else if prev == b':' && k >= 2 && b[k - 2] == b':' {
            k -= 2;
        } else {
            break;
        }
    }
    String::from_utf8_lossy(&b[k..end]).to_string()
}

fn last_seg(path: &str) -> &str {
    path.rsplit("::").next().unwrap_or(path)
}

struct Frame {
    bracket: u8,
    callee: String,
    open: usize,
}

struct Hit {
    name: String,
    start: usize,
    end: usize,
    line: usize,
    bracket: u8,
    callee: String,
    open: usize,
}

/// Uma passagem só por ficheiro: salta comentários/strings/chars, mantém a pilha
/// de delimitadores, e devolve cada identificador MAIÚSCULO com o contexto.
fn scan_file(src: &str) -> Vec<Hit> {
    let b = src.as_bytes();
    let mut stack: Vec<Frame> = Vec::new();
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut line = 1usize;
    while i < b.len() {
        let c = b[i];
        match c {
            b'\n' => {
                line += 1;
                i += 1;
            }
            b'/' if i + 1 < b.len() && b[i + 1] == b'/' => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < b.len() && b[i + 1] == b'*' => {
                i += 2;
                while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                    if b[i] == b'\n' {
                        line += 1;
                    }
                    i += 1;
                }
                i = (i + 2).min(b.len());
            }
            b'"' => {
                i += 1;
                while i < b.len() {
                    if b[i] == b'\\' {
                        i += 2;
                        continue;
                    }
                    if b[i] == b'"' {
                        break;
                    }
                    if b[i] == b'\n' {
                        line += 1;
                    }
                    i += 1;
                }
                i += 1;
            }
            b'\'' => {
                // Literal de carácter vs. lifetime.
                if i + 2 < b.len() && b[i + 1] == b'\\' {
                    i += 2;
                    while i < b.len() && b[i] != b'\'' {
                        i += 1;
                    }
                    i += 1;
                } else if i + 2 < b.len() && b[i + 2] == b'\'' {
                    i += 3;
                } else {
                    i += 1;
                }
            }
            b'(' | b'[' | b'{' => {
                let callee = if c == b'(' {
                    callee_before(b, i)
                } else {
                    String::new()
                };
                stack.push(Frame {
                    bracket: c,
                    callee,
                    open: i,
                });
                i += 1;
            }
            b')' | b']' | b'}' => {
                stack.pop();
                i += 1;
            }
            _ if c.is_ascii_uppercase() => {
                let start = i;
                while i < b.len() && is_ident_byte(b[i]) {
                    i += 1;
                }
                // Fronteira de palavra à esquerda.
                if start > 0 && is_ident_byte(b[start - 1]) {
                    continue;
                }
                let name = String::from_utf8_lossy(&b[start..i]).to_string();
                if name.len() < 3
                    || !name
                        .chars()
                        .all(|ch| ch.is_ascii_uppercase() || ch == '_' || ch.is_ascii_digit())
                {
                    continue;
                }
                let top = stack.last();
                out.push(Hit {
                    name,
                    start,
                    end: i,
                    line,
                    bracket: top.map_or(0, |f| f.bracket),
                    callee: top.map(|f| f.callee.clone()).unwrap_or_default(),
                    open: top.map_or(usize::MAX, |f| f.open),
                });
            }
            _ if is_ident_byte(c) => {
                while i < b.len() && is_ident_byte(b[i]) {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }
    out
}

/// As tabelas `const|static NAME: .. = &[ .. ]` que contêm ids, e quais.
///
/// ⚠️ **O elemento NÃO precisa de ser um `NodeId`.** Metade das secções deste
/// repo vive numa tabela de STRUCTS (`static SECTIONS: &[Section]`, cada uma com
/// um campo `id`) que o `event.rs` percorre com `.iter().any(|s| s.id == id)` —
/// e uma leitura que só aceitasse `[NodeId]` daria seis cabeçalhos vivos por
/// mortos. Exigir apenas *"um array com ids dentro"* é o que os une.
fn node_id_tables(src: &str, ids: &BTreeSet<String>) -> BTreeMap<String, BTreeSet<String>> {
    let b = src.as_bytes();
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for kw in ["const ", "static "] {
        let mut from = 0usize;
        while let Some(rel) = src[from..].find(kw) {
            let at = from + rel + kw.len();
            from = at;
            let name: String = src[at..]
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if name.is_empty() || !name.starts_with(|c: char| c.is_ascii_uppercase()) {
                continue;
            }
            let rest = &src[at + name.len()..];
            let Some(head) = rest.get(..200.min(rest.len())) else {
                continue;
            };
            let Some(eq) = head.find('=') else { continue };
            let decl = &head[..eq];
            if !decl.contains('[') {
                continue;
            }
            let tail = &head[eq + 1..];
            let Some(brk) = tail.find('[') else { continue };
            if tail[..brk].chars().any(|c| !c.is_whitespace() && c != '&') {
                continue;
            }
            let open = at + name.len() + eq + 1 + brk;
            let end = close_of(b, open);
            let body = &src[open + 1..end.min(src.len())];
            let mut members = BTreeSet::new();
            for h in scan_file(body) {
                if ids.contains(&h.name) {
                    members.insert(h.name);
                }
            }
            out.entry(name).or_default().extend(members);
        }
    }
    out
}

/// Os intervalos de byte de cada `panel_seam! { .. }`.
fn seam_ranges(src: &str) -> Vec<(usize, usize)> {
    let b = src.as_bytes();
    let mut out = Vec::new();
    let mut from = 0usize;
    while let Some(rel) = src[from..].find("panel_seam!") {
        let at = from + rel;
        from = at + "panel_seam!".len();
        let Some(brace_rel) = src[from..].find('{') else {
            break;
        };
        let open = from + brace_rel;
        out.push((at, close_of(b, open)));
    }
    out
}

fn line_text(src: &str, off: usize) -> &str {
    let start = src[..off].rfind('\n').map_or(0, |i| i + 1);
    let end = src[off..].find('\n').map_or(src.len(), |i| off + i);
    &src[start..end]
}

/// O texto está dentro de um item `use ...;`?
fn in_use_item(src: &str, off: usize) -> bool {
    let from = floor_boundary(src, off.saturating_sub(2000));
    let seg = src.get(from..off).unwrap_or("");
    let semi = seg.rfind(';').map_or(-1i64, |i| i as i64);
    let usei = seg.rfind("use ").map_or(-1i64, |i| i as i64);
    usei > semi
}

fn floor_boundary(src: &str, mut i: usize) -> usize {
    while i > 0 && !src.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// A linha compara / faz padrão sobre `name`?
fn compares(line: &str, name: &str) -> bool {
    let mut from = 0usize;
    while let Some(rel) = line[from..].find(name) {
        let at = from + rel;
        from = at + name.len();
        let bytes = line.as_bytes();
        if at > 0 && is_ident_byte(bytes[at - 1]) {
            continue;
        }
        let after = line[at + name.len()..].trim_start();
        if after.starts_with("==")
            || after.starts_with("!=")
            || after.starts_with("=>")
            || after.starts_with('|')
            || after.starts_with(".contains")
        {
            return true;
        }
        // À esquerda, saltando um caminho (`ids::`) e os sinais `&`/`*`.
        let mut k = at;
        while k > 0 {
            let prev = line.as_bytes()[k - 1];
            if is_ident_byte(prev) || prev == b'&' || prev == b'*' {
                k -= 1;
            } else if prev == b':' && k >= 2 && line.as_bytes()[k - 2] == b':' {
                k -= 2;
            } else {
                break;
            }
        }
        let before = line[..k].trim_end();
        if before.ends_with("==") || before.ends_with("!=") || before.ends_with("contains(") {
            return true;
        }
    }
    false
}

/// O corpo do braço que começa em `off` (o `{ .. }` depois do `=>`, ou a linha).
fn arm_body(src: &str, off: usize) -> &str {
    let b = src.as_bytes();
    let window_end = (off + 400).min(src.len());
    let window = src.get(off..floor_boundary(src, window_end)).unwrap_or("");
    let Some(arrow) = window.find("=>") else {
        // Um `if ..` sem `=>`: usa o bloco seguinte.
        return match window.find('{') {
            Some(br) => {
                let open = off + br;
                let end = close_of(b, open);
                src.get(open..end.min(src.len())).unwrap_or("")
            }
            None => window,
        };
    };
    let after = &window[arrow + 2..];
    match after.find('{') {
        Some(br) if after[..br].trim().is_empty() => {
            let open = off + arrow + 2 + br;
            let end = close_of(b, open);
            src.get(open..end.min(src.len())).unwrap_or("")
        }
        _ => {
            let stop = after.find('\n').unwrap_or(after.len());
            &after[..stop]
        }
    }
}

/// O corpo devolve o MESMO id a outro canal? Então é um SALTO, não um término.
fn forwards_same_id(body: &str) -> bool {
    for needle in ["PanelEvent::", "ToolPanelEvent("] {
        let mut from = 0usize;
        while let Some(rel) = body[from..].find(needle) {
            let at = from + rel + needle.len();
            from = at;
            let rest = if needle == "ToolPanelEvent(" {
                body[at..].trim_start()
            } else {
                match body[at..].find('(') {
                    Some(p) => body[at + p + 1..].trim_start(),
                    None => continue,
                }
            };
            let rest = rest.strip_prefix('*').unwrap_or(rest);
            if let Some(tail) = rest.strip_prefix("id")
                && !tail.starts_with(|c: char| c.is_alphanumeric() || c == '_')
            {
                return true;
            }
        }
    }
    false
}

#[allow(clippy::too_many_arguments)]
fn classify(
    src: &str,
    rel: &str,
    h: &Hit,
    is_table: bool,
    decl_here: bool,
    seams: &[(usize, usize)],
) -> Kind {
    let line = line_text(src, h.start);
    if is_table
        && (line.contains(&format!("const {}", h.name))
            || line.contains(&format!("static {}", h.name)))
    {
        return Kind::TblDecl;
    }
    if seams.iter().any(|(a, b)| h.start >= *a && h.start < *b) {
        return Kind::Seam;
    }
    if in_use_item(src, h.start) {
        return Kind::Use;
    }
    if decl_here {
        return Kind::Decl;
    }
    if is_ids_dir(rel) {
        // Dentro do módulo de ids, tudo o que não é a declaração é TABELA.
        return Kind::Map;
    }
    if compares(line, &h.name) {
        return if forwards_same_id(arm_body(src, h.start)) {
            Kind::Fwd
        } else if line.contains("=>") || line.contains('|') {
            Kind::Arm
        } else {
            Kind::Cmp
        };
    }
    let seg = last_seg(&h.callee);
    if h.bracket == b'(' && PANEL_CALLS.contains(&seg) {
        return Kind::Panel;
    }
    if line.contains("parent:") {
        return Kind::Panel;
    }
    if h.bracket == b'(' && SELF_CALLS.contains(&seg) {
        return Kind::Self_;
    }
    if h.bracket == b'(' && seg == "register" && h.open != usize::MAX {
        let end = close_of(src.as_bytes(), h.open);
        let call = src.get(h.open..end.min(src.len())).unwrap_or("");
        if SELF_KINDS
            .iter()
            .any(|k| call.contains(&format!("InteractiveState::{k}")))
        {
            return Kind::Self_;
        }
    }
    let next_non_space = src[h.end..].trim_start().chars().next().unwrap_or(' ');
    if h.bracket == b'('
        && (DECOR_CALLS.contains(&h.callee.as_str())
            || (GROUP_CALLS.contains(&seg) && next_non_space == ','))
    {
        return Kind::Decor;
    }
    if h.bracket == b'(' && LINK_CALLS.contains(&seg) {
        return Kind::Link;
    }
    if h.bracket == b'(' && h.callee.is_empty() && next_non_space == ',' && h.open != usize::MAX {
        let head = src.get(h.open + 1..h.start).unwrap_or("").trim();
        if head.is_empty() || head.ends_with("::") {
            return Kind::Map;
        }
    }
    if h.bracket == b'(' && READ_CALLS.contains(&seg) {
        return if is_paint_file(rel) {
            Kind::ReadPaint
        } else {
            Kind::Read
        };
    }
    if is_paint_file(rel) {
        return Kind::Paint;
    }
    if is_pop_file(rel) {
        return Kind::Pop;
    }
    if h.bracket == b'[' {
        return Kind::List;
    }
    if h.bracket == b'(' && seg == "register" {
        return Kind::Reg;
    }
    Kind::Other
}

struct Dsu(BTreeMap<String, String>);
impl Dsu {
    fn find(&mut self, x: &str) -> String {
        let p = self.0.get(x).cloned().unwrap_or_else(|| x.to_string());
        if p == x {
            return p;
        }
        let r = self.find(&p);
        self.0.insert(x.to_string(), r.clone());
        r
    }
    fn union(&mut self, a: &str, b: &str) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra != rb {
            self.0.insert(ra, rb);
        }
    }
}

#[test]
fn the_painted_control_reaches_a_consumer() {
    let root = repo_root();
    let mut files: Vec<(String, String)> = Vec::new();
    collect_rs(&root.join("crates"), &root, &mut files);
    collect_rs(&root.join("shells"), &root, &mut files);

    // 1. A população de ids: `pub const X: NodeId = ..` no módulo de ids.
    let mut ids: BTreeSet<String> = BTreeSet::new();
    let mut decl_site: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for (fi, (rel, src)) in files.iter().enumerate() {
        if !is_ids_dir(rel) {
            continue;
        }
        for (li, line) in src.lines().enumerate() {
            let t = line.trim_start();
            let Some(rest) = t.strip_prefix("pub const ") else {
                continue;
            };
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if name.is_empty() {
                continue;
            }
            if rest[name.len()..].trim_start().starts_with(": NodeId") {
                ids.insert(name.clone());
                decl_site.entry(name).or_insert((fi, li + 1));
            }
        }
    }
    assert!(
        ids.len() >= MIN_IDS,
        "a sonda so' achou {} `pub const X: NodeId` em `ph2d-editor-core/src/ids/**` \
         (baseline 2026-08-30: {MIN_IDS}). Num corpus vazio este gate seria verde para sempre.",
        ids.len()
    );

    // 2. As tabelas de `NodeId` (a família que herda os usos do nome da tabela).
    let mut tables: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (_, src) in &files {
        for (k, v) in node_id_tables(src, &ids) {
            tables.entry(k).or_default().extend(v);
        }
    }
    tables.retain(|_, v| !v.is_empty());
    assert!(
        tables.len() >= MIN_TABLES,
        "a sonda so' achou {} tabela(s) `[NodeId]` (baseline 2026-08-30: {MIN_TABLES}). \
         Sem elas, dezenas de ids vivos por UM `for` no populate leriam como mortos.",
        tables.len()
    );
    let mut member_of: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (t, ms) in &tables {
        for m in ms {
            member_of.entry(m.clone()).or_default().insert(t.clone());
        }
    }

    // 3. Uma passagem por ficheiro: classificar + colher os pares.
    let mut occ: BTreeMap<String, Vec<Occ>> = BTreeMap::new();
    let mut dsu = Dsu(BTreeMap::new());
    for (fi, (rel, src)) in files.iter().enumerate() {
        let hits = scan_file(src);
        if hits.is_empty() {
            continue;
        }
        let seams = seam_ranges(src);
        let mut groups: BTreeMap<usize, Vec<String>> = BTreeMap::new();
        for h in &hits {
            let is_id = ids.contains(&h.name);
            let is_table = tables.contains_key(&h.name);
            if !is_id && !is_table {
                continue;
            }
            if is_id && h.bracket == b'(' && h.open != usize::MAX && !is_ids_dir(rel) {
                groups.entry(h.open).or_default().push(h.name.clone());
            }
            let decl_here = decl_site
                .get(&h.name)
                .is_some_and(|&(f, l)| f == fi && l == h.line);
            let kind = classify(src, rel, h, is_table, decl_here, &seams);
            occ.entry(h.name.clone()).or_default().push(Occ {
                kind,
                file: fi,
                line: h.line,
                text: line_text(src, h.start).trim().chars().take(120).collect(),
            });
        }
        for members in groups.values() {
            let uniq: BTreeSet<&String> = members.iter().collect();
            let list: Vec<&String> = uniq.into_iter().collect();
            for w in list.windows(2) {
                dsu.union(w[0], w[1]);
            }
        }
    }

    // 4. Términos por componente.
    let panel_registry: BTreeSet<String> = tables
        .get(PANEL_REGISTRY_TABLE)
        .cloned()
        .unwrap_or_default();
    assert!(
        !panel_registry.is_empty(),
        "a tabela `{PANEL_REGISTRY_TABLE}` (o registo de paineis do repo) nao foi lida — sem ela \
         todo CORPO de painel lê como controlo morto."
    );

    let mut term_of: BTreeMap<String, usize> = BTreeMap::new();
    let mut wired_of: BTreeMap<String, usize> = BTreeMap::new();
    let mut excluded: BTreeSet<String> = BTreeSet::new();
    for name in &ids {
        let mut kinds: Vec<Kind> = occ
            .get(name)
            .map(|v| v.iter().map(|o| o.kind).collect())
            .unwrap_or_default();
        for t in member_of.get(name).into_iter().flatten() {
            for o in occ.get(t).into_iter().flatten() {
                if !matches!(o.kind, Kind::Decl | Kind::TblDecl | Kind::List | Kind::Use) {
                    kinds.push(o.kind);
                }
            }
        }
        let term = kinds.iter().filter(|k| k.is_terminus()).count();
        let wired = kinds.iter().filter(|k| k.is_wired()).count();
        if kinds.iter().any(|k| matches!(k, Kind::Decor | Kind::Panel))
            || panel_registry.contains(name)
        {
            excluded.insert(name.clone());
        }
        term_of.insert(name.clone(), term);
        wired_of.insert(name.clone(), wired);
    }
    let mut comp_term: BTreeMap<String, usize> = BTreeMap::new();
    for name in &ids {
        let r = dsu.find(name);
        *comp_term.entry(r).or_insert(0) += term_of[name];
    }

    let population: Vec<&String> = ids
        .iter()
        .filter(|n| wired_of[*n] > 0 && !excluded.contains(*n))
        .collect();
    assert!(
        population.len() >= MIN_POPULATION,
        "a populacao caiu para {} controlos (baseline 2026-08-30: {MIN_POPULATION}). \
         A classificacao partiu, e um corpus encolhido passa sempre.",
        population.len()
    );

    let mut detected: BTreeSet<String> = BTreeSet::new();
    let mut offenders: Vec<String> = Vec::new();
    for name in &population {
        let root_id = dsu.find(name);
        if comp_term[&root_id] > 0 {
            continue;
        }
        detected.insert((*name).clone());
        if NO_CONSUMER_PENDING.iter().any(|(n, _)| n == *name) {
            continue;
        }
        let where_ = occ
            .get(*name)
            .and_then(|v| {
                v.iter()
                    .find(|o| matches!(o.kind, Kind::Paint | Kind::ReadPaint | Kind::Reg))
                    .or_else(|| v.iter().find(|o| o.kind.is_wired()))
            })
            .map(|o| format!("{}:{} — {}", files[o.file].0, o.line, o.text))
            .unwrap_or_else(|| "?".to_string());
        offenders.push(format!("ids::{name}\n      {where_}"));
    }

    offenders.sort();
    assert!(
        offenders.is_empty(),
        "controlos PINTADOS/REGISTADOS cujo id nao chega a consumidor nenhum — o artista ve, \
         clica, e nada acontece:\n  {}\n\n\
         cura: um braco que PERGUNTE pelo id (`id == ids::X`, `ids::X =>`, uma tabela \
         `(ids::X, acao)`), ou tirar o controlo da tela. ⛔ Encaminhar `PanelEvent::Click(id)` \
         para outro canal NAO conta: quem tem de responder e' o outro lado.\n\
         Se o id estiver VIVO por uma porta que esta regua nao ve (o cabecalho lista os pontos \
         cegos), a linha vai para NO_CONSUMER_PENDING **com o motivo medido**.",
        offenders.join("\n  ")
    );

    // **Controlo POSITIVO / catraca que só encolhe.**
    let stale: Vec<String> = NO_CONSUMER_PENDING
        .iter()
        .filter(|(n, _)| !detected.contains(*n))
        .map(|(n, why)| format!("ids::{n} ({why})"))
        .collect();
    assert!(
        stale.is_empty(),
        "estas linhas do NO_CONSUMER_PENDING ja' nao sao detectadas. Ou a divida foi paga — e a \
         catraca DESCE, apagando a linha — ou a regua deixou de ver a familia, e entao todo o \
         verde deste gate e' falso:\n  {}",
        stale.join("\n  ")
    );

    // **Controlo NEGATIVO** — os vivos verificados à mão continuam vivos, e
    // continuam a ser VISTOS (um id que sumiu do corpus não prova nada).
    let unseen: Vec<&str> = KNOWN_LIVE
        .iter()
        .filter(|(n, _)| !ids.contains(*n))
        .map(|(n, _)| *n)
        .collect();
    assert!(
        unseen.is_empty(),
        "ids do controlo NEGATIVO que a sonda ja' nao ve — ela pode ter ficado cega sem que nada \
         reprove:\n  {}",
        unseen.join("\n  ")
    );
    let false_positives: Vec<String> = KNOWN_LIVE
        .iter()
        .filter(|(n, _)| detected.contains(*n))
        .map(|(n, why)| format!("ids::{n} — VIVO por: {why}"))
        .collect();
    assert!(
        false_positives.is_empty(),
        "a regua sinalizou controlos VERIFICADOS VIVOS. Ela ficou grosseira: leia a regra antes \
         de mexer no codigo do produto:\n  {}",
        false_positives.join("\n  ")
    );
}
