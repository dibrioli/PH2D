//! ADR-0114 W4 — **o balde**, do lado do shell: o clique vira uma região preenchida.
//!
//! O solver (`ph2d-flip-fill`) é puro e não sabe o que é um `FlipStroke`. Aqui é a
//! fronteira: converter a geometria do desenho no que ele entende, chamar, e virar o
//! resultado em documento.
//!
//! Três decisões que moram nesta fronteira:
//!
//! 1. **A espessura do traço é em px de TELA** (brush absoluto — Enio 2026-07-11), e o
//!    solver trabalha no espaço do documento. Então a meia-espessura em unidades do
//!    documento **depende do zoom** — e isso é honesto, não um bug: com o zoom afastado
//!    as linhas ficam relativamente mais grossas e os vãos fecham. É a mesma dependência
//!    que o GP tem (lá o solver também é em espaço de tela). O `Precision` multiplica a
//!    resolução do buffer por cima disso.
//! 2. **Os fechamentos de gap viram traços INVISÍVEIS persistentes** (o twist do
//!    Harmony): eles entram no desenho como qualquer outro traço, com `hide_stroke` e
//!    sem fill. Assim o vão fica fechado para sempre — re-preencher com outra cor, ou
//!    preencher o quadro vizinho, ou reabrir amanhã, não depende de a ferramenta estar
//!    com os mesmos parâmetros.
//! 3. **O fill entra ATRÁS** dos traços (índice 0 da lista): a cor vai por baixo do
//!    line-art, que é o que "colorir" significa. Em `PaintBehind`, ele entra por baixo
//!    também dos fills que já existem.

use ph2d_core::Vec2;
use ph2d_flip::{Fill, FlipDrawing, FlipStroke, Point, Rgba};
use ph2d_flip_fill::{FillError, FillMode, FillParams, fill_at};
use ph2d_tool_flip::FlipStyleSnapshot;
use ph2d_vec_scene::Xform;

/// As linhas que delimitam o preenchimento: TODOS os traços do desenho que não são,
/// eles próprios, um preenchimento sem contorno.
///
/// Um fill anterior (`hide_stroke`) **não** é fronteira — senão a 2ª cor não conseguiria
/// entrar por baixo da 1ª. Mas um fechamento de gap persistente (que também é
/// `hide_stroke`) **é** — é exatamente para isso que ele existe. Os dois se distinguem
/// pelo `fill`: o preenchimento tem cor; o fechamento não.
fn boundaries(drawing: &FlipDrawing, px_to_world: f32) -> Vec<(Vec<Vec2>, Vec<f32>, bool)> {
    drawing
        .strokes
        .iter()
        .filter(|s| !(s.hide_stroke && s.fill.is_some())) // fills anteriores não barram
        .filter(|s| s.len() >= 2)
        .map(|s| {
            let pts = s.positions().to_vec();
            // **A conversão de unidade que faltava** (e que matava o balde no produto):
            // `width` é guardado em px de TELA já recuados pela escala do objeto
            // (`width_px × mean_scale`, `flip_draw::build_stroke`), enquanto os PONTOS
            // são unidades do documento. Misturar os dois punha uma linha de 3 unidades
            // de mundo (≈324 px!) num desenho de 2,8 unidades: o clique caía sempre
            // DENTRO do traço e o balde respondia "clicked on a line", sempre.
            //
            // `× px_to_world` — NÃO `× doc_per_px`: o `mean_scale` já está embutido no
            // `width`, e multiplicá-lo de novo aplicaria a escala do objeto duas vezes.
            // É exatamente o que a borracha faz (`flip_erase`: raio = w·0,5·px_to_world).
            let half: Vec<f32> = s.widths().iter().map(|w| w * 0.5 * px_to_world).collect();
            (pts, half, s.closed)
        })
        .collect()
}

/// O traço que materializa a região preenchida.
fn fill_stroke(outer: &[Vec2], holes: Vec<Vec<Vec2>>, color: Rgba, opacity: f32) -> FlipStroke {
    let mut s = FlipStroke::new();
    for &p in outer {
        s.push_point(Point {
            pos: p,
            width: 0.0, // sem contorno próprio…
            opacity: 1.0,
            color,
        });
    }
    s.closed = true;
    s.hide_stroke = true; // …e o contorno NÃO é rasterizado: só o preenchimento
    s.holes = holes;
    s.fill = Some(Fill { color, opacity });
    s
}

/// O traço invisível que fecha um vão — persistente, sem cor, sem preenchimento.
fn closure_stroke(a: Vec2, b: Vec2) -> FlipStroke {
    let mut s = FlipStroke::new();
    for &p in &[a, b] {
        s.push_point(Point {
            pos: p,
            // Largura ~zero: ele delimita, mas não pinta. (No raster do solver ele vira
            // uma linha de 1px, como no GP.)
            width: 0.0,
            opacity: 0.0,
            color: Rgba::TRANSPARENT,
        });
    }
    s.hide_stroke = true;
    s
}

/// Um traço é um preenchimento (não um fechamento nem line-art)?
fn is_fill(s: &FlipStroke) -> bool {
    s.hide_stroke && s.fill.is_some()
}

/// O ponto `p` (LOCAL) está dentro do preenchimento do traço `s`? (Even-odd sobre o
/// contorno externo, descontando os buracos — a mesma regra do render.)
fn fill_contains(s: &FlipStroke, p: Vec2) -> bool {
    if !is_fill(s) {
        return false;
    }
    let inside = |ring: &[Vec2]| -> bool {
        let n = ring.len();
        let mut c = false;
        for i in 0..n {
            let (a, b) = (ring[i], ring[(i + 1) % n]);
            if (a.y > p.y) != (b.y > p.y) {
                let t = (p.y - a.y) / (b.y - a.y);
                if p.x < a.x + t * (b.x - a.x) {
                    c = !c;
                }
            }
        }
        c
    };
    inside(s.positions()) && !s.holes.iter().any(|h| inside(h))
}

/// **O clique do balde.** `local` é o ponto clicado no espaço do desenho; `px_to_world`
/// converte px de tela em unidades do documento (o zoom da câmera).
///
/// Devolve `Ok(())` se o documento mudou.
pub(crate) fn fill_click(
    drawing: &mut FlipDrawing,
    style: &FlipStyleSnapshot,
    local: Vec2,
    px_to_world: f32,
    world_to_local: &Xform,
) -> Result<(), FillError> {
    // O modo Unpaint não roda o solver: ele só apaga o fill que está sob o clique.
    let mode = match style.fill_mode {
        ph2d_tool_flip::FillMode::Paint => FillMode::Paint,
        ph2d_tool_flip::FillMode::PaintBehind => FillMode::PaintBehind,
        ph2d_tool_flip::FillMode::Unpaint => {
            // O de CIMA primeiro (o que o usuário vê): varre de trás para a frente.
            let hit = drawing
                .strokes
                .iter()
                .rposition(|s| fill_contains(s, local));
            return match hit {
                Some(i) => {
                    drawing.strokes.remove(i);
                    Ok(())
                }
                None => Err(FillError::Degenerate), // nada para despintar aqui
            };
        }
    };

    let strokes = boundaries(drawing, px_to_world);
    if strokes.is_empty() {
        return Err(FillError::Empty);
    }
    // A escala do objeto: a geometria é LOCAL (ADR-0111), e o zoom da câmera é em mundo.
    let obj_scale = world_to_local.mean_scale() as f32;
    let doc_per_px = px_to_world * obj_scale;
    let params = FillParams {
        // `precision` = pixels do buffer por unidade do documento. Um px de tela vale
        // `doc_per_px` unidades, então "1 buffer-px por px de tela" é `1/doc_per_px`.
        precision: (style.precision as f32) / doc_per_px.max(1e-6),
        gap_reach: (style.gap_px as f32) * doc_per_px,
        grow: style.grow.round() as i32,
        mode,
    };
    let r = fill_at(&strokes, local, params)?;

    let color = crate::flip_draw::srgb8_to_linear(style.fill_color);
    let stroke = fill_stroke(&r.outer, r.holes, color, 1.0);

    // Os fechamentos que a solução usou viram traços invisíveis PERSISTENTES.
    for c in &r.closures {
        drawing.strokes.insert(0, closure_stroke(c.a, c.b));
    }

    // Onde o preenchimento entra na pilha:
    // - `Paint`: atrás de todo o line-art (a cor vai por baixo da linha) mas NA FRENTE
    //   dos fills que já existem (a cor nova cobre a velha — é o balde de sempre);
    // - `PaintBehind`: atrás de tudo, inclusive dos fills antigos (colorir o que ainda
    //   não foi colorido, sem tocar no que já está).
    let at = match mode {
        FillMode::PaintBehind => 0,
        _ => drawing
            .strokes
            .iter()
            .rposition(is_fill)
            .map_or(0, |i| i + 1),
    };
    drawing.strokes.insert(at, stroke);
    Ok(())
}

impl crate::App {
    /// A tool Flip quer o canvas para PREENCHER agora? (ativa + modo Fill.) Lê o cache
    /// que o `flip_bridge` publica — sem downcast (o `input_dispatch` é livre).
    #[must_use]
    pub(crate) fn flip_wants_fill(&self) -> bool {
        self.flip_active
            && matches!(
                self.flip_style.map(|s| s.mode),
                Some(ph2d_tool_flip::FlipMode::Fill)
            )
    }

    /// O clique do balde. `true` = consumido (o gizmo/pick não vê o clique).
    ///
    /// O balde é um CLIQUE, não um arrasto: uma única chamada faz tudo. O desenho-alvo
    /// vem do **autokey por-tool** (`flip_autokey`, política `Modify`): preencher é
    /// MODIFICAR o que está na tela, então no rabo de um hold a chave nova nasce como
    /// duplicata — nunca em branco (preencher um quadro vazio e invisível seria o mesmo
    /// desastre da borracha, `docs/Flip/05 §4`).
    pub(crate) fn flip_fill_canvas_down(&mut self, x: f32, y: f32) -> bool {
        if !self.flip_wants_fill() {
            return false;
        }
        let Some(style) = self.flip_style else {
            return false;
        };
        let active_layer = self.flip_active_layer;
        let w2l = self.flip_active_world_to_local();
        let playhead = self.playhead;
        let strip = &self.flip_strip;

        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.surface.size();
        let world = gfx.camera.screen_to_world((x, y), win);
        let px_to_world = gfx.camera.height_world.max(f32::EPSILON) / win.height.max(1) as f32;
        let local = w2l.apply([f64::from(world[0]), f64::from(world[1])]);
        let local = Vec2::new(local[0] as f32, local[1] as f32);

        let Some((oid, _lid, did)) = crate::flip_autokey::target_drawing(
            &mut gfx.flip,
            &playhead,
            active_layer,
            strip,
            crate::flip_autokey::FlipEdit::Modify,
        ) else {
            // Sem desenho-alvo — camada TRAVADA, ou sem chave com o AutoKey desligado.
            // Também aqui o balde tem de DIZER: consumir o clique e não fazer nada é
            // exatamente o que faz uma ferramenta parecer quebrada (é o mesmo princípio
            // dos erros do solver, logo abaixo — só que este caminho tinha escapado).
            gfx.toasts.push(ph2d_editor::Toast::warning(
                "Fill: the layer is locked, or has no drawing on this frame",
            ));
            self.title_dirty = true;
            return true;
        };
        let Some(drawing) = gfx.flip.object_mut(oid).and_then(|o| o.drawing_mut(did)) else {
            return true;
        };
        match fill_click(drawing, &style, local, px_to_world, &w2l) {
            Ok(()) => {}
            Err(e) => {
                // Um fill que não aconteceu DIZ por quê — em vez de não fazer nada em
                // silêncio (que é como um balde parece quebrado).
                let msg = match e {
                    FillError::Leaked => "Fill leaked — raise Gap Closure to seal the outline",
                    FillError::OnBoundary => "Fill: clicked on a line",
                    FillError::Empty => "Fill: nothing to fill here",
                    FillError::Degenerate => "Fill: no region under the cursor",
                };
                gfx.toasts.push(ph2d_editor::Toast::warning(msg));
                self.title_dirty = true;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_tool_flip::FillMode as ToolFillMode;

    fn style(mode: ToolFillMode) -> FlipStyleSnapshot {
        FlipStyleSnapshot {
            fill_mode: mode,
            fill_color: [200, 100, 50, 255],
            gap_px: 0.0,
            grow: 2.0,
            precision: 1.0,
            ..Default::default()
        }
    }

    /// Um quadrado de line-art no desenho.
    fn boxed_drawing() -> FlipDrawing {
        let mut d = FlipDrawing::new();
        let mut s = FlipStroke::new();
        for p in [
            Vec2::new(0.0, 0.0),
            Vec2::new(20.0, 0.0),
            Vec2::new(20.0, 20.0),
            Vec2::new(0.0, 20.0),
        ] {
            s.push_point(Point {
                pos: p,
                width: 0.6,
                opacity: 1.0,
                color: Rgba::BLACK,
            });
        }
        s.closed = true;
        d.strokes.push(s);
        d
    }

    /// O caminho feliz: clicar dentro do quadrado cria UM traço de preenchimento — que
    /// entra ATRÁS do line-art (a cor vai por baixo da linha).
    #[test]
    fn clicking_inside_creates_a_fill_behind_the_line_art() {
        let mut d = boxed_drawing();
        fill_click(
            &mut d,
            &style(ToolFillMode::Paint),
            Vec2::new(10.0, 10.0),
            1.0,
            &Xform::IDENTITY,
        )
        .expect("um quadrado fechado preenche");
        assert_eq!(d.strokes.len(), 2);
        assert!(is_fill(&d.strokes[0]), "o fill entra ATRÁS do line-art");
        assert!(!d.strokes[1].hide_stroke, "o line-art continua visível");
        assert!(d.strokes[0].fill.is_some());
    }

    /// **Unpaint** remove o preenchimento sob o clique — e não toca no line-art.
    #[test]
    fn unpaint_removes_the_fill_under_the_click_only() {
        let mut d = boxed_drawing();
        fill_click(
            &mut d,
            &style(ToolFillMode::Paint),
            Vec2::new(10.0, 10.0),
            1.0,
            &Xform::IDENTITY,
        )
        .unwrap();
        assert_eq!(d.strokes.len(), 2);

        fill_click(
            &mut d,
            &style(ToolFillMode::Unpaint),
            Vec2::new(10.0, 10.0),
            1.0,
            &Xform::IDENTITY,
        )
        .expect("há um fill sob o clique");
        assert_eq!(d.strokes.len(), 1, "só o fill saiu");
        assert!(!d.strokes[0].hide_stroke, "o line-art ficou");

        // E despintar o vazio não faz nada (nem entra em pânico).
        assert!(
            fill_click(
                &mut d,
                &style(ToolFillMode::Unpaint),
                Vec2::new(10.0, 10.0),
                1.0,
                &Xform::IDENTITY,
            )
            .is_err()
        );
    }

    /// **Um fill anterior NÃO é fronteira** — senão a 2ª cor nunca entraria por baixo
    /// da 1ª, e o `Paint Behind` seria impossível. (Um fechamento de gap, que também é
    /// invisível, É fronteira: os dois se distinguem pelo `fill`.)
    #[test]
    fn an_existing_fill_is_not_a_boundary() {
        let mut d = boxed_drawing();
        fill_click(
            &mut d,
            &style(ToolFillMode::Paint),
            Vec2::new(10.0, 10.0),
            1.0,
            &Xform::IDENTITY,
        )
        .unwrap();
        // O 2º clique preenche a MESMA região (o fill velho não a dividiu).
        fill_click(
            &mut d,
            &style(ToolFillMode::Paint),
            Vec2::new(10.0, 10.0),
            1.0,
            &Xform::IDENTITY,
        )
        .expect("o fill anterior não pode barrar o novo");
        assert_eq!(d.strokes.len(), 3, "dois fills + o line-art");
    }

    /// **Gap Closure:** um "C" aberto não preenche… até o Gap Closure fechá-lo — e o
    /// fechamento fica no desenho como traço invisível PERSISTENTE.
    #[test]
    fn gap_closure_leaves_a_persistent_invisible_stroke() {
        let mut d = FlipDrawing::new();
        // Um "C" e uma parede à direita (as pontas do C apontam para ela).
        let mut c = FlipStroke::new();
        for p in [
            Vec2::new(20.0, 0.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 20.0),
            Vec2::new(20.0, 20.0),
        ] {
            c.push_point(Point {
                pos: p,
                width: 0.6,
                opacity: 1.0,
                color: Rgba::BLACK,
            });
        }
        d.strokes.push(c);
        // A parede fica a 10 unidades das pontas do C — bem além do filtro de
        // vazamento cruzado (3px), então SEM Gap Closure isto vaza de verdade.
        let mut wall = FlipStroke::new();
        for p in [Vec2::new(30.0, -2.0), Vec2::new(30.0, 22.0)] {
            wall.push_point(Point {
                pos: p,
                width: 0.6,
                opacity: 1.0,
                color: Rgba::BLACK,
            });
        }
        d.strokes.push(wall);

        // Sem Gap Closure: vaza.
        let err = fill_click(
            &mut d,
            &style(ToolFillMode::Paint),
            Vec2::new(10.0, 10.0),
            1.0,
            &Xform::IDENTITY,
        )
        .unwrap_err();
        assert_eq!(err, FillError::Leaked, "o C aberto tem de recusar");

        // Com Gap Closure: fecha, e o fechamento FICA no documento.
        let s = FlipStyleSnapshot {
            gap_px: 15.0,
            ..style(ToolFillMode::Paint)
        };
        fill_click(&mut d, &s, Vec2::new(10.0, 10.0), 1.0, &Xform::IDENTITY)
            .expect("com Gap Closure, o C fecha");
        let closures = d
            .strokes
            .iter()
            .filter(|st| st.hide_stroke && st.fill.is_none())
            .count();
        assert!(
            closures >= 1,
            "o fechamento tem de virar traço invisível persistente"
        );
        // E ele é FRONTEIRA no próximo fill: re-preencher (outra cor) funciona SEM o
        // Gap Closure ligado — o vão ficou fechado. É todo o ponto do twist do Harmony.
        let mut plain = style(ToolFillMode::Paint);
        plain.fill_color = [10, 200, 10, 255];
        plain.gap_px = 0.0;
        fill_click(&mut d, &plain, Vec2::new(10.0, 10.0), 1.0, &Xform::IDENTITY)
            .expect("o vão já está fechado: re-preencher não precisa do Gap Closure");
    }

    /// **O bug que matou o balde no produto (auditoria 2026-07-12).**
    ///
    /// Todos os testes acima passam `px_to_world = 1.0` — o ÚNICO valor em que um px
    /// de tela vale uma unidade de documento. Foi exatamente esse valor que escondeu a
    /// conversão faltante: com a câmera de verdade (`height_world = 10` numa janela de
    /// 1080p, `px_to_world ≈ 0,0093`), a meia-espessura de um traço de 6 px era lida
    /// como **3 unidades de mundo** — uma linha de ~324 px atravessando um desenho de
    /// 2,8 unidades. O clique caía sempre dentro do traço.
    ///
    /// Este teste usa os números do PRODUTO. Mutação que sangra: tire o `px_to_world`
    /// do `boundaries` e ele volta a `OnBoundary`.
    #[test]
    fn the_bucket_fills_at_the_real_camera_scale() {
        // Câmera default: 10 unidades de mundo na altura de uma janela de 1080p.
        let px_to_world = 10.0 / 1080.0;
        // Um quadrado de ~300 px na tela, desenhado com o pincel de 6 px.
        let side = 300.0 * px_to_world;
        let mut d = FlipDrawing::new();
        let mut s = FlipStroke::new();
        for p in [
            Vec2::new(0.0, 0.0),
            Vec2::new(side, 0.0),
            Vec2::new(side, side),
            Vec2::new(0.0, side),
        ] {
            s.push_point(Point {
                pos: p,
                width: 6.0, // px de TELA (o que o `build_stroke` guarda)
                opacity: 1.0,
                color: Rgba::BLACK,
            });
        }
        s.closed = true;
        d.strokes.push(s);

        fill_click(
            &mut d,
            &style(ToolFillMode::Paint),
            Vec2::new(side * 0.5, side * 0.5),
            px_to_world,
            &Xform::IDENTITY,
        )
        .expect("um quadrado fechado no zoom PADRAO tem de preencher");

        assert!(is_fill(&d.strokes[0]), "o fill entra atras do line-art");
        // E a regiao preenchida cobre a maior parte do quadrado (nao uma casquinha).
        let area = {
            let r = d.strokes[0].positions();
            let n = r.len();
            (0..n)
                .map(|i| {
                    let (a, b) = (r[i], r[(i + 1) % n]);
                    a.x * b.y - b.x * a.y
                })
                .sum::<f32>()
                .abs()
                * 0.5
        };
        let full = side * side;
        assert!(
            area > full * 0.8,
            "o fill cobriu so {:.0}% do quadrado (a espessura ainda esta em unidade errada)",
            area / full * 100.0
        );
    }
}
