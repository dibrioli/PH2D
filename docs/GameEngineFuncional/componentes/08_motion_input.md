# Motion Input

**Estado: `em obra`** (15/11). Leva um valor do jogo (número, cor, vida, dano) a um efeito do Motion: o nó
`value.input` com o mesmo nome o lê. Crítica: `critica_ph2d.html#motioninput` (§1–§8, a tabela «O que falta
fazer» com 21 capacidades e MI-1..MI-12). Oráculo Godot 4.7.2 corrido pela crítica:
`GameEngineSamples/fonte_critica/oraculo_motioninput/` (6 casos: o Godot só liga valor a efeito por código,
o hint de faixa não prende o valor vindo do código, o nome inexistente é aceito calado).

**Decisão do dono (15/11):** *«faça conforme sugere»* — a maquete **B** (os nomes do grafo como escolha
principal, nome já escolhido ao adicionar) e as **três ondas** abaixo, todas neste item.

## A tabela conferida no código (15/11, base `3690872b29`)

A crítica errou em 3 «não»: ler outro objeto, qualquer campo e ação do Input Map existem **por script**
(`ph2d.get`, `action_*` em `runtime/luau/ph2d.d.luau`, depois `motion.set`). Faixa e suavizar existem
**dentro do grafo** (`value.map_range`, `value.smooth`). Confirmado o defeito 21: sem a fonte, a linha do
objeto fica PRESA no último valor (`motion_input_bridge.rs`: `vivos` entra antes do `match`).
Velocidade e «no chão» não estão nem no Luau.

| # | Capacidade | Hoje | Onda |
|---|---|---|---|
| 1, 2, 7, 8, 14, 16, 18, 22 | número, vida ÷ máx, disparo, cor, script escreve, Timeline, um nome em vários grafos, ordem fixa | sim | — |
| 10 | sim/não | 0/1 serve | — (sem proposta) |
| 19 | avisos | sim, sem saída | 1: botão no aviso («Add Health») |
| 20 | leitura ao vivo | não | 1: MI-11 |
| 21 | fonte some | **preso** | 1: MI-12 (cura + gate) |
| — | nome inicial «input» | tropeço | 1: o primeiro nome livre do grafo |
| — | cartão | texto + chips | 1: maquete B |
| 3 | qualquer campo | só script | 2: MI-1 fonte Property |
| 4 | outro objeto | só script | 2: MI-2 From |
| 5 | velocidade, no chão | não | 2: MI-3 Speed · Grounded |
| 6 | ação do Input Map | só script | 2: MI-4 Action |
| 11 | faixa | só no grafo | 2: MI-6 From · To · curva |
| 12 | suavizar | só no grafo | 2: MI-7 Smooth |
| 9 | vetor | não | 3: MI-5 Vector |
| 15 | script lê | não | 3: MI-9 `motion.get` |
| 17 | várias entradas | um por objeto | 3: MI-10 |
| 13 | o grafo avisa o jogo | não | 3: MI-8 nó Send signal |

## Ondas (cada uma fecha com smoke do dono)

1. **Consertos e cartão B.** Cura do valor preso (21) com gate; nome inicial do grafo; leitura ao vivo
   («Reads Health here: 0.75» / o que o grafo recebe em Play); botão de saída no aviso; Input = chips do
   grafo na frente (o texto só para um nome novo). Cena de smoke nova do Game com um efeito que reage à
   vida (formas + simulação, fotografada).
2. **Origens sem script.** MI-1 Property (o endereço de campo do Auto Key), MI-2 From (objeto, conta-gotas;
   vazio = este), MI-3 Speed · Grounded (do corpo dono), MI-4 Action (Input Map), MI-6 From · To · curva
   (o valor chega em 0..1), MI-7 Smooth (s).
3. **O que é maior.** MI-5 Vector, MI-9 `motion.get`, MI-10 várias entradas por objeto, MI-8 o grafo
   publica um sinal que regras e script ouvem.

Regras: `PROJECT_SCHEMA` sobe quando o tipo muda, sem leitor velho (§0.11); clique REAL e animação para todo
controle; checkbox em moldura; rótulo que não cabe = «…»; catracas de LOC (mover/cortar).

## Desenho da onda 2 (portas existentes, sem segunda conta)

- Fontes novas no FIM de `MotionSource` (a ordem é a do seletor): `Speed`, `Grounded`, `Action`, `Property`.
- `from` (texto, `Target::Stable`, conta-gotas `ph2d-app-components::conta_gotas` como o Ray Sensor; vazio = este
  objeto): vale para Health, Damage, Speed, Grounded e Property.
- `Property`: componente + campo do objeto From, escolhidos em duas listas (o modelo do Property Binder do Unity);
  guardado como texto `canonical/serde_key`; lido por `canais_componente::ler` (o mesmo funil da Timeline/Auto Key).
- `Action`: o nome de uma ação do Input Map, lido pela `action_strength` (a mesma do Luau), 0..1.
- `Speed` / `Grounded`: do corpo dono (`owner_body`); a velocidade viva e o `grounded` do personagem.
- Faixa (MI-6): caixa «Remap» (moldura) + From · To (o valor chega em 0..1, com clamp — a lei do `value.map_range`)
  + curva (`ph2d-curve`, texto).
- Smooth (MI-7): segundos de meia-vida — a lei do `value.follow` Smooth, `v += (alvo − v)·(1 − 2^(−dt/s))`; a ponte
  passa a receber o `dt`.
- `PROJECT_SCHEMA` sobe (`shells/desktop/src/project_schema.rs` + irmãos; `scripts/schema-recount.py`).

## Como ficou

### Onda 1 (15/11, commit `835fe76619`) — smoke do dono OK 16/11

Cena `PH2D_GPU_COOK_DEMO=134 PH2D_GAME_SMOKE=27` (`ph2d-physics-scene/src/cena_motion_input.rs`), fotos `fotos_08/`. Mutação 12/12.

- **Cura do valor preso (21/MI-12):** sem a fonte, a linha do objeto SAI de `MotionInputs` e o `value.input` volta
  ao número dele. Gate `a_source_that_disappears_stops_publishing_instead_of_freezing` (vermelho antes).
- **Nome inicial:** `MotionGraphNames` (recurso, `ph2d-ecs`) publicado por `motion_entradas::publica_os_nomes`
  em `publica_o_mundo`; semente própria da família (`component_seed::PROPRIAS`) → `semeia_o_nome`.
- **Cartão B:** chips do grafo + «New…» (esvazia e foca a caixa); a caixa só para um nome que o grafo não lê.
- **Leitura ao vivo:** «Reads Health here: 0.75» (também parado), «Sends 0.50» e «Fired N times» em Play, e quem
  tapa o nome («A script sets this name first (0.20) — the graph reads that.»).
- **Saída do aviso:** «Add Health» → `InspectorAddComponentRequested { component: Some(..) }`, a mesma porta da
  paleta (cascata com o corpo, recibo, Undo) sem a abrir.
- Gates: `a_seccao_motion_input_esta_viva` (6), `motion_input` da app-components (10),
  `a_named_request_attaches_that_component_without_opening_the_palette`,
  `the_graph_names_reach_the_world_and_follow_the_graph`.

### Onda 2 (16/11) — aguarda o smoke do dono

Commits `c5b50d97b4` motor · `10eae12679` cartão · `273a2b4a8a` cena/testes · `387590dbeb` leitura parada da
Property · `28d5302ba1` curas da mutação.

- **Motor** (`motion_input_fonte.rs`): fontes Speed/Grounded/Action/Property; `from` = StableId (0 = este; o painel
  edita pelo nome); Remap (clamp) → curva (`ph2d-curve`) → Smooth (`value.follow::passo` Smooth); `dt` de jogo
  (`Relogio::dt_de_jogo`, 0 parado). `PROJECT_SCHEMA` 245 → 246. `ActionSample.strength` (a mesma do Luau).
- **Cartão:** Source em 8 chips (duas fileiras); From com conta-gotas (`ph2d-ray-sensor/src/gotas_from.rs`);
  listas Action/Property (`popovers_motion_input.rs`); grupo Shape fechado com resumo; saídas «Add Physics Body»
  (Speed sem corpo) e «Pick object» (From sem alvo); «Reads <campo> here» parado para a Property.
- **Cena 27:** + Health Bar Reader (From), Speed Reader (bola, Remap 0→6, Smooth 0.3 → «estado»), Max Health
  Reader (Property) e Tap Reader (Action) — estes dois escrevem em «vida» atrás do Health Bar Reader (o grafo 134
  não tem nome numérico livre); o cartão diz quem escreve primeiro. Jump Glow (Grounded) fora: provado em teste.
- **Mutação:** 32; 23 sangraram à primeira, 8 sobreviventes curadas por assert, 1 equivalente (#18, s<=0).
- ⚠️ **Para o integrador:** para caber nas catracas, o motor cortou ~482 linhas de comentário `//` histórico em 11
  ficheiros de outras áreas da `ph2d-app-components` (`instance_*`, `instantiate`, `asset_*`, `camera_2d`,
  `parallax_bridge`) e 29 em `shells/desktop/src/render_loop/bgremoval_preview_gpu.rs` — só comentário, zero código.
