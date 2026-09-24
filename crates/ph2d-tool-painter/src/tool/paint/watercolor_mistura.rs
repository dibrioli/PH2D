//! **O `Pigment` da aquarela mistura TINTA com TINTA — nunca com o papel, e também molhado sobre
//! molhado** (ordem do dono, 2026-09-24: *«a capacidade de mixing pigment no modo aquarela molhado
//! (seco já funciona)»*).
//!
//! ## As duas metades do defeito, medidas (`diag_pigment_molhado_sobre_molhado`)
//!
//! 1. **O termo do composite misturava com o PAPEL.** O parceiro da mistura é a base congelada, e
//!    sobre papel virgem a base É o papel: com o botão ligado um amarelo sozinho sobre papel lia
//!    `254,252,235` (quase branco) contra `253,245,140` desligado. *O papel não é pigmento.* ⇒ o peso
//!    do `Pigment` passa a ser multiplicado pela [`presenca_de_tinta`] da base — a MESMA régua que o
//!    rewet já usa (`build_rewet_fields`), agora escrita UMA vez aqui. Sobre papel ela é `0` e o
//!    composite sai byte a byte o de botão desligado; sobre tinta seca é `~1` e nada muda.
//! 2. **Molhado sobre molhado o vizinho NÃO está na base** (ele é da mesma sessão), logo o termo do
//!    composite nunca o via. Quem o vê é o DEPÓSITO da cor, que escreve sobre o plano da sessão — e
//!    ali o `over` punha a cor nova por cima: no meio da sobreposição o amarelo substituía o azul.
//!
//! ## A lei do depósito
//!
//! O plano da sessão guarda, por texel, a cor que lá estava ANTES deste traço (`antes`, fotografada no
//! primeiro toque do traço) e a cor que ESTE traço depositou sozinho (`proprio`, o mesmo `over` de
//! sempre). A cor da sessão é a combinação das duas:
//!
//! * alfa — o `over` de sempre (`proprio` sobre `antes`);
//! * cor — o `over` de sempre, puxado para a MISTURA de pigmentos (`ph2d_pigment::mix_unit`, a lei do
//!   Wet Paint e do Digital) pelo peso do `Pigment`, com a fracção do traço novo a ser
//!   `aₚ / (aₚ + aₐ)` — a quantidade de tinta de cada um.
//!
//! ⭐ **É por isto que um traço que repassa o mesmo texel não lava o vizinho:** a fracção depende do
//! alfa do PRÓPRIO traço, que satura em `1`, e nunca de quantos dabs passaram. As duas curas
//! construídas e revertidas em 2026-09-20 (handoff §17.4) falhavam exactamente aí — cada dab voltava a
//! misturar e `~20` dabs amarelos lavavam o azul.
//!
//! ⚠️ **Sem tinta anterior da sessão (`aₐ = 0`) a cor é o `proprio` AO BIT**, logo todo traço de uma
//! sessão nova sai igual ao de antes desta lei; e com o `Pigment` a zero esta rota nem corre.

use super::watercolor_field::smoothstep;

/// **Quanta TINTA há neste pixel da base** (`0` = papel, `1` = tinta franca): o quanto a base sobre o
/// chão LOCAL se afasta desse chão — só a tinta da camada activa difere dele, logo a referência é
/// verdadeira por pixel, pigmentos claros incluídos. Zona morta `14..50` em bytes, para migalhas de
/// anti-serrilhado não contarem como tinta (wet_edges `PAINT_LO`/`PAINT_HI`).
///
/// Devolve também a base sobre o chão em bytes sRGB (o rewet borra-a pesada pela presença).
/// ⚠️ **Uma porta, dois leitores** — o campo do rewet e o termo `Pigment` do composite: escrita duas
/// vezes, a régua do «isto é tinta?» divergiria entre as duas coisas que a perguntam.
#[inline]
pub(super) fn presenca_de_tinta(base: &[u8], ground: &[u8], bi: usize) -> (f32, [f32; 3]) {
    let ab = f32::from(base[bi + 3]) / 255.0;
    let chao = [
        f32::from(ground[bi]),
        f32::from(ground[bi + 1]),
        f32::from(ground[bi + 2]),
    ];
    let visto = [
        f32::from(base[bi]) * ab + chao[0] * (1.0 - ab),
        f32::from(base[bi + 1]) * ab + chao[1] * (1.0 - ab),
        f32::from(base[bi + 2]) * ab + chao[2] * (1.0 - ab),
    ];
    let d = (chao[0] - visto[0])
        .abs()
        .max((chao[1] - visto[1]).abs())
        .max((chao[2] - visto[2]).abs());
    (smoothstep(14.0, 50.0, d), visto) // LITERAL-PX-OK: wet_edges PAINT_LO/PAINT_HI
}

/// Os dois planos da mistura molhada (RGBA, `w*h*4` cada), vivos só enquanto o `Pigment` está ligado.
/// `proprio` recomeça a zero em cada traço; `antes` é escrito no primeiro toque, logo nunca precisa
/// de ser limpo.
#[derive(Default)]
pub(super) struct PlanosDaMistura {
    pub(super) antes: Vec<u8>,
    pub(super) proprio: Vec<u8>,
}

impl PlanosDaMistura {
    /// Garante o tamanho `n` texels; devolve `true` se os planos acabaram de nascer (a zero).
    pub(super) fn garante(&mut self, n: usize) -> bool {
        if self.proprio.len() == n * 4 && self.antes.len() == n * 4 {
            return false;
        }
        self.antes = vec![0; n * 4];
        self.proprio = vec![0; n * 4];
        true
    }

    /// Um traço novo começa: nada deste traço foi depositado ainda.
    pub(super) fn novo_traco(&mut self) {
        self.proprio.iter_mut().for_each(|b| *b = 0);
    }
}

/// O `over` de alfa recto de sempre (a fórmula do depósito, byte a byte).
#[inline]
fn over(dst: &mut [u8], col: [u8; 3], a: f32) {
    let da = f32::from(dst[3]) / 255.0;
    let na = a + da * (1.0 - a);
    if na <= 0.0 {
        return;
    }
    for c in 0..3 {
        let dc = f32::from(dst[c]) / 255.0;
        let sc = f32::from(col[c]) / 255.0;
        let out = (sc * a + dc * da * (1.0 - a)) / na;
        dst[c] = (out * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
    }
    dst[3] = (na * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
}

/// Deposita um toque de alfa `a` e cor `col` no texel `idx` (índice de byte) do plano da sessão
/// `buf`, misturando com a tinta que a sessão lá tinha ANTES deste traço pelo peso `mistura`.
pub(super) fn deposita(
    buf: &mut [u8],
    planos: &mut PlanosDaMistura,
    idx: usize,
    col: [u8; 3],
    a: f32,
    mistura: f32,
) {
    let px = idx..idx + 4;
    if planos.proprio[idx + 3] == 0 {
        // O primeiro toque DESTE traço: o que o plano tem agora é a tinta das pinceladas anteriores.
        planos.antes[px.clone()].copy_from_slice(&buf[px.clone()]);
    }
    over(&mut planos.proprio[px.clone()], col, a);
    let antes = [
        planos.antes[idx],
        planos.antes[idx + 1],
        planos.antes[idx + 2],
        planos.antes[idx + 3],
    ];
    let proprio = [
        planos.proprio[idx],
        planos.proprio[idx + 1],
        planos.proprio[idx + 2],
        planos.proprio[idx + 3],
    ];
    if antes[3] == 0 {
        buf[px].copy_from_slice(&proprio);
        return;
    }
    // O `over` de sempre, do traço inteiro sobre o que havia antes dele.
    let mut out = antes;
    over(
        &mut out,
        [proprio[0], proprio[1], proprio[2]],
        f32::from(proprio[3]) / 255.0,
    );
    let (ap, aa) = (f32::from(proprio[3]), f32::from(antes[3]));
    let t = ap / (ap + aa);
    let unit = |p: [u8; 4]| {
        [
            f32::from(p[0]) / 255.0,
            f32::from(p[1]) / 255.0,
            f32::from(p[2]) / 255.0,
        ]
    };
    let misturado = ph2d_pigment::mix_unit(unit(antes), unit(proprio), t);
    for c in 0..3 {
        let o = f32::from(out[c]);
        let m = misturado[c].clamp(0.0, 1.0) * 255.0;
        out[c] = (o + (m - o) * mistura + 0.5).clamp(0.0, 255.0) as u8;
    }
    buf[px].copy_from_slice(&out);
}
