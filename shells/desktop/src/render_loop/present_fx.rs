//! ⭐⭐ **OS PASSES DE LUZ do quadro** — irmão por ASSUNTO do [`super::present`], e pelo tecto de
//! 600 LOC do shell (HR-18).
//!
//! Duas famílias, a MESMA máquina e listas diferentes: a **sprite emissiva** (a §8 do Sprite
//! Inspector) e o **glow do Motion** (o `fx.glow` autorado no grafo). As duas re-desenham em
//! ISOLAMENTO o que emite luz num RT `Rgba16Float` próprio, passam-lhe o bright-pass borrado e
//! **somam** o resultado sobre o `game_rt` — antes do tonemap, para o halo tonemapear com o resto.
//!
//! ⚠️ **A ordem entre as duas é load-bearing**, e é por isso que elas ficam no mesmo ficheiro: os
//! dois passes partilham o RT do `motion_fx` e cada um escreve-o inteiro para o consumir logo a
//! seguir. Em sequência funciona; entrelaçados, um apagaria o outro.
//!
//! ⛔ Sem nada a emitir e sem `fx.glow` no grafo, nada disto toca a GPU e o quadro é
//! **byte-idêntico** ao de antes destas features existirem (há gate).

use ph2d_host::WindowSize;

/// ⚠️ **A engrenagem viaja numa struct, e não em onze argumentos** — o mesmo motivo do
/// [`super::present_bands::BandGear`]: os dois passes usam exactamente as mesmas peças, e passá-las
/// soltas faria duas listas que divergiriam na primeira peça nova.
pub(super) struct FxGear<'a> {
    pub renderer: &'a mut ph2d_render::SpriteRenderer,
    pub motion_fx: &'a mut ph2d_render::MotionFx,
    pub game_rt: &'a ph2d_render::GameRt,
    pub camera: &'a ph2d_render::Camera2d,
    pub window_size: WindowSize,
    pub scene_viewport: Option<[f32; 4]>,
    pub sim: &'a mut ph2d_ecs::SimWorld,
    pub motion: &'a ph2d_app_motion::motion_state::MotionState,
    pub motion_active: bool,
    pub present: &'a mut ph2d_ecs::PresentWorld,
    /// O scratch das instâncias emissivas — vive no `App` porque é lixo de quadro, e re-alocá-lo
    /// por quadro seria uma alocação por frame para uma lista quase sempre vazia.
    pub instances: &'a mut ph2d_render::LiftedInstances,
    /// As formas do Motion deste quadro, já desenhadas pela placa (doc 121 §9.14).
    pub formas: &'a ph2d_app_motion::motion_shape_placa::PlacaDeFormas,
    /// Os palcos das macros em cena — cada um brilha pelo MESMO passe.
    pub palcos: &'a ph2d_app_motion::macro_track::palcos::PalcosDasMacros,
}

/// Corre os dois passes, nesta ordem.
pub(super) fn run(gpu: &ph2d_gpu::GpuContext, g: FxGear<'_>) {
    // Pass 1b-bis: **A SPRITE COMO FONTE DE LUZ** (plano `docs/Sprite_projeto/18` W8,
    //   Enio 2026-08-21). Mesma máquina do glow do Motion logo abaixo, outra lista:
    //   re-desenha em ISOLAMENTO as sprites que carregam `SpriteEmissive` — com o
    //   `tint` já multiplicado pela intensidade — no RT `Rgba16Float` do `motion_fx`,
    //   e SOMA o bright-pass borrado sobre o `game_rt`, antes do tonemap.
    //
    // ⚠️ **Corre ANTES do glow do Motion, e a ordem é load-bearing.** Os dois passes
    //   partilham o RT do `motion_fx` e cada um escreve-o inteiro para o consumir logo
    //   a seguir — em sequência funciona, entrelaçados um apagaria o outro.
    //
    // ⚠️ Sem nenhuma sprite a emitir a lista sai VAZIA e este bloco não toca a GPU: o
    //   quadro é byte-idêntico ao de antes desta feature existir (há gate).
    super::sprite_emissive::collect(g.sim, g.present, g.instances);
    if !g.instances.is_empty() {
        g.renderer.render_lifted_instances(
            g.motion_fx.rt_view(),
            g.camera,
            g.window_size,
            wgpu::Color::TRANSPARENT,
            &*g.instances,
            // O MESMO sub-rect da cena fundida — senão o halo desliza para fora da
            // sprite que o emitiu (o mesmo cuidado que o glow do Motion documenta).
            g.scene_viewport,
            // A emissão É luz: nunca recebe a luz 2D.
            false,
        );
        g.motion_fx.bloom_over(
            gpu,
            g.game_rt.view(),
            &super::sprite_emissive::bloom_params(),
            // ⚠️ **Sem rampa, e é a mesma razão dos outros campos deste sítio**: a
            // rampa é autoria de um NÓ, e um emissor de sprite não tem nó nenhum.
            None,
            // ⚠️ **E sem máscara de sujidade, pela MESMA razão levada um passo
            // adiante**: a imagem dela nomeia um objecto da cena, e é o nó que
            // guarda o nome — um emissor de sprite não tem onde o escrever.
            None,
        );
    }
    // Pass 1c: o `fx.glow` do Motion — uma vez por FONTE, o editor e cada palco de uma macro em cena
    //   (a lei e o porquê vivem em `ph2d_app_motion::motion_glow_passe`; depois da sprite emissiva).
    let (motion, formas, ativo, palcos) = (g.motion, g.formas, g.motion_active, g.palcos);
    let mut brilho = ph2d_app_motion::motion_glow_passe::Brilho {
        renderer: g.renderer,
        motion_fx: g.motion_fx,
        game_rt: g.game_rt,
        camera: g.camera,
        window_size: g.window_size,
        scene_viewport: g.scene_viewport,
        sim: g.sim,
    };
    ph2d_app_motion::motion_glow_passe::passe(gpu, &mut brilho, motion, formas, ativo);
    for (palco, placa) in palcos.fontes() {
        ph2d_app_motion::motion_glow_passe::passe(gpu, &mut brilho, palco, placa, true);
    }
}
