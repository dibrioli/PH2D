//! ADR-0114 W4 — **o balde**, do lado do shell: o clique vira uma região preenchida.
//!
//! O solver (`ph2d-flip-fill`) é puro e não sabe o que é um `FlipStroke`. Aqui é a
//! fronteira: converter a geometria do desenho no que ele entende, chamar, e virar o
//! resultado em documento.
//!
//! Três decisões que moram nesta fronteira:
//!
//! 1. **A espessura do traço é em unidades de MUNDO** (§4.C.6 — Enio 2026-07-17 reverteu
//!    o brush absoluto em px de tela de 2026-07-11), e o fill é assado nas MESMAS
//!    unidades: a relação entre os dois virou CONSTANTE, imune ao zoom, e a conversão que
//!    morava aqui SUMIU (ver `boundaries`). A âncora do solver segue sendo o **EIXO da
//!    linha** (BUGS #14), que é geometria pura. (O 1º corte ancorava na silhueta —
//!    congelada no zoom do clique, ela transbordava `(w/2)·(zoom−1)` px ao aproximar a
//!    câmera depois.) O `Precision` multiplica a resolução do buffer por cima disso.
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

use crate::flip_fill_dilate::{boundaries, fill_stroke};
use crate::flip_fill_target::filled_shape_target;

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
pub(crate) fn ring_area(r: &[Vec2]) -> f32 {
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
///
/// `pub(crate)` porque o pick do Edit Mode (`flip_select`) faz a MESMA pergunta ao clicar
/// dentro de uma região — e duas cópias desta regra derivariam (o render usa even-odd, e
/// um pick que discordasse dele selecionaria o que não está sob o cursor).
pub(crate) fn ring_contains(ring: &[Vec2], p: Vec2) -> bool {
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
            // (Sem `closed` aqui, pela MESMA razão do `filled_shape_target`: a forma
            // desenhada à mão é aberta, e o Unpaint tem de alcançá-la — senão a cor que o
            // balde acabou de pôr nela não sai mais.)
            let hit = drawing.strokes.iter().rposition(|s| {
                (is_fill(s) && fill_contains(s, local))
                    || (s.fill.is_some() && !s.hide_stroke && ring_contains(s.positions(), local))
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

    let strokes = boundaries(drawing);
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
        // O Trap segue a MESMA conversao do Grow (px de tela -> px de buffer): sem
        // ela, subir a Precision encolheria a bola em silencio — o acoplamento
        // escondido que o BUGS #11 pagou caro para descobrir.
        trap_px: (style.trap * style.precision) as f32,
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
    // A tolerância do "abraço": múltiplos do erro que a própria vetorização admite.
    // O `eps` do RDP vive em px de BUFFER, então em unidades de documento ele é
    // `RDP_EPSILON_PX / precision`.
    //
    // **O 8 saiu de uma varredura, não do olho** (`measure_which_criterion_separates_the_two_cases`,
    // com os números do produto): formas legítimas — quadrado, polígonos de 64 e 200
    // lados, e um contorno TREMIDO como o da mão — ficam entre **0,76 e 1,37** ε; a gota
    // que se cruza fica em **205** ε. O fosso é de 150×, e 8 deixa ~6× de folga para a
    // forma legítima sem chegar perto da quebrada.
    //
    // ⚠️ Se a grade tiver cedido resolução ao `MAX_SIDE`, o ε efetivo é MAIOR que este —
    // então a tolerância fica apertada demais e o critério recusa. Recusar é o lado
    // seguro: cai no contorno vetorizado, que é o caminho que sempre funcionou.
    const HUG_TOL_EPS: f32 = 8.0; // LITERAL-PX-OK: multiplo do eps, medido (tabela acima)
    let hug_tol = HUG_TOL_EPS * ph2d_flip_fill::RDP_EPSILON_PX / precision.max(1e-6);
    if let Some(i) = filled_shape_target(drawing, &r.outer, local, hug_tol) {
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
    // **A dilatação é LOCAL, não uma média.** Ela veste a linha que o contorno está
    // abraçando NAQUELE ponto — e num desenho com espessuras diferentes a média fica
    // entre elas, então onde o contorno abraça a linha FINA a cor era desenhada larga
    // demais e aparecia do outro lado dela (o smoke do Enio, BUGS #20). É a lição do
    // BUGS #12 outra vez: quando nenhuma constante serve, falta um DADO — aqui, QUAL
    // linha cada ponto do contorno está vestindo.
    // A LEI mora em `ph2d-flip-fill` (ver o módulo `dilate` de lá, e por quê): o oráculo
    // de pixel que a verifica vive na crate de render e **precisa alcançá-la**. Enquanto
    // ela morava aqui, ele montava a própria cópia e ficou verde durante o BUGS #20.
    //
    // ⚠️ **Nenhuma conversão de unidade aqui, e isso é de propósito.** A dilatação é
    // `w·(1+k) + 2s`: a espessura da linha, uma FRAÇÃO adimensional dela, e um desvio
    // medido na mesma unidade da geometria. Não sobrou grandeza de TELA neste caminho,
    // então não há fronteira px↔mundo para alguém esquecer de atravessar — que foi
    // exatamente o BUGS #20.
    let widths = ph2d_flip_fill::contour_widths(&strokes, &r.outer);
    let stroke = fill_stroke(&r.outer, r.holes, color, 1.0, &widths);

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
        let strip = &mut self.flip_strip;

        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.surface.size();
        let world = gfx.camera.screen_to_world((x, y), win);
        let px_to_world = gfx.camera.height_world.max(f32::EPSILON) / win.height.max(1) as f32;
        let local = w2l.apply([f64::from(world[0]), f64::from(world[1])]);
        let local = Vec2::new(local[0] as f32, local[1] as f32);

        let Some((oid, lid, did)) = crate::flip_autokey::target_drawing(
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

        // **O balde multiframe** (W7): com chaves selecionadas na tira, o MESMO clique
        // preenche todas. O pipeline roda **por quadro** (`02_referencia §11`: *"N fills
        // independentes — a região pode mudar de forma"*): a linha se move entre os
        // quadros, então o solver tem de re-traçar a região em cada um. Não há como
        // reaproveitar o contorno.
        //
        // **Falloff = 1.0 sempre** — meio-preenchimento não existe. O falloff só multiplica
        // influência de PINCEL (a regra 2 da referência), e o balde é uma op discreta.
        //
        // Os quadros vizinhos são preenchidos em SILÊNCIO (sem toast): um quadro em que a
        // região não fecha não pode derrubar o clique nos outros — o toast fala pelo quadro
        // ATIVO, que é onde o usuário está olhando.
        let extra: Vec<_> = crate::flip_multiframe::targets(
            &gfx.flip,
            oid,
            lid,
            &playhead,
            strip.selected_keys(),
            (did, frame),
            false,
        )
        .into_iter()
        .filter(|t| t.did != did)
        .collect();
        for t in extra {
            if let Some(dr) = gfx.flip.object_mut(oid).and_then(|o| o.drawing_mut(t.did)) {
                let _ = fill_click(dr, &style, local, px_to_world, &w2l);
            }
        }

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
                    // Aponta para o lado CONTRARIO do Leaked: aqui a bola e grande
                    // demais para o lugar, entao a saida e BAIXAR o Trap.
                    FillError::BallTooFat => "Fill: Trap is wider than this area — lower it",
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
pub(crate) mod tests;
