# TOP-20 #18 — `ParticleEmitter`: o plano

> Linha `line/components`, 2026-09-16. Item seguinte ao #16 por **ordem do dono**: o #17 (tilemap)
> foi pulado — ele é do projeto separado `docs/Tilling/` (porte Rust atrás da Fase R, que o dono
> suspendeu). Levantamento: [`00_levantamento_componentes.md`](00_levantamento_componentes.md) §7 ·
> pesquisa: [`pesquisa/sintese_visual_camera.md`](pesquisa/sintese_visual_camera.md) §D.1.

---

## §0 — O que o artista consegue FAZER quando isto fechar

Escolher um objecto → *Add Component* → **Particles** → e ele passa a soltar partículas, **sempre
visíveis** (não só com a ferramenta MOTION na mão), a correr com o relógio do jogo e a renascer ao
rebobinar. No Inspector, por módulos: quantas e por quanto tempo · uma rajada só (e a explosividade)
· a forma de onde nascem · a velocidade, a direcção e o cone · a gravidade e o amortecimento · o
tamanho e a cor **ao longo da vida** · se as partículas **andam com o objecto** ou **ficam onde
nasceram** · a semente. Um sinal **liga**, **desliga** ou **recomeça** o emissor; uma rajada que
acaba **grita** um sinal para a tabela de acções do #5.

---

## §1 — ⭐⭐⭐ A pergunta de antes: o que JÁ existe? (§5.0)

Medido por leitura (agente de pesquisa + leitura directa, com `file:line` no histórico da sessão):

| afirmação do levantamento | o que há |
|---|---|
| *«a simulação existe e é a melhor da classe»* | ✅ `motion.emitter` (**sem estado**: o conjunto vivo é função pura do instante), `motion.integrate`, `force.*`, CPU **e** GPU (4,19 M @ 3,6 ms) |
| *«ligada a objetos via `motion_object_bake`»* | ⛔ **só no sentido objecto → grafo** (um objecto vira MODELO de cópias). **Nada** liga um grafo a um objecto da cena, e nada devolve partículas a um objecto |
| (implícito) *«o grafo desenha»* | ⚠️ **UM grafo por app** (`MotionState.doc`), cozido e desenhado **só com a ferramenta MOTION activa** (`motion_bridge.rs:326`, `present.rs:214`) |
| *«abrir como grafo = poder total»* | ⚠️ não há *«abrir o grafo X»* — há **um** grafo |
| (implícito) *«seguir o objecto»* | ⚠️ o canal `$at:<nome>` publica o `Transform` **local** e só o `motion.look_at` o lê; nenhum canal responde *onde estava o objecto no instante t* |

**Três factos que decidem o desenho:**

1. **O `MotionCookPump` e o `GpuCook` são bibliotecas por instância** (`ph2d-eval-motion/src/lib.rs:75`,
   `ph2d-gpu-cook/src/lib.rs:107`) — um componente pode ter o seu.
2. **O renderer recebe UMA fatia de CPU e UM buffer de GPU por passo** (`render_with_streams`,
   `renderer_draw.rs:72`), e as instâncias extra entram na ordem da cena pelo `z_order`
   (`sprite_collect.rs:72`) ⇒ partículas de CPU **herdam a profundidade do emissor** de graça.
3. **O `value.table` amostra, no instante do cálculo, uma tabela que o APP publica**
   (`ph2d-node-value-table/src/lib.rs:144`) — e a **história** do emissor (`Leave`/`Inherit`,
   ADR-0163) recoze a sub-árvore que conduz `x`/`y` **no instante de nascimento**. ⇒ publicar a
   **trajectória** do objecto como tabela dá o modo *«ficam onde nasceram»* **exacto**, sem nó novo.

**E um buraco real:** o emissor não sabe **parar** de emitir. Ele é contínuo desde `t = 0` ou por
rajadas periódicas — não há *«ligado de 0 a 2, desligado, ligado de novo»*. Sem isso não há rajada
única com explosividade, nem ligar/desligar com as partículas vivas a acabar a vida.

---

## §2 — O ORÁCULO (§0.9): o relógio do Godot 4.7.2 (MIT), corrido sem interface

[`godot_particles_probe.gd`](ferramentas/godot_particles_probe.gd) —
`godot --headless --fixed-fps 60 --script …`. ⚠️ A API do `CPUParticles2D` **não expõe posições**
(`--doctool`: 11 métodos, nenhum getter de partícula) ⇒ o oráculo sem interface é o **RELÓGIO**
(`emitting` e o sinal `finished`). Controlo **C0** verde (um nó que nunca emite não grita).

| caso (vida 1 s = 60 quadros, 8 partículas) | `emitting` → falso | `finished` |
|---|---|---|
| rajada, explosividade 0 | 60 | **112** |
| rajada, explosividade 0,5 | 60 | **86** |
| rajada, explosividade 1 | 60 | **60** |
| vida 0,5 s, explosividade 0 / 0,5 / 1 | 31 | 56 / 43 / 30 |
| pré-aquecimento 0,5 s | 30 | 82 |
| pré-aquecimento 1,5 s | 2 | 22 |
| velocidade ×2 | 31 | 56 |
| vida aleatória 0,5 | 60 | 86 |
| 1 partícula | 60 | 60 |
| contínuo, desligado no quadro 30 | 31 | **82** |
| contínuo, desligado no 30 e religado no 45 | — (fica ligado) | **nunca** |
| rajada com `restart()` no quadro 30 | 89 | **141** |

**As leis (portadas):**

- **L1** — a emissão de uma rajada dura **uma vida**; a partícula `i` (de `n`) nasce em
  `i/n · vida · (1 − explosividade)`.
- **L2** — `finished` dispara quando **morre a última partícula**:
  `vida · (1 + (n−1)/n · (1 − explosividade))` (112 = 60·1,875 ✓; 86 ✓; 60 ✓).
- **L3** — o **pré-aquecimento** adianta o relógio inteiro (e a rajada pode já ter acabado ao nascer).
- **L4** — a **velocidade** escala o relógio.
- **L5** — desligar um contínuo **não mata** as vivas; `finished` vem quando a última morre.
- **L6** — religar antes disso **cancela** o `finished` e as vivas continuam.
- **L7** — `restart()` põe o relógio a **zero** e as vivas **somem**.
- **C0** — quem nunca emitiu nunca grita.

**Divergências a declarar (e gatear):** nenhuma no relógio. A *forma* das partículas (cone, forma,
velocidade) é a do `motion.emitter`, já conferido pela conferência dos nós do Motion — ⛔ não se
porta a do Godot (um segundo motor seria o que o levantamento proíbe).

---

## §3 — O desenho, com a porta ÚNICA de cada pergunta

### §3.1 — *Quando nasce uma partícula?* ⇒ a **AGENDA** do emissor (`emit_mode = Scheduled`)

O emissor ganha um terceiro modo de nascer, ao lado de `Continuous` e `Burst`, **sem estado**:

- a agenda são **segmentos** ligados `[a, b)` em segundos do emissor (o que um sinal liga e
  desliga) e um **pulso** opcional (`pulse ON/PERIOD`: ligado `ON` de cada `PERIOD` segundos,
  contado desde o início de **cada** segmento) — num **parâmetro de texto** (`schedule`), o molde da
  onda `Custom`;
- o **relógio de emissão** `τ(t)` = tempo ligado acumulado até `t`;
- a partícula `k` nasce no **primeiro** instante em que `τ = k/rate` ⇒ as ids vivas continuam um
  **intervalo contíguo** (o `SourceWindow` não muda de forma) e a idade é `t − nascimento(k)`;
- ⚠️ **agenda vazia = `Continuous` ao bit** (o braço que sempre shipou não é tocado);
- ⛔ **CPU apenas**, com a recusa declarada onde o planeador a vê (`applicable`: `emit_mode < 1,5`) —
  o molde do `Leave`/`Inherit`. A fronteira fica **nomeada** (o kernel precisaria da agenda numa LUT).

Com ela: **rajada única** = `[0, (1−e)·vida)` a `rate = n/((1−e)·vida)` (L1, e `e = 1` é o `Burst`
que já existe) · **contínuo com explosividade** = `pulse (1−e)·vida / vida` dentro dos segmentos ·
**ligar/desligar** = acrescentar uma fronteira · **recomeçar** = relógio novo (L7).

### §3.2 — *Onde mora o que o artista escolhe?* ⇒ `ph2d_ecs::ParticleEmitter` (gravado)

Dados simples, registado no `ph2d-ecs` (o molde do `Timer`/`Factory`/`StateMachine`), por módulos:
**Emission** (`emitting`, `rate`, `life`, `life_random`, `one_shot`, `explosiveness`, `prewarm`,
`max`, `seed`) · **Shape** (`kind`, `size`) · **Velocity** (`speed`, `speed_random`, `angle`,
`spread`) · **Forces** (`gravity: [f32; 2]`, `damping`) · **Look** (`size`, `size_random`,
`size_end`, `color`, `color_end`) · **Space** (`Local` = anda com o objecto · `World` = fica onde
nasceu) · **Signals** (`start_on`, `stop_on`, `restart_on`, `finished_signal`).

⚠️ **Os tectos são os do NÓ** (`max` até `MAX_ALIVE`, as faixas do `params_ui`) — ⛔ nenhum tecto
novo escolhido aqui (§0.0).

### §3.3 — *Que grafo o objecto corre?* ⇒ uma função pura, `ph2d_particles::compile`

`compile(&ParticleEmitter, agenda) -> Compiled { graph, sink }` com **nós reais** do Motion:
`motion.emitter` → (`force.wind` como gravidade + `force.drag` → `motion.integrate`, só se houver
força) → cor e tamanho pela **fracção de vida** → `motion.output`. Recompilado **só quando a
assinatura das definições muda** (o `MotionCookPump::mark_dirty` a seguir).

### §3.4 — *Onde as partículas estão no mundo?* ⇒ o **espaço**

- **`Local`** — o grafo corre na origem; as instâncias são **transformadas pela pose de MUNDO actual**
  do objecto na saída. Exacto, e custa uma multiplicação por partícula.
- **`World`** — o grafo corre em coordenadas de mundo, com a origem do emissor conduzida por
  `value.table` sobre a **trajectória** que a ponte publica (tique → pose de mundo) e o emissor em
  `Leave`: a história recoze a origem **no nascimento** de cada partícula. ⚠️ A trajectória nasce e
  morre com a corrida (rebobinar = renascer).

### §3.5 — *Quando corre?* ⇒ com o **relógio a andar**, e rebobinar é **renascer** (a lei dos #11–#16)

Um tique fixo por passo; o relógio **local** do emissor começa quando ele liga; o **pré-aquecimento**
marcha `prewarm/dt` tiques no nascimento (L3). **Nada é gravado** do estado vivo (o
`ParticleRuntime` não se regista — a cerca é o **tipo**), e `ph2d_ecs::rewind_runtime` ganha um
membro.

### §3.6 — *Quem o liga e quem ouve o fim?* ⇒ **sinais**, pelo barramento do #5

A ponte **lê** o outbox (cursor próprio) para `start_on`/`stop_on`/`restart_on` e **publica**
`finished_signal` com `SignalOrigin::Particles` quando a agenda acabou **e** a última partícula morreu
(L2, L5, L6, C0).

### §3.7 — *Onde se desenha?* ⇒ na fatia de CPU do passo de sprites, **sempre**

A ponte junta as instâncias de todos os emissores numa fatia; a shell concatena-a ao `extra` que já
leva o onion e o Motion — **independente** da ferramenta MOTION. `z_order` = o do emissor.

### §3.8 — O que NÃO entra nesta wave, e porquê

- ⛔ **O caminho de GPU dos componentes** — o renderer recebe **um** buffer de GPU por passo, e a
  `ph2d-gpu-cook` está a ser reescrita pela `line/motion-value` (22 ficheiros no diff dela) e o
  `renderer_draw.rs` pela `line/Vector`. Nomeado, **não** um tecto: o CPU computa a mesma resposta.
- ⏳ **«Abrir como grafo»** — o editor de nós edita **um** documento. A escada é uma wave própria
  (editar o grafo do componente no lugar do do projecto, com *Done*/*Cancel*, o molde do *Edit
  Prefab*), decidida depois de a fachada existir.
- ⛔ Sub-emissores, rastros por partícula, textura por partícula — suplentes, com gatilho.

---

## §4 — Contrato congelado e contadores (DELTA)

- **Contrato congelado: NÃO.** `NodeManifest` fica com a mesma forma (um param a mais na lista do
  emissor é dado, não contrato — o `emitter_motion` entrou assim).
- **`PROJECT_SCHEMA`: +1** · **registo do `ph2d-ecs`: +1** e **os dois espelhos +1** ·
  **`LIVE_SECTIONS`: +1** · **`SignalOrigin`: +1** · **`EditorAction`: +1** · catálogo: +1 entrada.
- ⚠️ **Tecto da shell: 492 de folga.** O corpo mora em `ph2d-particles` (a lei e o compilador) e em
  `ph2d-app-components` (a ponte, o Inspector, a cena); a shell só compõe.

---

## §5 — As quatro condições de UI

1. **existe** — `ParticleEmitter` no catálogo, `Authored`, família **Rendering**.
2. **é pintado e registado** — a secção **Particles**, por módulos.
3. **o clique chega ao barramento** — `ParticleFieldEdit` pelo `EditorAction`.
4. **a sequência leva a algum lugar** — a cena de smoke (§6).

---

## §6 — As waves

| W | o quê | pronto quando |
|---|---|---|
| **W0** | a **agenda** no `motion.emitter` (lei pura + braço `Scheduled` + recusa de GPU) | a agenda vazia é o contínuo ao bit; ids contíguas; L1 reproduzida pelo nó |
| **W1** | o componente + o **compilador** + o **relógio** (`ph2d-particles`) | L1–L7 e C0 gateados sobre o motor real, headless |
| **W2** | a **ponte**: corre com o Play, desenha sempre, `Local`/`World`, sinais, renascer | um objecto a mexer-se deixa rasto em `World` e leva o penacho em `Local`; rebobinar renasce |
| **W3** | a **secção do Inspector** | as 4 condições |
| **W4** | a **cena de smoke** (fotografada) + mutação + handoff | o dono corre e vê |

---

## §7 — ⚠️ Premissas DESTE plano que a implementação derrubar

*(escrito durante a construção)*

1. ⛔ **«Um período global basta para o contínuo com explosividade»** — a 1.ª redacção da agenda
   (`a-b every P`, os intervalos dentro de um período global) não sabia **cortar** um padrão
   periódico num segmento finito: desligar um emissor que pulsa só se exprimia expandindo o padrão
   em N intervalos (um por ciclo, e o custo de `birth` é linear neles). ⇒ **segmentos + pulso por
   segmento** (`0-10 20- pulse 0.5/1`), e religar recomeça o ciclo. Achado ao desenhar o relógio do
   componente, antes de alguém depender da forma velha.

2. ⛔ **«O `clamp` do campo inteiro é a cerca»** — o dreno convertia o `f32` do painel para `u32`
   com um `clamp(0, u32::MAX)` ao lado do `round`, e **a mutação que o apagou não matou gate
   nenhum**: em Rust um `as` de vírgula flutuante para inteiro **satura** desde a 1.45 (`-3.0 as
   u32` é `0`). *Uma cerca que repete o que a linguagem já garante lê-se como a cerca que falta* —
   o que não é de graça é o `round`, e é isso que o gate mede agora.

3. ⛔ **«A régua do enquadramento é o alcance da partícula»** — a 1.ª versão do gate da cena media
   `velocidade × vida` em todas as direcções e acusava um jacto ESTREITO de sair pelo lado quando
   ele vai todo para cima. ⇒ a fracção lateral é o **seno da abertura**, e ⛔ acima dos `90°` ela
   **desce** (a `180°` dá zero, que leria uma esfera como um fio): a partir dali é `1`.

4. ⛔⛔ **«Caber no ecrã é a legibilidade»** — **falso, e foi uma MUTAÇÃO SOBREVIVENTE que o disse.**
   Devolver ao anel a rapidez das outras deixava o gate do enquadramento VERDE (ele cabia na banda)
   e punha os penachos das quatro colunas uns por cima dos outros. *Caber no ecrã e cada coluna
   ficar na coluna dela são DUAS grandezas, e só uma estava medida* ⇒ `cada_coluna_fica_na_coluna_dela`.

5. ⛔ **«O smoke abre com o Inspector à frente»** — a foto mostrou o painel do **esqueleto** por
   cima numa corrida e o do **vector** noutra: a arrumação vive **fora do repositório**
   (`~/.ph2d/layout.txt`) e estava a ser reescrita por outra árvore a correr em paralelo. ⚠️ E a
   1.ª cura não chegou: o `reconcile_z` acrescenta, no **início de cada quadro**, os painéis que
   ainda não estão na ordem z — logo um `bump` feito no quadro em que a cena monta fica **por
   baixo** dos que chegam a seguir. ⇒ a subida repete-se por três quadros e **pára** (passado isso a
   aba é do dono).

---

## §8 — O que FECHOU, com os números

| wave | o que shipa | a prova |
|---|---|---|
| **W0/W0b** | a **agenda** (`emit_mode = Scheduled`) no `motion.emitter` | 13 gates, entre eles a enumeração por força bruta (5 agendas × 3 taxas × 260 instantes) e a identidade ao bit com o contínuo |
| **W1** | `ph2d-particles` (o compilador + o relógio) | a tabela do oráculo reproduzida com a janela `[0, 1 quadro]` |
| **W2** | a ponte: corre com o Play, desenha sempre, `World`/`Local`, sinais, renascer | 6 gates — e **dois defeitos de produto** que só eles viram (um emissor PARADO mostrava uma partícula; um `restart_on` num emissor autorado `emitting = false` nunca emitia) |
| **W3** | a **secção do Inspector** (19 números, 4 sinais, 2 caixas, 2 segmentados, 2 cores) | 6 gates de costura com **cliques reais** + 15 provas de mutação |
| **W4** | as **duas cenas** (`PH2D_PARTICLES_SMOKE=1\|2`), fotografadas | 7 gates de cena + 10 provas de mutação; **25 provas no total, todas a sangrar** |

⭐⭐ **O que a FOTO apanhou e nenhum gate via** (a lição do #16, outra vez): a fila a `±7,5 m` com
duas colunas fora do ecrã · as fontes a `−5 m`, cortadas pela borda de baixo · o jacto a `0,52 m` de
alto, colado à fonte, com a gravidade do mundo · o anel borrado num disco · a `=2` a ficar um chão
**vazio** passados três segundos · e o painel da direita a ser o do esqueleto.
