//! ⭐ **A escultura VOLTA com o arquivo** — o documento guarda o **nome**, e é aqui que o nome
//! volta a ser campo ([ADR-0161], plano W5; a W22 escreveu a entrada, esta fecha o ciclo).
//!
//! # O defeito que este arquivo existe para matar
//!
//! A W22 pôs uma escultura dentro da peça e a W21 fez a booleana cortá-la. O que nenhuma das duas
//! tinha é o **regresso**: `NodeKind::Sampled` guarda o caminho do arquivo (12 bytes em vez de 12 MB
//! de grade), o mundo é salvo e desfeito com ele — e o **registo** que traduz nome → campo é
//! memória de processo. Fechar o app apaga-o.
//!
//! Reabrir o projeto trazia então a Hierarquia inteira, com o nó da escultura no sítio certo, e:
//!
//! | onde a escultura estava | o que aparecia |
//! |---|---|
//! | sozinha, ou numa união | **nada** — o nó existe, a linha está na Hierarquia, a tela é vazia |
//! | como base de uma subtração | **nada**, pela mesma conta |
//! | numa **interseção** | ⚠️ **a peça INTEIRA some** — `max(a, ABSENT)` é `ABSENT` |
//!
//! …e nenhum dos três dizia uma palavra. *Um nome que não resolve lê como espaço vazio* (a decisão
//! está certa e é do `hybrid::ABSENT`: o oposto encheria a cena de um bloco que ninguém autorizou) —
//! mas ler como vazio **em silêncio** é a metade que faltava.
//!
//! # As duas leis desta wave
//!
//! 1. ⭐ **Recarregar é a MESMA função que importar** ([`crate::field3d_import::field_from_file`]).
//!    O que volta do arquivo tem de ser o que entrou por ele: uma segunda cópia — outra resolução,
//!    ou sem o `recenter` — daria uma peça que muda de forma ao reabrir, e nada na tela a dizê-lo.
//! 2. ⭐ **O que não resolve FALA.** O arquivo pode ter sido movido, renomeado, ou estar noutra
//!    máquina — e aí a resposta certa continua a ser espaço vazio, mas com o **nome do arquivo** num
//!    aviso. Sem isso o artista vê uma peça errada e conclui que o app perdeu o trabalho dele.
//!
//! # ⚠️ Uma tentativa por nome, e é o que impede um aviso por QUADRO
//!
//! A reconciliação corre no cozimento, que corre a cada quadro. Um caminho que falha falha sempre
//! (o arquivo não volta sozinho), então tentar de novo custaria uma leitura de disco falhada por
//! quadro **e** um aviso novo por quadro — a tela ficaria inutilizável exactamente no caso em que o
//! artista precisa de a ler. O conjunto [`TRIED`] guarda o que já foi tentado.
//!
//! [ADR-0161]: ../../../docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md

use ph2d_field::{FieldDoc, NodeKind};
use ph2d_field_eval::hybrid::Registry;

/// **As esculturas que o documento nomeia e o registo não conhece**, em ordem e sem repetições.
///
/// ⚠️ Função pura, e é ela que carrega a decisão: o documento é a **pergunta** (que nomes esta peça
/// precisa) e o registo é a **resposta parcial**. Separá-la do disco é o que a torna gateável sem
/// arquivo nenhum.
pub(crate) fn missing_keys(doc: &FieldDoc, reg: &Registry) -> Vec<String> {
    let mut out: Vec<String> = doc
        .nodes()
        .iter()
        .filter_map(|n| match &n.kind {
            NodeKind::Sampled { key } if !reg.contains_key(key) => Some(key.clone()),
            _ => None,
        })
        .collect();
    // ⚠️ Duas cópias da mesma escultura (um `duplicate` da subárvore) nomeiam a MESMA chave: sem o
    // dedup, o arquivo seria lido duas vezes e o aviso sairia duplicado.
    out.sort();
    out.dedup();
    out
}

/// Resolve o que falta lendo os arquivos, e devolve **o que não deu** — uma mensagem por escultura.
///
/// ⚠️ Ela regista o que consegue e continua: uma peça com duas esculturas em que só uma sumiu tem de
/// abrir com a que existe. *Recusar tudo por causa de uma perderia o resto do trabalho.*
pub(crate) fn resolve_missing(doc: &FieldDoc) -> Vec<String> {
    let reg = crate::field3d_smoke::sampled_registry();
    let mut failed = Vec::new();
    for key in missing_keys(doc, &reg) {
        if !first_try(&key) {
            continue;
        }
        let path = std::path::Path::new(&key);
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(key.as_str());
        match crate::field3d_import::field_from_file(path) {
            Ok(loaded) => {
                crate::field3d_smoke::register_sampled(&key, std::sync::Arc::new(loaded.field));
            }
            // ⚠️ O **nome do arquivo** na frente: é o que o artista reconhece, e é o que ele tem de
            // ir procurar. O caminho inteiro fica no log.
            Err(e) => {
                eprintln!("[field3d] escultura ausente: {key} — {e}");
                failed.push(format!("Sculpture {name} is missing: {e}"));
            }
        }
    }
    failed
}

/// `true` na **primeira** vez que este nome é tentado. Ver a nota do módulo.
fn first_try(key: &str) -> bool {
    TRIED.with(|t| t.borrow_mut().insert(key.to_string()))
}

thread_local! {
    /// Nomes já tentados nesta sessão — o que falhou não se repete.
    ///
    /// ⚠️ **`BTreeSet` e não `HashSet`**: `HashMap`/`HashSet` são tipos proibidos neste repositório
    /// (lint estrutural do determinismo, HR-5).
    static TRIED: std::cell::RefCell<std::collections::BTreeSet<String>> =
        const { std::cell::RefCell::new(std::collections::BTreeSet::new()) };
}

/// ⚠️ **Só para gates**: esquece o que já foi tentado, para que dois gates no mesmo processo não se
/// contaminem pela ordem em que correram.
#[cfg(test)]
pub(crate) fn forget_tried() {
    TRIED.with(|t| t.borrow_mut().clear());
}

#[cfg(test)]
#[path = "field3d_reload_tests.rs"]
mod tests;
