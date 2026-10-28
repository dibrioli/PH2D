//! O que os testes das vistas partilham: a cena 1 com datas num «agora» fixo em UTC, o contexto e o
//! fotógrafo.

use ph2d_kanban_dates::{CivilDate, Weekday, Zone, instant};
use ph2d_kanban_filter::Filter;
use ph2d_kanban_model::{KanbanId, KanbanSet, Member, Moment};
use ph2d_kanban_views::Ctx;

/// 2026-10-28 10:00 UTC (uma quarta-feira).
pub fn now() -> Moment {
    let d = CivilDate::new(2026, 10, 28).expect("data");
    Moment(instant(d, 10, 0, &Zone::utc()).expect("instante"))
}

/// O que um contexto pede emprestado.
pub struct World {
    pub set: KanbanSet,
    pub id: KanbanId,
    pub zone: Zone,
    pub me: Member,
    pub team: Vec<Member>,
    pub filter: Filter,
}

impl World {
    /// A cena 1 com datas.
    pub fn scene1() -> Self {
        let mut set = KanbanSet::default();
        let id = ph2d_kanban_views::fixture::short_film(&mut set, now());
        Self::over(set, id)
    }

    pub fn over(set: KanbanSet, id: KanbanId) -> Self {
        Self {
            set,
            id,
            zone: Zone::utc(),
            me: Member("Enio".into()),
            team: ["Enio", "Ana", "Rui", "Mia"]
                .into_iter()
                .map(|n| Member(n.into()))
                .collect(),
            filter: Filter::default(),
        }
    }

    pub fn cx(&self) -> Ctx<'_> {
        Ctx {
            now: now(),
            zone: &self.zone,
            me: &self.me,
            team: &self.team,
            filter: &self.filter,
            week_start: Weekday::Sunday,
        }
    }

    /// Aplica um plano (como o app: `apply_logged`, um lote por Kanban).
    pub fn apply(&mut self, plan: ph2d_kanban_model::Plan) {
        let actor = ph2d_kanban_model::Actor {
            by: self.me.clone(),
            at: now(),
        };
        for (k, ops) in plan.into_steps() {
            if let Some(doc) = self.set.doc_mut(k) {
                ph2d_kanban_model::apply_logged(doc, ops, &actor);
            }
        }
    }
}

/// 📷 Uma janela: a placa e o texto; grava `<name>.png` em `docs/TrelloClone/smoke/W7/`.
pub struct Shots {
    gpu: ph2d_gpu::GpuContext,
    pub ts: ph2d_text::TextSystem,
}

impl Shots {
    pub fn new() -> Self {
        let gpu =
            ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).expect("gpu");
        Self {
            gpu,
            ts: ph2d_text::TextSystem::new(),
        }
    }

    /// Pinta com `paint` (duas vezes: a 1.ª assenta o que o desenho mede) e grava.
    pub fn shot(
        &mut self,
        name: &str,
        size: (u32, u32),
        mut paint: impl FnMut(&mut ph2d_vector::VectorScene, &mut ph2d_text::TextSystem),
    ) {
        let mut pass =
            ph2d_render::VelloPass::new(&self.gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, size)
                .expect("pass");
        let mut scene = ph2d_vector::VectorScene::new();
        paint(&mut scene, &mut self.ts);
        let mut scene = ph2d_vector::VectorScene::new();
        paint(&mut scene, &mut self.ts);
        let px = pass
            .render_and_readback(&self.gpu, scene.inner(), size)
            .expect("readback");
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/TrelloClone/smoke/W7");
        std::fs::create_dir_all(&dir).expect("pasta");
        let path = dir.join(format!("{name}.png"));
        image::save_buffer(&path, &px, size.0, size.1, image::ExtendedColorType::Rgba8)
            .expect("png");
        println!("foto: {}", path.display());
    }
}
