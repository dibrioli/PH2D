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

/// **O alvo da mistura de uma lavagem sobre TINTA SECA** — a cor para onde o composite puxa o pixel,
/// com o peso `max(pelo_botao, pela_agua)` que o chamador já aplica.
///
/// São DOIS termos com DUAS leis, e o K–M manda só na parte da mistura que o botão pede ALÉM da
/// água (`(pelo_botao − pela_agua)⁺ / pelo_botao`):
///
/// * **o botão `Pigment`** (`pelo_botao = pigment_mix × presença de tinta`) mistura duas TINTAS que o
///   artista escolheu, e a lei é a do Wet Paint e do Digital (`ph2d_pigment::mix_unit`, Kubelka–Munk)
///   — ordem do dono de 2026-09-20 (*«os três meios passam a misturar igual»*). Medido na
///   `diag_pigment_molhado_sobre_molhado`, amarelo sobre azul: SECO `119,209,228` (um CIANO) com a lei
///   antiga e `128,173,139` com esta, contra `159,198,159` molhado — *é esta troca que faz o seco e o
///   molhado darem a mesma família de tom*;
/// * **a água que molha a tinta seca** (`pela_agua = wet × tinta molhada`) continua na lei antiga
///   (RYB, Gossett & Chen). ⛔ Não por inércia: o pigmento dela é o que a própria água DISSOLVEU da
///   base, e numa mistura de Kubelka–Munk o parceiro mais absorvente domina — o clarear do *soak*
///   desaparece (`watercolor_soak_deepens_and_widens_the_dissolve_while_parked` lê o MESMO pixel,
///   `228,23,23`, para 2 s de demora e para a passagem rápida; medido 2026-09-20 e outra vez
///   2026-09-29). Esse clarear é o *«segredo do wet-on-wet»* que o dono aprovou em 2026-07-06.
///
/// ⚠️ **Onde a água já mistura tanto quanto o botão pede, o botão não muda nada** — é a lei do
/// `watercolor_wet_drives_the_paint_mix_without_pigment` (dono, 2026-07-06: *a mistura da água não
/// fica presa atrás da caixa*), e com `wet = 1` a saída é a de botão desligado AO BIT. Logo:
/// * botão desligado ⇒ RYB **ao bit** (o K–M nem é calculado) — todo pixel sem `Pigment` sai igual ao
///   de antes desta lei;
/// * água a zero ⇒ K–M **ao bit**;
/// * entre os dois a partilha é contínua (em `pelo_botao = pela_agua` ela é zero dos dois lados).
pub(super) fn alvo_sobre_seco(
    base: [f32; 3],
    pigmento: [f32; 3],
    film_a: f32,
    pelo_botao: f32,
    pela_agua: f32,
) -> [f32; 3] {
    // Negativa quando a água passa o botão — e aí o `<= 0` abaixo devolve o RYB (um `.max(0.0)` aqui
    // seria uma linha que nenhuma mutação consegue matar: medido, 2026-09-29).
    let km_share = if pelo_botao > 0.0 {
        (pelo_botao - pela_agua) / pelo_botao
    } else {
        0.0
    };
    if km_share >= 1.0 {
        return ph2d_pigment::mix_unit(base, pigmento, film_a);
    }
    let ryb = ph2d_painter_brush::blend::ryb_mix(base, pigmento, film_a);
    if km_share <= 0.0 {
        return ryb;
    }
    let km = ph2d_pigment::mix_unit(base, pigmento, film_a);
    [
        ryb[0] + (km[0] - ryb[0]) * km_share,
        ryb[1] + (km[1] - ryb[1]) * km_share,
        ryb[2] + (km[2] - ryb[2]) * km_share,
    ]
}

/// Os dois planos da mistura molhada (RGBA, `w*h*4` cada), vivos só enquanto o `Pigment` (ou o
/// Smudge sobre tinta molhada) está ligado. `proprio` recomeça a zero em cada traço; `antes` é escrito
/// na CAPTURA de cada texel, logo nunca precisa de ser limpo.
#[derive(Default)]
pub(super) struct PlanosDaMistura {
    pub(super) antes: Vec<u8>,
    pub(super) proprio: Vec<u8>,
    /// **O `antes` deste texel já é deste traço?** Recomeça a `false` em cada traço.
    ///
    /// ⚠️ Era o alfa do `proprio` (`== 0` ⇒ primeiro toque), e com o Smudge isso deixou de bastar: o
    /// arrasto escreve no `antes` de texels onde este traço ainda não depositou nada (a frente do dab,
    /// e a orla que o `feather` deixa a zero), e o primeiro depósito a seguir voltava a fotografar o
    /// plano da sessão POR CIMA do arrasto — a tinta arrastada desaparecia exactamente onde chegava.
    /// Sem Smudge a captura só acontece no depósito, logo a flag e o alfa marcam os mesmos texels.
    pub(super) capturado: Vec<bool>,
    /// **Há tinta MOLHADA de traços anteriores nesta sessão?** Escrito no pen-down (é o
    /// `wet_session_continues` de lá). É a porta do Smudge sobre tinta molhada
    /// ([`super::watercolor_smudge`]): o 1.º traço de uma sessão não tem o que arrastar (o `antes`
    /// dele é papel), e sem ela passava na mesma pela rota de depósito da mistura, que ignora a
    /// prioridade do mixer — com o `Charge < 1` esse traço mudava só por o Smudge estar ligado.
    pub(super) ha_tinta_da_sessao: bool,
    /// A cadeia do Smudge sobre a COR da sessão: o centro do último dab ORIGINAL (nunca uma cópia de
    /// Tiling), recomeçada a cada traço.
    pub(super) arrasto: Option<[f32; 2]>,
}

impl PlanosDaMistura {
    /// Garante o tamanho `n` texels; devolve `true` se os planos acabaram de nascer (a zero).
    pub(super) fn garante(&mut self, n: usize) -> bool {
        if self.proprio.len() == n * 4 && self.antes.len() == n * 4 && self.capturado.len() == n {
            return false;
        }
        self.antes = vec![0; n * 4];
        self.proprio = vec![0; n * 4];
        self.capturado = vec![false; n];
        true
    }

    /// Um traço novo começa: nada deste traço foi depositado ainda, e a cadeia do arrasto recomeça.
    pub(super) fn novo_traco(&mut self) {
        self.proprio.iter_mut().for_each(|b| *b = 0);
        self.capturado.iter_mut().for_each(|c| *c = false);
        self.arrasto = None;
    }

    /// Fotografa no `antes` o plano da sessão `buf` em todo texel do rectângulo `[x0,x1)×[y0,y1)`
    /// (linhas de `fw` texels) que este traço ainda não capturou. O Smudge chama-a ANTES de arrastar o
    /// `antes` ([`super::watercolor_mistura_arrasto`]).
    pub(super) fn captura(&mut self, buf: &[u8], fw: usize, [x0, y0, x1, y1]: [usize; 4]) {
        for y in y0..y1 {
            for x in x0..x1 {
                let i = y * fw + x;
                if !self.capturado[i] {
                    self.capturado[i] = true;
                    self.antes[i * 4..i * 4 + 4].copy_from_slice(&buf[i * 4..i * 4 + 4]);
                }
            }
        }
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
/// `buf`, misturando com a tinta que a sessão lá tinha ANTES deste traço.
///
/// Os dois pesos são os do composite sobre tinta SECA, e passam pela MESMA porta
/// ([`alvo_sobre_seco`]): `pelo_botao` é tinta a misturar com tinta (o `Pigment`, e o Smudge que
/// empurra uma para dentro da outra) e vai pelo Kubelka–Munk; `pela_agua` é a água do Rewet a
/// redissolver a tinta molhada de baixo e vai pela lei da água. Com `pela_agua = 0` a mistura é o
/// Kubelka–Munk ao bit, como era antes de a água entrar aqui.
pub(super) fn deposita(
    buf: &mut [u8],
    planos: &mut PlanosDaMistura,
    idx: usize,
    col: [u8; 3],
    a: f32,
    (pelo_botao, pela_agua): (f32, f32),
    parceiro: Option<[u8; 4]>,
) {
    let mistura = pelo_botao.max(pela_agua);
    let px = idx..idx + 4;
    if !planos.capturado[idx / 4] {
        // O primeiro toque DESTE traço: o que o plano tem agora é a tinta das pinceladas anteriores.
        planos.capturado[idx / 4] = true;
        planos.antes[px.clone()].copy_from_slice(&buf[px.clone()]);
    }
    over(&mut planos.proprio[px.clone()], col, a);
    // O parceiro da mistura: a tinta que estava neste texel, ou — com o Rewet — a da vizinhança
    // dissolvida na água ([`super::watercolor_mistura_agua`]).
    let antes = parceiro.unwrap_or([
        planos.antes[idx],
        planos.antes[idx + 1],
        planos.antes[idx + 2],
        planos.antes[idx + 3],
    ]);
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
    let misturado = alvo_sobre_seco(unit(antes), unit(proprio), t, pelo_botao, pela_agua);
    for c in 0..3 {
        let o = f32::from(out[c]);
        let m = misturado[c].clamp(0.0, 1.0) * 255.0;
        out[c] = (o + (m - o) * mistura + 0.5).clamp(0.0, 255.0) as u8;
    }
    buf[px].copy_from_slice(&out);
}

impl super::PainterTool {
    /// **O Rewet sobre a tinta MOLHADA da sessão** — `Some(água)` quando este traço redissolve a
    /// tinta que os traços anteriores da sessão deixaram molhada (report do dono, 2026-09-29: *«o
    /// Rewet não afeta a mancha molhada»*; medido `2,6×` mais fraco que sobre a seca, doc 44 §3).
    ///
    /// O Rewet do composite lê a base da sessão — a tinta SECA de baixo —, e de propósito: a tinta
    /// molhada vizinha ali seria contada duas vezes (a base refeita com ela re-renderizava-a como um
    /// rectângulo que clareia, Enio 2026-07-09). A tinta molhada vive no plano da cor da sessão, e é
    /// lá que a água a encontra: o depósito deste traço mistura com ela pela lei da água
    /// ([`deposita`]), com o peso do knob — o `st_wet` do composite, que dentro do traço é ele.
    ///
    /// ⚠️ Só com tinta de traços ANTERIORES (`ha_tinta_da_sessao`), pela mesma razão do Smudge
    /// ([`super::PlanosDaMistura::ha_tinta_da_sessao`]).
    pub(super) fn agua_da_sessao(&self) -> Option<f32> {
        let agua = self.paint.brush.wet_rewet.clamp(0.0, 1.0);
        (self.paint.wet_mistura.ha_tinta_da_sessao && agua > 0.0).then_some(agua)
    }
}
