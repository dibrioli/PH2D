//! A costura do módulo 3D com o shell (W1/M2) — **a cena, o gesto e o passe**.
//!
//! ⚠️ **A navegação orbital mora AQUI, nunca numa `Tool`.** Girar o modelo não é
//! esculpir: o artista gira com o pincel na mão, e uma `Tool` que capturasse o
//! ponteiro para navegar teria de devolvê-lo a cada gesto. É também o que mantém
//! o contrato congelado intacto (ADR-0145) — nenhum método novo em `Tool`.
//!
//! ⚠️ **Tudo isto é inerte sem a cena armada.** `AppGfx.sculpt3d` nasce `None` e
//! só o smoke a cria, então num run normal cada porta daqui devolve `false` no
//! primeiro `if` e o frame 2D é byte-idêntico. Quando o Tool chegar (W2+), é
//! esta mesma porta que ele arma — e é por isso que ela não é um `if smoke`.

use std::sync::atomic::{AtomicBool, Ordering};

use ph2d_mesh::Mesh;
use ph2d_mesh_render::{Camera3d, MeshRenderer};

use crate::app_state::App;

/// Quantos radianos um pixel de arrasto vale.
///
/// Decisão de **smoke**, como a tolerância do RDP do Flip: 0,01 dá meia volta a
/// cada ~314 px, que é uma varredura confortável de trackpad. Não é um teto de
/// recurso, então não tem tabela de medição ao lado — tem o olho do Enio.
const ORBIT_RAD_PER_PX: f32 = 0.01;

/// O que o arrasto está fazendo.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Drag {
    Orbit,
    Pan,
}

/// A cena 3D viva: a malha, a câmera e o pipeline que a desenha.
pub(crate) struct Sculpt3dScene {
    pub(crate) mesh: Mesh,
    pub(crate) camera: Camera3d,
    renderer: MeshRenderer,
    /// A malha já subiu para o device? A W2 troca isto por um carimbo de versão;
    /// na W1 a malha não muda depois de criada.
    uploaded: bool,
    drag: Option<Drag>,
    last: (f32, f32),
}

impl Sculpt3dScene {
    pub(crate) fn new(device: &wgpu::Device, mesh: Mesh, aspect: f32) -> Self {
        let mut camera = Camera3d {
            yaw: 0.6,
            pitch: 0.35,
            ..Camera3d::default()
        };
        camera.frame(mesh.bounds(), aspect);
        Self {
            mesh,
            camera,
            renderer: MeshRenderer::new(device, ph2d_render::GameRt::FORMAT),
            uploaded: false,
            drag: None,
            last: (0.0, 0.0),
        }
    }

    /// Desenha a malha sobre o que já está no alvo. O upload acontece na
    /// primeira passagem — é aqui que o device é conhecido.
    pub(crate) fn render(
        &mut self,
        gpu: &ph2d_gpu::GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        color: &wgpu::TextureView,
        size: (u32, u32),
    ) {
        if !self.uploaded {
            self.renderer.upload(&gpu.device, &gpu.queue, &self.mesh);
            self.uploaded = true;
        }
        self.renderer
            .render(&gpu.device, &gpu.queue, encoder, color, &self.camera, size);
    }
}

impl App {
    /// `PH2D_SCULPT3D_SMOKE=1` — a cena pronta do M2: uma esfera na tela, para
    /// girar. Roda uma vez, no primeiro frame com GPU.
    pub(crate) fn sculpt3d_smoke(&mut self) {
        // Guard estático, o mesmo idioma dos outros smokes do shell — evita um
        // campo em `App` que só existe para dizer "já rodei".
        static ARMED: AtomicBool = AtomicBool::new(false);
        if std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() != Some("1")
            || self.gfx.is_none()
            || ARMED.swap(true, Ordering::Relaxed)
        {
            return;
        }
        let mesh = ph2d_mesh::shapes::uv_sphere(64, 96, 1.0);
        // A cena IMPRIME o que montou. Um smoke que não se declara deixa o
        // artista sem saber se está vendo a feature ou o app vazio — a lição
        // que o smoke do Colorize pagou.
        eprintln!(
            "[sculpt3d] esfera com {} vértices / {} faces / {} triângulos\n\
             [sculpt3d] arraste com o botão ESQUERDO para girar · MEIO para deslocar · RODA para aproximar",
            mesh.vert_count(),
            mesh.face_count(),
            mesh.triangle_count()
        );
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let size = gfx.surface.size();
        let aspect = size.width as f32 / size.height.max(1) as f32;
        let device = std::sync::Arc::clone(&gfx.surface.gpu().device);
        gfx.sculpt3d = Some(Sculpt3dScene::new(&device, mesh, aspect));
    }

    /// O botão apertou. Devolve `true` se a cena 3D tomou o gesto.
    pub(crate) fn sculpt3d_pointer_down(&mut self, button: winit::event::MouseButton) -> bool {
        let pos = self.last_pointer;
        let Some(scene) = self.sculpt3d_scene_mut() else {
            return false;
        };
        let drag = match button {
            winit::event::MouseButton::Left => Drag::Orbit,
            winit::event::MouseButton::Middle => Drag::Pan,
            _ => return false,
        };
        scene.drag = Some(drag);
        scene.last = pos;
        true
    }

    /// O botão soltou.
    pub(crate) fn sculpt3d_pointer_up(&mut self) -> bool {
        let Some(scene) = self.sculpt3d_scene_mut() else {
            return false;
        };
        let was = scene.drag.take();
        was.is_some()
    }

    /// O ponteiro moveu. Só consome com um arrasto EM CURSO — senão a cena 3D
    /// engoliria todo hover do app.
    pub(crate) fn sculpt3d_pointer_move(&mut self, x: f32, y: f32) -> bool {
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let height = gfx.surface.size().height.max(1) as f32;
        let Some(scene) = gfx.sculpt3d.as_mut() else {
            return false;
        };
        let Some(drag) = scene.drag else {
            return false;
        };
        let (dx, dy) = (x - scene.last.0, y - scene.last.1);
        scene.last = (x, y);
        match drag {
            // O sinal do pitch é invertido: arrastar para BAIXO na tela olha o
            // modelo de cima, que é o que a mão espera (o modelo gira, não a
            // câmera voa).
            Drag::Orbit => scene
                .camera
                .orbit(dx * ORBIT_RAD_PER_PX, -dy * ORBIT_RAD_PER_PX),
            Drag::Pan => scene.camera.pan(dx / height, dy / height),
        }
        true
    }

    /// A roda aproxima.
    pub(crate) fn sculpt3d_wheel(&mut self, steps: f32) -> bool {
        let Some(scene) = self.sculpt3d_scene_mut() else {
            return false;
        };
        scene.camera.dolly(steps);
        true
    }

    fn sculpt3d_scene_mut(&mut self) -> Option<&mut Sculpt3dScene> {
        self.gfx.as_mut()?.sculpt3d.as_mut()
    }
}
