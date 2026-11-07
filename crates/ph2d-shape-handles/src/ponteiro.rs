//! **O gesto:** quem está em edição e o arrasto em curso, num recurso do mundo (o cartão lê-o pelo retrato), e a
//! porta que o shell chama com o ponteiro real, em píxeis de ecrã.

use crate::{ARMA_PX, Alca, Medida, Pose, Teclas, alca_em, ponto_local, puxa};
use bevy_ecs::prelude::Resource;
use bevy_ecs::world::World;
use ph2d_ecs::Entity;
use ph2d_editor_core::action_bus::{ActionBus, EditorAction};
use ph2d_host::WindowSize;
use ph2d_physics_ecs::Collider;
use ph2d_render::Camera2d;

pub type Mods = Teclas;

/// Um arrasto: tudo parte do `Down`, nada lê a forma já mexida.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Arrasto {
    pub alca: Alca,
    pub inicio: Medida,
    /// A folga entre o ponteiro e a alça no `Down`, no referencial escalado: a alça não salta ao agarrar.
    folga: [f32; 2],
    ecra0: (f32, f32),
    pub armado: bool,
}

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct FormaNoCanvas {
    pub alvo: Option<u64>,
    /// A identidade durável do alvo: o undo pode devolver o mesmo objecto com outros bits.
    id: Option<ph2d_ecs::StableId>,
    pub arrasto: Option<Arrasto>,
}

/// O objecto em edição.
#[must_use]
pub fn editando(world: &World) -> Option<u64> {
    world.get_resource::<FormaNoCanvas>().and_then(|f| f.alvo)
}

/// As alças da forma estão acesas?
#[must_use]
pub fn ativo(world: &World) -> bool {
    editando(world).is_some()
}

/// O botão do cartão: liga neste objecto; carregar de novo desliga.
pub fn alterna(world: &mut World, bits: u64) {
    let id = world
        .get::<ph2d_ecs::StableId>(Entity::from_bits(bits))
        .copied();
    let mut f = world.get_resource_or_insert_with(FormaNoCanvas::default);
    f.arrasto = None;
    f.alvo = if f.alvo == Some(bits) { None } else { Some(bits) };
    f.id = id;
}

/// Uma vez por quadro: trocar de objecto, dar Play ou ficar sem forma encerram. Devolve se continua aceso.
pub fn quadro(world: &mut World, selecao: Option<u64>, jogando: bool) -> bool {
    let Some(mut alvo) = editando(world) else {
        return false;
    };
    // O mesmo objecto depois de um Ctrl+Z que o refez: a edição continua nele.
    if let Some(s) = selecao.filter(|s| *s != alvo) {
        let f = world.get_resource::<FormaNoCanvas>().copied().unwrap_or_default();
        if f.id.is_some() && world.get::<ph2d_ecs::StableId>(Entity::from_bits(s)).copied() == f.id {
            alvo = s;
            if let Some(mut f) = world.get_resource_mut::<FormaNoCanvas>() {
                f.alvo = Some(s);
            }
        }
    }
    let vivo = selecao == Some(alvo)
        && !jogando
        && world.get::<Collider>(Entity::from_bits(alvo)).is_some();
    if !vivo && let Some(mut f) = world.get_resource_mut::<FormaNoCanvas>() {
        *f = FormaNoCanvas::default();
    }
    vivo
}

/// O ponteiro do shell: o mundo, a vista desenhada neste quadro e o barramento do Inspector.
pub struct Porta<'a> {
    pub world: &'a mut World,
    pub camera: Camera2d,
    pub window: WindowSize,
    pub bus: &'a mut ActionBus,
}

impl Porta<'_> {
    fn forma(&self, bits: u64) -> Option<(Pose, Medida)> {
        let e = Entity::from_bits(bits);
        let col = self.world.get::<Collider>(e)?;
        Some((Pose::de(self.world, e)?, Medida::de(col)))
    }

    /// `true` = agarrou (o `Down` não segue para a seleção).
    pub fn desce(&mut self, s: (f32, f32)) -> bool {
        let Some(alvo) = editando(self.world) else {
            return false;
        };
        let Some((pose, m)) = self.forma(alvo) else {
            return false;
        };
        let Some(alca) = alca_em(&pose, &m, &self.camera, self.window, s) else {
            return false;
        };
        let q = pose.do_mundo(self.camera.screen_to_world(s, self.window));
        let p = ponto_local(&pose, &m, alca);
        let arrasto = Arrasto {
            alca,
            inicio: m,
            folga: [q[0] - p[0], q[1] - p[1]],
            ecra0: s,
            armado: false,
        };
        if let Some(mut f) = self.world.get_resource_mut::<FormaNoCanvas>() {
            f.arrasto = Some(arrasto);
        }
        true
    }

    /// `true` = consumiu o movimento. Pose e vista são as de agora: o pai pode ter andado.
    pub fn move_(&mut self, s: (f32, f32), teclas: Teclas) -> bool {
        let Some(f) = self.world.get_resource::<FormaNoCanvas>().copied() else {
            return false;
        };
        let (Some(mut a), Some(alvo)) = (f.arrasto, f.alvo) else {
            return false;
        };
        if !a.armado && (s.0 - a.ecra0.0).hypot(s.1 - a.ecra0.1) < ARMA_PX {
            return true;
        }
        a.armado = true;
        let Some((pose, _)) = self.forma(alvo) else {
            return true;
        };
        let q = pose.do_mundo(self.camera.screen_to_world(s, self.window));
        let e = [q[0] - a.folga[0], q[1] - a.folga[1]];
        let novo = puxa(&a.inicio, a.alca, &pose, e, teclas);
        self.escreve(alvo, &novo);
        if let Some(mut f) = self.world.get_resource_mut::<FormaNoCanvas>() {
            f.arrasto = Some(a);
        }
        true
    }

    /// `true` = havia um arrasto. O desfazer pós-quadro fecha o gesto num passo.
    pub fn sobe(&mut self) -> bool {
        self.world
            .get_resource_mut::<FormaNoCanvas>()
            .and_then(|mut f| f.arrasto.take())
            .is_some()
    }

    /// O `Esc`: a meio do arrasto devolve a forma do `Down`; parado, encerra a edição. `true` = consumido.
    pub fn esc(&mut self) -> bool {
        let Some(f) = self.world.get_resource::<FormaNoCanvas>().copied() else {
            return false;
        };
        let Some(alvo) = f.alvo else {
            return false;
        };
        if let Some(a) = f.arrasto {
            if a.armado {
                self.escreve(alvo, &a.inicio);
            }
            if let Some(mut f) = self.world.get_resource_mut::<FormaNoCanvas>() {
                f.arrasto = None;
            }
        } else if let Some(mut f) = self.world.get_resource_mut::<FormaNoCanvas>() {
            f.alvo = None;
        }
        true
    }

    fn escreve(&mut self, alvo: u64, m: &Medida) {
        self.bus.push(EditorAction::InspectorPhysicsEdit {
            entity_bits: alvo,
            edit: m.edicao(),
        });
    }
}
