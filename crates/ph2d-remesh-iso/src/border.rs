//! ⛔⛔⛔ **A LEI DO REBORDO — construída, MEDIDA e REJEITADA**, e ela cumpre o que promete.
//!
//! ⚠️ **Ela vive num irmão do [`crate::lib`] por causa do tecto de LOC**, e o corte é por
//! RESPONSABILIDADE: aqui está a lei do **bordo**, lá a do interior. *A tabela da recusa
//! viaja com o código que ela recusa* — apagar um sem o outro deixaria a próxima janela a
//! reconstruir o que já foi pago.

use ph2d_mesh::Mesh;

use crate::dot;

/// ⛔⛔⛔ **FALSE — A LEI DO REBORDO FOI CONSTRUÍDA, MEDIDA E REJEITADA, e ela CUMPRE o
/// que promete.**
///
/// A lei: um vértice de bordo é alisado **ao longo do rebordo** (só os vizinhos de bordo
/// contam) e projectado na **poligonal** da referência, nunca na superfície. Ela entrega o
/// perímetro do buraco **exacto** — `0,6046 ⇒ 0,6046` na `sculpt_t002`, contra
/// `0,6046 ⇒ 0,7841` (**+30 %**) sem ela — e tem gate e provas de mutação.
///
/// # ⛔⛔ E o produto fica PIOR, em todas as colunas
///
/// | `sculpt_t002` | perímetro | `χ` · não-manif. | enviesamento p50 · >60° | aspecto p99 · >4× |
/// |---|---|---|---|---|
/// | ⭐ **sem a lei** | `0,7841` (+30 %) | **`1` · `0`** | ⭐ **`7,1°` · `1`** | ⭐ **`1,72` · `0`** |
/// | ⛔ com a lei | ⭐ `0,6046` exacto | `1` · ⛔ `1` | ⛔ `9,9°` · `22` | ⛔ `3,25` · `13` |
///
/// | `sculpt_punctured` | enviesamento p50 · >60° | aspecto p99 · >4× |
/// |---|---|---|
/// | ⭐ **sem a lei** | ⭐ **`5,7°` · `1`** | ⭐ **`1,67` · `0`** |
/// | ⛔ com a lei | ⛔ **`24,3°` · `72`** | ⛔ **`14,49` · `87`** |
///
/// # ⭐⭐⭐ O mecanismo, e ele reprecifica um «defeito»
///
/// **O rebordo de um buraco esculpido SERRILHA** — viragem média `43,7°` na `punctured` e
/// `53,6°` na `t002`, contra os `10°` de um círculo de 36 lados. Preservá-lo **exactamente**
/// preserva o serrilhado, e o bordo é uma **linha de feição**: o campo cruzado passa a ser
/// forçado a segui-lo, e os patches saem do que essa zig-zag pede.
///
/// ⇒ ⭐⭐ *O «defeito» de ontem — a Laplaciana interior a arrastar o rebordo, `+30 %` — estava
/// a pagar por uma coisa que ninguém tinha precificado: **um rebordo LISO**. Tirar o defeito
/// tirou o pagamento.* ⚠️ **A cura seguinte não é esta afinada:** é alisar o rebordo **e**
/// repor o comprimento dele, que é outra operação.
///
/// ⚠️ `PH2D_BORDER_LAW=1` liga-a, para reabrir a experiência sem recompilar.
pub(crate) const BORDER_LAW: bool = false;

pub(crate) fn border_law_on() -> bool {
    std::env::var("PH2D_BORDER_LAW").map_or(BORDER_LAW, |v| v == "1")
}

/// ⭐⭐⭐ **ZERO — quando a [`BORDER_LAW`] corre, o rebordo NÃO é alisado.**
///
/// ⛔⛔ **Alisar uma poligonal ENCURTA-A por construção** (é fluxo de encurtamento de
/// curva), e um rebordo é o buraco que o artista fez. Medido 2026-08-26, com o perímetro
/// **verdadeiro** do buraco como régua — ⚠️ nunca a contagem de arestas de bordo, que é
/// função do passo:
///
/// | `λ` | `sculpt_punctured` (entrada `5,6463`) | `sculpt_t002` (entrada `0,6046`) |
/// |---|---|---|
/// | ⭐ **`0,0`** | ⭐ **`5,6463`** — exacto | ⭐ **`0,6046`** — exacto |
/// | `0,1` | `5,4124` (⛔ −4,1 %) | `0,5387` (⛔ −10,9 %) |
/// | `0,3` | `5,2763` (⛔ −6,6 %) | `0,5222` (⛔ −13,6 %) |
/// | `0,5` | `5,2566` (⛔ −6,9 %) | `0,5199` (⛔ −14,0 %) |
///
/// ⚠️ **E sem a lei nenhuma o erro é do outro lado:** a `t002` ia a `0,7841`, **+30 %** — a
/// Laplaciana **interior** puxa o rebordo para dentro e a projecção de **superfície**
/// deixa-o deslizar para onde a peça é mais larga. *O sinal do erro muda com a lei; a
/// magnitude não desaparece sozinha.*
///
/// ⭐ **Com `λ = 0` o rebordo é reamostrado na mesma** (`sculpt_punctured`: `38 → 104`
/// arestas) porque as divisões caem **sobre** a poligonal. *Refinado sem distorcer.*
///
/// `PH2D_BORDER_LAMBDA` sobrepõe-se, para reabrir a varredura sem recompilar.
const BORDER_LAMBDA: f32 = 0.0;

pub(crate) fn border_lambda() -> f32 {
    std::env::var("PH2D_BORDER_LAMBDA")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(BORDER_LAMBDA)
}

/// ⭐⭐⭐ **A POLIGONAL DO BORDO** de uma malha — os segmentos das arestas com **uma** face.
///
/// ⚠️ Vazia numa peça fechada, e é isso que torna toda a lei do bordo **inerte** ali.
pub(crate) fn border_polyline(mesh: &Mesh) -> Vec<([f32; 3], [f32; 3])> {
    let mut n: std::collections::BTreeMap<(u32, u32), usize> = std::collections::BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            *n.entry(if a < b { (a, b) } else { (b, a) }).or_default() += 1;
        }
    }
    let pos = mesh.positions();
    n.into_iter()
        .filter(|(_, c)| *c == 1)
        .map(|((a, b), _)| (pos[a as usize], pos[b as usize]))
        .collect()
}

/// **O ponto mais próximo numa poligonal.** ⚠️ Nos SEGMENTOS, não nos vértices dela: o
/// rebordo remalhado tem mais pontos que o original, e encaixá-los todos nos vértices de
/// referência apertaria a curva em nós.
pub(crate) fn project_onto_polyline(segs: &[([f32; 3], [f32; 3])], p: [f32; 3]) -> [f32; 3] {
    let mut best = (f32::INFINITY, p);
    for &(a, b) in segs {
        let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ll = dot(d, d);
        let t = if ll > 1.0e-20 {
            (dot([p[0] - a[0], p[1] - a[1], p[2] - a[2]], d) / ll).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let q = [
            t.mul_add(d[0], a[0]),
            t.mul_add(d[1], a[1]),
            t.mul_add(d[2], a[2]),
        ];
        let e = [p[0] - q[0], p[1] - q[1], p[2] - q[2]];
        let dist = dot(e, e);
        if dist < best.0 {
            best = (dist, q);
        }
    }
    best.1
}
