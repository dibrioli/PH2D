//! ADR-0114 **Tween v2 — o overlay da correção de pares** (`docs/Flip/11 §6`).
//!
//! A lição CACAni: o matcher erra, e o artista precisa VER *quem vira quem* para corrigir.
//! Este overlay é essa visão — os dois desenhos-chave sobrepostos (A frio, B quente), uma
//! linha ligando cada par, pintada pela CONFIANÇA da correspondência.
//!
//! **É chrome, não arte** (como todo overlay do Flip — ver `flip_selection_overlay`): a
//! geometria sai daqui já em **px de TELA** e o `stroke` desenha sob `Affine::IDENTITY`,
//! porque no Vello o transform do `stroke` MULTIPLICA a espessura. A cadeia arte→tela é a
//! MESMA que o render dobra (`câmera ∘ objeto ∘ pose_da_chave`), e A e B carregam poses
//! diferentes — cada lado tem seu afim ([`crate::flip_tween_correct::screen_affine`]).

use ph2d_flip::FlipStroke;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vec_scene::Xform;
use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Stroke, VectorScene};

use crate::flip_tween_correct::{Side, TweenCorrect, screen_affine, stroke_centroid};

/// Espessura do contorno de A/B, em px de tela — fino: é referência, não a arte final.
const OUTLINE_PX: f64 = 1.5; // LITERAL-PX-OK: chrome de overlay, espessura de tela
/// Espessura da linha de par, em px de tela.
const LINK_PX: f64 = 1.5; // LITERAL-PX-OK: chrome de overlay, espessura de tela
/// Espessura do traço SELECIONADO (o 1º clique de um re-par), em px de tela — grosso, acesa.
const SEL_PX: f64 = 3.0; // LITERAL-PX-OK: chrome de overlay, espessura de tela
/// Raio do ponto na ponta de uma linha de par, em px de tela.
const END_DOT_PX: f64 = 2.5; // LITERAL-PX-OK: chrome de overlay, raio de tela
/// Raio do anel de ÓRFÃO, em px de tela (um traço que some/nasce no tween).
const ORPHAN_R_PX: f64 = 5.0; // LITERAL-PX-OK: chrome de overlay, raio de tela

/// A — o desenho de PARTIDA, azul frio (o "de onde").
const A_RGBA: [f32; 4] = [0.45, 0.66, 1.0, 0.65]; // LITERAL-COLOR-OK: overlay de correção de pares
/// B — o desenho de CHEGADA, laranja quente (o "para onde").
const B_RGBA: [f32; 4] = [1.0, 0.62, 0.25, 0.65]; // LITERAL-COLOR-OK: overlay de correção de pares

/// Par de baixo custo — casou com confiança (verde).
const LINK_GOOD: [f32; 4] = [0.35, 0.9, 0.45, 0.95]; // LITERAL-COLOR-OK: overlay de correção de pares
/// Par de alto custo — duvidoso (vermelho): o candidato a corrigir.
const LINK_BAD: [f32; 4] = [1.0, 0.35, 0.3, 0.98]; // LITERAL-COLOR-OK: overlay de correção de pares
/// Par CORRIGIDO à mão (âmbar) — sem pontuação de matcher, é a escolha do artista.
const LINK_MANUAL: [f32; 4] = [1.0, 0.78, 0.15, 0.98]; // LITERAL-COLOR-OK: overlay de correção de pares
/// Órfão — magenta, o matiz que nenhum par usa: um traço que some (A) ou nasce (B).
const ORPHAN_RGBA: [f32; 4] = [0.95, 0.4, 0.9, 0.9]; // LITERAL-COLOR-OK: overlay de correção de pares
/// O traço marcado (aguardando o 2º clique) — branco, o acento máximo.
const SEL_RGBA: [f32; 4] = [1.0, 1.0, 1.0, 0.98]; // LITERAL-COLOR-OK: overlay de correção de pares

/// **A cor da linha de par pela confiança:** manual (sem custo) = âmbar; senão verde→vermelho
/// interpolando o custo em `[0, PAIR_REJECT_COST]` (o teto além do qual o motor recusaria o
/// par). Sem transcendental (HR-5).
#[must_use]
pub(super) fn link_color(cost: Option<f32>) -> [f32; 4] {
    let Some(c) = cost else {
        return LINK_MANUAL;
    };
    let t = (c / ph2d_flip::PAIR_REJECT_COST).clamp(0.0, 1.0);
    // Endpoints EXATOS (a interpolação em f32 não devolve `b` ao bit em `t=1`): custo 0 é
    // verde puro, no teto é vermelho puro. É também a saturação certa — um custo acima do
    // teto não "estoura" a cor, para no vermelho.
    if t <= 0.0 {
        return LINK_GOOD;
    }
    if t >= 1.0 {
        return LINK_BAD;
    }
    [
        LINK_GOOD[0] + (LINK_BAD[0] - LINK_GOOD[0]) * t,
        LINK_GOOD[1] + (LINK_BAD[1] - LINK_GOOD[1]) * t,
        LINK_GOOD[2] + (LINK_BAD[2] - LINK_GOOD[2]) * t,
        LINK_GOOD[3] + (LINK_BAD[3] - LINK_GOOD[3]) * t,
    ]
}

/// A polilinha de um traço, **já em px de TELA** (o afim é aplicado aos PONTOS — ver o
/// cabeçalho: o Vello multiplicaria a espessura se ele fosse o transform).
fn stroke_path(s: &FlipStroke, aff: Affine) -> BezPath {
    let mut path = BezPath::new();
    for (i, p) in s.positions().iter().enumerate() {
        let pt = aff * Point::new(f64::from(p.x), f64::from(p.y));
        if i == 0 {
            path.move_to(pt);
        } else {
            path.line_to(pt);
        }
    }
    if s.closed && s.len() >= 3 {
        path.close_path();
    }
    path
}

/// O centróide de um traço em px de tela — a âncora da linha de par.
fn centroid_screen(s: &FlipStroke, aff: Affine) -> Point {
    let c = stroke_centroid(s);
    aff * Point::new(f64::from(c.x), f64::from(c.y))
}

/// Desenha a correspondência do intervalo pinado na sessão. No-op fora do modo Pairs.
///
/// `l2w` é o afim LOCAL→mundo do objeto (a pose do gizmo), o MESMO que o realce de seleção
/// usa; a pose da CHAVE (A ou B) entra por cima dele em [`screen_affine`].
pub(super) fn draw(
    active: bool,
    tc: Option<&TweenCorrect>,
    l2w: &Xform,
    camera: &Camera2d,
    window: WindowSize,
    vector_scene: &mut VectorScene,
) {
    if !active {
        return;
    }
    let Some(tc) = tc else {
        return;
    };
    let cam = camera.world_to_screen_affine(window);
    let aff_a = screen_affine(l2w, tc.pose_a, cam);
    let aff_b = screen_affine(l2w, tc.pose_b, cam);

    let stroke_it = |scene: &mut VectorScene, path: &BezPath, w: f64, rgba: [f32; 4]| {
        scene.inner_mut().stroke(
            &Stroke::new(w),
            Affine::IDENTITY,
            &Brush::Solid(Color::new(rgba)),
            None,
            path,
        );
    };
    let dot = |scene: &mut VectorScene, c: Point, r: f64, rgba: [f32; 4]| {
        scene.inner_mut().fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(Color::new(rgba)),
            None,
            &Circle::new(c, r),
        );
    };

    // 1) Os dois desenhos, por baixo de tudo (referência): A frio, B quente.
    for s in &tc.a.strokes {
        stroke_it(vector_scene, &stroke_path(s, aff_a), OUTLINE_PX, A_RGBA);
    }
    for s in &tc.b.strokes {
        stroke_it(vector_scene, &stroke_path(s, aff_b), OUTLINE_PX, B_RGBA);
    }

    // 2) As linhas de par, pintadas pela confiança — com um ponto em cada ponta (onde a
    //    linha prende, para o par não parecer flutuar entre os traços).
    for (i, sa) in tc.a.strokes.iter().enumerate() {
        let Some(j) = tc.plan.pair_of_a(i) else {
            continue;
        };
        let Some(sb) = tc.b.strokes.get(j) else {
            continue;
        };
        let pa = centroid_screen(sa, aff_a);
        let pb = centroid_screen(sb, aff_b);
        let rgba = link_color(tc.plan.cost_of_a(i));
        let mut link = BezPath::new();
        link.move_to(pa);
        link.line_to(pb);
        stroke_it(vector_scene, &link, LINK_PX, rgba);
        dot(vector_scene, pa, END_DOT_PX, rgba);
        dot(vector_scene, pb, END_DOT_PX, rgba);
    }

    // 3) Os órfãos — um anel magenta no centróide de cada traço sem par (some, em A; nasce,
    //    em B). É o erro mais caro do tween (um traço que pisca), então ele se anuncia.
    for (i, sa) in tc.a.strokes.iter().enumerate() {
        if tc.plan.pair_of_a(i).is_none() {
            orphan_ring(vector_scene, centroid_screen(sa, aff_a));
        }
    }
    for (j, sb) in tc.b.strokes.iter().enumerate() {
        if tc.plan.pair_of_b(j).is_none() {
            orphan_ring(vector_scene, centroid_screen(sb, aff_b));
        }
    }

    // 4) O traço MARCADO (o 1º clique de um re-par), por cima de tudo, aceso.
    if let Some(sel) = tc.pending {
        let (s, aff) = match sel.side {
            Side::A => (tc.a.strokes.get(sel.idx), aff_a),
            Side::B => (tc.b.strokes.get(sel.idx), aff_b),
        };
        if let Some(s) = s {
            stroke_it(vector_scene, &stroke_path(s, aff), SEL_PX, SEL_RGBA);
        }
    }
}

/// O anel de órfão (contorno, não preenchido — distinto do ponto sólido da ponta do par).
fn orphan_ring(scene: &mut VectorScene, c: Point) {
    scene.inner_mut().stroke(
        &Stroke::new(OUTLINE_PX),
        Affine::IDENTITY,
        &Brush::Solid(Color::new(ORPHAN_RGBA)),
        None,
        &Circle::new(c, ORPHAN_R_PX),
    );
}

#[cfg(test)]
#[path = "flip_tween_overlay_tests.rs"]
mod tests;
