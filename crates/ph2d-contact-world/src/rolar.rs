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

/// **A trava destranca só ao 12.º excesso SEGUIDO** do binário pedido (`0,2` s a `60` Hz) — doc 121
/// §9.23. Numa peça TRANCADA o binário que os contactos pedem é estaticamente indeterminado (vários
/// apoios, o aquecimento do solver) e salta acima e abaixo da capacidade de passo para passo; ao 2.º
/// excesso a pilha assente trocava `20`–`39` vezes por peça por segundo, rastejava, e uma queda em
/// onze desabava. Rodada (a `=114`, `1` sub-passo, `11` realizações; queda = o rodopio de `120..180`,
/// mediana · pior; assente = o pior rodopio e o pior deslize por janela depois do `240`):
///
/// | soltar ao | discos `0,05` queda · assente | discos `0,1` queda · assente | caixas `0,1` assente (trocas/s) |
/// |---|---|---|---|
/// | sem o botão (`Rolling 0`) | `131` · `225` · `21°` | (o mesmo) | — |
/// | 2.º (a de antes) | `225` · `295` · `117°` | `1,8` · **`152`** · `75°`/`0,18` u | `1,4°` (`35`) |
/// | 3.º | `189` · `271` · `71°` | `1,0` · `1,2` · `1,7°` | `2,5°` (`24`) |
/// | 8.º | `1,4` · `137` · `88°` | `0,4` · `0,4` · `0,5°` | `1,1°` (`9,5`) |
/// | **12.º** | **`1,4` · `170` · `11°`** | **`0,3` · `0,3` · `0,4°`** | **`0,6°` (`6,6`)** |
///
/// ⇒ só o `12` nunca gira mais que sem o botão (na queda e assente); a `0,15`/`0,25`/`0,75` os
/// discos ficam a `0` com todos. ⚠️ O preço: uma peça cujo binário passa a capacidade de VERDADE
/// começa a rolar `0,2` s depois. ⛔ Medidos e recusados na mesma rodada: a folga no limiar
/// (`×1,25`, `×2`: discos `0,1` pior queda `153`/`138°`), o passo que solta com o binário CONTRA o
/// pedido (queda `36°` de mediana) e o motor angular do rapier (o rolamento dentro do solver:
/// `58°` de mediana e a pilha a rastejar, `28°`) — a tabela no doc.
///
/// A rodada de 06/10 (§9.21, ainda com a taça numa polilinha): ao 1.º excesso `69°`/`44°` (discos
/// `0,1`/`0,25`), só trancar sem giro pedido `320°`/`90°`, esperar `4` passos `273°`/`85°`.
pub(crate) const EXCESSOS_PARA_SOLTAR: u8 = 12;

/// **E só TRANCA quem não acelerou no passo** (`|ω| ≤ |ω antes|`) — ou quem mal se mexeu (o momento
/// `≤ QUASE ·` a capacidade). Trancar logo que o giro de um passo cabe na capacidade prendia uma bola
/// numa rampa MAIS inclinada que o `Rolling` dela: ao soltar, o 1.º passo dá-lhe sempre um giro
/// pequeno, e ela voltava a trancar — a de antes (ao 2.º e ao 12.º excesso) deixava-a a `2 %` da
/// descida. A rampa da `=115` (`12°`, `2` s; a distância contra a de `Rolling 0`, teoria
/// `(tg θ − μr) / tg θ`) e as pilhas (a tabela de cima, `11` realizações):
///
/// | trava | rampa `0,1` · `0,15` · `0,2` · `0,5` | discos `0,1` queda · assente | caixas `0,1` · `0,25` assente |
/// |---|---|---|---|
/// | (teoria) | `0,530` · `0,294` · `0,059` · `0` | — | — |
/// | tranca com o momento `≤` capacidade | `0,019` · `0,019` · `0,019` · `0,018` | `0,3` · `0,3` · `0,4°` | `0,6°` · `0,5°` |
/// | só quem não acelera | `0,556` · `0,241` · `0,103` · `0,019` | `35` · `80` · `32°` | `1,7°` · `3,4°` |
/// | **e quem mal se mexeu (`25 %`)** | **`0,556` · `0,241` · `0,103` · `0,018`** | **`5,6` · `41` · `11,6°`** | **`0,9°` · `0,6°`** |
/// | idem com `10 %` · `50 %` | igual · a `0,15` volta a prender | `6,3` · `16` · `47°` · `1,6` · `102` · `28°` | — |
///
/// ⇒ a pior queda dos discos `0,1` (`41°`) é um disco a ROLAR `0,10` u por um monte de `37°` (o
/// caminho `1,2×` o giro vezes o raio, `0 %` de giro no lugar) — avalanche, a física: `Rolling 0,1`
/// só segura até `5,7°`. ⛔ O passo que solta com o binário CONTRA o pedido volta a prender a bola a
/// `0,15`/`0,2` (`0,026`).
pub(crate) const QUASE: f32 = 0.25;

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
