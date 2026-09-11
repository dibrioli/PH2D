#![forbid(unsafe_code)]
//! **Codegen do `ph2d-app-registry-init`** — varre `crates/ph2d-app-*` e renderiza os QUATRO
//! blocos gerados: o corpo de `register_all_app_families`, e os `[dependencies]`, `default` e
//! `[features]` do `Cargo.toml`.
//!
//! Espelho do `ph2d-panel-sync`, com **duas** diferenças de forma que são decisões, não acidentes:
//!
//! 1. **Toda família entra no `default`.** Os painéis são opt-out de propósito; uma família fora do
//!    `default` é a recusa medida da auditoria §9 (a feature `smokes` que esconde as cenas do gate
//!    e do CI **em silêncio**). Aqui a feature existe para bissectar, nunca para esconder.
//! 2. **A família exporta uma `const FAMILY`**, não um tipo — quem descreve a família é um valor,
//!    e é ele que o registo empurra. ⚠️ O `ph2d-panel-sync` faz o contrário (parse do nome de um
//!    `pub struct *Panel`), e é por isso que ele precisa de um espelho de contagem escrito à mão —
//!    o espelho que derivou durante quase um mês. Uma `const` com nome fixo não tem o que derivar.
//!
//! Std puro, orientado a linha (sem `syn`), como os três `*-sync` irmãos.

use std::path::Path;

pub const RS_BEGIN: &str = "// <ph2d-app-sync:begin>";
pub const RS_END: &str = "// <ph2d-app-sync:end>";
pub const TOML_DEPS_BEGIN: &str = "# <ph2d-app-sync:deps:begin>";
pub const TOML_DEPS_END: &str = "# <ph2d-app-sync:deps:end>";
pub const TOML_DEFAULT_BEGIN: &str = "# <ph2d-app-sync:default:begin>";
pub const TOML_DEFAULT_END: &str = "# <ph2d-app-sync:default:end>";
pub const TOML_FEATURES_BEGIN: &str = "# <ph2d-app-sync:features:begin>";
pub const TOML_FEATURES_END: &str = "# <ph2d-app-sync:features:end>";

/// Varre `crates_dir` por crates de família (`ph2d-app-*`), **excluindo** os dois agregadores.
///
/// ⚠️ `ph2d-app-host` e `ph2d-app-registry-init` partilham o prefixo e **não** são famílias — o
/// primeiro é a interface, o segundo é este registo. Uma varredura por prefixo que os apanhasse
/// poria o registo a depender de si próprio.
#[must_use]
pub fn scan_app_crates(crates_dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(crates_dir)
        .expect("crates dir readable")
        .filter_map(Result::ok)
        .filter(|e| e.path().join("Cargo.toml").is_file())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| {
            n.starts_with("ph2d-app-") && n != "ph2d-app-host" && n != "ph2d-app-registry-init"
        })
        .collect();
    names.sort();
    names
}

/// `ph2d-app-field3d` → `field3d`.
#[must_use]
pub fn family_key(crate_name: &str) -> String {
    crate_name
        .strip_prefix("ph2d-app-")
        .expect("scan_app_crates filtra pelo prefixo")
        .to_string()
}

/// `ph2d-app-field3d` → `app-field3d`.
#[must_use]
pub fn feature_slug(crate_name: &str) -> String {
    format!("app-{}", family_key(crate_name))
}

/// `ph2d-app-field3d` → `ph2d_app_field3d`.
#[must_use]
pub fn crate_ident(crate_name: &str) -> String {
    crate_name.replace('-', "_")
}

#[must_use]
pub fn render_rs(crates: &[String]) -> String {
    crates
        .iter()
        .map(|c| {
            format!(
                "    #[cfg(feature = \"{}\")]\n    reg.push({}::FAMILY);\n",
                feature_slug(c),
                crate_ident(c)
            )
        })
        .collect()
}

#[must_use]
pub fn render_deps(crates: &[String]) -> String {
    crates
        .iter()
        .map(|c| format!("{c} = {{ path = \"../{c}\", optional = true }}\n"))
        .collect()
}

#[must_use]
pub fn render_default(crates: &[String]) -> String {
    crates
        .iter()
        .map(|c| format!("    \"{}\",\n", feature_slug(c)))
        .collect()
}

#[must_use]
pub fn render_features(crates: &[String]) -> String {
    crates
        .iter()
        .map(|c| format!("{} = [\"dep:{c}\"]\n", feature_slug(c)))
        .collect()
}

/// Substitui a região entre `begin` e `end` (exclusivos) por `body`.
///
/// ⚠️ Devolve `None` quando um marcador falta **ou está fora de ordem** — um `rewrite` que
/// silenciosamente não escrevesse deixaria o gate de staleness a comparar o ficheiro consigo
/// próprio e a passar.
#[must_use]
pub fn rewrite(src: &str, begin: &str, end: &str, body: &str) -> Option<String> {
    let b = src.find(begin)?;
    let e = src.find(end)?;
    if e < b {
        return None;
    }
    let after_begin = b + begin.len();
    // engole a quebra de linha que segue o marcador de abertura
    let body_start = src[after_begin..].find('\n').map(|i| after_begin + i + 1)?;
    Some(format!("{}{}{}", &src[..body_start], body, &src[e..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_family_renders_all_four_blocks() {
        let c = vec!["ph2d-app-field3d".to_string()];
        assert!(render_rs(&c).contains("ph2d_app_field3d::FAMILY"));
        assert!(render_deps(&c).contains("path = \"../ph2d-app-field3d\""));
        assert!(render_default(&c).contains("\"app-field3d\","));
        assert_eq!(
            render_features(&c),
            "app-field3d = [\"dep:ph2d-app-field3d\"]\n"
        );
    }

    /// ⛔ Os dois agregadores partilham o prefixo e NÃO são famílias.
    #[test]
    fn the_two_aggregators_are_not_families() {
        assert_eq!(family_key("ph2d-app-field3d"), "field3d");
        // a varredura filtra-os pelo nome; aqui prova-se que o filtro é sobre os DOIS
        for nome in ["ph2d-app-host", "ph2d-app-registry-init"] {
            assert!(
                nome.starts_with("ph2d-app-"),
                "se {nome} deixar de casar o prefixo, o filtro da varredura deixa de ter sujeito"
            );
        }
    }

    #[test]
    fn a_rewrite_with_markers_out_of_order_refuses() {
        let src = "// <ph2d-app-sync:end>\nx\n// <ph2d-app-sync:begin>\n";
        assert!(rewrite(src, RS_BEGIN, RS_END, "novo\n").is_none());
    }

    #[test]
    fn a_rewrite_replaces_only_between_the_markers() {
        let src = "antes\n// <ph2d-app-sync:begin>\nvelho\n// <ph2d-app-sync:end>\ndepois\n";
        let out = rewrite(src, RS_BEGIN, RS_END, "novo\n").expect("marcadores presentes");
        assert_eq!(
            out,
            "antes\n// <ph2d-app-sync:begin>\nnovo\n// <ph2d-app-sync:end>\ndepois\n"
        );
    }
}
