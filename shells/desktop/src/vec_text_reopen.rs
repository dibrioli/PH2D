//! **Reabrir** um texto já finalizado para digitar de novo — módulo irmão de
//! [`crate::vec_text`] (a sessão) e [`crate::vec_text_object`] (o objeto); separado
//! pelo teto de 600 LOC/arquivo (HR-18).
//!
//! Sem isto a string de um texto commitado era IMUTÁVEL: as propriedades (fonte,
//! tamanho, peso) eram editáveis pela seleção, mas o conteúdo não — um typo ficava para
//! sempre. Dois gestos, os de todo editor vetorial:
//!
//! - **Ferramenta Text clicando SOBRE um texto** ⇒ entra nele (padrão Illustrator).
//! - **Duplo-clique no modo Select** ⇒ entra nele (o gesto que o usuário tenta primeiro).
//!
//! Ambos passam por [`reopen_text_session`], e a sutileza está lá: a pose.

use ph2d_ecs::{Entity, SimWorld, Transform, VecShape, VecTextParams};
use ph2d_vec_scene::{VecPathId, VecScene};
use ph2d_vector_font::AxisTag;

use crate::vec_entities::VecEntityMap;
use crate::vec_glyph::text_to_compound_path;
use crate::vec_text::VecTextEdit;
use crate::vec_text_object::{align_from_u8, axes_of_params, layout_of_params};

/// REABRE um objeto de texto já finalizado como SESSÃO de digitação — é o que torna a
/// string editável depois do commit (antes, só as propriedades eram; um typo era
/// para sempre). Devolve a sessão pronta, com o caret no fim.
///
/// **A pose é a sutileza.** Enquanto a sessão vive, o `upsert_text_shape` reescreve a
/// pose a cada frame como `Transform = origin + center` — então reabrir com o `origin`
/// GRAVADO nos params teleportaria o texto de volta ao ponto onde foi criado, perdendo
/// todo `move` do gizmo (o gizmo mexe no `Transform`, não no `origin`). Por isso o
/// `origin` é DEDUZIDO da pose atual: `origin = translation − center`, com o `center`
/// vindo do mesmo cozimento que a sessão vai usar. Assim a identidade fecha e o texto
/// não se move ao ser reaberto.
#[must_use]
pub(crate) fn reopen_text_session(
    sim: &SimWorld,
    id: VecPathId,
    entity: Entity,
    params: &VecTextParams,
    scene: &VecScene,
) -> VecTextEdit {
    let font = crate::vec_font::resolve(params.family.as_deref());
    let layout = layout_of_params(params);
    let axes = axes_of_params(params);
    let (fill, stroke) = scene
        .paths()
        .iter()
        .find(|p| p.id == id)
        .map_or((None, None), |p| (p.fill.clone(), p.stroke));
    // O MESMO cozimento da sessão (baseline local, depois centrado) — daqui sai o
    // `center` que liga a pose ao `origin`.
    let center = text_to_compound_path(
        &font,
        &params.text,
        &layout,
        &axes,
        [0.0, 0.0],
        &fill,
        &stroke,
    )
    .map_or([0.0, 0.0], |c| crate::vec_glyph::path_center(&c));
    // A pose que o usuário deixou (o gizmo pode ter movido o texto desde a criação).
    let pose = sim
        .world()
        .get::<Transform>(entity)
        .map_or([0.0, 0.0], |t| {
            [f64::from(t.translation.x), f64::from(t.translation.y)]
        });
    VecTextEdit {
        origin: [pose[0] - center[0], pose[1] - center[1]],
        size: params.size,
        weight: params.weight,
        line_height: params.line_height,
        tracking: params.tracking,
        align: align_from_u8(params.align),
        extra_axes: params
            .axes
            .iter()
            .map(|(b, v)| (AxisTag::new(*b), *v))
            .collect(),
        family: params.family.clone(),
        fill,
        stroke,
        text: params.text.clone(),
        id: Some(id),
        center,
    }
}

/// O objeto de TEXTO sob o ponto `world` (o path que o hit-test pegou É um texto vivo).
/// `None` = o clique não caiu num texto — o chamador cria uma sessão nova.
#[must_use]
pub(crate) fn text_object_at(
    sim: &SimWorld,
    map: &VecEntityMap,
    id: VecPathId,
) -> Option<(Entity, VecTextParams)> {
    let &bits = map.get(&id)?;
    let e = Entity::from_bits(bits);
    match sim.world().get::<VecShape>(e) {
        Some(VecShape::Text(p)) => Some((e, p.clone())),
        _ => None,
    }
}

impl crate::app_state::App {
    /// O cursor está sobre o CANVAS — aceitando o gizmo por cima. É a guarda do
    /// duplo-clique, e a diferença para o `on_canvas` do dispatch é o ponto todo: aquele
    /// exige o `hit_index` vazio, mas o gizmo registra as alças (e o interior de
    /// translação) NELE. Como o 1º clique seleciona o objeto, no 2º o gizmo já cobre a
    /// forma — com `on_canvas` o gesto morria aí. Um painel (ou qualquer outro widget)
    /// sob o cursor segue barrando.
    #[must_use]
    pub(crate) fn over_canvas_or_gizmo(&self, x: f32, y: f32) -> bool {
        let Some(hero) = self.gfx.as_ref().and_then(|g| g.hero_screen.as_ref()) else {
            return false;
        };
        if hero.store.panel_at(x, y).is_some() {
            return false;
        }
        match hero.hit_index.hit(x, y) {
            None => true, // canvas cru
            Some(id) => {
                ph2d_editor::gizmo_kind_for_id(id).is_some()
                    || hero.gizmo.gizmo_hit_map.contains_key(&id)
            }
        }
    }

    /// Janela do duplo-clique (a mesma do chrome) e a folga de posição: um segundo
    /// clique além disso é um clique novo, não um duplo.
    const DOUBLE_CLICK_MS: u128 = 350;
    const DOUBLE_CLICK_SLOP_PX: f32 = 5.0;

    /// **Duplo-clique num texto no modo Select entra na edição dele** — o gesto que todo
    /// editor vetorial tem, e o primeiro que o usuário tenta. O canvas não emite
    /// `DoubleClick` (esse evento é por-widget do chrome), então o par (instante,
    /// posição) do clique anterior é rastreado aqui. `true` = entrou num texto (o
    /// chamador consome a pressão; sem isso ela iria para o gizmo e arrastaria a forma).
    pub(crate) fn vec_text_double_click(&mut self, x: f32, y: f32) -> bool {
        let now = std::time::Instant::now();
        let is_double = self.vec_last_canvas_click.is_some_and(|(t, (px, py))| {
            now.duration_since(t).as_millis() <= Self::DOUBLE_CLICK_MS
                && (x - px).abs() <= Self::DOUBLE_CLICK_SLOP_PX
                && (y - py).abs() <= Self::DOUBLE_CLICK_SLOP_PX
        });
        self.vec_last_canvas_click = Some((now, (x, y)));
        // Diagnóstico opt-in (`PH2D_TEXT_LOG=1`): o gesto tem três degraus (é duplo? há
        // path sob o cursor? o path é um texto?) e o 1º smoke morreu no degrau da guarda
        // — sem isto, um "não abre" volta a ser adivinhação.
        let log = std::env::var_os("PH2D_TEXT_LOG").is_some();
        if !is_double {
            if log {
                eprintln!("[text] Down em ({x:.0},{y:.0}) — clique simples");
            }
            return false;
        }
        let Some(w) = self
            .gfx
            .as_ref()
            .map(|gfx| gfx.camera.screen_to_world((x, y), gfx.surface.size()))
        else {
            return false;
        };
        let Some(edit) = self.vec_text_reopen_at([f64::from(w[0]), f64::from(w[1])]) else {
            if log {
                eprintln!("[text] duplo-clique, mas nao ha TEXTO sob o cursor");
            }
            return false; // não era texto: o duplo-clique segue o caminho normal
        };
        if log {
            eprintln!("[text] duplo-clique: reabrindo \"{}\"", edit.text);
        }
        self.vec_set_draw_mode(ph2d_tool_vector::DrawMode::Text);
        self.vec_text_edit = Some(edit);
        self.vec_text_regen();
        true
    }

    /// A sessão de digitação do objeto de TEXTO sob `world`, se o clique caiu num.
    /// `None` = não há texto ali (o chamador cria um novo). O raio de captura é o mesmo
    /// do clique de seleção (10 px), para pegar o texto pelo contorno dos glyphs.
    pub(crate) fn vec_text_reopen_at(&mut self, world: [f64; 2]) -> Option<VecTextEdit> {
        const HIT_PX: f64 = 10.0;
        let gfx = self.gfx.as_ref()?;
        let hit = self
            .vec_pen
            .path_at(&gfx.vec_scene, world, HIT_PX * self.vec_px_to_world())?;
        let (entity, params) = text_object_at(&gfx.sim, &self.vec_entities, hit)?;
        let edit = reopen_text_session(&gfx.sim, hit, entity, &params, &gfx.vec_scene);
        // O objeto reaberto vira a seleção (o painel passa a mostrá-lo, e o Convert /
        // recolor agem sobre ele) — é o que o usuário acabou de apontar.
        self.vec_pen.select_many(&[hit]);
        Some(edit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec_text::regen_into;
    use crate::vec_text_object::upsert_text_shape;
    use ph2d_tool_vector::TextAlign;
    use ph2d_vec_scene::{Paint, Rgba8};

    fn black() -> Paint {
        Paint::solid(Rgba8::new(0, 0, 0, 255))
    }

    /// Reabrir um texto MOVIDO não pode teleportá-lo de volta ao ponto onde foi criado.
    /// A sessão viva reescreve a pose todo frame como `origin + center`, então o
    /// `origin` tem de ser DEDUZIDO da pose atual — usar o `origin` gravado nos params
    /// (que é o do clique original) perderia todo `move` do gizmo. Este teste fixa
    /// exatamente isso: params com `origin` velho + entidade movida ⇒ a sessão
    /// reconstruída reproduz a pose ATUAL.
    #[test]
    fn reopening_a_moved_text_keeps_it_where_the_user_left_it() {
        use crate::vec_text::VecTextEdit;
        let mut sim = SimWorld::default();
        let mut scene = VecScene::new();
        let mut map = VecEntityMap::new();

        // Um texto criado em [0,0]…
        let mut edit = Some(VecTextEdit {
            origin: [0.0, 0.0],
            size: 1.0,
            weight: 400.0,
            line_height: 1.2,
            tracking: 0.0,
            align: TextAlign::Left,
            extra_axes: Vec::new(),
            family: None,
            fill: Some(black()),
            stroke: None,
            text: "Hi".to_string(),
            id: None,
            center: [0.0, 0.0],
        });
        regen_into(&mut scene, edit.as_mut().unwrap());
        crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
        upsert_text_shape(&mut sim, &map, edit.as_ref().unwrap());
        let id = edit.unwrap().id.expect("o compound");
        let entity = Entity::from_bits(*map.get(&id).expect("a entidade"));

        // …e depois MOVIDO pelo gizmo (que mexe no Transform, não no `origin` dos
        // params — que segue dizendo [0,0]).
        const MOVED: [f32; 2] = [40.0, -15.0];
        if let Ok(mut e) = sim.world_mut().get_entity_mut(entity)
            && let Some(mut t) = e.get_mut::<Transform>()
        {
            t.translation = ph2d_core::Vec2::new(MOVED[0], MOVED[1]);
        }
        let params = match sim.world().get::<VecShape>(entity) {
            Some(VecShape::Text(p)) => p.clone(),
            _ => panic!("é um texto vivo"),
        };
        assert_eq!(
            params.origin,
            [0.0, 0.0],
            "os params guardam o origin VELHO"
        );

        // Reabrir: a sessão reconstruída, ao reescrever a pose (`origin + center`),
        // TEM de cair de volta em MOVED — não em [0,0].
        let mut reopened = reopen_text_session(&sim, id, entity, &params, &scene);
        assert_eq!(
            reopened.text, "Hi",
            "a string voltou (é o ponto da feature)"
        );
        regen_into(&mut scene, &mut reopened);
        upsert_text_shape(&mut sim, &map, &reopened);
        let t = sim.world().get::<Transform>(entity).expect("Transform");
        assert!(
            (t.translation.x - MOVED[0]).abs() < 1e-4 && (t.translation.y - MOVED[1]).abs() < 1e-4,
            "o texto reaberto pulou para {:?} em vez de ficar em {MOVED:?}",
            t.translation
        );
    }
}
