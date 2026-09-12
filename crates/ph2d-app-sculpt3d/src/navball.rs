//! ⭐⭐⭐ **O GIZMO DA VIEWPORT DA ESCULTURA** — as seis bolas de eixo no canto,
//! e as seis vistas nomeadas que elas dão.
//!
//! Ordem do Enio, 2026-09-08: *«veja o módulo de Modelagem 3d. Ele tem
//! viewports (4), gizmos de transformação e o gizmo da viewport. Traga esses
//! features para esse módulo.»*
//!
//! # ⭐⭐ O que este ficheiro é: uma CÂMERA, não uma segunda lei
//!
//! A lei do widget — onde as bolas caem, qual está à frente, qual está sob o
//! cursor, e para onde ele foge da moldura — **já existe e não se copia**:
//! [`ph2d_viewport3d::navball`], com a pesquisa do ViewCube patenteado e a
//! calibração das duas folgas registadas no cabeçalho dela. A lista de vistas
//! também: [`ph2d_viewport3d::views::Standard`].
//!
//! ⚠️ **O que faltava era o único pedaço que depende da câmera**: aquele módulo
//! nasceu contra a [`ph2d_field_render::Orbit`] (rotação livre, sem polo) e a
//! escultura tem a [`Camera3d`] (órbita `yaw`/`pitch`, **com** trava de polo).
//! ⇒ a lei passou a receber a **base** (direita · cima · para-o-observador), que
//! é o que as duas câmeras têm em comum, e cada módulo entrega a sua
//! ([`ph2d_viewport3d::navball::balls_from_basis`]).
//!
//! # ⛔⛔ A TRAVA DO POLO é a divergência, e ela é declarada
//!
//! A `Top` pede `pitch = π/2` e esta câmera **não consegue guardá-lo**: ali a
//! direcção da vista fica paralela ao [`Camera3d::UP`] e a `look_at` produz
//! `NaN`. Ela guarda `π/2 − 0,01` (`0,57°` de desvio), e é por isso que
//! [`aim_of`] existe: *o reconhecimento compara contra o valor que a câmera de
//! facto guarda, nunca contra o ideal que foi pedido.* Comparar com o ideal
//! faria o painel dizer **não** sobre a vista que ele próprio acabou de pedir —
//! e o módulo vizinho já pagou esta lição uma vez, com o número errado escrito
//! ao lado da tolerância dele.

use super::{ORBIT_RAD_PER_PX, Sculpt3dScene};
use ph2d_editor_core::zones::Rect as EditorRect;
use ph2d_mesh_render::Camera3d;
use ph2d_viewport3d::views::Standard;

/// Quão perto a câmera tem de estar para o produto dizer que **é** aquela vista.
///
/// ⚠️ **Em RADIANOS de ângulo de câmera, e o recurso de que ela é feita é o
/// menor GESTO que existe:** o [`ORBIT_RAD_PER_PX`] é `0,01` rad por pixel, ou
/// seja **um pixel de arrasto vale `0,57°`**. A barra fica **10× abaixo** dele
/// (`0,057°`) e cinco ordens de grandeza acima do ruído de `f32` num ângulo
/// (~`1e-7`).
///
/// ⇒ mover a mão **um pixel** solta a vista nomeada, e nada menos que isso a
/// solta. *Uma tolerância que não separa o ruído do menor gesto é um chip que
/// pisca ou um chip que mente.*
const RECOGNISE_RAD: f32 = 0.1 * ORBIT_RAD_PER_PX;

/// ⭐ **O `(yaw, pitch)` que ESTA câmera guarda naquela vista** — já preso à
/// trava do polo. Ver a nota do módulo.
pub(crate) fn aim_of(v: Standard) -> (f32, f32) {
    let (yaw, pitch) = v.yaw_pitch();
    (yaw, Camera3d::clamp_pitch(pitch))
}

/// A diferença entre dois ângulos, no círculo — sempre em `[0, π]`.
///
/// ⚠️ **`rem_euclid` e não uma subtracção**: o `yaw` desta câmera **não é
/// normalizado** (o [`Camera3d::orbit`] soma sem envolver), então depois de duas
/// voltas de arrasto ele vale `4π + ε` e uma subtracção crua leria `12,6 rad` de
/// desvio sobre a vista de frente exacta.
fn ang_diff(a: f32, b: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let d = (a - b).rem_euclid(tau);
    d.min(tau - d)
}

/// ⭐⭐ **Em que vista nomeada a câmera está** — ou `None`, que é a vista *livre*.
///
/// ⚠️ **Derivado da câmera, nunca guardado** — a mesma lei do módulo vizinho, e
/// pela mesma razão: um modo guardado ficaria aceso depois de o artista
/// arrastar meio grau para longe dele, que é o modo de falha clássico de um
/// espelho de estado.
pub(crate) fn named_view(cam: &Camera3d) -> Option<Standard> {
    Standard::ALL.into_iter().find(|s| {
        let (yaw, pitch) = aim_of(*s);
        ang_diff(cam.yaw, yaw) < RECOGNISE_RAD && (cam.pitch - pitch).abs() < RECOGNISE_RAD
    })
}

impl Sculpt3dScene {
    /// ⭐ **A BASE DA CÂMERA** — direita, cima e para-o-observador, em mundo.
    ///
    /// ⚠️ **Os dois primeiros saem do [`Camera3d::screen_basis`]**, que é a porta
    /// que o Grab e o estêncil do alpha já leem; derivá-los aqui seria a terceira
    /// resposta a *«onde está a tela, em mundo»*. O terceiro é o eixo alvo→olho,
    /// e é ele que decide que bola está à frente.
    fn nav_basis(&self) -> ([f32; 3], [f32; 3], [f32; 3]) {
        let (right, up) = self.camera.screen_basis();
        (right.into(), up.into(), self.camera.view_axis().into())
    }

    /// As seis bolas, prontas a pintar — de trás para a frente.
    pub fn navball(
        &self,
        area: EditorRect,
        safe: EditorRect,
    ) -> Vec<ph2d_viewport3d::navball::Ball> {
        let (right, up, fwd) = self.nav_basis();
        ph2d_viewport3d::navball::balls_from_basis(right, up, fwd, area, safe)
    }

    /// A bola sob o cursor no último quadro — o realce.
    pub fn nav_hot(&self) -> Option<Standard> {
        self.janela.nav_hot
    }

    /// ⭐⭐ **APONTA A CÂMERA A UMA VISTA NOMEADA** — o clique numa bola, e as
    /// teclas `Numpad1/3/7`.
    ///
    /// ⚠️ **A distância e o alvo NÃO se mexem**, e é a lei do Blender: a vista
    /// nomeada muda para **onde** se olha, nunca o enquadramento. Reenquadrar
    /// aqui faria `Numpad1` significar duas coisas, e a segunda desfaria o zoom
    /// que o artista acabou de escolher.
    pub(crate) fn aim_view(&mut self, v: Standard) {
        let (yaw, pitch) = aim_of(v);
        self.camera.aim(yaw, pitch);
    }

    /// **O que o gizmo faz com este ponteiro**, no pen-down. `true` = ele tomou o
    /// gesto.
    ///
    /// ⚠️ **Arrastar orbita, e o clique é o caminho secundário** — a medição da
    /// própria pesquisa que criou o ViewCube diz que os utilizadores são *quase
    /// 2× mais rápidos* a arrastar, **independentemente da representação**. Ver o
    /// cabeçalho do [`ph2d_viewport3d::navball`].
    ///
    /// ⇒ o pen-down **não** salta para a vista: ele arma o arrasto, e é o
    /// pen-**up** sem movimento que conta como clique. *Saltar já no down faria
    /// todo arrasto começar com um corte de câmera.*
    pub(crate) fn nav_pointer_down(&mut self, x: f32, y: f32) -> bool {
        let Some((area, safe)) = self.nav_rects() else {
            return false;
        };
        if !ph2d_viewport3d::navball::hits_widget(area, safe, [x - area.x, y - area.y]) {
            return false;
        }
        let balls = self.navball(area, safe);
        self.janela.nav_drag = Some(NavDrag {
            from: (x, y),
            moved: false,
            ball: ph2d_viewport3d::navball::pick(&balls, [x - area.x, y - area.y]),
        });
        true
    }

    /// O dedo andou sobre o widget: orbita. Devolve `true` se o gesto é dele.
    pub(crate) fn nav_pointer_move(&mut self, x: f32, y: f32) -> bool {
        let Some(drag) = self.janela.nav_drag.as_mut() else {
            return false;
        };
        let (dx, dy) = (x - drag.from.0, y - drag.from.1);
        // ⚠️ **A zona morta é UM pixel**, e não zero: sem ela o tremor do dedo ao
        // largar o botão transformaria todo clique num arrasto de meio grau, e a
        // bola nunca chegaria a saltar. É o mesmo `1 px` que a
        // [`RECOGNISE_RAD`] mede como o menor gesto que existe.
        if dx.hypot(dy) >= 1.0 {
            drag.moved = true;
        }
        drag.from = (x, y);
        // O MESMO sentido do arrasto na peça (`sculpt3d_input`): manipulação
        // directa, o modelo segue a mão.
        self.camera
            .orbit(-dx * ORBIT_RAD_PER_PX, dy * ORBIT_RAD_PER_PX);
        true
    }

    /// O dedo saiu. Um pen-up **sem movimento** sobre uma bola é o clique dela.
    pub(crate) fn nav_pointer_up(&mut self) -> bool {
        let Some(drag) = self.janela.nav_drag.take() else {
            return false;
        };
        if let Some(v) = drag.ball
            && !drag.moved
        {
            self.aim_view(v);
        }
        true
    }

    /// O quadro publica onde o widget mora — a área do canvas, a parte dela que
    /// a moldura do app não tapa, e onde o cursor está.
    ///
    /// ⚠️ **Publicado pelo desenho e lido pelo gesto**, exactamente como o
    /// `viewport`: o ponteiro corre fora do quadro e não tem nem o layout nem a
    /// lista de painéis abertos na mão.
    ///
    /// ⚠️ **O realce é derivado AQUI e não no pintor**, porque quem sabe onde o
    /// cursor está é o quadro e quem sabe onde as bolas caem é a lei — e a
    /// segunda depende da primeira. *Duas derivações do «qual bola está quente»
    /// divergiriam no quadro em que a câmera se mexe entre elas.*
    pub fn note_nav(&mut self, safe: EditorRect, pointer: (f32, f32)) {
        self.janela.nav_safe = Some(safe);
        let Some((area, _)) = self.nav_rects() else {
            self.janela.nav_hot = None;
            return;
        };
        let at = [pointer.0 - area.x, pointer.1 - area.y];
        self.janela.nav_hot = ph2d_viewport3d::navball::hits_widget(area, safe, at)
            .then(|| ph2d_viewport3d::navball::pick(&self.navball(area, safe), at))
            .flatten();
    }

    /// ⭐⭐ **Onde o widget mora agora** — `(a área do viewport ACTIVO, a parte
    /// livre da moldura)`, ou `None` antes do primeiro desenho.
    ///
    /// ⚠️⚠️ **A área é a do QUADRANTE ACTIVO e o `safe` é do CANVAS INTEIRO**, e
    /// a assimetria é o desenho: *um gizmo por quadrante seria quatro respostas
    /// à mesma pergunta* (a lei está escrita no `field3d_smoke_draw`), e a
    /// moldura do app que empurra o widget não conhece divisão nenhuma — ela
    /// está por cima do canvas todo.
    pub fn nav_rects(&self) -> Option<(EditorRect, EditorRect)> {
        Some((self.vp_rect(self.vp_active())?, self.janela.nav_safe?))
    }
}

/// **O ARRASTO NO GIZMO** — de onde ele veio, se já andou, e que bola o começou.
#[derive(Clone, Copy, Debug)]
pub(crate) struct NavDrag {
    from: (f32, f32),
    /// ⚠️ **Um `bool` e não `from != agora`**: o `from` é actualizado a cada
    /// evento (o arrasto é incremental), então a diferença voltaria a zero
    /// assim que a mão parasse, e um arrasto longo terminado parado seria lido
    /// como clique.
    moved: bool,
    ball: Option<Standard>,
}

#[cfg(test)]
#[path = "navball_tests.rs"]
mod tests;
