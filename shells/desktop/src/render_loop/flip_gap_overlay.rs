//! **O overlay dos helpers do Gap Closure** (doc `06 §8`): em modo Fill, cada vão que o
//! alcance atual fecha ganha um segmento no canvas — o artista VÊ o que o clique vai
//! selar, e ajusta o alcance com Ctrl+roda sem largar o mouse.
//!
//! **É chrome, não arte** (a regra de todo overlay do Flip — ver `flip_tween_overlay`):
//! a geometria sobe para px de TELA e o `stroke` desenha sob `Affine::IDENTITY`, porque
//! no Vello o transform do `stroke` MULTIPLICA a espessura. A cadeia arte→tela é a MESMA
//! que o render dobra (`câmera ∘ objeto ∘ pose_da_chave` — o `screen_affine` do tween).
//!
//! **Só os vãos PENDENTES** (`GapHelper::pending`): a killer feature do GP é *"helpers
//! visíveis só nos gaps pendentes"*. Onde a tinta que o artista pintou já cobre o vão (as
//! junções de traços que se sobrepõem), a solda das juntas já veda e o line-art está
//! fechado — um helper ali seria a tela apontando um vão que não existe, e de perto fica
//! gritante (a linha branca se vê contínua). O motor do fill sela tudo de qualquer jeito;
//! o overlay só desenha o que ainda está ABERTO.
//!
//! **Duas camadas, porque o motor sela um vão de dois jeitos** (`GapHelper`): um **par
//! ponta-a-ponta** é a PONTE limpa de um vão (as duas pontas são fatos do desenho) e uma
//! **extensão** é uma ponta esticada na tangente até bater numa parede (a outra ponta é
//! um ponto de CORTE, arbitrário). A ponte é a resposta que o artista lê — verde cheio
//! com marcador nas duas pontas; a extensão é o mecanismo — um fio fino, e o marcador só
//! na ponta REAL (*as pontas são o FATO, o segmento é a promessa*). Um dot no ponto de
//! corte era o "dot flutuante" do smoke: um nó que não existe.
//!
//! Quem computa os helpers é o worker (`flip_gap_live.rs` — o custo foi MEDIDO em
//! 5-339 ms, ver `measure_closures.rs`); aqui só se projeta o que está instalado. É por
//! estarem em coords de ARTE que ficam colados no desenho durante zoom/pan enquanto o
//! resultado novo não chega.

use ph2d_flip::Pose;
use ph2d_flip_fill::GapHelper;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vec_scene::Xform;
use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Stroke, VectorScene};

use crate::flip_tween_correct::screen_affine;

/// Espessura da PONTE (o par ponta-a-ponta), em px de tela — a resposta, com peso.
const BRIDGE_PX: f64 = 2.0; // LITERAL-PX-OK: chrome de overlay, espessura de tela
/// Espessura do FIO de extensão, em px de tela — o mecanismo, mais leve que a ponte para
/// não competir com ela na leitura.
const WHISKER_PX: f64 = 1.0; // LITERAL-PX-OK: chrome de overlay, espessura de tela
/// Raio do marcador numa ponta REAL, em px de tela.
const TIP_DOT_PX: f64 = 2.5; // LITERAL-PX-OK: chrome de overlay, raio de tela
/// Verde "vai fechar" — o matiz de ação positiva; nenhum outro overlay do Flip o usa
/// para geometria de canvas (o verde do tween é a COR DE LINHA da confiança de par, em
/// outra sessão). A ponte é opaca; o fio é o mesmo verde translúcido (o mecanismo cede).
const BRIDGE_RGBA: [f32; 4] = [0.35, 0.9, 0.45, 0.95]; // LITERAL-COLOR-OK: overlay de helper
const WHISKER_RGBA: [f32; 4] = [0.35, 0.9, 0.45, 0.5]; // LITERAL-COLOR-OK: overlay de helper

/// Desenha os helpers instalados. `active` = a pergunta do modo, respondida pela MESMA
/// porta do tick ([`crate::flip_gap_live::wants_gap_helpers`]) — o caller a passa
/// resolvida para este módulo não re-derivar política.
pub(super) fn draw(
    active: bool,
    helpers: &[GapHelper],
    l2w: &Xform,
    pose: Pose,
    camera: &Camera2d,
    window: WindowSize,
    vector_scene: &mut VectorScene,
) {
    if !active || helpers.is_empty() {
        return;
    }
    let aff = screen_affine(l2w, pose, camera.world_to_screen_affine(window));
    let scene = vector_scene.inner_mut();
    for h in helpers {
        // Só os vãos ABERTOS — onde a tinta já cobre, a solda vedou e não há o que mostrar.
        if !h.pending {
            continue;
        }
        let a = aff * Point::new(f64::from(h.seg.a.x), f64::from(h.seg.a.y));
        let b = aff * Point::new(f64::from(h.seg.b.x), f64::from(h.seg.b.y));
        // Ponte = as DUAS pontas reais; qualquer outra coisa é um fio de extensão.
        let bridge = h.a_is_tip && h.b_is_tip;
        let (w, rgba) = if bridge {
            (BRIDGE_PX, BRIDGE_RGBA)
        } else {
            (WHISKER_PX, WHISKER_RGBA)
        };
        let mut path = BezPath::new();
        path.move_to(a);
        path.line_to(b);
        scene.stroke(
            &Stroke::new(w),
            Affine::IDENTITY,
            &Brush::Solid(Color::new(rgba)),
            None,
            &path,
        );
        // O marcador SÓ numa ponta real — nunca num ponto de corte (o nó que não existe).
        for (p, is_tip) in [(a, h.a_is_tip), (b, h.b_is_tip)] {
            if is_tip {
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(Color::new(BRIDGE_RGBA)),
                    None,
                    &Circle::new(p, TIP_DOT_PX),
                );
            }
        }
    }
}
