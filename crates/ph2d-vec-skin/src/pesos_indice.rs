//! ⭐⭐⭐ **ONDE ESTÁ O TRIÂNGULO** — a grelha de baldes que tira a consulta ao campo de
//! `O(triângulos)`.
//!
//! ⚠️ Irmão da [`super`] por tecto de LOC (ela bateu nos `700` exactos ao receber isto), e o corte
//! é por RESPONSABILIDADE: ali vive *o que é um peso*, aqui vive *onde ele está*.

/// ⭐⭐⭐ **O ÍNDICE DA MALHA — a grelha de baldes que tira a consulta de `O(triângulos)`.**
///
/// # ⛔⛔⛔ Porque ele existe, com o número
///
/// A [`CampoDoDominio::linha`] procurava o triângulo que contém um ponto **varrendo todos**
/// (`for t in &m.tris`), e a malha da barra da cena tem **878**. Medido em `--release` sobre a
/// peça do dono:
///
/// | | µs por chamada |
/// |---|---:|
/// | a varredura (`campo.linha`) | **`0,6853`** |
/// | a LEI que ela alimenta (`weights_corrected` + `blend`) | `0,0341` |
///
/// ⇒ **`95 %` do custo de amostrar um ponto era a busca, e `20×` a lei que ela serve.** Isto não
/// é uma optimização prematura ([`project-memory`](../../../project-memory/project_m5_perf_validated.md)):
/// é o §0.0 — *nunca deixe o caminho lento definir o tecto do rápido*. A busca é paga **por
/// amostra de curva, por forma, por quadro**.
///
/// ⚠️ **Ele mora FORA da [`CampoDoDominio`] de propósito.** Aquela struct é **gravada** (ela viaja
/// nos bytes do bind) e deriva `Clone`/`Debug`/`PartialEq` + `serde`; um cache lá dentro obrigaria
/// a escrever as cinco implementações à mão para esconder um campo derivado. ⇒ o mesmo arranjo que
/// o [`crate::pesos_suave::CampoSuave`] já tem: **derivado UMA vez por forma, pelo chamador**.
#[derive(Clone, Debug)]
pub struct IndiceDoCampo {
    /// O canto inferior-esquerdo da caixa da malha, em coordenadas de MALHA.
    origem: [f64; 2],
    /// O recíproco do lado da célula, por eixo — multiplicar é mais barato que dividir, e isto
    /// corre por amostra.
    inv_passo: [f64; 2],
    /// Quantas células por eixo.
    lado: [usize; 2],
    /// Os triângulos que tocam cada célula, em ordem de linha (`y * lado[0] + x`).
    baldes: Vec<Vec<u32>>,
}

impl IndiceDoCampo {
    /// Constrói o índice de uma malha. `None` quando ela não tem triângulos ou tem área nula —
    /// ⛔ nunca uma grelha de uma célula, que seria a varredura com um passo a mais.
    #[must_use]
    pub fn novo(malha: &ph2d_poly2d::Mesh2d) -> Option<Self> {
        if malha.tris.is_empty() || malha.rest.is_empty() {
            return None;
        }
        let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
        for v in &malha.rest {
            lo = [lo[0].min(v[0]), lo[1].min(v[1])];
            hi = [hi[0].max(v[0]), hi[1].max(v[1])];
        }
        let (w, h) = (hi[0] - lo[0], hi[1] - lo[1]);
        if !(w > 0.0 && h > 0.0) {
            return None;
        }
        // ⚠️ **O lado da grelha sai da CONTAGEM de triângulos, não de um número escolhido:**
        // `√n` por eixo dá `~1` triângulo por célula, que é o ponto em que a busca deixa de
        // depender de `n`. O tecto de `128` é memória (`128² = 16 384` baldes).
        #[expect(clippy::cast_precision_loss, reason = "contagem de triângulos")]
        let alvo = (malha.tris.len() as f64).sqrt().ceil();
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "alvo >= 1, clampado"
        )]
        let n = (alvo as usize).clamp(1, 128);
        let lado = [n, n];
        #[expect(clippy::cast_precision_loss, reason = "n <= 128")]
        let inv_passo = [lado[0] as f64 / w, lado[1] as f64 / h];
        let mut baldes = vec![Vec::new(); lado[0] * lado[1]];
        for (i, t) in malha.tris.iter().enumerate() {
            let (mut tl, mut th) = ([f64::MAX; 2], [f64::MIN; 2]);
            for &v in t {
                let q = malha.rest[v as usize];
                tl = [tl[0].min(q[0]), tl[1].min(q[1])];
                th = [th[0].max(q[0]), th[1].max(q[1])];
            }
            // ⛔ Por CAIXA do triângulo e não pelo centro: o triângulo que contém o ponto tem de
            // estar no balde dele, sempre. *Um índice que pode falhar precisa de uma varredura de
            // reserva, e aí não há índice nenhum.*
            let (x0, y0) = (
                celula(tl[0], lo[0], inv_passo[0], lado[0]),
                celula(tl[1], lo[1], inv_passo[1], lado[1]),
            );
            let (x1, y1) = (
                celula(th[0], lo[0], inv_passo[0], lado[0]),
                celula(th[1], lo[1], inv_passo[1], lado[1]),
            );
            for y in y0..=y1 {
                for x in x0..=x1 {
                    #[expect(clippy::cast_possible_truncation, reason = "índice de triângulo")]
                    baldes[y * lado[0] + x].push(i as u32);
                }
            }
        }
        Some(Self {
            origem: lo,
            inv_passo,
            lado,
            baldes,
        })
    }

    /// O maior balde da grelha — a régua de que ela DIVIDE. ⚠️ Sem ela um índice de uma célula só
    /// passa em todo gate de igualdade e não indexa nada.
    #[must_use]
    pub fn maior_balde(&self) -> usize {
        self.baldes.iter().map(Vec::len).max().unwrap_or(0)
    }

    /// Os triângulos candidatos para `p` (em coordenadas de MALHA). Vazio ⇒ nenhum o contém.
    #[must_use]
    pub fn candidatos(&self, p: [f64; 2]) -> &[u32] {
        let x = celula(p[0], self.origem[0], self.inv_passo[0], self.lado[0]);
        let y = celula(p[1], self.origem[1], self.inv_passo[1], self.lado[1]);
        // ⚠️ Um ponto FORA da caixa cai na célula da borda, e é isso que se quer: ali os
        // candidatos são os triângulos da borda, e nenhum o contém — a resposta é `None`, a
        // mesma da varredura, sem a varrer.
        &self.baldes[y * self.lado[0] + x]
    }
}

/// A célula de uma coordenada, **clampada** à grelha.
fn celula(v: f64, origem: f64, inv_passo: f64, lado: usize) -> usize {
    let t = (v - origem) * inv_passo;
    if !t.is_finite() || t < 0.0 {
        return 0;
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "t >= 0 e finito; o clamp fecha o topo"
    )]
    let i = t as usize;
    i.min(lado - 1)
}
