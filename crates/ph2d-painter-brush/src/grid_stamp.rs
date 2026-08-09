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

/// O quanto o carimbo pode ENCOLHER em relação à célula, como fração dela ([`BrushSpec::grid_fit`]).
/// `−0.5` deixa o carimbo com metade da célula — meia célula de espaço em volta de cada um.
pub const GRID_FIT_MIN: f32 = -0.5;

/// O quanto o carimbo pode CRESCER em relação à célula ([`BrushSpec::grid_fit`]). `+0.5` o deixa com
/// uma célula e meia — vizinhos se sobrepõem por meia célula.
///
/// ⚠️ A faixa é a que o Enio pediu (2026-08-09), e ela **não é um limite de recurso**: um carimbo é
/// um dab como qualquer outro e o motor pinta raios muito maiores. Ela é o alcance ÚTIL do gesto —
/// fora de `[-0.5, 0.5]` a relação com a célula deixa de ser legível (a menos de meia célula o
/// carimbo é um ponto perdido no meio da rede; a mais de uma e meia ele cobre o vizinho do vizinho).
pub const GRID_FIT_MAX: f32 = 0.5;

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

    /// O quanto o carimbo mede em relação à célula: `1.0` a preenche exatamente, e
    /// [`BrushSpec::grid_fit`] em `[`[`GRID_FIT_MIN`]`, `[`GRID_FIT_MAX`]`]` dá folga ou
    /// sobreposição. Saneado aqui (um NaN vindo de uma caixa de texto não pode virar um raio NaN).
    #[must_use]
    pub fn grid_fit_scale(&self) -> f32 {
        let fit = if self.grid_fit.is_finite() {
            self.grid_fit.clamp(GRID_FIT_MIN, GRID_FIT_MAX)
        } else {
            0.0
        };
        1.0 + fit
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
    /// ⚠️ **O raio é ANCORADO NA BORDA DA TINTA, não no círculo geométrico** — e sem isso o carimbo
    /// não enche a célula. A silhueta de um falloff macio acaba bem antes do raio (o `W_TAIL` do
    /// [`crate::height_film::body_edge_t`]: o Smooth padrão termina em `t ≈ 0,61`), então pôr o raio
    /// em meia célula deixa um carimbo com ~60% da largura dela e uma folga que o artista lê como
    /// "o método não faz o que promete". Dividir por `t0` estica a tinta até a borda.
    ///
    /// É a MESMA correção — e a MESMA porta — que a âncora do bow wave pagou em 2026-07-15: *o aro
    /// nascia em `t = 1` do footprint enquanto a tinta macia termina em `t ≈ 0,61`*. Uma ponta dura
    /// (Constant, hardness ≥ 1) tem `t0 = 1` e o raio é meia célula, exatamente como antes — e uma
    /// silhueta de **Shape** também, que é o que `has_shape` diz.
    #[must_use]
    pub fn grid_stamp_frame(&self, has_shape: bool) -> (f32, f32, u16) {
        let [w, h] = self.grid_cell();
        let (major, minor, angle) = if w >= h { (w, h, 0) } else { (h, w, 90) };
        let flatten = (1.0 - minor / major).clamp(0.0, crate::DAB_FLATTEN_MAX);
        // ⚠️ **`has_shape` é a MESMA pergunta que o aro do bow wave faz, pela MESMA porta**
        // ([`crate::height_push::rim_t0`]): *onde a tinta deste dab ACABA?* Um falloff macio acaba
        // em `t ≈ 0,61`, então o raio o compensa; uma silhueta de **Shape** (imagem ou procedural)
        // acaba no aro geométrico, `t = 1`, e compensá-la assim mesmo é o defeito que o Enio
        // reportou em 2026-08-09 — *"a imagem extrapola a célula do grid"*: a imagem saía **1,64×**
        // a célula, e as vizinhas se comiam.
        //
        // Perguntar à porta em vez de re-derivar aqui é o que impede o carimbo de discordar do aro
        // sobre onde a tinta termina — eles são a mesma afirmação sobre o mesmo pincel.
        // O piso é defesa contra um perfil custom degenerado, que produziria um raio infinito.
        let t0 = crate::height_push::rim_t0(self, has_shape).max(0.05);
        (major * self.grid_fit_scale() * 0.5 / t0, flatten, angle)
    }

    /// **As LINHAS da grade num eixo**, dentro de `[lo, hi]` (px de imagem): `(primeira, passo,
    /// quantas)`. A linha `k` é `offset + k·célula` — a FRONTEIRA entre as células `k−1` e `k`, a
    /// mesma que o [`Self::grid_cell_at`] usa para decidir de quem é um ponto.
    ///
    /// ⚠️ **É a mesma porta que o carimbo**, e é o ponto inteiro deste módulo: um overlay que
    /// re-derivasse a rede desenharia linhas que a tinta não respeita — e um desenho errado parece
    /// autoritativo, então o artista acreditaria nele em vez do carimbo. O gate que ancora as duas é
    /// o `the_drawn_line_is_the_boundary_the_stamp_lands_between`.
    ///
    /// A contagem é `0` para uma janela vazia ou invertida; nenhum chamador precisa checar antes.
    #[must_use]
    pub fn grid_line_run(&self, axis: usize, lo: f32, hi: f32) -> (f32, f32, u32) {
        let step = self.grid_cell()[axis & 1];
        let off = self.grid_offset_px[axis & 1];
        let off = if off.is_finite() { off } else { 0.0 };
        if !(lo.is_finite() && hi.is_finite()) || hi < lo {
            return (lo, step, 0);
        }
        let k0 = ((lo - off) / step).ceil();
        let first = off + k0 * step;
        // `+1` porque as duas pontas contam: uma janela de exatamente uma célula tem DUAS linhas.
        let n = ((hi - first) / step).floor() + 1.0;
        (first, step, if n > 0.0 { n as u32 } else { 0 })
    }

    /// Este spec, reescrito para carimbar UMA célula: raio, achatamento e ângulo saem da grade.
    ///
    /// **É a porta ÚNICA**, e o motor não tem outra: o tool a aplica ao spec ANTES de abrir o traço
    /// ([`crate::Stroke::new`]), então o `radius_px` que o `grid_dab` emite é este mesmo — e a
    /// aplica de novo antes de estampar, o que é seguro porque ela é **idempotente** (o frame sai da
    /// célula, que ela não toca). Era o motor quem chamava o frame, e ali `has_shape` é
    /// inalcançável: os pixels do Shape vivem no tool, não no `BrushSpec`, que é `Copy`.
    #[must_use]
    pub fn as_grid_stamp(&self, has_shape: bool) -> Self {
        let (radius, flatten, angle) = self.grid_stamp_frame(has_shape);
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

    /// **A linha desenhada É a fronteira entre as células em que o carimbo pousa.**
    ///
    /// O gate que ancora o overlay ao carimbo: o centro da célula `k` tem de cair exatamente no MEIO
    /// entre a linha `k` e a linha `k+1`. Uma re-derivação que tivesse deslizado meia célula — o erro
    /// clássico de quem confunde `floor` com `round` — passa por qualquer teste que só olhe para o
    /// espaçamento, e falha aqui.
    ///
    /// **Mutação que deve sangrar:** trocar o `ceil` do `grid_line_run` por `floor`, ou o `off + k·c`
    /// por `off + (k + 0.5)·c` (as linhas passando pelos centros em vez das bordas).
    #[test]
    fn the_drawn_line_is_the_boundary_the_stamp_lands_between() {
        for (cell, off) in [
            ([32.0, 32.0], [0.0, 0.0]),
            ([40.0, 24.0], [7.0, 13.0]),
            ([9.5, 9.5], [0.25, 0.75]),
        ] {
            let s = spec(cell, off);
            for axis in 0..2 {
                let (first, step, n) = s.grid_line_run(axis, 0.0, 200.0);
                assert!(
                    n >= 2,
                    "a janela tem de conter linhas — fixture vazia não prova nada"
                );
                for k in 0..n - 1 {
                    let a = first + step * k as f32;
                    let b = a + step;
                    let mid = 0.5 * (a + b);
                    // O ponto médio entre duas linhas vizinhas é o centro de UMA célula, e é aquela
                    // a que ele pertence.
                    let mut p = [mid, mid];
                    p[1 - axis] = s.grid_cell_center([0, 0])[1 - axis]; // o outro eixo, num centro
                    let cellf = s.grid_cell_at(p);
                    let centre = s.grid_cell_center(cellf)[axis];
                    assert!(
                        (centre - mid).abs() < 1e-3,
                        "meio das linhas {a}..{b} = {mid}, mas o carimbo pousa em {centre} (eixo \
                         {axis}, célula {cell:?}, offset {off:?}) — o desenho e a tinta discordam \
                         sobre onde a célula está"
                    );
                }
            }
        }
    }

    /// A janela degenerada não produz linha nenhuma — e nem pânico. Um overlay é desenhado a cada
    /// quadro, inclusive no quadro em que a sprite ainda não tem tamanho.
    #[test]
    fn a_degenerate_window_draws_nothing() {
        let s = spec([32.0, 32.0], [0.0, 0.0]);
        for (lo, hi) in [(10.0, 5.0), (f32::NAN, 100.0), (0.0, f32::INFINITY)] {
            let (_, _, n) = s.grid_line_run(0, lo, hi);
            if hi.is_infinite() {
                continue; // um alcance infinito é o chamador errando, não a porta
            }
            assert_eq!(n, 0, "janela ({lo}, {hi}) devia dar zero linhas");
        }
    }

    /// **Toda linha devolvida está DENTRO da janela pedida.** É a promessa literal da porta, e sem
    /// este gate ela não estava sendo verificada por nada: uma mutação `ceil → floor` no primeiro
    /// índice mantém a rede — mesmo passo, mesma fase — e só desloca o COMEÇO para fora de `lo`, então
    /// passa por todo teste que só olhe para o espaçamento ou para o alinhamento com o carimbo.
    ///
    /// O preço da falha é o chamador desenhar fora da janela que ele recortou, que é exatamente o
    /// custo que o recorte existe para remover.
    ///
    /// **Mutação que deve sangrar:** `ceil` → `floor` no `k0`.
    #[test]
    fn every_line_lands_inside_the_window_it_was_asked_for() {
        for (cell, off) in [
            ([32.0, 32.0], [0.0, 0.0]),
            ([40.0, 24.0], [7.0, 13.0]),
            ([9.5, 9.5], [0.25, 0.75]),
        ] {
            let s = spec(cell, off);
            for axis in 0..2 {
                // Janelas que NÃO começam no zero — é ali que `ceil` e `floor` divergem, e uma fixture
                // ancorada em zero com offset zero não contém o fenômeno.
                for (lo, hi) in [(0.0f32, 200.0f32), (37.0, 210.0), (100.5, 140.5)] {
                    let (first, step, n) = s.grid_line_run(axis, lo, hi);
                    assert!(
                        n > 0,
                        "janela ({lo}, {hi}) devia conter linhas (célula {cell:?})"
                    );
                    assert!(
                        first >= lo - 1e-3,
                        "a primeira linha ({first}) está ANTES de lo ({lo}) — o chamador desenha fora \
                         da janela que recortou (eixo {axis}, célula {cell:?}, offset {off:?})"
                    );
                    let last = first + step * (n - 1) as f32;
                    assert!(
                        last <= hi + 1e-3,
                        "a última linha ({last}) passa de hi ({hi})"
                    );
                    // E a SEGUINTE já está fora: a contagem não deixa linha visível de fora.
                    assert!(
                        last + step > hi + 1e-3,
                        "a contagem parou cedo — sobrou uma linha dentro da janela sem desenhar"
                    );
                }
            }
        }
    }

    /// Uma janela de EXATAMENTE uma célula tem DUAS linhas — as duas pontas contam. É o off-by-one
    /// que faz a última coluna da tela ficar sem borda.
    #[test]
    fn a_window_of_one_cell_has_both_of_its_edges() {
        let s = spec([32.0, 32.0], [0.0, 0.0]);
        let (first, _, n) = s.grid_line_run(0, 0.0, 32.0);
        assert_eq!((first, n), (0.0, 2));
    }

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
        // O ACHATAMENTO e o ÂNGULO são pura razão de aspecto da célula — o RAIO tem lei própria (a
        // âncora na borda da tinta), gateada em `the_radius_is_anchored_on_the_paints_edge`.
        let (_, f, a) = spec([40.0, 40.0], [0.0; 2]).grid_stamp_frame(false);
        assert_eq!((f, a), (0.0, 0), "quadrada: sem achatamento");

        let (_, f, a) = spec([40.0, 10.0], [0.0; 2]).grid_stamp_frame(false);
        assert_eq!(a, 0, "deitada: eixo maior horizontal");
        assert!((f - 0.75).abs() < 1e-6, "menor = 10 = 40*(1-0.75)");

        let (_, f, a) = spec([10.0, 40.0], [0.0; 2]).grid_stamp_frame(false);
        assert_eq!(
            a, 90,
            "em pé: eixo maior VERTICAL — o ângulo é quem diz isso"
        );
        assert!((f - 0.75).abs() < 1e-6);
    }

    /// ⚠️ **O raio é ancorado na borda da TINTA, não no círculo geométrico** — a correção que o
    /// render-and-look pegou (2026-08-09: os carimbos enchiam ~60% da célula e sobrava folga).
    ///
    /// O **CONTROLE** é uma ponta DURA: com `Constant` a tinta chega ao aro (`t0 = 1`) e o raio é
    /// exatamente meia célula. É ele que prova que a divisão não é um fator inventado — e que uma
    /// ponta que já enchia a célula não passou a transbordar.
    #[test]
    fn the_radius_is_anchored_on_the_paints_edge() {
        let hard = BrushSpec {
            falloff: crate::Falloff::Constant,
            ..spec([40.0, 40.0], [0.0; 2])
        };
        let (r, _, _) = hard.grid_stamp_frame(false);
        assert!(
            (r - 20.0).abs() < 1e-3,
            "ponta dura: o raio É meia célula ({r})"
        );

        let soft = spec([40.0, 40.0], [0.0; 2]); // o falloff default, macio
        let (r_soft, _, _) = soft.grid_stamp_frame(false);
        let t0 = crate::height_film::body_edge_t(&soft);
        assert!(
            t0 < 0.8,
            "fixture: o falloff default TEM de ser macio, senão o gate é vácuo (t0={t0})"
        );
        assert!(
            (r_soft - 20.0 / t0).abs() < 1e-3,
            "o raio macio compensa exatamente a cauda do falloff"
        );
        assert!(r_soft > 20.0, "…e por isso é MAIOR que meia célula");
    }

    /// ⚠️ O limite de REPRESENTAÇÃO, com o número medido ao lado: além de 1:20 o carimbo para de
    /// afinar porque um eixo menor zero não é pintável. Um gate, não um comentário.
    #[test]
    fn the_aspect_the_stamp_can_honour_tops_out_at_twenty_to_one() {
        let (_, f, _) = spec([200.0, 10.0], [0.0; 2]).grid_stamp_frame(false);
        assert!((f - 0.95).abs() < 1e-6, "1:20 é exatamente o teto");
        let (_, f, _) = spec([2000.0, 10.0], [0.0; 2]).grid_stamp_frame(false);
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
    ///
    /// ⚠️ **O traço é aberto com o spec que passou por [`BrushSpec::as_grid_stamp`]** — a porta
    /// única, e a única que sabe se o slot de Shape carrega uma silhueta. É o contrato que o tool
    /// honra (`stroke_spec`), e a 2ª metade deste gate é o que o torna afirmação e não decoração:
    /// aberto com o spec CRU o motor emite o pincel, então quem largar a porta vê o carimbo encolher
    /// para o tamanho do pincel em vez de silenciosamente pintar o número errado.
    #[test]
    fn the_emitted_radius_comes_from_the_cell_not_the_brush() {
        let raw = BrushSpec {
            stroke_method: crate::StrokeMethod::GridStamp,
            radius_px: 3.0, // um pincel minúsculo — e irrelevante aqui
            ..spec([80.0, 20.0], [0.0, 0.0])
        };
        let mut out = Vec::new();
        let mut st = crate::Stroke::new(raw.as_grid_stamp(false), crate::Dynamics::default(), 0);
        st.begin(pt(10.0, 10.0), &mut out);
        let want = raw.grid_stamp_frame(false).0;
        assert_eq!(out[0].radius_px, want, "o raio emitido é o da CÉLULA");
        assert!(
            want > 40.0,
            "…e ancorado na borda da tinta, logo > meia célula"
        );

        let mut st = crate::Stroke::new(raw, crate::Dynamics::default(), 0);
        st.begin(pt(10.0, 10.0), &mut out);
        assert_eq!(
            out[0].radius_px, 3.0,
            "sem a porta o motor carimba o PINCEL — a falha é visível, não muda o número em silêncio"
        );
    }

    /// ⚠️ **Uma silhueta de Shape enche a célula EXATAMENTE** — o report do Enio de 2026-08-09 (*"a
    /// imagem extrapola a célula do grid"*). Um carimbo de imagem pinta até o aro geométrico, então
    /// compensá-lo pela cauda de um falloff que ele não tem o esticava **1,64×** para fora da célula.
    ///
    /// O **CONTROLE** é o falloff macio ao lado, que continua compensado: é ele que prova que a
    /// correção não é um fator inventado, e que o carimbo macio não passou a sub-encher.
    ///
    /// **Mutação que tem de sangrar:** `rim_t0(self, has_shape)` → `body_edge_t(self)`.
    #[test]
    fn a_shape_silhouette_fills_the_cell_exactly() {
        let s = spec([40.0, 40.0], [0.0; 2]);
        let (shaped, _, _) = s.grid_stamp_frame(true);
        assert!(
            (shaped - 20.0).abs() < 1e-3,
            "com silhueta o raio É meia célula ({shaped}) — a tinta acaba no aro"
        );
        let (soft, _, _) = s.grid_stamp_frame(false);
        assert!(
            soft > 20.0 * 1.5,
            "controle: sem silhueta o falloff macio segue compensado ({soft}), senão o gate não \
             distingue a correção de um raio simplesmente menor"
        );
    }

    /// **O Cell Fit abre espaço e sobrepõe** (Enio, 2026-08-09), e não mexe na REDE: o centro da
    /// célula e a contagem de linhas ficam onde estavam — é o que cada célula DESENHA que muda.
    ///
    /// **Mutação que tem de sangrar:** ignorar `grid_fit_scale()` no raio.
    #[test]
    fn the_cell_fit_grows_and_shrinks_the_stamp_without_moving_the_lattice() {
        let base = spec([40.0, 40.0], [0.0; 2]);
        let full = base.grid_stamp_frame(true).0;
        for (fit, want) in [(GRID_FIT_MIN, 0.5f32), (0.0, 1.0), (GRID_FIT_MAX, 1.5)] {
            let s = BrushSpec {
                grid_fit: fit,
                ..base
            };
            let r = s.grid_stamp_frame(true).0;
            assert!(
                (r / full - want).abs() < 1e-4,
                "fit {fit} devia dar {want}× a célula, deu {}×",
                r / full
            );
            // A rede não se move: mesmo centro, mesma célula sob o mesmo ponto.
            assert_eq!(s.grid_cell_center([2, 3]), base.grid_cell_center([2, 3]));
            assert_eq!(
                s.grid_cell_at([57.0, 91.0]),
                base.grid_cell_at([57.0, 91.0])
            );
        }
    }

    /// Um `grid_fit` degenerado (NaN, ou fora da faixa) não produz um raio degenerado — a caixa de
    /// texto do chip é uma entrada de usuário como qualquer outra.
    #[test]
    fn a_degenerate_fit_never_reaches_the_radius() {
        for bad in [f32::NAN, f32::INFINITY, -5.0, 5.0] {
            let s = BrushSpec {
                grid_fit: bad,
                ..spec([40.0, 40.0], [0.0; 2])
            };
            let r = s.grid_stamp_frame(true).0;
            assert!(r.is_finite() && r > 0.0, "fit {bad} deu raio {r}");
        }
    }

    /// `as_grid_stamp` e `grid_stamp_frame` têm de concordar — são a MESMA resposta, e se divergirem
    /// o motor emite um raio e o sampler avalia outro.
    #[test]
    fn the_stamp_spec_carries_exactly_the_frame() {
        let s = spec([48.0, 16.0], [0.0; 2]);
        let (r, f, a) = s.grid_stamp_frame(false);
        let g = s.as_grid_stamp(false);
        assert_eq!((g.radius_px, g.dab_flatten, g.dab_angle_deg), (r, f, a));
        // …e nada mais mudou: a cor, o falloff e o blend do artista atravessam intactos.
        assert_eq!(g.color, s.color);
        assert_eq!(g.falloff, s.falloff);
        assert_eq!(g.strength, s.strength);
    }
}
