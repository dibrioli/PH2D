//! **Fase do quadro: A RECOZEDURA DE FX E DE PADRÕES** — os padrões de textura, o FX raster por forma e o
//! bake de objectos do Motion e do Flip, sobre a geometria viva do quadro (OBRA 2 da `line/render-loop`, 2026-09-12).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_vector_fx_recook(
        &mut self,
        vec_view: ph2d_vec_scene::VecViewState,
        vec_xf: ph2d_vec_scene::VecXforms,
        cam_affine: ph2d_vector::Affine,
        vec_live: ph2d_vec_render::LiveGeometry,
    ) -> Option<(
        ph2d_vec_scene::VecViewState,
        ph2d_vec_scene::VecXforms,
        ph2d_vector::Affine,
        ph2d_vec_render::LiveGeometry,
    )> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            surface,
            renderer,
            sim,
            asset_db,
            vello_pass,
            vec_scene,
            flip,
            hero_screen,
            motion,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        // O FX raster por-forma (plano 24 — Blur/Glow/Drop Shadow). O produtor
        // (`fx_live`) rasteriza a forma isolada num scratch de GPU, lê de volta e borra na CPU,
        // injetando as imagens no z da forma. Roda DEPOIS de `vec_live` porque honra a geometria
        // derivada. Sem `VecFilter` na cena o mapa fica vazio = BYTE-IDÊNTICO ao mundo pré-FX.
        // ⭐ **OS LADRILHOS DE PADRÃO** (plano 33, W4) — assados AQUI, e a posição é a lei:
        // ⛔ **ANTES do `fx_live`**, que os consome. O report do Enio (*"filters anula
        // pattern"*) foi exactamente isto: com o assado depois, a rasterização isolada do FX
        // desenhava a forma com a cor de recurso, e a imagem de FX **toma o lugar** do desenho.
        //
        // Memoizados, porque assar custa (`1,047 ms` para um ladrilho de `536x1072` em colmeia)
        // e desenhar não custa nada (uma `fill()`).
        //
        // ⚠️ O filtro sai da porta ÚNICA da casa (`image_quality_for`), a mesma que o upscale e
        // a pré-visualização do BgRemoval usam — um padrão de pixel art tem de amostrar como uma
        // sprite de pixel art, e adivinhá-lo aqui daria duas respostas à mesma pergunta.
        //
        // ⚠️ Usa o `asset_db` que JÁ está desestruturado neste escopo: o empréstimo MUTÁVEL do
        // `gfx` abre muito acima e vive até ao fim do quadro.
        // ⭐ O assador de FORMA (W7) entra INJECTADO: ele é render + readback, e cablá-lo no
        // memo poria uma `GpuContext` na assinatura e tornaria todo gate dele dependente de uma
        // placa. Aqui ele é a porta única `motion_object_bake::bake_rgba`.
        // ⭐⭐⭐ **A arte de uma estampa e' um OBJECTO, e um objecto pode ser um GRUPO**
        // (Enio, 2026-08-30). O `object_selection_for` e' a MESMA porta que o clique no canvas
        // usa para decidir o que uma seleccao apanha — *"um grupo entra e sai da seleccao
        // INTEIRO"* —, e ela devolve os caminhos pela ordem do documento, que e' a de z.
        let object_of = |id| {
            ph2d_vec_entities::entities::object_selection_for(
                sim,
                vec_scene,
                &self.vec.entities,
                id,
            )
        };
        let mut bake_shape = |id| {
            ph2d_app_motion::motion_object_bake::bake_rgba_many(
                &mut self.texture_pattern_scratch,
                vec_scene,
                &vec_xf,
                &vec_live,
                &object_of(id),
                surface.gpu(),
                surface.format(),
                &object_of,
            )
            .map(|(rgba, w, h, _)| (w, h, rgba))
        };
        // ⭐⭐⭐ **A POSE dos membros entra pela MESMA fonte que o assado lê** (report do Enio,
        // 2026-08-30: *"ao mover os objetos do grupo que serve como shape, a pattern não
        // atualiza em tempo real"*). O `bake_rgba_many` acima recebe o `vec_xf`; a chave do memo
        // tem de o ler também, senão ela é cega ao gesto — a geometria de um `VecPath` é LOCAL
        // (ADR-0110) e mover um membro não lhe toca um byte.
        let pose_of = |id| vec_xf.get(&id).copied().unwrap_or_default();
        self.texture_pattern_live.recook(
            vec_scene,
            asset_db,
            ph2d_editor_core::image_quality_for(hero.project.image_filter),
            &mut bake_shape,
            &object_of,
            &pose_of,
        );
        self.fx_live.recook(
            vec_scene,
            sim,
            &self.vec.entities,
            &vec_xf,
            &vec_live,
            self.fx_silhouette.live(),
            &vec_view,
            self.texture_pattern_live.tiles(),
            cam_affine,
            surface.gpu(),
            surface.format(),
            vello_pass,
        );
        // doc 86 §2 (A2): bake the named vector shapes to tiles, right where
        // the FX stack bakes — the same handles (renderer, gpu, scene,
        // transforms, live geometry) are in hand. Cached by content ⇒ a
        // static scene bakes once; the membrane publishes the tiles next frame.
        ph2d_app_motion::motion_bridge::bake_objects(
            motion,
            vec_scene,
            &self.vec.entities,
            &vec_xf,
            &vec_live,
            surface.gpu(),
            renderer,
            surface.format(),
            sim,
        );
        // ⚠️ **E os tiles das formas PARAMÉTRICAS** — assados, e desde 2026-09-21 também
        // PARTICIONADOS pelo LOD. Vive numa função própria porque é um assunto próprio (e o tecto
        // de LOC da fase obrigou-o a acontecer no dia certo): aqui compõe-se a fase, ali
        // decide-se quem desenha crisp e quem desenha como tile.
        assa_e_particiona_tiles_de_forma(
            motion,
            cam_affine,
            surface.gpu(),
            renderer,
            surface.format(),
        );
        // doc 86 §2 (A3): bake the named FLIP objects to tiles, alongside the
        // vector bake. The Flip doc is destructured above (`flip`); the entity
        // map + playhead are disjoint `self` fields. Composes each object's
        // layers at the current frame through a scratch Flip raster + compositor.
        ph2d_app_motion::motion_bridge::bake_flip_objects(
            motion,
            flip,
            &self.flip_state.entities,
            &self.playhead,
            surface.gpu(),
            renderer,
            sim,
        );
        Some((vec_view, vec_xf, cam_affine, vec_live))
    }
}

/// ⭐⭐⭐ **Os tiles das formas paramétricas: assar, PARTICIONAR e despejar** — as três metades do
/// mesmo assunto, no sítio onde `renderer` + `gpu` estão em mão.
///
/// Irmã livre da [`crate::App::fase_vector_fx_recook`] por RESPONSABILIDADE: a fase COMPÕE o
/// quadro, isto decide **quem desenha crisp e quem desenha como tile**. O corte veio do tecto de
/// LOC da fase, e o ficheiro ficou melhor do que era.
fn assa_e_particiona_tiles_de_forma(
    motion: &mut ph2d_app_motion::motion_state::MotionState,
    cam_affine: ph2d_vector::Affine,
    gpu: &ph2d_gpu::GpuContext,
    renderer: &mut ph2d_render::SpriteRenderer,
    surface_format: wgpu::TextureFormat,
) {
    {
        // ⚠️ **E só assa se HOUVER quem consuma o tile.** Ele existe para o
        // bright-pass do glow e para mais nada; o `present` já pergunta pelo
        // `fx.glow` com esta mesma função, e a pergunta é feita AQUI pela MESMA
        // porta para as duas não divergirem. Sem a guarda, uma forma com um param
        // animado paga um **readback de GPU por quadro** por um tile que ninguém
        // lê — e o readback é a metade lenta deste assador, como o doc dele diz.
        let glows =
            ph2d_node_fx_glow::from_graph(&motion.doc.graph).is_some_and(|g| g.intensity > 0.0);
        let ph2d_app_motion::motion_state::MotionState {
            shape_bake,
            shape_store,
            object_bake,
            pump,
            ..
        } = &mut *motion;
        // ⚠️ **O conjunto VIVO é o de todas as instâncias deste quadro** — é ele
        // que decide o DESPEJO. O pedido ao assador é um subconjunto (tira o que
        // o `object_bake` já cobre); despejar por ele largaria um tile que ainda
        // está em cena. Ver `ShapeBake::evict_outside`, e o OOM que o motivou.
        let live: std::collections::BTreeSet<u32> = pump
            .vector_instances
            .iter()
            .map(|vi| vi.geometry_id)
            .collect();
        // ⭐⭐⭐ **O LOD DA FORMA — o SEGUNDO consumidor da tile** (report do Enio,
        // 2026-09-21: *«ao dar o zoom … o app trava. Não seria interessante criar um LOD
        // para shapes?»*). A guarda acima dizia `glows` porque o bright-pass era o ÚNICO
        // leitor; medido, a cena `=126` está `5,6×` acima do joelho do LOD e ele movia ZERO
        // cópias, porque o irmão pergunta ao `ObjectBake` e uma forma paramétrica não tem
        // `VecPathId`. ⇒ o assador passa a ter dois leitores, e a lei de QUEM é do
        // `motion_shape_lod` (tamanho no ecrã + contagem), não daqui.
        let quer = ph2d_app_motion::motion_shape_lod::geometrias_para_lod(
            &pump.vector_instances,
            shape_store,
            cam_affine,
            ph2d_app_motion::motion_bridge::objects::LOD_COUNT,
        );
        // Sem glow E sem LOD ninguém lê tile nenhum, então o conjunto pedido é VAZIO — e o
        // despejo abaixo corre na mesma, largando o que a sessão já assou.
        let wanted: Vec<u32> = live
            .iter()
            .copied()
            .filter(|gid| glows || quer.contains(gid))
            .filter(|gid| object_bake.tile_texture_for_gid(*gid).is_none())
            .collect();
        // ⚠️ **`PH2D_GLOW_DIAG=1`** diz se este assador correu e o que ele
        // conseguiu — sem isto, «não assou» e «não foi chamado» leem igual.
        let asked = wanted.len();
        shape_bake.bake_missing(shape_store, wanted, gpu, renderer, surface_format);
        // **E A PARTIÇÃO** — as geometrias que o LOD quer e que já têm tile passam a
        // quads de sprite. ⚠️ Corre DEPOIS do assado (a tile do 1.º quadro só existe agora) e
        // ANTES do despejo, que é o que lê o resultado dela.
        let movidas = ph2d_app_motion::motion_shape_lod::aplica_lod_de_forma(
            &mut pump.instances,
            &mut pump.vector_instances,
            shape_bake,
            &quer,
        );
        // ⭐⭐⭐ **E QUEM MOVEU TEM DE PODER DESFAZER** — senão APROXIMAR não devolve o desenho.
        //
        // ⛔⛔ A partição ESVAZIA o lado crisp, e com a cena parada o cozimento devolve cedo
        // (`if !self.dirty && self.last_cooked_tick == Some(tick)`): no quadro seguinte não há
        // instâncias para decidir, logo o LOD não pode mudar de ideias e os quads ficam **para
        // sempre**. O artista aproxima e a forma continua a ser a tile — exactamente o passo (7)
        // do roteiro desta cena, que prometia o contrário.
        //
        // ⚠️ E ele só marca quando de facto MOVEU: sem o LOD armado o cozimento não é forçado, e
        // o caminho de omissão fica como estava.
        if movidas > 0 {
            pump.mark_dirty();
        }
        // ⚠️ **E LARGA o que saiu de cena, libertando a textura.** Sem isto um
        // param de forma animado assa um tile por QUADRO e a placa acaba
        // (medido: OOM no quadro 19706 da `=76`).
        //
        // ⛔⛔ **O conjunto vivo NÃO é o `live` de cima desde que o LOD existe:** o que ele
        // moveu já não está em `vector_instances` — está em `instances`, como quad —, e com a
        // cena parada o cozimento devolve cedo e as duas listas persistem. Despejar pelo
        // `live` largaria a textura que os quads ainda amostram, para sempre.
        let vivas =
            ph2d_app_motion::motion_shape_lod::vivas_com_o_lod(&live, &pump.instances, shape_bake);
        let freed = shape_bake.evict_outside(&vivas, renderer);
        if freed > 0 && std::env::var_os("PH2D_GLOW_DIAG").is_some() {
            eprintln!("[glow-diag] assador de formas: tiles largados={freed}");
        }
        if asked > 0 && std::env::var_os("PH2D_GLOW_DIAG").is_some() {
            let done = pump
                .vector_instances
                .iter()
                .filter(|vi| shape_bake.tile_for_gid(vi.geometry_id).is_some())
                .count();
            eprintln!("[glow-diag] assador de formas: pedidas={asked} com_tile_agora={done}");
        }
    }
}
