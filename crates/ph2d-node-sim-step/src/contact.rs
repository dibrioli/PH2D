//! ⭐⭐ **O PASSO RESOLVE O CONTACTO ENTRE PEÇAS** — doc 109, ordem do dono (2026-09-13):
//! *«colidem sozinhas»*.
//!
//! Todo motor faz num `step` duas coisas: integra e resolve contatos. Este nó fazia a primeira;
//! a segunda passa a correr aqui, **depois** da integração, sobre as peças que declaram colisor
//! (as colunas `collider` / `collider_box`, escritas por quem desenha). Sem nenhuma das duas nada
//! aqui corre, e o passo é o de sempre.
//!
//! ## A posição: a lei da folha `ph2d-contact`
//!
//! Cada peça é o colisor que DECLAROU — um disco ou uma caixa, escalado pelo `size` e girado pelo
//! `rot` dela (doc 109 §5) —, lido pela porta `ph2d_contact::colisores`; e o peso é o `inv_mass`,
//! para um pino ser obstáculo.
//!
//! ## E a ROTAÇÃO (doc 109 §6)
//!
//! Report do dono: *«precisa destravar a rot»*. A correcção reparte-se entre mover e rodar pela
//! massa efectiva no PONTO do contacto, e o quanto cada peça rodou volta daqui em GRAUS para o
//! [`super::step`] somar à coluna `rot`. Quem trava é a coluna `inv_inertia` a `0` — o botão
//! `Lock Rotation` do cartão da forma.
//!
//! ## A velocidade: só se CANCELA a aproximação
//!
//! ⚠️ **Sem isto a pilha RESPIRA:** a velocidade continuaria a empurrar a peça para dentro da
//! vizinha a cada tique, a posição seria corrigida outra vez, e o monte nunca assentaria (é o `93 %`
//! do vão que a cena `=114` mede com o `motion.collide`, que só mexe em `P`).
//!
//! ⚠️ **E só se cancela — nunca se acrescenta.** A correcção `Δp` diz para que lado o contacto
//! empurrou; a componente da velocidade CONTRA esse lado é tirada, até `|Δp| / dt`, e nada mais.
//! Duas peças que NASCEM sobrepostas separam-se em posição e **não** ganham velocidade — somar
//! `Δp / dt` inteiro faria delas uma explosão. É a regra do `sim.collide`: *só se responde a quem se
//! move PARA DENTRO*.
//!
//! ⛔ **A rotação não tem velocidade angular** — ela é projecção de posição, como o afastamento. Uma
//! peça roda enquanto toca e não continua a girar no ar (doc 109 §6, nomeado).

use ph2d_nodegraph::attr::Stream;

/// **Varreduras por passo.** É o default que o `motion.collide` shipa, medido na tabela dele
/// (`measure_packing_and_order_dependence`: a folga mínima de uma nuvem apertada sobe de `0,050`
/// com Gauss–Seidel para `0,270` com este esquema a 8); e os `substeps` da zona multiplicam-no sem
/// mais nenhum knob.
pub(crate) const VARREDURAS: usize = 8;

/// Separa as peças com colisor, cancela a aproximação delas e devolve **quanto cada uma rodou**, em
/// graus (vazio quando ninguém declara colisor). `dt(i)` é o passo daquela peça.
pub(crate) fn resolve(
    state: &Stream,
    p: &mut [[f32; 2]],
    vel: &mut [[f32; 2]],
    pesos: &[f32],
    dt: impl Fn(usize) -> f32,
) -> Vec<f32> {
    let n = p.len();
    let Some(colisores) = ph2d_contact::colisores(state) else {
        return Vec::new();
    };
    if colisores.len() != n {
        return Vec::new();
    }
    let inv_inercia = ph2d_contact::inv_inercias(state, &colisores, pesos);
    let mut giro = vec![0.0_f32; n];
    let antes = p.to_vec();
    ph2d_contact::separate(p, &mut giro, &colisores, pesos, &inv_inercia, VARREDURAS);
    for i in 0..n {
        let d = [p[i][0] - antes[i][0], p[i][1] - antes[i][1]];
        let len = d[0].hypot(d[1]);
        let dti = dt(i);
        if len <= 0.0 || dti <= 0.0 {
            continue;
        }
        let normal = [d[0] / len, d[1] / len];
        let vn = vel[i][0] * normal[0] + vel[i][1] * normal[1];
        if vn >= 0.0 {
            continue;
        }
        let tira = (-vn).min(len / dti);
        let v = [vel[i][0] + normal[0] * tira, vel[i][1] + normal[1] * tira];
        if v.iter().all(|x| x.is_finite()) {
            vel[i] = v;
        }
    }
    giro
}

#[cfg(test)]
#[path = "contact_tests.rs"]
mod tests;
