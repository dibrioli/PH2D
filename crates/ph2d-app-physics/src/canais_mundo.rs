//! ⭐⭐ **A família MUNDO nos canais** (F30 §B) — a gravidade e os outros campos de `PhysicsSettings`
//! que o painel *Physics* edita, sobre a entidade da Cena (`ph2d_ecs::cena`): [`publicar`] o
//! AUTORADO em `CanaisLidos`, [`aplicar`] o que a curva mandou em `CanaisEscritos`.
//!
//! ⭐ **A curva é pré-visualização e o autorado nunca muda por ela**: o autorado é o
//! `PhysicsBridge::settings` (o que o save grava, `project_save.rs`, e o painel edita); a curva vai
//! para o mundo VIVO por `PhysicsBridge::conduzir_settings`, que ⛔ não limpa o anel do scrub (o
//! `set_settings` limpa). Sem escrita da curva neste quadro, o mundo corre o autorado.
//!
//! ⭐ **O eco** diz ao Auto Key «o documento vale a curva» enquanto o autorado não muda; o quadro em
//! que a MÃO muda o autorado ([`ConducaoMundo`]) mostra a mão — no mundo e em `CanaisLidos` — e o
//! Auto Key grava-a.

use std::collections::BTreeMap;

use ph2d_ecs::canal::familia::MUNDO;
use ph2d_ecs::canal_mundo as m;
use ph2d_ecs::{CanaisEscritos, CanaisLidos, FieldKey, SimWorld};
use ph2d_physics_ecs::{LayerMatrix, MAX_LAYERS, PhysicsBridge, PhysicsSettings};

const fn k(campo: u16) -> FieldKey {
    FieldKey::new(MUNDO, campo, 0, 0)
}

/// A célula `i` da matriz de camadas (`m::CAMADAS`).
#[must_use]
pub const fn kc(i: u16) -> FieldKey {
    FieldKey::new(MUNDO, m::CAMADAS, i, 0)
}

/// ⭐ **As células da matriz**: `(i do canal, camada a, camada b)`, `i` = o índice da célula no
/// painel (o triângulo de baixo com a diagonal).
pub fn celulas() -> impl Iterator<Item = (u16, usize, usize)> {
    (0..MAX_LAYERS).flat_map(|a| {
        (0..=a).map(move |b| {
            let i = ph2d_panel_physics::ids::physics_layer_cell_index(a, b);
            (u16::try_from(i).expect("36 células"), a, b)
        })
    })
}

/// Quanto vale `campo` em `s` — `None` fora do vocabulário.
#[must_use]
#[expect(clippy::cast_precision_loss, reason = "sub-passos e iterações ≤ 16")]
pub fn ler(s: &PhysicsSettings, campo: u16) -> Option<f32> {
    Some(match campo {
        m::GRAVIDADE_X => s.gravity_x,
        m::GRAVIDADE_Y => s.gravity_y,
        m::SUBPASSOS => s.substeps as f32,
        m::ITERACOES => s.solver_iterations as f32,
        m::CONTACTO_HZ => s.contact_hz,
        m::AMORT_LINEAR => s.linear_damping,
        m::AMORT_ANGULAR => s.angular_damping,
        m::AR => s.air_drag,
        m::SONO_VELOCIDADE => s.sleep_linear_threshold,
        m::SONO_DEMORA => s.time_until_sleep,
        m::ADORMECE => f32::from(u8::from(s.sleep_enabled())),
        _ => return None,
    })
}

/// Escreve `v` em `campo` de `s` (o `clamped` da ponte prende as faixas).
pub fn escrever(s: &mut PhysicsSettings, campo: u16, v: f32) {
    let inteiro = || {
        if v.is_finite() {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "≥ 1"
            )]
            let n = v.round().max(1.0) as u32;
            n
        } else {
            1
        }
    };
    match campo {
        m::GRAVIDADE_X => s.gravity_x = v,
        m::GRAVIDADE_Y => s.gravity_y = v,
        m::SUBPASSOS => s.substeps = inteiro(),
        m::ITERACOES => s.solver_iterations = inteiro(),
        m::CONTACTO_HZ => s.contact_hz = v,
        m::AMORT_LINEAR => s.linear_damping = v,
        m::AMORT_ANGULAR => s.angular_damping = v,
        m::AR => s.air_drag = v,
        m::SONO_VELOCIDADE => s.sleep_linear_threshold = v,
        m::SONO_DEMORA => s.time_until_sleep = v,
        m::ADORMECE => *s = s.with_sleep_enabled(v >= 0.5),
        _ => {}
    }
}

fn cena(sim: &SimWorld) -> Option<u64> {
    ph2d_ecs::bits_da_cena(sim.world())
}

/// ⭐ **Publica o AUTORADO de cada campo do mundo** (através do eco) na entidade da Cena.
pub fn publicar(sim: &mut SimWorld, ponte: &PhysicsBridge) {
    let Some(c) = cena(sim) else { return };
    let s = ponte.settings();
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    lidos.esquecer(c, MUNDO);
    for campo in m::CAMPOS {
        if let Some(v) = ler(&s, campo) {
            lidos.publicar(c, k(campo), v);
        }
    }
    let mx = LayerMatrix::from_rows(s.layer_matrix);
    for (i, a, b) in celulas() {
        lidos.publicar(c, kc(i), f32::from(u8::from(mx.collides(a, b))));
    }
}

/// **O autorado que este funil viu no quadro anterior**, por campo — a diferença é a MÃO (o painel
/// mudou o autorado entre dois quadros), e nesse quadro a mão ganha à curva.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ConducaoMundo {
    /// Por `(campo, i)`: `i` só não é `0` nas células da matriz.
    visto: BTreeMap<(u16, u16), f32>,
    /// Os `(campo, i)` que o painel mexeu no que MOSTRAVA ([`Self::tocar`]): a mão ganha neles
    /// mesmo quando o autorado já valia o que se pediu (a curva escondia-o).
    tocadas: std::collections::BTreeSet<(u16, u16)>,
}

impl ConducaoMundo {
    /// ⭐ **O painel mostrava `visto` e pediu `pedido`** — todo campo e célula em que diferem é a
    /// mão (ver `panel_bridge::settings_do_painel`).
    pub fn tocar(&mut self, visto: &PhysicsSettings, pedido: &PhysicsSettings) {
        let campos = m::CAMPOS
            .into_iter()
            .filter(|&campo| ler(visto, campo) != ler(pedido, campo))
            .map(|campo| (campo, 0));
        let (v, n) = (
            LayerMatrix::from_rows(visto.layer_matrix),
            LayerMatrix::from_rows(pedido.layer_matrix),
        );
        let celulas = celulas()
            .filter(|&(_, a, b)| v.collides(a, b) != n.collides(a, b))
            .map(|(i, _, _)| (m::CAMADAS, i));
        self.tocadas.extend(campos.chain(celulas));
    }
}

/// ⭐ **Aplica ao mundo vivo o que a curva mandou neste quadro** (sem limpar o anel) e regista o eco.
/// Devolve `true` se o mundo mudou.
pub fn aplicar(sim: &mut SimWorld, ponte: &mut PhysicsBridge, cond: &mut ConducaoMundo) -> bool {
    let escritas = sim
        .world_mut()
        .get_resource_mut::<CanaisEscritos>()
        .map(|mut w| w.drenar(MUNDO))
        .unwrap_or_default();
    let Some(c) = cena(sim) else { return false };
    let curva: BTreeMap<(u16, u16), f32> = escritas
        .into_iter()
        .filter(|&(e, kk, _)| e == c && kk.j == 0 && (kk.i == 0 || kk.campo == m::CAMADAS))
        .map(|(_, kk, v)| ((kk.campo, kk.i), v))
        .collect();
    let autorado = ponte.settings();
    let mut alvo = autorado;
    let mut ecos = Vec::with_capacity(m::CAMPOS.len() + usize::from(m::CELULAS));
    // A mão ganha à curva no quadro em que muda o autorado; senão a curva manda (e o eco di-lo).
    let mut conduz = |chave: FieldKey, a: f32| -> Option<f32> {
        let tocada = cond.tocadas.remove(&(chave.campo, chave.i));
        let mao = (cond.visto.insert((chave.campo, chave.i), a)).is_some_and(|antes| antes != a)
            || tocada;
        let v = curva.get(&(chave.campo, chave.i)).copied().filter(|_| !mao);
        ecos.push((chave, v.unwrap_or(a), a));
        v
    };
    for campo in m::CAMPOS {
        if let Some(a) = ler(&autorado, campo)
            && let Some(v) = conduz(k(campo), a)
        {
            escrever(&mut alvo, campo, v);
        }
    }
    let feito = LayerMatrix::from_rows(autorado.layer_matrix);
    let mut mx = feito;
    for (i, a, b) in celulas() {
        if let Some(v) = conduz(kc(i), f32::from(u8::from(feito.collides(a, b)))) {
            mx.set(a, b, v >= 0.5);
        }
    }
    alvo.layer_matrix = mx.rows();
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for (kk, escrito, ficou) in ecos {
        lidos.eco(c, kk, escrito, ficou);
    }
    ponte.conduzir_settings(alvo)
}

#[cfg(test)]
#[path = "canais_mundo_tests.rs"]
mod tests;
