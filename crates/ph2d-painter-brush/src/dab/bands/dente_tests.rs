//! B26.A2: o `t` do dente do papel lido da memória (uma amostra por píxel do traço, não por dab), ao byte.

use super::oraculo_tests::stamp_band_antiga;
use super::*;

const LADO: usize = 64;

fn spec_com_dente() -> BrushSpec {
    let papel = crate::texture::TextureSettings {
        kind: crate::texture::TextureKind::Noise,
        ..Default::default()
    };
    BrushSpec {
        radius_px: 20.0,
        dente: crate::dente::Dente::do_papel(papel, 0.6),
        ..Default::default()
    }
}

fn ctx<'a>(
    spec: &'a BrushSpec,
    dente: Option<std::sync::Arc<crate::texture::grao_cache::GraoCache>>,
) -> DabCtx<'a> {
    let center = [30.3f32, 33.7];
    DabCtx {
        spec,
        tex: None,
        image: None,
        shape: None,
        shape_cor: None,
        ramp: None,
        footprint: spec.dab_footprint([0.8, 0.6]),
        center,
        cx: center[0],
        cy: center[1],
        inv_radius: 1.0 / spec.radius_px,
        radius: spec.radius_px,
        coverage: 0.8,
        tecto: 0.9,
        flow: 0.35,
        preserve_alpha: false,
        x0: 3,
        x1: 61,
        stride: LADO * 4,
        ox: 0,
        film_aa: None,
        lut: None,
        capsula: None,
        lei: crate::capsula::lei(),
        po: None,
        grao: None,
        dente,
    }
}

type Banda = fn(
    &DabCtx,
    &mut [u8],
    Option<&mut [u8]>,
    Option<&mut [u8]>,
    Option<&mut [u16]>,
    i64,
) -> (bool, bool);

/// `dabs` dabs no mesmo sítio; devolve (dst, máscara, amostras do papel).
fn pinta(c: &DabCtx, f: Banda, dabs: usize) -> (Vec<u8>, Vec<u8>, u64) {
    let (mut d, mut m) = (vec![200u8; LADO * LADO * 4], vec![0u8; LADO * LADO]);
    let antes = crate::dente::AMOSTRAS.with(std::cell::Cell::get);
    for _ in 0..dabs {
        let _ = f(c, &mut d, None, Some(&mut m), None, 0);
    }
    (d, m, crate::dente::AMOSTRAS.with(std::cell::Cell::get) - antes)
}

/// **O dente amostra o papel no máximo uma vez por píxel do traço** (antes: uma vez por píxel de CADA dab; um
/// traço pousa ~90 dabs por píxel), e o carimbo pinta os mesmos bytes que a banda antiga.
#[test]
fn o_dente_do_papel_amostra_cada_pixel_uma_vez_e_pinta_os_mesmos_bytes() {
    let spec = spec_com_dente();
    let d = spec.dente.as_ref().expect("o fixture tem dente");
    let mem = (0..3)
        .find_map(|_| crate::texture::grao_cache::para_o_dente(d, LADO as u32, LADO as u32))
        .expect("a memória do dente nasce na 2.ª chave igual");
    let caixa = (LADO * (61 - 3)) as u64;
    let (dn, mn, amostras) = pinta(&ctx(&spec, Some(mem)), stamp_band, 8);
    let (da, ma, amostras_antigas) = pinta(&ctx(&spec, None), stamp_band_antiga, 8);
    assert!(amostras > 0, "o dente tem de ser amostrado");
    assert!(
        amostras <= caixa,
        "o papel do dente foi amostrado {amostras}× para {caixa} píxeis (antiga: {amostras_antigas})"
    );
    assert!(amostras_antigas > caixa, "o fixture tem de repetir píxeis");
    assert!(dn == da, "o dst diverge da banda antiga");
    assert!(mn == ma, "a máscara diverge da banda antiga");
    assert!(dn != vec![200u8; LADO * LADO * 4], "o fixture tem de pintar");
}

/// Sonda B26.A2 (`--profile smoke -- --ignored`): o carimbo com a memória do dente × sem ela, no MESMO processo,
/// intercaladas em blocos, o MÍNIMO de cada — dab r20 com papel Noise, 8 dabs sobre o mesmo sítio.
#[test]
#[ignore = "sonda de relógio: --profile smoke"]
fn sonda_o_dente_com_memoria_custa_menos() {
    let spec = spec_com_dente();
    let d = spec.dente.as_ref().expect("o fixture tem dente");
    let mem = (0..3)
        .find_map(|_| crate::texture::grao_cache::para_o_dente(d, LADO as u32, LADO as u32))
        .expect("a memória do dente nasce na 2.ª chave igual");
    let (com, sem) = (ctx(&spec, Some(mem)), ctx(&spec, None));
    let mede = |c: &DabCtx| {
        let t = std::time::Instant::now();
        let _ = pinta(c, stamp_band, 16);
        t.elapsed().as_secs_f64() * 1e3 / 16.0
    };
    let (mut a, mut b) = (f64::MAX, f64::MAX);
    for bloco in 0..12 {
        if bloco % 2 == 0 {
            a = a.min(mede(&sem));
            b = b.min(mede(&com));
        } else {
            b = b.min(mede(&com));
            a = a.min(mede(&sem));
        }
    }
    println!("[B26.A2] dente sem memória {a:.4} ms/dab · com {b:.4} ms/dab · {:.2}×", a / b);
}
