//! **O BINDING DE TOKEN** — uma propriedade da forma deixa de ser um literal e passa a
//! REFERENCIAR um token, resolvido por MODO.
//!
//! É a feature de maior alavancagem do plano de UI/UX (§4/W4), e aqui ela tem um segundo consumidor
//! que nenhuma outra ferramenta de design tem: **o próprio editor**. A mesma tabela
//! (`docs/design/tokens.json` → `ph2d-tokens`) que veste os 44 widgets do app passa a vestir a arte
//! do artista, então trocar de modo re-veste **o card que ele desenhou E o app inteiro**.
//!
//! # A tabela é LATERAL, e essa é a decisão inteira
//!
//! ⚠️ **Nenhum campo é apendado a `Paint`, a `StrokeSpec` ou a `VecShape`.** Se o binding morasse
//! dentro do `Paint`, **todo** save de vetor mudaria de forma e o `VEC_SCENE_SCHEMA` bumparia por
//! uma feature que 90% dos documentos não usa — e um bump **recusa todo projeto já salvo**. Um
//! componente NOVO cunha `stable_type_id = blake3(NOME)[..8]` e não move nada.
//!
//! É a mesma lei que o repo já aplica em quatro lugares (*"todo canal novo é side-metadata no
//! registry, nunca contrato"* — os 6 canais que o `KernelResolver` ganhou sem mover
//! `NodeOp`/`OpResolver`/`NodeManifest`), e o precedente direto é o [`crate::VecStrokeProfile`]
//! (ADR-0148): *o perfil é um componente ECS, não um campo do `StrokeSpec`*.
//!
//! # A chave é o NOME do token, nunca o índice
//!
//! ⚠️ [`TokenRef`] guarda a chave kebab-case (`"accent"`), que é a identidade estável de um sistema
//! de tokens — a que o `tokens.json` usa, a que o DTCG fala, e a que
//! [`ph2d_tokens::ColorToken::from_key`] resolve. Guardar o índice do variant amarraria todo
//! projeto salvo à ORDEM da lista, e inserir um token no meio dela re-pintaria arte que ninguém
//! tocou.
//!
//! Corolário que vale para a wave seguinte: quando a tabela virar autorável (tokens do ARTISTA,
//! aliases, math), eles entram por aqui **sem migração** — a chave já é o endereço.
//!
//! # O literal SOBREVIVE
//!
//! Bindar não apaga a cor autorada: o `Paint` do documento fica onde está, e o binding é uma
//! camada por cima na hora de DESENHAR (a costura fonte ≠ cozido do ADR-0121, agora na TINTA).
//! Desbindar devolve exatamente a cor que estava lá — sem isso, experimentar um token custaria a
//! escolha anterior.
//!
//! # ⚠️ Um token de ESCALA fala PIXELS, e este documento fala MUNDO (W4c.4)
//!
//! Uma cor é adimensional e atravessa a fronteira sem conversão; um comprimento não. Os três alvos
//! de escala que o plano nomeia vivem em **unidades de mundo** (`StrokeSpec::width`,
//! `VecLayout::gap`, `VecVertex::corner_radius`), e um [`ph2d_tokens::NumToken`] vale **pixels**.
//!
//! ⚠️ **Ler o número do token como se fosse mundo erra por duas ordens de grandeza, e o número
//! está medido:** `stroke.default = 1.5` px viraria 1,5 unidades, e a moldura de telefone mede
//! **8** unidades no lado maior (`ph2d_tool_vector::frames::LONG_SIDE`) — um traço com **19% da
//! altura do aparelho**. `radius.full = 999` daria 125 molduras.
//!
//! A régua existe e **já tem um dono declarado**: `ProjectSettings::pixels_per_meter` (ADR-0131
//! D4 — *"a única px→m é a do PROJETO; um 2º `PIXELS_PER_METER` seria a segunda porta que
//! diverge"*). Com o default de 100, `stroke.default` vale 0,015 unidades = 1,58 pt naquela
//! moldura, que é o cabelo que o token promete.
//!
//! ⚠️ E ela **não** é o `px_to_world` da câmera, embora a row *Width* do painel fale nele: aquele
//! número é px de TELA no zoom do momento (`vector_bridge.rs`, *"a largura viaja em px de tela na
//! tool e em MUNDO no documento"*), então resolver por ele faria a arte SALVA depender de quão
//! perto o artista estava quando o token mudou.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **Que propriedade desta forma está bindada.**
///
/// ⚠️ Os discriminantes são valores de ARQUIVO e a lista é **append-only**: um variant novo entra
/// no fim, nunca no meio. Inserir um no meio re-interpretaria todo binding já salvo — o `Fill` de
/// ontem viraria o `StrokeColor` de hoje, em silêncio e com o projeto abrindo normalmente.
///
/// ⚠️ **`CornerRadius` continua FORA, e o motivo mudou** (medido 2026-08-06, W4c.4). A nota
/// anterior dizia que as três propriedades de ESCALA esperavam *"o canal que as resolve"*, e o
/// canal chegou (`ph2d_tokens::NumToken`, autorável desde a W4c.1). O que falta ao raio é outra
/// coisa: ele é **por-VÉRTICE** (`VecVertex::corner_radius`, autorado pela alça do modo Node e
/// pela ferramenta Fillet) e o painel **não tem um controle por-FORMA** para ele. Um binding é
/// por-forma, então `BoundProp::CornerRadius` seria hoje um alvo que nada preenche — a mesma
/// frase, sobre um vão diferente.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u16)]
pub enum BoundProp {
    /// O preenchimento (`VecPath::fill`).
    Fill = 0,
    /// A cor do traço (`VecPath::stroke.paint`).
    StrokeColor = 1,
    /// A **espessura** do traço (`VecPath::stroke.width`).
    StrokeWidth = 2,
    /// O vão do auto layout no eixo PRINCIPAL (`VecLayout::gap[0]`).
    LayoutGapMain = 3,
    /// O vão do auto layout no eixo TRANSVERSAL (`VecLayout::gap[1]`).
    LayoutGapCross = 4,
}

impl BoundProp {
    /// **A INVERSA do discriminante** — o código de arquivo de volta ao alvo.
    ///
    /// ⚠️ Ela existe porque o discriminante É o código que a UI usa para nomear as opções do
    /// picker (`ph2d_editor_core::ids::vector_token_option_id`), e sem esta porta a shell teria de
    /// escrever um `match` paralelo — uma segunda lista, que envelhece no dia em que um alvo novo
    /// entra só numa delas e o clique deixa de chegar a lado nenhum.
    ///
    /// ⚠️ **De que TABELA cada alvo se serve NÃO se pergunta aqui**, e a ausência é deliberada: é
    /// uma pergunta de UI (*que lista o picker pinta?*), e a resposta mora na crate que o painel e
    /// a shell alcançam — `ph2d_editor_core::ids::TOKEN_SLOTS`. Duas respostas ofereceriam cores
    /// para escolher uma espessura no dia em que uma delas ganhasse um membro sozinha.
    #[must_use]
    pub const fn from_code(code: u16) -> Option<Self> {
        match code {
            0 => Some(Self::Fill),
            1 => Some(Self::StrokeColor),
            2 => Some(Self::StrokeWidth),
            3 => Some(Self::LayoutGapMain),
            4 => Some(Self::LayoutGapCross),
            _ => None,
        }
    }

    /// As propriedades que se podem bindar hoje — a lista que o painel OFERECE.
    ///
    /// Ela é DADO pelo mesmo motivo que o `ColorToken::ALL`: uma segunda lista escrita à mão na UI
    /// nasce desatualizada no dia em que esta ganhar um membro.
    pub const ALL: &'static [Self] = &[
        Self::Fill,
        Self::StrokeColor,
        Self::StrokeWidth,
        Self::LayoutGapMain,
        Self::LayoutGapCross,
    ];

    /// Rótulo curto, para a UI dizer QUAL propriedade está presa.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Fill => "Fill",
            Self::StrokeColor => "Stroke",
            Self::StrokeWidth => "Width",
            Self::LayoutGapMain => "Gap",
            Self::LayoutGapCross => "Gap (cross)",
        }
    }
}

/// A chave kebab-case de um token (`"accent"`, `"bg-2"`, `"text-1"`).
pub type TokenRef = String;

/// **As propriedades desta forma que seguem um token.**
///
/// Ausência do componente = nada bindado, e o desenho é **byte-idêntico** ao mundo pré-token — que
/// é o caso de todo documento que já existe.
///
/// ⚠️ **Uma propriedade tem no máximo UM token.** As entradas são mantidas ordenadas por
/// [`BoundProp`] e `set` SUBSTITUI: duas entradas para o mesmo alvo seriam duas respostas a *"de
/// que cor é este preenchimento?"*, e qual vence dependeria da ordem de inserção — um fato que o
/// artista não vê e não controla.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VecBindings {
    /// Os pares `(propriedade, token)`, ordenados pela propriedade.
    pub entries: Vec<(BoundProp, TokenRef)>,
}

impl VecBindings {
    /// O token que dirige esta propriedade, se houver.
    #[must_use]
    pub fn get(&self, prop: BoundProp) -> Option<&str> {
        self.entries
            .iter()
            .find(|(p, _)| *p == prop)
            .map(|(_, t)| t.as_str())
    }

    /// Prende a propriedade a um token, SUBSTITUINDO o que lá estava.
    pub fn set(&mut self, prop: BoundProp, token: impl Into<TokenRef>) {
        let token = token.into();
        match self.entries.iter_mut().find(|(p, _)| *p == prop) {
            Some(slot) => slot.1 = token,
            None => {
                self.entries.push((prop, token));
                self.entries.sort_by_key(|(p, _)| *p);
            }
        }
    }

    /// Solta a propriedade — ela volta a valer o literal que o documento sempre guardou.
    pub fn clear(&mut self, prop: BoundProp) {
        self.entries.retain(|(p, _)| *p != prop);
    }

    /// Nada preso ⇒ o componente não tem razão de existir.
    ///
    /// Quem edita usa isto para DESANEXAR em vez de deixar um componente vazio: um vazio viaja no
    /// save, entra no diff do undo e faz duas cenas logicamente iguais compararem diferente.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl SimComponent for VecBindings {}

#[cfg(test)]
#[path = "vec_bindings_tests.rs"]
mod tests;
