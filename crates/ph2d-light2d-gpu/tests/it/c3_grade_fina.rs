//! ⭐ **C3 na grade FINA** (W4b) — a luz direta das lâmpadas numa grade `fina` vezes mais fina que a
//! das cascatas: o que o RECEPTOR lê (a bilinear da fluência) contra o gabarito no centro de cada
//! texel fino; o refino pelas bordas contra a visibilidade medida em todo texel; a região; o G10.

use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_gpu::{Config, Grade, REFINO_PADRAO, Refino, Solver};

use crate::cenas::{DEGRAUS, ceu_e_vidro, faisca, penumbra};
use crate::comum::{gabarito, gabarito_por, gpu, grade, ler_bytes, ler_fluencia, rmse_rel};

const FINA: u32 = 4;

fn rodar(gpu: &ph2d_gpu::GpuContext, cfg: Config, cena: &CenaDeLuz) -> (Vec<[f32; 3]>, Grade) {
    let mut s = Solver::novo(gpu, cfg).expect("solver");
    s.quadro(gpu, cena);
    (ler_fluencia(gpu, &s), s.grade_da_fluencia())
}

/// A conta do `receber.wgsl`: a fluência `f` (na grade `g`) bilinear em `p`, bordas presas.
fn bilinear(f: &[[f32; 3]], g: &Grade, p: [f64; 2]) -> [f32; 3] {
    let tx = (p[0] - g.origem_m[0]) / g.texel_m - 0.5;
    let ty = (p[1] - g.origem_m[1]) / g.texel_m - 0.5;
    let (i0, j0) = (tx.floor(), ty.floor());
    let (fx, fy) = ((tx - i0) as f32, (ty - j0) as f32);
    let em = |i: f64, j: f64| {
        let i = (i.max(0.0) as u32).min(g.largura - 1);
        let j = (j.max(0.0) as u32).min(g.altura - 1);
        f[(j * g.largura + i) as usize]
    };
    let mut out = [0.0; 3];
    for (c, o) in out.iter_mut().enumerate() {
        let a = em(i0, j0)[c] * (1.0 - fx) + em(i0 + 1.0, j0)[c] * fx;
        let b = em(i0, j0 + 1.0)[c] * (1.0 - fx) + em(i0 + 1.0, j0 + 1.0)[c] * fx;
        *o = a * (1.0 - fy) + b * fy;
    }
    out
}

/// ⭐⭐⭐ **O que o receptor lê, no texel FINO, contra o gabarito** — no degrau Baixo, as cenas com
/// lâmpadas diretas: a grade fina tem de bater o gabarito muito melhor que a grossa interpolada.
#[test]
fn a_grade_fina_e_o_gabarito_no_texel_fino() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[2];
    for (nome, cena, ganho) in [
        ("penumbra", penumbra(), TETO_GANHO[0]),
        ("faísca", faisca(), TETO_GANHO[1]),
        ("céu e vidro", ceu_e_vidro(), TETO_GANHO[2]),
    ] {
        let grossa = Config::padrao(grade(lado, texel));
        let fina = Config {
            fina: FINA,
            ..grossa
        };
        let (ff, gf) = rodar(&gpu, fina, &cena);
        let (fg, gg) = rodar(&gpu, grossa, &cena);
        let r = gabarito(&cena, &gf);
        let lida_da_grossa: Vec<[f32; 3]> = (0..gf.altura)
            .flat_map(|j| (0..gf.largura).map(move |i| (i, j)))
            .map(|(i, j)| bilinear(&fg, &gg, gf.centro(i, j)))
            .collect();
        // Fora dos opacos: dentro a lei é a do preenchimento (gate próprio, abaixo).
        let dentro = crate::comum::dentro_de_opaco(&cena, &gf);
        let (e_fina, e_grossa) = (
            rmse_rel(&ff, &r, |k| !dentro(k)),
            rmse_rel(&lida_da_grossa, &r, |k| !dentro(k)),
        );
        eprintln!(
            "C3 fina · {nome}: RMSE relativo no texel fino, fora dos opacos, {e_fina:.4} (a grossa interpolada {e_grossa:.4}; razão {:.3}, teto {ganho})",
            e_fina / e_grossa,
        );
        assert!(
            e_fina <= ganho * e_grossa,
            "C3 fina · {nome}: {e_fina:.4} não melhora a grossa {e_grossa:.4} (teto ×{ganho})"
        );
    }
}

/// O teto da razão fina ÷ grossa por cena = o MEDIDO + 10 % (W4b, RTX 5060 Ti, `fina` ×4 no Baixo):
/// penumbra 0,0533 ÷ 0,5226 = 0,102 · faísca 0,0804 ÷ 1,8661 = 0,043 · céu e vidro 0,0275 ÷ 0,0568 =
/// 0,484 (as cascatas pesam mais nela: o céu e os vidros não são luz direta). ⚠️ W5a: a população é
/// a de FORA dos opacos (dentro, o gabarito deixou de ser 0 — o F3; gate próprio), e o teto é o
/// MEDIDO + 10 % de hoje: penumbra 0,0446 ÷ 0,5207 = 0,086 · faísca 0,0739 ÷ 1,8647 = 0,040 · céu e
/// vidro 0,0194 ÷ 0,0561 = 0,347 (sem a c0 só das sondas de FORA dos opacos 0,433 — o teto da W4b,
/// 0,533, deixava-a sobreviver à mutação).
const TETO_GANHO: [f64; 3] = [0.095, 0.044, 0.382];

/// ⭐⭐ **O refino pelas BORDAS dá a visibilidade medida em todo texel** (`Refino::Tudo`), a menos
/// do que a interpolação das sondas deixa onde a visibilidade é suave.
#[test]
fn o_refino_pelas_bordas_e_o_exato_em_todo_texel() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[2];
    for (nome, cena) in [("penumbra", penumbra()), ("faísca", faisca())] {
        let bordas = Config {
            fina: FINA,
            ..Config::padrao(grade(lado, texel))
        };
        let tudo = Config {
            refino: Refino::Tudo,
            ..bordas
        };
        let (fb, _) = rodar(&gpu, bordas, &cena);
        let (ft, _) = rodar(&gpu, tudo, &cena);
        let r: Vec<[f64; 3]> = ft.iter().map(|v| v.map(f64::from)).collect();
        let e = rmse_rel(&fb, &r, |_| true);
        eprintln!("C3 bordas × tudo · {nome}: RMSE relativo {e:.5} (teto {TETO_BORDAS})");
        assert!(
            e <= TETO_BORDAS,
            "C3 bordas × tudo · {nome}: {e:.5} > {TETO_BORDAS}"
        );
    }
}

/// O MEDIDO + 10 % (W4b, o `REFINO_PADRAO`): penumbra 0,00127 · faísca 0,00026 — o que a interpolação
/// das sondas deixa no miolo da penumbra (com as sondas no passo grosso e sem o joelho: 0,0060 · 0,0030).
const TETO_BORDAS: f64 = 0.0014;

/// ⭐⭐ **Só a REGIÃO se calcula, e por dentro dela é o MESMO bit** que a grade inteira (cada
/// texel fino é independente); fora dela o solver novo deixa a textura como nasceu (zeros). ⭐ W5n: o
/// miolo do opaco que atravessa a borda também (o anel de fora da região, `c3_anel.wgsl`); controlo
/// na mesma corrida: sem ele (`anel_fora_da_regiao: false`, a W5h) o miolo por dentro muda.
#[test]
fn so_a_regiao_se_calcula_e_por_dentro_e_o_mesmo_bit() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[2];
    let cfg = Config {
        fina: FINA,
        ..Config::padrao(grade(lado, texel))
    };
    // ⭐ W5a: e uma parede que ATRAVESSA a borda esquerda da região (x = 0,37 m), com a face mais
    // próxima do interior visível FORA dela — o preenchimento corre na grade inteira, senão a parte
    // de dentro acharia a outra face (e mudaria de cor no pan).
    let mut cena = penumbra();
    cena.oclusores.push(crate::comum::oclusor(
        vec![[0.30, 1.0], [0.60, 1.0], [0.60, 2.0], [0.30, 2.0]],
        1.0,
    ));
    // E uma lâmpada FORA da região que acende essa face: o texel de dentro que se calcula nela mede a
    // visibilidade (as sondas em volta estão fora da região).
    cena.emissores.push(crate::comum::disco(
        [0.1, 1.5],
        0.05,
        [1.0; 3],
        f32::INFINITY,
    ));
    let mut inteira = Solver::novo(&gpu, cfg).expect("solver");
    inteira.quadro(&gpu, &cena);
    let a = ler_bytes(&gpu, inteira.fluencia());
    let mut parte = Solver::novo(&gpu, cfg).expect("solver");
    let g = parte.grade_da_fluencia();
    let (ini, fim) = ([37, 101], [g.largura - 53, g.altura - 9]);
    parte.regiao_fina(ini, fim);
    parte.quadro(&gpu, &cena);
    let b = ler_bytes(&gpu, parte.fluencia());
    let px = (b.len() / (g.largura * g.altura) as usize) as u32;
    let mut sem_anel = Solver::novo(
        &gpu,
        Config {
            anel_fora_da_regiao: false,
            ..cfg
        },
    )
    .expect("solver");
    sem_anel.regiao_fina(ini, fim);
    sem_anel.quadro(&gpu, &cena);
    let c = ler_bytes(&gpu, sem_anel.fluencia());
    let mudam = (0..g.altura)
        .flat_map(|j| (0..g.largura).map(move |i| (i, j)))
        .filter(|&(i, j)| (ini[0]..fim[0]).contains(&i) && (ini[1]..fim[1]).contains(&j))
        .filter(|&(i, j)| {
            let o = ((j * g.largura + i) * px) as usize;
            a[o..o + px as usize] != c[o..o + px as usize]
        })
        .count();
    eprintln!("região: sem o anel, {mudam} texels de dentro mudam");
    assert!(
        mudam > 0,
        "controlo: sem o anel o miolo da região não muda — a régua não vê o enquadramento"
    );
    let (mut dentro, mut fora) = (0, 0);
    for j in 0..g.altura {
        for i in 0..g.largura {
            let o = ((j * g.largura + i) * px) as usize;
            let (ta, tb) = (&a[o..o + px as usize], &b[o..o + px as usize]);
            if (ini[0]..fim[0]).contains(&i) && (ini[1]..fim[1]).contains(&j) {
                assert_eq!(ta, tb, "o texel ({i}, {j}) DENTRO da região mudou");
                dentro += 1;
            } else {
                assert!(
                    tb.iter().all(|&x| x == 0),
                    "o texel ({i}, {j}) FORA da região foi calculado"
                );
                fora += 1;
            }
        }
    }
    assert!(
        a.iter().any(|&x| x != 0) && dentro > 0 && fora > 0,
        "controlo: {dentro} dentro, {fora} fora"
    );
}

/// **G10 na grade fina**: dois quadros e dois solvers dão o mesmo bit.
#[test]
fn g10_a_grade_fina_e_bit_identica() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[2];
    let cfg = Config {
        fina: FINA,
        ..Config::padrao(grade(lado, texel))
    };
    let cena = ceu_e_vidro();
    let mut s = Solver::novo(&gpu, cfg).expect("solver");
    s.quadro(&gpu, &cena);
    let a = ler_bytes(&gpu, s.fluencia());
    s.quadro(&gpu, &cena);
    let b = ler_bytes(&gpu, s.fluencia());
    let mut outro = Solver::novo(&gpu, cfg).expect("solver");
    outro.quadro(&gpu, &cena);
    let c = ler_bytes(&gpu, outro.fluencia());
    assert!(a.iter().any(|&x| x != 0), "a fluência não pode ser zero");
    assert!(a == b, "dois quadros da mesma cena diferem");
    assert!(a == c, "dois solvers da mesma configuração diferem");
}

/// O fio de sombra de um oclusor-PONTO de 0,8 texel (centrado num texel: a captura vê-o) diante de
/// uma lâmpada pontual na diagonal: a sombra é mais estreita que o espaço entre as sondas e corre de
/// viés, então passa ENTRE elas. A fração das amostras da linha do meio dela, de 0,2 a 1,2 m atrás,
/// que a placa põe na sombra (com ÷ sem < ½).
fn fio_na_sombra(gpu: &ph2d_gpu::GpuContext, refino: Refino) -> f64 {
    let (lado, texel) = (128, 0.04);
    let cfg = Config {
        fina: FINA,
        refino,
        ..Config::padrao(grade(lado, texel))
    };
    let c = [60.5 * texel as f32, 70.5 * texel as f32];
    let m = 0.4 * texel as f32;
    let dir = [0.8_f32, 0.6];
    let lampada = ph2d_light2d_ecs::Emissor::Ponto {
        centro: [c[0] - 1.5 * dir[0], c[1] - 1.5 * dir[1]],
        intensidade: [5.0; 3],
        alcance_m: f32::INFINITY,
    };
    let com = CenaDeLuz {
        emissores: vec![lampada.clone()],
        oclusores: vec![crate::comum::oclusor(
            vec![
                [c[0] - m, c[1] - m],
                [c[0] + m, c[1] - m],
                [c[0] + m, c[1] + m],
                [c[0] - m, c[1] + m],
            ],
            1.0,
        )],
        ceu: [0.0; 3],
        ..Default::default()
    };
    let sem = CenaDeLuz {
        oclusores: vec![],
        ..com.clone()
    };
    let (fc, g) = rodar(gpu, cfg, &com);
    let (fs, _) = rodar(gpu, cfg, &sem);
    let n = 200;
    let escuras = (0..n)
        .filter(|&k| {
            let t = 0.2 + f64::from(k) / f64::from(n);
            let p = [
                f64::from(c[0]) + t * f64::from(dir[0]),
                f64::from(c[1]) + t * f64::from(dir[1]),
            ];
            bilinear(&fc, &g, p)[0] < 0.5 * bilinear(&fs, &g, p)[0]
        })
        .count();
    escuras as f64 / f64::from(n)
}

/// ⭐⭐ **O fio de sombra mais estreito que o espaço entre as sondas** sai como no exato em todo texel
/// (o refino pelas bordas não o perde). Medido (W4b): 0,515 das amostras na sombra pelas bordas e pelo
/// exato — e 0,515 também com a sonda «certificada» por uma célula de folga (por isso ela saiu).
#[test]
fn o_fio_de_sombra_entre_as_sondas_e_o_do_exato() {
    let Some(gpu) = gpu() else { return };
    let bordas = fio_na_sombra(&gpu, REFINO_PADRAO);
    let tudo = fio_na_sombra(&gpu, Refino::Tudo);
    eprintln!("fio de sombra: na sombra {bordas:.3} (exato em todo texel {tudo:.3})");
    assert!(
        tudo > 0.3,
        "controlo: o fio tem de existir na captura ({tudo:.3})"
    );
    assert!(
        (bordas - tudo).abs() <= 0.02,
        "o refino perde o fio: {bordas:.3} × {tudo:.3}"
    );
}

/// A lei do interior até a W5g — o exterior mais próximo em todo o miolo (`suavidade = 0`): a do
/// `Config::miolo = None` e a que o anel do miolo (o pé na borda) dá à difusão.
const EXTERIOR: ph2d_light2d_ref::OpcoesDoMiolo = ph2d_light2d_ref::OpcoesDoMiolo {
    suavidade: 0.0,
    penetracao_m: f64::INFINITY,
};

/// ⭐⭐⭐ **F3 — o INTERIOR de um opaco é o exterior mais próximo** (W5a; a regra do gabarito,
/// `ph2d-light2d-ref`): nos texels finos dentro dos opacos a até [`FAIXA_INTERIOR`] texels da borda,
/// a placa contra o gabarito. Sem o preenchimento o interior é 0. ⚠️ W5n: a lei é a da `suavidade = 0`
/// (o produto com `miolo: None` contra o gabarito [`EXTERIOR`]) — com a difusão (W5h) o interior FUNDO
/// é outro (portão próprio: `o_miolo_da_placa_e_o_do_gabarito`) e a média no eixo não decide nada nele. ⚠️ Mais fundo, no EIXO MEDIAL do
/// opaco, «a face mais próxima» troca de lado num salto (acesa ↔ sombra) e qualquer discretização
/// erra por inteiro num fio de texels (medido: os 6 piores da penumbra todos na diagonal, placa ~0,27
/// × gabarito 0) — fica fora da régua. ⚠️ Só luz DIRETA: com céu (a «céu e vidro») o EXTERIOR junto à
/// parede já sai baixo pelas cascatas (0,17 → 0,01 contra 0,25 em ~2 texels grossos, sem o
/// preenchimento — ABERTO da W5a, território das cascatas/HRC) e o interior copia-o fielmente.
#[test]
fn o_interior_de_um_opaco_e_o_exterior_mais_proximo() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[2];
    // Uma parede GROSSA (40 texels finos) acesa de um lado: o interior fundo é do JFA.
    let parede = CenaDeLuz {
        emissores: vec![crate::comum::disco(
            [1.2, 2.56],
            0.3,
            [1.0; 3],
            f32::INFINITY,
        )],
        oclusores: vec![crate::comum::oclusor(
            vec![[1.8, 1.5], [2.2, 1.5], [2.2, 3.5], [1.8, 3.5]],
            1.0,
        )],
        ceu: [0.0; 3],
        ..Default::default()
    };
    for (k, (nome, cena)) in [
        ("penumbra", penumbra()),
        ("faísca", faisca()),
        ("parede grossa", parede),
    ]
    .into_iter()
    .enumerate()
    {
        let teto = TETO_INTERIOR[k];
        // A captura do PRODUTO: duas vezes mais fina que a grade fina (a grossa arredonda o canto em
        // ~1 texel fino e troca a face mais próxima junto a ele).
        let com = Config {
            fina: FINA,
            captura: 2 * FINA,
            miolo: None,
            ..Config::padrao(grade(lado, texel))
        };
        let sem = Config {
            preenchimento: None,
            ..com
        };
        let (fc, g) = rodar(&gpu, com, &cena);
        let (fs, _) = rodar(&gpu, sem, &cena);
        let (fm, _) = rodar(
            &gpu,
            Config {
                media_no_eixo: false,
                ..com
            },
            &cena,
        );
        // Dentro dos opacos, o gabarito é a média da lei sobre o texel (o texel que o eixo medial corta
        // guarda a média das faces, W5c).
        let lei = ph2d_light2d_ref::Gabarito::com_miolo(&cena, EXTERIOR);
        let mut r = gabarito_por(&g, |p| lei.fluencia(p));
        let fundo = crate::comum::dentro_de_opaco(&cena, &g);
        let max = r.iter().map(|v| v[1]).fold(0.0_f64, f64::max);
        for k in (0..r.len()).filter(|&k| fundo(k)) {
            r[k] = media_no_texel(&cena, &lei, &g, k, r[k]);
        }
        let dentro = crate::comum::dentro_de_opaco_a_menos_de(&cena, &g, FAIXA_INTERIOR);
        let (ec, es, em) = (
            rmse_rel(&fc, &r, &dentro),
            rmse_rel(&fs, &r, &dentro),
            rmse_rel(&fm, &r, &dentro),
        );
        // O interior FUNDO (o do JFA): a MÉDIA do erro, relativa ao máximo do gabarito.
        let media_fundo = |f: &[[f32; 3]]| {
            let erros: Vec<f64> = (0..r.len())
                .filter(|&k| fundo(k) && !dentro(k))
                .map(|k| (f64::from(f[k][1]) - r[k][1]).abs() / max)
                .collect();
            (
                erros.iter().sum::<f64>() / erros.len().max(1) as f64,
                erros.len(),
            )
        };
        let ((media, n), (media_sem, _)) = (media_fundo(&fc), media_fundo(&fm));
        // ⭐ W5o — o controlo da face PELO PONTO: com a marcha a partir do ponto afastado (a W5g), a
        // penumbra passa do teto.
        if k == 0 {
            let (fa, _) = rodar(
                &gpu,
                Config {
                    face_pelo_ponto: false,
                    ..com
                },
                &cena,
            );
            let antiga = media_fundo(&fa).0;
            assert!(
                antiga > TETO_FUNDO[k],
                "controlo: a face afastada dá {antiga:.6} na penumbra — o teto {} não a separa",
                TETO_FUNDO[k]
            );
        }
        eprintln!(
            "F3 interior fundo · {nome}: erro médio {media:.6} do máximo ({n} texels; sem a média no eixo {media_sem:.6}; teto {})",
            TETO_FUNDO[k]
        );
        assert!(
            media <= TETO_FUNDO[k] && media < media_sem,
            "F3 interior fundo · {nome}: {media:.5} (sem a média no eixo {media_sem:.5})"
        );
        let mut piores: Vec<(f64, usize)> = (0..r.len())
            .filter(|&k| dentro(k))
            .map(|k| ((f64::from(fc[k][1]) - r[k][1]).abs(), k))
            .collect();
        piores.sort_by(|a, b| b.0.total_cmp(&a.0));
        let w = g.largura as usize;
        for &(d, k) in piores.iter().take(6) {
            eprintln!(
                "  pior ({}, {}): placa {:.4} · gabarito {:.4} · |Δ| {d:.4} · de {} texels de dentro",
                k % w,
                k / w,
                fc[k][1],
                r[k][1],
                piores.len()
            );
        }
        eprintln!(
            "F3 interior · {nome}: RMSE relativo {ec:.4} (sem o preenchimento {es:.4}; sem a média no eixo {em:.4}; teto {teto})"
        );
        assert!(
            es > 0.9,
            "controlo: sem o preenchimento o interior é 0 ({es:.4})"
        );
        assert!(
            ec <= teto && ec < em,
            "F3 interior · {nome}: {ec:.4} > {teto} (sem a média no eixo {em:.4})"
        );
    }
}

/// ⭐ W5c: a lei MÉDIA sobre a parte do texel `k` DENTRO dos opacos (de valor `centro` no centro; a
/// parte de fora é a borda do receptor, que o interior não guarda): onde a média dos cantos de dentro
/// se afasta do centro mais de 1 % do maior deles no verde (um corte — o eixo medial — passa dentro do
/// texel; um gradiente liso não muda a média), a média das subamostras de dentro de 8×8; senão o centro.
fn media_no_texel(
    cena: &CenaDeLuz,
    lei: &ph2d_light2d_ref::Gabarito,
    g: &Grade,
    k: usize,
    centro: [f64; 3],
) -> [f64; 3] {
    let c = g.centro(k as u32 % g.largura, k as u32 / g.largura);
    let t = g.texel_m;
    let ponto = |dx: f64, dy: f64| [c[0] + dx * t, c[1] + dy * t];
    let cantos: Vec<f64> = [(-0.5, -0.5), (0.5, -0.5), (-0.5, 0.5), (0.5, 0.5)]
        .into_iter()
        .map(|(dx, dy)| ponto(dx, dy))
        .filter(|&p| dentro_de_algum(cena, p))
        .map(|p| lei.fluencia(p)[1])
        .collect();
    let local = cantos.iter().fold(centro[1].abs(), |m, v| m.max(v.abs()));
    if cantos.is_empty()
        || (cantos.iter().sum::<f64>() / cantos.len() as f64 - centro[1]).abs() <= 0.01 * local
    {
        return centro;
    }
    let o = |a: u32| (f64::from(a) + 0.5) / 8.0 - 0.5;
    let (mut m, mut n) = ([0.0; 3], 0.0);
    for i in 0..64 {
        let p = ponto(o(i % 8), o(i / 8));
        if dentro_de_algum(cena, p) {
            let v = lei.fluencia(p);
            for (mc, vc) in m.iter_mut().zip(v) {
                *mc += vc;
            }
            n += 1.0;
        }
    }
    m.map(|v| v / n)
}

/// `p` cai dentro de um oclusor opaco ou de um emissor poligonal (os corpos de [`dentro_de_opaco`]).
///
/// [`dentro_de_opaco`]: crate::comum::dentro_de_opaco
fn dentro_de_algum(cena: &CenaDeLuz, p: [f64; 2]) -> bool {
    let opacos = cena
        .oclusores
        .iter()
        .filter(|o| o.opacidade >= 1.0)
        .map(|o| &o.vertices)
        .chain(cena.emissores.iter().filter_map(|e| match e {
            ph2d_light2d_ecs::Emissor::Poligono { vertices, .. } => Some(vertices),
            _ => None,
        }));
    let p = [p[0] as f32, p[1] as f32];
    opacos.into_iter().any(|v| {
        let mut dentro = false;
        let mut j = v.len() - 1;
        for i in 0..v.len() {
            let (a, b) = (v[i], v[j]);
            if (a[1] > p[1]) != (b[1] > p[1])
                && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
            {
                dentro = !dentro;
            }
            j = i;
        }
        dentro
    })
}

/// Texels finos de dentro, a partir da borda, que a régua mede.
const FAIXA_INTERIOR: f64 = 2.0;
/// O MEDIDO + 10 % (W5c, a régua da MÉDIA da lei sobre a parte de dentro do texel —
/// [`media_no_texel`]): o erro médio do interior FUNDO, do máximo do gabarito — penumbra 0,001281 ·
/// faísca 0,000043 · parede grossa 0,000337; sem a média no eixo (o controlo, o desenho da W5a)
/// 0,002545 · 0,000311 · 0,000536. A régua de PONTO da W5a (o centro do texel) media 0,0016 · 0,0001 ·
/// 0,0005; com a semente MAIS PERTO em vez do ponto de fronteira mais perto, 0,0043 · 0,0007 ·
/// 0,0011 (e o desempate decidia a face: 0,0035). ⚠️ W5n (com `miolo: None`, RTX 5060 Ti): penumbra
/// 0,001436 · faísca 0,000012 · parede 0,000050 (sem a média 0,002717 · 0,000306 · 0,000251). A
/// penumbra sobe por causa do `vis_na_face` da W5g (a marcha parte 1,5 texel PARA FORA da face: na
/// penumbra a visibilidade varia com a posição) — ablação na mesma corrida: só o pé 0,000337, só a
/// face 0,001498, sem os dois 0,000292, a W5c 0,001281 (a faixa: 0,058 · 0,207 · 0,077 · 0,321). O
/// teto da penumbra é o medido + 10 %; a troca da face (os pontos tangentes × a penumbra) está no
/// handoff W5n. ⭐ W5o — a face PELO PONTO (`Config::face_pelo_ponto`, `sonda_a_face_pelo_ponto`):
/// penumbra 0,001436 → 0,000182 (faixa 0,2009 → 0,0563), faísca e parede iguais; o teto da penumbra é
/// o medido + 10 %, e a face afastada (o controlo) reprova.
const TETO_FUNDO: [f64; 3] = [0.00020, 0.000048, 0.00037];
/// O MEDIDO + 10 % (W5c, a régua da média, a captura 2 × fina, a faixa de 2 texels): penumbra 0,3207
/// · faísca 0,1136 · parede grossa 0,0063; sem a média no eixo 0,3664 · 0,2370 · 0,0522; sem o
/// preenchimento 1,65 · 1,96 · 1,58. Na faísca o resto é o canto do quadrado girado, onde o eixo
/// medial chega à borda junto à sombra que nasce no canto. A régua de PONTO da W5a media 0,3135 ·
/// 0,0032 · 0,0759 (o centro do texel é a lei do centro — a média sobre o texel não se via). Com a
/// captura GROSSA a penumbra dava 0,80 (o canto arredondado troca a face mais próxima); escolhendo o
/// texel exterior mais perto em vez do ponto de FRONTEIRA mais perto, 0,83; só com as sementes do
/// JFA de células de 4, 1,15.
/// ⭐ W5o — a penumbra com a face pelo ponto: 0,0563 (a W5n 0,2009), o teto o medido + 10 %.
const TETO_INTERIOR: [f64; 3] = [0.062, 0.125, 0.0070];

/// ⭐ W5o — a rodada da FACE PELO PONTO (`Config::face_pelo_ponto`): o interior fundo e a faixa de dentro
/// do [`o_interior_de_um_opaco_e_o_exterior_mais_proximo`] por variante, no mesmo processo
/// (`miolo: None`, a lei do exterior mais próximo — a régua que mede a luz da borda).
#[test]
#[ignore = "a rodada da face pelo ponto: corre-se à mão"]
fn sonda_a_face_pelo_ponto() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[2];
    let parede = CenaDeLuz {
        emissores: vec![crate::comum::disco(
            [1.2, 2.56],
            0.3,
            [1.0; 3],
            f32::INFINITY,
        )],
        oclusores: vec![crate::comum::oclusor(
            vec![[1.8, 1.5], [2.2, 1.5], [2.2, 3.5], [1.8, 3.5]],
            1.0,
        )],
        ceu: [0.0; 3],
        ..Default::default()
    };
    // (nome, pé, face, pelo ponto)
    const V: [(&str, bool, bool, bool); 4] = [
        ("pé+face afastada (W5g)", true, true, false),
        ("pé+face pelo ponto (W5o)", true, true, true),
        ("só o pé", true, false, false),
        ("nenhum", false, false, false),
    ];
    for (nome, cena) in [
        ("penumbra", penumbra()),
        ("faísca", faisca()),
        ("parede grossa", parede),
    ] {
        let base = Config {
            fina: FINA,
            captura: 2 * FINA,
            miolo: None,
            ..Config::padrao(grade(lado, texel))
        };
        let lei = ph2d_light2d_ref::Gabarito::com_miolo(&cena, EXTERIOR);
        let g = Solver::novo(&gpu, base)
            .expect("solver")
            .grade_da_fluencia();
        let mut r = gabarito_por(&g, |p| lei.fluencia(p));
        let fundo = crate::comum::dentro_de_opaco(&cena, &g);
        let max = r.iter().map(|v| v[1]).fold(0.0_f64, f64::max);
        for k in (0..r.len()).filter(|&k| fundo(k)) {
            r[k] = media_no_texel(&cena, &lei, &g, k, r[k]);
        }
        let dentro = crate::comum::dentro_de_opaco_a_menos_de(&cena, &g, FAIXA_INTERIOR);
        let mut linha = format!("{nome}:");
        for (v, pe, face, ponto) in V {
            let cfg = Config {
                pe_na_borda: pe,
                vis_na_face: face,
                face_pelo_ponto: ponto,
                ..base
            };
            let (f, _) = rodar(&gpu, cfg, &cena);
            let erros: Vec<f64> = (0..r.len())
                .filter(|&k| fundo(k) && !dentro(k))
                .map(|k| (f64::from(f[k][1]) - r[k][1]).abs() / max)
                .collect();
            let media = erros.iter().sum::<f64>() / erros.len().max(1) as f64;
            let faixa = rmse_rel(&f, &r, &dentro);
            linha += &format!(" {v}: fundo {media:.6} · faixa {faixa:.4} ·");
        }
        eprintln!("{linha}");
    }
}

/// ⭐ W6d §21 — o vértice do quadrado girado da [`penumbra`] (o pior texel do F3 com a união, (155, 257)):
/// a luz contra o gabarito do F3 numa janela 7×7, nas leis antes da W6d × o produto (as rodadas 11–12
/// mediram também a união: handoff W6d §21–§23). À mão (`--ignored --nocapture`, nome `medida_w6d_vertice_girado`).
#[test]
#[ignore = "a sonda do vértice da W6d: corre-se à mão"]
fn medida_w6d_vertice_girado() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[2];
    let cena = penumbra();
    let base = Config {
        fina: FINA,
        captura: 2 * FINA,
        miolo: None,
        ..Config::padrao(grade(lado, texel))
    };
    let lei = ph2d_light2d_ref::Gabarito::com_miolo(&cena, EXTERIOR);
    for (nome, cfg) in [
        (
            "antes da W6d",
            Config {
                borda_continua: false,
                borda_fora: false,
                ..base
            },
        ),
        ("produto", base),
    ] {
        let (f, g) = rodar(&gpu, cfg, &cena);
        let mut r = gabarito_por(&g, |p| lei.fluencia(p));
        let fundo = crate::comum::dentro_de_opaco(&cena, &g);
        for k in (0..r.len()).filter(|&k| fundo(k)) {
            r[k] = media_no_texel(&cena, &lei, &g, k, r[k]);
        }
        let w = g.largura as usize;
        eprintln!("== {nome} · placa/gabarito (* = dentro do opaco), linhas de cima para baixo");
        for j in (254..=260).rev() {
            let mut linha = format!("j {j}:");
            for i in 152..=158 {
                let k = j * w + i;
                linha += &format!(
                    " {:.3}/{:.3}{}",
                    f[k][1],
                    r[k][1],
                    if fundo(k) { "*" } else { " " }
                );
            }
            eprintln!("{linha}");
        }
    }
}

/// ⭐ W6e — o F3 da penumbra com e sem a cunha (`Config::borda_vertice`) no mesmo processo: o RMSE, os
/// texels finos que a cunha mais piora e o campo da captura (o 4.º canal do `c3_borda`) contra a
/// distância exata em volta de cada vértice do quadrado. À mão (handoff W6e, nome `medida_luz2d_w6e_f3`).
#[test]
#[ignore = "a sonda do F3 da W6e: corre-se à mão"]
fn medida_luz2d_w6e_f3() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[2];
    let cena = penumbra();
    let base = Config {
        fina: FINA,
        captura: 2 * FINA,
        miolo: None,
        ..Config::padrao(grade(lado, texel))
    };
    let lei = ph2d_light2d_ref::Gabarito::com_miolo(&cena, EXTERIOR);
    let poli: Vec<[f64; 2]> = cena.oclusores[0].vertices.iter().map(|v| v.map(f64::from)).collect();
    // A distância com sinal (positiva dentro) ao quadrado (convexo, anti-horário ou não: pelo módulo).
    let exata = |p: [f64; 2]| {
        let n = poli.len();
        let mut d = f64::INFINITY;
        let mut dentro = true;
        let area: f64 = (0..n)
            .map(|i| {
                let (a, b) = (poli[i], poli[(i + 1) % n]);
                a[0] * b[1] - b[0] * a[1]
            })
            .sum();
        for i in 0..n {
            let (a, b) = (poli[i], poli[(i + 1) % n]);
            let ab = [b[0] - a[0], b[1] - a[1]];
            let c = (ab[0] * (p[1] - a[1]) - ab[1] * (p[0] - a[0])) * area.signum();
            dentro &= c >= 0.0;
            let t = (((p[0] - a[0]) * ab[0] + (p[1] - a[1]) * ab[1]) / (ab[0] * ab[0] + ab[1] * ab[1])).clamp(0.0, 1.0);
            d = d.min((p[0] - a[0] - t * ab[0]).hypot(p[1] - a[1] - t * ab[1]));
        }
        if dentro { d } else { -d }
    };
    let mut lidas = Vec::new();
    for (nome, v) in [("sem a cunha", None), ("cunha", Some(ph2d_light2d_gpu::VERTICE_PADRAO))] {
        let cfg = Config { borda_vertice: v, ..base };
        let mut s = Solver::novo(&gpu, cfg).expect("solver");
        s.quadro(&gpu, &cena);
        let fl = ler_fluencia(&gpu, &s);
        let g = s.grade_da_fluencia();
        let gc = cfg.grade_da_captura();
        let campo = ler_bytes(&gpu, s.borda_da_captura().expect("borda"));
        let mut r = gabarito_por(&g, |p| lei.fluencia(p));
        let fundo = crate::comum::dentro_de_opaco(&cena, &g);
        for k in (0..r.len()).filter(|&k| fundo(k)) {
            r[k] = media_no_texel(&cena, &lei, &g, k, r[k]);
        }
        let dentro = crate::comum::dentro_de_opaco_a_menos_de(&cena, &g, FAIXA_INTERIOR);
        eprintln!("F3 · {nome}: RMSE relativo {:.4}", rmse_rel(&fl, &r, &dentro));
        // O campo em volta de cada vértice: 6×6 texels da captura.
        for q in &poli {
            let (ci, cj) = (
                ((q[0] - gc.origem_m[0]) / gc.texel_m) as i64,
                ((q[1] - gc.origem_m[1]) / gc.texel_m) as i64,
            );
            eprintln!(
                "  vértice ({:.2}; {:.2}) fino · (cobertura campo/exata), linhas de cima para baixo",
                (q[0] - g.origem_m[0]) / g.texel_m,
                (q[1] - g.origem_m[1]) / g.texel_m
            );
            for j in (cj - 3..=cj + 2).rev() {
                let mut l = String::new();
                for i in ci - 3..=ci + 2 {
                    let k = (j as u32 * gc.largura + i as u32) as usize * 8;
                    let h = |o: usize| half::f16::from_le_bytes([campo[k + o], campo[k + o + 1]]).to_f32();
                    let e = exata(gc.centro(i as u32, j as u32)) / gc.texel_m;
                    l += &format!(" {:.2} {:+.2}/{:+.2}", h(0), h(6), e);
                }
                eprintln!("   {l}");
            }
        }
        lidas.push((fl, r, dentro, g));
    }
    let (f0, r, dentro, g) = &lidas[0];
    let f1 = &lidas[1].0;
    let w = g.largura as usize;
    let mut dif: Vec<(f64, usize)> = (0..r.len())
        .filter(|&k| dentro(k))
        .map(|k| ((f64::from(f1[k][1]) - r[k][1]).abs() - (f64::from(f0[k][1]) - r[k][1]).abs(), k))
        .collect();
    dif.sort_by(|a, b| b.0.total_cmp(&a.0));
    for &(d, k) in dif.iter().take(12) {
        eprintln!(
            "  piora ({}, {}): +{d:.4} · sem {:.4} · cunha {:.4} · gabarito {:.4}",
            k % w,
            k / w,
            f0[k][1],
            f1[k][1],
            r[k][1]
        );
    }
}
