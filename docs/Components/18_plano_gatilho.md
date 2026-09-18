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

## §4-bis — ⛔⛔ A TECLA do smoke, e a afirmação que o dono refutou (2026-09-18)

A cena nasceu a ligar a acção `fire` ao **ESPAÇO**, com o comentário a dizer que ele era *«a tecla
que ninguém do editor usa no canvas»*. O report do dono: ***«espaço é o atalho do play da timeline e
há conflito»***.

⇒ **verdade, e medida**: `crates/ph2d-editor-core/src/interaction/dispatch/key.rs` tem
`KEY_SPACE if !cmd => GraphKey::TogglePlay`. Um toque fazia **duas** coisas — parava a corrida **e**
disparava. *Uma afirmação sobre um atalho é uma MEDIÇÃO que eu não fiz.*

### A medição, e as DUAS vezes que ela mentiu antes de dizer a verdade

| tentativa | o que ela disse | porque estava errada |
|---|---|---|
| o estudo de **2026-08-12** | *«só NOVE letras livres: `H I J M N P U V Y`»* | **envelheceu**: o `P` foi tomado pelo menu radial (E4) e o `U` pelo detalhe da escultura, no mês seguinte |
| a minha 1.ª re-medição | *«NENHUMA letra está livre»* | contou a tabela do **NORMALIZADOR** (`KeyCode::KeyQ => 0x51`, que traduz winit→keycode) como se fossem atalhos |
| a 3.ª, sem o `keymap.rs` | **`H` · `J` · `Q`** não têm braço simples | ✅ e o `H` é o *Bypass* do grafo do Motion |

⭐⭐ **O `Q` ganha das outras duas porque não é só livre: é a tecla que o MAPA DE FÁBRICA deste app
já escolheu para uma acção de JOGADOR** (`PLAYER_DASH`), e a regra está escrita ao lado dela desde o
`#13`: *«o `W` desta shell abre o painel de mundo, e um default que briga com um atalho que já existe
é uma armadilha que só o artista descobre»*. **Esta cena tinha exactamente essa armadilha.**

⚠️ Partilhar tecla com o `dash` é deliberado e tem precedente no mesmo mapa (o `jump` e o `move_up`
partilham a seta de cima): *as duas leis nunca correm na mesma cena*.

⛔ **E a medição virou INSTRUMENTO**, que é o que faltou ao censo de 12/08 para não apodrecer:
`a_tecla_do_gatilho_nao_e_reclamada_pelo_editor` (shell), com o **ESPAÇO como controlo positivo** e
o **`P` como controlo de que a varredura vê alguma coisa**. Duas mutações, as duas sangram.

## §4-ter — ⭐⭐⭐ O SEGUNDO silêncio: a acção que EXISTE e não tem tecla (2026-09-18)

O smoke aprovado deixou o loop meio fechado: o painel avisa quando o nome da acção **não existe** e
diz que está tudo bem quando ela existe **sem tecla nenhuma** — e nesse estado o gatilho fica
**exactamente tão calado**.

### A medição

O `ActionState::tick` percorre o **MAPA** e não os dispositivos, e o doc dele escreve a lei por
extenso: *«uma acção sem ligação nenhuma tem de aparecer com `Sample::default()` — declarada e por
atribuir não é inexistente»*. ⇒ **as três leituras dão `false`**, igual a um nome errado.

⇒ *duas causas, o mesmo silêncio, e o painel só nomeava uma.* É a família que o `CLAUDE.md` nomeia:
**um gesto que não faz nada e não diz porquê é indistinguível de um partido**.

### O desenho: um booleano onde a pergunta tem TRÊS lados

```rust
pub enum NoMapa { Desconhecida, SemTecla, Ligada }
impl NoMapa { pub const fn fala(self) -> bool { matches!(self, Self::Ligada) } }
```

⚠️ **As CURAS é que obrigam a distinguir:** uma pede *criar a acção*, a outra *ligar-lhe uma tecla*.
Um aviso só mandaria metade dos artistas ao sítio errado. ⭐ E a `fala()` existe para quem só quer
*«isto vai funcionar?»* não ter de saber que são três — com gate a exigir `false` nos **dois** mudos.

⛔ **O `SemTecla` NÃO conta como órfã no título**, e é decisão: o título diz *«partidas»*, e uma
acção por ligar é uma configuração a meio — a mesma fronteira que o nome de sinal vazio já tem.

### O que o gate teve de provar, e a mutação que o obrigou

| gate | o que afirma |
|---|---|
| `uma_accao_sem_tecla_existe_e_fica_calada_na_mesma` | as **duas** metades: ela existe **e** não fala (com a tecla em baixo, e com o controlo da que fala) |
| `o_aviso_da_accao_sem_tecla_chega_a_pixel` | a frase chega a **GLIFO** — e por comparação de duas cenas, porque um número absoluto não diz nada |
| `o_prologo_deixa_uma_accao_ligada_e_outra_por_ligar` | a cena cria as duas, em estados opostos |

⛔⛔ **E o último SOBREVIVEU à 1.ª redacção:** ele **reconstruía** o prólogo à mão, logo afirmava que
a lei era possível e nunca que o prólogo a seguia. ⇒ a 2.ª metade lê o ficheiro do prólogo por
`include_str!`. *Um gate que chama a função em vez de percorrer a rota afirma que a peça existe,
nunca que quem a usa a usa* — a forma que esta casa já pagou quatro vezes.

⚠️ **E a cena teve de CRIAR a acção sem tecla**, porque as **sete** de fábrica têm todas ligação
(medido, e dentro do gate): um passo de roteiro que mandasse escrever `grab` ensinaria o contrário
do que acontece.

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
- ⛔⛔ **E o buraco de FUNDO, que o report do dono expôs e que esta wave NÃO fecha:** o teclado do
  jogo e o teclado do editor são o mesmo, e o Input Map **não tem âmbitos com prioridade** — o item
  que o `#1` da fila do Vector deixou aberto em 24/08, **bloqueado no `shells/game`/R1, adiado pelo
  dono**. Hoje a cura é escolher a tecla com a medição do §4-bis; a cura de fundo é o âmbito.
