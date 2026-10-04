//! **CADA POÇA SECA NO SEU TEMPO** (decisão do dono 2026-10-04: *«Dry Time: fazer a tinta da aquarela
//! secar de facto com o tempo»*; doc 46 §2-5). A sessão molhada era TUDO-OU-NADA: enquanto UM canto do
//! papel estivesse molhado, toda aguada da sessão — mesmo a que secou há um minuto do outro lado — fundia
//! com o traço seguinte. Agora, no pen-down, a poça que já secou e está LONGE de toda a tinta molhada
//! assa-se na base da sessão: o traço novo VELA por cima dela, e continua a fundir onde a tinta está
//! molhada.

use super::*;

const LADO: u32 = 256;

/// O pincel do `watercolor_touching_wet_washes_merge_without_double_rim`: aro forte, sem warp, sem
/// granulação — a junção de dois traços diz sozinha se eles fundiram ou se velaram.
fn tela(dry_time_s: f32) -> PainterTool {
    let mut t = white_canvas(LADO, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.12,
        depth: 1.0,
        edge_gain: 2.5,
        edge_spread: 6.0,
        warp: 0.0,
        granulation: 0.0,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    t.set_dry_time_s(dry_time_s);
    t
}

fn vertical(t: &mut PainterTool, x: f32) {
    assert!(t.on_canvas_pointer(cp([x, 30.0], PointerPhase::Down)));
    let mut y = 30.0f32;
    while y < 160.0 {
        y += 2.0;
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        frame(t);
    }
    t.on_canvas_pointer(cp([x, 160.0], PointerPhase::Up));
}

fn espera(t: &mut PainterTool, s: f32) {
    for _ in 0..(s / 0.25).round() as usize {
        t.paint_tick(0.25);
    }
}

/// A aguada A em `x = 60`; depois, durante 16 s, um traço C em `x = 220` a cada 3 s — o papel NUNCA
/// seca inteiro, a sessão continua viva.
fn a_e_os_traços_de_c(t: &mut PainterTool) {
    vertical(t, 60.0);
    for _ in 0..5 {
        espera(t, 3.0);
        vertical(t, 220.0);
    }
    espera(t, 1.0);
    assert!(
        t.wet_session_continues(),
        "a régua: o C mantém a sessão molhada viva — senão este teste mede o caminho que já secava"
    );
}

/// A janela de A e B (a banda `x 40..100`), byte a byte.
fn a_janela_de_a(t: &PainterTool) -> Vec<[u8; 4]> {
    let mut v = Vec::new();
    for y in 20..170u32 {
        for x in 36..=100u32 {
            v.push(px(t, LADO, x, y));
        }
    }
    v
}

/// ⭐⭐⭐ **A poça que secou VELA, mesmo com outra molhada do outro lado do papel.** O traço B sobre a A
/// seca tem de dar EXACTAMENTE a imagem de quando o papel secou inteiro antes dele (a sessão acabou e o
/// B nasceu numa nova) — é a mesma física: tinta molhada sobre tinta seca.
///
/// **Mutação que sangra:** tirar a chamada do assar no pen-down (`assa_as_pocas_secas`).
#[test]
fn a_poca_seca_vela_mesmo_com_outra_molhada_no_papel() {
    let mut viva = tela(10.0);
    a_e_os_traços_de_c(&mut viva);
    vertical(&mut viva, 68.0);

    // A referência: o mesmo desenho, mas o papel seca INTEIRO antes do B (a sessão morre).
    let mut seca = tela(10.0);
    a_e_os_traços_de_c(&mut seca);
    espera(&mut seca, 70.0);
    assert!(!seca.wet_session_continues(), "a referência secou inteira");
    vertical(&mut seca, 68.0);

    assert_eq!(
        a_janela_de_a(&viva),
        a_janela_de_a(&seca),
        "o B sobre a poça A JÁ SECA tem de velar como sobre o papel seco — e fundiu com ela, porque o C \
         ainda molhado mantinha a sessão inteira viva"
    );
}

/// ⭐⭐ **O mesmo desenho com Dry Time diferente dá imagens diferentes** — o controlo age. Com 60 s a A
/// ainda está molhada quando o B chega (funde: a junção fica clara, sem aro duplo); com 10 s secou
/// (vela: os dois aros escurecem a junção).
#[test]
fn o_dry_time_decide_se_o_traco_funde_ou_vela() {
    let junção = |dry_time_s: f32| -> f32 {
        let mut t = tela(dry_time_s);
        a_e_os_traços_de_c(&mut t);
        vertical(&mut t, 68.0);
        let mut acc = 0.0f32;
        for x in [58u32, 59, 60, 61, 68, 69, 70, 71] {
            for y in 80..110u32 {
                acc += f32::from(px(&t, LADO, x, y)[1]);
            }
        }
        acc / (8.0 * 30.0)
    };
    let molhada = junção(60.0);
    let seca = junção(10.0);
    assert!(
        molhada > seca + 40.0,
        "Dry Time 60 s: a A ainda molhada funde com o B (G {molhada:.1}); Dry Time 10 s: a A seca vela \
         (G {seca:.1}) — a diferença tem de ser a do aro duplo"
    );
}

/// Um traço recto de `de` a `ate`, um quadro por amostra.
fn traco(t: &mut PainterTool, de: [f32; 2], ate: [f32; 2]) {
    assert!(t.on_canvas_pointer(cp(de, PointerPhase::Down)));
    let passos = ((ate[0] - de[0]).hypot(ate[1] - de[1]) / 2.0).ceil() as usize;
    for k in 1..=passos {
        let f = k as f32 / passos as f32;
        let p = [de[0] + (ate[0] - de[0]) * f, de[1] + (ate[1] - de[1]) * f];
        t.on_canvas_pointer(cp(p, PointerPhase::Move));
        frame(t);
    }
    t.on_canvas_pointer(cp(ate, PointerPhase::Up));
}

/// Quantos texels mudaram FORA da faixa `x ∈ fora` entre duas fotografias da tela.
fn mudou_fora(antes: &[u8], depois: &[u8], fora: std::ops::RangeInclusive<u32>) -> usize {
    let mut n = 0;
    for y in 0..LADO {
        for x in (0..LADO).filter(|x| !fora.contains(x)) {
            let i = ((y * LADO + x) * 4) as usize;
            n += usize::from(antes[i..i + 4] != depois[i..i + 4]);
        }
    }
    n
}

/// ⭐⭐⭐ **Assar não muda UM byte da tela.** A nasce; C nasce 5 s depois (a sessão continua); 7 s depois
/// a A secou e a C não, e NADA assou ainda (o assar é do pen-down). O traço B no meio, longe dos dois,
/// assa a A no pen-down dele, e o pen-up re-renderiza a união inteira da sessão (o retângulo
/// cumulativo): a A assada e a C molhada têm de sair idênticas à fotografia de ANTES do assar.
///
/// **Mutações que sangram:** a base da sessão não receber a tela na zona assada; não zerar a cobertura.
#[test]
fn assar_uma_poca_seca_nao_muda_um_byte_da_tela() {
    let mut t = tela(10.0);
    vertical(&mut t, 60.0);
    espera(&mut t, 5.0);
    vertical(&mut t, 220.0);
    espera(&mut t, 7.0);
    let fw = LADO as usize;
    assert!(
        t.wet_session_continues()
            && t.paint.stroke_coverage[90 * fw + 60] > 0
            && t.paint.canvas_wet[90 * fw + 60] == 0,
        "a régua: a sessão continua, a A secou e ainda não assou"
    );
    let antes = t.canvas_rgba.as_ref().clone();
    vertical(&mut t, 140.0);
    assert_eq!(
        t.paint.stroke_coverage[90 * fw + 60],
        0,
        "a régua: o pen-down do B assou a A (a união esqueceu-a)"
    );
    assert!(px(&t, LADO, 140, 90)[1] < 250, "a régua: o B pintou");
    let mudou = mudou_fora(&antes, &t.canvas_rgba, 110..=170);
    assert_eq!(
        mudou, 0,
        "o bake do B re-renderizou a união e mudou {mudou} texels longe dele (a poça assada ou a molhada)"
    );
}

/// ⭐⭐⭐ **Assar nunca corta uma poça.** A, horizontal; 6 s depois A2 desce a partir da ponta direita de
/// A (fundem: uma poça só); 5 s depois a parte esquerda de A secou inteira e a A2 não. O pen-down de um
/// traço longe NÃO pode assar a parte seca: ela toca a molhada, e cortar a união dentro de uma poça muda
/// a vizinhança do lado molhado. O pen-up re-renderiza a união — a poça tem de sair idêntica.
///
/// **Mutação que sangra:** o elo entre células a `0` (cada célula seca assa sozinha).
#[test]
fn assar_nunca_corta_uma_poca_meio_seca() {
    let mut t = tela(10.0);
    traco(&mut t, [24.0, 60.0], [150.0, 60.0]);
    espera(&mut t, 6.0);
    traco(&mut t, [150.0, 60.0], [150.0, 200.0]);
    espera(&mut t, 5.0);
    let fw = LADO as usize;
    let seca = (40..90).all(|x| (40..80).all(|y| t.paint.canvas_wet[y * fw + x] == 0));
    assert!(
        t.wet_session_continues() && seca && t.paint.canvas_wet[150 * fw + 150] > 0,
        "a régua: a parte esquerda de A secou inteira ({seca}) e a A2 está molhada"
    );
    let antes = t.canvas_rgba.as_ref().clone();
    vertical(&mut t, 232.0);
    assert!(
        t.paint.stroke_coverage[60 * fw + 50] > 0,
        "a parte seca de A foi assada — e ela é da MESMA poça que a A2 molhada"
    );
    let mudou = mudou_fora(&antes, &t.canvas_rgba, 212..=255);
    assert_eq!(
        mudou, 0,
        "o bake do traço longe re-renderizou a poça meio seca e mudou {mudou} texels — o assar cortou-a"
    );
}

/// A tela inteira em PPM (`P6`), para OLHAR a fronteira molhado/seco (DIRETIVA §4).
fn grava_ppm(t: &PainterTool, nome: &str) {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/prova/secagem");
    std::fs::create_dir_all(&dir).expect("a pasta da prova");
    let mut bytes = format!("P6\n{LADO} {LADO}\n255\n").into_bytes();
    for px in t.canvas_rgba.as_chunks::<4>().0 {
        bytes.extend_from_slice(&px[..3]);
    }
    std::fs::write(dir.join(nome), bytes).expect("gravar a prova");
}

/// Sonda: as três imagens — (1) A e B molhados (fundem), (2) A seca com C molhado: B sobre A vela e um
/// B2 sobre C funde, (3) o mesmo desenho com o papel inteiro seco antes do B.
#[test]
#[ignore = "sonda: grava as imagens da fronteira molhado/seco em target/prova/secagem"]
fn sonda_as_imagens_da_secagem_por_poca() {
    let mut molhada = tela(10.0);
    vertical(&mut molhada, 60.0);
    vertical(&mut molhada, 68.0);
    grava_ppm(&molhada, "1_a_e_b_molhados.ppm");
    let mut viva = tela(10.0);
    a_e_os_traços_de_c(&mut viva);
    vertical(&mut viva, 68.0);
    vertical(&mut viva, 228.0);
    grava_ppm(&viva, "2_a_seca_c_molhado.ppm");
    let mut seca = tela(10.0);
    a_e_os_traços_de_c(&mut seca);
    espera(&mut seca, 70.0);
    vertical(&mut seca, 68.0);
    vertical(&mut seca, 228.0);
    grava_ppm(&seca, "3_tudo_seco.ppm");
}

/// Sonda: o preço do assar no pen-down, numa tela `4096²` com duas poças grandes (uma assa).
#[test]
#[ignore = "sonda: o custo do assa_as_pocas_secas a 4096²"]
fn sonda_o_custo_de_assar_a_4096() {
    let lado = 4096u32;
    let mut t = white_canvas(lado, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 120.0,
        watercolor: true,
        space_attenuation: false,
        ..tela(10.0).paint.brush
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    let risca = |t: &mut PainterTool, x: f32| {
        assert!(t.on_canvas_pointer(cp([x, 200.0], PointerPhase::Down)));
        let mut y = 200.0f32;
        while y < 3900.0 {
            y += 40.0;
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
            frame(t);
        }
        t.on_canvas_pointer(cp([x, 3900.0], PointerPhase::Up));
    };
    // A nasce, a C nasce 5 s depois (a sessão continua), e 6 s depois a A secou e a C não.
    risca(&mut t, 600.0);
    espera(&mut t, 5.0);
    risca(&mut t, 3400.0);
    espera(&mut t, 6.0);
    assert!(
        t.wet_session_continues(),
        "a régua: a C mantém a sessão viva"
    );
    // A 1.ª chamada assa a A (seca, longe da C molhada); a 2.ª só varre — o preço do pen-down comum.
    for rotulo in ["assa a poça A", "nada a assar"] {
        let t0 = std::time::Instant::now();
        let k = t.assa_as_pocas_secas();
        eprintln!(
            "ASSAR\t{rotulo}\t{k} células\t{:.2} ms",
            t0.elapsed().as_secs_f64() * 1e3
        );
    }
}

/// ⭐⭐⭐ **O gesto do dono (smoke de 2026-10-04): Dry Time 2 s, cinco traços SEPARADOS, um por
/// segundo — o 1.º tem de estar seco (assado) quando o 5.º começa.** Com o pincel de FÁBRICA da
/// aquarela (raio 10). A 1.ª versão do assar exigia ~65 px entre poças (células de 16 px e um elo de
/// quatro células), e traços a 40 px uns dos outros eram «uma poça só» — nada secava enquanto se
/// pintava ao lado.
#[test]
fn cinco_tracos_separados_cada_um_seca_no_seu_tempo() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (LADO * LADO * 4) as usize], LADO, LADO);
    t.set_paint_media(crate::PaintMedia::Watercolor);
    t.set_dry_time_s(2.0);
    let fw = LADO as usize;
    for (k, x) in [40.0f32, 80.0, 120.0, 160.0, 200.0].into_iter().enumerate() {
        if k == 4 {
            assert_eq!(
                t.paint.canvas_wet[90 * fw + 40],
                0,
                "a régua: 4 s depois, o papel sob o 1.º traço secou"
            );
        }
        vertical(&mut t, x);
        espera(&mut t, 1.0);
    }
    assert_eq!(
        t.paint.stroke_coverage[90 * fw + 40],
        0,
        "o 1.º traço continuou na sessão molhada (fundiria com um traço por cima) — ele secou há 2 s"
    );
    assert!(
        t.paint.stroke_coverage[90 * fw + 200] > 0,
        "a régua: o 5.º traço, molhado, continua na sessão"
    );
}

/// ⭐⭐⭐ **A [`separacao`](super::super::watercolor_secagem) é EXACTA na fronteira** — a varredura da
/// distância entre duas poças, píxel a píxel, com o pincel de fábrica (Ragged Edge 6 px), com Rewet
/// (a dissolução lê a base) e com Dilution (a água serrilhada). Em CADA distância: se a poça seca
/// assou, o pen-up do traço longe (que re-renderiza a união inteira) não muda um byte fora dele. E a
/// varredura tem de chegar a assar perto (a régua de que mede alguma coisa).
///
/// **Mutação que sangra:** a separação a `reach` só (sem a zona da base nem o deslocamento).
#[test]
fn a_separacao_das_pocas_e_exacta_na_fronteira() {
    type Ajuste = fn(&mut BrushSpec);
    let casos: [(&str, Ajuste); 3] = [
        ("fábrica", |_| {}),
        ("Rewet 0,6", |b| b.wet_rewet = 0.6),
        ("Dilution 0,5", |b| b.wet_dilution = 0.5),
    ];
    let fw = LADO as usize;
    for (nome, ajusta) in casos {
        let mut assou_mais_perto = None;
        for xc in 56..=120u32 {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; (LADO * LADO * 4) as usize], LADO, LADO);
            t.set_paint_media(crate::PaintMedia::Watercolor);
            ajusta(&mut t.paint.brush);
            t.set_dry_time_s(2.0);
            vertical(&mut t, 40.0);
            espera(&mut t, 1.0);
            vertical(&mut t, xc as f32);
            espera(&mut t, 1.25);
            if !t.wet_session_continues() || t.paint.stroke_coverage[90 * fw + 40] == 0 {
                continue;
            }
            let antes = t.canvas_rgba.as_ref().clone();
            vertical(&mut t, 224.0);
            let assou = t.paint.stroke_coverage[90 * fw + 40] == 0;
            let mudou = mudou_fora(&antes, &t.canvas_rgba, 196..=255);
            assert!(
                !assou || mudou == 0,
                "[{nome}] a poça seca a {xc} px assou e o re-render mudou {mudou} texels — a separação é curta"
            );
            if assou && assou_mais_perto.is_none() {
                assou_mais_perto = Some(xc);
            }
        }
        let perto =
            assou_mais_perto.unwrap_or_else(|| panic!("[{nome}] a régua: nada assou na varredura"));
        eprintln!("SEPARACAO [{nome}] o 2.º traço mais perto que deixou o 1.º assar: x = {perto}");
    }
}
