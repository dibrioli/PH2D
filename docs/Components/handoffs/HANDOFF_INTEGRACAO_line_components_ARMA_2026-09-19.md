# HANDOFF DE INTEGRAÇÃO — `line/components` · **A ARMA DO JOGADOR** (2026-09-19)

> Leitor: a próxima LLM e o agente INTEGRADOR. Denso de propósito (`CLAUDE.md` §0.8).
> Plano: [`22_plano_weapon_fire.md`](../22_plano_weapon_fire.md) · Sonda do §5.0:
> [`mede_o_que_a_composicao_ja_da_a_arma`](../../../crates/ph2d-app-components/tests/it/mede_o_que_a_composicao_ja_da_a_arma.rs)

## §1 — O que a jornada entrega, em uma frase

**O `WeaponFire`** — a arma do jogador, que o `#14 ProjectileMotion` deixou **ABERTO por escrito** no
`CLAUDE.md` §5 e que o [levantamento §6](../00_levantamento_componentes.md) descreve como *«completa
o trio WeaponFire + ProjectileMotion + Spawner»*: um gatilho passa a ter **RITMO**, **PENTE** com
recarga, e a fábrica passa a saber **ESPALHAR** uma rajada.

## §2 — A medição do §5.0, antes da primeira linha

O lado medido **não é um espantalho**: são **cinco** componentes que esta mesma linha shipou.

| a pergunta | a resposta MEDIDA | o que ela decidiu |
|---|---|---|
| **A MIRA** — a bala sai apontada? | ⭐ **SIM, ao bit**: `1,5707964` rad do corpo chegam a `Birth::aim`, pose `[2,0 ; 0,0]` | ✅ **já paga** pelo gatilho (#18) — **a wave não lhe toca** |
| **A CADÊNCIA** — segurar um segundo | ⛔ **`60` balas** (a linha `Hold` fala em todo tique e a fábrica nasce a cada sinal) | o buraco nº 1 |
| **…e pelo RELÓGIO** (o desvio que um artista escreveria) | a 1.ª bala no tique **`15`** (`0,250 s` atrasada) e **`3`** onde se pediam `4` | ⇒ o `cooldown` **não** é um `Timer` |
| **AS MUNIÇÕES** | ⛔ `10` gatilhos num pente de `6` dão **`10`** balas, e o contador vai a **`−4`** | o buraco nº 2 |
| **A RECARGA** | ⛔ **um** verbo escreve um contador e ele **SOMA** ⇒ **INEXPRIMÍVEL** | idem |
| **A DISPERSÃO** — 3 balas da mesma rajada | ⛔ os três ângulos são **o MESMO** (`0,0`) | o buraco nº 3 |

⭐⭐⭐ **A medição REESCREVEU a wave em duas frentes:** ela apagou uma das quatro peças do item (a
mira, já paga) e **mudou o dono de outra** (o espalhamento, que é da fábrica — §4.3).

## §3 — Os CONTADORES, como DELTA contra o `main` (⛔ nunca o literal)

| contador | delta | porquê |
|---|---|---|
| `PROJECT_SCHEMA` | **+1** (`154 → 155`) | o `WeaponFire` entra no registo **e** a `Factory` ganha `spread_deg` — **qualquer um dos dois sozinho já obrigava o degrau**, porque o postcard é POSICIONAL |
| registo do `ph2d-ecs` | **+1** (`102 → 103`) | ⛔ **UM só**: nem o `WeaponRuntime` (a cerca é o TIPO) nem a MUNIÇÃO (ela é um `Counter`, que já cá estava) |
| espelhos `ph2d-render` · `ph2d-script` | **+1** cada (`103 → 104`) | eles contam `ecs + render` / `ecs + script` |
| `LIVE_SECTIONS` | **+1** (`37 → 38`) | a secção WEAPON, no MESMO commit que a secção (a lei do censo) |
| `any_live_section` | **+1** (`31 → 32`) | a arity do array muda com ela |
| `ComponentEdit` · `SignalOrigin` | **+1** cada | **append-only**: a posição é a tag |

⚠️ **Conte o DELTA, nunca o literal** — e a coluna `base:` do `collision-surface.sh` é o
**merge-base**, não o `main` de agora.

## §4 — As decisões, e o que cada uma custou

### §4.1 — ⛔ O `cooldown` NÃO é um `Timer`, e a recusa da fábrica continua de pé

A `Factory` **recusou** ter um `rate` próprio, com a razão escrita no `factory.rs`. **Aquela recusa
responde outra pergunta.** Um relógio é uma **emissão periódica** e dispara no **FIM** do período;
uma cadência é um **piso no intervalo entre pedidos honrados**, e o primeiro é **imediato**. A
diferença está medida: pelo relógio a 1.ª bala chega `0,250 s` tarde e a taxa lê `3` em vez de `4`.

### §4.2 — ⭐⭐ O PENTE é um `Counter`, e é isso que o põe no HUD de graça

O `LabelSource::Counter` do #20 lê a soma dos contadores com um nome. Com a munição a viver num
`Counter`: o placar mostra-a **sem uma linha nova**, o Inspector já a edita, *«apanhei um pacote de
balas»* é o `Add to Counter` que já existe, e o `rewind_runtime` já a enche.

⚠️ **A ponte lê e escreve o contador DESTA entidade** (o catálogo declara-o com `requires`): o
`counter::soma` soma todos os do nome — é a pergunta certa para um placar e a **errada** para uma
escrita, que precisa de um dono. *As duas perguntas, cada uma com a sua porta.*

⛔ **A munição não é duplicada no `WeaponRuntime`**: *dois sítios com o número de balas divergem no
dia em que um deles ganhar uma cerca.*

### §4.3 — ⭐⭐ O ESPALHAMENTO é da FÁBRICA, e a prova é o GERADOR

Ele é irmão do `burst` (*quantas de cada vez*) e do `aim_from_spawner` (*para onde*). E há uma prova
mais dura que a arrumação: um espalhamento precisa de um **gerador com semente**, determinista e que
renasça no rebobinar — e a `Factory` tem um (`FactoryRuntime::rng`). Pô-lo na arma criaria **um
segundo gerador**, que é a forma exacta do defeito que o *splitmix64* saiu da fábrica para não
repetir (#25).

⚠️ **Com `spread_deg = 0` a lei DEGENERA e o gerador NEM É TOCADO** — as duas metades, e a segunda é
a que importa: se o sorteio corresse e o resultado fosse descartado, a **sequência** deslocava-se e
uma `SpawnAt::Area` na mesma fábrica punha as cópias noutro sítio.

### §4.4 — A ARMA não tem `master`, e a ausência é a lei

Ela publica `on_fire` e a `Factory` ouve. Instanciar já tem um motor, e um segundo seria a segunda
resposta a *«como nasce uma cópia»*.

### §4.5 — *Rebobinar é RENASCER*

O `WeaponRuntime` entra no `rewind_runtime` (a **oitava** espécie): a cadência zera e uma recarga a
meio é **cancelada**. ⛔ **O pente não é reposto ali** — ele é um `Counter`, e o bloco dos contadores
já o enche do `start`.

## §5 — ⚠️⚠️ O que este diff MEXE em ficheiro PARTILHADO (leia antes de fundir)

| ficheiro | o que muda | risco |
|---|---|---|
| `ph2d-ecs/src/factory.rs` | campo `spread_deg` + o sorteio por cópia | ⚠️ **struct partilhada** — um merge textual pode perder o campo, e o `Default` também |
| `ph2d-ecs/src/rewind_runtime.rs` | mais um bloco + uma linha na tabela do doc | baixo |
| `ph2d-ecs/src/scene/registry.rs` + os 3 testes de contagem | `+1` e `+1`/`+1` | ⚠️ **recontar contra o `main`** |
| `ph2d-runtime/src/lib.rs` | `SignalOrigin::Weapon` + os DOIS `match` exaustivos | ⭐ um `match` esquecido **não compila** — é o desenho |
| `ph2d-editor-core/src/action_bus_component.rs` | `ComponentEdit::Weapon` | append-only |
| `ph2d-editor-core/src/ids/live_sections.rs` | `37 → 38` | ⚠️ a arity do array é literal |
| `ph2d-panel-inspector/src/paint_frame.rs` | `any_live_section([bool; 31 → 32])` | idem |
| `shells/desktop/src/project_schema*.rs` | o degrau `155` e a tripla | ⚠️ **a escada acumula** — ver §5.0 |
| `shells/desktop/src/render_loop/motores_do_quadro.rs` | o motor `armas` | ⚠️ **a ordem interna é load-bearing** (§6) |

⭐ **Um corte de LOC por RESPONSABILIDADE:** o `paint_optional_top20.rs` foi a `616` de `600`, e as
**duas** molduras dos SUPLENTES (o raio e a arma) saíram para `paint_optional_suplentes.rs`. ⚠️ **Não
foi por tamanho:** aquele ficheiro declara-se, no próprio cabeçalho, como *«a CAUDA da fila do
TOP-20»*, e as duas não são dela. ⛔ **Nunca uma entrada nova no `FILE_OVERAGE_OK`.**

## §6 — O que só a árvore COMBINADA pode reprovar

- Os **três censos de registo** (ECS + 2 espelhos): eles vivem em crates que uma linha vizinha pode
  não correr. *Um portão que só corre o que a linha editou é cego a todo espelho* — a **quinta**
  ocorrência.
- Os **censos de TEXTO** (HR-15): os `tr(…)` novos vivem no `ph2d-i18n` e o censo é da soma.
- O **tecto de LOC da shell** (`the_shell_only_shrinks`): ⛔ ele **REPROVOU no portão desta linha**
  (`197 052` contra `196 990`, **62** linhas acima) e está curado por **CORTE** — ver §7-bis. Uma
  rodada que acumule pode voltar a estourá-lo, e a cura é sempre a mesma: **mover para uma crate**.
- A **ordem no quadro**: o `a_arma_corre_depois_do_gatilho` lê o CORPO da porta dos motores, e um
  merge que reordene as chamadas passa `fmt` e `clippy`.

## §7 — A prova de fecho

| o quê | onde | número |
|---|---|---|
| a LEI da arma | `ph2d-ecs/src/weapon_tests.rs` | **12** gates |
| o LEQUE | `ph2d-ecs/src/factory_tests.rs` | **2** gates (a neutralidade e o sorteio por cópia) |
| a PONTE | `ph2d-app-components/src/weapon_bridge_tests.rs` | **6** gates |
| o instantâneo e o dreno | `…/weapon_inspector_tests.rs` | **6** gates |
| o vocabulário e a queixa | `ph2d-editor-core/src/weapon_edits.rs` | **2** gates |
| a secção VIVA (clique REAL) | `ph2d-panel-inspector/tests/it/a_seccao_weapon_esta_viva.rs` | **5** gates |
| a CENA | `…/weapon_smoke_tests.rs` | **8** gates |
| a ORDEM no quadro | `shells/desktop/tests/it/o_cerebro_fala_antes_de_a_tabela_ouvir.rs` | **2** gates |
| a PORTA da precisão | `shells/desktop/tests/it/geometric_tools_keep_their_precision.rs` | **5** gates (era 4 — ver §7-bis) |
| **provas de mutação** | `docs/Components/ferramentas/mutacao_arma_2026-09-19.sh` | **24 de 24 sangram** |
| **portão de fecho** | `scripts/nextest-impacted.sh` | **15 441 de 15 441 verdes** · clippy `-D warnings` a zero · `fmt` limpo |

## §7-bis — ⛔⛔ A CATRACA DA SHELL REPROVOU, e a cura mandou uma folha para casa

`the_shell_only_shrinks` leu **`197 052`** contra o tecto de **`196 990`** — **62** linhas acima, por
ACUMULAÇÃO (esta wave acrescenta o motor `armas`, o prólogo e o dreno). ⛔ **A cura é MOVER, nunca
subir o número**, e o candidato veio de uma MEDIÇÃO e não do tamanho:
`shells/desktop/src/precision_geometry.rs` (**315** linhas, 6 consumidores) é a geometria pura de
quem só move pixels — *um ficheiro assim, numa shell, já é uma lei; falta-lhe o endereço*. Ele vive
agora em [`ph2d-sprite-precision`](../../../crates/ph2d-sprite-precision/), a **sétima** folha nascida
deste molde (`ph2d-cloth` · `-pose` · `-boundary` · `-shake` · `-sweep` · `-topdown`).

**Prova de que nada evaporou:** `8` `#[test]` no ficheiro movido, `8` listados pelo `nextest` na crate
nova — a régua que o §5.0 exige, porque *um `.rs` que nenhum `mod` declara não é compilado, e `check`,
`clippy` e as suítes ficam VERDES com os testes ausentes*.

⚠️⚠️ **E a MEDIÇÃO que escolheu o candidato tinha um ponto cego, que o compilador apanhou:** eu
varri por `^use ` e li `0`, logo declarei a folha **sem dependências** — e ela chama
`ph2d_color::f32_to_half` em caminho **QUALIFICADO**, dentro de uma função. *Um ficheiro sem `use`
não é um ficheiro sem dependências*, e um `grep '^use '` não vê um caminho escrito por inteiro no
sítio onde é usado. A folha declara `ph2d-color` (que é ela própria uma folha, e a conversão
meio-float é o assunto dela), e a afirmação falsa está **corrigida no cabeçalho dela**, com a
medição errada escrita ao lado.

⚠️ **Três funções eram `pub(crate)`** e atravessam a fronteira nova ⇒ `pub`. *Numa shell aquilo
queria dizer «visível à casa»; numa crate quer dizer «privado ao ficheiro».*

⛔⛔ **E o move partiu um gate que NOMEIA UM ENDEREÇO DE FIAÇÃO** — a espécie que o HOWTO classifica
como **falha alto**, que é a barata: `the_preserving_tools_go_through_the_preserving_door` procurava
o literal `precision_geometry::` no corpo das cinco ferramentas. O endereço passou a ser uma const
(`PRECISION_DOOR`) com a mudança escrita nela. ⭐ **E ele ganhou o controlo positivo que não tinha:**
o próprio ficheiro já escrevia a lei para as outras duas portas (*«renomear qualquer uma delas faria
os dois gates acima passarem por não encontrarem nada — verdes sobre um aparelho morto»*) e o
terceiro endereço estava sem ela ⇒ `the_precision_door_is_real` referencia os **símbolos** com a
assinatura exacta, logo uma renomeação deixa de **COMPILAR**, que é mais forte que qualquer asserção
textual.

## §7-ter — ⚠️ Uma flake de FAN-OUT para PROMOVER, e é a mais irónica da lista

O portão devolveu `3` reprovadas numa corrida e **`0` na seguinte, na mesma árvore** — a assinatura
mais forte da família do §5.0 (*um defeito de lógica reprova o mesmo caso sempre*). Duas já são
membros NOMEADOS; a terceira não é:

**`the_pen_down_is_still_a_canvas_copy_and_this_is_its_number`**
([`ph2d-tool-painter/src/tool/paint/measure_input_cost.rs`](../../../crates/ph2d-tool-painter/src/tool/paint/measure_input_cost.rs))
— gate de RAZÃO entre dois relógios, **3 de 3 verde sozinho a `load 26,16`** (mais do dobro da carga
em que reprovou) e **zero linhas** do diff desta linha naquela crate.

⛔ **É o SÉTIMO deste repo cujo doc-comment se declara imune por escrito, e a redacção é a mais
explícita de todas:** ela NARRA uma flake anterior (*«sob a carga da suíte completa os dois flutuam
de forma independente e o gate flakou na 1ª rodada»*), atribui-a a comparar **dois instantes
diferentes**, e conclui que *«medidos juntos, os dois números sobem e descem juntos»*. ⚠️ **Verdade
sobre o par de instantes e falsa sobre o FAN-OUT** — que é exactamente a distinção que aquela lista
existe para guardar. *Uma razão entre dois relógios medidos no mesmo instante ainda é uma razão
entre dois relógios.*

As outras duas, já nomeadas e confirmadas na mesma corrida:
`the_mask_stroke_cost_does_not_follow_the_canvas` (3/3 a `load 10,85`–`11,53`) e
`a_long_stroke_is_bounded_by_the_redundancy_floor_not_by_a_budget` (a família `orcamento`, 3/3 a
`load 26,16`) — as duas com zero linhas de diff nas crates delas.

## §8 — ⛔⛔ TRÊS mutações SOBREVIVERAM primeiro, e as três eram MINHAS

| # | o que aconteceu | a lição |
|---|---|---|
| 1 | *«o primeiro tiro atrasado»* passou: eu pus `cooldown_left_us: 1`, e a lei faz `saturating_sub(dt_us)` **antes** de olhar o gatilho — `1 − 16 666` satura em `0` | ⛔ *uma mutação que não muda o observável lê-se exactamente como uma que sobreviveu* |
| 2 | a âncora do leque levava **asteriscos de markdown** e casou `0` vezes | ⭐ o arnês abortou **em voz alta** (`⛔ ANCORA`) — é para isso que ele existe |
| 3 | *«a caçadeira sem leque»* passou porque o gate era **AUTO-REFERENTE**: ele comparava o campo da cena com a MESMA const que a cena lê | ⛔⛔ *um gate auto-referente afirma que a cena concorda consigo mesma, nunca que ela ensina alguma coisa* ⇒ a barra passou a ser **derivada da geometria** (dois chumbos têm de acabar o voo mais afastados do que a largura de um deles) |

## §9 — ⛔⛔⛔ A FOTO apanhou um defeito com os SETE gates da cena VERDES

A 1.ª redacção pôs as torretas em `y = −0,8`, e a foto (`fotografa_cena.sh`, `1930×1012`) mostrou-as
**cortadas pela borda de baixo**: com meia altura de `0,5 m`, o pé ficava em `−1,3` e a banda acaba
em `−1,19`.

⚠️⚠️ **Os gates estavam verdes porque mediam o CENTRO de cada peça** — e um centro dentro da banda
não diz que a peça inteira cabe. É a mesma família do gate do `#25` que media a ALTURA em vez da
POSIÇÃO. ⇒ o gate passa a medir a **CAIXA**, com a rotação dentro (as três estão a `90°`, logo a
meia-altura no mundo é metade da LARGURA do sprite).

⭐ **E a banda foi RE-MEDIDA na própria foto**, em vez de herdada: a régua do canvas põe o `0` do
mundo em `y = 491 px` de ecrã e o `−100` em `588`, logo `1 m` mede `98 px` e a banda vai de
**`+4,09`** a **`−1,19`**. ⛔ A 1.ª redacção escreveu `4,2`/`1,3` — os números da cena do FIM DE JOGO
— e *um número herdado de outra cena é um palpite com cara de medição*.

## §10 — Cinco leituras do diff que se invertem

1. **`WeaponFire` não tem campo de munição** — e isso não é um esquecimento: o pente é um `Counter`
   NOMEADO, que é o que o põe no HUD, no Inspector e no `Add to Counter` **de graça**.
2. **`spread_deg` está na `Factory` e não na arma** — o dono é quem tem o GERADOR (§4.3).
3. **A mira não aparece nesta wave** — ela estava **paga** desde o #18, e a sonda mede-a a passar
   ao bit. *A quarta peça do item era uma ausência já preenchida.*
4. **O `WeaponRuntime` não é registado e a porta fecha-se pelo TIPO** — ele não deriva `Serialize`,
   logo a linha do registo nem compila.
5. **`WEAPON_MAX_MS_UI` é uma SEGUNDA cópia do tecto, de propósito** — a `ph2d-editor-core` não vê o
   `ph2d-ecs`, e a alternativa era um literal solto dentro do painel. ⭐ Ela só é honesta porque há
   um **gate a atá-la** ao original, e ele vive na `ph2d-app-components`, a única crate que vê os
   dois lados: *quem declara um espelho não pode ser quem o verifica.*

## §11 — ABERTO, e de quem é cada item

| item | de quem |
|---|---|
| **`overheat`** (o levantamento nomeia-o) | **produto** — é a mesma grandeza do pente com outra palavra; construí-lo agora seria um **segundo motor de munição** antes de alguém o pedir |
| munição de **RESERVA** (o pente enche de um depósito maior) | **produto** — pede um 2.º contador e uma lei de transferência; o depósito compõe-se hoje com `Add to Counter` |
| um verbo `Fire` na tabela de acções | ⛔ **recusa medida**: um verbo age sobre um ALVO e uma arma dispara de quem a TEM — o mesmo argumento que manteve o `Shake` fora (#25) |
| o **custo a N armas por quadro** | não varrido (a ponte é `O(armas)` com uma ordenação por identidade) |
| a secção WEAPON no FIM do painel | igual às irmãs desde o #18 — o roteiro manda rolar |

## §12 — O SMOKE (o que o dono vai correr)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_WEAPON_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

**Três torretas, uma tecla (`Q`), e uma diferença cada** — e o CONTROLO ao lado é o que torna a wave
legível (a lei desta linha desde o #13):

| coluna | o que ela demonstra |
|---|---|
| **azul** | ~4 tiros/s, pára aos 6, recarrega sozinha — o **RITMO** e o **PENTE** |
| **cinzenta** | ⛔ **sem arma**: o gatilho liga direto à fábrica e segurar dá uma **MANGUEIRA** — *é isto que a composição de hoje dá* |
| **laranja** | 5 chumbos num **LEQUE**, 2 cartuchas, recarga lenta — o **ESPALHAMENTO** |

⚠️ A acção e a tecla vêm das consts da cena do GATILHO — ⛔ re-declará-las seria a segunda resposta
a *«que tecla dispara neste app?»*, e o `Q` foi **medido** (o dono rejeitou o espaço em 18/09).
