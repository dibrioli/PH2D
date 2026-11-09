//! **O gesto:** quem está em edição e o arrasto em curso, num recurso do mundo (o cartão lê-o pelo retrato), e a
//! porta que o shell chama com o ponteiro real, em píxeis de ecrã.

use crate::{ARMA_PX, Alca, Medida, Pose, Teclas, alca_em, no_ecra, ponto_local, puxa};
use bevy_ecs::prelude::Resource;
use bevy_ecs::world::World;
use ph2d_ecs::canal::familia::FISICA;
use ph2d_ecs::{Entity, MaosDasFamilias};
use ph2d_editor_core::action_bus::{ActionBus, EditorAction};
use ph2d_host::WindowSize;
use ph2d_physics_ecs::Collider;
use ph2d_render::Camera2d;

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
    f.alvo = if f.alvo == Some(bits) {
        None
    } else {
        Some(bits)
    };
    f.id = id;
}

/// Uma vez por quadro: trocar de objecto, dar Play ou ficar sem forma encerram. Devolve se continua aceso.
pub fn quadro(world: &mut World, selecao: Option<u64>, jogando: bool) -> bool {
    let Some(mut alvo) = editando(world) else {
        return false;
    };
    // O mesmo objecto depois de um Ctrl+Z que o refez: a edição continua nele.
    if let Some(s) = selecao.filter(|s| *s != alvo) {
        let f = world
            .get_resource::<FormaNoCanvas>()
            .copied()
            .unwrap_or_default();
        if f.id.is_some()
            && world
                .get::<ph2d_ecs::StableId>(Entity::from_bits(s))
                .copied()
                == f.id
        {
            alvo = s;
            if let Some(mut f) = world.get_resource_mut::<FormaNoCanvas>() {
                f.alvo = Some(s);
            }
        }
    }
    let vivo = selecao == Some(alvo)
        && !jogando
        && world.get::<Collider>(Entity::from_bits(alvo)).is_some();
    if !vivo {
        if let Some(mut f) = world.get_resource_mut::<FormaNoCanvas>() {
            *f = FormaNoCanvas::default();
        }
        world.remove_resource::<crate::pontos::PontosNoCanvas>();
        publica_a_mao(world, None);
    }
    vivo
}

/// A mão da família: quem ela segura não é conduzido pela timeline, e o Auto Key grava ao soltar, num passo só.
fn publica_a_mao(world: &mut World, alvo: Option<u64>) {
    world
        .get_resource_or_insert_with(MaosDasFamilias::default)
        .publicar(FISICA, alvo.into_iter().collect());
}

/// Um arrasto armado está a meio: o `Esc` tem o que desfazer.
#[must_use]
pub fn a_meio(world: &World) -> bool {
    arrasto(world).is_some_and(|a| a.armado)
}

fn arrasto(world: &World) -> Option<Arrasto> {
    world.get_resource::<FormaNoCanvas>()?.arrasto
}

/// Onde foi o clique, se o miolo nunca armou: é um clique, volta à seleção normal (17b A1).
#[must_use]
pub fn clique_parado(world: &World) -> Option<(f32, f32)> {
    arrasto(world)
        .filter(|a| a.alca == Alca::Miolo && !a.armado)
        .map(|a| a.ecra0)
}

/// O que a alça promete, para o cursor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorDaAlca {
    Mover,
    Horizontal,
    Vertical,
    /// `/` no ecrã.
    Sobe,
    /// `\\` no ecrã.
    Desce,
}

/// O cursor da alça agarrada ou sob o ponteiro, na direcção em que ela está NO ECRÃ (a forma pode estar rodada).
#[must_use]
pub fn cursor_em(
    world: &World,
    cam: &Camera2d,
    win: WindowSize,
    s: (f32, f32),
) -> Option<CursorDaAlca> {
    let alvo = editando(world)?;
    let e = Entity::from_bits(alvo);
    let (pose, m) = (Pose::de(world, e)?, Medida::de(world.get::<Collider>(e)?));
    let alca = match arrasto(world) {
        Some(a) => a.alca,
        None => alca_em(&pose, &m, cam, win, s)?,
    };
    if alca == Alca::Miolo {
        return Some(CursorDaAlca::Mover);
    }
    let (a, c) = (
        no_ecra(&pose, &m, alca, cam, win),
        no_ecra(&pose, &m, Alca::Miolo, cam, win),
    );
    // O ecrã desce em `y`: `\` é o ângulo positivo. Quatro setores de 45° em torno de cada eixo.
    let oitavo = ((a.1 - c.1).atan2(a.0 - c.0) / std::f32::consts::FRAC_PI_4).round();
    Some(match oitavo.rem_euclid(4.0) as u8 {
        0 => CursorDaAlca::Horizontal,
        1 => CursorDaAlca::Desce,
        2 => CursorDaAlca::Vertical,
        _ => CursorDaAlca::Sobe,
    })
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
        self.desce_com(s, Teclas::default())
    }

    /// O `Down` dos pontos do «Outline» antes das alças. Com pontos, as alças da caixa NÃO arrastam: um clique
    /// dentro dela é consumido (os pontos são donos do canvas). `true` = consumido.
    pub fn desce_com(&mut self, s: (f32, f32), teclas: Teclas) -> bool {
        let Some(alvo) = editando(self.world) else {
            return false;
        };
        let Some((pose, m)) = self.forma(alvo) else {
            return false;
        };
        let alca = alca_em(&pose, &m, &self.camera, self.window, s);
        if crate::pontos::do_alvo(self.world, alvo).is_some() {
            return self.desce_no_ponto(alvo, s, teclas) || alca.is_some();
        }
        let Some(alca) = alca else {
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

    /// Alt+clique tira; Shift+clique alterna na seleção; clique seleciona e agarra; clique na aresta acrescenta,
    /// seleciona e agarra o novo; no vazio a seleção limpa.
    fn desce_no_ponto(&mut self, alvo: u64, s: (f32, f32), teclas: Teclas) -> bool {
        use crate::pontos as p;
        let Some((pose, m, o)) = p::do_alvo(self.world, alvo) else {
            return false;
        };
        let ecra = p::no_ecra(&pose, &m, &o, &self.camera, self.window);
        let q = pose.do_mundo(self.camera.screen_to_world(s, self.window));
        let mut sel = self
            .world
            .get_resource::<p::PontosNoCanvas>()
            .map(|x| x.selecionados.clone())
            .unwrap_or_default();
        let mut arrasto = None;
        let consumido = if let Some(pt) = p::ponto_em(&ecra, s) {
            if teclas.alt {
                if let Some(v) = p::tirado(&o.rings, pt) {
                    p::escreve(self.world, alvo, v, true);
                    sel.clear();
                }
            } else if teclas.shift {
                match sel.binary_search(&pt) {
                    Ok(k) => {
                        sel.remove(k);
                    }
                    Err(k) => sel.insert(k, pt),
                }
            } else {
                if sel.binary_search(&pt).is_err() {
                    sel = vec![pt];
                }
                let at = p::locais(&pose, &m, &o)[pt.anel][pt.i];
                arrasto = Some(p::ArrastoDoPonto {
                    ponto: pt,
                    inicio: o.rings.clone(),
                    custom: o.custom,
                    folga: [q[0] - at[0], q[1] - at[1]],
                    ecra0: s,
                    ecra: s,
                    armado: false,
                });
            }
            true
        } else if let Some((pt, _)) = p::aresta_em(&ecra, s) {
            if let Some(v) = p::acrescentado(&o.rings, pt, p::para_q4(&pose, &m, &o, q)) {
                p::escreve(self.world, alvo, v, true);
                let novo = p::Ponto {
                    anel: pt.anel,
                    i: pt.i + 1,
                };
                sel = vec![novo];
                // O mesmo gesto arrasta o ponto novo; o `Esc` devolve o anel de antes (sem ele).
                arrasto = Some(p::ArrastoDoPonto {
                    ponto: novo,
                    inicio: o.rings.clone(),
                    custom: o.custom,
                    folga: [0.0, 0.0],
                    ecra0: s,
                    ecra: s,
                    armado: false,
                });
            }
            true
        } else {
            sel.clear();
            false
        };
        let mut x = self
            .world
            .get_resource_or_insert_with(p::PontosNoCanvas::default);
        x.selecionados = sel;
        x.arrasto = arrasto;
        consumido
    }

    /// O Delete: tira os pontos selecionados num passo (o que deixaria um anel abaixo de 3 fica). `true` = havia
    /// seleção (a tecla não chega a apagar o objecto).
    pub fn apaga(&mut self) -> bool {
        use crate::pontos as p;
        let Some(alvo) = editando(self.world) else {
            return false;
        };
        let sel = self
            .world
            .get_resource_mut::<p::PontosNoCanvas>()
            .map(|mut x| std::mem::take(&mut x.selecionados))
            .unwrap_or_default();
        if sel.is_empty() {
            return false;
        }
        if let Some((_, _, o)) = p::do_alvo(self.world, alvo)
            && let Some(v) = p::tirados(&o.rings, &sel)
        {
            p::escreve(self.world, alvo, v, true);
        }
        true
    }

    /// O arrasto de um ponto: a posição que cruzaria é recusada e o ponto fica na última válida.
    fn move_ponto(&mut self, alvo: u64, s: (f32, f32)) -> bool {
        use crate::pontos as p;
        let Some(mut a) = self
            .world
            .get_resource::<p::PontosNoCanvas>()
            .and_then(|x| x.arrasto.clone())
        else {
            return false;
        };
        if !a.armado && (s.0 - a.ecra0.0).hypot(s.1 - a.ecra0.1) < ARMA_PX {
            return true;
        }
        a.armado = true;
        a.ecra = s;
        publica_a_mao(self.world, Some(alvo));
        if let Some((pose, m, o)) = p::do_alvo(self.world, alvo) {
            let q = pose.do_mundo(self.camera.screen_to_world(s, self.window));
            let e = [q[0] - a.folga[0], q[1] - a.folga[1]];
            if let Some(v) = p::movido(&o.rings, a.ponto, p::para_q4(&pose, &m, &o, e)) {
                p::escreve(self.world, alvo, v, true);
            }
        }
        if let Some(mut x) = self.world.get_resource_mut::<p::PontosNoCanvas>() {
            x.arrasto = Some(a);
        }
        true
    }

    /// `true` = consumiu o movimento. Pose e vista são as de agora: o pai pode ter andado.
    pub fn move_(&mut self, s: (f32, f32), teclas: Teclas) -> bool {
        if editando(self.world).is_some() {
            self.world
                .insert_resource(crate::cursor_pontos::TeclasNoCanvas(teclas));
        }
        if let Some(alvo) = editando(self.world)
            && self.move_ponto(alvo, s)
        {
            return true;
        }
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
        publica_a_mao(self.world, Some(alvo));
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
        let solto = self
            .world
            .get_resource_mut::<crate::pontos::PontosNoCanvas>()
            .and_then(|mut p| p.arrasto.take());
        let ponto = solto.is_some();
        if let Some(a) = solto.filter(|a| a.armado) {
            self.funde(a);
        }
        let havia = ponto
            || self
                .world
                .get_resource_mut::<FormaNoCanvas>()
                .and_then(|mut f| f.arrasto.take())
                .is_some();
        if havia {
            publica_a_mao(self.world, None);
        }
        havia
    }

    /// Soltar sobre outro ponto do mesmo anel tira o arrastado (no mesmo passo do arrasto).
    fn funde(&mut self, a: crate::pontos::ArrastoDoPonto) {
        use crate::pontos as p;
        let Some(alvo) = editando(self.world) else {
            return;
        };
        let Some((pose, m, o)) = p::do_alvo(self.world, alvo) else {
            return;
        };
        let ecra = p::no_ecra(&pose, &m, &o, &self.camera, self.window);
        if p::alvo_da_fusao(&ecra, a.ponto, a.ecra).is_some()
            && let Some(v) = p::tirado(&o.rings, a.ponto)
        {
            p::escreve(self.world, alvo, v, true);
            if let Some(mut x) = self.world.get_resource_mut::<p::PontosNoCanvas>() {
                x.selecionados.clear();
            }
        }
    }

    /// O `Esc`: a meio do arrasto devolve a forma do `Down`; parado, encerra a edição. `true` = consumido.
    pub fn esc(&mut self) -> bool {
        let Some(f) = self.world.get_resource::<FormaNoCanvas>().copied() else {
            return false;
        };
        let Some(alvo) = f.alvo else {
            return false;
        };
        let ponto = self
            .world
            .get_resource_mut::<crate::pontos::PontosNoCanvas>()
            .and_then(|mut p| p.arrasto.take());
        if let Some(a) = ponto {
            if a.armado {
                crate::pontos::escreve(self.world, alvo, a.inicio.to_vec(), a.custom);
            }
            publica_a_mao(self.world, None);
        } else if let Some(a) = f.arrasto {
            if a.armado {
                self.escreve(alvo, &a.inicio);
            }
            if let Some(mut f) = self.world.get_resource_mut::<FormaNoCanvas>() {
                f.arrasto = None;
            }
            publica_a_mao(self.world, None);
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
