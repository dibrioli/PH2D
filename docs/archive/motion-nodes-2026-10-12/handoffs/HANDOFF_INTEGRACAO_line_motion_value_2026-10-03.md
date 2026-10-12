# ARQUIVO — HANDOFF_INTEGRACAO_line_motion_value_2026-10-03.md (história, 165 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`HANDOFF_INTEGRACAO_line_motion_value_2026-10-03.md`](../../../Motion%20Nodes/handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-10-03.md) em 2026-10-12, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-190,216-340,481-610` do original.
>
> ⚠️ **A única alteração ao corpo:** 6 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

### §6.2 — A memória das células: `66 → 43 MB` (03/10, mesma linha)

- ✅ **`9d1058a40` — a cobertura NO LUGAR do 1.º acumulador e a capacidade ao OITAVO do degrau**
  ([doc 121 §9.12](../../../Motion%20Nodes/121_as_formas_na_placa.md), o fim). O `cs_varre` grava o `pack2x16unorm` na palavra
  do preenchimento do próprio pixel (já lida); sai o buffer `cobertura` (ligação `9` do grupo `2` do
  cálculo) e a ligação `2` do grupo `1` do desenho passa a ser a ACUMULAÇÃO. `528 → 400 B` por célula.
  A capacidade das células sobe a `n.next_multiple_of(2^(⌊log₂ n⌋−3))` (`ao_oitavo_do_degrau`, só as
  células): `107 520 → 114 688` em vez de `131 072`. `650128529` = `cargo fmt` (inclui um `use` antigo do
  `tests/it/contorno_calculado.rs`).
- **Kill-criterion (escrito antes) PASSOU:** app `=127` densa `114 688` células = **`43 MB`** (`≤ 46`),
  `60 fps` nas duas placas, uma só criação. Sonda intercalada contra `fab8999a8`, `PERFIL=1`: iGPU
  `0,88–0,89 → 0,89–0,90` (esticadas, `+1 %`) · `0,61 → 0,59` · `1,01 → 1,01`; RTX igual em tudo (`0,10`
  das conformes UMA vez em cada binário: o degrau de `0,01 ms`). Fragmento `56` VGPRs / `18` ondas, igual.
- ⚠️ **O preço do arredondamento fino:** numa cena que cresce UMA célula de cada vez, `118` recriações
  contra `18` (≤ `8` por oitava, com gate). Na `=127` a contagem não cresce.
- ⛔ **Para quem lê o diff:** a coluna `x` do fragmento corre a FILEIRA inteira — com o passo `ACUMULA` ela
  parte-se em célula (`x / 32`) e pixel (`x % 32`). O 1.º rascunho sem a partição deu `7/10` vermelhos
  e virou a mutação A19.
- **Gates re-corridos:** shape-gpu `10/10` + `3` unitários novos · produto `5/5` + sonda · gpu-cook formas
  `2/2` · [mutação](../../../Motion%20Nodes/ferramentas/mutacao_o_buffer_de_acumulacao_2026-10-03.py) **`19/19`** (A13
  re-ancorada; A18 e A19 novas; pré-voo `19/19`, corrida limpa `10` verdes, nenhuma por shader inválido)
  · [tracejado](../../../Motion%20Nodes/ferramentas/mutacao_o_tracejado_no_ecra_2026-10-02.py) **`21/21`** (corrida limpa `16` verdes) · nextest-impacted `17 045/17 045` · `cargo check --workspace
  --all-targets` com `-D warnings` · clippy das três crates · machete · os dois censos.
- Foundational: nenhum. Ids/consts novos: nenhum (sai o `COBERTURA`; a ligação `9` do grupo `2` fica livre).

### §6.6 — O bloco de 04/10 (`CLAUDE.md` §0.10): o tracejado por troço, `M6`/`S6`/`S8`, o brilho, o `cs_varre`

Plano e kill-criteria ANTES de construir, tabelas e recusas: [doc 121 §9.14](../../../Motion%20Nodes/121_as_formas_na_placa.md).
Uma rodada intercalada, partida em três por motivos medidos: a placa presa `25` min por outra linha deixou
células vazias (as duas ferramentas passaram a repetir a célula) e as densas da iGPU decidiram o (d) e
encurtaram o resto.

| commit | o quê |
|---|---|
| `4f0dcce34` | plano + kill-criteria dos quatro itens (§9.14) |
| `90672dcb2` → `f76f0212b` | (a) o passe de GRUPO por cópia tracejada — construído, medido e ⛔ RECUSADO (o contorno volta ao texto de antes) |
| `90672dcb2` → `d770fcfd9` | (d) D1 e D2 no `cs_varre` — construídos, medidos e ⛔ RECUSADOS |
| `90672dcb2` (parte que fica) | (b) gate de CPU `a_esquadria_de_um_vertice_liso_nunca_passa_da_flecha_do_nivel`; (c) `ShapePass::redesenha` |
| `5b88822fe` | (c) o halo do `fx.glow` pela rota do quadro (`motion_glow_layer::halo_do_quadro`); sai a `RECUSA_FORMA_COM_BRILHO` |
| `2c16ead81` | (c) ⭐ o halo COBRE onde o destino é transparente — o brilho voltava a zero em todo quadro por faixas |
| `e2340ad18` · `879e471b7` · o da shell | gates (o redesenho, a costura do halo), arneses, fecho (fmt, clippy, a contagem herdada da família de comunicação `64 → 65`), `present_chrome` sem o `motion` |
| `556e2f261` · `97790b57f` | as duas ferramentas de medição repetem a célula que a placa ocupada deixou vazia |
| `51ff6fee4` | memória: o censo de rota conta o cozimento; uma cerca implicada por outra nunca decide |

- **(a) ⛔** iGPU tracejadas esticadas `1,18`–`1,20 → 1,27` ms (`+6 %`; `conta + escreve 0,41 → 0,49`, o
  critério pedia `≤ 0,22`); RTX `0,37 → 0,19` (`−49 %`); `A0` (todas pelo grupo) densas iGPU `2,89`. A
  imagem era a mesma (gate pelas duas escritas: `6` famílias `40/40` pelo grupo, alfa `≤ 1`).
- **(d) ⛔** densas iGPU base `1,01`–`1,03` · D1 `0,98`–`0,99` (conformes `+8`–`20 %`) · D2 `1,03`–`1,05`
  (esticadas `+0,13`–`0,18`) · `V0` sem o prefixo `0,93` (o tecto da alavanca).
- **(b) ✅** `126 870` vértices lisos, ZERO esquadrias acima da cerca, a pior `0,086` px.
- **(c) ✅** as `4` cenas já desenhavam as formas pela placa (o censo media o COZIMENTO); o defeito real
  era o halo, e a foto da `=70` achou um segundo, anterior a esta linha: **com o passe de formas ligado o
  brilho não aparecia em rota nenhuma** (o quadro por faixas tem o `game_rt` transparente e o tonemap
  divide pelo alfa). Depois das duas curas a `=70` vai ao dispositivo (`device: HIBRIDO`, formas «do
  dispositivo») com o halo verde à volta da forma, a `60 fps`. Censo de rota: dispositivo `116` de `128`.
- **No produto** (`mede_formas_na_placa.sh`, `=127` densa, `16 384` cópias, `1930 × 1040`): contínua e
  tracejada `16,6`–`16,8 ms` (`60 fps`) nas duas placas, formas «do dispositivo», células `114 688` =
  `43 MB` — o mesmo de antes do bloco.

**Foundational tocado (aditivo):** `ph2d-render` — `render_instances_only` ganha `gpu_extra`; o composite
do brilho escreve cobertura (byte a byte igual sobre destino opaco; o emissivo das sprites ganha a mesma
cura). **Ids/consts novos:** nenhum (sai a `RECUSA_FORMA_COM_BRILHO`). **Contratos congelados:** nenhum.
**Shell:** a camada das formas desenha-se antes dos passes de luz (`present.rs`) e cola-se onde sempre
(`present_chrome.rs`); o `present_fx` recebe a placa das formas.

**⚠️ Para quem lê o diff:** o `cs_escreve_grande` e o D1/D2 entraram e saíram na mesma jornada — o
`contorno.wgsl`, o `contorno.rs`, o `contorno_sondas.rs` e o `tests/it/tracejado.rs` estão IGUAIS aos de
`4a1c99644`; as tabelas vivem no doc 121.

**Smoke:** §7.3.

#### §6.6.1 — O fecho do bloco (corrido nesta árvore, sobre o diff desde `1ad60a1ce`)

- nextest-impacted **`17 549/17 549`** · `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` ✓ ·
  clippy `--all-targets --all-features -D warnings` de `ph2d-shape-gpu`, `ph2d-app-motion`, `ph2d-render` e
  `ph2d-host-desktop` ✓ (dois achados no gate novo do brilho, curados em `fecho: clippy …`) · `fmt --check` ✓ ·
  censos **`127/127`** (`12/12`) · machete ✓.
- GPU: `ph2d-shape-gpu --test it --ignored` **`12/12`** · `motion_shape_placa::gpu_tests --ignored` **`6/6`** ·
  `ph2d-gpu-cook formas --ignored` **`2/2`** · `ph2d-render --lib motion_fx` **`26/26`** · costura da shell
  `placa_tests` **`3/3`** · gate de CPU da esquadria dos lisos ✓.
- **Mutação `9/9`** ([arnês](../../../Motion%20Nodes/ferramentas/mutacao_o_bloco_do_9_14_2026-10-04.py)): R1 · B1–B3 · H1–H5 — a
  B3 sobreviveu à 1.ª corrida (fixtura com halo forte de mais) e sangra depois da cura da fixtura.
- Perfil do loop (`agent-loop-profile.sh`, 20 sessões): paralelismo `1,14` · `test:check` `2,2:1` · Edit `35 %`
  · contexto relido `492 mil` · respostas por sessão `233` ✓ · contexto inicial `63 mil` ✓.

### §6.7 — O bloco §9.15 (04/10): onde está o custo das tracejadas grandes esticadas — ⚠️ FECHO PARCIAL

Plano, kill-criteria, tabelas e veredictos: [doc 121 §9.15](../../../Motion%20Nodes/121_as_formas_na_placa.md). ⚠️ **A rodada de
medição foi PARADA por ordem do dono (`19:26`)** depois de `94` de `360` células: das `16:45` às `19:00`
três outras linhas seguraram a carga em `16`–`30` (`journalctl`: `components` `216` comandos pesados,
`vector` `148`, `sculpt3d` `96`). Medido só na iGPU (o proxy de telemóvel): as esticadas contínuas e
tracejadas e as conformes contínuas.

| commit | o quê |
|---|---|
| `79e680c21` | instrumentos sem relógio: `arestas_do_ultimo_quadro` (reservadas · escritas · do contorno), CPU por fase, parede por quadro e a do 1.º; `mede_sonda_das_estrelas.sh` com `TRACEJADOS="0 1"`; `tabela_da_sonda.py` |
| `1ebba057c` | plano + kill-criteria (doc 121 §9.15), escrito antes de construir |
| `e6daad1fa` | os pedaços, cada um atrás de uma constante `true` (o código = o binário `F` medido) + 2 gates novos |
| `e00efb99d` | `troca_os_pedacos_9_15.py` (os 15 binários da rodada) |
| o desta secção | resultados (doc 121 §9.15) e este handoff |

**O que o código entrega (todos os pedaços LIGADOS):**

- **(c) explicado:** a «parede − soma» nunca explicada era o 1.º quadro cronometrado, ainda pixel a pixel
  pela variante completa (`76`–`78` ms na média de `250`). **c2** (`contorno_capacidade.rs`): uma cena NOVA
  (1.º desenho, ou as geometrias carregadas mudam) MEDE a capacidade bloqueando antes do 1.º quadro — o 1.º
  quadro de uma cena tracejada `78 → 1,6` ms na iGPU.
- **(b) a premissa caiu:** as tracejadas escrevem MENOS arestas; o `cs_deposita` corria um fio por aresta
  RESERVADA (`15,5×` as escritas). **B1** `cs_soma_escritas` + o `cs_deposita` por aresta ESCRITA (células
  `0,62 → 0,47`); **B2** a reserva conta a junta uma vez por troço (prova no doc): reservadas `561 600 →
  285 120` — e o buffer de arestas, que cresce à potência de dois da reserva, deixa de pedir o dobro.
- **(a) A1a** o ajuste do tracejado sai da contagem (o acumulador `Ajuste` em `shape.wgsl` é UMA porta para
  a contagem e o pixel a pixel); **A1b** o total de um fechado sai do próprio percurso (o 1.º traço, o da
  emenda, emite-se no fim). ⚠️ O critério combinado FALHOU (`0,34` contra `≤ 0,30`); fica pela imagem igual
  e o ganho sem regressão — ver «decisões» abaixo. A ablação `E1` mostra onde está o resto: a EMISSÃO.
- **(d) D** o `cs_varre` por subgrupo (`contorno_subgrupo.wgsl`, módulo à parte só com `Features::SUBGROUP`),
  a imagem igual byte a byte nas duas placas.
- **`F` (iGPU):** esticadas tracejadas `1,17 → 0,95` ms (`−19 %`), parede mediana `1,27 → 1,03` contra o Vello
  `0,95`; esticadas contínuas `0,90 → 0,85`; conformes `0,61 → 0,60`.

**Foundational tocado (aditivo):** `ph2d-gpu` (`context.rs`) — o dispositivo pede `Features::SUBGROUP`
quando o adaptador a anuncia (a mesma interseção das outras; onde falta, o passe usa o prefixo de memória
de grupo). **Ids/consts novos:** em `ph2d-shape-gpu` (privados): `AJUSTE_NA_CONTAGEM`, `TOTAL_NO_PERCURSO`,
`ARESTAS_COMPACTAS` (WGSL e Rust — mudam juntos), `JUNTA_UMA_POR_TROCO`, `MEDE_NO_INICIO`,
`PREFIXO_POR_SUBGRUPO`, `QUINTOS = 5` (a `contagem` passa de três a cinco contagens por cópia). **API
pública nova** (`ShapePass`): `com_subgrupo`, `tem_subgrupo`, `mede_a_capacidade_no_inicio`,
`arestas_do_ultimo_quadro`. **Contratos congelados:** nenhum. **Shell:** não tocada. **LOC:**
`contorno.rs` `627 → 684` (tecto `700`).

**⚠️ Para quem lê o diff:** os gates que mediam a MISTURA do 1.º quadro (o pixel a pixel ao lado das
células) desligam o c2 no ajudante (`pelo_passe_observado` → `mede_a_capacidade_no_inicio(false)`): a régua
deles continua a mesma; o c2 tem gate próprio (`a_cena_nova_mede_a_capacidade_antes_do_primeiro_quadro`,
com o CONTROLO do 1.º quadro sem a medida e uma segunda cena com `4×` as cópias).

**O fecho que CORREU:** `cargo check` das três crates e dos testes · `ph2d-shape-gpu --test it --ignored`
**`14/14` nas duas placas** (RTX e iGPU — o `D` com subgrupos de `32` e de `64`), com os dois gates novos
(`a_cena_nova_mede_a_capacidade_antes_do_primeiro_quadro`, `o_prefixo_por_subgrupo_e_o_mesmo_que_o_de_memoria_de_grupo`).

**✅ O gate batched CORREU depois (04/10, nesta árvore, HEAD `518fda62b`, carga `3,7`–`4,7`), tudo verde:**
nextest-impacted (`BASE=1ad60a1ce`) **`17 580/17 580`** · `CARGO_BUILD_WARNINGS=deny cargo check --workspace
--all-targets` ✓ · clippy `-D warnings --all-targets --all-features` de `ph2d-shape-gpu`, `ph2d-gpu` e `ph2d-app-motion` ✓ ·
`fmt --check` ✓ · censos **`127/127`** (`12/12`) · machete ✓ · `ph2d-shape-gpu --test it --ignored` **`14/14`** na RTX e
**`14/14`** na iGPU · `motion_shape_placa::gpu_tests --ignored --skip sonda_` `5/5` · `ph2d-gpu-cook formas --ignored`
`2/2` · `ph2d-render --lib motion_fx` `26/26` · `ph2d-gpu device_enables…` `1/1` · o tecto de LOC (`architecture_workspace_file_loc_cap`) ✓.

**⏳ O que continua por correr:**

1. as mutações dos pedaços (arnês por escrever; as previstas no doc 121 §9.15 — o ajuste da contagem a `1`,
   o traço adiado nunca emitido, o prefixo das escritas deslocado, a reserva sem as pontas, o c2 desligado e
   a cena nova sem `medir_ja`, a correção da fronteira da célula no subgrupo, esta só na iGPU);
2. a medição que faltou: densas da iGPU (o critério do B1 e do D), a RTX inteira (nenhum arranjo pode
   piorar `+10 %`), o app (`mede_formas_na_placa.sh`, `=127` densa contínua e tracejada — e a memória das
   arestas, que o B2 corta), os registos (`registos_dos_shaders.sh`);
(o smoke está compilado e fotografado: §7.4.)

**Decisões técnicas registadas:** (i) o **A1 fica** apesar do critério combinado: imagem igual, `−0,06` ms
sem regressão em arranjo nenhum medido; a régua dele pressupunha que as voltas eram o custo, e a ablação
`E1` provou que é a emissão. (ii) As **constantes dos pedaços NÃO foram dobradas** (o código é o `F` medido;
dobrá-las pede os gates outra vez) — o próximo a mexer aqui apaga os ramos `false`. (iii) ⛔ **pontas
partilhadas RECUSADAS por construção** (a ponta fecha cada traço). (iv) ⛔ continuam recusados: o GRUPO
por cópia tracejada, o D1/D2, a esparsa (§6.6, §6.5).

Perfil do loop (`agent-loop-profile.sh`, 20 sessões): paralelismo `1,13` · `test:check` `3,0:1` · Edit `33 %` ·
contexto relido `487 mil` · respostas por sessão `248` ✓ · contexto inicial `63 mil` ✓.

