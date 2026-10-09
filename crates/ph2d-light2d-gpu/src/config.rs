//! A configuração do solver: a grade ANCORADA NO MUNDO e as escolhas que a rodada de medição fixou.

/// A grade da luz. O texel `(i, j)` cobre `origem + [i, i+1)·texel × [j, j+1)·texel` em metros do
/// mundo; a sonda da c0 é o centro dele. A linha `j` cresce com o `y` do mundo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grade {
    pub origem_m: [f64; 2],
    pub texel_m: f64,
    pub largura: u32,
    pub altura: u32,
}

impl Grade {
    /// O centro do texel `(i, j)`, no mundo.
    #[must_use]
    pub fn centro(&self, i: u32, j: u32) -> [f64; 2] {
        [
            self.origem_m[0] + (f64::from(i) + 0.5) * self.texel_m,
            self.origem_m[1] + (f64::from(j) + 0.5) * self.texel_m,
        ]
    }

    /// Um ponto do mundo em coordenadas de texel (o centro do texel `(i, j)` é `(i+½, j+½)`).
    #[must_use]
    pub fn em_texels(&self, p: [f32; 2]) -> [f64; 2] {
        [
            (f64::from(p[0]) - self.origem_m[0]) / self.texel_m,
            (f64::from(p[1]) - self.origem_m[1]) / self.texel_m,
        ]
    }
}

/// Como a cascata `i` funde a de cima (plano §5, K6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fusao {
    /// RC de Sannikov sem correção: um raio por direção, a radiância de cima interpolada.
    Classica,
    /// *Bilinear fix*: quatro raios por direção, cada um até à sonda de cima que ele funde.
    CorrecaoBilinear,
    /// Um raio por direção, até UMA das quatro sondas de cima, escolhida num padrão que alterna
    /// por sonda e por subdireção (a nossa reconstrução do *nearest-interlaced*, medida, não portada).
    VizinhaIntercalada,
}

impl Fusao {
    /// O valor da constante `override FUSAO` do `c2_rc.wgsl`.
    pub(crate) fn codigo(self) -> f64 {
        match self {
            Self::Classica => 0.0,
            Self::CorrecaoBilinear => 1.0,
            Self::VizinhaIntercalada => 2.0,
        }
    }
}

/// Precisão de um caminho do solver.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Precisao {
    F16,
    F32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    pub grade: Grade,
    /// Comprimento do intervalo da c0, em texels; o da cascata `i` é `intervalo_c0 · 4^i`.
    pub intervalo_c0: f32,
    /// Quantos texels finos por célula do campo de distância (C1).
    pub fator_jfa: u32,
    /// O passo da marcha perto do conteúdo, em texels.
    pub passo_perto: f32,
    pub fusao: Fusao,
    /// A aritmética da marcha e da fusão. `F16` exige `SHADER_F16` no dispositivo.
    pub aritmetica: Precisao,
    /// O formato das cascatas: `Rgba16Float` ou `Rgba32Float`.
    pub armazenamento: Precisao,
}

/// Onde a cascata `i` começa, em múltiplos de `intervalo_c0`: `(4^i − 1)/3`.
fn inicio_relativo(i: u32) -> f64 {
    (4f64.powi(i32::try_from(i).unwrap_or(i32::MAX)) - 1.0) / 3.0
}

/// A menor contagem de cascatas cujo topo passa a diagonal da grade: todo raio sai da grade antes
/// de acabar, e o que escapa recebe o céu.
fn cascatas_para(l0: f64, largura: u32, altura: u32) -> u32 {
    let diagonal = f64::from(largura).hypot(f64::from(altura));
    let mut n = 1;
    while l0 * inicio_relativo(n) < diagonal {
        n += 1;
    }
    n
}

impl Config {
    /// A configuração padrão para `grade` — as escolhas da rodada de medição da W2 (a tabela está no
    /// handoff `docs/Luz2D/handoffs/HANDOFF_CONTINUACAO_W2_2026-10-08.md`, RTX 5060 Ti, 512²):
    /// - `CorrecaoBilinear`: custa 3,1× a clássica (K6 > 2×); a vizinha intercalada, medida como o
    ///   K6 manda, erra 5× mais (0,449 × 0,090) ⇒ aceita-se o custo;
    /// - `intervalo_c0` 1 — erro (penumbra · céu e vidro) e C2: ½ → 0,095 · 0,021 a 1,72 ms (+1
    ///   cascata); **1 → 0,090 · 0,013 a 1,45 ms**; 2 → 0,116 · 0,010 a 1,36 ms;
    /// - `passo_perto` 1: o passo ½ custa +40–75 % e só ajuda no Baixo (o governador da W5 escolhe
    ///   por degrau);
    /// - `fator_jfa` 2: o 4 empata na penumbra e perde no enxame denso; o 1 custa 2,3× na C1;
    /// - cascatas `F16`: as `F32` não mudam o erro (0,0900 × 0,0902) e gastam +53 % de memória;
    /// - aritmética `F32` (portátil): a `F16` mediu IGUAL neste desktop; ver [`Self::padrao_para`].
    #[must_use]
    pub fn padrao(grade: Grade) -> Self {
        Self {
            grade,
            intervalo_c0: 1.0,
            fator_jfa: 2,
            passo_perto: 1.0,
            fusao: Fusao::CorrecaoBilinear,
            aritmetica: Precisao::F32,
            armazenamento: Precisao::F16,
        }
    }

    /// O padrão com aritmética f16 onde o dispositivo tem `SHADER_F16` (o caminho f32 é o mesmo
    /// algoritmo; o solver RECUSA f16 sem a feature, nunca troca em silêncio).
    #[must_use]
    pub fn padrao_para(device: &wgpu::Device, grade: Grade) -> Self {
        let f16 = device.features().contains(wgpu::Features::SHADER_F16);
        Self {
            aritmetica: if f16 { Precisao::F16 } else { Precisao::F32 },
            ..Self::padrao(grade)
        }
    }

    /// A grade que o solver de facto usa: a pedida, estendida à direita e em cima até múltiplos de
    /// `2^(n−1)` (o arranjo direção-primeiro da cascata de topo precisa de blocos inteiros).
    #[must_use]
    pub fn grade_efetiva(&self) -> Grade {
        let l0 = f64::from(self.intervalo_c0);
        let mut g = self.grade;
        loop {
            let bloco = 1u32 << (cascatas_para(l0, g.largura, g.altura) - 1);
            let (w, h) = (
                g.largura.div_ceil(bloco) * bloco,
                g.altura.div_ceil(bloco) * bloco,
            );
            if (w, h) == (g.largura, g.altura) {
                return g;
            }
            (g.largura, g.altura) = (w, h);
        }
    }

    #[must_use]
    pub fn n_cascatas(&self) -> u32 {
        let g = self.grade_efetiva();
        cascatas_para(f64::from(self.intervalo_c0), g.largura, g.altura)
    }

    /// `[início, fim)` do intervalo da cascata `nivel`, em texels.
    #[must_use]
    pub fn intervalo(&self, nivel: u32) -> (f32, f32) {
        let l0 = f64::from(self.intervalo_c0);
        (
            (l0 * inicio_relativo(nivel)) as f32,
            (l0 * inicio_relativo(nivel + 1)) as f32,
        )
    }

    /// As dimensões do campo de distância (C1).
    #[must_use]
    pub fn grade_jfa(&self) -> [u32; 2] {
        let g = self.grade_efetiva();
        [
            g.largura.div_ceil(self.fator_jfa),
            g.altura.div_ceil(self.fator_jfa),
        ]
    }

    /// Os saltos do JFA, do maior ao 1, mais um passo de 1 a fechar (JFA+1).
    #[must_use]
    pub fn saltos_jfa(&self) -> Vec<u32> {
        let [w, h] = self.grade_jfa();
        let mut k = w.max(h).next_power_of_two() / 2;
        let mut saltos = Vec::new();
        while k >= 1 {
            saltos.push(k);
            k /= 2;
        }
        saltos.push(1);
        saltos
    }
}
