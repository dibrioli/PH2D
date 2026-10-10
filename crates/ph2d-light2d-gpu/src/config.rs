//! A configuração do solver: a grade ANCORADA NO MUNDO e as escolhas que a rodada de medição fixou.

use crate::solver::ErroSolver;

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

/// ⭐ Como a luz DIRETA (C3) chega à grade fina (W4b).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Refino {
    /// A visibilidade de cada lâmpada medida em TODO texel fino (uma marcha por texel e lâmpada).
    Tudo,
    /// A visibilidade das sondas da c0 interpolada; medida no texel fino só nas BORDAS.
    Bordas(Bordas),
}

/// Onde o refino pelas bordas mede a visibilidade no texel fino em vez de a interpolar — as
/// escolhas que a rodada da W4b fixou ([`REFINO_PADRAO`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bordas {
    /// Os quatro cantos da célula diferem mais que isto.
    pub limiar: f32,
    /// A célula mistura o fim da penumbra (0 ou 1) com o meio dela: o JOELHO que a interpolação
    /// corta.
    pub joelho: bool,
    /// A sonda só conta como «à vista» com o cone livre de uma CÉLULA inteira em volta (um oclusor
    /// fino pode passar entre as sondas).
    pub certificar: bool,
    /// O passe grosso marcha no passo FINO (senão no das cascatas).
    pub visibilidade_no_passo_fino: bool,
    /// Os raios fixos da abertura (nas sondas e no texel refinado).
    pub raios: u32,
    /// As bissecções da borda na abertura (nas sondas e no texel refinado): a borda fica a
    /// 1/(raios·2^bissecções) da abertura.
    pub bissecoes: u32,
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
    /// ⭐ Texels FINOS por texel das cascatas (potência de 2): a fluência final — as cascatas
    /// interpoladas mais a luz direta das lâmpadas — vive na grade fina ([`Self::grade_da_fluencia`]).
    pub fina: u32,
    pub refino: Refino,
}

/// O refino por omissão — MEDIDO (rodada `medida_luz2d_w4b`, RTX 5060 Ti, 1920×1080, grade fina de
/// 1 px; tabela no handoff W4b). Régua: a escada da cena 4 (degrau dos contornos 2 · 10 · 50 · 90 ·
/// 98 %, do canto até o muro) e o custo por lâmpada (Low · Medium):
/// - o limiar 0,1 (0,25 deixava o meio da penumbra interpolado: Low 0,51 px no de 10 %);
/// - o `joelho`: sem ele uma célula de cantos «0, 0, 0, 0,08» era interpolada e desenhava MANCHAS de
///   até 8 % de luz sobre o preto (vistas na FOTO; a régua de 10 · 50 · 90 % não as via): contorno de
///   2 % no Low 0,352 → 0,165 px;
/// - a visibilidade das sondas no passo FINO: no grosso o tremor da marcha entra onde a penumbra é
///   larga e se interpola (Medium 0,26 · 0,31 · 0,22 px → 0,18 · 0,12 · 0,08; a penumbra da lâmpada
///   grande também em Low), por +30 % de custo;
/// - 4 raios + 5 bissecções: 8+5 custava +50 % pela mesma régua (o exato); 2+6 mede igual e custa
///   −20 %, mas com 3 raios fixos um oclusor mais fino que MEIA abertura da lâmpada passa sem sombra
///   (com 5, um quarto) — fica a robustez;
/// - certificar: custo empatado — fica, de graça, contra o oclusor fino entre as sondas.
///
/// Resultado: Low 0,163 · 0,105 · 0,085 · 0,139 · 0,182 px · Medium 0,176 · 0,123 · 0,079 · 0,122 ·
/// 0,170 a 0,69 · 0,67 ms por lâmpada (o exato em todo texel: a mesma régua a 1,22 · 1,11 ms).
pub const REFINO_PADRAO: Refino = Refino::Bordas(Bordas {
    limiar: 0.1,
    joelho: true,
    certificar: true,
    visibilidade_no_passo_fino: true,
    raios: 4,
    bissecoes: 5,
});

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
            fina: 1,
            refino: REFINO_PADRAO,
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

    /// ⭐ A grade onde a FLUÊNCIA final vive (a que os receptores leem): a efetiva, com o texel
    /// ÷ [`Self::fina`] e a MESMA origem — a âncora do G11 vale para as duas.
    #[must_use]
    pub fn grade_da_fluencia(&self) -> Grade {
        let g = self.grade_efetiva();
        Grade {
            origem_m: g.origem_m,
            texel_m: g.texel_m / f64::from(self.fina),
            largura: g.largura * self.fina,
            altura: g.altura * self.fina,
        }
    }

    /// A configuração é aceitável? (o solver recusa-a antes de alocar nada)
    ///
    /// # Errors
    /// [`ErroSolver::ConfigInvalida`] com o campo que a recusa.
    pub(crate) fn validar(&self) -> Result<(), ErroSolver> {
        let cfg = self;
        let g = &cfg.grade;
        if !(g.texel_m.is_finite() && g.texel_m > 0.0) {
            return Err(ErroSolver::ConfigInvalida(
                "texel_m tem de ser finito e > 0",
            ));
        }
        if !(g.origem_m[0].is_finite() && g.origem_m[1].is_finite()) {
            return Err(ErroSolver::ConfigInvalida("origem_m tem de ser finita"));
        }
        if g.largura == 0 || g.altura == 0 {
            return Err(ErroSolver::ConfigInvalida(
                "a grade tem de ter ao menos um texel",
            ));
        }
        if !(cfg.intervalo_c0.is_finite() && cfg.intervalo_c0 > 0.0) {
            return Err(ErroSolver::ConfigInvalida(
                "intervalo_c0 tem de ser finito e > 0",
            ));
        }
        if cfg.fator_jfa == 0 {
            return Err(ErroSolver::ConfigInvalida("fator_jfa tem de ser ≥ 1"));
        }
        if !(cfg.passo_perto.is_finite() && cfg.passo_perto > 0.0) {
            return Err(ErroSolver::ConfigInvalida(
                "passo_perto tem de ser finito e > 0",
            ));
        }
        if !cfg.fina.is_power_of_two() {
            return Err(ErroSolver::ConfigInvalida(
                "fina tem de ser uma potência de 2",
            ));
        }
        if let Refino::Bordas(b) = cfg.refino
            && (!(0.0..=1.0).contains(&b.limiar) || b.raios == 0)
        {
            return Err(ErroSolver::ConfigInvalida(
                "refino: limiar fora de [0, 1] ou sem raios",
            ));
        }
        Ok(())
    }

    /// O passe grosso da visibilidade corre? Só com a grade fina MAIS fina que a das cascatas e o
    /// refino pelas bordas (com `fina = 1` o passe fino mede cada texel: é a sonda da c0).
    #[must_use]
    pub fn usa_visibilidade(&self) -> bool {
        self.fina > 1 && matches!(self.refino, Refino::Bordas(_))
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
