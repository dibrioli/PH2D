# Handoff de integração — `line/physics`, W10: a quina e o vagão (2026-08-04)

**Status:** FECHADO 2026-08-04 · no `main` em `b5005e7f7` (o commit que trouxe este arquivo).

> **A linha FECHA aqui e PARA.** Integração e ship só por ordem EXPLÍCITA do Enio
> (CLAUDE.md §0.7), por um agente integrador dedicado munido deste documento.

## §1 — O que a wave é

As **duas metades que o W8 nomeou e não construiu**, e que o handoff de 04/08
listava no §7 como abertas: **corner correction** e **lift momentum**.

Nada do desenho do W8 foi refutado. **Uma frase dele estava errada** e três
números que ele não tinha vieram da medição — os três estão abaixo.

## §2 — (A) A QUINA — `corner_reach`

O gesto que ela perdoa é o mais comum de um platformer: o jogador pula rente a
uma plataforma, a cabeça encosta na quina por três centímetros, o solver come a
velocidade vertical inteira, e o personagem despenca de um pulo que — para o olho
— tinha passado.

**Preditiva, como o W8 exigiu.** Subindo, o sensor mede o teto que a cabeça
alcançaria no PRÓXIMO tique (`rel_up · dt · CORNER_LOOKAHEAD`); se um
deslocamento lateral pequeno o livra, o personagem é movido **antes** do contato.
Nada é devolvido porque nada foi tirado.

### ⚠️ O W8 dizia "boost lateral", e a medição derrubou

Um impulso de `escape / dt` dá o deslocamento certo **neste** tique e
**sobrevive**, porque ninguém o remove — o personagem sai derivando de lado. A
mutação que o instala mede **5,05 m** de desvio contra os **0,11 m** do
deslocamento (com o controle aéreo desligado, que é o que torna o oráculo
honesto: com ele ligado o próprio controle aéreo frearia o boost e o gate não
distinguiria as duas implementações).

Por isso `PlayerStep::nudge` é **metros**, não um `Motor`, e a ponte o aplica por
`PhysicsWorld::nudge_body` — que escreve a translação e **não toca a velocidade**.
⚠️ Não é `set_body_pose`: aquela zera a velocidade, e usá-la aqui mataria o pulo
no exato tique em que a assistência tenta salvá-lo.

### ⚠️ A resolução do perfil foi MEDIDA, e o primeiro corte estava errado

O sensor é um **perfil de 65 raios** ao longo da largura do corpo mais o alcance,
dos dois lados. O primeiro corte usava **25**, e o passo saía 2,7 cm num corpo de
40 cm: **um encosto de 10 cm não era salvo com o alcance em 12 cm** — a meia
célula que uma amostra não pode afirmar, mais o arredondamento da busca, comiam os
2 cm de folga. Com 65 o passo cai para 1,0 cm.

⚠️ **O custo não foi o que decidiu, porque ele não existe:** o sensor inteiro
(perfil + as duas laterais) mede **+0,0004 ms por tique de subida** — cerca de
8 ns por raio, e só nos tiques em que o personagem sobe. Escolher 25 para
"economizar" seria pagar precisão por nada.

| encosto | pico SEM | pico COM | desvio lateral |
|---|---|---|---|
| 0,04 m | 0,784 | **0,833** | −0,052 |
| 0,08 m | 0,741 | **0,833** | −0,090 |
| 0,10 m | 0,727 | **0,833** | −0,112 |
| 0,12 m | 0,716 | 0,716 | 0,000 |
| **0,20 m (cabeça inteira)** | **0,702** | **0,702** | **0,000** |

A última linha é a que separa a assistência de um teletransporte.

### As três metades da lei, e por que nenhuma é opcional

- **O perfil** diz onde a quina está — *"a cabeça bate?"* responde-se com um raio,
  mas *"quanto preciso andar de lado?"* pede saber ONDE.
- **A busca** tem resolução PRÓPRIA (`CORNER_SEARCH_STEPS`), separada da do
  perfil: amarrá-las somaria os dois erros de quantização.
- **A folga lateral** (`side_clear`) impede a cura de ser pior que a doença.
  ⚠️ **A primeira fixture do gate dela não continha o fenômeno:** uma parede alta
  ao lado **já aparece no perfil do teto** (os raios sobem a partir do topo da
  caixa, e ela está ali), então a mutação que deleta a `side_clear` ficava VERDE.
  O caso que só ela cobre é o **bloco baixo** — a cabeça já passou dele, o corpo
  não.

## §3 — (B) O VAGÃO — `lift_momentum`

### ⚠️ A doença não era do solver

O corpo **sempre** manteve a velocidade que a plataforma lhe deu — isso é
conservação de momento, e o rapier a faz de graça. Quem a apagava era a
**assistência**: a caminhada mira `drive × speed` *relativo ao chão*, e no ar o
chão valia zero, então no tique em que o pé sai de um vagão a 4 m/s o alvo salta
para o referencial do MUNDO e o controle aéreo começa a frear justamente o que a
física acabou de dar. Medido: o pulo avançava **11% do voo balístico**.

### ⚠️ O desvanecimento foi construído e REPROVADO pela medição

A primeira versão desvanecia a memória linearmente na janela — mais suave, e
entregava **metade**: o alvo caía continuamente e o controle aéreo freava o tempo
todo (**1,03 m** contra os 2,67 do balístico). A lei que ficou **SEGURA** o valor
cheio e solta no fim da janela; o degrau não é solavanco porque o que muda ali é o
ALVO, e o controle aéreo é uma aceleração limitada.

| janela | avanço no voo | fração do balístico |
|---|---|---|
| 0,00 s | 0,291 m | **11%** |
| 0,25 s | 1,358 m | 51% |
| 0,50 s | 2,291 m | 86% |
| **0,75 s** | **2,667 m** | **100%** |

**O default é 1,5 s e é função do PULO:** um pulo default de altura cheia fica
**1,45 s no ar** (pico 2,101 m, medido). Em chão estático a memória é `[0, 0]`, o
que torna o default ligado **inerte** em toda cena sem plataforma móvel — e há
gate afirmando a identidade, não um limiar.

## §4 — Superfície e números

- **`PROJECT_SCHEMA` 51 → 52** — `PlatformPlayer` ganhou `corner_reach` e
  `lift_momentum`; postcard é posicional. ⚠️ **PROVISÓRIO: o valor se CONTA contra
  o `main` do dia** [[feedback_numbers_that_sum_across_lines_count_dont_pick]].
  Tripla `(52, 13, 14)`.
- ⚠️ **A escada tinha um BURACO e ele foi fechado aqui:** a W-JointCustom bumpou
  50→51 **sem escrever o degrau**. Uma escada com buraco não é cosmética — o
  próximo a bumpar lê o último degrau documentado, conta a partir dele e escreve
  um número que já existe, que é exatamente a colisão que esta linha e a
  `line/FLIP` já pagaram **três** vezes. **Quem bumpa documenta no MESMO commit.**
- **Registro do `ph2d-physics-ecs`: 26, intocado** (nenhum componente novo).
- **`physics_ecs_c9`: `b3dbe792…`, 108 corpos, debug ≡ release** (era
  `8c7ba624…`, 101). ⚠️ **A lane nova é VIVA por ABLAÇÃO** — com `corner_reach = 0`
  no player dela o hash volta ao anterior. A primeira versão da lane era **INERTE**
  (a beirada estava em 3,3 e o personagem não a alcançava, porque a fita segura o
  pulo por 8 tiques e o `cut_gravity` corta o resto); a altura foi **varrida** até
  o ramo ficar vivo.
- **Gizmo ids: nenhum novo** (o último segue 973, próximo livre **974**).
- **Nenhum ADR** (tudo sob o ADR-0131) · **zero `Cargo.toml`** · **nenhuma dep
  nova** · **nenhuma crate nova** · contrato congelado intacto.

### Superfície pública nova (`ph2d-platformer`)

`corner::{CORNER_SAMPLES, CORNER_SEARCH_STEPS, CORNER_LOOKAHEAD, CeilingProbe,
corner_offsets, corner_escape, corner_nudge, corner_probe_wanted}` ·
`jump::carried_frame` · `relative_rise` · `PlayerStep::nudge` ·
`JumpConfig::{corner_reach, lift_momentum}` · `JumpState::{lift, lift_time}`.

⚠️ **Duas assinaturas mudaram:** `player_motor` ganhou o 3º argumento
(`ceiling: Option<&CeilingProbe>`) e `walk` ganhou o 6º (`carried: Vec2`).
`[0, 0]` no segundo é byte-idêntico ao mundo de antes.

⚠️ **E a nota antiga do `player_motor` foi CORRIGIDA em vez de cumprida.** Ela
prometia empacotar *o quadro físico* "quando a lista crescer de novo". O argumento
que entrou **não é** parte do quadro físico — é um **segundo SENSOR**, irmão do
`sample`, e empacotá-lo com `gravity`/`dt` juntaria coisas que mudam por motivos
diferentes. O pacote certo, no dia em que valer a pena, é *os sentidos*.

### Wrapper (`ph2d-physics`)

`PhysicsWorld::body_aabb` (⚠️ **todas** as formas do corpo — a lição da
W-Compound escrita como código: uma caixa é sempre UMA, seja o corpo simples ou
composto) · `PhysicsWorld::nudge_body`.

## §5 — Gates e mutações

**8 mutações, 8 sangram.** Uma sobrevivente **documentada, não escondida**:

- **`CORNER_LOOKAHEAD 2.0 → 1.0` passa nos cinco gates de comportamento.** O
  segundo tique é MARGEM, não correção — o deslocamento é de POSIÇÃO e acontece
  antes do `step` do mesmo tique, então ver a quina no tique do contato já
  bastaria. Ele fica em 2,0 por dois motivos que não são *"senão quebra"*, e os
  dois estão escritos na const ([[feedback_layered_defenses_need_per_layer_gates]]).

As que sangram: ignorar a folga lateral · a busca ignorar o alcance autorado · o
escape virar boost · o deslocamento zerar a velocidade · o referencial carregado
ser sempre zero · a janela nunca escorrer · o referencial vazar para o CHÃO · a
memória guardar zero em vez da velocidade do chão.

**Sondas** (`--ignored`): `measure_corner` (o que o alcance salva · o custo · a
janela da chaminé) · `platform_lift::{measure_what_the_window_delivers,
measure_how_long_a_default_jump_lasts}`.

## §6 — A UI, pelas quatro condições da política

1. **Os componentes existem** — `corner_reach` e `lift_momentum` no
   `PlatformPlayer`.
2. **Pintados e registrados** — duas rows novas no card **FORGIVENESS**
   (`PLAYER_ROW_COUNT` 19 → 21), com dica de hover, pela mesma tabela de três
   consumidores.
3. **O clique chega ao barramento** — a varredura de seam cobre as 21
   (`every_number_raises_its_own_edit`, que **reprovou sozinha** quando a tabela
   cresceu, que é o que ela existe para fazer).
4. **A sequência leva a algum lugar** —
   `the_two_w10_assists_land_on_the_component_and_are_clamped`.

⚠️ **A unidade está no RÓTULO** (`Corner Reach (m)`) porque as quatro rows do card
**não são a mesma grandeza**: três são segundos e uma é metros. Sem o `(m)` ali,
um artista que leu as três de cima escreve `0.1` esperando um décimo de segundo e
recebe dez centímetros.

**A metade VISÍVEL: nenhum overlay novo, e é decisão.** As duas assistências se
veem no próprio personagem — uma o desloca, a outra muda onde ele pousa. Um
desenho para elas seria um marcador que só aparece no tique em que já não há mais
nada a mostrar.

## §7 — Smokes

- **`PH2D_PHYSICS_SMOKE=89`** — **A CHAMINÉ**. Um vão de 0,60 m sobre um corpo de
  0,40. A janela em que o pulo passa sai de **±0,10 m para ±0,22 m** (medido). O
  controle é `Corner Reach = 0` no painel; o passo 4 é a outra metade — mire 30 cm
  fora e ele **bate**.
- **`PH2D_PHYSICS_SMOKE=90`** — **O VAGÃO**. Pular PARADO numa plataforma que anda
  e pousar em cima dela. O controle é `Lift Momentum = 0`: o vagão sai debaixo do
  personagem.

⚠️ **A tecla de PULO é a SETA PARA CIMA (ou `Z`), nunca o Espaço** — ele é o
Play/Pause do transporte, e as duas cenas o dizem.

## §8 — Aberto, com o preço ao lado

- **A folga lateral olha só a altura do MEIO do corpo.** Uma geometria que ocupe
  o espaço de escape só perto dos pés (ou só perto dos ombros) não é vista. Curá-la
  é um segundo par de raios, ou um shapecast — e o shapecast traz de volta a
  pergunta *"qual das formas de um corpo composto?"*, que a caixa envolvente
  responde hoje sem discussão.
- **A quina é só para CIMA.** Um personagem que bate a cabeça é o caso que rouba
  um pulo; um que raspa um ombro numa parede lateral é outra assistência (o
  *"wall nudge"*), com sensor próprio.
- **`lift_momentum` mede um TEMPO, e há um desenho alternativo não construído:**
  guardar a memória até o POUSO (sem janela) é o que a maioria dos platformers 2D
  faz na prática. A janela foi escolhida porque o plano a nomeou e porque ela dá ao
  artista um dial; se um smoke mostrar que a memória "acaba no meio do voo" em
  algum ponto de operação, o número é dele.
- **As duas assistências não têm entrada no `BUGS_physics.md`** — nenhuma delas
  nasceu de um bug cuja causa enganava.

---

# W11 — O REPOUSO (2026-08-04, `fc711de0a`)

Wave de **correção**, aberta pelo smoke da W10. Um commit, e ele é pequeno de
propósito: 2 arquivos de produto (um deles uma linha de re-export), 2 de teste,
2 de doc.

## §9 — O que a wave é

Relato do Enio: *"de tempos em tempos enquanto está parado o player dá pulinhos
involuntários"* e *"nas rampas, se parado, a depender do Float Height ele pode
subir a rampa sozinho bem devagar"*.

**O segundo reproduziu e está corrigido; o primeiro NÃO reproduziu** — em cinco
configurações de repouso (plano estático · vagão dinâmico a 3 m/s · vagão
kinematic · `float_height` abaixo do mínimo geométrico · dispatch devendo
0/1/2 tiques) a folga fica constante ao sexto decimal e nenhum tique move mais
de 2 mm. Está escrito assim no [`BUGS_physics.md`](../BUGS_physics.md) §7, com a
sonda que a próxima hipótese deve usar.

## §10 — A causa, e por que ela não estava onde eu procurei

⚠️ **A ablação pela GRAVIDADE é o que fecha:** a deriva é **linear em `g`** e vale
**zero** num mundo sem gravidade. A perna cancela a gravidade com um **impulso no
topo do tique** e o `rapier` a integra **ao longo** dele — sobra meio tique de
velocidade para cima, com componente **tangente** numa rampa. O freio da
caminhada não a vê porque é um controlador de VELOCIDADE amostrado no fim do
tique (medido: `v` perpendicular à tangente ao 4º decimal, freio a calcular
`−8,7e−8`), e ninguém removia o que sobrava porque o **eixo do amortecedor era o
`up`** — verdade **só no plano**, onde a normal É o `up`.

⚠️ **Quatro hipóteses minhas foram refutadas por medição e estão registradas** —
a "oscilação" (era a própria subida, medida por uma métrica que mede a rampa), o
erro de repouso da mola, a rigidez, e a de-polarização de meio tique (construída
e **revertida por PIORAR** a deriva). Elas estão no §7 do BUGS para ninguém as
reconstruir.

## §11 — Superfície e números

- **Público novo:** `ph2d_platformer::damping_axis` (1 função pura).
- ⚠️ **`PROJECT_SCHEMA` INTOCADO** · registro do `ph2d-physics-ecs` **intocado** ·
  gizmo ids **nenhum** (próximo livre segue **974**) · **nenhum ADR** · **zero
  `Cargo.toml`** · contrato congelado intacto.
- ⚠️ **`physics_ecs_c9` = `b3dbe792…`, 108 corpos, debug ≡ release — o MESMO do
  W10.** Não é sorte: a lane do player é em chão plano, onde `n == up` exato, e o
  hash é o oráculo mais forte disponível para a byte-identidade que a wave alega.

**Deriva a 30°, 10 s, `float_height = 0,9`:** 0,3295 → **0,1644 m** com o default;
**0,0000** com o amortecimento no teto.

## §12 — ⚠️ A DECISÃO que fica com o Enio, com as quatro colunas

O `spring_damping` no **teto zera a deriva** — e custa peso:

| `spring_damping` | deriva parado | quique ao pousar | peso transmitido |
|---|---|---|---|
| 0,25 | 0,2476 m | **196 mm** | 88% |
| **0,50** (o que shipa) | 0,1644 m | 20 mm | **77%** |
| 0,75 | 0,0819 m | 0 mm | 65% |
| 1,00 | **0,0000 m** | 0 mm | **53%** |

A perna segura o personagem em parte com um **boost**, e boost não é força: o
`react` não o devolve ao chão (cerca **medida** daquele módulo — devolvê-lo fazia
a jangada disparar). ⚠️ **Subir a rigidez não recupera** (`k · erro` constante em
4,60 m/s², medido de 400 a 6400).

**Eu NÃO mudei o default**, e a razão é a lei do repo: um número cujo único valor
correto é o teto de outro knob é bug de design, não default. O que a wave entrega
é que **o knob passou a governar isto de verdade** — com o eixo no `up`, nenhum
valor dele removia a deriva (1,0 dava 0,3276 contra 0,3295 do controle).

## §13 — Gates e mutações

- **Lei** (`ph2d-platformer::ride::tests`, 4 novos): byte-identidade do eixo no
  plano · **andar ao longo da rampa não acorda o amortecedor** (o gate da
  correção) · o teto zera a aproximação num tique · o default é uma decisão.
- **Produto** (`ph2d-physics-ecs/tests/platform_idle.rs`, 5): com o amortecedor
  no teto o personagem fica parado em 10°/20°/30°/40° **e por um minuto** · o
  resíduo do default é pinado **dos dois lados** · o plano é o controle · a perna
  segura a altura.
- **Sonda** (`measure_idle.rs`, `--ignored`): as sete medições da wave.
- **4 mutações, 4 sangram.** O eixo de volta ao `up` sangra a lei **e** o produto.
- ⚠️ **Um gate meu nasceu a sangrar pelo motivo errado** e foi corrigido: o
  `sliding_along_the_ramp…` herdava o `spring_damping` do `STARTING_POINT`, então
  mexer no default o derrubava — ele julga o EIXO, e agora fixa o amortecimento.
- ⚠️ **E a fixture do gate de produto nasceu a medir o ASSENTAMENTO** (0,0115 m
  de "viagem" no chão plano, que é a perna a acomodar-se). O defeito é um regime
  permanente; a medição começa depois de 2 s.

## §14 — Aberto, com o preço

- **A cura que compra as duas colunas: a perna SUBSTITUI a gravidade em vez de a
  cancelar.** Sem impulso de cancelamento não há assimetria a retificar (a deriva
  vai a zero em qualquer amortecimento) e a perna deixa de segurar o personagem
  por boost (o peso volta inteiro). ⚠️ **É uma wave, não um fix:** toca o output
  da lei (a gravidade que o corpo deve sentir neste tique), uma escrita por-tique
  de `gravity_scale` no wrapper, o `rewind`, o hash do c9 e a semântica do
  `react` do W6 — que é smokada e aprovada. A medição inteira dela já está feita.
- **O "pulinho" segue sem repro.** Se o smoke o mostrar, a sonda `measure_idle`
  já traz o instrumento (folga por tique, saltos por tique, partição por chão).
- **`float_height` abaixo do mínimo geométrico** (0,5 nesta cápsula) faz a
  cápsula ENCOSTAR e o solver de contato assume: ali ela escorrega para BAIXO
  (−0,0745 m a 30°). É pré-existente, é o que o `min_float_height` já documenta,
  e é provavelmente o *"a depender do Float Height"* do relato.

## §15 — Smoke

**Não há cena nova.** O que a wave corrige é visível nas cenas que já existem:

```
env PH2D_PHYSICS_SMOKE=81 cargo run -p ph2d-host-desktop --release   # a rampa de 30°
env PH2D_PHYSICS_SMOKE=88 cargo run -p ph2d-host-desktop --release   # o par 40°/50°
```

⚠️ **O gesto é NÃO FAZER NADA:** marque **Physics** no transporte, dê Play, leve o
personagem até a rampa e **solte as teclas**. Ele tem de ficar onde está. O
controle é o painel: **Spring Damping** em `1,0` deixa-o cravado, e é lá que a
tabela do §12 vira uma escolha sua — a jangada da cena `=85` afunda menos com ele
no teto.
