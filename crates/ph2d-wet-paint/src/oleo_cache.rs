//! **O cache do campo de altura por DEPENDÊNCIA** (filho de [`super`], Óleo Real W3; medição em
//! `docs/Painter/49_realshader/ESTADO_MISSAO_W3.md`, linha «cache do campo»). O [`CampoAltura::de_com`] é ~metade
//! de uma recomposição a 4K e lê do cartão só o que a [`ChaveCampo`] guarda; o resto (Specular, a fenda, o material
//! sem Subsurface, a Transparency) é do sombreado. A [`CacheCampo`] é o campo da GRADE INTEIRA montado faixa a
//! faixa: cada recomposição de largura inteira copia as suas linhas pedidas (os valores de uma janela nessas linhas
//! são os da janela inteira — a margem cobre o desfoque; o mesmo invariante do gate
//! `a_tela_depois_da_varredura_e_a_recomposicao_inteira_ao_byte`), e a seguinte com a mesma chave e as mesmas
//! grades ([`crate::painter::Engine::revisao`]) usa-o sem o refazer.

use super::{Afinacao, CampoAltura, ChaveLuz, Grid, MaterialCelula, OleoReal, alcance};

/// O que o [`CampoAltura::de_com`] lê do cartão e da pilha (além das grades): com outra chave o campo refaz-se.
#[derive(Clone, Debug, PartialEq)]
pub struct ChaveCampo {
    profundidade: f64,
    af: Afinacao,
    /// O albedo do rebote só existe com sombra.
    sombra_ligada: bool,
    /// Com Subsurface (no cartão, ou material em alguma camada visível): a irradiância lê a luz, a fenda, o ambiente
    /// e o raio — `None` sem ela.
    sss: Option<(ChaveLuz, [f64; 4])>,
    ratio: usize,
    visiveis: Vec<bool>,
    /// W3b: o dente do papel (a altura fina lê-o com o Relevo do papel de cada célula — este vem com a revisão).
    papel: f64,
    /// D1 W3b: o raio do Smooth Edges da tela (a silhueta do campo, `oleo_real::amacia_a_silhueta`).
    suave_px: f64,
}

impl ChaveCampo {
    #[must_use]
    pub fn de(o: &OleoReal, camadas: &[(&Grid, bool)], ratio: usize) -> Self {
        let com_sss = o.material.subsuperficie > 0.0
            || camadas.iter().any(|&(g, v)| v && !g.material.is_empty());
        Self {
            profundidade: o.profundidade,
            af: o.af,
            sombra_ligada: o.sombra > 0.0,
            sss: com_sss.then(|| {
                (
                    ChaveLuz::de(o),
                    [
                        o.fenda,
                        o.forca_ambiente,
                        o.material.subsuperficie,
                        o.material.raio_subsuperficie,
                    ],
                )
            }),
            ratio,
            visiveis: camadas.iter().map(|&(_, v)| v).collect(),
            papel: o.dente_papel,
            suave_px: o.suave_px,
        }
    }
}

/// O campo da grade inteira e as linhas dele já montadas, para UMA chave e UMA revisão das grades.
pub struct CacheCampo {
    chave: ChaveCampo,
    revisao: u64,
    campo: CampoAltura,
    /// `validas[y]`: a linha `y` (`1..=gh`) já veio de uma recomposição com esta chave.
    validas: Vec<bool>,
}

impl CacheCampo {
    /// O campo guardado, se a chave e a revisão são as de agora e as linhas que a região `(cy0, cy1)` lê (a região
    /// e o [`alcance`] de `o` dos dois lados) já estão montadas.
    #[must_use]
    pub fn serve<'a>(
        cache: Option<&'a Self>,
        chave: &ChaveCampo,
        revisao: u64,
        o: &OleoReal,
        (cy0, cy1): (usize, usize),
    ) -> Option<&'a CampoAltura> {
        let c = cache?;
        if c.revisao != revisao || c.chave != *chave {
            return None;
        }
        let a = alcance(o);
        let (y0, y1) = (cy0.saturating_sub(a).max(1), (cy1 + a).min(c.campo.h));
        c.validas[y0..=y1].iter().all(|&v| v).then_some(&c.campo)
    }

    /// Guarda as linhas `cy0..=cy1` do campo `de` (calculado para a região `(cx0, cy0, cx1, cy1)` da grade
    /// `gw × gh`) — só de uma região de LARGURA inteira (a varredura do cartão, a recomposição inteira). Outra
    /// chave ou revisão recomeça o cache.
    pub fn guarda(
        cache: &mut Option<Self>,
        chave: ChaveCampo,
        revisao: u64,
        de: &CampoAltura,
        (gw, gh): (usize, usize),
        (cx0, cy0, cx1, cy1): (usize, usize, usize, usize),
    ) {
        if cx0 != 1 || cx1 != gw || de.x0 != 1 || de.x0 + de.w != gw + 1 {
            return;
        }
        let reusa = cache
            .as_ref()
            .is_some_and(|c| c.revisao == revisao && c.chave == chave && c.campo.w == gw);
        if !reusa {
            *cache = Some(Self {
                chave,
                revisao,
                campo: CampoAltura {
                    x0: 1,
                    y0: 1,
                    w: gw,
                    h: gh,
                    v: vec![0.0; gw * gh],
                    detalhe: vec![0.0; gw * gh],
                    sombra: vec![0.0; gw * gh],
                    material: Vec::new(),
                    sss: Vec::new(),
                    luz_ceu: Vec::new(),
                    luz_ceu_janela: (0, 0, 0, 0),
                    luz_ceu_chave: None,
                    albedo: Vec::new(),
                    borda: Vec::new(),
                },
                validas: vec![false; gh + 1],
            });
        }
        let Some(c) = cache.as_mut() else {
            return;
        };
        let n = gw * gh;
        let f = &mut c.campo;
        for y in cy0..=cy1 {
            let (d, s) = ((y - 1) * gw, (y - de.y0) * gw);
            let linha = |v: &mut [f32], o: &[f32]| v[d..d + gw].copy_from_slice(&o[s..s + gw]);
            linha(&mut f.v, &de.v);
            linha(&mut f.detalhe, &de.detalhe);
            linha(&mut f.sombra, &de.sombra);
            copia(
                &mut f.material,
                &de.material,
                MaterialCelula::NEUTRO,
                n,
                (d, s, gw),
            );
            copia(&mut f.sss, &de.sss, [0.0; 3], n, (d, s, gw));
            copia(&mut f.albedo, &de.albedo, [1.0; 3], n, (d, s, gw));
            copia(&mut f.borda, &de.borda, 1.0, n, (d, s, gw));
            c.validas[y] = true;
        }
    }
}

/// Uma linha de um plano OPCIONAL (vazio = `neutro` em toda a parte): alocado no primeiro que o traz.
fn copia<T: Copy>(
    dst: &mut Vec<T>,
    src: &[T],
    neutro: T,
    n: usize,
    (d, s, w): (usize, usize, usize),
) {
    if src.is_empty() {
        if !dst.is_empty() {
            dst[d..d + w].fill(neutro);
        }
        return;
    }
    if dst.is_empty() {
        *dst = vec![neutro; n];
    }
    dst[d..d + w].copy_from_slice(&src[s..s + w]);
}
