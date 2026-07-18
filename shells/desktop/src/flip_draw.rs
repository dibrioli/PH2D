//! ADR-0114 W2 T2.5/T2.6 — a interação de DESENHO do Flip no shell (o documento
//! + a interação vivem aqui, não na tool; mesmo padrão do Vector).
//!
//! `FlipDraw` acumula as amostras do traço em curso (mundo + pressão); no pen-up
//! o traço é assado num `FlipStroke` e empurrado no desenho ativo do `FlipDoc`.
//! O estilo (cor/largura/dureza/opacidade) vem do cache que o `flip_bridge`
//! publica (downcast lá, não aqui — mantém o `input_dispatch` livre de downcast).
//!
//! **1º corte (T2.6):** amostragem simples com override a <2px (evita pontos
//! redundantes); pressão→largura linear. O active smoothing (o "assentar"
//! premium) é T2.7; RDP no pen-up é T2.8.

use ph2d_core::Vec2;
use ph2d_flip::{FlipDoc, FlipDrawing, FlipStroke, LayerId, Point, Rgba};
use ph2d_flip_render::{FlipGpuData, pack_drawing};
use ph2d_tool_flip::{FlipMode, FlipStyleSnapshot};
use ph2d_vec_scene::Xform;

/// O traço do Flip em curso: amostras em MUNDO + pressão por amostra.
#[derive(Default)]
pub(crate) struct FlipDraw {
    points: Vec<Vec2>,
    pressures: Vec<f32>,
    active: bool,
}

/// Distância mínima (px de tela) entre amostras — abaixo disso o move é
/// ignorado (override), evitando pontos redundantes num pixel parado.
const MIN_SAMPLE_PX: f32 = 2.0;

impl FlipDraw {
    #[must_use]
    pub(crate) fn is_active(&self) -> bool {
        self.active
    }

    /// Começa um traço com a 1ª amostra (mundo + pressão).
    pub(crate) fn begin(&mut self, world: Vec2, pressure: f32) {
        self.points.clear();
        self.pressures.clear();
        self.points.push(world);
        self.pressures.push(pressure);
        self.active = true;
    }

    /// Adiciona uma amostra se andou ≥ `MIN_SAMPLE_PX` desde a última (medido em
    /// tela, via `px_per_world`). Devolve `true` se aceitou (pra o caller pintar).
    pub(crate) fn extend(&mut self, world: Vec2, pressure: f32, px_per_world: f32) -> bool {
        let Some(&last) = self.points.last() else {
            return false;
        };
        let d = world - last;
        let dist_px = (d.x * d.x + d.y * d.y).sqrt() * px_per_world;
        if dist_px < MIN_SAMPLE_PX {
            return false;
        }
        self.points.push(world);
        self.pressures.push(pressure);
        true
    }

    /// As amostras acumuladas (mundo, pressão) — pra o preview ao vivo.
    #[must_use]
    pub(crate) fn samples(&self) -> (&[Vec2], &[f32]) {
        (&self.points, &self.pressures)
    }

    /// Encerra o traço e devolve as amostras (mundo, pressão), limpando o estado.
    /// `None` se não há amostras suficientes (< 2 pontos = um toque, sem traço).
    pub(crate) fn take(&mut self) -> Option<(Vec<Vec2>, Vec<f32>)> {
        self.active = false;
        if self.points.len() < 2 {
            self.points.clear();
            self.pressures.clear();
            return None;
        }
        Some((
            std::mem::take(&mut self.points),
            std::mem::take(&mut self.pressures),
        ))
    }
}

/// sRGB8 → `Rgba` linear straight-alpha (o `FlipDoc` guarda linear; o picker/tool
/// dá sRGB). Transfer padrão; fora de qualquer caminho de sim (não é HR-5).
pub(crate) fn srgb8_to_linear(c: [u8; 4]) -> Rgba {
    fn ch(b: u8) -> f32 {
        let v = b as f32 / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }
    Rgba::new(ch(c[0]), ch(c[1]), ch(c[2]), c[3] as f32 / 255.0)
}

/// Assa `(points, pressures)` (mundo) num `FlipStroke` e o empurra no desenho
/// ativo do 1º objeto na CAMADA ATIVA (fallback: topo) no quadro atual. Cria uma
/// chave se o quadro ainda não tem desenho. `px_to_world` = mundo por pixel de
/// tela (a largura do brush é em px → convertida pra mundo). Uma camada TRAVADA
/// (`locked`) recusa o traço. Devolve `true` se assou.
#[allow(clippy::too_many_arguments)] // doc+playhead+estilo+camada+amostras+afim são intrínsecos
pub(crate) fn bake_stroke(
    flip: &mut FlipDoc,
    playhead: &ph2d_core::Playhead,
    style: &FlipStyleSnapshot,
    active_layer: Option<LayerId>,
    strip: &mut crate::flip_strip::FlipStrip,
    points: &[Vec2],
    pressures: &[f32],
    world_to_local: &Xform,
) -> Option<(ph2d_flip::FlipObjectId, ph2d_flip::DrawingId, usize)> {
    if points.len() < 2 {
        return None;
    }
    // **O autokey por-tool (W3.T3.4)**: quem decide o desenho-alvo — e se uma chave
    // nova nasce (em branco, ou como cópia sob *Additive*) — é o `flip_autokey`, o
    // mesmo ponto que a borracha usa. A caneta nunca resolve isso na mão.
    let (oid, _lid, did) = crate::flip_autokey::target_drawing(
        flip,
        playhead,
        active_layer,
        strip,
        crate::flip_autokey::FlipEdit::Draw,
    )?;
    let drawing = flip.object_mut(oid)?.drawing_mut(did)?;

    // Active smoothing (T2.7): assa EXATAMENTE o traço que o preview mostrou — o
    // mesmo `active_smooth`, sem decimar. O RDP do 1º corte (0.75px) deixava o
    // traço assado mais anguloso que o preview (Enio 2026-07-11: "o desenho em
    // tempo real está mais suave que o traço cosido após mouse up"); mantê-los
    // idênticos vale mais que "enxuto". As pressões seguem 1:1 (o smooth só move
    // posições). Uma decimação visualmente-perdida-zero (RDP fininho) tira só
    // pontos EXATAMENTE colineares, sem cortar curva.
    drawing.strokes.push(stroke_from_samples(
        style,
        points,
        pressures,
        world_to_local,
    ));
    Some((oid, did, drawing.strokes.len() - 1))
}

/// **A tolerância da simplificação: uma FRAÇÃO da espessura do traço.**
///
/// O amostrador guarda um ponto a cada `MIN_SAMPLE_PX` (2 px de tela), e antes de
/// 2026-07-18 a decimação usava `0,05 · px_to_world` — *"só remove colinear puro
/// (invisível)"*. Era literalmente isso: uma simplificação calibrada para não simplificar.
/// O resultado é o que o Enio viu no smoke — *"o traço gera muitos pontos muito próximos
/// e até sobrepostos"* —, e ele fica PIOR quando se desenha com a câmera perto, porque um
/// limiar em px de TELA vira uma distância minúscula em unidades de MUNDO.
///
/// A grandeza certa é adimensional, pela mesma razão do `FILL_TUCK_FRACTION`: **um desvio
/// muito menor que a própria linha é invisível por definição**, e é a linha que diz o que
/// é "muito menor". Medido num arco de mão (400 amostras, pincel default):
///
/// | fração | pontos | % do cru | desvio máx (da espessura) |
/// |---|---|---|---|
/// | 0,0008 (o valor antigo, ≈0,05 px) | 210 | 52,5 % | 0,05 % |
/// | 0,02 | 173 | 43,2 % | 1,9 % |
/// | 0,05 | 95 | 23,8 % | 5,0 % |
/// | **0,10** | **17** | **4,2 %** | **7,4 %** |
/// | 0,20 | 10 | 2,5 % | 19,5 % |
///
/// ⚠️ **O joelho da curva é 0,10 (95 → 17 pontos), e o valor escolhido é 0,05 — de
/// propósito, por causa de uma CERCA.** Em 2026-07-11 o Enio reportou *"o desenho em
/// tempo real está mais suave que o traço cosido após mouse up"*, causado por um RDP de
/// 0,75 px ≈ **12,5 % da espessura** do pincel default. O RDP substitui trechos por
/// RETAS, então tolerância demais não desloca a arte — ela a deixa **angulosa**, que é
/// outro defeito e não aparece na coluna de desvio.
///
/// 0,05 é **2,5× mais conservador** que o valor que causou a queixa e ainda assim corta
/// os pontos 4× (400 → 95, espaçamento ~8 px em vez de 2). Subir para 0,10 é um ganho
/// grande e está medido — mas é o smoke do Enio que decide, porque quem julga
/// angulosidade é o olho dele.
///
/// E o que torna qualquer valor SEGURO agora é a porta única: o preview ao vivo passa
/// pelo MESMO `stroke_from_samples`, então o traço assado é idêntico ao que ele viu
/// enquanto desenhava — a cerca de 2026-07-11 virou estrutura em vez de calibração.
///
/// Quinas sobrevivem por construção: o RDP mantém sempre o ponto de desvio MÁXIMO, e uma
/// quina é o desvio máximo do trecho.
const STROKE_SIMPLIFY_FRACTION: f32 = 0.05; // adimensional: fracao da espessura, MEDIDO

/// A tolerância em unidades de MUNDO (os pontos crus são mundo; a conversão para local é
/// do `build_stroke`, depois).
fn simplify_tolerance(style: &FlipStyleSnapshot) -> f32 {
    STROKE_SIMPLIFY_FRACTION * ph2d_tool_flip::size_to_world(style.width_px)
}

/// **Das amostras CRUAS ao traço** — smoothing + decimação invisível + estilo.
///
/// É `pub(crate)` porque os testes o dirigem direto, sem passar pelo gesto do
/// painel: mudar o Smoothing exige refazer *a partir das amostras*, não do traço assado
/// (o smoothing filtra o insumo; um traço já filtrado não tem como "desfiltrar").
pub(crate) fn stroke_from_samples(
    style: &FlipStyleSnapshot,
    points: &[Vec2],
    pressures: &[f32],
    world_to_local: &Xform,
) -> FlipStroke {
    let smoothed = crate::flip_smooth::active_smooth(points, style.smoothing);
    let keep = crate::flip_smooth::simplify_rdp(&smoothed, simplify_tolerance(style));
    let pts: Vec<Vec2> = keep.iter().map(|&i| smoothed[i]).collect();
    let prs: Vec<f32> = keep.iter().map(|&i| pressures[i]).collect();
    build_stroke(style, &pts, &prs, world_to_local)
}

/// Constrói um `FlipStroke` a partir das amostras (MUNDO) + estilo. Compartilhado
/// pelo bake (pen-up) e pelo preview ao vivo (durante o arrasto).
///
/// A largura é guardada em **unidades de MUNDO** (ADR-0114 §4.C.6 — `size_to_world` é a
/// porta única; o render multiplica por `px_per_world`, então dar zoom engrossa o traço
/// na tela, como qualquer arte). ADR-0111: a geometria é LOCAL (o gizmo pode ter movido/
/// escalado o objeto), então a largura recua pela escala do objeto
/// (`world_to_local.mean_scale`) — o render refaz `× object_scale`. Objeto não-movido =
/// `wscale=1`. Com isto, POSIÇÃO e LARGURA do `Point` ficam finalmente na MESMA unidade.
fn build_stroke(
    style: &FlipStyleSnapshot,
    points: &[Vec2],
    pressures: &[f32],
    world_to_local: &Xform,
) -> FlipStroke {
    let color = srgb8_to_linear(style.stroke);
    let wscale = world_to_local.mean_scale() as f32;
    // Size → MUNDO (porta única), recuado pela escala do objeto (ADR-0111).
    let base_w = ph2d_tool_flip::size_to_world(style.width_px) * wscale;
    let mut s = FlipStroke::new();
    for (&p, &pr) in points.iter().zip(pressures.iter()) {
        let l = world_to_local.apply([f64::from(p.x), f64::from(p.y)]);
        s.push_point(Point {
            pos: Vec2::new(l[0] as f32, l[1] as f32),
            // Pressão→largura (1º corte, linear; a curva de falloff é T2.6+).
            width: base_w * pr.clamp(0.05, 1.0),
            opacity: style.opacity,
            color,
        });
    }
    s.hardness = style.hardness;
    // **O traço PREENCHIDO** (o material stroke+fill do GP — como o Suzanne é feito):
    // o fill é a triangulação dos pontos DESTE traço, então linha e cor são UMA
    // geometria. Esculpir a linha move a cor exatamente junto, no mesmo frame — nada a
    // re-preencher, nada para ficar para trás. E ele fecha: uma forma preenchida é uma
    // forma fechada (o traço à mão quase nunca encontra a própria ponta).
    if style.draw_filled {
        s.closed = true;
        s.fill = Some(ph2d_flip::Fill {
            color: srgb8_to_linear(style.fill_color),
            opacity: 1.0,
        });
    } else {
        s.closed = false;
    }
    s
}

impl crate::App {
    /// A tool Flip quer capturar o canvas AGORA? (ativa + modo Draw). Lê o cache
    /// publicado pelo `flip_bridge` — sem downcast (o `input_dispatch` é livre).
    #[must_use]
    pub(crate) fn flip_wants_canvas(&self) -> bool {
        self.flip_active && matches!(self.flip_style.map(|s| s.mode), Some(FlipMode::Draw))
    }

    /// O afim MUNDO→LOCAL do objeto Flip ativo (o 1º). ADR-0111: o gizmo pode ter
    /// movido o objeto (geometria LOCAL + `Transform`); a mão desenha/apaga em
    /// MUNDO, então converte-se na fronteira. Identidade se o objeto nunca foi
    /// movido, sumiu, ou colapsou — caminho comum (desenho normal), no-op.
    #[must_use]
    pub(crate) fn flip_active_world_to_local(&self) -> Xform {
        // **A POSE DA CHAVE ativa entra no funil** (W7.2). A cadeia da arte é
        // `objeto ∘ pose_da_chave`, então o inverso dela é o que leva o cursor ao espaço
        // do DESENHO — onde a geometria vive. Sem isto, desenhar/esculpir/preencher numa
        // chave deslocada erraria pelo tanto do deslocamento: o usuário aponta para o que
        // VÊ, e o que ele vê já está posado.
        //
        // A pose sai do MESMO amostrador que o render usa (`offset_at_cycled`) — seed e
        // sample são a mesma função (`feedback_derived_coordinate_seed_must_match_sample`).
        crate::flip_transform::world_to_art(
            &self.flip_active_object_xform(),
            self.flip_active_pose(),
        )
    }

    /// O afim LOCAL(objeto)→MUNDO do objeto Flip ativo — **sem a pose da chave**. É a
    /// cadeia do `Transform` do ECS (o gizmo), e nada mais.
    #[must_use]
    fn flip_active_object_xform(&self) -> Xform {
        let Some(gfx) = self.gfx.as_ref() else {
            return Xform::IDENTITY;
        };
        let Some(oid) = gfx.flip.objects().first().map(|o| o.id) else {
            return Xform::IDENTITY;
        };
        self.flip_entities
            .get(&oid)
            .map(|&bits| ph2d_ecs::Entity::from_bits(bits))
            .filter(|e| gfx.sim.world().get_entity(*e).is_ok())
            .map_or(Xform::IDENTITY, |e| {
                crate::flip_transform::object_xform(&gfx.sim, e)
            })
    }

    /// A pose (afim) da chave que está NA TELA agora — a MESMA que o render dobra
    /// (`pose_at_cycled`, W7.2).
    #[must_use]
    fn flip_active_pose(&self) -> ph2d_flip::Pose {
        let Some(gfx) = self.gfx.as_ref() else {
            return ph2d_flip::Pose::IDENTITY;
        };
        let Some(obj) = gfx.flip.objects().first() else {
            return ph2d_flip::Pose::IDENTITY;
        };
        let frame = obj.frame_at(&self.playhead);
        self.flip_active_layer
            .filter(|id| obj.layer(*id).is_some())
            .or_else(|| obj.layers().last().map(|l| l.id))
            .and_then(|lid| obj.layer(lid))
            .map_or(ph2d_flip::Pose::IDENTITY, |l| l.pose_at_cycled(frame))
    }

    /// O afim MUNDO→LOCAL(objeto) **sem a pose da chave** — o funil do gesto de MOVER.
    ///
    /// Por que este não pode ter a pose: mover uma instância ESCREVE a pose, e a pose
    /// entra no [`Self::flip_active_world_to_local`]. Usar aquele funil aqui criaria um
    /// laço — cada amostra converte o cursor num referencial que a amostra anterior
    /// acabou de mover — e o desenho **treme** (smoke do Enio, 2026-07-14). O delta é um
    /// VETOR (uma diferença de dois pontos); a translação da pose se cancela nele, então
    /// tirar a pose não muda o resultado no caso comum e **elimina o laço** no instanciado.
    #[must_use]
    pub(crate) fn flip_active_world_to_object(&self) -> Xform {
        self.flip_active_object_xform()
            .inverse()
            .unwrap_or(Xform::IDENTITY)
    }

    /// Pen-down do desenho Flip: começa um traço na coord de mundo. Devolve
    /// `true` se consumiu (a tool está desenhando) — o caller não deixa cair no
    /// gizmo/pick.
    pub(crate) fn flip_canvas_down(&mut self, x: f32, y: f32) -> bool {
        if !self.flip_wants_canvas() {
            return false;
        }
        let Some(gfx) = self.gfx.as_ref() else {
            return false;
        };
        let win = gfx.surface.size();
        let w = gfx.camera.screen_to_world((x, y), win);
        self.flip_draw.begin(Vec2::new(w[0], w[1]), 1.0);
        true
    }

    /// Move enquanto desenha: adiciona uma amostra (override a <2px). Devolve
    /// `true` se um traço está em curso (consome o move).
    pub(crate) fn flip_canvas_move(&mut self, x: f32, y: f32) -> bool {
        if !self.flip_draw.is_active() {
            return false;
        }
        let Some(gfx) = self.gfx.as_ref() else {
            return false;
        };
        let win = gfx.surface.size();
        let w = gfx.camera.screen_to_world((x, y), win);
        let px_per_world = win.height.max(1) as f32 / gfx.camera.height_world.max(f32::EPSILON);
        self.flip_draw
            .extend(Vec2::new(w[0], w[1]), 1.0, px_per_world);
        true
    }

    /// GPU-data do traço em curso pro **preview ao vivo** (renderizado por cima do
    /// composite a cada frame; vira documento só no pen-up). `None` quando não há
    /// gesto ou < 2 amostras.
    #[must_use]
    pub(crate) fn flip_preview_data(&self) -> Option<FlipGpuData> {
        if !self.flip_draw.is_active() {
            return None;
        }
        let style = self.flip_style?;
        // O preview é dobrado na fatia da camada ativa (espaço LOCAL do objeto); as
        // amostras são MUNDO → converte, senão o preview folga do traço final. A
        // largura é px de tela ABSOLUTO (o render não escala pelo zoom) → sem câmera.
        let w2l = self.flip_active_world_to_local();
        let (pts, prs) = self.flip_draw.samples();
        if pts.len() < 2 {
            return None;
        }
        // **A MESMA porta do bake** — não "o mesmo smoothing", a mesma FUNÇÃO.
        //
        // Antes o preview repetia só o `active_smooth` e o bake acrescentava um RDP; os
        // dois só coincidiam porque esse RDP estava calibrado para não fazer nada
        // (tolerância de 0,05 px). Ou seja: o invariante *"o preview mostra o traço
        // final"* era mantido **castrando** um dos lados, e qualquer simplificação de
        // verdade o quebrava em silêncio — foi exatamente o que o Enio reportou em
        // 2026-07-11 (*"o desenho em tempo real está mais suave que o traço cosido"*).
        // Compartilhando a função, ele passa a valer por CONSTRUÇÃO.
        let mut d = FlipDrawing::default();
        d.strokes.push(stroke_from_samples(&style, pts, prs, &w2l));
        Some(pack_drawing(&d))
    }

    /// Pen-up: assa o traço acumulado no `FlipDoc`. Devolve `true` se um gesto
    /// estava em curso (consome o Up, mesmo que um toque simples não vire traço).
    pub(crate) fn flip_canvas_up(&mut self) -> bool {
        if !self.flip_draw.is_active() {
            return false;
        }
        let Some((points, pressures)) = self.flip_draw.take() else {
            return true; // toque simples (<2 pontos): consumido, sem traço
        };
        let style = self.flip_style;
        let active_layer = self.flip_active_layer;
        // Fronteira MUNDO→LOCAL (ADR-0111): num objeto já movido pelo gizmo o traço
        // é guardado no espaço local dele. Identidade num objeto novo (o comum).
        let w2l = self.flip_active_world_to_local();
        let playhead = self.playhead;
        let strip_ref = &mut self.flip_strip;
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(style) = style
        {
            // O traço é assado e ACABOU. (Ele já foi o "alvo vivo" — os controles do
            // painel continuavam reescrevendo o último traço até o usuário fazer outra
            // coisa. O Enio mandou parar com isso em 2026-07-18: um traço desenhado é um
            // FATO, não uma pré-visualização que os sliders continuam editando.)
            bake_stroke(
                &mut gfx.flip,
                &playhead,
                &style,
                active_layer,
                strip_ref,
                &points,
                &pressures,
                &w2l,
            );
        }
        true
    }
}

#[cfg(test)]
#[path = "flip_draw_tests.rs"]
mod tests;
