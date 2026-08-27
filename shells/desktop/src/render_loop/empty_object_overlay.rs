//! ⭐ **O CÍRCULO de um objeto vazio** (Enio, 2026-08-26: *«para o objeto vazio precisamos de um
//! gizmo simples — um círculo simples de um tamanho razoável»*).
//!
//! Um objeto sem geometria não emite `RenderInstance` nenhuma: ele existe na Hierarquia, tem pose,
//! tem filhos — e **não há um pixel dele na tela**. Sem uma marca o artista não sabe onde ele está,
//! e é a mesma lição que o realce do Flip pagou: *o que não se vê não existe*.
//!
//! # ⚠️ Ele NÃO segue a seleção — o report de 2026-08-26
//!
//! > *«Se desseleciono o objeto vazio, o círculo some. O círculo só pode sumir no runtime.»*
//!
//! A 1.ª versão desenhava-o só para o selecionado, e a razão escrita era *«a marca serve o gesto
//! que está a acontecer»*. Estava errada: o anel **é o corpo** de um objeto que não tem pixels — a
//! mesma coisa que o quad é para uma sprite —, e um corpo que só existe enquanto se olha para ele
//! não é um corpo. ⛔ É também por isso que **ter filhos não o apaga**: um grupo continua a ser um
//! objeto que o artista tem de conseguir ver e pegar pelo centro.
//!
//! ⇒ ele desaparece em **duas** situações, as duas por não estar na cena: o olho fechado
//! (`Visibility`) e ser peça de uma **receita**. E há uma terceira, quando existir: o modo de jogo,
//! que não pinta chrome nenhum (o `shells/game`/R1 está adiado).
//!
//! # ⛔ E ele NÃO esmaece fora da seleção
//!
//! A 1.ª versão pintava o anel do não-selecionado a `alpha = 0,35`, *«senão uma cena com seis
//! grupos leria como seis coisas selecionadas»*. O smoke devolveu-o em duas palavras: **«quase
//! invisível»**. ⚠️ O argumento estava certo sobre o problema e errado sobre o remédio — *a
//! seleção já é dita pela CAIXA e pelas oito alças à volta*, que é muito mais tinta que meio tom
//! num traço de 1,5 px. Dizer a mesma coisa duas vezes com o único canal que este anel tem só
//! apaga o corpo do objeto. ⇒ **todos os anéis a peso cheio**.
//!
//! # ⚠️ A pergunta é feita UMA vez
//!
//! *«Este objeto é um vazio?»* e *«que raio tem o anel dele?»* são respondidas por
//! [`crate::group_gizmo_view`] — as **mesmas** funções que o dedo usa
//! ([`crate::group_gizmo_view::pick_empty_at_world`]). Uma segunda leitura aqui seria uma segunda
//! opinião, e o anel acabaria pintado num sítio e agarrável noutro.

use ph2d_ecs::SimWorld;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, Circle, Stroke, VectorScene};

/// Espessura do anel, em px de TELA.
///
/// ⚠️ Ela sai daqui sob `Affine::IDENTITY`: no Vello o transform de um `stroke` **multiplica** a
/// espessura, então entregar o afim mundo→tela transformaria 1,5 px em `1,5 × px_por_metro`. É o
/// defeito que o realce do Flip apanhou num smoke em 2026-07-13.
const RING_PX: f64 = 1.5; // LITERAL-PX-OK: chrome de overlay, espessura de tela

/// **Desenha o anel de TODO objeto vazio da cena.**
pub(super) fn draw_empty_object_marks(
    sim: &SimWorld,
    pixels_per_meter: f32,
    theme: Theme,
    camera: &Camera2d,
    window: WindowSize,
    vector_scene: &mut VectorScene,
) {
    let ppm = pixels_per_meter.max(crate::EPS_PIXELS_PER_METER);
    let color = ph2d_editor::paint::resolve(ColorToken::Selection, theme);
    for entity in crate::group_gizmo_view::empty_objects(sim) {
        let c = crate::vec_transform::world_transform(sim, entity).translation;
        let r_world = crate::group_gizmo_view::marker_world_radius(sim, entity, ppm);
        // ⚠️ **O raio é MEDIDO na tela, e não convertido à mão**: o anel tem de crescer com o zoom
        // exatamente como a caixa do gizmo cresce, e a única coisa que sabe a conversão é a câmara.
        // Um `raio × zoom` escrito aqui seria a segunda régua, e ela divergiria no primeiro pan.
        let (sx, sy) = camera.world_to_screen([c.x, c.y], window);
        let (ex, ey) = camera.world_to_screen([c.x + r_world, c.y], window);
        let r = f64::from(((ex - sx).powi(2) + (ey - sy).powi(2)).sqrt());
        if !(r.is_finite() && r > 0.0) {
            continue;
        }
        vector_scene.inner_mut().stroke(
            &Stroke::new(RING_PX),
            Affine::IDENTITY,
            color,
            None,
            &Circle::new((f64::from(sx), f64::from(sy)), r),
        );
    }
}
