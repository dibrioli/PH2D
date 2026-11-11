//! ⭐⭐ **A família FISICA nos canais** (plano 35 §A.5): [`publicar`] o que o mundo vale, [`aplicar`] o
//! que a curva mandou, pela porta do evento e o funil do painel; a escrita da curva é pré-visualização
//! ([`ConducaoFisica`]). Os pontos do «Outline» (51/52): `ph2d_physics_outline::canais_pontos`. O porquê:
//! [plano 35](../../../docs/UI_New_and_Simple/vetor_nova_ui/35_plano_F24_o_auto_key_grava_tudo.md) e
//! [f2 «Auto Key dos pontos»](../../../docs/GameEngineFuncional/componentes/f2_contorno_e_formas_plano.md).

use ph2d_ecs::canal::familia::FISICA;
use ph2d_ecs::canal_conducao::ConducaoDosCanais;
use ph2d_ecs::canal_fisica as f;
use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue, apply_editor_commands};
use ph2d_ecs::{CanaisEscritos, CanaisLidos, FieldKey, SimWorld};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::screens::hero::{InspectorPhysicsInfo, PhysicsFieldEdit};
use ph2d_panel_inspector::ids as insp;
use ph2d_physics_ecs::{Collider, ColliderOutline};
use ph2d_physics_outline::canais_pontos as op;
use std::collections::BTreeMap;

/// O controlo da §11 que mostra um campo.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Porta {
    /// Um número (o valor na unidade do painel).
    Numero(NodeId),
    /// Uma escolha: a opção `i` é o valor `i`.
    Escolha(&'static [NodeId]),
}

fn porta(campo: u16) -> Option<Porta> {
    use Porta::{Escolha, Numero};
    Some(match campo {
        f::TIPO => Escolha(&insp::INSP_PHYS_KIND),
        f::CCD => Escolha(&insp::INSP_PHYS_CCD),
        f::TRAVA_ROTACAO => Escolha(&insp::INSP_PHYS_LOCKROT),
        f::TRAVA_X => Escolha(&insp::INSP_PHYS_LOCKX),
        f::TRAVA_Y => Escolha(&insp::INSP_PHYS_LOCKY),
        f::ESCALA_GRAVIDADE => Numero(insp::INSP_PHYS_GRAVITY_SCALE),
        f::DOMINANCIA => Numero(insp::INSP_PHYS_DOMINANCE),
        f::VEL_X => Numero(insp::INSP_PHYS_LINVEL_X),
        f::VEL_Y => Numero(insp::INSP_PHYS_LINVEL_Y),
        f::VEL_ANGULAR => Numero(insp::INSP_PHYS_ANGVEL),
        f::AMORT_LINEAR => Numero(insp::INSP_PHYS_LINEAR_DAMPING),
        f::AMORT_ANGULAR => Numero(insp::INSP_PHYS_ANGULAR_DAMPING),
        f::AMORT_MODO => Escolha(&insp::INSP_PHYS_DAMPMODE),
        f::MASSA_MANUAL => Escolha(&insp::INSP_PHYS_MASSMODE),
        f::MASSA => Numero(insp::INSP_PHYS_MASS),
        f::FORMA => Escolha(&insp::INSP_PHYS_SHAPE),
        f::RAIO => Numero(insp::INSP_PHYS_RADIUS),
        f::MEIO_X => Numero(insp::INSP_PHYS_HALF_X),
        f::MEIO_Y => Numero(insp::INSP_PHYS_HALF_Y),
        f::CAPSULA => Numero(insp::INSP_PHYS_CAP_HALF_H),
        f::DESLOC_X => Numero(insp::INSP_PHYS_OFFSET_X),
        f::DESLOC_Y => Numero(insp::INSP_PHYS_OFFSET_Y),
        f::DENSIDADE => Numero(insp::INSP_PHYS_DENSITY),
        f::RESTITUICAO => Numero(insp::INSP_PHYS_RESTITUTION),
        f::ATRITO => Numero(insp::INSP_PHYS_FRICTION),
        f::COMBINA_RESTITUICAO => Escolha(&insp::INSP_PHYS_REST_COMBINE),
        f::COMBINA_ATRITO => Escolha(&insp::INSP_PHYS_FRIC_COMBINE),
        f::CAMADA => Escolha(&insp::INSP_PHYS_LAYER),
        f::SENSOR => Escolha(&insp::INSP_PHYS_SENSOR),
        f::UMA_VIA => Escolha(&insp::INSP_PHYS_ONEWAY),
        f::SEM_PAREDE => Escolha(&insp::INSP_PHYS_WALLMAT),
        f::ADERENCIA => Numero(insp::INSP_PHYS_WALK_GRIP),
        f::ESTEIRA => Numero(insp::INSP_PHYS_WALK_BELT),
        f::FORCA_X => Numero(insp::INSP_PHYS_FORCE_X),
        f::FORCA_Y => Numero(insp::INSP_PHYS_FORCE_Y),
        f::FORCA_NO_MUNDO => Escolha(&insp::INSP_PHYS_FORCE_AXES),
        f::ZONA_ARRASTO => Numero(insp::INSP_PHYS_AREA_DRAG),
        f::ZONA_DENSIDADE => Numero(insp::INSP_PHYS_AREA_DENSITY),
        f::ZONA_ARRASTO_FORMA => Numero(insp::INSP_PHYS_AREA_FORM_DRAG),
        f::ZONA_TORQUE => Numero(insp::INSP_PHYS_AREA_TORQUE),
        f::ZONA_QUEDA => Numero(insp::INSP_PHYS_AREA_FALLOFF),
        f::CONTORNO_DETALHE => Numero(insp::INSP_PHYS_OUTLINE_DETAIL),
        f::CONTORNO_ALFA => Numero(insp::INSP_PHYS_OUTLINE_ALPHA),
        f::CONTORNO_MANCHAS => Numero(insp::INSP_PHYS_OUTLINE_SPECKS),
        _ => return None,
    })
}

const fn k(campo: u16) -> FieldKey {
    FieldKey::new(FISICA, campo, 0, 0)
}

fn bit(b: bool) -> f32 {
    if b { 1.0 } else { 0.0 }
}

/// Quanto vale `campo` no retrato da §11 — `None` fora do vocabulário.
#[must_use]
pub fn ler(info: &InspectorPhysicsInfo, campo: u16) -> Option<f32> {
    Some(match campo {
        f::TIPO => f32::from(info.kind_tag),
        f::CCD => bit(info.ccd),
        f::TRAVA_ROTACAO => bit(info.lock_rotation),
        f::TRAVA_X => bit(info.lock_x),
        f::TRAVA_Y => bit(info.lock_y),
        f::ESCALA_GRAVIDADE => info.gravity_scale,
        f::DOMINANCIA => f32::from(info.dominance),
        f::VEL_X => info.linvel[0],
        f::VEL_Y => info.linvel[1],
        f::VEL_ANGULAR => info.angvel.to_degrees(),
        f::AMORT_LINEAR => info.linear_damping,
        f::AMORT_ANGULAR => info.angular_damping,
        f::AMORT_MODO => f32::from(info.damp_mode_tag),
        f::MASSA_MANUAL => bit(info.mass_manual),
        f::MASSA => info.mass,
        f::FORMA => f32::from(info.shape_tag),
        f::RAIO => info.radius,
        f::MEIO_X => info.half_x * 2.0,
        f::MEIO_Y => info.half_y * 2.0,
        f::CAPSULA => (info.cap_half_height + info.radius) * 2.0,
        f::DESLOC_X => info.offset[0],
        f::DESLOC_Y => info.offset[1],
        f::DENSIDADE => info.density,
        f::RESTITUICAO => info.restitution,
        f::ATRITO => info.friction,
        f::COMBINA_RESTITUICAO => f32::from(info.restitution_combine_tag),
        f::COMBINA_ATRITO => f32::from(info.friction_combine_tag),
        f::CAMADA => f32::from(info.layer),
        f::SENSOR => bit(info.is_sensor),
        f::UMA_VIA => bit(info.one_way),
        f::SEM_PAREDE => bit(info.no_wall_cling),
        f::ADERENCIA => info.walk_grip,
        f::ESTEIRA => info.walk_belt,
        f::FORCA_X => info.force[0],
        f::FORCA_Y => info.force[1],
        f::FORCA_NO_MUNDO => bit(info.force_world_axes),
        f::ZONA_ARRASTO => info.area_drag,
        f::ZONA_DENSIDADE => info.area_density,
        f::ZONA_ARRASTO_FORMA => info.area_form_drag,
        f::ZONA_TORQUE => info.area_torque,
        f::ZONA_QUEDA => info.area_falloff,
        f::CONTORNO_DETALHE => info.outline.unwrap_or_default().detail_px,
        f::CONTORNO_ALFA => info.outline.unwrap_or_default().alpha_cutoff,
        f::CONTORNO_MANCHAS => info.outline.unwrap_or_default().specks_px,
        _ => return None,
    })
}

/// O índice de uma escolha de `n` opções (`n ≤ 64`, a maior lista da §11): arredonda e prende.
fn opcao(v: f32, n: usize) -> u8 {
    let max = u8::try_from(n.saturating_sub(1)).unwrap_or(u8::MAX);
    if v.is_finite() {
        v.round().clamp(0.0, f32::from(max)) as u8
    } else {
        0
    }
}

/// ⭐ **A edição da §11 que escreve `v` em `campo`** — o controlo do painel e a porta única do
/// evento dele (as guardas do painel). `None` quando o painel não aceitaria.
#[must_use]
pub fn edicao(info: &InspectorPhysicsInfo, campo: u16, v: f32) -> Option<PhysicsFieldEdit> {
    match porta(campo)? {
        // O canal guarda o número do cartão (graus/s, Width, Height, a altura inteira da cápsula).
        Porta::Numero(id) => ph2d_panel_inspector::physics_edit_for(id, Some(v), info),
        Porta::Escolha(ops) => {
            let id = *ops.get(usize::from(opcao(v, ops.len())))?;
            ph2d_panel_inspector::physics_edit_for(id, None, info)
        }
    }
}

/// `true` se a §11 mostra `campo` para este objecto agora — a face do corpo, ou a da PEÇA (um
/// colisor sem corpo: forma, medidas, deslocamento, densidade); e a densidade OU a massa, pela
/// fonte da massa. O resto é a guarda do próprio evento.
#[must_use]
pub fn mostrado(info: &InspectorPhysicsInfo, campo: u16) -> bool {
    if !info.has_collider {
        return false;
    }
    let da_peca = matches!(
        campo,
        f::FORMA
            | f::RAIO
            | f::MEIO_X
            | f::MEIO_Y
            | f::CAPSULA
            | f::DESLOC_X
            | f::DESLOC_Y
            | f::DENSIDADE
    );
    if !info.has_body && !da_peca {
        return false;
    }
    let massa_manual = info.mass_is_read && info.mass_manual;
    match campo {
        f::DENSIDADE if massa_manual => false,
        f::MASSA => massa_manual,
        _ => ler(info, campo).is_some_and(|v| edicao(info, campo, v).is_some()),
    }
}

/// O retrato da §11 de `bits` — só os números (os factos que só a shell vê ficam neutros).
#[must_use]
pub fn retrato(sim: &SimWorld, bits: u64) -> Option<InspectorPhysicsInfo> {
    crate::inspector::body::build_physics_info(
        sim.world(),
        bits,
        0,
        0,
        0,
        false,
        0,
        (0.0, 0.0),
        0,
        &ph2d_tags::TagTree::default(),
    )
}

/// O valor que o documento guardaria para `v` (o índice de uma escolha).
fn normal(campo: u16, v: f32) -> f32 {
    match porta(campo) {
        Some(Porta::Escolha(ops)) => f32::from(opcao(v, ops.len())),
        _ => v,
    }
}

/// Escreve `v` em `campo` pelos dois funis do painel; devolve `(antes, ficou)`.
fn escrever(
    sim: &mut SimWorld,
    registry: &ComponentRegistry,
    bits: u64,
    campo: u16,
    v: f32,
) -> Option<(f32, f32)> {
    // A forma principal fala pelo corpo; o funil leva o campo de forma à filha (opção B).
    let bits = ph2d_physics_ecs::corpo_da_forma(sim.world(), bits);
    let info = retrato(sim, bits)?;
    let antes = ler(&info, campo)?;
    if !mostrado(&info, campo) || antes == normal(campo, v) {
        return Some((antes, antes));
    }
    let Some(ed) = edicao(&info, campo, v) else {
        return Some((antes, antes));
    };
    let fila = EditorCommandQueue::new();
    crate::physics_apply::apply_physics_edit(sim, bits, ed, &fila, registry);
    // Uma recusa do registo deixa o componente como estava — o eco diz o que ficou.
    let _ = apply_editor_commands(sim.world_mut(), &fila, registry);
    let ficou = retrato(sim, bits).and_then(|i| ler(&i, campo))?;
    Some((antes, ficou))
}

/// Quem guarda `campo` de `b` (opção B): o de forma de um corpo é a forma principal.
fn dono(sim: &SimWorld, b: u64, campo: u16) -> u64 {
    ph2d_physics_ecs::dono_do_campo(sim.world(), b, campo)
}

/// ⭐ **A trilha que uma chave de `prop` em `e` grava** — a do campo de forma de um corpo é a da
/// forma principal (opção B); qualquer outra, a de `e`.
#[must_use]
pub fn dono_da_chave(world: &ph2d_ecs::World, e: u64, prop: ph2d_timeline::PropKind) -> u64 {
    match prop {
        ph2d_timeline::PropKind::Field(kk) if kk.familia == FISICA => {
            ph2d_physics_ecs::dono_do_campo(world, e, kk.campo)
        }
        _ => e,
    }
}

/// O rótulo de cada camada de colisão, o do campo Layer, para a palavra do canal `CAMADA` — só quando muda.
fn publicar_nomes_das_camadas(sim: &mut SimWorld) {
    use ph2d_ecs::canal_palavra::NomesDasCamadasDeColisao as Nomes;
    let tabela = ph2d_physics_ecs::LayerNames::of(sim.world());
    let nomes = Nomes(
        (0..ph2d_physics_ecs::MAX_LAYERS)
            .map(|n| ph2d_panel_physics::aba::nome_inteiro(&tabela, n))
            .collect(),
    );
    if sim.world().get_resource::<Nomes>() != Some(&nomes) {
        sim.world_mut().insert_resource(nomes);
    }
}

/// ⭐ **Publica o que cada corpo vale AGORA** (através do eco) — os campos que a §11 mostra —,
/// esquecido antes o que cada entidade publicou desta família.
pub fn publicar(sim: &mut SimWorld, entidades: &[u64]) {
    publicar_nomes_das_camadas(sim);
    let mut lotes: Vec<(u64, FieldKey, f32)> = entidades
        .iter()
        .filter_map(|&b| Some((b, retrato(sim, b)?)))
        .flat_map(|(b, info)| {
            f::CAMPOS
                .iter()
                .filter(|&&c| mostrado(&info, c))
                .filter_map(|&c| Some((dono(sim, b, c), k(c), ler(&info, c)?)))
                .collect::<Vec<_>>()
        })
        .collect();
    for &b in entidades {
        let forma = dono(sim, b, f::CONTORNO_PONTO_X);
        let pontos = contorno(sim, forma).map(|o| op::publicados(&o.1));
        lotes.extend(pontos.into_iter().flatten().map(|(kk, v)| (forma, kk, v)));
    }
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for &(b, _, _) in &lotes {
        lidos.esquecer(b, FISICA);
    }
    for &b in entidades {
        lidos.esquecer(b, FISICA);
    }
    for (b, kk, v) in lotes {
        lidos.publicar(b, kk, v);
    }
}

/// ⭐ **Aplica aos corpos o que a curva mandou neste quadro**, regista o eco e o autorado.
/// Devolve `true` se algum corpo mudou.
pub fn aplicar(
    sim: &mut SimWorld,
    registry: &ComponentRegistry,
    conducao: &mut ConducaoFisica,
    doc: &ph2d_timeline::TimelineDoc,
) -> bool {
    let Some(mut escritas) = sim
        .world_mut()
        .get_resource_mut::<CanaisEscritos>()
        .map(|mut w| w.drenar(FISICA))
    else {
        return false;
    };
    // As escolhas antes dos números: a forma decide que medidas existem, o tipo e o sensor
    // decidem que campos a §11 aceita, a fonte da massa decide se a massa é lida.
    escritas.sort_by_key(|&(e, kk, _)| (e, kk.info().is_none_or(|i| !i.discreto()), kk));
    let mut mudou = false;
    let mut ecos = Vec::with_capacity(escritas.len());
    let (pontos, escritas): (Vec<_>, Vec<_>) =
        (escritas.into_iter()).partition(|&(_, kk, _)| f::e_do_ponto(kk.campo));
    for lote in pontos.chunk_by(|a, b| a.0 == b.0) {
        mudou |= aplicar_pontos(sim, conducao, doc, lote, &mut ecos);
    }
    for (e, kk, v) in escritas {
        if kk.i != 0 || kk.j != 0 {
            continue;
        }
        let foto = do_contorno(kk.campo).then(|| contorno(sim, e)).flatten();
        if let Some((antes, ficou)) = escrever(sim, registry, e, kk.campo, v) {
            mudou |= conducao.escreveu(e, kk, antes, ficou);
            if let Some(foto) = foto
                && conducao.campos().any(|c| c == (e, kk))
            {
                conducao.1.entry(e).or_insert(foto);
            }
            ecos.push((e, kk, v, ficou));
        }
    }
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for (e, kk, v, ficou) in ecos {
        lidos.eco(e, kk, v, ficou);
    }
    mudou
}

/// ⭐ **O que a curva manda num TIQUE chega ao corpo nesse tique** (GEF02 onda 8 item 3): o passo
/// por tique (`TimelineScene`: Play com vários tiques por quadro e o replay do scrub) aplica-o pelo
/// mesmo funil. Medido antes: Height 1 → 2 → 1 → 2, 3 tiques por quadro contra 1 = 4,3 cm de desvio
/// do caixote. O livro do tique é descartável; o que ele tirou à curva passa ao `livro` do quadro
/// (`conduziu`: o primeiro valor da corrida fica). O registo é o dos componentes que a §11 escreve.
pub fn aplicar_do_tique(
    sim: &mut SimWorld,
    doc: &ph2d_timeline::TimelineDoc,
    t: f64,
    livro: Option<&mut ConducaoFisica>,
) {
    static REGISTO: std::sync::OnceLock<ComponentRegistry> = std::sync::OnceLock::new();
    let reg = REGISTO.get_or_init(|| {
        let mut r = ComponentRegistry::new();
        ph2d_ecs::scene::register_ecs_components(&mut r);
        ph2d_physics_ecs::register_physics_components(&mut r);
        r
    });
    let mut descartavel = ConducaoFisica::default();
    descartavel.no_tempo(t);
    let _ = aplicar(sim, reg, &mut descartavel, doc);
    if let Some(livro) = livro {
        for (e, kk, antes) in descartavel.autorado() {
            livro.conduziu(e, kk, antes);
        }
        for (e, foto) in descartavel.1 {
            livro.1.entry(e).or_insert(foto);
        }
    }
}

/// Os pontos (51/52) que a curva mandou a UMA forma; a fotografia autorada e o eco.
fn aplicar_pontos(
    sim: &mut SimWorld,
    conducao: &mut ConducaoFisica,
    doc: &ph2d_timeline::TimelineDoc,
    lote: &[(u64, FieldKey, f32)],
    ecos: &mut Vec<(u64, FieldKey, f32, f32)>,
) -> bool {
    let (e, t, mut mudou) = (lote[0].0, conducao.2, false);
    let foto = contorno(sim, e);
    let anterior = |kk| op::na_chave_anterior(doc, e, kk, t);
    for (kk, v, antes, ficou) in op::aplicar(sim.world_mut(), e, lote, anterior) {
        mudou |= conducao.escreveu(e, kk, antes, ficou);
        ecos.push((e, kk, v, ficou));
    }
    if let Some(foto) = foto
        && conducao
            .campos()
            .any(|(c, kk)| c == e && f::e_do_ponto(kk.campo))
    {
        conducao.1.entry(e).or_insert(foto);
    }
    mudou
}

/// **A condução dos corpos pela curva** — guarda o valor AUTORADO de cada campo que a curva está a
/// mover, para que o undo e o save o fotografem em vez do instante da reprodução. O livro e a lei
/// de esquecer são os de [`ConducaoDosCanais`].
/// O traço do «Outline» é DERIVADO do pedido: a fotografia vê também o par AUTORADO
/// (`Collider`, `ColliderOutline`) da forma cujo pedido a curva move.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ConducaoFisica(ConducaoDosCanais, BTreeMap<u64, Contorno>, f64);

type Contorno = (Collider, ColliderOutline);

/// O vivo que [`ConducaoFisica::substituir`] tirou.
pub struct VivoFisica(Vec<(u64, FieldKey, f32)>, Vec<(u64, Contorno)>);

const fn do_contorno(campo: u16) -> bool {
    matches!(campo, f::CONTORNO_DETALHE..=f::CONTORNO_PONTO_Y)
}

fn contorno(sim: &SimWorld, bits: u64) -> Option<Contorno> {
    let w = sim
        .world()
        .get_entity(ph2d_ecs::Entity::try_from_bits(bits)?)
        .ok()?;
    Some((*w.get::<Collider>()?, w.get::<ColliderOutline>()?.clone()))
}

fn pousar(sim: &mut SimWorld, bits: u64, (c, o): Contorno) {
    if let Some(e) = ph2d_ecs::Entity::try_from_bits(bits)
        && let Ok(mut w) = sim.world_mut().get_entity_mut(e)
    {
        w.insert((c, o));
    }
}

impl std::ops::Deref for ConducaoFisica {
    type Target = ConducaoDosCanais;
    fn deref(&self) -> &ConducaoDosCanais {
        &self.0
    }
}

impl std::ops::DerefMut for ConducaoFisica {
    fn deref_mut(&mut self) -> &mut ConducaoDosCanais {
        &mut self.0
    }
}

impl ConducaoFisica {
    /// O instante do clip activo neste quadro (a guarda dos pontos amostra a chave anterior nele).
    pub fn no_tempo(&mut self, t: f64) {
        self.2 = t;
    }

    /// Esquece quem não foi conduzido neste quadro, e o contorno de quem deixou de o ser.
    pub fn settle(&mut self) {
        self.0.settle();
        let vivos: std::collections::BTreeSet<u64> = self
            .0
            .campos()
            .filter_map(|(e, kk)| do_contorno(kk.campo).then_some(e))
            .collect();
        self.1.retain(|e, _| vivos.contains(e));
    }

    /// Repõe o AUTORADO para a fotografia; devolve o vivo para [`Self::repor`]. O contorno
    /// autorado primeiro; as escolhas antes dos números, como no [`aplicar`].
    pub fn substituir(&self, sim: &mut SimWorld, registry: &ComponentRegistry) -> VivoFisica {
        let mut tracos = Vec::with_capacity(self.1.len());
        for (&e, autorado) in &self.1 {
            if let Some(v) = contorno(sim, e) {
                tracos.push((e, v));
                pousar(sim, e, autorado.clone());
            }
        }
        let mut vivo = Vec::with_capacity(self.len());
        for (e, kk, v) in ordenar(self.autorado()) {
            if let Some((antes, _)) = escrever(sim, registry, e, kk.campo, v) {
                vivo.push((e, kk, antes));
            }
        }
        VivoFisica(vivo, tracos)
    }

    /// Devolve o vivo que [`Self::substituir`] tirou.
    pub fn repor(
        &self,
        sim: &mut SimWorld,
        registry: &ComponentRegistry,
        VivoFisica(vivo, tracos): VivoFisica,
    ) {
        for (e, kk, v) in ordenar(vivo.into_iter()) {
            let _ = escrever(sim, registry, e, kk.campo, v);
        }
        for (e, v) in tracos {
            pousar(sim, e, v);
        }
    }
}

fn ordenar(it: impl Iterator<Item = (u64, FieldKey, f32)>) -> Vec<(u64, FieldKey, f32)> {
    let mut v: Vec<_> = it.collect();
    v.sort_by_key(|&(e, kk, _)| (e, kk.info().is_none_or(|i| !i.discreto()), kk));
    v
}

#[cfg(test)]
#[path = "canais_corpo_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "canais_por_tique_tests.rs"]
mod por_tique_tests;

#[cfg(test)]
#[path = "canais_por_tique_jm_tests.rs"]
mod por_tique_jm_tests;

#[cfg(test)]
#[path = "canais_donos_tests.rs"]
mod donos;

#[cfg(test)]
#[path = "canais_contorno_tests.rs"]
mod contorno;
