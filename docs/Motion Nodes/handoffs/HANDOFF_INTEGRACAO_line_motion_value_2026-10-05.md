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
| **(5) o contacto em Play** | ✅ medido e **curado na CPU**: o `monta` dos impulsos era `O(n²)`; pelos pares da grelha (os mesmos bits) `1 024` peças por taça `247` ms → `60` fps. ⏳ **ABERTO: o contacto da caixa na placa** — a recusa expirou (a CPU paga `48`–`93` ms por tique a `4 096` por taça com a pilha formada; ⚠️ a régua do APP em Play não funcionou nesta janela — a cena não cai no roteiro de foto — e as tabelas do app mediram a cena parada, doc 121 §9.19 correcção). Modelo de LEI provado (Jacobi refutado: pilha `1,85×` mais agitada; Gauss–Seidel por cores na banda da ordem) e de CUSTO (`1,6` µs por despacho num passe); não construído nesta janela (quatro peças, duas topologias novas do dispositivo). Prompt: [`HANDOFF_CONTINUACAO_line_motion_value_2026-10-05_CONTACTO_NA_PLACA.md`](HANDOFF_CONTINUACAO_line_motion_value_2026-10-05_CONTACTO_NA_PLACA.md) |

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
