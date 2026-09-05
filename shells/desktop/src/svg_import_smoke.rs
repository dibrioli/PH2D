//! ⭐⭐⭐ **IMPORTAR SVG** — `PH2D_VEC_SVG_SMOKE=1` (estudo 42, item 3).
//!
//! # O que a cena fecha
//!
//! Até 2026-09-05 este app **exportava** uma curva e não sabia **ler** nenhuma: o
//! `ph2d-imageio-svg` validava o ficheiro e devolvia um documento vazio, *"intentionally empty"*.
//! Nenhum acervo de artista entrava.
//!
//! # ⭐ A cena ESCREVE o próprio `.svg`
//!
//! É a lei do smoke do `.ase`: testar o importador não pode depender de o operador ter um ficheiro
//! à mão. Ele é escrito no directório temporário, o caminho é impresso (para o Enio o poder
//! **arrastar outra vez**, ou abrir noutro programa e comparar) e depois entra pela **mesma porta**
//! que o arrastar-e-largar usa — ⛔ não por um atalho de teste.
//!
//! # O que o ficheiro carrega, e porquê cada coisa
//!
//! | No ficheiro | O que prova |
//! |---|---|
//! | Um triângulo com a ponta em CIMA | a lei dos eixos (o SVG mede o Y ao contrário do mundo) |
//! | **Dois** quadrados dentro de dois `<g transform>` | a pose ANINHADA chega certa, e o `<g id>` vira um GRUPO na Hierarquia |
//! | Uma curva `Q` (quadrática) | a elevação a cúbica é exacta |
//! | Um `<linearGradient>` | a rampa viaja com a geometria |
//! | Um traço `stroke-dasharray` | o tracejado entra em múltiplos da largura |
//! | Um `<g opacity mix-blend-mode>` sobre uma barra | a opacidade e a mistura da v19 do schema |
//! | Um `<text>` | ⛔ o que NÃO entra sai NOMEADO, e não em silêncio |
//!
//! ⚠️ **O `grupo-aninhado` tem DOIS filhos de propósito.** O verbo de agrupar desta casa exige dois
//! membros (a mesma regra que o artista lê no menu), então um `<g>` com um filho só é achatado — e
//! a cena não mostraria grupo nenhum na Hierarquia, que é metade do que ela existe para provar.

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

impl crate::App {
    /// No prólogo do frame, uma vez. No-op sem a env.
    pub(crate) fn svg_import_smoke(&mut self) {
        if self.svg_import_smoke_done || std::env::var_os("PH2D_VEC_SVG_SMOKE").is_none() {
            return;
        }
        if self.gfx.is_none() {
            return; // sem mundo ainda; tenta no próximo frame
        }
        self.svg_import_smoke_done = true;

        let caminho = std::env::temp_dir().join("ph2d_smoke_desenho.svg");
        if let Err(e) = std::fs::write(&caminho, DESENHO) {
            eprintln!(
                "[vec-svg-smoke] nao consegui escrever {}: {e}",
                caminho.display()
            );
            return;
        }
        let map = &mut self.vec_entities;
        let gfx = self.gfx.as_mut().expect("gfx");
        let ppm = gfx
            .hero_screen
            .as_ref()
            .map_or(ph2d_editor::DEFAULT_PIXELS_PER_METER, |h| {
                h.project.pixels_per_meter
            });
        // ⚠️ A MESMA função que o arrastar-e-largar chama. Um atalho aqui mediria outro programa.
        match crate::svg_import::import_svg(
            &mut gfx.sim,
            &mut gfx.vec_scene,
            map,
            &caminho,
            [0.0, 0.0],
            ppm,
        ) {
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
}
