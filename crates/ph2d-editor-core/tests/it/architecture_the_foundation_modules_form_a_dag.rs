//! ⭐⭐ **Os módulos de topo da `ph2d-editor-core` formam um DAG** — auditoria de arquitectura
//! 2026-09-12, A10.
//!
//! # O defeito, medido
//!
//! A fundação é o maior leque do repo: **57** crates dependem dela (a auditoria contava 43), e uma
//! edição de UMA linha na `src/` recompila **61** (medido 2026-09-12, `cargo test --no-run --workspace
//! --profile ci-test`). Parti-la em crates é a cura óbvia — e é **inexprimível** enquanto os módulos
//! dependerem uns dos outros nos DOIS sentidos: um módulo que usa outro e é usado por ele não vira
//! crate própria sem um ciclo de dependências. A auditoria mediu três pares (`widget ↔ interaction`
//! 73/165, `screens ↔ interaction` 162/22, `interaction ↔ ids` 90/10); medido pela ÁRVORE de módulos,
//! depois da descida dos ids (A5b), era **UM ciclo de 23 módulos com 1 188 referências** — e o
//! `interaction ↔ ids` já tinha morrido com ela.
//!
//! As curas por ASSUNTO desta auditoria (um TIPO desce para o módulo dono do conceito, uma LEI desce
//! para quem a corre, uma TABELA é injectada por quem chama; ⛔ nunca um re-export) partiram as
//! arestas pequenas: os três ficheiros do `action_bus` passaram a filhos dele, os gates régua×porta
//! voltaram para a régua, o pintor da fila de avisos mudou-se para o `toast`, e os pintores de texto
//! cortado para o `paint`. Medido depois delas: **UM ciclo de 14 módulos com 835 referências**, que
//! as sete arestas da catraca fecham — cada uma é necessária sozinha. O que sobra está lá, com o
//! número e a cura escrita.
//!
//! # O que conta como aresta
//!
//! O leitor mora em [`crate::foundation_module_tree`] (dep-free, pelo tecto de LOC), e o doc dele
//! lista as formas que vê e as que NÃO vê. Em resumo:
//!
//! - **O módulo de topo de um ficheiro** lê-se da ÁRVORE, a partir do `lib.rs` e seguindo `#[path]` —
//!   nunca do nome do ficheiro: o `motion_tests.rs` é do `motion`, o `action_bus_queue.rs` do
//!   `action_bus`.
//! - **Uma referência** é `crate::X`, cada `X` de `crate::{X, …}`, e uma cadeia `super::…::X` (ou
//!   `super::…::{X, …}`) que sobe até à raiz — só em CÓDIGO (comentários e literais em branco).
//! - **Os testes contam.** Um `#[cfg(test)]` que chama o módulo de cima também impede o corte: numa
//!   crate partida, esse teste teria de ver a crate de cima, e a de cima dependeria dele.
//! - **Uma re-exportação do `lib.rs` vê-se através, em TODA forma** (qualquer visibilidade, atributos
//!   à frente, `crate::` à frente, grupos aninhados): `crate::Toast` é aresta para o `toast`. ⚠️ Sem
//!   isto, esconder uma aresta atrás de um `use` na raiz passava — que é exactamente a cura que esta
//!   auditoria proíbe. ⛔ E uma GLOB de módulo de topo na raiz REPROVA: ela esconde arestas que o
//!   leitor não sabe nomear.
//!
//! # A catraca
//!
//! `(de, para, tecto de referências, porquê)`. Só ENCOLHE: acima do tecto reprova (a aresta cresceu);
//! abaixo dele reprova também (desça o número — senão a folga é licença para voltar a crescer); e uma
//! entrada que já não fecha ciclo nenhum reprova (apague a linha). ⭐ **Alvo: catraca VAZIA.**

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::foundation_module_tree::{arvore, reexportacoes, referencias};

/// ⛔ **A catraca — só encolhe.** Tectos medidos em 2026-09-12, com este leitor.
const ARESTAS_TOLERADAS: &[(&str, &str, usize, &str)] = &[
    (
        "widget",
        "interaction",
        49,
        "o SUBSTRATO que os pintores recebem — `HitIndex`, `WidgetStore`, `InteractiveState`, \
         `WidgetEvent`. Cura: o estado de widget (os `*State` + `InteractiveState` + `HitIndex` + \
         `WidgetStore`) desce para um módulo ABAIXO dos pintores, e o `dispatch` fica em cima.",
    ),
    (
        "interaction",
        "screens",
        18,
        "os tipos de DOCA que o estado guarda (`DockSide`, `TaskLayout`, `Slot`, `ChromeBands`, \
         `TIMELINE_DOCK_H`) e três tabelas do hero que o despacho consulta (`panel_for_tab`, \
         `MENUS`, `LEGACY_PILL_MENUS`). Cura: os tipos descem para um módulo de doca abaixo do \
         `interaction`; as tabelas são injectadas pelo hero. ⚠️ `Slot`/`SlotSet` têm 144 leitores \
         em 86 ficheiros fora da fundação.",
    ),
    (
        "action_bus",
        "screens",
        24,
        "os PAYLOADS do Inspector (`*FieldEdit`, `*Info`, `HierReparentIntent`) moram em \
         `screens::hero::inspector_model*`. Cura: descem para um módulo de vocabulário abaixo do \
         `action_bus`. ⛔ Nesta rodada a cerca da `line/render-loop` nomeia quatro deles pelo \
         caminho `screens::hero::` — descem depois das duas fusões.",
    ),
    (
        "panel",
        "screens",
        7,
        "o contrato do painel nomeia a doca (`slot::{Slot, SlotSet}`) e o hero (`HeroSelection`, \
         `HeroLayout`). Cura: os tipos de doca da entrada `interaction → screens`; a selecção e o \
         layout passam ao contrato como os tipos de baixo que eles são.",
    ),
    (
        "motion",
        "interaction",
        4,
        "só em teste: `motion_surface_tests.rs` e `motion_grid_tests.rs` montam um `WidgetStore`. \
         Cura: os testes sobem para o `interaction` — o `motion_grid_tests` lê privados do `motion` \
         (`Track`, `value`), que precisam de porta antes.",
    ),
    (
        "motion",
        "screens",
        1,
        "só em teste: `motion_grid_tests.rs` monta o `HeroScreen`. A mesma cura da entrada acima.",
    ),
    (
        "motion",
        "widget",
        1,
        "só em teste: um teste do `motion_tests.rs` pinta um `Button`. Cura: ele sobe para o `widget`.",
    ),
];

/// A metade justa — medido 2026-09-12: 32 módulos de topo, 448 ficheiros na árvore, 1 615 referências.
const PISO_MODULOS: usize = 30;
const PISO_FICHEIROS: usize = 420;
const PISO_REFERENCIAS: usize = 1_500;
/// ⚠️ **A metade justa da leitura ATRAVÉS da raiz** — medido 2026-09-12: 4 referências só viram aresta
/// por um `pub use` do `lib.rs` (`crate::LengthDisplay`, no `gizmo/readout_tests.rs`). Nenhuma delas
/// muda o veredito de hoje, e foi por isso que a mutação que desligava a leitura através SOBREVIVEU
/// à primeira prova: sem este piso, o leitor podia deixar de ver através de uma fachada na raiz e
/// continuar verde.
const PISO_VIA_REEXPORT: usize = 1;

fn src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// `(de, para) → (referências, sítios de exemplo)`.
type Arestas = BTreeMap<(String, String), (usize, Vec<String>)>;

struct Grafo {
    tops: BTreeSet<String>,
    ficheiros: usize,
    arestas: Arestas,
    via_reexport: usize,
    globs_na_raiz: Vec<String>,
}

fn grafo() -> Grafo {
    let src = src_root();
    let arv = arvore(&src);
    let tops: BTreeSet<String> = arv.iter().filter_map(|(_, c)| c.first().cloned()).collect();
    let lib = std::fs::read_to_string(src.join("lib.rs")).expect("lib.rs");
    let (reexp, globs_na_raiz) = reexportacoes(&lib, &tops);
    let mut arestas = Arestas::new();
    let mut via_reexport = 0;
    for (f, caminho) in &arv {
        let Some(de) = caminho.first() else { continue };
        let texto = std::fs::read_to_string(f).unwrap_or_default();
        for (at, nome) in referencias(&texto, caminho) {
            // Um nome da raiz que também é módulo continua a ser o módulo.
            let (para, atraves) = match reexp.get(&nome) {
                Some(m) if !tops.contains(&nome) => (m.clone(), true),
                _ => (nome, false),
            };
            if tops.contains(&para) && &para != de {
                via_reexport += usize::from(atraves);
                let e = arestas.entry((de.clone(), para)).or_default();
                e.0 += 1;
                if e.1.len() < 3 {
                    let rel = f.strip_prefix(&src).unwrap_or(f).display().to_string();
                    e.1.push(format!("{rel}:{}", texto[..at].matches('\n').count() + 1));
                }
            }
        }
    }
    Grafo {
        tops,
        ficheiros: arv.len(),
        arestas,
        via_reexport,
        globs_na_raiz,
    }
}

/// Um ciclo no grafo sem as arestas de `fora`, se houver.
fn um_ciclo(
    tops: &BTreeSet<String>,
    arestas: &Arestas,
    fora: &BTreeSet<(String, String)>,
) -> Option<Vec<String>> {
    let mut adj: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (a, b) in arestas.keys() {
        if !fora.contains(&(a.clone(), b.clone())) {
            adj.entry(a).or_default().push(b);
        }
    }
    fn dfs<'a>(
        v: &'a str,
        adj: &BTreeMap<&'a str, Vec<&'a str>>,
        cor: &mut BTreeMap<&'a str, u8>,
        pilha: &mut Vec<&'a str>,
    ) -> Option<Vec<String>> {
        cor.insert(v, 1);
        pilha.push(v);
        for &w in adj.get(v).map_or(&[][..], Vec::as_slice) {
            match cor.get(w) {
                Some(1) => {
                    let k = pilha.iter().position(|x| *x == w).unwrap_or(0);
                    let mut c: Vec<String> = pilha[k..].iter().map(|s| (*s).to_owned()).collect();
                    c.push(w.to_owned());
                    return Some(c);
                }
                None => {
                    if let Some(c) = dfs(w, adj, cor, pilha) {
                        return Some(c);
                    }
                }
                _ => {}
            }
        }
        pilha.pop();
        cor.insert(v, 2);
        None
    }
    let mut cor = BTreeMap::new();
    for v in tops {
        if !cor.contains_key(v.as_str())
            && let Some(c) = dfs(v, &adj, &mut cor, &mut Vec::new())
        {
            return Some(c);
        }
    }
    None
}

fn toleradas() -> BTreeSet<(String, String)> {
    ARESTAS_TOLERADAS
        .iter()
        .map(|(a, b, _, _)| ((*a).to_owned(), (*b).to_owned()))
        .collect()
}

#[test]
fn the_foundation_modules_form_a_dag() {
    let g = grafo();
    let total: usize = g.arestas.values().map(|(n, _)| n).sum();
    assert!(
        g.tops.len() >= PISO_MODULOS,
        "li {} módulos de topo (piso {PISO_MODULOS})",
        g.tops.len()
    );
    assert!(
        g.ficheiros >= PISO_FICHEIROS,
        "a árvore alcançou {} ficheiros (piso {PISO_FICHEIROS})",
        g.ficheiros
    );
    assert!(
        total >= PISO_REFERENCIAS,
        "li {total} referências entre módulos (piso {PISO_REFERENCIAS})"
    );
    assert!(
        g.via_reexport >= PISO_VIA_REEXPORT,
        "li {} referências através de uma re-exportação do lib.rs (piso {PISO_VIA_REEXPORT}) — \
         o leitor deixou de ver através da raiz, e uma fachada lá passaria a esconder uma aresta",
        g.via_reexport
    );
    assert!(
        g.globs_na_raiz.is_empty(),
        "o lib.rs põe na raiz uma GLOB de módulo de topo ({:?}) — ela esconde arestas que o leitor \
         não sabe nomear: nomeie os itens um a um (ou, melhor, não os ponha na raiz)",
        g.globs_na_raiz
    );
    if let Some(ciclo) = um_ciclo(&g.tops, &g.arestas, &toleradas()) {
        let passos: Vec<String> = ciclo
            .windows(2)
            .map(|p| {
                let (n, ex) = &g.arestas[&(p[0].clone(), p[1].clone())];
                format!("{} → {} ({n}: {})", p[0], p[1], ex.join(", "))
            })
            .collect();
        panic!(
            "os módulos de topo da fundação voltaram a formar um CICLO:\n  {}\n\n\
             cura (a da auditoria A10): leve o que se usa do outro lado para o módulo do ASSUNTO — um \
             tipo para o módulo dono do conceito, uma lei para quem a corre, uma tabela injectada por \
             quem chama. ⛔ Nunca um re-export: uma fachada é a mesma aresta com outro nome.",
            passos.join("\n  ")
        );
    }
    // A tabela medida — a `--success-output` do nextest mostra-a, e é ela que o handoff cola.
    println!(
        "módulos {} · ficheiros {} · referências {total} · através de re-exportação {}",
        g.tops.len(),
        g.ficheiros,
        g.via_reexport
    );
    for ((a, b), (n, _)) in &g.arestas {
        let volta = g.arestas.get(&(b.clone(), a.clone())).map_or(0, |x| x.0);
        if volta > 0 {
            println!("  {a} → {b}: {n} (volta {volta})");
        }
    }
    let cresceram: Vec<String> = ARESTAS_TOLERADAS
        .iter()
        .filter_map(|(a, b, tecto, _)| {
            let n = g
                .arestas
                .get(&((*a).to_owned(), (*b).to_owned()))
                .map_or(0, |x| x.0);
            (n > *tecto).then(|| format!("{a} → {b}: {n} referências, tecto {tecto}"))
        })
        .collect();
    assert!(
        cresceram.is_empty(),
        "arestas TOLERADAS que CRESCERAM — a catraca só encolhe:\n  {}",
        cresceram.join("\n  ")
    );
}

#[test]
fn the_ratchet_only_describes_what_is_still_true() {
    let g = grafo();
    let todas = toleradas();
    for (a, b, tecto, porque) in ARESTAS_TOLERADAS {
        assert!(!porque.trim().is_empty(), "`{a} → {b}` tolerada sem motivo");
        let n = g
            .arestas
            .get(&((*a).to_owned(), (*b).to_owned()))
            .map_or(0, |x| x.0);
        assert!(
            n > 0,
            "`{a} → {b}` está na catraca e já não é aresta — apague a linha"
        );
        assert!(
            n >= *tecto,
            "`{a} → {b}` desceu para {n} referências e a catraca diz {tecto} — desça o número"
        );
        let mut com_ela = todas.clone();
        com_ela.retain(|e| e != &((*a).to_owned(), (*b).to_owned()));
        assert!(
            um_ciclo(&g.tops, &g.arestas, &com_ela).is_some(),
            "`{a} → {b}` já não fecha ciclo nenhum — ela deixou de ser dívida: apague a linha"
        );
    }
}

fn nomes(src: &str, caminho: &[&str]) -> Vec<String> {
    let c: Vec<String> = caminho.iter().map(|s| (*s).to_owned()).collect();
    referencias(src, &c).into_iter().map(|(_, n)| n).collect()
}

/// ⚠️ **E o leitor sabe dizer «não» — e «sim».**
#[test]
fn the_reader_sees_what_it_claims_to_see() {
    // Uma aresta que desce, e que tem de estar lá: sem ela o leitor partiu-se e mede nada.
    let g = grafo();
    for (a, b) in [
        ("widget", "paint"),
        ("screens", "interaction"),
        ("toast", "progress"),
    ] {
        assert!(
            g.arestas.contains_key(&(a.to_owned(), b.to_owned())),
            "sentinela: o leitor não viu `{a} → {b}`"
        );
    }
    // Comentários e strings não contam; grupos e cadeias de `super` contam.
    let src = "// crate::screens::x\nlet s = \"crate::widget\";\nuse crate::{paint::A, zones::{B, C}};\nfn f() { super::super::motion::g(); }\n";
    assert_eq!(nomes(src, &["a", "b"]), ["paint", "zones", "motion"]);
    // Dentro de um `mod tests { … }` inline, a raiz fica um `super` mais longe.
    let src = "mod tests {\n    use super::super::widget::W;\n    use super::paint::P;\n}\n";
    assert_eq!(nomes(src, &["a"]), ["widget"]);
}

/// ⚠️ **As formas que a auditoria de fecho (2026-09-12) achou CEGAS.** Nenhuma aparecia na árvore
/// desse dia — e é exactamente por isso que cada uma tem sentinela: no dia em que aparecer, o leitor
/// tem de a ver, e um leitor que não a vê fica verde.
#[test]
fn the_reader_sees_the_forms_the_closing_audit_named() {
    // Um GRUPO depois de uma cadeia de `super` que chega à raiz.
    assert_eq!(
        nomes("use super::{widget::W, paint};\n", &["a"]),
        ["widget", "paint"]
    );
    // Uma GLOB da raiz faz o caminho solto contar — e só no escopo dela.
    let src =
        "use super::*;\nfn f() { widget::W::new(); }\nmod tests {\n    fn g() { zones::Z; }\n}\n";
    assert_eq!(nomes(src, &["a"]), ["widget"]);
    let src = "mod tests {\n    use crate::*;\n    fn g() { paint::P; }\n}\nfn f() { zones::Z; }\n";
    assert_eq!(nomes(src, &["a"]), ["paint"]);
    // `'\''` fecha no SEU fecho: a aspa que vem a seguir não abre um literal falso que engula código.
    let src = "const Q: [char; 2] = ['\\'','\"'];\nuse crate::paint::P;\nconst S: &str = \"x\";\n";
    assert_eq!(nomes(src, &["a"]), ["paint"]);
    // Um literal cru de BYTES com uma aspa dentro não sai no meio dela.
    let src = "const B: &[u8] = br#\"x\" crate::widget \"#;\nuse crate::paint::P;\nconst S: &str = \"x\";\n";
    assert_eq!(nomes(src, &["a"]), ["paint"]);

    // As re-exportações da raiz em toda forma — e o que está dentro de um `mod { }` não é da raiz.
    let tops: BTreeSet<String> = [
        "toast", "progress", "zones", "paint", "widget", "screens", "ruler",
    ]
    .iter()
    .map(|s| (*s).to_owned())
    .collect();
    let lib = "pub(crate) use toast::A;\n#[doc(inline)]\npub use progress::B;\npub use crate::zones::C;\n\
               use paint::{D, text::{E as F}};\npub use ruler::{self as regua};\npub use widget::*;\n\
               pub mod m {\n    pub use screens::G;\n}\npub use ph2d_x as y;\n";
    let (raiz, globs) = reexportacoes(lib, &tops);
    let esperado: BTreeMap<String, String> = [
        ("A", "toast"),
        ("B", "progress"),
        ("C", "zones"),
        ("D", "paint"),
        ("F", "paint"),
        ("regua", "ruler"),
    ]
    .iter()
    .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
    .collect();
    assert_eq!(raiz, esperado);
    assert_eq!(globs, ["widget"]);

    // A árvore: um doc-comment e uma linha em branco entre o `#[path]` e o `mod`, e `mod x;` dentro
    // de um `mod { }` inline (com e sem `#[path]`).
    let src = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("dag_leitor_arvore")
        .join("src");
    let _ = std::fs::remove_dir_all(&src);
    for (f, texto) in [
        (
            "lib.rs",
            "#[path = \"longe/a_impl.rs\"]\n/// um doc-comment no meio\n\nmod a;\nmod b {\n    mod c;\n    #[path = \"d_impl.rs\"]\n    mod d;\n}\n",
        ),
        ("longe/a_impl.rs", ""),
        ("b/c.rs", ""),
        ("b/d_impl.rs", ""),
    ] {
        let p = src.join(f);
        std::fs::create_dir_all(p.parent().expect("dir")).expect("tmp");
        std::fs::write(&p, texto).expect("tmp");
    }
    let mut vistos: Vec<(String, Vec<String>)> = arvore(&src)
        .into_iter()
        .map(|(f, c)| {
            let rel = f.strip_prefix(&src).expect("dentro").display().to_string();
            (rel, c)
        })
        .collect();
    vistos.sort();
    let caminho = |s: &[&str]| s.iter().map(|x| (*x).to_owned()).collect::<Vec<_>>();
    assert_eq!(
        vistos,
        vec![
            ("b/c.rs".to_owned(), caminho(&["b", "c"])),
            ("b/d_impl.rs".to_owned(), caminho(&["b", "d"])),
            ("lib.rs".to_owned(), caminho(&[])),
            ("longe/a_impl.rs".to_owned(), caminho(&["a"])),
        ]
    );
}
