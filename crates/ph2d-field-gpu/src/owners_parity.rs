//! ⭐⭐⭐ **A LEI DO DONO, NOS DOIS MOTORES** — o arnês que mede a
//! [`ph2d_field_eval::owners::Owners::mix_at`] contra a irmã de WGSL.
//!
//! # ⚠️ Porque ela precisa de arnês PRÓPRIO, e não do gate de paridade do quadro
//!
//! O gate do quadro compara **imagens**, e uma imagem esconde esta lei: numa peça de duas folhas da
//! mesma cor o dono errado pinta exactamente o mesmo pixel. *Um zero de «igual» e um de «nenhum dos
//! dois olhou» são o mesmo byte* — a mesma armadilha que a [`crate::supports`] existe para tapar,
//! um nível acima.
//!
//! ⇒ o que se mede aqui é a **RESPOSTA** (`a`, `b`, `t`), ponto a ponto, sobre a grelha em que ela
//! muda: a fronteira entre duas folhas.
//!
//! # ⚠️⚠️ A divergência DECLARADA, e é a mesma da fita
//!
//! A CPU avalia cada folha em `f64` ([`ph2d_field_eval::owners`]) e o dispositivo em `f32`. ⇒ **os
//! índices têm de bater sempre** (eles são um desempate, e um desempate não tem meio termo), e o `t`
//! bate à precisão de `f32`. ⛔ Na fronteira exacta o desempate é **legitimamente** instável — dois
//! campos que valem o mesmo número dão donos diferentes por um ULP —, e é por isso que o gate mede
//! os índices **onde eles não empatam** e mede o `t` em todo lado.

use ph2d_field_eval::owners::Owners;

/// A resposta de um ponto: as duas folhas e o peso da segunda.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mistura {
    pub a: u32,
    pub b: u32,
    pub t: f32,
}

/// O molde do arnês — os três bindings que o [`crate::probe::evaluate`] exige, nesta ordem.
const MOLDE: &str = r"
@group(0) @binding(0) var<storage, read> k: array<f32>;
@group(0) @binding(1) var<storage, read> entrada: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> saida: array<vec4<f32>>;
{OWNERS}
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) g: vec3<u32>) {
    let i = g.x;
    if (i >= arrayLength(&entrada)) { return; }
    let e = entrada[i];
    let d = dono_mix(e.xyz, e.w);
    saida[i] = vec4<f32>(f32(d.a), f32(d.b), d.t, 0.0);
}
";

/// ⭐ **A lei do dono corrida no DISPOSITIVO**, sobre `(ponto, largura)` por amostra.
///
/// `None` quando não há adaptador ou quando alguma folha não tem fita.
#[must_use]
pub fn on_device(owners: &Owners, samples: &[([f32; 3], f32)]) -> Option<Vec<Mistura>> {
    let t = crate::trace::Tracer::new()?;
    let (device, queue) = t.parts();
    on_device_with(device, queue, owners, samples)
}

/// O mesmo, sobre um dispositivo que o chamador já tem — ver [`crate::trace::Tracer::parts`].
#[must_use]
pub fn on_device_with(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    owners: &Owners,
    samples: &[([f32; 3], f32)],
) -> Option<Vec<Mistura>> {
    // ⚠️ **`const_base` é `0` aqui porque o `k` deste arnês é SÓ das folhas.** No produto ele é o
    // tamanho do vector da fita da peça, e é o traçado que o escolhe.
    let lei = owners.to_wgsl(0)?;
    let inputs: Vec<[f32; 4]> = samples
        .iter()
        .map(|(p, w)| [p[0], p[1], p[2], *w])
        .collect();
    let out = crate::probe::evaluate(
        device,
        queue,
        &MOLDE.replace("{OWNERS}", &lei.source),
        "main",
        &[],
        &[&lei.consts],
        &inputs,
        1,
    );
    Some(
        out.into_iter()
            .map(|q| Mistura {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                a: q[0] as u32,
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                b: q[1] as u32,
                t: q[2],
            })
            .collect(),
    )
}

/// A MESMA pergunta na CPU — a porta do produto, sem uma linha de tradução.
#[must_use]
pub fn on_cpu(owners: &Owners, samples: &[([f32; 3], f32)]) -> Vec<Mistura> {
    samples
        .iter()
        .map(|(p, w)| {
            let (a, b, t) = owners.mix_at(*p, *w);
            Mistura {
                #[allow(clippy::cast_possible_truncation)]
                a: a as u32,
                #[allow(clippy::cast_possible_truncation)]
                b: b as u32,
                t,
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "owners_parity_tests.rs"]
mod tests;
