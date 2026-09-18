# `SignalOnAction` — a MÃO de quem joga vira um sinal

> **Estado:** implementado (2026-09-18). A wave nasce de uma MEDIÇÃO, não da tabela do TOP-20 — e a
> medição mudou o que ela é: o item que o levantamento chama de `WeaponFire` **não é um
> componente**, são **duas** peças pequenas que compõem com o que já existe.

## §1 — ⭐⭐⭐ A medição que abriu a wave

A §5.0 do `CLAUDE.md` manda medir a composição **antes** de construir. Corrida sobre o `WeaponFire`,
ela devolve dois buracos exactos — e nenhum deles é *«uma arma»*.

### O censo dos PRODUTORES de sinal

O [`ph2d_runtime::SignalOrigin`] tinha **treze** variantes:

| # | origem | quem a publica |
|---|---|---|
| 1 | `Timeline` | um marcador da timeline |
| 2 | `Contact` | dois corpos que se tocam |
| 3 | `Control` | um controlo do editor |
| 4 | `Motion` | o grafo de movimento |
| 5 | `Animation` | uma tag de sprite que acaba ou dá a volta |
| 6 | `Timer` | o relógio do #2 |
| 7 | `Spawn` | a fábrica do #11 |
| 8 | `Death` | a higiene do #12 |
| 9 | `StateMachine` | o cérebro do #15 |
| 10 | `Script` | o `.luau` do #16 |
| 11 | `Particles` | o emissor do #18 |
| 12 | `UiButton` | o botão do HUD do #20 |
| 13 | `CounterWatch` | a vigia do #17 |

⛔⛔ **E nenhuma é a mão de quem joga.** *A tabela de acções do #5 sabia reagir a tudo menos a uma
tecla.* Um jogo em que o herói dispara é o exemplo mais banal que existe, e ele era inexprimível sem
escrever um `.luau`.

⚠️ **A ausência NÃO era do substrato**, e é isso que faz a wave ser pequena: a
[`ph2d_input::Input`] já resolve acções **nomeadas**, já entrega as três leituras que uma lei de
gatilho precisa (`pressed` · `just_pressed` · `just_released`), já as torna religáveis pelo Input
Map e já grava **a acção resolvida** (nunca a tecla) na fita determinística. *O que faltava era um
componente que as ouça.*

### O segundo buraco: a fábrica DEITA A MIRA FORA

Medido no [`Factory::onde`]: ele devolve **posições** e mais nada. ⇒ uma bala nascida por uma fábrica
saía sempre com a rotação **autorada no molde**, para onde quer que o herói estivesse virado.

⇒ **a wave são duas peças e não uma:** *quem dispara* (`SignalOnAction`) e *para onde a coisa sai*
(`Factory::aim_from_spawner`). Compostas com o `#11`, o `#12` e o `#14`, elas dão a arma inteira —
com o alcance, o ricochete, o arco e a higiene que aquelas waves já pagaram.

## §2 — O estado da arte, e o que ele já tentou

| referência | como se exprime | o que se aprende |
|---|---|---|
| **Godot** | `_input()` / `Input.is_action_just_pressed("fire")` | ⭐ a acção **nomeada** é o contrato; a tecla é config |
| **Unity** (Input System) | `InputAction` + *performed* / *canceled* / *started* | três fases — que é a nossa `ActionEdge` com outros nomes |
| **Construct 3** | condição *«On key pressed»* contra *«Key is down»* | ⭐⭐ o produto **obriga** o autor a dizer se quer a ARESTA ou o NÍVEL |
| **GDevelop** | condição por quadro | nível puro ⇒ o autor paga o *«disparou 60×»* |
| **Unreal** | *Enhanced Input*: `Triggered` · `Started` · `Completed` | idem, e com um asset por acção |

⭐⭐⭐ **A lição que decide o desenho é a mesma do `#17`, um nível abaixo:** *uma tecla lida como
nível dispara por quadro*. É por isso que `Press` é o valor de fábrica e o `Hold` existe **e não é o
padrão** — um é uma pistola, o outro é um lança-chamas, e escolher errado por omissão entrega a
ferramenta partida a quem nunca leu o manual.

## §3 — O desenho

```rust
pub enum ActionEdge { Press, Release, Hold }      // a posição É a tag serializada

pub struct ActionTriggerRow {
    pub action: String,   // QUAL acção — pelo NOME (o `ActionId` é um contador; o nome é o contrato)
    pub edge: ActionEdge,
    pub signal: String,   // vazio = a linha segue a tecla e fica CALADA
}
pub struct SignalOnAction(pub Vec<ActionTriggerRow>);   // REGISTADO (config)
```

### ⭐⭐ O que ele NÃO tem, e porquê

⛔ **Não há `SignalOnActionRuntime`, e a ausência é a decisão.** As cinco irmãs registadas desta
linha (`Timer` · `Factory` · `CounterWatch` · `StateMachine` · `GameCamera`) têm todas um `*Runtime`
não-registado e uma entrada no [`ph2d_ecs::rewind_runtime`]. **Esta não tem nenhum dos dois:** *a
aresta já é trabalho do INPUT* — a [`ph2d_input::ActionState`] guarda **um tique atrás** de
propósito (é o que paga o `just_pressed` dela), logo um estado vivo aqui seria **a segunda resposta**
a *«ela já estava premida?»*, e as duas divergiriam no primeiro quadro em que a fita reproduzisse um
passado diferente do presente.

⚠️ **Sem gate, a ausência lê-se como esquecimento** — que é exactamente o report que o `#14` pagou
quando o `projectile_state` ficou fora do `rebuild_from_rest`. ⇒ `o_gatilho_nao_guarda_estado`.

⛔ **Não há filtro por jogador.** O `Input` deste app é **um** (o *override* por-jogador em
`~/.ph2d/` é item aberto do Input Map desde 24/08), e um campo `player` aqui seria um knob morto.

### A lei da acção que NÃO EXISTE

Uma linha que nomeie uma acção ausente do mapa fica **calada**, e isso é de graça: as três leituras
da `Input` são `is_some_and`, logo um nome desconhecido devolve `false` nas três.

⚠️⚠️ **E é a varredura da shell que o garante, não um `if`:** o `amostras_das_accoes` percorre o
**MAPA** e não os gatilhos, logo um nome que o mapa não conhece simplesmente **não está lá**.
*Perguntar nome a nome ao mundo poria a mesma decisão em dois sítios*, e o defeito mudo que isso
abre é um `Release` a disparar em **TODO quadro** sobre uma acção que ninguém ligou — porque
`!pressed` é trivialmente verdade.

### A cerca do relógio

O gatilho só fala com `playhead.is_playing()`. ⚠️ **Sem ela o editor fica inutilizável:** *as teclas
do jogo são as teclas do editor*, e um gatilho ligado ao espaço publicaria o sinal dele a cada
espaço que o artista carrega a escrever num campo. É a mesma cerca que a fábrica do `#11` já tem.

### A mira

```rust
pub struct Factory { /* … */ pub aim_from_spawner: bool }   // append-only, OFF de fábrica
pub struct Birth   { /* … */ pub aim: Option<f32> }
```

⚠️ **`Option<f32>` e não um `f32` com neutro:** `0` é um ângulo legítimo (apontar para a direita),
logo um sentinela tornaria essa mira inexprimível. O caminho de omissão é `None` ⇒ a cópia fica com
a rotação do MOLDE, byte a byte como antes desta wave.

⚠️ **A mira sai da MESMA travessia da árvore que a posição** — uma segunda leitura do
`world_transform` poderia responder de um quadro diferente se alguém movesse o dreno do reparent.

⛔ **E ela nasce DESLIGADA**, com o número ao lado: uma chuva cujas gotas nascem viradas para onde o
emissor calhou estar é pior do que uma que ignora o emissor. *A ausência é medida ao lado da
presença* — o gate `a_copia_sai_apontada_para_onde_a_fabrica_aponta` tem as duas metades.

## §4 — Os contadores (DELTA contra o `main`, nunca o literal)

| contador | delta | porquê |
|---|---|---|
| `PROJECT_SCHEMA` | **+1** | um componente registado novo (`SignalOnAction`) |
| registo do `ph2d-ecs` | **+1** | idem |
| espelho do `ph2d-render` | **+1** | ele conta `ecs + render` |
| espelho do `ph2d-script` | **+1** | ele conta `ecs + script` |
| `LIVE_SECTIONS` | **+1** | a secção do Inspector |
| `SignalOrigin` | **+1** | `Action { source, row }`, **append-only** |
| `Factory` | **0** | o `aim_from_spawner` é um campo novo num componente que já existe |

⚠️ **`SignalOrigin` é `Copy`** ⇒ o nome **não** entra lá dentro; a origem carrega a `row`, como a do
`CounterWatch` — *um `Arc<str>` ali tirava o `Copy` a **todas** as origens*.

## §5 — O que fica ABERTO

- ⏳ **Nenhum gesto de canvas cria um gatilho** — ele entra por *Add Component → Trigger*, como as
  irmãs. Um verbo de *«ligar esta tecla a este objecto»* seria produto novo.
- ⏳ **A acção não se cria a partir da secção.** A linha diz se o nome é órfão e o painel explica-o,
  mas quem cria a acção é o *Input Map*, noutra janela. ⭐ Um botão *«criar esta acção»* é a cura
  óbvia e **não** foi construído: ele escreve no `HeroScreen`, que é do editor, e a secção do
  Inspector só fala com o mundo.
- ⏳ **Sem `once`.** O `#17` tem-no; aqui a `ActionEdge::Press` já é *uma vez por toque*, e um
  *«uma vez por vida»* não teve quem o pedisse — a espécie de knob que o §5.0 manda não construir
  antes do consumidor.
- ⛔ **Sem repetição automática** (o *auto-repeat* de um teclado). A composição já a dá: um `Hold`
  mais o `Timer` do `#2` entregam a cadência, com o número na mão do artista.
