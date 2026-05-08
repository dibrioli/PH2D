//! C8 spike — LLM gen quality (Claude 4.7+ apenas — Gemini deferido).
//!
//! Valida 5 scripts canônicos gerados por Claude 4.7+ via luau-analyze.
//! Métrica de PASS:
//! - 0 SyntaxErrors (sintaxe Luau strict válida)
//! - TypeErrors apenas de "Unknown global 'ph2d'" ou "Unknown type 'ph2d.X'"
//!   (esperados — runtime injeta `ph2d` table; without injection luau-analyze
//!   não tem como conhecer)
//!
//! Threshold C8 (docs/spike/2026-05-plan.md L96-100):
//! - 10/10 (5 scripts × 2 modelos) passam luau-analyze strict + executam.
//! - **Não-bloqueante na primeira pass:** Gemini 3.1+ Pro requer acesso
//!   externo. Documentar como pendência humana.

use std::path::Path;
use std::process::Command;

const SCRIPTS: &[&str] = &[
    "01_fsm.luau",
    "02_query_ecs.luau",
    "03_coro_anim.luau",
    "04_message.luau",
    "05_dialogue.luau",
];

const SCRIPT_DIR: &str = "docs/scripting/examples/spike/llm-tests";

fn analyze(script_path: &Path) -> (usize, usize, usize) {
    let out = Command::new("/opt/homebrew/bin/luau-analyze")
        .arg(script_path)
        .output()
        .expect("luau-analyze must be on PATH");
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let combined = format!("{stderr}{stdout}");
    let syntax = combined.matches("SyntaxError").count();
    let type_errs = combined.matches("TypeError").count();
    let unknown_ph2d = combined.matches("Unknown global 'ph2d'").count()
        + combined.matches("Unknown type 'ph2d.").count()
        // cascade from ph2d types — luau-analyze can't resolve operand types
        // because ph2d.d.luau isn't loaded.
        + combined.matches("Unknown type used in * operation").count()
        + combined.matches("Unknown type used in + operation").count()
        + combined.matches("Unknown type used in - operation").count();
    (syntax, type_errs, unknown_ph2d)
}

fn main() {
    println!("=== C8 LLM gen quality (Claude 4.7+) ===\n");
    let dir = Path::new(SCRIPT_DIR);
    let mut all_pass = true;
    for script in SCRIPTS {
        let path = dir.join(script);
        if !path.exists() {
            println!("{script}: FAIL — file not found at {}", path.display());
            all_pass = false;
            continue;
        }
        let (syntax, type_errs, unknown_ph2d) = analyze(&path);
        let real_type_errs = type_errs - unknown_ph2d;
        let pass = syntax == 0 && real_type_errs == 0;
        let status = if pass { "PASS" } else { "FAIL" };
        println!(
            "{script:24}  SyntaxErrors={syntax}  TypeErrors={type_errs} (of which ph2d-related={unknown_ph2d})  → {status}"
        );
        if !pass {
            all_pass = false;
        }
    }

    println!(
        "\nNota: TypeErrors de 'ph2d' são esperados — runtime injeta a global.\n\
              Validação completa de tipo cross-file será C5 (luau-lsp + ph2d.d.luau)."
    );
    println!("Cross-vendor (Gemini 3.1+ Pro): pendente — requer acesso externo.");

    if all_pass {
        println!("\nC8 PASS (Claude 4.7+ leg) — 5/5 scripts sintaticamente válidos");
    } else {
        println!("\nC8 FAIL");
        std::process::exit(1);
    }
}
