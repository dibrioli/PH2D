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
| HEAD | `311413f1e` + este handoff · ⚠️ e, DEPOIS dele (03/10, após o smoke aprovado): `87e605534` (a capacidade só MEDIDA das células, código) e `9b133f887` (recusa da largura, doc) — ver §6.1; e `9d1058a40` · `650128529` · os docs (a memória das células `66 → 43 MB`) — ver §6.2; e `a6e067f45` · `1c8f25d1e` · `757c5356a` · `64120d51a` · `2062db7d8` + docs (o Number no Strength do Vortex) — ver §6.3; e `82f9623e2` (o grafo, §6.4); e `133e306af` · `b723b02d1` · `40a2cbc1a` · `018218976` · `62929e077` + o commit da §6.5 (os itens 3 e 4) — ver §6.5; e o bloco de 04/10 (`4f0dcce34` … o commit da §6.6) — ver §6.6 |
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
| ✅ memória no app MEDIDA (§6.1) e CORTADA (§6.2) — `66 → 43 MB` na `=127` densa | fechado |
| ⛔ **variante ESPARSA** — RECUSADA com a tabela (§6.5) | `60 %` das células tocadas nas densas; líquido `≤ 10 %` no melhor caso, conformes piores, `+4 B`/célula |
| ⛔ o `cs_varre` — D1 (as famílias presentes) e D2 (`4` px por fio) RECUSADOS (§6.6) | o prefixo vale no MÁXIMO `0,10` ms nas densas da iGPU (ablação `V0`); o resto é memória. Sobra o prefixo por SUBGRUPO, tecto `0,10` ms, não construído |
| ✅ a variante COMPLETA (com tracejado) — no regime corre a ENXUTA (§6.5) | ⏳ as tracejadas GRANDES esticadas ainda perdem para o Vello na sonda da iGPU (parede `1,58`–`1,76` contra `0,90`–`1,15` ms, com o Vello a receber o tracejado já cortado); no produto `60 fps`, e a alternativa é `5×` mais lenta. ⛔ A topologia por troço (um GRUPO por cópia) foi RECUSADA (§6.6: RTX `−49 %`, iGPU `+6 %`); a pergunta seguinte é a emissão dupla × a fase em série, por ablação |
| a mordida do traço rente depois de uma quina | divergência DECLARADA (§9.9), sem acção |
| ✅ `M6` / `S6` / `S8` e o `fx.glow` que lê o `pump` anterior | fechados (§6.6): `M6` sem código desde §9.8; `S6`/`S8` equivalentes, com número e gate de CPU; o halo pela rota do quadro — e o halo que sumia em TODO quadro por faixas |
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

### §6.4 — O grafo abria com a parte de cima escondida (03/10, mesma linha, depois do smoke da §6.3)

- **`82f9623e2`** — mecanismo e medição: [BUGS #13](../BUGS_motion_nodes.md). O `fit` do painel do grafo
  mede pelo regime que desenha (pílula ou cartão), re-enquadra uma vista automática intocada quando o
  painel muda de tamanho (o quadro 0 tem `1918` px, o 1 tem `1310`), e alinha pelo topo o que não cabe
  ao piso de leitura. Estado novo no painel: `MotionGraphPanelState::enquadrado_em` (privado do crate).
  A cadeia de forças partilhada da `=126`/`=127` passa a `220` de passo (pílulas sobrepostas a `160`).
- ⚠️ **Para quem lê o diff:** no regime de cartões ABERTOS o `fit` é o de antes ao bit (a mesma
  geometria); muda só quando o zoom cai no regime das pílulas, que num painel de `205` px é quase sempre.
- Gates: `paint_fit_tests.rs` (4, pela costura) · 4 mutações a sangrar · nextest das duas crates
  `1 582` + `14` de integração do painel · clippy limpo. Fotos `=127` e `=128` a 1930×1040: a fila de cima
  inteira; a `=128` cabe e centra.
- ✅ **Feito ANTES de integrar (ordem do dono, 03/10):** os itens 3 e 4 do §6 — ver §6.5.

### §6.5 — Os itens 3 e 4 do §6: a variante completa encolhida e a esparsa recusada (03–04/10)

Mecanismo, tabelas, prova do tecto e recusa: [doc 121 §9.13](../121_as_formas_na_placa.md).

| commit | o quê |
|---|---|
| `133e306af` | plano + kill-criteria ANTES de construir; sonda `PH2D_SONDA_TRACEJADO=1`; instrumento `celulas_tocadas_do_ultimo_quadro` (o `acumula` ganha `COPY_SRC`) |
| `b723b02d1` | **(3a)** a placa escolhe a variante do desenho por quadro (dois `draw_indirect`); **(3b)** o tecto da contagem sem o ajuste e o ajuste numa volta; gate novo; arnês de mutação novo; V7 da variante enxuta re-ancorada |
| `40a2cbc1a` · `018218976` | doc: o resultado medido, a recusa da esparsa, a cena do report no app |
| `62929e077` | os instrumentos de leitura do contorno em `contorno_sondas.rs` (o `contorno.rs` passara o tecto de `700` LOC: `741 → 627`) |

- **(3a)** Numa cena com tracejado o `cs_soma` põe as `n` cópias nos argumentos da ENXUTA (palavras
  `[9, 13)` do buffer do `despacho`) e `0` na COMPLETA (`[13, 17)`); o `cs_escreve` troca-os quando uma
  cópia tracejada e VISÍVEL fica sem células (pixel a pixel). O passe grava os dois `draw_indirect`; uma
  só chamada desenha tudo, a ordem da mistura é a de sempre. No regime corre o fragmento da enxuta (iGPU
  `56` VGPRs · `18` ondas contra `128` · `8`); os dois primeiros quadros de uma cena nova, a completa.
- **(3b)** O tecto de arestas por troço tracejado é `⌈len/per + 1/2⌉ + 2` sem o ajuste (prova no doc: o
  ajuste nunca encurta o período mais que meia peça por troço); o `cs_conta` da completa `48 · 20 → 40 ·
  24` VGPRs. O ajuste soma o arco numa volta — as imagens das `7` famílias iguais BYTE A BYTE.
- **Medido (iGPU, sonda intercalada, os 4 binários na mesma janela, soma dos passes, tracejado):**
  esticadas `1,33 → 1,17` · conformes `0,61 → 0,55` · densas `1,45 → 1,32`; RTX esticadas `0,43 → 0,37`,
  o resto igual. O desenho das tracejadas IGUALA o das contínuas (`0,15` · `0,15` · `0,11`). Sem tracejado,
  antes = depois nas duas placas. **No app** (`=127` densa tracejada, rota `HIBRIDO`): `60 fps` nas duas
  placas com a placa de formas, `12 fps` sem ela; células `43 MB`, iguais.
- **(4) RECUSADA:** células tocadas `60 %` (densas e esticadas), `97 %` (conformes grandes) — ⛔ a premissa
  «nas densas quase toda célula é tocada» caiu. Teto `t_zera + (1−f)·t_varre` medido por ablação:
  `19 %` / `15 %` / `9 %`; os custos fixos (marcas e lista `+0,04`–`0,05`, fragmento `+0,01`–`0,02`)
  deixam o MELHOR caso a `≤ 10 %` nas densas, abaixo nas esticadas, PIOR nas conformes, e a lista custa
  `4 B` por célula (`43 → 44 MB`). Tabela no doc 121 §9.13.

**⚠️ Para quem lê o diff (§3 deste handoff vale):**
- O `despacho_rw` do WGSL passou a `array<atomic<u32>>` (os dois desenhos são escritos por muitos fios);
  o `despacha` usa `atomicStore`. O buffer `36 → 68 B`, com `COPY_SRC` (o instrumento `copias_por_variante`).
- ⛔ **Os relógios POR passe mentem na fronteira:** o carimbo de início de um passe sai antes da barreira, e
  a cauda do anterior cai na conta dele (nas densas a «escrita» caiu `0,31 → 0,17` com uma mudança só na
  contagem, com o `cs_escreve` byte a byte igual). A régua é a SOMA dos passes.
- O `plano_de` passou a usar `caixa_das_celulas` + `fora_do_ecra` (a MESMA folga de um pixel, uma porta
  partilhada com o `pede_a_completa`).

**Gates:** `ph2d-shape-gpu` GPU **`11/11`** (os `10` + `a_placa_escolhe_a_variante_completa_so_quando_um_tracejado_vai_pixel_a_pixel`,
com controlo no 1.º quadro e na metade das células, imagem igual) · produto `motion_shape_placa::gpu_tests` **`5/5`** · `ph2d-gpu-cook` formas `2/2` · mutações: **tracejado `21/21`**, **acumulação
`19/19`**, **a variante da placa `6/8`** — W6/W7 (o tecto sem a meia peça, e com METADE das peças)
SOBREVIVEM e é medido porquê: o orçamento por peça é folgado, e um tecto curto cai no pixel a pixel pela
completa com a mesma imagem · nextest-impacted `17 543/17 544` (o vermelho era `architecture_workspace_file_loc_cap`,
curado por `62929e077` e re-corrido verde) · `cargo check --workspace --all-targets` com `CARGO_BUILD_WARNINGS=deny` ·
clippy `--all-targets --all-features -D warnings` das duas crates · `fmt --check` · censos `12/12` (`127`) ·
machete · standalone-optional · workflow-packages · `cargo-test-narrow` shape-gpu `21`.

**Foundational:** nenhum. **Ids/consts novos:** `DESENHO_ENXUTA = 36` · `DESENHO_COMPLETO = 52` · `DESPACHO = 68`
(bytes, `contorno.rs`); `DESENHO_ENXUTA = 9u` · `DESENHO_COMPLETO = 13u` (palavras, `contorno.wgsl`); `pub fn`
novas em `ShapePass`: `copias_por_variante`, `celulas_tocadas_do_ultimo_quadro` (instrumentos). Shell: `0` linhas.
Contratos congelados: nenhum.

**Smoke:** §7.2.

### §6.6 — O bloco de 04/10 (`CLAUDE.md` §0.10): o tracejado por troço, `M6`/`S6`/`S8`, o brilho, o `cs_varre`

Plano e kill-criteria ANTES de construir, tabelas e recusas: [doc 121 §9.14](../121_as_formas_na_placa.md).
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
- **Mutação `9/9`** ([arnês](../ferramentas/mutacao_o_bloco_do_9_14_2026-10-04.py)): R1 · B1–B3 · H1–H5 — a
  B3 sobreviveu à 1.ª corrida (fixtura com halo forte de mais) e sangra depois da cura da fixtura.
- Perfil do loop (`agent-loop-profile.sh`, 20 sessões): paralelismo `1,14` · `test:check` `2,2:1` · Edit `35 %`
  · contexto relido `492 mil` · respostas por sessão `233` ✓ · contexto inicial `63 mil` ✓.

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

### §7.2 — Smoke da §6.5 (a `=127` com o contorno contínuo e tracejado)

✅ **Smoke do dono APROVADO em 04/10.** ⚠️ Na mesma conversa o dono pôs a regra `CLAUDE.md` §0.10 (problemas
equivalentes num único bloco): o `CLAUDE.md` desta linha traz esse item a mais — o integrador mantém-no.

Binário já compilado: `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` corrido 2× depois de
`ac0201c99` e do `rm -rf target/*/incremental`; a 2.ª saída: `Finished smoke profile [optimized] target(s) in 0.20s`,
zero `Compiling`. As duas cenas fotografadas na tela virtual (`fotografa_cena.sh`, rota `HIBRIDO`, `59`–`60 fps`).

1. No terminal:
   ```
   cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=127 PH2D_TRACO_ESTICADO_TRACEJADO=1 cargo run -p ph2d-host-desktop --profile smoke
   ```
2. Espere a janela abrir; não precisa clicar em nada: são as estrelas amarelas esticadas, agora com o contorno
   azul em TRAÇOS (fotografada a 1930×1040: `1 024` estrelas pela placa).
3. Tem de acontecer: cada estrela com o contorno azul partido em traços certinhos, sem riscos nem buracos,
   nada a piscar; a barra de baixo mostra cerca de 60 fps. Depois feche e repita SEM o
   `PH2D_TRACO_ESTICADO_TRACEJADO=1`: o contorno volta a ser inteiro, também a ~60 fps.
4. Deu errado se: os traços faltarem, ficarem tortos ou piscarem, o contorno contínuo aparecer partido, ou a
   barra cair muito abaixo de 60. Para comparar com o desenho antigo, ponha `PH2D_FORMAS_NA_PLACA=0`
   antes de `cargo` (fica bem mais lento na versão tracejada — é o esperado).

### §7.3 — Smoke da §6.6 (a `=70`: o brilho com a forma, agora pela placa)

Binário já compilado: `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` corrido 2×
depois do último commit de código e do `rm -rf target/*/incremental`; a 2.ª saída: `Finished smoke profile
[optimized] target(s) in 0.20s`, zero `Compiling`. Fotografada com esse binário a 1930×2000 na tela virtual
(`device: HIBRIDO`, formas «do dispositivo», o halo verde à volta da forma); com `PH2D_FORMAS_NA_PLACA=0` o
halo é o mesmo de antes.

1. No terminal:
   ```
   cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=70 cargo run -p ph2d-host-desktop --profile smoke
   ```
2. Espere a janela abrir; não precisa clicar em nada. É a cena dos efeitos de luz: em cima, as sombras; no
   meio, um halo esticado; em baixo à esquerda, uma FORMA branca grande (fotografada a 1930×2000: ela
   aparece no canto de baixo da vista — numa janela mais baixa, role a vista para baixo).
3. Tem de acontecer: a forma branca de baixo tem um BRILHO verde à volta, que se espalha para fora da
   borda; a barra de baixo mostra cerca de 60 fps.
4. Deu errado se: a forma aparecer sem brilho nenhum (era o que acontecia antes), ou se o brilho aparecer
   deslocado da forma (ao lado em vez de à volta). Para comparar com o desenho antigo, ponha
   `PH2D_FORMAS_NA_PLACA=0` antes de `cargo`: o brilho tem de ser parecido.

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
