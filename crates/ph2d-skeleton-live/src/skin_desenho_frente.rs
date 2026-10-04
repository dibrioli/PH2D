//! ⭐⭐⭐ **A PARTE DA FRENTE TAPA O QUE É ABERTO NA DE TRÁS** (A2, 2026-10-04) — numa dobra forte
//! as riscas de um *Hatch* (subcontornos ABERTOS) da parte de trás pintavam por cima da da frente.
//!
//! A ordem é a da imagem presa ([`crate::skin_image_fecho::ordena_pelo_osso`]): cada triângulo da
//! malha do campo tem a [`crate::skin_image_fecho::chave_de_osso`] (média dos três vértices; o osso
//! mais FUNDO na corrente) e o de chave maior fica por cima. Uma amostra de um contorno aberto mora, em REPOUSO, num triângulo; posada, está TAPADA
//! quando cai dentro de um triângulo posado de chave maior que não toca o dela. O contorno parte-se
//! no REPOUSO (de Casteljau, exacto) e o bake só percorre os pedaços à vista. Os FECHADOS não se
//! tocam: o contacto deles é a união ([`super::uniao_dos_fechados`]).

use ph2d_skeleton::{Correccao, Skin};
use ph2d_vec_scene::{VecPath, VecVertex};
use ph2d_vec_skin::pesos::{CampoDoDominio, IndiceDoCampo};

/// Amostras por segmento de um contorno aberto — onde o estado muda entre duas, a fronteira é
/// bissectada ([`BISSECCOES`]).
const AMOSTRAS: usize = 32;
/// Passos da bissecção da fronteira: `1/32 · 2⁻¹²` do segmento.
const BISSECCOES: usize = 12;

/// A malha do campo POSADA neste quadro, com a chave de cada triângulo e uma grelha por caixa.
struct Posada<'a> {
    campo: &'a CampoDoDominio,
    indice: Option<&'a IndiceDoCampo>,
    pele: &'a Skin,
    correcoes: &'a [Correccao],
    rigido: bool,
    pos: Vec<[f64; 2]>,
    chave_tri: Vec<f64>,
    /// O triângulo posado está do AVESSO (a dobra virou-o): o lado de baixo de um papel dobrado.
    virado: Vec<bool>,
    grelha: Grelha,
}

/// Baldes de triângulos posados por célula — a consulta de um ponto só vê os do balde dele.
struct Grelha {
    origem: [f64; 2],
    lado: f64,
    dim: [usize; 2],
    baldes: Vec<Vec<u32>>,
}

/// Coordenadas baricêntricas `(u, v)` de `p` em `abc` (sinal livre); `None` num triângulo nulo.
fn bari(p: [f64; 2], a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> Option<(f64, f64)> {
    let den = (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]);
    if den.abs() < 1e-12 {
        return None;
    }
    let u = ((p[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (p[1] - a[1])) / den;
    let v = ((b[0] - a[0]) * (p[1] - a[1]) - (p[0] - a[0]) * (b[1] - a[1])) / den;
    Some((u, v))
}

fn dentro(uv: Option<(f64, f64)>) -> bool {
    uv.is_some_and(|(u, v)| u >= -1e-9 && v >= -1e-9 && u + v <= 1.0 + 1e-9)
}

impl Grelha {
    fn nova(pos: &[[f64; 2]], tris: &[[u32; 3]]) -> Self {
        let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
        for p in pos {
            for k in 0..2 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
        #[expect(clippy::cast_precision_loss, reason = "contagem de triângulos")]
        let n = tris.len().max(1) as f64;
        let area = ((hi[0] - lo[0]) * (hi[1] - lo[1])).max(1e-12);
        let lado = (area / n).sqrt() * 2.0;
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "dimensão da grelha, finita e positiva"
        )]
        let dim = [0, 1].map(|k| (((hi[k] - lo[k]) / lado).floor() as usize + 1).min(4096));
        let mut g = Self {
            origem: lo,
            lado,
            dim,
            baldes: vec![Vec::new(); dim[0] * dim[1]],
        };
        for (i, t) in tris.iter().enumerate() {
            let ps = t.map(|v| pos[v as usize]);
            let c0 = g.celula([ps[0][0].min(ps[1][0]).min(ps[2][0]), ps[0][1].min(ps[1][1]).min(ps[2][1])]);
            let c1 = g.celula([ps[0][0].max(ps[1][0]).max(ps[2][0]), ps[0][1].max(ps[1][1]).max(ps[2][1])]);
            for y in c0[1]..=c1[1] {
                for x in c0[0]..=c1[0] {
                    #[expect(clippy::cast_possible_truncation, reason = "índice de triângulo u32")]
                    g.baldes[y * dim[0] + x].push(i as u32);
                }
            }
        }
        g
    }

    fn celula(&self, p: [f64; 2]) -> [usize; 2] {
        [0, 1].map(|k| {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "célula da grelha, presa ao intervalo"
            )]
            let c = ((p[k] - self.origem[k]) / self.lado).floor().max(0.0) as usize;
            c.min(self.dim[k] - 1)
        })
    }

    fn balde(&self, p: [f64; 2]) -> &[u32] {
        let c = self.celula(p);
        &self.baldes[c[1] * self.dim[0] + c[0]]
    }
}

impl<'a> Posada<'a> {
    fn nova(
        campo: &'a CampoDoDominio,
        indice: Option<&'a IndiceDoCampo>,
        pele: &'a Skin,
        correcoes: &'a [Correccao],
        rigido: bool,
        prof: &[f64],
    ) -> Option<Self> {
        if campo.ossos() < 2 || !campo.valida() {
            return None;
        }
        let mut w = pele.scratch();
        let mut pos = Vec::with_capacity(campo.malha.rest.len());
        let mut chave_v = Vec::with_capacity(campo.malha.rest.len());
        for i in 0..campo.malha.rest.len() {
            let p = campo.local_do_vertice(i)?;
            let linha = campo.linha_do_vertice(i)?;
            pele.weights_corrected(p, Some(linha), &mut w, correcoes);
            pos.push(if rigido { pele.blend(p, &w) } else { pele.blend_linear(p, &w) });
            chave_v.push(crate::skin_image_fecho::chave_de_osso(linha, prof));
        }
        let tris = &campo.malha.tris;
        let chave_tri = tris
            .iter()
            .map(|t| t.iter().map(|&v| chave_v[v as usize]).sum::<f64>() / 3.0)
            .collect();
        let area = |q: [[f64; 2]; 3]| {
            (q[1][0] - q[0][0]) * (q[2][1] - q[0][1]) - (q[2][0] - q[0][0]) * (q[1][1] - q[0][1])
        };
        let virado = tris
            .iter()
            .map(|t| {
                let (r, p) = (t.map(|v| campo.malha.rest[v as usize]), t.map(|v| pos[v as usize]));
                area(r) * area(p) < 0.0
            })
            .collect();
        let grelha = Grelha::nova(&pos, tris);
        Some(Self {
            campo,
            indice,
            pele,
            correcoes,
            rigido,
            pos,
            chave_tri,
            virado,
            grelha,
        })
    }

    /// O ponto `p` (LOCAL, em repouso) está tapado por um triângulo de chave maior? `false` fora
    /// da malha.
    fn tapado(&self, p: [f64; 2]) -> bool {
        let Some((dono, q)) = self.onde(p) else {
            return false;
        };
        if self.virado[dono] {
            return true;
        }
        let m = &self.campo.malha;
        let t = m.tris[dono];
        let minha = self.chave_tri[dono];
        self.grelha.balde(q).iter().any(|&k| {
            let o = m.tris[k as usize];
            self.chave_tri[k as usize] > minha
                && !o.iter().any(|v| t.contains(v))
                && dentro(bari(
                    q,
                    self.pos[o[0] as usize],
                    self.pos[o[1] as usize],
                    self.pos[o[2] as usize],
                ))
        })
    }

    /// O ponto `p` posado (a lei do bake: o campo na posição dele), sem a malha a pele derivada.
    fn posado(&self, p: [f64; 2]) -> [f64; 2] {
        self.onde(p).map_or_else(
            || {
                let mut w = self.pele.scratch();
                self.pele.weights_corrected(p, None, &mut w, self.correcoes);
                if self.rigido {
                    self.pele.blend(p, &w)
                } else {
                    self.pele.blend_linear(p, &w)
                }
            },
            |(_, q)| q,
        )
    }

    /// O triângulo onde `p` (LOCAL, em repouso) mora, e `p` posado. `None` fora da malha.
    fn onde(&self, p: [f64; 2]) -> Option<(usize, [f64; 2])> {
        let m = &self.campo.malha;
        let pm = self.campo.para_malha_pub(p);
        let todos: Vec<u32>;
        let cand: &[u32] = if let Some(i) = self.indice {
            i.candidatos(pm)
        } else {
            #[expect(clippy::cast_possible_truncation, reason = "índice de triângulo u32")]
            {
                todos = (0..m.tris.len() as u32).collect();
            }
            &todos
        };
        let n = self.campo.ossos();
        let (dono, (u, v)) = cand.iter().find_map(|&k| {
            let t = m.tris[k as usize];
            let uv = bari(pm, m.rest[t[0] as usize], m.rest[t[1] as usize], m.rest[t[2] as usize]);
            dentro(uv).then(|| (k as usize, uv.unwrap_or_default()))
        })?;
        let t = m.tris[dono];
        let linha: Vec<f64> = (0..n)
            .map(|j| {
                let r = |v: u32| self.campo.pesos[v as usize * n + j];
                (1.0 - u - v) * r(t[0]) + u * r(t[1]) + v * r(t[2])
            })
            .collect();
        let mut w = self.pele.scratch();
        self.pele
            .weights_corrected(p, Some(&linha), &mut w, self.correcoes);
        let q = if self.rigido {
            self.pele.blend(p, &w)
        } else {
            self.pele.blend_linear(p, &w)
        };
        Some((dono, q))
    }

    /// O comprimento POSADO do contorno aberto `vs` entre `u0` e `u1` (polilinha de 16 troços).
    fn comprimento(&self, vs: &[VecVertex], u0: f64, u1: f64) -> f64 {
        let ponto = |u: f64| {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "parâmetro no intervalo do contorno"
            )]
            let k = (u.floor() as usize).min(vs.len() - 2);
            #[expect(clippy::cast_precision_loss, reason = "índice de segmento")]
            self.posado(avalia(&cubica(vs, k), u - k as f64))
        };
        let pts: Vec<[f64; 2]> = (0..=16)
            .map(|i| ponto((u1 - u0).mul_add(f64::from(i) / 16.0, u0)))
            .collect();
        pts.windows(2)
            .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
            .sum()
    }
}

/// A cúbica `k → k+1` de um contorno aberto, em `t`.
fn cubica(vs: &[VecVertex], k: usize) -> [[f64; 2]; 4] {
    [
        vs[k].anchor,
        vs[k].out_handle,
        vs[k + 1].in_handle,
        vs[k + 1].anchor,
    ]
}

fn lerp(a: [f64; 2], b: [f64; 2], t: f64) -> [f64; 2] {
    [(b[0] - a[0]).mul_add(t, a[0]), (b[1] - a[1]).mul_add(t, a[1])]
}

fn avalia(c: &[[f64; 2]; 4], t: f64) -> [f64; 2] {
    let (ab, bc, cd) = (lerp(c[0], c[1], t), lerp(c[1], c[2], t), lerp(c[2], c[3], t));
    lerp(lerp(ab, bc, t), lerp(bc, cd, t), t)
}

/// O pedaço `[t0, t1]` da cúbica (de Casteljau, exacto).
fn pedaco(c: &[[f64; 2]; 4], t0: f64, t1: f64) -> [[f64; 2]; 4] {
    let parte = |c: &[[f64; 2]; 4], t: f64| -> ([[f64; 2]; 4], [[f64; 2]; 4]) {
        let (ab, bc, cd) = (lerp(c[0], c[1], t), lerp(c[1], c[2], t), lerp(c[2], c[3], t));
        let (abc, bcd) = (lerp(ab, bc, t), lerp(bc, cd, t));
        let m = lerp(abc, bcd, t);
        ([c[0], ab, abc, m], [m, bcd, cd, c[3]])
    };
    let direita = parte(c, t0).1;
    if t1 >= 1.0 {
        return direita;
    }
    let s = if t0 < 1.0 { (t1 - t0) / (1.0 - t0) } else { 1.0 };
    parte(&direita, s).0
}

/// Os intervalos À VISTA de um contorno aberto, no parâmetro `k + t` (`0 ..= segmentos`).
/// `None` quando nada está tapado.
fn a_vista(vs: &[VecVertex], f: &Posada<'_>) -> Option<Vec<(f64, f64)>> {
    let segs = vs.len().saturating_sub(1);
    let mut vis: Vec<(f64, f64)> = Vec::new();
    let mut aberto: Option<f64> = None;
    let mut algum = false;
    for k in 0..segs {
        let c = cubica(vs, k);
        let mut antes = f.tapado(c[0]);
        if k == 0 {
            algum = antes;
            if !antes {
                aberto = Some(0.0);
            }
        }
        for i in 1..=AMOSTRAS {
            #[expect(clippy::cast_precision_loss, reason = "amostra")]
            let t = i as f64 / AMOSTRAS as f64;
            let agora = f.tapado(avalia(&c, t));
            if agora != antes {
                #[expect(clippy::cast_precision_loss, reason = "amostra")]
                let (mut a, mut b) = ((i - 1) as f64 / AMOSTRAS as f64, t);
                for _ in 0..BISSECCOES {
                    let m = 0.5 * (a + b);
                    if f.tapado(avalia(&c, m)) == antes {
                        a = m;
                    } else {
                        b = m;
                    }
                }
                #[expect(clippy::cast_precision_loss, reason = "índice de segmento")]
                let u = k as f64 + 0.5 * (a + b);
                if agora {
                    vis.extend(aberto.take().map(|s| (s, u)));
                } else {
                    aberto = Some(u);
                }
            }
            algum |= agora;
            antes = agora;
        }
    }
    if !algum {
        return None;
    }
    #[expect(clippy::cast_precision_loss, reason = "índice de segmento")]
    vis.extend(aberto.map(|s| (s, segs as f64)));
    Some(vis)
}

/// De onde vem um nó do recorte: um nó da fonte (índice no contorno) ou um ponto novo `(k, t)`.
#[derive(Clone, Copy)]
enum Origem {
    Fonte(usize),
    Novo(usize, f64),
}

/// O contorno aberto `vs` entre `u0` e `u1` (parâmetro `k + t`), com a origem de cada nó.
fn recorta(vs: &[VecVertex], u0: f64, u1: f64) -> Vec<(VecVertex, Origem)> {
    let segs = vs.len() - 1;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "parâmetro no intervalo do contorno"
    )]
    let seg = |u: f64| (u.floor() as usize).min(segs - 1);
    let (k0, k1) = (seg(u0), seg(u1));
    #[expect(clippy::cast_precision_loss, reason = "índice de segmento")]
    let (t0, t1) = (u0 - k0 as f64, u1 - k1 as f64);
    let mut out: Vec<(VecVertex, Origem)> = Vec::new();
    for k in k0..=k1 {
        let (a, b) = (if k == k0 { t0 } else { 0.0 }, if k == k1 { t1 } else { 1.0 });
        let c = pedaco(&cubica(vs, k), a, b);
        match out.last_mut() {
            Some((v, _)) => v.out_handle = c[1],
            None => {
                let mut v = VecVertex::corner(c[0]);
                v.out_handle = c[1];
                out.push((v, Origem::Novo(k, a)));
            }
        }
        let (mut fim, origem) = if b >= 1.0 {
            (vs[k + 1], Origem::Fonte(k + 1))
        } else {
            (VecVertex::corner(c[3]), Origem::Novo(k, b))
        };
        fim.anchor = c[3];
        fim.in_handle = c[2];
        out.push((fim, origem));
    }
    out
}

/// ⭐⭐⭐ **A fonte só com o que se VÊ**, e a tabela dela — os contornos ABERTOS cortados, os fechados
/// como estão. `None` quando nada está tapado (o caso comum, sem alocar a fonte). A tabela tem três
/// linhas por nó, como a guardada: a de um nó da fonte é COPIADA (o fechado sai ao bit), a de um nó
/// novo é a do campo na âncora e, fora dele, a mistura das linhas do segmento em `t` (a lei da F28).
#[must_use]
pub(super) fn so_o_que_se_ve(
    fonte: &VecPath,
    tabela: &[f64],
    campo: (&CampoDoDominio, Option<&IndiceDoCampo>),
    pele: (&Skin, &[Correccao], bool),
    prof: &[f64],
) -> Option<(VecPath, Vec<f64>)> {
    recorta_a_fonte(fonte, tabela, campo, pele, prof, false)
}

/// ⭐⭐ **O TRAÇO à vista dos contornos FECHADOS** (A6) — só eles, cada um em trechos abertos onde a
/// dobra o tapa (inteiro e fechado onde não tapa), para a camada de traço de uma forma sem união.
/// `None` quando nenhum fechado está tapado.
#[must_use]
pub(super) fn tracos_a_vista(
    fonte: &VecPath,
    tabela: &[f64],
    campo: (&CampoDoDominio, Option<&IndiceDoCampo>),
    pele: (&Skin, &[Correccao], bool),
    prof: &[f64],
) -> Option<(VecPath, Vec<f64>)> {
    recorta_a_fonte(fonte, tabela, campo, pele, prof, true)
}

/// A porta das duas: `fechados = false` corta os abertos e deixa os fechados; `true` corta os
/// fechados (uma volta aberta com o 1.º nó repetido no fim, e o trecho que passa pela emenda cosido
/// de volta) e larga os abertos.
fn recorta_a_fonte(
    fonte: &VecPath,
    tabela: &[f64],
    (campo, indice): (&CampoDoDominio, Option<&IndiceDoCampo>),
    (pele, correcoes, rigido): (&Skin, &[Correccao], bool),
    prof: &[f64],
    fechados: bool,
) -> Option<(VecPath, Vec<f64>)> {
    let contornos: Vec<(&[VecVertex], bool)> = (0..fonte.contour_count())
        .filter_map(|c| fonte.contour(c))
        .collect();
    let julga = |v: &[VecVertex], fechado: bool| fechado == fechados && v.len() > 1;
    if !contornos.iter().any(|(_, f)| *f) || !contornos.iter().any(|(v, f)| julga(v, *f)) {
        return None;
    }
    let f = Posada::nova(campo, indice, pele, correcoes, rigido, prof)?;
    // ⭐ Um pedaço CORTADO mais curto que a largura do próprio traço é um borrão, não uma risca
    // (FOTOGRAFADO a `110°`: tiques soltos junto às juntas) — sai.
    let largura = fonte.stroke.as_ref().map_or(0.0, |s| s.width);
    let volta = |v: &[VecVertex], fechado: bool| -> Vec<VecVertex> {
        let mut w = v.to_vec();
        if fechado {
            w.push(v[0]);
        }
        w
    };
    let cortes: Vec<Option<Vec<(f64, f64)>>> = contornos
        .iter()
        .map(|(v, fechado)| {
            if !julga(v, *fechado) {
                return None;
            }
            let w = volta(v, *fechado);
            let vis = a_vista(&w, &f)?;
            #[expect(clippy::cast_precision_loss, reason = "índice de segmento")]
            let fim = (w.len() - 1) as f64;
            Some(
                vis.into_iter()
                    .filter(|&(a, b)| (a <= 0.0 && b >= fim) || f.comprimento(&w, a, b) >= largura)
                    .collect(),
            )
        })
        .collect();
    if cortes.iter().all(Option::is_none) {
        return None;
    }
    let total: usize = contornos.iter().map(|(v, _)| v.len()).sum();
    let n = tabela.len().checked_div(3 * total).unwrap_or(0);
    let usa = n > 0 && tabela.len() == 3 * total * n;
    let linha_de = |g: usize| &tabela[3 * g * n..(3 * g + 1) * n];
    let mut nova_tabela: Vec<f64> = Vec::new();
    let mut novos: Vec<ph2d_vec_scene::Contour> = Vec::new();
    let mut base = 0;
    for ((vs, fechado), corte) in contornos.iter().zip(&cortes) {
        let m = vs.len();
        let pecas: Vec<Vec<(VecVertex, Origem)>> = match corte {
            None if fechados && !*fechado => Vec::new(),
            None => vec![vs.iter().enumerate().map(|(i, v)| (*v, Origem::Fonte(i))).collect()],
            Some(vis) => {
                let w = volta(vs, *fechado);
                #[expect(clippy::cast_precision_loss, reason = "índice de segmento")]
                let fim = (w.len() - 1) as f64;
                let mut p: Vec<Vec<(VecVertex, Origem)>> = vis
                    .iter()
                    .filter(|(a, b)| b > a)
                    .map(|&(a, b)| recorta(&w, a, b))
                    .collect();
                // A emenda: um trecho que acaba no fim da volta e outro que começa no início são o
                // MESMO trecho de um contorno fechado.
                let cose = *fechado
                    && p.len() > 1
                    && vis.first().is_some_and(|v| v.0 <= 0.0)
                    && vis.last().is_some_and(|v| v.1 >= fim);
                if cose {
                    let primeiro = p.remove(0);
                    let ultimo = p.last_mut().expect("mais de um trecho");
                    if let (Some(fim_v), Some(ini)) = (ultimo.last_mut(), primeiro.first()) {
                        fim_v.0.out_handle = ini.0.out_handle;
                    }
                    ultimo.extend(primeiro.into_iter().skip(1));
                }
                p
            }
        };
        for peca in pecas {
            if usa {
                for (v, o) in &peca {
                    let linha: Vec<f64> = match *o {
                        Origem::Fonte(i) => linha_de(base + i % m).to_vec(),
                        Origem::Novo(k, t) => campo.linha_com(v.anchor, indice).unwrap_or_else(|| {
                            linha_de(base + k % m)
                                .iter()
                                .zip(linha_de(base + (k + 1) % m))
                                .map(|(x, y)| (y - x).mul_add(t, *x))
                                .collect()
                        }),
                    };
                    for _ in 0..3 {
                        nova_tabela.extend_from_slice(&linha);
                    }
                }
            }
            novos.push(ph2d_vec_scene::Contour {
                verts: peca.into_iter().map(|(v, _)| v).collect(),
                closed: corte.is_none() && *fechado,
            });
        }
        base += m;
    }
    let mut saida = fonte.clone();
    let mut it = novos.into_iter();
    let primeiro = it.next()?;
    saida.verts = primeiro.verts;
    saida.closed = primeiro.closed;
    saida.subpaths = it.collect();
    Some((saida, nova_tabela))
}

#[cfg(test)]
#[path = "skin_desenho_frente_tests.rs"]
pub(super) mod tests;
