//! **O cartão CONSTRAINTS, lado da família** (plano §4): o snapshot que o painel pinta ([`info`]), as
//! edições que ele pede ([`apply_all`]) e o clique no palco que escolhe o alvo ([`escolher_alvo`]).

use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Resource;
use ph2d_constraints::{
    Action, Canais, ChildOf, CopyTransform, Distance, Floor, FollowPath, Ik, Kind, Limit, LookAt,
    Pivot, PontoDeForma, PontoDeSimulacao, Space, SplineIk, Spring, StretchTo, Transformation,
};
use ph2d_constraints_ecs::{Adotar, Constraints, Estado, Estados, bind_on_create, ordem};
use ph2d_ecs::{Name, SimWorld, Transform, entity_with_stable_id, stable_id_of, world_transform};
use ph2d_editor_core::constraint_edits::{
    ConstraintFieldEdit as E, EstadoDaConstraint, InspectorConstraintRow, InspectorConstraintsInfo,
    MAX_LINHAS, TIPO_BEHAVIOUR, TIPO_COLA_PINO, TIPO_COLA_RIGIDA, TIPO_PARTICULA, TIPOS,
};
use ph2d_preview_drive::PreviewDrive;

use crate::compensa::{Pega, Pegas};
use ph2d_ecs::canal_constraint::{FORCA, LIGADA};

/// O item cujo alvo o próximo clique no palco escolhe: `(dono, Item::id)`. Estado de sessão.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AlvoArmado(pub Option<(u64, u32)>);

/// O índice em [`TIPOS`] de um tipo — o inverso do [`novo`].
#[must_use]
pub fn tipo_de(k: &Kind) -> u8 {
    match k {
        Kind::ChildOf(_) => 0,
        Kind::CopyTransform(c) if c.canais == Canais::LOCATION => 1,
        Kind::CopyTransform(c) if c.canais == Canais::ROTATION => 2,
        Kind::CopyTransform(c) if c.canais == Canais::SCALE => 3,
        Kind::CopyTransform(_) => 4,
        Kind::LookAt(_) => 5,
        Kind::Limit(_) => 6,
        Kind::Distance(_) => 7,
        Kind::Ik(_) => 8,
        Kind::StretchTo(_) => 9,
        Kind::FollowPath(_) => 10,
        Kind::Pivot(_) => 11,
        Kind::Floor(_) => 12,
        Kind::Transformation(_) => 13,
        Kind::SplineIk(_) => 14,
        Kind::PontoDeForma(_) => 15,
        Kind::Action(_) => 16,
        Kind::Spring(_) => 17,
        Kind::Valor(_) => 20,
        Kind::PontoDeSimulacao(_) => TIPO_PARTICULA,
        Kind::Comportamento(c) => TIPO_BEHAVIOUR + c.perfil.tag(),
    }
}

/// O item que a entrada `i` do menu cria para o dono `e`. O Limit Rotation nasce centrado na
/// rotação actual, ±45° — ligar não move nada.
fn novo(sim: &SimWorld, e: Entity, i: u8) -> Option<Kind> {
    let copia = |canais| {
        Kind::CopyTransform(CopyTransform {
            canais,
            space: Space::World,
            ..CopyTransform::default()
        })
    };
    Some(match i {
        0 => Kind::ChildOf(ChildOf::default()),
        1 => copia(Canais::LOCATION),
        2 => copia(Canais::ROTATION),
        3 => copia(Canais::SCALE),
        4 => copia(Canais::ALL),
        5 => Kind::LookAt(LookAt::default()),
        6 => {
            let r = f64::from(
                world_transform(sim.world(), e)
                    .unwrap_or(Transform::IDENTITY)
                    .rotation,
            );
            let q = std::f64::consts::FRAC_PI_4;
            Kind::Limit(Limit {
                rot: Some([r - q, r + q]),
                ..Limit::default()
            })
        }
        7 => Kind::Distance(Distance::default()),
        // A IK nasce com o lado em que a corrente já dobra (como o «Add IK» do osso): ligar não vira o joelho.
        8 => Kind::Ik(Ik {
            bend: ph2d_skeleton_live::goal::side_for_chain(sim, e, Ik::default().chain),
            ..Ik::default()
        }),
        9 => Kind::StretchTo(StretchTo::default()),
        10 => Kind::FollowPath(FollowPath::default()),
        11 => Kind::Pivot(Pivot::default()),
        12 => Kind::Floor(Floor::default()),
        13 => Kind::Transformation(Transformation::default()),
        14 => Kind::SplineIk(SplineIk::default()),
        15 => Kind::PontoDeForma(PontoDeForma::default()),
        16 => Kind::Action(Action::default()),
        17 => Kind::Spring(Spring::default()),
        20 => Kind::Valor(ph2d_constraints::Valor::default()),
        TIPO_PARTICULA => Kind::PontoDeSimulacao(PontoDeSimulacao::default()),
        // O clique de um *behaviour*: os números de fábrica do perfil (o molde do CameraShake).
        i if i >= TIPO_BEHAVIOUR => Kind::Comportamento(
            ph2d_constraints::comportamento::Perfil::ALL
                .get(usize::from(i - TIPO_BEHAVIOUR))?
                .numeros(),
        ),
        _ => return None,
    })
}

fn estado(e: Estado) -> EstadoDaConstraint {
    match e {
        Estado::Activa => EstadoDaConstraint::Activa,
        Estado::Desligada => EstadoDaConstraint::Desligada,
        Estado::SemForca => EstadoDaConstraint::SemForca,
        Estado::SemAlvo => EstadoDaConstraint::SemAlvo,
        Estado::AlvoSiMesmo => EstadoDaConstraint::AlvoSiMesmo,
        Estado::Ciclo => EstadoDaConstraint::Ciclo,
        Estado::CorpoDinamico => EstadoDaConstraint::CorpoDinamico,
        Estado::EspacoColapsado => EstadoDaConstraint::EspacoColapsado,
        Estado::NaoEOsso => EstadoDaConstraint::NaoEOsso,
        Estado::SemClip => EstadoDaConstraint::SemClip,
        Estado::SemValor => EstadoDaConstraint::SemValor,
    }
}

/// ⭐ Publica o snapshot do cartão no painel do Inspector (a shell chama-a uma vez por quadro, com a
/// selecção): é a [`info`] da entidade escolhida, ou `None` sem selecção.
pub fn publica(sim: &SimWorld, escolhido: Option<u64>, selected_count: usize) {
    let pilha = escolhido.and_then(|b| info(sim, b, selected_count));
    ph2d_panel_inspector::set_current_inspector_constraints(pilha);
}

/// ⭐ O snapshot do cartão. `None` = a entidade não tem pilha nem cola (o cartão não aparece). A cola
/// aparece nos DOIS objectos que cola.
#[must_use]
pub fn info(
    sim: &SimWorld,
    entity_bits: u64,
    selected_count: usize,
) -> Option<InspectorConstraintsInfo> {
    let w = sim.world();
    let e = Entity::try_from_bits(entity_bits)?;
    let colas = crate::cola::linhas(w, e);
    let cola_armada = crate::cola::armada(w)
        .filter(|&(d, _)| d == entity_bits)
        .map(|(_, pino)| pino);
    let c = w.get::<Constraints>(e);
    if c.is_none() && colas.is_empty() && cola_armada.is_none() {
        return None;
    }
    let estados = w.get_resource::<Estados>();
    let armado = w.get_resource::<AlvoArmado>().and_then(|a| a.0);
    let rows = c
        .iter()
        .flat_map(|c| &c.items)
        .map(|it| {
            let alvo = it.target.and_then(|s| entity_with_stable_id(w, s));
            let (numeros, escolhas) = crate::campos::ajustes(&it.kind);
            #[expect(clippy::cast_possible_truncation, reason = "a força é 0..=1")]
            InspectorConstraintRow {
                id: it.id,
                tipo: tipo_de(&it.kind),
                alvo: alvo.map(|t| w.get::<Name>(t).map_or_else(String::new, |n| n.0.clone())),
                precisa_alvo: it.kind.needs_target(),
                forca: it.influence as f32,
                ligada: it.enabled,
                estado: estados
                    .and_then(|s| s.of(e, it.id))
                    .map_or(EstadoDaConstraint::Activa, estado),
                escolhendo: armado == Some((entity_bits, it.id)),
                numeros,
                escolhas,
                propriedades: crate::cartao_valor::propriedades(w, &it.kind, e, alvo),
                ordem: crate::cartao_valor::ordem(&it.kind),
                zera_desvio: crate::campos::zera_desvio(&it.kind),
                peso: crate::peso::linha(w, entity_bits, it),
            }
        })
        .collect();
    Some(InspectorConstraintsInfo {
        entity_bits,
        rows,
        selected_count,
        colas,
        cola_armada,
    })
}

pub(crate) fn armar(sim: &mut SimWorld, alvo: Option<(u64, u32)>) {
    if alvo.is_some() {
        crate::cola::armar(sim, None);
        sim.world_mut()
            .insert_resource(crate::peso::PesoArmado(None));
    }
    sim.world_mut().insert_resource(AlvoArmado(alvo));
}

/// Aplica as edições do cartão. `true` = o documento mudou (o undo grava um passo).
pub fn apply_all(sim: &mut SimWorld, edits: &[(u64, E)]) -> bool {
    let mut mudou = false;
    for &(bits, edit) in edits {
        let Some(e) = Entity::try_from_bits(bits) else {
            continue;
        };
        mudou |= aplicar(sim, e, edit);
    }
    mudou
}

fn aplicar(sim: &mut SimWorld, e: Entity, edit: E) -> bool {
    let bits = e.to_bits();
    if edit == E::Cozer {
        sim.world_mut()
            .get_resource_or_insert_with(crate::bake::Cozer::default)
            .0
            .push(bits);
        return false;
    }
    if let E::EscolherPeso(id) = edit {
        crate::peso::alternar(sim, bits, id);
        return false;
    }
    if let E::LargarPeso(id) = edit {
        return crate::peso::largar(sim, e, id);
    }
    if let E::EscolherAlvo(id) = edit {
        let ja = sim.world().get_resource::<AlvoArmado>().and_then(|a| a.0);
        armar(sim, (ja != Some((bits, id))).then_some((bits, id)));
        return false;
    }
    if let E::Adicionar(i @ (TIPO_COLA_RIGIDA | TIPO_COLA_PINO)) = edit {
        armar(sim, None);
        crate::cola::armar(sim, Some((bits, i == TIPO_COLA_PINO)));
        return false;
    }
    if let E::Descolar(dono, id) = edit {
        return crate::cola::descolar(sim, dono, id);
    }
    if let E::Adicionar(i) = edit {
        let cheio = sim
            .world()
            .get::<Constraints>(e)
            .is_some_and(|c| c.items.len() >= MAX_LINHAS);
        let Some(kind) = novo(sim, e, i).filter(|_| !cheio && usize::from(i) < TIPOS.len()) else {
            return false;
        };
        let mut kind = kind;
        // O Limit e o Look At não guardam desvio ao nascer; os outros esperam o alvo.
        let _ = bind_on_create(sim, &PreviewDrive::default(), e, &mut kind, None);
        let Some(id) = sim
            .world_mut()
            .get_mut::<Constraints>(e)
            .map(|mut c| c.push(kind, None))
        else {
            return false;
        };
        if kind.needs_target() {
            armar(sim, Some((bits, id)));
        }
        return true;
    }
    match edit {
        E::Forca(id, v) if v.is_finite() => {
            let v = f64::from(v.clamp(0.0, 1.0));
            return liga(sim, e, id, |it| it.influence = v);
        }
        E::Ligada(id, on) => return liga(sim, e, id, |it| it.enabled = on),
        E::Campo(id, campo, v) => {
            return muda(sim, e, id, |it| {
                crate::campos::escrever(&mut it.kind, campo, v)
            });
        }
        E::ZerarDesvio(id) => {
            return muda(sim, e, id, |it| crate::campos::zerar_desvio(&mut it.kind));
        }
        E::Propriedade(id, campo, p) => {
            return muda(sim, e, id, |it| {
                crate::cartao_valor::escrever(&mut it.kind, campo, p)
            });
        }
        _ => {}
    }
    let Some(mut c) = sim.world_mut().get_mut::<Constraints>(e) else {
        return false;
    };
    match edit {
        E::Apagar(id) => return c.remove(id),
        E::Mover(id, passo) => {
            let Some(de) = c.items.iter().position(|i| i.id == id) else {
                return false;
            };
            let Some(para) = de.checked_add_signed(isize::from(passo)) else {
                return false;
            };
            if para >= c.items.len() {
                return false;
            }
            c.move_to(id, para);
        }
        E::Forca(..)
        | E::Ligada(..)
        | E::EscolherAlvo(_)
        | E::Adicionar(_)
        | E::Campo(..)
        | E::Propriedade(..)
        | E::ZerarDesvio(_)
        | E::Descolar(..)
        | E::EscolherPeso(_)
        | E::LargarPeso(_)
        | E::Cozer => return false,
    }
    true
}

fn muda(
    sim: &mut SimWorld,
    e: Entity,
    id: u32,
    f: impl FnOnce(&mut ph2d_constraints_ecs::Item) -> bool,
) -> bool {
    sim.world_mut()
        .get_mut::<Constraints>(e)
        .and_then(|mut c| c.item_mut(id).map(f))
        .unwrap_or(false)
}

/// Muda a força ou o liga/desliga de um item SEM SALTO no gesto de pegar e largar: ao passar de
/// inerte a activo, o tipo que guarda a pose (Child Of, Limit Distance) re-acerta o desvio contra a
/// pose que se vê; ao largar um Child Of, a pose que se vê passa a ser a autorada ([`Adotar`]).
fn liga(
    sim: &mut SimWorld,
    e: Entity,
    id: u32,
    muda: impl FnOnce(&mut ph2d_constraints_ecs::Item),
) -> bool {
    let Some(antes) = sim
        .world()
        .get::<Constraints>(e)
        .and_then(|c| c.item(id))
        .copied()
    else {
        return false;
    };
    let mut depois = antes;
    muda(&mut depois);
    if depois == antes {
        return false;
    }
    let activo = |i: &ph2d_constraints_ecs::Item| i.enabled && i.influence > 0.0;
    if !activo(&antes) && activo(&depois) {
        let alvo = antes
            .target
            .and_then(|s| entity_with_stable_id(sim.world(), s));
        let _ = bind_on_create(sim, &PreviewDrive::default(), e, &mut depois.kind, alvo);
    }
    let larga_pose = activo(&antes) && !activo(&depois) && matches!(antes.kind, Kind::ChildOf(_));
    if larga_pose {
        sim.world_mut()
            .get_resource_or_insert_with(Adotar::default)
            .0
            .insert(e.to_bits());
    }
    if activo(&antes) != activo(&depois) {
        let campo = if antes.influence == depois.influence {
            LIGADA
        } else {
            FORCA
        };
        let ler = |it| crate::campos::ler_item(it, campo).unwrap_or_default();
        sim.world_mut()
            .get_resource_or_insert_with(Pegas::default)
            .0
            .push(Pega {
                dono: e.to_bits(),
                item: id,
                campo,
                antes: ler(&antes),
                depois: ler(&depois),
                larga_pose,
            });
    }
    if let Some(mut c) = sim.world_mut().get_mut::<Constraints>(e)
        && let Some(it) = c.item_mut(id)
    {
        *it = depois;
    }
    true
}

/// O resultado de um clique no palco com o alvo armado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Escolha {
    /// Nada armado: o clique não é deste cartão.
    Livre,
    /// O alvo gravou-se; o pick desarma.
    Gravado,
    /// O clique caiu no vazio: o pick desarma sem mudar nada.
    Desistiu,
    /// O candidato é o próprio dono ou fecharia um ciclo: recusado, o pick continua armado.
    Recusado,
}

/// `true` se o próximo clique no palco é deste cartão.
#[must_use]
pub fn armado(sim: &SimWorld) -> bool {
    sim.world()
        .get_resource::<AlvoArmado>()
        .is_some_and(|a| a.0.is_some())
        || crate::cola::armada(sim.world()).is_some()
        || crate::peso::armado(sim.world()).is_some()
}

/// ⭐ **O clique que escolhe o alvo.** `debaixo` = as entidades sob o cursor, a de cima primeiro (o
/// `pick_sprites_at_world` da shell); `mundo` = o ponto do clique (o do pino). O alvo grava-se pelo
/// `StableId`; o Child Of e o Limit Distance guardam a pose actual (ligar não salta). A cola armada
/// vem primeiro ([`crate::cola`]).
pub fn escolher_alvo(sim: &mut SimWorld, debaixo: &[u64], mundo: [f32; 2]) -> Escolha {
    if let Some(r) = crate::cola::escolher(sim, debaixo, mundo.map(f64::from)) {
        return r;
    }
    if let Some(r) = crate::peso::escolher(sim, debaixo) {
        return r;
    }
    let Some((dono_bits, id)) = sim.world().get_resource::<AlvoArmado>().and_then(|a| a.0) else {
        return Escolha::Livre;
    };
    let candidato = debaixo.iter().copied().find(|&b| b != dono_bits);
    let (Some(dono), Some(alvo)) = (Entity::try_from_bits(dono_bits), candidato) else {
        if debaixo.contains(&dono_bits) {
            return Escolha::Recusado;
        }
        armar(sim, None);
        return Escolha::Desistiu;
    };
    let Some(alvo) = Entity::try_from_bits(alvo) else {
        armar(sim, None);
        return Escolha::Desistiu;
    };
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let Some(sid) = stable_id_of(sim.world(), alvo) else {
        return Escolha::Recusado;
    };
    let Some(antes) = sim.world().get::<Constraints>(dono).cloned() else {
        armar(sim, None);
        return Escolha::Desistiu;
    };
    let Some(mut kind) = antes.item(id).map(|i| i.kind) else {
        armar(sim, None);
        return Escolha::Desistiu;
    };
    let _ = bind_on_create(sim, &PreviewDrive::default(), dono, &mut kind, Some(alvo));
    if let Some(mut c) = sim.world_mut().get_mut::<Constraints>(dono)
        && let Some(it) = c.item_mut(id)
    {
        it.target = Some(sid);
        it.kind = kind;
    }
    if ordem(sim.world()).1.contains(&dono) {
        sim.world_mut().entity_mut(dono).insert(antes);
        return Escolha::Recusado;
    }
    armar(sim, None);
    Escolha::Gravado
}
