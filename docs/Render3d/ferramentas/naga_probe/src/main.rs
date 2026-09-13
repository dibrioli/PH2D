//! Sonda: a ponte GLSL/SPIR-V -> WGSL do naga 29 sobre o OpenPBR gerado pelo MaterialX
//! (`docs/Render3d/04` §2).
//!
//! uso: naga-probe <ficheiro.{frag,vert,spv,wgsl}> [vert|frag] [saida.wgsl]
//!
//! Imprime FRONTEND OK/ERRO, VALIDACAO OK/ERRO e o tamanho do WGSL escrito. ⚠️ Validar no naga
//! prova que o módulo é WGSL LEGAL — nunca que ele sombreia os mesmos pixels que o
//! `MaterialXView`; essa é a régua da W2, e não esta.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let Some(path) = args.get(1) else {
        eprintln!("uso: naga-probe <ficheiro> [vert|frag] [saida.wgsl]");
        return ExitCode::from(2);
    };
    let stage = match args.get(2).map(String::as_str) {
        Some("vert") => naga::ShaderStage::Vertex,
        _ => naga::ShaderStage::Fragment,
    };
    let t0 = std::time::Instant::now();
    let module = if path.ends_with(".spv") {
        let bytes = std::fs::read(path).expect("ler spv");
        match naga::front::spv::parse_u8_slice(&bytes, &naga::front::spv::Options::default()) {
            Ok(m) => m,
            Err(e) => {
                println!("{path}: FRONTEND spv ERRO: {e}");
                return ExitCode::from(1);
            }
        }
    } else if path.ends_with(".wgsl") {
        let src = std::fs::read_to_string(path).expect("ler wgsl");
        match naga::front::wgsl::parse_str(&src) {
            Ok(m) => m,
            Err(e) => {
                println!("{path}: FRONTEND wgsl ERRO:\n{}", e.emit_to_string(&src));
                return ExitCode::from(1);
            }
        }
    } else {
        let src = std::fs::read_to_string(path).expect("ler glsl");
        let mut fe = naga::front::glsl::Frontend::default();
        match fe.parse(&naga::front::glsl::Options::from(stage), &src) {
            Ok(m) => m,
            Err(e) => {
                let first: Vec<String> = e.errors.iter().take(5).map(|x| format!("{x}")).collect();
                println!(
                    "{path}: FRONTEND glsl ERRO ({} erros): {}",
                    e.errors.len(),
                    first.join(" | ")
                );
                return ExitCode::from(1);
            }
        }
    };
    let parse_ms = t0.elapsed().as_secs_f64() * 1e3;
    let info = match naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    {
        Ok(i) => i,
        Err(e) => {
            println!("{path}: FRONTEND OK ({parse_ms:.1} ms) · VALIDACAO ERRO: {e:?}");
            return ExitCode::from(1);
        }
    };
    let wgsl = match naga::back::wgsl::write_string(
        &module,
        &info,
        naga::back::wgsl::WriterFlags::empty(),
    ) {
        Ok(s) => s,
        Err(e) => {
            println!("{path}: FRONTEND OK · VALIDACAO OK · WGSL-OUT ERRO: {e}");
            return ExitCode::from(1);
        }
    };
    println!(
        "{path}: FRONTEND OK ({parse_ms:.1} ms) · VALIDACAO OK · WGSL {} linhas · {} funcoes · {} globais",
        wgsl.lines().count(),
        module.functions.len(),
        module.global_variables.len()
    );
    if let Some(out) = args.get(3) {
        std::fs::write(out, &wgsl).expect("escrever wgsl");
    }
    ExitCode::SUCCESS
}
