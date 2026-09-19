# Suplente #24 — `Health`: **o sinal sabe QUEM**

> **Estado:** em implementação (2026-09-19). A wave nasce de uma MEDIÇÃO e a medição **reescreveu o
> item**: o que o levantamento chama de `Health` não é um componente — é a **cegueira da tabela de
> acções ao autor do golpe**, mais o verbo que tira da cena.

## §1 — A sonda, e o que ela mediu

`CLAUDE.md` §5.0 manda medir a composição antes da primeira linha. A sonda é
[`mede_o_que_a_composicao_ja_da_ao_golpe.rs`](../../crates/ph2d-ecs/tests/it/mede_o_que_a_composicao_ja_da_ao_golpe.rs)
(`--ignored`, imprime).

⚠️ **E havia uma razão a mais para a correr:** o cabeçalho do
[`counter_watch.rs`](../../crates/ph2d-ecs/src/counter_watch.rs) **declara por escrito** que o
`Health` é composição — *«`Counter{name:"vidas", start:3}` + `SignalActions[golpe → AddToCounter(-1)]`
+ `CounterWatch[vidas AtMost 0 → "morri"]`»*. A sonda pergunta se essa frase aguenta **mais do que um
sujeito**.

| bloco | pergunta | MEDIDO (2026-09-19) |
|---|---|---|
| A | dez inimigos iguais, **um** sinal `golpe` | **10 efeitos** — os dez perdem vida com um tiro |
| B | a tabela sabe dizer *«quem me bateu»*? | **não** — `SignalTarget` tem `Named` e `Tagged`, mais nada |
| C | há verbo que tire da cena? | **não** — `SignalVerb::ALL` tem 8 e nenhum apaga |
| D | dois inimigos com `Counter{name:"vida"}` | **`Some(6)`** — a porta SOMA por nome: **uma** vida partilhada |

⇒ **A frase do `counter_watch` é verdadeira para UM sujeito (o herói: vidas, pontos) e falsa para N.**
Ela não estava errada — estava a descrever o caso que existia.

### §1.1 — ⛔⛔ E o dado já existe, deitado fora UMA LINHA antes de ser preciso

O `SignalOrigin` tem **14** variantes e **11 carregam `source`**; o `Contact` carrega **`source` e
`other`** — *«quem GRITOU»* e *«quem chegou, ou quem saiu»*, com o doc a nomeá-los. E a shell faz:

```rust
// shells/desktop/src/render_loop/fase_signal_outbox.rs
let disparados: Vec<String> = self.signals.read(…).map(|s| s.name.to_string()).collect();
let efeitos = ph2d_ecs::resolve_signal_actions(sim.world_mut(), tags, &nomes);
```

*A origem é construída, publicada, lida — e descartada no `.map`.* ⇒ a wave **não descobre** o dado:
ela deixa de o deitar fora.

### §1.2 — ⚠️ E a `Visibility` não é morte, com número

A física **não lê** a `Visibility` (`git grep Visibility -- crates/ph2d-physics-ecs` = zero). ⇒
`Hide` esconde e **o corpo continua a travar balas**. *Um inimigo escondido que bloqueia o tiro
seguinte é pior do que um inimigo vivo.*

## §2 — O desenho: a linha ganha a TERCEIRA pergunta

Uma linha da tabela responde hoje a **duas** perguntas — *quando?* (`on`) e *a quem?*
(`target`/`target_by`). Falta a que a medição nomeia: **de quem?**

```
  on: "golpe"      de quem: EuMesmo      a quem: <este>      verbo: Destroy
  └─ quando        └─ a CERCA (nova)     └─ o alvo           └─ o que faz
```

### §2.1 — `Disparo` — o que um sinal traz além do nome

⛔ **O `ph2d-ecs` NÃO pode ver o `ph2d-runtime`** (medido no `Cargo.toml`, e é o ADR-0075: a
fundação não conhece o barramento). ⇒ o dado atravessa num tipo **desta** crate:

```rust
pub struct Disparo<'a> {
    pub nome: &'a str,
    /// Quem GRITOU. `None` = a origem não tem sujeito (timeline · botão do painel · Motion).
    pub quem: Option<Entity>,
    /// O OUTRO lado, quando o houver — hoje só o contacto o tem.
    pub outro: Option<Entity>,
}
```

⚠️ **`resolve` muda de assinatura, e é de propósito:** uma segunda porta *«resolve, mas com quem»*
seria a segunda resposta à mesma pergunta, e a que envelhece é a que o produto usa. Os ~7 sítios de
chamada passam a construir um `Disparo` — os de teste com [`Disparo::anonimo`], que é exactamente o
que o mundo de hoje diz.

### §2.2 — `SignalFrom` — a cerca

```rust
pub enum SignalFrom {
    /// Qualquer um. O de sempre, e o default: toda linha de um ficheiro v148 migra para aqui.
    #[default]
    Anyone,
    /// Só se fui EU que falei — o sinal tem de vir DESTE objecto.
    Myself,
}
```

⚠️ **Ela é uma cerca do REACTOR, nunca um alvo** — e as duas coisas não se substituem: com
`SignalTarget::Speaker` os dez reactores aplicariam o verbo ao mesmo sujeito (**`-10` numa vida**);
com `from: Myself` **um** reactor reage, e ao seu próprio alvo.

⚠️ **Um sinal sem sujeito nunca passa a cerca** (a timeline, o botão do painel, o Motion): `None`
não é *«qualquer um»*. É a mesma lei do nome vazio — *um campo por preencher não é um curinga*.

### §2.3 — `SignalTarget::{Speaker, Other}` — os dois lados

**APENDADOS** (o postcard é posicional e a posição é a tag). `Speaker` = quem falou; `Other` = o
outro lado do contacto. Sem sujeito, **ninguém** — a lei do alvo que não existe, que aquele enum já
escreve para uma tag apagada.

### §2.4 — `SignalVerb::Destroy` — e a FRONTEIRA é forçada, não escolhida

O verbo **anuncia** e nunca apaga: ele produz um [`Death`] pelo despachante que já existe, com uma
quarta causa (`Killed`). ⛔ *Dois despachantes de morte seriam duas respostas a «quando é que isto
sai da cena?»* — a frase que o `fase_fabrica_e_morte` já tem escrita.

⚠️⚠️ **E ele só tira quem NASCEU numa corrida** ([`is_transient`], o quarto leitor daquela porta).
A razão não é preferência: **apagar um objecto do documento durante a corrida tira-o do documento** —
a captura vê-o sumido e o `Ctrl+Z` herda a remoção —, e isso é a lei *«o que acontece numa corrida
não é DOCUMENTO»* invertida. O precedente é do #14, com a frase inteira: *«um projéctil que ele pôs
na cena à mão é documento, e apagá-lo destruiria autoria»*.

⇒ um alvo de documento é **recusado em voz** e contado como inerte. ⏳ A outra saída — **morte de
pré-visualização** (um morto que fica no documento e sai da corrida) — precisa de dois leitores novos
(desenho e física) e de reposição no rebobinar: **está nomeada no §5, e é decisão do dono.**

## §3 — As waves

| # | O quê | Onde |
|---|---|---|
| W1 | A lei: `Disparo` · `SignalFrom` · os dois alvos · o verbo | `ph2d-ecs/src/signal_actions.rs` |
| W2 | O degrau `PROJECT_SCHEMA 148 → 149` + a migração congelada | `shells/desktop/src/project_schema.rs` · `signal_actions_v2.rs` |
| W3 | A ponte: a shell deixa de deitar a origem fora; o `Destroy` entra no dreno da morte | `render_loop/fase_signal_outbox.rs` · `fase_fabrica_e_morte.rs` |
| W4 | O painel: a coluna *de quem?*, os alvos novos, o verbo novo | `ph2d-panel-inspector` + `ph2d-editor-core` |
| W5 | A cena `PH2D_DANO_SMOKE=1` | `ph2d-app-components/src/dano_smoke.rs` |

⚠️ **Conte o DELTA, nunca o literal:** `PROJECT_SCHEMA` **+1**; os três registos **0** (zero
componentes novos); `LIVE_SECTIONS` **0** (a secção do `SignalActions` já existe).

## §4 — A cena, e porque os inimigos NASCEM

⛔ **Os inimigos vêm de uma `Factory`**, e não postos à mão: o `Destroy` só tira quem nasceu numa
corrida (§2.4), logo uma cena com inimigos de documento ensinaria o contrário do que acontece — a
espécie que o §5.0 chama de **pior que uma cena ausente**.

A cena compõe o que a linha já shipou: o **gatilho** (`Q`) · a **fábrica** · o **projéctil** · o
**contacto** · a **tabela de acções** · o **contador** e o **HUD**.

E ela traz o **CONTROLO ao lado**: uma fileira com a MESMA tabela **sem** a cerca (`from: Anyone`) —
onde um tiro mata a fileira inteira. *É o que torna a wave legível numa imagem.*

## §5 — O que fica ABERTO (e de quem é)

| item | mecanismo |
|---|---|
| ⏳ **morte de pré-visualização** (matar um objecto de DOCUMENTO e o rebobinar devolvê-lo) | **decisão do dono** — dois leitores novos (desenho · física) + reposição; hoje a recusa é em voz |
| ⏳ **uma vida POR inimigo** | o `Counter` é somado por NOME em todo o mundo (§1 bloco D). Hoje o inimigo morre ao primeiro golpe; vida por-objecto pede que a porta do contador saiba de quem é |
| ⏳ `SignalFrom::Tagged` (*«só se quem falou pertence à tag X»*) | sem consumidor — ⛔ um verbo sem sink é um controlo morto com cara de feature |
