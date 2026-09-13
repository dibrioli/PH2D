//! ⭐⭐ **O CENSO DE COLISÕES é DERIVADO da workspace** — nenhum id se lista à mão.
//!
//! # A pergunta, e porque ela existe
//!
//! Os `NodeId` de widget são HASH de slug (FNV-1a, [`ph2d_tool_registry::hash_node_id`]) desde a
//! Wave 2 PR 11.3 — antes eram alocados à mão por faixas, e seis colisões numéricas já tinham
//! passado (380, 381, 382, 853, 854, 855). O hash acaba com a alocação; o que ele NÃO acaba é o
//! erro de copiar-colar: **o mesmo slug escrito em dois sítios** dá o mesmo id a dois widgets, e o
//! hit-test encaminha o clique de um para o outro em silêncio.
//!
//! # O que este ficheiro era, e porque morreu
//!
//! Até 2026-09-12 ele trazia `CHROME_IDS`, uma tabela **à mão** de **619** consts contra **3 453**
//! literais `hash_node_id("…")` na workspace. A própria tabela confessava o apodrecimento em cinco
//! comentários (*«a lista é mantida à mão, então um id novo só participa da checagem quando alguém
//! lembra de o trazer»*) — e fazia pior do que esquecer: **mantinha vivos ids que o produto já não
//! usava** (o `INSP_TRANSFORM_SECTION` de 2026-08-21; 33 órfãos medidos pelo `scripts/censo-ids.py`
//! quando ela saiu). E prendia a DEFINIÇÃO de cada id na `ph2d-editor-core`, porque nomeava
//! `ids::X` — a obra A5b (os ids descem para o painel ou a ferramenta dona) não cabia ao lado dela.
//! *Duas respostas à mesma pergunta divergem no dia seguinte.*
//!
//! # O que ele lê — as QUATRO formas de um id nascer
//!
//! 1. **o literal** `hash_node_id("slug")` — lido em TODO `.rs` de `crates/`, `shells/`, `tools/` e
//!    `tests/`, sem comentários, e hasheado com a porta de runtime (cuja concordância com a `const
//!    fn` é gateada na `ph2d-tool-registry` e nos três testes irmãos das famílias);
//! 2. **o molde** `hash_node_id_runtime(&format!("prefixo.{i}"))` — o id derivado em runtime; o
//!    que se verifica é que nenhum literal SOLETRA um molde e que dois moldes não soletram o mesmo;
//! 3. **o hash de uma expressão** (`hash_node_id(m.id)`, `hash_node_id_runtime(s)`) — nomes de
//!    registo (ferramentas, nós, componentes) e sementes; estes **listam-se um a um** em
//!    [`FORMAS_NAO_LITERAIS`], cada um com quem o cobre, e a tabela tem a metade «já não descreve
//!    nada»;
//! 4. **a FNV à mão** que devolve `NodeId` — idem, na mesma tabela.
//!
//! E o `NodeId(<inteiro>)` de um `const` (as linhas-fixture da hierarquia, 400..411, que ficam
//! numéricas de propósito pela conta dos bits de companheiro) entra nas réguas de raiz e de
//! colisão.
//!
//! ⚠️ **Os pisos de população estão no próprio varrimento** (HOWTO §2.7): um walker que perdesse um
//! directório varreria menos e ficaria verde.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_tool_registry::hash_node_id_runtime;

use crate::cfg_test_modules::is_declared_under_cfg_test;

// ────────────────────────────────────────────────────────────────────────────────────────────────
// As tabelas NOMEADAS (as duas só encolhem, e cada uma tem a metade «já não descreve nada»)
// ────────────────────────────────────────────────────────────────────────────────────────────────

/// ⛔ **O mesmo slug em DOIS sítios de produto** — a catraca, com o número de sítios medido.
///
/// As três entradas são o MESMO id de painel declarado na fundação (que o lê no despacho de rolagem
/// e no filtro de menus) **e** na crate da ferramenta/painel. A cura é UMA definição — a da fundação —
/// e os outros sítios a nomeá-la. ⛔ **Bloqueada nesta rodada pela cerca da `line/render-loop`**: os
/// `shells/desktop/src/render_loop/{color_equalization,equalize_sizes,upscale}_bridge.rs` nomeiam a
/// CÓPIA (`ph2d_tool_color_equalization::ids::CEQ_PANEL`, `ph2d_tool_equalize_sizes::ids::EQS_PANEL`,
/// `ph2d_panel_upscale::ids::UPS_PANEL`), e apagá-la sem editar esses ficheiros pediria uma fachada.
/// ⚠️ **A cerca não é o único leitor das cópias** (medido 2026-09-12, auditoria de fecho): o `NODE_ID`
/// do `lib.rs` e o `paint.rs` dos três painéis também as nomeiam, e um teste do
/// `ph2d-tool-color-equalization/src/ids.rs` também. ⇒ depois das duas fusões o integrador troca TODO
/// leitor (`grep -rn 'ids::CEQ_PANEL\|ids::EQS_PANEL\|ids::UPS_PANEL'`) por `ph2d_editor_core::ids::…`,
/// apaga as cópias e apaga estas linhas — e o compilador aponta cada leitor que ficar para trás.
const SLUGS_REPETIDOS_TOLERADOS: &[(&str, usize, &str)] = &[
    (
        "panel.color_equalization",
        2,
        "CEQ_PANEL na fundação e em ph2d-tool-color-equalization/src/ids.rs — cerca: render_loop/color_equalization_bridge.rs nomeia a cópia",
    ),
    (
        "panel.equalize_sizes",
        2,
        "EQS_PANEL na fundação e em ph2d-tool-equalize-sizes/src/ids.rs — cerca: render_loop/equalize_sizes_bridge.rs nomeia a cópia",
    ),
    (
        "panel.upscale",
        2,
        "UPS_PANEL na fundação e em ph2d-panel-upscale/src/ids.rs — cerca: render_loop/upscale_bridge.rs nomeia a cópia",
    ),
];

/// **As formas de id que NÃO são um literal nem um molde** — `(ficheiro, função, espécie, quem cobre)`.
///
/// ⚠️ A chave é o ficheiro e a FUNÇÃO que envolve o sítio (nunca a linha, que anda a cada edição):
/// mover a função de casa torna a entrada obsoleta, e a metade [`every_non_literal_hash_is_named`]
/// obriga a reescrevê-la — é essa a prova de que alguém olhou para a forma no sítio novo.
const FORMAS_NAO_LITERAIS: &[(&str, &str, Especie, &str)] = &[
    (
        "crates/ph2d-app-components/src/component_palette.rs",
        "item_id",
        Especie::HashDeExpressao,
        "item da paleta de componentes = hash do NOME CANÓNICO do tipo (espaço de nomes Rust, não de slugs); só é hit-registado com a paleta aberta",
    ),
    (
        "crates/ph2d-app-field3d/src/shape_palette.rs",
        "item_id",
        Especie::HashDeExpressao,
        "item da paleta de formas = hash da CHAVE i18n da forma (`panel.model3d.add.*`); só é hit-registado com a paleta aberta",
    ),
    (
        "crates/ph2d-app-motion/src/motion_bridge_library.rs",
        "build_palette_model",
        Especie::HashDeExpressao,
        "item da biblioteca de nós = hash do `type_name` do nó (`motion.*`); ida e volta gateada em motion_bridge_library_tests",
    ),
    (
        "crates/ph2d-app-motion/src/motion_bridge_library.rs",
        "route_palette_pick",
        Especie::HashDeExpressao,
        "a leitura inversa do mesmo hash (`type_name` → nó); a mesma porta, gateada com a ida",
    ),
    (
        "crates/ph2d-panel-timeline/src/ids/timeline.rs",
        "dynamic_id",
        Especie::HashDeExpressao,
        "a SEMENTE por domínio das famílias dinâmicas da timeline; a separação entre domínios é o timeline_dynamic_ids_dont_collide_with_chrome_or_each_other",
    ),
    (
        "crates/ph2d-panel-timeline/src/ids/timeline.rs",
        "dynamic_id",
        Especie::FnvAMao,
        "o corpo da mesma semente: FNV sobre os bytes `u64` das partes (não é a lei de slug, é uma extensão dela); coberto pelo mesmo teste",
    ),
    (
        "crates/ph2d-editor-core/src/screens/hero/chrome/image_actions.rs",
        "oneshot_tool_for",
        Especie::HashDeExpressao,
        "pill de ferramenta = hash do `manifest.id` — IGUAL por desenho ao const da fundação; tool-vs-tool é `detect_collisions`, id-vs-const é chrome_manifest_coverage",
    ),
    (
        "crates/ph2d-editor-core/src/screens/hero/chrome/image_actions.rs",
        "stateful_tool_for",
        Especie::HashDeExpressao,
        "idem (o braço Stateful do mesmo despacho)",
    ),
    (
        "crates/ph2d-editor-core/src/screens/hero/topbar/image_action_row.rs",
        "image_action_pills",
        Especie::HashDeExpressao,
        "idem (a fileira que pinta os pills)",
    ),
    (
        "crates/ph2d-editor-core/src/screens/hero/live.rs",
        "fold_track",
        Especie::FnvAMao,
        "id de TRACK de motion (não de hit): FNV com offset basis próprio, distinção medida no doc da função (0..100 000 → 100 000)",
    ),
    (
        "crates/ph2d-editor-core/src/screens/hero/live.rs",
        "scroll_track",
        Especie::FnvAMao,
        "idem, a família da rolagem (outra constante de mistura, as duas não partilham um valor)",
    ),
    (
        "crates/ph2d-editor-core/src/widget/command_palette/cascade.rs",
        "cascade_id",
        Especie::FnvAMao,
        "a cascata da paleta: FNV do índice semeada no const `command_palette.cascade` (id de motion, não de hit)",
    ),
    (
        "crates/ph2d-tool-registry/src/node_id.rs",
        "detect_collisions",
        Especie::HashDeExpressao,
        "é a PRÓPRIA verificação tool-vs-tool dos manifestos",
    ),
    (
        "crates/ph2d-viewport3d/src/view_menu.rs",
        "row_id",
        Especie::HashDeExpressao,
        "linha do menu de vistas = hash da CHAVE i18n da vista; só é hit-registada com o menu aberto",
    ),
    (
        "shells/desktop/src/render_loop/fase_image_tools_mode_and_pills.rs",
        "fase_image_tools_mode_and_pills",
        Especie::HashDeExpressao,
        "pill da ferramenta activa = hash do `manifest.id`, como os image_actions (mudou-se do `run_render_frame` com a fase, integração de 13/09)",
    ),
    (
        "crates/ph2d-app-motion/src/motion_bridge_color.rs",
        "card_swatch_id",
        Especie::FnvAMao,
        "amostra de cor do CARTÃO de nó: FNV de `motion-card/swatch/` + nó + âncora (espaço separado do painel, doc da função)",
    ),
    (
        "crates/ph2d-panel-motion-graph/src/paint.rs",
        "fnv_id",
        Especie::FnvAMao,
        "ids por-elemento do grafo (portas, fios, divisória) — cópia da lei sem o `0 → 1`; o espaço é o do grafo",
    ),
    (
        "crates/ph2d-panel-motion-params/src/snapshot_ids.rs",
        "fnv_id",
        Especie::FnvAMao,
        "ids por-row do painel de params (o mesmo esquema do grafo, com prefixo próprio)",
    ),
    (
        "crates/ph2d-panel-motion-params/src/rows_paint_sections.rs",
        "section_id",
        Especie::FnvAMao,
        "cabeçalho de secção do painel de params = FNV de `motion_param/section/<título>`",
    ),
    (
        "crates/ph2d-param-editors/src/lib.rs",
        "fnv_id",
        Especie::FnvAMao,
        "sub-ids dos editores ricos (curva, gradiente, paleta), o mesmo esquema dos dois painéis de Motion",
    ),
];

// ────────────────────────────────────────────────────────────────────────────────────────────────
// O varrimento
// ────────────────────────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Especie {
    /// `hash_node_id(expr)` / `hash_node_id_runtime(expr)` com uma expressão que não é literal nem molde.
    HashDeExpressao,
    /// Uma função que devolve `NodeId` e mistura o primo FNV à mão.
    FnvAMao,
}

#[derive(Debug)]
struct Literal {
    slug: String,
    file: String,
    line: usize,
    produto: bool,
}

#[derive(Debug)]
struct Molde {
    molde: String,
    file: String,
    func: String,
}

struct Censo {
    literais: Vec<Literal>,
    /// `const X: NodeId = NodeId(<inteiro>)` de produto: `(nome, valor, ficheiro)`.
    numericos: Vec<(String, u64, String)>,
    /// `const NOME: NodeId = …` de produto, ONDE quer que more (módulo ou corpo de função):
    /// `(nome, valor, ficheiro)`.
    nomes: Vec<(String, Valor, String)>,
    moldes: Vec<Molde>,
    formas: BTreeSet<(String, String, Especie)>,
    ficheiros_lidos: usize,
}

/// ⚠️ Os pisos: medidos em 2026-09-12 sobre o `main` da `line/editor-core`, com folga para baixo só
/// no que uma limpeza legítima pode reduzir. Uma varredura que falhe um directório cai abaixo deles.
const PISO_FICHEIROS: usize = 7_000; // medido: 7 553
const PISO_LITERAIS: usize = 3_000;
/// ⚠️ `15`, e não os `16` que um `grep` conta: um deles só tem o literal num COMENTÁRIO.
const PISO_CRATES_COM_LITERAL: usize = 15;
const PISO_MOLDES: usize = 150;
/// Nomes distintos de `const …: NodeId` de produto — medido **2 452** na integração de 13/09.
const PISO_NOMES: usize = 2_000;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("a raiz da workspace")
        .to_path_buf()
}

fn censo() -> &'static Censo {
    static C: OnceLock<Censo> = OnceLock::new();
    C.get_or_init(varrer)
}

fn varrer() -> Censo {
    let root = root();
    let mut ficheiros = Vec::new();
    for d in ["crates", "shells", "tools", "tests"] {
        walk(&root.join(d), &mut ficheiros);
    }
    // As crates que declaram OUTRO `NodeId` (o grafo de nós tem o seu): uma FNV à mão lá não é um
    // id de widget. Medido: só a `ph2d-nodegraph`; derivado, não escrito.
    let mut outro_node_id: BTreeSet<String> = BTreeSet::new();
    let mut textos: Vec<(PathBuf, String, String, String)> = Vec::new();
    for p in &ficheiros {
        let Ok(src) = std::fs::read_to_string(p) else {
            continue;
        };
        let rel = p
            .strip_prefix(&root)
            .expect("dentro da raiz")
            .to_string_lossy()
            .replace('\\', "/");
        let (com, cod) = limpar(&src);
        if cod.contains("struct NodeId") && !rel.starts_with("crates/ph2d-a11y/") {
            outro_node_id.insert(crate_dir(&rel));
        }
        if !(src.contains("hash_node_id") || tem_primo_fnv(&cod) || cod.contains("NodeId(")) {
            continue;
        }
        textos.push((p.clone(), rel, com, cod));
    }

    let mut literais = Vec::new();
    let mut numericos = Vec::new();
    let mut nomes = Vec::new();
    let mut moldes = Vec::new();
    let mut formas = BTreeSet::new();
    for (abs, rel, com, cod) in &textos {
        let ficheiro_de_teste = caminho_de_teste(rel)
            || ((com.contains("hash_node_id") || tem_primo_fnv(cod))
                && is_declared_under_cfg_test(abs));
        let regioes = regioes_de_teste(cod);
        let produto =
            |off: usize| !ficheiro_de_teste && !regioes.iter().any(|&(a, e)| off >= a && off < e);

        for (off, nome) in chamadas(cod, "hash_node_id") {
            debug_assert_eq!(nome, "hash_node_id");
            match argumento(com, off) {
                Arg::Literal(slug) => literais.push(Literal {
                    slug,
                    file: rel.clone(),
                    line: linha(com, off),
                    produto: produto(off),
                }),
                Arg::Molde(_) | Arg::Expressao => {
                    if produto(off) {
                        formas.insert((rel.clone(), funcao_em(cod, off), Especie::HashDeExpressao));
                    }
                }
            }
        }
        for (off, _) in chamadas(cod, "hash_node_id_runtime") {
            match argumento(com, off) {
                Arg::Literal(slug) => literais.push(Literal {
                    slug,
                    file: rel.clone(),
                    line: linha(com, off),
                    produto: produto(off),
                }),
                Arg::Molde(m) => {
                    if produto(off) {
                        moldes.push(Molde {
                            molde: m,
                            file: rel.clone(),
                            func: funcao_em(cod, off),
                        });
                    }
                }
                Arg::Expressao => {
                    if produto(off) {
                        formas.insert((rel.clone(), funcao_em(cod, off), Especie::HashDeExpressao));
                    }
                }
            }
        }
        if !outro_node_id.contains(&crate_dir(rel)) {
            for off in primos_fnv(cod) {
                if !produto(off) {
                    continue;
                }
                let f = funcao_em(cod, off);
                if assinatura_de(cod, off).contains("NodeId") {
                    formas.insert((rel.clone(), f, Especie::FnvAMao));
                }
            }
        }
        for (nome, valor, off) in consts_de_node_id(com, cod) {
            if produto(off) {
                nomes.push((nome, valor, rel.clone()));
            }
        }
        // ⚠️ Só nos módulos que DECLARAM ids (`…/ids/…` ou `…/ids.rs`, a mesma convenção do
        // `architecture_panel_wiring_parity`): a pele de canvas do widget tem um `PREVIEW_ID =
        // NodeId(0)` de propósito — um id real ali colidiria com o widget homónimo do painel nativo
        // (doc em `widget/skin.rs`) — e ele não é um id de encaminhamento.
        if rel.contains("/ids/") || rel.ends_with("/ids.rs") {
            for (nome, valor) in consts_numericos(cod) {
                if produto(0) {
                    numericos.push((nome, valor, rel.clone()));
                }
            }
        }
    }

    let crates: BTreeSet<String> = literais.iter().map(|l| crate_dir(&l.file)).collect();
    let c = Censo {
        literais,
        numericos,
        nomes,
        moldes,
        formas,
        ficheiros_lidos: ficheiros.len(),
    };
    assert!(
        c.ficheiros_lidos >= PISO_FICHEIROS,
        "o varrimento leu {} ficheiros .rs e esperava >= {PISO_FICHEIROS} — perdeu um directório",
        c.ficheiros_lidos
    );
    assert!(
        c.literais.len() >= PISO_LITERAIS,
        "o varrimento achou {} literais `hash_node_id(\"…\")` e esperava >= {PISO_LITERAIS} — o parser cegou",
        c.literais.len()
    );
    assert!(
        crates.len() >= PISO_CRATES_COM_LITERAL,
        "literais em só {} crates (piso {PISO_CRATES_COM_LITERAL}): {crates:?}",
        crates.len()
    );
    assert!(
        c.moldes.len() >= PISO_MOLDES,
        "o varrimento achou {} moldes `hash_node_id_runtime(&format!(…))` e esperava >= {PISO_MOLDES}",
        c.moldes.len()
    );
    c
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        let nome = e.file_name();
        let nome = nome.to_string_lossy();
        if p.is_dir() {
            if nome != "target" && !nome.starts_with('.') {
                walk(&p, out);
            }
        } else if nome.ends_with(".rs") {
            out.push(p);
        }
    }
}

fn crate_dir(rel: &str) -> String {
    rel.split('/').take(2).collect::<Vec<_>>().join("/")
}

fn caminho_de_teste(rel: &str) -> bool {
    rel.split('/')
        .any(|c| c == "tests" || c == "benches" || c == "examples")
}

/// `(sem_comentarios, sem_comentarios_nem_strings)` — os dois com os MESMOS offsets do original
/// (cada byte apagado vira um espaço; as quebras de linha ficam).
fn limpar(src: &str) -> (String, String) {
    let b = src.as_bytes();
    let n = b.len();
    let mut com = b.to_vec();
    let mut cod = b.to_vec();
    fn apaga(v: &mut [u8], a: usize, e: usize) {
        let e = e.min(v.len());
        for x in v.iter_mut().take(e).skip(a) {
            if *x != b'\n' {
                *x = b' ';
            }
        }
    }
    let mut i = 0;
    while i < n {
        let c = b[i];
        if c == b'/' && i + 1 < n && b[i + 1] == b'/' {
            let e = b[i..].iter().position(|&x| x == b'\n').map_or(n, |k| i + k);
            apaga(&mut com, i, e);
            apaga(&mut cod, i, e);
            i = e;
            continue;
        }
        if c == b'/' && i + 1 < n && b[i + 1] == b'*' {
            let (mut depth, mut j) = (1, i + 2);
            while j < n && depth > 0 {
                if b[j] == b'/' && j + 1 < n && b[j + 1] == b'*' {
                    depth += 1;
                    j += 2;
                } else if b[j] == b'*' && j + 1 < n && b[j + 1] == b'/' {
                    depth -= 1;
                    j += 2;
                } else {
                    j += 1;
                }
            }
            apaga(&mut com, i, j);
            apaga(&mut cod, i, j);
            i = j;
            continue;
        }
        let ident_antes = i > 0 && (b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_');
        if !ident_antes && (c == b'r' || c == b'b') {
            let mut j = i;
            if b[j] == b'b' && j + 1 < n && b[j + 1] == b'r' {
                j += 1;
            }
            if b[j] == b'r' && j + 1 < n && (b[j + 1] == b'"' || b[j + 1] == b'#') {
                let (mut k, mut h) = (j + 1, 0);
                while k < n && b[k] == b'#' {
                    h += 1;
                    k += 1;
                }
                if k < n && b[k] == b'"' {
                    let mut fim = n;
                    let mut t = k + 1;
                    while t < n {
                        if b[t] == b'"'
                            && b[t + 1..].iter().take(h).filter(|&&x| x == b'#').count() == h
                        {
                            fim = t;
                            break;
                        }
                        t += 1;
                    }
                    apaga(&mut cod, k + 1, fim);
                    i = (fim + 1 + h).min(n);
                    continue;
                }
            }
        }
        if c == b'"' {
            let mut j = i + 1;
            while j < n {
                if b[j] == b'\\' {
                    j += 2;
                } else if b[j] == b'"' {
                    break;
                } else {
                    j += 1;
                }
            }
            apaga(&mut cod, i + 1, j);
            i = j + 1;
            continue;
        }
        if c == b'\'' {
            if i + 1 < n && b[i + 1] == b'\\' {
                if let Some(k) = b[i + 2..].iter().take(12).position(|&x| x == b'\'') {
                    apaga(&mut cod, i + 1, i + 2 + k);
                    i = i + 3 + k;
                    continue;
                }
            } else if let Some(ch) = src[i + 1..].chars().next() {
                let l = ch.len_utf8();
                if i + 1 + l < n && b[i + 1 + l] == b'\'' {
                    apaga(&mut cod, i + 1, i + 1 + l);
                    i += 2 + l;
                    continue;
                }
            }
        }
        i += 1;
    }
    (
        String::from_utf8(com).expect("utf-8 preservado"),
        String::from_utf8(cod).expect("utf-8 preservado"),
    )
}

/// Os corpos `#[cfg(test)] mod x { … }` inline — e `#[cfg(all(test, …))]`, que também é só-teste
/// (o `mod x;` de ficheiro é o `is_declared_under_cfg_test`).
fn regioes_de_teste(cod: &str) -> Vec<(usize, usize)> {
    let b = cod.as_bytes();
    let mut out = Vec::new();
    let mut from = 0;
    loop {
        let proximo = ["#[cfg(test)]", "#[cfg(all(test"]
            .iter()
            .filter_map(|p| cod[from..].find(p).map(|r| from + r))
            .min();
        let Some(a) = proximo else {
            break;
        };
        from = a + 1;
        let Some(fecha) = cod[a..].find(']') else {
            break;
        };
        // o `]` do atributo: num `all(test, feature = "x")` o primeiro `]` é o dele (strings apagadas)
        let mut j = a + fecha + 1;
        // atributos seguintes, visibilidade, `mod nome`
        loop {
            while j < b.len() && b[j].is_ascii_whitespace() {
                j += 1;
            }
            if b[j..].starts_with(b"#[") {
                j += b[j..].iter().position(|&x| x == b']').map_or(0, |k| k + 1);
                continue;
            }
            break;
        }
        let resto = &cod[j..];
        let resto = resto.strip_prefix("pub ").unwrap_or(resto);
        let Some(depois_mod) = resto.strip_prefix("mod ") else {
            continue;
        };
        let Some(abre_rel) = depois_mod.find(['{', ';']) else {
            continue;
        };
        if depois_mod.as_bytes()[abre_rel] != b'{' {
            continue;
        }
        let abre = j + (resto.as_ptr() as usize - cod[j..].as_ptr() as usize) + 4 + abre_rel;
        let mut depth = 0usize;
        for (k, &x) in b.iter().enumerate().skip(abre) {
            if x == b'{' {
                depth += 1;
            } else if x == b'}' {
                depth -= 1;
                if depth == 0 {
                    out.push((a, k + 1));
                    break;
                }
            }
        }
    }
    out
}

fn ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Os offsets das CHAMADAS a `nome(` no código (não a definição `fn nome(`, não um identificador maior).
fn chamadas<'a>(cod: &'a str, nome: &'a str) -> Vec<(usize, &'a str)> {
    let b = cod.as_bytes();
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(rel) = cod[from..].find(nome) {
        let a = from + rel;
        from = a + nome.len();
        if a > 0 && ident(b[a - 1]) {
            continue;
        }
        let mut j = a + nome.len();
        while j < b.len() && b[j].is_ascii_whitespace() {
            j += 1;
        }
        if j >= b.len() || b[j] != b'(' {
            continue;
        }
        if cod[..a].trim_end().ends_with("fn") {
            continue;
        }
        out.push((a, nome));
    }
    out
}

enum Arg {
    Literal(String),
    Molde(String),
    Expressao,
}

/// O argumento da chamada que começa em `off`, lido no texto COM strings.
fn argumento(com: &str, off: usize) -> Arg {
    let depois = &com[off..];
    let Some(p) = depois.find('(') else {
        return Arg::Expressao;
    };
    let mut s = depois[p + 1..].trim_start();
    if let Some(r) = s.strip_prefix('"') {
        return r
            .split('"')
            .next()
            .map_or(Arg::Expressao, |x| Arg::Literal(x.to_owned()));
    }
    s = s.strip_prefix('&').unwrap_or(s).trim_start();
    if let Some(r) = s.strip_prefix("format!") {
        let r = r.trim_start().strip_prefix('(').unwrap_or(r).trim_start();
        if let Some(r) = r.strip_prefix('"') {
            return r
                .split('"')
                .next()
                .map_or(Arg::Expressao, |x| Arg::Molde(x.to_owned()));
        }
    }
    Arg::Expressao
}

fn linha(txt: &str, off: usize) -> usize {
    txt[..off].bytes().filter(|&x| x == b'\n').count() + 1
}

/// O nome da última `fn` declarada antes de `off` — a função que envolve o sítio.
fn funcao_em(cod: &str, off: usize) -> String {
    let b = cod.as_bytes();
    let mut ultimo = String::from("?");
    let mut from = 0;
    while let Some(rel) = cod[from..off].find("fn ") {
        let a = from + rel;
        from = a + 3;
        if a > 0 && ident(b[a - 1]) {
            continue;
        }
        let nome: String = cod[a + 3..]
            .trim_start()
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !nome.is_empty() {
            ultimo = nome;
        }
    }
    ultimo
}

/// A assinatura (de `fn` até à primeira `{`) da função que envolve `off`.
fn assinatura_de(cod: &str, off: usize) -> &str {
    let Some(a) = cod[..off].rfind("fn ") else {
        return "";
    };
    let e = cod[a..].find('{').map_or(cod.len(), |k| a + k);
    &cod[a..e]
}

const PRIMOS_FNV: [&str; 3] = ["0x0000_0100_0000_01b3", "0x100000001b3", "1099511628211"];

fn tem_primo_fnv(cod: &str) -> bool {
    let low = cod.to_ascii_lowercase();
    PRIMOS_FNV.iter().any(|p| low.contains(p))
}

fn primos_fnv(cod: &str) -> Vec<usize> {
    let low = cod.to_ascii_lowercase();
    let mut out = Vec::new();
    for p in PRIMOS_FNV {
        let mut from = 0;
        while let Some(rel) = low[from..].find(p) {
            out.push(from + rel);
            from += rel + p.len();
        }
    }
    out
}

/// `const NOME: NodeId = NodeId(<inteiro>);`
fn consts_numericos(cod: &str) -> Vec<(String, u64)> {
    let mut out = Vec::new();
    for l in cod.lines() {
        let t = l.trim_start();
        let t = t.strip_prefix("pub ").unwrap_or(t);
        let Some(r) = t.strip_prefix("const ") else {
            continue;
        };
        let Some((nome, resto)) = r.split_once(':') else {
            continue;
        };
        let Some(v) = resto
            .trim_start()
            .strip_prefix("NodeId")
            .and_then(|x| x.trim_start().strip_prefix('='))
            .and_then(|x| x.trim_start().strip_prefix("NodeId("))
            .and_then(|x| x.split(')').next())
        else {
            continue;
        };
        if let Ok(v) = v.replace('_', "").trim().parse::<u64>() {
            out.push((nome.trim().to_owned(), v));
        }
    }
    out
}

/// O VALOR de um `const NOME: NodeId` — o slug de um `hash_node_id("…")` ou o inteiro de um `NodeId(n)`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Valor {
    Slug(String),
    Numero(u64),
}

/// Todo `const NOME: NodeId = hash_node_id("slug")` e `= NodeId(n)` do texto, ONDE quer que more — no
/// nível do módulo ou no corpo de uma função (foi aí que o `INSP_BLENDER_PICKER = NodeId(380)` do
/// seletor de cor se escondeu). Devolve o offset do `const`, para a régua de produto.
fn consts_de_node_id(com: &str, cod: &str) -> Vec<(String, Valor, usize)> {
    let b = cod.as_bytes();
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(rel) = cod[from..].find("const ") {
        let at = from + rel;
        from = at + "const ".len();
        if at > 0 && ident(b[at - 1]) {
            continue;
        }
        let depois = cod[from..].trim_start();
        let fim = depois
            .find(|c: char| !(c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'))
            .unwrap_or(depois.len());
        if fim == 0 {
            continue;
        }
        let nome = &depois[..fim];
        let Some(r) = depois[fim..].trim_start().strip_prefix(':') else {
            continue;
        };
        let r = r.trim_start();
        let r = r.strip_prefix("ph2d_a11y::").unwrap_or(r);
        let Some(r) = r.strip_prefix("NodeId") else {
            continue;
        };
        let Some(r) = r.trim_start().strip_prefix('=') else {
            continue;
        };
        let r = r.trim_start();
        let r = r.strip_prefix("ph2d_tool_registry::").unwrap_or(r);
        if r.starts_with("hash_node_id(") {
            if let Arg::Literal(slug) = argumento(com, cod.len() - r.len()) {
                out.push((nome.to_owned(), Valor::Slug(slug), at));
            }
        } else if let Some(n) = r
            .strip_prefix("NodeId(")
            .and_then(|x| x.split(')').next())
            .and_then(|v| v.replace('_', "").trim().parse::<u64>().ok())
        {
            out.push((nome.to_owned(), Valor::Numero(n), at));
        }
    }
    out
}

fn h(slug: &str) -> u64 {
    hash_node_id_runtime(slug).0
}

/// `slug → hash` agrupado: os grupos com MAIS de um slug distinto.
fn colisoes<'a>(slugs: impl IntoIterator<Item = &'a str>) -> Vec<(u64, BTreeSet<&'a str>)> {
    let mut por_hash: BTreeMap<u64, BTreeSet<&'a str>> = BTreeMap::new();
    for s in slugs {
        por_hash.entry(h(s)).or_default().insert(s);
    }
    por_hash.into_iter().filter(|(_, v)| v.len() > 1).collect()
}

/// Os hashes de todo id de PRODUTO que o censo vê — literais e numéricos. É o «chrome» contra o
/// qual as famílias dinâmicas se medem.
fn produto_hashes() -> BTreeSet<u64> {
    let c = censo();
    c.literais
        .iter()
        .filter(|l| l.produto)
        .map(|l| h(&l.slug))
        .chain(c.numericos.iter().map(|(_, v, _)| *v))
        .collect()
}

/// Um molde casa uma string? — cada marcador `{…}` soletra um **INTEIRO decimal** (um ou mais
/// dígitos), e os pedaços literais aparecem exactamente entre eles.
///
/// ⚠️ **O inteiro é a semântica, e ela foi MEDIDA:** com um marcador que soletrasse qualquer coisa,
/// a 1.ª corrida acusou 25 pares — `vector.shape.{index}` a «soletrar» `vector.shape.group.{index}`,
/// o `…{row}` a «soletrar» o `…{row}.num` —, todos impossíveis, porque os índices só produzem
/// dígitos. Um marcador que carrega TEXTO (a tag de uma variante, a chave de um knob) fica fora
/// desta régua, e é por isso que os testes de família abaixo CHAMAM essas funções com os valores
/// reais (pintor, flip, timeline, vector).
fn casa(molde: &str, s: &str) -> bool {
    let pedacos = pedacos(molde);
    let Some(mut resto) = s.strip_prefix(pedacos[0]) else {
        return false;
    };
    for p in &pedacos[1..] {
        let digitos = resto.bytes().take_while(u8::is_ascii_digit).count();
        if digitos == 0 {
            return false;
        }
        let Some(r) = resto[digitos..].strip_prefix(p) else {
            return false;
        };
        resto = r;
    }
    resto.is_empty()
}

fn pedacos(molde: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut resto = molde;
    loop {
        match resto.find('{') {
            Some(a) => {
                out.push(&resto[..a]);
                let e = resto[a..].find('}').map_or(resto.len(), |k| a + k + 1);
                resto = &resto[e..];
            }
            None => {
                out.push(resto);
                return out;
            }
        }
    }
}

/// Um exemplar do molde: cada marcador vira `v`.
fn exemplar(molde: &str, v: &str) -> String {
    pedacos(molde).join(v)
}

// ────────────────────────────────────────────────────────────────────────────────────────────────
// Os testes
// ────────────────────────────────────────────────────────────────────────────────────────────────

/// ⭐ **(1) Dois slugs DIFERENTES nunca hasheiam igual** — sobre TODO literal da workspace (produto
/// e teste) e todo `const NodeId(<inteiro>)`.
#[test]
fn chrome_node_ids_are_pairwise_unique() {
    // Controlo positivo do agrupador: sem ele, uma função que nunca agrupasse deixava isto verde.
    assert_eq!(
        colisoes(["a", "a", "b"]).len(),
        0,
        "o mesmo slug duas vezes não é colisão de hash"
    );
    let c = censo();
    let achadas = colisoes(c.literais.iter().map(|l| l.slug.as_str()));
    assert!(
        achadas.is_empty(),
        "slugs DIFERENTES com o mesmo NodeId — o hit-test encaminha um para o outro:\n  {}\n\
         cura: renomeie um dos slugs.",
        achadas
            .iter()
            .map(|(h, s)| format!("{h:#018x} ← {s:?}"))
            .collect::<Vec<_>>()
            .join("\n  ")
    );
    // Os numéricos contra os hashes, e entre si.
    let hashes: BTreeSet<u64> = c.literais.iter().map(|l| h(&l.slug)).collect();
    let mut vistos: BTreeMap<u64, &str> = BTreeMap::new();
    for (nome, v, file) in &c.numericos {
        assert!(
            !hashes.contains(v),
            "`{nome}` ({file}) = NodeId({v}) colide com um slug hasheado"
        );
        if let Some(outro) = vistos.insert(*v, nome) {
            assert_eq!(outro, nome, "`{nome}` e `{outro}` são o mesmo NodeId({v})");
        }
    }
}

/// ⭐ **(2) O MESMO slug nunca é escrito em dois sítios de PRODUTO** — o erro de copiar-colar para
/// que este ficheiro existe. Um sítio de teste que soletre o slug para o conferir não conta.
#[test]
fn no_slug_is_declared_in_two_places() {
    let c = censo();
    let mut sitios: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for l in c.literais.iter().filter(|l| l.produto) {
        sitios
            .entry(l.slug.as_str())
            .or_default()
            .push(format!("{}:{}", l.file, l.line));
    }
    let tolerados: BTreeMap<&str, usize> = SLUGS_REPETIDOS_TOLERADOS
        .iter()
        .map(|(s, n, _)| (*s, *n))
        .collect();
    let repetidos: Vec<String> = sitios
        .iter()
        .filter(|(s, v)| v.len() > 1 && tolerados.get(*s).is_none_or(|n| v.len() > *n))
        .map(|(s, v)| format!("{s:?} em {} sítios: {}", v.len(), v.join(" · ")))
        .collect();
    assert!(
        repetidos.is_empty(),
        "o MESMO slug declarado em mais de um sítio de produto — dois widgets com um id só:\n  {}\n\n\
         cura: UMA definição (na crate mais baixa que todos os leitores vêem), e os outros sítios a \
         nomeá-la. ⛔ Nunca uma entrada nova em SLUGS_REPETIDOS_TOLERADOS sem o bloqueador escrito.",
        repetidos.join("\n  ")
    );
}

/// **A catraca dos slugs repetidos ainda descreve a árvore** — a metade que a impede de virar licença.
#[test]
fn the_repeated_slug_ratchet_still_describes_the_tree() {
    let c = censo();
    for (slug, n, porque) in SLUGS_REPETIDOS_TOLERADOS {
        assert!(!porque.trim().is_empty(), "{slug:?} tolerado sem motivo");
        let agora = c
            .literais
            .iter()
            .filter(|l| l.produto && l.slug == *slug)
            .count();
        assert_eq!(
            agora, *n,
            "{slug:?} está tolerado com {n} sítios e a árvore tem {agora} — a entrada já não descreve \
             nada: se a cura aconteceu, APAGUE a linha; se os sítios mudaram, reescreva o número"
        );
    }
}

/// **O varrimento vê a workspace** — os controlos positivos (os pisos estão no próprio `varrer`).
#[test]
fn the_census_sees_the_whole_workspace() {
    let c = censo();
    // Um slug da fundação, um de uma crate de ferramenta, um molde da família vectorial: três
    // directórios diferentes, e nenhum deles se apaga numa limpeza de rotina.
    for slug in [
        "insp_blender_picker",
        "panel.color_equalization",
        "flip.panel",
    ] {
        assert!(
            c.literais.iter().any(|l| l.slug == slug && l.produto),
            "controlo: o slug de produto {slug:?} não foi achado — o varrimento cegou para o sítio dele"
        );
    }
    assert!(
        c.moldes
            .iter()
            .any(|m| m.molde == "vector.texpat.{slot}.{knob:?}"),
        "controlo: o molde `vector.texpat.{{slot}}.{{knob:?}}` não foi achado — o leitor de moldes cegou"
    );
    assert!(
        c.literais.iter().any(|l| !l.produto),
        "controlo: nenhum literal de TESTE — a separação produto/teste não está a separar"
    );
    assert!(
        c.numericos
            .iter()
            .any(|(n, v, _)| n == "HIER_PLAYER" && *v == 400),
        "controlo: o `HIER_PLAYER = NodeId(400)` não foi achado — o leitor de numéricos cegou"
    );
    assert!(
        c.formas
            .iter()
            .any(|(f, func, e)| f.ends_with("screens/hero/live.rs")
                && func == "fold_track"
                && *e == Especie::FnvAMao),
        "controlo: a FNV à mão do `fold_track` não foi achada — o leitor de formas cegou"
    );
}

/// Reserved a11y root NodeId must not be shadowed by any product id — `hash_node_id` defends
/// against it with a `== 0 → 1` fixup, but a hand-numbered const could still hit it.
#[test]
fn no_chrome_id_is_root() {
    let c = censo();
    for l in c.literais.iter().filter(|l| l.produto) {
        assert_ne!(
            h(&l.slug),
            NodeId::ROOT.0,
            "{:?} ({}:{}) é o NodeId::ROOT",
            l.slug,
            l.file,
            l.line
        );
    }
    for (nome, v, file) in &c.numericos {
        assert_ne!(*v, NodeId::ROOT.0, "`{nome}` ({file}) é o NodeId::ROOT");
    }
}

/// Eye/expand companion-bit fixture: no product id may be mistaken for a hierarchy row companion.
/// Companion detection requires BOTH the high bit (61 or 62) set AND the un-masked low portion to
/// fall in the row-id range (< 2^32); a hashed id sets the high bits ~50% of the time, but the
/// residue lands in range with probability < 2^-30 per bit. This asserts the property for EVERY
/// product literal the workspace ships — not only the hand-listed subset the old table carried.
#[test]
fn no_chrome_id_is_companion_misread() {
    let c = censo();
    for l in c.literais.iter().filter(|l| l.produto) {
        let id = NodeId(h(&l.slug));
        assert!(
            ids::hier_eye_companion_to_row(id).is_none(),
            "{:?} ({}:{}, id {:#018x}) is misdetected as an eye-toggle row companion",
            l.slug,
            l.file,
            l.line,
            id.0
        );
        assert!(
            ids::hier_expand_companion_to_row(id).is_none(),
            "{:?} ({}:{}, id {:#018x}) is misdetected as an expand-toggle row companion",
            l.slug,
            l.file,
            l.line,
            id.0
        );
    }
}

/// ⭐ **Um molde de runtime nunca SOLETRA um literal, e dois moldes nunca soletram o mesmo** — a
/// colisão de copiar-colar das famílias derivadas, lida no texto (a amostragem por valores, que os
/// testes de família abaixo fazem, não a vê: ela pergunta a 64 bits).
#[test]
fn a_runtime_template_never_spells_a_literal_slug() {
    // Controlos do casador.
    assert!(casa("vector.fx.{r}.remove", "vector.fx.3.remove"));
    assert!(casa("vector.fx.{r}.remove", "vector.fx.42.remove"));
    assert!(
        !casa("vector.fx.{r}.remove", "vector.fx..remove"),
        "um marcador soletra >= 1 dígito"
    );
    assert!(!casa("vector.fx.{r}.remove", "vector.fx.3.up"));
    assert!(casa("a.{i}", "a.7") && !casa("a.{i}", "b.7") && !casa("a.{i}", "a.7.num"));
    assert!(
        !casa("vector.shape.{index}", "vector.shape.group_dd"),
        "um índice não soletra texto"
    );

    let c = censo();
    let mut soletrados = Vec::new();
    for m in &c.moldes {
        for l in c.literais.iter().filter(|l| l.produto) {
            if casa(&m.molde, &l.slug) {
                soletrados.push(format!(
                    "o literal {:?} ({}:{}) é soletrado pelo molde {:?} ({} :: {})",
                    l.slug, l.file, l.line, m.molde, m.file, m.func
                ));
            }
        }
    }
    for (i, a) in c.moldes.iter().enumerate() {
        for b in c.moldes.iter().skip(i + 1) {
            for v in ["0", "17"] {
                if casa(&b.molde, &exemplar(&a.molde, v)) || casa(&a.molde, &exemplar(&b.molde, v))
                {
                    soletrados.push(format!(
                        "os moldes {:?} ({} :: {}) e {:?} ({} :: {}) soletram a mesma string",
                        a.molde, a.file, a.func, b.molde, b.file, b.func
                    ));
                    break;
                }
            }
        }
    }
    assert!(soletrados.is_empty(), "{}", soletrados.join("\n"));
}

/// ⭐ **Toda forma de id que não é literal nem molde está NOMEADA**, e toda entrada nomeada ainda
/// existe — as duas metades de [`FORMAS_NAO_LITERAIS`].
#[test]
fn every_non_literal_hash_is_named() {
    let c = censo();
    let nomeadas: BTreeSet<(String, String, Especie)> = FORMAS_NAO_LITERAIS
        .iter()
        .map(|(f, func, e, porque)| {
            assert!(
                !porque.trim().is_empty(),
                "{f} :: {func} nomeada sem motivo"
            );
            ((*f).to_owned(), (*func).to_owned(), *e)
        })
        .collect();
    let novas: Vec<String> = c
        .formas
        .difference(&nomeadas)
        .map(|(f, func, e)| format!("{f} :: {func} ({e:?})"))
        .collect();
    assert!(
        novas.is_empty(),
        "formas de NodeId que este censo não sabe cobrir — nomeie cada uma em FORMAS_NAO_LITERAIS com \
         QUEM a cobre (ou troque-a por um literal / um molde, que ele cobre sozinho):\n  {}",
        novas.join("\n  ")
    );
    let obsoletas: Vec<String> = nomeadas
        .difference(&c.formas)
        .map(|(f, func, e)| format!("{f} :: {func} ({e:?})"))
        .collect();
    assert!(
        obsoletas.is_empty(),
        "entradas de FORMAS_NAO_LITERAIS que já não descrevem nada (a função mudou de casa, de nome, ou \
         deixou de hashear) — reescreva-as no sítio novo ou apague-as:\n  {}",
        obsoletas.join("\n  ")
    );
}

/// ⭐⭐ **Um NOME de id, um VALOR** — em toda a workspace de produto, contando os `const` escondidos no
/// corpo de uma função.
///
/// ⛔ O censo de colisões compara SLUGS, e um `const` local com o nome de um id e OUTRO valor passava
/// por ele: o `set_picker_target` subia à frente um `INSP_BLENDER_PICKER = NodeId(380)` (o id da era das
/// faixas, antes do hash de slug) enquanto o seletor é pintado e recebe o clique pelo
/// `hash_node_id("insp_blender_picker")` — abri-lo por cima de outro painel deixava-o POR BAIXO. Achado
/// pela auditoria de fecho da `line/editor-core` (§9 achado 11), curado na integração de 13/09. Medido
/// nesse dia: ESSE era o único nome a divergir no produto (`ID` e `SURFACE` divergem só em testes).
#[test]
fn a_node_id_name_has_one_value() {
    let c = censo();
    let mut por_nome: BTreeMap<&str, BTreeMap<&Valor, BTreeSet<&str>>> = BTreeMap::new();
    for (nome, valor, file) in &c.nomes {
        por_nome
            .entry(nome.as_str())
            .or_default()
            .entry(valor)
            .or_default()
            .insert(file.as_str());
    }
    assert!(
        por_nome.len() >= PISO_NOMES,
        "o censo de nomes achou {} nomes de `const …: NodeId` e esperava >= {PISO_NOMES} — o parser cegou",
        por_nome.len()
    );
    let divergentes: Vec<String> = por_nome
        .iter()
        .filter(|(_, v)| v.len() > 1)
        .map(|(n, v)| format!("{n}: {v:?}"))
        .collect();
    assert!(
        divergentes.is_empty(),
        "nomes de id com MAIS de um valor no produto ({} nomes lidos) — um deles é uma cópia à mão que o \
         desenho e o clique não usam; nomeie a definição única:\n  {}",
        por_nome.len(),
        divergentes.join("\n  ")
    );
}

/// A metade justa do censo de nomes: ele lê um `const` no CORPO de uma função (onde o do seletor se
/// escondia), lê os dois valores possíveis, e não lê prosa.
#[test]
fn the_name_census_reads_a_const_inside_a_function() {
    let src = "fn f() {\n    const X: NodeId = NodeId(380);\n}\n// const Z: NodeId = NodeId(1);\npub const Y: NodeId = hash_node_id(\"a.b\");\n";
    let (com, cod) = limpar(src);
    let achados: Vec<(String, Valor)> = consts_de_node_id(&com, &cod)
        .into_iter()
        .map(|(n, v, _)| (n, v))
        .collect();
    assert_eq!(
        achados,
        vec![
            ("X".to_owned(), Valor::Numero(380)),
            ("Y".to_owned(), Valor::Slug("a.b".to_owned())),
        ]
    );
}

/// W3 audit-2 B.3: the DYNAMIC painter row/blend ids (derived at runtime via
/// `hash_node_id_runtime`) must collide neither with any product literal id nor
/// with each other. A slug-scheme change that aliased a dynamic id onto a chrome
/// id would misroute a production click — the exact failure this file guards,
/// extended to the per-row painter id space.
#[test]
fn painter_dynamic_ids_dont_collide_with_chrome_or_each_other() {
    use ph2d_tool_painter::ids::PainterLayerWidget::{
        Blend, MoveDown, MoveUp, Opacity, OpacityChip, Row, Visibility,
    };

    let chrome = produto_hashes();
    let mut seen: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();

    let kinds = [
        Row,
        Visibility,
        Opacity,
        OpacityChip,
        Blend,
        MoveUp,
        MoveDown,
    ];
    // Dense small ids + sparse/large runtime ids (LayerId is a u64 monotonic).
    let layer_ids = [0u64, 1, 2, 3, 7, 42, 255, 1000, 0x_dead_beef, u64::MAX];

    for &lid in &layer_ids {
        for &kind in &kinds {
            let id = ph2d_tool_painter::ids::painter_layer_widget_id(lid, kind).0;
            assert!(
                !chrome.contains(&id),
                "painter_layer_widget_id({lid}, {kind:?}) (id {id:#018x}) collides with a chrome const",
            );
            assert!(
                seen.insert(id),
                "painter_layer_widget_id({lid}, {kind:?}) (id {id:#018x}) collides with another dynamic id",
            );
        }
        // 22 W3C blend modes today; sample a margin past that.
        for mode in 0u8..28 {
            let id = ph2d_tool_painter::ids::painter_layer_blend_option_id(lid, mode).0;
            assert!(
                !chrome.contains(&id),
                "painter_layer_blend_option_id({lid}, {mode}) (id {id:#018x}) collides with a chrome const",
            );
            assert!(
                seen.insert(id),
                "painter_layer_blend_option_id({lid}, {mode}) (id {id:#018x}) collides with another dynamic id",
            );
        }
    }
}

/// The Flip layers panel's per-row ids (ADR-0114 W2, from `flip_layer_widget_id`)
/// must collide neither with any fixed chrome const nor with each other — same
/// guard the painter dynamic ids get, extended to the Flip per-row id space.
#[test]
fn flip_dynamic_ids_dont_collide_with_chrome_or_each_other() {
    use ph2d_panel_flip::ids::FlipLayerWidget::{
        Blend, Lock, MoveDown, MoveUp, Opacity, Row, Visibility,
    };

    let chrome = produto_hashes();
    let mut seen: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();

    let kinds = [Row, Visibility, Lock, Opacity, Blend, MoveUp, MoveDown];
    let layer_ids = [0u64, 1, 2, 3, 7, 42, 255, 1000, 0x_dead_beef, u64::MAX];

    for &lid in &layer_ids {
        for &kind in &kinds {
            let id = ph2d_panel_flip::ids::flip_layer_widget_id(lid, kind).0;
            assert!(
                !chrome.contains(&id),
                "flip_layer_widget_id({lid}, {kind:?}) (id {id:#018x}) collides with a chrome const",
            );
            assert!(
                seen.insert(id),
                "flip_layer_widget_id({lid}, {kind:?}) (id {id:#018x}) collides with another dynamic id",
            );
        }
        for mode in 0u8..28 {
            let id = ph2d_panel_flip::ids::flip_layer_blend_option_id(lid, mode).0;
            assert!(
                !chrome.contains(&id),
                "flip_layer_blend_option_id({lid}, {mode}) (id {id:#018x}) collides with a chrome const",
            );
            assert!(
                seen.insert(id),
                "flip_layer_blend_option_id({lid}, {mode}) (id {id:#018x}) collides with another dynamic id",
            );
        }
    }
}

/// The timeline's six dynamic id families (key diamonds, row twirls,
/// graph-height grips, curve anchors, bézier handles, Summary columns) share one FNV-1a body,
/// differing only by their domain seed. Prove that a twirl for target `T` never
/// lands on a key hit for target `T` (which a shared seed would have made
/// possible), nor on any chrome const. The anchor family matters most here: one
/// key owns BOTH a diamond and an anchor, keyed by the same `(target, key)`.
#[test]
fn timeline_dynamic_ids_dont_collide_with_chrome_or_each_other() {
    let chrome = produto_hashes();
    let mut seen: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();
    let raw = [0u64, 1, 2, 3, 7, 42, 255, 1000, 0x_dead_beef, u64::MAX];

    let mut check = |what: String, id: NodeId| {
        assert!(
            !chrome.contains(&id.0),
            "{what} (id {:#018x}) collides with a chrome const",
            id.0
        );
        assert!(
            seen.insert(id.0),
            "{what} (id {:#018x}) collides with another dynamic timeline id",
            id.0
        );
    };

    for &target in &raw {
        check(
            format!("timeline_twirl_id({target})"),
            ph2d_panel_timeline::ids::timeline_twirl_id(target),
        );
        check(
            format!("timeline_graph_resize_id({target})"),
            ph2d_panel_timeline::ids::timeline_graph_resize_id(target),
        );
        check(
            format!("timeline_summary_hit_id({target})"),
            ph2d_panel_timeline::ids::timeline_summary_hit_id(target),
        );
        for &key in &raw {
            check(
                format!("timeline_key_hit_id({target}, {key})"),
                ph2d_panel_timeline::ids::timeline_key_hit_id(target, key),
            );
            check(
                format!("timeline_anchor_hit_id({target}, {key})"),
                ph2d_panel_timeline::ids::timeline_anchor_hit_id(target, key),
            );
            for which in 0..2u8 {
                check(
                    format!("timeline_handle_hit_id({target}, {key}, {which})"),
                    ph2d_panel_timeline::ids::timeline_handle_hit_id(target, key, which),
                );
            }
        }
    }
    // Loop-range braces: three singletons (start / end / body), not per-target.
    for edge in 0..3u8 {
        check(
            format!("timeline_loop_brace_id({edge})"),
            ph2d_panel_timeline::ids::timeline_loop_brace_id(edge),
        );
    }
    // Marker pennants, keyed by storage index.
    for index in [0usize, 1, 2, 7, 42, 1000] {
        check(
            format!("timeline_marker_hit_id({index})"),
            ph2d_panel_timeline::ids::timeline_marker_hit_id(index),
        );
    }
}

/// The Vector font-dropdown option ids (one per pickable family, keyed by list
/// index) must not collide with a chrome const nor with each other — a collision
/// would route a family pick to the wrong widget. Mirrors the painter / timeline
/// dynamic-id guards. Indices span dense (small) + sparse (large) family counts.
#[test]
fn vector_dynamic_ids_dont_collide_with_chrome_or_each_other() {
    let chrome = produto_hashes();
    let mut seen: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();
    for index in [0usize, 1, 2, 3, 7, 42, 255, 1000, 100_000] {
        let id = ph2d_panel_vector::ids::vector_text_font_option_id(index).0;
        assert!(
            !chrome.contains(&id),
            "vector_text_font_option_id({index}) (id {id:#018x}) collides with a chrome const",
        );
        assert!(
            seen.insert(id),
            "vector_text_font_option_id({index}) (id {id:#018x}) collides with another dynamic id",
        );
    }
    // Variation-axis fields (index into the current font's non-wght axes).
    for index in 0..ids::MAX_TEXT_VARIATION_AXES {
        let id = ids::vector_text_axis_id(index).0;
        assert!(
            !chrome.contains(&id),
            "vector_text_axis_id({index}) (id {id:#018x}) collides with a chrome const",
        );
        assert!(
            seen.insert(id),
            "vector_text_axis_id({index}) (id {id:#018x}) collides with another dynamic id",
        );
    }
    // As DUAS pilhas por-linha do painel Vector, no MESMO conjunto: a de EFEITOS (geometria,
    // `vector.fx.*`) e a de FILTROS (pixels, `vector.filter.*`). Elas partilham o prefixo
    // `vector.f…`, então "os nomes são diferentes" é uma afirmação que só vale medida — e um
    // conjunto por família a deixaria por provar exatamente onde ela é duvidosa.
    let mut check = |name: String, id: u64| {
        assert!(
            !chrome.contains(&id),
            "{name} (id {id:#018x}) collides with a chrome const",
        );
        assert!(
            seen.insert(id),
            "{name} (id {id:#018x}) collides with another dynamic id",
        );
    };
    for k in 0..ph2d_tool_vector::ids::MAX_FX_KINDS {
        check(
            format!("vector_fx_add_id({k})"),
            ph2d_tool_vector::ids::vector_fx_add_id(k).0,
        );
    }
    for r in 0..ph2d_tool_vector::ids::MAX_FX_ROWS {
        for (label, id) in [
            ("remove", ph2d_tool_vector::ids::vector_fx_remove_id(r)),
            ("up", ph2d_tool_vector::ids::vector_fx_up_id(r)),
            ("down", ph2d_tool_vector::ids::vector_fx_down_id(r)),
            ("card", ph2d_tool_vector::ids::vector_fx_card_id(r)),
            ("hide", ph2d_tool_vector::ids::vector_fx_hide_id(r)),
        ] {
            check(format!("vector_fx_{label}_id({r})"), id.0);
        }
        for prm in 0..ph2d_tool_vector::ids::MAX_FX_ROW_PARAMS {
            for (label, id) in [
                ("param", ph2d_tool_vector::ids::vector_fx_param_id(r, prm)),
                (
                    "param_num",
                    ph2d_tool_vector::ids::vector_fx_param_num_id(r, prm),
                ),
                ("toggle", ph2d_tool_vector::ids::vector_fx_toggle_id(r, prm)),
            ] {
                check(format!("vector_fx_{label}_id({r},{prm})"), id.0);
            }
        }
    }
    for k in 0..ph2d_panel_vector::ids::MAX_FILTER_KINDS {
        check(
            format!("filter_add_id({k})"),
            ph2d_panel_vector::ids::filter_add_id(k).0,
        );
    }
    for r in 0..ids::MAX_FILTER_ROWS {
        for (label, id) in [
            ("card", ph2d_panel_vector::ids::filter_card_id(r)),
            ("remove", ph2d_panel_vector::ids::filter_remove_id(r)),
            ("up", ph2d_panel_vector::ids::filter_up_id(r)),
            ("down", ph2d_panel_vector::ids::filter_down_id(r)),
            ("hide", ph2d_panel_vector::ids::filter_hide_id(r)),
            ("color", ph2d_panel_vector::ids::filter_color_id(r)),
            // ⚠️ **Esta lista tinha APODRECIDO, e a wave da segunda cor a encontrou assim:** as
            // waves da turbulência, da morfologia e do ajuste acrescentaram catorze ids de linha e
            // nenhuma entrou aqui, então o único gate que vigia colisões de id derivado estava
            // cego a metade da seção. Acrescentar só o `color_b` teria continuado a rotina.
            ("color_b", ph2d_panel_vector::ids::filter_color_b_id(r)),
            ("radius", ph2d_panel_vector::ids::filter_radius_id(r)),
            (
                "radius_num",
                ph2d_panel_vector::ids::filter_radius_num_id(r),
            ),
            ("offx", ph2d_panel_vector::ids::filter_offx_id(r)),
            ("offx_num", ph2d_panel_vector::ids::filter_offx_num_id(r)),
            ("offy", ph2d_panel_vector::ids::filter_offy_id(r)),
            ("offy_num", ph2d_panel_vector::ids::filter_offy_num_id(r)),
            ("opacity", ph2d_panel_vector::ids::filter_opacity_id(r)),
            (
                "opacity_num",
                ph2d_panel_vector::ids::filter_opacity_num_id(r),
            ),
            ("blend", ph2d_panel_vector::ids::filter_blend_id(r)),
            ("scale", ph2d_panel_vector::ids::filter_scale_id(r)),
            ("scale_num", ph2d_panel_vector::ids::filter_scale_num_id(r)),
            ("detail", ph2d_panel_vector::ids::filter_detail_id(r)),
            (
                "detail_num",
                ph2d_panel_vector::ids::filter_detail_num_id(r),
            ),
            ("seed", ph2d_panel_vector::ids::filter_seed_id(r)),
            ("seed_num", ph2d_panel_vector::ids::filter_seed_num_id(r)),
            ("grow", ph2d_panel_vector::ids::filter_grow_id(r)),
            ("grow_num", ph2d_panel_vector::ids::filter_grow_num_id(r)),
            ("hue", ph2d_panel_vector::ids::filter_hue_id(r)),
            ("hue_num", ph2d_panel_vector::ids::filter_hue_num_id(r)),
            ("sat", ph2d_panel_vector::ids::filter_sat_id(r)),
            ("sat_num", ph2d_panel_vector::ids::filter_sat_num_id(r)),
            ("bright", ph2d_panel_vector::ids::filter_bright_id(r)),
            (
                "bright_num",
                ph2d_panel_vector::ids::filter_bright_num_id(r),
            ),
        ] {
            check(format!("filter_{label}_id({r})"), id.0);
        }
        for m in 0..ph2d_panel_vector::ids::MAX_FILTER_MODES {
            check(
                format!("filter_mode_id({r},{m})"),
                ph2d_panel_vector::ids::filter_mode_id(r, m).0,
            );
        }
        for m in 0..ph2d_panel_vector::ids::MAX_FILTER_BLENDS {
            check(
                format!("filter_blend_option_id({r},{m})"),
                ph2d_panel_vector::ids::filter_blend_option_id(r, m).0,
            );
        }
    }
}
