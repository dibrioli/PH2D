//! As sondas da **RETÍCULA** — filhas das [`super`], e o corte é o SUJEITO.
//!
//! ⛔ Saíram do irmão por TECTO DE LOC (`899` contra `700`). O que fica lá é a
//! ATRIBUIÇÃO (*qual metade enruga a superfície*); o que vem para cá é a escada
//! que escolheu os dois números da lei nova — as **rondas** e o **lado da
//! célula** — e a tabela que a põe contra a lei que o dono reprovou.
//!
//! ⚠️ **A linha da retícula destas tabelas corre pela PORTA DO PRODUTO**
//! ([`crate::dyntopo::passe_nos_motores`], via o `traco_com` do gate); as
//! escadas correm um arnês próprio, porque elas variam exactamente os números
//! que a porta fixa. *Um instrumento que varre uma constante não pode passar
//! pela porta que a crava, e dizer isso é o que o impede de ser lido como o
//! produto.*

use super::*;

// ⭐⭐ **Os dois números da lei vêm do PRODUTO, e não de uma cópia aqui.** Estas
// escadas foram o que os escolheu; guardá-los outra vez neste ficheiro faria a
// sonda medir uma lei que o app não corre no dia em que um deles mudasse — a
// forma exacta que o gate desta cena pagou em 20/09.
use crate::dyntopo::{LADO_DA_CELULA, RONDAS_DA_GRELHA};

// ⚠️ A variável da sonda viaja por `thread_local` e não por argumento: o arnês
// tem seis parâmetros e o que se varre aqui é UMA hipótese de cada vez.
thread_local! {
    static SEMENTE_UNICA: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// ⭐⭐⭐ **SONDA — A GRELHA CONTRA A LEI DE HOJE, nas TRÊS colunas.**
///
/// A pergunta do dono depois do segundo report foi *«traga o estado da arte»*, e
/// o que a pesquisa devolveu tem de ser medido pelas mesmas réguas com que a lei
/// de hoje foi reprovada:
///
/// | coluna | o que ela lê | o lado aprovado |
/// |---|---|---|
/// | **grade** | que fracção das arestas segue a grade | `43,6 %` (a saída do ALVO) |
/// | **vinco** | o ângulo entre normais vizinhas — o que a LUZ mostra | `2,563°` por pentear |
/// | **razão** | comprimento ao longo ÷ atravessado | `1,04`–`1,13` (o ALVO, na chapa) |
///
/// ⛔ Ela **imprime**; quem decide é quem lê. A barra só se escreve depois de a
/// tabela existir.
#[test]
#[ignore = "sonda: imprime a tabela das duas leis, não afirma nada"]
fn diag_a_grelha_contra_a_lei_de_hoje() {
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    println!("rumo            lei        grade%   vinco_p50  vinco_p90  razão   V");
    for (nome, e) in RUMOS.iter().map(|(n, e)| (*n, *e)) {
        for (lei, campo, flip, desloca, grelha) in [
            ("por pentear ", false, false, false, 0.0f32),
            ("hoje        ", true, true, true, 0.0),
            ("retícula 1,0", false, false, false, 1.0),
            ("retícula 0,5", false, false, false, 0.5),
        ] {
            // ⭐ **A linha da retícula corre pela PORTA DO PRODUTO** (o
            // `traco_com` do gate, que percorre o `passe_nos_motores`); as
            // outras duas correm o arnês das METADES, que é um instrumento de
            // atribuição e mede leis que o produto já não compõe.
            let (m, c) = if grelha > 0.0 {
                super::super::traco_com(grelha, e, raio, alvo)
            } else {
                let (m, c, _) =
                    traco_por_metades_com(e, raio, alvo, campo, flip, desloca, 0.25, false);
                (m, c)
            };
            // ⚠️ **O balde ZERO é o ALINHADO** (`[0°,15°)`), que é o que o gate
            // da cena lê; o dois é a DIAGONAL. Ler o errado inverte a tabela
            // inteira sem nada acusar — e inverteu, na primeira corrida desta
            // sonda.
            let (g, nb) = grade_da_faixa(&m, &c, raio);
            let grade = 100.0 * g[0] as f64 / nb.max(1) as f64;
            let (v50, v90, _, _) = vinco_da_faixa(&m, &c, raio);
            let l = comprimento_por_direccao(&m, &c, raio);
            println!(
                "{nome:14}  {lei}  {grade:7.2}  {v50:9.3}  {v90:9.3}  {:6.3}  {}",
                l[0] / l[1].max(1e-12),
                m.vert_count()
            );
        }
    }
}

/// ⛔⛔⛔ **SONDA — A ESCADA DAS RONDAS E DO LADO DA CÉLULA.**
///
/// A pesquisa nomeou o risco da retícula **antes** de ela existir: *«uma
/// retícula por vértice pode DISCORDAR da vizinha por uma célula inteira, e uma
/// redução malfeita produz um SALTO DE CÉLULA — que na peça apareceria como um
/// degrau»*. A régua para isso é o **vinco**, e é a coluna do meio.
///
/// Duas variáveis, porque são duas hipóteses diferentes:
///
/// - **rondas** — se subir cura, o campo não estava consensual (convergência);
/// - **`k_passo`** — se a correcção do lado da célula cura, a aresta média de
///   uma malha de TRIÂNGULOS é grossa demais para uma grelha QUADRADA e os
///   vértices empilham-se (um quadrado de lado `e` cobre `e²` por vértice; um
///   triângulo equilátero de aresta `e` cobre `0,866 e²` ⇒ `√0,866 = 0,931`).
///
/// ```text
/// cargo test -p ph2d-app-sculpt3d --release --lib diag_a_escada_das_rondas -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda: imprime a escada, não afirma nada"]
fn diag_a_escada_das_rondas() {
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    let e = RUMOS[0].1;
    println!("rondas  k_passo   grade%   vinco_p50  vinco_p90  razão   V");
    // ⚠️ **A escada das rondas corre-se no lado de célula que SHIPA.** A
    // primeira corrida varreu-a a `k = 1,0`, onde tudo está mau, e leu «sem
    // tendência» — *uma escada medida no regime errado responde sobre outro
    // programa*.
    for rondas in [1usize, 2, 3, 4, 6, 8, 16] {
        let (m, c) = traco_com_grelha(e, raio, alvo, rondas, LADO_DA_CELULA);
        println!("{}", linha_da_grelha(&m, &c, raio, rondas, LADO_DA_CELULA));
    }
    for rondas in [1usize, 6] {
        let (m, c) = traco_com_grelha(e, raio, alvo, rondas, 1.0);
        println!("{}", linha_da_grelha(&m, &c, raio, rondas, 1.0));
    }
    for k in [LADO_DA_CELULA, 0.90, 0.931, 1.10, 1.25] {
        let (m, c) = traco_com_grelha(e, raio, alvo, RONDAS_DA_GRELHA, k);
        println!("{}", linha_da_grelha(&m, &c, raio, RONDAS_DA_GRELHA, k));
    }
    // ⭐ **O CONTROLO da RAZÃO: o mesmo traço com o carimbo a ZERO.** É ele que
    // separa *«a retícula estica»* de *«o relevo que o traço levanta estica»* —
    // a mesma partição que o §81 correu para a lei de hoje (chapa `1,029`, bola
    // sem carimbo `0,885`, bola com carimbo `0,653`).
    for (nome, forca) in [("carimbo 0,25", 0.25f32), ("carimbo ZERO", 0.0)] {
        let (m, c) =
            traco_com_grelha_com(e, raio, alvo, RONDAS_DA_GRELHA, LADO_DA_CELULA, forca);
        println!("{nome}  {}", linha_da_grelha(&m, &c, raio, RONDAS_DA_GRELHA, 0.80));
    }
}

fn linha_da_grelha(
    m: &ph2d_mesh::Mesh,
    c: &[[f32; 3]],
    raio: f32,
    rondas: usize,
    k: f32,
) -> String {
    let (g, nb) = grade_da_faixa(m, c, raio);
    let grade = 100.0 * g[0] as f64 / nb.max(1) as f64;
    let (v50, v90, _, _) = vinco_da_faixa(m, c, raio);
    let l = comprimento_por_direccao(m, c, raio);
    format!(
        "{rondas:>6}  {k:>7.3}  {grade:7.2}  {v50:9.3}  {v90:9.3}  {:6.3}  {}",
        l[0] / l[1].max(1e-12),
        m.vert_count()
    )
}

/// O traço da fixtura com **só** a retícula ligada, e com as duas variáveis
/// dela por parâmetro.
fn traco_com_grelha(
    e: [f32; 2],
    raio: f32,
    alvo: f32,
    rondas: usize,
    k_passo: f32,
) -> (ph2d_mesh::Mesh, Vec<[f32; 3]>) {
    traco_com_grelha_com(e, raio, alvo, rondas, k_passo, 0.25)
}

/// O mesmo, com a FORÇA do carimbo por parâmetro — `0` risca sem levantar
/// relevo, que é o controlo da coluna da razão.
fn traco_com_grelha_com(
    e: [f32; 2],
    raio: f32,
    alvo: f32,
    rondas: usize,
    k_passo: f32,
    forca: f32,
) -> (ph2d_mesh::Mesh, Vec<[f32; 3]>) {
    let mut malha = peca_uma_vez();
    malha.triangulate();
    let brush = Brush {
        verb: Verb::Draw,
        radius: raio,
        strength: forca,
        pente: 0.0,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&malha);
    let mut births = Vec::new();
    let mut remap = ph2d_mesh::Remap::default();
    let mut region = ph2d_mesh::RegionScratch::default();
    let mut centros = Vec::new();
    let passo_do_traco = raio * 0.15;
    for j in 0..24 {
        let u = -passo_do_traco * 12.0 + passo_do_traco * j as f32;
        let centro = [u.sin() * e[0], u.sin() * e[1], u.cos()];
        centros.push(centro);
        let direccao = stroke.direccao_do_traco(centro);
        let alvo_col = ph2d_mesh::collapse_target(alvo);
        if matches!(
            ph2d_mesh::collapse_in_sphere_com(
                &mut malha,
                centro,
                raio,
                alvo_col,
                None,
                ph2d_mesh::Guarda::ETambemAForma,
                &mut remap,
                &mut region,
            ),
            ph2d_mesh::Collapse::Done { .. }
        ) {
            stroke.shrink_with(&remap);
        }
        let _ = ph2d_mesh::refine_in_sphere_sized(
            &mut malha, centro, raio, alvo, None, &mut births, &mut region,
        );
        stroke.grow_with(&malha, &births);
        let mut andaram = Vec::new();
        let queda = |p: [f32; 3]| {
            let d = [p[0] - centro[0], p[1] - centro[1], p[2] - centro[2]];
            let r = d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
            brush.falloff.weight(r / raio.max(f32::MIN_POSITIVE))
        };
        if ph2d_quadflow::regiao::arruma_na_grelha_semeada(
            &mut malha,
            centro,
            raio,
            direccao,
            &queda,
            rondas,
            k_passo,
            SEMENTE_UNICA.with(|c| c.get()),
            &mut andaram,
        ) > 0
        {
            malha.refresh_region(&andaram, &mut region);
        }
        stroke.dab(
            &mut malha,
            &brush,
            &Dab::at(centro, raio, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
    }
    (malha, centros)
}

/// ⛔⛔⛔ **SONDA — O QUE A RETÍCULA CUSTA A UM CARIMBO.**
///
/// O orçamento de um dab desta casa é **`8 ms`** (o *kill* que o pincel de plano
/// e o esfregão já mediram), e esta lei acrescenta dois campos e uma cerca de
/// forma ao passe. *Uma lei nova sem o relógio ao lado é um limite por escrever.*
///
/// ⚠️ **Em `--release`:** o debug lê ~`20×` mais lento e daria um número que
/// descreve o perfil de build, não a lei.
///
/// ```text
/// cargo test -p ph2d-app-sculpt3d --release --lib diag_o_relogio_da_reticula -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda: imprime o relógio, não afirma nada"]
fn diag_o_relogio_da_reticula() {
    let raio = raio_do_app();
    println!("vertices    pegada   ms/carimbo   % do orcamento de 8 ms");
    for k in [0usize, 1, 2] {
        let mut malha = peca_uma_vez();
        malha.triangulate();
        for _ in 0..k {
            malha = ph2d_mesh::subdivide(&malha);
            malha.triangulate();
        }
        let centro = [0.0, 0.0, 1.0];
        let direccao = [raio * 0.15, 0.0, 0.0];
        let queda = |p: [f32; 3]| {
            let d = [p[0] - centro[0], p[1] - centro[1], p[2] - centro[2]];
            let r = d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
            ph2d_sculpt3d::Falloff::Smooth.weight(r / raio)
        };
        // ⚠️ **O MÍNIMO de cinco**, e não a média: numa workstation partilhada a
        // média mede a carga dos vizinhos, e o mínimo mede a lei.
        let mut andaram = Vec::new();
        let mut melhor = f64::INFINITY;
        let mut pegada = 0usize;
        for _ in 0..5 {
            let mut m = malha.clone();
            let t = std::time::Instant::now();
            let n = ph2d_quadflow::regiao::arruma_na_grelha_com(
                &mut m,
                centro,
                raio,
                direccao,
                &queda,
                RONDAS_DA_GRELHA,
                LADO_DA_CELULA,
                &mut andaram,
            );
            melhor = melhor.min(t.elapsed().as_secs_f64() * 1000.0);
            pegada = n;
        }
        println!(
            "{:>8}  {pegada:>8}  {melhor:>10.3}  {:>10.1}",
            malha.vert_count(),
            100.0 * melhor / 8.0
        );
    }
}

/// ⛔⛔⛔ **SONDA — DESENHA a retícula com N RONDAS, para se OLHAR.**
///
/// Report do dono (20/09): *«os dois ficaram lisos mas não percebo nenhuma
/// vantagem visualmente»* — com a grade a ler `65 %` contra `33 %`.
///
/// ⚠️ **Olhada, a saída tem RETALHOS de grelha e não FILEIRAS**: cada pedaço
/// tem a própria fase. A hipótese é que `2` rondas de Gauss-Seidel propagam a
/// fase ~`2` anéis e a pegada tem ~`10` — *a escada das rondas não a viu porque
/// a `grade` conta ARESTAS uma a uma e é cega à continuidade*.
///
/// ```text
/// cargo test -p ph2d-app-sculpt3d --release --lib diag_desenha_as_rondas -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda: escreve .ppm para se olhar"]
fn diag_desenha_as_rondas() {
    let dir = std::env::var("PH2D_PENTE_DUMP").unwrap_or_else(|_| "/tmp".into());
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    for rondas in [2usize, 8, 30, 90] {
        let (m, c) = traco_com_grelha(RUMOS[2].1, raio, alvo, rondas, LADO_DA_CELULA);
        let caminho = format!("{dir}/rondas_{rondas:03}.ppm");
        crate::scenes::pente::tests::desenhos::desenha(&m, &c, raio, &caminho);
        let (g, nb) = grade_da_faixa(&m, &c, raio);
        println!(
            "rondas {rondas:>3}: grade {:.2} % -> {caminho}",
            100.0 * g[0] as f64 / nb.max(1) as f64
        );
    }
}

/// ⛔⛔⛔ **SONDA — A SEMENTE do campo de posição, desenhada.**
///
/// Report do dono: *«não percebo nenhuma vantagem visualmente»*. Olhada, a saída
/// tem RETALHOS de grelha com fases diferentes, e a escada das rondas
/// (`2 · 8 · 30 · 90`) **não os junta** — a propagação está refutada.
///
/// A hipótese seguinte é a SEMENTE: a referência faz cada vértice nascer como a
/// **própria** origem de retícula, e um Gauss-Seidel de um nível só só chega a
/// consenso LOCAL (é para isso que o *Instant Meshes* tem hierarquia). Com uma
/// origem ÚNICA a mancha inteira partilha uma retícula.
///
/// ```text
/// cargo test -p ph2d-app-sculpt3d --release --lib diag_desenha_a_semente -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda: escreve .ppm para se olhar"]
fn diag_desenha_a_semente() {
    let dir = std::env::var("PH2D_PENTE_DUMP").unwrap_or_else(|_| "/tmp".into());
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    for (nome, unica, rondas, k) in [
        ("cada_um", false, RONDAS_DA_GRELHA, LADO_DA_CELULA),
        ("unica_r2", true, 2, LADO_DA_CELULA),
        ("unica_r8", true, 8, LADO_DA_CELULA),
        ("unica_k100", true, 2, 1.0),
    ] {
        SEMENTE_UNICA.with(|c| c.set(unica));
        let (m, c) = traco_com_grelha(RUMOS[2].1, raio, alvo, rondas, k);
        SEMENTE_UNICA.with(|c| c.set(false));
        let caminho = format!("{dir}/semente_{nome}.ppm");
        crate::scenes::pente::tests::desenhos::desenha(&m, &c, raio, &caminho);
        let (g, nb) = grade_da_faixa(&m, &c, raio);
        let (v50, v90, _, _) = vinco_da_faixa(&m, &c, raio);
        println!(
            "{nome:<12} grade {:.2} %  vinco {v50:.3}/{v90:.3}  -> {caminho}",
            100.0 * g[0] as f64 / nb.max(1) as f64
        );
    }
}
