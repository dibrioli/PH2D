//! **A RESPOSTA de um contacto, uma só para todas as formas** — e a metade que roda a peça
//! (doc 109 §7, report do dono 2026-09-13: *«os círculos não rotacionam com a colisão, talvez por
//! falta de atrito»*).
//!
//! Irmão do `lib.rs` pelo tecto de LOC (HR-18) e por ASSUNTO: ali cada forma responde *«quão fundo
//! e para que lado»*, aqui escreve-se o que isso FAZ à peça — uma vez, para as quatro formas.
//! *Escrever a resposta por forma é como um colisor ganha um defeito por forma.*
//!
//! ## Duas leis de atrito, e a fronteira entre elas é DECLARADA
//!
//! - **A peça sem forma declarada** (um ponto, ou um raio) fica no **sangramento tangencial** de
//!   sempre — `vt *= 1 − atrito` —, byte a byte. Ela não tem alavanca nenhuma para receber: um
//!   ponto não tem raio, e inventar-lhe um mudaria toda cena que este nó já shipou.
//! - **A peça que DECLAROU uma forma** (doc 109 §5) tem um ponto de contacto e, com ele, a alavanca
//!   `bt = r · n`. Aí o atrito é **Coulomb com alavanca**: o impulso tangencial que anula a
//!   velocidade **do ponto de contacto**, limitado a `μ · jn`, repartido entre travar e RODAR pela
//!   massa efectiva. Num disco `bt` é o raio inteiro (e a alavanca da normal é zero) — é isto, e só
//!   isto, que faz uma bola rolar em vez de derrapar.
//!
//! ⚠️ **A rotação daqui é VELOCIDADE ANGULAR (`spin`), não um empurrão de ângulo.** É a moeda deste
//! nó: ele responde em `vel`, não tem `dt` nenhum (é `Effect::Pure`) e, sem um passo, um
//! deslocamento não é derivável de uma velocidade. O `sim.step` integra o `spin` no `rot` no tique
//! seguinte, que é o mecanismo que já existia — e é ele que faz a bola **continuar** a rolar.
//!
//! ⚠️ **E ela AUTO-CORRIGE**: a velocidade que o atrito lê é a do PONTO (`v·t + ω·bt`), então uma
//! bola a girar depressa demais é travada pelo mesmo termo que a pôs a girar. Sem esse `ω·bt` ela
//! acelerava até ao infinito com o `angular_damping` no default (`1` = sem arrasto).

use ph2d_contact::GRAUS;

/// **O que este contacto faz à peça** — o material do par, já combinado, e a alavanca dela.
pub(super) struct Resposta {
    /// Quanto do embate volta, `0..1`.
    pub salto: f32,
    /// O coeficiente de atrito do par, `0..1`.
    pub atrito: f32,
    /// ⭐ **O atrito de ROLAMENTO da peça** (doc 109 §7.10) — dela, não do par (ver
    /// [`ph2d_nodegraph::attr::ROLLING_COLUMN`]). `0` = a lei de sempre, ao bit.
    pub rolar: f32,
    /// `(bt, invI)`: a alavanca da tangente no ponto e o quanto a peça roda por unidade de
    /// binário. `None` na peça sem forma declarada — ver o cabeçalho.
    pub rolamento: Option<(f32, f32)>,
}

/// Responde ao contacto: empurra a peça para fora, reflecte a normal, trava (ou faz rolar) a
/// tangente. Devolve **quanto o `spin` da peça mudou**, em graus por segundo.
pub(super) fn respond(
    p: &mut [f32; 2],
    v: &mut [f32; 2],
    spin: f32,
    n: [f32; 2],
    depth: f32,
    r: &Resposta,
) -> f32 {
    p[0] += n[0] * depth;
    p[1] += n[1] * depth;

    let vn = v[0] * n[0] + v[1] * n[1];
    // Already leaving (or sliding along) the surface: touching it must not change it. Reflecting
    // here is the classic collider jitter — the element buzzes on the ground forever, fed by its
    // own contact test.
    if vn >= 0.0 {
        return 0.0;
    }
    // Reflect the normal component, keep (and bleed) the tangential one.
    let bounce = (1.0 + r.salto) * vn;
    let out = [v[0] - bounce * n[0], v[1] - bounce * n[1]];
    let vn_out = out[0] * n[0] + out[1] * n[1];
    let tangent = [out[0] - vn_out * n[0], out[1] - vn_out * n[1]];
    let Some((bt, inv_i)) = r.rolamento else {
        let keep = 1.0 - r.atrito;
        *v = [
            vn_out * n[0] + tangent[0] * keep,
            vn_out * n[1] + tangent[1] * keep,
        ];
        return 0.0;
    };
    // ⭐⭐⭐ **COULOMB COM ALAVANCA** (doc 109 §7). `jn` é o impulso normal que acabou de ser
    // aplicado (por unidade de massa — este nó não lê `inv_mass`, ver [`super::declared::toque`]).
    let t = [0.0 - n[1], n[0]];
    let jn = 0.0 - bounce;
    let vt = tangent[0] * t[0] + tangent[1] * t[1] + (spin / GRAUS) * bt;
    let kt = 1.0 + inv_i * bt * bt;
    let teto = (r.atrito * jn).max(0.0);
    let jt = ((0.0 - vt) / kt).clamp(-teto, teto); // CLAMP-OK: teto >= 0
    *v = [out[0] + t[0] * jt, out[1] + t[1] * jt];
    let d_spin = jt * inv_i * bt;
    if r.rolar <= 0.0 || inv_i <= 0.0 {
        return d_spin * GRAUS;
    }
    // ⭐⭐⭐ **O ROLAMENTO** (doc 109 §7.10, a ponta que o §7.7 nomeava). ⚠️ Ele lê o `ω` **já
    // corrigido pelo tangencial** — os dois escrevem a mesma grandeza, e lidos do mesmo `ω` este
    // desfaria parte do giro que aquele acabou de dar.
    let omega = spin / GRAUS + d_spin;
    let jr = ph2d_contact::atrito::rolamento(omega / inv_i, r.rolar, jn, bt);
    (d_spin - jr * inv_i) * GRAUS
}

#[cfg(test)]
#[path = "resposta_tests.rs"]
mod tests;
