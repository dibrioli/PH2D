//! **A IDENTIDADE DE UM TOKEN NUMÉRICO** — o que a camada de override autora, e o acessor VIVO que
//! o lê (plano UI/UX W4c.1).
//!
//! # A família é *o que se mede em PIXELS*, e a fronteira não é gosto
//!
//! [`Spacing`], [`Radius`] e [`StrokeToken`] respondem à mesma pergunta na mesma unidade — *quantos
//! px?* —, então partilham o editor, a porta de escrita e o slot do arquivo. É isso que faz deles
//! **uma** família em vez de três.
//!
//! ⚠️ **O que ficou de FORA, e o motivo de cada um** (nomear é mais honesto do que uma lista que
//! por acaso parou onde parou):
//!
//! - **`Motion`** (`Duration`) mede-se em **milissegundos**. Outra unidade é outra régua, outro
//!   passo de arrasto e outro campo — pô-la aqui daria um chip cujo número o artista lê como px.
//! - **`Density`** já É uma escolha do artista (o modo de linha), não um valor de escala.
//! - **`chrome.*`** são consts livres, sem identidade de token (`id()`/`ALL`): dar-lhes uma é uma
//!   wave própria, e sem ela não há chave estável para o arquivo guardar.
//!
//! # A FÁBRICA é `const fn`; o VIVO não precisa de ser (W4c.2)
//!
//! A parede que segurou a W4c.1 nunca foi de performance ([plano W4c](../../../docs/Vector%20Module/Estudos/PLANO_UI_UX_padrao_figma.md)):
//! `const PAD: f32 = Spacing::Sm.px();` **não pode** chamar uma fn não-const. A W4c.1 respondeu
//! mantendo `px()` na fábrica e pondo um `px_live(theme)` ao lado; a W4c.2 **mediu** e virou a
//! resposta ao contrário — `Spacing::px()` **é** o valor vivo (lê a tabela do
//! [`crate::num_runtime`]) e [`Spacing::factory_px`] é a fábrica, `const fn` como sempre.
//!
//! ⚠️ **A medição que decidiu:** a escala é lida em ~1200 sítios e só **13** são `const` items.
//! Manter `px()` como fábrica obrigaria os 1187 restantes a mudar de nome para ficarem vivos;
//! virando a resposta, eles ficam vivos **sem serem tocados** e os treze quebram na compilação —
//! o compilador enumera-os, e nenhuma lista escrita à mão pode envelhecer.
//!
//! # ⚠️ O TETO de um valor autorado pertence a quem o CONSOME — e agora existe consumidor
//!
//! A porta recusa o que não é um comprimento (não-finito, negativo — ver
//! [`crate::num_overrides::set_num_override`]) e **não inventa um máximo**. O que a W4c.2 mediu
//! antes de ligar a leitura viva está no [`crate::num_runtime`]: **o painel de Tokens desenha-se a
//! si mesmo com estes tokens**, então o teto é sobre a *alcançabilidade do Reset*, não sobre um
//! recurso — e a medição está no gate `the_panel_survives_an_absurd_scale`.

use crate::radius::Radius;
use crate::spacing::Spacing;
use crate::stroke::StrokeToken;
use crate::theme::Theme;

/// Um token de escala em PIXELS — qual família, qual degrau.
///
/// ⚠️ Ele **embrulha** os enums que já existem em vez de os repetir numa lista plana: um degrau
/// novo em [`Spacing`] passa a existir aqui sem ninguém o copiar, e a chave dele é cobrada pelo
/// compilador (ver [`num_tokens!`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumToken {
    /// Um degrau da escala de espaçamento.
    Spacing(Spacing),
    /// Um degrau da escala de raio de canto.
    Radius(Radius),
    /// Um degrau da escala de espessura de traço.
    Stroke(StrokeToken),
}

/// **A LISTA — e ela não pode driftar dos enums, porque o compilador a cobra.**
///
/// O `match` que a macro gera para a [`NumToken::key`] é **exaustivo** sobre `(família, degrau)`:
/// acrescentar um `Spacing::Xl5` sem o pôr aqui **não compila**. É a mesma propriedade que o
/// `color_tokens!` tem, e a razão de a lista ser uma macro e não uma tabela `static`: uma tabela
/// escrita à mão ao lado dos enums envelhece em silêncio, e o modo de falha seria um token que o
/// artista não consegue editar.
///
/// ⚠️ **A chave é PONTUADA** (`"spacing.md"`) e a de cor não é (`"accent"`). Não é inconsistência:
/// as chaves de cor já viajam em todo projeto salvo e mudá-las re-pintaria arte que ninguém tocou.
/// O ponto é também a forma que o **DTCG** fala (W4c.5), e ele é o que torna as duas famílias
/// **provavelmente disjuntas** — há gate a afirmá-lo, porque as duas partilham o slot do arquivo.
macro_rules! num_tokens {
    ( $( $variant:ident : $ty:ident { $( $member:ident => $key:literal ),* $(,)? } ),* $(,)? ) => {
        impl NumToken {
            /// Todos os tokens numéricos, na ordem em que o painel os lista.
            pub const ALL: &'static [Self] = &[ $( $( Self::$variant($ty::$member) ),* ),* ];

            /// A chave estável deste token — a identidade que o **arquivo** guarda.
            ///
            /// ⚠️ Nunca o índice do variant: guardá-lo amarraria todo projeto salvo à ORDEM desta
            /// lista, e inserir um degrau no meio de uma escala re-escreveria os valores autorados
            /// para os tokens errados. É a mesma lei do [`crate::ColorToken::key`].
            #[must_use]
            pub const fn key(self) -> &'static str {
                match self { $( $( Self::$variant($ty::$member) => $key ),* ),* }
            }

            /// A INVERSA da [`NumToken::key`] — gerada pela MESMA lista, e é isso que importa: as
            /// duas não podem discordar sobre que chaves existem.
            #[must_use]
            pub fn from_key(key: &str) -> Option<Self> {
                match key { $( $( $key => Some(Self::$variant($ty::$member)), )* )* _ => None }
            }

            /// A posição deste token em [`NumToken::ALL`] — o índice da tabela achatada de runtime.
            ///
            /// ⚠️ Gerado pela MESMA lista que produz o `ALL`, então não há como um degrau novo
            /// entrar num e faltar no outro. E ⚠️ **isto NÃO é identidade durável** — o arquivo
            /// guarda a [`NumToken::key`], porque inserir um degrau no meio de uma escala move
            /// todos os índices depois dele.
            #[must_use]
            pub const fn index(self) -> usize {
                let mut i = 0;
                $( $(
                    if matches!(self, Self::$variant($ty::$member)) { return i; }
                    i += 1;
                )* )*
                let _ = i;
                0
            }
        }
    };
}

num_tokens! {
    Spacing: Spacing {
        Xxs => "spacing.xxs",
        Xs => "spacing.xs",
        Sm => "spacing.sm",
        Md => "spacing.md",
        Lg => "spacing.lg",
        Xl => "spacing.xl",
        Xl2 => "spacing.2xl",
        Xl3 => "spacing.3xl",
        Xl4 => "spacing.4xl",
    },
    Radius: Radius {
        Xs => "radius.xs",
        Sm => "radius.sm",
        Md => "radius.md",
        Lg => "radius.lg",
        Xl => "radius.xl",
        Xl2 => "radius.2xl",
        Full => "radius.full",
    },
    Stroke: StrokeToken {
        Hairline => "stroke.hairline",
        Thin => "stroke.thin",
        Default => "stroke.default",
        Thick => "stroke.thick",
        Heavy => "stroke.heavy",
    },
}

impl NumToken {
    /// O valor de **FÁBRICA** — a tabela gerada do `tokens.json`, sem passar pela camada.
    ///
    /// ⚠️ Ele delega para o `px()` de cada família em vez de reproduzir os números: uma segunda
    /// tabela aqui seria a que discorda do `design_token_sync`, que mede exactamente aquela.
    #[must_use]
    pub const fn factory_px(self) -> f32 {
        match self {
            Self::Spacing(s) => s.factory_px(),
            Self::Radius(r) => r.factory_px(),
            Self::Stroke(s) => s.factory_px(),
        }
    }

    /// **O acessor VIVO** — o que o artista autorou neste modo, ou a fábrica.
    ///
    /// É o irmão numérico do [`crate::ColorToken::resolve`], e a forma é a mesma: a cadeia de
    /// aliases é seguida pela camada, e o que volta é ou um número ou o token em que ela terminou —
    /// cuja **fábrica** é lida aqui.
    ///
    /// ⚠️ Com a camada vazia isto é **uma leitura de bool** e o resultado é o `factory_px`, bit a
    /// bit. É o que torna a camada gratuita para quem nunca abriu o painel.
    #[must_use]
    pub fn px(self, theme: Theme) -> f32 {
        match crate::num_overrides::resolved_num_override(theme, self) {
            Some(crate::num_overrides::AuthoredNum::Px(v)) => v,
            Some(crate::num_overrides::AuthoredNum::Factory(t)) => t.factory_px(),
            None => self.factory_px(),
        }
    }
}

/// ⚠️ **O acessor VIVO por-família NÃO mora aqui, e a ausência é a decisão.**
///
/// A W4c.1 gerou um `Spacing::px_live(theme)` ao lado do `px()` de fábrica, à espera de que a
/// W4c.2 trocasse os sítios de uso um a um. A medição da W4c.2 derrubou esse plano: são **~1200**
/// sítios de leitura contra **13** `const` items, e enfiar o modo em cada um deles seria responder
/// mil e duzentas vezes a uma pergunta que o app responde **uma** vez por quadro.
///
/// Hoje `Spacing::px()` **é** o acessor vivo (lê a tabela do [`crate::num_runtime`]) e
/// `Spacing::factory_px()` é o de fábrica. Ter os dois **mais** um `px_live(theme)` daria três
/// portas para duas perguntas, e a terceira seria a que alguém chama por engano.

#[cfg(test)]
#[path = "num_tests.rs"]
mod tests;
