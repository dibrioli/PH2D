//! **O Smudge arrasta a tinta MOLHADA da sessão** (report do dono, 2026-09-29: *«o Smudge e o Rewet
//! não afetam a mancha de tinta quando a tinta está molhada»*; medido: `0` texels mudados — doc 44 §3).
//!
//! A fixtura é a do instrumento `diag_smudge_e_rewet_sobre_molhado`: um traço azul vertical molhado,
//! e um amarelo horizontal que o ATRAVESSA na mesma sessão com o Smudge. ⚠️ O pincel é o de FÁBRICA
//! na queda (a suave): com a queda constante e força `1` o arrasto copia o papel do início do traço
//! por cima de tudo, e o azul é APAGADO em vez de arrastado — a mesma lei do Smudge seco, e uma
//! fixtura onde o rasto não existe.

use super::*;

const AZUL: [f32; 3] = [0.25, 0.45, 0.95];
const AMARELO: [f32; 3] = [0.98, 0.90, 0.25];
const SIZE: u32 = 192;
const Y: u32 = 96;

fn pincel(cor: [f32; 3], smudge: f32, pigment: bool) -> BrushSpec {
    let fabrica = BrushSpec::default();
    BrushSpec {
        radius_px: 14.0,
        hardness: fabrica.hardness,
        falloff: fabrica.falloff,
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
        wet_smudge: smudge,
        pigment,
        pigment_mix: 1.0,
        ..Default::default()
    }
}

fn arma(t: &mut PainterTool, b: BrushSpec) {
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
}

/// Um traço de `de` a `ate`, em passos de 2 px, sem levantar a caneta.
fn traco(t: &mut PainterTool, de: [f32; 2], ate: [f32; 2]) {
    assert!(t.on_canvas_pointer(cp(de, PointerPhase::Down)));
    let n = ((ate[0] - de[0]).hypot(ate[1] - de[1]) / 2.0).ceil() as usize;
    for k in 1..=n {
        let f = k as f32 / n as f32;
        let p = [de[0] + (ate[0] - de[0]) * f, de[1] + (ate[1] - de[1]) * f];
        t.on_canvas_pointer(cp(p, PointerPhase::Move));
    }
    t.on_canvas_pointer(cp(ate, PointerPhase::Up));
}

/// Azul molhado em `x = 86`, e o amarelo a atravessá-lo NA MESMA SESSÃO.
fn cena(smudge: f32, pigment: bool) -> PainterTool {
    cena_espacada(smudge, pigment, BrushSpec::default().spacing)
}

/// A mesma cena com o amarelo a carimbar dabs de `spacing` (fracção do diâmetro).
fn cena_espacada(smudge: f32, pigment: bool, spacing: f32) -> PainterTool {
    let mut t = white_canvas(SIZE, 14.0);
    arma(&mut t, pincel(AZUL, 0.0, false));
    traco(&mut t, [86.0, 40.0], [86.0, 150.0]);
    arma(
        &mut t,
        BrushSpec {
            spacing,
            ..pincel(AMARELO, smudge, pigment)
        },
    );
    traco(&mut t, [40.0, 96.0], [160.0, 96.0]);
    t
}

/// Quanto o azul passa o vermelho: o amarelo lê `−113`, e todo arrasto de azul sobe-o.
fn azul(p: [u8; 4]) -> i32 {
    i32::from(p[2]) - i32::from(p[0])
}

fn luz(p: [u8; 4]) -> i32 {
    i32::from(p[0]) + i32::from(p[1]) + i32::from(p[2])
}

/// O rasto: à direita da faixa azul (`86 ± 14`), onde só o arrasto pode pôr azul.
const RASTO: std::ops::RangeInclusive<u32> = 108..=150;

#[test]
fn o_smudge_arrasta_a_tinta_molhada_num_rasto_continuo() {
    // Medido (2026-09-29), o ganho de azul sobre o traço sem Smudge, em TODO texel do rasto:
    // * antes da cura — `0` em todos (o Smudge não tocava na tinta molhada);
    // * com o arrasto escrito no plano da cor da sessão em vez do `antes` — riscas: texels a `0`
    //   entre os dabs (o depósito recompunha-os do `antes` e deitava o arrasto fora);
    // * com a cura — `13` a `44`, sem uma falha.
    // ⇒ a barra `8` fica no vale entre os dois lados, e é por TEXEL: um rasto às riscas reprova.
    //
    // ⚠️ E corre com os dabs ESPAÇADOS também (`0,6` do diâmetro, pior texel `14`): com o espaçamento
    // de fábrica cada texel é arrastado por ~7 dabs, e perder o PRIMEIRO arrasto de cada um (a
    // fotografia do `antes` a chegar DEPOIS dele) ficava escondido pelos seis seguintes — medido, a
    // mutação que tira a captura do destino lia igual ao bit ali e abria texels a `0` aqui.
    const BARRA: i32 = 8;
    for spacing in [BrushSpec::default().spacing, 0.6] {
        let (sem, com) = (
            cena_espacada(0.0, false, spacing),
            cena_espacada(1.0, false, spacing),
        );
        let ganho: Vec<(u32, i32)> = RASTO
            .map(|x| (x, azul(px(&com, SIZE, x, Y)) - azul(px(&sem, SIZE, x, Y))))
            .collect();
        let pior = ganho.iter().min_by_key(|g| g.1).copied().unwrap_or((0, 0));
        assert!(
            pior.1 >= BARRA,
            "espaçamento {spacing}: o Smudge tem de arrastar o azul molhado num rasto CONTÍNUO — o \
             pior texel ({}, {Y}) ganha {} (barra {BARRA}): {ganho:?}",
            pior.0,
            pior.1
        );
    }
}

#[test]
fn o_arrasto_nao_escurece_a_tinta() {
    // O arrasto leva o PAPEL (o transparente do plano da sessão, `0,0,0,0`) por cima da tinta. Em
    // alfa recto isso é média com PRETO: medido, o rasto saía `152,152,129` — um oliva sujo. Em
    // pré-multiplicado sai `219,235,141`.
    //
    // O CHÃO não é um número escolhido: é a mistura mais escura que as duas tintas dão sem arrasto
    // nenhum — o meio da sobreposição com o `Pigment` a `1` (o K–M a meias, `159,198,159`). Um rasto
    // é tinta amarela com azul DILUÍDO; nunca pode ser mais escuro do que as duas a meias.
    let chao = luz(px(&cena(0.0, true), SIZE, 86, Y));
    let com = cena(1.0, false);
    let escuro = RASTO
        .map(|x| (x, px(&com, SIZE, x, Y)))
        .min_by_key(|(_, p)| luz(*p))
        .unwrap_or((0, [255; 4]));
    assert!(
        luz(escuro.1) >= chao,
        "o arrasto escureceu a tinta: ({}, {Y}) lê {:?}, mais escuro que o azul e o amarelo a meias \
         (luz {chao})",
        escuro.0,
        escuro.1
    );
}

#[test]
fn o_primeiro_traco_da_sessao_nao_arrasta_a_propria_cor() {
    // Sem tinta molhada de traços ANTERIORES não há o que arrastar: o plano da cor da sessão tem de
    // sair AO BIT o de Smudge desligado (o seco é do `smear_wet_base`, que isto não toca).
    let so = |smudge: f32| {
        let mut t = white_canvas(SIZE, 14.0);
        arma(&mut t, pincel(AMARELO, smudge, false));
        traco(&mut t, [40.0, 96.0], [160.0, 96.0]);
        t.paint.stroke_color.clone()
    };
    let (sem, com) = (so(0.0), so(1.0));
    assert!(
        sem.iter().any(|&b| b > 0),
        "controlo: o traço depositou cor"
    );
    assert!(
        sem == com,
        "o 1.º traço de uma sessão arrastou a própria cor"
    );
}

#[test]
fn a_porta_do_arrasto_molhado_so_abre_sobre_tinta_de_tracos_anteriores() {
    // Arrastar o `antes` de um 1.º traço (que é papel) não muda um byte — o gate acima prova-o. O que
    // a porta `ha_tinta_da_sessao` guarda é a ROTA: sem ela o 1.º traço passava pelo depósito da
    // mistura, que ignora a prioridade do mixer, e com o `Charge < 1` mudava só por o Smudge estar
    // ligado. ⚠️ Ali a cor MUDA de qualquer maneira (o Smudge arrasta também a reserva do mixer, e é
    // devido), logo uma comparação de bytes não separa as duas rotas — a régua é a PORTA, lida pelo
    // caminho do produto, nos três estados da sessão.
    let porta = |t: &PainterTool| t.paint.wet_mistura.ha_tinta_da_sessao;
    let mut t = white_canvas(SIZE, 14.0);
    arma(&mut t, pincel(AZUL, 1.0, false));
    traco(&mut t, [86.0, 40.0], [86.0, 150.0]);
    assert!(
        !porta(&t),
        "o 1.º traço de uma sessão não tem tinta molhada alheia"
    );
    assert!(
        t.arrasto_da_sessao().is_none(),
        "e o arrasto molhado fica desligado nele"
    );
    t.paint.brush.wet_rewet = 1.0;
    assert!(
        t.agua_da_sessao().is_none(),
        "e a água do Rewet também — não há tinta molhada alheia a redissolver"
    );
    arma(&mut t, pincel(AMARELO, 1.0, false));
    traco(&mut t, [40.0, 96.0], [160.0, 96.0]);
    assert!(
        porta(&t),
        "o 2.º traço da MESMA sessão arrasta a tinta molhada do 1.º"
    );
    t.paint.brush.wet_rewet = 1.0;
    assert!(
        t.agua_da_sessao().is_some(),
        "e a água do Rewet redissolve-a pela MESMA porta"
    );
    for _ in 0..300 {
        t.paint_tick(0.5);
    }
    traco(&mut t, [40.0, 120.0], [160.0, 120.0]);
    assert!(
        !porta(&t),
        "depois de a sessão secar o traço seguinte abre outra — a tinta de antes é SECA"
    );
}

#[test]
fn fora_da_selecao_o_arrasto_nao_mexe_na_tinta_de_antes() {
    // O Smudge seco repõe a base fora da seleção pelo `keep` (`smear_wet_base`); o molhado faz o
    // mesmo no `antes`. ⚠️ A régua é o PLANO, lido pelo caminho do produto: fora da seleção o traço
    // não deposita nem recompõe, logo um arrasto não pesado ficava INVISÍVEL ali — até o traço
    // seguinte (ou este, a voltar para trás) levar essa tinta arrastada para dentro da zona pintável.
    // A fixtura põe o azul FORA da seleção (`x ≥ 104`), senão o arrasto moveria papel sobre papel e
    // o gate não afirmava nada.
    const CORTE: u32 = 104;
    let mut t = white_canvas(SIZE, 14.0);
    arma(&mut t, pincel(AZUL, 0.0, false));
    traco(&mut t, [118.0, 40.0], [118.0, 150.0]);
    let antes_do_traco = t.paint.stroke_color.clone();
    t.paint.selection_mask = std::sync::Arc::new(
        (0..SIZE * SIZE)
            .map(|i| if i % SIZE >= CORTE { 0 } else { 255 })
            .collect(),
    );
    t.paint.selection_active = true;
    arma(&mut t, pincel(AMARELO, 1.0, false));
    traco(&mut t, [40.0, 96.0], [170.0, 96.0]);
    let planos = &t.paint.wet_mistura;
    let (mut vistos, mut com_tinta) = (0usize, 0usize);
    for y in 80..=112u32 {
        for x in CORTE..SIZE {
            let i = (y * SIZE + x) as usize;
            if !planos.capturado[i] {
                continue;
            }
            vistos += 1;
            let de_antes = &antes_do_traco[i * 4..i * 4 + 4];
            com_tinta += usize::from(de_antes[3] > 0);
            assert_eq!(
                &planos.antes[i * 4..i * 4 + 4],
                de_antes,
                "({x},{y}) fora da seleção: o arrasto mexeu na tinta de antes do traço"
            );
        }
    }
    assert!(
        vistos > 500 && com_tinta > 200,
        "controlo: a fixtura tem de pôr o arrasto sobre TINTA fora da seleção ({vistos} capturados, \
         {com_tinta} com tinta)"
    );
}

#[test]
fn o_keep_parcial_pesa_o_arrasto() {
    // A lei, na porta: com `keep = k` o `antes` fica a `k` do caminho entre o de antes do arrasto e o
    // arrastado, na MESMA mistura pré-multiplicada do arrasto; `k = 0` não mexe; sem portões é o
    // arrasto inteiro.
    use super::super::watercolor_mistura::PlanosDaMistura;
    const W: usize = 32;
    let buf: Vec<u8> = (0..W * W)
        .flat_map(|i| {
            if i % W >= 16 {
                [60, 110, 240, 255]
            } else {
                [0, 0, 0, 0]
            }
        })
        .collect();
    let raio = BrushSpec {
        radius_px: 6.0,
        ..BrushSpec::default()
    };
    let corre = |guarda: Option<&dyn Fn(usize) -> f32>| {
        let mut p = PlanosDaMistura::default();
        p.garante(W * W);
        p.novo_traco();
        p.arrasta(
            &buf,
            (W, W),
            [16.0, 16.0],
            None,
            &raio,
            1.0,
            [false; 2],
            guarda,
        );
        p.arrasta(
            &buf,
            (W, W),
            [19.0, 16.0],
            Some([3.0, 0.0]),
            &raio,
            1.0,
            [false; 2],
            guarda,
        );
        (p.antes, p.capturado)
    };
    let (cheio, capturado) = corre(None);
    let (nada, _) = corre(Some(&|_| 0.0));
    let (meio, _) = corre(Some(&|_| 0.5));
    let mut mudados = 0;
    for i in (0..W * W).filter(|&i| capturado[i]) {
        let px = |v: &[u8]| [v[i * 4], v[i * 4 + 1], v[i * 4 + 2], v[i * 4 + 3]];
        let (a, b) = (px(&buf), px(&cheio));
        if a != b {
            mudados += 1;
        }
        assert_eq!(px(&nada), a, "texel {i}: com keep 0 o arrasto mexeu");
        let (aa, ab) = (f32::from(a[3]) / 255.0, f32::from(b[3]) / 255.0);
        let na = aa + (ab - aa) * 0.5;
        let m = px(&meio);
        assert!(
            (f32::from(m[3]) - na * 255.0).abs() <= 1.0,
            "texel {i}: o alfa com keep ½ ({}) não está a meio ({})",
            m[3],
            na * 255.0
        );
        if na > 0.0 {
            for c in 0..3 {
                let pre = (f32::from(a[c]) * aa
                    + (f32::from(b[c]) * ab - f32::from(a[c]) * aa) * 0.5)
                    / na;
                assert!(
                    (f32::from(m[c]) - pre).abs() <= 1.0,
                    "texel {i}, canal {c}: com keep ½ lê {} e o meio PRÉ-MULTIPLICADO é {pre}",
                    m[c]
                );
            }
        }
    }
    assert!(
        mudados > 20,
        "controlo: o arrasto cheio mexeu no plano ({mudados} texels)"
    );
}

/// Azul em `x = 86` (molhado, ou seco se `secar`), e o amarelo com o Rewet a atravessá-lo.
fn cena_rewet(rewet: f32, secar: bool) -> PainterTool {
    let mut t = white_canvas(SIZE, 14.0);
    arma(&mut t, pincel(AZUL, 0.0, false));
    traco(&mut t, [86.0, 40.0], [86.0, 150.0]);
    if secar {
        for _ in 0..300 {
            t.paint_tick(0.5);
        }
    }
    arma(
        &mut t,
        BrushSpec {
            wet_rewet: rewet,
            ..pincel(AMARELO, 0.0, false)
        },
    );
    traco(&mut t, [40.0, 96.0], [160.0, 96.0]);
    t
}

/// O verde de uma mistura de azul com amarelo: o verde é o canal DOMINANTE, com folga sobre os dois.
fn e_verde(p: [u8; 4]) -> bool {
    let [r, g, b, _] = p.map(i32::from);
    g >= r + 20 && g >= b + 20
}

#[test]
fn o_rewet_mistura_a_tinta_molhada() {
    // Medido (2026-09-29): o meio da faixa lia `253,245,140` com e sem o Rewet (o amarelo TAPA o azul
    // no plano da sessão, e a água do composite só via a tinta SECA). Com a cura, `188,226,174`.
    let sem = px(&cena_rewet(0.0, false), SIZE, 86, Y);
    assert!(
        !e_verde(sem) && sem[0] > sem[2] + 60,
        "controlo: sem Rewet o amarelo molhado tapa o azul ({sem:?})"
    );
    let com = px(&cena_rewet(1.0, false), SIZE, 86, Y);
    assert!(
        e_verde(com),
        "com o Rewet, a água do amarelo tem de redissolver o azul molhado e misturá-los — lê {com:?}"
    );
}

#[test]
fn o_rewet_espalha_a_tinta_molhada_como_espalha_a_seca() {
    // A régua é o LADO APROVADO: o Rewet sobre tinta SECA (o dono aprovou-o em 2026-07-06) espalha o
    // azul para fora da faixa até ao raio do Spread. Ganho de azul sobre o traço sem Rewet, menos o
    // ganho LONGE da tinta (o clarear da própria lavagem molhada, `27`, igual nos dois):
    // * seco — sobe a partir de `x = 63`, some depois de `x = 108`;
    // * molhado, com a cura — a MESMA faixa, `63..108`, a menos de `8` por texel;
    // * molhado, antes da cura — zero fora da faixa (o azul molhado não se mexia).
    // ⇒ por texel, nas duas orlas de fora da faixa azul: o molhado a `≤ 12` do seco (folga sobre o
    //   `8` medido), e o espalhamento tem de EXISTIR (o seco passa `20` em `x = 104`).
    const FOLGA: i32 = 12;
    let orla = |secar: bool| {
        let (sem, com) = (cena_rewet(0.0, secar), cena_rewet(1.0, secar));
        let g = move |x: u32| azul(px(&com, SIZE, x, Y)) - azul(px(&sem, SIZE, x, Y));
        let longe = g(140);
        move |x: u32| g(x) - longe
    };
    let (seco, molhado) = (orla(true), orla(false));
    assert!(
        seco(104) >= 20,
        "controlo: o Rewet seco espalha o azul até x = 104"
    );
    let mut pior = (0, 0);
    for x in (60..=71u32).chain(101..=112) {
        let d = (molhado(x) - seco(x)).abs();
        if d > pior.1 {
            pior = (x, d);
        }
    }
    assert!(
        pior.1 <= FOLGA,
        "o Rewet molhado não espalha como o seco: em ({}, {Y}) o molhado ganha {} e o seco {}",
        pior.0,
        molhado(pior.0),
        seco(pior.0)
    );
}

#[test]
fn a_agua_mistura_pela_lei_da_agua_e_o_botao_pela_do_pigmento() {
    // O depósito passa os dois pesos pela MESMA porta do composite sobre tinta seca
    // (`alvo_sobre_seco`): o botão (Pigment, Smudge) pelo Kubelka–Munk, a água do Rewet pela lei da
    // água. O controlo: as duas leis dão cores DIFERENTES para o mesmo par, senão isto não afirma nada.
    use super::super::watercolor_mistura::{PlanosDaMistura, alvo_sobre_seco, deposita};
    const AZ: [u8; 4] = [60, 110, 240, 255];
    const AM: [u8; 3] = [250, 230, 64];
    let unit = |p: [u8; 3]| p.map(|c| f32::from(c) / 255.0);
    let corre = |pesos: (f32, f32)| {
        let mut p = PlanosDaMistura::default();
        p.garante(1);
        p.novo_traco();
        let mut buf = AZ.to_vec();
        deposita(&mut buf, &mut p, 0, AM, 1.0, pesos, None);
        [buf[0], buf[1], buf[2]]
    };
    let esperado = |botao: f32, agua: f32| {
        alvo_sobre_seco(unit([AZ[0], AZ[1], AZ[2]]), unit(AM), 0.5, botao, agua)
            .map(|c| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8)
    };
    let (km, agua) = (esperado(1.0, 0.0), esperado(0.0, 1.0));
    assert!(
        km.iter().zip(&agua).any(|(a, b)| a.abs_diff(*b) > 8),
        "controlo: as duas leis dão cores distintas para o azul com o amarelo ({km:?} · {agua:?})"
    );
    let perto = |a: [u8; 3], b: [u8; 3]| a.iter().zip(&b).all(|(x, y)| x.abs_diff(*y) <= 1);
    let (lido_km, lido_agua) = (corre((1.0, 0.0)), corre((0.0, 1.0)));
    assert!(
        perto(lido_km, km),
        "o botão mistura pelo K–M: lê {lido_km:?}, esperado {km:?}"
    );
    assert!(
        perto(lido_agua, agua),
        "a água do Rewet mistura pela lei da água: lê {lido_agua:?}, esperado {agua:?}"
    );
}
