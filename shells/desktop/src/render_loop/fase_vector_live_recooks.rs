//! **Fase do quadro: AS ETIQUETAS, A VISTA DO PEN E AS RECOZEDURAS VIVAS** — as etiquetas vivas, a vista e as
//! transformações entregues ao pen, a sincronia da selecção, o afim da câmera do Vello e as recozeduras de
//! offset, padrão, dilatação e contorno (OBRA 2 da `line/render-loop`, 2026-09-12).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_vector_live_recooks(
        &mut self,
        window_size: ph2d_host::WindowSize,
        vector_active: bool,
        vec_view: ph2d_vec_scene::VecViewState,
        mut vec_xf: ph2d_vec_scene::VecXforms,
    ) -> Option<(
        ph2d_vector::Affine,
        ph2d_vec_scene::VecXforms,
        ph2d_vec_scene::VecViewState,
    )> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            sim,
            camera,
            vec_scene,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        // **Rótulos:** o texto que pertence a uma forma (ou a um conector) e a segue. A pose
        // é uma função pura do hospedeiro — como a rota do conector é da relação dele.
        //
        // **DEPOIS do `recook`, e isso não é arrumação:** a âncora do rótulo de um conector é
        // o meio da rota, e a rota deste frame acabou de ser escrita ali em cima. Antes do
        // `recook` o rótulo penderia da polilinha do frame ANTERIOR — e arrastaria a forma
        // sempre um quadro atrás da linha. (E depois do `build`, pela mesma razão: os afins
        // das formas-alvo já são os deste frame.)
        //
        // O `upkeep_pending` vem primeiro: um rótulo nasce VAZIO, e é a 1ª letra que cria o
        // objeto — o vínculo tem de estar pendurado antes do passe procurar por ele.
        let text_id = self.vec.text_edit.as_ref().and_then(|e| e.id);
        crate::label_live::upkeep_pending(
            sim,
            &self.vec.entities,
            &mut self.vec.label_pending,
            text_id,
            self.vec.text_edit.is_some(),
        );
        crate::label_live::upkeep(
            sim,
            vec_scene,
            &self.vec.entities,
            &mut vec_xf,
            text_id,
            &mut self.vec.label_poses,
        );
        self.vec.pen.set_view(vec_view.clone());
        self.vec.pen.set_xforms(vec_xf.clone());
        // Seleção casada nos dois sentidos: clique na Hierarquia chega no canvas,
        // clique no canvas acende a linha (e a do grupo, se cheio). A seleção do
        // gizmo é COMPARTILHADA com os sprites — só o subconjunto vetorial é nosso.
        crate::vec_selection::sync_selection(
            &mut hero.gizmo,
            sim,
            vec_scene,
            &self.vec.entities,
            &mut self.vec.pen,
            &mut self.vec.sel,
            vector_active,
        );

        // Motion drift fix (2026-07-25 continuação): sob o split da tool Motion a CENA
        // renderiza num sub-retângulo (present.rs, `CenterSplit::scene_viewport`) e as
        // instâncias/grade projetam com as dims DA CENA (a porta única). As formas VETORIAIS
        // ficaram de fora daquele fix e projetavam a JANELA CHEIA — então um `motion.path`
        // andava numa cópia da curva deslocada+encolhida (o report do Enio: "objetos afastados
        // do path, com drift em relação ao canvas"). Projetá-las com as MESMAS dims casa a
        // curva desenhada com os walkers. Fora do split = janela cheia, byte-idêntico.
        let cam_affine = camera.world_to_screen_affine(
            ph2d_app_motion::field_gizmo::scene_camera_window(hero.view.center_split, window_size),
        );
        // A SONDA (`PH2D_PAN_DIAG=1`): a cena do Vello é construída AQUI, com o
        // mundo→tela já aplicado na CPU; as sprites recebem a câmera noutro ponto do
        // quadro. Guardar o centro daqui é o que permite comparar os dois instantes.
        ph2d_pan_diag::note_vello_camera(camera.center);
        // A geometria DERIVADA deste frame — hoje, os offsets vivos. Cozida aqui (depois
        // do `sync`, senão uma forma recém-criada ainda não tem entidade e o componente
        // dela não seria encontrado) e desenhada pelo `dispatch` no z de cada forma.
        self.offset_live
            .recook(vec_scene, sim, &self.vec.entities, &vec_xf);
        // O Pattern Along Path vivo (plano 23): as cópias de um motivo ao longo de um guia,
        // cozidas aqui e desenhadas no z do motivo — a fonte nunca é tocada.
        self.pattern_live.recook(vec_scene, sim, &self.vec.entities);
        // ⭐ O offset de CAD de cada camada (v22). ⚠️ Só precisa da CENA: a distância é LOCAL,
        // então a pose não entra na chave — é isso que faz o memo sobreviver ao arrasto.
        self.paint_dilate_live.recook(vec_scene);
        // O Contour vivo (pesquisa 20 #9): os anéis concêntricos + a rampa de cor, cozidos
        // aqui e desenhados no z da fonte — que entra na lista junto com eles.
        self.contour_live
            .recook(vec_scene, sim, &self.vec.entities, &vec_xf);
        Some((cam_affine, vec_xf, vec_view))
    }
}
