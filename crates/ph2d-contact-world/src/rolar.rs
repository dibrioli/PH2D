//! ⭐⭐ **O ATRITO DE ROLAMENTO** (doc 121 §9.20, ponto 4) — a lei de Coulomb do rolamento em DUAS
//! FASES, porque o rapier `0.35.3` não a tem (lido no fonte: só o `ray_cast_vehicle_controller` fala
//! de *rolling*).
//!
//! A porta é a de antes ([`ph2d_contact::atrito::rolamento`]): o giro de cada peça é travado contra
//! o PRÓPRIO momento, com o PRÓPRIO `μr` (é da peça, nunca do par), até à capacidade
//! `Σ μr · λn · |braço|` dos contactos dela — o `λn` que o rapier RESOLVEU no passo
//! (`ContactData::impulse`), o braço do centro ao ponto ao longo da normal.
//!
//! - **PARADA** (`|L| ≤ capacidade`: o rolamento pára-a neste passo) ⇒ a rotação fica TRAVADA no
//!   solver (o eixo angular do corpo trancado, uma restrição exacta). Ela destrava quando o binário
//!   que os contactos lhe pedem passa da capacidade — logo segura numa rampa enquanto
//!   `tg θ ≤ μr`, a lei estática (a `=115`: `0,5` prende numa rampa de `12°`, `tg = 0,21`).
//! - **A ROLAR** (`|L| > capacidade`) ⇒ um binário constante de `capacidade / dt` contra o giro, no
//!   passo seguinte — a desaceleração de Coulomb, sem inverter.
//!
//! ⛔ Medido e recusado (a pilha da `=114`, caixas e discos, `Rolling 0,05..0,75`; a bola na rampa):
//!
//! | lei | o que faz |
//! |---|---|
//! | cortar o `ω` logo DEPOIS do passo | a pilha de caixas mais trémula COM o botão (`0,048` °/tique contra `0,004`): o aquecimento do rapier corrige uma velocidade que já não existe |
//! | binário do travão no passo seguinte, sempre (ganho `1`) | o mesmo; e a pilha parada CHACOALHA (o binário passa do zero e volta) |
//! | ganho `½` | limpo até `0,1`; com `≥ 0,25` as peças paradas RASTEJAM (`1,3`–`4` °/s) |
//! | o amortecimento angular do rapier (uma vez por passo, depois do solver) | como o corte |
//! | o travão RELATIVO por par (o do Box2D v3) | errático (um evento de `42°` numa pilha parada) |
//! | o contacto do disco ACHATADO (dois pontos a `±μr·R`, pelo gancho) | ACELERA a bola: os pontos ficam presos ao disco e giram com ele |
//!
//! Só a trava estática dá o repouso EXACTO, porque é a única que fica dentro do solver.

/// **A trava destranca só ao 2.º excesso SEGUIDO** do binário pedido (doc 121 §9.20, a rodada das
/// variantes, `1` sub-passo; o rodopio somado das três janelas `120..180 · 240..300 · 420..480`):
///
/// | regra | caixas `0,1` · `0,25` | discos `0,1` · `0,25` |
/// |---|---|---|
/// | destranca ao 1.º excesso | `3,7°` · `4,4°` | `69°` · `44°` |
/// | **ao 2.º seguido** | **`3,2°` · `3,9°`** (tremor `0`) | `110°` · **`0°`** |
/// | só tranca se os contactos não pedem giro | `5,9°` · `2,8°` | `320°` · `90°` |
/// | espera `4` passos depois de destrancar | `2,7°` · `7,1°` | `273°` · `85°` |
///
/// ⚠️ Os discos a `0,1` agitam na QUEDA em todas (`104°` na 1.ª janela contra `10,7` sem o botão) e
/// assentam como sem ele nas outras (`3,1°` contra `2,4°`): leitura — um monte de discos com
/// rolamento fica inclinado e desaba enquanto assenta; sem rolamento eles rolam logo ao fundo.
pub(crate) const EXCESSOS_PARA_SOLTAR: u8 = 2;

use rapier2d::prelude::*;
use std::collections::BTreeMap;

/// O que cada peça precisa para ser travada: o `μr`, o inverso da inércia (`0` = travada pelo
/// `Lock Rotation`) e a linha do stream onde está.
pub(crate) struct Rolante {
    pub mu_r: f32,
    pub inercia_inv: f32,
    pub linha: usize,
}

/// O que os contactos de cada linha fizeram no passo: a `capacidade` do rolamento
/// (`Σ μr·λn·|braço|`) e o `pedido` — o impulso angular que os contactos lhe aplicaram (o que uma
/// peça TRAVADA precisou de segurar).
pub(crate) struct Leitura {
    pub capacidade: Vec<f32>,
    pub pedido: Vec<f32>,
}

fn cruz(a: Vector, b: Vector) -> f32 {
    a.x * b.y - a.y * b.x
}

/// Lê os contactos do passo. ⚠️ A convenção é a do solver do rapier: o impulso no 1.º corpo vai
/// ao longo de `−normal` (`force_dir1`) e o de atrito ao longo de `perp(−normal)`; o 2.º recebe o
/// oposto.
pub(crate) fn le(
    narrow: &NarrowPhase,
    colliders: &ColliderSet,
    bodies: &RigidBodySet,
    rolantes: &BTreeMap<u32, Rolante>,
    linhas: usize,
) -> Leitura {
    let mut l = Leitura {
        capacidade: vec![0.0; linhas],
        pedido: vec![0.0; linhas],
    };
    if rolantes.is_empty() {
        return l;
    }
    for par in narrow.contact_pairs() {
        if !par.has_any_active_contact() {
            continue;
        }
        let (Some(c1), Some(c2)) = (colliders.get(par.collider1), colliders.get(par.collider2))
        else {
            continue;
        };
        let lado = |c: &Collider| {
            let b = c.parent().and_then(|h| bodies.get(h))?;
            #[expect(
                clippy::cast_possible_truncation,
                reason = "o id vive nos 32 bits de baixo"
            )]
            let r = rolantes.get(&(b.user_data as u32))?;
            (r.mu_r > 0.0 && r.inercia_inv > 0.0).then_some((r, b.translation()))
        };
        let (a, b) = (lado(c1), lado(c2));
        if a.is_none() && b.is_none() {
            continue;
        }
        for m in &par.manifolds {
            let n = m.data.normal;
            let f = -n;
            let t = Vector::new(-f.y, f.x);
            for ponto in &m.points {
                let (ln, lt) = (ponto.data.impulse, ponto.data.tangent_impulse.x);
                if !(ln.is_finite() && lt.is_finite()) {
                    continue;
                }
                let impulso = f * ln + t * lt;
                for (lado, c, local, sinal) in [
                    (a.as_ref(), c1, ponto.local_p1, 1.0),
                    (b.as_ref(), c2, ponto.local_p2, -1.0),
                ] {
                    let Some((r, centro)) = lado else { continue };
                    let braco = c.position().transform_point(local) - *centro;
                    if ln > 0.0 {
                        l.capacidade[r.linha] += r.mu_r * ln * braco.dot(n).abs();
                    }
                    l.pedido[r.linha] += sinal * cruz(braco, impulso);
                }
            }
        }
    }
    for p in &mut l.pedido {
        *p = p.abs();
    }
    l
}

/// **O binário da fase A ROLAR** para o passo de `dt`: o impulso angular que pára o giro, no máximo
/// `capacidade` (nunca inverte), dividido pelo passo — o rapier integra-o por sub-passo, exacto
/// (medido: a desaceleração de um disco a rolar com um binário constante é `1,00×` a teoria).
pub(crate) fn binario(w: f32, inercia_inv: f32, capacidade: f32, dt: f32) -> f32 {
    if capacidade <= 0.0 || inercia_inv <= 0.0 || dt <= 0.0 {
        return 0.0;
    }
    let j = ph2d_contact::atrito::rolamento(w / inercia_inv, capacidade, 1.0, 1.0);
    if j.is_finite() { -j / dt } else { 0.0 }
}
