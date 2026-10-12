# HANDOFF DE INTEGRAÇÃO — `line/motion-value`, 2026-10-05 (O ABERTO DO PASSE DE FORMAS: MUTAÇÕES, A DOBRA, O APP, A EMISSÃO TRACEJADA)

> História verbatim: [arquivo](../../archive/motion-nodes-2026-10-12/handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-10-05.md) (cortado 12/10 por doc-split.py, sha256 provado).

## ⛔ Recusas MEDIDAS

| recusa | mecanismo medido | onde |
|---|---|---|
| Encurtar a emissão tracejada por arrumação | Quatro arrumações (inline único, hoisting por troço, arestas por peça, grupo de `32`) tiram `≤ 0,014` ms dos `0,244` | vivo §4 |
| Registos do `cs_escreve` completo como custo | Com `72` cópias `0` derrames e `112` VGPRs não mudam o tempo | vivo §4 |
| Critério «tirar não melhora nenhuma cena `> 2 %`» | Cego à TROCA entre cenas (`−2,2 %` conformes contra `+16 %` densas) | vivo §4 |
| Passeio em paralelo (`cs_trocos`, um grupo por cópia) | RECUSA MEDIDA: prova `0,134` ms (critério `≤ 0,13`), produto `0,226` (`≤ 0,15`), densas `+22 %` iGPU / `+16 %` RTX | [arquivo](../../archive/motion-nodes-2026-10-12/handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-10-05.md) §12 |
| Fita (`U+A`) contra a parede do Vello | RECUSA MEDIDA: não funde nenhum pedaço (os dois lados do vértice calculam a faixa em ordem diferente) | [arquivo](../../archive/motion-nodes-2026-10-12/handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-10-05.md) §12 |
| Emissão tracejada por cópia no proxy de telemóvel | Fundo do modelo «um fio por cópia»; o grupo por cópia está recusado (§9.14) | vivo §6 |

> Leitor: o agente integrador e a próxima janela desta linha. O plano, as tabelas e os veredictos vivem no
> [doc 121 §9.16–§9.17](../121_as_formas_na_placa.md); aqui fica o que um merge precisa de saber. O handoff anterior
> (03–04/10) já está no `main`.

## §0 — IDENTIDADE

| | |
|---|---|
| ramo · worktree | `line/motion-value` · `Worktrees/line-motion-value/` |
| merge-base | `5d596eaaf` (= `main` quando a onda começou; nenhum rebase foi preciso) |
| HEAD | o commit deste handoff |
| commits da onda | `81df613b3` memória PT-BR · `071e56a5d` sonda intercalada (§9.16) · `08c58fd87` regra de medir · `bfad80ea1` índice DevOps · `014c93238` handoff de continuação · `64b08daab` plano §9.17 · `3609f28d2` construção (candidatos + portas) · `353156e55` critério da série G32 · `4f7f511ad` resultados · `df46623f5` a DOBRA · `f42e1b618` mutações + memória · este |

## §1 — O QUE A LINHA ENTREGA

- **(b) Os pedaços do §9.15 dobrados** (`A1a` ajuste na contagem · `A1b` total no percurso · `B1` prefixo das arestas
  ESCRITAS · `B2` junta uma vez por troço): `override` → código fixo, ramos `false` apagados, e o recurso «ajuste = 0 ⇒
  calcula-o aqui» do `percorre` (morto desde o A1a). Decidido por **deixar-um-de-fora** na sonda intercalada: tirar
  qualquer um piora as tracejadas (até `+12,6 %` na RTX). Registos: `16/17` shaders iguais ao `F` por `override`.
- **(d) A emissão tracejada medida até ao fundo e os quatro candidatos RECUSADOS** (`L` o adiado no laço · `H` a
  geometria por troço · `P` as arestas por peça · `G32` grupos de `32`, este também na série justificada das
  contínuas: `−2,4 %` contra `≥ 3 %`). As ablações partem os `0,160` ms da emissão (iGPU): `0,066` andar o laço dos
  pedaços, `0,094` a geometria e as arestas. Zero derrames e `112` VGPRs não mexem no tempo (duas ondas). A alavanca
  que resta é o paralelismo dentro de uma cópia — topologia já recusada (§9.14); NÃO reconstruída.
- **(c) O app medido:** a `=127` densa tracejada passa de `128` para **`64` MB** de arestas com os pedaços do §9.15
  (reservadas `4 507 648 → 2 544 640`), as duas placas; a contínua `32` MB; parede presa a `60` fps.
- **(a) Mutações dos pedaços do §9.15: `7/7` sangraram** (m6 na iGPU, subgrupo `64`).
- **(e) Registos** `base → F`: só o `cs_escreve` completo (`30 140 → 47 112` B, `12 → 28` derrames — o A1b inlinava o
  `emite_pedaco` duas vezes) e o `cs_conta` completo (`3 964 → 4 384` B) mudam; o `F` dobrado `46 216` B, `26`.
- **Instrumentos:** a sonda intercalada imprime `conta` e `escreve` à parte, as arestas reservadas · escritas e a
  imagem de cada variante contra a do `F` byte a byte; `mede_formas_na_placa.sh` SEM espera de calma e com
  `VARIANTES`/`TRACEJADOS`; `mede_intercalado.sh` aceita (e diz alto) o SIGSEGV da RTX depois do `test result: ok`.

## §2 — SUPERFÍCIE DE COLISÃO

| ficheiro | o quê |
|---|---|
| `crates/ph2d-shape-gpu/src/contorno.wgsl` | os `override` do §9.15 dobrados (`−40` linhas) |
| `crates/ph2d-shape-gpu/src/contorno.rs` | sem `compactas`; o `garante` SAIU (`695 → 624`) |
| `crates/ph2d-shape-gpu/src/contorno_capacidade.rs` | o `garante` mudou-se para cá + a linha `[formas] arestas: capacidade …` (`73 → 144`) |
| `crates/ph2d-shape-gpu/src/pass.rs` · `lib.rs` | `CONSTANTES_DO_CONTORNO` (vazia) e `VarianteDoPasse::do_ambiente` |
| `crates/ph2d-app-motion/src/motion_shape_placa.rs` | sem variante de sonda, a placa nasce com `do_ambiente()` |
| `crates/ph2d-app-motion/src/motion_shape_placa_gpu_intercalada_tests.rs` | variantes `F` · `F-D` · `F-c2`; colunas novas |
| `docs/Motion Nodes/…` · `project-memory/…` | doc 121 §9.17, ferramentas, este handoff, uma memória |

- **Foundational tocado:** nenhum. **Shell (`shells/desktop`):** não tocada (delta `0`). **Contratos congelados:** nenhum.
- **API pública nova** (`ph2d-shape-gpu`): `pub const CONSTANTES_DO_CONTORNO: &[&str]` · `VarianteDoPasse::do_ambiente()`.
- **Env nova (porta de medição):** `PH2D_FORMAS_CONSTANTES=NOME=v,…` — lida ao criar a placa do app; nomes fora da
  lista vão a `stderr` como «ignorada». Vazia/ausente = o produto. Sem custo em execução.
- **Ids/consts:** os `override` `AJUSTE_NA_CONTAGEM` · `TOTAL_NO_PERCURSO` · `ARESTAS_COMPACTAS` · `JUNTA_UMA_POR_TROCO`
  SAÍRAM (quem os passar por `com_constantes` recebe erro de pipeline do wgpu: constante inexistente).
- **Itens partilhados apagados:** nenhum. **Listas/catracas baixadas:** nenhuma.

## §3 — ⚠️ O que uma leitura rápida do diff entende ao contrário

- O `contorno.rs` encolhe `71` linhas mas nada se perdeu: o `garante` é o MESMO texto em `contorno_capacidade.rs`
  (submódulo `capacidade`, `pub(super)`), com uma linha de relato a mais.
- `CONSTANTES_DO_CONTORNO` vazia não é resto: é a porta do PRÓXIMO candidato (`MEDIR_VELOCIDADE.md`: peça nova nasce
  `override`). A porta `PH2D_FORMAS_CONSTANTES` mediu o A/B do app com UM binário.
- ⚠️ **Na RTX o `A1b` muda `183` B da camada nas densas tracejadas** (o `F−A1b` e o `base` diferem do `F` pelo mesmo):
  o total de um fechado somado no percurso é a mesma soma termo a termo, mas o compilador da NVIDIA funde-a de outro
  modo e uma emenda no limite vira. Os gates contra o Vello passam; na iGPU é igual byte a byte.

## §4 — ⛔ Premissas do briefing que a medição derrubou

- «A próxima alavanca das tracejadas é a EMISSÃO por peça»: é, mas **não se encurta por arrumação** — quatro
  arrumações (inline único, hoisting por troço, arestas por peça, grupo de `32`) tiram `≤ 0,014` ms dos `0,244`.
- «Os registos do `cs_escreve` completo pesam»: com `72` cópias (duas ondas) `0` derrames e `112` VGPRs não mudam
  o tempo.
- O critério do (b) que eu escrevi («tirar não melhora nenhuma cena `> 2 %`») tirava o `B1` — era cego à TROCA entre
  cenas (`−2,2 %` conformes contra `+16 %` densas). Ficou o critério do §9.15; a falha está escrita no doc e na
  memória [`feedback_a_leave_one_out_criterion_is_blind_to_a_trade_between_scenes`](../../../project-memory/feedback_a_leave_one_out_criterion_is_blind_to_a_trade_between_scenes.md).

## §5 — A PROVA DE FECHO (corrida nesta árvore, HEAD `df46623f5`, base `5d596eaaf`)

`nextest-impacted` **`15 713/15 713`** · `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` ✓ · clippy
`-D warnings --all-targets --all-features` de `ph2d-shape-gpu`, `ph2d-gpu`, `ph2d-app-motion` ✓ · `fmt --check` ✓ ·
`machete` ✓ · `check-standalone-optional` ✓ · `check-workflow-packages` ✓ · `ph2d-shape-gpu --test it --ignored`
**`14/14` iGPU e `14/14` RTX** · `motion_shape_placa --ignored --skip sonda_` `5/5` nas duas · `ph2d-gpu-cook formas`
`2/2` nas duas · tecto de LOC ✓ · censos `114/114`. Mutações `7/7` (§1). Depois de `df46623f5` só docs e memória.

## §6 — ⏳ O QUE FICA ABERTO

| item | o endereço |
|---|---|
| ⛔ a emissão tracejada por cópia no proxy de telemóvel | no fundo do modelo «um fio por cópia» (§9.17 d); só o paralelismo dentro da cópia a move, e o GRUPO por cópia está recusado (§9.14). Quem voltar, mede primeiro, por ablação, a emissão dupla contra a fase em série |
| o SIGSEGV da RTX ao sair de um processo com dezenas de pipelines | com `13` variantes volta mesmo com `__GL_SHADER_DISK_CACHE=0`; os dados saem antes; o roteiro di-lo |
| a mordida do traço rente depois de uma quina | divergência DECLARADA (§9.9), sem acção |
| `fk.rs` duplicado (bug #11) · o contacto dos colisores na placa | fora de escopo (Física / wave própria) |

## §7 — SMOKE (o dono)

✅ **Smoke do dono APROVADO em 05/10.**

Binário compilado nesta árvore (`rm -rf target/*/incremental` e `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` 2×; a 2.ª saída):

```
    Finished `smoke` profile [optimized] target(s) in 0.21s      (zero «Compiling»)
```

Fotografado a `1930 × 1040` (RTX): as estrelas amarelas esticadas com o contorno azul tracejado, os cartões Falloff ·
Vortex · Attractor · Curl Noise · Integrate · Duplicator · Scale · Output; `59`–`60` fps; `[formas] 1024 copias … pela
PLACA (do dispositivo)`, arestas `4` MB.

1. No terminal:
   ```
   cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=127 PH2D_TRACO_ESTICADO_TRACEJADO=1 cargo run -p ph2d-host-desktop --profile smoke
   ```
2. Não precisa clicar em nada: abre a cena das estrelas amarelas com o contorno azul em traços, a mexerem com a
   simulação (os cartões Vortex, Attractor e Curl Noise no grafo de baixo).
3. Tem de acontecer: igual ao smoke de 04/10 — a cena certa desde o primeiro instante, os traços certinhos, nada a
   piscar, a barra de baixo a ~60 fps. Esta onda não mudou o que se vê; mudou a memória que a cena usa (metade).
4. Deu errado se: algum traço faltar, ficar torto ou piscar, a janela engasgar ao abrir, ou a barra cair muito abaixo
   de 60.

## §8 — A UMA LINHA para o `CLAUDE.md` §5 (o integrador aplica)

Troca só o link do handoff na entrada **Motion Nodes**: `Último: [handoff 05/10](docs/Motion%20Nodes/handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-10-05.md)`. A frase do módulo não muda.

## §9 — PERFIL DO LOOP DO AGENTE (`bash scripts/agent-loop-profile.sh`, verbatim)

```
  ✗ paralelismo de ferramenta              1.13/passo   alvo: >= 1,5  (8% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                296   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                729 : 234   alvo: <= 1,0  razao 3.1x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  33%   alvo: >= 80%  (1788 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         460 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```

## §10 — A 2.ª onda (05/10, depois do smoke aprovado): «precisamos resolver tudo»

✅ **Smoke do dono da 2.ª onda APROVADO em 05/10** (a `=127` tracejada, a mesma imagem e `60` fps).

Ordem do dono sobre o §6. Plano e números: [doc 121 §9.18](../121_as_formas_na_placa.md). Commits
`0bb491ca3` … `cd11b022c` (⚠️ `849ca87f0` SOZINHO não compila — o `name` do `Cargo.toml` do kit entrou no seguinte; não o
escolha num cherry-pick nem num bisect).

| item | o que ficou |
|---|---|
| **bug #11 e a família** | ✅ `crates/ph2d-rig-kinematics` (`fk`/`pose`/`trig` da família `rig.*`: 6+6+3 cópias → 1; `fk::pai`, a porta que o `rig.bones` copiava) e `crates/ph2d-motion-kit` (`hash`/`trig` sem dependências; `forca` atrás da feature `forca`): a MESMA lei em `38` crates (nós, `ph2d-contact`, `ph2d-bloom`). Ficheiros `.rs` byte-iguais `> 1 KB` entre crates `5 → 0`. Portão novo `architecture_a_lei_partilhada_dos_nos_vive_numa_porta` (`ph2d-editor-core`), `3/3` mutações a sangrar. ⚠️ O nome NÃO pode começar por `ph2d-node-` (o `ph2d-node-sync` trata toda `ph2d-node-*` como nó — foi o vermelho do gate). Espelhos WGSL intocados |
| **(F) o SIGSEGV da RTX ao sair** | ✅ causa medida (o perfilador segurava o dispositivo: `4/6` com ele, `0/9` sem). `ph2d_gpu::pass_profiler`: `Mutex<Option<Arc<…>>>`, `PassTimestamps` (cópia dona do `QuerySet`) e `shutdown()`. ⏳ a prova directa sob carga não reproduziu nem sem a cura (§9.18 F) |
| **(D) o contacto na placa** | ✅ recusa do doc 115 RENOVADA com o número: `4 096` peças custam `9,3 %` de um quadro na CPU (`8` varreduras do produto); expira a `16 384`. A tabela antiga lia `~8×` a mais (carga `45`–`91`) |
| **(E) a emissão por peça** · **(C) a mordida** | ⏳ planos e kill-criteria no §9.18; prompt da próxima janela: [`HANDOFF_CONTINUACAO_line_motion_value_2026-10-05_E_e_C.md`](HANDOFF_CONTINUACAO_line_motion_value_2026-10-05_E_e_C.md) |

**Superfície de colisão nova (o integrador mede):** `ph2d-gpu` (foundational, aditivo na intenção mas MUDA a assinatura:
`compute_writes`/`render_writes` devolvem `Option<PassTimestamps>`) e os `15` sítios instrumentados — `13` em
`ph2d-render` (`band_blit`, `clip_pass` ×2, `compositor`, `frost`, `impasto_light`, `layer_compositor/compositor/{dispatch,pass}`,
`motion_fx`, `preview_premul`, `renderer_draw`, `tonemap`) e `2` em `ph2d-shape-gpu`; ⚠️ **outra linha que acrescente um
`timestamp_writes: …compute_writes(..)` NÃO compila depois deste merge** — a forma nova é
`compute_writes(..).as_ref().map(ph2d_gpu::pass_profiler::PassTimestamps::compute)`. ~`40` `Cargo.toml` de nós (dependência
nova do kit) e o `Cargo.lock`; `ph2d-node-registry-init` intocado. Crates novas: `ph2d-rig-kinematics`, `ph2d-motion-kit`.
Ficheiros apagados: os `fk.rs`/`pose.rs`/`trig.rs`/`hash.rs`/`accum.rs` copiados (um merge com outra linha que os edite
dá conflito modify/delete — a edição vai para a porta). Teste novo: `ph2d-contact` `custo_da_separacao_aos_milhares`
(`#[ignore]`, sonda). Porta de medição nova: `PH2D_PILHA_LADO`/`PH2D_PILHA_COLIDE` (cena `=114`). Memória:
`feedback_killing_the_parent_of_an_orphan_kills_the_owners_session.md` — ⚠️ o checkout PRINCIPAL tem uma cópia igual
POR RASTREAR (e a linha no `MEMORY.md` dele, por comitar): o merge recusa-se a sobrescrevê-la até ela sair de lá.

**Fecho corrido (HEAD `ce5095c32` + a mudança de nome):** `nextest-impacted` `17 057/17 057` · `check --workspace
--all-targets` `-D warnings` ✓ · clippy das `9` crates ✓ · `fmt`/`machete`/`standalone`/`workflow` ✓ · `ph2d-shape-gpu`
`14/14` iGPU e RTX · `ph2d-gpu --ignored` `1/1` nas duas · `motion_shape_placa` `5/5` nas duas · `motion_fx` `26/26` nas
duas · `architecture_*` `106` ✓ · censos `114/114` · staleness do registo `2/2`.

## §14 — A 6.ª onda (06/10): o que o §13 deixou aberto, num bloco

Smoke do §13 **APROVADO** pelo dono (06/10: *«smoke ok. siga com o que está em aberto»*). Números e recusas:
[doc 121 §9.21](../121_as_formas_na_placa.md). Commits `12d3f220d` · `02106af1a` · o seguinte (docs).

| item do §13 | o que ficou |
|---|---|
| **2. `16 384` por taça** | ✅ **FECHA:** o rapier `parallel` (os mesmos bits com `1`/`4`/`16` fios, e o hash do `ph2d_physics_c9` igual com e sem a feature) + a taça em SEGMENTOS (o 1.º passo de um mundo novo `213 → 8` ms). App: **`57`–`60` fps em todas as janelas nas duas placas** (cozimento `7,7` ms de mediana). O item da placa (prompt de 05/10) fecha com a medição |
| **3. o p95 do tique a `4 096`** | ✅ `7,5 → 5,5` ms no produto; o app fica nos `60` fps (cozimento `2,5` ms de mediana) |
| **1. o rolamento numa pilha de discos** | ✅ medido em três variantes; a trava destranca ao 2.º excesso SEGUIDO (discos `0,25` assentam: `44° → 0°`; gate novo). ⚠️ Os discos a `Rolling 0,1` agitam na QUEDA (`104°` contra `10,7` sem o botão) e assentam como sem ele — a leitura é um monte que desaba enquanto assenta; fica registado, não é um tremor da pilha parada |
| **4. o comportamento que mudou** | ✅ aprovado no smoke |

**Gate retirado com o porquê:** `the_second_lap_of_the_loop_repeats_the_first` — a mutação do mundo velho dava a MESMA
diferença (o mundo já morria na pausa); no lugar dele `the_loop_pause_drops_the_world_and_the_next_fall_starts_a_new_one`.

**Superfície de colisão (o integrador mede):** `ph2d-contact-world` (`Cargo.toml`: `rapier2d` + `parallel` — ⚠️ a feature
UNIFICA-SE com a `ph2d-physics` em toda build que junte as duas; `Cargo.lock` ganha `rayon` em `parry2d`/`rapier2d`),
`fixo.rs` (`colisores`, a taça por lados), `lib.rs` (`Fixo.colisores`, `excessos`), `rolar.rs`
(`EXCESSOS_PARA_SOLTAR`); `ph2d-app-motion`: os gates da pilha e do recuo. Foundational: nenhum novo. Shell: não tocada.

**Fecho corrido** (HEAD `02106af1a`, árvore limpa): `nextest-impacted` (`BASE=a46c4c200`) **`18 770/18 770`** ·
`check --workspace --all-targets` (warnings negados) ✓ · clippy das `9` crates (com `ph2d-physics`/`-ecs`) ✓ · `fmt` ✓ ·
censos `114/114` ✓ · standalone, workflow, `adr-index`, `doc-index`, `archive-index`, `machete`, **`cargo deny`** ✓ ·
`tests/it` das `5` crates (com as duas da Física) `1 026/1 026`. Mutações `m3`/`m14`/`m15` sangram.

**⏳ ABERTO:** nada desta família. Integrar só por ordem do dono.

## §15 — 06/10, report do dono: *«a animação não dura o suficiente para ver todos os quadrados colidirem»*

A duração da zona da `=114` cresce com a pilha (`duracao_de`: `3` s até `15` de lado — a cena do smoke ao bit —, `8` s
até `31`, `15` s daí para cima), pela medição do instante em que cada pilha assenta (doc 121 §9.22). Gate
`the_pile_stops_before_the_fall_restarts`; mutação `m16` sangra. Sondas novas: `quanto_tempo_a_pilha_leva_a_assentar`,
`custo_por_fase_da_queda`.

⛔ **CORRECÇÃO do §14:** «`16 384` por taça a `57`–`60` fps» valia para os primeiros `3` s (a queda). Com a pilha
FORMADA o tique é `11`–`16` ms e o app lê `21`–`32` fps. `4 096` por taça fica nos `60` fps na queda inteira (`2,8` ms
plano). ⇒ **o item da placa REABRE** (doc 121 §9.22: dormir não ajuda; menos iterações tiram `20`–`30 %`; o rapier
puro gasta `6`–`9` ms nessa pilha).

**Decisão do dono (06/10):** *«por enquanto deixamos assim»* — `16 384` por taça com a pilha formada fica nos `~30`
fps; o contacto no dispositivo NÃO entra agora (o item da placa fica registado como adiado pelo dono, com os números
do doc 121 §9.22, não como recusa medida).

## §16 — A 8.ª onda (06/10): os discos com `Rolling` baixo — avalanche ou a trava a alternar?

Prompt: [`HANDOFF_CONTINUACAO_line_motion_value_2026-10-06_ROLAMENTO_DISCOS.md`](HANDOFF_CONTINUACAO_line_motion_value_2026-10-06_ROLAMENTO_DISCOS.md).
Desenho escrito antes, rodadas, tabelas e recusas: [doc 121 §9.23](../121_as_formas_na_placa.md). Commit `838384182`
(código, gates, sondas, mutações) · o seguinte (docs, memória). Base `a46c4c200` (o main não andou).

| item do prompt | o que ficou |
|---|---|
| **(1) provar ou refutar a leitura** | ✅ **as duas, por partes.** Na árvore de hoje o `104°` não reproduzia: a tabela do §9.21 foi medida com a taça numa POLILINHA (passou a segmentos no mesmo commit) — com a polilinha de volta, `103,6°`/`10,7°` ao bit, e era UM disco solto a rolar `0,73` u (avalanche). Mas com ONZE realizações o defeito existe na taça de hoje (pior queda `152°`; a `0,05` gira mais que sem o botão), e a régua «trancado = `spin == 0`» era cega: a trava PISCAVA `20`–`39` vezes por peça por segundo na pilha assente (referência `Lock Rotation`: `0,000` u de deslize) ⇒ defeito da trava, na pilha assente |
| **(2) a cura** | ✅ a trava solta ao **12.º** excesso seguido (`EXCESSOS_PARA_SOLTAR`) e só TRANCA quem **não acelerou** no passo, ou quem mal se mexeu (`|L| ≤ 25 %` da capacidade, `QUASE`). ⛔ **Recusados por medição:** o motor angular do rapier (o candidato do prompt: queda `58°` de mediana, a pilha a rastejar, caixas paradas a tremer), o binário CONTRA o pedido no passo que solta, a folga no limiar, soltar ao 3.º/8.º, «só quem não acelera» sem o `QUASE` |
| **os `210°` dos discos `0,05` assentes** | ✅ provado (`probe_o_mecanismo_por_rolamento`): UM disco a rolar `0,41` u monte abaixo (caminho `1,11×` o giro vezes o raio, `0 %` no lugar); os outros `≤ 1,2°` — avalanche (sem o botão os discos chegam logo ao fundo) |
| **⚠️ um defeito que o prompt não nomeava** | a bola numa rampa MAIS inclinada que o `Rolling` dela ficava PRESA (`0,020` da descida a `0,1`/`0,15`/`0,2`; teoria `0,530`/`0,294`/`0,059`) — com a lei de antes também; nenhum gate media a faixa `0 < μr < tg θ`. Agora `0,556`/`0,241`/`0,103`; a `0,5` presa (`0,018`) |
| **(3) kill-criteria** | ✗ *discos `0,1` `120..180` `≤ 21°`* na PIOR das onze (`41°`; a mediana `5,6°`) — e essa queda é um disco a ROLAR `0,10` u por um monte de `37°` (avalanche; `Rolling 0,1` só segura até `5,7°`); o `21°` saiu de UMA realização da taça antiga. ✓ caixas `0,25`/`0,75`; ⚠️ discos `0,25` `0° → 0,6°` (pior queda) e `1,0°` assente; ✓ tempos da taça `±2 %`; ✓ rampa `0,5` `0,0201`; ✓ os gates da pilha; ✓ duas corridas iguais ao bit; ✓ o recuo seguido de Play (corrida limpa da mutação). ⚠️ O app a `4 096` não foi re-medido: com `Rolling 0` (a `=114` de sempre) o passo só ganha a leitura de um `ω` por peça — ver o fecho |

**Gates novos:** `a_ball_rolls_down_a_ramp_steeper_than_its_rolling` (`material_demo_tests.rs`) ·
`the_rolling_lock_does_not_flicker_in_a_pile_of_discs` (`pilha_demo_tests.rs`, onze realizações, `~5,6` s em debug).
**Mutações** ([`mutacao_a_onda_5_2026-10-06.py`](../ferramentas/mutacao_a_onda_5_2026-10-06.py), `m17` solta ao 2.º ·
`m18` tranca a acelerar · `m19` sem o `QUASE`): **`19/19` sangram**, pré-voo `19/19`, corrida limpa `62` verdes
(`target/prova/onda8/mutacao.txt`). A `m7` (a trava nunca fecha) passa a sangrar também no gate novo da rampa.

**Fecho corrido** (HEAD `838384182`, o código comitado): `nextest-impacted` (`BASE=a46c4c200`) **`18 773/18 773`** ·
`check --workspace --all-targets` (avisos negados) ✓ · clippy `ph2d-contact-world`/`ph2d-app-motion`/`ph2d-node-sim-step`
`-D warnings` ✓ · `fmt` ✓ · censos `12/12` · standalone · workflow · `adr-index` · `doc-index` · `archive-index` ·
`machete` ✓ · `tests/it` da `ph2d-contact-world` `1/1`. Depois dele só a sonda `0,05` (teste `#[ignore]`; clippy da crate ✓).

**Superfície de colisão (o integrador mede):** `ph2d-contact-world` — `rolar.rs` (`EXCESSOS_PARA_SOLTAR = 12`, `QUASE`
novo, as tabelas), `lib.rs` (`w_antes`, a condição de trancar); `ph2d-app-motion` — `motion_state_pilha_demo.rs`
(`Medida.discos`/`rolar`, a porta `PH2D_PILHA_DISCOS`/`PH2D_PILHA_ROLAR`, o índice `Circle` perguntado ao registo, o
`mod discos_diag`), `motion_state_material_demo.rs` (a porta `PH2D_RAMPA_ROLAR`), `motion_state_pilha_demo_discos_diag.rs`
(NOVO: a marcha única, as réguas do mecanismo, `rodada`/`realizacoes`), `motion_state_pilha_demo_giro_diag.rs`
(`perfil_com` passa a usar a marcha única: `−98` linhas), os dois `*_tests.rs`. Foundational: nenhum. Shell: não
tocada. `Cargo.lock`: não muda. **Env novas (portas de medição, nenhuma muda o produto sem elas):** `PH2D_PILHA_DISCOS`,
`PH2D_PILHA_ROLAR`, `PH2D_RAMPA_ROLAR`.

**Smoke (o dono), fotografado** (`target/prova/onda8/smoke/`, `release`, `1930 × 1040`): a `=114` em discos com
`Rolling 0,1` — a grelha de discos (`e14.5`), o monte assente e arredondado na taça da direita (`e16`), `59` fps;
a `=115` com a bola da rampa a `Rolling 0,1` (`e13`/`e15`). Ver o §16 do smoke abaixo (passos ao dono no relatório).

**⏳ O QUE FICA ABERTO:**
1. **A rampa perto do limiar:** a `0,2` (logo abaixo de `tg 12° = 0,213`) a bola desce `0,103` da solta contra `0,059`
   da teoria (`1,7×`), a `0,15` `0,241` contra `0,294`. O gate aceita `±0,06`. Uma lei exacta perto do limiar pede o
   rolamento DENTRO do solver — e o único que o rapier oferece (o motor) foi recusado pela pilha.
2. **Um oráculo externo** para a pilha com rolamento (o Box2D `3.1.1` está no repositório `extra`, MIT, não
   instalado — pede a senha do dono; e a lei dele é por PAR, a que esta casa mediu e recusou). Não corrido.

## §17 — A 9.ª onda (07/10): o aberto do §16, num bloco

Smoke do §16 **APROVADO** pelo dono (07/10: *«smoke ok. vamos resolver o que está em aberto»*). Números, tabelas e
recusas: [doc 121 §9.24](../121_as_formas_na_placa.md). Base `a46c4c200` (o main não andou).

| item do §16 | o que ficou |
|---|---|
| **1. a rampa perto do limiar** | ✅ **mecanismo provado e cinco variantes RECUSADAS por medição; a lei fica.** A `0,15` desce pouco porque a trava espera `0,2` s (`0,294 × 0,81 = 0,238`, medido `0,241`); a `0,2` desce demais porque o travão a rolar é `clamp(L)` enquanto o momento é pequeno. O travão inteiro aplicado de fora do solver PRENDE a bola (`0,020` a `0,1`); soltar mais cedo pelo sentido do pedido devolve o pisca-pisca ou faz a pilha de `0,25` desabar (`62°`). E o oráculo (o rolamento DENTRO do solver) também se afasta da teoria perto do limiar, para o outro lado (`0,023` contra `0,059` a `0,2`) |
| **2. o oráculo externo** | ✅ **corrido:** o Box2D `3.1.1` (MIT; o pacote do `extra` baixado com a assinatura conferida, NADA instalado — não precisou da senha do dono) sobre as posições iniciais exportadas da nossa cena, onze realizações ([`oraculo_box2d_rolamento/`](../ferramentas/oraculo_box2d_rolamento/README.md), fixture com cabeçalho). A unidade do `rollingResistance` é a nossa. ⭐ **Nas pilhas o Box2D tem o «defeito» do prompt de 06/10:** discos com rolamento giram MAIS na queda que sem ele (`0,05`: `198°` contra `77°`) — é das pilhas com rolamento, não da nossa trava; a nossa lei é a mais calma a `0,1`/`0,25`. ⚠️ Não é gate de paridade: a lei dele é por PAR (recusada para nós) |
| **3. o app a `4 096`** | ✅ **`61`–`70` fps em todas as janelas** fora do arranque, nas duas placas, carga `1,9`–`3,8`; Motion `≤ 4,6` ms; a sonda sem vídeo igual à de 06/10 (`2,7`–`3,0` ms com a pilha formada). A 1.ª leitura a carga `14` dava `51`–`57` — não vale |

**Superfície de colisão (o integrador mede):** `ph2d-app-motion` — `motion_state_pilha_demo_discos_diag.rs` (a sonda
`exporta_a_pilha_para_o_oraculo`), `motion_state_material_demo_tests.rs` (o comentário do gate da rampa);
`docs/Motion Nodes/ferramentas/oraculo_box2d_rolamento/` (NOVO: `main.c`, `corre.sh`, `README.md`, a fixture). O
produto NÃO mudou nesta onda (a chave das variantes saiu antes do commit). `Cargo.lock`: não muda.

**Fecho corrido** (o diff desta onda é uma sonda `#[ignore]`, um comentário e docs): clippy `ph2d-app-motion`
`--all-targets --all-features -D warnings` ✓ · `fmt` ✓ · os gates da rampa, das pilhas de discos e da `=115`
`13/13` · censos `12/12` · `doc-index` ✓. O fecho batched completo da linha é o do §16 (`18 773/18 773`, código
igual).

**⏳ O QUE FICA ABERTO:** nada desta família. ⚠️ Registado (não é aberto): a rampa perto do limiar segue a
teoria a `±0,07` (o gate aceita `±0,06` a `0,1`/`0,15`/`0,2`); uma lei exacta ali pede o rolamento dentro do solver,
e o único que o rapier oferece (o motor) foi recusado pela pilha (§9.23). Integrar só por ordem do dono.

## §18 — A 10.ª onda (07/10): os ABERTOS do ciclo 6, num ciclo

Ordem do dono (07/10): *«resolver num único ciclo o Ciclo 6 completo»*. Estado medido, desenho, critério de abandono
e resultado de cada item: [doc 110 §14](../110_ciclo_6_valor_e_pulso.md). Base `a46c4c200` (o main não andou); commits
desta onda `7257b737e` · `cd5f23179` · `bdaf1f8fa` · (docs) · `05f92f165` · `6ca487037` · `0dc494bf4` · `19e20fa02` · `561d4ec23` · este.

| item | o que ficou |
|---|---|
| **1. W1(b) — porta ≠ 0** | ✅ duas espécies: o COMPLEMENTO de um `Compact` (`StreamOp::Compact.complement`, `died`/`pulse` do `sim.lifetime`, a mesma varredura e a mesma leitura de 8 bytes) e a PORTA PROJECTADA (`ProjectedPort` + `KernelResolver::projected_ports`: o `carry` do `pulse.counter`). `GpuSource::StagePort(nó, porta)`. Censo PELA PORTA DO PRODUTO (corrigido: planeava sem os fios): **60 → 64** cenas inteiras na placa (`=24 =25 =26 =27`; a `=117` sobe a híbrida — com as ESTRELAS do smoke a forma vem da CPU); presas por porta ≠ 0 **4 → 0** |
| **2. o condutor na placa** | ✅ `plan_with_device_drivers` (porta NOVA; a `plan()` sem mapa é a lei antiga ao bit) + cópia de 4 bytes `v[0]` → uniform; só consumidores de mapa puro. CPU-lado −37 % num condutor caro; zero leituras de volta |
| **3. W3b — três chaves** | ✅ ficam, com catraca (`ph2d-editor-core` `architecture_as_chaves_partilhadas_nao_ganham_leitor_partilhado`: 0 leituras partilhadas em 3 840 ficheiros) |
| **4. as cópias** | ⛔ recusado por medição (5 consumidores fazem do comprimento significado; a costura custa `0,002 ms`); o censo achou e curou `value.wrap` (1→N) e `motion.distribute_radial` (`spin` → `ReadBroadcast`) |
| **5. o relógio** | ✅ `sonda_o_relogio_do_ciclo_6` pela porta do produto (`motion_bridge::quadro::coze_o_quadro`, partida do `dispatch`): fio `0,051` · sem fio `0,053` · fio a derrubar `0,134` ms (`2,6×`) |
| **6. a barra a um ULP** | ✅ era o `value.noise` em espaço de índice; o gate do slope mede em espaço de mundo (vale `6,2e-6`/`5,0e-4`); o comparador já não engole NaN |
| **7. o consumidor do pulso** | ✅ `sim.spawn` com pulso na placa (`SourceRows.fired: FiredBirth`); 531/999/12 891 nascidos ao bit nas três cadeias (os fogos incluídos) |

**Achados curados no caminho:** a placa HERDAVA o `age` ao nascer (`SourceRows::not_inherited`); um documento
inteiro na placa com `pulse.signal` gritava errado (a bomba avança o `pre` do cone das tomadas sem fronteira, e a
ponte marcha-o quando há sinais — `a_ponte_na_placa_grita_o_que_a_cpu_grita`); a `=116` dizia `102 400` peças e
um engasgo que não existe (legenda, anúncio e tutorial 06 §7 refeitos).

**Superfície de colisão (o integrador mede):**
- ⚠️ **foundational, aditiva:** `ph2d-nodegraph` (`stream_op_meta.rs`: campos NOVOS em `StreamOp::Compact`
  — `complement` — e `StreamOp::SourceRows` — `fired`, `not_inherited` —; `gpu.rs`: `ProjectedPort` e o método
  com omissão `KernelResolver::projected_ports`). ⛔ **Qualquer linha que construa um `StreamOp::Compact` ou
  `StreamOp::SourceRows` por literal não compila depois do merge** — os construtores de hoje (`motion.cull`,
  `sim.lifetime`, os seis `SourceRows`: `fx.drop_shadow`, `fx.rgb_split`, `motion.clone`, `motion.duplicator`,
  `motion.kaleidoscope`, `motion.mirror`, e o `sim.spawn`) foram atualizados aqui; um novo de outra linha leva
  `complement: &[]` / `fired: None, not_inherited: &[]`. Contratos congelados (§6): nenhum tocado
  (`NodeOp`/`OpResolver`/`NodeManifest` intactos).
- `ph2d-node-registry` (`lib.rs` 699/700: um campo; `gpu_channels.rs`: `register_projected_ports`).
- `ph2d-gpu-cook`: `plan.rs` cortado (`plan_forma.rs` NOVO, `plan_condutor.rs` NOVO), `lib.rs`, `encode.rs`
  (`create_pipeline` mudou-se para cá), `estado.rs`, `stream_op.rs` (concat/junção → `stream_op_concat.rs`, o
  `Carry` → `stream_op_carry.rs`, NOVOS `stream_op_complemento.rs`, `stream_op_disparo.rs`), `debug_read.rs`;
  testes `gpu_cpu_parity*.rs`, `gpu_stream_ops*.rs`.
- `ph2d-eval-motion` (`lib.rs`, `scrub.rs`: uma condição cada).
- `ph2d-app-motion`: `motion_bridge.rs` (o fim do `dispatch` → `motion_bridge_quadro.rs`), `motion_bridge_gpu.rs`
  (os valores dirigidos → `motion_bridge_gpu_dirigidos.rs`), `motion_state.rs`/`motion_state_verbos.rs` (dois
  campos: `driven_gpu`, `condutores_na_placa`), a `=116` e o anúncio dela, o censo de rota, sondas e gates.
- nós: `pulse-counter`, `sim-lifetime`, `sim-spawn`, `value-wrap`, `motion-distribute-radial`, `motion-cull` e os
  seis `SourceRows`. `ph2d-editor-core` (a catraca), `ph2d-label-census` (`so_codigo_de_produto` público),
  `shells/desktop/tests/it/the_gpu_cook_recusal_placement.rs`.
- docs: doc 110 §14, doc 103 §5 (nota de 07/10), tutorial 06 (fonte + PDF). `Cargo.lock`: não muda.

**Fecho corrido (sobre o diff acumulado):** `nextest-impacted` (base `a46c4c200`) **19 542/19 542**; suíte da
placa `334/335` — o vermelho é `gpu_collide::crossing_the_reach_boundary_does_not_step_the_cost`, o candidato à
família de flakes de carga que o doc 110 §6 já nomeava (2/2 sozinho, a `load 9` e a `load 35`; zero linhas
desta onda no teste ou no nó) — ⇒ **pedido ao integrador: promovê-lo à lista** (`FLAKES_DE_CARGA.md`); clippy
`--all-targets -D warnings` nas crates tocadas ✓; `fmt --all --check` ✓; censos `114/114` ✓; `doc-index` ✓.
**Provas de mutação: 16/16 sangraram** (14 do lote + as duas metades da cura dos sinais). Smoke: a `=117`
ganhou ESTRELAS depois do report do dono (*«vejo apenas gizmos. eles não piscam»*) — rota `HIBRIDO` (a
forma vem da CPU); gate `the_bottom_cloths_flash_on_the_device_like_on_the_cpu` (as duas rotas piscam nas mesmas
batidas). Os gates que usavam a `=117` como cena «só posições» (`ponto_gizmo_tests`) passaram à `=6`.

**⏳ O QUE FICA ABERTO (nomeado, fora da lista do ciclo 6):** ⛔ [Bug #14](../BUGS_motion_nodes.md) — formas
carimbadas a partir da SAÍDA de uma zona de simulação não aparecem no ecrã (o cozimento produ-las, o app não as
desenha; bissecção feita; a `=27` ficou sem forma por isso); preparar `value.attribute` + `value.reduce` na placa
custa ~`0,09 ms` de CPU por quadro (é o que come o ganho do item 2 na parede); o gate de paridade do `pulse.beat`
só corre 6 tiques e nunca chega aos empates batida-tique da fixtura decimal. Integrar só por ordem do dono.

## §19 — INTEGRADA no `main` (07/10, por ordem do dono, depois da MiroClone)

- **Ordem:** a MiroClone fechou em `126845087`; esta linha entrou a seguir, sozinha (nada a medir par-a-par).
  `collision-surface`: 4 ficheiros nos dois lados (`Cargo.lock`, `CLAUDE.md`, `MEMORY.md`, a família
  `reference_topic_code_pattern_gotchas`); schemas, registo de componentes e contratos §6 intocados; sem ADR.
- **Rebase** sobre `126845087`, 73 commits: dois conflitos, ambos na memória (os dois lados acrescentaram
  à mesma família). Resolvidos pela UNIÃO e pela CONTAGEM do ficheiro (`26 → 28` main, `→ 27/28` linha ⇒
  `30` reais), nunca por um dos lados. `range-diff`: só a contagem e o contexto do `Cargo.lock` mudaram.
- **Gate da árvore combinada** (`foundational-integrate.sh`): sync/registos ✓, `check --workspace
  --all-targets` com avisos negados ✓, `nextest-impacted` **19 757/19 758** — o vermelho foi
  `o_indice_cabe_no_orcamento_do_carregador`: o `MEMORY.md` SOMADO passava o tecto (`22 109 > 22 000`),
  e nenhuma das duas linhas o passava sozinha. Cura: a entrada do rapier desceu verbatim para a família de
  Arquitetura (`19 → 20`, contada); índice `21 972`; os 4 gates da memória verdes.
- **Pedido do §18 cumprido:** `gpu_collide::crossing_the_reach_boundary_does_not_step_the_cost` promovido à
  família de flakes de carga — as DUAS escritas (`FLAKES_DE_CARGA.md` + a faixa do `.config/nextest.toml`).
- **Fusão `--ff-only`** no primário: as edições de memória POR COMITAR de outras sessões (índice + uma
  versão maior do `feedback_killing_the_parent…`, que esta linha comita) foram guardadas, a fusão feita e
  elas reaplicadas por cima — nada perdido. ⚠️ Com elas o `MEMORY.md` do primário fica em `22 583` bytes:
  quem as comitar tem de descer entradas antes (o gate reprova).
- **Sem push.** O `main` segue à frente do `origin`; o ship é só por ordem do dono.
