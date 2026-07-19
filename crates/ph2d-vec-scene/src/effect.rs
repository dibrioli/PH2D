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

use crate::fx_trim::{self, TrimSpec};
use crate::fx_zigzag::{self, ZigZagSpec};
use crate::{Contour, VecPath};

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
pub const MAX_FX_PARAMS: usize = 4;

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
    /// Gira em torno do centro com força radial — o *Twist*. Ver [`crate::fx_warp`].
    Twist(crate::fx_warp::TwistSpec),
    /// Puxa/empurra os pontos do centro — o *Pucker & Bloat*. Ver [`crate::fx_warp`].
    /// **Apendado por último**: postcard é posicional.
    Bloat(crate::fx_warp::BloatSpec),
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
            Self::Twist(t) => t.is_neutral(),
            Self::Bloat(b) => b.is_neutral(),
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
            Self::ZigZag(_) | Self::Repeat(_) | Self::Twist(_) | Self::Bloat(_) => None,
        }
    }

    /// O irmão mutável do [`Self::as_trim`] — para quem AJUSTA um parâmetro.
    pub fn as_trim_mut(&mut self) -> Option<&mut TrimSpec> {
        match self {
            Self::Trim(t) => Some(t),
            Self::ZigZag(_) | Self::Repeat(_) | Self::Twist(_) | Self::Bloat(_) => None,
        }
    }

    /// O Zig Zag deste efeito, se for um — irmão do [`Self::as_trim`].
    #[must_use]
    pub fn as_zigzag(&self) -> Option<&ZigZagSpec> {
        match self {
            Self::ZigZag(z) => Some(z),
            Self::Trim(_) | Self::Repeat(_) | Self::Twist(_) | Self::Bloat(_) => None,
        }
    }

    /// O irmão mutável do [`Self::as_zigzag`].
    pub fn as_zigzag_mut(&mut self) -> Option<&mut ZigZagSpec> {
        match self {
            Self::ZigZag(z) => Some(z),
            Self::Trim(_) | Self::Repeat(_) | Self::Twist(_) | Self::Bloat(_) => None,
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
            Self::Twist(_) => "Twist",
            Self::Bloat(_) => "Pucker & Bloat",
        }
    }

    /// **A tabela de tipos** — o menu "Add" do painel sai daqui, então acrescentar um efeito
    /// não toca no painel. A ordem é a dos variants.
    pub const KINDS: &'static [&'static str] = &[
        "Trim Path",
        "Zig Zag",
        "Repeater",
        "Twist",
        "Pucker & Bloat",
    ];

    /// Um efeito novo do tipo `kind`, no ponto NEUTRO. `None` se o índice não existe.
    ///
    /// Nascer neutro não é detalhe: é o que faz o clique em "Add" não mover um pixel.
    #[must_use]
    pub fn from_kind(kind: usize) -> Option<Self> {
        match kind {
            0 => Some(Self::Trim(TrimSpec::default())),
            1 => Some(Self::ZigZag(ZigZagSpec::default())),
            2 => Some(Self::Repeat(crate::fx_repeat::RepeatSpec::default())),
            3 => Some(Self::Twist(crate::fx_warp::TwistSpec::default())),
            4 => Some(Self::Bloat(crate::fx_warp::BloatSpec::default())),
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
            Self::Twist(_) => 3,
            Self::Bloat(_) => 4,
        }
    }

    /// Os parâmetros que este efeito oferece, na ordem em que o painel os desenha.
    ///
    /// ⚠️ Nunca mais de [`MAX_FX_PARAMS`] — há gate.
    #[must_use]
    pub fn params(&self) -> &'static [FxParam] {
        /// Um slider de fração `0..=1`, a forma mais comum.
        const fn frac(name: &'static str) -> FxParam {
            FxParam {
                name,
                min: 0.0,
                max: 1.0,
                toggle: false,
                integer: false,
            }
        }
        /// Uma caixinha (o valor só é 0 ou 1).
        const fn flag(name: &'static str) -> FxParam {
            FxParam {
                name,
                min: 0.0,
                max: 1.0,
                toggle: true,
                integer: false,
            }
        }
        const TRIM: &[FxParam] = &[frac("Start"), frac("End"), frac("Offset")];
        const ZIGZAG: &[FxParam] = &[
            // `Size` é PERCENTAGEM da forma: 100 = a média das dimensões dela. A faixa já era
            // `0..100`, mas em unidades de MUNDO — e era isso que a tornava inútil.
            FxParam {
                name: "Size",
                min: 0.0,
                max: 100.0,
                toggle: false,
                integer: false,
            },
            // **128, e o número é MEDIDO** (§0 do CLAUDE.md — um teto diz de que recurso é).
            // O recurso é o tempo de `cooked()`, que é chamado por vários consumidores por
            // frame; o custo é LINEAR nas cristas. Medido em release, círculo de 4 âncoras:
            //
            // | cristas |  8   |  32  |  64  | 128  | 256  |
            // |---------|------|------|------|------|------|
            // | ms/cook |0,019 |0,104 |0,219 |0,475 |0,902 |
            //
            // Enio pediu 128 (2026-07-18) e 128 custa 0,41 ms. Não há parede física antes de
            // ~2000 (o `MAX_SAMPLES` de guarda); quem quiser subir só tem de re-medir.
            FxParam {
                name: "Ridges",
                min: 1.0,
                max: 128.0,
                toggle: false,
                integer: true,
            },
            flag("Smooth"),
            flag("Rough"),
        ];
        const REPEAT: &[FxParam] = &[
            // **128, e o numero saiu da MEDICAO** (CLAUDE.md §0). Eu tinha escrito 32 por
            // conforto; medido numa silhueta de 24 ancoras, o custo de `cooked()` e' linear e
            // ridiculamente barato -- oito vezes menos que as cristas do Zig Zag no MESMO teto:
            //
            // | copias  |  2   |  8   |  16  |  32  |  64  | 128  |
            // |---------|------|------|------|------|------|------|
            // | ms/cook |0,0008|0,0042|0,0086|0,0144|0,0290|0,0614|
            //
            // ⚠️ **O cozimento NAO e' o recurso que limita isto** -- 0,06 ms a 128 nao limita
            // nada. O custo por medir e' o RENDER de 128 contornos (Vello), e quem quiser subir
            // tem de medir ESSE, nao este. 128 espelha o teto das cristas por simetria, nao por
            // ter batido numa parede.
            FxParam {
                name: "Copies",
                min: 1.0,
                max: 128.0,
                toggle: false,
                integer: true,
            },
            // Distancias em PERCENTAGEM da forma: `100` = uma largura-media. A mesma lei do
            // `Size` do Zig Zag, e pela mesma razao.
            FxParam {
                name: "Move X",
                min: -200.0,
                max: 200.0,
                toggle: false,
                integer: false,
            },
            FxParam {
                name: "Move Y",
                min: -200.0,
                max: 200.0,
                toggle: false,
                integer: false,
            },
            FxParam {
                name: "Rotate",
                min: -180.0,
                max: 180.0,
                toggle: false,
                integer: false,
            },
        ];
        // Um parametro cada. O Twist entrega o angulo na BORDA da forma; o Bloat e' uma
        // percentagem do raio de cada ponto (`-100` colapsa no centro, `100` duplica).
        const TWIST: &[FxParam] = &[FxParam {
            name: "Angle",
            min: -360.0,
            max: 360.0,
            toggle: false,
            integer: false,
        }];
        const BLOAT: &[FxParam] = &[FxParam {
            name: "Amount",
            min: -100.0,
            max: 100.0,
            toggle: false,
            integer: false,
        }];
        match self {
            Self::Trim(_) => TRIM,
            Self::ZigZag(_) => ZIGZAG,
            Self::Repeat(_) => REPEAT,
            Self::Twist(_) => TWIST,
            Self::Bloat(_) => BLOAT,
        }
    }

    /// O valor do parâmetro `i`, ou `0.0` se ele não existe.
    #[must_use]
    pub fn get(&self, i: usize) -> f64 {
        match (self, i) {
            (Self::Trim(t), 0) => t.start,
            (Self::Trim(t), 1) => t.end,
            (Self::Trim(t), 2) => t.offset,
            (Self::ZigZag(z), 0) => z.amplitude,
            (Self::ZigZag(z), 1) => z.ridges,
            (Self::ZigZag(z), 2) => f64::from(u8::from(z.smooth)),
            (Self::ZigZag(z), 3) => f64::from(u8::from(z.rough_seed.is_some())),
            (Self::Repeat(r), 0) => r.copies,
            (Self::Repeat(r), 1) => r.move_x,
            (Self::Repeat(r), 2) => r.move_y,
            (Self::Repeat(r), 3) => r.rotate,
            (Self::Twist(t), 0) => t.angle,
            (Self::Bloat(b), 0) => b.amount,
            _ => 0.0,
        }
    }

    /// Escreve o parâmetro `i`. Índice inexistente é no-op.
    ///
    /// Um parâmetro de CONTAGEM é arredondado **aqui**, na porta única de escrita — assim é o
    /// DOCUMENTO que guarda o inteiro, e o motor, o chip e o slider não podem discordar sobre
    /// que número está em uso. Arredondar só na exibição deixaria o chip a mostrar `37,42`
    /// enquanto a geometria desenha `37`.
    pub fn set(&mut self, i: usize, v: f64) {
        let v = if self.params().get(i).is_some_and(|p| p.integer) {
            v.round()
        } else {
            v
        };
        match (self, i) {
            (Self::Trim(t), 0) => t.start = v,
            (Self::Trim(t), 1) => t.end = v,
            (Self::Trim(t), 2) => t.offset = v,
            (Self::ZigZag(z), 0) => z.amplitude = v,
            (Self::ZigZag(z), 1) => z.ridges = v,
            (Self::ZigZag(z), 2) => z.smooth = v >= 0.5,
            // A seed é FIXA por enquanto: o que o artista liga é o *modo* Roughen. Um knob de
            // seed entra quando alguém quiser duas rugosidades diferentes na mesma cena.
            (Self::ZigZag(z), 3) => z.rough_seed = (v >= 0.5).then_some(1),
            (Self::Repeat(r), 0) => r.copies = v,
            (Self::Repeat(r), 1) => r.move_x = v,
            (Self::Repeat(r), 2) => r.move_y = v,
            (Self::Repeat(r), 3) => r.rotate = v,
            (Self::Twist(t), 0) => t.angle = v,
            (Self::Bloat(b), 0) => b.amount = v,
            _ => {}
        }
    }

    /// Aplica este efeito a um caminho inteiro — o contorno primário **e** cada subpath.
    ///
    /// Cada contorno é tratado **independentemente** (o modo "Individually" do After
    /// Effects). Um contorno que o efeito esvazie é DESCARTADO em vez de virar um contorno
    /// de zero vértices: um buraco vazio num compound não é geometria, é ruído para a
    /// booleana e para o preenchimento.
    #[must_use]
    pub fn apply(&self, path: &VecPath, ctx: &FxCtx) -> VecPath {
        let mut out = path.clone();
        match self {
            Self::Trim(spec) => {
                apply_per_contour(path, &mut out, |v, c| fx_trim::trim_contour(v, c, spec));
            }
            Self::ZigZag(spec) => {
                apply_per_contour(path, &mut out, |v, c| {
                    fx_zigzag::zigzag_contour(v, c, spec, ctx.ref_size)
                });
            }
            // ⚠️ O Repeater NÃO passa pelo `apply_per_contour`: ele multiplica a forma inteira,
            // e um buraco copiado por conta própria deixaria de ser buraco.
            // ⚠️ Sem `ctx`: o Repeater mede a ENTRADA dele, não o caminho autorado — ladrilhar
            // é uma operação sobre a coisa ladrilhada, e é isso que faz Repeater∘Repeater dar
            // uma grelha (o Blender constrói grelhas com dois modificadores Array empilhados).
            Self::Repeat(spec) => out = crate::fx_repeat::repeat_path(path, spec),
            Self::Twist(spec) => {
                apply_per_contour(path, &mut out, |v, c| {
                    crate::fx_warp::twist_contour(v, c, spec, ctx)
                });
            }
            Self::Bloat(spec) => {
                apply_per_contour(path, &mut out, |v, c| {
                    crate::fx_warp::bloat_contour(v, c, spec, ctx)
                });
            }
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
    for e in active {
        cur = Some(e.effect.apply(cur.as_ref().unwrap_or(path), &ctx));
    }
    let mut out = cur?;
    out.effects.clear();
    Some(out)
}

#[cfg(test)]
#[path = "effect_tests.rs"]
mod tests;
