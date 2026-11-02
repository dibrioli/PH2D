//! ⭐⭐⭐ **IMPORTAR SVG** — `PH2D_VEC_SVG_SMOKE=1` (estudo 42, item 3).
//!
//! O porquê e a história: `docs/Vector Module/50_cabecalhos_do_app_vec.md` § `smoke_svg.rs`.

/// O desenho da cena. ⚠️ Escrito à mão de propósito: um ficheiro gerado por um editor traria
/// centenas de dígitos e nenhuma das sete perguntas acima ficaria legível ao lado do resultado.
const DESENHO: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="600" height="400" viewBox="0 0 600 400">
  <defs>
    <linearGradient id="rampa" gradientUnits="userSpaceOnUse" x1="40" y1="0" x2="200" y2="0">
      <stop offset="0" stop-color="#e04a2f"/>
      <stop offset="1" stop-color="#f2c14e"/>
    </linearGradient>
  </defs>
  <rect id="barra" x="20" y="250" width="560" height="120" fill="#2f6fb0"/>
  <path id="ponta-para-cima" d="M 300 30 L 380 170 L 220 170 Z" fill="#3aa76d"/>
  <g id="grupo-aninhado" transform="translate(40 40)">
    <g transform="scale(2)">
      <rect id="quadrado" x="0" y="0" width="40" height="40" fill="url(#rampa)"/>
      <rect id="quadradinho" x="50" y="10" width="20" height="20" fill="#3aa76d"/>
    </g>
  </g>
  <path id="curva" d="M 420 40 Q 500 160 580 40" fill="none" stroke="#7a4fd6" stroke-width="10"
        stroke-linecap="round"/>
  <path id="tracejado" d="M 40 200 L 560 200" stroke="#333333" stroke-width="8"
        stroke-dasharray="24 16"/>
  <g id="mistura" opacity="0.75" style="mix-blend-mode:multiply">
    <circle cx="200" cy="310" r="55" fill="#f2c14e"/>
  </g>
  <text x="380" y="320" font-size="40" fill="#ffffff">texto</text>
</svg>
"##;

/// **O roteador desta cena** — lê a `PH2D_VEC_SVG_SMOKE`.
///
/// ⚠️ **Ela é o QUINTO roteador desta família e quase não foi contado**: o ficheiro que a lê
/// chama-se `svg_import_smoke.rs` e **não** `vec_*`, logo um censo por PREFIXO de nome não o vê.
/// *A unidade da posse é o assunto, não o nome do ficheiro* — a mesma forma da §2.7 do HOWTO,
/// um nível acima.
#[must_use]
pub fn armed() -> bool {
    std::env::var_os("PH2D_VEC_SVG_SMOKE").is_some()
}

/// **O CORPO da cena.** O prólogo — o `gfx` existir, a memória «já montou?», e a régua de
/// píxeis-por-metro, que sai do `hero_screen` — fica na shell.
pub fn build(
    sim: &mut ph2d_ecs::SimWorld,
    scene: &mut ph2d_vec_scene::VecScene,
    map: &mut ph2d_vec_entities::entity_map::VecEntityMap,
    ppm: f32,
) {
    let caminho = std::env::temp_dir().join("ph2d_smoke_desenho.svg");
    if let Err(e) = std::fs::write(&caminho, DESENHO) {
        eprintln!(
            "[vec-svg-smoke] nao consegui escrever {}: {e}",
            caminho.display()
        );
        return;
    }
    // ⚠️ A MESMA função que o arrastar-e-largar chama. Um atalho aqui mediria outro programa.
    match crate::svg_import::import_svg(sim, scene, map, &caminho, [0.0, 0.0], ppm) {
        crate::svg_import::SvgImportResult::Ok {
            name,
            shapes,
            size,
            notes,
            ..
        } => {
            eprintln!(
                "[vec-svg-smoke] {} entrou: {shapes} formas, {:.2} x {:.2} unidades de mundo. \
                 Ficheiro em {} — arraste-o para a janela para o importar outra vez.",
                name,
                size[0],
                size[1],
                caminho.display()
            );
            for n in &notes {
                eprintln!("[vec-svg-smoke] NAO entrou (nomeado): {n}");
            }
            if notes.is_empty() {
                eprintln!(
                    "[vec-svg-smoke] ATENCAO: o ficheiro tem um <text> e nada foi nomeado — \
                     o aviso do importador esta' partido."
                );
            }
        }
        crate::svg_import::SvgImportResult::Err { name, error } => {
            eprintln!("[vec-svg-smoke] {name} RECUSADO: {error}");
        }
    }
}
