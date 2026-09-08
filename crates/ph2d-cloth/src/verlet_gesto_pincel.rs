//! **O VOCABULÁRIO DOS CONTROLOS do pincel de tecido** — irmão (`#[path]`) do
//! [`super::verlet_gesto`], cortado por ASSUNTO.
//!
//! Aqui vive *o que o artista escolhe* (espec §8.1): os quatro selectores, os
//! números do pincel e o peso de BANDA que a área simulada desenha (§2.2). Lá
//! vive *o que o gesto FAZ com essas escolhas*.
//!
//! ⚠️ **Nenhum caminho de chamador muda:** o `verlet_gesto` re-exporta os cinco
//! tipos e a função, e é por lá que toda a casa os lê.
//!
//! ⚠️ **O gate de LOC foi o gatilho, não a razão.** O `verlet_gesto.rs` cruzou os
//! 700 do `architecture_workspace_file_loc_cap` na wave da emenda Q18, e o que
//! saiu foram as duas metades com fronteira própria — esta e a normal da área.

use crate::V3;
use crate::verlet::{Solver, dist};

/// Os oito tipos de deformação (espec §4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Modo {
    /// Força na direcção UNITÁRIA do movimento do cursor — a mesma para todos.
    Arrastar,
    /// Força para DENTRO, ao longo da normal da área, com magnitude `2R`.
    Empurrar,
    /// Força unitária do vértice PARA o cursor.
    ApertarPonto,
    /// Força para a LINHA do traço (só as componentes perpendiculares).
    ApertarLinha,
    /// Força ao longo da normal do vértice, para fora.
    Inflar,
    /// Âncora `p⁰ + δ_total · f`, pegada congelada na malha de partida.
    Agarrar,
    /// Âncora `x + δ_incremental · f`, re-pegada a cada passo.
    Gancho,
    /// Desvio do comprimento de repouso, `τ += 0,01 · f` por passo.
    Expandir,
}

/// A área simulada (espec §2.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Area {
    /// Esfera de raio `R₀(1+L)` centrada na localização inicial do traço.
    Local,
    /// Tudo; `w ≡ 1`.
    Global,
    /// Esfera de raio `R(1+L)` centrada no cursor actual.
    Dinamica,
}

/// A forma espacial do peso da força (espec §4.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FalloffForca {
    Radial,
    Plano,
}

/// A curva de falloff do pincel (espec §4.1). Só as que o oráculo usa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Curva {
    /// `3u² − 2u³`, `u = 1 − d/R`.
    Suave,
    /// `u²`.
    Aguda,
    /// `1`.
    Constante,
}

impl Curva {
    /// O peso em `[0, 1]` de uma distância `d` num raio `r`.
    #[must_use]
    pub fn peso(self, d: f64, r: f64) -> f64 {
        if r <= 0.0 || d >= r {
            return 0.0;
        }
        let u = 1.0 - d / r;
        match self {
            Self::Suave => 3.0 * u * u - 2.0 * u * u * u,
            Self::Aguda => u * u,
            Self::Constante => 1.0,
        }
    }
}

/// Os controlos do pincel de tecido (espec §8.1 — as omissões do CÓDIGO).
#[derive(Clone, Copy, Debug)]
pub struct Pincel {
    pub modo: Modo,
    pub area: Area,
    pub falloff_forca: FalloffForca,
    pub curva: Curva,
    /// O raio, em unidades de objecto.
    pub raio: f64,
    /// *Strength* em `[0,1]` — ⚠️ o alvo eleva ao QUADRADO nos modos de força.
    pub forca: f64,
    /// A dureza `h ∈ [0,1]`: distância `< h·R` ⇒ peso `1`.
    pub dureza: f64,
    /// *Simulation Limit* `L` (omissão `2,5`).
    pub limite: f64,
    /// *Simulation Falloff* `F` (omissão `0,75`).
    pub banda: f64,
    /// *Pin Simulation Boundary* (só *Local*; omissão `false`).
    pub pino: bool,
    /// `flip = ±1` (Add/Subtract).
    pub flip: f64,
    pub solver: Solver,
    /// ⚠️ **Experimento de paridade** — a escala do `limite` que o leitor de
    /// banda das RESTRIÇÕES (`φ`) usa: `1` = a espec §2.2 (o mesmo `R(1+L)` da
    /// força). Medido em 2026-09-06 porque nenhum conjunto de restrições
    /// reproduz Local e Global ao mesmo tempo.
    pub escala_phi: f64,
    /// Idem para o leitor da RETENÇÃO de velocidade.
    pub escala_retencao: f64,
    /// **Quantas PASSAGENS o traço faz por passo** (as cópias de simetria; `1`
    /// = sem simetria). Só a área *Local* a lê, e é ela que decide quantas
    /// vezes a lista de restrições é construída — ver [`Self::construcoes`].
    pub passagens: u32,
    /// ⭐⭐⭐ **A LEI DO ALVO, CONVERGIDA — o instrumento do §5.2-ter.** ⛔ `false`
    /// por omissão, e o caminho de omissão é **byte-idêntico**.
    ///
    /// Ligado, o avanço que os DOIS APERTOS produzem num passo é limitado ao que
    /// falta até ao alvo daquele modo (o cursor · o plano do cursor · a projecção).
    ///
    /// ⚠️ **Não é um tecto escolhido: é o LIMITE da própria lei do alvo.** A
    /// direcção dos apertos é `unit(cursor − p)` — a única, entre os cinco modos
    /// que escrevem aceleração, que é função da posição **actual** do vértice.
    /// Sub-dividir o passo re-avaliando essa direcção faz o vértice caminhar em
    /// recta até ao alvo e **parar lá**, porque a espec já manda separação nula
    /// dar força nula; e esse limite escreve-se em fechado como o `min`. ⇒ *o nó
    /// que o alvo faz com força alta é artefacto de um passo de `2,1×` a aresta,
    /// não uma lei* — e a saída **(b)** do §5.2-ter deixa de ser «mudar o
    /// produto» para ser «integrar a lei dele como ela pede».
    ///
    /// ⛔ Só os apertos: nos outros três a direcção é constante no passo, logo
    /// não há nada a convergir e o `min` seria uma lei NOVA.
    pub converge_aperto: bool,
}

impl Pincel {
    /// **Quantas vezes a lista de restrições é construída** (espec, emenda Q8):
    /// na área *Local* são `passagens + 1`; nas outras é uma só.
    ///
    /// ⚠️ **É daqui que sai a rigidez que separava a *Local* da *Global***, e
    /// não de uma contagem de varreduras: os dois ramos relaxam o mesmo número
    /// de vezes, mas na *Local* cada restrição está na lista `passagens + 1`
    /// vezes, porque o registo de duplicados vive UMA construção
    /// ([`ph2d_cloth::verlet::Verlet::reabrir`]).
    ///
    /// ⚠️ **Sem isto a *Local* rende como a *Global*** — medido em 06/09 sobre
    /// as 50 fixtures: `plano_arrastar_radial_local` erra `125 %` com uma cópia
    /// e `7 %` com duas, e a contagem de vértices movidos passa a bater EXACTA
    /// em oito traços. ⛔ Não «optimize» a lista deduplicando-a.
    #[must_use]
    pub fn construcoes(&self) -> u32 {
        if self.area == Area::Local {
            self.passagens.max(1) + 1
        } else {
            1
        }
    }
}

impl Default for Pincel {
    fn default() -> Self {
        Self {
            modo: Modo::Arrastar,
            area: Area::Local,
            falloff_forca: FalloffForca::Radial,
            curva: Curva::Suave,
            raio: 0.35,
            forca: 1.0,
            dureza: 0.0,
            limite: 2.5,
            banda: 0.75,
            pino: false,
            flip: 1.0,
            solver: Solver::default(),
            escala_phi: 1.0,
            escala_retencao: 1.0,
            passagens: 1,
            converge_aperto: false,
        }
    }
}

/// **O peso de BANDA** `w(p)` (espec §2.2): `1` dentro do início, `0` fora do
/// limite, *smoothstep* entre os dois.
#[must_use]
pub fn banda(p: V3, c: V3, r: f64, limite: f64, falloff: f64) -> f64 {
    let fim = r * (1.0 + limite);
    let inicio = r * (1.0 + limite * falloff);
    let d = dist(p, c);
    if d < inicio {
        1.0
    } else if d > fim || fim <= inicio {
        0.0
    } else {
        let t = 1.0 - (d - inicio) / (fim - inicio);
        3.0 * t * t - 2.0 * t * t * t
    }
}
