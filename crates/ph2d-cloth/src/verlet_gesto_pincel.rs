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
    /// ⭐ **SÓ O FILTRO** (espec §7): força na direcção que o CHAMADOR dita —
    /// [`Pincel::eixo_da_gravidade`] —, a mesma em toda a peça.
    ///
    /// ⚠️ **É a única direcção deste ficheiro que não sai da malha nem do
    /// cursor, e isso foi MEDIDO antes de ela existir**
    /// (`tests/mede_a_composicao_do_filtro.rs`): rodando a peça E o gesto um
    /// quarto de volta, os oito modos do pincel rodam junto — desvio de
    /// equivariância `≤ 5,0e-11` nos oito. *Uma força que roda com a peça não é
    /// uma gravidade*, e é exactamente essa diferença que a orientação do filtro
    /// compra.
    ///
    /// ⛔ **Não a confunda com o [`Self::Empurrar`]:** aquele também move a peça
    /// toda para um lado só (constância `1,0000` medida numa esfera), mas a
    /// direcção dele é a normal da **área** — a malha dita-a — e a magnitude
    /// dele traz o raio dentro (`2R`).
    Gravidade,
    /// ⭐ **SÓ O FILTRO** (espec §7): o único tipo por ÂNCORA — `p⁰ + p⁰ · f`,
    /// com as componentes dos eixos desligados anuladas no
    /// [`Pincel::referencial`], e força de âncora `0,01`.
    ///
    /// ⚠️ **A homotetia é em torno da ORIGEM DO OBJECTO**, porque a âncora é
    /// construída sobre `p⁰` cru. A consequência fica nomeada: *uma peça cujo
    /// pivô não está no meio dela escala para longe do pivô* — a mesma nota que
    /// o filtro de malha (`FilterKind::Scale`) já carrega, e pela mesma razão.
    ///
    /// ⛔ **Ela não é o [`Self::Agarrar`] com outro alvo:** o Grab põe a âncora
    /// em `p⁰ + δ·f` (o `δ` do cursor) e σ vale `1`/`clamp(f)`; aqui o alvo é
    /// função da PRÓPRIA posição de repouso e σ é a constante `0,01`.
    Escala,
}

/// ⭐⭐⭐ **O QUE ACCIONA O GESTO** — o traço de um pincel, ou o arrasto de um
/// filtro. É a fronteira que a medição de 2026-09-07 obrigou a nomear.
///
/// ⚠️⚠️ **Ela não é conforto de arrumação: as duas leis de força são de FAMÍLIAS
/// diferentes, e isso está medido** (`tests/mede_a_composicao_do_filtro.rs`):
/// o traço da espec §4.1 põe `B = 10 · força² · flip · pressão` — **quadrático e
/// sem sinal** (picos `0,00625 / 0,025 / 0,100` para forças `0,25 / 0,50 / 1,00`,
/// erro relativo `0,0000`; e força `−1` dá o **mesmo** que `+1`) — enquanto o
/// filtro da §7 põe `S = força_base · Δpx · 0,001`, que é uma **recta com
/// sinal**. Levar o `S` do filtro dentro de um traço obrigaria a
/// `força = √|S/10|` mais `flip = sinal(S)`, e o preço não é estético: perto de
/// `S = 0` a raiz tem derivada infinita, ou seja *os primeiros pixels de arrasto
/// mexeriam a peça muito mais que os últimos*.
///
/// ⛔ **E o falloff também não se finge.** Esconder «não há pincel» atrás de um
/// raio enorme foi medido a explodir o [`Modo::Empurrar`] `2000×` (pico `598`
/// contra os `~0,3` dos outros sete), porque a magnitude dele traz o raio
/// dentro. *Um raio que finge ser infinito não é neutro — é lido como
/// comprimento por quem tem comprimento na lei.*
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Accionamento {
    /// **O traço** (espec §4.1): falloff de raio e curva, dureza, banda, força
    /// ao QUADRADO, sinal pelo `flip`, e a pressão da caneta.
    Traco,
    /// **O filtro** (espec §7): sem pincel nenhum — o factor por vértice é só
    /// `1 − máscara`, e o accionamento é o escalar `s`, com sinal.
    ///
    /// ⚠️ **Ele é o MESMO para os cinco tipos.** No traço o `B` muda por arm
    /// (`10` nos de força, `0,1` no Expand — um factor `100`); no filtro a espec
    /// §7 dá um `f` só a toda a gente, e as constantes que sobram (`0,01` do
    /// Expand, `0,01` da âncora da Escala) vivem **dentro** do tipo.
    Filtro { s: f64 },
}

/// **O REFERENCIAL do filtro** (espec §7, *Orientation* + *Force Axis*) — os
/// três eixos, **já resolvidos em coordenadas de mundo pelo chamador**, e quais
/// deles estão ligados.
///
/// ⚠️ **Esta crate não sabe o que é uma vista nem uma matriz de objecto**, e é
/// de propósito (o cabeçalho do [`super`] diz que ela não sabe sequer o que é
/// uma malha). *Local* · *World* · *View* são escolhas que o adaptador resolve
/// antes de chegar aqui — inclusive o caso especial em que, na vista, o «baixo»
/// da gravidade é o eixo do ECRÃ e não a profundidade.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Referencial {
    /// As direcções de mundo dos três eixos do referencial.
    pub eixos: [V3; 3],
    /// Que eixos a [`Modo::Escala`] deixa a âncora usar (omissão: os três).
    pub activo: [bool; 3],
}

impl Default for Referencial {
    fn default() -> Self {
        Self {
            eixos: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            activo: [true; 3],
        }
    }
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
    /// ⭐⭐⭐ **O TRAÇO ou o FILTRO** — ver [`Accionamento`]. Omissão: o traço, e o
    /// caminho dele é **byte-idêntico** ao de antes desta porta existir.
    pub accionamento: Accionamento,
    /// **A direcção de mundo da gravidade do filtro**, unitária (espec §7).
    /// Só a [`Modo::Gravidade`] a lê; omissão `−Z`.
    pub eixo_da_gravidade: V3,
    /// **O referencial e os eixos ligados** (espec §7). Só a [`Modo::Escala`] o
    /// lê; omissão: a identidade com os três eixos ligados.
    pub referencial: Referencial,
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
            accionamento: Accionamento::Traco,
            eixo_da_gravidade: [0.0, 0.0, -1.0],
            referencial: Referencial::default(),
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
