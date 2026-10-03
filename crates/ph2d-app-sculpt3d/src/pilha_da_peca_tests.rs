//! Gates da pilha de camadas da peça (`docs/3D/30` §5, W1).

use super::*;
use ph2d_tool_painter::{HsbParams, LayerImage, MapPixelSource, composite};

/// Um plano de `n` amostras com cor e alfa variados (nenhuma corrida junta
/// duas), semente `s`.
fn plano(n: usize, s: u32) -> Vec<u8> {
    let mut v = vec![0u8; n * 4];
    for (i, px) in v.chunks_exact_mut(4).enumerate() {
        let h = (i as u32 ^ s).wrapping_mul(2_654_435_761);
        px.copy_from_slice(&[
            (h >> 8) as u8,
            (h >> 16) as u8,
            (h >> 24) as u8,
            (h & 0xff) as u8 | 0x10,
        ]);
    }
    v
}

/// Escreve `px` (N·4 bytes) no plano da camada `id`.
fn pinta(p: &mut PilhaDaPeca, id: LayerId, px: &[u8]) {
    p.planos.get_mut(&id).expect("plano").rgba8[..px.len()].copy_from_slice(px);
}

/// ⭐ A FIXTURA: base opaca + Multiply a 60 % + Overlay recortada com máscara
/// + um ajuste HSB — modo, opacidade, recorte, máscara e ajuste exercidos.
fn pilha_rica(n: usize) -> (PilhaDaPeca, [LayerId; 4]) {
    let mut p = PilhaDaPeca::de_partes(LayerStack::new(), BTreeMap::new(), n);
    let base = p.nova_camada(nome_da_base()).expect("base");
    let mut opaca = plano(n, 1);
    opaca.chunks_exact_mut(4).for_each(|px| px[3] = 255);
    pinta(&mut p, base, &opaca);
    let mult = p.nova_camada("mult").expect("mult");
    pinta(&mut p, mult, &plano(n, 2));
    p.define_modo(mult, BlendMode::Multiply);
    p.define_opacidade(mult, 0.6);
    let over = p.nova_camada("over").expect("over");
    pinta(&mut p, over, &plano(n, 3));
    p.define_modo(over, BlendMode::Overlay);
    p.define_recorte(over, true);
    let mascara = p.nova_mascara(over).expect("máscara");
    pinta(&mut p, mascara, &plano(n, 4));
    let hsb = p
        .novo_ajuste(AdjustmentKind::HueSaturationBrightness)
        .expect("HSB");
    p.define_parametros(
        hsb,
        AdjustmentParams::HueSaturationBrightness(HsbParams {
            h: 30.0,
            s: 0.2,
            b: 0.1,
        }),
    )
    .expect("parâmetros do mesmo tipo");
    assert!(p.sincronizada());
    (p, [base, mult, over, hsb])
}

/// A MESMA pilha como o Painter 2D a compõe, numa imagem de `l × h`.
fn composto_2d(p: &PilhaDaPeca, l: u32, h: u32) -> Vec<u8> {
    let n = p.amostras;
    let mut pilha = p.pilha.clone();
    let mut fonte = MapPixelSource::default();
    for (id, pl) in &p.planos {
        let mut rgba8 = pl.rgba8[..n * 4].to_vec();
        rgba8.resize(l as usize * h as usize * 4, 0);
        fonte.insert(
            *id,
            LayerImage {
                width: l,
                height: h,
                rgba8,
            },
        );
    }
    let ids: Vec<LayerId> = pilha.all_ids().collect();
    for id in ids {
        match pilha.get_mut(id).map(|c| &mut c.kind) {
            Some(LayerKind::Raster(r)) => (r.width, r.height) = (l, h),
            Some(LayerKind::Mask(m)) => (m.width, m.height) = (l, h),
            _ => {}
        }
    }
    let mut c = composite(&pilha, &fonte, l, h);
    c.truncate(n * 4);
    c
}

/// ⭐⭐⭐⭐ **GATE — A COMPOSIÇÃO DA PEÇA É A DO PAINTER, AO BIT** — contra a
/// MESMA imagem 2D `1024 × ⌈N/1024⌉` e contra a peça desdobrada `N × 1` (a
/// composição ponto a ponto não depende da forma).
///
/// ⛔ O CONTROLO é o fenómeno que a porta recusa: um desfoque sobre a mesma
/// pilha DÁ outra resposta dobrado e desdobrado — ele borra pela ORDEM das
/// amostras, e é por isso que a peça só o aceita com o gancho de vizinhança.
#[test]
fn a_composicao_da_peca_e_a_do_painter_ao_bit() {
    let n = 2_500; // nem múltiplo da dobra: a última linha é parcial
    let (p, _) = pilha_rica(n);
    let peca = p.compor();
    assert_eq!(peca.len(), n * 4);
    let (l, h) = dobra(n);
    assert_eq!((l, h), (1024, 3));
    assert_eq!(peca, composto_2d(&p, l, h), "a mesma imagem 2D dobrada");
    assert_eq!(peca, composto_2d(&p, n as u32, 1), "a peça desdobrada N×1");

    let mut borrada = p.clone();
    let blur = borrada
        .pilha
        .add_adjustment(AdjustmentKind::GaussianBlur)
        .expect("o controlo entra pela pilha crua");
    if let Some(a) = borrada.pilha.adjustment_mut(blur)
        && let AdjustmentParams::GaussianBlur(g) = &mut a.params
    {
        g.radius = 6.0;
    }
    assert!(!borrada.sincronizada(), "a porta não a aceitaria");
    assert_ne!(
        composto_2d(&borrada, l, h),
        composto_2d(&borrada, n as u32, 1),
        "o CONTROLO: um desfoque depende da forma da imagem"
    );
}

/// ⭐⭐⭐ **GATE — Compor uma FAIXA é o pedaço da peça inteira, ao bit** — a
/// cabeça parcial, as linhas inteiras e a cauda, e os casos de borda.
#[test]
fn compor_uma_faixa_e_o_pedaco_da_peca_inteira() {
    let n = 3_333;
    let (p, _) = pilha_rica(n);
    let tudo = p.compor();
    for (a, b) in [
        (0, n),
        (0, 1),
        (5, 6),
        (1000, 1030),
        (1024, 2048),
        (700, 3100),
        (2048, n),
        (n - 1, n),
        (17, 17),
    ] {
        assert_eq!(p.compor_faixa(a, b), tudo[a * 4..b * 4], "faixa {a}..{b}");
    }
    assert_eq!(
        p.compor_faixa(n - 3, n + 50),
        tudo[(n - 3) * 4..],
        "o fim corta-se em N"
    );
}

/// ⭐⭐⭐ **GATE — PILHA ↔ PLANOS EM SINCRONIA** depois de cada operação da
/// porta, e as recusas deixam-na como estava.
#[test]
fn a_pilha_e_os_planos_andam_juntos() {
    let n = 1_100;
    let (mut p, [base, mult, over, _hsb]) = pilha_rica(n);
    let conta = |p: &PilhaDaPeca| p.planos.len();
    assert_eq!(conta(&p), 4, "3 rasters + 1 máscara");

    let copia = p.duplica(mult).expect("duplica");
    assert!(p.sincronizada());
    assert_eq!(p.plano(copia), p.plano(mult), "a cópia leva o plano");

    assert_eq!(
        p.novo_ajuste(AdjustmentKind::GaussianBlur),
        Err(RecusaDaPilha::LeAVizinhanca(AdjustmentKind::GaussianBlur))
    );
    assert!(
        p.novo_ajuste(AdjustmentKind::Noise).is_ok(),
        "o ruído não lê vizinhos"
    );
    assert!(p.sincronizada());

    let mascara = p
        .pilha
        .get(over)
        .and_then(|c| c.mask)
        .expect("a máscara da fixtura");
    p.apaga(over).expect("apaga");
    assert!(p.sincronizada());
    assert!(p.plano(mascara).is_none(), "a máscara foi com a dona");
    assert_eq!(p.apaga(over), Err(RecusaDaPilha::Desconhecida));

    assert_eq!(
        p.nova_mascara(base)
            .map(|m| p.plano(m).map(|pl| pl.rgba8(n)[0])),
        Ok(Some(255)),
        "nasce branca"
    );
    assert!(p.sincronizada());

    // ⛔ CONTROLO: a régua vê um plano a faltar e um plano a mais.
    let mut sem = p.clone();
    sem.planos.remove(&base);
    assert!(!sem.sincronizada(), "uma camada sem plano");
    let mut mais = p.clone();
    mais.planos
        .insert(LayerId(9_999), PlanoDaCamada::transparente(n));
    assert!(!mais.sincronizada(), "um plano sem camada");
    let mut curto = p.clone();
    curto.planos.get_mut(&base).expect("base").rgba8.pop();
    assert!(!curto.sincronizada(), "um plano do tamanho errado");
}

/// ⭐⭐ **GATE — O TECTO de camadas recusa sem partir a sincronia.**
#[test]
fn o_tecto_de_camadas_recusa_e_a_pilha_fica_inteira() {
    let mut p = PilhaDaPeca::de_partes(LayerStack::new(), BTreeMap::new(), 8);
    for i in 0..ph2d_tool_painter::HARD_CAP_LAYERS {
        p.nova_camada(&format!("c{i}")).expect("abaixo do tecto");
    }
    assert_eq!(p.nova_camada("uma a mais"), Err(RecusaDaPilha::Tecto));
    assert!(p.sincronizada());
}

fn tinta_variada(relevo: bool) -> Tinta {
    let faces: Vec<[u32; 3]> = vec![[0, 1, 2], [0, 2, 3], [0, 3, 1], [1, 3, 2]];
    let mut t = Tinta::nova(4, faces.iter().map(|f| &f[..]), 3);
    let n = t.amostras().len();
    for (i, c) in t.amostras_mut().iter_mut().enumerate() {
        let x = i as f32 / n as f32;
        *c = [x, 1.0 - x, (x * 7.3).fract()];
    }
    if relevo {
        for (i, r) in t.relevo_mut().iter_mut().enumerate() {
            *r = [i as f32 * 1e-4 - 0.01, (i % 5) as f32 * 0.25];
        }
    }
    t
}

/// ⭐⭐⭐ **GATE — Um plano anterior às camadas é UMA camada opaca**, e
/// recompô-la devolve a cor a no máximo MEIO degrau de sRGB8 e o relevo ao
/// bit. O fundo nem é pedido: a camada é opaca.
#[test]
fn um_plano_vira_uma_camada_opaca_a_meio_degrau() {
    let t = tinta_variada(true);
    let p = PilhaDaPeca::de_tinta(&t);
    assert!(p.sincronizada());
    assert_eq!(p.pilha().len(), 1);
    let base = p.pilha().root()[0];
    assert_eq!(
        p.pilha().get(base).map(|c| (c.name.as_str(), c.has_relief)),
        Some((nome_da_base(), true))
    );

    let mut volta = t.clone();
    volta.amostras_mut().fill([0.0; 3]);
    volta.com_relevo(None);
    p.pinta_tinta(&mut volta, || {
        panic!("o fundo de uma pilha opaca não é lido")
    });
    let pior = t
        .amostras()
        .iter()
        .zip(volta.amostras())
        .flat_map(|(a, b)| (0..3).map(move |c| (a[c] - b[c]).abs()))
        .fold(0.0f32, f32::max);
    assert!(pior <= 0.5 / 255.0 + 1e-6, "pior desvio {pior}");
    assert!(
        pior > 0.0,
        "o CONTROLO: a fixtura tem cor que não cai em degrau"
    );
    let bits = |r: &[[f32; 2]]| r.iter().map(|x| x.map(f32::to_bits)).collect::<Vec<_>>();
    assert_eq!(
        bits(volta.relevo().expect("relevo")),
        bits(t.relevo().expect("relevo"))
    );

    let sem = PilhaDaPeca::de_tinta(&tinta_variada(false));
    assert!(sem.relevo_composto().is_none(), "sem relevo, nenhum nasce");
}

/// ⭐⭐ **GATE — O composto sobre o FUNDO**: opaco é `byte / 255` exacto e
/// não lê o fundo; transparente é o fundo; o meio mistura em LUZ.
#[test]
fn o_composto_assenta_no_fundo_em_luz() {
    use ph2d_color::srgb::{linear_to_srgb_unit, srgb_to_linear_byte, srgb_to_linear_unit};
    let composto = [10u8, 128, 250, 255, 200, 100, 50, 0, 200, 100, 50, 128];
    let fundo = [0.3f32, 0.6, 0.9];
    let mut lidos = Vec::new();
    let mut out = [[9.0f32; 3]; 3];
    let lido = std::cell::RefCell::new(&mut lidos);
    achata(
        &composto,
        |i| {
            lido.borrow_mut().push(i);
            fundo
        },
        &mut out,
    );
    assert_eq!(
        lidos,
        vec![1, 2],
        "o fundo só se lê onde o composto não é opaco"
    );
    assert_eq!(out[0], [10.0 / 255.0, 128.0 / 255.0, 250.0 / 255.0]);
    for c in 0..3 {
        assert!((out[1][c] - fundo[c]).abs() < 1e-5, "alfa 0 é o fundo");
        let a = 128.0 / 255.0;
        let luz =
            srgb_to_linear_byte(composto[8 + c]) * a + srgb_to_linear_unit(fundo[c]) * (1.0 - a);
        assert_eq!(out[2][c], linear_to_srgb_unit(luz), "a meio, em luz");
    }
    assert!(precisa_de_fundo(&composto) && !precisa_de_fundo(&composto[..4]));
}
