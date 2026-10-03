//! ⭐⭐⭐ **O CHÃO QUE TAPA AS PEÇAS** ([`crate::chao_tapa`]) — contra o Cycles na placa, e a lei de
//! [`RAIOS_CHAO`] direcções contra o integral denso na CPU.
//!
//! O oráculo (`docs/3DModeling/ferramentas/oraculo_chao_tapa_blender.py` →
//! `fixtures/oraculo_chao_tapa.csv`): uma caixa e uma esfera pousadas, uma esfera a `6 cm` do chão,
//! céu uniforme, chão branco infinito; câmara BAIXA para ver a barriga das peças. A coluna `com` é o
//! céu que chega com o chão (o que o chão devolve incluído), `sem` sem ele.

use crate::chao_tapa::{RAIOS_CHAO, escurecimento, escurecimento_com, espiral, reflexo};
use crate::tests::{ID, cena};
use crate::tests_contacto::{
    LADO, Peca, Ponto, bordas, camera_do_blender, desenhista_de, linear, norm,
};
use crate::{Forward, Instancia};

const ORACULO: &str = include_str!("../fixtures/oraculo_chao_tapa.csv");
const DE: [f32; 3] = [0.6, 0.15, 1.0];
const ALVO: [f32; 3] = [0.0, 0.2, 0.15];
const MEIA: f32 = 0.8;
/// As peças do oráculo, pela ordem do índice dele (`1` = a caixa).
pub(crate) const PECAS: [Peca; 3] = [
    ([0.4, 0.2, -0.05], 0.2, true),
    ([-0.45, 0.3, 0.0], 0.3, false),
    ([0.05, 0.21, 0.5], 0.15, false),
];

/// O céu que o chão vê à volta de uma esfera de raio `r` pousada na origem — a forma fechada que o
/// Cycles dá a `1 %`: o escurecimento é `(r/D)³`, `D` a distância ao centro.
fn chao_da_esfera(r: f32) -> impl Fn(f32, f32) -> f32 {
    move |x, z| 1.0 - (r / (x * x + z * z + r * r).sqrt()).powi(3)
}

/// O mesmo integral, denso (`192 × 768` células em `(cos θ, φ)` à volta de `-y`).
fn denso(p: [f32; 3], n: [f32; 3], v: &impl Fn(f32, f32) -> f32) -> f32 {
    let (nt, nf) = (192usize, 768usize);
    let mut soma = 0.0f64;
    for a in 0..nt {
        let y = -((a as f64 + 0.5) / nt as f64);
        let r = (1.0 - y * y).sqrt();
        for b in 0..nf {
            let f = (b as f64 + 0.5) / nf as f64 * std::f64::consts::TAU;
            let d = [r * f.cos(), y, r * f.sin()];
            let w = n[0] as f64 * d[0] + n[1] as f64 * d[1] + n[2] as f64 * d[2];
            if w <= 0.0 {
                continue;
            }
            let t = f64::from(p[1]) / -y;
            let x = (f64::from(p[0]) + d[0] * t) as f32;
            let z = (f64::from(p[2]) + d[2] * t) as f32;
            // dω = d(cos θ) dφ, e a célula tem 1/nt × 2π/nf.
            soma += w * (1.0 - f64::from(v(x, z)));
        }
    }
    (soma * std::f64::consts::TAU / (nt * nf) as f64 / std::f64::consts::PI) as f32
}

/// Os pontos da metade de baixo da esfera pousada de raio `r`, e a normal de cada um.
fn barriga(r: f32) -> Vec<([f32; 3], [f32; 3])> {
    let mut v = Vec::new();
    for a in 1..12 {
        let t = std::f32::consts::FRAC_PI_2 + a as f32 / 12.0 * std::f32::consts::FRAC_PI_2 * 0.98;
        for b in 0..5 {
            let f = b as f32 * 1.3;
            let n = [t.sin() * f.cos(), t.cos(), t.sin() * f.sin()];
            v.push(([n[0] * r, r + n[1] * r, n[2] * r], n));
        }
    }
    v
}

/// ⭐⭐ **A lei das [`RAIOS_CHAO`] direcções é o integral** — na barriga de uma esfera pousada, sobre
/// o chão dela. Controlos: o chão todo aceso não apaga nada e o chão todo apagado apaga a forma
/// fechada `(1 − n.y)/2`.
#[test]
fn a_lei_do_chao_que_tapa_e_o_integral() {
    let r = 0.3;
    let v = chao_da_esfera(r);
    let pts = barriga(r);
    let mede = |k: usize| {
        let dirs: Vec<[f32; 3]> = (0..k).map(|i| espiral(i, k)).collect();
        let (mut s, mut pior) = (0.0f32, 0.0f32);
        for &(p, n) in &pts {
            let e = (escurecimento_com(&dirs, p, n, 0.0, &v) - denso(p, n, &v)).abs();
            s += e;
            pior = pior.max(e);
        }
        (s / pts.len() as f32, pior)
    };
    for k in [4, 8, 16, 32] {
        let (m, p) = mede(k);
        eprintln!("{k:>2} direcções: |Δ| médio {m:.4} · máx {p:.4}");
    }
    for &(p, n) in &pts {
        assert_eq!(escurecimento(p, n, 0.0, |_, _| 1.0), 0.0);
        let todo = escurecimento(p, n, 0.0, |_, _| 0.0);
        assert!(
            (todo - (1.0 - n[1]) * 0.5).abs() < 1.0e-5,
            "{todo} em n = {n:?}"
        );
    }
    // Medido (03/10): `4 / 8 / 16 / 32` direcções `0,0180 / 0,0101 / 0,0047 / 0,0022` de média.
    let (m, p) = mede(RAIOS_CHAO);
    assert!(
        m < 0.01 && p < 0.03,
        "a lei afastou-se do integral: médio {m}, máx {p}"
    );
    // E o integral denso confere com a forma fechada num caso que a tem: o ponto mais baixo de uma
    // esfera vê, a direito para baixo, o seu próprio ponto de contacto.
    let fundo = denso([0.0, 1.0e-3, 0.0], [0.0, -1.0, 0.0], &v);
    assert!(
        fundo > 0.95,
        "o contacto debaixo da esfera é escuro: {fundo}"
    );
}

/// O resto de uma linha do oráculo: a difusa com o chão e o metal (`espelho_sem`, `espelho_com`,
/// `aspero_sem`, `aspero_com`, `verniz_sem`, `verniz_com`, e os `*_solo` — cada peça a ver só o céu).
pub(crate) struct Linha {
    pub(crate) com: f32,
    pub(crate) brilho: [f32; 9],
}

/// O lobo do pré-filtro com `n = v = r`, denso: `∫ (1 − V) D(h) (r·l) dl / ∫ D(h) (r·l) dl`.
fn lobo_denso(p: [f32; 3], r: [f32; 3], alpha: f32, v: &impl Fn(f32, f32) -> f32) -> f32 {
    let (nt, nf) = (512usize, 1024usize);
    let a2 = f64::from(alpha * alpha);
    let (mut soma, mut pesos) = (0.0f64, 0.0f64);
    for a in 0..nt {
        let y = 1.0 - 2.0 * (a as f64 + 0.5) / nt as f64;
        let rr = (1.0 - y * y).sqrt();
        for b in 0..nf {
            let f = (b as f64 + 0.5) / nf as f64 * std::f64::consts::TAU;
            let l = [rr * f.cos(), y, rr * f.sin()];
            let c = f64::from(r[0]) * l[0] + f64::from(r[1]) * l[1] + f64::from(r[2]) * l[2];
            if c <= 0.0 {
                continue;
            }
            // cos θh = √((1 + r·l)/2); D do GGX.
            let nh2 = (1.0 + c) * 0.5;
            let w = a2 / (nh2 * (a2 - 1.0) + 1.0).powi(2) * c;
            pesos += w;
            if y < 0.0 {
                let t = f64::from(p[1]) / -y;
                let x = (f64::from(p[0]) + l[0] * t) as f32;
                let z = (f64::from(p[2]) + l[2] * t) as f32;
                soma += w * (1.0 - f64::from(v(x, z)));
            }
        }
    }
    (soma / pesos) as f32
}

/// ⭐⭐ **O reflexo de dois anéis é o lobo** — na barriga da esfera pousada, para reflexos que descem,
/// de lado e rasantes, contra o lobo denso. Controlos: `α = 0` é o reflexo nítido (o chão no ponto
/// onde a reflectida o acerta), o chão aceso não escurece nada.
#[test]
fn o_reflexo_e_o_lobo() {
    let r0 = 0.3;
    let v = chao_da_esfera(r0);
    let refl = [
        [0.0, -1.0, 0.0],
        [0.6, -0.8, 0.0],
        [0.0, -0.5, 0.866],
        [-0.95, -0.312, 0.0],
    ];
    let (mut n, mut s, mut pior) = (0usize, 0.0f32, 0.0f32);
    for &(p, _) in &barriga(r0) {
        for r in refl {
            let t = p[1] / -r[1];
            let nitido = 1.0 - v(p[0] + r[0] * t, p[2] + r[2] * t);
            assert!((reflexo(p, r, 0.0, 0.0, &v) - nitido).abs() < 1.0e-5);
            assert_eq!(reflexo(p, r, 0.25, 0.0, |_, _| 1.0), 0.0);
            for alpha in [0.0625f32, 0.25, 0.5] {
                let e = (reflexo(p, r, alpha, 0.0, &v) - lobo_denso(p, r, alpha, &v)).abs();
                s += e;
                pior = pior.max(e);
                n += 1;
            }
        }
    }
    eprintln!("{n} lobos · |Δ| médio {:.4} · máx {pior:.4}", s / n as f32);
    // Medido (03/10, `4` por anel): médio `0,0159`, máx `0,111` (o rasante a `r.y = −0,31` junto ao
    // contacto, α `0,5`); com `8` por anel `0,0120` / `0,093`. Três anéis (`1/6, 1/2, 5/6`) chegavam
    // mais perto do lobo e NADA contra o Cycles na placa (`0,0236 → 0,0235`) — recusado.
    assert!(
        s / (n as f32) < 0.02 && pior < 0.14,
        "o reflexo afastou-se do lobo"
    );
}

/// ⭐ **O reflexo de `p` sai da zona da própria peça?** — a reflectida da câmara (a olhar ao longo de
/// `−de`) bate no chão a mais de `3` raios da sua peça (`pecas[p.obj]`). Ali a verdade inclui as
/// VIZINHAS e as sombras delas; o reflexo daqui não desenha as vizinhas, e a sombra delas sem elas é o
/// que o dono recusou (03/10) — logo ali ele não escurece.
pub(crate) fn reflete_fora(p: &Ponto, de: [f32; 3], pecas: &[Peca]) -> bool {
    let f = norm(de.map(|c| -c));
    let fn_ = f[0] * p.n[0] + f[1] * p.n[1] + f[2] * p.n[2];
    let r: [f32; 3] = std::array::from_fn(|e| f[e] - 2.0 * fn_ * p.n[e]);
    if r[1] >= -1.0e-3 {
        return false;
    }
    let t = p.p[1] / -r[1];
    let x = [p.p[0] + r[0] * t, p.p[2] + r[2] * t];
    let (c, raio, _) = pecas[p.obj];
    ((x[0] - c[0]).powi(2) + (x[1] - c[2]).powi(2)).sqrt() / raio > 3.0
}

fn oraculo() -> (Vec<Ponto>, Vec<Linha>) {
    oraculo_de(ORACULO)
}

/// As linhas de um oráculo do chão que tapa (o de cima ou o de baixo — as mesmas colunas).
pub(crate) fn oraculo_de(texto: &'static str) -> (Vec<Ponto>, Vec<Linha>) {
    texto
        .lines()
        .filter(|l| !l.starts_with('#'))
        .skip(1)
        .map(|l| {
            let c: Vec<f32> = l.split(',').map(|x| x.parse().expect("número")).collect();
            let p = Ponto {
                i: c[0] as u32,
                j: c[1] as u32,
                obj: c[2] as usize - 1,
                p: [c[3], c[4], c[5]],
                n: norm([c[6], c[7], c[8]]),
                vis: c[9],
            };
            let brilho: [f32; 9] = std::array::from_fn(|k| c[12 + k]);
            (p, Linha { com: c[10], brilho })
        })
        .unzip()
}

/// A difusa SÓ — a do oráculo. ⚠️ Com o realce do cinzento de omissão a barriga lia `−0,05`: de
/// raspão o realce pesa, e o reflexo nítido lê o chão escuro do contacto — física nossa que o Cycles
/// (difusa branca) não tem.
pub(crate) fn difusa() -> [f32; ph2d_material::wgsl::PACKED] {
    empacota(ph2d_material::OpenPbr {
        specular_weight: 0.0,
        ..ph2d_material::OpenPbr::default()
    })
}

/// O metal branco do oráculo, de rugosidade `r`.
pub(crate) fn metal(r: f32) -> [f32; ph2d_material::wgsl::PACKED] {
    empacota(ph2d_material::OpenPbr {
        base_color: [1.0; 3],
        base_metalness: 1.0,
        specular_roughness: r,
        ..ph2d_material::OpenPbr::default()
    })
}

/// O metal áspero (`0,5`) com verniz NÍTIDO de índice `1,5` — o Principled do oráculo. Duas perguntas de
/// reflexo com rugosidades diferentes no mesmo pixel: é o que põe à prova a memória por pixel.
fn envernizado() -> [f32; ph2d_material::wgsl::PACKED] {
    empacota(ph2d_material::OpenPbr {
        base_color: [1.0; 3],
        base_metalness: 1.0,
        specular_roughness: 0.5,
        coat_weight: 1.0,
        coat_roughness: 0.0,
        coat_ior: 1.5,
        ..ph2d_material::OpenPbr::default()
    })
}

fn empacota(m: ph2d_material::OpenPbr) -> [f32; ph2d_material::wgsl::PACKED] {
    let s = m.prepare();
    ph2d_material::wgsl::pack(&s, ph2d_material::wgsl::EnvLobe::of(&s))
}

fn desenha(fw: &mut Forward, chao: bool) -> Vec<u8> {
    desenha_de(fw, chao, difusa())
}

fn desenha_de(fw: &mut Forward, chao: bool, mat: [f32; ph2d_material::wgsl::PACKED]) -> Vec<u8> {
    desenha_em(fw, chao, mat, camera_do_blender(DE, ALVO, MEIA))
}

/// As peças do oráculo com `mat`, da câmara `cam`, com o chão (e a caixa de sombra) ou sem nada.
pub(crate) fn desenha_em(
    fw: &mut Forward,
    chao: bool,
    mat: [f32; ph2d_material::wgsl::PACKED],
    cam: crate::Camera,
) -> Vec<u8> {
    let objs: Vec<Instancia> = PECAS
        .iter()
        .enumerate()
        .map(|(k, (c, _, _))| {
            let mut m = ID;
            m[3][..3].copy_from_slice(c);
            Instancia {
                malha: k as u64 + 1,
                modelo: m,
            }
        })
        .collect();
    let mats = [mat];
    let mut c = cena(&objs, &mats, cam);
    c.tamanho = (LADO, LADO);
    if chao {
        c.chao = Some(0.0);
        c.caixa_tan = Some(0.47);
    }
    fw.quadro(&c).expect("quadro")
}

/// ⭐⭐⭐ **O chão tapa as peças como no Cycles** — pixel a pixel, longe das silhuetas e das arestas:
/// com o chão (e o contacto entre peças) contra a coluna `com`. Controlo: sem o chão o mesmo
/// desenhista é o contacto de antes, e afasta-se do `com` onde o chão escurece.
#[test]
#[ignore = "precisa de aparelho"]
fn o_chao_tapa_as_pecas_como_no_cycles() {
    let (Some(mut grades), Some(mut nua)) =
        (desenhista_de(&PECAS, true), desenhista_de(&PECAS, false))
    else {
        eprintln!("sem aparelho — o gate não corre aqui");
        return;
    };
    let com_chao = desenha(&mut grades, true);
    let sem_chao = desenha(&mut grades, false);
    let base = desenha(&mut nua, false);
    let (pontos, com) = oraculo();
    let borda = bordas(&pontos, &base);
    let (mut n, mut s, mut s_ctl, mut s_sem, mut pior) = (0usize, 0.0f32, 0.0f32, 0.0f32, 0.0f32);
    let (mut nb, mut sb, mut sb_ctl) = (0usize, 0.0f32, 0.0f32);
    // A barriga da esfera pousada (`obj 1`, `n.y < −0,6`): o viés com sinal.
    let (mut nv, mut viés) = (0usize, 0.0f32);
    for (p, c) in pontos.iter().zip(com.iter().map(|l| l.com)) {
        if borda[(p.j * LADO + p.i) as usize] {
            continue;
        }
        let k = ((p.j * LADO + p.i) * 4 + 1) as usize;
        let lb = linear(base[k]);
        if lb < 0.05 {
            continue;
        }
        let (nosso, ctl) = (linear(com_chao[k]) / lb, linear(sem_chao[k]) / lb);
        let e = (nosso - c).abs();
        s += e;
        s_ctl += (ctl - c).abs();
        s_sem += (ctl - p.vis).abs();
        pior = pior.max(e);
        n += 1;
        if p.obj == 1 && p.n[1] < -0.6 {
            viés += nosso - c;
            nv += 1;
        }
        if p.vis - c > 0.05 {
            sb += e;
            sb_ctl += (ctl - c).abs();
            nb += 1;
        }
    }
    let (nf, nbf) = (n as f32, nb as f32);
    eprintln!(
        "{n} px · contra o Cycles |Δ| médio {:.4} · onde o chão escurece ({nb} px) {:.4} · máx {pior:.3} · \
         SEM a lei {:.4} / {:.4} · o contacto contra o `sem` {:.4} · viés na barriga ({nv} px) {:+.4}",
        s / nf,
        sb / nbf,
        s_ctl / nf,
        sb_ctl / nbf,
        s_sem / nf,
        viés / nv as f32
    );
    assert!(
        n > 5000 && nb > 1000 && nv > 200,
        "a fixtura encolheu: {n} / {nb} / {nv} px"
    );
    assert!(
        sb_ctl / nbf > 0.08,
        "CONTROLO: sem a lei o erro tem de ser o escurecimento do chão"
    );
    assert!(
        s_sem / nf < 0.02,
        "o contacto de antes afastou-se do Cycles"
    );
    // Medido (03/10, `11 783` px): médio `0,0098`, onde o chão escurece `0,0130`, máx `0,074` (a pousada
    // virada para a que flutua: o contacto e o chão escuro por trás dela contam DUAS vezes); sem a lei
    // `0,0721 / 0,1189`. Viés na barriga `+0,0075` — com UM intervalo por fatia no céu do chão `−0,052`.
    assert!(
        s / nf < 0.013 && sb / nbf < 0.018 && pior < 0.1,
        "o chão afastou-se do Cycles"
    );
    assert!(
        (viés / nv as f32).abs() < 0.025,
        "a barriga da pousada: o céu do chão enche o vão entre duas peças"
    );
}

/// ⭐⭐ **O reflexo do chão é o do Cycles** — as peças num metal branco: o reflexo nítido lê o chão
/// onde o raio o acerta, o áspero o lobo à volta (`chao_tapa.rs`). A régua que AFIRMA é a razão
/// com/sem chão (a mesma câmara, as grelhas nas duas): isola a lei do chão. ⚠️ Ela conta a direcção
/// que vai à base de uma VIZINHA duas vezes no nosso lado (o contacto suave nas duas, o chão escuro só
/// no `com`) e uma no Cycles (a vizinha tapa nas duas): ler essa base escura — o certo, e o report do
/// dono — custa `+0,005` aqui. A régua ABSOLUTA (tudo sobre nada, contra o `solo` do Cycles) imprime-se
/// ao lado e não afirma: é dominada pelo contacto suave, que não faz reflexos nítidos das vizinhas
/// (áspero `0,060` contra `0,021` na razão). Controlo: sem a lei a razão é `1`, e o erro é o escurecimento.
#[test]
#[ignore = "precisa de aparelho"]
fn o_reflexo_do_chao_e_o_do_cycles() {
    let (Some(mut fw), Some(mut nua)) = (desenhista_de(&PECAS, true), desenhista_de(&PECAS, false))
    else {
        eprintln!("sem aparelho — o gate não corre aqui");
        return;
    };
    let (pontos, linhas) = oraculo();
    let base = desenha(&mut fw, false);
    let borda = bordas(&pontos, &base);
    // Medido (03/10, a razão; o reflexo lê só a zona da peça — `chao_tapa::pegada`): nítido na zona
    // `0,0182` / onde o chão escurece `0,0550` (sem a lei `0,1045`), fora dela `0` de `4 012` px escurecem
    // (o Cycles escurece `1 042`: as sombras das vizinhas, que ele mostra porque desenha as vizinhas);
    // áspero `0,0323` / `0,0652` (sem `0,0883`); verniz `0,0320` / `0,0657` (sem `0,0881`). Antes da zona
    // (o chão todo no reflexo): nítido `0,0030` / `0,0083`, áspero `0,0206` / `0,0304` — e a sombra órfã.
    for (k, nome, mat, barra) in [
        (0usize, "metal nítido", metal(0.0), (0.022f32, 0.065f32)),
        (1, "metal áspero", metal(0.5), (0.038, 0.075)),
        (2, "metal áspero com verniz", envernizado(), (0.038, 0.075)),
    ] {
        let (com, sem, so) = (
            desenha_de(&mut fw, true, mat),
            desenha_de(&mut fw, false, mat),
            desenha_de(&mut nua, false, mat),
        );
        let (mut n, mut s, mut s_ctl, mut nb, mut sb) = (0usize, 0.0f32, 0.0f32, 0usize, 0.0f32);
        let (mut sa, mut sba) = (0.0f32, 0.0f32);
        // Fora da zona da peça: quantos px, onde o Cycles escurece (as sombras alheias), e os nossos que
        // escurecem (a sombra órfã).
        let (mut n_fora, mut alheias, mut orfaos) = (0usize, 0usize, 0usize);
        for (p, l) in pontos.iter().zip(&linhas) {
            let (cs, cc, solo) = (l.brilho[2 * k], l.brilho[2 * k + 1], l.brilho[6 + k]);
            let i = ((p.j * LADO + p.i) * 4 + 1) as usize;
            let (lc, ls, lo) = (linear(com[i]), linear(sem[i]), linear(so[i]));
            if borda[(p.j * LADO + p.i) as usize]
                || ls < 0.05
                || lo < 0.05
                || cs < 0.05
                || solo < 0.05
            {
                continue;
            }
            if k == 0 && reflete_fora(p, DE, &PECAS) {
                n_fora += 1;
                alheias += usize::from(cc / cs < 0.97);
                orfaos += usize::from(lc / ls < 0.97);
                continue;
            }
            let ciclos = cc / cs;
            let e = (lc / ls - ciclos).abs();
            let ea = (lc / lo - cc / solo).abs();
            s += e;
            sa += ea;
            s_ctl += (1.0 - ciclos).abs();
            n += 1;
            if ciclos < 0.9 {
                sb += e;
                sba += ea;
                nb += 1;
            }
        }
        let (nf, nbf) = (n as f32, nb as f32);
        eprintln!(
            "{nome}: {n} px · razão com/sem |Δ| médio {:.4} · onde o chão escurece ({nb} px) {:.4} · SEM a lei \
             {:.4} · (absoluta, contra o `solo`: {:.4} / {:.4}) · fora da zona da peça {n_fora} px: o Cycles \
             escurece {alheias}, nós {orfaos}",
            s / nf,
            sb / nbf,
            s_ctl / nf,
            sa / nf,
            sba / nbf
        );
        assert!(n > 4000 && nb > 300, "a fixtura encolheu: {n} / {nb} px");
        if k == 0 {
            assert!(
                alheias > 100,
                "CONTROLO: o Cycles tem de mostrar sombras alheias no reflexo"
            );
            assert!(
                orfaos * 50 < n_fora,
                "o reflexo mostra a sombra de uma vizinha sem a vizinha: {orfaos} de {n_fora} px"
            );
        }
        assert!(
            s_ctl / nf > 0.04,
            "CONTROLO: sem a lei o erro tem de ser o escurecimento"
        );
        assert!(
            s / nf < barra.0 && sb / nbf < barra.1,
            "o reflexo do chão afastou-se do Cycles"
        );
    }
}
