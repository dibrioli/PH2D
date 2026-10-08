//! ⭐⭐ **O corpo LÊ o colisor que as suas peças declaram** (doc 124 §6, decisão do dono de
//! 2026-10-08: *o botão `Collide` do Shape segura também as bolinhas de um corpo mole*).
//!
//! As peças entram pela porta `shape` (`Grid → Duplicator(Shape) → Soft Body`), e com o `Collide`
//! do Shape ligado a corrente traz a DECLARAÇÃO do colisor de cada uma. O corpo separa-as com o
//! motor da casa (`ph2d_contact`) dentro de cada passo — o que o `sim.step` faz numa zona —, porque
//! o passe do fim do cozimento é acabamento do desenho e não segura nada que um solver empurre
//! (doc 115 §15.3, e o smoke do dono de 08/10 mediu-o outra vez nesta cena).
//!
//! Os pares que o REPOUSO já sobrepõe são do corpo, não do contacto ([`ph2d_contact::Ligados`]).

use ph2d_contact::{Colisor, Pecas, Saida};
use ph2d_nodegraph::attr::Stream;

use crate::layout::BodyLayout;
use crate::{BODY_COL, EMITTED_COL, Params, REST_COL, STEP_DT_COL};

/// As varreduras do contacto por passo — o default da casa (o `motion.collide`, o passe do
/// Output). ⚠️ Medido na cena `=129`: `8`/`32`/`64` dão o mesmo; quem decide são os SUB-PASSOS.
const VARREDURAS: usize = 8;

/// O que as peças declararam, e o corpo a que pertencem.
pub(crate) struct Contacto {
    colisores: Vec<Option<Colisor>>,
    corpo: f32,
}

impl Contacto {
    /// `None` quando a corrente não declara colisor nenhum — o corpo de sempre, ao bit.
    pub(crate) fn de(entrada: &Stream, n: usize, corpo: f32) -> Option<Self> {
        let colisores = ph2d_contact::colisores(entrada)?;
        (colisores.len() == n && colisores.iter().any(Option::is_some))
            .then_some(Self { colisores, corpo })
    }

    /// Separa as peças no sítio. Uma partícula PINADA (o pino do corpo, ou massa inversa `0`) é um
    /// obstáculo: não se move, e as outras arrumam-se à volta dela.
    pub(crate) fn separa(
        &self,
        pos: &mut [[f32; 2]],
        layout: &BodyLayout,
        p: &Params,
        inv_mass: &[f32],
    ) {
        let n = pos.len();
        let pesos: Vec<f32> = (0..n)
            .map(|i| {
                if p.is_pinned(layout, i) {
                    0.0
                } else {
                    inv_mass.get(i).copied().unwrap_or(1.0).max(0.0)
                }
            })
            .collect();
        // As partículas de um corpo mole não rodam sozinhas: quem as orienta é o corpo.
        let inercia = vec![0.0f32; n];
        let corpo = vec![self.corpo; n];
        let mut giro = vec![0.0f32; n];
        ph2d_contact::separate(
            pos,
            &mut Saida { giro: &mut giro },
            &Pecas::novas(&self.colisores, &pesos, &inercia).com_ligados(&layout.rest, &corpo),
            VARREDURAS,
        );
    }
}

/// As colunas que o corpo ESCREVE ou LÊ do próprio estado — nunca vêm da aparência das peças.
pub(crate) const DO_CORPO: &[&str] = &[
    "P",
    "sb_vel",
    "sim_t",
    "accel",
    "inv_mass",
    "falloff",
    EMITTED_COL,
    STEP_DT_COL,
    REST_COL,
    BODY_COL,
];

/// ⭐ **O corpo passa adiante a APARÊNCIA das peças** (doc 124 §6): as colunas da porta `shape` que
/// não são dele — a forma, o tamanho, a cor, o colisor declarado — viajam com cada partícula, e as
/// peças desenham-se sem um segundo Duplicator. ⚠️ As do corpo ([`DO_CORPO`]) ficam de fora: uma
/// `falloff` de montante passaria a ser o peso do corpo no tique seguinte.
pub(crate) fn veste(out: &mut Stream, forma: &Stream) {
    for (nome, col) in forma.columns() {
        if !DO_CORPO.contains(&nome.as_str()) && out.get(nome).is_none() {
            out.set(nome.clone(), col.clone());
        }
    }
}
