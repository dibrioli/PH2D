# 37 — Editor F2: as teclas que nunca chegavam · probe + sparkline · smart-connect — nota-ADR

**Data:** 2026-07-12 · **Linha:** `line/motion-value` (Modo L) · **Fase:** editor **F2**
**Status:** implementado, testado, **smoke do Enio OK**
**Contrato congelado encostado:** **nenhum** · **Foundational tocado:** `shells/desktop/src/input_handlers.rs`

---

## 1. A causa-raiz: o Ctrl+D e o K estavam MORTOS (bug do smoke)

O painel implementava `GraphKey::{Duplicate, Knife}` e o `dispatch/key.rs` **mapeava** Ctrl+D / K.
Mesmo assim, **nada acontecia**. A costura estava rompida um nível acima:

> **O shell empurra as teclas do grafo por conta própria** — `input_handlers.rs` empurra
> `push_graph_key(...)` **gated pelo CURSOR** (`over_motion_graph`), justamente porque o caminho
> focus-gated do editor-core pode estar preso num foco velho. E ele **só cobria F / Delete / A / Space**.

Ou seja: eu fiei a ponta do PAINEL e **não verifiquei o PRODUTOR**. É exatamente a *costura não-testada*
que a DIRETIVA §1 descreve — o painel compila, os testes do painel passam, e a tecla **nunca chega**.
`unit-verde ≠ vivo no produto` ([[feedback_tool_unit_green_integration_dead]]).

**Fix:** três braços novos em `input_handlers.rs`, gated por `over_motion_graph`:
`Ctrl+D` → `Duplicate` · `K` → `Knife` · `P` → `Probe`.

**A armadilha de ordenação** (quem pegou foi o **clippy**, não eu): `KeyCode::KeyK` **já era casado antes**
— é o *insert-key* global da timeline. Um braço novo **abaixo** dele é inalcançável: a faca nunca armaria,
**em silêncio**. `clippy::unreachable_pattern` reprovou; os braços do grafo passaram a vir **ACIMA** do
`KeyK` global. Lição durável: *num `match` de teclado global, um braço mais específico colocado depois de
um mais genérico não é um bug de lógica — é código morto que o compilador só denuncia se você o escutar.*

## 2. "Não entendi K o que faz" (a segunda metade do mesmo bug)

A faca era um **modo invisível**: armava, não mostrava nada, e mudava o que o próximo arrasto fazia.
Isso é adivinhação, não interface — e a resposta certa não é documentação, é **estado na tela**.

**Chips de Knife e Probe na toolbar** (`CHROME_KNIFE = 4`, `CHROME_PROBE = 5`), com **anel `Accent`
enquanto armados**. Clicar o chip arma/desarma, `Esc` desarma. A tecla continua existindo pra quem já
sabe; o chip é o que **ensina** que ela existe.

## 3. Probe + sparkline

`P` (ou o chip) arma; o clique seguinte **aponta o probe** pro nó. O nó ganha um anel e um card flutuante
mostra o valor + a **sparkline dos últimos 60 ticks**.

- **O shell lê pelo `Cook` DO PRÓPRIO PUMP** (`motion.pump.cook.cook(...)`) — o memo já tem o resultado
  **deste tick**, e o `pre` é o estado vivo da simulação. Custo: uma consulta ao memo, **não** uma segunda
  avaliação da cadeia. E, por construção, o probe **não pode** reportar um tick diferente do que está na
  tela (um `Cook` próprio divergiria no primeiro frame perdido).
- **O painel não guarda histórico.** O ring de 60 vive no shell (`MotionState.probe_ring`) e viaja inteiro
  no snapshot (`ProbeView`). Duas histórias = duas verdades.
- **O que é o número:** stream de VALOR lê o escalar (coluna `v`); qualquer outro lê **quantas instâncias**
  carrega. São as duas perguntas que um artista realmente faz a um fio ("quanto vale?" / "quantos são?") e
  o **label diz qual** está na tela — um número pelado é charada.
- **A sparkline auto-escala pro PRÓPRIO range.** Um wiggle de ±0.001 tem que *ler* como wiggle; um eixo
  fixo `0..1` o achataria numa reta morta. Pico no topo (y de tela cresce pra baixo — o sinal que desenha
  todo gráfico de cabeça pra baixo), sinal chato desce pelo meio (sem divisão por zero), `NaN` (sim
  divergida) não envenena a escala.
- O probe **morre com o nó**: nó deletado → `probe = None`, ring limpo.

## 4. Smart-connect

Soltar um fio no vazio era um **no-op silencioso**: o artista desenhava o gesto e não recebia nada.

Agora abre o **add-menu carregando o socket de origem** (`AddMenu.connect_from`), listando **só** os tipos
cujo input aceita aquele fio (`snapshot::menu_catalog` filtra por domain+dim+clock). A escolha vira **UM**
intent (`GraphIntent::SmartConnect`) → `edit::smart_connect()` **adiciona E liga**, em **um passo de undo**
— porque foi **um gesto** (um add que o artista ainda tem que ligar na mão é o add-menu que ele já tinha).

**O filtro do menu é cortesia, não licença.** A conexão passa pela **mesma autoridade** de um fio desenhado
à mão (ciclo · input ocupado · tipagem · membrana do time-scope), validada num clone de teste antes de
entrar no doc; se for recusada, **o nó ainda nasce** (o artista pediu por ele) e o toast diz por quê.

## 5. Superfície nova (pro integrador)

| Onde | O quê |
|---|---|
| `shells/desktop/src/input_handlers.rs` | 3 braços de tecla (`Ctrl+D`/`K`/`P`), **acima** do `KeyK` global |
| `ph2d-panel-motion-graph` | `CHROME_KNIFE = 4` · `CHROME_PROBE = 5` · `PROBE_SAMPLES = 60` · `ProbeView` · `probe.rs` |
| `GraphIntent` | `SetProbe { node }` · `SmartConnect { from_node, from_port, to_type, x, y }` |
| `MotionState` | `probe: Option<NodeId>` · `probe_ring: Vec<f32>` |

**Nada de contrato:** `NodeManifest`/`NodeOp`/`OpResolver` intactos (8/2/1).

## 6. Aberto (F2 restante)

`waypoints`/branches nos fios · readouts inline no body do card · template "nó sequencial".
Nenhum deles bloqueia uso.
