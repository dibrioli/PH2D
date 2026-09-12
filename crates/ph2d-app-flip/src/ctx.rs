//! **O que a família Flip lê do quadro** — o agrupador de parâmetros da W2/L5 (2.ª volta).
//!
//! # Por que ele existe, e por que NÃO é um handle
//!
//! Os 34 blocos `impl crate::App` desta família pediam, medido, **onze** coisas — e nenhuma
//! delas é um tipo local da shell: `FlipDoc` (`ph2d-flip`), `Playhead` (`ph2d-core`),
//! `Camera2d` (`ph2d-render`), `WindowSize`/`SurfaceContext` (`ph2d-host`/`ph2d-gpu`),
//! `ToolRegistry`/`ToastQueue`/`HeroScreen` (`ph2d-editor-core`), `SimWorld` (`ph2d-ecs`).
//! ⇒ a resposta é **ASSINATURA**, nunca um 6.º método no [`ph2d_app_host::AppHost`]
//! (regra 4 do bloco de reabertura: *«se a resposta é três coisas que a `App` segura, não é
//! porta — é assinatura»*).
//!
//! ⛔ **Isto NÃO desfaz a fronteira**, e a diferença é onde a escolha é feita: o HOWTO §1.5
//! proíbe um método do TRAIT **devolver** um handle (`&App`, `&AppGfx`), porque aí a família
//! alcança tudo a qualquer hora. Aqui é a **shell** que constrói o agrupador no sítio de
//! chamada que ela controla, com os campos que ela escolheu — o mesmo precedente do
//! `HeroScreen` por parâmetro, da Fase B.
//!
//! ⚠️ **O `FlipState` entra SEPARADO**, e de propósito: quase todo corpo precisa dele
//! `&mut` ao mesmo tempo que lê o `flip`, e dois campos do mesmo `&mut` struct não se
//! emprestam em separado através de uma fronteira de função.

/// O que a família lê do quadro, em tipos de outras crates.
pub struct FlipFrame<'a> {
    /// O DOCUMENTO (`gfx.flip`). ⚠️ Ele nunca foi desta família — é partilhado desde a F8
    /// dos Componentes e vive em `ph2d-flip`; esta é a ponte a não partir.
    pub flip: &'a mut ph2d_flip::FlipDoc,
    /// O relógio (`App::playhead`).
    pub playhead: &'a ph2d_core::Playhead,
    /// A câmera 2D (`gfx.camera`) — tela ⟷ mundo.
    pub camera: &'a ph2d_render::Camera2d,
    /// O tamanho da janela (`gfx.surface.size()`), que a câmera precisa para converter.
    pub win: ph2d_host::WindowSize,
}

impl FlipFrame<'_> {
    /// Tela → mundo, com o tamanho de janela já dentro.
    pub(crate) fn to_world(&self, x: f32, y: f32) -> [f32; 2] {
        self.camera.screen_to_world((x, y), self.win)
    }

    /// Quantas unidades de mundo vale um pixel de tela agora.
    ///
    /// ⚠️ Esta conta estava escrita **cinco** vezes nos blocos `impl App` desta família, com
    /// o mesmo `max(EPSILON)` e o mesmo `max(1)` — *uma lei escrita em cinco sítios ainda não
    /// é uma lei, só uma PORTA é*.
    pub(crate) fn px_to_world(&self) -> f32 {
        self.camera.height_world.max(f32::EPSILON) / self.win.height.max(1) as f32
    }
}
