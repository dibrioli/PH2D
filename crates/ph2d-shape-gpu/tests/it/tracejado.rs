//! ⭐⭐ **O TRACEJADO SOB ESCALA NÃO UNIFORME** (doc 121 §9.9) — o eixo percorrido no ecrã contra a lei
//! da casa (o Vello sobre a geometria transformada, caneta e padrão `× √|det|`), e o contorno
//! calculado contra o caminho pixel a pixel.
//!
//! ```text
//! cargo test -p ph2d-shape-gpu --test it -- --ignored --nocapture tracejado
//! ```

use ph2d_gpu::GpuContext;
use ph2d_shape_gpu::{
    Copias, EixoItem, FillRule, ShapeGeometry, ShapeInput, ShapeInstance, ShapePass, ShapeView,
    StrokeInput,
};
use ph2d_vector::{BezPath, Cap, Join, Stroke};

use super::paridade_com_o_vello::{
    Copia, Forma, LADO, basis, bytes_de_textura, circulo, copias, corre, esticadas, estrela, gpu,
    pelo_passe_ajustado, pelo_passe_com, pelo_passe_observado, textura, zigue_zague,
};

/// ⭐ **A barra, do VALE medido** (2026-10-02, RTX, meio-float; arnês
/// `docs/Motion Nodes/ferramentas/mutacao_o_tracejado_no_ecra_2026-10-02.py`): a rota que shipa lê
/// alfa `≤ 84` · cor `≤ 89` em todas as famílias; as 16 mutações do percurso leem alfa `126`–`255`.
/// ⇒ `100`/`100`, a barra das curvas. ⚠️ A FRACÇÃO de pixels com alfa `> 1` (a régua do traço
/// esticado contínuo) NÃO separa aqui: o controlo conforme — o caminho pré-expandido, intocado — lê
/// `5,7 %` (cada traço com pontas redondas é borda curva), e a emenda que falta lê `2,4 %`.
const BARRA: u8 = 100;

/// Uma estrela com um FURO hexagonal: dois sub-caminhos fechados de comprimentos diferentes. O ajuste
/// no ecrã fecha o mais LONGO num número inteiro de períodos — o de dentro não, e é nele que o último
/// traço EMENDA no primeiro. ⚠️ Hexágono e não estrela: numa quina de `120°` a faixa SERVE (numa ponta
/// de estrela o recuo passa sempre dos pedaços), e é aí que o recuo da emenda decide alguma coisa.
pub(super) fn estrela_com_furo() -> BezPath {
    let mut bp = estrela();
    for i in 0..6 {
        let a = std::f64::consts::PI * f64::from(i) / 3.0 + 0.2;
        let p = (0.14 * a.cos(), 0.14 * a.sin());
        if i == 0 {
            bp.move_to(p);
        } else {
            bp.line_to(p);
        }
    }
    bp.close_path();
    bp
}

fn casos<'a>(
    est: &'a BezPath,
    circ: &'a BezPath,
    zz: &'a BezPath,
    furo: &'a BezPath,
) -> Vec<(&'static str, Forma<'a>, Vec<Copia>)> {
    let forma = |bp: &'a BezPath, linha: Option<&'a BezPath>, s: Stroke, cor: [f32; 4]| Forma {
        bp,
        linha,
        regra: FillRule::NonZero,
        marcas: None,
        traco: Some((s, cor)),
    };
    vec![
        (
            "estrela tracejada esticada",
            forma(
                est,
                None,
                Stroke::new(0.06)
                    .with_join(Join::Miter)
                    .with_dashes(0.0, [0.12, 0.08]),
                [0.1, 0.1, 0.1, 1.0],
            ),
            esticadas(40, 40.0, 220.0, 21),
        ),
        // Traços mais longos que um troço: a faixa e a junta DENTRO de um traço, e a emenda no início.
        (
            "estrela de tracos longos esticada",
            forma(
                est,
                None,
                Stroke::new(0.05)
                    .with_join(Join::Miter)
                    .with_dashes(0.0, [0.9, 0.25]),
                [0.6, 0.1, 0.4, 1.0],
            ),
            esticadas(40, 40.0, 220.0, 22),
        ),
        (
            "circulo tracejado esticado, pontas redondas",
            forma(
                circ,
                None,
                Stroke::new(0.08)
                    .with_caps(Cap::Round)
                    .with_dashes(0.0, [0.15, 0.1]),
                [0.9, 0.2, 0.1, 1.0],
            ),
            esticadas(40, 30.0, 200.0, 23),
        ),
        // As pontas do INÍCIO e do FIM de cada traço DIFERENTES (o kurbo põe a `start_cap` e a
        // `end_cap` em cada traço) — sem esta família a troca das duas passava (mutação T8).
        // ⚠️ O fim QUADRADO e não rente: um traço que acaba a menos de meia largura depois de uma
        // quina sai do traçador da casa com uma MORDIDA no lado de dentro (a junta interior passa
        // pelo pivô e o pedaço curto cruza-se; a régua EXACTA — o kurbo a expandir, o Vello só a
        // preencher — tem a mesma mordida), e o passe desenha a união verdadeira: `140` de alfa num
        // pixel (medido 02/10, uma cópia, pedaço de `9,75 px` com raio `11,4`). Divergência
        // DECLARADA (doc 121 §9.9); a ponta quadrada cobre-a e a família continua a distinguir as
        // duas pontas.
        (
            "zigue-zague tracejado esticado, pontas diferentes",
            forma(
                zz,
                Some(zz),
                Stroke::new(0.07)
                    .with_join(Join::Bevel)
                    .with_start_cap(Cap::Round)
                    .with_end_cap(Cap::Square)
                    .with_dashes(0.0, [0.14, 0.1]),
                [0.2, 0.2, 0.7, 1.0],
            ),
            esticadas(40, 40.0, 220.0, 27),
        ),
        (
            "zigue-zague tracejado esticado, chanfro e pontas quadradas",
            forma(
                zz,
                Some(zz),
                Stroke::new(0.07)
                    .with_join(Join::Bevel)
                    .with_caps(Cap::Square)
                    .with_dashes(0.0, [0.1, 0.06]),
                [0.1, 0.5, 0.2, 1.0],
            ),
            esticadas(40, 40.0, 220.0, 24),
        ),
        // ⭐ A EMENDA: desde o ajuste no ecrã o contorno mais longo nunca emenda (a folga põe o fim no
        // último vão); o de DENTRO emenda conforme o esticão — pontas quadradas e esquadria, para a
        // emenda (junta) e a falta dela (duas pontas) desenharem diferente.
        (
            "estrela com furo tracejada esticada",
            forma(
                furo,
                None,
                Stroke::new(0.05)
                    .with_join(Join::Miter)
                    .with_caps(Cap::Square)
                    .with_dashes(0.0, [0.17, 0.09]),
                [0.5, 0.1, 0.1, 1.0],
            ),
            esticadas(40, 40.0, 220.0, 28),
        ),
        // CONTROLO: as mesmas estrelas CONFORMES vão pelo contorno pré-expandido (o do Vello ao bit).
        (
            "estrela tracejada conforme",
            forma(
                est,
                None,
                Stroke::new(0.06)
                    .with_join(Join::Miter)
                    .with_dashes(0.0, [0.12, 0.08]),
                [0.1, 0.1, 0.1, 1.0],
            ),
            copias(40, 40.0, 220.0, 25),
        ),
    ]
}

/// ⭐⭐ **O passe traceja como a casa** — pixel a pixel contra o Vello.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn o_tracejado_esticado_desenha_o_que_o_vello_desenha() {
    let (est, circ, zz, furo) = (estrela(), circulo(), zigue_zague(), estrela_com_furo());
    // Todas as famílias medidas antes de reprovar — o placar inteiro é o que decide uma barra.
    let mut falhas = Vec::new();
    for (nome, forma, cs) in &casos(&est, &circ, &zz, &furo) {
        let Some(d) = corre(nome, forma, cs) else {
            eprintln!("sem adaptador — nada a medir");
            return;
        };
        assert!(
            d.pixels_com_tinta > 1000,
            "{nome}: a fixtura quase nao desenha"
        );
        eprintln!(
            "  PLACAR {nome}: alfa {} · cor {} · {} px acima de 1",
            d.alfa_max, d.cor_max, d.alfa_acima_de_1
        );
        if d.alfa_max > BARRA || d.cor_max > BARRA {
            falhas.push(format!("{nome}: {d:?}"));
        }
    }
    assert!(
        falhas.is_empty(),
        "o passe traceja outra coisa que a casa: {falhas:#?}"
    );
}

/// ⭐⭐ **O contorno calculado traceja o que o caminho pixel a pixel traceja** — uma régua de PAR: as
/// duas rotas são a mesma lei, e o que sobra é arredondamento. E todas as cópias têm de ter ganho o
/// contorno (senão a imagem igual não prova que ele correu).
#[test]
#[ignore = "precisa de adapter de GPU"]
fn o_tracejado_calculado_desenha_o_que_o_pixel_desenha() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adaptador — nada a medir");
        return;
    };
    let (est, circ, zz, furo) = (estrela(), circulo(), zigue_zague(), estrela_com_furo());
    let mut falhas = Vec::new();
    for (nome, forma, cs) in &casos(&est, &circ, &zz, &furo) {
        let fmt = wgpu::TextureFormat::Rgba16Float;
        let (calc, com) = pelo_passe_com(&gpu, forma, cs, fmt, true, 4)
            .pop()
            .expect("um quadro");
        let (pixel, sem) = pelo_passe_com(&gpu, forma, cs, fmt, false, 1)
            .pop()
            .expect("um quadro");
        let n = u32::try_from(cs.len()).expect("cabem");
        assert_eq!(sem, 0, "{nome}: CONTROLO — desligado, nenhuma o ganha");
        let (mut pior, mut acima, mut tinta) = (0u8, 0usize, 0usize);
        for (x, y) in calc.as_chunks::<4>().0.iter().zip(pixel.as_chunks::<4>().0) {
            let d = x[3].abs_diff(y[3]);
            pior = pior.max(d);
            acima += usize::from(d > 1);
            tinta += usize::from(x[3] > 0 || y[3] > 0);
        }
        eprintln!(
            "  PAR {nome}: alfa max {pior} · {acima} px > 1 · {tinta} px · {com}/{n} com contorno"
        );
        assert!(tinta > 1000, "{nome}: a fixtura quase nao desenha");
        if com != n || pior > 2 {
            falhas.push(format!(
                "{nome}: alfa {pior}, {acima} px, {com}/{n} com contorno"
            ));
        }
    }
    assert!(
        falhas.is_empty(),
        "os dois caminhos tracejam diferente: {falhas:#?}"
    );
}

/// ⭐ doc 121 §9.10 — **só um eixo com troço TRACEJADO pede a variante COMPLETA.** Inline, o ramo do
/// tracejado dobrava os registos do fragmento e do `cs_escreve` em TODA a cena (iGPU `56 → 128`
/// VGPRs, as estrelas esticadas `1,74 → 2,45 ms`). Os gates de pixel só provam que a cena tracejada
/// escolhe a completa: escolhê-la sempre desenha a mesma imagem, mais devagar — esta régua é a que vê.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn so_um_eixo_tracejado_pede_a_variante_completa() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adaptador — nada a medir");
        return;
    };
    let est = estrela();
    let prepara = |s: &Stroke| {
        ShapeGeometry::prepare(&ShapeInput {
            fill: Some((&est, FillRule::NonZero)),
            strokes: vec![StrokeInput {
                path: &est,
                style: s,
                color: [0.1, 0.1, 0.1, 1.0],
            }],
            stroke_fills: vec![],
        })
        .expect("a forma prepara")
    };
    let continua = prepara(&Stroke::new(0.06));
    let tracejada = prepara(&Stroke::new(0.06).with_dashes(0.0, [0.12, 0.08]));
    // CONTROLO: a contínua TEM eixo — sem ele, a enxuta seria escolhida por não haver nada a percorrer.
    assert!(!continua.eixo.is_empty() && !continua.eixo.iter().any(EixoItem::tracejado));
    assert!(tracejada.eixo.iter().any(EixoItem::tracejado));
    let mut p = ShapePass::new(&gpu, wgpu::TextureFormat::Rgba16Float);
    p.set_geometries(&gpu, [(1u32, &continua)]);
    assert!(!p.usa_o_tracejado(), "sem tracejado, a variante ENXUTA");
    p.set_geometries(&gpu, [(1u32, &continua), (2u32, &tracejada)]);
    assert!(
        p.usa_o_tracejado(),
        "uma geometria tracejada no conjunto pede a COMPLETA"
    );
    p.set_geometries(&gpu, [(1u32, &continua)]);
    assert!(!p.usa_o_tracejado(), "sem ela, volta à ENXUTA");
}

/// ⭐ doc 121 §9.13 — **numa cena tracejada a PLACA escolhe a variante do desenho por quadro**: a
/// completa (o fragmento a `128` VGPRs na iGPU) só quando uma cópia tracejada vai pixel a pixel — o 1.º
/// quadro, antes da capacidade medida, e as cópias que não cabem nas células; no regime, a ENXUTA. Os
/// gates de pixel não vêem «completa sempre» (a mesma imagem, mais devagar): esta régua é a que vê. E
/// a metade das células desenha o MESMO que todas (a completa a tracejar as que ficaram de fora).
#[test]
#[ignore = "precisa de adapter de GPU"]
fn a_placa_escolhe_a_variante_completa_so_quando_um_tracejado_vai_pixel_a_pixel() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adaptador — nada a medir");
        return;
    };
    let (est, circ, zz, furo) = (estrela(), circulo(), zigue_zague(), estrela_com_furo());
    let todos = casos(&est, &circ, &zz, &furo);
    let (nome, forma, cs) = &todos[0];
    let n = u32::try_from(cs.len()).expect("cabem");
    let fmt = wgpu::TextureFormat::Rgba16Float;
    let corre = |celulas_no_maximo: u64| {
        let mut por_quadro = Vec::new();
        let quadros = pelo_passe_observado(
            &gpu,
            forma,
            &[(cs.as_slice(), 4)],
            fmt,
            (true, 0.0, celulas_no_maximo),
            &mut |g, p| por_quadro.push(p.copias_por_variante(g)),
        );
        (quadros, por_quadro)
    };
    let (livre, v_livre) = corre(u64::MAX);
    eprintln!("  {nome}: (enxuta, completa) por quadro {v_livre:?}");
    // CONTROLO: o 1.º quadro, sem capacidade, desenha tudo pixel a pixel — pela completa.
    assert_eq!(v_livre[0], (0, n), "{nome}: o 1.º quadro pede a COMPLETA");
    let (_, com, _) = livre.last().expect("quadros");
    assert_eq!(*com, n, "{nome}: no regime toda cópia tem contorno");
    assert_eq!(
        *v_livre.last().expect("quadros"),
        (n, 0),
        "{nome}: no regime, a ENXUTA"
    );
    let (_, _, (pedido, _)) = livre.last().expect("quadros");
    let (metade, v_metade) = corre(pedido / 2);
    let (img_metade, com_metade, _) = metade.last().expect("quadros");
    assert!(
        *com_metade > 0 && *com_metade < n,
        "{nome}: CONTROLO — o tecto morde e alguma cópia continua nas células ({com_metade}/{n})"
    );
    assert_eq!(
        *v_metade.last().expect("quadros"),
        (0, n),
        "{nome}: uma cópia tracejada sem células pede a COMPLETA"
    );
    let pior = livre
        .last()
        .expect("quadros")
        .0
        .as_chunks::<4>()
        .0
        .iter()
        .zip(img_metade.as_chunks::<4>().0)
        .map(|(a, b)| a[3].abs_diff(b[3]))
        .max()
        .unwrap_or(0);
    assert!(
        pior <= 2,
        "{nome}: metade das células desenha outra coisa (alfa {pior})"
    );
}

/// A geometria de uma fixtura, como o passe a recebe.
fn geometria(forma: &Forma<'_>) -> ShapeGeometry {
    let traco = forma.traco.as_ref().map(|(s, cor)| StrokeInput {
        path: forma.linha.unwrap_or(forma.bp),
        style: s,
        color: *cor,
    });
    ShapeGeometry::prepare(&ShapeInput {
        fill: Some((forma.bp, forma.regra)),
        strokes: traco.into_iter().collect(),
        stroke_fills: forma.marcas.into_iter().collect(),
    })
    .expect("a forma prepara")
}

/// Um quadro de `cs` cópias da geometria `h`, submetido e esperado; devolve os bytes da camada.
fn um_quadro(
    gpu: &GpuContext,
    p: &mut ShapePass,
    tex: &wgpu::Texture,
    h: u32,
    cs: &[Copia],
) -> Vec<u8> {
    let insts: Vec<ShapeInstance> = cs
        .iter()
        .map(|c| ShapeInstance {
            pos: c.pos,
            size: [c.lado, c.lado * c.aspecto],
            basis: basis(c.ang),
            anchor: [0.0, 0.0],
            geometry: h,
            _pad: 0,
            tint: c.tint,
        })
        .collect();
    p.upload_instances(gpu, &insts);
    let copias = p.uploaded().expect("carregou").clone();
    let vista = tex.create_view(&wgpu::TextureViewDescriptor::default());
    #[expect(clippy::cast_precision_loss, reason = "LADO é 512")]
    let alvo = [LADO as f32, LADO as f32];
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    p.draw(
        gpu,
        &mut enc,
        &vista,
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        ShapeView {
            lin: [1.0, 0.0, 0.0, 1.0],
            t: [0.0, 0.0],
            alvo,
        },
        Copias {
            buffer: &copias,
            count: u32::try_from(insts.len()).expect("cabem"),
        },
    );
    gpu.queue.submit(Some(enc.finish()));
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    bytes_de_textura(gpu, tex, 8)
}

/// ⭐ doc 121 §9.15 (c2) — **uma cena NOVA mede a capacidade antes do 1.º quadro dela.** A leitura do
/// total é assíncrona e chega dois quadros depois; até lá cada cópia ia pixel a pixel — numa cena
/// tracejada pela variante COMPLETA (`76` ms por quadro na sonda da iGPU, contra `1,27` no regime). Com
/// a medida no início o 1.º quadro já é o do regime: todas as cópias nas células, a ENXUTA, e a MESMA
/// imagem byte a byte. E outra cena (as geometrias carregadas mudam, com `4×` as cópias — o que passa
/// de qualquer capacidade medida) mede outra vez. CONTROLO: sem a medida, o 1.º quadro é o pixel a pixel.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn a_cena_nova_mede_a_capacidade_antes_do_primeiro_quadro() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adaptador — nada a medir");
        return;
    };
    let (est, circ, zz, furo) = (estrela(), circulo(), zigue_zague(), estrela_com_furo());
    let todos = casos(&est, &circ, &zz, &furo);
    let (nome, forma, cs) = &todos[0];
    let g = geometria(forma);
    let n = u32::try_from(cs.len()).expect("cabem");
    let fmt = wgpu::TextureFormat::Rgba16Float;
    let tex = textura(&gpu, wgpu::TextureUsages::RENDER_ATTACHMENT, fmt);
    let novo = |mede: bool| {
        let mut p = ShapePass::new(&gpu, fmt);
        p.area_minima_conforme(0.0);
        p.mede_a_capacidade_no_inicio(mede);
        p.set_geometries(&gpu, [(7u32, &g)]);
        p
    };
    // CONTROLO: sem a medida no início, o 1.º quadro vai pixel a pixel, pela completa.
    let mut sem = novo(false);
    let _ = um_quadro(&gpu, &mut sem, &tex, 7, cs);
    assert_eq!(
        sem.copias_por_variante(&gpu),
        (0, n),
        "{nome}: CONTROLO — o 1.º quadro sem a medida"
    );
    let mut p = novo(true);
    let primeiro = um_quadro(&gpu, &mut p, &tex, 7, cs);
    assert_eq!(
        p.copias_com_contorno(&gpu, n).0,
        n,
        "{nome}: o 1.º quadro tem todas nas células"
    );
    assert_eq!(
        p.copias_por_variante(&gpu),
        (n, 0),
        "{nome}: o 1.º quadro desenha pela ENXUTA"
    );
    let (pedido, cap) = p.celulas_do_ultimo_quadro(&gpu);
    assert!(
        pedido > 0 && pedido <= cap,
        "{nome}: celulas {pedido} de {cap} no 1.º quadro"
    );
    let mut regime = Vec::new();
    for _ in 0..3 {
        regime = um_quadro(&gpu, &mut p, &tex, 7, cs);
    }
    assert!(
        primeiro.iter().any(|&b| b != 0),
        "{nome}: a fixtura nao desenha"
    );
    assert!(
        primeiro == regime,
        "{nome}: o 1.º quadro desenha outra coisa que o regime"
    );
    // Outra cena: outra chave e `4×` as cópias.
    let muitas: Vec<Copia> = (0..4).flat_map(|_| cs.iter().cloned()).collect();
    let m = u32::try_from(muitas.len()).expect("cabem");
    p.set_geometries(&gpu, [(8u32, &g)]);
    let _ = um_quadro(&gpu, &mut p, &tex, 8, &muitas);
    assert_eq!(
        p.copias_com_contorno(&gpu, m).0,
        m,
        "{nome}: a cena nova mediu outra vez"
    );
    assert_eq!(
        p.copias_por_variante(&gpu),
        (m, 0),
        "{nome}: a cena nova pela ENXUTA"
    );
}

/// ⭐ doc 121 §9.15 (d) — **o prefixo das células por SUBGRUPO é o MESMO que o de memória de grupo**, byte
/// a byte: a soma é de inteiros, por outra ordem. Nas sete famílias do tracejado, no regime. Sem
/// `Features::SUBGROUP` o passe só tem um caminho e o gate diz que não mediu nada.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn o_prefixo_por_subgrupo_e_o_mesmo_que_o_de_memoria_de_grupo() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adaptador — nada a medir");
        return;
    };
    if !ShapePass::new(&gpu, wgpu::TextureFormat::Rgba16Float).tem_subgrupo() {
        eprintln!("sem Features::SUBGROUP — um caminho só, nada a comparar");
        return;
    }
    let (est, circ, zz, furo) = (estrela(), circulo(), zigue_zague(), estrela_com_furo());
    let fmt = wgpu::TextureFormat::Rgba16Float;
    for (nome, forma, cs) in &casos(&est, &circ, &zz, &furo) {
        let n = u32::try_from(cs.len()).expect("cabem");
        let corre = |subgrupo: bool| {
            pelo_passe_ajustado(
                &gpu,
                forma,
                &[(cs.as_slice(), 4)],
                fmt,
                &mut |p| {
                    p.area_minima_conforme(0.0);
                    p.com_subgrupo(subgrupo);
                },
                &mut |_, _| {},
            )
            .pop()
            .expect("um quadro")
        };
        let (com_sg, com, _) = corre(true);
        let (sem_sg, com2, _) = corre(false);
        eprintln!("  {nome}: {com}/{n} e {com2}/{n} nas celulas");
        assert!(
            com == n && com2 == n,
            "{nome}: CONTROLO — todas nas células ({com}, {com2} de {n})"
        );
        assert!(
            com_sg.iter().any(|&b| b != 0),
            "{nome}: a fixtura nao desenha"
        );
        assert!(
            com_sg == sem_sg,
            "{nome}: o prefixo por subgrupo desenha outra coisa"
        );
    }
}
