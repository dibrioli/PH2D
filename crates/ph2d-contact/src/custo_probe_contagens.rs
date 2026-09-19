//! **DE QUE É FEITO O CUSTO DE UMA VARREDURA — as CONTAGENS, que a CARGA não estraga.**
//!
//! Irmã do [`super`] e da [`super::atribuicao`] pelo tecto de LOC (HR-18) e por ASSUNTO: aquela
//! mede RELÓGIO e esta mede NÚMEROS.
//!
//! ⚠️⚠️ **Ela existe porque o relógio desta máquina não vale nada acima de `load ~5`** e a decisão
//! seguinte não podia esperar por uma máquina calma: *quantos candidatos a grelha entrega*,
//! *quantos deles de facto se tocam* e *quantas peças ainda se mexem na varredura `n`* são
//! contagens **determinísticas** — a mesma corrida dá o mesmo número a `load 0` e a `load 90`.
//!
//! ⭐ **E ela não tem uma segunda cópia da lei:** as duas contagens saem da grelha do produto
//! ([`crate::grelha::Grelha`]) e do [`crate::manifesto`], e a curva de assentamento corre o passe
//! pela porta pública ([`crate::separate`]), **uma varredura de cada vez** — o que é exactamente a
//! mesma coisa que uma corrida de `N` varreduras, porque o laço recopia a fotografia e reconstrói
//! a grelha em cada passagem (a única diferença é a saída antecipada, que aqui não arma).

use super::*;

/// **Os candidatos e os TOQUES de uma configuração** — o numerador e o denominador da pergunta
/// *«quanto do trabalho de uma varredura é rejeição?»*.
///
/// As duas contagens são de pares **ORDENADOS** (cada par a tocar conta duas vezes, uma por lado),
/// porque é assim que a varredura os percorre — ver o cabeçalho do [`crate::varredura`].
fn candidatos_e_toques(c: &[Option<Colisor>], p: &[[f32; 2]]) -> (usize, usize) {
    let n = p.len();
    let vivo: Vec<bool> = (0..n).map(|i| ativo(p[i], c[i].as_ref())).collect();
    let alcance_max = (0..n)
        .filter(|&i| vivo[i])
        .filter_map(|i| c[i].map(|x| x.alcance()))
        .fold(0.0_f32, f32::max);
    if alcance_max <= 0.0 {
        return (0, 0);
    }
    let mut grade = crate::grelha::Grelha::default();
    grade.planeia_numa_camada(&vivo, 2.0 * alcance_max);
    grade.constroi(p, &vivo);
    let mut viz: Vec<u32> = Vec::new();
    let (mut cand, mut toca) = (0usize, 0usize);
    for k in 0..n {
        grade.vizinhos_de(k, &mut viz);
        cand += viz.len();
        for &j in &viz {
            let j = j as usize;
            if j == k || !vivo[j] {
                continue;
            }
            let (lo, hi) = (k.min(j), k.max(j));
            if let (Some(a), Some(b)) = (c[lo], c[hi])
                && manifesto(&a, p[lo], &b, p[hi], (lo + hi) % 2 == 0).is_some()
            {
                toca += 1;
            }
        }
    }
    (cand, toca)
}

/// A mediana de uma lista de alcances.
fn mediana(mut v: Vec<f32>) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(f32::total_cmp);
    v[v.len() / 2]
}

/// ⭐⭐⭐ **UMA PEÇA GRANDE INFLA A GRELHA DE TODAS** — a hipótese que separa a cena do dono
/// (`132`–`156` vizinhos por peça, medidos pelo perfilador dele) da fixtura desta crate
/// (`~11` vizinhos, medidos pela sonda irmã).
///
/// ⚠️ O lado da célula é `2 · alcance_max`, e o `alcance_max` é o **MÁXIMO GLOBAL**. Se uma única
/// peça da cena for `k ×` maior que as outras, a célula de **toda** peça cresce `k ×` e o número
/// de candidatos de **toda** peça cresce `~k²` — *sem que a resposta mude um bit*.
///
/// A fixtura é a mesma nuvem em todas as linhas: só o raio da peça `0` muda.
#[test]
#[ignore = "sonda de medição, não gate"]
fn uma_peca_grande_infla_a_grelha_de_todas() {
    const N: usize = 1000;
    const RAIO: f32 = 100.0;
    let (p, c0, _) = super::atribuicao::campo_de_discos(N, RAIO, 1.8);
    eprintln!("\n  ═══ UMA PEÇA GRANDE INFLA A GRELHA DE TODAS ({N} discos) ═══\n");
    eprintln!("   raio da peça 0 │ alcance max/mediana │ candidatos/peça │ tocam/peça");
    eprintln!("  ────────────────┼─────────────────────┼─────────────────┼───────────");
    for k in [1.0_f32, 2.0, 4.0, 8.0, 16.0] {
        let mut c = c0.clone();
        c[0] = Some(Colisor::disco(RAIO * k));
        let alcances: Vec<f32> = c.iter().filter_map(|x| x.map(|x| x.alcance())).collect();
        let razao = alcances.iter().fold(0.0_f32, |a, b| a.max(*b)) / mediana(alcances);
        let (cand, toca) = candidatos_e_toques(&c, &p);
        #[expect(clippy::cast_precision_loss, reason = "contagens de fixtura")]
        let (cpp, tpp) = (cand as f32 / N as f32, toca as f32 / N as f32);
        eprintln!(
            "        {k:>5.0} × R │            {razao:>6.1} × │        {cpp:>8.1} │   {tpp:>7.2}"
        );
    }
    eprintln!(
        "\n  ⇒ os TOQUES não mudam (a nuvem é a mesma); o que cresce é o que se REJEITA.\n\n  load: {}\n",
        carga()
    );
}

/// ⭐⭐⭐ **QUANTO DE UMA VARREDURA É REJEIÇÃO** — a fracção dos candidatos que não se tocam,
/// varrida pela densidade da nuvem.
///
/// ⚠️ Ela responde à pergunta que decide a wave seguinte: *se `95 %` dos candidatos são rejeitados,
/// o alvo é a REJEIÇÃO (menos candidatos, ou rejeitá-los mais barato) e não a lei do par.*
#[test]
#[ignore = "sonda de medição, não gate"]
fn quanto_de_uma_varredura_e_rejeicao() {
    const N: usize = 1000;
    const RAIO: f32 = 100.0;
    eprintln!("\n  ═══ QUANTO DE UMA VARREDURA É REJEIÇÃO ({N} discos) ═══\n");
    eprintln!("   passo │ candidatos/peça │ tocam/peça │ rejeitados");
    eprintln!("  ───────┼─────────────────┼────────────┼───────────");
    for passo in [1.4_f32, 1.6, 1.8, 2.0, 2.5, 3.0] {
        let (p, c, _) = super::atribuicao::campo_de_discos(N, RAIO, passo);
        let (cand, toca) = candidatos_e_toques(&c, &p);
        #[expect(clippy::cast_precision_loss, reason = "contagens de fixtura")]
        let (cpp, tpp) = (cand as f32 / N as f32, toca as f32 / N as f32);
        let rej = if cand == 0 {
            0.0
        } else {
            100.0 - tpp / cpp * 100.0
        };
        eprintln!("   {passo:>4.1} │        {cpp:>8.1} │  {tpp:>9.2} │    {rej:>5.1} %");
    }
    eprintln!("\n  load: {}\n", carga());
}

/// ⭐⭐⭐ **A CURVA DE ASSENTAMENTO** — quantas peças ainda se mexem na varredura `n`.
///
/// ⚠️ Ela responde à segunda pergunta: *se na varredura `32` só `3 %` das peças ainda se mexem, a
/// varredura `32` está a recalcular `97 %` de um resultado que não vai mudar* — e um conjunto
/// activo seria bit-idêntico **por indução**, exactamente como a saída antecipada global que o
/// [`crate::separate`] já tem.
///
/// ⚠️⚠️ **Corre pela porta do produto, uma varredura de cada vez** — ver o cabeçalho.
#[test]
#[ignore = "sonda de medição, não gate"]
fn a_curva_de_assentamento() {
    const N: usize = 1000;
    const RAIO: f32 = 100.0;
    const ATE: usize = 64;
    let (p0, c, w) = super::atribuicao::campo_de_discos(N, RAIO, 1.8);
    let inv: Vec<f32> = (0..N)
        .map(|i| c[i].map_or(0.0, |x| x.inv_inercia(w[i])))
        .collect();
    let pecas = Pecas::novas(&c, &w, &inv);
    let mut p = p0.clone();
    let mut giro = vec![0.0_f32; N];
    let parado = REPOUSO_VISIVEL * RAIO;
    eprintln!("\n  ═══ A CURVA DE ASSENTAMENTO ({N} discos, passo 1,8·R) ═══\n");
    eprintln!("   varredura │ peças que MEXEM │ acima do repouso │ tocam/peça");
    eprintln!("  ───────────┼─────────────────┼──────────────────┼───────────");
    for v in 1..=ATE {
        let antes = p.clone();
        separate_com(
            &mut p,
            &mut Saida { giro: &mut giro },
            &pecas,
            1,
            false,
            REPOUSO_VISIVEL,
        );
        let mexem = (0..N).filter(|&i| p[i] != antes[i]).count();
        let visiveis = (0..N)
            .filter(|&i| (p[i][0] - antes[i][0]).hypot(p[i][1] - antes[i][1]) >= parado)
            .count();
        if v <= 8 || v % 8 == 0 {
            let (_, toca) = candidatos_e_toques(&c, &p);
            #[expect(clippy::cast_precision_loss, reason = "contagens de fixtura")]
            let tpp = toca as f32 / N as f32;
            eprintln!(
                "      {v:>4} │          {mexem:>6} │           {visiveis:>6} │   {tpp:>7.2}"
            );
        }
    }
    eprintln!("\n  load: {}\n", carga());
}

/// Os candidatos que uma grelha de lado `lado` entrega a um SUBCONJUNTO das peças.
fn candidatos_com(foto: &[[f32; 2]], vivo: &[bool], lado: f32) -> usize {
    let mut grade = crate::grelha::Grelha::default();
    grade.planeia_numa_camada(vivo, lado);
    grade.constroi(foto, vivo);
    let mut viz: Vec<u32> = Vec::new();
    let mut soma = 0usize;
    for (k, v) in vivo.iter().enumerate() {
        if !*v {
            continue;
        }
        grade.vizinhos_de(k, &mut viz);
        soma += viz.len();
    }
    soma
}

/// **Os candidatos das DUAS CAMADAS, previstos sem uma linha de código novo.**
///
/// A previsão é exacta por construção: uma peça pequena vê as `3 × 3` células da grelha FINA (que
/// contém só as pequenas) mais **todas** as grandes; uma grande vê a nuvem activa inteira.
fn candidatos_em_duas_camadas(
    foto: &[[f32; 2]],
    vivo: &[bool],
    alcances: &[f32],
    corte: f32,
) -> usize {
    let n = foto.len();
    let pequena: Vec<bool> = (0..n).map(|i| vivo[i] && alcances[i] <= corte).collect();
    let (g, p, a) = (
        (0..n).filter(|&i| vivo[i] && alcances[i] > corte).count(),
        pequena.iter().filter(|x| **x).count(),
        vivo.iter().filter(|x| **x).count(),
    );
    candidatos_com(foto, &pequena, 2.0 * corte) + p * g + g * a
}

/// ⭐⭐⭐ **ONDE O CORTE PAGA** — de onde saem o [`crate::GRANDES_MAX`] e o [`crate::FATOR_DO_CORTE`].
///
/// ⚠️ O recurso é o **CANDIDATO**: ele é o multiplicador do custo de uma varredura, e a
/// [`crate::Relatorio`] já o publica. As duas colunas são a MESMA nuvem — só muda quem decide o
/// lado da célula.
///
/// ⛔ Uma peça grande **não é de graça na lista dos grandes**: ela passa a ver a nuvem inteira. É
/// isso que faz a coluna virar, e é isso que a cerca tem de nomear.
#[test]
#[ignore = "sonda de medição, não gate"]
fn onde_o_corte_dos_grandes_paga() {
    const N: usize = 1000;
    const RAIO: f32 = 100.0;
    let (p, c0, _) = super::atribuicao::campo_de_discos(N, RAIO, 1.8);
    let vivo = vec![true; N];
    let mede = |g: usize, k: f32| {
        let mut c = c0.clone();
        for x in c.iter_mut().take(g) {
            *x = Some(Colisor::disco(RAIO * k));
        }
        let alcances: Vec<f32> = c.iter().map(|x| x.map_or(0.0, |x| x.alcance())).collect();
        let alcance_max = alcances.iter().fold(0.0_f32, |a, b| a.max(*b));
        let uma = candidatos_com(&p, &vivo, 2.0 * alcance_max);
        let duas = candidatos_em_duas_camadas(&p, &vivo, &alcances, RAIO);
        #[expect(clippy::cast_precision_loss, reason = "contagens de fixtura")]
        let razao = uma as f32 / duas as f32;
        (uma, duas, razao)
    };
    eprintln!("\n  ═══ ONDE O CORTE DOS GRANDES PAGA ({N} discos, passo 1,8·R) ═══\n");
    eprintln!("  (a) quantas peças GRANDES, com 4 × R cada");
    eprintln!("   grandes │ uma camada │ duas camadas │ ganho");
    eprintln!("  ─────────┼────────────┼──────────────┼───────");
    for g in [1usize, 2, 4, 8, 16, 32, 64, 128, 256] {
        let (uma, duas, razao) = mede(g, 4.0);
        eprintln!("    {g:>5} │ {uma:>10} │ {duas:>12} │ {razao:>4.2} ×");
    }
    eprintln!("\n  (b) quão GRANDE precisa ser, com 4 delas");
    eprintln!("   tamanho │ uma camada │ duas camadas │ ganho");
    eprintln!("  ─────────┼────────────┼──────────────┼───────");
    for k in [1.0_f32, 1.25, 1.5, 2.0, 3.0, 4.0, 8.0] {
        let (uma, duas, razao) = mede(4, k);
        eprintln!("   {k:>5.2} × │ {uma:>10} │ {duas:>12} │ {razao:>4.2} ×");
    }
    eprintln!("\n  load: {}\n", carga());
}
