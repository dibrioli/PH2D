//! ⭐⭐ **O ENQUADRAMENTO DAS SOMBRAS** — dois mapas, cada um pela sua pergunta:
//!
//! | mapa | olha | quem o lê |
//! |---|---|---|
//! | sombra (profundidade, [`crate::SOMBRA_LADO`]) | ao longo da LUZ-CHAVE (`−L`) | os objetos e, sob o céu fotográfico, o chão (PCSS) |
//! | cobertura (vista de cima) | a direito para baixo | o céu que o chão vê, e no estúdio a sombra MOLE da caixa (`25°`) |
//!
//! ⭐ **No estúdio a luz-chave é `+y` e os dois são o MESMO mapa** — as contas reduzem-se às de antes
//! (há gate: as fotos do estúdio ao byte). Sob o céu fotográfico a chave é o SOL dele.

use crate::Cena;
use crate::gpu_alvo::PROFUNDIDADE;

/// ⭐⭐ **Os NÍVEIS do mapa de sombra** — o MESMO enquadramento redesenhado a `2048`, `512` e `128`:
/// cada pixel lê o nível em que a penumbra dele cabe nos `48` texels do PCSS (`16` amostras: além
/// disso a penumbra mostra degraus). É a pirâmide da cobertura do chão, para a profundidade (que
/// não se pode reduzir por média: redesenha-se).
///
/// ⛔ Medido (03/10, `tests_sol::a_sombra_do_sol_e_a_do_cycles`): com UM nível e o tecto de `48`
/// texels a penumbra saía cortada — sol a `15°` de raio `1°` pede `~115` texels (o mapa enquadra-se
/// justo e o texel é pequeno), a caixa do estúdio `~230`; o lado de fora ficava a `0` onde o Cycles
/// dá `0,13–0,22`.
pub(super) const NIVEIS: usize = 3;

/// Os mapas dos [`NIVEIS`], do mais fino ao mais grosso (`÷4` por nível).
pub(super) fn mapas(device: &wgpu::Device) -> [wgpu::TextureView; NIVEIS] {
    std::array::from_fn(|k| {
        let lado = crate::SOMBRA_LADO >> (2 * k);
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("ph2d-mesh-forward mapa de sombra"),
                size: wgpu::Extent3d {
                    width: lado,
                    height: lado,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: PROFUNDIDADE,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default())
    })
}

/// ⭐ **A luz que faz sombra** — a direcção PARA ela (no mundo, unitária) e a tangente do raio
/// angular (a penumbra).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Chave {
    pub l: [f32; 3],
    pub tan: f32,
}

/// ⚠️ **Abaixo de `1°` de altura o sol não projecta no chão** — a sombra iria para o infinito (a
/// distância ao longo do chão é `h / sen(altura)`) e a luz que ele dá ao chão é `∝ sen(altura)`, ~0.
/// O pôr do sol embarcado está a `3,3°` (sombra de `17×` a altura) e projecta.
const SEN_ALTURA_MIN: f32 = 0.017_452_406;

/// ⭐ **A chave do quadro**: sob o céu fotográfico, o SOL dele (girado com o céu); senão a caixa de
/// quem chama, de cima. `cena.caixa_tan == None` desliga todas as sombras.
pub(super) fn chave(
    cena: &Cena<'_>,
    tem_ceu: bool,
    sol: Option<&super::texturas::SolGpu>,
) -> Option<Chave> {
    let tan_caixa = cena.caixa_tan?;
    match cena.foto.filter(|_| tem_ceu) {
        Some(f) => {
            let s = sol?;
            if !(f.caixa > 0.0) {
                return None;
            }
            // O inverso do `ph2d_sky::gira` (o céu → o mundo).
            let ([c, sn], d) = (f.giro, s.dir);
            Some(Chave {
                l: [c * d[0] + sn * d[2], d[1], -sn * d[0] + c * d[2]],
                tan: s.raio.tan(),
            })
        }
        None => Some(Chave {
            l: [0.0, 1.0, 0.0],
            tan: tan_caixa,
        }),
    }
}

/// O que o quadro precisa dos dois mapas.
pub(super) struct Enquadra {
    /// Há chave: o passe do mapa de sombra corre.
    pub ha_sombra: bool,
    /// Há sombras (`caixa_tan`) e objetos: o chão e a cobertura correm, com ou sem chave.
    pub ha_chao: bool,
    /// Mundo → recorte do mapa de sombra (ao longo da chave).
    pub sombra_vp: [[f32; 4]; 4],
    pub fundo: f32,
    /// A aresta do texel do mapa de sombra, no mundo.
    pub texel: f32,
    pub tan: f32,
    /// Mundo → recorte da cobertura (vista de cima).
    pub ceu_vp: [[f32; 4]; 4],
    /// `[meia-aresta, profundidade em mundo, tangente da caixa de cima, 1 = o chão lê a chave na
    /// cobertura (estúdio) · 0 = no mapa de sombra (sol)]`.
    pub ceu: [f32; 4],
    /// O quadrado do CHÃO em `xz`: centro `(x, z)` e meias-arestas `(x, z)` — a cobertura e, sob o
    /// sol, as sombras compridas que ele deita.
    pub chao_xz: [f32; 4],
}

fn aplica(m: &[[f32; 4]; 4], p: [f32; 3]) -> [f32; 3] {
    [0, 1, 2].map(|i| m[0][i] * p[0] + m[1][i] * p[1] + m[2][i] * p[2] + m[3][i])
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cruz(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn unit(v: [f32; 3]) -> [f32; 3] {
    let l = dot(v, v).sqrt().max(1.0e-30);
    v.map(|x| x / l)
}

/// ⭐ **Um mapa ortográfico ao longo de `l`** sobre os `pontos` (os cantos dos objetos) e o `chao`:
/// `x = (p·direita − ca)/meia`, `y = (p·cima − cb)/meia`, `z = (topo − p·l)/fundo`. A margem é a
/// penumbra máxima (`tan ×` a distância ao longo de `l` do ponto mais fundo ao topo).
/// ⚠️ Com `l = +y`: `direita = x`, `cima = z` — exactamente o mapa «a olhar para baixo» de antes.
fn mapa(
    pontos: &[[f32; 3]],
    l: [f32; 3],
    tan: f32,
    fundo_min: f32,
) -> ([[f32; 4]; 4], [f32; 2], f32, f32) {
    let r = {
        let c = cruz(l, [0.0, 0.0, 1.0]);
        if dot(c, c) > 1.0e-12 {
            unit(c)
        } else {
            [1.0, 0.0, 0.0]
        }
    };
    let u = cruz(r, l);
    let (mut lo, mut hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
    for p in pontos {
        let q = [dot(*p, r), dot(*p, u), dot(*p, l)];
        for i in 0..3 {
            lo[i] = lo[i].min(q[i]);
            hi[i] = hi[i].max(q[i]);
        }
    }
    let altura = (hi[2] - lo[2]).max(1.0e-3);
    let topo = hi[2] + 0.01 * altura;
    let base = fundo_min.min(lo[2]) - 0.01 * altura;
    let fundo = (topo - base).max(1.0e-4);
    let margem = tan * (topo - fundo_min).max(0.0);
    let (ca, cb) = ((lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5);
    let meia = ((hi[0] - lo[0]).max(hi[1] - lo[1]) * 0.5 + margem).max(1.0e-3) * 1.02;
    let m = [
        [r[0] / meia, u[0] / meia, -l[0] / fundo + 0.0, 0.0],
        [r[1] / meia, u[1] / meia, -l[1] / fundo + 0.0, 0.0],
        [r[2] / meia, u[2] / meia, -l[2] / fundo + 0.0, 0.0],
        [-ca / meia, -cb / meia, topo / fundo, 1.0],
    ];
    (m, [ca, cb], meia, fundo)
}

/// ⭐⭐ **Os dois mapas do quadro.**
pub(super) fn enquadra(
    cena: &Cena<'_>,
    chave: Option<Chave>,
    caixa_de: impl Fn(u64) -> Option<([f32; 3], [f32; 3])>,
) -> Enquadra {
    let mut cantos = Vec::new();
    for o in cena.objetos {
        let Some((a, b)) = caixa_de(o.malha) else {
            continue;
        };
        for k in 0..8 {
            let c = [
                if k & 1 == 0 { a[0] } else { b[0] },
                if k & 2 == 0 { a[1] } else { b[1] },
                if k & 4 == 0 { a[2] } else { b[2] },
            ];
            cantos.push(aplica(&o.modelo, c));
        }
    }
    let tem = !cantos.is_empty();
    if !tem {
        cantos = vec![[-1.0; 3], [1.0; 3]];
    }
    let lo_y = cantos.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
    let chao = cena.chao.unwrap_or(lo_y);
    // A cobertura: de cima, com a margem da caixa de quem chama — o mapa de sempre.
    let tan_ceu = cena.caixa_tan.unwrap_or(0.0);
    let (ceu_vp, [cx, cz], meia_ceu, fundo_ceu) = mapa(&cantos, [0.0, 1.0, 0.0], tan_ceu, chao);
    let vertical = chave.is_none_or(|k| k.l == [0.0, 1.0, 0.0]);
    let Some(k) = chave.filter(|_| !vertical) else {
        return Enquadra {
            ha_sombra: tem && chave.is_some(),
            ha_chao: tem && cena.caixa_tan.is_some(),
            sombra_vp: ceu_vp,
            fundo: fundo_ceu,
            texel: 2.0 * meia_ceu / crate::SOMBRA_LADO as f32,
            tan: tan_ceu,
            ceu_vp,
            ceu: [meia_ceu, fundo_ceu, tan_ceu, 1.0],
            chao_xz: [cx, cz, meia_ceu, meia_ceu],
        };
    };
    // ⭐ O mapa do SOL: os objetos e as sombras que eles deitam no chão (o ponto `p` cai em
    // `p − t·l`, `t = (p.y − chão)/l.y`; ao longo de `l` fica `t` mais fundo).
    let projecta = k.l[1] > SEN_ALTURA_MIN && cena.chao.is_some();
    let no_chao: Vec<[f32; 3]> = if projecta {
        cantos
            .iter()
            .map(|p| {
                let t = (p[1] - chao).max(0.0) / k.l[1];
                [p[0] - t * k.l[0], chao, p[2] - t * k.l[2]]
            })
            .collect()
    } else {
        Vec::new()
    };
    let topo = cantos
        .iter()
        .map(|p| dot(*p, k.l))
        .fold(f32::NEG_INFINITY, f32::max);
    let mais_fundo = no_chao
        .iter()
        .chain(&cantos)
        .map(|p| dot(*p, k.l))
        .fold(f32::INFINITY, f32::min);
    // A penumbra no chão passa da sombra dura (`tan ×` a distância, esticada por `1/sen(altura)`): o
    // fundo do mapa desce o que ela anda para longe do sol. ⛔ Medido (03/10, oráculo, sol de `4°`):
    // sem isto a ponta da penumbra além da caixa saía `0` onde o Cycles dá `0,22`.
    let pen = k.tan * (topo - mais_fundo).max(0.0) / k.l[1].max(SEN_ALTURA_MIN);
    let horizontal = (k.l[0] * k.l[0] + k.l[2] * k.l[2]).sqrt();
    let fundo_min = if projecta {
        mais_fundo - pen * horizontal
    } else {
        mais_fundo
    };
    let (sombra_vp, _, meia, fundo) = mapa(&cantos, k.l, k.tan, fundo_min);
    // O chão: a cobertura e as sombras do sol (com a penumbra delas).
    let (mut lx, mut hx) = (cx - meia_ceu, cx + meia_ceu);
    let (mut lz, mut hz) = (cz - meia_ceu, cz + meia_ceu);
    for p in &no_chao {
        lx = lx.min(p[0] - pen);
        hx = hx.max(p[0] + pen);
        lz = lz.min(p[2] - pen);
        hz = hz.max(p[2] + pen);
    }
    Enquadra {
        ha_sombra: tem,
        ha_chao: tem,
        sombra_vp,
        fundo,
        texel: 2.0 * meia / crate::SOMBRA_LADO as f32,
        tan: k.tan,
        ceu_vp,
        ceu: [meia_ceu, fundo_ceu, tan_ceu, 0.0],
        chao_xz: [
            (lx + hx) * 0.5,
            (lz + hz) * 0.5,
            (hx - lx) * 0.5,
            (hz - lz) * 0.5,
        ],
    }
}
