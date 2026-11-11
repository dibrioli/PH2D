use super::super::super::Region;
use super::MascaraDoTraco;
use ph2d_painter_brush::StrokeMethod;

/// O `prepare_stroke_mask` de antes de B26.A1, VERBATIM: o oráculo da identidade da máscara.
fn prepare_stroke_mask_antiga(mask: &mut Vec<u8>, len: usize, method: StrokeMethod) {
    if !matches!(
        method,
        StrokeMethod::Space | StrokeMethod::Dots | StrokeMethod::Airbrush
    ) {
        mask.clear(); // fill re-stamp → fresh cap
    }
    mask.resize(len, 0);
}

fn caixa(x: u32, y: u32, w: u32, h: u32) -> Region {
    Region { x, y, w, h }
}

fn pinta(m: &mut [u8], largura: usize, c: Region, v: u8) {
    for y in c.y as usize..(c.y + c.h) as usize {
        let i = y * largura + c.x as usize;
        m[i..i + c.w as usize].fill(v);
    }
}

#[test]
fn o_recarimbo_de_um_preenchimento_zera_so_a_caixa_do_anterior() {
    let (w, h) = (3840usize, 2160usize);
    let n = w * h;
    let mut m = MascaraDoTraco::default();
    m.clear(); // o pen-down
    let a = caixa(100, 100, 400, 60);
    m.prepara(n, w, StrokeMethod::Line, [a]);
    pinta(m.fatia_do_lote(), w, a, 200);
    assert_eq!(m.zerados(), 0, "o 1.º lote do traço recebe páginas do SO");
    for _ in 0..3 {
        let antes = m.zerados();
        m.prepara(n, w, StrokeMethod::Line, [a]);
        let feito = m.zerados() - antes;
        assert!(
            feito <= u64::from(a.w * a.h),
            "um re-carimbo zerou {feito} bytes; a caixa anterior tem {}",
            a.w * a.h
        );
        assert!(m.iter().all(|&b| b == 0));
        pinta(m.fatia_do_lote(), w, a, 200);
    }
    let antes = m.zerados();
    m.clear();
    m.prepara(n, w, StrokeMethod::Space, [a]);
    assert_eq!(
        m.zerados(),
        antes,
        "a 1.ª pincelada do Space não zera 8,3 MB"
    );
}

#[test]
fn a_mascara_com_caixa_e_a_do_clear_mais_resize() {
    let (w, h) = (97usize, 61usize);
    let n = w * h;
    let metodos = [
        StrokeMethod::Line,
        StrokeMethod::Space,
        StrokeMethod::Arc,
        StrokeMethod::Dots,
        StrokeMethod::Airbrush,
    ];
    let mut s = 0x9E37_79B9u32;
    let mut sorteia = |m: u32| {
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        s % m
    };
    let mut nova = MascaraDoTraco::default();
    let mut antiga: Vec<u8> = Vec::new();
    for passo in 0..400 {
        if sorteia(9) == 0 {
            nova.clear();
            antiga.clear();
        }
        let metodo = metodos[sorteia(metodos.len() as u32) as usize];
        let (x, y) = (sorteia(w as u32 - 1), sorteia(h as u32 - 1));
        let c = caixa(x, y, 1 + sorteia(w as u32 - x), 1 + sorteia(h as u32 - y));
        nova.prepara(n, w, metodo, [c]);
        prepare_stroke_mask_antiga(&mut antiga, n, metodo);
        assert!(nova[..] == antiga[..], "passo {passo}: a máscara divergiu");
        let v = 1 + sorteia(255) as u8;
        pinta(nova.fatia_do_lote(), w, c, v);
        pinta(&mut antiga, w, c, v);
        if sorteia(7) == 0 {
            // um escritor fora da porta do lote (o `DerefMut`): o mapa inteiro fica sujo
            let i = sorteia(n as u32) as usize;
            nova[i] = 9;
            antiga[i] = 9;
        }
    }
}
