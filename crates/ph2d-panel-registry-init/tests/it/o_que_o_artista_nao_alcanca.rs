//! ⭐⭐⭐ **O NOME DE UM CONTROLO — o mapa inverso `NodeId` → *slug*, partilhado pelos censos.**
//!
//! Um censo que diz *«esta fileira cai abaixo da dobra»* ou *«este nome tapa este controlo»* só
//! manda a wave para o sítio certo se **nomear** o controlo. Uma régua que agrega diz que há dívida;
//! só uma que nomeia diz o que fazer com ela.
//!
//! # ⭐⭐ O nome é DERIVADO, nunca uma lista
//!
//! Um `NodeId` de widget é o FNV-1a de um *slug* ([`ph2d_tool_registry::hash_node_id`]), logo o
//! mapa inverso constrói-se lendo os literais `hash_node_id("…")` das crates que os declaram e
//! re-hashando cada um. ⛔ Nada aqui é uma lista de nomes escrita à mão — e o que o varrimento não
//! souber nomear aparece como `(sem nome)` no censo que o usa.
//!
//! ⚠️ **Piso de população no próprio varrimento** (HOWTO §2.7): um extractor que deixasse de casar
//! devolveria o mapa vazio, **toda** linha leria `(sem nome)` e o censo continuaria a imprimir uma
//! tabela — *que é a forma exacta de um instrumento partido se ler como um produto limpo*.
//!
//! ⚠️ A sonda que aqui vivia (o painel da escultura fileira a fileira, com a dobra marcada) saiu com
//! o 3D (ADR-0179); o extractor fica, e é dos censos que o chamam.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ph2d_editor_core::NodeId;

fn raiz(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn varre(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        panic!("nao consegui ler {}", dir.display());
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            varre(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// Os *slugs* de `hash_node_id("…")` de um ficheiro.
fn slugs_de(src: &str) -> Vec<String> {
    // ⚠️ A agulha monta-se: este ficheiro é lido por um extractor irmão, e um censo textual que se
    //    lê a si mesmo encontra sempre o que procura. Aqui ela só custa uma linha.
    let abre = concat!("hash_node_id", "(\"");
    let mut out = Vec::new();
    let mut resto = src;
    while let Some(i) = resto.find(abre) {
        resto = &resto[i + abre.len()..];
        match resto.find('"') {
            Some(j) => {
                out.push(resto[..j].to_string());
                resto = &resto[j..];
            }
            None => break,
        }
    }
    out
}

/// ⭐⭐ **O extractor, sobre as árvores que a PERGUNTA pede.**
///
/// ⚠️ A lista de fontes e o piso são **por pergunta** — cada censo chamador declara os seus.
/// ⛔ O que NÃO se duplica é o [`slugs_de`]: *uma segunda cópia do FNV-1a seria a segunda resposta
/// a «que id é este?»*.
pub(crate) fn nomes_de(fontes: &[&str], piso: usize) -> BTreeMap<NodeId, String> {
    let mut ficheiros = Vec::new();
    for f in fontes {
        varre(&raiz(f), &mut ficheiros);
    }
    let mut mapa = BTreeMap::new();
    for f in &ficheiros {
        let src = std::fs::read_to_string(f).unwrap_or_default();
        for s in slugs_de(&src) {
            mapa.insert(ph2d_editor_core::registry::hash_node_id_runtime(&s), s);
        }
    }
    assert!(
        mapa.len() >= piso,
        "o varrimento leu {} slugs e esperava >= {piso} — o extractor cegou, e um mapa \
         vazio faz esta sonda imprimir `(sem nome)` em toda linha",
        mapa.len()
    );
    mapa
}
