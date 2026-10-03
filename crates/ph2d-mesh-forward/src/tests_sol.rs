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
/// Os enquadramentos do oráculo: `(nome, centro x, centro z, meia-aresta)`.
const QUADROS: [(&str, f32, f32, f32); 2] = [("cena", 0.1, 0.0, 1.5), ("perto", 0.85, -0.2, 0.05)];
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
            // CCW visto de FORA, como as malhas do campo.
            if s > 0.0 {
                idx.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            } else {
                idx.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
            }
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
fn o_nosso(
    fw: &mut Forward,
    (cx, cz, meia): (f32, f32, f32),
    (altura, raio): (f32, f32),
    chave: f32,
) -> Vec<Option<f32>> {
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
    // De CIMA: x → x, z → −y do ecrã (a linha 0 da imagem é z = cz − meia, como no oráculo).
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

/// Uma linha do oráculo.
struct Ponto {
    quadro: &'static str,
    caso: (f32, f32),
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
                quadro: c[0],
                caso: (c[1].parse().expect("altura"), c[2].parse().expect("raio")),
                corte: c[3],
                i: c[4].parse().expect("i"),
                j: c[5].parse().expect("j"),
                escuro: c[8].parse().expect("escuro"),
            }
        })
        .collect()
}

/// ⭐⭐⭐ **A sombra do sol é a do Cycles**, passo a passo. Na cena inteira: a linha do meio (a esfera e
/// a caixa) e a coluna que atravessa a sombra da esfera, com o sol a `40°` (raio `1°` e `4°`) e a
/// `15°` (sombra comprida, de `3,7×` a altura). DE PERTO (`0,26 mm` por pixel): três colunas que
/// cruzam a borda da sombra da caixa junto do canto que toca o chão — dura no Cycles, e o mapa de
/// sombra da cena inteira teria aí `~4` pixels por texel. Controlo: com a luz-chave a `0` a sombra do
/// sol some.
///
/// ⚠️ **Limite MEDIDO** (03/10): de perto, a cauda de fora (`≤ 0,47` no Cycles, `~38` px) vem de uma
/// face exactamente de PERFIL para o sol — nenhum mapa de sombra a guarda. Curá-la pede outra classe
/// de algoritmo (retroprojeção, ou dois mapas, frente e trás, a dobrar o preço); o contacto DURO e o
/// miolo sem fuga, que são o que uma cena real mostra, batem com o Cycles.
#[test]
#[ignore = "precisa de aparelho"]
fn a_sombra_do_sol_e_a_do_cycles() {
    let cena_linha = ORACULO
        .lines()
        .find(|l| l.starts_with("# CENA"))
        .expect("o oráculo tem a linha da cena");
    assert!(
        cena_linha.contains(
            "lado=384 quadros=[('cena', 0.1, 0.0, 1.5), ('perto', 0.85, -0.2, 0.05)] \
             esfera=((-0.8, 0.6, 0.0), 0.3) caixa=((0.6, 0.2, 0.0), 0.4)"
        ),
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
    let pontos = oraculo();
    let mut casos: Vec<(&str, (f32, f32))> = Vec::new();
    for p in &pontos {
        if !casos.contains(&(p.quadro, p.caso)) {
            casos.push((p.quadro, p.caso));
        }
    }
    let quadro_de = |nome: &str| {
        let q = QUADROS
            .iter()
            .find(|q| q.0 == nome)
            .expect("quadro conhecido");
        (q.1, q.2, q.3)
    };
    for (quadro, caso) in casos {
        let nosso = o_nosso(&mut fw, quadro_de(quadro), caso, 1.0);
        let (mut pior, mut soma, mut n, mut fora) = (0.0f32, 0.0f32, 0usize, 0usize);
        let (mut cauda, mut n_cauda) = (0.0f32, 0usize);
        let mut miolo_pior = 0.0f32;
        for p in pontos
            .iter()
            .filter(|p| p.quadro == quadro && p.caso == caso)
        {
            let Some(e) = nosso[(p.j * LADO + p.i) as usize] else {
                continue;
            };
            let d = (e - p.escuro).abs();
            // ⚠️ DE PERTO, a cauda de FORA da sombra da caixa (o Cycles abaixo de `0,5`) vem da face
            // lateral de PERFIL para o sol (o sol do oráculo está no plano `xy`): nenhum mapa de sombra
            // a vê. Mede-se à parte, com o seu tecto (o doc do gate).
            if quadro == "perto" && p.escuro < 0.5 {
                cauda += d;
                n_cauda += 1;
                continue;
            }
            // ⚠️ Num sol ENORME (`10°`) o miolo não é escuro de todo: a separação prende cada texel do
            // disco ao limiar (`16 ×` a média, e a média inclui o disco), e esse resto fica no céu, sem
            // sombra — `16·Ω/4π` da energia, `12 %` a `10°` e `0,02 %` num sol de verdade (`0,4°`). O
            // miolo mede-se contra essa conta; a penumbra contra o Cycles.
            if caso.1 > 6.0 && p.escuro >= 0.95 {
                let omega = std::f32::consts::TAU * (1.0 - caso.1.to_radians().cos());
                let resto = 16.0 * omega / (2.0 * std::f32::consts::TAU);
                miolo_pior = miolo_pior.max((e - (1.0 - resto)).abs());
                continue;
            }
            if std::env::var("PH2D_SOL_PERFIL")
                .is_ok_and(|f| f == format!("{quadro}/{}/{}/{}", caso.0, caso.1, p.corte))
            {
                eprintln!("P {} {} {e:.3} {:.3}", p.i, p.j, p.escuro);
            }
            pior = pior.max(d);
            soma += d;
            n += 1;
            if d > 0.1 {
                fora += 1;
            }
        }
        let medio = soma / n as f32;
        let (altura, raio) = caso;
        eprintln!(
            "{quadro}: sol a {altura}° raio {raio}°: {n} px do chão · |Δ| médio {medio:.4} · máx {pior:.3} · {fora} px com |Δ| > 0,1 · miolo contra o resto {miolo_pior:.3}"
        );
        // ⚠️ Tectos MEDIDOS (03/10: PCSS com a espiral de Vogel, três níveis, faces de TRÁS no mapa e
        // o raio do pixel na busca). Médio / máximo:
        //
        // | caso | Poisson, 1 nível, faces da frente | agora |
        // |---|---|---|
        // | cena 40°/1° | `0,0031` / `0,20` | `0,0006` / `0,041` |
        // | cena 40°/4° | `0,0215` / `0,26` | `0,0095` / `0,083` |
        // | cena 15°/1° | `0,0065` / `0,24` | `0,0019` / `0,143` |
        // | cena 40°/10° (penumbra; o miolo contra o resto) | — | `0,0154` / `0,098` (`0,047`) |
        // | perto 40°/1° (borda e miolo) | `0,022` / `0,44` (vazava `0,81` no miolo) | `0,0005` / `0,061` |
        let (t_medio, t_max) = if raio > 2.0 {
            (0.03, 0.15)
        } else {
            (0.004, 0.2)
        };
        assert!(
            miolo_pior < 0.08,
            "{quadro}: o miolo do sol de {raio}° fugiu do resto: {miolo_pior}"
        );
        if n_cauda > 0 {
            let c = cauda / n_cauda as f32;
            eprintln!("{quadro}: a cauda da face de perfil: {n_cauda} px · |Δ| médio {c:.3}");
            // Medido (03/10): `0,043` (o nosso dá 0 onde o Cycles tem a cauda). Pior é regressão.
            assert!(c < 0.08, "{quadro}: a cauda piorou: {c}");
        }
        assert!(n > 450, "{quadro}: o chão comparado encolheu: {n} px");
        assert!(
            medio < t_medio && pior < t_max,
            "{quadro}: sol a {altura}° raio {raio}°: médio {medio} máx {pior}"
        );
        assert!(
            fora <= 4,
            "{quadro}: sol a {altura}° raio {raio}°: {fora} px com |Δ| > 0,1"
        );
    }
    // O controlo: com a luz-chave a 0 a SOMBRA DO SOL some — nos pixels onde o Cycles a dá cheia
    // (`≥ 0,95`) o chão fica claro (sobra só o escurecimento do céu junto dos objetos).
    let sem = o_nosso(&mut fw, quadro_de("cena"), (40.0, 1.0), 0.0);
    let (mut na_sombra, mut pior_sem) = (0usize, 0.0f32);
    for p in pontos
        .iter()
        .filter(|p| p.quadro == "cena" && p.caso == (40.0, 1.0))
    {
        if let Some(e) = sem[(p.j * LADO + p.i) as usize]
            && p.escuro >= 0.95
        {
            na_sombra += 1;
            pior_sem = pior_sem.max(e);
        }
    }
    eprintln!(
        "controlo sem a luz-chave: {na_sombra} px de sombra cheia no Cycles · o nosso escurece no máx {pior_sem:.3}"
    );
    assert!(
        na_sombra > 100 && pior_sem < 0.5,
        "sem a luz-chave a sombra do sol ficou: {pior_sem}"
    );
}

/// Os ajudantes são CCW vistos de fora, como as malhas do campo (o mapa de sombra descarta por isso).
#[test]
fn os_ajudantes_sao_ccw_para_fora() {
    for (nome, (p, _, idx)) in [("esfera", esfera(0.5)), ("cubo", cubo(0.4))] {
        let mut maus = 0;
        for t in idx.chunks_exact(3) {
            let (a, b, c) = (p[t[0] as usize], p[t[1] as usize], p[t[2] as usize]);
            let (u, v) = (
                [0, 1, 2].map(|i| b[i] - a[i]),
                [0, 1, 2].map(|i| c[i] - a[i]),
            );
            let n = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            let m = [0, 1, 2].map(|i| (a[i] + b[i] + c[i]) / 3.0);
            let area = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if area > 1.0e-9 && n[0] * m[0] + n[1] * m[1] + n[2] * m[2] <= 0.0 {
                maus += 1;
            }
        }
        assert_eq!(maus, 0, "{nome}: {maus} triângulos para dentro");
    }
}
