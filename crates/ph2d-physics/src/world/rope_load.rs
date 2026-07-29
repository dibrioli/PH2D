//! **O LIVRO-RAZÃO DA CORDA** (W-Pulley W2) — o que o passe PUBLICA, além dos
//! impulsos: a carga que cada corda e cada eixo seguraram, e o que cedeu.
//!
//! Irmão de `pulley.rs` pelo cap de LOC, e o corte é por responsabilidade: lá
//! mora o que a corda FAZ com os corpos, aqui o que ela CONTA sobre si mesma.
//! Módulo FILHO de `world`, então os campos privados do mundo seguem alcançáveis.
//!
//! ⚠️ **Uma conta, dois consumidores.** A resultante no eixo
//! (`T·|u_saída − u_entrada|`) é a MESMA expressão do Jacobiano da rota
//! (`∂L/∂centro = −(u_entra + u_sai)`, o cabeçalho da [`super::rope_route`]) —
//! e é por isso que o W3 (a talha de verdade, com a roldana montada num corpo
//! que se move) sai barato: ele precisa exatamente deste número.

use std::collections::{BTreeMap, BTreeSet};

use super::rope_route::{RopeWheel, Tangent};

/// **Uma corda ou um eixo que cedeu** durante o último [`super::PhysicsWorld::step`].
///
/// Um EVENTO, com canal próprio, pela razão que o W-TickContacts estabeleceu: uma
/// transição não é derivável do estado que a segue. A carga no instante da
/// ruptura está aqui e em lugar nenhum mais — um instante depois a corda não
/// carrega nada, porque ela não está mais segurando.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PulleyBreak {
    /// O `stable_name_id` do que partiu — a CORDA, ou a RODA.
    pub id: u64,
    /// `true` se o que cedeu foi o eixo de uma roldana; `false` para a corda.
    pub is_wheel: bool,
    /// Onde, em metros de mundo: o centro da roda, ou o meio da rota da corda.
    pub point: [f32; 2],
    /// A carga no instante em que cedeu, newtons.
    pub load: f32,
}

/// O que o passe escreve de volta além dos impulsos: os picos que a UI lê e o
/// que rompeu.
///
/// Um agregado e não seis argumentos soltos porque os seis são **um assunto** —
/// e porque `apply` já carregava sete.
pub struct PulleyLedger<'a> {
    /// Quanto cada guincho já recolheu, por id de corda.
    pub payout: &'a mut BTreeMap<u64, f32>,
    /// A maior tensão que cada corda segurou neste tique, newtons.
    pub tension: &'a mut BTreeMap<u64, f32>,
    /// A maior resultante que cada EIXO segurou neste tique, newtons.
    pub axle: &'a mut BTreeMap<u64, f32>,
    /// As cordas que já romperam — elas não voltam sem um rebuild.
    pub broken_ropes: &'a mut BTreeSet<u64>,
    /// Os eixos que já romperam: a roldana **sai da rota**.
    pub broken_wheels: &'a mut BTreeSet<u64>,
    /// O que rompeu NESTE tique.
    pub breaks: &'a mut Vec<PulleyBreak>,
}

/// **A carga de cada EIXO, e quem cedeu.**
///
/// ⚠️ **A resultante NÃO é a tensão.** A corda puxa o eixo com força `T` ao longo
/// de cada uma das duas direções em que ela SAI da roda — para trás pelo trecho
/// que chegou, para a frente pelo que parte —, então a força no eixo é
/// `T·|u_saída − u_entrada|`: um enlace de 180° carrega **2T**, e uma roldana que
/// quase não desvia a corda carrega quase nada. **É a mesma conta do Jacobiano**
/// (`∂L/∂centro = −(u_entra + u_sai)`, o cabeçalho da [`super::rope_route`]) —
/// uma conta, dois consumidores.
pub(super) fn ledger_axles(
    led: &mut PulleyLedger<'_>,
    live: &[RopeWheel],
    legs: &[Tangent],
    tension: f32,
) {
    for (i, w) in live.iter().enumerate() {
        // ⚠️ **A MESMA porta que o impulso do W3 usa** (`∂L/∂C`), e não uma
        // segunda subtração escrita aqui: a resultante no eixo É a magnitude do
        // Jacobiano daquele centro. Enquanto isto era uma cópia local ela dizia
        // `u_sai − u_entra`, o NEGATIVO do que a porta devolve — e as duas
        // magnitudes são bit-idênticas (o IEEE-754 faz `a−b` exatamente `−(b−a)`),
        // então a troca é livre e o que se ganha é não haver duas respostas.
        let Some(j) = super::rope_route::wheel_jacobian(legs, i) else {
            continue;
        };
        // `sqrt`, nunca `hypot`: o IEEE-754 especifica o primeiro exatamente e o
        // segundo é a libm da plataforma — e este número decide uma ruptura, que
        // move poses e alcança o hash C9 (a lei que o W-AreaFalloff escreveu).
        let load = tension * (j[0] * j[0] + j[1] * j[1]).sqrt();
        let slot = led.axle.entry(w.id).or_insert(0.0);
        *slot = slot.max(load);
        if load > w.break_force {
            led.broken_wheels.insert(w.id);
            led.breaks.push(PulleyBreak {
                id: w.id,
                is_wheel: true,
                point: w.centre,
                load,
            });
        }
    }
}

impl super::PhysicsWorld {
    /// O que rompeu durante o último [`Self::step`]. Vazio em quase todo tique.
    #[must_use]
    pub fn pulley_breaks(&self) -> &[PulleyBreak] {
        &self.pulley_breaks
    }

    /// **A maior tensão que esta corda segurou no último tique**, newtons.
    ///
    /// Zero para uma corda frouxa (ela não segura nada) e para uma que ainda não
    /// deu um passo — que é a mesma resposta, e é a certa: as duas não estão
    /// carregando.
    ///
    /// ⚠️ **O PICO sobre os sub-passos, não o valor de fim de passo** — pela razão
    /// que o `ContactReport::impact` escreve por extenso: a carga que decide uma
    /// ruptura acontece ENTRE os sub-passos e já foi resolvida quando `step`
    /// retorna.
    #[must_use]
    pub fn pulley_tension(&self, id: u64) -> f32 {
        self.pulley_tension.get(&id).copied().unwrap_or(0.0)
    }

    /// **A maior resultante que este EIXO segurou no último tique**, newtons —
    /// `T·|u_saída − u_entrada|`, e não a tensão da corda.
    #[must_use]
    pub fn pulley_axle_load(&self, wheel_id: u64) -> f32 {
        self.pulley_axle.get(&wheel_id).copied().unwrap_or(0.0)
    }

    /// **Esta corda ainda segura?** `false` depois que ela partiu.
    #[must_use]
    pub fn pulley_is_intact(&self, id: u64) -> bool {
        !self.pulley_broken_ropes.contains(&id)
    }

    /// **Este eixo ainda aguenta?** `false` depois que ele cedeu — a roldana saiu
    /// da rota e a corda passa direto.
    #[must_use]
    pub fn pulley_wheel_is_intact(&self, wheel_id: u64) -> bool {
        !self.pulley_broken_wheels.contains(&wheel_id)
    }
}
