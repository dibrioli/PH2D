//! Gates do Wet blend do Digital ([`crate::mistura_opaca`]) — a lei medida do CSP
//! (`docs/Painter/CSP_COMO_MISTURA_2026-11-08.md`).

use crate::mistura_opaca::{
    Carimbo, MisturaOpaca, ModoMisturaOpaca, PASSO_DA_MISTURA, Pegada, ReservatorioOpaco, Sombra,
    Tela, mistura_opaca_carimbo,
};
use crate::spec::BrushSpec;

const W: u32 = 200;
const H: u32 = 40;
const VERMELHO: [u8; 4] = [220, 20, 20, 255];
const AZUL: [u8; 4] = [20, 40, 220, 255];
const AMARELO: [f32; 3] = [0.94, 0.78, 0.0];

/// Vermelho à esquerda de `x = 60`, azul à direita.
fn fundo() -> Vec<u8> {
    let mut b = vec![0u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let i = ((y * W + x) * 4) as usize;
            b[i..i + 4].copy_from_slice(if x < 60 { &VERMELHO } else { &AZUL });
        }
    }
    b
}

fn pincel(amount: f32, density: f32, stretch: f32) -> BrushSpec {
    BrushSpec {
        radius_px: 8.0,
        hardness: 0.5,
        mistura_opaca: MisturaOpaca {
            modo: ModoMisturaOpaca::WetBlend,
            paint_amount: amount,
            paint_density: density,
            color_stretch: stretch,
            ..MisturaOpaca::default()
        },
        ..BrushSpec::default()
    }
}

/// Um traço de `x = 10` a `190` a meio da altura, ao passo da mistura.
fn traco(buf: &mut [u8], spec: &BrushSpec, cor: [f32; 3]) {
    let antes = buf.to_vec();
    let mut res = ReservatorioOpaco::default();
    let mut sombra = Sombra::default();
    let passo = (2.0 * spec.radius_px * spec.spacing_do_traco()).max(0.25);
    let mut x = 10.0f32;
    while x <= 190.0 {
        let mut tela = Tela {
            buf,
            width: W,
            height: H,
            sombra: &mut sombra,
            trava_alfa: false,
            plano: false,
        };
        let _ = mistura_opaca_carimbo(
            &mut tela,
            |i| [antes[i], antes[i + 1], antes[i + 2], antes[i + 3]],
            spec,
            Carimbo {
                center: [x, 20.0],
                radius_px: spec.radius_px,
                cor,
                coverage: 1.0,
                pressao: 1.0,
            },
            Pegada::Disco {
                centro: [x, 20.0],
                raio: spec.radius_px,
            },
            &mut res,
        );
        x += passo;
    }
}

fn px(buf: &[u8], x: u32, y: u32) -> [u8; 4] {
    let i = ((y * W + x) * 4) as usize;
    [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
}

#[test]
fn wet_blend_off_nao_toca_em_nada() {
    let mut b = fundo();
    let antes = b.clone();
    let mut res = ReservatorioOpaco::default();
    let mut sombra = Sombra::default();
    let spec = BrushSpec::default();
    assert_eq!(spec.mistura_opaca.modo, ModoMisturaOpaca::Off);
    let mut tela = Tela {
        buf: &mut b,
        width: W,
        height: H,
        sombra: &mut sombra,
        trava_alfa: false,
        plano: false,
    };
    let r = mistura_opaca_carimbo(
        &mut tela,
        |i| [antes[i], antes[i + 1], antes[i + 2], antes[i + 3]],
        &spec,
        Carimbo {
            center: [40.0, 20.0],
            radius_px: 8.0,
            cor: [0.0; 3],
            coverage: 1.0,
            pressao: 1.0,
        },
        Pegada::Disco {
            centro: [40.0, 20.0],
            raio: 8.0,
        },
        &mut res,
    );
    assert!(r.is_none());
    assert_eq!(b, antes);
    assert_eq!(res, ReservatorioOpaco::default());
    assert_eq!(sombra.bytes(), 0);
}

/// O defeito da foto do dono (08/11): a fronteira de baixo NÃO pode aparecer como degrau dentro do traço.
#[test]
fn wet_blend_nao_deixa_a_costura_de_baixo_dentro_do_traco() {
    let mut b = fundo();
    traco(&mut b, &pincel(0.5, 1.0, 0.6), AMARELO);
    let maior = (40..120)
        .map(|x| {
            let (a, c) = (px(&b, x, 20), px(&b, x + 1, 20));
            (0..3)
                .map(|k| (i32::from(a[k]) - i32::from(c[k])).abs())
                .max()
                .unwrap_or(0)
        })
        .max()
        .unwrap_or(0);
    assert!(
        maior <= 8,
        "degrau de {maior} níveis entre colunas vizinhas no miolo do traço"
    );
}

/// O patamar do CSP com a amostra da tela VIVA: num fundo uniforme o carimbo vê metade de tinta sua e metade de
/// fundo, e a fração do fundo fica ABAIXO de `1 − Amount` (Round mixing do CSP, Amount 0,8, ponta redonda: 0,07).
#[test]
fn wet_blend_patamar_e_o_amount() {
    let fracao = |amount: f32| {
        let mut b = fundo();
        traco(&mut b, &pincel(amount, 0.5, 1.0), [1.0, 1.0, 1.0]);
        // Branco por cima do azul: quanto do azul ficou no canal vermelho.
        let r = f32::from(px(&b, 170, 20)[0]);
        (255.0 - r) / (255.0 - f32::from(AZUL[0]))
    };
    let f8 = fracao(0.8);
    eprintln!(
        "fração do fundo a Amount 0,8: {f8:.3} · 0,5: {:.3}",
        fracao(0.5)
    );
    assert!(
        (0.03..=0.15).contains(&f8),
        "Amount 0,8: {f8:.3} (CSP 0,07)"
    );
    assert!(
        fracao(0.5) > f8 && fracao(0.5) < 0.5,
        "Amount 0,5: {:.3}",
        fracao(0.5)
    );
}

/// Color stretch maior leva o vermelho apanhado mais longe sobre o azul.
#[test]
fn wet_blend_stretch_e_a_memoria() {
    let alcance = |stretch: f32| {
        let mut b = fundo();
        traco(&mut b, &pincel(0.3, 1.0, stretch), [0.5, 0.5, 0.5]);
        (60..W).take_while(|&x| px(&b, x, 20)[0] > 110).count()
    };
    let (curto, longo) = (alcance(0.1), alcance(0.9));
    assert!(
        longo > curto + 10,
        "stretch 0,9 leva {longo} px, 0,1 leva {curto} px"
    );
}

/// Pigment 1: amarelo sobre azul dá VERDE em algum Amount; Pigment 0 (RGB do CSP) nunca tão verde.
#[test]
fn wet_blend_pigmento_faz_verde() {
    let verde = |pig: f32, amount: f32| {
        let mut b = fundo();
        let mut s = pincel(amount, 1.0, 0.3);
        s.mistura_opaca.pigmento = pig;
        traco(&mut b, &s, AMARELO);
        let p = px(&b, 170, 20).map(i32::from);
        p[1] - p[0].max(p[2])
    };
    let amounts = [0.05, 0.1, 0.15, 0.25, 0.4];
    let melhor = |pig: f32| amounts.iter().map(|&a| verde(pig, a)).max().unwrap_or(0);
    let (pig, rgb) = (melhor(1.0), melhor(0.0));
    eprintln!("verde: pigmento {pig} · RGB {rgb}");
    assert!(pig > 8, "pigmento: verde por {pig}");
    assert!(pig > rgb + 15, "pigmento {pig} contra RGB {rgb}");
}

/// Sobre o transparente o pincel pinta a cor dele, cheia (o vazio não é tinta e não suja).
#[test]
fn wet_blend_no_vazio_pinta_a_cor_do_pincel() {
    let mut b = vec![0u8; (W * H * 4) as usize];
    traco(&mut b, &pincel(0.3, 1.0, 0.5), AMARELO);
    let p = px(&b, 100, 20);
    assert!(p[3] >= 250, "alfa {}", p[3]);
    for k in 0..3 {
        assert!((f32::from(p[k]) - AMARELO[k] * 255.0).abs() <= 2.0, "{p:?}");
    }
}

/// A Density é a força de cada carimbo (o CSP): 1 cobre numa passada, 0,5 ainda cobre quase tudo ao passo de
/// fábrica (CSP: 88–98 %), e só uma Density baixa com carimbos espaçados deixa ver o de baixo.
#[test]
fn wet_blend_density_e_a_forca_do_carimbo() {
    let branco = |density: f32, spacing: f32| {
        let mut b = fundo();
        let mut s = pincel(1.0, density, 0.5);
        s.hardness = 1.0;
        s.spacing = spacing;
        traco(&mut b, &s, [1.0, 1.0, 1.0]);
        px(&b, 150, 20)[0]
    };
    assert!(
        branco(1.0, 0.1) >= 250,
        "Density 1 cobre: {}",
        branco(1.0, 0.1)
    );
    assert!(
        branco(0.5, 0.1) >= 235,
        "Density 0,5 cobre quase tudo: {}",
        branco(0.5, 0.1)
    );
    let fraca = branco(0.1, 0.25);
    assert!(fraca < 200, "Density 0,1 a spacing 0,25 vela: {fraca}");
}

#[test]
fn wet_blend_aperta_o_passo_so_quando_ligada() {
    let mut s = BrushSpec {
        spacing: 0.15,
        ..BrushSpec::default()
    };
    assert!((s.spacing_do_traco() - 0.15).abs() < 1e-6);
    s.mistura_opaca.modo = ModoMisturaOpaca::WetBlend;
    assert!((s.spacing_do_traco() - PASSO_DA_MISTURA).abs() < 1e-6);
}

#[test]
fn wet_blend_carrega_os_nomes_antigos() {
    for n in ["PigmentMix", "Blend", "RunningColor", "WetBlend"] {
        assert_eq!(
            ModoMisturaOpaca::do_nome(n),
            Some(ModoMisturaOpaca::WetBlend)
        );
    }
    assert_eq!(
        ModoMisturaOpaca::do_nome("Off"),
        Some(ModoMisturaOpaca::Off)
    );
}

/// Amount 0 é um esbatedor: arrasta a cor de baixo e não pinta o vazio.
#[test]
fn wet_blend_amount_zero_so_arrasta() {
    let mut vazio = vec![0u8; (W * H * 4) as usize];
    traco(&mut vazio, &pincel(0.0, 1.0, 0.5), AMARELO);
    assert!(vazio.iter().all(|&v| v == 0), "Amount 0 pintou o vazio");
    let mut b = fundo();
    traco(&mut b, &pincel(0.0, 1.0, 0.8), AMARELO);
    // O vermelho arrasta-se sobre o azul, sem amarelo nenhum.
    let p = px(&b, 66, 20);
    assert!(p[0] > 60 && p[1] < 60, "{p:?}");
}
