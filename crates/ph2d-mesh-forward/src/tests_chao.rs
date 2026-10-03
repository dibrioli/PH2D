//! ⭐⭐⭐ **O CÉU QUE O CHÃO VÊ contra o Cycles** — a mesma cena dos dois lados (uma esfera e uma
//! caixa POUSADAS num chão que só recebe, sob um céu UNIFORME sem caixa nem sol), comparada PASSO A
//! PASSO nas linhas do oráculo (`docs/3DModeling/ferramentas/oraculo_ceu_do_chao_blender.py` →
//! `fixtures/oraculo_ceu_do_chao.csv`). Sob um céu uniforme o escurecimento do chão é a fracção do céu,
//! ponderada pelo cosseno, que as peças tapam — o Cycles dá-a à esfera a `1 %` da forma fechada
//! `(r/D)³`.

use crate::tests::{ID, ambiente, cena, esfera, material_cinza};
use crate::tests_sol::cubo;
use crate::{Camera, Forward, Instancia, Malha};

const LADO: u32 = 384;
/// O enquadramento do oráculo: `(centro x, centro z, meia-aresta)`, visto de cima.
const QUADRO: (f32, f32, f32) = (0.0, 0.0, 1.6);
const ESFERA: ([f32; 3], f32) = ([-0.8, 0.3, 0.0], 0.3);
const CAIXA: ([f32; 3], f32) = ([0.8, 0.2, 0.0], 0.4);
const ORACULO: &str = include_str!("../fixtures/oraculo_ceu_do_chao.csv");
/// Os pixels de chão a menos disto (em pixels) de uma peça não se comparam: a silhueta da esfera de
/// `48 × 96` não é a de `256 × 128` do Blender.
const BORDA: i32 = 2;

/// A pose que só desloca para `p`.
fn em(p: [f32; 3]) -> [[f32; 4]; 4] {
    let mut m = ID;
    m[3][..3].copy_from_slice(&p);
    m
}

/// O quadro visto de cima no enquadramento do oráculo, com a malha `2` na pose `mc` e o chão ou sem
/// ele.
fn de_cima(fw: &mut Forward, mc: [[f32; 4]; 4], chao: bool) -> Vec<u8> {
    de_cima_em(fw, mc, chao, QUADRO)
}

fn de_cima_em(
    fw: &mut Forward,
    mc: [[f32; 4]; 4],
    chao: bool,
    (cx, cz, meia): (f32, f32, f32),
) -> Vec<u8> {
    let me = em(ESFERA.0);
    let objs = [
        Instancia {
            malha: 1,
            modelo: me,
        },
        Instancia {
            malha: 2,
            modelo: mc,
        },
    ];
    let mats = [material_cinza()];
    let cam = Camera {
        view_proj: [
            [1.0 / meia, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.1, 0.0],
            [0.0, -1.0 / meia, 0.0, 0.0],
            [-cx / meia, cz / meia, 0.5, 1.0],
        ],
        olho: [0.0; 3],
        perspectiva: false,
        dir_vista: [0.0, -1.0, 0.0],
    };
    let mut c = cena(&objs, &mats, cam);
    c.tamanho = (LADO, LADO);
    c.chao = chao.then_some(0.0);
    c.caixa_tan = Some(0.47);
    fw.quadro(&c).expect("quadro")
}

/// O nosso chão no enquadramento do oráculo: `escuro` por pixel (`None` = uma peça tapa o chão, ou
/// está a menos de [`BORDA`] dele).
fn o_nosso(fw: &mut Forward) -> Vec<Option<f32>> {
    let img = de_cima(fw, em(CAIXA.0), true);
    let sem_chao = de_cima(fw, em(CAIXA.0), false);
    let peca = |x: i32, y: i32| {
        (x >= 0 && y >= 0 && x < LADO as i32 && y < LADO as i32)
            && sem_chao[((y as u32 * LADO + x as u32) * 4 + 3) as usize] != 0
    };
    (0..LADO * LADO)
        .map(|k| {
            let (x, y) = ((k % LADO) as i32, (k / LADO) as i32);
            let perto = (-BORDA..=BORDA).any(|dy| (-BORDA..=BORDA).any(|dx| peca(x + dx, y + dy)));
            (!perto).then(|| f32::from(img[(k * 4 + 3) as usize]) / 255.0)
        })
        .collect()
}

/// O desenhista com a esfera (`1`) e a caixa (`2`) do oráculo.
fn desenhista() -> Option<Forward> {
    let Some(mut fw) = Forward::no_aparelho(&ambiente()) else {
        eprintln!("sem aparelho — o gate não corre aqui");
        return None;
    };
    for (id, (p, n, idx)) in [(1u64, esfera(ESFERA.1)), (2, cubo(CAIXA.1))] {
        sobe(&mut fw, id, &p, &n, &idx);
    }
    Some(fw)
}

fn sobe(fw: &mut Forward, id: u64, p: &[[f32; 3]], n: &[[f32; 3]], idx: &[u32]) {
    let ao = vec![1.0; p.len()];
    let mat = vec![0u32; p.len()];
    fw.sobe(
        id,
        &Malha {
            posicoes: p,
            normais: n,
            ao: &ao,
            material: &mat,
            indices: idx,
        },
    );
}

/// As barras do gate: `|Δ|` médio, máximo, e máximo na CAUDA (o Cycles abaixo de `0,1`, longe das
/// peças). ⚠️ Medido (03/10, `1024` de cobertura, `512` de céu, `4` fatias, razão `1,25`): médio
/// `≤ 0,0062`, máximo `≤ 0,030`, cauda `≤ 0,016`. Antes desta wave (oclusão por horizonte sobre o
/// topo, 8 direcções × 5 passos, por pixel): médio `0,029–0,063`, máximo `0,10–0,16`.
const BARRA: (f32, f32, f32) = (0.008, 0.04, 0.02);

/// Uma linha do oráculo.
struct Ponto {
    corte: &'static str,
    i: u32,
    j: u32,
    escuro: f32,
}

fn oraculo() -> Vec<Ponto> {
    ORACULO
        .lines()
        .filter(|l| !l.starts_with('#'))
        .skip(1)
        .map(|l| {
            let c: Vec<&'static str> = l.split(',').collect();
            Ponto {
                corte: c[0],
                i: c[1].parse().expect("i"),
                j: c[2].parse().expect("j"),
                escuro: c[5].parse().expect("escuro"),
            }
        })
        .collect()
}

/// ⭐⭐⭐ **O chão escurece o que o Cycles escurece**, passo a passo: a linha do meio (as duas peças),
/// a coluna de cada uma e a diagonal da caixa (a quina).
#[test]
#[ignore = "precisa de aparelho"]
fn o_ceu_do_chao_e_o_do_cycles() {
    let cena_linha = ORACULO
        .lines()
        .find(|l| l.starts_with("# CENA"))
        .expect("o oráculo tem a linha da cena");
    assert!(
        cena_linha.contains(
            "lado=384 quadro=(0.0, 0.0, 1.6) esfera=((-0.8, 0.3, 0.0), 0.3) \
             caixa=((0.8, 0.2, 0.0), 0.4) céu=uniforme"
        ),
        "o CSV é de outra cena: {cena_linha}"
    );
    let Some(mut fw) = desenhista() else {
        return;
    };
    let nosso = o_nosso(&mut fw);
    let pontos = oraculo();
    let perfil = std::env::var("PH2D_CHAO_PERFIL").ok();
    let mut cortes: Vec<&str> = Vec::new();
    for p in &pontos {
        if !cortes.contains(&p.corte) {
            cortes.push(p.corte);
        }
    }
    let mut falhas = Vec::new();
    for corte in cortes {
        let (mut pior, mut soma, mut n) = (0.0f32, 0.0f32, 0usize);
        let mut cauda = 0.0f32;
        let mut apagados = 0usize;
        for p in pontos.iter().filter(|p| p.corte == corte) {
            let Some(e) = nosso[(p.j * LADO + p.i) as usize] else {
                continue;
            };
            if perfil.as_deref() == Some(corte) {
                eprintln!("P {} {} {e:.3} {:.3}", p.i, p.j, p.escuro);
            }
            let d = (e - p.escuro).abs();
            if p.escuro < 0.1 {
                cauda = cauda.max(d);
            }
            // O céu tapado não pode SUMIR longe das peças (o Cycles ainda escurece ali).
            if p.escuro >= 0.02 && e == 0.0 {
                apagados += 1;
            }
            pior = pior.max(d);
            soma += d;
            n += 1;
        }
        let medio = soma / n as f32;
        eprintln!(
            "{corte}: {n} px do chão · |Δ| médio {medio:.4} · máx {pior:.3} · na cauda {cauda:.3} · \
             {apagados} apagados"
        );
        if n < 100 || medio > BARRA.0 || pior > BARRA.1 || cauda > BARRA.2 || apagados > 0 {
            falhas.push(format!(
                "{corte}: {n} px · médio {medio:.4} · máx {pior:.3} · cauda {cauda:.3} · \
                 {apagados} apagados"
            ));
        }
    }
    assert!(falhas.is_empty(), "o chão afastou-se do Cycles: {falhas:?}");
}

/// ⭐ **A cauda do céu do chão não tem degrau** — longe das peças o escurecimento cai como `(r/D)³` e
/// não pode SUMIR de repente onde a busca da luz-chave deixa de achar peça (`~1,3` do centro dela).
/// Na linha `z = 0` de um enquadramento de `±3`, até `1,6` de distância da esfera (a forma fechada
/// dá ali `0,0063`, `~1,6` byte) o chão tem de escurecer, e na cauda (abaixo de `25` bytes) dois
/// vizinhos nunca diferem mais de `3`.
#[test]
#[ignore = "precisa de aparelho"]
fn a_cauda_do_ceu_do_chao_nao_tem_degrau() {
    let Some(mut fw) = desenhista() else {
        return;
    };
    let meia = 3.0;
    let img = de_cima_em(&mut fw, em(CAIXA.0), true, (0.0, 0.0, meia));
    let y = LADO / 2;
    let alfa = |i: u32| img[((y * LADO + i) * 4 + 3) as usize];
    let x = |i: u32| (i as f32 + 0.5) / LADO as f32 * 2.0 * meia - meia;
    let mut degraus = Vec::new();
    for i in 1..LADO {
        // Só na CAUDA (os dois abaixo de `25`): junto da peça o escurecimento é íngreme de verdade.
        let d = alfa(i).abs_diff(alfa(i - 1));
        if d > 3 && alfa(i).max(alfa(i - 1)) < 25 {
            degraus.push((x(i), alfa(i - 1), alfa(i)));
        }
    }
    let apagados: Vec<f32> = (0..LADO)
        .filter(|&i| (x(i) - ESFERA.0[0]).abs() <= 1.6 && x(i) < ESFERA.0[0] && alfa(i) == 0)
        .map(x)
        .collect();
    assert!(degraus.is_empty(), "degraus no chão: {degraus:?}");
    assert!(
        apagados.is_empty(),
        "o céu do chão sumiu a menos de 1,6 da esfera em x = {apagados:?}"
    );
}

/// ⭐⭐ **O céu do chão segue as peças** — ele só se refaz quando a chave muda; aqui, a chave tem de
/// mudar com a POSE e com a FORMA. Mover a caixa, trocar a malha dela (o MESMO id, outra forma) e
/// GIRAR uma peça sem mudar o enquadramento (a caixa dela é a mesma) dão, ao byte, o quadro de um
/// desenhista novo que nunca viu a cena antiga. Controlo: cada mudança muda o chão.
#[test]
#[ignore = "precisa de aparelho"]
fn o_ceu_do_chao_segue_as_pecas() {
    let (Some(mut fw), Some(mut novo)) = (desenhista(), desenhista()) else {
        return;
    };
    let antes = de_cima(&mut fw, em(CAIXA.0), true);
    let movida = em([0.4, 0.2, 0.3]);
    let depois = de_cima(&mut fw, movida, true);
    assert_ne!(antes, depois, "o controlo: mover a caixa muda o chão");
    assert!(
        depois == de_cima(&mut novo, movida, true),
        "o céu do chão ficou o da pose antiga"
    );
    // Uma esfera fora do centro da sua caixa (um triângulo degenerado em `+x` alarga-a): meia volta
    // em `y` deixa a caixa — e o enquadramento — iguais e muda o chão.
    let (mut p, mut n, mut idx) = esfera(0.2);
    for v in &mut p {
        v[0] -= 0.1;
    }
    let k = p.len() as u32;
    p.extend([[0.3, 0.0, 0.0]; 3]);
    n.extend([[1.0, 0.0, 0.0]; 3]);
    idx.extend([k, k + 1, k + 2]);
    sobe(&mut fw, 2, &p, &n, &idx);
    let trocada = de_cima(&mut fw, movida, true);
    let mut girada = movida;
    girada[0][0] = -1.0;
    girada[2][2] = -1.0;
    let virada = de_cima(&mut fw, girada, true);
    assert_ne!(trocada, virada, "o controlo: girar a peça muda o chão");
    let Some(mut outro) = desenhista() else {
        return;
    };
    sobe(&mut outro, 2, &p, &n, &idx);
    assert!(
        trocada == de_cima(&mut outro, movida, true),
        "o céu do chão ficou o da forma antiga"
    );
    let Some(mut outro) = desenhista() else {
        return;
    };
    sobe(&mut outro, 2, &p, &n, &idx);
    assert!(
        virada == de_cima(&mut outro, girada, true),
        "o céu do chão ficou o da peça antes de girar"
    );
}
