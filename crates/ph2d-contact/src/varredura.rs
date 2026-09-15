//! **O QUE UMA PEÇA RECEBE DOS VIZINHOS NUMA VARREDURA** — a acumulação por peça.
//!
//! Irmã do `lib.rs` pelo tecto de LOC (HR-18) e por ASSUNTO: ali mora a NUVEM (a grelha, o laço das
//! varreduras, a média de Jacobi, a escrituração); aqui mora o que **um** `k` soma dos parceiros
//! dele — a normal, o [`ENCOSTO DE DOIS PONTOS`](super::Manifesto) e a metade tangencial.
//!
//! ⚠️ O par é sempre lido na ordem do PAR (menor → maior) e os parceiros chegam em ordem CRESCENTE:
//! é isso que faz a grelha dar os MESMOS BITS que todos-os-pares. Ver o cabeçalho do `lib.rs`.

use super::{Colisor, GRAUS, Nova, Pecas, atrito, dot, manifesto};

/// A posição, o giro e o salto de `k` depois desta varredura, ou `None` se nada lhe tocou. Os `parceiros` têm
/// de vir em ordem CRESCENTE — ver o cabeçalho.
pub(super) fn corrigida(
    k: usize,
    parceiros: impl Iterator<Item = usize>,
    foto: &[[f32; 2]],
    girado: &[f32],
    colisores: &[Option<Colisor>],
    pecas: &Pecas<'_>,
    ativo: &[bool],
) -> Nova {
    let (pesos, inv_inercia) = (pecas.pesos, pecas.inv_inercia);
    let mut delta = [0.0_f32; 2];
    let mut giro = 0.0_f32;
    let mut salto = 0.0_f32;
    let mut contatos = 0_u32;
    for j in parceiros {
        if j == k || !ativo[j] {
            continue;
        }
        let (lo, hi) = (k.min(j), k.max(j));
        let (Some(clo), Some(chi)) = (colisores[lo], colisores[hi]) else {
            continue;
        };
        // O par na ordem do PAR (ver o cabeçalho): a normal vai do menor para o maior.
        // ⭐⭐⭐ **O ENCOSTO DE DOIS PONTOS** (doc 111 §5.10): de caixa contra caixa o par entrega o
        // TRECHO, e cada extremo dele traz a PROFUNDIDADE dele. Ver [`Manifesto`].
        let Some(m) = manifesto(&clo, foto[lo], &chi, foto[hi], (lo + hi) % 2 == 0) else {
            continue;
        };
        // O centro de cada lado, e daí a massa efectiva no PONTO do contacto (doc 109 §6).
        let (ck, cj) = if k == lo {
            (clo.centro(foto[lo]), chi.centro(foto[hi]))
        } else {
            (chi.centro(foto[hi]), clo.centro(foto[lo]))
        };
        let sinal = if k == lo { -1.0 } else { 1.0 };
        let massa = |w: f32, inv_i: f32, b: f32| w + inv_i * b * b;
        // ⭐⭐⭐ **OS DOIS PONTOS SÃO RESTRIÇÕES INDEPENDENTES, cada um com o SEU `λ`.**
        //
        // ⛔⛔ **A alternativa — média e diferença — foi construída, MEDIDA e REFUTADA**, e a razão
        // é física: partir o trecho em «a média translada, a diferença roda» obriga os dois pontos
        // a partilhar UM `λ`, e com isso o solver **perde a capacidade de redistribuir a pressão**
        // ao longo do apoio. É precisamente essa redistribuição que deixa uma caixa apoiada pelo
        // CENTRO ficar quieta sobre um apoio que só cobre parte da base dela: a resultante desloca-se
        // dentro do trecho até passar pelo centro de massa. Com o `λ` partilhado a resultante fica
        // presa no MEIO do trecho, e a caixa apoiada tomba `5,06°` — o defeito de volta, mais
        // depressa. *Convergir e acertar são coisas diferentes.*
        //
        // ⚠️ **O preço é CONVERGÊNCIA, e está medido:** cada ponto carrega o braço na massa efectiva
        // dele (`k = w + invI·b²`, e `b` numa ponta é maior que no meio), logo cada varredura
        // corrige menos. Duas caixas `0,5 × 2,0` ficam `3,5 %` curtas às `8` varreduras e
        // **exactas às `64`**; os quadrados da `=114` ficam `0,15 %` curtos às `8` e exactos às
        // `32`. ⇒ *um resíduo que encolhe com as varreduras é convergência, não viés*, e os gates
        // medem as DUAS pontas.
        // ⚠️⚠️ **E eles entram pela MÉDIA, que é o esquema que o cabeçalho já declara** («cada
        // peça soma o que os contatos dela pedem, e aplica a MÉDIA»): dois pontos de um par são
        // dois contactos, e a lei deles é a dos vizinhos. ⛔ **SOMAR foi medido e REFUTADO** — sem
        // a média o par sobrepassa e **cinco** gates de geometria caem de uma vez (as caixas lado a
        // lado, a empilhada, a girada, as coincidentes e a apoiada).
        let pontos = m.pontos();
        #[expect(
            clippy::cast_precision_loss,
            reason = "o manifesto tem 1 ou 2 pontos, nunca mais"
        )]
        let quota = 1.0 / pontos.len() as f32;
        let mut tocou = false;
        for c in pontos {
            let (bk, bj) = (c.braco(ck), c.braco(cj));
            let soma = massa(pesos[k], inv_inercia[k], bk) + massa(pesos[j], inv_inercia[j], bj);
            // Dois obstáculos (ou dois pesos infinitos) não têm correcção a repartir.
            if soma <= 0.0 {
                continue;
            }
            tocou = true;
            let lambda = c.penetracao / soma;
            let empurra = lambda * pesos[k] * sinal * quota;
            delta[0] += c.normal[0] * empurra;
            delta[1] += c.normal[1] * empurra;
            giro += bk * lambda * inv_inercia[k] * sinal * GRAUS * quota;
            // ⭐⭐⭐ **A METADE TANGENCIAL** (doc 109 §7) — o deslize desfeito, limitado por Coulomb.
            // ⚠️ Sem `sinal`: o deslize já é medido **de `k` para `j`**, então a correcção dele é
            // simétrica por construção e os dois lados do par concordam sem desempate nenhum.
            if let Some(d) = pecas.deslize {
                let (mk, mj) = (pecas.material(k), pecas.material(j));
                salto = salto.max(atrito::salto(mk.salto, mj.salto));
                let t = c.tangente();
                let (tk, tj) = (c.braco_tangente(ck), c.braco_tangente(cj));
                // Quanto o PONTO de contacto de cada lado andou desde o início do passo: o centro
                // MAIS o que a rotação do corpo lhe acrescentou (`dθ × r`, em 2D `dθ · perp(r)`).
                let andou = |i: usize, centro: [f32; 2]| {
                    let dtheta = (girado[i] + d.girou_antes[i]) / GRAUS;
                    let r = c.raio(centro);
                    [
                        foto[i][0] - d.antes[i][0] - dtheta * r[1],
                        foto[i][1] - d.antes[i][1] + dtheta * r[0],
                    ]
                };
                let (ak, aj) = (andou(k, ck), andou(j, cj));
                let desliza = dot([ak[0] - aj[0], ak[1] - aj[1]], t);
                let soma_t =
                    massa(pesos[k], inv_inercia[k], tk) + massa(pesos[j], inv_inercia[j], tj);
                let lt = atrito::lambda(desliza, soma_t, atrito::mu(mk.atrito, mj.atrito), lambda);
                delta[0] -= t[0] * lt * pesos[k] * quota;
                delta[1] -= t[1] * lt * pesos[k] * quota;
                giro -= tk * lt * inv_inercia[k] * GRAUS * quota;
            }
        }
        if !tocou {
            continue;
        }
        contatos += 1;
    }
    (contatos > 0).then(|| {
        #[expect(
            clippy::cast_precision_loss,
            reason = "uma contagem de contatos de uma peca, muito abaixo de 2^24"
        )]
        let inv = 1.0 / contatos as f32;
        (
            [foto[k][0] + delta[0] * inv, foto[k][1] + delta[1] * inv],
            giro * inv,
            salto,
        )
    })
}
