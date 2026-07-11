//! ADR-0113 W2 T2.5/T2.6 — a interação de DESENHO do Flip no shell (o documento
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
use ph2d_flip::{FlipDoc, FlipDrawing, FlipStroke, Hold, KeyKind, Point, Rgba};
use ph2d_flip_render::{FlipGpuData, pack_drawing};
use ph2d_tool_flip::{FlipMode, FlipStyleSnapshot};

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
fn srgb8_to_linear(c: [u8; 4]) -> Rgba {
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
/// ativo do 1º objeto (camada de topo) no quadro atual. Cria uma chave se o
/// quadro ainda não tem desenho. `px_to_world` = mundo por pixel de tela (a
/// largura do brush é em px → convertida pra mundo). Devolve `true` se assou.
pub(crate) fn bake_stroke(
    flip: &mut FlipDoc,
    playhead: &ph2d_core::Playhead,
    style: &FlipStyleSnapshot,
    points: &[Vec2],
    pressures: &[f32],
    px_to_world: f32,
) -> bool {
    if points.len() < 2 {
        return false;
    }
    let Some(oid) = flip.objects().first().map(|o| o.id) else {
        return false;
    };
    let Some(obj) = flip.object_mut(oid) else {
        return false;
    };
    // Camada de topo (última do slice). Sem UI de camadas ainda (T2.15), é onde o
    // traço vai.
    let Some(layer_id) = obj.layers().last().map(|l| l.id) else {
        return false;
    };
    let frame = obj.frame_at(playhead);
    // Desenho ativo no quadro; se não há (antes da 1ª chave / sentinela), cria uma.
    let did = match obj.layer(layer_id).and_then(|l| l.drawing_at(frame)) {
        Some(d) => d,
        None => match obj.insert_frame(layer_id, frame, Hold::Implicit, KeyKind::Keyframe) {
            Some(d) => d,
            None => return false,
        },
    };
    let Some(drawing) = obj.drawing_mut(did) else {
        return false;
    };

    // Active smoothing (T2.7): o mesmo que o preview mostrou (o baked casa com o
    // que se via). As pressões seguem 1:1 (o smooth só move posições).
    let smoothed = crate::flip_smooth::active_smooth(points, style.smoothing);
    drawing
        .strokes
        .push(build_stroke(style, &smoothed, pressures, px_to_world));
    true
}

/// Constrói um `FlipStroke` a partir das amostras (mundo) + estilo. Compartilhado
/// pelo bake (pen-up) e pelo preview ao vivo (durante o arrasto).
fn build_stroke(
    style: &FlipStyleSnapshot,
    points: &[Vec2],
    pressures: &[f32],
    px_to_world: f32,
) -> FlipStroke {
    let color = srgb8_to_linear(style.stroke);
    let base_w = (style.width_px as f32) * px_to_world; // px de tela → mundo
    let mut s = FlipStroke::new();
    for (&p, &pr) in points.iter().zip(pressures.iter()) {
        s.push_point(Point {
            pos: p,
            // Pressão→largura (1º corte, linear; a curva de falloff é T2.6+).
            width: base_w * pr.clamp(0.05, 1.0),
            opacity: style.opacity,
            color,
        });
    }
    s.hardness = style.hardness;
    s.closed = false;
    s
}

impl crate::App {
    /// A tool Flip quer capturar o canvas AGORA? (ativa + modo Draw). Lê o cache
    /// publicado pelo `flip_bridge` — sem downcast (o `input_dispatch` é livre).
    #[must_use]
    pub(crate) fn flip_wants_canvas(&self) -> bool {
        self.flip_active && matches!(self.flip_style.map(|s| s.mode), Some(FlipMode::Draw))
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
        let gfx = self.gfx.as_ref()?;
        let (pts, prs) = self.flip_draw.samples();
        if pts.len() < 2 {
            return None;
        }
        let win = gfx.surface.size();
        let px_to_world = gfx.camera.height_world.max(f32::EPSILON) / win.height.max(1) as f32;
        // Mesmo active smoothing do bake → o preview mostra o traço FINAL.
        let smoothed = crate::flip_smooth::active_smooth(pts, style.smoothing);
        let mut d = FlipDrawing::default();
        d.strokes
            .push(build_stroke(&style, &smoothed, prs, px_to_world));
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
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(style) = style
        {
            let win = gfx.surface.size();
            let px_to_world = gfx.camera.height_world.max(f32::EPSILON) / win.height.max(1) as f32;
            bake_stroke(
                &mut gfx.flip,
                &self.playhead,
                &style,
                &points,
                &pressures,
                px_to_world,
            );
        }
        true
    }
}
