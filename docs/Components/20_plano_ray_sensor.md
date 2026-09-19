# Suplente #21 — `RaySensor`: o objecto passa a OLHAR

> **O item, na ordem do dono** ([levantamento §7](00_levantamento_componentes.md), suplentes 21–25):
> *«raio persistente com gizmo: chão, parede, mira — copiar a REFLEXÃO do Construct»*.
>
> ⚠️ **Este doc começa pela MEDIÇÃO**, porque o item ANTERIOR desta mesma lista — o **#3
> `SensorZone`** — foi medido em 17/09 e estava **fechado por composição**: a costura que ele pedia
> já tinha sido construída pela wave das Tags, e reconstruí-la teria sido trabalho já pago
> (`CLAUDE.md` §5.0). *E lá a leitura por `grep` não bastou: o veredito saiu de CORRER a cena.*

## §1 — O que a composição JÁ dá (medido em 2026-09-19)

A sonda é [`mede_o_que_a_composicao_ja_da_ao_raio`](../../crates/ph2d-physics-ecs/tests/it/mede_o_que_a_composicao_ja_da_ao_raio.rs),
e o lado que ela mede **não é um espantalho**: é a melhor composição que a casa tem — um colisor
**`is_sensor`** fino deitado ao longo da linha, que é exactamente como o #3 fechou (sensor +
`SignalOnHit` + `SignalTagFilter`, com a cena `PH2D_TAGS_SMOKE=2` a prová-lo).

| a pergunta | a COMPOSIÇÃO (barra sensor) | o MOTOR (`cast_ray`) |
|---|---|---|
| **ORDEM** — qual das duas paredes está mais perto? | `triggered_sensors()` devolve `["Barra"]` — **um** elemento, e as duas paredes estão lá dentro | acerta **a de x = 2**, a mais perto |
| **MÉTRICA** — a que distância, em que ponto, com que normal? | **`0` contactos de pé** — *um sensor atravessa*, logo o canal que traz `point`/`normal`/`impulse` está **vazio** por esta rota | `distância = 1,7500` · `ponto = (1,7500 ; 0,0000)` · `normal = (−1,0000 ; 0,0000)` |
| **DIRECÇÃO** — distingue a frente de trás? | **não**: uma FORMA é simétrica por construção, e a barra centrada na origem apanha a parede de `x = −3` | a parede de trás está **dentro do alcance** e **não** é devolvida |
| **quem lhe chega hoje?** | — | **5 chamadas em 2 ficheiros**, todas dentro da ponte do platformer |

⇒ **O buraco tem três nomes — ordem, métrica e direcção — e nenhum deles se compõe do que existe.**
O motor está pago e é rico; o que falta é **um componente autorável que lhe chegue**.

### §1.1 — ⭐ E o censo confirmou o doc da porta, com número

O doc de [`cast_ray`](../../crates/ph2d-physics/src/world/cast.rs) afirma por escrito que ele tem
*«exactamente cinco consumidores no repo inteiro — o sensor de chão, os dois de teto, o de headroom
e o de parede»*. A sonda contou **`1 + 4 = 5`**, nos dois ficheiros da ponte do platformer. *Uma
afirmação de cabeçalho que a medição confirma passa a ser uma propriedade; até lá era memória.*

### §1.2 — ⚠️⚠️ A NOTA que este componente obriga a reconferir (§0.0)

A mesma porta põe `QueryFilterFlags::EXCLUDE_SENSORS` **dentro dela**, e justifica-o assim:

> *«o `cast_ray` tem exactamente cinco consumidores no repo inteiro … e os cinco querem matéria. Um
> parâmetro seria uma escolha oferecida a ninguém, e o dia em que alguém quiser detectar um sensor a
> resposta já existe e é outra: o canal de TRIGGERS (W7).»*

O `RaySensor` é o **sexto** consumidor, e o primeiro autorável — logo o §0.0 obriga a reconferir a
nota em vez de a herdar. **Veredito: ela continua de pé, e agora por uma razão mais forte.** As três
perguntas que este componente serve — *há chão por baixo? · há parede à frente? · a arma aponta para
quê?* — são todas sobre **matéria**; e um volume de gatilho que bloqueasse a linha de visão seria um
defeito, não uma opção (é a mesma frase que o `buoyancy` escreve do outro lado: *um sensor é um
marcador, não matéria*). ⇒ **nenhum parâmetro novo na porta**, e a decisão fica escrita aqui.

## §2 — O desenho, com a PORTA única de cada pergunta

| a pergunta | a porta ÚNICA |
|---|---|
| *onde nasce e para onde aponta?* | `RaySensor { origin, dir, reach, layer }`, os dois vectores em coordenadas **LOCAIS** — é isso que faz o raio **rodar com o objecto** sem uma segunda lei |
| *o que ele está a ver AGORA?* | o **BRIDGE** guarda-o, nunca um componente — ⭐ o precedente é o canal de triggers (`bridge.triggers`) e o `ProbeState` do platformer, os dois no mesmo ficheiro-família |
| *quem ele pode ver?* | `layer` (a matriz do mundo) **+** o `SignalTagFilter` que já existe — ⛔ zero filtros novos |
| *como ele fala?* | `signal_on_enter` / `signal_on_exit`, **vazio = calado** — a regra do `SignalOnHit`, palavra por palavra |
| *quem ouve?* | a tabela do **#5**, sem uma linha nova: a wave de 19/09 deu-lhe o `from` e o `Who Hit` |
| *e o que o artista VÊ?* | uma linha no canvas com o ponto de impacto — o precedente é o overlay das sondas do platformer, que já desenha exactamente isto |

### §2.1 — ⛔ O que NÃO se constrói, e porquê

- **Um `RaySensorRuntime` registado.** O que nasce numa corrida não é documento (a lei do #11 e do
  #20), e aqui nem sequer é preciso um componente: a memória de um tique cabe no mapa do bridge, que
  é onde o canal de triggers já guarda a dele.
- **Um verbo novo na tabela do #5.** O raio **publica um sinal**; quem decide o que fazer é a tabela
  que já existe. Um verbo *«lança um raio»* seria uma segunda maneira de dizer a mesma coisa.
- **Um segundo filtro.** O `SignalTagFilter` já responde *«de quem?»*, e a wave de 19/09 pôs a cerca
  no reactor.
- **Um parâmetro de sensores na porta do motor** — §1.2.

## §3 — Onde isto encosta (contrato §6 e schema)

- **Contrato congelado (§6):** ⛔ nenhum. `Tool`, `NodeOp` e a superfície do vector ficam intactos —
  isto é um componente de ECS e uma fase do quadro.
- **`PROJECT_SCHEMA`:** **+1** *quando o registo entrar* — ⚠️ **DOIS tipos e UM degrau**: o número
  mede o que o FICHEIRO passa a conter, não quantos tipos nasceram (a lei do degrau `130`).
- **Registos:** `ph2d-physics-ecs` **+2**; ⛔ os **dois espelhos não se mexem** (eles contam
  `ecs + render` e `ecs + script`) — a lição que o #13 pagou e o plano dele escreveu errado.

### §3.1 — ⛔⛔⛔ O REGISTO ESPERA PELA UI, e a decisão tem endereço

O gate [`every_registered_physics_component_has_a_ui_writer`](../../shells/desktop/tests/it/every_physics_component_is_authorable.rs)
**reprovou no instante** em que os dois componentes entraram no `register_physics_components`, e a
mensagem dele oferece duas saídas: *«ou dê a ele uma row na §11 (e um arm em `apply_physics_edit`),
ou **não o registre ainda**»*.

⇒ **A wave da LEI fechou e a da UI não**, logo o registo, o degrau de schema e as entradas do
catálogo **ficam para a W4**. ⚠️ *Registar agora seria pior do que esperar, e a razão não é gosto:*
com o registo o componente entra no `.ph2dproj` e no `Ctrl+Z`, e o artista que o anexasse pelo `+`
ficaria com quatro números que **nenhuma row deixa mexer**, gravados para sempre no ficheiro dele —
o **órfão** que a DIRETIVA §2 proíbe, e que funciona em toda cena de smoke porque as cenas
constroem com código.

⭐ **O que a W4 herda pronto:** o vocabulário do painel
([`ph2d_editor_core::ray_edits`](../../crates/ph2d-editor-core/src/ray_edits.rs)) já existe, com o
instantâneo, as oito edições e a **porta da queixa** — as quatro razões pelas quais um raio pode não
fazer o que o artista espera, *da mais específica para a mais geral*, com dois gates e sem um device
à vista.

## §4 — As waves

| # | o que fecha | a prova |
|---|---|---|
| ✅ **W1** | o componente + a lei (uma fase que lança um raio por sensor e diz o que ele viu) | gates de unidade sobre ordem · métrica · direcção |
| ✅ **W2** | os sinais de entrar e sair, com a cerca de tag | o diff de um tique, e o CONTROLO (sem tag, cala-se) |
| ✅ **W3** | rebobinar RENASCE — o mapa do bridge entra no `rebuild_from_rest` | ⚠️ a família que o smoke do #14 expôs por report |
| ✅ **W4** | a secção do Inspector **+ o registo + o degrau de schema + o catálogo** (§3.1) | gate de costura (clique REAL) |
| ✅ **W5** | o desenho no canvas | o gate que mede a LINHA, não a contagem |
| ✅ **W6** | a cena `PH2D_RAY_SMOKE=1` + o roteiro | ⚠️ e o censo de teclas de 19/09 já o vigia |

### §4.2 — ⛔⛔⛔ O que a FOTO da W6 apanhou, com a suíte inteira VERDE

A cena foi conduzida e fotografada ([`fotografa_cena.sh`](../../docs/Components/ferramentas/fotografa_cena.sh))
antes de ir ao dono, e as **quatro** coisas que ela achou eram invisíveis a todo gate desta linha:

| o que a foto mostrou | a causa | de quem era |
|---|---|---|
| o raio desenhado `~210 px` abaixo do olho e `1,8×` mais comprido | o overlay da física era projectado contra a **janela** enquanto a cena desenha numa **banda** (ferramenta Motion activa, que o `layout.txt` do dono reactiva no quadro 1) | da SHELL — a **quinta** vez que a lei do `scene_mapping` é paga |
| o painel da direita era o do **Sculpt 3D** | `panel_visibility` diz *«existe»* e não *«está à frente»*; o `reconcile_z` acrescenta os painéis no início do quadro | da cena (a cura das PARTÍCULAS, `raise` em 3 quadros) |
| `Direction 0 / −1` e `Reach 1 m` sobre um olho a `(1, 0)` com alcance `6` | **faltava a SEMENTE** do painel (`sync_ray`) — a secção mostrava os valores de fábrica do `populate_ray` | da **W4**, e nenhum gate dela podia vê-la |
| a luz do CONTROLO fora do ecrã, e depois invisível por estar apagada | o enquadramento, e `Show`/`Hide` a tornar *«não acendeu»* igual a *«não existe»* | da cena — ⇒ a [`JANELA_UTIL`] medida, e um **SUPORTE** escuro por baixo de cada luz |

⚠️⚠️ **Os três primeiros tinham a suíte inteira verde por cima**, e o terceiro é a mesma família que
a secção do ÁUDIO e a da CÂMERA pagaram em 10/09 e a da VIGIA DO CONTADOR em 16/09 — *quatro números
plausíveis são a pior forma deste defeito: eles não parecem partidos, e quem escreva por cima de um
grava o default nos outros cinco.*

⛔ **E o censo que devia ter apanhado o primeiro mede só METADE da lei:** o
`todo_aponte_passa_pela_janela_da_cena` resolve o último argumento de `screen_to_world` — a metade
**cursor → mundo**. A metade **mundo → ecrã** entra por PINTORES que recebem a janela, e a shell tem
**`11`** chamadas de `world_to_screen`/`_affine`, **`4`** delas em pintores de chrome da cena que a
recebem por parâmetro (`anchor_overlay` · `empty_object_overlay` · `padding_bridge` ·
`upscale_bridge`). Esta wave **cura e gateia a sua** e deixa as outras quatro **NOMEADAS e por
auditar** — a prova delas está nos chamadores, e cada uma pede a foto da cena que a exercita.

### §4.1 — ⭐⭐⭐ O que a W5 MEDIU antes de escrever um pintor (§5.0)

A pergunta era *«como se desenha um raio?»*, e a resposta **já existia**: o
[`ProbeShape::Ray`](../../crates/ph2d-physics-ecs/src/bridge/player_view.rs) tem os cinco números
certos (origem · rumo · alcance · acerto · pele) e o pintor dos sensores do personagem
(`ph2d_app_physics::overlay::probes::probe_marks`) já desenha **a linha, a ponta do alcance e o tique
do acerto** — *exactamente* o que a §2 desta página pedia. **Faltava um [`ProbeKind`].**

⇒ a wave é **publicar, não pintar**, e o que ela de facto acrescentou foi:

| o quê | porquê |
|---|---|
| `ProbeKind::Sensor` | ⛔ um segundo pintor seria a segunda resposta a *como se desenha um raio* |
| `ray_marks`, publicado **dentro do laço que lança** | derivá-lo do lado do desenho seria a segunda resposta a *onde este raio nasce* |
| a direcção **normalizada** na marca | ⚠️ **correcção medida:** `dir = (1,1)` com `reach = 3` casta 3 m e desenhava **4,243** |
| `preview_ray_marks` no `hold` | o toggle *Physics* nasce desmarcado ⇒ sem isto o componente é **invisível** no caminho de omissão |
| `discard_ray_history` no `hold` | ⛔ **correcção da W2**: as arestas eram limpas no topo do `dispatch`, e o `hold` é chamado **em vez** dele ⇒ a entrada ficava de pé e o sinal soava **em todo quadro** |
| `ray_marks.clear()` no `rebuild_from_rest` | o laço de replay não casta ⇒ o canvas desenharia os raios onde os corpos **estavam** |

⚠️ **As duas listas ficam SEPARADAS na ponte e compõem-se na SHELL** — quem enche uma é o
`player_marks` e a outra o `ray_sensors`, logo fundi-las poria dois escritores numa lista só.

⚠️ **E o `bridge.rs` foi CORTADO por responsabilidade** (`692 → 590`): o `new`/`rebuild` mudaram-se
para o irmão `bridge/birth.rs`. Ele estava a **8 linhas** do tecto de 700, e um tecto por-ficheiro é
a única grandeza deste repo que **soma entre linhas sem ninguém a contar** — deixá-lo assim era
entregar a catraca vermelha ao integrador.

## §5 — ⛔ Recusas e riscos NOMEADOS antes da primeira linha

1. ⚠️ **O raio acha-se a si mesmo.** O `cast_ray` só exclui um corpo se lhe dermos o handle — e o
   gate `the_caster_can_exclude_itself` do motor mede exactamente o defeito que isso causa
   (*«um personagem acha-se no chão para sempre»*). ⇒ a fase **tem** de excluir o corpo dono.
2. ⚠️ **Um objecto sem corpo também pode querer um raio.** Ali não há handle a excluir, e a resposta
   certa é *não excluir nada* — mas isso tem de estar **medido**, não suposto.
3. ⚠️ **A ordem no quadro é load-bearing.** O raio lê o mundo DEPOIS do passo da física e publica
   ANTES do dreno de sinais, senão o sinal chega um quadro atrasado — a lei que o `ph2d-runtime`
   já escreve e gateia.
4. ⛔ **`dir` nulo não é um raio.** A porta do motor devolve `None` para direcção nula; o painel tem
   de o dizer, senão é um controlo que parece partido.
