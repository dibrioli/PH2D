//! ADR-0114 W4 — **o balde**, do lado do shell: o clique vira uma região preenchida.
//!
//! O solver (`ph2d-flip-fill`) é puro e não sabe o que é um `FlipStroke`. Aqui é a
//! fronteira: converter a geometria do desenho no que ele entende, chamar, e virar o
//! resultado em documento.
//!
//! Três decisões que moram nesta fronteira:
//!
//! 1. **A espessura do traço é em px de TELA** (brush absoluto — Enio 2026-07-11), e o
//!    fill é assado em unidades de DOCUMENTO — a relação entre os dois MUDA com o zoom.
//!    Por isso **a âncora do solver é o EIXO da linha** (BUGS #14), que é geometria
//!    pura: a espessura convertida abaixo só folga o bbox da grade, nunca decide onde a
//!    cor para. (O 1º corte ancorava na silhueta — congelada no zoom do clique, ela
//!    transbordava `(w/2)·(zoom−1)` px ao aproximar a câmera depois.) O `Precision`
//!    multiplica a resolução do buffer por cima disso.
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

/// Margem da dilatação do fill, em px de tela — a folga que cobre o erro de
/// VETORIZAÇÃO do contorno (marching squares + RDP + alisamento deixam o contorno até
/// ~1,5 px DENTRO do eixo nos picos de tremor).
///
/// **O valor saiu de uma varredura no pixel**, não do olho: dois defeitos OPOSTOS se
/// tocam aqui (`gpu_fill_fit::sweep_tuck`, medido no anel da linha, 256 raios):
///
/// | margem | fundo sob a linha | transbordo além dela |
/// |---|---|---|
/// | 0,0 | **4 px** (o defeito do smoke) | 5 |
/// | **0,5** | **0** | **16** |
/// | 1,5 | 0 | 99 |
/// | 2,0 | 0 | 195 |
///
/// `0,5` é o menor valor que zera o vazamento — margem a mais volta a empurrar a cor
/// para FORA da linha, que é exatamente o defeito que matou o `grow = +2` default
/// (BUGS #11).
const FILL_TUCK_PX: f32 = 0.5; // LITERAL-PX-OK: erro de vetorizacao do contorno, MEDIDO

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
            //
            // (Desde a âncora no eixo — BUGS #14 — o solver usa isto SÓ para o bbox da
            // grade; a parede e a borda da cor são o eixo, imunes ao zoom do clique.)
            let half: Vec<f32> = s.widths().iter().map(|w| w * 0.5 * px_to_world).collect();
            (pts, half, s.closed)
        })
        .collect()
}

/// O traço que materializa a região preenchida.
///
/// **A largura do contorno é a espessura da LINHA** — e isso não é um contorno de
/// verdade (o `hide_stroke` segue ligado): é a **dilatação da cor por baixo do
/// line-art**, sem a qual a arte não fecha.
///
/// A geometria do fill termina no **eixo** da linha (BUGS #14 — a única âncora imune
/// ao zoom), e o eixo fica a meia-espessura da silhueta. Sem dilatar, a metade externa
/// da linha não tem cor por baixo: com um pincel MACIO ela mistura com o fundo, e o
/// contorno ganha um halo escuro (o *"o fill não se ajusta à linha"* do smoke). Com a
/// dilatação, a cor vai exatamente até a silhueta — e como as duas grandezas estão na
/// MESMA unidade (px de tela, absoluta), elas escalam juntas: o encaixe é invariante
/// ao zoom, que era todo o ponto da âncora no eixo.
///
/// Largura VARIÁVEL (pressão): usa-se a média. A dilatação erra por (w_local − w_média)/2
/// nos pontos extremos — sub-pixel num traço de mouse (largura constante), e sempre
/// menor que o erro de não dilatar nada.
fn fill_stroke(
    outer: &[Vec2],
    holes: Vec<Vec<Vec2>>,
    color: Rgba,
    opacity: f32,
    line_width_px: f32,
) -> FlipStroke {
    let mut s = FlipStroke::new();
    for &p in outer {
        s.push_point(Point {
            pos: p,
            width: line_width_px, // a dilatação: a cor entra por baixo da linha
            opacity: 1.0,
            color,
        });
    }
    s.closed = true;
    s.hide_stroke = true; // não é line-art: é a região (o `is_fill` continua valendo)
    s.holes = holes;
    s.fill = Some(Fill { color, opacity });
    s
}

/// A espessura MÉDIA do line-art que delimita a região (px de tela) — a dilatação que
/// o contorno do fill veste. Ignora as regiões (que não têm tinta) e os fechamentos de
/// gap (largura zero).
fn mean_line_width(drawing: &FlipDrawing) -> f32 {
    let (sum, n) = drawing
        .strokes
        .iter()
        .filter(|s| !s.hide_stroke)
        .flat_map(|s| s.widths().iter().copied())
        .filter(|w| *w > 0.0)
        .fold((0.0f32, 0usize), |(sum, n), w| (sum + w, n + 1));
    if n == 0 { 0.0 } else { sum / n as f32 }
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

/// A área (com sinal) de um anel.
fn ring_area(r: &[Vec2]) -> f32 {
    let n = r.len();
    (0..n)
        .map(|i| {
            let (a, b) = (r[i], r[(i + 1) % n]);
            a.x * b.y - b.x * a.y
        })
        .sum::<f32>()
        * 0.5
}

/// O ponto está dentro do anel (even-odd)?
fn ring_contains(ring: &[Vec2], p: Vec2) -> bool {
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
}

/// **A região preenchida É o interior de um traço fechado?** Se for, o índice dele.
///
/// É a pergunta que muda tudo (smoke do Enio 2026-07-13, com o Suzanne ao lado):
///
/// > *"nem todo vertex da linha está conectado ao vertex de fill… isso cria áreas de
/// > dessincronização e gaps"*
///
/// Exato — e a causa é que o contorno do balde sai do **raster** (marching squares +
/// RDP), então os vértices dele não têm relação com os da linha: nas quinas ele chanfra,
/// nas retas ele desliza, e como o erro é assado em unidades de DOCUMENTO enquanto a
/// linha é absoluta em px de TELA, **o zoom o amplia**.
///
/// A cura não é aproximar melhor: é **não vetorizar**. Quando a região é o interior de
/// uma forma fechada, o preenchimento é o **fill do próprio traço** (a triangulação dos
/// pontos DELE — exatamente o que o Grease Pencil faz num material `stroke + fill`, que
/// é como o Suzanne é desenhado). Aí não há dois conjuntos de vértices para
/// dessincronizar: **há um só**. Esculpir a linha move a cor junto, de graça, para
/// sempre, em qualquer zoom.
///
/// O critério é conservador — os três têm de valer:
/// 1. o traço é **fechado** e é line-art (não uma região);
/// 2. o **clique** cai dentro dele;
/// 3. a área do contorno que o solver traçou **bate** com a do traço (≤ `AREA_TOL`) — é
///    isso que separa "preencheu a forma" de "preencheu um pedaço entre ela e outra".
///
/// Quando não bate (uma região delimitada por VÁRIOS traços — colorir entre linhas), não
/// existe "a curva" para carregar a cor, e o contorno vetorizado é o caminho — que é o
/// que o balde do GP também faz.
fn filled_shape_target(drawing: &FlipDrawing, outer: &[Vec2], click: Vec2) -> Option<usize> {
    /// Quanto a área do contorno traçado pode diferir da do traço (fração).
    const AREA_TOL: f32 = 0.15; // LITERAL-PX-OK: fracao, nao metrica de design
    let target_area = ring_area(outer).abs();
    if target_area <= 0.0 {
        return None;
    }
    drawing
        .strokes
        .iter()
        .enumerate()
        .filter(|(_, s)| s.closed && !s.hide_stroke && s.len() >= 3)
        .filter(|(_, s)| ring_contains(s.positions(), click))
        .find(|(_, s)| {
            let a = ring_area(s.positions()).abs();
            (a - target_area).abs() <= AREA_TOL * a.max(target_area)
        })
        .map(|(i, _)| i)
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
            //
            // **Um traço PREENCHIDO perde o fill; ele não é deletado.** Uma região é um
            // objeto de cor e some inteira; um traço com fill é LINE-ART que por acaso
            // carrega cor — apagá-lo levaria a linha junto, e o usuário só pediu para
            // tirar a cor.
            let hit = drawing.strokes.iter().rposition(|s| {
                (is_fill(s) && fill_contains(s, local))
                    || (s.fill.is_some()
                        && !s.hide_stroke
                        && s.closed
                        && ring_contains(s.positions(), local))
            });
            return match hit {
                Some(i) if drawing.strokes[i].hide_stroke => {
                    drawing.strokes.remove(i);
                    Ok(())
                }
                Some(i) => {
                    drawing.strokes[i].fill = None;
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
    let precision = (style.precision as f32) / doc_per_px.max(1e-6);
    let params = FillParams {
        // `precision` = pixels do buffer por unidade do documento. Um px de tela vale
        // `doc_per_px` unidades, então "1 buffer-px por px de tela" é `1/doc_per_px`.
        precision,
        gap_reach: (style.gap_px as f32) * doc_per_px,
        // **O Grow é em px de TELA** (é o que o usuário vê e ajusta), e o solver conta em
        // px de BUFFER — que valem `1/style.precision` px de tela. Sem esta conversão,
        // subir a Precision *encolhia* o Grow em silêncio: dois controles independentes
        // que secretamente se multiplicavam.
        grow: (style.grow * style.precision).round() as i32,
        mode,
    };
    // `PH2D_FLIP_FILL_DEBUG=1` — a régua do balde no app REAL. A auditoria mostrou que um
    // harness reproduz o mecanismo, não o contexto: os números que chegam aqui são os
    // únicos que importam.
    let debug = std::env::var("PH2D_FLIP_FILL_DEBUG").is_ok();
    if debug {
        let half0 = strokes
            .first()
            .and_then(|s| s.1.first().copied())
            .unwrap_or(0.0);
        eprintln!(
            "[fill] px_to_world={px_to_world:.6} obj_scale={obj_scale:.3} \
             style(precision={:.2} grow={:.1} gap={:.1}) \
             => buffer {:.1} px/unid ({:.2} px de tela por px de buffer) \
             | meia-espessura={half0:.4} unid ({:.1} px de tela) \
             | grow={} px de buffer = {:.1} px de TELA",
            style.precision,
            style.grow,
            style.gap_px,
            params.precision,
            1.0 / (params.precision * px_to_world).max(1e-9),
            half0 / px_to_world,
            params.grow,
            params.grow as f32 / (params.precision * px_to_world).max(1e-9),
        );
    }

    let r = fill_at(&strokes, local, params)?;

    if debug {
        let n = r.outer.len();
        let (mut lo, mut hi) = (Vec2::new(f32::MAX, f32::MAX), Vec2::new(f32::MIN, f32::MIN));
        for p in &r.outer {
            lo = Vec2::new(lo.x.min(p.x), lo.y.min(p.y));
            hi = Vec2::new(hi.x.max(p.x), hi.y.max(p.y));
        }
        eprintln!(
            "[fill] contorno: {n} vertices | bbox {:.1} x {:.1} px de tela | {} buracos | {} fechamentos",
            (hi.x - lo.x) / px_to_world,
            (hi.y - lo.y) / px_to_world,
            r.holes.len(),
            r.closures.len(),
        );
    }

    let color = crate::flip_draw::srgb8_to_linear(style.fill_color);

    // **A forma fechada pinta A SI MESMA** (a lição do Suzanne — ver `filled_shape_target`).
    // Sem contorno vetorizado, sem dois conjuntos de vértices, sem dessincronização: a cor
    // é a triangulação dos pontos da própria linha, em qualquer zoom, para sempre.
    if let Some(i) = filled_shape_target(drawing, &r.outer, local) {
        // Os fechamentos de gap que a solução usou continuam valendo (o twist do Harmony).
        for c in &r.closures {
            drawing.strokes.insert(0, closure_stroke(c.a, c.b));
        }
        let idx = i + r.closures.len(); // os fechamentos entraram ANTES dele
        drawing.strokes[idx].fill = Some(Fill {
            color,
            opacity: 1.0,
        });
        // Os buracos do solver (o "O") entram no traço: a região é o interior DELE.
        drawing.strokes[idx].holes = r.holes;
        return Ok(());
    }

    // A dilatação sai da ARTE (a espessura do line-art que delimita a região), não de
    // um parâmetro: é isso que faz a cor encaixar na linha sem o usuário ajustar nada.
    //
    // **Mais a margem de vetorização.** O contorno é traçado num buffer e simplificado
    // (marching squares + RDP), então ele cai até ~1,5 px de tela DENTRO do eixo nos
    // pontos de tremor. Sem a margem, sobra ali um fio de linha sem cor por baixo — e
    // com pincel macio esse fio deixa o fundo aparecer. A margem custa um transbordo de
    // no máximo ~1 px sob o anti-aliasing da linha (invisível), e é o mesmo remédio do
    // GP, cujo Grow default é +2 px pela MESMA razão.
    let dilate = mean_line_width(drawing) + 2.0 * FILL_TUCK_PX;
    let stroke = fill_stroke(&r.outer, r.holes, color, 1.0, dilate);

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
        let frame = gfx.flip.object(oid).map_or(0, |o| o.frame_at(&playhead));
        let Some(drawing) = gfx.flip.object_mut(oid).and_then(|o| o.drawing_mut(did)) else {
            return true;
        };
        // A base PRISTINA — o desenho ANTES do fill. É dela que todo reajuste vai partir
        // (`flip_live`): re-preencher sobre o resultado anterior faria os parâmetros se
        // COMPOREM (os fechamentos de gap se empilhariam a cada mexida no slider) em vez
        // de se substituírem, e o slider deixaria de ser reversível.
        let base = drawing.strokes.clone();
        match fill_click(drawing, &style, local, px_to_world, &w2l) {
            Ok(()) => {
                // O preenchimento vira o **alvo vivo**: Gap/Grow/Precision/cor continuam
                // mexendo NELE até o usuário fazer outra coisa.
                self.flip_live = Some(crate::flip_live::FlipLive {
                    oid,
                    did,
                    frame,
                    layer: active_layer,
                    mode: style.mode,
                    applied: style,
                    px_to_world,
                    w2l,
                    kind: crate::flip_live::LiveKind::Fill { base, click: local },
                });
            }
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
#[path = "flip_fill_tests.rs"]
mod tests;
