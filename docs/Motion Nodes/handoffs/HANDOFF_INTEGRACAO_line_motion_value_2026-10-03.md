# HANDOFF DE INTEGRAÇÃO — `line/motion-value`, 2026-10-03 (AS LISTAS, O TRACEJADO NO ECRÃ E O BUFFER DE ACUMULAÇÃO)

> **Para o agente INTEGRADOR** (só por ordem do Enio — `CLAUDE.md` §0.7). A linha está sobre o `main`
> (o rebase da jornada foi no-op), passou o gate batched de fecho (§5) e **não integra nem pusha
> sozinha.** SUPERSEDE, como documento de integração, o de
> [2026-10-01](HANDOFF_INTEGRACAO_line_motion_value_2026-10-01.md) (as formas na placa; já no `main`).
> Os **18 commits** abaixo vêm depois dele. Mecanismo, tabelas e recusas medidas: [doc 121
> §9.8–§9.12](../121_as_formas_na_placa.md) — aqui só o que quem funde precisa de saber. O que a linha fez
> antes de hoje: [continuação 02/10](HANDOFF_CONTINUACAO_line_motion_value_2026-10-02.md) e
> [continuação 03/10](HANDOFF_CONTINUACAO_line_motion_value_2026-10-03.md). ⭐ **O §3 do de 02/10 («o que um
> leitor do diff entende ao contrário») continua válido** — leia-o (com a ressalva do §3 abaixo: o item 1
> dele, `QUADROS` 3→5 / 4→6, foi revertido por este fecho).

## §0 — IDENTIDADE

| | |
|---|---|
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value` |
| ramo | `line/motion-value` |
| HEAD | `311413f1e` + este handoff · ⚠️ e, DEPOIS dele (03/10, após o smoke aprovado): `87e605534` (a capacidade só MEDIDA das células, código) e `9b133f887` (recusa da largura, doc) — ver §6.1; e `9d1058a40` · `650128529` · os docs (a memória das células `66 → 43 MB`) — ver §6.2; e `a6e067f45` · `1c8f25d1e` · `757c5356a` · `64120d51a` · `2062db7d8` + docs (o Number no Strength do Vortex) — ver §6.3 |
| base / merge-base | `main` @ `1ad60a1ce` — **0** commits do `main` por trazer; `--ff-only` possível |
| commits | **18** (2026-10-02 → 03) · `36` ficheiros (+3 621 / −685) |

Commits por grupo:

| grupo | commits |
|---|---|
| §9.8 as listas das células | `5febba023` · `22e7d069f` (gate da cena que muda + mutação 14/14) · `0d94bdfbc` · `c3c2c7f33` · `f213e1998` |
| §9.9 letras e tracejado no ecrã | `7a58e7aaf` · `3abf99b77` · `4aa06e738` · `cefdb9667` · `042327a6a` · `19a359f2c` · `55d71872f` |
| §9.10–§9.11 variante enxuta e ablação | `8cb0ab9e1` · `a9fbbf385` · `ff6cc0e57` |
| §9.12 o buffer de acumulação | `78d95690c` (plano + kill-criterion) · `cd3ec059e` (código) · `311413f1e` (resultado) |

## §1 — O QUE A LINHA ENTREGA

**Um assunto: o passe das formas na placa chega ao proxy de telemóvel (iGPU) sem perder nitidez** — a
continuação do item aberto do handoff de 01/10.

| peça | o que é | doc 121 |
|---|---|---|
| listas das células | cada célula de `32 × 1 px` guardava as arestas que a cruzam (ladrilho do Vello) — **SUBSTITUÍDAS hoje** pelo buffer de acumulação, ver abaixo | §9.8 |
| letras e tracejado | glifos com gate de pixel; o traço TRACEJADO sob escala não uniforme vai à placa (censo `19` das `23`); ajuste da emenda no ECRÃ nas duas rotas (`ajusta_no_ecra` / `ajuste_do_tracejado`, folga `1e-4`) | §9.9 |
| variante enxuta | o ramo do tracejado inline dobrara os registos de TODA a cena na iGPU (fragmento `56 → 128` VGPRs); `override TRACEJADO`, duas variantes, escolhida pelo eixo carregado | §9.10 |
| ablação do desenho | `84 %` do desenho das esticadas era o laço das listas; recusa medida da origem `flat` | §9.11 |
| **buffer de acumulação** | **cada aresta deposita, em ponto fixo `2¹⁶`, a diferença da `contribuicao` nos pixels que CRUZA** (recortada à fileira); o prefixo recomeça por célula de `32 px`; um fio por pixel (`cs_varre`, grupos de `64` = duas células, prefixo segmentado em memória de grupo) soma o fundo, aplica as regras (par-ímpar / não-nulo; traço = marcas + contorno) e grava `pack2x16unorm`; **o fragmento faz UMA leitura** | §9.12 |

**Passes de cálculo (hoje):** `cs_zera` (pixel) · `cs_deposita` (aresta) · `cs_fundo` (fileira) ·
`cs_varre` (pixel), despachos indirectos (o `despacho` ganhou `[6,9)` por pixel, `36 B`). Buffers por
célula: registo `4` palavras (3 fundos + regra), acumula `96`, cobertura `32` (**528 B/célula**).

**Saiu:** `listas`, `lista_total`, `SEM_LISTA`, `cs_conta_listas` / `cs_lugar_das_listas` /
`cs_escreve_listas` e o 2.º atraso da capacidade. A única capacidade é a das **células** (em CÉLULAS, não
palavras); a cópia que não cabe vai INTEIRA pelo caminho de sempre.

**Kill-criterion** (escrito ANTES, doc §9.11/§9.12) — **PASSOU à 1.ª tentativa**. Soma dos passes do
perfilador, `PH2D_FLUID_PROFILE=1`, binário da cura `8cb0ab9e1` intercalado na mesma janela, 2 corridas:

| arranjo | iGPU antes → depois (ms) | critério |
|---|---|---|
| esticadas | `1,50 / 1,52 → 0,88 / 0,98` (a 2.ª com média de 5 min a `8,5`) | `≤ 1,0` |
| conformes | `0,76 → 0,59` | `≤ 0,76` |
| densas | `1,50 → 1,00` | `≤ 1,50` |
| RTX (nenhum `> +10 %`) | esticadas `0,25 / 0,26 → 0,22 / 0,22` · conformes `0,10 → 0,09 / 0,10` · densas `0,11 → 0,10` | ✅ |

Decomposição iGPU das esticadas: desenho `0,83 → 0,14`; células `0,43 → 0,50`. Relógio de parede da sonda
iGPU (placa · Vello): esticadas `0,98 · 0,94` (antes `1,64 · 1,13`) · conformes `0,70 · 0,87` · densas
`1,13 · 2,65`. Registos (`registos_dos_shaders.sh`): fragmento `56` VGPRs / `18` ondas (igual — o caminho de
sempre manda), código `16 620 → 15 336 B`; kernels novos `≤ 32` VGPRs, scratch `0`. Memória: sonda das
esticadas `10 362` células (~`5,5 MB`); gate das estrelas sobrepostas a `512²` `77 502` células (~`41 MB`,
buffer arredondado a potência de 2).

## §2 — SUPERFÍCIE DE COLISÃO

Medida contra o merge-base `1ad60a1ce` (= `main`); `git diff --stat 1ad60a1ce..HEAD`. Reconfira com
`/home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh` (caminho ABSOLUTO) no momento de integrar.

```
shells/desktop        delta 0 linhas (NÃO tocada)
SCHEMAS / REGISTROS   nada tocado (nenhum PROJECT_SCHEMA, tripla, registo de componentes, ADR, cena de smoke nova)
CONTRATOS (§6)        Nodes=2/1/8 e Tool=12/5/1/4 intocados
Cargo.lock            nenhum pacote novo
```

### §2.1 FOUNDATIONAL tocado — só aditivo

| crate / ficheiro | mudança | aditiva? |
|---|---|---|
| `crates/ph2d-vector/src/lib.rs` (`+2`) | re-export `ParamCurve`, `ParamCurveNearest`, `PathSeg` do `kurbo` (§9.9) | ✅ — `architecture_vector_contract_surface` verde no nextest |
| `crates/ph2d-vec-render/src/stroke_uniform.rs` (`+129`) | o tracejado ajusta-se ao contorno do ECRÃ sob afim NÃO conforme (`ajusta_no_ecra`, `FOLGA_DO_AJUSTE = 1e-4`) — **muda de lei, também no vetor de documento**; nenhuma outra linha toca o ficheiro (conferido em 02/10) | ⚠️ comportamento — ver o [handoff 02/10 §3.7](HANDOFF_CONTINUACAO_line_motion_value_2026-10-02.md) |

### §2.2 O que um merge pode partir

- **Portas públicas renomeadas em `ShapePass`** (`ph2d-shape-gpu`, crate NOSSA, **não congelada**):
  `listas_do_ultimo_quadro` → `celulas_do_ultimo_quadro`, `limita_as_listas` → `limita_as_celulas`. Espelho em
  `ph2d-app-motion`: `PlacaDeFormas::celulas_do_ultimo_quadro` (`cfg(test)`). Um chamador novo de outra
  linha com o nome velho **não compila** (falha alta). Medido: só `ph2d-shape-gpu` e `ph2d-app-motion` usam.
- **Gate renomeado:** `as_fileiras_que_nao_cabem_nas_listas_desenham_o_mesmo` →
  `as_copias_que_nao_cabem_nas_celulas_desenham_o_mesmo` (controlo: o tecto mordeu **E** `0 < cópias nas
  células < n`). Quem citar o nome velho em doc/lista de nextest reconta.
- `ccopias[3ii+2].z` passa a levar a REGRA da cópia (era `0`; um só leitor novo: `cs_fundo`).
- `QUADROS_DO_PRODUTO` `5 → 3` (`paridade_com_o_vello.rs`) e `QUADROS` `6 → 4` (`contorno_calculado.rs`):
  sem o 2.º atraso a capacidade chega um quadro antes. Gates da crate; nenhuma outra linha lê.
- O arnês [`mutacao_as_listas_das_celulas_2026-10-02.py`](../ferramentas/mutacao_as_listas_das_celulas_2026-10-02.py)
  está **APOSENTADO** (as âncoras saíram com o código); o sucessor é o do §5.

## §3 — ⚠️ O que uma leitura rápida do diff entende ao contrário

1. **O diff das listas (`5febba023`…) é história: o código final não tem listas.** Leia o estado final, não
   commit a commit — `cd3ec059e` apaga o que `5febba023` pôs. O item 1 do §3 de 02/10 (`QUADROS` 3→5)
   deixou de valer (ver §2.2).
2. **Cópia que não cabe nas células vai INTEIRA pelo caminho de sempre**, nunca um contorno truncado — no
   1.º quadro de uma cena nova a mistura dos dois caminhos é normal e desenha a mesma imagem (gate).
3. **Só cópias com blocos ganham `tela`**: células de cópia recusada nunca são lidas (auditoria de correção).
4. **Acumuladores e fundos são apagados por quadro** e a cobertura é reescrita em toda célula em uso — é o
   que torna o resultado independente da ordem dos atómicos (ponto fixo) e da cena anterior; as mutações
   A6/A7 morrem no gate da cena que muda.
5. **Na iGPU as densas pagam mais nas células** (`0,51 → 0,78 ms`): o `cs_zera` toca todas as células
   do quadro. Nomeado, não curado — ver §6 (variante esparsa). O total das densas desce (`1,50 → 1,00`).
6. **A variante completa (cena com tracejado) continua a `128` VGPRs** — só a enxuta (o caso comum) está a
   `56`. Item aberto (§6).

## §4 — ⛔ Premissas do briefing que a medição derrubou

1. *«Guardas equivalentes para as fileiras que não cabem.»* Com a acumulação não há fileira a meio: a guarda
   virou por **CÓPIA** (gate `as_copias_que_nao_cabem_nas_celulas_desenham_o_mesmo`, com controlo).
2. *«Re-correr a mutação 14/14 das listas.»* Não se pode: foi aposentada com o código; a sucessora é a
   `17/17` do §5.
3. *«Uma 2.ª tentativa do kill-criterion.»* Não foi precisa: passou à 1.ª (a densa; a variante esparsa
   ficou como alavanca medida, não feita).
4. (§9.10) *«O `EixoItem` `56 → 72 B` é o que pesa»* — era o ramo inline do tracejado.

**Recusas MEDIDAS** (doc 121; não reconstruir): fio-por-FILEIRA (`0,57 → 1,52`) · células de `16`/`64 px` ·
origem das células `flat` (`0,81 → 0,80`) · «cobertura 8 px por fio» (§9.7).

## §5 — A PROVA DE FECHO (corrida nesta árvore)

| portão | resultado |
|---|---|
| `nextest-impacted.sh` sobre o diff acumulado | ✅ **17 042 / 17 042** (exit 0) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | ✅ |
| `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -D warnings` | ✅ |
| `shells/desktop` `file_loc_caps` | ✅ 4/4 |
| `cargo machete` · `check-standalone-optional.sh` · `check-workflow-packages.sh` | ✅ limpos |
| GPU (RTX, `#[ignore]`) `ph2d-shape-gpu --test it` | ✅ **10/10** (todos os quadros alfa `≤ 1`, `0` px `> 1` contra o eixo; tracejado vs Vello no mesmo `71` do §9.9) |
| `ph2d-app-motion motion_shape_placa::gpu_tests` | ✅ **5/5** + a sonda |
| `ph2d-gpu-cook formas` | ✅ **2/2** |
| mutação [`o_buffer_de_acumulacao`](../ferramentas/mutacao_o_buffer_de_acumulacao_2026-10-03.py) | ✅ **17/17** |
| mutação [`o_tracejado_no_ecra`](../ferramentas/mutacao_o_tracejado_no_ecra_2026-10-02.py) | ✅ **21/21** re-corrida (corrida limpa: 16 verdes) |
| mutação [`a_variante_enxuta`](../ferramentas/mutacao_a_variante_enxuta_2026-10-03.py) | pré-voo `9/9` (não re-corrida; a cura não mudou). A V9 sobrevive por desenho — só `registos_dos_shaders.sh` a vê |

**Auditoria, 2 lentes.** Correção: só cópias com blocos ganham `tela` ⇒ células de cópia recusada nunca são
lidas. Determinismo: acumuladores e fundos apagados por quadro (A6/A7 morrem no gate da cena que muda) e
cobertura reescrita em toda célula em uso.

**O que só o `ship.sh` / a integração pega:** nada conhecido. A árvore combinada com as outras linhas é do
integrador — releia a ORDEM das fases do quadro só se outra linha mexer em `fase_vector_bands` /
`present_chrome` (`shells/desktop` não é tocada por esta linha, logo o atrito é nulo daqui).

## §6 — ⏳ O QUE FICA ABERTO

| item | o endereço |
|---|---|
| ✅ memória no app MEDIDA (§6.1) e CORTADA (§6.2) — `66 → 43 MB` na `=127` densa | o resto é a variante ESPARSA (abaixo) |
| **variante ESPARSA** (só as células tocadas; tira o `cs_zera`) | ⚠️ nas densas quase toda célula é tocada (estrelas de `~14 px`): MEÇA a fracção de células tocadas antes de a construir; a alavanca é das formas GRANDES |
| a variante COMPLETA (com tracejado) a `128` VGPRs | encolher o ramo do tracejado; item próprio |
| a mordida do traço rente depois de uma quina | divergência DECLARADA (§9.9), sem acção |
| `M6` / `S6` / `S8` (§9.4–§9.5) e o `fx.glow` que lê o `pump` anterior | nomeados desde 01/10, sem mudança |
| `fk.rs` duplicado em seis crates (bug #11) | wave própria |

### §6.1 — Depois deste handoff (03/10, mesma linha, após o smoke aprovado)

- ⛔→✅ **`87e605534` — as células só com a capacidade MEDIDA.** A W5 no app (`mede_formas_na_placa.sh`,
  novo botão `FORMAS`) achou `132 MB` na `=127` densa e `264 MB` na escada de `32 768`, pedidos pelo
  palpite de fábrica `16` células por cópia que entrava num `max` e nunca saía. Saiu
  `CELULAS_POR_COPIA_INICIAL`; os 2 primeiros quadros de uma cena nova vão pelo caminho de sempre (a mesma
  imagem). Re-medido: escadas `0` células; `=127` densa `107 520` (`66 MB`); iGPU `16,6 ms` com placa
  contra `20,8` sem (as listas davam `17,6`), CPU `3,1` contra `8,2 ms`. Relato novo `[formas] celulas:`
  sob `PH2D_FLUID_PROFILE=1`. Gates re-corridos: shape-gpu `10/10`, produto `5/5` + sonda, clippy das
  duas crates, `cargo-test-narrow` shape-gpu `18` verdes. A mutação `17/17` foi ANTES desta mudança (ela
  só toca a capacidade em `garante`, fora das âncoras).
- ⛔ **`9b133f887` — recusa medida: estreitar a célula** (`16` empata, `8` perde nas esticadas). Fica `32`.
  Tabela no doc 121 §9.12.

### §6.2 — A memória das células: `66 → 43 MB` (03/10, mesma linha)

- ✅ **`9d1058a40` — a cobertura NO LUGAR do 1.º acumulador e a capacidade ao OITAVO do degrau**
  ([doc 121 §9.12](../121_as_formas_na_placa.md), o fim). O `cs_varre` grava o `pack2x16unorm` na palavra
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
  `2/2` · [mutação](../ferramentas/mutacao_o_buffer_de_acumulacao_2026-10-03.py) **`19/19`** (A13
  re-ancorada; A18 e A19 novas; pré-voo `19/19`, corrida limpa `10` verdes, nenhuma por shader inválido)
  · [tracejado](../ferramentas/mutacao_o_tracejado_no_ecra_2026-10-02.py) **`21/21`** (corrida limpa `16` verdes) · nextest-impacted `17 045/17 045` · `cargo check --workspace
  --all-targets` com `-D warnings` · clippy das três crates · machete · os dois censos.
- Foundational: nenhum. Ids/consts novos: nenhum (sai o `COBERTURA`; a ligação `9` do grupo `2` fica livre).

### §6.3 — Os dois reports do smoke de 03/10: o Number no Strength do Vortex (mesma linha)

Reports do Enio: *«ligar um Number ao Strength de um Vortex: o fio aparece, mas o valor não tem efeito
(o Vortex continua com o valor do cartão)»* e *«Number não aceita valores negativos»*. Mecanismo, as
hipóteses que caíram e a conclusão intermédia errada: [BUGS #12](../BUGS_motion_nodes.md).

| commit | o quê |
|---|---|
| `a6e067f45` | a row LIGADA do cartão lê o número do fio pela porta do painel (`params_stream::driven_value`); doc do `CardParam::value` corrigido; sondas + gate CPU; paridade CPU↔placa de um Number no Strength do Vortex dentro da sim (`gpu_cpu_parity_driven.rs`, ao nível do `GpuCook`) |
| `1c8f25d1e` | `strength` `0..40 → ±40` no Vortex, Attractor, Curl e Wind; leis «negativo = o outro sentido ao bit»; figura `params_sim.html` regenerada |
| `757c5356a` | a caixa de número do cartão aplica a faixa digitável (`safe_clamp_f64`, nova, aditiva, em `ph2d-editor-core::math`); `MockPanelHost::type_keys_then_enter` (testkit, aditivo) |
| `64120d51a` | cena `=128` + gates do roteiro; BUGS #12 (1.ª redacção) |
| `2062db7d8` | ⭐ **o ramo HÍBRIDO do `cook_gpu` entrega os valores do fio**; gates por ramo na placa; cena `=128` re-arrumada (as fotos apanharam o Number e o Vortex fora de vista) |
| (o commit deste handoff) | BUGS #12 reescrito; esta §6.3 e o smoke §7.1 |

- ⛔⛔ **O defeito do efeito era o ramo HÍBRIDO** (`motion_bridge_gpu.rs`, `GpuRoute::Hybrid`): ele nunca
  chamava `set_driven`, e o `strength` caía no override. A `=127` do dono (formas + galáxia) vai por
  ele; medido ANTES: Number `1`, `30` e sem fio dão o mesmo `P` ao bit; DEPOIS: `p[0]` com Number 30 =
  `[-17.4267, 9.4171]` na placa e na CPU (a `1 ULP`). O ramo `FullyGpu` sempre esteve certo, e a minha
  cena de reprodução simples ia por ele — por isso a 1.ª redacção do BUGS #12 declarou o motor
  inocente. ⚠️ **Os valores derivam-se ANTES do `handed`** (o empréstimo dos fluxos da fronteira segura a
  bomba que os coze), e o `rewind_for` só corre quando há entrega (era assim antes).
- **O cartão mentia nos dois ramos:** desenhava o override (`2`) com o fio a pôr `30`. O gate
  `the_card_shows_and_drags_the_same_numbers_the_panel_does` era cego por POPULAÇÃO (só nós soltos).
- **Negativos:** a digitação já passava (medido pelo teclado real: `-1` chega, solto e ligado); o que
  parava no zero era o ARRASTO do Number ligado, que veste a faixa do destino (`FromWire`). Densidade e
  arrasto ficam `≥ 0` (controlo no gate).
- ⛔ **Achado vizinho, pela mutação M10 (que sobreviveu por ser equivalente):** a caixa do cartão não
  aplicava faixa digitável NENHUMA (`999999` num tecto de `100` entrava) e o doc dela afirmava o
  contrário. Curado no `commit`.
- **Gates novos, cada um vermelho antes (ou medido vermelho pela sonda), com controlo:**
  `the_hybrid_route_reads_the_wire_like_the_cpu` (placa; *«mudou 0»* antes) ·
  `the_fully_gpu_route_reads_the_wire_like_the_cpu` (placa; fixa o ramo irmão) ·
  `a_driven_row_on_the_card_shows_the_number_the_wire_puts_in` · `a_number_wired_to_a_field_strength_drags_below_zero`
  · `a_negative_strength_is_the_other_turn_to_the_bit` (vortex) · `a_negative_strength_is_repel_to_the_bit`
  (attractor) · `a_negative_number_typed_over_the_seed_reaches_the_document` · `a_typed_number_outside_the_typed_range_stops_at_the_limit`
  · `a_number_on_the_vortex_strength_moves_the_sim_like_the_card_would` · os dois da cena `=128` ·
  `a_number_driving_the_vortex_strength_inside_the_sim_agrees_on_both_routes` (placa, `GpuCook`).
- **Mutação `12/13` + 1 equivalente:** M1–M9 (cartão, as 4 faixas, o controlo do arrasto, as 2 leis do
  sinal, o `-` no teclado), M11 (faixa digitável), M12 (`set_driven` do híbrido) e M13 (`set_driven` do
  `FullyGpu`) sangram; M10 (estreitar a faixa do `store` no `open_box`) é EQUIVALENTE — nada a lê ao
  comitar (o `NumberInput` só clampa ligado a slider).
- **Gate de fecho:** nextest-impacted `17 540/17 540` (inclui `arch_safe_clamp_only`, `file_loc_caps`,
  `architecture_workspace_file_loc_cap`, `the_shell_only_shrinks`) · clippy `--all-targets --all-features
  -D warnings` nas 9 crates · `fmt --check` · censos `12/12` (`127` testes) · machete · 2 lentes de
  auditoria (consumidores do `CardParam::value`: rows ligadas estão fora de todo gesto; faixas com sinal:
  kernels e CPU multiplicam o `strength` sem `max(0)`/`sqrt`) — sem achados além da figura. Depois da
  cura do híbrido (só `ph2d-app-motion`): nextest da crate `1 344/1 344`, clippy e fmt limpos; gates de
  placa (`--ignored`): os 2 por ramo + os 2 de `gpu_cpu_parity_driven` verdes.
- **Foundational tocado (aditivo):** `ph2d-editor-core::math::safe_clamp_f64` (função nova) ·
  `ph2d-ui-testkit` (método novo). Contratos congelados: nenhum (`NodeManifest` dos nós de força
  intacto — só `ParamUiHint`, registry de UI). Cenas: `MAX_DEMO_LEVEL 127 → 128`.
- ⚠️ **Para quem lê o diff:** a figura `params_sim.html` regenerada traz também atrasos ANTIGOS dela
  (Pivot Offset, Bone/Rope Segment, Bounciness `0 a 1`) — é o que o código já dizia.
- Smoke ao dono: §7.1.

## §7 — OS SMOKES

✅ **Smoke do dono APROVADO em 03/10** (a `=127` pelo comando abaixo); e de novo depois da §6.2, a memória das células).

Do dono (passos; binário já compilado — `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`
corrido 2× DEPOIS do `f2b830bcf` e do `rm -rf target/*/incremental`, a 2.ª saída: `Finished smoke profile [optimized] target(s) in 0.26s`, zero `Compiling`; a `=127` re-fotografada depois da §6.2):

1. No terminal:
   ```
   cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=127 cargo run -p ph2d-host-desktop --profile smoke
   ```
2. Espere a janela abrir; não precisa clicar em nada: a cena já é a das estrelas esticadas com contorno azul.
3. Tem de acontecer: as estrelas desenham com o contorno azul INTEIRO, iguais às de antes, sem riscos, sem
   buracos, nenhum pixel a piscar; a barra de baixo mostra cerca de 60 fps (fotografada hoje a 1930×1040 com
   `docs/Components/ferramentas/fotografa_cena.sh`).
4. Deu errado se: o contorno tiver riscos ou falhas, algum ponto piscar, ou a barra cair muito abaixo de 60.
   Para comparar com o desenho antigo, repita com `PH2D_FORMAS_NA_PLACA=0` antes de `cargo`.

Técnico — variante densa para perf (release), e a mesma sem a placa para comparar:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=127 PH2D_TRACO_ESTICADO_DENSO=1 cargo run -p ph2d-host-desktop --release
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=127 PH2D_TRACO_ESTICADO_DENSO=1 PH2D_FORMAS_NA_PLACA=0 cargo run -p ph2d-host-desktop --release
```

Olhar: as estrelas iguais às de antes e, no proxy de telemóvel, o quadro não pior. Diagnóstico:
`PH2D_MOTION_ROUTE_LOG=1` (rota por quadro), `PH2D_FLUID_PROFILE=1` (relógio por passe). Gates GPU (da raiz
do worktree, com adaptador): `cargo test -p ph2d-shape-gpu --test it -- --ignored` ·
`cargo test -p ph2d-app-motion --lib motion_shape_placa::gpu_tests -- --ignored` ·
`cargo test -p ph2d-gpu-cook formas -- --ignored`.

### §7.1 — Smoke da §6.3 (a cena `=128`: o Number no Strength do Vortex)

✅ **Smoke do dono APROVADO em 03/10.**

1. No terminal:
   ```
   cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=128 cargo run -p ph2d-host-desktop --profile smoke
   ```
2. Espere a janela abrir: estrelas amarelas a girar como uma galáxia. No grafo (em baixo), a fila de
   BAIXO tem `Number · Falloff · Vortex · Attractor · Curl Noise`. Ponha o rato sobre o `Vortex` e role a
   roda para aproximar até ler as linhas dele (fotografada a 1930×1040: os cartões abrem como pílulas).
3. Tem de acontecer: a linha `Strength` do Vortex mostra `4` (o número do fio), não o `2` escrito nele;
   arrastar o Number para ~10 faz a galáxia girar mais depressa (e abrir-se devagar); em `0` ela pára;
   abaixo de zero (~−4) gira AO CONTRÁRIO; clicar no número do Number, escrever `-2` e Enter deixa `-2`.
4. Deu errado se: o `Strength` do Vortex ficar em `2`; a velocidade não mudar; o arrasto parar no `0`;
   ou o `-2` não entrar. Bissecção: repetir com `PH2D_GPU_COOK=0` antes do comando (a CPU).

## §8 — A UMA LINHA proposta para o `CLAUDE.md` §5 (o integrador aplica; ≤ 700 B)

Troca só o link do handoff na entrada **Motion Nodes**: `Último: [handoff 03/10](docs/Motion%20Nodes/handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-10-03.md)`. A frase do módulo não muda de natureza.

## §9 — PERFIL DO LOOP DO AGENTE (`bash scripts/agent-loop-profile.sh`, verbatim, no fecho da §6.2)

```
PERFIL DO LOOP DO AGENTE — 20 sessao(oes) mais recentes
  ✗ paralelismo de ferramenta              1.14/passo   alvo: >= 1,5  (10% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                178   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                491 : 175   alvo: <= 1,0  razao 2.8x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  37%   alvo: >= 80%  (715 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         350 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```
