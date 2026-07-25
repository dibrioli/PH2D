//! **A pilha de Live Path Effects** (ADR-0132) — efeitos não-destrutivos e empilháveis
//! sobre um caminho.
//!
//! O caminho GUARDA a fonte autorada + a lista de efeitos; o mundo CONSOME o resultado
//! ([`crate::VecPath::cooked`]). É o `inkscape:original-d` + `d`, um nível acima: o
//! ADR-0121 fez a costura para o raio de quina, e esta é a mesma costura generalizada.
//!
//! ```text
//! verts autorados → [estágio 0: quina] → efeito₁ → efeito₂ → … → mundo
//! ```
//!
//! # Três invariantes, e nenhum é decoração
//!
//! 1. **Um efeito é `VecPath -> VecPath`, puro.** É *por isso* que a pilha compõe — a saída
//!    de um é entrada legítima do seguinte, sem caso especial no meio.
//! 2. **O ponto neutro é um no-op byte-idêntico**, e a pilha SALTA efeitos neutros. Sem
//!    isso, ligar a seção "Effects" no painel custaria uma alocação por frame a todo
//!    documento que a abrisse e nunca a usasse. (É o invariante que a rack de áudio provou
//!    valer a pena: 42 efeitos, gate por-efeito.)
//! 3. **`Cow::Borrowed` sobrevive.** Sem raio e sem efeito ativo, `cooked()` devolve o
//!    mesmo ponteiro — foi essa propriedade que permitiu ligar o `cooked()` em TODO
//!    consumidor sem mudar comportamento, e ela não pode morrer aqui.
//!
//! # A quina é o estágio ZERO, e não entra na pilha
//!
//! O raio mora no vértice **autorado**, e arredondar **divide** um vértice em dois. Um
//! efeito a jusante resampleia — a contagem de vértices é *saída* dele. Não há para onde
//! levar o raio da quina que deixou de existir, e por isso a quina corre primeiro, sempre,
//! e o que segue é geometria **plana** (ADR-0132 §3).
//!
//! # Acrescentar um efeito
//!
//! Um `variant` novo em [`PathEffect`] (**no fim** — postcard é posicional), um módulo
//! `fx_*.rs` irmão com a matemática pura, um braço em [`PathEffect::apply`] e um em
//! [`PathEffect::is_neutral`]. A matemática mora num módulo próprio e não conhece a pilha;
//! é isso que mantém aberto o caminho de um dia embrulhá-la num nó (ADR-0132 §4).

use crate::fx_falloff::{Falloff, FalloffSpec};
use crate::fx_trim::{self, TrimSpec};
use crate::fx_twist::{self, TwistSpec};
use crate::fx_zigzag::{self, ZigZagSpec};
use crate::{Contour, VecPath, VecPathId, VecScene};

/// **Um parâmetro de efeito, como o painel o desenha.**
///
/// O efeito DESCREVE os seus parâmetros e o painel os RENDERIZA — é o padrão da rack de áudio
/// (*"o painel se auto-popula da tabela `KINDS`"*), e é o que faz o próximo efeito custar zero
/// mudança de painel. Antes disto, acrescentar o Zig Zag custou uma rodada inteira dos 8 sites
/// de costura, e o gargalo tinha deixado de ser a geometria.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FxParam {
    /// O rótulo que o painel mostra.
    pub name: &'static str,
    /// A faixa do slider, no domínio do DOCUMENTO.
    pub min: f64,
    pub max: f64,
    /// `true` = caixinha (o valor só é 0 ou 1); `false` = slider.
    pub toggle: bool,
    /// `true` = o valor é uma CONTAGEM. O motor arredonda no `set`, e o painel mostra-o sem
    /// casas decimais — os três têm de concordar, senão o chip diz "37,42 cristas" enquanto a
    /// geometria desenha 37.
    pub integer: bool,
}

/// O teto de parâmetros por efeito — o painel registra este número de linhas, sempre, e pinta
/// só as que o efeito de facto declara. Registrar de menos deixa um controle clicável e morto;
/// registrar de mais é inerte. É o padrão dos presets do Envelope.
pub const MAX_FX_PARAMS: usize = 6;

/// O teto de efeitos numa pilha, pela mesma razão.
pub const MAX_PATH_EFFECTS: usize = 4;

/// **A escala de referência da forma** — o que dá sentido a um parâmetro de DISTÂNCIA.
///
/// Uma onda é uma distância, e por isso a 1ª versão pôs a amplitude do Zig Zag em unidades de
/// MUNDO. Estava errado, e o Enio apanhou-o no smoke: as formas desta cena têm ~2-3 unidades de
/// largura, então um slider que ia a 100 era absurdo — o efeito destruía a forma no primeiro
/// centésimo do curso.
///
/// A regra (Enio, 2026-07-18): **`100` = a média das dimensões da própria forma**. Assim o
/// mesmo número desenha a mesma coisa numa forma pequena e numa grande, que é o que o artista
/// espera de um slider de percentagem.
///
/// ⚠️ **A referência é a do caminho AUTORADO**, tirada uma vez antes da pilha correr — não a do
/// que chega a cada efeito. Se fosse a da entrada, pôr um Trim antes do Zig Zag encolheria a
/// onda, e o mesmo `Size` significaria coisas diferentes conforme a ORDEM da pilha.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FxCtx {
    /// A média entre a largura e a altura da caixa de controle do caminho autorado.
    pub ref_size: f64,
    /// O centro dessa mesma caixa. É a âncora de toda rotação de efeito — sai da MESMA medição
    /// que o `ref_size`, e do caminho AUTORADO, para que o significado de um botão não dependa
    /// da ordem da pilha.
    pub center: [f64; 2],
    /// A **semi-extensão** da caixa (metade da largura, metade da altura). É o que normaliza a
    /// posição de um ponto para `[-1, 1]²` — o espaço em que os warps paramétricos (Arc/Bulge/…)
    /// agem, exatamente como o diálogo Warp do Illustrator, que deforma DENTRO da bounding box.
    /// Um eixo degenerado (linha horizontal ⇒ `half.y == 0`) é tratado pelo consumidor.
    pub half: [f64; 2],
}

impl FxCtx {
    /// A referência de `path` — média das dimensões da caixa dos pontos de CONTROLE (âncoras e
    /// alças). A caixa de controle, e não a da curva: é mais barata, contém a curva, e a
    /// diferença entre as duas não muda o que um slider de percentagem significa.
    #[must_use]
    pub fn of(path: &VecPath) -> Self {
        let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
        let mut seen = false;
        for v in path.verts_all() {
            for p in [v.anchor, v.in_handle, v.out_handle] {
                seen = true;
                for k in 0..2 {
                    lo[k] = lo[k].min(p[k]);
                    hi[k] = hi[k].max(p[k]);
                }
            }
        }
        Self {
            ref_size: if seen {
                ((hi[0] - lo[0]) + (hi[1] - lo[1])) * 0.5
            } else {
                0.0
            },
            center: if seen {
                [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5]
            } else {
                [0.0; 2]
            },
            half: if seen {
                [(hi[0] - lo[0]) * 0.5, (hi[1] - lo[1]) * 0.5]
            } else {
                [0.0; 2]
            },
        }
    }
}

/// **Uma entrada da pilha** — o efeito e se ele está LIGADO.
///
/// O "ligado" é propriedade da ENTRADA, não do efeito: desarmar não pode custar os parâmetros.
/// Zerar a amplitude para "desligar" um Zig Zag e depois querer o valor de volta é a definição
/// de uma feature que obriga o artista a lembrar-se de números.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FxEntry {
    pub effect: PathEffect,
    /// Desligado = a pilha o SALTA, como se fosse neutro — mas os parâmetros ficam.
    pub enabled: bool,
}

impl FxEntry {
    /// Uma entrada nova, ligada.
    #[must_use]
    pub fn new(effect: PathEffect) -> Self {
        Self {
            effect,
            enabled: true,
        }
    }

    /// Esta entrada contribui para a geometria?
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.enabled && !self.effect.is_neutral()
    }
}

/// **Um efeito da pilha.** Dado de documento — viaja no save e no undo como qualquer
/// geometria.
///
/// ⚠️ **Append-only**: o postcard serializa o índice do variant. Um variant inserido no meio
/// relê saves antigos como o efeito errado, em silêncio.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PathEffect {
    /// Revela só um trecho do caminho — o *draw-on*. Ver [`crate::fx_trim`].
    Trim(TrimSpec),
    /// Ondula o caminho em cristas espaçadas por ARCO — o *Zig Zag* / *Roughen*.
    /// Ver [`crate::fx_zigzag`]. **Apendado por último**: postcard é posicional.
    ZigZag(ZigZagSpec),
    /// N cópias com transformação cumulativa — o *Repeater*. Ver [`crate::fx_repeat`].
    Repeat(crate::fx_repeat::RepeatSpec),
    /// Puxa/empurra os pontos do centro — o *Pucker & Bloat*. Ver [`crate::fx_warp`].
    /// **Apendado por último**: postcard é posicional.
    ///
    /// ⚠️ Houve um `Twist` entre este e o `Repeat`, e foi **CORTADO** antes de qualquer save
    /// existir (2026-07-18) — rasgava a geometria em toda a variante que tentei, e a razão está
    /// no cabeçalho do [`crate::fx_warp`]. Os índices posicionais fecharam-se atrás dele.
    Bloat(crate::fx_warp::BloatSpec),
    /// A família de warp paramétrico (Arc/Bulge/Wave/Fisheye/Rise) — o menu *Effect > Warp* do
    /// Illustrator. Ver [`crate::fx_warp_presets`]. **Apendado por último**: postcard é posicional.
    Warp(crate::fx_warp_presets::WarpSpec),
    /// **Falloff** — um campo escalar espacial que NÃO deforma nada sozinho: modula a FORÇA do
    /// deformador SEGUINTE na pilha (a ideia do *Falloff* do Cavalry). Ver [`crate::fx_falloff`].
    /// **Apendado por último**: postcard é posicional.
    Falloff(FalloffSpec),
    /// **Twist** — o remoinho: gira cada ponto em torno do centro por um ângulo que cresce com a
    /// distância. Ver [`crate::fx_twist`]. **Apendado por último**: postcard é posicional.
    ///
    /// ⚠️ Houve um `Twist` cortado em 2026-07-18 (cabeçalho do [`crate::fx_warp`]); este é a volta,
    /// sobre o esqueleto de reamostragem maduro dos presets de Warp, provada por render-and-look.
    Twist(TwistSpec),
}

impl PathEffect {
    /// Este efeito está no ponto neutro — ou seja, é um no-op byte-idêntico?
    ///
    /// A pilha usa isto para **saltá-lo por inteiro**, e é o que mantém o `Cow::Borrowed`
    /// vivo num documento que tem a seção aberta mas nada configurado.
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        match self {
            Self::Trim(t) => t.is_neutral(),
            Self::ZigZag(z) => z.is_neutral(),
            Self::Repeat(r) => r.is_neutral(),
            Self::Bloat(b) => b.is_neutral(),
            Self::Warp(w) => w.is_neutral(),
            Self::Falloff(f) => f.is_neutral(),
            Self::Twist(t) => t.is_neutral(),
        }
    }

    /// Este efeito é um Trim? Devolve os parâmetros dele.
    ///
    /// ⚠️ O `match` é **exaustivo de propósito**: quando o 2º tipo de efeito entrar, isto
    /// deixa de compilar e quem o acrescentar TEM de decidir o que este acessor responde.
    /// Um `_ => None` hoje seria um silêncio que ninguém voltaria a ler.
    #[must_use]
    pub fn as_trim(&self) -> Option<&TrimSpec> {
        match self {
            Self::Trim(t) => Some(t),
            Self::ZigZag(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Warp(_)
            | Self::Falloff(_)
            | Self::Twist(_) => None,
        }
    }

    /// O irmão mutável do [`Self::as_trim`] — para quem AJUSTA um parâmetro.
    pub fn as_trim_mut(&mut self) -> Option<&mut TrimSpec> {
        match self {
            Self::Trim(t) => Some(t),
            Self::ZigZag(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Warp(_)
            | Self::Falloff(_)
            | Self::Twist(_) => None,
        }
    }

    /// O Zig Zag deste efeito, se for um — irmão do [`Self::as_trim`].
    #[must_use]
    pub fn as_zigzag(&self) -> Option<&ZigZagSpec> {
        match self {
            Self::ZigZag(z) => Some(z),
            Self::Trim(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Warp(_)
            | Self::Falloff(_)
            | Self::Twist(_) => None,
        }
    }

    /// O irmão mutável do [`Self::as_zigzag`].
    pub fn as_zigzag_mut(&mut self) -> Option<&mut ZigZagSpec> {
        match self {
            Self::ZigZag(z) => Some(z),
            Self::Trim(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Warp(_)
            | Self::Falloff(_)
            | Self::Twist(_) => None,
        }
    }

    /// O nome que o painel mostra. Mora aqui (e não numa tabela no painel) porque uma
    /// segunda lista dos efeitos divergiria da primeira assim que alguém acrescentasse um.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Trim(_) => "Trim Path",
            Self::ZigZag(z) => {
                if z.rough_seed.is_some() {
                    "Roughen"
                } else {
                    "Zig Zag"
                }
            }
            Self::Repeat(_) => "Repeater",
            Self::Bloat(_) => "Pucker & Bloat",
            Self::Warp(w) => w.style.label(),
            Self::Falloff(f) => f.shape.label(),
            Self::Twist(_) => "Twist",
        }
    }

    /// O Warp deste efeito, se for um — irmão do [`Self::as_trim`].
    #[must_use]
    pub fn as_warp(&self) -> Option<&crate::fx_warp_presets::WarpSpec> {
        match self {
            Self::Warp(w) => Some(w),
            Self::Trim(_)
            | Self::ZigZag(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Falloff(_)
            | Self::Twist(_) => None,
        }
    }

    /// O irmão mutável do [`Self::as_warp`].
    pub fn as_warp_mut(&mut self) -> Option<&mut crate::fx_warp_presets::WarpSpec> {
        match self {
            Self::Warp(w) => Some(w),
            Self::Trim(_)
            | Self::ZigZag(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Falloff(_)
            | Self::Twist(_) => None,
        }
    }

    /// O Falloff deste efeito, se for um — irmão do [`Self::as_trim`].
    #[must_use]
    pub fn as_falloff(&self) -> Option<&FalloffSpec> {
        match self {
            Self::Falloff(f) => Some(f),
            Self::Trim(_)
            | Self::ZigZag(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Warp(_)
            | Self::Twist(_) => None,
        }
    }

    /// O irmão mutável do [`Self::as_falloff`].
    pub fn as_falloff_mut(&mut self) -> Option<&mut FalloffSpec> {
        match self {
            Self::Falloff(f) => Some(f),
            Self::Trim(_)
            | Self::ZigZag(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Warp(_)
            | Self::Twist(_) => None,
        }
    }

    /// O Twist deste efeito, se for um — irmão do [`Self::as_trim`].
    #[must_use]
    pub fn as_twist(&self) -> Option<&TwistSpec> {
        match self {
            Self::Twist(t) => Some(t),
            Self::Trim(_)
            | Self::ZigZag(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Warp(_)
            | Self::Falloff(_) => None,
        }
    }

    /// O irmão mutável do [`Self::as_twist`].
    pub fn as_twist_mut(&mut self) -> Option<&mut TwistSpec> {
        match self {
            Self::Twist(t) => Some(t),
            Self::Trim(_)
            | Self::ZigZag(_)
            | Self::Repeat(_)
            | Self::Bloat(_)
            | Self::Warp(_)
            | Self::Falloff(_) => None,
        }
    }

    /// **Este efeito CONSOME um campo de Falloff?** Só os deformadores por-ponto (Pucker & Bloat,
    /// Warp, Zig Zag, Twist) sabem escalar a força amostra a amostra. Trim (revela um trecho) e
    /// Repeat (multiplica contornos) não têm uma "força" que um campo espacial module, então um
    /// Falloff imediatamente acima deles é **inerte na geometria** — e o painel diz isso, em vez de
    /// deixar o artista adivinhar (a lei do "nenhum controle mudo", DIRETIVA §2).
    #[must_use]
    pub fn takes_falloff(&self) -> bool {
        match self {
            Self::ZigZag(_) | Self::Bloat(_) | Self::Warp(_) | Self::Twist(_) => true,
            Self::Trim(_) | Self::Repeat(_) | Self::Falloff(_) => false,
        }
    }

    /// **A tabela de tipos** — o menu "Add" do painel sai daqui, então acrescentar um efeito
    /// não toca no painel. A ordem é a dos variants.
    pub const KINDS: &'static [&'static str] = &[
        "Trim Path",
        "Zig Zag",
        "Repeater",
        "Pucker & Bloat",
        // ⚠️ Os NOVE warps do catálogo ÚNICO, DEPOIS dos quatro base, na ordem de
        // [`ph2d_warp_style::WarpStyle::ALL`] (a MESMA lista que os presets do Envelope). Não são
        // uma 2ª lista solta: o gate `every_effect_kind_is_reachable` exige
        // `from_kind(i).label() == KINDS[i]`, e o `label` de um Warp É o da `WarpStyle` — um typo
        // aqui fica VERMELHO. (Uma tabela por-KIND ainda; o dropdown de estilo é a próxima fatia.)
        "Arc",
        "Arc Upper",
        "Arc Lower",
        "Bulge",
        "Flag",
        "Wave",
        "Squeeze",
        "Fisheye",
        "Rise",
        // ⚠️ As QUATRO formas de Falloff, DEPOIS de tudo, na ordem de
        // [`crate::fx_falloff::FalloffShape::ALL`] — como os warps, um KIND por forma, todos o
        // MESMO variant `Falloff`. O `label` de um Falloff É o da `FalloffShape`, então o gate
        // `every_effect_kind_is_reachable` (`from_kind(i).label() == KINDS[i]`) fica VERMELHO num typo.
        "Falloff Radial",
        "Falloff Linear",
        "Falloff Rect",
        "Falloff Sweep",
        // ⚠️ O Twist, DEPOIS de tudo — um variant só (sem família de formas). `from_kind`/`kind_index`
        // o tratam pelo índice `TWIST_KIND`, e `label(Twist) == "Twist"`, então o gate
        // `every_effect_kind_is_reachable` (`from_kind(i).label() == KINDS[i]`) fica VERMELHO num typo.
        "Twist",
    ];

    /// Quantos KINDS vêm ANTES da família de warp — o offset dos estilos em [`Self::KINDS`].
    const WARP_BASE: usize = 4;

    /// Quantos KINDS vêm ANTES da família de falloff — os 4 base + os estilos de warp.
    const FALLOFF_BASE: usize = Self::WARP_BASE + crate::fx_warp_presets::WarpStyle::ALL.len();

    /// O índice do Twist em [`Self::KINDS`] — depois das formas de falloff. Um KIND só (o Twist não
    /// tem família de formas, ao contrário de Warp/Falloff).
    const TWIST_KIND: usize = Self::FALLOFF_BASE + crate::fx_falloff::FalloffShape::ALL.len();

    /// Um efeito novo do tipo `kind`, no ponto NEUTRO. `None` se o índice não existe.
    ///
    /// Nascer neutro não é detalhe: é o que faz o clique em "Add" não mover um pixel.
    #[must_use]
    pub fn from_kind(kind: usize) -> Option<Self> {
        match kind {
            0 => Some(Self::Trim(TrimSpec::default())),
            1 => Some(Self::ZigZag(ZigZagSpec::default())),
            2 => Some(Self::Repeat(crate::fx_repeat::RepeatSpec::default())),
            3 => Some(Self::Bloat(crate::fx_warp::BloatSpec::default())),
            // Um KIND por estilo de warp, todos o MESMO variant `Warp` — o estilo é escolhido no
            // Add, não é parâmetro vivo (ver `WarpSpec`).
            k if k < Self::FALLOFF_BASE => crate::fx_warp_presets::WarpStyle::ALL
                .get(k - Self::WARP_BASE)
                .map(|&s| Self::Warp(crate::fx_warp_presets::WarpSpec::new(s))),
            // ...e um KIND por forma de falloff, todos o MESMO variant `Falloff` (mesmo padrão).
            k if k < Self::TWIST_KIND => crate::fx_falloff::FalloffShape::ALL
                .get(k - Self::FALLOFF_BASE)
                .map(|&s| Self::Falloff(FalloffSpec::new(s))),
            // O Twist é um KIND só, no fim.
            k if k == Self::TWIST_KIND => Some(Self::Twist(TwistSpec::new())),
            _ => None,
        }
    }

    /// O índice deste efeito em [`Self::KINDS`].
    #[must_use]
    pub fn kind_index(&self) -> usize {
        match self {
            Self::Trim(_) => 0,
            Self::ZigZag(_) => 1,
            Self::Repeat(_) => 2,
            Self::Bloat(_) => 3,
            // O estilo, na ordem da `WarpStyle::ALL`, deslocado pelos base.
            Self::Warp(w) => Self::WARP_BASE + w.style as usize,
            // A forma, na ordem da `FalloffShape::ALL`, deslocada pelos base + os warps.
            Self::Falloff(f) => Self::FALLOFF_BASE + f.shape as usize,
            Self::Twist(_) => Self::TWIST_KIND,
        }
    }

    /// Aplica este efeito a um caminho inteiro — o contorno primário **e** cada subpath.
    ///
    /// Cada contorno é tratado **independentemente** (o modo "Individually" do After
    /// Effects). Um contorno que o efeito esvazie é DESCARTADO em vez de virar um contorno
    /// de zero vértices: um buraco vazio num compound não é geometria, é ruído para a
    /// booleana e para o preenchimento.
    ///
    /// `falloff` é o campo de FORÇA do(s) Falloff(s) que precedem este efeito na pilha
    /// ([`run_stack`]). Os deformadores por-ponto (Bloat/Warp/Zig Zag) o passam adiante e escalam o
    /// deslocamento amostra a amostra; os outros o ignoram. `None` = sem modulação, byte-idêntico
    /// ao que era antes do Falloff existir.
    #[must_use]
    pub fn apply(&self, path: &VecPath, ctx: &FxCtx, falloff: Option<&Falloff>) -> VecPath {
        let mut out = path.clone();
        match self {
            Self::Trim(spec) => {
                apply_per_contour(path, &mut out, |v, c| fx_trim::trim_contour(v, c, spec));
            }
            Self::ZigZag(spec) => {
                apply_per_contour(path, &mut out, |v, c| {
                    fx_zigzag::zigzag_contour(v, c, spec, ctx.ref_size, falloff)
                });
            }
            // ⚠️ O Repeater NÃO passa pelo `apply_per_contour`: ele multiplica a forma inteira,
            // e um buraco copiado por conta própria deixaria de ser buraco.
            // ⚠️ Sem `ctx`: o Repeater mede a ENTRADA dele, não o caminho autorado — ladrilhar
            // é uma operação sobre a coisa ladrilhada, e é isso que faz Repeater∘Repeater dar
            // uma grelha (o Blender constrói grelhas com dois modificadores Array empilhados).
            Self::Repeat(spec) => out = crate::fx_repeat::repeat_path(path, spec),
            Self::Bloat(spec) => {
                apply_per_contour(path, &mut out, |v, c| {
                    crate::fx_warp::bloat_contour(v, c, spec, ctx, falloff)
                });
            }
            Self::Warp(spec) => {
                apply_per_contour(path, &mut out, |v, c| {
                    crate::fx_warp_presets::warp_contour(v, c, spec, ctx, falloff)
                });
            }
            Self::Twist(spec) => {
                apply_per_contour(path, &mut out, |v, c| {
                    fx_twist::twist_contour(v, c, spec, ctx, falloff)
                });
            }
            // O Falloff não é geometria: ele é CONSUMIDO pelo `run_stack`, que o passa como o
            // `falloff` do efeito seguinte. Chamar `apply` num Falloff (fora da pilha) é um no-op.
            Self::Falloff(_) => {}
        }
        out
    }
}

/// Aplica `f` a CADA contorno do caminho — o primário e cada subpath, independentemente (o
/// modo "Individually" do After Effects).
///
/// Um contorno que o efeito esvazie é **descartado** em vez de virar um contorno de zero
/// vértices: um buraco vazio num compound não é geometria, é ruído para a booleana e para o
/// preenchimento.
///
/// Mora aqui, e não em cada `fx_*`, porque *"o que um efeito faz a um compound"* é uma
/// pergunta da PILHA — se cada efeito a respondesse por conta própria, o segundo responderia
/// diferente do primeiro e ninguém veria.
fn apply_per_contour(
    path: &VecPath,
    out: &mut VecPath,
    mut f: impl FnMut(&[crate::VecVertex], bool) -> (Vec<crate::VecVertex>, bool),
) {
    let (verts, closed) = f(&path.verts, path.closed);
    out.verts = verts;
    out.closed = closed;
    out.subpaths = path
        .subpaths
        .iter()
        .filter_map(|c| {
            let (v, cl) = f(&c.verts, c.closed);
            (!v.is_empty()).then_some(Contour {
                verts: v,
                closed: cl,
            })
        })
        .collect();
}

/// **Roda a pilha** sobre `path`, na ordem em que ela está.
///
/// `None` quando não há nada a fazer (pilha vazia, ou toda ela neutra) — e é esse `None`
/// que permite ao `cooked()` devolver `Cow::Borrowed`.
///
/// **A saída sai com a pilha VAZIA**, e isso não é higiene: é o que faz cozinhar duas vezes
/// ser igual a cozinhar uma. O `corner_live` garante o mesmo zerando o `corner_radius` do
/// que emite. Sem isso, qualquer consumidor que chamasse `cooked()` sobre um resultado já
/// cozido aplicaria a pilha outra vez — e o sintoma seria uma forma que encolhe a cada
/// passagem, sem erro nenhum.
#[must_use]
pub fn run_stack(path: &VecPath, stack: &[FxEntry]) -> Option<VecPath> {
    let mut active = stack.iter().filter(|e| e.is_active()).peekable();
    active.peek()?;
    // UMA vez, do caminho autorado: um parâmetro de distância tem de significar o mesmo
    // independentemente de onde o efeito está na pilha.
    let ctx = FxCtx::of(path);
    let mut cur: Option<VecPath> = None;
    // O campo de força que se acumula até o próximo deformador consumi-lo. Dois Falloffs seguidos
    // COMPÕEM (produto dos pesos = interseção das influências); o deformador seguinte é modulado e
    // o campo é zerado. Um Falloff sobre um efeito que não o consome (Trim/Repeat) é inerte na
    // geometria — o painel diz isso (`takes_falloff`).
    let mut pending = Falloff::default();
    for e in active {
        if let PathEffect::Falloff(spec) = &e.effect {
            pending.push(spec, &ctx);
            continue;
        }
        let field = (!pending.is_empty()).then_some(&pending);
        cur = Some(e.effect.apply(cur.as_ref().unwrap_or(path), &ctx, field));
        pending = Falloff::default();
    }
    let mut out = cur?;
    out.effects.clear();
    Some(out)
}

impl VecScene {
    /// **Assa a pilha de Live Path Effects** (ADR-0132) do caminho `id` — substitui a geometria
    /// autorada pela COZIDA ([`VecPath::cooked`]) e esvazia a pilha. É o *Expand Appearance* do
    /// Illustrator / *Object to Path* do Inkscape: o que se via vira a geometria, e as features
    /// vivas deixam de existir. Alimenta o botão **Apply** da seção Effects e o **Convert to
    /// Curves** sobre um caminho com efeitos.
    ///
    /// O cozido resolve a quina (estágio zero) **antes** de rodar a pilha, então um caminho que
    /// tenha efeito **e** raio de quina sai com os dois assados — é o que preserva a aparência,
    /// porque os efeitos rodam sobre a geometria já arredondada. Um caminho só com quina (sem
    /// efeito) NÃO é tocado: este é o bake da PILHA, e a alça de raio continua viva.
    ///
    /// `false` se o caminho sumiu ou a pilha está vazia — o botão "Apply" não é oferecido nesse
    /// caso, mas isto não depende de o painel ter razão. **Idempotente**: assar o resultado de
    /// volta é `false` (a pilha já está vazia). O `id`, o `fill` e o `stroke` SOBREVIVEM (o
    /// `cooked()` os preserva por clonagem, e reforço o `id` aqui): assar não é recriar o objeto,
    /// é congelar a aparência dele.
    pub fn bake_cooked(&mut self, id: VecPathId) -> bool {
        let cooked = match self.path(id) {
            Some(p) if p.has_live_geometry() => p.cooked().into_owned(),
            _ => return false,
        };
        let Some(p) = self.path_mut(id) else {
            return false;
        };
        let keep_id = p.id;
        *p = cooked;
        p.id = keep_id;
        // `cooked()` já esvazia a pilha quando roda algum efeito; numa pilha só de neutros ele
        // devolve a fonte emprestada (com os efeitos ainda lá), então limpar aqui cobre os dois.
        p.effects.clear();
        // O cozimento zera o raio das quinas que ARREDONDOU, mas um raio autorado numa quina que
        // não era arredondável (vizinhos colineares) sobrevive — não havia ângulo a comer. Zerá-lo
        // torna o bake IDEMPOTENTE: sem isto o botão ficaria aceso para sempre sobre um caminho já
        // assado, e um 2º clique não teria o que fazer.
        p.for_each_vert_mut(|v| v.corner_radius = 0.0);
        true
    }
}

/// A superfície de PARÂMETROS (`params`/`get`/`set`) — `impl PathEffect` continuado noutro
/// arquivo pelo teto de LOC.
#[path = "effect_params.rs"]
mod params_surface;

#[cfg(test)]
#[path = "effect_tests.rs"]
mod tests;
