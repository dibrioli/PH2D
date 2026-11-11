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

mod carimbo; // a lei num carimbo — filho por LOC
pub use carimbo::mistura_opaca_carimbo;

/// O passo dos carimbos com a mistura ligada, em diâmetros: o «intervalo 3 %» medido nos pincéis de mistura do CSP
/// (Round mixing, Gouache, Oil paint). Acima disto a amostra por carimbo volta a fazer manchas.
pub const PASSO_DA_MISTURA: f32 = 0.03;

/// A partir deste raio (px) a AMOSTRA lê só os texels pares da grelha da tela (¼; a película continua inteira): a
/// média de milhares de texels não muda à vista (decisão 10 do dono, missão 4K B28.3; prancha em
/// `docs/Painter/missao_4k/pranchas/B28.3/`).
pub const RAIO_DA_AMOSTRA_ESPARSA: f32 = 16.0;

/// A sonda e a prancha comparam a amostra esparsa com a inteira no MESMO processo (`MEDIR_VELOCIDADE.md`).
#[doc(hidden)]
pub static AMOSTRA_ESPARSA: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(true);

/// A pegada SEM Grain do lote também em buffers locais, em paralelo entre os dabs (como a do Grain, Bug #84); `false`
/// = o kernel em série na máscara da tela inteira (a rota de antes, o oráculo do gate ao bit).
#[doc(hidden)]
pub static PEGADAS_LOCAIS_SEM_GRAO: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(true);

/// Quantos carimbos amostraram esparsos (a prancha prova que a variante armou).
#[doc(hidden)]
pub static CARIMBOS_ESPARSOS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

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
    /// No plano do traço, a pegada e a película a MEIA definição (¼ dos texels) e o plano ampliado (Bug #84): o véu
    /// largo do Wet Wash Mix. Sem UI; só a receita o liga.
    pub meia_definicao: bool,
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
        meia_definicao: false,
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
