# TOP-20 #13 — `TopDownPlayer`: o plano

> **Fila:** [levantamento §7](00_levantamento_componentes.md) item **13**, *«o 2º controller
> canônico; reusa o desenho lei-pura+ponte validado no platformer»*. Os #1–#12 estão fechados.
> **Entrega declarada:** [síntese de movimento §TopDownPlayer](pesquisa/sintese_movimento_fisica.md).
> **Protocolo:** `/pd-feature` — plano antes de código, oráculo CORRIDO, números MEDIDOS.

---

## §0 — O que o artista consegue FAZER quando isto fechar

Põe um objecto na cena, carrega em **+ Add Component → Top-Down Player**, e:

1. as setas movem-no em **8 direcções** (ou 4, ou livre, ou só num eixo — é um menu);
2. **ao raspar numa parede em diagonal ele desliza ao longo dela à velocidade CHEIA**, em vez de
   ficar preso ou de rastejar — é o caso que a síntese chama *«o que trava iniciante»*;
3. um menu **Viewpoint** reprojecta a entrada para **isometria** sem trocar de componente: a mesma
   seta «cima» anda para o fundo do tabuleiro isométrico, não para o topo do ecrã;
4. um menu **Rotation** vira o objecto para onde ele anda (suave, ou em passos de 90°/45°);
5. desligando **Default Controls**, o componente vira motor puro — quem manda passa a ser um sinal,
   a timeline ou um script, e o corpo continua a deslizar pelas mesmas leis.

---

## §1 — O ORÁCULO (§0.9): triagem, porta e o que ele respondeu

### §1.1 — Triagem de licença — **pára na primeira porta aberta**

| app | artefacto instalado | licença | veredito |
|---|---|---|---|
| **Godot** | `/usr/bin/godot` 4.7.2 | **MIT** (`pacman -Qi godot`) | ⭐ **PORTA ABERTA** — corre-se e porta-se, com atribuição |
| GDevelop | — | — | **não instalado** |
| Construct 3 | — | — | não instalado (proprietário, web) |
| Unity / Unreal | — | — | não têm controller 2D de prateleira (dossiês) |

⇒ o oráculo é **o Godot**, corrido sem interface:
`godot --headless --script docs/Components/ferramentas/godot_topdown_probe.gd`.

⚠️ **Metade da entrega NÃO tem oráculo instalado, e isso diz-se:** o *dropdown de viewpoint*
(isometria) é do **GDevelop**, que não está nesta máquina. Essa metade é desenhada da geometria (uma
base 2×2) e defendida por gates próprios — ⛔ nunca apresentada como paridade com ninguém.

### §1.2 — O alvo dentro do Godot

`CharacterBody2D` com **`motion_mode = MOTION_MODE_FLOATING`** — que a doc dele chama o modo de vista
de cima — mais `move_and_slide()`. ⚠️ Ele resolve **a metade de baixo** (colisão e deslize); gravidade,
aceleração, direcções e isometria o utilizador escreve. É a lacuna que o dossiê já nomeava.

### §1.3 — ⭐⭐⭐ O que a corrida devolveu (a tabela é o §6)

Quatro leis, todas com número:

1. **O ORÇAMENTO DE MOVIMENTO CONSERVA-SE.** Ao deslizar, o corpo anda `|v|·dt` **inteiro** ao longo
   da parede — não a projecção tangencial. A 45° isso é `1,414×`; a 20° de incidência é **`2,92×`**.
2. **Abaixo de `wall_min_slide_angle` (15° de fábrica) ele PÁRA**, e o penhasco é do knob: medido ao
   grau, `15°` anda `0,017` e `16°` anda `3,999`; **com o knob a `0` o penhasco desaparece** (controlo).
3. **O motor NUNCA reescreve `velocity`** em `FLOATING` — o deslize é só posicional. Em `GROUNDED`
   ele **zera** a componente normal.
4. **Uma quina interior PÁRA**, com uma oscilação de `±0,06 px` (a margem de des-penetração).

### §1.4 — ⛔⛔ E a mesma corrida acusou o NOSSO lado

O par da sonda corre sobre o nosso `PhysicsWorld::move_character`
([`character_slide_probe.rs`](../../crates/ph2d-physics/src/world/character_slide_probe.rs)) e a
razão `|d| / tangencial` dá **`1,000` em TODOS os ângulos** — nós fazemos a **projecção**, que é a
lei do platformer. ⇒ *o nosso controlador rasteja numa parede*, e o item 13 não é empacotar o que já
existe: é uma lei que a casa não tem.

⚠️ **E não é escolha da casa:** o `slide: true` do `KinematicCharacterController` é da `rapier`. A
cura é um **laço de orçamento NOSSO** por cima da porta (§2.1).

### §1.5 — ⚠️ O que esta sonda pagou para dizer a verdade: **quatro fixturas mediram o VAZIO**

| bloco | o que a fixtura fazia | como se LIA |
|---|---|---|
| B (1.ª) | `dir = (−cos, sin)` apontava para **fora** da parede | `\|d\| ≈ 4,0` em todos os ângulos ⇒ *«o deslize é perfeito»* |
| E (1.ª) | 20 tiques = 56,6 px contra um chão a 84 | as duas colunas idênticas ⇒ *«o dropdown não muda nada»* |
| E (2.ª) | a parede vivia em `x ∈ [−2000, 0]` e o corpo anda para **+x** | idem |
| F (1.ª) | rectângulo rodado **sobre a origem**: encontrava a PONTA, não a face | números plausíveis e mudos |

⭐ **As quatro dão resultado, e as quatro respondem a uma pergunta que ninguém fez.** É a família
«a fixtura não contém o fenómeno», e aqui ela apareceu **quatro vezes na mesma sonda** — a defesa que
funcionou foi **ler a tabela linha a linha** contra o que a geometria obriga, nunca o exit code.

---

## §2 — O DESENHO (uma porta por pergunta)

### §2.1 — ⭐⭐⭐ A LEI: o orçamento sobrevive, a direcção é que muda

A diferença entre um mover de plataforma e um de vista de cima **não é a gravidade** — é isto:

```
plataforma (a nossa hoje)   deslize = projecção  ⇒ anda |v|·dt·sin θ
vista de cima (o oráculo)   deslize = orçamento  ⇒ anda |v|·dt,  na tangente
```

A lei pura devolve um **plano de passos**, e a ponte alimenta cada passo com *«quanto coube»*:

```rust
pub struct SlideStep { pub dir: Vec2, pub budget: f32 }
pub fn first_step(v: Vec2, dt: f32) -> Option<SlideStep>;
pub fn next_step(step: SlideStep, moved: f32, normal: Vec2, cfg: &SlideLaw) -> Option<SlideStep>;
```

⚠️ **É uma porta só, e ela é RE-ENTRANTE** — não há uma segunda função «com deslize». O laço vive na
ponte porque é ela que tem o mundo; a decisão vive na lei porque é ela que se testa sem mundo nenhum.

⛔ **A cerca:** `next_step` devolve `None` quando (a) o orçamento acabou, (b) o ângulo de incidência
é `<=` ao `min_slide_angle` (a lei nº 2 do oráculo, com o **`<=`** medido: 15° ainda pára), ou (c) o
plano esgotou `max_slides`.

### §2.2 — A ORDEM: intenção → quantizar → reprojectar → mundo; a rotação lê o FIM

```
entrada crua ──► quantizar (4/8/livre/eixo) ──► reprojectar (viewpoint) ──► direcção de MUNDO
                        ▲                                                         │
             no espaço da INTENÇÃO                                     a rotação lê AQUI
```

⚠️⚠️ **Quantizar DEPOIS de reprojectar é o defeito que faz a isometria não servir para nada:** num
tabuleiro 2:1, «4 direcções» tem de encaixar nas diagonais do tabuleiro, e um `snap` feito no ecrã
encaixa nos eixos do ecrã. As duas ordens compilam e só uma é um jogo isométrico.

⚠️ **A rotação lê a direcção de MUNDO** — o desenho está no ecrã, não no tabuleiro.

### §2.3 — ⛔ Por que é COMPONENTE PRÓPRIO, e não um modo do `PlatformPlayer`

O Godot resolve com um dropdown (`motion_mode`) e **o argumento dele não transfere**: o
`CharacterBody2D` é fino (só colisão), o nosso `PlatformPlayer` é o controlador inteiro.

**Contado no ficheiro:** `PlatformPlayer` tem **55 campos**, e os que significam alguma coisa sem
gravidade e sem chão são **`speed`, `acceleration`, `brake_scale`, `reaction_push`** — *quatro*.
⇒ um modo entregaria **51 knobs mortos** na mesma secção de painel, que é exactamente o defeito que
o painel do L-System pagou (*«nenhum molde mostra um knob que a gramática não sabe LER»*).

⭐ O que se **reusa** é o desenho e o vocabulário, não a struct: crate de lei pura irmã, ponte no
mesmo sítio, `PlayerMode` (Dynamic/Kinematic/Pure) **partilhado** — ele já é componente separado.

### §2.4 — Um dono do `Transform` por vez

Lei transversal 2 da síntese. `TopDownPlayer` + `PlatformPlayer` na mesma entidade é **conflito**, e
o Inspector **acusa-o** (uma linha de aviso na secção), ⛔ nunca comportamento indefinido.

### §2.5 — `Default Controls` + `Simulate Control` (lei transversal 3)

Ligado (de fábrica), o componente lê as acções **nomeadas** do Input Map. Desligado, ele só obedece a
quem lhe escrever a intenção — e a ponte não toca no teclado.

⚠️ **Faltam DUAS acções ao mapa de fábrica** (contadas em [`map.rs`](../../crates/ph2d-input/src/map.rs):
existem 6, e nenhuma é «andar para cima»). O doc daquele ficheiro diz *«acrescentar é livre»*.
⇒ `move_up` e `move_down`, atados às **setas** — ⛔ **não** a `W`/`S` de fábrica: o `W` já abre o
painel de mundo desta shell, e um default que briga com um atalho existente é uma armadilha que só o
artista descobre. WASD fica como **decisão do dono** (§8), a dois cliques no painel de Input Map.

### §2.6 — O `min_slide_angle` é um knob, e o default é MEDIDO

`15°`, que é o do oráculo — e o motivo é físico, não estético: sem ele um toque quase frontal numa
parede atira o corpo para o lado à velocidade cheia. ⚠️ O penhasco é **descontínuo por desenho**
(`235×` entre 15° e 16°); é assim no alvo, e a alternativa (rampa) seria produto novo sem lado
aprovado.

### §2.7 — A porta do mundo ganha um IRMÃO, e o velho delega

O laço precisa de perguntar *«e a partir DAQUI, quanto cabe?»* sem escrever a pose (escrevê-la sem
`step()` deixa o colisor desactualizado). ⇒

```rust
pub fn move_character(&self, h, wanted, params, excl, layer, hits) -> CharacterMove {
    self.move_character_from(h, [0.0, 0.0], wanted, params, excl, layer, hits)
}
pub fn move_character_from(&self, h, offset: [f32; 2], wanted, …) -> CharacterMove { … }
```

⭐ É a forma que a wave da fábrica já pagou (`deep_copy_subtree` → `…_many`): **porta nova, porta
velha a delegar**, zero chamadores a mexer, e gate a manter o delegado com um corpo só.

---

## §3 — Schema e contratos congelados: a PROVA

| grandeza | delta | porquê |
|---|---|---|
| `PROJECT_SCHEMA` | **+1** (`130 → 131`) | um componente registado novo (`TopDownPlayer`) |
| registo da FÍSICA (`register_physics_components`) | **+1** (`33 → 34`) | é lá que ele vive |
| espelho `ph2d-render` · espelho `ph2d-script` | **0** | ⛔ **a 1.ª redacção disse `+1` e ESTAVA ERRADA** |
| registo do `ph2d-ecs` | **0** | idem — o componente não é de lá |
| `VEC_SCENE_SCHEMA` · `FLIP_SCHEMA` · `FIELD_DOC_VERSION` | **0** | não os toca |

⛔⛔ **A correcção vale por si:** os dois espelhos contam `ecs + render` e `ecs + script`, e **não**
a física. Um componente de física move dois contadores, não quatro — e escrever `+1` nos quatro
teria deixado dois gates vermelhos na integração com a cura errada ao lado.

⛔ **Contrato congelado (§6): não encosta.** `Tool` / `RasterEditTool` / `CanvasPaintTool` /
`PanelEvent` / `NodeOp` / `OpResolver` / `NodeManifest` / `VectorOp` — nenhum aparece no desenho.
*A prova por `grep` corre na W1 e vai para o handoff, não para aqui.*

⚠️ **O `move_character_from` é FOUNDATIONAL** (`ph2d-physics`), e o ADR-0107 autoriza-o sob o
protocolo testado. ⭐ Não há `line/physics` viva (`git worktree list`, 14/09) — colisão de símbolo
é improvável e está declarada no handoff.

---

## §4 — A UI: as QUATRO condições, que são independentes

1. **EXISTE** — `TopDownPlayer` no catálogo ([`catalog/physics.rs`](../../crates/ph2d-component-desc/src/catalog/physics.rs)),
   com `RigidBody` como componente requerido (o molde do `PlatformPlayer`);
2. **É PINTADO E REGISTADO** — secção do Inspector irmã de
   [`sections/player_rows.rs`](../../crates/ph2d-panel-inspector/src/sections/player_rows.rs), com
   ids próprios e `HitIndex::register` em cada linha;
3. **O CLIQUE CHEGA AO BARRAMENTO** — `EditorAction::InspectorTopDownEdit`, com o vocabulário a
   nascer **abaixo** do `action_bus` (a catraca do DAG já mordeu seis vezes: o molde é o
   `ph2d_editor_core::factory_edits` da wave anterior);
4. **A SEQUÊNCIA LEVA A ALGUM LADO** — o valor editado chega ao componente e o corpo **anda
   diferente** no mesmo quadro; é a costura que só um gate de gesto real mede.

⚠️ **O painel é SEMEADO do snapshot**, nunca dos defaults do `populate` — o defeito que as waves do
áudio e da câmera pagaram (*«números plausíveis que não são os da cena»*).

---

## §5 — Os gates, red-first, e a fixtura que CONTÉM o fenómeno

**Lei pura** (`ph2d-topdown`):
- `o_orcamento_sobrevive_ao_deslize` — o corpus do oráculo, uma linha por ângulo;
- `abaixo_do_limiar_ele_para_e_o_controlo_e_o_knob_a_zero` — ⚠️ **com o controlo dentro**, que é o
  que separa «o knob manda» de «a geometria manda»;
- `a_quantizacao_corre_no_espaco_da_intencao` — a mesma entrada em 2:1 dá direcções do tabuleiro;
- `a_rotacao_le_a_direccao_de_mundo`;
- `o_neutro_e_identidade_ao_bit` — `viewpoint = TopDown` + `direction = Free` não toca no vector.

**Ponte** (`ph2d-physics-ecs`):
- `a_porta_velha_delega_e_nao_tem_corpo` (censo do `move_character`);
- `o_laco_respeita_o_tecto_de_deslizes`;
- `uma_quina_interior_para`.

**Costura** (a shell): o gesto real no painel muda o andamento no mesmo quadro.

⚠️ **A fixtura do corpus é a saída do oráculo com CABEÇALHO**, versionada — e ⛔ ela nasce das
corridas **corrigidas**: as quatro fixturas do §1.5 seriam um corpus que aprova tudo.

---

## §6 — As MEDIÇÕES (§0.0 — nenhum número escrito sem a tabela ao lado)

### §6.0 — ⭐⭐⭐ O PRODUTO, depois de construído (a porta que manda)

Fracção do orçamento que o corpo anda num tique, medida pela **ponte inteira** contra a `rapier`:

| ângulo | fracção | razão contra a projecção |
|---|---|---|
| 5° · 10° | `0,0872` · `0,1736` | `1,00` — a **divergência declarada** (§6-bis) |
| 15° | `1,0000` | `3,86` |
| 20° | `1,0000` | `2,92` |
| 30° | `1,0000` | `2,00` |
| **45°** | **`1,0000`** | **`1,414`** |
| 90° | `1,0000` | `1,00` |

⭐ A coluna da direita é **exactamente `1/sin θ`** — que é a lei do orçamento escrita como número.

**O custo**, medido na mesma porta (mínimo de 5 corridas, `load 1,5`):

| n movers | ms/tique | por mover | % de um quadro |
|---|---|---|---|
| 1 | `0,082` | — | `0,5 %` |
| 100 | `3,54` | `35 µs` | `21 %` |
| 1 000 | `43,8` | `44 µs` | `263 %` |

⇒ **~45 µs por mover por tique**, e o `max_slides` é a alavanca. ⚠️ Cem movers são `21 %` de um
quadro: é folga larga para um jogo de vista de cima e é o número que quem puser uma horda tem de ler.

### §6-bis — ⛔ A DIVERGÊNCIA declarada, e a alternativa que foi CONSTRUÍDA e recusada

Abaixo do limiar o oráculo **pára seco** (`0,09 %` do orçamento) e nós andamos a **projecção**
(`8,72 %` a 5°). A cura óbvia — desligar o deslize da própria biblioteca (`slide: false`) — foi
construída e medida: o controlador **deixa de reportar contacto de forma fiável** quando o corpo já
toca, e a mesma varredura devolveu **zeros erráticos a 25°, 30°, 40° e 65°**. Isso é um defeito pior
do que a divergência que ele curava.

⭐ E o que fica defende-se por si: o knob existe para que um toque quase frontal **não** atire o
corpo à velocidade cheia, e a projecção faz isso de forma **contínua** — onde o alvo tem um penhasco
de **`235×`** entre 15° e 16°.

### §6.1 — O deslize: o oráculo contra nós, por ÂNGULO (um tique, corpo já encostado)

Razão `|d| / projecção tangencial` — `1,00` é rastejar, `1/sin θ` é o orçamento inteiro:

| ângulo de incidência | Godot `FLOATING` | nós (`move_character`) |
|---|---|---|
| 5° | **pára** (`0,006` de `4,0`) | `1,000` |
| 15° | **pára** (`0,017`) | `1,000` |
| 16° | **`≈ 1/sin` (3,999)** | `1,000` |
| 20° | `2,923` | `1,000` |
| 30° | `2,000` | `1,000` |
| 45° | `1,414` | `1,000` |
| 90° | `1,000` | `1,000` |

### §6.2 — ⭐⭐⭐ O dropdown, com a MESMA geometria e as MESMAS entradas

| `motion_mode` | anda por tique | `velocity` depois | `is_on_floor` |
|---|---|---|---|
| `FLOATING` | **3,976 px** | intacta `(169,7 · 169,7)` | `false` |
| `GROUNDED` | **2,828 px** | `(169,7 · 0,0)` — o motor **zerou** | `true` |

`1,406×`, que é o `√2` de 45° menos a margem. *É este número que o artista sente ao raspar na parede.*

### §6.3 — O custo (as duas portas, warm)

| n corpos | Godot `move_and_slide` | o nosso `move_character` |
|---|---|---|
| 100 | `7,23 µs` | `8,14 µs` |
| 1 000 | `9,58 µs` | `6,92 µs` |
| 5 000 | `10,35 µs` | `7,10 µs` |

⇒ o laço de orçamento custa, no pior caso (`max_slides = 4`), **`~28 µs` por jogador por tique** —
`0,17 %` de um quadro. ⚠️ **A medir de novo na porta do PRODUTO** (a wave da fábrica ensinou que uma
sonda sobre um sucedâneo mede outro programa): o número que manda é o da ponte, não o da porta.

### §6.4 — O componente, contado

| | campos |
|---|---|
| `PlatformPlayer` (o que existe) | **55** |
| … que significam algo sem gravidade | **4** |
| `TopDownPlayer` (proposto) | **11** |

---

## §7 — As WAVES

| # | o que fecha | smoke |
|---|---|---|
| **W1** | a crate de lei pura `ph2d-topdown` + o corpus do oráculo como gates + `move_character_from` | — (headless) |
| **W2** | o componente registado + a ponte + a cena do dono | `PH2D_TOPDOWN_SMOKE=1` |
| **W3** | a secção do Inspector (as 4 condições da §4) + as duas acções do Input Map | `=1` outra vez, agora pelo painel |
| **W4** | provas de mutação, portão de fecho, handoff, uma linha no §5 | — |

⭐ **Todas fecharam.** 24 provas de mutação (10 na W1, 14 nas W2/W3), portão de fecho a
**14 746 testes verdes** e clippy a zero com `-D warnings` na workspace inteira.

---

## §8 — O que é DECISÃO DO DONO (com o número ao lado, não em aberto)

1. **WASD de fábrica?** O `W` desta shell abre o painel de mundo. Hoje o plano ata `move_up`/
   `move_down` às **setas** e deixa o WASD a dois cliques no Input Map. Alternativa: mudar o atalho
   do painel — que é mexer num gesto que já existe.
2. **O penhasco dos 15°**, que é descontínuo por desenho (`235×` entre 15° e 16°). É o que o alvo
   faz; uma rampa seria produto novo sem lado aprovado.
3. **A isometria nasce em `Top-Down`** (identidade ao bit). Ligá-la por omissão mudaria o
   comportamento de toda cena existente que venha a pôr o componente.
