//! **COMO UMA CENA NASCE** — o construtor, e só ele.
//!
//! Irmão (`#[path]`) de [`super`], e o corte é o mesmo que separa o
//! `SceneObject::new` do resto do `sculpt3d_objects`: o pai responde *o que uma
//! cena É* (a struct e o que cada campo significa) e os outros irmãos respondem
//! *o que se FAZ com ela*; aqui mora *de onde ela vem*.
//!
//! ⚠️ **O gate de LOC foi o gatilho, não a razão.** O pai cruzou os 600 do HR-18
//! quando o slot de imagem ganhou um campo, e o que saiu foi a metade com
//! fronteira própria — não a última coisa que alguém escreveu.

use ph2d_mesh::{Mesh, Pose};
use ph2d_mesh_render::{Camera3d, MeshRenderer};
use ph2d_sculpt3d::{Brush, SculptStroke, Symmetry};

use super::donation::FormRole;
use super::{DEFAULT_RADIUS_PX, ObjectId, SceneObject, Sculpt3dScene, dyntopo, scenes};

impl Sculpt3dScene {
    pub(crate) fn new(device: &wgpu::Device, mesh: Mesh, aspect: f32) -> Self {
        let mut camera = Camera3d {
            yaw: 0.6,
            pitch: 0.35,
            ..Camera3d::default()
        };
        camera.frame(mesh.bounds(), aspect);
        // ⚠️ **UM binding, dois campos** — o rig e a testemunha dele. Escrever a expressão duas
        // vezes deixaria a testemunha nascer discordando do rig no dia em que uma das duas mudasse,
        // e o defeito seria *"o Bake to Sprite disparou sozinho"*: a discordância vira um gesto de
        // lâmpada que ninguém fez. Aqui isso é inexprimível.
        let rig = scenes::shading::scene_rig().unwrap_or_default();
        Self {
            objects: vec![SceneObject::new(ObjectId(0), mesh, Pose::IDENTITY)],
            active: 0,
            next_id: 1,
            isolated: None,
            dyntopo: dyntopo::Dyntopo::default(),
            dyn_before: None,
            dyn_births: Vec::new(),
            dyn_remap: ph2d_mesh::Remap::default(),
            dyn_region: ph2d_mesh::RegionScratch::default(),
            slots: Vec::new(),
            camera,
            renderer: MeshRenderer::new(device, ph2d_render::GameRt::FORMAT),
            drag: None,
            last: (0.0, 0.0),
            viewport: (1, 1),
            // **O MATERIAL COM QUE O APP ABRE** — ver
            // [`ph2d_mesh_render::DEFAULT_MATCAP`], hoje o `Skin Haz 2`.
            //
            // ⚠️ **A const do RENDERIZADOR e não um literal aqui, porque o fato
            // já tem dono** (`MATCAPS[0]`, com gate que pina o nome). Ele
            // nascia `None` — o rig do documento — enquanto o
            // `Shade::default()` do renderizador dizia `Some(0)` e o
            // `sculpt3d_announce` anunciava *"o app abre no 'Skin Haz 2'"*:
            // **três respostas para uma pergunta, e a que ganhava era a que
            // ninguém tinha escrito de propósito**, porque é a shell quem passa
            // o campo ao `Shade`. O anúncio sobreviveu ao fato.
            matcap: ph2d_mesh_render::DEFAULT_MATCAP,
            wireframe: false,
            brush: Brush::default(),
            mode_by_verb: [ph2d_sculpt3d::RefMode::default(); ph2d_sculpt3d::Verb::ALL.len()],
            ui_level: ph2d_panel_sculpt3d::UiLevel::default(),
            alpha_preview: true,
            alpha_image: None,
            radius_px: DEFAULT_RADIUS_PX,
            stroke_anchor: [0.0, 0.0],
            grab: None,
            pending_grab: None,
            twist: None,
            transform_arm: None,
            transform: None,
            transform_from: (0.0, 0.0),
            // ⚠️ **DESLIGADA por default, e é decisão do smoke.** O ZBrush
            // nasce com espelho ligado — e MOSTRA isso. Aqui o artista clicava
            // de um lado e via uma segunda protuberância do outro, sem nada na
            // tela explicando por quê: *"o local onde está esculpindo não
            // coincide com a posição do mouse"*. Um default que só se descobre
            // por acidente é pior que um default menos ambicioso; o `X` liga.
            symmetry: Symmetry::default(),
            // ⚠️ **A cena pode ser dona da própria LUZ**, e uma cena de
            // sombreamento quase sempre é: ver [`scenes::shading::scene_rig`].
            rig,
            cavity: ph2d_mesh_render::DEFAULT_CAVITY,
            env: ph2d_mesh_render::DEFAULT_ENV,
            ao: ph2d_mesh_render::DEFAULT_AO_STRENGTH,
            ssao: ph2d_mesh_render::DEFAULT_SSAO_STRENGTH,
            sss: ph2d_mesh_render::SssParams::default().strength,
            sss_scatter: ph2d_mesh_render::SSS_SCATTER_FRACTION,
            extract: ph2d_mesh::Extract::default(),
            // A fonte é a const do motor, e não uma cópia dela.
            remesh_res: ph2d_sdf::DEFAULT_RESOLUTION,
            stroke: SculptStroke::default(),
            undo: Vec::new(),
            redo: Vec::new(),
            edits: 0,
            role: FormRole::Clay,
            clay_was_on: false,
            // ⚠️ **O rig com que ela NASCE**, e não um valor neutro: a cena acabou de existir e não
            // mexeu em lâmpada nenhuma. Semear isto com um default faria o primeiro frame parecer
            // um gesto do artista — e o preço é a luz de todo objeto já assado no documento.
            rig_was: crate::baked_form::rig_stamp(&rig),
            donated: None,
        }
    }
}
