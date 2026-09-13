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
    // Pass 1c: Motion glow (doc 67, Option B) — the Motion module's
    //   OWN HDR effect, authored as an `fx.glow` node in the graph.
    //   Only runs when the artist has dropped that node (and dialed
    //   intensity > 0); otherwise this whole block is skipped and the
    //   frame is byte-identical (the fused sprite+Motion pass and the
    //   tonemap are untouched → blast radius zero). Re-render the
    //   Motion instances IN ISOLATION into `motion_fx`'s own Rgba16Float
    //   RT, bright-pass + blur them, and ADD the glow over `game_rt`
    //   (before the tonemap, so the glow tonemaps with everything
    //   else). Additive = emitted light, so it bleeds over whatever is
    //   in front — the sparks look lit, not pasted.
    let glow = ph2d_node_fx_glow::from_graph(&g.motion.doc.graph);
    // ⚠️ **Assada FORA do `if` de propósito**: dentro dele o `glow` já foi movido, e
    // pô-la lá obrigaria a reordenar o bloco. O custo de a assar sem a usar é uma
    // varredura de 512 avaliações num quadro em que existe um `fx.glow` com rampa —
    // e o passe só corre nesse quadro de qualquer forma.
    let halo_lut = ph2d_node_fx_glow::bake_halo_lut(&g.motion.doc.graph);
    // ⚠️ **A LISTA DO GLOW É A CAMADA MOTION, e não o passe de sprites**
    // (bug do Enio, 2026-08-20: *"Glow não funciona com shape"*, e a
    // ordem dele depois: *"tudo deve brilhar"*). Ver
    // [`ph2d_app_motion::motion_glow_layer`] — a metade vetorial viva entra aqui
    // pelo TILE assado, porque um halo é imediatamente reduzido por seis
    // níveis de mip e nunca precisou de nitidez de tela.
    let glow_layer = ph2d_app_motion::motion_glow_layer::layer_instances(
        &g.motion.pump.instances,
        &g.motion.pump.vector_instances,
        &g.motion.object_bake,
        &g.motion.shape_bake,
    );
    // ⚠️ **`PH2D_GLOW_DIAG=1`** — de que é feita a camada, quando ela muda.
    // Ver o doc de [`ph2d_app_motion::motion_glow_layer::diag`]: «o halo não
    // aparece» tem cinco causas indistinguíveis a olho.
    ph2d_app_motion::motion_glow_layer::diag(
        &g.motion.pump.instances,
        &g.motion.pump.vector_instances,
        &g.motion.object_bake,
        &g.motion.shape_bake,
        glow.as_ref().map(|k| k.intensity),
        glow_layer.len(),
    );
    if let Some(glow) = glow
        && g.motion_active
        && glow.intensity > 0.0
        && !glow_layer.is_empty()
    {
        g.renderer.render_instances_only(
            g.motion_fx.rt_view(),
            g.camera,
            g.window_size,
            wgpu::Color::TRANSPARENT,
            &glow_layer,
            // SAME sub-rect the fused scene used above — or the glow
            // desyncs from the sparks (the halo floats away).
            g.scene_viewport,
        );
        // **A MÁSCARA DE SUJIDADE** (doc 89 folha 11) — o nó guarda o NOME de um
        // objecto da cena e o passe de tela quer uma `TextureView`. As duas metades
        // encontram-se aqui, DEPOIS do passe de isolamento: aquele leva o renderer
        // emprestado mutável, e a resolução só o lê.
        //
        // ⚠️ **A resolução corre só quando há nome autorado.** Sem ele o `resolve`
        // nem é chamado — uma varredura da cena por quadro para responder *"nada"*
        // é o custo que o caminho de sempre não pode pagar.
        let dirt_cooked = |id| g.renderer.cooked_texture_id(id);
        let dirt = ph2d_node_fx_glow::dirt::source(&g.motion.doc.graph).and_then(|n| {
            ph2d_app_motion::motion_glow_dirt::resolve(
                g.sim,
                ph2d_app_motion::motion_bridge::Appearance {
                    atlas: g.renderer.atlas(),
                    cooked: &dirt_cooked,
                },
                &n,
            )
            .or_else(|| {
                // ⚠️ **Um nome que não resolve é a SEXTA causa indistinguível a
                // olho** desta família (ver `motion_glow_layer::diag`, que já
                // documenta cinco). Ele é legítimo — um nome pode ser escrito antes
                // de a sprite existir —, então não é erro; mas ficar mudo é o que
                // torna *"escrevi o nome e não aconteceu nada"* indiagnosticável.
                ph2d_app_motion::motion_glow_dirt::diag_unresolved(&n);
                None
            })
        });
        let dirt = dirt.and_then(|r| ph2d_app_motion::motion_glow_dirt::mask(r, g.renderer));
        g.motion_fx.bloom_over(
            gpu,
            g.game_rt.view(),
            &ph2d_render::BloomParams {
                threshold: glow.threshold,
                knee: glow.knee,
                intensity: glow.intensity,
                radius: glow.radius,
                saturation: glow.saturation,
                tint: glow.tint,
                stretch: glow.stretch,
                angle: glow.angle,
                clamp: glow.clamp,
                operation: glow.operation,
                source: glow.source,
                dirt_intensity: glow.dirt_intensity,
            },
            // **A RAMPA DO HALO** (doc 89 folha 11) — assada pelo nó, que é quem
            // possui a semântica do gradiente; aqui ela só atravessa. `None` quando
            // o artista não desenhou nenhuma, e aí o `tint` constante manda.
            halo_lut.as_deref(),
            dirt,
        );
    }
}
