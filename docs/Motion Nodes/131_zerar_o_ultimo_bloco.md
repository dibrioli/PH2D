# 131 — ZERAR, o último bloco: a `=70`, o vermelho da `=129` e a fila C · D · E (plano, 12/10)

> Ordem do dono (12/10): os itens 2, 3 e 4 do que ficou aberto no
> [handoff B6](handoffs/HANDOFF_CONTINUACAO_line_motion_value_2026-10-12_ZERAR_FAMILIA_B_B6.md) num ÚNICO ciclo
> (`CLAUDE.md` §0.10): um plano com TODOS os casos antes de agir, UMA rodada de medição intercalada para todos os
> itens de velocidade, um gate batched no fecho. Protocolo: [103](103_dinamica_dos_ciclos.md). Linha
> `line/motion-value`, rebaseada sobre o `main` em `71df3743b0` (59 commits, nada integrado).
>
> Recusas consultadas antes de desenhar: [130 §3.1](130_o_passe_na_placa.md) («todos os pares» sem grelha),
> [115 §9.4–9.5](115_o_colisor_sai_do_grafo.md) (a separação a 32 varreduras custa 40,8 % do quadro),
> [118 §6](118_a_mistura_em_grupo.md), [127](127_ciclo_18_a_cor_que_o_olho_ve.md) (`color_array` não mistura),
> [128 §5.3](128_ciclo_19_o_jogo_manda.md) (empurrão radial).

## §0 — A ordem (o que é série e porquê)

| lote | itens | porque é este o lugar |
|---|---|---|
| **1 — paralelo** | 2 (gate + foto) · 3 (sonda de deriva) · D.13 · D.14 · E.15 (`documentador`) | nenhum muda o outro |
| **2 — série** | as variantes C.7 e C.8 construídas | ⚠️ C.8 depende do 3: se a deriva da `=129` for defeito do sufixo híbrido, a cura muda a rota que a sonda C mede |
| **3 — UMA rodada** | sonda C (7 · 8 · 10 · 11 · 12) num processo, perfil `smoke`, `PH2D_GPU=1`; C.9 à parte (é perfil de build, ver §4) | a régua de [MEDIR_VELOCIDADE](../DevOps/MEDIR_VELOCIDADE.md) |
| **4 — fecho** | mutações (`mutacao`), smoke fotografado, gate batched (`verificador`), handoff ≤ 8 KB | §1.5.9 |

## §1 — Item 2: as estrelas da `=70` giram em volta do centro DELAS

- **Estado:** a cura `1a71c8f800` (era `f0370a2786`) está viva no HEAD — `CENTRO_DAS_ESTRELAS`, vórtice, atrator e
  arrasto em `motion_state_conferencia_demos_fx.rs:38-43, 148-190`; nenhum commit posterior toca os dois ficheiros.
- **Lacuna:** o gate `the_stars_orbit_and_stay_on_screen` (`…_fx_tests.rs:163`) prova que andam e não fogem; **não**
  prova que GIRAM. O defeito original (vórtice na origem do mundo) passava nele.
- **Lei:** por estrela, o ângulo desenrolado em volta de `CENTRO_DAS_ESTRELAS` avança com o MESMO sinal em 10 s, e o
  raio fica numa banda. Os limiares saem da medição da cena curada (tabela no §7), nunca escolhidos.
- **Gate:** `as_estrelas_da_70_giram_em_volta_do_centro_delas` (CPU, 600 tiques).
- **Mutações:** centro do vórtice de volta à origem · `strength` do vórtice a 0 · sinal do vórtice trocado na
  metade das estrelas (deve reprovar o «mesmo sinal»).
- **Smoke:** foto pelo `fotografa_cena.sh "PH2D_GPU_COOK_DEMO=70" 1600 972 <png> 12`, e a `=70` entra no roteiro.

## §2 — Item 3: o vermelho da `=129` é lei ou defeito?

Factos: as duas colunas da cena são IGUAIS menos o colisor (`coluna(g, k, colisor, r)`), e só a 2.ª saída (com
colisor) falha. O `motion.transform` dela só soma deslocamento (`p·1 + o`, `f = 1`), com a mesma álgebra nos dois
lados (`lib.rs:234` × `kernel.rs:55-63`). Δ = `2,74e-3` a `TIQUE = 3`, amplitude `4,686`.

- **Medição que decide** (sonda `#[ignore]`, placa, MESMA ponte do gate): para `T ∈ {1, 2, 3, 5, 10, 30, 60, 120}`,
  `max|ΔP|` placa × CPU em: (a) as duas saídas; (b) o nó `soft_body` de cada coluna — a **ablação do transform**;
  (c) por peça, a distância ao par em contacto mais próximo no tique em que o Δ aparece.
- **Critério, escrito antes de medir:**
  1. `Δ(corpo) ≠ Δ(saída)` acima de 1 ULP ⇒ o defeito é do sufixo do transform ⇒ cura + gate.
  2. `Δ(T=1) ≥ 1e-4` com a coluna de controlo a `~1e-6` ⇒ diferença SEMÂNTICA no primeiro passo (não é
     arredondamento: o ULP a `|P| ≈ 5` é `4,8e-7`) ⇒ defeito ⇒ cura + gate.
  3. Δ nasce no ULP e cresce com os tiques, ou salta num tique em que um par está a `≤ ε` do limiar de contacto
     (a paragem cai de lados diferentes, como o 130 §4) ⇒ **lei** ⇒ barra RELATIVA à amplitude, classe
     `Contato` em `barra_da_saida`, com a tabela por tique ao lado e o par limiar nomeado.
- **Gate:** o próprio `as_cenas_de_varias_saidas_pela_placa_dao_o_que_a_cpu_da`, verde, sem subir a barra
  absoluta das outras saídas.
- **Mutação:** a classe `Contato` aplicada a TODA saída (deve deixar passar um defeito plantado numa saída sem
  colisor) · e, se houver cura, a cura revertida.

## §3 — Fila D (produto)

**D.13 — a linha de COR num gesto; vida e dano.** Medido no código: a cor já chega com 1 nó (`value.input` kind
Color) + 3 `drive_param` ao `motion.tint` (`=134`, `motion_state_jogo_demo.rs:237-255`); no CARTÃO do objecto já é
um gesto só. O que custa três gestos é o GRAFO: ligar o Input ao Tint fio a fio. Os portos não têm tipo cor
(`Dim` sem Color; os params são `f32`), e mudar o manifesto é contrato congelado (§6 do `CLAUDE.md`) ⇒ **não**.
- **Lei a construir:** soltar um fio de uma saída de cor (`r`/`g`/`b`/`a` de um Input Color) sobre QUALQUER canal
  de um grupo de cor (`r g b a` ou `r2 g2 b2 a2`) liga o GRUPO inteiro, canal a canal, num só passo de undo.
- **Medição antes:** quantos gestos o editor de nós pede hoje (teste que dirige o gesto real pelo `ph2d-ui-testkit`).
- **Gate:** seam test: um arrasto ⇒ 4 `param_sources` · um `Ctrl+Z` ⇒ 0. **Mutação:** ligar só o canal solto.
- **Vida e dano:** cada componente tem UM `source` (`motion_input.rs:43`) — por desenho. A composição JÁ exprime
  dano a partir da vida: `value.input vida → pulse.on_change (Fall)`. **Por medir:** que ele dispara nos MESMOS
  tiques que o `Damage` do componente. Gate de equivalência; se iguais, a pendência fecha por lei (um objecto basta)
  e a `=134` ganha o passo; se não, a diferença é o achado.
- **Smoke:** cena nova `=136` (o próximo nível livre, `MAX_DEMO_LEVEL = 135`): formas + simulação com campos,
  um objecto com vida cuja cor muda e pisca no dano.

**D.14 — o colisor pelo NOME na placa.** `RECUSA_COLISOR` dispara quando um text param vale `collider`/
`collider_box`/`collider_offset` (`motion_bridge_gpu_colisor.rs:119-130`). O passe da placa JÁ lê essas colunas
(`passe_na_placa.rs:30-41`); o que falta é escrevê-las: o `motion.drive` `Custom` não tem kernel
(`kernel.rs:447-450`) porque o nome só existe no cozimento.
- **Lei a construir:** os três nomes são constantes conhecidas ⇒ três canais de kernel no `motion.drive`
  (como os `CH_*`), e o `Custom` com um desses nomes cai neles; a recusa passa a valer só para nomes que a placa não
  conhece. ⚠️ Respeitar o tipo da coluna (`channel.rs:124-127`: escalar sobre `Vec2` muda o tipo).
- **Gate:** CPU × placa numa cena que declara o colisor pelo nome (paridade com a tolerância do 130 §4) · e
  a recusa continua para um nome desconhecido. **Mutação:** canal do nome trocado; o arm que nunca dispara.
- **Smoke:** cabe na `=136` ou numa `=137`, contada no roteador.

## §4 — Fila C (desempenho): UMA sonda, variantes no mesmo processo

Sonda `a_fila_c_intercalada` (release/`smoke`, `PH2D_GPU=1`, blocos com a ordem rodada, o MÍNIMO, `loadavg` ao
lado). Cada linha decide se o item constrói ou fecha por lei.

| item | variantes no MESMO processo | o que decide |
|---|---|---|
| **7** scan por subgrupos (a placa: RTX 5060 Ti, subgrupo 32) + ligações por cozimento | pipeline `scan` atual × pipeline com `subgroupExclusiveAdd` (feature pedida no `GpuContext`); ligações recriadas × em cache | cadeia 16 · tecto 1024 (130 §3.1: `11,3 ms` × CPU `1,94`). Entra se ganha no mínimo E o scan inteiro der o MESMO resultado ao bit |
| **8** piso híbrido ~0,55 ms | 1 024 estrelas: `FullyGpu` (sem `wiggle`) × `Hybrid` × CPU, com o tempo de `cook_taps_only` separado do sufixo e da espera | se o piso é o prefixo da CPU, a cura é levar o prefixo à placa; se é a espera, é lei do dispositivo (dita com a tabela) |
| **10** silhueta W3b, 4 096 = 4 páginas (`4,64` × `0,61 ms`) | 1 · 2 · 4 páginas, a decomposição por passe (`pass_profiler`) | onde vão os 4 ms; página maior só com o tecto do RECURSO (limite de textura do adaptador) |
| **11** `Tint` OKLCH e `Bezier` na CPU · fita > 32 MiB | OKLCH × sRGB a 1k/16k/64k linhas; fita abaixo × acima de `FITA_BYTES` | o preço por linha ao lado do quadro; a fita é orçamento de RAM do histórico — o custo da dobra medido |
| **12** `0,115 ms` UMA vez por troca de rota (CICLO_20 §9) | troca a meio × sem troca, 200 trocas | se `≤ 1 %` de um quadro de 60 Hz, fecha por lei (invisível) com a tabela |

**C.9 (`opt-level` da `ph2d-contact`)** é perfil de BUILD, não um caminho do processo: dois binários de teste
pré-compilados (alvos separados DENTRO da worktree), corridas alternadas, o mínimo; o número decide como decidiu a
`ph2d-node-motion-collide` (`12,9 s` × `87,7 s`). Só afecta `dev`/`ci-test`.

## §5 — Fila E: os 33 docs do Motion acima de 40 KB

`python3 scripts/doc-split.py <doc> --keep <faixas> --archive docs/archive/motion-nodes-2026-10-12/<doc>` (prova
sha256 antes de gravar); o vivo fica com cabeçalho-roteador, o ABERTO e a tabela `⛔ Recusas MEDIDAS`;
`bash scripts/doc-index.sh` no fim. Modelo: `docs/Skeleton/01_a_fila.md`. Delegado ao `documentador`; o
corte é verificado por `git diff --stat` (linhas totais iguais entre vivo + arquivo e original).

## §6 — Fecho

`verificador`: `nextest-impacted`, `check --workspace` com warnings negados, GPU com `PH2D_GPU=1`, clippy
`-D warnings`, fmt, typos, machete; flakes de carga confirmados sozinhos com o `loadavg`. Mutações pelo `mutacao`
(RED só sobre VERDE). Smoke fotografado: `=70`, `=136` (D), números do terminal (C). Handoff ≤ 8 KB, binário
`smoke` compilado 2×, `rm -rf target/*/incremental`. Não integra nem envia.

## §7 — Medições (preenchidas ao correr)

_(vazio até à rodada)_
