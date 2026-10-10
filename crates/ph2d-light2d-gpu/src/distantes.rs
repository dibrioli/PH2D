//! ⭐ **O mundo FORA da grade, para a luz DIRETA** (W5b, F4 da fila do dono: «ao dar zoom e pan o
//! aspecto da iluminação muda»). A lâmpada direta (C3) ilumina de qualquer distância, mas a captura só
//! tem o que está DENTRO da grade (a vista e a margem): o raio até uma lâmpada fora dela atravessava o
//! opaco que cruzava fora. Aqui, por lâmpada direta que não cabe na grade, os opacos do caminho até
//! ela (os que tocam o fecho convexo da grade e do disco e não cabem inteiros na grade), cada um na
//! sua MOLDURA:
//! - o polígono da cena: as arestas, exatas (a entrada pela orientação — o interior à esquerda);
//! - o sprite (e o caminho vetorial, que chega como sprite sobre o desenho do Vello): o quad dele e
//!   uma MÁSCARA do alfa na própria moldura — a lei da captura (a cobertura bilinear ≥ ½) na
//!   resolução do objeto, não na da vista. Sprites com a mesma textura e o mesmo UV partilham-na.
//!
//! A C3 testa-os só no trecho do raio FORA da grade (dentro dela manda a captura). As cascatas não
//! mudam: um emissor fora da grade também não existe para elas (a margem é o desenho delas).

use std::collections::BTreeMap;

use bytemuck::{Pod, Zeroable};
use ph2d_light2d_ecs::{CenaDeLuz, Emissor};

use crate::captura::Quem;
use crate::captura_sprite::{CapturaDeSprites, SpriteNaC0};
use crate::config::Grade;

/// [`Distante::tipo`]: uma máscara na atlas.
pub(crate) const MASCARA: u32 = 0;
/// [`Distante::tipo`]: as arestas de um polígono.
pub(crate) const POLIGONO: u32 = 1;

/// O lado da atlas das máscaras, no máximo (16 MB em `R8Unorm`; a 256 por máscara, 256 máscaras
/// distintas antes de o lado de cada uma descer).
pub(crate) const ATLAS_MAX: u32 = 4096;

/// Um objeto distante como o `c3_direta.wgsl` o lê (80 bytes).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub(crate) struct Distante {
    /// O paralelogramo que o envolve, em texels das cascatas: `o.xy, a.xy` (os cantos `o`, `o + a`,
    /// `o + a + b`, `o + b`) …
    pub quad_o_a: [f32; 4],
    /// … `b.xy`; e o canto da janela da máscara na atlas (texels dela).
    pub quad_b_janela: [f32; 4],
    /// Texel das cascatas → moldura `[0, 1]²` da máscara: `s = s.x·px + s.y·py + s.z`; `w` = o lado
    /// da janela.
    pub s: [f32; 4],
    /// Idem `t`; `w` = τ dentro (`1 − opacidade`).
    pub t: [f32; 4],
    pub tipo: u32,
    /// Polígono: as arestas `[primeira, primeira + n)` em [`MundoDistante::arestas`].
    pub primeira: u32,
    pub n: u32,
    pub _p: u32,
}

/// Uma máscara da atlas: de onde vem o alfa.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Mascara {
    /// Índice em [`CapturaDeSprites::texturas`].
    pub textura: usize,
    /// O UV em `(s, t) = (0, 0)` e `(1, 1)`.
    pub uv: [f32; 4],
    /// O canto da janela na atlas, em texels.
    pub janela: [u32; 2],
}

/// O que a C3 precisa do mundo fora da grade neste quadro.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct MundoDistante {
    pub objetos: Vec<Distante>,
    /// `a.xy, b.xy` em texels das cascatas, o interior à ESQUERDA de `a → b`.
    pub arestas: Vec<[f32; 4]>,
    /// Os objetos de cada lâmpada, por índice em [`Self::objetos`].
    pub indices: Vec<u32>,
    /// Por lâmpada direta (a ordem do buffer das lâmpadas): `[primeiro, quantos]` em `indices`.
    pub listas: Vec<[u32; 2]>,
    pub mascaras: Vec<Mascara>,
    /// O lado de cada janela e o da atlas.
    pub lado: u32,
    pub lado_da_atlas: u32,
}

type P = [f64; 2];

fn sub(a: P, b: P) -> P {
    [a[0] - b[0], a[1] - b[1]]
}

fn cruz(a: P, b: P) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}

/// O fecho convexo (cadeia monótona), anti-horário.
fn fecho(mut pts: Vec<P>) -> Vec<P> {
    pts.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
    pts.dedup();
    if pts.len() < 3 {
        return pts;
    }
    let mut h: Vec<P> = Vec::with_capacity(pts.len() * 2);
    for passada in 0..2 {
        let inicio = h.len();
        let ordem: Box<dyn Iterator<Item = &P>> = if passada == 0 {
            Box::new(pts.iter())
        } else {
            Box::new(pts.iter().rev())
        };
        for &p in ordem {
            while h.len() >= inicio + 2 && cruz(sub(h[h.len() - 1], h[h.len() - 2]), sub(p, h[h.len() - 2])) <= 0.0 {
                h.pop();
            }
            h.push(p);
        }
        h.pop();
    }
    h
}

/// Dois convexos SE TOCAM? (eixos separadores: as normais das arestas de cada um.)
fn tocam(a: &[P], b: &[P]) -> bool {
    let separa = |poli: &[P]| {
        (0..poli.len()).any(|i| {
            let e = sub(poli[(i + 1) % poli.len()], poli[i]);
            let n = [-e[1], e[0]];
            let proj = |q: &[P]| {
                q.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), p| {
                    let x = n[0] * p[0] + n[1] * p[1];
                    (lo.min(x), hi.max(x))
                })
            };
            let (a0, a1) = proj(a);
            let (b0, b1) = proj(b);
            a1 < b0 || b1 < a0
        })
    };
    !(separa(a) || separa(b))
}

/// Um candidato a distante, antes de saber de que lâmpada é.
enum Candidato {
    Poligono { pts: Vec<P>, tau: f32 },
    Mascara { quad: [P; 3], s: [f64; 3], t: [f64; 3], tau: f32, textura: usize, uv: [f32; 4] },
}

impl Candidato {
    /// Os quatro cantos do paralelogramo que o envolve.
    fn cantos(&self) -> [P; 4] {
        let [o, a, b] = self.quad();
        [o, [o[0] + a[0], o[1] + a[1]], [o[0] + a[0] + b[0], o[1] + a[1] + b[1]], [o[0] + b[0], o[1] + b[1]]]
    }

    fn quad(&self) -> [P; 3] {
        match self {
            Self::Poligono { pts, .. } => {
                let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
                for p in pts {
                    lo = [lo[0].min(p[0]), lo[1].min(p[1])];
                    hi = [hi[0].max(p[0]), hi[1].max(p[1])];
                }
                [lo, [hi[0] - lo[0], 0.0], [0.0, hi[1] - lo[1]]]
            }
            Self::Mascara { quad, .. } => *quad,
        }
    }
}

/// O quad de um sprite em texels das cascatas e a inversa dele (`(s, t)` a partir do texel); `None`
/// se degenerado.
fn moldura(s: &SpriteNaC0, g: &Grade) -> Option<([P; 3], [f64; 3], [f64; 3])> {
    let o = [(s.origem[0] - g.origem_m[0]) / g.texel_m, (s.origem[1] - g.origem_m[1]) / g.texel_m];
    let a = [s.eixo_s[0] / g.texel_m, s.eixo_s[1] / g.texel_m];
    let b = [s.eixo_t[0] / g.texel_m, s.eixo_t[1] / g.texel_m];
    let det = cruz(a, b);
    if !(det.is_finite() && det.abs() > 1e-12) {
        return None;
    }
    // (s, t) = M⁻¹ (p − o), M = [a b] em colunas.
    let sl = [b[1] / det, -b[0] / det];
    let tl = [-a[1] / det, a[0] / det];
    let lin = |r: [f64; 2]| [r[0], r[1], -(r[0] * o[0] + r[1] * o[1])];
    Some(([o, a, b], lin(sl), lin(tl)))
}

/// Por lâmpada direta (a ordem do buffer delas): `None` se o disco cabe na grade; senão o fecho do
/// caminho até ela (a grade e a caixa do disco), em texels das cascatas.
fn caminhos(cena: &CenaDeLuz, g: &Grade, diretas: &[usize]) -> Vec<Option<Vec<P>>> {
    let (w, h) = (f64::from(g.largura), f64::from(g.altura));
    let na_grade = |p: &P| (0.0..=w).contains(&p[0]) && (0.0..=h).contains(&p[1]);
    let mut caminhos = Vec::with_capacity(diretas.len());
    for &i in diretas {
        let (c, r) = match cena.emissores.get(i) {
            Some(Emissor::Disco { centro, raio, .. }) => {
                (g.em_texels(*centro), f64::from(*raio) / g.texel_m)
            }
            Some(Emissor::Ponto { centro, .. }) => (g.em_texels(*centro), 0.0),
            _ => continue,
        };
        let caixa = [
            [c[0] - r, c[1] - r],
            [c[0] + r, c[1] - r],
            [c[0] + r, c[1] + r],
            [c[0] - r, c[1] + r],
        ];
        if caixa.iter().all(na_grade) {
            caminhos.push(None);
            continue;
        }
        let mut pts = vec![[0.0, 0.0], [w, 0.0], [w, h], [0.0, h]];
        pts.extend(caixa);
        caminhos.push(Some(fecho(pts)));
    }
    caminhos
}

/// ⭐ **Um opaco de caixa `[min, max]` (metros do mundo) tapa a luz direta FORA da grade `g`?** — o
/// que a C3 vai testar: não cabe inteiro na grade e toca o caminho até uma lâmpada direta que não
/// cabe nela. Quem desenha a máscara de um caminho vetorial pergunta aqui antes (o Vello custa).
#[must_use]
pub fn tapa_fora_da_grade(cena: &CenaDeLuz, g: &Grade, min: [f64; 2], max: [f64; 2]) -> bool {
    let t = |x: f64, y: f64| [(x - g.origem_m[0]) / g.texel_m, (y - g.origem_m[1]) / g.texel_m];
    let cantos = [t(min[0], min[1]), t(max[0], min[1]), t(max[0], max[1]), t(min[0], max[1])];
    let (w, h) = (f64::from(g.largura), f64::from(g.altura));
    if cantos.iter().all(|p| (0.0..=w).contains(&p[0]) && (0.0..=h).contains(&p[1])) {
        return false;
    }
    caminhos(cena, g, &crate::direta::lampadas_diretas(cena))
        .iter()
        .flatten()
        .any(|c| tocam(c, &cantos))
}

/// ⭐ **O mundo fora da grade `g` para as lâmpadas `diretas`** (a ordem do buffer delas): os opacos
/// da cena que não foram trocados pelos sprites, e os sprites (`sprites.sprites` e
/// `sprites.distantes`), com máscaras de lado `mascara_px` (menos, se não couberem em
/// [`ATLAS_MAX`]).
pub(crate) fn montar(
    cena: &CenaDeLuz,
    g: &Grade,
    diretas: &[usize],
    sprites: &CapturaDeSprites<'_>,
    mascara_px: u32,
) -> MundoDistante {
    let (w, h) = (f64::from(g.largura), f64::from(g.altura));
    let na_grade = |p: &P| (0.0..=w).contains(&p[0]) && (0.0..=h).contains(&p[1]);
    let em_texels = |v: &[[f32; 2]]| -> Vec<P> { v.iter().map(|&p| g.em_texels(p)).collect() };

    let caminhos = caminhos(cena, g, diretas);
    let mut m = MundoDistante {
        listas: vec![[0, 0]; caminhos.len()],
        ..Default::default()
    };
    if caminhos.iter().all(Option::is_none) {
        return m;
    }

    // Os candidatos: o que tapa e não cabe inteiro na grade.
    let mut candidatos: Vec<Candidato> = Vec::new();
    let substituido = |q: Quem| sprites.substituidos.contains(&q);
    for (k, o) in cena.oclusores.iter().enumerate() {
        if o.opacidade > 0.0 && o.vertices.len() >= 3 && !substituido(Quem::Oclusor(k)) {
            candidatos.push(Candidato::Poligono { pts: em_texels(&o.vertices), tau: 1.0 - o.opacidade.min(1.0) });
        }
    }
    for (k, e) in cena.emissores.iter().enumerate() {
        if let Emissor::Poligono { vertices, .. } = e
            && vertices.len() >= 3
            && !substituido(Quem::Emissor(k))
        {
            candidatos.push(Candidato::Poligono { pts: em_texels(vertices), tau: 0.0 });
        }
    }
    for s in sprites.sprites.iter().chain(&sprites.distantes) {
        if s.opacidade <= 0.0 {
            continue;
        }
        if let Some((quad, sl, tl)) = moldura(s, g) {
            candidatos.push(Candidato::Mascara {
                quad,
                s: sl,
                t: tl,
                tau: 1.0 - s.opacidade.min(1.0),
                textura: s.textura,
                uv: s.uv,
            });
        }
    }
    candidatos.retain(|c| !c.cantos().iter().all(na_grade));

    // Por lâmpada, os que tocam o caminho; cada candidato vira UM objeto (o primeiro uso o cria).
    let mut objeto_de: Vec<Option<u32>> = vec![None; candidatos.len()];
    let mut janela_de: BTreeMap<(usize, [u32; 4]), u32> = BTreeMap::new();
    let mut janela_do_objeto: Vec<(usize, u32)> = Vec::new();
    for (l, caminho) in caminhos.iter().enumerate() {
        let Some(caminho) = caminho else { continue };
        let primeiro = u32::try_from(m.indices.len()).expect("índices cabem em u32");
        for (ci, c) in candidatos.iter().enumerate() {
            if !tocam(caminho, &c.cantos()) {
                continue;
            }
            let id = *objeto_de[ci].get_or_insert_with(|| {
                let id = u32::try_from(m.objetos.len()).expect("objetos cabem em u32");
                let [o, a, b] = c.quad();
                let mut d = Distante {
                    quad_o_a: [o[0] as f32, o[1] as f32, a[0] as f32, a[1] as f32],
                    quad_b_janela: [b[0] as f32, b[1] as f32, 0.0, 0.0],
                    ..Default::default()
                };
                match c {
                    Candidato::Poligono { pts, tau } => {
                        // Anti-horário (área > 0): o interior à esquerda de cada aresta.
                        let area: f64 = (0..pts.len()).map(|i| cruz(pts[i], pts[(i + 1) % pts.len()])).sum();
                        d.tipo = POLIGONO;
                        d.t[3] = *tau;
                        d.primeira = u32::try_from(m.arestas.len()).expect("arestas cabem em u32");
                        d.n = u32::try_from(pts.len()).expect("arestas cabem em u32");
                        for i in 0..pts.len() {
                            let (mut a, mut b) = (pts[i], pts[(i + 1) % pts.len()]);
                            if area < 0.0 {
                                std::mem::swap(&mut a, &mut b);
                            }
                            m.arestas.push([a[0] as f32, a[1] as f32, b[0] as f32, b[1] as f32]);
                        }
                    }
                    Candidato::Mascara { s, t, tau, textura, uv, .. } => {
                        d.tipo = MASCARA;
                        d.s = [s[0] as f32, s[1] as f32, s[2] as f32, 0.0];
                        d.t = [t[0] as f32, t[1] as f32, t[2] as f32, *tau];
                        let n = u32::try_from(janela_de.len()).expect("máscaras cabem em u32");
                        let j = *janela_de.entry((*textura, uv.map(f32::to_bits))).or_insert(n);
                        if j == n {
                            m.mascaras.push(Mascara { textura: *textura, uv: *uv, janela: [0, 0] });
                        }
                        janela_do_objeto.push((id as usize, j));
                    }
                }
                m.objetos.push(d);
                id
            });
            m.indices.push(id);
        }
        let n = u32::try_from(m.indices.len()).expect("índices cabem em u32") - primeiro;
        m.listas[l] = [primeiro, n];
    }

    // As janelas na atlas: o lado pedido, ou menos se não couberem todas.
    if !m.mascaras.is_empty() {
        let n = m.mascaras.len() as u32;
        let por_lado = n.isqrt() + u32::from(n.isqrt() * n.isqrt() < n);
        let mut lado = mascara_px.min(ATLAS_MAX);
        while lado > 8 && por_lado * lado > ATLAS_MAX {
            lado /= 2;
        }
        m.lado = lado;
        m.lado_da_atlas = (por_lado * lado).next_power_of_two().min(ATLAS_MAX);
        let por_linha = m.lado_da_atlas / lado;
        for (j, mk) in m.mascaras.iter_mut().enumerate() {
            let j = j as u32;
            mk.janela = [(j % por_linha) * lado, (j / por_linha) * lado];
        }
        for (id, j) in janela_do_objeto {
            let [x, y] = m.mascaras[j as usize].janela;
            let d = &mut m.objetos[id];
            d.quad_b_janela[2] = x as f32;
            d.quad_b_janela[3] = y as f32;
            d.s[3] = lado as f32;
        }
    }
    m
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn o_fecho_e_o_toque() {
        let h = fecho(vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [2.0, 2.0], [10.0, 2.0]]);
        assert_eq!(h.len(), 5, "{h:?}");
        let perto = [[5.0, 1.5], [6.0, 1.5], [6.0, 2.5], [5.0, 2.5]];
        let longe = [[5.0, 4.0], [6.0, 4.0], [6.0, 5.0], [5.0, 5.0]];
        assert!(tocam(&h, &perto));
        assert!(!tocam(&h, &longe));
    }
}
