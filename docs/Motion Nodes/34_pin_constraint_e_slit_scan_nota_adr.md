# 34 — Pin Constraint (massa inversa) + Slit Scan — nota-ADR

**Data:** 2026-07-12 · **Linha:** `line/motion-value` (Modo L) · **Fase:** cauda do M3 (simulação + tempo)
**Status:** implementado, testado, **pendente smoke do Enio**
**Contratos congelados encostados:** **nenhum** (`NodeManifest` 8 / `NodeOp` 2 / `OpResolver` 1 intactos)

---

## 1. O que entrou

| Crate nova | Nó | Categoria |
|---|---|---|
| `ph2d-node-motion-pin-constraint` | `motion.pin_constraint` | Focus (campo de peso) |
| `ph2d-node-motion-slit-scan` | `motion.slit_scan` | Fx (efeito de tempo) |

E — o que faz a fatia valer — os **consumidores** do peso: `motion.integrate`, `motion.spring` e
`motion.collide` passaram a ler a coluna `inv_mass`. Sem isso o nó novo seria uma **flag órfã**
(DIRETIVA §1: *o consumidor faz parte DESTE work item*).

## 2. A fatia que NÃO entrou, e por quê (`motion.distribute_poisson`)

A fila (handoff de continuação, A1) pedia `motion.distribute_poisson` (Bridson 2007). **Cancelado.**
`motion.scatter` **já é** o gerador blue-noise do módulo, e o cabeçalho dele *ratifica por escrito* a
escolha de **Mitchell best-candidate (1991) em vez de Bridson**: o nó tem param `count`, e
best-candidate dá **contagem exata** enquanto o dart-throwing do Bridson é *density-driven* (a
contagem é implícita no raio). Somado a `motion.collide` (raio duro, PBD), um nó Poisson seria um
**quase-duplicado** — pior, um que contradiz uma decisão documentada. Cerca de Chesterton: a decisão
fica. Se um dia o módulo quiser preenchimento por densidade (*"encha esta área com o que couber no
raio r"*), isso é um **param `mode` no `scatter`**, não uma crate nova.

## 3. `motion.pin_constraint` — a primitiva que faltava

### 3.1 Pesquisa (o padrão-ouro)

Pinagem, na indústria, **não é um booleano** — é **massa inversa**:

- **Position Based Dynamics** (Müller et al., 2007): cada partícula carrega `w = 1/m`; a correção de
  uma constraint é distribuída **proporcionalmente aos `w`** das partículas que ela toca. `w = 0` é
  massa infinita: nenhuma força, contato ou constraint a move.
- **Houdini Vellum**: o atributo `pintoanimation` — o ponto pinado é *cinemático*, dirigido pela
  animação, imune à sim (é exatamente a nossa semântica: `P = rest.P`).
- **Blender** (cloth *pin group*) e **Bullet** (`invMass`): o peso do vértice é **fracionário** — pin
  parcial = elemento pesado, não imóvel.

Um bool daria só o pin duro e jogaria fora o parcial. Então o nó escreve um **campo de peso**.

### 3.2 A superfície nova (aditiva, sem contrato)

**Coluna `inv_mass`** (Scalar, por elemento) — registrada na convenção canônica do stream
(plano §1.2, dona: `ph2d-eval-motion`):

| valor | significado |
|---|---|
| **ausente** | `1.0` — livre. **Todo grafo pré-pin cozinha idêntico** (o `·1.0` é exato: trajetória bit-a-bit). |
| `1` | livre |
| `0` | pinado (massa infinita) |
| `0 < w < 1` | pesado / lento (o pin parcial do Blender) |

**Seleção** = faixa de índice `[first, first + count)` **×** o campo `falloff` (multiplicativo, o
mecanismo de região que o módulo já tem) **×** `strength`. Pins **compõem** (multiplicam no `inv_mass`
que já estiver no stream), igual aos falloffs — dois nós de pin empilham em vez do segundo apagar o
primeiro. `count = 0` é a identidade.

### 3.3 Quem lê (e como)

| Solver | Regra | Em `w = 1` |
|---|---|---|
| `motion.integrate` | `v += a·dt·w` e `d += v·dt·w`; a `vel` de seed também escala por `w`. `w = 0` ⇒ `sim_d = 0` ⇒ **`P = rest.P`**: o elemento pinado *segue a animação upstream* (o `pintoanimation`), e nem a velocidade de bocal do emitter o carrega embora. | bit-idêntico |
| `motion.spring` | o blend de saída vira `falloff × inv_mass`: pinado ⇒ saída = alvo cru (rastreio rígido, sem lag/overshoot). | bit-idêntico |
| `motion.collide` | a penetração é dividida por `w_i/(w_i+w_j)` (a regra de projeção do PBD): livre×livre = metade cada (o midpoint de antes, bit-a-bit); contra um pinado, **o livre absorve a penetração inteira e o pin não sai do lugar** — é o que faz de um disco pinado um **obstáculo**. Dois pinados: sem correção a repartir (nada de divisão por zero). | bit-idêntico |

**Quem NÃO lê, e por quê:** `motion.verlet_rope`, `motion.soft_body` e `motion.boids` são
**geradores** (mintam os próprios pontos a partir de params e carregam estado; **nenhum stream de
instâncias entra**), então um pin upstream **não tem fio por onde chegar até eles**. Os pins
intrínsecos deles (head/tail da corda, top-row do corpo) ficam. Um dia que ganhem porta `in`, herdam
a coluna de graça.

### 3.4 Robustez

`inv_mass` negativo ou não-finito (documento editado à mão) lê como **pinado** (`0`), nunca invertendo
o empurrão do `collide`. `strength` NaN lê como **livre**. HR-5: só aritmética.

## 4. `motion.slit_scan` — cada elemento vê um AGORA diferente

### 4.1 Pesquisa

**Slit-scan** fotográfico (o Star Gate de Trumbull em *2001*; as aberturas do Hitchcock): lê-se o
sujeito por uma fenda em movimento e o eixo **espacial** do filme vira eixo de **tempo**. Os
descendentes em motion graphics são o **Time Displacement** do After Effects e o offset de tempo
por-elemento do Cavalry. O plano (§3, M3) pedia *"amostrar o campo em tempos defasados por instância
(`t − i·delay`)"*.

### 4.2 A decisão de design que importa: `lag` é o SPREAD, não o passo

Delay por-elemento **constante** (`i·delay`) faz o espalhamento **explodir com a contagem** — 500
elementos × 1 tick = 500 ticks de história, e a cauda cai fora de qualquer buffer. Aqui o `lag` é o
**tempo que o conjunto INTEIRO abrange**: elemento `i` mostra onde o stream estava
`lag · i/(n−1)` ticks atrás. O parâmetro tem o mesmo significado com 8 ou 800 elementos, e o buffer é
limitado por construção (`MAX_LAG = 32` ticks).

**A ordem é o eixo.** A rampa segue a ordem do stream (row-major, num `motion.grid`). Para varrer por
outro eixo — a fenda fotográfica de verdade — basta um **`motion.sort` upstream** (ordene por X e a
defasagem cresce da esquerda pra direita). Composição, não param novo.

**Defasagem fracionária:** um lookback de 3.4 ticks **interpola** entre os slots 3 e 4, então um `lag`
pequeno cisalha suave em vez de escadinhar por tick.

**O que é atrasado é a POSIÇÃO.** As colunas de aparência (tint/size/rot) seguem vivas: slit-scan é um
cisalhamento *geométrico* do tempo; ecoar linhas inteiras (cor e tudo) é o `motion.trail`.

### 4.3 A linha de atraso (estado)

Nó **sequencial** (porta `state`, self-loop `pre` plumbado pelo editor ao soltar). O ring vive como
**colunas planas na própria saída** (`ss_1`..`ss_32`, módulo irmão `ring.rs`) — o mesmo truque do
`motion.trail` ("o stream É o ring"), mas por-elemento em vez de por-geração, porque a saída precisa
continuar com `n` linhas. `slot(k)` = a posição de `k` ticks atrás, então um lookback é uma leitura de
coluna, não uma busca. Tick 0 (ou contagem mudada — emitter churnou, grid redimensionou): a linha
**re-semeia plana na pose viva**, então o scan se forma nos próximos `lag` ticks em vez de estourar
num passado-lixo. `Effect::Pure` (o tick entra na fingerprint pela aresta `pre` consumida).

## 5. Demo (o documento Motion default — `motion_demo_strobe.rs`)

Duas cenas pequenas, cada uma isolando um nó da fatia, **auto-play no boot**:

```text
ESQ  (pin):       grid ─> pin_constraint ─> integrate ─> collide ─> move(−7) ─> output
                                                ^
                                  pre└─ attractor ─ drag ─┘
DIR  (slit scan): grid ─> oscillator ─> slit_scan ─> move(+7) ─> output
                                          ^   └─pre─┘
```

- **ESQ:** a fileira de cima da cortina 8×8 está **pinada**; as outras 56 são sugadas pelo attractor e
  **empacotam ao redor** da fileira (que não sai do lugar quando o monte encosta nela). Puxe o slider
  `count` do pin pra 0 no painel de params e a cortina inteira desaba — é a leitura falsificável.
- **DIR:** o `phase_stagger` do oscilador é **0** (todos bobam na MESMA fase). A onda viajante que se
  vê só pode vir do scan. Ponha `lag` em 0 e a onda colapsa num bob rígido.

## 6. Verificação (o que ficaria VERMELHO)

**Testes de unidade:** 48 verdes nas 5 crates (9 pin · 14 integrate · 7 slit-scan · 10 collide · 8 spring).

**Testes de CORRENTE INTEIRA** (`shells/desktop/src/motion_state_tests.rs`, registry REAL, `pre`
avançado tick a tick — cozinhar um frame isolado mostraria a pose de seed e não provaria nada):

- `the_pinned_row_holds_while_the_rest_falls_into_the_attractor`
- `the_lockstep_bob_becomes_a_travelling_wave`

**E foram provados por MUTAÇÃO** (verde não prova nada — DIRETIVA §3):

| mutante | resultado |
|---|---|
| `motion.integrate` ignora `inv_mass` | ✗ VERMELHO — *"pinned element 0 never moved"* (a fileira cai junto) |
| `motion.collide` volta ao split 0.5/0.5 | ✗ VERMELHO — *"pinned element 3 never moved"* (o monte encosta e empurra o pin) |
| `motion.slit_scan` encaminha a pose viva | ✗ VERMELHO — o bob fica em lockstep (spread 0) |

O mutante do `collide` é o que fecha o buraco típico: ele prova que o contato **de fato acontece** na
demo, e não que a asserção passou por sorte.

## 7. Aberto

- `motion.path` (A2 da fila) segue **não decidido** — o plano diz "integra `vector.*`", mas os nós
  vetoriais foram retirados (ADR-0108) e a geometria vive em `ph2d-vec-scene`. É cross-module: ou
  crate satélite que só LÊ, ou defere. **Decisão do Enio.**
- `motion.verlet_rope` / `soft_body` / `boids` ganharem porta `in` (aí o pin genérico substitui os
  pins intrínsecos deles). Não é urgente e não foi feito: hoje não há como um stream entrar neles.
