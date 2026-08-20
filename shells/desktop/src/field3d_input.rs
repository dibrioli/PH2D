//! **A navegação da janela 3D de modelagem** — órbita, pan e zoom ([ADR-0161], W4).
//!
//! # ⚠️ Mora no SHELL, e isso é o que mantém o contrato de ferramentas fora do caminho
//!
//! [ADR-0150] já resolveu esta pergunta para a escultura: *a navegação orbital mora no SHELL, nunca
//! numa `Tool`*. É o que impede que uma janela 3D obrigue a mexer no `Tool=12` — que está
//! **congelado** (`CLAUDE.md §6`) e cuja alteração exigiria ADR e Coordenador. Este arquivo é o
//! gémeo do [`crate::sculpt3d_input`] para o outro módulo 3D, com a mesma forma e os mesmos botões.
//!
//! # A superfície partilhada é de QUATRO linhas
//!
//! O `input_dispatch.rs` é o arquivo onde duas linhas paralelas mais se encontram. Por isso o que
//! entra lá são **só as chamadas** — quatro `if` que devolvem `false` quando o smoke não está
//! armado —, e toda a decisão mora aqui. É a forma que a `line/sculpt3d` já usa, e segui-la faz os
//! dois diffs ficarem **adjacentes** em vez de sobrepostos.
//!
//! # As leis do gesto são as MESMAS da escultura
//!
//! Esquerdo e direito orbitam, o do meio faz pan, a roda aproxima — e as constantes são as de lá
//! (`ORBIT_RAD_PER_PX = 0,01`, `1,1` por passo de roda). ⚠️ Não é herança por analogia (o
//! [`project-memory`] avisa contra isso): é que são **duas janelas 3D no mesmo aplicativo**, e uma
//! mão que aprendeu a girar numa tem de girar na outra. Divergir aqui seria uma decisão, e não há
//! nenhuma a tomar.
//!
//! [ADR-0161]: ../../../docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md
//! [ADR-0150]: ../../../docs/architecture/decisions/0150-3d-sculpt-is-a-mesh-that-donates-shading-sculptgl-referenced.md
//! [`project-memory`]: ../../../project-memory/feedback_inherited_affordance_must_be_rederived.md

use crate::app_state::App;
use crate::field3d_smoke::{Drag, with_smoke};

/// Radianos de órbita por pixel de arrasto — **o mesmo número do módulo de escultura**
/// (`sculpt3d_rulers::ORBIT_RAD_PER_PX`).
const ORBIT_RAD_PER_PX: f32 = 0.01;

/// Fator de zoom por passo de roda — **a mesma lei do `dolly`** da câmera da casa
/// (`ph2d_mesh_render::camera`): multiplicativa, para que cada passo aproxime a mesma **fração**.
const ZOOM_PER_STEP: f32 = 1.1;

/// O quanto se pode aproximar, e o recurso de que este limite é: a **precisão da representação**.
///
/// O campo é avaliado em `f32`, o que dá ~10⁻⁷ de erro absoluto à escala de uma peça unitária.
/// Quando um pixel passa a medir menos do que isso em unidades de mundo, a imagem deixa de ser
/// forma e passa a ser ruído de arredondamento. Com um quadro de ~1000 px, o pixel vale
/// `2·half_extent/1000`, e `10⁻⁴` põe-no em 2·10⁻⁷ — no fio.
///
/// ⚠️ São ~8000× de aproximação a partir do enquadramento inicial (0,8). O que costumava limitar o
/// zoom era a tolerância de acerto, e essa **deixou de ser um limite**: ela desce com o pixel
/// (`ph2d_field_render::Sharpness`). *Este é o único piso que sobrou, e ele nomeia o seu recurso.*
const MIN_HALF_EXTENT: f32 = 1.0e-4;

/// O quanto se pode afastar, e o recurso: o **alcance da marcha**.
///
/// Os raios partem a 4 unidades do alvo e andam 8, logo cobrem uma profundidade de ±4 em torno do
/// plano do alvo. Enquadrar mais largo do que isso mostraria uma cena que os raios não alcançam.
const MAX_HALF_EXTENT: f32 = 4.0;

/// A elevação máxima. Nos polos a base da câmera continua ortonormal (o `right` só depende do
/// `yaw`), então o limite não é numérico — é de gesto: passar do polo inverte o mundo debaixo da
/// mão, e ninguém quer isso a meio de um arrasto.
const MAX_PITCH: f32 = std::f32::consts::FRAC_PI_2;

/// **As três leis da câmera, puras** — sem `App`, sem ponteiro, sem estado do smoke.
///
/// ⭐ A separação não é estética: é o que torna as leis **testáveis pela porta do produto**. Um gate
/// que precisasse de um `App` inteiro para perguntar *"arrastar para a direita vira o modelo para a
/// direita?"* não seria escrito — e foi exatamente essa pergunta que a `line/sculpt3d` respondeu
/// errado nos dois sinais até um smoke a pegar. Aqui o gate traça a peça e **mede-a na tela**.
pub(crate) mod law {
    use ph2d_field_render::Orbit;

    use super::{MAX_HALF_EXTENT, MAX_PITCH, MIN_HALF_EXTENT, ORBIT_RAD_PER_PX, ZOOM_PER_STEP};

    /// Órbita por um arrasto de `(dx, dy)` pixels.
    pub(crate) fn orbit(cam: &mut Orbit, dx: f32, dy: f32) {
        cam.yaw -= dx * ORBIT_RAD_PER_PX;
        cam.pitch = (cam.pitch + dy * ORBIT_RAD_PER_PX).clamp(-MAX_PITCH, MAX_PITCH);
    }

    /// Pan por um arrasto de `(dx, dy)` pixels, num quadro cujo lado menor mede `half_px` de meia
    /// altura.
    pub(crate) fn pan(cam: &mut Orbit, dx: f32, dy: f32, half_px: f32) {
        let k = cam.half_extent / half_px.max(1.0);
        let (right, up, _) = cam.basis();
        for i in 0..3 {
            cam.target[i] += -right[i] * dx * k + up[i] * dy * k;
        }
    }

    /// Zoom por `steps` linhas de roda.
    pub(crate) fn zoom(cam: &mut Orbit, steps: f32) {
        cam.half_extent =
            (cam.half_extent / ZOOM_PER_STEP.powf(steps)).clamp(MIN_HALF_EXTENT, MAX_HALF_EXTENT);
    }
}

impl App {
    /// O ponteiro desceu. Devolve `true` se a janela 3D tomou o gesto.
    pub(crate) fn field3d_pointer_down(&mut self, button: winit::event::MouseButton) -> bool {
        let pos = self.last_pointer;
        // ⚠️ **A moldura do app não é da cena.** A mesma lei do `sculpt3d_pointer_down`, e pelo
        // mesmo motivo medido lá: painel é só uma espécie de UI — a faixa do topo e o rail não
        // publicam `panel_rect`, e sem esta pergunta a cena engoliria o clique em todo pill do topo.
        if crate::forwarding::cursor_over_hero_chrome(self.gfx.as_ref(), pos.0, pos.1) {
            return false;
        }
        let drag = match button {
            winit::event::MouseButton::Left | winit::event::MouseButton::Right => Drag::Orbit,
            winit::event::MouseButton::Middle => Drag::Pan,
            _ => return false,
        };
        with_smoke(|s| {
            // ⚠️ **Fora da área desenhada, o gesto não é meu.** O `Move` e o `Up` NÃO fazem esta
            // pergunta, de propósito: um arrasto em curso continua a ser do gesto que o abriu mesmo
            // que o cursor passeie por fora — a regra de captura que todo gizmo deste shell segue.
            let Some(area) = s.area else {
                return false;
            };
            if pos.0 < area.x
                || pos.1 < area.y
                || pos.0 >= area.x + area.w
                || pos.1 >= area.y + area.h
            {
                return false;
            }
            s.drag = Some(drag);
            s.last_pointer = pos;
            s.manual = true;
            true
        })
        .unwrap_or(false)
    }

    /// O ponteiro moveu. **Só consome com um arrasto em curso** — senão a janela 3D engoliria todo
    /// hover do app 2D.
    pub(crate) fn field3d_pointer_move(&mut self, x: f32, y: f32) -> bool {
        with_smoke(|s| {
            let Some(drag) = s.drag else {
                return false;
            };
            let (dx, dy) = (x - s.last_pointer.0, y - s.last_pointer.1);
            s.last_pointer = (x, y);
            match drag {
                // ⚠️ **Manipulação direta: o modelo segue a mão.** `yaw` positivo leva o OLHO para
                // `+X`, e a câmera indo para a direita faz o modelo *parecer* ir para a esquerda —
                // então arrastar para a direita pede `yaw -= dx`. Arrastar para BAIXO mostra o topo.
                //
                // Os dois sinais são os que a `line/sculpt3d` já pagou para descobrir, e o gate que
                // os prende aqui mede **o modelo na tela**, nunca o sinal: foi argumentando sobre
                // sinais que o erro entrou lá.
                Drag::Orbit => law::orbit(&mut s.cam, dx, dy),
                // O alvo anda ao CONTRÁRIO da mão: mover o ponto olhado para a esquerda é o que faz
                // o modelo aparecer mais à direita.
                //
                // ⚠️ O passo é em **fração do lado menor do quadro**, vezes `half_extent` — é isso
                // que faz arrastar o mesmo tanto de tela mover o mesmo tanto de modelo em qualquer
                // zoom. Um passo em unidades de mundo fixas ficaria absurdo assim que se aproxima.
                Drag::Pan => {
                    let Some(area) = s.area else {
                        return true;
                    };
                    law::pan(&mut s.cam, dx, dy, area.w.min(area.h) * 0.5);
                }
            }
            true
        })
        .unwrap_or(false)
    }

    /// O ponteiro subiu. Fecha o arrasto, se havia um.
    pub(crate) fn field3d_pointer_up(&mut self) -> bool {
        with_smoke(|s| s.drag.take().is_some()).unwrap_or(false)
    }

    /// A roda aproxima. `steps` em linhas de roda.
    pub(crate) fn field3d_wheel(&mut self, steps: f32) -> bool {
        let pos = self.last_pointer;
        // A pergunta é feita INTEIRA e neste arquivo de propósito — quem decide de quem é o gesto é
        // o módulo da cena, não o roteador (a nota do `sculpt3d_wheel`).
        if crate::forwarding::cursor_over_hero_chrome(self.gfx.as_ref(), pos.0, pos.1) {
            return false;
        }
        with_smoke(|s| {
            let Some(area) = s.area else {
                return false;
            };
            if pos.0 < area.x
                || pos.1 < area.y
                || pos.0 >= area.x + area.w
                || pos.1 >= area.y + area.h
            {
                return false;
            }
            law::zoom(&mut s.cam, steps);
            s.manual = true;
            true
        })
        .unwrap_or(false)
    }
}

#[cfg(test)]
#[path = "field3d_input_tests.rs"]
mod tests;
