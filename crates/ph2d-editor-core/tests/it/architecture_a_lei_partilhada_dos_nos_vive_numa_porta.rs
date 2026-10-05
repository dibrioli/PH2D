//! **A lei partilhada dos nós vive numa PORTA** (`ph2d-motion-kit`), nunca em N cópias que se
//! prometem iguais.
//!
//! Bug #11 (`docs/Motion Nodes/BUGS_motion_nodes.md`): o `hash3`, o `cos_sin_cycles` e a
//! acumulação das forças viviam copiados em 38 crates, cada cópia a dizer que «o vocabulário
//! partilhado é o comportamento» — e a cura do `rot` foi digitada seis vezes. Dois portões:
//!
//! - (a) nenhum grupo de `.rs` com mais de 1 KB e conteúdo BYTE-IGUAL em `crates/*/src` e
//!   `shells/*/src` (o recenso `md5sum | uniq` que fechou o bug, agora gateado);
//! - (b) nenhuma definição de `fn hash3(`, `fn cos_sin_cycles(` ou `fn sin_cycles(` fora de
//!   `crates/ph2d-motion-kit/` — a cópia que diverge num comentário escapa a (a), não a (b).
//!
//! Os kernels WGSL que espelham estas leis usam nomes prefixados (`em_hash3`, `bend_sin_cycles`)
//! e não casam com (b) de propósito: o que os prende à porta é o gate de paridade de cada nó.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

/// `find -size +1k`: estritamente mais de 1024 bytes.
const MIN_BYTES: usize = 1024;
/// As leis que só a porta define.
const LEIS: [&str; 3] = ["hash3", "cos_sin_cycles", "sin_cycles"];
const PORTA: &str = "crates/ph2d-motion-kit/";
/// `(caminho a partir da raiz, lei, porquê)`. Vazia: toda cópia conhecida foi fundida. Uma
/// entrada nova traz o PORQUÊ, e o portão reprova a que já não existir (censo de obsolescência).
const EXCECOES: &[(&str, &str, &str)] = &[];

fn raiz() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rel(raiz: &Path, p: &Path) -> String {
    p.strip_prefix(raiz)
        .unwrap_or(p)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Todo `.rs` sob `dir`, recursivo, sem `target/` nem pastas escondidas.
fn rs_sob(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let nome = e.file_name();
        let nome = nome.to_string_lossy();
        if p.is_dir() {
            if nome != "target" && !nome.starts_with('.') {
                rs_sob(&p, out);
            }
        } else if nome.ends_with(".rs") {
            out.push(p);
        }
    }
}

/// `crates/*/src/**.rs` e `shells/*/src/**.rs` — a população do recenso.
fn fontes_src(raiz: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for topo in ["crates", "shells"] {
        let Ok(rd) = fs::read_dir(raiz.join(topo)) else {
            continue;
        };
        for e in rd.flatten() {
            rs_sob(&e.path().join("src"), &mut out);
        }
    }
    out.sort();
    out
}

/// Todo `.rs` sob `crates/` e `shells/` (src, tests, benches…): uma cópia da lei num teste
/// também é uma cópia.
fn fontes_todas(raiz: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for topo in ["crates", "shells"] {
        rs_sob(&raiz.join(topo), &mut out);
    }
    out.sort();
    out
}

/// Este ficheiro NOMEIA as leis (na prosa e no controlo do detector) sem as definir.
const ESTE: &str = "architecture_a_lei_partilhada_dos_nos_vive_numa_porta.rs";

/// As leis que `src` DEFINE: `fn <lei>(` precedido de nada que seja identificador, fora de
/// linhas de comentário (uma definição comentada não é definição).
fn leis_definidas(src: &str) -> BTreeSet<&'static str> {
    let mut achadas = BTreeSet::new();
    for linha in src.lines().filter(|l| !l.trim_start().starts_with("//")) {
        for lei in LEIS {
            let agulha = format!("fn {lei}(");
            for (i, _) in linha.match_indices(&agulha) {
                let antes = linha[..i].chars().next_back();
                if !antes.is_some_and(|c| c.is_alphanumeric() || c == '_') {
                    achadas.insert(lei);
                }
            }
        }
    }
    achadas
}

#[test]
fn nenhum_rs_maior_que_1kb_e_byte_igual_a_outro() {
    let raiz = raiz();
    let fontes = fontes_src(&raiz);
    assert!(
        fontes.len() > 1000,
        "o controlo: o recenso tem de VER a árvore — varreu só {} ficheiros sob {}",
        fontes.len(),
        raiz.display()
    );
    let mut grupos: BTreeMap<(usize, u64), Vec<PathBuf>> = BTreeMap::new();
    for p in &fontes {
        let bytes = fs::read(p).unwrap_or_default();
        if bytes.len() > MIN_BYTES {
            let mut h = DefaultHasher::new();
            bytes.hash(&mut h);
            grupos
                .entry((bytes.len(), h.finish()))
                .or_default()
                .push(p.clone());
        }
    }
    let mut copias = Vec::new();
    for membros in grupos.values().filter(|m| m.len() > 1) {
        // O hash só agrupa; a igualdade é a dos BYTES.
        let primeiro = fs::read(&membros[0]).unwrap_or_default();
        let iguais: Vec<String> = membros
            .iter()
            .filter(|p| fs::read(p).unwrap_or_default() == primeiro)
            .map(|p| rel(&raiz, p))
            .collect();
        if iguais.len() > 1 {
            copias.push(iguais.join(" = "));
        }
    }
    assert!(
        copias.is_empty(),
        "{} grupo(s) de ficheiros byte-iguais — a lei partilhada mora numa porta \
         (ph2d-motion-kit ou a crate-folha da família), não numa cópia:\n  {}",
        copias.len(),
        copias.join("\n  ")
    );
}

#[test]
fn as_leis_partilhadas_so_se_definem_na_porta() {
    let raiz = raiz();
    let fontes = fontes_todas(&raiz);
    assert!(
        fontes.len() > 1000,
        "o controlo: o portão tem de VER a árvore — varreu só {} ficheiros",
        fontes.len()
    );
    let mut na_porta = BTreeSet::new();
    let mut fora = Vec::new();
    for p in fontes.iter().filter(|p| !p.ends_with(ESTE)) {
        let src = fs::read_to_string(p).unwrap_or_default();
        let r = rel(&raiz, p);
        for lei in leis_definidas(&src) {
            if r.starts_with(PORTA) {
                na_porta.insert(lei);
            } else if !EXCECOES.iter().any(|(c, l, _)| *c == r && *l == lei) {
                fora.push(format!("{r}: fn {lei}("));
            }
        }
    }
    assert_eq!(
        na_porta,
        LEIS.into_iter().collect::<BTreeSet<_>>(),
        "o controlo positivo: o portão tem de ver as três leis DEFINIDAS na porta"
    );
    assert!(
        fora.is_empty(),
        "a lei partilhada definida fora do `{PORTA}` — use `ph2d_motion_kit::{{hash, trig}}` \
         (ou acrescente a EXCECOES com o porquê):\n  {}",
        fora.join("\n  ")
    );
}

#[test]
fn cada_excecao_ainda_define_a_lei() {
    let raiz = raiz();
    let obsoletas: Vec<String> = EXCECOES
        .iter()
        .filter(|(c, lei, _)| {
            let src = fs::read_to_string(raiz.join(c)).unwrap_or_default();
            !leis_definidas(&src).contains(lei)
        })
        .map(|(c, lei, porque)| format!("{c}: fn {lei}( — «{porque}»"))
        .collect();
    assert!(
        obsoletas.is_empty(),
        "exceção que já não existe (censo de obsolescência — apague-a da lista):\n  {}",
        obsoletas.join("\n  ")
    );
}

#[test]
fn o_detector_de_definicao_ve_a_lei_e_nao_o_espelho_prefixado() {
    // O controlo do predicado: casa a definição, não o WGSL prefixado nem uma chamada.
    let src = "pub fn hash3(a: u32) {}\nfn em_hash3(a: u32) {}\nlet x = cos_sin_cycles(0.1);";
    assert_eq!(leis_definidas(src), BTreeSet::from(["hash3"]));
    assert!(leis_definidas("fn bend_sin_cycles(p: f32) -> f32").is_empty());
    assert_eq!(
        leis_definidas("    fn sin_cycles(phase: f32) -> f32 {"),
        BTreeSet::from(["sin_cycles"])
    );
    assert!(leis_definidas("// fn hash3(a: u32) — a prosa nomeia, não define").is_empty());
}
