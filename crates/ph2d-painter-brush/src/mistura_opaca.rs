//! **Color mixing** do Digital — «Wet blend», a lei MEDIDA do Clip Studio Paint
//! (`docs/Painter/CSP_COMO_MISTURA_2026-11-08.md`). Substitui o Pigment mix por píxel de 07/11, que deixava a
//! costura dura da tinta de baixo DENTRO do traço (foto do dono, 08/11).
//!
//! Por CARIMBO (nunca por píxel ou cerda), e CAUSAL (Bug #81: a cor de baixo vai no sentido do traço):
//! 1. amostra a **média** da tela de antes do traço sob a metade de TRÁS da pegada (o toque parado: a pegada
//!    inteira; peso = a pegada × o alfa) — UMA cor;
//! 2. a amostra entra no **reservatório**, que esquece pelo *Color stretch* (memória em diâmetros, por distância);
//! 3. a tinta do carimbo é `P = lerp(reservatório, cor do pincel, Amount)` — em RGB, ou em pigmento pelo
//!    *Pigment* (azul + amarelo = verde, a espectral do pincel);
//! 4. a pegada (falloff, Shape, Grain) dá só a **cobertura α** por píxel, × *Density*, normalizada pela distância
//!    (a Density é a opacidade de uma passada, qualquer que seja o passo); source-over sobre a tela VIVA.
//!    O texel ATRÁS do centro recebe a tinta de quando o centro passou por ele (o histórico do reservatório).
//! 5. o passo cai a [`PASSO_DA_MISTURA`] do diâmetro ([`BrushSpec::spacing_do_traco`]): a amostra é refeita a cada
//!    píxel de avanço — é isto, e a cor única por carimbo, que dá a mescla lisa sem manchas nem degraus.
//!
//! A tela escrita vive numa [`Sombra`] `f32` durante o traço: a soma de centenas de carimbos de α pequeno em `u8`
//! parava a meio (degraus de 1 byte) — a sombra acumula sem perda e o `u8` é só o arredondamento dela.
//! `Off` (o default) não é lido por ninguém: a rota de sempre pinta, ao bit. Sem libm ⇒ o mesmo byte em qualquer
//! máquina.

use crate::dab::DirtyRect;
use crate::spec::BrushSpec;
use rayon::prelude::*;

/// O passo dos carimbos com a mistura ligada, em diâmetros: o «intervalo 3 %» medido nos pincéis de mistura do CSP
/// (Round mixing, Gouache, Oil paint). Acima disto a amostra por carimbo volta a fazer manchas.
pub const PASSO_DA_MISTURA: f32 = 0.03;

/// O Amount a partir do qual a tinta do pincel cobre inteira o que não apanhou cor (o vazio).
pub const AMOUNT_QUE_COBRE: f32 = 0.25;

/// O modo do Color mixing. `Off` = a lei não existe para este pincel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ModoMisturaOpaca {
    /// Desligada (o default de todo pincel e de todo ficheiro antigo).
    #[default]
    Off,
    /// A tinta apanha e arrasta a cor da tela, carimbo a carimbo (a lei do CSP).
    WetBlend,
}

impl ModoMisturaOpaca {
    /// Todos, na ordem do seletor.
    pub const TODOS: [Self; 2] = [Self::Off, Self::WetBlend];

    /// O modo de um nome gravado; os de antes de 08/11 (`Blend`, `RunningColor`, `PigmentMix`) carregam como
    /// [`Self::WetBlend`].
    #[must_use]
    pub fn do_nome(s: &str) -> Option<Self> {
        match s {
            "Off" => Some(Self::Off),
            "WetBlend" | "PigmentMix" | "Blend" | "RunningColor" => Some(Self::WetBlend),
            _ => None,
        }
    }

    /// O nome gravado.
    #[must_use]
    pub fn nome(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::WetBlend => "WetBlend",
        }
    }
}

/// **Sample**: de onde a mistura apanha a cor (Bug #79, dono 08/11).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AmostraDaMistura {
    /// A camada em que se pinta, como estava no início do traço (o default de todo pincel e ficheiro antigo).
    #[default]
    EstaCamada,
    /// Todas as camadas visíveis compostas, como estavam no início do traço: o esbatedor numa camada nova vazia
    /// arrasta a cor das de baixo.
    TodasVisiveis,
}

impl AmostraDaMistura {
    /// Todas, na ordem do seletor.
    pub const TODAS: [Self; 2] = [Self::EstaCamada, Self::TodasVisiveis];

    /// O nome gravado.
    #[must_use]
    pub fn nome(self) -> &'static str {
        match self {
            Self::EstaCamada => "ThisLayer",
            Self::TodasVisiveis => "AllVisibleLayers",
        }
    }

    /// A amostra de um nome gravado.
    #[must_use]
    pub fn do_nome(s: &str) -> Option<Self> {
        Self::TODAS.into_iter().find(|a| a.nome() == s)
    }
}

/// Os controlos da mistura. Tudo em `0..=1`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MisturaOpaca {
    pub modo: ModoMisturaOpaca,
    /// **Amount**: quanto da tinta é a cor do pincel (`0` = só arrasta a da tela; `1` = só a do pincel).
    pub paint_amount: f32,
    /// **Density**: a opacidade de uma passada (`< 1` deixa ver o de baixo).
    pub paint_density: f32,
    /// **Color stretch**: o comprimento da memória do reservatório (`0` = esquece já; `1` = arrasta longe).
    pub color_stretch: f32,
    /// Quanto a pressão escala o Amount (`0` = ignora; `1` = amount × pressão).
    pub amount_pressure: f32,
    /// Quanto a pressão escala a Density (`0` = ignora; `1` = density × pressão).
    pub density_pressure: f32,
    /// De onde se amostra.
    pub amostra: AmostraDaMistura,
}

impl Default for MisturaOpaca {
    fn default() -> Self {
        Self::DE_FABRICA
    }
}

impl MisturaOpaca {
    /// O default, em `const` (o retrato de fábrica do painel é um `const`). Os números do Round mixing do CSP.
    pub const DE_FABRICA: Self = Self {
        modo: ModoMisturaOpaca::Off,
        // < 1: com Amount 1 a tinta é só a do pincel e não há mistura (censo dos controlos, 07/11).
        paint_amount: 0.5,
        paint_density: 1.0,
        color_stretch: 0.5,
        amount_pressure: 0.0,
        density_pressure: 0.0,
        amostra: AmostraDaMistura::EstaCamada,
    };

    /// A Density à pressão `pressao`.
    #[must_use]
    pub fn densidade(&self, pressao: f32) -> f32 {
        Self::pela_pressao(self.paint_density, self.density_pressure, pressao)
    }

    /// A lei age neste pincel?
    #[must_use]
    pub fn ligada(&self) -> bool {
        self.modo != ModoMisturaOpaca::Off
    }

    fn pela_pressao(valor: f32, peso: f32, pressao: f32) -> f32 {
        let p = pressao.clamp(0.0, 1.0);
        let k = peso.clamp(0.0, 1.0);
        (valor.clamp(0.0, 1.0) * (1.0 - k + k * p)).clamp(0.0, 1.0)
    }

    /// A retenção do reservatório por DIÂMETRO andado: `e^(−1/L)` com a memória `L = 0,1·e^(4·stretch)` diâmetros
    /// (medido no CSP: Gouache stretch 15 muda dentro de uma largura, Oil 87 leva ~3 larguras).
    fn retem_por_diametro(&self) -> f32 {
        let s = self.color_stretch.clamp(0.0, 1.0);
        let memoria = 0.1 * crate::espectral::exp2(s * 4.0 * std::f32::consts::LOG2_E);
        crate::espectral::exp2(-std::f32::consts::LOG2_E / memoria)
    }
}

/// `a` (RGBA recto) source-over `b`.
fn sobre(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let al = a[3] + b[3] * (1.0 - a[3]);
    if al <= 0.0 {
        return [0.0; 4];
    }
    let f = b[3] * (1.0 - a[3]);
    [
        (a[0] * a[3] + b[0] * f) / al,
        (a[1] * a[3] + b[1] * f) / al,
        (a[2] * a[3] + b[2] * f) / al,
        al,
    ]
}

/// `x^t` para `x ∈ [0, 1]`, `t ≥ 0`, sem transcendentais da libm (o mesmo byte em qualquer máquina).
fn potencia(x: f32, t: f32) -> f32 {
    if x <= 0.0 {
        return if t > 0.0 { 0.0 } else { 1.0 };
    }
    if x >= 1.0 || t <= 0.0 {
        return 1.0;
    }
    crate::espectral::exp2((t * crate::espectral::log2(x)).max(-126.0))
}

/// Marcos do [`Historico`]: um a cada `raio / MARCOS_POR_RAIO` de caminho, cobrindo dois raios para trás.
const MARCOS: usize = 32;
const MARCOS_POR_RAIO: f32 = 16.0;

/// O estado do pincel num ponto do caminho: o que um texel ATRÁS do centro recebe (a tinta que o pincel levava
/// quando o centro passou por cima dele). A mistura é CAUSAL: a cor apanhada à frente não pinta o que ficou atrás
/// (Bug #81, dono 10/11: o vermelho aparecia ~1,4 raio antes da borda).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Marco {
    arco: f32,
    /// `reservatório.cor`, o peso do pincel e a tinta com a cor do pincel do carimbo.
    cor: [f32; 3],
    a_eff: f32,
    tinta: [f32; 3],
}

impl Marco {
    fn lerp(a: Self, b: Self, t: f32) -> Self {
        let l = |x: f32, y: f32| x + (y - x) * t;
        Self {
            arco: l(a.arco, b.arco),
            cor: std::array::from_fn(|k| l(a.cor[k], b.cor[k])),
            a_eff: l(a.a_eff, b.a_eff),
            tinta: std::array::from_fn(|k| l(a.tinta[k], b.tinta[k])),
        }
    }
}

/// Os últimos [`MARCOS`] estados do reservatório, por caminho andado (anel).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Historico {
    marcos: [Marco; MARCOS],
    n: u8,
    /// O índice do mais NOVO.
    cabeca: u8,
}

impl Historico {
    fn novo(&self) -> Option<Marco> {
        (self.n > 0).then(|| self.marcos[self.cabeca as usize])
    }

    fn guarda(&mut self, m: Marco) {
        self.cabeca = ((self.cabeca as usize + 1) % MARCOS) as u8;
        self.marcos[self.cabeca as usize] = m;
        self.n = (self.n + 1).min(MARCOS as u8);
    }

    /// O estado no caminho `alvo`, interpolado; `agora` é o do carimbo corrente (o mais novo de todos).
    fn em(&self, alvo: f32, agora: Marco) -> Marco {
        let mut depois = agora;
        for k in 0..self.n as usize {
            let m = self.marcos[(self.cabeca as usize + MARCOS - k) % MARCOS];
            if m.arco <= alvo {
                let vao = depois.arco - m.arco;
                let t = if vao > 0.0 {
                    ((alvo - m.arco) / vao).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                return Marco::lerp(m, depois, t);
            }
            depois = m;
        }
        depois
    }
}

/// O reservatório do pincel durante UM traço (cor recta `0..1` + carga). Novo a cada pen-down.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ReservatorioOpaco {
    cor: [f32; 3],
    /// Quanto do reservatório é tinta apanhada (`0` = só papel transparente sob o pincel até agora).
    carga: f32,
    cheio: bool,
    /// O centro do carimbo anterior (o passo das taxas).
    ultimo: [f32; 2],
    /// O caminho andado pelos carimbos deste reservatório (px): a idade da película.
    arco: f32,
    /// A direção do movimento (unitária), `None` antes do 2.º carimbo ou parado.
    dir: Option<[f32; 2]>,
    historico: Historico,
}

impl ReservatorioOpaco {
    /// A cor levada agora (recta) e a carga, `None` antes do primeiro carimbo.
    #[must_use]
    pub fn cor(&self) -> Option<([f32; 3], f32)> {
        self.cheio.then_some((self.cor, self.carga))
    }

    /// O carimbo cai no MESMO centro do anterior: não move o reservatório (`passo 0` ⇒ `retem 1`) e a película recebe
    /// só o `k_passo` mínimo. É a cópia do spray do Wet Wash Mix (véu 4,5×, `jitter 0`): metade dos carimbos, ~3 % da
    /// tinta. Saltá-la custa Δ médio 0,8–1,0 / máx 8–10 à tela e corta o traço 1,9×; afinar o véu ao próprio diâmetro
    /// (¼–½ passo) dava 3,3–5,6× mas listras e borda ondulada — o liso é a MÉDIA de muitos carimbos sorteados
    /// (`sonda_custo_do_veu`, recusa em `docs/Painter/BUGS_painter.md` #84).
    #[must_use]
    pub fn repete_o_centro(&self, centro: [f32; 2]) -> bool {
        self.cheio && centro == self.ultimo
    }
}

/// O lado de um ladrilho da [`Sombra`] (texels).
const LADO: u32 = 64;
/// A altura de um ladrilho da [`Sombra`] (linhas): a FAIXA que o carimbo pinta num núcleo (`mistura_opaca_carimbo`).
const ALTO: u32 = 16;
/// `f32` por texel da [`Sombra`]: RGB recto, a cobertura e o `arco` do 1.º depósito.
const POR_TEXEL: usize = 5;
/// O `arco` de um texel sem tinta deste traço.
const SEM_TINTA: f32 = f32::INFINITY;
/// A tinta que o traço pôs há menos de `DESFASAGEM × raio` de caminho ainda não conta na amostra. O CSP medido
/// (08/11): numa reta a amostra vê sobretudo o fundo (frações 0,07/0,17/0,39 do Round mixing/Paint and apply/Oil =
/// `(1 − Amount)` × 0,35–1), mas no zigue-zague a volta apanha a ida (o vermelho segue vermelho sobre o amarelo).
/// A amostra viva sem atraso via só a tinta fresca do carimbo anterior: o G-Pen não misturava e o resultado
/// dependia do spacing do pincel. Varrido contra a folha do CSP (erro médio por região, 4 pincéis): 0,75 → 95 ·
/// 1,0 → 79 · 1,25 → 69 · 1,5 → 67; o Round mixing (a mesma ponta redonda nos dois) 7 · 12 · 16 · 20 — 1,0 é o
/// equilíbrio (`docs/Painter/handoffs/material_2026-11-08/wetblend_vs_csp_medidas.txt`).
///
/// Com a amostra CAUSAL (só a metade de trás, Bug #81) a película da passada corrente cobre-a desde `arco − b − raio`;
/// varrido no mesmo processo contra a lei de disco inteiro (fração do fundo a Amount 0,8 · 0,5 · vermelho máximo ao
/// atravessar uma faixa): disco 1,0 → 0,136 · 0,379 · 132 | causal 1,0 → 0,060 · 0,183 · 66 · 1,25 → 0,123 · 0,349 ·
/// 122 · 1,5 → 0,179 · 0,455 · 157 · 2,0 → 0,204 · 0,498 · 166. 1,25 guarda o patamar afinado pelo dono (08/11).
const DESFASAGEM: f32 = 1.25;

/// A PELÍCULA do traço em `f32` (RGB recto `0..1` + a cobertura), por ladrilhos de [`LADO`] × [`ALTO`], criados no 1.º toque
/// (vazios na tela directa; lidos do plano do traço, que já é a película). Vive do pen-down ao pen-up; quem repõe a tela por baixo dela (re-carimbo de figura, a cauda)
/// chama [`Sombra::recomeca`].
#[derive(Clone, Debug, Default)]
pub struct Sombra {
    w: u32,
    h: u32,
    colunas: u32,
    ladrilhos: Vec<Option<Box<[f32]>>>,
}

impl Sombra {
    /// Esquece tudo: o próximo toque relê a tela.
    pub fn recomeca(&mut self) {
        self.ladrilhos.clear();
    }

    /// Bytes guardados agora (a memória que a mistura paga).
    #[must_use]
    pub fn bytes(&self) -> usize {
        self.ladrilhos.iter().flatten().map(|b| b.len() * 4).sum()
    }

    fn mesma_tela(&self, w: u32, h: u32) -> bool {
        self.w == w && self.h == h && !self.ladrilhos.is_empty()
    }

    /// Cria os ladrilhos da caixa: lidos de `buf` (`da_tela`) ou vazios.
    fn garante(
        &mut self,
        buf: &[u8],
        w: u32,
        h: u32,
        (x0, y0, x1, y1): (u32, u32, u32, u32),
        da_tela: bool,
    ) {
        if self.w != w || self.h != h || self.ladrilhos.is_empty() {
            self.w = w;
            self.h = h;
            self.colunas = w.div_ceil(LADO);
            self.ladrilhos.clear();
            self.ladrilhos
                .resize((self.colunas * h.div_ceil(ALTO)) as usize, None);
        }
        for ty in y0 / ALTO..=(y1 - 1) / ALTO {
            for tx in x0 / LADO..=(x1 - 1) / LADO {
                let slot = &mut self.ladrilhos[(ty * self.colunas + tx) as usize];
                if slot.is_some() {
                    continue;
                }
                let (lx, ly) = (tx * LADO, ty * ALTO);
                let (lw, lh) = (LADO.min(w - lx), ALTO.min(h - ly));
                let mut b = Vec::with_capacity((lw * lh * POR_TEXEL as u32) as usize);
                for y in ly..ly + lh {
                    for x in lx..lx + lw {
                        let i = ((y * w + x) * 4) as usize;
                        if da_tela {
                            let px = &buf[i..i + 4];
                            b.extend(px.iter().map(|&v| f32::from(v) / 255.0));
                            // O plano já pintado é tinta VELHA deste traço (visível à amostra).
                            b.push(if px[3] > 0 {
                                f32::NEG_INFINITY
                            } else {
                                SEM_TINTA
                            });
                        } else {
                            b.extend([0.0, 0.0, 0.0, 0.0, SEM_TINTA]);
                        }
                    }
                }
                *slot = Some(b.into_boxed_slice());
            }
        }
    }

    /// O texel `(x, y)` se o ladrilho dele existe (`None` = a película ainda não passou ali).
    fn le(&self, x: u32, y: u32) -> Option<[f32; POR_TEXEL]> {
        let (tx, ty) = (x / LADO, y / ALTO);
        let lw = LADO.min(self.w - tx * LADO);
        let t = self
            .ladrilhos
            .get((ty * self.colunas + tx) as usize)?
            .as_deref()?;
        let j = ((y - ty * ALTO) * lw + (x - tx * LADO)) as usize * POR_TEXEL;
        t[j..j + POR_TEXEL].try_into().ok()
    }
}

/// A pegada do carimbo: o peso de cada texel (`0..1`) dentro de `caixa` `(x0, y0, x1, y1)`.
#[derive(Clone, Copy, Debug)]
pub enum Pegada<'a> {
    /// O alfa de uma máscara RGBA do tamanho da tela onde o kernel de sempre pintou o carimbo (Shape, Grain, cauda).
    Mascara {
        mascara: &'a [u8],
        caixa: (u32, u32, u32, u32),
        /// O texel da tela que é o 1.º da máscara e a largura dela em texels: `[0, 0]` e a largura da tela numa
        /// máscara do tamanho da tela; a caixa do dab numa pegada própria ([`crate::PegadaLocal`]).
        org: [u32; 2],
        largura: u32,
        /// A mesma pegada SEM o Grain (só com o Grain ativo): o grão é o TECTO da cobertura e o resto a TAXA, como
        /// no pincel sem a mistura (`stroke_cover::cover_add`); sem isto o lápis enchia os furos do papel.
        sem_grao: Option<&'a [u8]>,
        /// O RGB da máscara É a cor do pincel em cada texel (a rampa do Grain ou do Shape, as camadas do Per-Layer
        /// Color): a tinta mistura o reservatório com ELA, não com a cor única do carimbo (Bug #77).
        com_cor: bool,
    },
    /// Só o falloff do pincel num disco de `raio` à volta de `centro`.
    Disco { centro: [f32; 2], raio: f32 },
}

/// A tela em que um carimbo age.
pub struct Tela<'a> {
    /// A tela viva (RGBA8 recto), escrita pelo carimbo.
    pub buf: &'a mut [u8],
    pub width: u32,
    pub height: u32,
    /// A precisão da tela viva durante o traço.
    pub sombra: &'a mut Sombra,
    /// O alpha lock: a cor muda na proporção do alfa de antes e o alfa fica.
    pub trava_alfa: bool,
    /// A tela é o PLANO do traço (transparente, composto sobre a base depois): escreve-se a película, não
    /// `antes ⊕ película`.
    pub plano: bool,
}

/// O que o carimbo pinta.
#[derive(Clone, Copy, Debug)]
pub struct Carimbo {
    pub center: [f32; 2],
    pub radius_px: f32,
    /// A tinta nova (`Dab::color`, recta `0..1`).
    pub cor: [f32; 3],
    /// `Dab::coverage`.
    pub coverage: f32,
    /// A pressão da caneta (`1` no rato).
    pub pressao: f32,
}

/// Um carimbo do Wet blend. `antes(i)` = o RGBA do byte `i` (múltiplo de 4) da tela de ANTES do traço, sobre a qual
/// a película se compõe; `amostra_de(i)` = o fundo de onde se amostra (a mesma tela, ou todas as camadas visíveis —
/// [`AmostraDaMistura`]). Chamados só dentro da caixa da pegada. Devolve `None` com a lei `Off`, raio nulo ou fora da tela —
/// e nesses casos a tela e o reservatório ficam intactos.
pub fn mistura_opaca_carimbo(
    tela: &mut Tela<'_>,
    antes: impl Fn(usize) -> [u8; 4] + Sync,
    amostra_de: impl Fn(usize) -> [u8; 4] + Sync,
    spec: &BrushSpec,
    c: Carimbo,
    pegada: Pegada<'_>,
    res: &mut ReservatorioOpaco,
) -> Option<DirtyRect> {
    let m = spec.mistura_opaca;
    if !m.ligada() || c.radius_px <= 0.0 || c.coverage <= 0.0 {
        return None;
    }
    let (w, h) = (tela.width, tela.height);
    let (x0, y0, x1, y1) = match pegada {
        Pegada::Mascara { caixa, .. } => caixa,
        Pegada::Disco { centro, raio } => caixa_do_dab(w, h, centro, raio)?,
    };
    if x1 <= x0 || y1 <= y0 || x1 > w || y1 > h {
        return None;
    }
    // O texel `(x, y)` da tela na máscara da pegada.
    let na_mascara = |x: u32, y: u32| -> usize {
        match pegada {
            Pegada::Mascara { org, largura, .. } => {
                (((y - org[1]) * largura + (x - org[0])) * 4) as usize
            }
            Pegada::Disco { .. } => 0,
        }
    };
    let peso = |x: u32, y: u32| -> f32 {
        match pegada {
            Pegada::Mascara { mascara, .. } => f32::from(mascara[na_mascara(x, y) + 3]) / 255.0,
            Pegada::Disco { centro, raio } => {
                let dx = x as f32 + 0.5 - centro[0];
                let dy = y as f32 + 0.5 - centro[1];
                let t = (dx * dx + dy * dy).sqrt() / raio.max(1e-6);
                if t >= 1.0 {
                    0.0
                } else {
                    spec.falloff_weight(t)
                }
            }
        }
    };
    let dist = if res.cheio {
        let (dx, dy) = (c.center[0] - res.ultimo[0], c.center[1] - res.ultimo[1]);
        (dx * dx + dy * dy).sqrt()
    } else {
        0.0
    };
    let arco = res.arco + dist;
    let velha_desde = arco - DESFASAGEM * c.radius_px;
    // A película desta PASSADA (a metade de trás foi coberta há no máximo dois raios) tinge a amostra mas não
    // RECARREGA o pincel: a tinta que ele acabou de pôr saiu dele (o dedo esgota-se no vazio, dono 10/11). A volta de
    // um zigue-zague, mais velha, recarrega.
    let carrega_desde = arco - (DESFASAGEM + 2.0) * c.radius_px;
    let sombra_valida = tela.sombra.mesma_tela(w, h);
    let dir = if dist > 1e-4 {
        Some([
            (c.center[0] - res.ultimo[0]) / dist,
            (c.center[1] - res.ultimo[1]) / dist,
        ])
    } else {
        res.dir
    };
    // Quanto o texel ficou ATRÁS do centro, ao longo do movimento (`≤ 0` = à frente ou ao lado).
    let atras = |x: u32, y: u32| -> f32 {
        dir.map_or(0.0, |d| {
            -((x as f32 + 0.5 - c.center[0]) * d[0] + (y as f32 + 0.5 - c.center[1]) * d[1])
        })
    };
    // 1. A amostra: a média sob a pegada da tela de antes com a película VELHA deste traço por cima, pesada pelo
    // alfa; a carga = quanto ali é tinta. Em movimento, só a metade de TRÁS: o pincel leva o que já atravessou.
    // As FAIXAS de `ALTO` linhas (as linhas de ladrilhos da película): a amostra soma-se por faixa e as faixas em
    // ordem (o mesmo byte com qualquer número de núcleos); a película pinta-se por faixa (cada texel lê e escreve só o
    // seu). Um carimbo pequeno corre as mesmas faixas num núcleo.
    let (ty0, ty1) = (y0 / ALTO, (y1 - 1) / ALTO);
    let paralelo = ((x1 - x0) * (y1 - y0)) as usize >= crate::PARALLEL_MIN_AREA;
    let sombra_le: &Sombra = tela.sombra;
    let soma_da_faixa = |ty: u32| -> [f32; 6] {
        // `[r, g, b]` pesados pelo alfa · o alfa · a carga · o peso.
        let mut s = [0.0f32; 6];
        for y in (ty * ALTO).max(y0)..((ty + 1) * ALTO).min(y1) {
            for x in x0..x1 {
                let wt = peso(x, y);
                if wt <= 0.0 || atras(x, y) < 0.0 {
                    continue;
                }
                let i = ((y * w + x) * 4) as usize;
                let fundo = amostra_de(i).map(|v| f32::from(v) / 255.0);
                let (px, carga_px) = match sombra_valida.then(|| sombra_le.le(x, y)).flatten() {
                    Some(f) if f[4] <= velha_desde && f[3] > 0.0 => {
                        let px = sobre([f[0], f[1], f[2], f[3]], fundo);
                        (
                            px,
                            if f[4] <= carrega_desde {
                                px[3]
                            } else {
                                fundo[3]
                            },
                        )
                    }
                    _ => (fundo, fundo[3]),
                };
                let t = wt * px[3];
                for k in 0..3 {
                    s[k] += t * px[k];
                }
                s[3] += t;
                s[4] += wt * carga_px;
                s[5] += wt;
            }
        }
        s
    };
    let parciais: Vec<[f32; 6]> = if paralelo {
        (ty0..=ty1).into_par_iter().map(soma_da_faixa).collect()
    } else {
        (ty0..=ty1).map(soma_da_faixa).collect()
    };
    let mut total = [0.0f32; 6];
    for p in &parciais {
        for (t, v) in total.iter_mut().zip(p) {
            *t += v;
        }
    }
    let (soma, soma_a, soma_carga, soma_w) =
        ([total[0], total[1], total[2]], total[3], total[4], total[5]);
    if soma_w <= 0.0 {
        return None;
    }
    let pincel = c.cor.map(|v| v.clamp(0.0, 1.0));
    let carga_aqui = soma_carga / soma_w;
    let amostra = if soma_a > 0.0 {
        soma.map(|v| v / soma_a)
    } else {
        pincel
    };
    // 2. O reservatório, por distância (em diâmetros).
    let primeiro = !res.cheio;
    let passo = if res.cheio {
        (dist / (2.0 * c.radius_px)).min(1.0)
    } else {
        1.0
    };
    let retem = if res.cheio {
        potencia(m.retem_por_diametro(), passo)
    } else {
        0.0
    };
    let velha = retem * res.carga;
    let nova = (1.0 - retem) * carga_aqui;
    let carga = velha + nova;
    if carga > 0.0 {
        let f = nova / carga;
        let peso = if res.cheio && res.carga > 0.0 { f } else { 1.0 };
        for (cor, a) in res.cor.iter_mut().zip(amostra.iter()) {
            *cor += (a - *cor) * peso;
        }
    }
    res.carga = carga;
    res.cheio = true;
    res.ultimo = c.center;
    res.arco = arco;
    res.dir = dir;
    // 3. A tinta do carimbo: o papel transparente não suja (o pincel fica com a cor dele onde não apanhou nada).
    let amount = MisturaOpaca::pela_pressao(m.paint_amount, m.amount_pressure, c.pressao);
    let a_eff = peso_do_pincel(amount, res.carga);
    // O «Pigment» é UM controlo do pincel (`BrushSpec::effective_pigment_mix`, cartão Paint Mixing, dono 10/11): o mesmo
    // número mistura o pincel com o que ele apanhou e a tinta com a tela.
    let pig = spec.effective_pigment_mix();
    let tinta_de = |res_cor: [f32; 3], a_eff: f32, pincel: [f32; 3]| -> [f32; 3] {
        let rgb: [f32; 3] = std::array::from_fn(|k| res_cor[k] + (pincel[k] - res_cor[k]) * a_eff);
        if pig > 0.0 {
            let e = crate::espectral::mistura(res_cor, pincel, a_eff);
            std::array::from_fn(|k| rgb[k] + (e[k] - rgb[k]) * pig)
        } else {
            rgb
        }
    };
    let agora = Marco {
        arco,
        cor: res.cor,
        a_eff,
        tinta: tinta_de(res.cor, a_eff, pincel),
    };
    // O texel atrás do centro recebe o estado de quando o centro passou por ele: uma tabela por fração do raio.
    let tabela: [Marco; MARCOS] = std::array::from_fn(|k| {
        res.historico
            .em(arco - c.radius_px * k as f32 / (MARCOS - 1) as f32, agora)
    });
    if res
        .historico
        .novo()
        .is_none_or(|m| arco - m.arco >= c.radius_px / MARCOS_POR_RAIO)
    {
        res.historico.guarda(agora);
    }
    let estado_em = |x: u32, y: u32| -> Marco {
        let b = atras(x, y);
        if b <= 0.0 {
            return agora;
        }
        let f = (b / c.radius_px.max(1e-6)).min(1.0) * (MARCOS - 1) as f32;
        let k = (f as usize).min(MARCOS - 2);
        Marco::lerp(tabela[k], tabela[k + 1], f - k as f32)
    };
    let cor_da_mascara = match pegada {
        Pegada::Mascara {
            mascara,
            com_cor: true,
            ..
        } => Some(mascara),
        _ => None,
    };
    // 4. A película do traço: cada carimbo põe `tinta` nela à taxa da pegada × Density × Flow (normalizada pelo
    // passo do pincel), e a cobertura dela sobe só até ao TECTO do pincel sem a mistura (Accumulate desligado: a
    // coverage do dab × o grão; ligado: 1). A tela é `antes ⊕ película`: a de baixo só aparece
    // por `1 − cobertura`, nunca como degrau dentro da cor (a costura da foto do dono, 08/11).
    // O pincel só cobre o que não apanhou na medida em que TEM tinta: Amount 0 (um esbatedor, o Gouache blender)
    // não pinta o vazio; a partir de [`AMOUNT_QUE_COBRE`] cobre inteiro (o vazio não lava a cor).
    let corpo = res.carga + (1.0 - res.carga) * (amount / AMOUNT_QUE_COBRE).min(1.0);
    let densidade = m.densidade(c.pressao);
    let cobertura = c.coverage.clamp(0.0, 1.0);
    // O CSP medido (08/11, `wetblend_vs_csp_medidas.txt`): a Density é a TAXA por carimbo (Density 45–50 ainda
    // cobre 88–98 % numa passada) e o tecto é a opacidade do pincel; como tecto ela dava um véu chapado.
    // O `corpo` é a opacidade que a tinta ATINGE (a película tende a ele), não uma taxa: com a taxa, o dedo de carga
    // pequena empilhava carimbos até ao opaco e nunca esmaecia (dono 10/11). Com `corpo = 1` é a lei de antes.
    let taxa = densidade * spec.flow.clamp(0.0, 1.0);
    let tecto = if spec.accumulate { 1.0 } else { cobertura };
    let taxa = if spec.accumulate {
        taxa * cobertura
    } else {
        taxa
    };
    if tecto <= 0.0 || taxa <= 0.0 {
        return None;
    }
    // A opacidade acumula como o pincel SEM a mistura: cada carimbo vale `passo / passo do pincel` de um dab dele
    // (a mistura adensa os carimbos a 3 %, não a cobertura); o 1.º vale um passo da mistura (sem disco no início).
    let nominal = spec.spacing.max(PASSO_DA_MISTURA);
    // O pincel que sorteia a pegada (Pointillism, Pastel, Chalk) carimba também a 3 % e enche: o look aprovado pelo
    // dono (08/11, revoga o Bug #78).
    let k_passo = if primeiro {
        spec.spacing_do_traco() / nominal
    } else {
        passo / nominal
    };
    let k_passo = k_passo.clamp(PASSO_DA_MISTURA * 0.25, 1.0);
    // No plano do traço a tela É a película (relê-se dela); na tela directa a película nasce vazia no pen-down.
    tela.sombra
        .garante(tela.buf, w, h, (x0, y0, x1, y1), tela.plano);
    let alfa_de = |wt: f32| 1.0 - potencia(1.0 - (wt * taxa).min(1.0), k_passo);
    // A pegada da máscara é um BYTE: a taxa de cada texel é uma de 256 por carimbo (a potência por texel era o maior
    // custo do carimbo de véu).
    let tabela_alfa: Option<[f32; 256]> = matches!(pegada, Pegada::Mascara { .. })
        .then(|| std::array::from_fn(|b| alfa_de(f32::from(b as u8) / 255.0)));
    let (plano, trava_alfa) = (tela.plano, tela.trava_alfa);
    let pinta_faixa = |(ty, linhas, ladrilhos): (u32, &mut [u8], &mut [Option<Box<[f32]>>])| {
        let base = ty * ALTO;
        for y in base.max(y0)..(base + ALTO).min(y1) {
            for x in x0..x1 {
                let (wt, byte) = match pegada {
                    Pegada::Mascara { mascara, .. } => {
                        let b = mascara[na_mascara(x, y) + 3];
                        (f32::from(b) / 255.0, Some(b))
                    }
                    Pegada::Disco { .. } => (peso(x, y), None),
                };
                if wt <= 0.0 {
                    continue;
                }
                // O grão: a razão entre a pegada e a pegada sem ele vai ao tecto; a pegada sem ele é a taxa.
                let (wt, grao, byte) = match pegada {
                    Pegada::Mascara {
                        sem_grao: Some(sg), ..
                    } => {
                        let b = sg[na_mascara(x, y) + 3];
                        let liso = f32::from(b) / 255.0;
                        if liso > 0.0 {
                            (liso, (wt / liso).min(1.0), Some(b))
                        } else {
                            (wt, 1.0, byte)
                        }
                    }
                    _ => (wt, 1.0, byte),
                };
                let tecto = tecto * grao;
                let alfa = match (&tabela_alfa, byte) {
                    (Some(t), Some(b)) => t[usize::from(b)],
                    _ => alfa_de(wt),
                };
                if alfa <= 0.0 {
                    continue;
                }
                let i = ((y * w + x) * 4) as usize;
                let tx = x / LADO;
                let lw = LADO.min(w - tx * LADO);
                let ladrilho = ladrilhos[tx as usize]
                    .as_deref_mut()
                    .expect("ladrilho da sombra garantido antes do toque");
                let j = ((y - base) * lw + (x - tx * LADO)) as usize * POR_TEXEL;
                let px = &mut ladrilho[j..j + POR_TEXEL];
                if px[4] == SEM_TINTA {
                    px[4] = arco;
                }
                let estado = estado_em(x, y);
                let tinta = match cor_da_mascara {
                    Some(mc) => tinta_de(estado.cor, estado.a_eff, {
                        let mi = na_mascara(x, y);
                        [mc[mi], mc[mi + 1], mc[mi + 2]].map(|v| f32::from(v) / 255.0)
                    }),
                    None => estado.tinta,
                };
                let s0 = px[3];
                let s_livre = s0 * (1.0 - alfa) + corpo * alfa;
                if s_livre > 0.0 {
                    for k in 0..3 {
                        px[k] = (px[k] * s0 * (1.0 - alfa) + tinta[k] * corpo * alfa) / s_livre;
                    }
                }
                px[3] = s_livre.max(s0).min(tecto.max(s0));
                let pel = [px[0], px[1], px[2], px[3]];
                let out = if plano {
                    pel
                } else {
                    let ant = antes(i).map(|v| f32::from(v) / 255.0);
                    let sc = pel[3];
                    if trava_alfa {
                        let sc = if ant[3] > 0.0 { sc } else { 0.0 };
                        [
                            ant[0] + (pel[0] - ant[0]) * sc,
                            ant[1] + (pel[1] - ant[1]) * sc,
                            ant[2] + (pel[2] - ant[2]) * sc,
                            ant[3],
                        ]
                    } else {
                        let a = sc + ant[3] * (1.0 - sc);
                        if a > 0.0 {
                            let f = ant[3] * (1.0 - sc);
                            [
                                (pel[0] * sc + ant[0] * f) / a,
                                (pel[1] * sc + ant[1] * f) / a,
                                (pel[2] * sc + ant[2] * f) / a,
                                a,
                            ]
                        } else {
                            [0.0; 4]
                        }
                    }
                };
                let li = (((y - base) * w + x) * 4) as usize;
                for (dst, v) in linhas[li..li + 4].iter_mut().zip(out.iter()) {
                    *dst = (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                }
            }
        }
    };
    let colunas = tela.sombra.colunas as usize;
    let faixas: Vec<_> = tela
        .buf
        .chunks_mut((ALTO * w * 4) as usize)
        .zip(tela.sombra.ladrilhos.chunks_mut(colunas))
        .enumerate()
        .skip(ty0 as usize)
        .take((ty1 - ty0 + 1) as usize)
        .map(|(ty, (linhas, ladrilhos))| (ty as u32, linhas, ladrilhos))
        .collect();
    if paralelo {
        faixas.into_par_iter().for_each(pinta_faixa);
    } else {
        faixas.into_iter().for_each(pinta_faixa);
    }
    Some(DirtyRect {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    })
}

/// Quanto da tinta é a cor do pincel, pesada pela QUANTIDADE de cada lado (Bug #76, aprovado pelo dono 08/11):
/// `a / (c·(1 − a) + a)` — o reservatório entra com o que leva (`c·(1 − a)`), o pincel com o Amount. `a = 0` → só o
/// reservatório (o esbatedor não solta a cor do pincel quando a carga decai no vazio); `c = 0` → só o pincel;
/// `c = 1` → `a`, a lei de antes. Sem nada dos dois lados não há tinta (o `corpo` é 0): devolve o pincel.
#[must_use]
pub fn peso_do_pincel(amount: f32, carga: f32) -> f32 {
    let a = amount.clamp(0.0, 1.0);
    let c = carga.clamp(0.0, 1.0);
    let d = c * (1.0 - a) + a;
    if d > 0.0 { a / d } else { 1.0 }
}

/// A caixa de píxeis `(x0, y0, x1, y1)` de um disco de raio `r` em `c` (`None` fora da tela).
#[must_use]
pub fn caixa_do_dab(
    width: u32,
    height: u32,
    center: [f32; 2],
    radius_px: f32,
) -> Option<(u32, u32, u32, u32)> {
    let x0 = (center[0] - radius_px).floor().max(0.0);
    let y0 = (center[1] - radius_px).floor().max(0.0);
    let x1 = (center[0] + radius_px).ceil().min(width as f32);
    let y1 = (center[1] + radius_px).ceil().min(height as f32);
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    Some((x0 as u32, y0 as u32, x1 as u32, y1 as u32))
}
