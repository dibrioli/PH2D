//! **O SOMBREAMENTO da casca depois do corte** — a régua que o olho lê, o gate
//! e a sonda que produziu a tabela do handoff.
//!
//! Filho (`#[path]`) do [`super`]; os ajudantes de fixtura são os dele.
//!
//! # ⛔⛔⛔ A nota que esta wave REFUTOU
//!
//! O cabeçalho da [`ph2d_mesh_bool::costura`] dizia, desde 15/09, que os `~632`
//! triângulos piores que `20` que sobravam eram *«cunhas finas onde a curva de
//! interseção passa rente a um vértice da peça»*. **É falso, e por uma margem
//! total:** medido, a esfera de entrada tem `632` triângulos piores que `20` e
//! **todos** têm `|y| > 0,99` — são o **leque do PÓLO** de uma esfera UV, que o
//! doc da [`ph2d_mesh::shapes::sculpt_sphere`] já descreve por escrito. O corte
//! não lhes toca: a régua é que somava a peça inteira.
//!
//! ⚠️ *E a sonda que já existia imprimia a resposta há um dia* — o
//! `diag_o_pico_na_borda` escreve `vértices ANTIGOS: 3/3` ao lado de cada uma —,
//! e eu li aquilo como confirmação da hipótese que tinha.
//!
//! # ⭐ A régua que decide, e as TRÊS que foram construídas e não decidem
//!
//! O que o artista vê não é o aspecto: é a **normal do VÉRTICE**, que o
//! sombreamento usa e que a [`ph2d_mesh::normals`] soma a partir de normais de
//! face **unitárias** (um *gather* sem peso de área) ⇒ *uma lasca vota com peso
//! cheio*. Numa esfera de raio `1` a resposta certa é exacta.
//!
//! Antes desta chegar, três falharam e cada uma pela mesma família:
//!
//! 1. **o desvio radial da normal da FACE** — lê `0,52°` nas lascas, melhor que
//!    a mediana da costura sadia: *uma lasca é fina e PLANA, e uma face plana
//!    tem normal perfeita*;
//! 2. **a MÉDIA das normais das vizinhas** — mede a **QUINA** do corte, não a
//!    lasca (`p99 = 33°` sobre geometria correcta);
//! 3. **o MÍNIMO sobre as vizinhas** — separa a quina, e o **CONTROLO
//!    refutou-a**: a saída CRUA, com aspecto `2 573 809`, lê `1,15°`.
//!
//! ⚠️ E a quarta precisou de **duas** correcções de população antes de dizer a
//! verdade: `r > 0,995` deixa entrar a **parede** do corte, que junto da
//! silhueta é quase tangente à esfera. A população honesta é o vértice cujas
//! faces estão **TODAS** a `|r − 1| < 1e-4`.

use super::*;
use ph2d_mesh::{Mesh, shapes};

/// A esfera do report, cortada por um círculo cujo centro é **parâmetro**.
///
/// ⚠️⚠️ **O centro é parâmetro porque a fixtura CENTRADA não contém o
/// fenómeno:** com o círculo no meio, a borda do corte vive a `|z| = 0,8` e
/// **nunca encontra a silhueta** da peça — ali a saída crua já não tem dano de
/// sombreamento nenhum (`0` vértices acima de `5°`), e um gate escrito só nela
/// ficaria verde por vácuo.
fn corta_a_bola(centro_x: f32) -> (Mesh, Mesh, Mesh) {
    let bola = shapes::sphere_with_triangles(50_000, 1.0);
    let tris: usize = bola
        .faces()
        .iter()
        .map(|f| f.verts().len().saturating_sub(2))
        .sum();
    let alvo = ph2d_mesh::edge_for_tri_count(bola.surface_area(), tris as f32);
    let n = 200usize;
    let anel: Vec<[f32; 2]> = (0..n)
        .map(|i| {
            let t = i as f32 / n as f32 * std::f32::consts::TAU;
            [centro_x + 0.6 * t.cos(), 0.6 * t.sin()]
        })
        .collect();
    let lamina = prisma(
        &anel,
        &raios_orto(&anel),
        &plano(),
        &bola,
        Profundidade::DaPeca,
        Paredes::Fixas,
        Resolucao::Ate(alvo),
    )
    .expect("o prisma");
    let cru = ph2d_mesh_bool::corta_cru(&bola, &lamina, ph2d_mesh_bool::Op::Subtrair).expect("cru");
    let limpo = ph2d_mesh_bool::corta(&bola, &lamina, ph2d_mesh_bool::Op::Subtrair).expect("limpo");
    (bola, cru, limpo)
}

/// **O desvio, em graus, da normal de cada vértice PURO DA CASCA contra a
/// radial** — ver o cabeçalho. Devolve a lista ordenada.
fn desvio_da_casca(m: &Mesh) -> Vec<f32> {
    let p = m.positions();
    let ns = m.normals();
    let mut t = Vec::new();
    for f in m.faces() {
        f.triangles(&mut t);
    }
    let na_casca = |i: u32| {
        let q = p[i as usize];
        ((q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt() - 1.0).abs() < 1e-4
    };
    // Um vértice é PURO da casca quando TODAS as faces dele estão na casca — é
    // isso que tira a quina do corte da população.
    let mut puro = vec![true; p.len()];
    let mut toca = vec![false; p.len()];
    for x in &t {
        let casca = x.iter().all(|&i| na_casca(i));
        for &i in x {
            toca[i as usize] = true;
            if !casca {
                puro[i as usize] = false;
            }
        }
    }
    let mut v: Vec<f32> = Vec::new();
    for (i, q) in p.iter().enumerate() {
        let k = u32::try_from(i).unwrap_or(u32::MAX);
        if !toca[i] || !puro[i] || !na_casca(k) {
            continue;
        }
        let n = ns[i];
        let ln = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if ln <= 0.0 {
            v.push(180.0);
            continue;
        }
        let r = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt();
        let c = ((n[0] * q[0] + n[1] * q[1] + n[2] * q[2]) / (ln * r)).clamp(-1.0, 1.0);
        v.push(c.acos().to_degrees());
    }
    v.sort_by(f32::total_cmp);
    v
}

/// O percentil `q` de uma lista JÁ ORDENADA.
fn pc(v: &[f32], q: usize) -> f32 {
    if v.is_empty() {
        return f32::NAN;
    }
    v[(v.len() - 1) * q / 100]
}

/// Quantos graus de desvio um vértice pode ter sem que isso seja uma mancha.
///
/// ⭐ **Medido, e o vale é de duas ordens de grandeza:** a peça INTACTA lê
/// `0,03°` de máximo, a saída limpa `0,19°`–`0,89°`, e a crua `15,13°`–`56,07°`.
/// *Não há nada entre `1` e `15`.*
const MANCHA_GRAUS: f32 = 5.0;

/// ⭐⭐⭐ **A COSTURA NÃO ESTRAGA O SOMBREAMENTO DA CASCA — e a saída CRUA
/// estraga.**
///
/// Este é o gate que o report *«melhore a topologia das bordas do corte»*
/// pedia e que não existia: os dois que havia mediam **aspecto**, e o aspecto
/// não é o que se vê (ver o cabeçalho — as lascas que sobram têm normal melhor
/// que a costura sadia).
///
/// ⚠️⚠️ **O CONTROLO é metade do gate:** sem a linha que exige que a saída CRUA
/// tenha dano, isto passaria sobre uma limpeza que não fizesse nada. E ele tem
/// de correr numa posição em que o corte **SAI pela silhueta** — na centrada a
/// crua também está limpa.
#[test]
fn a_costura_nao_estraga_o_sombreamento_da_casca() {
    let mut cru_com_dano = 0usize;
    for centro in [0.0f32, 0.3, 0.9] {
        let (bola, cru, limpo) = corta_a_bola(centro);

        // A PEÇA é o chão da régua: se ela lesse mal, o resto não significa nada.
        let da_peca = desvio_da_casca(&bola);
        assert!(
            da_peca.last().copied().unwrap_or(f32::NAN) < 0.5,
            "a régua acusa a esfera INTACTA em {:.2}° — ela está errada",
            da_peca.last().copied().unwrap_or(f32::NAN)
        );

        let d = desvio_da_casca(&limpo);
        assert!(
            d.len() > 18_000,
            "PISO DE POPULAÇÃO: só {} vértices puros da casca em x = {centro} — \
             a régua deixou de achar a casca",
            d.len()
        );
        let acima = d.iter().filter(|&&x| x > MANCHA_GRAUS).count();
        assert_eq!(
            acima,
            0,
            "x = {centro}: {acima} vértices da casca com o sombreamento torcido \
             (pior {:.2}°)",
            d.last().copied().unwrap_or(f32::NAN)
        );

        // ⭐ O CONTROLO: a saída CRUA do motor tem de acusar nalguma posição.
        cru_com_dano += usize::from(desvio_da_casca(&cru).iter().any(|&x| x > MANCHA_GRAUS));
    }
    assert!(
        cru_com_dano > 0,
        "CONTROLO: nenhuma das posições produziu dano na saída CRUA — \
         a fixtura deixou de conter o fenómeno e este gate não afirma nada"
    );
}

/// SONDA (`#[ignore]`) — a tabela do handoff, seis posições do corte.
#[test]
#[ignore = "sonda de diagnóstico: corre à mão"]
fn diag_o_sombreamento_da_casca() {
    println!(
        "{:>7}  {:>28}  {:>28}",
        "centro", "CRUA  (>5°  ·  MAX)", "LIMPA (>5°  ·  MAX)"
    );
    for centro in [0.0f32, 0.3, 0.5, 0.7, 0.8, 0.9] {
        let (_bola, cru, limpo) = corta_a_bola(centro);
        let linha = |v: &[f32]| {
            format!(
                "{:>4}  ·  {:>8.2}°",
                v.iter().filter(|&&x| x > MANCHA_GRAUS).count(),
                v.last().copied().unwrap_or(f32::NAN)
            )
        };
        println!(
            "{centro:>7.1}  {:>28}  {:>28}",
            linha(&desvio_da_casca(&cru)),
            linha(&desvio_da_casca(&limpo))
        );
    }
}

/// ⭐⭐⭐ **A LIGAÇÃO DA PEÇA SOBREVIVE AO CORTE — nenhuma face só de vértices
/// ANTIGOS é INVENTADA.**
///
/// ⛔⛔ **Este gate nasceu de uma mutação SOBREVIVENTE, e o buraco era da casa
/// inteira:** apagar a cerca *«nenhuma das duas faces pode ser inteiramente da
/// PEÇA»* da troca de diagonais **não partia um único teste**. O irmão que devia
/// apanhá-la — o `longe_do_corte_nenhum_vertice_se_move_um_bit` — mede
/// **POSIÇÕES**, e *uma troca de diagonal não move vértice nenhum*: ela reescreve
/// só a LIGAÇÃO. ⇒ *a propriedade que decide a arquitectura desta linha tinha
/// metade sem régua desde que a limpeza existe.*
///
/// A régua é a inclusão: toda face da saída feita só de vértices antigos tem de
/// **já existir na peça**, comparada por posição (os antigos são bit-idênticos
/// por construção, e há gate a dizê-lo). Medido: `41 890`–`42 626` faces dessas
/// por posição do corte, e **zero** inventadas.
#[test]
fn a_ligacao_da_peca_sobrevive_ao_corte() {
    for centro in [0.0f32, 0.3, 0.9] {
        let (bola, _cru, limpo) = corta_a_bola(centro);
        let chave = |m: &Mesh, x: &[u32; 3]| {
            let p = m.positions();
            let mut k: Vec<[u32; 3]> = x
                .iter()
                .map(|&i| {
                    let v = p[i as usize];
                    [v[0].to_bits(), v[1].to_bits(), v[2].to_bits()]
                })
                .collect();
            k.sort_unstable();
            k
        };
        let mut tb = Vec::new();
        for f in bola.faces() {
            f.triangles(&mut tb);
        }
        let da_peca: std::collections::BTreeSet<Vec<[u32; 3]>> =
            tb.iter().map(|x| chave(&bola, x)).collect();
        let antigos: std::collections::BTreeSet<[u32; 3]> = bola
            .positions()
            .iter()
            .map(|v| [v[0].to_bits(), v[1].to_bits(), v[2].to_bits()])
            .collect();
        let p = limpo.positions();
        let mut ts = Vec::new();
        for f in limpo.faces() {
            f.triangles(&mut ts);
        }
        let (mut so_antigos, mut inventados) = (0usize, 0usize);
        for x in &ts {
            let todos_antigos = x.iter().all(|&i| {
                let v = p[i as usize];
                antigos.contains(&[v[0].to_bits(), v[1].to_bits(), v[2].to_bits()])
            });
            if !todos_antigos {
                continue;
            }
            so_antigos += 1;
            if !da_peca.contains(&chave(&limpo, x)) {
                inventados += 1;
            }
        }
        assert!(
            so_antigos > 40_000,
            "PISO DE POPULAÇÃO: só {so_antigos} faces de vértices antigos em \
             x = {centro} — a régua deixou de achar a peça"
        );
        assert_eq!(
            inventados, 0,
            "x = {centro}: {inventados} faces só de vértices ANTIGOS que a peça \
             NÃO tinha — a limpeza reescreveu a ligação da malha do artista"
        );
    }
}

/// ⭐⭐ **A LIMPEZA É UM PONTO FIXO — correr outra vez não troca nada.**
///
/// ⛔ É a régua da **cerca 4** (a melhora estrita da troca de diagonal), que no
/// corpus do corte dispara **uma vez** e não tinha quem a matasse: sem ela um
/// par pode trocar numa passagem e trocar de volta na seguinte, e o resultado
/// deixa de ser função da entrada.
///
/// ⚠️ A comparação é **byte a byte** nas posições e **índice a índice** nas
/// faces: uma oscilação muda a lista de faces sem mover um vértice, que é
/// exactamente o ponto cego que este ficheiro já pagou uma vez.
#[test]
fn a_limpeza_e_um_ponto_fixo() {
    for centro in [0.0f32, 0.3] {
        let (bola, cru, uma) = corta_a_bola(centro);
        let _ = cru;
        let duas = ph2d_mesh_bool::limpa_a_costura(&uma, &bola);
        assert_eq!(
            uma.positions(),
            duas.positions(),
            "x = {centro}: a 2.ª limpeza moveu vértices"
        );
        assert_eq!(
            uma.faces().len(),
            duas.faces().len(),
            "x = {centro}: a 2.ª limpeza mudou a contagem de faces"
        );
        let iguais = uma
            .faces()
            .iter()
            .zip(duas.faces())
            .filter(|(a, b)| a.verts() == b.verts())
            .count();
        assert_eq!(
            iguais,
            uma.faces().len(),
            "x = {centro}: {} faces mudaram na 2.ª limpeza — a lei OSCILA",
            uma.faces().len() - iguais
        );
    }
}

/// SONDA (`#[ignore]`) — o que cada cerca muda na SAÍDA (volume e faces).
#[test]
#[ignore = "sonda de diagnóstico: corre à mão"]
fn diag_o_que_a_cerca_muda() {
    for centro in [0.0f32, 0.3, 0.9] {
        let (_bola, cru, limpo) = corta_a_bola(centro);
        let vol = |m: &Mesh| -> f64 {
            let p = m.positions();
            let mut t = Vec::new();
            for f in m.faces() {
                f.triangles(&mut t);
            }
            t.iter()
                .map(|x| {
                    let (a, b, c) = (p[x[0] as usize], p[x[1] as usize], p[x[2] as usize]);
                    f64::from(
                        a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0])
                            + a[2] * (b[0] * c[1] - b[1] * c[0]),
                    ) / 6.0
                })
                .sum()
        };
        let (vc, vl) = (vol(&cru), vol(&limpo));
        println!(
            "x = {centro}: faces {} -> {}   volume {vc:.9} -> {vl:.9}   (rel {:+.3e})",
            cru.faces().len(),
            limpo.faces().len(),
            (vl - vc) / vc
        );
    }
}

/// Parte os vértices da borda em ANTIGOS e NOVOS, cada um com o desvio ordenado.
fn partido(
    m: &Mesh,
    antigos: &std::collections::BTreeSet<[u32; 3]>,
    alvo: f32,
) -> (Vec<f32>, Vec<f32>) {
    let p = m.positions();
    let mut t = Vec::new();
    for f in m.faces() {
        f.triangles(&mut t);
    }
    let na_casca = |i: u32| {
        let q = p[i as usize];
        ((q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt() - 1.0).abs() < 1e-4
    };
    let mut por_aresta: std::collections::BTreeMap<(u32, u32), Vec<bool>> =
        std::collections::BTreeMap::new();
    for x in &t {
        let casca = x.iter().all(|&i| na_casca(i));
        for (a, b) in [(x[0], x[1]), (x[1], x[2]), (x[2], x[0])] {
            por_aresta
                .entry((a.min(b), a.max(b)))
                .or_default()
                .push(casca);
        }
    }
    let mut verts = std::collections::BTreeSet::new();
    for (k, v) in &por_aresta {
        if v.len() == 2 && v[0] != v[1] {
            verts.insert(k.0);
            verts.insert(k.1);
        }
    }
    let (mut a, mut n) = (Vec::new(), Vec::new());
    for &i in &verts {
        let q = p[i as usize];
        let dv = (q[0].hypot(q[1]) - 0.6).abs() / alvo;
        if antigos.contains(&[q[0].to_bits(), q[1].to_bits(), q[2].to_bits()]) {
            a.push(dv);
        } else {
            n.push(dv);
        }
    }
    a.sort_by(f32::total_cmp);
    n.sort_by(f32::total_cmp);
    (a, n)
}

/// SONDA (`#[ignore]`) — **a BORDA do corte zigue-zagueia: de quem é?**
///
/// O gesto desenhou um círculo de raio `0,6` no plano `xy`, logo a borda vive
/// **exactamente** em `x² + y² = 0,36` sobre a esfera. O desvio mede-se em
/// arestas da peça.
#[test]
#[ignore = "sonda de diagnóstico: corre à mão"]
fn diag_de_quem_e_o_zigue_zague_da_borda() {
    let (bola, cru, limpo) = corta_a_bola(0.0);
    let tris: usize = bola
        .faces()
        .iter()
        .map(|f| f.verts().len().saturating_sub(2))
        .sum();
    let alvo = ph2d_mesh::edge_for_tri_count(bola.surface_area(), tris as f32);
    let na_casca = |p: &[[f32; 3]], i: u32| {
        let q = p[i as usize];
        ((q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt() - 1.0).abs() < 1e-4
    };
    let desvio = |m: &Mesh| -> Vec<f32> {
        let p = m.positions();
        let mut t = Vec::new();
        for f in m.faces() {
            f.triangles(&mut t);
        }
        let mut por_aresta: std::collections::BTreeMap<(u32, u32), Vec<bool>> =
            std::collections::BTreeMap::new();
        for x in &t {
            let casca = x.iter().all(|&i| na_casca(p, i));
            for (a, b) in [(x[0], x[1]), (x[1], x[2]), (x[2], x[0])] {
                por_aresta
                    .entry((a.min(b), a.max(b)))
                    .or_default()
                    .push(casca);
            }
        }
        let mut verts = std::collections::BTreeSet::new();
        for (k, v) in &por_aresta {
            if v.len() == 2 && v[0] != v[1] {
                verts.insert(k.0);
                verts.insert(k.1);
            }
        }
        let mut d: Vec<f32> = verts
            .iter()
            .map(|&i| {
                let q = p[i as usize];
                (q[0].hypot(q[1]) - 0.6).abs() / alvo
            })
            .collect();
        d.sort_by(f32::total_cmp);
        d
    };
    // ⭐ A população parte-se em ANTIGOS e NOVOS: um vértice da borda que é
    // ANTIGO só lá está porque o colapso fundiu um da curva dentro dele.
    let antigos: std::collections::BTreeSet<[u32; 3]> = bola
        .positions()
        .iter()
        .map(|v| [v[0].to_bits(), v[1].to_bits(), v[2].to_bits()])
        .collect();
    for (nome, m) in [("CRUA", &cru), ("LIMPA", &limpo)] {
        let v = desvio(m);
        let (a, n) = partido(m, &antigos, alvo);
        println!(
            "{nome:>6}: n={:>4}  desvio do círculo — p50={:.4} p90={:.4} MAX={:.4}   \
             || ANTIGOS n={:>4} p50={:.4}  ·  NOVOS n={:>4} p50={:.4}",
            v.len(),
            pc(&v, 50),
            pc(&v, 90),
            v.last().copied().unwrap_or(f32::NAN),
            a.len(),
            pc(&a, 50),
            n.len(),
            pc(&n, 50)
        );
    }
}
