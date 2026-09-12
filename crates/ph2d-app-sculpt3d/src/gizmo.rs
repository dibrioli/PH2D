//! ⭐⭐⭐ **O GIZMO DE TRANSFORMAÇÃO DA ESCULTURA** — as alças que se agarram.
//!
//! Ordem do Enio, 2026-09-08: *«veja o módulo de Modelagem 3d. Ele tem
//! viewports (4), gizmos de transformação e o gizmo da viewport. Traga esses
//! features para esse módulo.»*
//!
//! # ⛔⛔ O que este módulo TIRA: uma cerca de 2026-08
//!
//! O [`crate::transform`] abre com a secção *«Por que não há gizmo»*, e
//! a razão escrita ali era exacta para o dia: *«o gizmo deste repo é o de SPRITE
//! (ADR-0110/0111), 2D, e um gizmo 3D é wave própria e grande»*. O substituto —
//! o *modal transform*, em que a ferramenta é armada e o arrasto a executa —
//! **fica**, e não é um compromisso: é a outra metade do desenho do Blender.
//!
//! ⚠️ **A premissa dissolveu-se porque um vizinho pagou a wave:** o módulo de
//! modelagem 3D tem um gizmo de treze alças desde a W26, e ele é **lei pura**.
//! ⇒ §0.0 desta casa: *quem move o número que tornava algo inalcançável tem de
//! reconferir a nota.*
//!
//! # ⭐⭐ E o que ele ACRESCENTA não é chrome: são gestos que não existiam
//!
//! O transform modal entrega **três** gestos — mover no plano da tela, rodar em
//! torno da vista, escalar. O kernel ([`ph2d_sculpt3d::Gesture`]) sempre soube
//! fazer mais: `Move { delta }` aceita qualquer vector e `Rotate { axis, radians }`
//! qualquer eixo. O que faltava era **como pedir**: sem alças, mover só ao longo
//! de `X` ou rodar só em torno de `Y` eram **inexprimíveis por gesto nenhum** — a
//! mesma forma do buraco que o picker de filtros curou na W9b, em que três das
//! nove leis não tinham como ser pedidas.
//!
//! # A lei é a do vizinho, com esta câmera
//!
//! Onde as alças caem, qual está sob o cursor, qual degenera de perfil e como se
//! pintam já é [`ph2d_viewport3d::gizmo`] — e ele passou a receber a câmera por um
//! [`GizmoCamera`] de três perguntas em vez de conhecer a `Orbit`. *Copiar as
//! ~130 linhas de projecção daria duas ideias de onde uma alça está, e a que
//! envelhecesse agarraria ao lado do que diz mover.*

use super::Sculpt3dScene;
use ph2d_mesh_render::Camera3d;
use ph2d_sculpt3d::{Gesture, TransformKind};
use ph2d_viewport3d::gizmo::{Anchor, GizmoCamera, Handle, Mode, Projected};

/// A câmera desta cena no vocabulário do gizmo, **em coordenadas de JANELA**.
///
/// ⚠️ **A quina do viewport entra aqui e em mais lado nenhum.** As alças são
/// pintadas e apontadas em coordenadas de janela (é onde o ponteiro vive) e a
/// [`Camera3d`] projecta no referencial da vista — somar a quina em cada sítio
/// que projecta seria a mesma linha escrita cinco vezes, e a que faltasse daria
/// um gizmo deslocado só fora do quadrante de cima à esquerda.
struct SculptCam {
    cam: Camera3d,
    viewport: (u32, u32),
    origin: (f32, f32),
}

impl GizmoCamera for SculptCam {
    fn project_px(&self, p: [f32; 3]) -> Option<[f32; 2]> {
        let (x, y) = self.cam.project(p, self.viewport)?;
        Some([x + self.origin.0, y + self.origin.1])
    }

    /// ⚠️ **O INVERSO da porta que já existe**, e não uma conta nova: a
    /// [`Camera3d::world_radius_for_screen_px`] responde *«quanto mundo vale
    /// este tanto de pixels ali»*, que é exactamente o recíproco. Derivar a
    /// escala de outra forma faria o braço do gizmo e o raio do pincel medirem
    /// coisas diferentes na mesma profundidade.
    fn px_per_world(&self, at: [f32; 3]) -> f32 {
        let mundo_por_px = self
            .cam
            .world_radius_for_screen_px(at, 1.0, self.viewport)
            .max(1e-9);
        1.0 / mundo_por_px
    }

    fn fwd(&self) -> [f32; 3] {
        self.cam.view_axis().into()
    }
}

/// O `Mode` do gizmo que corresponde ao verbo armado.
///
/// ⚠️ **A tradução é total e não tem `_ =>`**: um `TransformKind` novo tem de
/// dizer que alças oferece, senão ele nasce a mostrar as de mover.
fn mode_of(kind: TransformKind) -> Mode {
    match kind {
        TransformKind::Move => Mode::Move,
        TransformKind::Rotate => Mode::Rotate,
        TransformKind::Scale => Mode::Scale,
    }
}

impl Sculpt3dScene {
    /// A câmera desta cena, no vocabulário do gizmo.
    fn gizmo_cam(&self) -> SculptCam {
        SculptCam {
            cam: self.camera,
            viewport: self.viewport(),
            origin: self.vp_origin(),
        }
    }

    /// ⭐⭐ **ONDE O GIZMO ESTÁ** — o centro do que pode mover-se, em MUNDO.
    ///
    /// ⚠️ **O pivô sai da MESMA porta que o gesto usa**
    /// ([`ph2d_sculpt3d::free_pivot`], que a `MaskTransform::begin` chama):
    /// desenhar num centro e girar em torno de outro é o defeito que o artista
    /// lê como *«o gizmo está torto»*.
    ///
    /// ⚠️ **Os eixos são os do MUNDO**, e não há selector — a `ph2d_mesh::Pose`
    /// de uma escultura não tem rotação, então *global* e *local* dariam os
    /// mesmos três vectores. É a mesma medição (e o mesmo gate) que mantém o
    /// *World* fora do referencial do filtro de tecido.
    ///
    /// `None` com a peça inteira protegida — ali não há o que mover, e um gizmo
    /// desenhado prometeria um gesto que a lei recusa.
    pub(crate) fn gizmo_anchor(&self) -> Option<Anchor> {
        let pivot = ph2d_sculpt3d::free_pivot(self.mesh())?;
        Some(Anchor::global(0, self.pose().point_to_world(pivot)))
    }

    /// As alças prontas a pintar e a apontar — vazio sem transform armado.
    pub fn gizmo_handles(&self) -> Vec<Projected> {
        let (Some(kind), Some(anchor)) = (self.transform_arm(), self.gizmo_anchor()) else {
            return Vec::new();
        };
        ph2d_viewport3d::gizmo::project_with(anchor, &self.gizmo_cam(), mode_of(kind))
    }

    /// A alça sob este ponto da janela, se houver.
    pub(crate) fn gizmo_pick(&self, x: f32, y: f32) -> Option<Handle> {
        ph2d_viewport3d::gizmo::pick(&self.gizmo_handles(), [x, y])
    }

    /// A alça sob o cursor no último quadro — só realce.
    pub fn gizmo_hot(&self) -> Option<Handle> {
        self.janela.gizmo_hot
    }

    /// O quadro publica qual alça está quente.
    pub fn note_gizmo_hot(&mut self, pointer: (f32, f32)) {
        self.janela.gizmo_hot = self.gizmo_pick(pointer.0, pointer.1);
    }

    /// ⭐⭐ **O PEN-DOWN AGARROU UMA ALÇA?** — arma a restrição do gesto.
    ///
    /// ⚠️ **`false` não é uma recusa**: sem alça sob o dedo o transform corre
    /// **livre**, que é exactamente o gesto modal que este módulo já tinha. *Um
    /// gizmo que tomasse conta do botão inteiro tiraria uma ferramenta que
    /// funciona para dar outra.*
    pub(crate) fn gizmo_grab(&mut self, x: f32, y: f32) -> bool {
        self.janela.gizmo_grip = self.gizmo_pick(x, y);
        self.janela.gizmo_grip.is_some()
    }

    /// Larga a alça — chamado pelo fecho do transform.
    pub(crate) fn gizmo_release(&mut self) {
        self.janela.gizmo_grip = None;
    }

    /// ⭐⭐⭐ **A RESTRIÇÃO QUE A ALÇA AGARRADA IMPÕE** — o coração desta wave.
    ///
    /// Recebe o gesto **livre** que o transform modal já produzia e devolve o
    /// mesmo gesto **preso** ao que a alça promete. Sem alça, devolve-o intacto.
    ///
    /// ⚠️⚠️ **A projecção é do DESLOCAMENTO, nunca do ponto**: prender o
    /// *destino* ao eixo faria a peça saltar para cima do eixo no primeiro pixel
    /// de arrasto. O que a seta promete é *«só nesta direcção»*, e isso é a
    /// componente do deslocamento.
    pub(super) fn constrain(&self, g: Gesture) -> Gesture {
        let Some(handle) = self.janela.gizmo_grip else {
            return g;
        };
        match (handle, g) {
            // ⭐ **Só ao longo do eixo** — a componente do deslocamento nele.
            (Handle::Axis(n), Gesture::Move { delta }) => Gesture::Move {
                delta: componente(delta, eixo(n), true),
            },
            // ⭐ **Só no plano perpendicular ao eixo** — o deslocamento MENOS a
            // componente nele. O quadrado `XY` é o `Plane(2)`, como no vizinho.
            (Handle::Plane(n), Gesture::Move { delta }) => Gesture::Move {
                delta: componente(delta, eixo(n), false),
            },
            // ⭐⭐ **Rodar em torno de um eixo do MUNDO.**
            //
            // ⚠️⚠️ **O eixo é virado para o OBSERVADOR antes de descer à peça**,
            // e não é enfeite: a varredura que dá o ângulo é medida no ECRÃ,
            // então o mesmo arrasto tem de rodar a peça no mesmo sentido VISTO.
            // Com o eixo a apontar para trás, o sinal cru inverteria a rotação, e
            // o artista lê isso como *«a argola de trás roda ao contrário»*. É a
            // mesma lei da `ViewRing`, que aponta ao olho por construção.
            (Handle::Ring(n), Gesture::Rotate { radians, .. }) => Gesture::Rotate {
                axis: self.eixo_para_o_olho(eixo(n)),
                radians,
            },
            // As restantes já SÃO o gesto livre: a `View`, a `ViewRing` e o
            // `Grip` fazem exactamente o que o transform modal faz.
            (_, g) => g,
        }
    }

    /// O eixo do mundo, virado para o observador e descido à peça.
    fn eixo_para_o_olho(&self, a: [f32; 3]) -> [f32; 3] {
        let fwd = self.gizmo_cam().fwd();
        let s = if dot(a, fwd) < 0.0 { -1.0 } else { 1.0 };
        self.dir_to_local([a[0] * s, a[1] * s, a[2] * s])
    }
}

/// O eixo `n` do mundo.
fn eixo(n: usize) -> [f32; 3] {
    let mut a = [0.0; 3];
    a[n.min(2)] = 1.0;
    a
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

/// A componente de `v` ao longo de `a` (`ao_longo`), ou o que sobra sem ela.
fn componente(v: [f32; 3], a: [f32; 3], ao_longo: bool) -> [f32; 3] {
    let k = dot(v, a);
    if ao_longo {
        [a[0] * k, a[1] * k, a[2] * k]
    } else {
        [v[0] - a[0] * k, v[1] - a[1] * k, v[2] - a[2] * k]
    }
}

#[cfg(test)]
#[path = "gizmo_tests.rs"]
mod tests;
