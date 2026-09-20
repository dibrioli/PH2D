//! ⭐⭐⭐ **A SOBREPOSIÇÃO — o vermelho que impede o atlas de ser pintado, medido por
//! ÁREA e ATRIBUÍDO a um mecanismo.**
//!
//! A W1 mediu-a por TEXEL e a resposta foi `0,04 %` numa peça e `38,1 %` noutra. ⛔ Um
//! número agregado diz que há um defeito e **não diz o que se corta**: *uma régua que
//! conta QUANTOS nunca vê QUAIS* — a lei que esta casa já pagou no `edge_max` cego ao
//! quad fino, no `χ` cego à almofada e nas três réguas da ponta que deitavam fora o
//! índice antes de devolver.
//!
//! # As quatro classes, e porque cada uma tem outra cura
//!
//! | classe | o que aconteceu | onde está a cura |
//! |---|---|---|
//! | [`Classe::Dobra`] | os dois triângulos são **vizinhos na peça** e mesmo assim se cruzam ⇒ o mapa inverteu-se ali | a montante, no solver contínuo (G3) |
//! | [`Classe::MesmaCarta`] | uma carta não é injectiva **sozinha** | a montante, ou cortar a carta |
//! | [`Classe::MesmaIlha`] | duas cartas da mesma ilha foram **assentadas uma em cima da outra** | o CORTE: a ilha parte-se |
//! | [`Classe::IlhasDiferentes`] | ⛔ **CONTROLO — tem de ser ZERO**: as caixas das ilhas são disjuntas por construção, logo um acerto aqui acusa o empacotador | o empacotador |
//!
//! ⚠️ **Sem a última linha isto não é uma régua, é uma opinião:** as três primeiras classes
//! não têm lado aprovado nenhum, e a quarta é a única que este ficheiro sabe dizer que
//! está certa.
//!
//! # ⛔ O que a [`Sobreposicao::area_cruzada`] NÃO é
//!
//! Ela soma **pares**. Onde três folhas caem no mesmo sítio, a mesma área é contada três
//! vezes — logo ela é um **limite superior** da área pintada mais do que uma vez, e é
//! isso que a torna útil a quem CORTA (o corte trabalha em pares). Quem quiser a área
//! honesta pergunta por texel, que é a régua da sonda.

use crate::topo;
use ph2d_mesh::Mesh;

/// ⚠️ **A fracção abaixo da qual um cruzamento é ruído da aritmética e não geometria.**
///
/// Dois triângulos que partilham uma aresta recortam-se num polígono de área exactamente
/// zero, e em `f64` sobre coordenadas de ordem `1` isso lê-se `~1e-16`. A barra é
/// **relativa ao menor dos dois triângulos** e fica `10` ordens de grandeza acima desse
/// ruído — ⛔ um epsilon ABSOLUTO acusaria toda a vizinhança de uma malha fina, onde um
/// triângulo inteiro mede `1e-6` do atlas.
pub const RUIDO_RELATIVO: f64 = 1.0e-6;

/// De onde vem um cruzamento. Ver a tabela do cabeçalho deste módulo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Classe {
    /// Vizinhos na peça que se cruzam no atlas: o mapa dobrou.
    Dobra,
    /// Mesma carta, sem fronteira comum.
    MesmaCarta,
    /// Cartas diferentes da mesma ilha.
    MesmaIlha,
    /// ⛔ Ilhas diferentes — controlo, tem de ser zero.
    IlhasDiferentes,
}

impl Classe {
    /// As quatro, na ordem em que o relatório as indexa.
    pub const ALL: [Self; 4] = [
        Self::Dobra,
        Self::MesmaCarta,
        Self::MesmaIlha,
        Self::IlhasDiferentes,
    ];

    /// O índice desta classe nas tabelas do [`Sobreposicao`].
    #[must_use]
    pub const fn indice(self) -> usize {
        match self {
            Self::Dobra => 0,
            Self::MesmaCarta => 1,
            Self::MesmaIlha => 2,
            Self::IlhasDiferentes => 3,
        }
    }

    /// O nome curto, para uma tabela impressa.
    #[must_use]
    pub const fn nome(self) -> &'static str {
        match self {
            Self::Dobra => "dobra",
            Self::MesmaCarta => "mesma-carta",
            Self::MesmaIlha => "mesma-ilha",
            Self::IlhasDiferentes => "ilhas-diferentes",
        }
    }
}

/// Um par de triângulos do atlas que se cruzam.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Par {
    /// Índices no vector de [`topo::triangulos`], com `a < b`.
    pub a: u32,
    /// Ver [`Self::a`].
    pub b: u32,
    /// A área do cruzamento, em unidades de `[0,1]²`.
    pub area: f64,
    /// Ver [`Classe`].
    pub classe: Classe,
}

/// O que a régua achou.
#[derive(Debug, Clone, Default)]
pub struct Sobreposicao {
    /// A soma das áreas de todos os triângulos do atlas — o denominador honesto.
    pub area_pintada: f64,
    /// A soma das áreas dos cruzamentos. ⚠️ Ver o cabeçalho: é um limite SUPERIOR.
    pub area_cruzada: f64,
    /// Quantos pares, por [`Classe::indice`].
    pub pares_por_classe: [usize; 4],
    /// Quanta área, por [`Classe::indice`].
    pub area_por_classe: [f64; 4],
    /// Quantos triângulos entram em pelo menos um cruzamento.
    pub triangulos_tocados: usize,
    /// Quantos triângulos o atlas tem.
    pub triangulos: usize,
    /// ⛔ **O piso de população desta régua.** Sem ele um atlas vazio lê-se como um atlas
    /// limpo — *um zero de «não medido» e um de «perfeito» são o mesmo byte*.
    pub triangulos_com_area: usize,
    /// Os pares, do maior cruzamento para o menor.
    pub pares: Vec<Par>,
}

impl Sobreposicao {
    /// A fracção da área pintada que está debaixo de pelo menos um cruzamento.
    #[must_use]
    pub fn fraccao(&self) -> f64 {
        if self.area_pintada <= 0.0 {
            0.0
        } else {
            self.area_cruzada / self.area_pintada
        }
    }
}

/// A área com sinal de um polígono.
fn area_com_sinal(p: &[[f64; 2]]) -> f64 {
    let mut s = 0.0;
    for i in 0..p.len() {
        let (a, b) = (p[i], p[(i + 1) % p.len()]);
        s += a[0].mul_add(b[1], -(b[0] * a[1]));
    }
    s * 0.5
}

/// Recorta um polígono convexo ao semiplano à ESQUERDA de `a → b` (Sutherland–Hodgman).
fn recorta(poly: &[[f64; 2]], a: [f64; 2], b: [f64; 2]) -> Vec<[f64; 2]> {
    let dentro = |p: [f64; 2]| -> f64 {
        (b[0] - a[0]).mul_add(p[1] - a[1], -((b[1] - a[1]) * (p[0] - a[0])))
    };
    let mut out: Vec<[f64; 2]> = Vec::with_capacity(poly.len() + 3);
    for i in 0..poly.len() {
        let (p, q) = (poly[i], poly[(i + 1) % poly.len()]);
        let (dp, dq) = (dentro(p), dentro(q));
        if dp >= 0.0 {
            out.push(p);
        }
        if (dp > 0.0 && dq < 0.0) || (dp < 0.0 && dq > 0.0) {
            let t = dp / (dp - dq);
            out.push([p[0] + t * (q[0] - p[0]), p[1] + t * (q[1] - p[1])]);
        }
    }
    out
}

/// ⭐ **A área em que dois triângulos se cruzam**, exacta a menos do arredondamento de
/// `f64`.
///
/// ⚠️ Os dois são orientados no mesmo sentido antes do recorte: um triângulo DOBRADO tem
/// área negativa, e o recorte a um semiplano «de dentro» leria o lado errado.
#[must_use]
pub fn area_de_interseccao(a: [[f32; 2]; 3], b: [[f32; 2]; 3]) -> f64 {
    let sextuplo = |t: [[f32; 2]; 3]| -> [[f64; 2]; 3] {
        let mut q = [
            [f64::from(t[0][0]), f64::from(t[0][1])],
            [f64::from(t[1][0]), f64::from(t[1][1])],
            [f64::from(t[2][0]), f64::from(t[2][1])],
        ];
        if area_com_sinal(&q) < 0.0 {
            q.swap(1, 2);
        }
        q
    };
    let (qa, qb) = (sextuplo(a), sextuplo(b));
    let mut poly: Vec<[f64; 2]> = qa.to_vec();
    for k in 0..3 {
        if poly.is_empty() {
            return 0.0;
        }
        poly = recorta(&poly, qb[k], qb[(k + 1) % 3]);
    }
    if poly.len() < 3 {
        0.0
    } else {
        area_com_sinal(&poly).abs()
    }
}

/// A área (sem sinal) de um triângulo do atlas.
fn area(uv: &[[f32; 2]], t: [u32; 3]) -> f64 {
    let (Some(&a), Some(&b), Some(&c)) = (
        uv.get(t[0] as usize),
        uv.get(t[1] as usize),
        uv.get(t[2] as usize),
    ) else {
        return 0.0;
    };
    let (ux, uy) = (f64::from(b[0] - a[0]), f64::from(b[1] - a[1]));
    let (vx, vy) = (f64::from(c[0] - a[0]), f64::from(c[1] - a[1]));
    (ux.mul_add(vy, -(uy * vx)) * 0.5).abs()
}

fn canto(uv: &[[f32; 2]], t: [u32; 3]) -> [[f32; 2]; 3] {
    [
        *uv.get(t[0] as usize).unwrap_or(&[0.0, 0.0]),
        *uv.get(t[1] as usize).unwrap_or(&[0.0, 0.0]),
        *uv.get(t[2] as usize).unwrap_or(&[0.0, 0.0]),
    ]
}

/// ⭐⭐⭐ **Mede a sobreposição do atlas e atribui cada cruzamento a um mecanismo.**
///
/// A busca larga é uma grelha uniforme sobre `[0,1]²` dimensionada para `~2` triângulos
/// por célula; a busca estreita é [`area_de_interseccao`], que é exacta.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn medir(mesh: &Mesh, atlas: &crate::Atlas) -> Sobreposicao {
    let tris = topo::triangulos(mesh);
    let vizinhos: std::collections::BTreeSet<(u32, u32)> = topo::elos(mesh, &atlas.uv)
        .into_iter()
        .filter(|(_, _, e)| e.na_peca)
        .map(|(a, b, _)| (a, b))
        .collect();

    let mut s = Sobreposicao {
        triangulos: tris.len(),
        ..Sobreposicao::default()
    };
    let areas: Vec<f64> = tris.iter().map(|&t| area(&atlas.uv, t)).collect();
    s.area_pintada = areas.iter().sum();
    s.triangulos_com_area = areas.iter().filter(|&&a| a > 0.0).count();
    if tris.is_empty() {
        return s;
    }

    // ── Busca larga: uma grelha uniforme, ~2 triângulos por célula.
    let n = (((tris.len() as f64) / 2.0).sqrt().ceil() as usize).clamp(1, 4096);
    let mut celulas: Vec<Vec<u32>> = vec![Vec::new(); n * n];
    let indice = |z: f32| -> usize {
        let i = (f64::from(z) * n as f64).floor();
        (i.max(0.0) as usize).min(n - 1)
    };
    for (t, tri) in tris.iter().enumerate() {
        let c = canto(&atlas.uv, *tri);
        let (x0, x1) = (
            indice(c[0][0].min(c[1][0]).min(c[2][0])),
            indice(c[0][0].max(c[1][0]).max(c[2][0])),
        );
        let (y0, y1) = (
            indice(c[0][1].min(c[1][1]).min(c[2][1])),
            indice(c[0][1].max(c[1][1]).max(c[2][1])),
        );
        for y in y0..=y1 {
            for x in x0..=x1 {
                celulas[y * n + x].push(u32::try_from(t).unwrap_or(0));
            }
        }
    }

    let mut vistos: std::collections::BTreeSet<(u32, u32)> = std::collections::BTreeSet::new();
    let mut tocado = vec![false; tris.len()];
    for celula in &celulas {
        for i in 0..celula.len() {
            for j in (i + 1)..celula.len() {
                let (ta, tb) = (celula[i].min(celula[j]), celula[i].max(celula[j]));
                if !vistos.insert((ta, tb)) {
                    continue;
                }
                let (ia, ib) = (ta as usize, tb as usize);
                let piso = areas[ia].min(areas[ib]) * RUIDO_RELATIVO;
                let cruz =
                    area_de_interseccao(canto(&atlas.uv, tris[ia]), canto(&atlas.uv, tris[ib]));
                if cruz <= piso {
                    continue;
                }
                let classe = classifica(atlas, &tris, ia, ib, &vizinhos, (ta, tb));
                s.area_cruzada += cruz;
                s.pares_por_classe[classe.indice()] += 1;
                s.area_por_classe[classe.indice()] += cruz;
                tocado[ia] = true;
                tocado[ib] = true;
                s.pares.push(Par {
                    a: ta,
                    b: tb,
                    area: cruz,
                    classe,
                });
            }
        }
    }
    s.triangulos_tocados = tocado.iter().filter(|&&b| b).count();
    s.pares.sort_by(|x, y| y.area.total_cmp(&x.area));
    s
}

fn classifica(
    atlas: &crate::Atlas,
    tris: &[[u32; 3]],
    ia: usize,
    ib: usize,
    vizinhos: &std::collections::BTreeSet<(u32, u32)>,
    par: (u32, u32),
) -> Classe {
    let ilha = |t: usize| {
        atlas
            .ilha
            .get(tris[t][0] as usize)
            .copied()
            .unwrap_or(u32::MAX)
    };
    let carta = |t: usize| {
        atlas
            .carta
            .get(tris[t][0] as usize)
            .copied()
            .unwrap_or(u32::MAX)
    };
    if vizinhos.contains(&par) {
        Classe::Dobra
    } else if ilha(ia) != ilha(ib) {
        Classe::IlhasDiferentes
    } else if carta(ia) == carta(ib) {
        Classe::MesmaCarta
    } else {
        Classe::MesmaIlha
    }
}
