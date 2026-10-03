//! ⭐⭐⭐ **A SOMBRA DO SOL contra o Cycles** — a mesma cena dos dois lados (uma esfera a flutuar e uma
//! caixa pousada num chão que só recebe, sob um sol de DISCO de direcção e raio conhecidos), comparada
//! PASSO A PASSO nas linhas e colunas do oráculo
//! (`docs/3DModeling/ferramentas/oraculo_sombra_sol_blender.py` → `fixtures/oraculo_sombra_sol.csv`).
//!
//! ⭐ Do nosso lado o sol NÃO é dado: é um céu preto com o disco pintado, e o desenhista acha-o
//! ([`ph2d_sky::Ceu::com_sol`]) — a separação e a sombra são medidas juntas.

use crate::tests::{ID, ambiente, cena, esfera, material_cinza};
use crate::{Camera, Forward, Foto, Instancia, Malha};

const LADO: u32 = 384;
const CENTRO_X: f32 = 0.1;
const MEIA: f32 = 1.5;
const ESFERA: ([f32; 3], f32) = ([-0.8, 0.6, 0.0], 0.3);
const CAIXA: ([f32; 3], f32) = ([0.6, 0.2, 0.0], 0.4);
const ORACULO: &str = include_str!("../fixtures/oraculo_sombra_sol.csv");

/// Uma caixa de aresta `a`, faces planas.
fn cubo(a: f32) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let h = a * 0.5;
    let (mut p, mut n, mut idx) = (Vec::new(), Vec::new(), Vec::new());
    for eixo in 0..3 {
        for s in [-1.0f32, 1.0] {
            let mut nn = [0.0; 3];
            nn[eixo] = s;
            let (u, v) = ((eixo + 1) % 3, (eixo + 2) % 3);
            let base = p.len() as u32;
            for (du, dv) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                let mut q = [0.0; 3];
                q[eixo] = s * h;
                q[u] = du * h;
                q[v] = dv * h;
                p.push(q);
                n.push(nn);
            }
            idx.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
    (p, n, idx)
}

/// O céu do oráculo: preto, com um disco de radiância uniforme a vir de `L = (−cos h, sen h, 0)`.
fn ceu_com_disco(altura: f32, raio: f32) -> ph2d_sky::Panorama {
    let (h, r) = (altura.to_radians(), raio.to_radians());
    let l = [-h.cos(), h.sin(), 0.0];
    let mut p = ph2d_sky::Panorama {
        largura: 1024,
        altura: 512,
        rgb: vec![[0.0; 3]; 1024 * 512],
    };
    for y in 0..p.altura {
        for x in 0..p.largura {
            let d = p.direcao(x, y);
            let c = d[0] * l[0] + d[1] * l[1] + d[2] * l[2];
            if c >= r.cos() {
                p.rgb[(y * p.largura + x) as usize] = [1000.0; 3];
            }
        }
    }
    p
}

/// O nosso chão no enquadramento do oráculo: `escuro` por pixel (`None` = um objeto tapa o chão).
fn o_nosso(fw: &mut Forward, altura: f32, raio: f32, chave: f32) -> Vec<Option<f32>> {
    let ceu = ph2d_sky::Ceu::com_sol(&ceu_com_disco(altura, raio));
    let sol = ceu.sol().expect("o disco é um sol");
    eprintln!(
        "sol achado: dir {:?} raio {:.3}° (pintado {raio}°)",
        sol.dir,
        sol.raio.to_degrees()
    );
    fw.sobe_ceu(&ceu);
    let mut me = ID;
    me[3][..3].copy_from_slice(&ESFERA.0);
    let mut mc = ID;
    mc[3][..3].copy_from_slice(&CAIXA.0);
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
    // De CIMA: x → x, z → −y do ecrã (a linha 0 da imagem é z = −MEIA, como no oráculo).
    let cam = Camera {
        view_proj: [
            [1.0 / MEIA, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.1, 0.0],
            [0.0, -1.0 / MEIA, 0.0, 0.0],
            [-CENTRO_X / MEIA, 0.0, 0.5, 1.0],
        ],
        olho: [0.0; 3],
        perspectiva: false,
        dir_vista: [0.0, -1.0, 0.0],
    };
    let mut c = cena(&objs, &mats, cam);
    c.tamanho = (LADO, LADO);
    c.chao = Some(0.0);
    c.caixa_tan = Some(0.47);
    c.foto = Some(Foto {
        giro: [1.0, 0.0],
        forca: 1.0,
        caixa: chave,
        fundo: None,
    });
    let img = fw.quadro(&c).expect("quadro");
    // ⚠️ A máscara dos objetos vem de um quadro SEM chão: sob um céu preto o lado da esfera longe do
    // sol é RGB 0 e passaria por chão.
    c.chao = None;
    let sem_chao = fw.quadro(&c).expect("quadro sem chão");
    img.chunks_exact(4)
        .zip(sem_chao.chunks_exact(4))
        .map(|(px, obj)| (obj[3] == 0).then(|| f32::from(px[3]) / 255.0))
        .collect()
}

/// ⭐⭐⭐ **A sombra do sol é a do Cycles**, passo a passo: a linha do meio (a esfera e a caixa) e a
/// coluna que atravessa a sombra da esfera, com o sol a `40°` (raio `1°` e `4°`) e a `15°` (sombra
/// comprida, de `3,7×` a altura). Controlo: com a luz-chave a `0` o chão não escurece.
#[test]
#[ignore = "precisa de aparelho"]
fn a_sombra_do_sol_e_a_do_cycles() {
    let cena_linha = ORACULO
        .lines()
        .find(|l| l.starts_with("# CENA"))
        .expect("o oráculo tem a linha da cena");
    assert!(
        cena_linha.contains("lado=384 centro_x=0.1 meia=1.5 esfera=((-0.8, 0.6, 0.0), 0.3) caixa=((0.6, 0.2, 0.0), 0.4)"),
        "o CSV é de outra cena: {cena_linha}"
    );
    let Some(mut fw) = Forward::no_aparelho(&ambiente()) else {
        eprintln!("sem aparelho — o gate não corre aqui");
        return;
    };
    for (id, (p, n, idx)) in [(1u64, esfera(ESFERA.1)), (2, cubo(CAIXA.1))] {
        let ao = vec![1.0; p.len()];
        let mat = vec![0u32; p.len()];
        fw.sobe(
            id,
            &Malha {
                posicoes: &p,
                normais: &n,
                ao: &ao,
                material: &mat,
                indices: &idx,
            },
        );
    }
    let mut casos: Vec<(f32, f32)> = Vec::new();
    for l in ORACULO.lines().filter(|l| !l.starts_with('#')).skip(1) {
        let c: Vec<&str> = l.split(',').collect();
        let k = (c[0].parse().expect("altura"), c[1].parse().expect("raio"));
        if !casos.contains(&k) {
            casos.push(k);
        }
    }
    for (altura, raio) in casos {
        let nosso = o_nosso(&mut fw, altura, raio, 1.0);
        let (mut pior, mut soma, mut n, mut fora) = (0.0f32, 0.0f32, 0usize, 0usize);
        for l in ORACULO.lines().filter(|l| !l.starts_with('#')).skip(1) {
            let c: Vec<&str> = l.split(',').collect();
            if (c[0].parse::<f32>().ok(), c[1].parse::<f32>().ok()) != (Some(altura), Some(raio)) {
                continue;
            }
            let (i, j): (u32, u32) = (c[3].parse().expect("i"), c[4].parse().expect("j"));
            let ciclos: f32 = c[7].parse().expect("escuro");
            let Some(e) = nosso[(j * LADO + i) as usize] else {
                continue;
            };
            let d = (e - ciclos).abs();
            if let Ok(f) = std::env::var("PH2D_SOL_PERFIL")
                && f == format!("{altura}/{raio}/{}", c[2])
            {
                eprintln!("P {} {} {e:.3} {ciclos:.3}", if c[2] == "linha" { i } else { j }, if c[2] == "linha" { c[5] } else { c[6] });
            }
            pior = pior.max(d);
            soma += d;
            n += 1;
            if d > 0.1 {
                fora += 1;
                if std::env::var("PH2D_SOL_DIAG").is_ok() {
                    eprintln!(
                        "  {} i {i} j {j} (x {} z {}): nosso {e:.3} cycles {ciclos:.3}",
                        c[2], c[5], c[6]
                    );
                }
            }
        }
        let medio = soma / n as f32;
        eprintln!(
            "sol a {altura}° raio {raio}°: {n} px do chão · |Δ| médio {medio:.4} · máx {pior:.3} · {fora} px com |Δ| > 0,1"
        );
        // ⚠️ Tectos MEDIDOS (03/10, PCSS com a espiral de Vogel e os três níveis): médio `0,0010` /
        // `0,0113` / `0,0027` e máximo `0,082` / `0,143` / `0,099` — com folga de `~2×` no médio. O
        // padrão de Poisson de antes dava `0,0031` / `0,0215` / `0,0065` e `9–51` pixels acima de `0,1`.
        let (t_medio, t_max) = if raio > 2.0 { (0.02, 0.25) } else { (0.006, 0.15) };
        assert!(n > 600, "o chão comparado encolheu: {n} px");
        assert!(medio < t_medio && pior < t_max, "sol a {altura}° raio {raio}°: médio {medio} máx {pior}");
        assert!(fora <= 4, "sol a {altura}° raio {raio}°: {fora} px com |Δ| > 0,1");
    }
    // O controlo: com a luz-chave a 0 a SOMBRA DO SOL some — nos pixels onde o Cycles a dá cheia
    // (`≥ 0,95`) o chão fica claro (sobra só o escurecimento do céu junto dos objetos).
    let sem = o_nosso(&mut fw, 40.0, 1.0, 0.0);
    let (mut na_sombra, mut pior_sem) = (0usize, 0.0f32);
    for l in ORACULO.lines().filter(|l| l.starts_with("40.0,1.0,")) {
        let c: Vec<&str> = l.split(',').collect();
        let (i, j): (u32, u32) = (c[3].parse().expect("i"), c[4].parse().expect("j"));
        let ciclos: f32 = c[7].parse().expect("escuro");
        if let Some(e) = sem[(j * LADO + i) as usize]
            && ciclos >= 0.95
        {
            na_sombra += 1;
            pior_sem = pior_sem.max(e);
        }
    }
    eprintln!("controlo sem a luz-chave: {na_sombra} px de sombra cheia no Cycles · o nosso escurece no máx {pior_sem:.3}");
    assert!(na_sombra > 100 && pior_sem < 0.5, "sem a luz-chave a sombra do sol ficou: {pior_sem}");
}
