//! ⭐⭐⭐ **O `SignalActions`** — a tabela **nome → acção**, e o item que o levantamento chama de
//! *«o mais importante»* do TOP-20 (#5).
//!
//! # O que ele fecha
//!
//! Até 2026-09-09 o `ph2d-runtime` publicava sinais de **cinco** origens (timeline · contacto ·
//! sensor · animação · **relógio**) e tinha **três** consumidores, todos de diagnóstico: um toast,
//! uma linha de terminal e a máquina de estados de UI. *Um sinal não fazia nada acontecer na cena.*
//!
//! Este componente é o consumidor que faltava: uma lista de linhas `(sinal, alvo, verbo)` que o
//! artista escreve no Inspector. Com ela, **um sinal vira jogo sem uma linha de script** — que é a
//! frase com que o levantamento justifica o bloco 2–5 inteiro.
//!
//! # ⚠️ As leis que este módulo herda, e onde cada uma foi paga
//!
//! - ⭐⭐ **O alvo é o NOME, nunca os bits.** É a lei do `stable_name_id` escrita no CLAUDE.md §5:
//!   *o undo respawna tudo com bits novos, e bits dentro dos bytes de um componente envenenam o
//!   próprio undo*. ⇒ o alvo viaja como `String`, e a resolução é por [`crate::stable_id`].
//!
//!   ⭐ E há um segundo ganho, medido no desenho: **uma cópia de um prefab funciona sem rewiring**
//!   quando o alvo é vazio (= *este objecto*), que é o caso comum de uma porta, de uma armadilha
//!   ou de um inimigo.
//! - **Um sinal sem nome nunca dispara** — o espelho exacto da lei do produtor (*«um produtor sem
//!   nome não fala, em vez de falar com um nome vazio»*). Aqui: um consumidor sem nome não escuta,
//!   em vez de escutar tudo.
//! - ⛔ **NÃO há verbo que EMITA um sinal**, e a ausência é a decisão: seria a classe inteira dos
//!   laços (`a` dispara `b` dispara `a`), e não existe hoje consumidor que a peça. Quando existir,
//!   ela entra com um orçamento de profundidade, não com um `if`.
//!
//! # ⚠️ O que a RESOLUÇÃO faz, e o que ela deliberadamente NÃO faz
//!
//! [`resolve`] é **pura sobre o mundo**: ela lê, casa nomes e devolve a lista de efeitos. Quem os
//! **aplica** é a shell, e a razão é o undo — escrever a [`crate::Visibility`] de um objecto é
//! escrever um componente **registado**, logo um passo de `Ctrl+Z` por cada porta que abre. A shell
//! declara essas escritas ao ledger de pré-visualização (*o documento é o valor AUTORADO; o que um
//! motor escreve agora é pré-visualização*), e essa maquinaria não vive nesta crate.
//!
//! ⇒ **duas metades, duas casas**, e a fronteira é exactamente a que o `tick_timers` já usa: a lei
//! devolve factos, a ponte publica-os.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::{Entity, Name, World};
use ph2d_tags::{TagId, TagTree};

/// Quantas acções uma entidade pode ter.
///
/// ⚠️ **O número sai do PAINEL**, como o [`crate::TIMERS_MAX`] e pela mesma lei: *um modelo que
/// aceita o que o painel não mostra produz estado inalcançável*. A secção do Inspector desenha uma
/// linha por acção dentro do dock, e `16` é o que cabe sem a secção sozinha passar a altura útil da
/// coluna.
pub const SIGNAL_ACTIONS_MAX: usize = 16;

/// **O que uma acção FAZ.**
///
/// ⛔ **A lista é o que tem SINK hoje, e não o que seria bonito ter.** Um verbo cujo consumidor não
/// existe é um controlo morto com cara de feature — o defeito que a caça de 2026-08-30 mediu em 34
/// controlos. Cada entrada abaixo escreve num componente que existe e que alguém lê.
///
/// # ⏳ Os que ficam de FORA, com o motivo
///
/// - ✅ **Tocar um som DEIXOU de estar aqui** — a recusa dizia *«não existe `AudioSource2D` na
///   árvore»* e ela dissolveu no dia seguinte, porque **esta linha construiu-o** (TOP-20 #4). É o
///   §0.0 outra vez: *quem move o número que tornava algo inalcançável tem de reconferir a nota.*
///   ⇒ os verbos `PlaySound`/`StopSound` estão na lista abaixo.
/// - **Tocar uma animação nomeada.** ⏳ O sink existe (`SpriteAnimator::current`/`playing`), e o
///   que o barra é uma **colisão de granularidade**: o ledger já conduz aquele componente pelo
///   `Driver::SpriteAnim`, cujo recorte são os três campos do RELÓGIO mais a célula. Um segundo
///   motor sobre os campos autorados do mesmo componente precisa do seu próprio recorte, e isso é
///   desenho, não uma linha.
/// - **Nascer um objecto** (*spawn*). ⏳ É o item **#11** da fila e precisa de uma referência a uma
///   receita, que é outro campo e outro picker.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SignalVerb {
    /// **Arranca um relógio do alvo.** O `arg` é o NOME do timer; vazio = **todos** os do alvo.
    ///
    /// ⭐ É o verbo que fecha o buraco que a W3 do Timer deixou nomeado: até aqui o `autostart` era
    /// o único caminho para um timer começar, e um timer com ele desligado era inalcançável.
    #[default]
    StartTimer,
    /// **Pára um relógio do alvo**, guardando o progresso — é o *Pause* do Godot, não o *Stop*, e a
    /// escolha é a que a lei pura do timer já declara (`running = false` não zera o `elapsed`).
    StopTimer,
    /// Mostra o alvo.
    Show,
    /// Esconde o alvo.
    Hide,
    /// Inverte o que o alvo está a fazer agora.
    ToggleVisibility,
    /// **Toca o som do alvo** (o `AudioSource2D` dele). ⛔ Um alvo sem fonte de som não faz nada —
    /// não é erro, é a mesma lei de um alvo que não existe.
    ///
    /// ⚠️ **Ele NÃO lê o `arg`, e é deliberado:** o ficheiro é do componente, não da linha da
    /// tabela. Pôr um caminho aqui daria **duas** respostas a *«que som é este objecto?»*, e a que
    /// o artista vê no Inspector seria a que envelhece.
    PlaySound,
    /// **Cala o som do alvo** — pára as vozes que ele tem a soar agora.
    StopSound,
    /// ⭐⭐⭐ **Soma ao contador do alvo** (TOP-20 #20) — o verbo que faz um placar existir.
    ///
    /// O `arg` é quanto somar (um inteiro com sinal); vazio ou ilegível vale **`1`**, que é o caso
    /// comum (*«apanhei uma moeda»*). ⚠️ **Somar `0` é INERTE e é contado como tal** — um valor que
    /// não move nada não pode ler-se como aplicado.
    ///
    /// ⚠️ **APENDADO no fim, e isso é obrigatório:** o `SignalVerb` é `#[repr(u8)]` e viaja no
    /// documento pelo postcard, que é POSICIONAL — uma variante no meio reescreveria o sentido de
    /// todas as linhas de acção já gravadas, em silêncio.
    AddToCounter,
    /// ⭐⭐⭐ **Tira o alvo da cena** (suplente #24) — o verbo que faltava para um golpe acabar em
    /// alguma coisa.
    ///
    /// ⚠️ **Ele ANUNCIA e nunca apaga:** produz um [`crate::Death`] pelo despachante que já existe,
    /// com a causa [`crate::DeathCause::Killed`]. ⛔ *Dois despachantes de morte seriam duas
    /// respostas a «quando é que isto sai da cena?»* — a frase que a fase da fábrica já tem escrita.
    ///
    /// ⚠️⚠️ **E ele só tira quem NASCEU numa corrida** ([`crate::is_transient`]), com a fronteira
    /// FORÇADA e não escolhida: apagar um objecto do documento durante a corrida **tira-o do
    /// documento** (a captura vê-o sumido e o `Ctrl+Z` herda a remoção), que é a lei *«o que
    /// acontece numa corrida não é documento»* invertida. O precedente é do #14, com a frase
    /// inteira: *«um projéctil que ele pôs na cena à mão é documento, e apagá-lo destruiria
    /// autoria»*. ⇒ um alvo de documento é **recusado em voz** e contado como inerte.
    ///
    /// ⛔ **Ele NÃO lê o `arg`** — *«tira este»* não tem parâmetro.
    Destroy,
    /// ⭐⭐⭐ **A CORRIDA RECOMEÇA** — o sétimo passo do laço de um jogo, e o único sem porta até
    /// aqui.
    ///
    /// A sonda do §5.0 (`mede_o_que_a_composicao_ja_da_ao_fim_de_jogo`) mediu-o pelo caminho do
    /// produto: ligando os **nove** verbos anteriores ao sinal de fim, um contador de vidas que
    /// chegou a `0` fica em `1` — e o princípio dele é `3`. *Andar · nascer · bater · morrer ·
    /// contar · perder já se autoravam; recomeçar não.*
    ///
    /// # ⚠️⚠️ Ele é o PRIMEIRO verbo cujo sujeito NÃO é uma entidade
    ///
    /// Os outros nove agem sobre um alvo; este age sobre a **corrida**. ⇒ ele **não lê o
    /// `target`**, e o painel di-lo em vez de pintar uma escolha que o consumidor deita fora — a
    /// lei que esta casa escreve para todo controlo morto.
    ///
    /// # ⚠️ Ele ANUNCIA, e é a segunda vez que este idioma se usa
    ///
    /// Como o [`SignalVerb::Destroy`], ele não age: põe um pedido no relatório da ponte e quem o
    /// serve é a shell, **uma vez por quadro**. ⛔ *Dois recomeços no mesmo quadro são UM* — o
    /// pedido é um booleano e não uma contagem, e é o tipo que o diz.
    ///
    /// ⛔ **Ele NÃO lê o `arg`** — *«recomeça»* não tem parâmetro.
    RestartRun,
    /// ⭐⭐⭐ **Tira VIDA ao alvo** (plano 28, W2b) — o `arg` é quanto (um número `> 0`).
    ///
    /// # ⚠️ Ele ANUNCIA, pela terceira vez este idioma
    ///
    /// A vida vive na PONTE da física (o `HealthState`, no anel de checkpoints) e anda por TIQUE; a
    /// tabela corre por QUADRO. ⇒ o verbo põe um pedido no relatório, a shell entrega-o à ponte, e
    /// a ponte aplica-o no tique seguinte **e grava-o por tique** — é isso que faz um scrub devolver
    /// a vida exacta (um pedido aplicado por quadro seria re-aplicado ou esquecido num replay).
    ///
    /// ⚠️ **Passa pelo pipeline inteiro da lei** (invencibilidade · esquiva · armadura · escudo),
    /// como o `Hit` do oráculo com os dois interruptores de fábrica ligados. ⛔ **Não tem equipa**:
    /// a equipa é uma cerca do CONTACTO (quem bate em quem), e um verbo autorado já escolheu o alvo.
    ///
    /// ⛔ **Um `arg` vazio, ilegível ou `≤ 0` é INERTE** — ao contrário do `AddToCounter`, não há um
    /// valor natural a adivinhar (*«apanhei uma moeda»* é `1`; *«queimei-me»* não tem número).
    Damage,
    /// ⭐⭐⭐ **Devolve VIDA ao alvo** (plano 28, W2b) — o irmão do [`Self::Damage`], pelo mesmo
    /// caminho, e a primeira porta que acende o sinal `On Heal` da vida (antes dele, esse campo não
    /// tinha produtor nenhum).
    ///
    /// ⚠️ **Um morto não é curado** — a regra da casa (`morto_nao_e_final`): tirar alguém da morte
    /// é o *reviver*, que é outra porta.
    Heal,
}

impl SignalVerb {
    /// Todos, em ordem — **a fonte da iteração**. ⛔ Nunca escreva a lista uma segunda vez.
    /// ⚠️ **APPEND-ONLY**: a posição é a tag e ela viaja no ficheiro. Um verbo novo entra no FIM.
    pub const ALL: [SignalVerb; 12] = [
        SignalVerb::StartTimer,
        SignalVerb::StopTimer,
        SignalVerb::Show,
        SignalVerb::Hide,
        SignalVerb::ToggleVisibility,
        SignalVerb::PlaySound,
        SignalVerb::StopSound,
        SignalVerb::AddToCounter,
        SignalVerb::Destroy,
        SignalVerb::RestartRun,
        SignalVerb::Damage,
        SignalVerb::Heal,
    ];

    /// O rótulo que o artista lê, em INGLÊS — um ACESSÓRIO derivado da tabela desde 2026-09-19
    /// (ver [`Self::label_key`]).
    #[must_use]
    pub fn label(self) -> &'static str {
        ph2d_i18n::tr_em(ph2d_i18n::Idioma::Ingles, self.label_key())
    }

    /// ⭐⭐ **A CHAVE do rótulo** — `ecs.signal_verb.<variante>`. A shell resolve-a ao montar o
    /// `verb_labels` do snapshot (`render_loop/inspector_action.rs`), que é o único sítio onde
    /// estas sete palavras chegam a um pixel.
    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            SignalVerb::StartTimer => "ecs.signal_verb.start_timer",
            SignalVerb::StopTimer => "ecs.signal_verb.stop_timer",
            SignalVerb::Show => "ecs.signal_verb.show",
            SignalVerb::Hide => "ecs.signal_verb.hide",
            SignalVerb::ToggleVisibility => "ecs.signal_verb.toggle_visibility",
            SignalVerb::PlaySound => "ecs.signal_verb.play_sound",
            SignalVerb::StopSound => "ecs.signal_verb.stop_sound",
            SignalVerb::AddToCounter => "ecs.signal_verb.add_to_counter",
            SignalVerb::Destroy => "ecs.signal_verb.destroy",
            SignalVerb::RestartRun => "ecs.signal_verb.restart_run",
            SignalVerb::Damage => "ecs.signal_verb.damage",
            SignalVerb::Heal => "ecs.signal_verb.heal",
        }
    }

    /// **Este verbo LÊ o `arg`?** — é o que decide se o painel pinta o campo.
    ///
    /// ⚠️ **Derivado do verbo, nunca uma segunda lista.** Um painel que mostra um campo que o verbo
    /// não lê é um controlo morto; um que o esconde onde o verbo o lê é uma feature inalcançável.
    #[must_use]
    pub const fn uses_arg(self) -> bool {
        matches!(
            self,
            SignalVerb::StartTimer
                | SignalVerb::StopTimer
                | SignalVerb::AddToCounter
                | SignalVerb::Damage
                | SignalVerb::Heal
        )
    }

    /// **Este verbo tem ALVO?** — é o que decide se o painel pinta a coluna de quem sofre.
    ///
    /// ⚠️ **Nove dos dez respondem `true`, e é o décimo que faz a pergunta existir:** o
    /// [`SignalVerb::RestartRun`] age sobre a **corrida** e não sobre uma entidade. Pintar-lhe uma
    /// escolha de alvo seria um controlo cujo consumidor a deita fora — o defeito que a caça de
    /// 2026-08-30 mediu em 34 controlos, e a mesma razão pela qual o [`Self::uses_arg`] existe.
    ///
    /// ⛔ **Derivado do verbo, nunca uma segunda lista.**
    #[must_use]
    pub const fn uses_target(self) -> bool {
        !matches!(self, SignalVerb::RestartRun)
    }

    /// A posição em [`Self::ALL`] — a tag que o painel usa nos segmentados.
    #[must_use]
    pub const fn tag(self) -> u8 {
        self as u8
    }

    /// O verbo desta posição, ou o primeiro. ⚠️ **A POSIÇÃO NO ARRAY É A TAG**, e reordenar
    /// [`Self::ALL`] faria um clique escrever outro verbo — e compila.
    #[must_use]
    pub fn from_tag(tag: u8) -> Self {
        Self::ALL.get(tag as usize).copied().unwrap_or_default()
    }
}

/// ⭐⭐ **A QUEM a acção se aplica** — pelo nome, ou a todos os que pertencem a uma tag (TOP-20 #9,
/// `docs/Components/08_plano_tags.md` §2.3).
///
/// ⚠️ **Ele é APENDADO ao fim do [`SignalAction`]** e o default é o de sempre, então uma linha que
/// nunca escolheu tag resolve exactamente como antes. O postcard é posicional, e é por isso que o
/// campo custa o degrau `128 -> 129` do `PROJECT_SCHEMA` e a migração em `signal_actions_v1.rs`.
///
/// ⛔ **Não é um segundo campo de texto ao lado do `target`**: com os dois, *«a quem?»* teria duas
/// respostas escritas ao mesmo tempo e o painel teria de escolher uma. Aqui a variante escolhe, e o
/// `target` só é lido pelo `Named`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalTarget {
    /// O objecto chamado [`SignalAction::target`] — vazio = **este objecto**. O de sempre, e o
    /// default: toda linha de um ficheiro v128 migra para aqui.
    #[default]
    Named,
    /// **Todos os que pertencem à tag**, com a subárvore dela, pela ordem da identidade
    /// ([`crate::tags::tagged`]). ⚠️ Inclui quem reage, se pertencer — o `call_group` do Godot
    /// (medido). Uma tag que já não existe = **ninguém** (a lei do alvo que não existe).
    ///
    /// ⚠️ `u64` e não [`TagId`] porque a folha das tags não fala `serde` (de propósito).
    Tagged(u64),
    /// ⭐⭐ **O OUTRO LADO do disparo** — hoje, quem tocou em quem gritou (o `other` de um contacto).
    ///
    /// ⚠️ **APENDADO**, como o verbo e pela mesma razão: a posição é a tag e ela viaja no ficheiro.
    ///
    /// ⚠️ **Só o contacto o tem.** Para toda outra origem isto dá **ninguém** — `None` **não** é
    /// *«qualquer um»*, e é a lei do alvo que não existe, que este enum já escreve para uma tag
    /// apagada.
    ///
    /// ⛔ **E NÃO há um `Speaker` ao lado dele, com medição:** com a cerca [`SignalFrom::Myself`]
    /// quem reage **é** quem falou, logo o alvo vazio (*este objecto*) já o exprime. Um `Speaker`
    /// só serviria a um reactor que não é o sujeito — e nenhuma cena o pede hoje. *Um alvo sem
    /// consumidor é um controlo morto com cara de feature.*
    Other,
}

impl SignalTarget {
    /// A tag escolhida, se o alvo for por tag.
    #[must_use]
    pub const fn tag(self) -> Option<TagId> {
        match self {
            Self::Tagged(id) => Some(TagId(id)),
            Self::Named | Self::Other => None,
        }
    }
}

/// ⭐⭐⭐ **DE QUEM** o sinal tem de vir para esta linha reagir — a **TERCEIRA** pergunta de uma
/// linha, e a que faltava (suplente #24, [`plano`]).
///
/// [`plano`]: https://github.com/dibrioli/PH2D/blob/main/docs/Components/19_plano_o_sinal_sabe_quem.md
///
/// # ⛔⛔ Porque ela é uma CERCA do reactor e não mais um alvo
///
/// Medido (a sonda `mede_o_que_a_composicao_ja_da_ao_golpe`): dez inimigos iguais com a linha
/// *«ao ouvir `golpe`, perco uma vida»* resolvem **dez** efeitos com **um** tiro — o sinal é um nome
/// global e a tabela não sabia quem levou o golpe.
///
/// ⚠️ **E um alvo «quem falou» NÃO a substituiria:** com ele os dez reactores aplicariam o verbo ao
/// mesmo sujeito (`-10` numa vida). *Quem reage* e *a quem* são duas perguntas, e esta é a primeira.
///
/// ⚠️ **É o ÚLTIMO campo do [`SignalAction`]**, pela mesma razão do `target_by`: é o que torna a
/// migração de um v148 uma leitura com um tipo congelado e um re-encode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalFrom {
    /// **Qualquer um.** O de sempre, e o default: toda linha de um ficheiro v148 migra para aqui.
    #[default]
    Anyone,
    /// **Só se fui EU que falei** — o sinal tem de vir deste objecto.
    ///
    /// ⚠️ **Um sinal sem sujeito nunca passa por aqui**, e não é um caso esquecido: a timeline, o
    /// botão do painel e o Motion publicam sem `source`, e tratar `None` como *«sim»* faria uma
    /// cerca fechada deixar passar tudo — o modo de falha mais caro que uma cerca pode ter.
    Myself,
}

impl SignalFrom {
    /// Todas, em ordem — **a fonte da iteração** do segmentado do painel.
    /// ⚠️ **APPEND-ONLY**: a posição é a tag e ela viaja no ficheiro.
    pub const ALL: [SignalFrom; 2] = [SignalFrom::Anyone, SignalFrom::Myself];

    /// ⭐ **A CHAVE do rótulo que o artista lê** — quem pinta é que a resolve (HR-15).
    ///
    /// ⚠️ **O nome mudou de `label` para `label_key` na integração de 2026-09-20**, com o valor:
    /// um método chamado `label` que devolve `ecs.signal_from.anyone` mente ao chamador seguinte,
    /// e o irmão `SignalVerb` já se chamava assim.
    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            SignalFrom::Anyone => "ecs.signal_from.anyone",
            SignalFrom::Myself => "ecs.signal_from.myself",
        }
    }

    /// A posição em [`Self::ALL`] — a tag que o painel usa no segmentado.
    #[must_use]
    pub const fn tag(self) -> u8 {
        self as u8
    }

    /// A cerca desta posição, ou a primeira. ⚠️ **A POSIÇÃO NO ARRAY É A TAG.**
    #[must_use]
    pub fn from_tag(tag: u8) -> Self {
        Self::ALL.get(tag as usize).copied().unwrap_or_default()
    }

    /// **Este disparo passa a cerca de quem reage?**
    ///
    /// ⚠️ **A porta ÚNICA da pergunta**, e ela é `const`: escrita duas vezes (aqui e no painel, que
    /// quer pintar *«esta linha não vai reagir»*), as duas divergiriam no dia da terceira variante.
    #[must_use]
    pub fn deixa_passar(self, quem_falou: Option<Entity>, reactor: Entity) -> bool {
        match self {
            SignalFrom::Anyone => true,
            SignalFrom::Myself => quem_falou == Some(reactor),
        }
    }
}

/// **Uma linha da tabela** — *quando o sinal `on` chegar, faz `verb` em `target`*.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalAction {
    /// O nome do sinal que a dispara. **Vazio = nunca** (ver o doc do módulo).
    pub on: String,
    /// O NOME do objecto que sofre a acção. **Vazio = este objecto.**
    ///
    /// ⚠️ **Nunca bits** — ver o doc do módulo. E o vazio é o caso comum, não uma abreviatura: é
    /// ele que faz uma cópia de prefab funcionar sem rewiring.
    pub target: String,
    pub verb: SignalVerb,
    /// O parâmetro do verbo — hoje, o nome do timer. Vazio quando o verbo não o lê
    /// ([`SignalVerb::uses_arg`]), e **vazio também significa «todos»** para os verbos de timer.
    pub arg: String,
    /// ⭐ **Por nome ou por tag** — ver [`SignalTarget`]. ⚠️ Era o ÚLTIMO campo até 2026-09-19.
    pub target_by: SignalTarget,
    /// ⭐⭐⭐ **De quem o sinal tem de vir** — ver [`SignalFrom`]. ⚠️ **O ÚLTIMO campo agora**, pela
    /// mesma razão que o `target_by` foi: é o que torna a migração de um v148 uma leitura com um
    /// tipo congelado e um re-encode.
    pub from: SignalFrom,
}

/// **A tabela de uma entidade** — o componente registado.
///
/// ⚠️ **Uma LISTA**, como o [`crate::Timers`] e o `NamedAnchorList`: um componente ECS é único por
/// entidade, e um objecto tem tipicamente várias reacções (*abre com `botao`, fecha com `alarme`*).
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalActions(pub Vec<SignalAction>);

/// ⭐⭐⭐ **UM SINAL QUE SOOU NESTE QUADRO, com QUEM o disse** (suplente #24).
///
/// # ⛔⛔ Porque o tipo vive AQUI e não no barramento
///
/// O `ph2d-ecs` **não pode ver o `ph2d-runtime`** (ADR-0075: a fundação não conhece o barramento —
/// e o `Cargo.toml` mede-o). ⇒ o dado atravessa a fronteira num tipo desta crate, e quem o constrói
/// é a shell, que é dona dos dois lados. *A lei devolve factos, a ponte publica-os* — a mesma
/// fronteira que o `tick_timers` já usa.
///
/// # ⚠️ O dado JÁ EXISTIA, e era deitado fora uma linha antes de ser preciso
///
/// O `SignalOrigin` tem 14 variantes e **11 carregam `source`**; o `Contact` carrega `source` **e**
/// `other`, com o doc a chamar-lhes *«quem GRITOU»* e *«quem chegou, ou quem saiu»*. A shell fazia
/// `.map(|s| s.name)` e a origem morria ali. ⇒ esta wave não descobre o dado: **deixa de o deitar
/// fora**.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Disparo<'a> {
    /// O nome que soou.
    pub nome: &'a str,
    /// **Quem GRITOU.** `None` = a origem não tem sujeito (a timeline, um botão do painel, o Motion).
    pub quem: Option<Entity>,
    /// **O OUTRO lado**, quando o houver — hoje só o contacto o tem.
    pub outro: Option<Entity>,
}

impl<'a> Disparo<'a> {
    /// Um sinal **sem sujeito** — o que todo chamador de teste quer dizer, e o que a timeline e o
    /// botão do painel de facto publicam.
    ///
    /// ⚠️ Ele **não passa** a cerca [`SignalFrom::Myself`], e isso é a lei e não um esquecimento.
    #[must_use]
    pub const fn anonimo(nome: &'a str) -> Self {
        Self {
            nome,
            quem: None,
            outro: None,
        }
    }
}

/// **O efeito que uma acção decidiu**, com o alvo já resolvido para uma entidade.
///
/// ⚠️ **A resolução vem feita**, e é isso que separa esta metade da outra: a shell recebe *«faz
/// isto NESTA entidade»* e não tem de saber o que é um nome. Um alvo que não existe **não produz
/// efeito nenhum** — ver [`resolve`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignalEffect {
    /// Quem sofre.
    pub target: Entity,
    pub verb: SignalVerb,
    /// O `arg` da linha, copiado. Vazio = «todos», para os verbos que o lêem.
    pub arg: String,
    /// Quem reagiu — a entidade que carrega a tabela. ⚠️ **Não é o alvo**, e a distinção é a que
    /// torna um diagnóstico legível: *«a Porta reagiu ao sinal `botao` e escondeu a Parede»*.
    pub source: Entity,
}

/// ⭐⭐⭐ **A RESOLUÇÃO** — que efeitos os sinais deste quadro produzem.
///
/// # ⚠️ A ordem é DETERMINISTA, e não é a da query
///
/// Duas entidades que reajam ao mesmo sinal e escrevam no mesmo alvo têm de o fazer sempre na
/// mesma ordem, senão o replay diverge. A ordem da query do `bevy_ecs` é a do arquétipo, que muda
/// quando um componente é inserido — ⇒ a saída é ordenada pelo [`crate::StableId`] de quem reage,
/// que é a identidade que sobrevive ao respawn do undo por construção.
///
/// ⚠️ **Dentro de uma entidade, a ordem é a que o artista ESCREVEU.** Ela é visível na lista do
/// painel, e reordená-la seria o painel a mentir sobre o que acontece.
///
/// # ⛔ Um alvo que não existe é SILÊNCIO, e é deliberado
///
/// Um nome que não casa devolve zero efeitos — nunca um efeito sobre quem reagiu. *Cair no próprio
/// objecto quando o alvo desapareceu faria uma porta esconder-se a si mesma no dia em que alguém
/// renomeasse a parede*, e o artista leria isso como um defeito do motor.
///
/// ⚠️ **`&mut World` porque a resolução de nome ATRIBUI ids em falta** (`stable_id_for_name`), e
/// isso é uma escrita. Ela é idempotente e o `assign_missing_stable_ids` já corre no quadro.
///
/// ⚠️ **A árvore de tags entra por parâmetro**, e é o documento do PROJECTO (a shell guarda-a no
/// `AppGfx`): uma acção por tag pergunta quem pertence à subárvore, e a pertença de um objecto é só
/// uma lista de ids. Uma acção por nome nunca a lê.
///
/// # ⭐⭐⭐ Uma linha reage a um DISPARO, não a um NOME (2026-09-19, suplente #24)
///
/// Até esta wave a entrada era `&[&str]` e a pergunta era *«este nome soou?»* — logo dois eventos
/// com o mesmo nome no mesmo quadro davam **um** efeito. Com o [`Disparo`] a pergunta é *«que
/// eventos soaram?»*, e uma linha produz um efeito **por evento que casa**.
///
/// ⚠️⚠️ **A mudança é OBSERVÁVEL e cura um defeito latente:** duas moedas apanhadas no mesmo quadro
/// somavam **1** ponto e passam a somar **2**. ⛔ Ela **não** é o colapso que as origens declaram
/// (`fires`/`cycles`/`rows`/`count`): esse é dentro de **um** produtor — *«sai um evento, com
/// quantos ele representa»* —, e dois inimigos atingidos são dois produtores diferentes.
#[must_use]
pub fn resolve(world: &mut World, tree: &TagTree, fired: &[Disparo<'_>]) -> Vec<SignalEffect> {
    if fired.is_empty() {
        return Vec::new();
    }
    // Quem reage, na ordem da identidade — nunca a da query.
    let mut reactors: Vec<(u64, Entity, SignalActions)> = world
        .query::<(Entity, &SignalActions, &crate::StableId)>()
        .iter(world)
        .filter(|(_, a, _)| !a.0.is_empty())
        .map(|(e, a, s)| (s.0, e, a.clone()))
        .collect();
    if reactors.is_empty() {
        return Vec::new();
    }
    reactors.sort_unstable_by_key(|(id, _, _)| *id);

    let mut out = Vec::new();
    for (_, source, table) in reactors {
        for action in &table.0 {
            // Um consumidor sem nome não escuta — o espelho da lei do produtor.
            if action.on.is_empty() {
                continue;
            }
            // ⚠️ **A ordem dos DISPAROS é a da fila do quadro** — ela é a ordem de publicação, que
            // é determinista porque cada produtor corre num sítio fixo do quadro.
            for disparo in fired.iter().filter(|d| d.nome == action.on) {
                // ⭐⭐⭐ **A CERCA** (suplente #24) — *«só reajo se fui eu que falei»*. Sem ela, um
                // tiro num inimigo tira vida aos dez (medido: 10 efeitos para 1 sinal).
                if !action.from.deixa_passar(disparo.quem, source) {
                    continue;
                }
                // ⚠️ A ordem dentro de uma linha é a dos ALVOS (pela identidade), depois da ordem
                // dos disparos, dos reactores e das linhas — as quatro, deterministas.
                for target in targets_of(world, tree, source, action, *disparo) {
                    out.push(SignalEffect {
                        target,
                        verb: action.verb,
                        arg: action.arg.clone(),
                        source,
                    });
                }
            }
        }
    }
    out
}

/// ⭐⭐ **Quem sofre esta acção** — a porta ÚNICA da pergunta *«a quem?»* (plano de Tags §2.2).
///
/// - [`SignalTarget::Named`]: o objecto com aquele nome, ou `source` com o nome vazio — zero ou um.
/// - [`SignalTarget::Tagged`]: todos os que pertencem à subárvore da tag, pela ordem do
///   [`crate::StableId`]; uma tag que já não existe dá **ninguém**.
/// - [`SignalTarget::Other`]: o outro lado do disparo — **ninguém** quando a origem não o tem.
///
/// ⛔ Resolver o alvo na shell seria a segunda resposta, e é a que envelhece.
///
/// ⚠️ **O outro lado é CONFERIDO contra o mundo** (`get_entity`) pela mesma razão que
/// o [`target_of`] o faz: entre o quadro em que o sinal foi publicado e este, um dreno de morte pode
/// ter levado a entidade — e um efeito sobre bits reciclados escreveria no objecto errado.
#[must_use]
pub fn targets_of(
    world: &mut World,
    tree: &TagTree,
    source: Entity,
    action: &SignalAction,
    disparo: Disparo<'_>,
) -> Vec<Entity> {
    match action.target_by {
        SignalTarget::Named => target_of(world, source, &action.target)
            .into_iter()
            .collect(),
        SignalTarget::Tagged(id) => crate::tags::tagged(world, tree, TagId(id)),
        SignalTarget::Other => vivo(world, disparo.outro).into_iter().collect(),
    }
}

/// A entidade, se ela **ainda existir** no mundo. Ver o doc de [`targets_of`].
fn vivo(world: &World, e: Option<Entity>) -> Option<Entity> {
    e.filter(|&e| world.get_entity(e).is_ok())
}

/// O alvo de uma acção: `source` quando o nome é vazio, senão quem tiver aquele [`Name`].
///
/// ⚠️ **Ele confere que a entidade AINDA existe.** Entre o quadro em que o artista escreveu o nome
/// e este, um `Ctrl+Z` pode tê-la levado.
fn target_of(world: &mut World, source: Entity, name: &str) -> Option<Entity> {
    if name.is_empty() {
        return world.get_entity(source).ok().map(|_| source);
    }
    let id = crate::stable_id_for_name(world, name);
    crate::entity_of_stable_id(world, crate::StableId(id))
}

/// **O nome do objecto que este efeito atinge**, para um diagnóstico legível. `None` = sem `Name`.
#[must_use]
pub fn name_of(world: &World, entity: Entity) -> Option<String> {
    world.get::<Name>(entity).map(|n| n.as_str().to_string())
}

/// O `SignalAction` como o `PROJECT_SCHEMA` 128 o gravava, congelado para a migração.
#[path = "signal_actions_v1.rs"]
mod v1;
pub use v1::migrate_v1_blob;

#[cfg(test)]
#[path = "signal_actions_tests.rs"]
mod tests;

/// ⭐ **A CERCA e os DOIS LADOS** (suplente #24) — irmão por ASSUNTO; ver o cabeçalho de lá.
#[cfg(test)]
#[path = "signal_actions_cerca_tests.rs"]
mod cerca_tests;
