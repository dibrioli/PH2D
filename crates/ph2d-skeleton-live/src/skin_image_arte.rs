//! ⭐⭐⭐ **ONDE A TINTA COMEÇA** (A5-a, ordem do dono 2026-10-04: *«corrigir»* a cúspide da imagem
//! presa junto a uma tampa redonda).
//!
//! A malha de uma imagem passa da tinta DE PROPÓSITO (`GridOptions::expand`, o *Expansion* do
//! *Puppet* — ⛔ fazê-la seguir o contorno trouxe de volta as células deformadas da borda), e nas
//! tampas a borda dela é a escada da grelha, até uma célula fora da arte. A costura
//! ([`crate::skin_image_fecho`]) media o vão entre as bordas da MALHA: na cúspide onde a tampa encosta
//! tangente noutro membro a malha dizia «sem vão» e a arte deixava um fio de fundo (FOTOGRAFADO a
//! `(36°, −144°)`: `< 1 px` de largura, `~16 px` de comprimento a `100 %`).
//!
//! ⇒ Ao prender guarda-se ONDE há tinta ([`Mascara`], corridas por linha da célula); uma vez por malha
//! a borda é amostrada a cada texel e cada ponto ENCOSTADO para dentro até à primeira tinta
//! ([`anel_da_arte`]), com o triângulo onde cai — posado no quadro pelos vértices desse triângulo,
//! ele está exactamente onde a imagem desenha aquele texel.

use ph2d_poly2d::Mesh2d;

/// Até onde se procura a tinta para dentro de um ponto da borda, em pixels da célula — a escada de
/// uma célula grossa (`GridOptions::coarse = 26`) na diagonal, mais a expansão.
const MARCHA_MAXIMA: f64 = 40.0;
/// O passo da procura, em pixels.
const PASSO: f64 = 0.25;
/// ⭐ **O alfa a partir do qual a máscara diz «tinta»: a tinta CHEIA** (`128`, a «tinta» das réguas
/// da costura). ⛔ Com o limiar da malha (`1`) o ponto parava no 1.º pixel da borda suave e entre ele
/// e a tinta cheia ficava uma meia-luz: um fio claro dentro do remendo (FOTOGRAFADO a `−147°`).
pub const TINTA_CHEIA: u8 = 128;
/// Quanto mais para dentro da tinta se vai buscar a cor de um ponto encostado — a borda suave tem
/// `~1` pixel.
pub const COR_DENTRO: f64 = 1.5;
/// Abaixo disto o ponto fica na borda da malha (ela já está sobre a tinta).
const ENCOSTO_MINIMO: f64 = 1.0;

/// ⭐⭐ **Onde há TINTA na célula da arte** — por linha de pixels, as corridas `[x0, x1)` de alfa
/// acima do limiar da malha.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Mascara {
    pub largura: u32,
    pub altura: u32,
    /// As corridas da linha `y` são `corridas[inicio[y]..inicio[y + 1]]`.
    pub inicio: Vec<u32>,
    pub corridas: Vec<[u32; 2]>,
}

impl Mascara {
    /// A tinta de um alfa `largura × altura` (alfa `≥ limiar`).
    #[must_use]
    pub fn do_alfa(alfa: &[u8], largura: u32, altura: u32, limiar: u8) -> Self {
        let mut t = Self {
            largura,
            altura,
            inicio: vec![0],
            corridas: Vec::new(),
        };
        for y in 0..altura {
            let mut x = 0;
            while x < largura {
                let tem = |x: u32| {
                    alfa.get((y * largura + x) as usize)
                        .is_some_and(|a| *a >= limiar)
                };
                if tem(x) {
                    let x0 = x;
                    while x < largura && tem(x) {
                        x += 1;
                    }
                    t.corridas.push([x0, x]);
                } else {
                    x += 1;
                }
            }
            #[expect(clippy::cast_possible_truncation, reason = "corridas de uma célula")]
            t.inicio.push(t.corridas.len() as u32);
        }
        t
    }

    /// O pixel que contém `p` (pixels da célula) tem tinta?
    #[must_use]
    pub fn tem(&self, p: [f64; 2]) -> bool {
        if !(p[0] >= 0.0 && p[1] >= 0.0) {
            return false;
        }
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "pixel"
        )]
        let (x, y) = (p[0] as u32, p[1] as u32);
        if x >= self.largura || y >= self.altura {
            return false;
        }
        let (a, b) = (
            self.inicio[y as usize] as usize,
            self.inicio[y as usize + 1] as usize,
        );
        let linha = &self.corridas[a..b];
        let i = linha.partition_point(|c| c[1] <= x);
        linha.get(i).is_some_and(|c| c[0] <= x)
    }
}

/// Um ponto do anel da arte: o triângulo da malha onde cai e as coordenadas baricêntricas `(u, v)`
/// nele (o ponto é `(1 − u − v)·a + u·b + v·c`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PontoDaArte {
    pub tri: u32,
    pub uv: [f64; 2],
    /// Onde a costura vai buscar a COR deste ponto, em pixels da célula: [`COR_DENTRO`] mais para
    /// dentro da tinta (o primeiro pixel com tinta é a borda suave, quase transparente).
    pub cor: [f64; 2],
}

/// ⭐⭐ **As bordas de uma malha desenhada**: os anéis (as arestas de um só triângulo) e, quando a
/// malha traz [`Mascara`], o anel da ARTE de cada um ([`anel_da_arte`]). Calculadas UMA vez por malha
/// ([`crate::skin_image_fecho::bordas_da`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BordasDaMalha {
    pub aneis: Vec<Vec<u32>>,
    /// O triângulo de cada segmento de cada anel — onde a costura daquela beira entra na ordem.
    pub faces: Vec<Vec<u32>>,
    /// Vazio quando a malha não traz tinta (um bind anterior a 2026-10-04): a costura mede a malha.
    pub arte: Vec<Vec<PontoDaArte>>,
}

impl BordasDaMalha {
    /// As bordas de `mesh`; com `tinta`, também o anel da arte. Sem ela, a lei de antes.
    #[must_use]
    pub fn da(mesh: &Mesh2d, tinta: Option<&Mascara>) -> Self {
        let aneis = crate::skin_image_fecho::aneis_da_borda(&mesh.tris);
        Self {
            faces: crate::skin_image_costura::faces_dos_aneis(&mesh.tris, &aneis),
            arte: tinta.map_or_else(Vec::new, |m| anel_da_arte(mesh, m, &aneis)),
            aneis,
        }
    }
}

/// Coordenadas baricêntricas `(u, v)` de `p` em `abc`; `None` num triângulo nulo.
fn bari(p: [f64; 2], a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> Option<[f64; 2]> {
    let den = (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]);
    if den.abs() < 1e-12 {
        return None;
    }
    let u = ((p[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (p[1] - a[1])) / den;
    let v = ((b[0] - a[0]) * (p[1] - a[1]) - (p[0] - a[0]) * (b[1] - a[1])) / den;
    Some([u, v])
}

/// ⭐⭐⭐ **O anel da ARTE de cada anel da borda** — um ponto por pixel ao longo de cada aresta,
/// encostado para dentro (a normal da aresta, para o lado da malha) até ao primeiro pixel com tinta;
/// sem tinta até [`MARCHA_MAXIMA`] fica na borda. Onde a borda já está sobre a tinta (a arte encosta
/// à orla da imagem) o ponto é o da borda: ali a lei é a de antes.
#[must_use]
pub fn anel_da_arte(mesh: &Mesh2d, tinta: &Mascara, aneis: &[Vec<u32>]) -> Vec<Vec<PontoDaArte>> {
    let r = |v: u32| mesh.rest[v as usize];
    // O sentido de dentro: os anéis vêm encadeados pela orientação dos triângulos, e a normal à
    // ESQUERDA aponta para a malha quando o anel de fora (o de maior área) roda no sentido directo.
    let area = |a: &[u32]| -> f64 {
        (0..a.len())
            .map(|i| {
                let (p, q) = (r(a[i]), r(a[(i + 1) % a.len()]));
                p[0] * q[1] - q[0] * p[1]
            })
            .sum()
    };
    let sinal = aneis
        .iter()
        .map(|a| area(a))
        .max_by(|a, b| a.abs().total_cmp(&b.abs()))
        .map_or(1.0, f64::signum);
    // Os triângulos por célula de `8` pixels, para achar onde cada ponto cai.
    const LADO: f64 = 8.0;
    #[expect(clippy::cast_possible_truncation, reason = "célula da grelha")]
    let cel = |p: [f64; 2]| [(p[0] / LADO).floor() as i64, (p[1] / LADO).floor() as i64];
    let mut grelha: Vec<([i64; 2], u32)> = Vec::new();
    for (i, t) in mesh.tris.iter().enumerate() {
        let q = t.map(r);
        let lo = cel([
            q[0][0].min(q[1][0]).min(q[2][0]),
            q[0][1].min(q[1][1]).min(q[2][1]),
        ]);
        let hi = cel([
            q[0][0].max(q[1][0]).max(q[2][0]),
            q[0][1].max(q[1][1]).max(q[2][1]),
        ]);
        for y in lo[1]..=hi[1] {
            for x in lo[0]..=hi[0] {
                #[expect(clippy::cast_possible_truncation, reason = "índice de triângulo")]
                grelha.push(([x, y], i as u32));
            }
        }
    }
    grelha.sort_unstable();
    let onde = |p: [f64; 2]| -> Option<PontoDaArte> {
        let c = cel(p);
        let de = grelha.partition_point(|e| e.0 < c);
        grelha[de..]
            .iter()
            .take_while(|e| e.0 == c)
            .find_map(|&(_, i)| {
                let q = mesh.tris[i as usize].map(r);
                let uv = bari(p, q[0], q[1], q[2])?;
                (uv[0] >= -1e-9 && uv[1] >= -1e-9 && uv[0] + uv[1] <= 1.0 + 1e-9)
                    .then_some(PontoDaArte { tri: i, uv, cor: p })
            })
    };
    aneis
        .iter()
        .map(|anel| {
            let n = anel.len();
            let mut out = Vec::new();
            for i in 0..n {
                let (a, b) = (r(anel[i]), r(anel[(i + 1) % n]));
                let d = [b[0] - a[0], b[1] - a[1]];
                let l = d[0].hypot(d[1]);
                if l <= 0.0 {
                    continue;
                }
                let dentro = [-d[1] / l * sinal, d[0] / l * sinal];
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "amostras"
                )]
                let k = (l.ceil() as usize).max(1);
                for j in 0..k {
                    #[expect(clippy::cast_precision_loss, reason = "amostras")]
                    let t = j as f64 / k as f64;
                    let p = [d[0].mul_add(t, a[0]), d[1].mul_add(t, a[1])];
                    let mut s = 0.0;
                    let encosto = loop {
                        let q = [dentro[0].mul_add(s, p[0]), dentro[1].mul_add(s, p[1])];
                        if tinta.tem(q) {
                            // ⚠️ O ponto é a ÚLTIMA amostra fora da tinta: a 1.ª dentro dela deixava
                            // um remendo fino com o centro sobre a tinta (gate da F49, MEDIDO).
                            let fora = (s - PASSO).max(0.0);
                            break Some((
                                [dentro[0].mul_add(fora, p[0]), dentro[1].mul_add(fora, p[1])],
                                q,
                            ));
                        }
                        s += PASSO;
                        if s > MARCHA_MAXIMA {
                            break None;
                        }
                    };
                    // ⚠️ Só onde a borda está LONGE da tinta (`> 1` pixel, a escada de uma tampa): onde
                    // ela já encosta à tinta (a arte na orla da imagem) fica o ponto dela, a lei de
                    // antes e os seus gates (MEDIDO: encostar também ali regredia o risquinho).
                    let encosto = encosto.filter(|_| s >= ENCOSTO_MINIMO);
                    let (encosto, tinta_em) = (encosto.map(|e| e.0), encosto.map(|e| e.1));
                    if let Some(mut ponto) = encosto.and_then(onde).or_else(|| onde(p)) {
                        if let Some(q) = tinta_em {
                            let c = [
                                dentro[0].mul_add(COR_DENTRO, q[0]),
                                dentro[1].mul_add(COR_DENTRO, q[1]),
                            ];
                            ponto.cor = if tinta.tem(c) { c } else { q };
                        }
                        out.push(ponto);
                    }
                }
            }
            out
        })
        .collect()
}

#[cfg(test)]
#[path = "skin_image_arte_tests.rs"]
mod tests;
