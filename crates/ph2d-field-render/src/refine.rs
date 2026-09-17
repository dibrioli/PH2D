//! ⭐⭐⭐ **O REFINAMENTO DO QUADRO ASSENTE — as DUAS metades do hemisfério, uma direcção de cada
//! vez** (`docs/Render3d/08` §7).
//!
//! A luz que chega a um ponto pelo hemisfério tem duas parcelas: a do **céu**, que a
//! [`crate::occlusion`] mede e que **atenua**, e a das **superfícies**, que o [`crate::bounce`]
//! mede e que **soma**. As duas saem do mesmo conjunto de direcções ([`crate::occlusion::cone_dir`])
//! e do mesmo peso `max(0, n·d)`.
//!
//! # ⚠️⚠️ E é por isso que elas avançam com o MESMO `k`, num laço só
//!
//! Uma publicação em que o céu já consumiu `k` direcções e o ricochete outras tantas **diferentes**
//! somaria dois hemisférios distintos. *A partilha do conjunto de direcções só é uma lei enquanto
//! as duas metades a percorrerem no mesmo passo* — e escrever dois laços seria a forma mais fácil
//! de as deixar divergir sem ninguém dar por isso.
//!
//! # ⏱️ O que uma passagem custa (medido 2026-09-17, `--release`, CPU a `95 %` ociosa, mínimo de 3)
//!
//! | cena | UMA passagem: céu | ricochete | razão |
//! |---|---:|---:|---:|
//! | caixa fechada `640×360` | `18,23 ms` | `28,62` | `1,6×` |
//! | caixa fechada `1920×1080` | `186,33` | `270,60` | `1,5×` |
//! | peça no aberto `640×360` | `0,40` | `1,63` | `4,0×` |
//! | **peça no aberto `1920×1080`** | **`4,88`** | **`19,06`** | **`3,9×`** |
//!
//! ⭐ **É esta tabela que autoriza o mesmo `k`.** A sonda do preço
//! ([`crate::tests::cornell::sonda_o_preco_do_ricochete`]) mediu a sequência INTEIRA e ela não cabe
//! num quadro (`79 ms` a `64` direcções no aberto); o que o refinamento paga é **uma**, e ela custa
//! `1,5`–`4` vezes o que a metade do céu já paga hoje na mesma passagem. ⇒ no caso do modelador, o
//! quadro assente passa de `~0,23 s` para `~1,15 s` de refinamento, publicando `48` vezes pelo
//! caminho e cancelável em cada uma.
//!
//! ⚠️ **A régua é a PASSAGEM e não o total**: o que o artista sente é o intervalo entre duas
//! imagens, e a granularidade do cancelamento é uma passagem.
//!
//! # ⏳ O que ele NÃO refina, e está declarado
//!
//! O **chão invisível** da [`crate::ground`] recebe o céu dele pelo passe da sombra e **não recebe
//! ricochete nenhum**: os pixels que mostram o chão falham a peça (`g.hit == false`), logo não
//! entram em fatia nenhuma. *A cor que a peça devolveria ao chão à volta dela é a wave seguinte, e
//! a marcha dela é outra — o ponto de partida está no plano, não no campo.*

use crate::occlusion::{OCCLUSION_PASSES, blur_occlusion, occlusion_slice};
use crate::{ConeSlice, Gbuffer, Orbit, PointLamp, Shadows, Surfaces};
use ph2d_field::FieldDoc;
use ph2d_field_eval::hybrid::Registry;

/// ⭐⭐⭐ **`OCCLUSION_PASSES` passagens sobre o MESMO G-buffer**, cada uma com uma direcção a mais
/// nas duas metades.
///
/// Cada passagem publica as médias acumuladas por `entrega`, e pára assim que ela devolver `false`
/// — que é como a mão a voltar a mexer cancela o trabalho.
///
/// ⚠️⚠️ **Ela existe como PORTA, e não como laço dentro da thread do traçado, por causa do gate.**
/// O laço vivia no `std::thread::spawn` do `smoke_draw`, onde nenhum teste lhe chega: a lei da
/// acumulação, a ordem das passagens e a paragem por cancelamento ficavam todas **inalcançáveis**.
/// *A costura não-testada é a causa nº 1 da `DIRETIVA_IMPLEMENTACAO` §1, e um laço dentro de uma
/// thread é a forma mais fácil de a produzir sem dar por isso.*
///
/// ⭐ **Sem lâmpadas ou sem materiais o ricochete degenera para o canal vazio, e o quadro fica o de
/// sempre AO BIT** — é isso que faz esta porta não precisar de um interruptor ao lado.
///
/// Devolve quantas passagens correram — `< OCCLUSION_PASSES` quer dizer que foi cancelado.
#[allow(clippy::too_many_arguments)] // a peça, a vista, os materiais, as luzes, os canais e a entrega
pub fn refine_hemisphere(
    doc: &FieldDoc,
    reg: &Registry,
    cam: &Orbit,
    g: &Gbuffer,
    surfaces: &Surfaces<'_>,
    lampadas: &[PointLamp],
    shadows: &mut Shadows,
    mut entrega: impl FnMut(&Shadows, u32) -> bool,
) -> u32 {
    let pixels = g.hit.len();
    let mut ceu = ConeSlice {
        sum: vec![0.0f32; pixels],
        weight: vec![0.0f32; pixels],
    };
    let mut devolvida = crate::bounce::BounceSlice::empty(pixels);
    // ⭐⭐ **O CHÃO já tem o céu dele**, e não pelos cones: o passe da sombra calculou-o
    // ([`crate::ground::ground_sky`]), e aqui ele só atravessa cada publicação. ⚠️ Os cones num chão
    // plano desenham ANÉIS — as `48` direcções fixas viram `24` sombras fracas sobrepostas (medido,
    // `docs/Render3d/07`).
    let chao: Vec<(usize, f32)> = crate::ground::ground_points(cam, g, shadows.ground())
        .iter()
        .enumerate()
        .filter(|(_, q)| q.is_some())
        .map(|(i, _)| (i, shadows.ambient_at(i)))
        .collect();
    for k in 0..OCCLUSION_PASSES {
        ceu.add(&occlusion_slice(doc, reg, cam, g, k, 1, OCCLUSION_PASSES));
        devolvida.add(&crate::bounce::bounce_slice(
            doc,
            reg,
            cam,
            g,
            surfaces,
            lampadas,
            k,
            1,
            OCCLUSION_PASSES,
        ));
        // ⚠️ **A média é sobre o PESO já acumulado**, e não sobre o total nem sobre a contagem de
        // passagens — senão a imagem mudaria de nível a cada passo em vez de afinar. Ver
        // [`ConeSlice`], que é onde essa lei vive.
        let mut cru = ceu.average(&g.hit);
        for &(i, v) in &chao {
            cru[i] = v;
        }
        // ⭐ **Suavizadas no PUBLICAR, e não no acumulador** — as somas têm de continuar cruas,
        // senão cada passagem borraria o que a anterior já borrou e os canais espalhar-se-iam.
        shadows.set_ambient(blur_occlusion(g, &cru));
        shadows.set_bounce(crate::bounce::blur_bounce(g, &devolvida.average(&g.hit)));
        if !entrega(shadows, k + 1) {
            return k + 1;
        }
    }
    OCCLUSION_PASSES
}
