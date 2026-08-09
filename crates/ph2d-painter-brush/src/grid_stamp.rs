//! **Grid Stamp** — a geometria da grade própria do método (`StrokeMethod::GridStamp`).
//!
//! Uma pergunta, uma porta. O motor precisa saber **que raio** um dab de célula tem para emiti-lo; o
//! tool precisa do **footprint inteiro** (raio + achatamento + ângulo) para carimbá-lo esticado até a
//! borda da célula; e o overlay precisa das **linhas**. Se cada um derivasse o seu, o carimbo, a linha
//! desenhada e a região que o undo grava discordariam sobre onde a célula está — e o artista veria a
//! tinta encostar num lado e sobrar do outro.
//!
//! ⚠️ **A elipse é o que um footprint sabe ser.** O deform de um dab é rotação + compressão do eixo
//! menor ([`crate::FootprintDeform`]), então "esticar até caber na célula" é, para um falloff redondo,
//! a **elipse inscrita** — e, para uma imagem de Shape, o quadrado do carimbo mapeado no retângulo da
//! célula, que é exatamente o que o pedido diz. A razão de aspecto exprimível vai até
//! **1 / (1 − [`crate::DAB_FLATTEN_MAX`]) = 20:1**; além disso a célula é mais fina do que um dab
//! consegue ser, e o carimbo para de encolher (limite de REPRESENTAÇÃO, não de gosto — e é por isso
//! que ele está escrito aqui em vez de virar um clamp mudo no slider).

use crate::spec::BrushSpec;

/// O piso de uma célula, em px de imagem. Abaixo de um pixel a grade deixa de ter interior e o
/// "centro da célula" não é mais um lugar distinto de suas bordas.
pub const GRID_CELL_MIN_PX: f32 = 1.0;

impl BrushSpec {
    /// A célula desta grade, saneada (nunca zero, nunca negativa, nunca NaN).
    #[must_use]
    pub fn grid_cell(&self) -> [f32; 2] {
        [sane(self.grid_cell_px[0]), sane(self.grid_cell_px[1])]
    }

    /// O índice da célula que cobre um ponto de imagem. Piso, não arredondamento: a célula `0` é a
    /// que vai de `offset` a `offset + cell`, e um ponto na linha da grade pertence à célula da
    /// DIREITA/ABAIXO — a mesma convenção de meio-aberto `[a, b)` que o resto do canvas usa.
    #[must_use]
    pub fn grid_cell_at(&self, p: [f32; 2]) -> [i32; 2] {
        let c = self.grid_cell();
        [
            ((p[0] - self.grid_offset_px[0]) / c[0]).floor() as i32,
            ((p[1] - self.grid_offset_px[1]) / c[1]).floor() as i32,
        ]
    }

    /// O **centro** de uma célula, em px de imagem — onde o carimbo pousa.
    #[must_use]
    pub fn grid_cell_center(&self, cell: [i32; 2]) -> [f32; 2] {
        let c = self.grid_cell();
        [
            self.grid_offset_px[0] + (cell[0] as f32 + 0.5) * c[0],
            self.grid_offset_px[1] + (cell[1] as f32 + 0.5) * c[1],
        ]
    }

    /// O footprint de um carimbo de grade: **`(raio, achatamento, ângulo em graus)`**, derivado da
    /// CÉLULA e não do tamanho do pincel.
    ///
    /// O eixo maior vira o raio; o menor é obtido comprimindo por `1 − f`. Quando a célula é mais
    /// alta que larga o eixo maior é o **vertical**, e é o ângulo de 90° que o diz — girar o frame é
    /// como um footprint exprime "a compressão é no outro eixo".
    ///
    /// ⚠️ O `flatten` é **clampado** em [`crate::DAB_FLATTEN_MAX`]: numa célula mais estreita que
    /// 1:20 o carimbo para de afinar. A alternativa seria um dab degenerado (eixo menor zero), que
    /// não é pintável.
    #[must_use]
    pub fn grid_stamp_frame(&self) -> (f32, f32, u16) {
        let [w, h] = self.grid_cell();
        let (major, minor, angle) = if w >= h { (w, h, 0) } else { (h, w, 90) };
        let flatten = (1.0 - minor / major).clamp(0.0, crate::DAB_FLATTEN_MAX);
        (major * 0.5, flatten, angle)
    }

    /// Este spec, reescrito para carimbar UMA célula: raio, achatamento e ângulo saem da grade.
    ///
    /// É a porta que o tool usa antes de estampar — assim o dab que o motor emitiu (cujo `radius_px`
    /// veio de [`Self::grid_stamp_frame`]) e a silhueta que o sampler avalia falam do mesmo retângulo.
    #[must_use]
    pub fn as_grid_stamp(&self) -> Self {
        let (radius, flatten, angle) = self.grid_stamp_frame();
        Self {
            radius_px: radius,
            dab_flatten: flatten,
            dab_angle_deg: angle,
            ..*self
        }
    }
}

fn sane(v: f32) -> f32 {
    if v.is_finite() {
        v.max(GRID_CELL_MIN_PX)
    } else {
        GRID_CELL_MIN_PX
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(cell: [f32; 2], off: [f32; 2]) -> BrushSpec {
        BrushSpec {
            grid_cell_px: cell,
            grid_offset_px: off,
            ..BrushSpec::default()
        }
    }

    /// A célula e o centro dela são **inversos**: o centro de uma célula tem de cair de volta nela.
    /// Sem isso o carimbo pousa numa célula e a próxima amostra o lê como "célula nova".
    #[test]
    fn the_cell_and_its_centre_are_inverses() {
        for off in [[0.0, 0.0], [7.5, -3.25], [-40.0, 12.0]] {
            for cell in [[32.0, 32.0], [24.0, 8.0], [5.0, 90.0]] {
                let s = spec(cell, off);
                for cx in -3..=3 {
                    for cy in -3..=3 {
                        let c = [cx, cy];
                        assert_eq!(
                            s.grid_cell_at(s.grid_cell_center(c)),
                            c,
                            "cell {c:?} cell={cell:?} off={off:?}"
                        );
                    }
                }
            }
        }
    }

    /// Um ponto **na linha** da grade pertence à célula seguinte (meio-aberto), e um ponto um épsilon
    /// antes pertence à anterior — a fronteira não pode ser ambígua, senão a mesma posição carimba
    /// duas células conforme o arredondamento do dia.
    #[test]
    fn the_grid_line_belongs_to_the_cell_after_it() {
        let s = spec([20.0, 20.0], [0.0, 0.0]);
        assert_eq!(s.grid_cell_at([20.0, 20.0]), [1, 1]);
        assert_eq!(s.grid_cell_at([19.999, 19.999]), [0, 0]);
        assert_eq!(s.grid_cell_at([-0.001, -0.001]), [-1, -1]);
    }

    /// O footprint é o da CÉLULA: numa célula quadrada não há achatamento; numa deitada o eixo maior
    /// é o horizontal (ângulo 0) e numa em pé é o vertical (ângulo 90).
    #[test]
    fn the_footprint_is_the_cell_not_the_brush_size() {
        let (r, f, a) = spec([40.0, 40.0], [0.0; 2]).grid_stamp_frame();
        assert_eq!((r, f, a), (20.0, 0.0, 0), "quadrada: sem achatamento");

        let (r, f, a) = spec([40.0, 10.0], [0.0; 2]).grid_stamp_frame();
        assert_eq!(a, 0, "deitada: eixo maior horizontal");
        assert!((r - 20.0).abs() < 1e-6);
        assert!((f - 0.75).abs() < 1e-6, "menor = 10 = 40*(1-0.75)");

        let (r, f, a) = spec([10.0, 40.0], [0.0; 2]).grid_stamp_frame();
        assert_eq!(
            a, 90,
            "em pé: eixo maior VERTICAL — o ângulo é quem diz isso"
        );
        assert!((r - 20.0).abs() < 1e-6);
        assert!((f - 0.75).abs() < 1e-6);
    }

    /// ⚠️ O limite de REPRESENTAÇÃO, com o número medido ao lado: além de 1:20 o carimbo para de
    /// afinar porque um eixo menor zero não é pintável. Um gate, não um comentário.
    #[test]
    fn the_aspect_the_stamp_can_honour_tops_out_at_twenty_to_one() {
        let (_, f, _) = spec([200.0, 10.0], [0.0; 2]).grid_stamp_frame();
        assert!((f - 0.95).abs() < 1e-6, "1:20 é exatamente o teto");
        let (_, f, _) = spec([2000.0, 10.0], [0.0; 2]).grid_stamp_frame();
        assert!((f - 0.95).abs() < 1e-6, "1:200 não afina mais que 1:20");
    }

    /// Uma célula degenerada (zero, negativa, NaN) não pode produzir divisão por zero nem um índice
    /// sem sentido — o piso é um pixel, que é onde "centro" deixa de ser distinto de "borda".
    #[test]
    fn a_degenerate_cell_falls_back_to_one_pixel() {
        for bad in [[0.0, 32.0], [-8.0, 32.0], [f32::NAN, 32.0]] {
            let s = spec(bad, [0.0; 2]);
            assert_eq!(s.grid_cell()[0], GRID_CELL_MIN_PX);
            let c = s.grid_cell_at([10.0, 10.0]);
            assert!(c[0].abs() < 100_000, "índice finito: {c:?}");
        }
    }

    // ── A emissão: o motor a percorrer a grade ────────────────────────────────────────────────────

    fn grid_stroke(cell: [f32; 2]) -> crate::Stroke {
        let s = BrushSpec {
            stroke_method: crate::StrokeMethod::GridStamp,
            ..spec(cell, [0.0, 0.0])
        };
        crate::Stroke::new(s, crate::Dynamics::default(), 0)
    }

    fn pt(x: f32, y: f32) -> crate::StrokePoint {
        crate::StrokePoint {
            pos: [x, y],
            pressure: 1.0,
        }
    }

    /// **A lei do método**: o carimbo pousa no CENTRO da célula, não onde o cursor está. Um clique em
    /// qualquer canto de uma célula produz o mesmo dab.
    #[test]
    fn the_stamp_lands_at_the_cell_centre_wherever_in_the_cell_you_press() {
        let mut out = Vec::new();
        let mut centres = Vec::new();
        for p in [(1.0, 1.0), (19.0, 3.0), (10.0, 18.0)] {
            let mut st = grid_stroke([20.0, 20.0]);
            st.begin(pt(p.0, p.1), &mut out);
            assert_eq!(out.len(), 1, "o pen-down carimba a célula sob o cursor");
            centres.push(out[0].center);
        }
        assert!(
            centres.iter().all(|c| *c == [10.0, 10.0]),
            "três pressões na mesma célula, um único centro: {centres:?}"
        );
    }

    /// **Uma vez por célula, não uma por evento.** O ponteiro treme dentro de uma célula; sem esta lei
    /// cada micro-movimento re-depositaria, e com Strength < 1 a célula escureceria sozinha.
    #[test]
    fn jitter_inside_one_cell_stamps_once() {
        let mut out = Vec::new();
        let mut st = grid_stroke([20.0, 20.0]);
        st.begin(pt(10.0, 10.0), &mut out);
        assert_eq!(out.len(), 1);
        let mut extra = 0;
        for d in [0.5f32, -0.7, 1.3, -1.1, 0.2] {
            st.extend(pt(10.0 + d, 10.0 - d), &mut out);
            extra += out.len();
        }
        assert_eq!(extra, 0, "tremer dentro da célula não re-carimba");
    }

    /// **Um arrasto rápido não deixa buracos.** UM salto de dez células tem de carimbar as dez — é o
    /// que o passo de meia célula compra, e é o defeito que um "carimba onde o evento caiu" teria.
    #[test]
    fn a_fast_drag_leaves_no_gap() {
        let mut out = Vec::new();
        let mut st = grid_stroke([20.0, 20.0]);
        st.begin(pt(10.0, 10.0), &mut out);
        let mut seen = vec![out[0].center];
        st.extend(pt(210.0, 10.0), &mut out); // 10 células de uma vez
        seen.extend(out.iter().map(|d| d.center));
        let xs: Vec<f32> = seen.iter().map(|c| c[0]).collect();
        assert_eq!(
            xs,
            (0..11).map(|i| 10.0 + 20.0 * i as f32).collect::<Vec<_>>(),
            "as onze células do caminho, em ordem e sem buraco"
        );
        assert!(seen.iter().all(|c| c[1] == 10.0), "a linha não desviou");
    }

    /// ⚠️ **A cópia de Symmetry é re-encaixada na célula.** O espelho de um centro não é, em geral, um
    /// centro — e um carimbo fora da grade é a única coisa que este método não pode fazer.
    ///
    /// **Mutação que must bleed:** apagar o re-encaixe em `stamp_cell`.
    #[test]
    fn the_mirrored_stamp_also_lands_on_a_cell_centre() {
        // Eixo deliberadamente FORA da grade: com o espelho numa linha de célula a lei seria honrada
        // por acidente, e a fixture não conteria o fenômeno.
        let sym = crate::SymmetrySettings {
            enabled: true,
            center: [107.0, 0.0],
            ..crate::SymmetrySettings::default()
        };
        let s = BrushSpec {
            stroke_method: crate::StrokeMethod::GridStamp,
            symmetry: sym,
            ..spec([20.0, 20.0], [0.0, 0.0])
        };
        let mut st = crate::Stroke::new(s, crate::Dynamics::default(), 0);
        let mut out = Vec::new();
        st.begin(pt(10.0, 10.0), &mut out);
        assert!(out.len() >= 2, "fixture: o espelho produziu uma cópia");
        for d in &out {
            assert_eq!(
                d.center,
                s.grid_cell_center(s.grid_cell_at(d.center)),
                "todo carimbo — inclusive o espelhado — pousa num centro de célula"
            );
        }
    }

    /// O raio do dab emitido é o da CÉLULA, não o do pincel: é o que faz o carimbo caber nela.
    #[test]
    fn the_emitted_radius_comes_from_the_cell_not_the_brush() {
        let s = BrushSpec {
            stroke_method: crate::StrokeMethod::GridStamp,
            radius_px: 3.0, // um pincel minúsculo — e irrelevante aqui
            ..spec([80.0, 20.0], [0.0, 0.0])
        };
        let mut st = crate::Stroke::new(s, crate::Dynamics::default(), 0);
        let mut out = Vec::new();
        st.begin(pt(10.0, 10.0), &mut out);
        assert_eq!(out[0].radius_px, 40.0, "eixo maior da célula / 2");
    }

    /// `as_grid_stamp` e `grid_stamp_frame` têm de concordar — são a MESMA resposta, e se divergirem
    /// o motor emite um raio e o sampler avalia outro.
    #[test]
    fn the_stamp_spec_carries_exactly_the_frame() {
        let s = spec([48.0, 16.0], [0.0; 2]);
        let (r, f, a) = s.grid_stamp_frame();
        let g = s.as_grid_stamp();
        assert_eq!((g.radius_px, g.dab_flatten, g.dab_angle_deg), (r, f, a));
        // …e nada mais mudou: a cor, o falloff e o blend do artista atravessam intactos.
        assert_eq!(g.color, s.color);
        assert_eq!(g.falloff, s.falloff);
        assert_eq!(g.strength, s.strength);
    }
}
