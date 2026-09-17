//! **A FRONTEIRA da pose** — porque a borda da deformação entalha, e de onde
//! vem a largura de fábrica da [`crate::PoseControlos::transicao`].
//!
//! ⚠️ **O que só aqui se pode afirmar.** A bancada da `ph2d-pose` mede a LEI
//! contra `69` traços do oráculo, com os controlos que cada fixtura declara —
//! e **nenhuma delas corre no regime do report**: uma malha fina com um arrasto
//! grande. *Uma paridade medida noutro regime não afirma nada sobre este*, e o
//! defeito que o dono fotografou em 2026-09-17 vive exactamente aqui.
//!
//! ⛔⛔ **A régua é a FACE VIRADA DO AVESSO, e não a suavidade.** A região da
//! pose nasce binária (§2.2 escreve `1` em quem alcança e `0` no resto) e o
//! que a esbate é a TRANSIÇÃO; a banda entre os dois tem de absorver a
//! rotação inteira, e quando ela é estreita de mais a superfície **dobra sobre
//! si mesma**. *O entalhe escuro que o artista vê é uma normal invertida* — e
//! medi-lo é binário, ao contrário de «está suave?».

use ph2d_mesh::Mesh;

use crate::{Brush, Dab, PoseControlos, SculptStroke, Symmetry, Verb};

/// ⚠️⚠️ **A densidade é load-bearing, e está MEDIDA: o fenómeno NÃO EXISTE numa
/// malha grossa.** Varridas quatro densidades com o mesmo gesto, as faces
/// viradas no tecto antigo (`N = 100`) leem `0` a `1 490`, `0` a `6 050`, `0` a
/// `24 386` e **`1 390`** a `97 922` vértices. ⇒ *um gate escrito numa esfera
/// de teste barata ficaria verde sobre o defeito do report*, que é a forma
/// que este repo já pagou com o plano ancorado na origem e com o viewport de
/// 2 400 px.
///
/// `97 922` é a densidade da peça de fábrica do módulo, que tem `98 306`.
fn esfera_do_report() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(256, 384, 1.0)
}

fn pincel(transicao: f32) -> Brush {
    Brush {
        verb: Verb::Pose,
        radius: 0.8,
        strength: 1.0,
        pose: PoseControlos {
            transicao,
            ..PoseControlos::default()
        },
        ..Brush::default()
    }
}

fn puxao(centro: [f32; 3], raio: f32, puxao: [f32; 3]) -> Dab {
    let l = (centro[0] * centro[0] + centro[1] * centro[1] + centro[2] * centro[2]).sqrt();
    let olho = [-centro[0] / l, -centro[1] / l, -centro[2] / l];
    Dab::pulling(centro, raio, olho, puxao)
}

/// Quantas faces saíram deste gesto com a normal **invertida**.
fn faces_viradas(malha: &Mesh, transicao: f32, arrasto: f32) -> usize {
    let mut m = malha.clone();
    let antes = m.positions().to_vec();
    let b = pincel(transicao);
    let mut s = SculptStroke::default();
    s.begin(&m);
    s.dab(
        &mut m,
        &b,
        &puxao([0.0, 0.0, 1.0], b.radius, [arrasto, 0.0, 0.0]),
        Symmetry::default(),
    );
    let depois = m.positions().to_vec();
    let normal = |p: &[[f32; 3]], f: &ph2d_mesh::Face| {
        let v = f.verts();
        let (a, b, c) = (p[v[0] as usize], p[v[1] as usize], p[v[2] as usize]);
        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let w = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        [
            u[1] * w[2] - u[2] * w[1],
            u[2] * w[0] - u[0] * w[2],
            u[0] * w[1] - u[1] * w[0],
        ]
    };
    m.faces()
        .iter()
        .filter(|f| {
            let (a, b) = (normal(&antes, f), normal(&depois, f));
            a[0] * b[0] + a[1] * b[1] + a[2] * b[2] < 0.0
        })
        .count()
}

/// ⭐⭐⭐ **A LARGURA DE FÁBRICA É O PONTO EM QUE A DOBRA MORRE — e o gate EXIGE
/// que abaixo dela ainda dobre**, senão o número era uma licença.
///
/// Report do dono, 2026-09-17: *«por que essas reentrâncias com pose? … mesmo
/// com Weight Smoothing no máximo não consigo uma transição mais suave»*.
///
/// ⚠️⚠️ **A PREMISSA DO GATE ANTERIOR MORREU AQUI.** Ele chamava-se
/// `o_tecto_das_suavizacoes_e_onde_a_dobra_morre` e media o tecto de uma
/// contagem de **iterações de difusão** (`100 → 300`); a lei do produto passou
/// a ser a **distância no barro**, e com ela o tecto deixou de ser onde a dobra
/// morre — quem a mata é a LARGURA, e o tecto passou a ser outro recurso (o
/// núcleo). *A morte está no `MEMORIAS` e a medição no doc do
/// [`PoseControlos::TRANSICAO_MAX`].*
#[test]
fn a_transicao_de_fabrica_e_onde_a_dobra_morre() {
    let malha = esfera_do_report();
    let arrasto = 0.6;
    let fabrica = PoseControlos::TRANSICAO_DE_FABRICA;

    // (1) — **o controlo positivo, e é ele que torna o resto uma afirmação.**
    // Mais estreita que a de fábrica, o mesmo gesto dobra a malha: sem esta
    // metade, um gate que só olhasse o `0` ficaria verde num arranjo onde nada
    // dobra.
    let estreita = faces_viradas(&malha, fabrica * 2.0 / 3.0, arrasto);
    assert!(
        estreita > 100,
        "a dois tercos da largura de fabrica so' {estreita} faces viraram do \
         avesso — o arranjo deixou de conter o fenomeno do report, e a metade \
         (2) deste gate passa a afirmar o nada"
    );

    // (2) — na largura de fábrica a dobra desaparece, que é o que o dono pediu.
    let de_fabrica = faces_viradas(&malha, fabrica, arrasto);
    assert_eq!(
        de_fabrica, 0,
        "na largura de fabrica ({fabrica}) ainda viraram {de_fabrica} faces — \
         ela deixou de ser o ponto em que a dobra morre"
    );

    // (3) — **e o TECTO é de outro recurso, que é o que o separa da largura:**
    // acima dele a transição come o NÚCLEO, e o vértice sob o cursor deixa de
    // se mover inteiro. Medido: `1,0000` até `2,0·R` e `0,9394` a `3,0·R`.
    let nucleo = |t: f32| -> f32 {
        let mut m = malha.clone();
        let b = pincel(t);
        let mut s = SculptStroke::default();
        s.begin(&m);
        s.dab(
            &mut m,
            &b,
            &puxao([0.0, 0.0, 1.0], b.radius, [0.05, 0.0, 0.0]),
            Symmetry::default(),
        );
        let sessao = s.pose_sessao().expect("a sessao vive durante o traco");
        let cadeia = sessao.cadeia();
        let pos = malha.positions();
        let eleito = pos
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let d = |p: &[f32; 3]| p[0] * p[0] + p[1] * p[1] + (p[2] - 1.0).powi(2);
                d(a).total_cmp(&d(b))
            })
            .map(|(i, _)| i)
            .expect("a malha tem vertices");
        cadeia.peso_total(eleito)
    };
    assert!(
        nucleo(PoseControlos::TRANSICAO_MAX) > 0.999,
        "no tecto ({}) o nucleo ja' diluiu para {:.4} — o tecto passou do ponto \
         em que o recurso que o nomeia acaba",
        PoseControlos::TRANSICAO_MAX,
        nucleo(PoseControlos::TRANSICAO_MAX)
    );
    let acima = nucleo(PoseControlos::TRANSICAO_MAX * 1.5);
    assert!(
        acima < 0.99,
        "meio acima do tecto o nucleo ainda vale {acima:.4} — o tecto esta' \
         mais baixo do que a medicao pede, e um tecto a menos e' uma faixa que \
         o artista nao alcanca"
    );
}

/// ⭐⭐⭐ **A FAIXA MEDE O BARRO, NÃO ANÉIS DA MALHA — a premissa do gate
/// anterior MORREU, e é isso que este afirma.**
///
/// ⚠️⚠️ **O irmão que estava aqui chamava-se
/// `a_banda_conta_aneis_da_malha_e_nao_raios_do_pincel` e afirmava o DEFEITO de
/// propósito**, com esta frase escrita nele: *«no dia em que a banda passar a
/// ancorar-se no raio do pincel ele reprova, e a premissa morre à vista no
/// diff»*. O dia foi 2026-09-17 e a cura foi trocar a lei do peso — a difusão
/// por uma **distância nas arestas** ([`ph2d_pose::pesos::por_distancia`]).
///
/// ⭐ **O par de densidades é o mesmo, e é o que torna o discriminador limpo:**
/// `24 386` e `97 922` vértices têm arestas na razão `2,00`. Na lei de hoje a
/// faixa lê `0,266` e `0,265` em **mundo** (razão `1,003`) contra `11,7` e
/// `23,3` em **arestas** (razão `2,00`) — *exactamente as duas colunas do gate
/// antigo, trocadas.*
#[test]
fn a_faixa_mede_o_barro_e_nao_aneis_da_malha() {
    let media = ph2d_mesh::shapes::uv_sphere(128, 192, 1.0);
    let fina = esfera_do_report();
    let t = PoseControlos::TRANSICAO_DE_FABRICA;
    let (bm, am) = banda_em_arestas(&media, t).expect("a malha media tem faixa");
    let (bf, af) = banda_em_arestas(&fina, t).expect("a malha fina tem faixa");

    // (0) — **o controlo do arranjo:** sem arestas de facto diferentes, as duas
    // metades abaixo seriam a mesma afirmação.
    let razao_das_arestas = am / af;
    assert!(
        (1.7..2.3).contains(&razao_das_arestas),
        "as duas malhas deixaram de estar a um factor de dois de densidade \
         (arestas {am:.5} e {af:.5}, razao {razao_das_arestas:.3}) — o \
         discriminador deste gate desapareceu"
    );

    // (1) — em MUNDO as duas concordam: a grandeza que o knob compra é do BARRO.
    let (mm, mf) = (bm * am, bf * af);
    let em_mundo = mm / mf;
    assert!(
        (0.93..1.08).contains(&em_mundo),
        "a faixa em mundo leu {mm:.4} na media e {mf:.4} na fina (razao \
         {em_mundo:.3}, medido 1,003) — se ela deixou de ser uma distancia, \
         alguem devolveu o peso a' difusao"
    );

    // (2) — e em ARESTAS elas NÃO concordam, que é a mesma saída vista na
    // unidade errada: a malha fina tem arestas metade, logo a mesma faixa cobre
    // o dobro delas.
    assert!(
        bf > bm * 1.7,
        "a faixa em arestas leu {bm:.2} na media e {bf:.2} na fina (medido \
         11,7 contra 23,3) — se as duas passaram a concordar, a faixa voltou a \
         ser uma contagem de aneis"
    );
}

/// A largura da transição (a média por concha cruza `0,9` e depois `0,1`), em
/// arestas médias, e a aresta média.
///
/// ⚠️⚠️ **A média por CONCHA, e não o extremo por vértice.** A 1.ª redacção
/// tomava `max{d : w ≥ 0,9}` e `min{d : w ≤ 0,1}`, e **um** vértice fora do
/// sítio alarga a banda inteira: ela leu `22,5` contra `18,0` na mesma lei
/// (`razão 1,25`) onde o estimador por concha lê `17,5` contra `17,7`.
/// *Um estimador feito de extremos mede a cauda da amostra, não a lei.*
fn banda_em_arestas(malha: &Mesh, transicao: f32) -> Option<(f32, f32)> {
    let mut m = malha.clone();
    let b = pincel(transicao);
    let mut s = SculptStroke::default();
    s.begin(&m);
    s.dab(
        &mut m,
        &b,
        &puxao([0.0, 0.0, 1.0], b.radius, [0.05, 0.0, 0.0]),
        Symmetry::default(),
    );
    let sessao = s.pose_sessao().expect("a sessao vive durante o traco");
    let cadeia = sessao.cadeia();
    let pos = malha.positions();

    // A aresta média — a unidade em que a banda se lê.
    let mut soma = 0.0f64;
    let mut n_arestas = 0u64;
    for f in malha.faces() {
        let vs = f.verts();
        for k in 0..vs.len() {
            let a = pos[vs[k] as usize];
            let c = pos[vs[(k + 1) % vs.len()] as usize];
            soma += f64::from(
                ((a[0] - c[0]).powi(2) + (a[1] - c[1]).powi(2) + (a[2] - c[2]).powi(2)).sqrt(),
            );
            n_arestas += 1;
        }
    }
    let aresta = (soma / n_arestas as f64) as f32;

    // O perfil `w(d)` por conchas de uma aresta de largura.
    let cursor = [0.0f32, 0.0, 1.0];
    let conchas = (2.0 / aresta).ceil() as usize;
    let mut acumulado = vec![0.0f64; conchas];
    let mut contagem = vec![0u32; conchas];
    for (v, &q) in pos.iter().enumerate() {
        let d =
            ((q[0] - cursor[0]).powi(2) + (q[1] - cursor[1]).powi(2) + (q[2] - cursor[2]).powi(2))
                .sqrt();
        let k = (d / aresta) as usize;
        if k < conchas {
            acumulado[k] += f64::from(cadeia.peso_total(v));
            contagem[k] += 1;
        }
    }
    let perfil: Vec<(f32, f64)> = (0..conchas)
        .filter(|&k| contagem[k] >= 4)
        .map(|k| {
            (
                (k as f32 + 0.5) * aresta,
                acumulado[k] / f64::from(contagem[k]),
            )
        })
        .collect();
    let cruza = |alvo: f64| -> Option<f32> {
        perfil.windows(2).find_map(|par| {
            let ((d0, m0), (d1, m1)) = (par[0], par[1]);
            (m0 >= alvo && m1 < alvo).then(|| {
                let t = ((m0 - alvo) / (m0 - m1).max(1e-12)) as f32;
                d0 + (d1 - d0) * t
            })
        })
    };
    // ⚠️ `None` quando a difusão **diluiu o próprio núcleo** e o perfil nunca
    // chega a `0,9` — medido: numa malha de `1 490` vértices isso acontece a
    // partir de `N = 100`. *Ali a banda deixa de existir como fronteira: o que
    // há é um pincel mais fraco.*
    let d90 = cruza(0.9)?;
    let d10 = cruza(0.1)?;
    Some(((d10 - d90).max(0.0) / aresta, aresta))
}

#[test]
#[ignore]
fn sonda_da_banda() {
    for (a, s) in [(32usize, 48usize), (64, 96), (128, 192), (256, 384)] {
        let m = ph2d_mesh::shapes::uv_sphere(a, s, 1.0);
        let vs = m.positions().len();
        for t in [0.2f32, 0.4, 0.6, 1.0, 2.0] {
            let Some((b, ar)) = banda_em_arestas(&m, t) else {
                println!("uv {a}x{s} v={vs} t={t} — sem faixa");
                continue;
            };
            println!(
                "uv {a}x{s} v={vs} aresta={ar:.5} t={t} faixa_ar={b:.2} faixa_mundo={:.4} faixa_R={:.3}",
                b * ar,
                b * ar / 0.8
            );
        }
    }
}

#[test]
#[ignore]
fn sonda_das_viradas() {
    let m = esfera_do_report();
    println!("arrasto  t=0.6  t=0.7  t=0.8  t=0.9  t=1.0  t=1.2");
    for g in [0.1f32, 0.2, 0.4, 0.6, 0.9, 1.2] {
        print!("{g:>7.2}");
        for t in [0.6f32, 0.7, 0.8, 0.9, 1.0, 1.2] {
            print!("{:>7}", faces_viradas(&m, t, g));
        }
        println!();
    }
}

#[test]
#[ignore]
fn sonda_de_onde_vivem_as_viradas() {
    let malha = esfera_do_report();
    for t in [0.0f32, 0.2, 0.4, 0.6, 1.0] {
        let mut m = malha.clone();
        let antes = m.positions().to_vec();
        let b = pincel(t);
        let mut s = SculptStroke::default();
        s.begin(&m);
        s.dab(
            &mut m,
            &b,
            &puxao([0.0, 0.0, 1.0], b.radius, [0.6, 0.0, 0.0]),
            Symmetry::default(),
        );
        let w: Vec<f32> = {
            let cad = s.pose_sessao().expect("sessao").cadeia();
            (0..antes.len()).map(|v| cad.peso_total(v)).collect()
        };
        let depois = m.positions().to_vec();
        let nrm = |p: &[[f32; 3]], f: &ph2d_mesh::Face| {
            let v = f.verts();
            let (a, c, d) = (p[v[0] as usize], p[v[1] as usize], p[v[2] as usize]);
            let u = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let z = [d[0] - a[0], d[1] - a[1], d[2] - a[2]];
            [
                u[1] * z[2] - u[2] * z[1],
                u[2] * z[0] - u[0] * z[2],
                u[0] * z[1] - u[1] * z[0],
            ]
        };
        let (mut total, mut na_banda, mut no_miolo, mut fora) = (0, 0, 0, 0);
        for f in m.faces() {
            let (a, c) = (nrm(&antes, f), nrm(&depois, f));
            if a[0] * c[0] + a[1] * c[1] + a[2] * c[2] >= 0.0 {
                continue;
            }
            total += 1;
            let pesos: Vec<f32> = f.verts().iter().map(|&v| w[v as usize]).collect();
            let alto = pesos.iter().copied().fold(0.0f32, f32::max);
            let baixo = pesos.iter().copied().fold(1.0f32, f32::min);
            if baixo >= 0.999 {
                no_miolo += 1;
            } else if alto <= 0.001 {
                fora += 1;
            } else {
                na_banda += 1;
            }
        }
        println!("t={t}: viradas={total} na_banda={na_banda} no_miolo={no_miolo} fora={fora}");
    }
}
