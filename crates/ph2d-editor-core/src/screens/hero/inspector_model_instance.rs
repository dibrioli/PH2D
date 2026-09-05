//! ⭐⭐⭐ **A seção COMPONENT do Inspector** (ADR-0164 / F5) — *«o que esta cópia tem de diferente
//! da receita»*.
//!
//! # O buraco que ela fecha, e por que ele durou
//!
//! O modelo de override existe desde a F4.4 e é **inteiramente invisível**: o artista sabia que
//! uma peça estava overridada pelo COMPORTAMENTO (a receita deixou de a alcançar) e pelos verbos
//! do menu de linha. ⚠️ *Um estado que só se lê pelo que ele impede não é um estado que o artista
//! possa gerir* — e a lista de abertos deste módulo carregava a frase «nada na tela MOSTRA que
//! campo está overridado» desde 26/08.
//!
//! A F5.3 juntou-lhe um segundo estado, os **órfãos** (uma excepção cuja peça o mestre apagou), e
//! aí a ausência de superfície deixou de ser só desconforto: o plano manda-os **nunca apagar
//! sozinhos**, então sem um gesto eles acumulavam para sempre num sítio que ninguém via.
//!
//! # ⚠️ O que ela mostra é DERIVADO, e é por isso que não há estado aqui
//!
//! Os nomes vêm do `ph2d-component-desc` (o mesmo catálogo de que o `+` deriva a paleta) e o
//! conjunto vem do `ObjectInstance` da RAIZ da instância. Guardar rótulos aqui seria uma segunda
//! resposta a *«como se chama este componente?»*, e ela envelheceria no dia em que o catálogo
//! mudasse um nome.

/// Snapshot do estado de instância da entidade selecionada.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct InspectorInstanceInfo {
    pub entity_bits: u64,
    /// O nome da RECEITA de que esta cópia nasceu — o que o artista lê na Hierarquia.
    pub master_name: String,
    /// Os componentes **desta peça** que a cópia possui contra a receita, por nome de exibição.
    ///
    /// ⚠️ **Desta peça, e não da instância inteira.** O `ObjectInstance` mora na raiz e chaveia por
    /// `(peça, tipo)`; mostrar o conjunto todo numa peça diria ao artista que ele mexeu em coisas
    /// que estão noutro sítio da cópia. *A lista é do que está selecionado.*
    pub overridden: Vec<String>,
    /// ⭐⭐⭐ **As excepções da instância inteira que ficaram SEM ALVO** (F5.3) — uma linha por
    /// cada, com a peça que as tinha e o componente.
    ///
    /// ⚠️ **Da instância INTEIRA**, ao contrário do campo acima, e a diferença é o sujeito: um
    /// órfão não tem peça viva, então não há peça em que ele pudesse ser listado. É o mesmo sítio
    /// onde o Unity os põe (a lista do `PrefabInstance`, não a do objeto).
    ///
    /// ⛔⛔ **Era uma CONTAGEM até 2026-09-04, e isso não cumpria o critério 3 da F5:** ele pede que
    /// a excepção *«apareça»*, e um número responde *«há três»* à pergunta *«quais três?»*. Ao lado
    /// dela vive o botão que apaga as três — *limpar sem ver o que se limpa é o gesto destrutivo
    /// mais barato deste painel.*
    ///
    /// ⚠️ **Sem tecto, de propósito.** O painel rola; um tecto na LISTA com o botão a apagar
    /// **tudo** seria o pior dos dois mundos — esconderia exactamente as que o gesto destrói.
    pub orphan_rows: Vec<OrphanRow>,
    /// A entidade da RAIZ da instância — quem recebe o gesto de limpar os órfãos.
    pub root_bits: u64,
    /// ⭐⭐⭐ **Esta cópia é ela própria uma RECEITA** — uma variante (report do Enio, 2026-08-27).
    ///
    /// # ⚠️ O cartão nomeava a RELAÇÃO e nunca o que o objeto É
    ///
    /// Uma variante é `MasterRoot` **e** `InstanceOf` ao mesmo tempo: ela segue a base *e* é a
    /// receita das cópias dela. O cartão chamava-lhe *«Instance of "Badge"»*, que é verdade e
    /// **esconde a metade que decide** — quem edita uma variante muda todas as cópias dela, e o
    /// artista não tinha como saber isso pelo painel.
    ///
    /// ⚠️ É lido da **raiz**, e não da peça selecionada: uma peça dentro de uma variante pertence
    /// à variante, e a pergunta *«de que sou cópia?»* é sempre da raiz.
    pub is_variant: bool,
    /// ⭐⭐⭐ **A ESCADA do *Aplicar*** (F5 critério 4) — as receitas que este override pode
    /// alcançar, **da mais externa para a mais interna**.
    ///
    /// Vazia, ou com **um** degrau, quando não há escolha nenhuma a fazer: uma cópia não aninhada
    /// tem uma receita só, e *«aplicar ao mestre»* já é o item do menu. ⚠️ É o mesmo critério da
    /// fileira de versões — *um controlo que não escolhe nada não é um controlo*.
    ///
    /// ⚠️ **Ela é da PEÇA selecionada**, como a lista de `overridden` logo acima e pela mesma
    /// razão: a chave de override é `(peça, tipo)`, e a escada de uma peça não é a de outra.
    pub apply_levels: Vec<ApplyChoice>,
    /// Quantos degraus ficaram **fora** da tabela de ids — ⛔ escrito, nunca truncado em silêncio.
    pub apply_levels_beyond: usize,
}

/// ⭐⭐⭐ **Um degrau da escada do *Aplicar*** — a receita que receberia.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ApplyChoice {
    /// O `StableId` da receita — **a identidade**, que é o que o gesto precisa de saber.
    pub master: u64,
    /// O `Name` dela, que é o que o artista lê na Hierarquia.
    pub name: String,
    /// ⭐ Este é o degrau mais **INTERNO** — onde a peça é de facto definida.
    ///
    /// ⚠️ **É ele que decide o verbo do rótulo**, e a distinção é a do Unity: aplicar ali muda a
    /// receita da peça (*«all instances of the 'Vase' Prefab»*); aplicar num degrau de fora deixa
    /// o valor como **excepção** da cópia que vive dentro daquela receita.
    pub innermost: bool,
}

impl ApplyChoice {
    /// ⭐⭐ **O rótulo do botão** — e as duas frases dizem coisas diferentes, de propósito.
    ///
    /// ⚠️ **A redacção é a do Unity, e isso é uma escolha**: *Apply to* e *Apply as override in*
    /// são as palavras que o artista já encontrou em todo o tutorial que leu, e inventar as nossas
    /// obrigá-lo-ia a re-aprender a única parte deste modelo que ele provavelmente já sabe.
    ///
    /// ⛔ **O nome atravessa VERBATIM** — é a lei do [`InspectorInstanceInfo::provenance`]: comer
    /// pedaços de um nome que o artista escreveu é o app a corrigi-lo.
    #[must_use]
    pub fn label(&self) -> String {
        let name = &self.name;
        if self.innermost {
            format!("Apply to \u{201c}{name}\u{201d}")
        } else {
            format!("Apply as override in \u{201c}{name}\u{201d}")
        }
    }
}

/// ⭐⭐ **Uma excepção SEM ALVO, como o cartão precisa de a ver.**
///
/// ⚠️ **As duas metades são precisas:** o componente diz *o que* se perde e a peça diz *de onde*.
/// Com duas peças apagadas, uma lista só de componentes lê-se `Sprite · Sprite · Transform` e não
/// responde a pergunta que o artista tem antes de carregar em *Clear*.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct OrphanRow {
    /// O nome de exibição do componente, do mesmo catálogo de que o `+` deriva a paleta.
    pub component: String,
    /// O `Name` que a peça tinha quando morreu. **Vazio** quando ela não tinha nome — ver
    /// [`Self::label`].
    pub piece: String,
    /// ⭐⭐⭐ **A IDENTIDADE da excepção** — o `StableId` da peça morta e o `type_id` do componente,
    /// que juntos são a chave por onde o gesto a alcança.
    ///
    /// ⚠️ **Sem eles a linha não é endereçável, e uma linha que não é endereçável não pode ter
    /// gesto próprio.** Os dois campos acima são RÓTULOS: duas peças podem ter tido o mesmo nome, e
    /// um *«apaga a que diz `Sprite — was on "Arm"`»* escolheria a errada em silêncio. *O que se
    /// mostra é o nome; o que se aponta é a chave.*
    pub piece_id: u64,
    /// O `type_id` do componente — a outra metade da chave.
    pub type_id: u64,
}

impl OrphanRow {
    /// ⚠️ **A frase vive no MODELO**, como a `provenance` e a `summary`: escrevê-la no pintor poria
    /// a escolha num sítio que nenhum gate de modelo alcança.
    ///
    /// ⛔ **Sem peça, sem a metade que a nomeia** — uma frase `was on ""` diria ao artista que a
    /// peça se chamava vazio.
    #[must_use]
    pub fn label(&self) -> String {
        let c = &self.component;
        if self.piece.is_empty() {
            return c.clone();
        }
        let p = &self.piece;
        format!("{c} \u{2014} was on \u{201c}{p}\u{201d}")
    }
}

/// Uma versão do componente que esta cópia pode passar a ser.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct VariantChoice {
    /// O `StableId` do mestre — o que o gesto de troca precisa de saber.
    pub master: u64,
    /// O `Name` dele, que é o que o artista lê na Hierarquia.
    pub label: String,
    /// Esta é a versão vigente.
    pub current: bool,
}

impl InspectorInstanceInfo {
    /// ⭐ **A linha de proveniência** — *«o que este objeto é, e de quem»*.
    ///
    /// ⚠️ **Duas palavras, e a diferença não é cosmética:** *Instance* é uma cópia que a receita
    /// alcança; *Variant* é ela própria uma receita, que a base alcança e que alcança as cópias
    /// dela. Editar as duas tem consequências diferentes, e é o painel que tem de o dizer.
    /// ⛔⛔ **E o nome vem CURTO** (report do Enio com foto, 2026-08-31: *«Card com Labels
    /// emboladas»*). O `master_name` é o `Name` cru da receita — `Canvas{Size=Small} Variant` —, e
    /// a frase inteira **quebrava em duas linhas** dentro de um cartão cuja altura é contada em
    /// linhas de texto: a 2.ª linha da proveniência e o resumo eram pintados no MESMO `y`.
    ///
    /// ⚠️ **O nome atravessa VERBATIM.** Ele já foi cortado — quando as propriedades viviam no
    /// `Name`, a frase inteira rebentava a caixa. O mecanismo foi recusado e está adiado; sem
    /// gramática não há nada a cortar, e comer pedaços de um nome que o artista escreveu seria o
    /// app a corrigi-lo.
    #[must_use]
    pub fn provenance(&self) -> String {
        let what = if self.is_variant {
            "Variant"
        } else {
            "Instance"
        };
        let name = &self.master_name;
        format!("{what} of \u{201c}{name}\u{201d}")
    }

    /// ⭐⭐ **Os degraus que o cartão de facto PINTA** — e as duas condições são leis, não
    /// economia de espaço.
    ///
    /// ⛔ **Sem excepção nenhuma nesta peça não há o que aplicar**: um botão permanentemente inerte
    /// é ruído que o artista aprende a ignorar — a mesma lei do gesto dos órfãos.
    ///
    /// ⛔ **Com UM degrau não há escolha**: *«aplicar ao mestre»* já é o item do menu da linha, e um
    /// botão único aqui seria a segunda porta para o mesmo verbo. É o critério da fileira de
    /// versões, dito outra vez: *um controlo que não escolhe nada não é um controlo.*
    #[must_use]
    pub fn apply_rows(&self) -> &[ApplyChoice] {
        if self.overridden.is_empty() || self.apply_levels.len() < 2 {
            return &[];
        }
        &self.apply_levels
    }

    /// A linha que resume o estado, para quem não quer ler a lista.
    ///
    /// ⚠️ **Sem excepção nenhuma ela diz «segue a receita»**, e isso é informação: é a diferença
    /// entre *«não mexi nesta»* e *«mexi e não vejo onde»*, que era exactamente o que faltava.
    /// Quantas excepções sem alvo a instância tem — **derivado das linhas**, nunca contado à
    /// parte: um número ao lado da lista é a segunda resposta que discorda dela no dia em que uma
    /// entrada for saltada. É o que o botão *Clear* promete apagar.
    #[must_use]
    pub fn orphans(&self) -> usize {
        self.orphan_rows.len()
    }

    #[must_use]
    pub fn summary(&self) -> String {
        match (self.overridden.len(), self.orphans()) {
            // ⚠️ Numa variante a palavra é a mesma e o sujeito é outro: ela segue a **base**. Dizer
            // «segue o componente» sobre uma receita seria a mesma ambiguidade um nível acima.
            (0, 0) if self.is_variant => "Follows its base".to_string(),
            (0, 0) => "Follows the component".to_string(),
            (0, n) => format!("Follows the component \u{b7} {n} unused"),
            (k, 0) => format!("{k} override(s) on this piece"),
            (k, n) => format!("{k} override(s) on this piece \u{b7} {n} unused"),
        }
    }
}
