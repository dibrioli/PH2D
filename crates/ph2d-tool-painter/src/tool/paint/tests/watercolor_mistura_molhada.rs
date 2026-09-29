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

/// **A medição do report «reduzindo Charge para < 1 não se percebe a mistura»** (dono, 2026-09-29).
/// Instrumento, não gate: imprime o meio da sobreposição e o amarelo sozinho, seco e molhado, com e
/// sem `Pigment`, ao longo do `Charge`.
#[test]
#[ignore = "measurement, not a gate — o instrumento do report do Charge"]
fn diag_pigment_com_charge() {
    let medir = |pigment: bool, secar: bool, charge: f32| {
        let mut t = white_canvas(SIZE, 14.0);
        let mut b = pincel(AZUL, pigment);
        b.wet_charge = charge;
        arma(&mut t, b);
        traco(&mut t, 86.0, 1);
        if secar {
            for _ in 0..300 {
                t.paint_tick(0.5);
            }
        }
        let mut b = pincel(AMARELO, pigment);
        b.wet_charge = charge;
        arma(&mut t, b);
        traco(&mut t, 100.0, 1);
        (
            px(&t, SIZE, X_MEIO, Y),
            px(&t, SIZE, X_SO_AMARELO, Y),
            px(&t, SIZE, 86, Y),
        )
    };
    println!();
    for secar in [true, false] {
        for charge in [1.0f32, 0.9, 0.75, 0.5, 0.25] {
            let (m0, a0, z0) = medir(false, secar, charge);
            let (m1, a1, z1) = medir(true, secar, charge);
            println!(
                "secar={secar:<5} charge={charge:<4} | meio sem {:?} com {:?} | só amarelo sem {:?} com {:?} | azul sem {:?} com {:?}",
                &m0[..3],
                &m1[..3],
                &a0[..3],
                &a1[..3],
                &z0[..3],
                &z1[..3]
            );
        }
    }
}

/// **Um pincel que «apanha cor» (`Charge < 1`) continua a misturar molhado sobre molhado** (report do
/// dono, 2026-09-29: *«reduzindo Charge para < 1 não se percebe a mistura»*). O mixer do Charge só
/// grava na tela o que APANHOU, e ele lê a base CONGELADA — a tinta molhada da própria sessão não
/// está lá, logo não apanha nada e não gravava nada: o `Pigment` ficava sem parceiro e o meio lia o
/// MESMO pixel com o botão ligado e desligado (`249,243,149` a `Charge 0,9`). Depois: `159,198,161`.
#[test]
fn o_charge_abaixo_de_um_nao_desliga_o_pigment_molhado() {
    let meio = |pigment: bool, charge: f32| {
        let mut t = white_canvas(SIZE, 14.0);
        for (x, cor) in [(86.0, AZUL), (100.0, AMARELO)] {
            let mut b = pincel(cor, pigment);
            b.wet_charge = charge;
            arma(&mut t, b);
            traco(&mut t, x, 1);
        }
        px(&t, SIZE, X_MEIO, Y)
    };
    for charge in [0.9f32, 0.5] {
        let sem = meio(false, charge);
        let com = meio(true, charge);
        assert!(
            !e_verde(sem),
            "controlo (Charge {charge}): sem Pigment o amarelo tapa o azul ({sem:?})"
        );
        assert!(
            e_verde(com),
            "Charge {charge}: com Pigment o meio molhado tem de dar verde — lê {com:?} (sem: {sem:?})"
        );
    }
}

/// **A medição do report «o Smudge e o Rewet não afetam a mancha quando a tinta está molhada»**
/// (dono, 2026-09-29). Instrumento: uma faixa azul VERTICAL, e um traço amarelo HORIZONTAL que a
/// atravessa da esquerda para a direita, com o knob em `0` e em `1`, com o azul SECO e MOLHADO.
/// Imprime quantos texels o knob mudou, quanto (soma de |Δ| por canal) e o azul ARRASTADO para lá da
/// faixa (texels à direita dela, na linha do traço, mais azuis que o amarelo sozinho).
#[test]
#[ignore = "measurement, not a gate — o instrumento do report do Smudge/Rewet"]
fn diag_smudge_e_rewet_sobre_molhado() {
    let tela = |secar: bool, smudge: f32, rewet: f32| {
        let mut t = white_canvas(SIZE, 14.0);
        arma(&mut t, pincel(AZUL, false));
        traco(&mut t, 86.0, 1);
        if secar {
            for _ in 0..300 {
                t.paint_tick(0.5);
            }
        }
        let mut b = pincel(AMARELO, false);
        b.wet_smudge = smudge;
        b.wet_rewet = rewet;
        arma(&mut t, b);
        assert!(t.on_canvas_pointer(cp([40.0, 96.0], PointerPhase::Down)));
        let mut x = 40.0f32;
        while x < 160.0 {
            x += 2.0;
            t.on_canvas_pointer(cp([x, 96.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([160.0, 96.0], PointerPhase::Up));
        t
    };
    let azul_depois = |t: &PainterTool| {
        // à direita da faixa (86 ± 14), na linha do traço: quanto o B passa o R (o amarelo tem R ≫ B)
        (104..150u32)
            .map(|x| {
                let p = px(t, SIZE, x, 96);
                (i32::from(p[2]) - i32::from(p[0])).max(-255)
            })
            .max()
            .unwrap_or(0)
    };
    println!();
    for secar in [true, false] {
        let base = tela(secar, 0.0, 0.0);
        for (nome, s, r) in [("smudge 1", 1.0f32, 0.0f32), ("rewet 1", 0.0, 1.0)] {
            let k = tela(secar, s, r);
            let (mut n, mut soma) = (0usize, 0i64);
            for (a, b) in base
                .canvas_rgba
                .as_chunks::<4>()
                .0
                .iter()
                .zip(k.canvas_rgba.as_chunks::<4>().0)
            {
                let d: i64 = (0..3)
                    .map(|c| (i64::from(a[c]) - i64::from(b[c])).abs())
                    .sum();
                if d > 0 {
                    n += 1;
                    soma += d;
                }
            }
            println!(
                "secar={secar:<5} {nome:<8} | texels mudados {n:>6} · soma |Δ| {soma:>8} | azul arrastado (B−R máx à direita): knob0 {} knob1 {} | meio {:?} → {:?}",
                azul_depois(&base),
                azul_depois(&k),
                &px(&base, SIZE, 86, 96)[..3],
                &px(&k, SIZE, 86, 96)[..3],
            );
        }
    }
}
