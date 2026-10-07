# HANDOFF DE INTEGRAÇÃO — `line/motion-value`, 2026-10-05 (O ABERTO DO PASSE DE FORMAS: MUTAÇÕES, A DOBRA, O APP, A EMISSÃO TRACEJADA)

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

## §11 — A 3.ª onda (05/10): (E) a emissão por peça e (C) a mordida do traço rente

Plano, kill-criteria e números: [doc 121 §9.18](../121_as_formas_na_placa.md) (o (E) e o (C) por baixo do plano).
Commits `afa0cc014` … `6112741eb` (base `5d596eaaf`; o main não andou).

| item | o que ficou |
|---|---|
| **(E) a emissão tracejada por peça** | ⛔ **RECUSA MEDIDA.** A prova do modelo (o passeio por troço + a tabela + o `pedaco` por peça, sem arestas) deu `escreve` `0,183` ms nas esticadas tracejadas da iGPU (critério `≤ 0,13`); o produto (`P2` reserva atómica por aresta, `P3` contado por peça) `0,297`/`0,275`, densas `+20`/`+15 %` na iGPU, RTX `−15 %` nas esticadas mas `+10 %` nas densas e `183 B` diferentes. A decomposição (`P4`–`P6`): o passeio SOZINHO custa `0,130` — o chão é o percurso em série por cópia, não a emissão (o `pedaco` custa `0,002`). Código medido em `afa0cc014`, retirado em `ce2fab7ad`: a árvore não tem nada dele |
| **(C) a mordida do traço rente** | ✅ **CURADA.** Quem mordia era o traçador do **Vello** (o `kurbo` das marcas conformes da placa não morde — varrido). A rota Vello do Motion passa a PREENCHER os polígonos da placa no MESMO nível de aplanamento: cópia conforme → as marcas (`contorno_conforme`); esticada → a porta CPU da lei (`ph2d-shape-gpu/src/contorno_cpu.rs`, o `percorre` do `contorno.wgsl` em `f32`). Família nova `a_rota_vello_traceja_o_pedaco_rente_como_a_placa`: alfa `1` contra a placa nas duas placas (a rota de antes `173`, o controlo); as `4` famílias de hoje `85 → 46`. `encode` da `=127` tracejada `−83 %` na CPU; a parede do Vello `+7`–`+50 %`, o quadro da rota Vello `58`–`79 %` mais curto |

**Superfície de colisão nova (o integrador mede):**
- `ph2d-vec-render` (a porta de lote do Motion): `instance.rs` ganha `TracoProprio` e `draw_shared_instances_com_traco`;
  o `draw_shape_instance_tessellated` (crate-privado) e o `draw_shared_instances_com` ganham um parâmetro no fim (os
  chamadores da casa passam `None`: `standalone.rs`, `encode_cost_tests.rs`). ⚠️ Outra linha que chame estes dois
  crate-privados não compila depois do merge — acrescenta `None`. As portas públicas de sempre não mudam.
- `ph2d-shape-gpu`: `contorno_cpu.rs` (novo) e `geometry.rs` (portas `tolerancia`/`extensao`/`eixo_do_nivel`/
  `contorno_conforme`, o `prepare` igual por construção — `14/14` nas duas placas); `lib.rs` exporta-as e o
  `tracejado_do_eixo`.
- `ph2d-app-motion`: `motion_shape_traco.rs` (novo), `motion_shape_mistura.rs`/`motion_shape_gen.rs` (o `encode` leva
  um `TracoDaPlaca`), `motion_shape_placa.rs` (`entrada_de`, partilhada pelas duas rotas).
- Memória: `feedback_an_ablation_that_stops_storing_the_walk_measures_less_than_the_walk.md` ·
  `feedback_a_defect_attributed_to_a_component_is_swept_before_the_cure.md` (nas famílias régua `190` e diagnóstico `25`).

**Fecho corrido (HEAD `0321a1b46`, depois `6112741eb` com o `px` retirado, `clippy`/`fmt` e os gates do tracejado
re-corridos):** `nextest-impacted` `17 209/17 211` (os `2` vermelhos são flakes de relógio sem diff nas crates deles —
`ph2d-physics-ecs` `the_cost_of_a_player_is_linear…` e `ph2d-tool-painter` `the_mask_stroke_cost…` — verdes `3/3`
sozinhos a `load 32`–`36`) · `check --workspace --all-targets` `-D warnings` ✓ · `clippy --all-targets --all-features
-D warnings` das `3` crates ✓ · `fmt` ✓ · `machete`/`standalone`/`workflow` ✓ · `ph2d-shape-gpu --test it --ignored`
`14/14` iGPU e RTX · `motion_shape_placa` `9/9` nas duas (com as sondas) · `ph2d-gpu-cook formas` `2/2` nas duas ·
`ph2d-vec-render --ignored` `6/6` nas duas · censos `114/114` · tecto de LOC ✓ (`contorno_cpu.rs` `~620`,
`motion_shape_gen.rs` `672`).

**Mutações** ([`mutacao_a_mordida_do_traco_rente_2026-10-05.py`](../ferramentas/mutacao_a_mordida_do_traco_rente_2026-10-05.py),
sozinho na árvore, pré-voo `12/12`): **`12/12` sangram** na família nova (a lista no doc 121 §9.18). A `m6` (a emenda
nunca) sobreviveu à 1.ª corrida — nenhuma forma da família tinha dois contornos; a engrenagem com furo entrou e ela sangra.

**Smoke (o dono):** ✅ **APROVADO em 05/10.** Ordem do dono: *integrar só quando não houver nada em aberto* — a 4.ª
onda fecha o §11 «Aberto» e o resto ([`HANDOFF_CONTINUACAO_line_motion_value_2026-10-05_TUDO.md`](HANDOFF_CONTINUACAO_line_motion_value_2026-10-05_TUDO.md)). Fotografado a `1930 × 1040` (RTX), as duas rotas: a placa (`[formas] 1024 copias de 1
geometrias pela PLACA (do dispositivo)`) e o Vello (`PH2D_FORMAS_NA_PLACA=0`: `[formas] 1024 copias … pela cena Vello`,
`59` fps) — o mesmo desenho, as estrelas amarelas esticadas com o contorno azul em traços (`target/prova/smoke_c/`).
Binário: `rm -rf target/*/incremental` e o build `smoke` do `ph2d-host-desktop` 2× (a 2.ª: `Finished … in 0.20s`, zero
«Compiling»).

1. No terminal:
   ```
   cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=127 PH2D_TRACO_ESTICADO_TRACEJADO=1 PH2D_FORMAS_NA_PLACA=0 cargo run -p ph2d-host-desktop --profile smoke
   ```
2. Não precisa clicar: abre a cena das estrelas amarelas com o contorno azul em traços, a mexerem com a simulação.
   Esta corrida desenha pelo caminho de RESERVA (o que o app usa quando a placa não pode desenhar) — é o que mudou.
3. Tem de acontecer: igual ao smoke de antes (sem o `PH2D_FORMAS_NA_PLACA=0`) — os traços inteiros nas pontas da
   estrela, sem pedacinhos «comidos» no lado de dentro das pontas; a barra de baixo perto de 60 fps.
4. Deu errado se: algum traço faltar ou aparecer cortado junto a uma ponta, a estrela ficar sem contorno, ou a barra
   cair muito abaixo de 60.

**Aberto:** a emissão por peça só volta com o PASSEIO fora da série (um fio por troço e um prefixo segmentado — a
4.ª topologia, two-strikes; a imagem deixa de ser a do `F` ao bit, e o critério tem de vir escrito antes, doc 121
§9.18 E). O nó de forma do Motion não expõe ponta nem junta (o produto traceja sempre rente/esquadria).

## §12 — A 4.ª onda (05/10): os cinco itens do §11 «Aberto» numa onda

Plano, kill-criteria (escritos antes) e números: [doc 121 §9.19](../121_as_formas_na_placa.md). Commits `3bb13c95a` …
(base `a46c4c200`: o main andou um commit, o guarda R4; rebase sem conflito).

| item | o que ficou |
|---|---|
| **(1) o passeio em paralelo** | ⛔ **RECUSA MEDIDA.** `cs_trocos` (um grupo por cópia, prefixos segmentados): a prova `0,134` ms (critério `≤ 0,13`), o produto `0,226` (critério `≤ 0,15`), densas `+22 %` iGPU / `+16 %` RTX; a imagem passava (alfa `1`). O chão passou a ser a EMISSÃO por peça (`0,092`). Código medido em `2774e1d17`, retirado em `c524f9b3d` |
| **(2) a parede do Vello** | ⛔ **RECUSA MEDIDA.** O `U` (um caminho por traço) já era o estado; a FITA (`U+A`) não funde nenhum pedaço (os dois lados do vértice calculam a faixa em ordem diferente), e a rota pela lei manda `2,45×` os segmentos (`98` por cópia contra `40`): parede iGPU `+24`/`+42 %`. `e115108de` → `274ed670e`. A sonda conta caminhos e segmentos |
| **(3) a ponta e a junta no cartão** | ✅ **CONSTRUÍDO.** `stroke_cap`/`stroke_join` (`Enum`, omissão Butt/Miter = o de sempre), no `ALL` e no `SPECS`, i18n, `build_shape_path` → `StrokeSpec`. Gates do nó, do cartão, da gravação (ida e volta e projecto antigo) e da IMAGEM (`3 × 3`, placa × rota Vello alfa `≤ 1`). Contrato `NodeManifest=8` intocado (ADR-0039 lido) |
| **(4) a prova do SIGSEGV** | ✅ **PROVADA.** `40` corridas alternadas na RTX: antes da cura `14/20` `SIGSEGV`, depois `0/20` (carga `3`–`38`) |
| **(5) o contacto em Play** | ✅ medido e **curado na CPU**: o `monta` dos impulsos era `O(n²)`; pelos pares da grelha (os mesmos bits) `1 024` peças por taça `247` ms → `60` fps. ⏳ **ABERTO: o contacto da caixa na placa** — a recusa expirou (a CPU paga `48`–`93` ms por tique a `4 096` por taça com a pilha formada; ⚠️ e o quadro do app no instante denso da pilha: `release` `1 024` por taça `60` fps depois da montagem dos pares em paralelo, `4 096` `13`–`25` fps; régua `[motion-quadro]`, doc 121 §9.19). Modelo de LEI provado (Jacobi refutado: pilha `1,85×` mais agitada; Gauss–Seidel por cores na banda da ordem) e de CUSTO (`1,6` µs por despacho num passe); não construído nesta janela (quatro peças, duas topologias novas do dispositivo). Prompt: [`HANDOFF_CONTINUACAO_line_motion_value_2026-10-05_CONTACTO_NA_PLACA.md`](HANDOFF_CONTINUACAO_line_motion_value_2026-10-05_CONTACTO_NA_PLACA.md) |

**Superfície de colisão nova (o integrador mede):**
- `ph2d-node-motion-shape`: `param.rs` (`STROKE_CAP`/`STROKE_JOIN` APENDADOS ao `ALL` e ao `SPECS` — outra linha que
  apende params a este nó conflita aqui: os dois blocos ficam, pela ordem do merge, e a chave muda), `stroke_style.rs`
  (novo), `lib.rs` (`Stroke` ganha `cap`/`join` — ⚠️ quem constrói `Stroke { width, rgba }` noutra linha não compila:
  acrescente os dois campos), `hints.rs`, `param_gates.rs`, `tests.rs`.
- `ph2d-i18n`: `node_params.rs` (`stroke_cap`/`stroke_join`) e `node_options.rs` (`cap_labels.*`/`join_labels.*`),
  inseridos em ordem.
- `ph2d-contact`: `impulso.rs` (o `monta` pela grelha; `Leis` ganha `jacobi` e `cores` — ⚠️ um `Leis { … }` literal
  noutra linha não compila sem eles), `impulso_jacobi.rs` e `impulso_tests.rs` (novos).
- `ph2d-node-sim-step`: `contact.rs` (`leis()` com as portas `PH2D_CONTACT_JACOBI`/`PH2D_CONTACT_CORES`; vazias = o
  produto).
- `ph2d-shape-gpu`: `contorno_cpu_vetor.rs` (novo — os vetores da porta CPU MUDARAM-SE para lá: `contorno_cpu.rs`
  `686 → 613`).
- `ph2d-app-motion`: `motion_state_pilha_demo.rs` (`Medida`/`medida_de`/
  `build_com`; a cena do smoke ao bit), `motion_state_traco_esticado_demo.rs` (`PH2D_TRACO_ESTICADO_PONTA`),
  `motion_shape_gen.rs` (uma linha: a ponta e a junta), `motion_shape_traco.rs` (`ponta_e_junta`), sondas e gates
  novos (`*_tests.rs`, `motion_shape_placa_gpu_despachos_probe.rs`).
- `ph2d-motion-doc`: um teste no `lib.rs`. Ferramentas: `mede_formas_na_placa.sh` (a `=114`, `COLIDE`),
  `mutacao_a_onda_4_2026-10-05.py`.
- **Foundational tocado:** nenhum. **Shell:** não tocada. **Contratos congelados:** nenhum.
- **Env novas (portas de medição):** `PH2D_TRACO_ESTICADO_PONTA=0|1|2` · `PH2D_CONTACT_JACOBI=<it>` ·
  `PH2D_CONTACT_CORES=1` · `PH2D_PILHA_LADOS`/`PH2D_PROVA_TIQUE` (só sondas).
- Memória: três entradas na família régua (`193`): a recusa medida sobre a metade barata de um passo · a porta que
  cresce a população e não o recipiente · a sonda de simulação sem o `advance_tick`.

**Mutações:** [`mutacao_a_onda_4_2026-10-05.py`](../ferramentas/mutacao_a_onda_4_2026-10-05.py) — **`12/12` sangram**
(o `i1` e o `i3` sobreviveram à 1.ª corrida e viraram gate: a lista das restrições, `lo < hi`, a fixtura sem a peça
grande).

**Fecho corrido (gate batched, HEAD `2b79379dc`, e as quatro falhas dele corrigidas e re-corridas sobre o fim):**
`nextest-impacted` (`BASE=a46c4c200`) `17 420/17 422` — os `2` vermelhos: o tecto de LOC (`motion_state_pilha_demo_tests.rs`
`744` → as sondas mudaram-se para `motion_state_pilha_demo_sondas.rs`, `528` + `223`; o gate re-corrido verde) e o flake
de relógio `ph2d-physics-ecs` `the_cost_of_a_player_is_linear…` (crate não tocada; verde `3/3` sozinho a `load 30`–`34`)
· `check --workspace --all-targets` `-D warnings` ✓ (depois de um `///` sobre um `thread_local!` virar `//`) · clippy
`--all-targets --all-features -D warnings` das `7` crates ✓ · `fmt` ✓ · `machete`/`standalone`/`workflow` ✓ · censos
`114/114` ✓ · `ph2d-shape-gpu --test it --ignored` `14/14` iGPU e RTX · `motion_shape_placa` (gates) `7/7` nas duas ·
`ph2d-gpu-cook formas` `2/2` nas duas · `ph2d-vec-render --ignored` `6/6` nas duas.

**Smoke (o dono), fotografado** (`target/prova/onda4/smoke/`, `smoke`, a `1930 × 1040`): a `=127` tracejada com a
ponta REDONDA à partida (`59` fps) e a `=114` com `1 024` peças por taça (`59` fps; ⚠️ no roteiro de foto a cena fica
no instante de partida — a queda não se fotografa nesta janela, §9.19 correcção).

Binário: `rm -rf target/*/incremental` e o build `smoke` do `ph2d-host-desktop` 2× (a 2.ª `Finished … in 0.20s`, zero
«Compiling»). Perfil do loop (`agent-loop-profile.sh`, `20` sessões): paralelismo `1,13`/passo ✗ · respostas por
sessão `265` ✓ · test:check `3,5×` ✗ · Edit `34 %` ✗ · contexto relido `466` mil ✗ · contexto inicial `63` mil ✓.

**06/10 — depois do smoke do dono** (smoke 1 ✅; smoke 2: *«FPS cai para 10 ou menos»*): o smoke de desempenho tinha
ido em `--profile smoke` e a régua (`[frame]`, `120` quadros) escondia o instante denso. Régua nova `[motion-quadro]`
(`PH2D_MOTION_RELOGIO=1`, `motion_bridge_relogio.rs`, porta de medição) e a montagem dos pares dos impulsos em
PARALELO (`ph2d-contact/src/impulso.rs`, os mesmos bits): `release` `1 024` por taça `60` fps em todas as janelas;
`smoke` `23`–`32` fps no instante denso. A porta `PH2D_PLAY` (que a 1.ª correcção dava por necessária) era
desnecessária: a demo já entra em Play. Superfície nova: `motion_bridge.rs` (`+6` linhas: `relogio::comeca`/`regista`)
e `motion_bridge_relogio.rs` (novo); `impulso.rs` usa `par_preenche_em_blocos`.

**06/10 — o oráculo e a DECISÃO DO DONO.** Depois do smoke (*«estamos muito aquém da performance de uma Unity»*): a mesma
pilha no `rapier2d` da casa é `~20`–`30×` mais rápida que o nosso contacto (doc 121 §9.19, o oráculo), por algoritmo.
O dono escolheu: **a colisão das formas do Motion passa a usar o motor da casa (rapier)**. O item aberto deixa de ser
«o contacto na placa» e passa a ser esta troca — prompt:
[`HANDOFF_CONTINUACAO_line_motion_value_2026-10-06_CONTACTO_RAPIER.md`](HANDOFF_CONTINUACAO_line_motion_value_2026-10-06_CONTACTO_RAPIER.md)
(o da placa fica como referência, a reavaliar com os números novos).

## §13 — A 5.ª onda (06/10): o contacto das formas pelo motor da casa (`rapier2d`)

Ordem do dono (06/10): *«usar o motor da casa»*. Desenho (escrito antes, medido no oráculo), prova, números e as
correcções que a medição impôs: [doc 121 §9.20](../121_as_formas_na_placa.md). Commits `e78b2c096` (construído, com
a lei de antes atrás de uma chave por fio para a prova) · `594a859fd` (a chave e os relógios saem) · `e21fd3195` (o
que ficou sem chamador na `ph2d-contact` sai, `−1 414` linhas) · os seguintes (docs, gates, memória). Base
`a46c4c200` (o main não andou).

| item do prompt | o que ficou |
|---|---|
| **(1) o desenho** | ✅ medido ANTES no oráculo (`ferramentas/oraculo_rapier_pilha_quente_frio/`): mundo PERSISTENTE (frio custa `10×` e agita `3×`), a taça DENTRO do mundo (fora: `8×` mais agitada), `1` passo por tique. O mundo é a MEMÓRIA do nó (`Memo`) no `Cook`, e viaja no ponto de recuo (`MEMO_A_CADA = 8`). Sincronia por `id`; o stream é a verdade do estado, o mundo a dos contactos |
| **(2) a prova** | ✅ `prova_do_mundo_de_contacto` (as duas leis no mesmo processo, pela porta do app): `4 096` por taça `17,9 → 4,30` ms de mediana (p95 `44 → 7,5`: o critério `≤ 5` passa na mediana e não no p95), `1 024` `10,8 → 1,06`; a pilha dentro da banda e menos funda (`41 % → 9,9 %` a `1 024`); determinismo ✓ |
| **(3) o produto** | ✅ o `sim.step` resolve pelo `ph2d-contact-world`; o `sim.collide` declara-se ao mundo (aperto de mão por colunas); o rolamento em duas fases; `ESTABILIZACOES = 4`; a `=114` a `1` sub-passo; as portas Jacobi/cores e os impulsos saíram |
| **(4) a medição no app** | ✅ `release`, `[motion-quadro]`: `1 024` e `4 096` por taça **`59`–`60` fps em todas as janelas, nas duas placas** (antes `13`–`25` a `4 096`); `16 384` `10`–`36` fps |
| **(5) o item da placa** | ⏳ **NÃO fecha:** a CPU com o rapier não chega a `16 384` com folga (`14` ms de mediana). A pergunta seguinte é o rapier `parallel` (oráculo: `16 384` `~17 → ~6` ms) e o DETERMINISMO dele, antes de qualquer contacto no dispositivo |

**⚠️ O que mudou de COMPORTAMENTO (vai ao dono no smoke):** a pilha encosta pela face e as peças já não entram umas
nas outras aos milhares (`41 % → 10 %` do lado); uma caixa que bate de FACE devolve `88 %` do `Bounciness` (a lei do
Box2D: dois pontos numa passagem; um disco devolve `100 %`); o `Rolling` agora TRAVA a peça parada (rampa: segura
enquanto `tg θ ≤ Rolling`) e acalma a QUEDA — a pilha de caixas sem ele já assenta sozinha; peças nascidas
sobrepostas separam-se em `~1` s por uma mola (antes: num passo).

**Superfície de colisão nova (o integrador mede):**
- **Foundational (append-only, Modo L):** `ph2d-nodegraph` — `cook_checkpoint.rs` (NOVO: o `CookCheckpoint`, o
  `checkpoint`/`restore` MUDARAM-SE de `cook.rs` para lá, mais `Memo`/`DynMemo`/`has_memo`), `cook.rs` (o campo
  `memo`, `take`/`keep` à volta do `op.eval`, a poda no `advance_tick`), `cook_eval_ctx.rs` (`memo`, `take_memo`,
  `keep_memo`). ⚠️ Quem construir `EvalCtx { … }` literal noutra linha não compila sem o campo `memo`. Contrato
  congelado: nenhum (ADR-0039 congela `NodeOp`/`OpResolver`/`NodeManifest`, não o `EvalCtx`).
- `ph2d-eval-motion`: `checkpoint.rs` (`MEMO_A_CADA`, `CheckpointRing::regista` — a porta ÚNICA do registo),
  `lib.rs`/`scrub.rs` (os dois sítios do registo passam por ela).
- **Crate NOVA:** `crates/ph2d-contact-world` (depende de `rapier2d 0.35` com as features da `ph2d-physics`;
  `dhat` em dev). `Cargo.lock` muda (a crate e o `dhat` na lista dela).
- `ph2d-contact`: `obstaculo.rs` (novo); `impulso*.rs` APAGADOS, `atrito.rs`/`laco.rs`/`lib.rs`/`tests.rs` aparados.
- `ph2d-node-sim-step`: `lib.rs` (`step_com` com o mundo; a metade angular ANTES do contacto), `contact.rs`
  (reescrito: uma porta fina), `contact_tests.rs` (cinco gates re-expressos), `Cargo.toml`.
- `ph2d-node-sim-collide`: `lib.rs` (a chave, `collide_com_chave`), `mundo.rs`/`geometria.rs`/`mundo_tests.rs`
  (novos; o `contact` da geometria MUDOU-SE para `geometria.rs` pelo tecto de LOC).
- `ph2d-app-motion`: `motion_state_pilha_demo.rs` (`SUBSTEPS = 1`), os gates da pilha e da `=115`, as sondas,
  `motion_state_pilha_demo_recuo_tests.rs` (novo).
- **Env:** saíram `PH2D_CONTACT_JACOBI`/`PH2D_CONTACT_CORES`. Nenhuma nova no produto. **Shell:** não tocada.

- `ph2d-contact`: `colisores_locais` (nova porta irmã da `colisores`, sem o `rot`); o censo
  `todo_leitor_do_colisor_declarado_se_regista` passa a ter `ph2d-contact-world → sim.step` no lugar da `ph2d-node-sim-step`.
- Ferramentas: `oraculo_rapier_pilha_quente_frio/` (novo), `oraculo_rapier_pilha/` (o `main.rs` foi para `src/`, o
  `cargo machete` lia uma pasta `src` inexistente), `mutacao_a_onda_5_2026-10-06.py`.
- `project-memory`: três lições nas famílias; o índice `22 065 → 21 956` bytes (estava acima do tecto do gate desde
  antes desta onda — saiu um parêntese de história).

**Mutações:** [`mutacao_a_onda_5_2026-10-06.py`](../ferramentas/mutacao_a_onda_5_2026-10-06.py) — **`13/13` sangram**
(pré-voo `13/13`, corrida limpa `58` verdes; `target/prova/onda5/mutacao.txt`). Na 1.ª corrida **`m7`** (a trava
estática nunca fecha) e **`m13`** (o atrito do par por `Max`) SOBREVIVERAM: a bola da rampa parecia presa pelo travão a
rolar (a barra `0,15` passou a `0,035` entre os dois lados MEDIDOS: com a trava `0,020`, sem ela `0,061`), e nenhum gate
tinha atritos DIFERENTES nas duas peças (novo: `ice_slides_over_sandpaper…`, com a gravidade a cada tique — o `corre`
dos gates do passo consome o `accel`). ⚠️ Sem gate: o espaçamento `MEMO_A_CADA` (só custo, nenhum bit muda).

**Fecho corrido** (HEAD `d9f3e413b`, árvore limpa; a 1.ª corrida foi com a árvore a mudar e não vale):
`nextest-impacted` (`BASE=a46c4c200`) **`18 768/18 768`** · `check --workspace --all-targets` (`CARGO_BUILD_WARNINGS=deny`)
✓ · clippy `--all-targets --all-features -D warnings` das `7` crates ✓ · `fmt` ✓ · censos `114/114` ✓ · workflow,
standalone, `adr-index`, `doc-index`, `archive-index`, `machete` ✓ · `tests/it` contact-world `1/1` (os bytes:
declarados `3 724 640` contra `1 659 885` alocados), eval-motion `12/12`, nodegraph `13/13`. Depois do fecho: os dois
gates das mutações sobreviventes (o passo `35/35`).

**Smoke (o dono), fotografado** (`target/prova/onda5/smoke/`, `release`, `1930 × 1040`): a `=114` com a pilha da
direita encostada e tombada, a da esquerda um borrão, `60` fps (`c114_e12.png`); a `=114` com `4 096` por taça a `60` fps
na régua (a pilha cai para fora do enquadramento — a taça cresce com ela; o passo 4 do smoke manda afastar a câmara).
Binário: `rm -rf target/*/incremental` e `cargo build -p ph2d-host-desktop --release` 2× (a 2.ª `Finished … in 0.22s`,
zero «Compiling»).

**⏳ O QUE FICA ABERTO:**
1. **O rolamento numa PILHA de discos com `Rolling` BAIXO** (`0,1`): na queda gira `61,7°` contra `10,7°` sem o botão
   (a trava alterna trancar e destrancar). Com `0,75` acalma ao zero; numa bola sozinha a lei é exacta. A cura de raiz
   é o rolamento DENTRO do solver (o rapier `0.35` não tem; o Box2D v3 tem) — doc 121 §9.20 ponto 5.
2. **O contacto aos `16 384` por taça** (`10`–`36` fps): a próxima alavanca medida é o rapier `parallel` (oráculo
   `~3×`) e o DETERMINISMO dele (a feature unifica-se com a `ph2d-physics`); o item da placa NÃO fecha.
3. **O p95 do tique a `4 096`** (`7,5` ms contra o critério `≤ 5` escrito antes): a mediana passa (`4,3`) e o app fica
   nos `60` fps; o p95 é o do próprio rapier no instante denso.
4. **O comportamento que mudou** (a pilha encosta pela face e quase não se sobrepõe; o `Bounciness` de uma caixa de
   face devolve `88 %`; o `Rolling` trava a peça parada) — ⚠️ vai ao dono no smoke; integrar só depois do smoke dele.

**A UMA LINHA para o `CLAUDE.md` §5 (o integrador aplica):** em *Motion Nodes*, trocar o «Último» por
`[handoff 05–06/10](docs/Motion%20Nodes/handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-10-05.md)` e acrescentar
«o contacto das formas é o `rapier2d` da casa (`ph2d-contact-world`)».

**Perfil do loop** (`agent-loop-profile.sh`, `20` sessões): paralelismo `1,11`/passo ✗ · respostas por sessão `254` ✓ ·
test:check `3,2×` ✗ · Edit `33 %` ✗ · contexto relido `405` mil ✗ · contexto inicial `62` mil ✓.

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
desta onda `7257b737e` · `cd5f23179` · `bdaf1f8fa` · (docs) · `6ca487037` · este.

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
