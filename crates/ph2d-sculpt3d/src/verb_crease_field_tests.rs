//! **O VINCO** — o único verbo COMPOSTO da tabela do §3, e por isso o único
//! cujo gate tem de olhar as duas metades separadas.
//!
//! ⚠️ **O corte deste arquivo é o do PAPER, não o do tamanho** (o irmão
//! [`super::verb_field`] serve os cinco verbos cujo deslocamento vem INTEIRO do
//! kernel). O Crease é `Draw + Kelvinlets pinch`: metade dele é um afundamento
//! pela normal, e a lei *"com campo, a curva é o SUPORTE do campo"* — que serve
//! os cinco — alcançaria também essa metade, que não é do campo. O que sai
//! disso está medido no gate abaixo, e não é sutil: o vinco vira cratera.

use crate::{RefMode, Verb};

use super::verb_field::{R, TIP, gesture, sphere, stroke};

/// **O VINCO CONTINUA ESTREITO ENQUANTO O APERTO ALCANÇA** — a forma do único
/// verbo COMPOSTO da tabela, e a única em que a lei do suporte precisa de um
/// segundo olhar.
///
/// ⚠️ **Os cinco verbos da W5-B têm o deslocamento inteiro vindo do kernel**, e
/// para eles *"com campo, a curva é o SUPORTE do campo"* é a lei toda. O Crease
/// é `Draw + pinch`: metade dele é um AFUNDAMENTO pela normal, que a mesma lei
/// alcançaria sem ter sido escrita para ela. Medido
/// (`tests/measure_field_verbs.rs::measure_the_crease_trench`):
///
/// | banda | s-mode | campo INGÊNUO | composto |
/// |---|---|---|---|
/// | 0,25-0,50 r | 0,08594 | 0,18890 | 0,06751 |
/// | 1,00-1,50 r | 0,00000 | **0,15410** | −0,00077 |
///
/// ⇒ com a indicadora no lugar da quártica o vinco vira **CRATERA** — 82 % do
/// bico a um raio e meio. A cura é a metade estreita tomar a estreiteza do
/// perfil do próprio Kelvinlet, e é isso que a 1ª metade deste gate pina.
///
/// ⚠️ **E a 2ª metade é o que o `l-mode` COMPRA — mas o ALCANCE não serve de
/// oráculo, e a mutação provou-o.** A 1ª versão pedia só *"o aperto passa do
/// anel"*, e trocar o kernel elástico pelo `lateral_pull` do `s-mode`
/// **PASSOU**: com um campo declarado a pegada já é 3× e o `w` já é a
/// indicadora, então o puxão cru alcança 2 raios sozinho. É a §7.12 outra vez —
/// *um `l-mode` com o alcance e sem a lei* —, desta vez no meu próprio gate.
///
/// ⇒ O que a pegada não finge é o **PERFIL**, e as duas leis são opostas nele:
///
/// | banda | `lateral_pull` × indicadora | Kelvinlet |
/// |---|---|---|
/// | 0,50-0,75 r | 0,05708 | 0,01730 |
/// | 1,50-2,00 r | **0,15174** | **0,00185** |
///
/// O puxão cru é o delta ao centro, logo **CRESCE** com a distância até cair de
/// um penhasco na borda da pegada; o campo **DECAI**. A razão longe÷perto vale
/// `2,66` num e `0,11` no outro.
#[test]
fn the_creased_trench_stays_narrow_while_the_squeeze_reaches_out() {
    /// A média, numa banda, de quanto o vértice AFUNDOU (`−z`, a normal do
    /// polo) e de quanto ele andou DE LADO (a componente no plano).
    fn halves(rest: &ph2d_mesh::Mesh, out: &ph2d_mesh::Mesh, lo: f32, hi: f32) -> (f32, f32) {
        let (a, b) = (rest.positions(), out.positions());
        let (mut dig, mut lat, mut n) = (0.0f32, 0.0f32, 0usize);
        for i in 0..a.len() {
            let r = [a[i][0] - TIP[0], a[i][1] - TIP[1], a[i][2] - TIP[2]];
            let rm = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
            if rm < lo * R || rm >= hi * R {
                continue;
            }
            let d = [b[i][0] - a[i][0], b[i][1] - a[i][1], b[i][2] - a[i][2]];
            dig += -d[2];
            lat += (d[0] * d[0] + d[1] * d[1]).sqrt();
            n += 1;
        }
        if n == 0 {
            (0.0, 0.0)
        } else {
            (dig / n as f32, lat / n as f32)
        }
    }

    let rest = sphere();
    let (amount, pull) = gesture(Verb::Crease);
    let s = stroke(Verb::Crease, RefMode::S, amount, pull);
    let l = stroke(Verb::Crease, RefMode::L, amount, pull);

    let (s_tip, s_far) = (halves(&rest, &s, 0.25, 0.5), halves(&rest, &s, 1.5, 2.0));
    let (l_tip, l_far) = (halves(&rest, &l, 0.25, 0.5), halves(&rest, &l, 1.5, 2.0));
    // O PICO do aperto elástico não é no bico — a `F` de traço zero anula-se em
    // `r = 0` —, então a razão do perfil mede contra a banda onde ele é forte.
    let l_near = halves(&rest, &l, 0.5, 0.75);

    // CONTROLE — a fixture contém o fenômeno: o s-mode de fato cava.
    assert!(
        s_tip.0 > 0.02,
        "CONTROLE VAZIO: o s-mode não cavou nada no bico ({:.5})",
        s_tip.0
    );

    // 1ª metade — o canal é ESTREITO. O `s-mode` mede exatamente 0,00000 longe;
    // a barra admite o pequeno negativo do rebote elástico, e recusa a cratera.
    let ratio = l_far.0 / l_tip.0;
    assert!(
        ratio < 0.15,
        "o vinco virou CRATERA: afunda {:.5} a 1,5-2 r contra {:.5} no bico \
         ({:.0} %) — a metade estreita perdeu a quártica do perfil e ficou com \
         a curva-indicadora do suporte",
        l_far.0,
        l_tip.0,
        ratio * 100.0
    );

    // 2ª metade — o que o modo COMPRA: o aperto passa do anel do cursor.
    assert!(
        s_far.1 < 1e-6,
        "CONTROLE: o s-mode não devia apertar nada além do anel ({:.5})",
        s_far.1
    );
    assert!(
        l_far.1 > 1e-3,
        "o aperto elástico não alcança além do anel ({:.5} a 1,5-2 r) — sem \
         isso o `l-mode` do Crease é um vinco mais fraco e nada mais",
        l_far.1
    );

    // 3ª metade — e é ela que a pegada não finge: o aperto DECAI do perto para
    // o longe. O `lateral_pull` sob a mesma pegada faz o contrário (cresce, e a
    // razão passa de 2,6), e sem esta linha a mutação que o reinstala PASSA.
    let profile = l_far.1 / l_near.1;
    assert!(
        profile < 0.5,
        "o aperto NÃO tem perfil: {:.5} a 1,5-2 r contra {:.5} a 0,5-0,75 r \
         (razão {:.2}) — isto é o delta CRU ao centro a crescer com a distância \
         sob a indicadora do suporte, não o kernel elástico",
        l_far.1,
        l_near.1,
        profile
    );
}
