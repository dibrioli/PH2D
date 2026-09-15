//! ⭐⭐⭐ **UMA linha de N campos tem UMA porta — em TODA a workspace.**
//!
//! ⛔⛔ **Report do dono, 2026-09-15, com o desenho:** *«A disposição ficou diferente. VC tinha
//! colocado x e y na mesma linha. Position X/Y Caixa Caixa»*. O Transform tinha sido posto na linha
//! de propriedade **com um pintor próprio**, e as duas portas concordavam na forma e **divergiam nos
//! números**: o vão entre caixas (`8` contra `3`) e uma coluna própria para a letra de eixo (`14`).
//! No dock dele (`369,74`) a coluna do controlo mede `160,9`: as Âncoras pediam `147` e cabiam; o
//! Transform pedia `180` e **não cabia**.
//!
//! ⇒ *«a mesma formatação» não se obtém com duas portas que concordam — obtém-se com UMA.*
//!
//! # ⚠️ Porque este gate mudou de casa, e de âmbito
//!
//! Ele nasceu no `ph2d-panel-inspector` e varria `src/sections/`. Em 2026-09-15 a porta mudou-se
//! para o `ph2d-editor-core` (sete painéis pintam campos e nenhum lá passava), e o gate **reprovou
//! a ler zero chamadores** — *um censo ancorado num DIRECTÓRIO escapa a quem muda o código de
//! sítio*. ⇒ ele varre agora **todas as crates**, que é o âmbito da lei.

use std::fs;
use std::path::{Path, PathBuf};

/// A raiz `crates/` da workspace.
fn crates_dir() -> PathBuf {
    // ⚠️ O `CARGO_MANIFEST_DIR` desta crate É `…/crates/ph2d-editor-core` ⇒ o pai é o `crates/`.
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn varre(dir: &Path, achados: &mut Vec<String>) {
    let Ok(entradas) = fs::read_dir(dir) else {
        return;
    };
    for e in entradas.flatten() {
        let p = e.path();
        if p.is_dir() {
            // ⛔ `target/` e `tests/` fora: o produto é `src/`, e um gate que se lê a si mesmo
            //    conta-se (o defeito `a_census_gate_that_scans_its_own_tree_counts_itself`).
            let nome = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if nome == "target" || nome == "tests" {
                continue;
            }
            varre(&p, achados);
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            let Ok(src) = fs::read_to_string(&p) else {
                continue;
            };
            // ⛔ A DECLARAÇÃO não é uma chamada — sem isto o gate acusa a própria lei.
            let chama = src.lines().any(|l| {
                l.contains("property_fields_layout(") && !l.contains("fn property_fields_layout(")
            });
            if chama {
                achados.push(
                    p.strip_prefix(crates_dir())
                        .unwrap_or(&p)
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }
}

#[test]
fn only_one_door_lays_out_a_row_of_fields() {
    let mut chamadores = Vec::new();
    varre(&crates_dir(), &mut chamadores);
    chamadores.sort();
    assert_eq!(
        chamadores,
        vec!["ph2d-editor-core/src/property_row.rs".to_string()],
        "a lei de dispor N campos numa linha tem de ter UM chamador \
         (`property_row::paint_fields_row`). Quem a chama por sua conta volta a ter os numeros \
         dele, e dois paineis a' mesma largura desenham coisas diferentes — foi o report do dono \
         de 2026-09-15."
    );
}
