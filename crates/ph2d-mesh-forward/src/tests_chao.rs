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

/// O quadro visto de cima no enquadramento do oráculo, com a caixa em `caixa` e o chão ou sem ele.
fn de_cima(fw: &mut Forward, caixa: [f32; 3], chao: bool) -> Vec<u8> {
    let (cx, cz, meia) = QUADRO;
    let mut me = ID;
    me[3][..3].copy_from_slice(&ESFERA.0);
    let mut mc = ID;
    mc[3][..3].copy_from_slice(&caixa);
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
    let img = de_cima(fw, CAIXA.0, true);
    let sem_chao = de_cima(fw, CAIXA.0, false);
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
            pior = pior.max(d);
            soma += d;
            n += 1;
        }
        let medio = soma / n as f32;
        eprintln!(
            "{corte}: {n} px do chão · |Δ| médio {medio:.4} · máx {pior:.3} · na cauda {cauda:.3}"
        );
        if n < 100 || medio > BARRA.0 || pior > BARRA.1 || cauda > BARRA.2 {
            falhas.push(format!(
                "{corte}: {n} px · médio {medio:.4} · máx {pior:.3} · cauda {cauda:.3}"
            ));
        }
    }
    assert!(falhas.is_empty(), "o chão afastou-se do Cycles: {falhas:?}");
}

/// ⭐⭐ **O céu do chão segue as peças** — ele só se refaz quando a chave muda; aqui, a chave tem de
/// mudar com a POSE e com a FORMA. Mover a caixa e trocar a malha dela (o MESMO id, outra forma) dão,
/// ao byte, o quadro de um desenhista novo que nunca viu a cena antiga. Controlo: mover muda o chão.
#[test]
#[ignore = "precisa de aparelho"]
fn o_ceu_do_chao_segue_as_pecas() {
    let (Some(mut fw), Some(mut novo)) = (desenhista(), desenhista()) else {
        return;
    };
    let antes = de_cima(&mut fw, CAIXA.0, true);
    let movida = [0.4, 0.2, 0.3];
    let depois = de_cima(&mut fw, movida, true);
    assert_ne!(antes, depois, "o controlo: mover a caixa muda o chão");
    assert!(
        depois == de_cima(&mut novo, movida, true),
        "o céu do chão ficou o da pose antiga"
    );
    let (p, n, idx) = esfera(0.2);
    sobe(&mut fw, 2, &p, &n, &idx);
    let (Some(mut outro), trocada) = (desenhista(), de_cima(&mut fw, movida, true)) else {
        return;
    };
    sobe(&mut outro, 2, &p, &n, &idx);
    assert!(
        trocada == de_cima(&mut outro, movida, true),
        "o céu do chão ficou o da forma antiga"
    );
}
