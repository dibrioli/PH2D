//! **C0, lado CPU** — a `CenaDeLuz` em formas na grade, e o relatório do que NÃO entra nela.
//!
//! Cada forma entra com a sua COBERTURA por texel, medida em 8×8 subamostras com a regra do
//! gabarito (paridade de cruzamentos; disco: `|p − c| < r`), e o seu material nos texels que POSSUI
//! (cobertura ≥ ½). O que não possui texel nenhum sai no [`RelatorioC0`] — nunca some em silêncio:
//! o `Ponto` (`tamanho_m = 0`) é o caminho analítico C3 (W5); uma forma fora da grade não existe
//! para a C0, e uma menor que o texel entra só como cobertura (W5a: semeia o campo de distância; não
//! acende nem tapa as cascatas, e a captura fina dos opacos vê-a).

use bytemuck::{Pod, Zeroable};
use ph2d_light2d_ecs::{CenaDeLuz, Emissor};

use crate::config::Grade;

/// Uma forma como o `c0_captura.wgsl` a lê (64 bytes).
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct Forma {
    /// A caixa em texels inteiros, já recortada à grade: `x0, y0, x1, y1`.
    pub caixa: [f32; 4],
    /// `(radiância, 1/alcance)`; um oclusor tem radiância 0.
    pub cor: [f32; 4],
    /// Disco: `centro.x, centro.y, raio, 1`; polígono: `w = 0`.
    pub disco: [f32; 4],
    /// Polígono: `(primeiro vértice, contagem)` em `vertices`.
    pub vertices: [u32; 2],
    /// 1 num corpo que brilha (opaco); 0 numa lâmpada (disco: luz, não corpo); a `opacidade` do
    /// `BloqueiaLuz` num oclusor.
    pub opacidade: f32,
    pub _pad: f32,
}

/// Quem ficou fora da C0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Quem {
    Emissor(usize),
    Oclusor(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motivo {
    /// `Emissor::Ponto`: sem área, não cabe num texel. É o caminho analítico C3 (W5).
    PontoSemArea,
    /// A forma não POSSUI texel nenhum (cobertura própria ≥ ½ em 8×8 subamostras): a marcha das
    /// CASCATAS, que diz «dentro» onde a cobertura interpolada passa de ½, nunca a vê. (W5a: a luz
    /// direta, sim — marcha na captura fina dos opacos.)
    MenorQueOTexel,
    /// A forma está toda fora da grade (a margem é escolha de quem monta a grade).
    ForaDaGrade,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForaDaC0 {
    pub quem: Quem,
    pub motivo: Motivo,
}

/// O que a C0 fez com a cena.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RelatorioC0 {
    pub fora: Vec<ForaDaC0>,
    /// Bytes de geometria que sobem à placa neste quadro (formas + vértices).
    pub bytes_da_geometria: u64,
    /// ⭐ W5b (F4): os opacos FORA da grade que a luz direta testa no caminho até as lâmpadas que
    /// não cabem nela, as máscaras distintas deles e o lado de cada uma (desce quando não cabem).
    pub distantes: usize,
    pub mascaras: usize,
    pub lado_da_mascara: u32,
}

pub(crate) struct Captura {
    pub formas: Vec<Forma>,
    pub vertices: Vec<[f32; 2]>,
    pub relatorio: RelatorioC0,
}

/// O maior finito do `f16` (o canal alfa da cena é `Rgba16Float`).
const F16_MAX: f32 = 65504.0;

/// Subamostras por eixo na cobertura (as mesmas do `c0_captura.wgsl`).
pub(crate) const SUB: u32 = 8;

/// `1/alcance` (0 = infinito), saturado no alcance do `f16` (≈ 15 µm): uma luz que não chega a
/// 15 µm só acende o próprio texel, e a queda dá 0 além dele dos dois modos.
fn inverso_do_alcance(alcance_m: f32) -> f32 {
    if alcance_m.is_infinite() {
        0.0
    } else {
        (1.0 / alcance_m).min(F16_MAX)
    }
}

/// Caixa em texels inteiros recortada à grade; `None` se vazia.
pub(crate) fn caixa(min: [f64; 2], max: [f64; 2], g: &Grade) -> Option<[f32; 4]> {
    let x0 = min[0].floor().max(0.0);
    let y0 = min[1].floor().max(0.0);
    let x1 = max[0].ceil().min(f64::from(g.largura));
    let y1 = max[1].ceil().min(f64::from(g.altura));
    (x1 > x0 && y1 > y0).then_some([x0 as f32, y0 as f32, x1 as f32, y1 as f32])
}

/// A regra de paridade do shader, na MESMA aritmética `f32`.
fn dentro_do_poligono(v: &[[f32; 2]], p: [f32; 2]) -> bool {
    let mut dentro = false;
    let mut j = v.len() - 1;
    for i in 0..v.len() {
        let (a, b) = (v[i], v[j]);
        if (a[1] > p[1]) != (b[1] > p[1])
            && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            dentro = !dentro;
        }
        j = i;
    }
    dentro
}

/// Uma forma na grade, para medir a cobertura como o `c0_captura.wgsl` a mede.
enum Geo<'a> {
    Disco([f32; 2], f32),
    Poligono(&'a [[f32; 2]]),
}

const MEIA_DIAGONAL: f32 = std::f32::consts::FRAC_1_SQRT_2;

impl Geo<'_> {
    fn dentro(&self, p: [f32; 2]) -> bool {
        match self {
            Self::Disco(c, r) => {
                let d = [p[0] - c[0], p[1] - c[1]];
                d[0] * d[0] + d[1] * d[1] < r * r
            }
            Self::Poligono(v) => dentro_do_poligono(v, p),
        }
    }

    fn distancia_a_borda(&self, p: [f32; 2]) -> f32 {
        match self {
            Self::Disco(c, r) => ((p[0] - c[0]).hypot(p[1] - c[1]) - r).abs(),
            Self::Poligono(v) => {
                let mut d = f32::INFINITY;
                let mut j = v.len() - 1;
                for i in 0..v.len() {
                    let a = v[j];
                    let ab = [v[i][0] - a[0], v[i][1] - a[1]];
                    let ap = [p[0] - a[0], p[1] - a[1]];
                    let t = ((ap[0] * ab[0] + ap[1] * ab[1])
                        / (ab[0] * ab[0] + ab[1] * ab[1]).max(1e-30))
                    .clamp(0.0, 1.0);
                    d = d.min((ap[0] - ab[0] * t).hypot(ap[1] - ab[1] * t));
                    j = i;
                }
                d
            }
        }
    }

    /// A cobertura do texel de centro `p` — a mesma conta do shader.
    fn cobertura(&self, p: [f32; 2]) -> f32 {
        if self.distancia_a_borda(p) >= MEIA_DIAGONAL {
            return if self.dentro(p) { 1.0 } else { 0.0 };
        }
        let mut n = 0u32;
        for j in 0..SUB {
            for i in 0..SUB {
                let q = [
                    p[0] - 0.5 + (i as f32 + 0.5) / SUB as f32,
                    p[1] - 0.5 + (j as f32 + 0.5) / SUB as f32,
                ];
                n += u32::from(self.dentro(q));
            }
        }
        n as f32 / (SUB * SUB) as f32
    }

    /// A forma POSSUI algum texel da caixa (cobertura ≥ ½)? Sem isso a marcha nunca a vê. Começa
    /// pelo texel do meio da caixa (o caso comum acaba ali).
    fn possui_algum_texel(&self, c: [f32; 4]) -> bool {
        let (x0, y0, x1, y1) = (c[0] as u32, c[1] as u32, c[2] as u32, c[3] as u32);
        let centro = |i: u32, j: u32| [i as f32 + 0.5, j as f32 + 0.5];
        let (im, jm) = (x0 + (x1 - x0) / 2, y0 + (y1 - y0) / 2);
        self.cobertura(centro(im, jm)) >= 0.5
            || (y0..y1).any(|j| (x0..x1).any(|i| self.cobertura(centro(i, j)) >= 0.5))
    }
}

struct Montagem<'a> {
    g: &'a Grade,
    formas: Vec<Forma>,
    vertices: Vec<[f32; 2]>,
    fora: Vec<ForaDaC0>,
}

impl Montagem<'_> {
    fn poligono(&mut self, quem: Quem, mundo: &[[f32; 2]], cor: [f32; 4], opacidade: f32) {
        if mundo.len() < 3 {
            self.fora.push(ForaDaC0 {
                quem,
                motivo: Motivo::MenorQueOTexel,
            });
            return;
        }
        let pts: Vec<[f64; 2]> = mundo.iter().map(|&p| self.g.em_texels(p)).collect();
        let min = pts
            .iter()
            .fold([f64::INFINITY; 2], |m, p| [m[0].min(p[0]), m[1].min(p[1])]);
        let max = pts.iter().fold([f64::NEG_INFINITY; 2], |m, p| {
            [m[0].max(p[0]), m[1].max(p[1])]
        });
        let Some(c) = caixa(min, max, self.g) else {
            self.fora.push(ForaDaC0 {
                quem,
                motivo: Motivo::ForaDaGrade,
            });
            return;
        };
        let v: Vec<[f32; 2]> = pts.iter().map(|p| [p[0] as f32, p[1] as f32]).collect();
        // ⭐ W5a: menor que o texel, fica DITA (não acende nem tapa as cascatas) mas entra como
        // cobertura — semeia o campo de distância para a captura fina dos opacos, que a vê.
        if !Geo::Poligono(&v).possui_algum_texel(c) {
            self.fora.push(ForaDaC0 {
                quem,
                motivo: Motivo::MenorQueOTexel,
            });
        }
        let primeiro = u32::try_from(self.vertices.len()).expect("vértices cabem em u32");
        let n = u32::try_from(v.len()).expect("vértices cabem em u32");
        self.vertices.extend(v);
        self.formas.push(Forma {
            caixa: c,
            cor,
            disco: [0.0; 4],
            vertices: [primeiro, n],
            opacidade,
            _pad: 0.0,
        });
    }

    fn disco(&mut self, quem: Quem, centro: [f32; 2], raio_m: f32, cor: [f32; 4]) {
        let c = self.g.em_texels(centro);
        let r = f64::from(raio_m) / self.g.texel_m;
        let Some(cx) = caixa([c[0] - r, c[1] - r], [c[0] + r, c[1] + r], self.g) else {
            self.fora.push(ForaDaC0 {
                quem,
                motivo: Motivo::ForaDaGrade,
            });
            return;
        };
        let (c32, r32) = ([c[0] as f32, c[1] as f32], r as f32);
        if !Geo::Disco(c32, r32).possui_algum_texel(cx) {
            self.fora.push(ForaDaC0 {
                quem,
                motivo: Motivo::MenorQueOTexel,
            });
        }
        self.formas.push(Forma {
            caixa: cx,
            cor,
            disco: [c32[0], c32[1], r32, 1.0],
            vertices: [0; 2],
            // Uma lâmpada é LUZ, não corpo: emite e deixa passar (a marcha soma-a na entrada).
            opacidade: 0.0,
            _pad: 0.0,
        });
    }
}

/// O que a C0 deixaria de fora de `cena` nesta grade — a mesma conta do quadro, sem placa.
#[must_use]
pub fn o_que_fica_fora(cena: &CenaDeLuz, g: &Grade) -> RelatorioC0 {
    capturar(cena, g).relatorio
}

pub(crate) fn capturar(cena: &CenaDeLuz, g: &Grade) -> Captura {
    let diretas = crate::direta::lampadas_diretas(cena);
    capturar_sem(cena, g, &std::collections::BTreeSet::new(), &diretas)
}

/// ⭐ **Só o que TAPA a luz** (W5a), para a captura fina dos opacos: os oclusores com opacidade e os
/// corpos que brilham (polígonos, opacos). As lâmpadas são luz, não corpo, e a luz direta só precisa
/// da transmissão (a emissão fica de fora: o alvo dela não existe na captura fina).
pub(crate) fn capturar_opacos(
    cena: &CenaDeLuz,
    g: &Grade,
    substituidas: &std::collections::BTreeSet<Quem>,
) -> Captura {
    let mut m = Montagem {
        g,
        formas: Vec::new(),
        vertices: Vec::new(),
        fora: Vec::new(),
    };
    for (k, o) in cena.oclusores.iter().enumerate() {
        if o.opacidade > 0.0 && !substituidas.contains(&Quem::Oclusor(k)) {
            m.poligono(Quem::Oclusor(k), &o.vertices, [0.0; 4], o.opacidade);
        }
    }
    for (k, e) in cena.emissores.iter().enumerate() {
        if let Emissor::Poligono { vertices, .. } = e
            && !substituidas.contains(&Quem::Emissor(k))
        {
            m.poligono(Quem::Emissor(k), vertices, [0.0; 4], 1.0);
        }
    }
    let bytes_da_geometria =
        (m.formas.len() * size_of::<Forma>() + m.vertices.len() * size_of::<[f32; 2]>()) as u64;
    Captura {
        formas: m.formas,
        vertices: m.vertices,
        relatorio: RelatorioC0 {
            fora: m.fora,
            bytes_da_geometria,
            ..Default::default()
        },
    }
}

/// A captura sem as formas `substituidas` (o quad de um sprite que a C0 captura pela textura).
pub(crate) fn capturar_sem(
    cena: &CenaDeLuz,
    g: &Grade,
    substituidas: &std::collections::BTreeSet<Quem>,
    diretas: &[usize],
) -> Captura {
    let mut m = Montagem {
        g,
        formas: Vec::new(),
        vertices: Vec::new(),
        fora: Vec::new(),
    };
    for (k, o) in cena.oclusores.iter().enumerate() {
        if substituidas.contains(&Quem::Oclusor(k)) {
            continue;
        }
        m.poligono(Quem::Oclusor(k), &o.vertices, [0.0; 4], o.opacidade);
    }
    for (k, e) in cena.emissores.iter().enumerate() {
        let quem = Quem::Emissor(k);
        if substituidas.contains(&quem) {
            continue;
        }
        match e {
            Emissor::Disco {
                centro,
                raio,
                radiancia,
                alcance_m,
            } => {
                // ⭐ Uma lâmpada DIRETA (C3) NÃO entra na captura: a luz dela vem do caminho direto
                // (não a contar duas vezes), e um disco aqui fazia o «cone livre» do caminho direto
                // falhar perto da PRÓPRIA lâmpada em todo ponto — medido 1,1 ms por lâmpada (Low,
                // 1080p). Nenhuma lâmpada bloqueia luz (é luz, não corpo).
                if diretas.binary_search(&k).is_ok() {
                    continue;
                }
                let [r, g, b] = *radiancia;
                m.disco(
                    quem,
                    *centro,
                    *raio,
                    [r, g, b, inverso_do_alcance(*alcance_m)],
                );
            }
            Emissor::Poligono {
                vertices,
                radiancia,
                alcance_m,
            } => {
                let [r, g, b] = *radiancia;
                m.poligono(
                    quem,
                    vertices,
                    [r, g, b, inverso_do_alcance(*alcance_m)],
                    1.0,
                );
            }
            // O ponto DIRETO (C3) não precisa da grade.
            Emissor::Ponto { .. } if diretas.binary_search(&k).is_ok() => {}
            Emissor::Ponto { .. } => m.fora.push(ForaDaC0 {
                quem,
                motivo: Motivo::PontoSemArea,
            }),
        }
    }
    let bytes_da_geometria =
        (m.formas.len() * size_of::<Forma>() + m.vertices.len() * size_of::<[f32; 2]>()) as u64;
    Captura {
        formas: m.formas,
        vertices: m.vertices,
        relatorio: RelatorioC0 {
            fora: m.fora,
            bytes_da_geometria,
            ..Default::default()
        },
    }
}
