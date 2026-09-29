//! **O `Pigment` da aquarela mistura tinta com tinta — molhado sobre molhado, e nunca com o papel**
//! (ordem do dono, 2026-09-24). A lei e as duas metades do defeito: [`super::super::watercolor_mistura`].
//!
//! A fixtura é a do instrumento `diag_pigment_molhado_sobre_molhado` (azul molhado, depois amarelo
//! que o SOBREPÕE na mesma sessão), agora com barras. Medido antes da cura, `Pigment` ligado: o meio
//! da sobreposição lia `254,252,235` (quase branco) e o amarelo SOZINHO sobre papel também — o botão
//! misturava com o papel. Depois: o meio lê verde `159,198,159`, e o amarelo sozinho lê o mesmo de
//! botão desligado.

use super::*;

const AZUL: [f32; 3] = [0.25, 0.45, 0.95];
const AMARELO: [f32; 3] = [0.98, 0.90, 0.25];
const SIZE: u32 = 192;
/// A fila medida: o meio da sobreposição (`X_MEIO`), no centro do traço amarelo (`X_AMARELO`, que
/// fica fora do alcance do azul só no lado de lá) e o papel ao lado do amarelo, que só ele toca.
const Y: u32 = 96;
const X_MEIO: u32 = 93;
const X_SO_AMARELO: u32 = 108;

fn pincel(cor: [f32; 3], pigment: bool) -> BrushSpec {
    BrushSpec {
        radius_px: 14.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: cor,
        space_attenuation: false,
        watercolor: true,
        fill: 0.30,
        depth: 1.2,
        edge_gain: 0.4,
        edge_spread: 10.0,
        warp: 0.0,
        granulation: 0.0,
        smooth_edges: true,
        pigment,
        pigment_mix: 1.0,
        ..Default::default()
    }
}

fn arma(t: &mut PainterTool, b: BrushSpec) {
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
}

/// Um traço vertical em `x`, repassado `vezes` vezes (ida e volta) SEM levantar a caneta.
fn traco(t: &mut PainterTool, x: f32, vezes: usize) {
    assert!(t.on_canvas_pointer(cp([x, 40.0], PointerPhase::Down)));
    for k in 0..vezes {
        let (a, b): (f32, f32) = if k % 2 == 0 {
            (40.0, 150.0)
        } else {
            (150.0, 40.0)
        };
        let mut y = a;
        while (b - y).abs() > 1.0 {
            y += if b > a { 2.0 } else { -2.0 };
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        }
    }
    t.on_canvas_pointer(cp([x, 95.0], PointerPhase::Up));
}

/// Azul molhado, e o amarelo por cima NA MESMA SESSÃO (ou depois de secar).
fn cena(pigment: bool, secar: bool, vezes_amarelo: usize) -> PainterTool {
    let mut t = white_canvas(SIZE, 14.0);
    arma(&mut t, pincel(AZUL, pigment));
    traco(&mut t, 86.0, 1);
    if secar {
        for _ in 0..300 {
            t.paint_tick(0.5);
        }
    }
    arma(&mut t, pincel(AMARELO, pigment));
    traco(&mut t, 100.0, vezes_amarelo);
    t
}

/// O verde de uma mistura de azul com amarelo: o verde é o canal DOMINANTE, com folga sobre os dois.
fn e_verde(p: [u8; 4]) -> bool {
    let [r, g, b, _] = p.map(i32::from);
    g >= r + 20 && g >= b + 20
}

#[test]
fn o_pigment_mistura_molhado_sobre_molhado() {
    // CONTROLO: sem o botão, o amarelo molhado TAPA o azul no meio (é o `over` de sempre) — sem isto
    // um «verde» com o botão podia ser outra coisa qualquer.
    let sem = px(&cena(false, false, 1), SIZE, X_MEIO, Y);
    assert!(
        !e_verde(sem) && sem[0] > sem[2] + 60,
        "controlo: sem Pigment o meio é o amarelo por cima ({sem:?})"
    );
    let com = px(&cena(true, false, 1), SIZE, X_MEIO, Y);
    assert!(
        e_verde(com),
        "com Pigment, azul e amarelo MOLHADOS encostados têm de dar verde no meio — lê {com:?} \
         (antes da cura: quase branco, 254,252,235: misturava com o papel)"
    );
}

#[test]
fn repassar_o_mesmo_traco_nao_lava_a_tinta_de_baixo() {
    // A 2.ª cura de 2026-09-20 morreu AQUI: cada dab voltava a misturar e ~20 dabs amarelos lavavam
    // o azul embora. A fracção da mistura sai do alfa do PRÓPRIO traço (que satura), nunca de quantos
    // dabs passaram — logo cinco passagens dão o mesmo meio que uma.
    let uma = px(&cena(true, false, 1), SIZE, X_MEIO, Y);
    let cinco = px(&cena(true, false, 5), SIZE, X_MEIO, Y);
    assert!(
        e_verde(cinco),
        "cinco passagens do amarelo lavaram o azul: {cinco:?}"
    );
    let desvio = uma
        .iter()
        .zip(&cinco)
        .map(|(a, b)| (i32::from(*a) - i32::from(*b)).abs())
        .max()
        .unwrap_or(0);
    assert!(
        desvio <= 6,
        "repassar o mesmo traço mudou o meio de {uma:?} para {cinco:?} (desvio {desvio})"
    );
}

#[test]
fn o_pigment_nao_mistura_com_o_papel() {
    // Um traço de uma cor só sobre papel virgem: com o botão ligado ou desligado a tela sai IGUAL AO
    // BYTE — o papel não é pigmento. Antes da cura o amarelo sozinho lia `254,252,235` com o botão
    // contra `253,245,140` sem ele.
    let tela = |pigment: bool| {
        let mut t = white_canvas(SIZE, 14.0);
        arma(&mut t, pincel(AMARELO, pigment));
        traco(&mut t, 100.0, 1);
        t.canvas_rgba.to_vec()
    };
    let (sem, com) = (tela(false), tela(true));
    let o = ((Y * SIZE + X_SO_AMARELO) * 4) as usize;
    assert!(
        sem[o] > sem[o + 2] + 60,
        "controlo: a fixtura pinta amarelo em x={X_SO_AMARELO} ({:?})",
        &sem[o..o + 4]
    );
    let diferentes = sem
        .as_chunks::<4>()
        .0
        .iter()
        .zip(com.as_chunks::<4>().0)
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        diferentes, 0,
        "com Pigment ligado a lavagem sobre PAPEL mudou em {diferentes} texels — ela mistura com o papel"
    );
}

#[test]
fn sobre_tinta_seca_o_pigment_continua_a_misturar() {
    // A metade que o dono já aprovava: sobre tinta SECA o botão mexe no meio. Sem isto a porta da
    // presença podia ter desligado o termo inteiro e as duas metades acima ficariam verdes.
    let sem = px(&cena(false, true, 1), SIZE, X_MEIO, Y);
    let com = px(&cena(true, true, 1), SIZE, X_MEIO, Y);
    assert_ne!(
        sem, com,
        "sobre tinta seca o Pigment deixou de fazer efeito"
    );
}

/// **Seco e molhado dão a MESMA família de tom** (ordem do dono de 2026-09-20: *«os três meios passam a
/// misturar igual»*; a metade da aquarela seca fechou em 2026-09-29, doc 44 §2). Medido nesta
/// fixtura, `Pigment` ligado, o meio da sobreposição:
///
/// | | seco | molhado | pior canal |
/// |---|---|---|---|
/// | lei antiga (RYB no seco) | `119,209,228` — um CIANO | `159,198,159` | `69` |
/// | K–M nos dois | `128,173,139` | `159,198,159` | `31` |
///
/// A barra (`45`) sai do vale entre os dois lados medidos. ⚠️ O `e_verde` é a metade que o CIANO
/// reprova: com a lei antiga o azul DOMINA (`228 > 209`).
#[test]
fn seco_e_molhado_dao_o_mesmo_tom() {
    let seco = px(&cena(true, true, 1), SIZE, X_MEIO, Y);
    let molhado = px(&cena(true, false, 1), SIZE, X_MEIO, Y);
    assert!(
        e_verde(molhado),
        "controlo: o molhado é o verde ({molhado:?})"
    );
    assert!(
        e_verde(seco),
        "amarelo sobre azul SECO com Pigment tem de dar verde, como o molhado — lê {seco:?} \
         (a lei antiga dava um ciano, 119,209,228)"
    );
    let pior = seco
        .iter()
        .zip(&molhado)
        .take(3)
        .map(|(a, b)| (i32::from(*a) - i32::from(*b)).abs())
        .max()
        .unwrap_or(0);
    assert!(
        pior <= 45,
        "seco {seco:?} e molhado {molhado:?} afastam-se {pior} num canal (K–M mede 31, RYB 69)"
    );
}

/// **A porta das duas leis** ([`super::super::watercolor_mistura::alvo_sobre_seco`]) nas três pontas,
/// AO BIT — é o que garante que tudo o que não liga o botão sai igual ao de antes.
#[test]
fn a_porta_das_duas_leis_tem_as_pontas_ao_bit() {
    use super::super::watercolor_mistura::alvo_sobre_seco;
    use ph2d_painter_brush::blend::ryb_mix;
    let base = [0.12, 0.44, 0.93];
    let pig = [0.97, 0.88, 0.22];
    let a = 0.37;
    let ryb = ryb_mix(base, pig, a);
    let km = ph2d_pigment::mix_unit(base, pig, a);
    assert_ne!(
        ryb, km,
        "controlo: as duas leis dão cores diferentes nesta fixtura"
    );
    // Botão desligado: a lei antiga, ao bit.
    assert_eq!(alvo_sobre_seco(base, pig, a, 0.0, 0.8), ryb);
    // Água a zero: a lei do Wet Paint, ao bit.
    assert_eq!(alvo_sobre_seco(base, pig, a, 0.6, 0.0), km);
    // A água já mistura tanto quanto o botão pede: o botão não muda nada (a lei de 2026-07-06).
    assert_eq!(alvo_sobre_seco(base, pig, a, 0.5, 0.5), ryb);
    assert_eq!(alvo_sobre_seco(base, pig, a, 0.5, 1.0), ryb);
    // Entre as duas: estritamente entre, em cada canal onde elas diferem.
    let meio = alvo_sobre_seco(base, pig, a, 1.0, 0.5);
    for c in 0..3 {
        let (lo, hi) = (ryb[c].min(km[c]), ryb[c].max(km[c]));
        if hi > lo {
            assert!(
                meio[c] > lo && meio[c] < hi,
                "canal {c}: {meio:?} fora de ]{lo}, {hi}["
            );
        }
    }
}
