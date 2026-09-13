//! **Fase do quadro: O DESENHO DO MODELADOR 3D** — o traçado do campo implícito (`ph2d_app_field3d::smoke::draw`)
//! na área do canvas (OBRA 2 da `line/render-loop`, 2026-09-12).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_field3d_smoke_draw(&mut self, viewport: EditorRect) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            #[cfg(feature = "sculpt3d")]
            sculpt3d,
            theme,
            vector_scene,
            text_system,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        let paint_ctx = PaintCtx {
            theme: *theme,
            viewport,
            text: text_system,
        };
        // ⭐⭐⭐ **A ÁREA, e não a JANELA** (report do Enio, 2026-08-31: *«a viewport ainda não
        // se encaixa na área correta para ela — veja que atravessa as réguas»*).
        //
        // ⛔⛔ Isto era `viewport` cru, então os quadrantes ladrilhavam o ecrã inteiro: por
        // baixo da barra de menus, da fila de ferramentas, da coluna da esquerda e das duas
        // réguas — e com a divisão aberta as costuras caíam onde não há área nenhuma.
        //
        // ⚠️ **A MESMA porta que alimenta o gizmo de navegação** (`ph2d_viewport3d::layout::area`), e é
        // por isso que os dois se encaixam: um segundo rect aqui seria a fonte por onde a
        // imagem e a moldura voltavam a discordar.
        ph2d_app_field3d::smoke::draw(
            ph2d_viewport3d::layout::area(
                hero,
                ph2d_editor_core::zones::Rect::new(viewport.x, viewport.y, viewport.w, viewport.h),
            ),
            hero.theme,
            paint_ctx.text,
            vector_scene,
        );
        // ⭐⭐⭐ **O GIZMO DA VIEWPORT DA ESCULTURA** (2026-09-08) — as seis bolas de eixo, a
        // MESMA lei e o MESMO pintor do módulo vizinho (`ph2d_viewport3d::navball`), com a base desta
        // câmera. Ver `sculpt3d_navball`.
        //
        // ⚠️ **Pintado aqui e não no bloco do anel do pincel**, que corre ~3 000 linhas acima:
        // é aqui que o par `área`/`safe` deste quadro já foi publicado, e desenhá-lo antes
        // usaria o do quadro anterior — meio widget deslocado sempre que uma coluna abrisse.
        //
        // ⚠️ **Sem barro na tela ele não é pintado**: o módulo está desarmado, e um gizmo de
        // navegação sobre uma cena 2D prometeria um gesto que não existe ali.
        #[cfg(feature = "sculpt3d")]
        if let Some(scene) = sculpt3d.as_mut()
            && scene.clay_on_screen()
        {
            // ⭐⭐⭐ **O RÓTULO DE CADA VISTA** — só com a divisão aberta. Com uma vista só a
            // pergunta *«qual é qual?»* não existe, e um rótulo permanente seria ruído sobre a
            // peça. ⚠️ A chave sai da CÂMERA daquele quadrante, nunca do sítio dele: orbitar a
            // vista de cima faz dela *User*, que é o que ela passou a ser.
            let quadros = scene.vp_rects();
            // ⚠️ **O chip de cada rótulo é PUBLICADO por quem o pinta** — a largura dele é a do
            // TEXTO, e só o pintor a mede. É ele o alvo do clique que abre o menu daquela
            // vista; com **uma** vista não há rótulo e a lista fica vazia, o que faz o chip ser
            // inalcançável em vez de invisível-mas-clicável.
            let chips: Vec<_> = if quadros.len() > 1 {
                quadros
                    .iter()
                    .enumerate()
                    .map(|(i, r)| {
                        ph2d_app_field3d::gizmo_paint::paint_view_label(
                            vector_scene,
                            paint_ctx.text,
                            *r,
                            scene.vp_label_key(i),
                            hero.theme,
                        )
                    })
                    .collect()
            } else {
                Vec::new()
            };
            scene.note_view_labels(chips);
            // ⭐⭐ **AS COSTURAS E A MOLDURA DO ACTIVO** — o MESMO pintor do módulo vizinho.
            // Ele já é no-op com uma vista só.
            ph2d_app_field3d::gizmo_paint::paint_split(
                vector_scene,
                &quadros,
                scene.vp_active(),
                hero.theme,
            );
            // ⭐⭐⭐ **O GIZMO DE TRANSFORMAÇÃO** (2026-09-08) — as alças por cima da peça e no
            // referencial da JANELA (a `SculptCam` já lhes soma a quina do quadrante activo).
            //
            // ⚠️ **Sem teste de profundidade, e é o que todo modelador faz**: uma alça
            // escondida atrás da superfície que ela move seria inalcançável exactamente quando
            // o artista precisa dela.
            //
            // ⚠️ **Vazio sem transform armado** — a lista sai vazia da porta, e o pintor é
            // no-op sobre ela. *Um `if` aqui seria a segunda resposta a «há gizmo agora?».*
            {
                let handles = scene.gizmo_handles();
                if !handles.is_empty() {
                    ph2d_app_field3d::gizmo_paint::paint(
                        vector_scene,
                        &handles,
                        scene.gizmo_hot(),
                        hero.theme,
                        // ⚠️ **Origem ZERO**: ao contrário do módulo vizinho, estas alças já
                        // chegam em coordenadas de janela. Somar a quina aqui seria somá-la
                        // duas vezes, e o gizmo sairia ao dobro da distância da peça.
                        [0.0, 0.0],
                    );
                }
            }
            // ⭐⭐ **O GIZMO DA VIEWPORT**, por cima da moldura — que é onde um gizmo de janela
            // vive. Ver `sculpt3d_navball`.
            if let Some((area, safe)) = scene.nav_rects() {
                let balls = scene.navball(area, safe);
                ph2d_viewport3d::navball_paint::paint(
                    vector_scene,
                    &balls,
                    scene.nav_hot(),
                    hero.theme,
                    [area.x, area.y],
                    ph2d_viewport3d::navball::centre_in(area, safe),
                );
            }
            // ⭐⭐⭐ **O MENU DA VISTA, POR CIMA DE TUDO** (report do Enio, 2026-09-08: *«ao
            // clicar nos nomes das views não aparece a lista de view como no módulo de
            // modelagem 3d»*) — ele é modal, e o clique seguinte é dele caia onde cair.
            //
            // ⚠️ **O rectângulo é PUBLICADO por quem o pinta**, como o chip: a largura dele é a
            // da linha mais comprida, e só o pintor a mede. É o mesmo pintor do módulo vizinho.
            if let Some(i) = scene.view_menu_open()
                && let (Some(chip), Some(canvas)) = (scene.chip_of(i), scene.canvas())
            {
                let r = ph2d_app_field3d::gizmo_paint::paint_view_menu(
                    vector_scene,
                    paint_ctx.text,
                    chip,
                    canvas,
                    hero.theme,
                );
                scene.note_view_menu_rect(r);
            }
        }
    }
}
