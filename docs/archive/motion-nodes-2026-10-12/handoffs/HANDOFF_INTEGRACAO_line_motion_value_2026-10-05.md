# ARQUIVO — HANDOFF_INTEGRACAO_line_motion_value_2026-10-05.md (história, 218 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`HANDOFF_INTEGRACAO_line_motion_value_2026-10-05.md`](../../../Motion%20Nodes/handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-10-05.md) em 2026-10-12, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-165,384-576` do original.
>
> ⚠️ **A única alteração ao corpo:** 9 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

## §11 — A 3.ª onda (05/10): (E) a emissão por peça e (C) a mordida do traço rente

Plano, kill-criteria e números: [doc 121 §9.18](../../../Motion%20Nodes/121_as_formas_na_placa.md) (o (E) e o (C) por baixo do plano).
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

**Mutações** ([`mutacao_a_mordida_do_traco_rente_2026-10-05.py`](../../../Motion%20Nodes/ferramentas/mutacao_a_mordida_do_traco_rente_2026-10-05.py),
sozinho na árvore, pré-voo `12/12`): **`12/12` sangram** na família nova (a lista no doc 121 §9.18). A `m6` (a emenda
nunca) sobreviveu à 1.ª corrida — nenhuma forma da família tinha dois contornos; a engrenagem com furo entrou e ela sangra.

**Smoke (o dono):** ✅ **APROVADO em 05/10.** Ordem do dono: *integrar só quando não houver nada em aberto* — a 4.ª
onda fecha o §11 «Aberto» e o resto ([`HANDOFF_CONTINUACAO_line_motion_value_2026-10-05_TUDO.md`](../../../Motion%20Nodes/handoffs/HANDOFF_CONTINUACAO_line_motion_value_2026-10-05_TUDO.md)). Fotografado a `1930 × 1040` (RTX), as duas rotas: a placa (`[formas] 1024 copias de 1
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

Plano, kill-criteria (escritos antes) e números: [doc 121 §9.19](../../../Motion%20Nodes/121_as_formas_na_placa.md). Commits `3bb13c95a` …
(base `a46c4c200`: o main andou um commit, o guarda R4; rebase sem conflito).

| item | o que ficou |
|---|---|
| **(1) o passeio em paralelo** | ⛔ **RECUSA MEDIDA.** `cs_trocos` (um grupo por cópia, prefixos segmentados): a prova `0,134` ms (critério `≤ 0,13`), o produto `0,226` (critério `≤ 0,15`), densas `+22 %` iGPU / `+16 %` RTX; a imagem passava (alfa `1`). O chão passou a ser a EMISSÃO por peça (`0,092`). Código medido em `2774e1d17`, retirado em `c524f9b3d` |
| **(2) a parede do Vello** | ⛔ **RECUSA MEDIDA.** O `U` (um caminho por traço) já era o estado; a FITA (`U+A`) não funde nenhum pedaço (os dois lados do vértice calculam a faixa em ordem diferente), e a rota pela lei manda `2,45×` os segmentos (`98` por cópia contra `40`): parede iGPU `+24`/`+42 %`. `e115108de` → `274ed670e`. A sonda conta caminhos e segmentos |
| **(3) a ponta e a junta no cartão** | ✅ **CONSTRUÍDO.** `stroke_cap`/`stroke_join` (`Enum`, omissão Butt/Miter = o de sempre), no `ALL` e no `SPECS`, i18n, `build_shape_path` → `StrokeSpec`. Gates do nó, do cartão, da gravação (ida e volta e projecto antigo) e da IMAGEM (`3 × 3`, placa × rota Vello alfa `≤ 1`). Contrato `NodeManifest=8` intocado (ADR-0039 lido) |
| **(4) a prova do SIGSEGV** | ✅ **PROVADA.** `40` corridas alternadas na RTX: antes da cura `14/20` `SIGSEGV`, depois `0/20` (carga `3`–`38`) |
| **(5) o contacto em Play** | ✅ medido e **curado na CPU**: o `monta` dos impulsos era `O(n²)`; pelos pares da grelha (os mesmos bits) `1 024` peças por taça `247` ms → `60` fps. ⏳ **ABERTO: o contacto da caixa na placa** — a recusa expirou (a CPU paga `48`–`93` ms por tique a `4 096` por taça com a pilha formada; ⚠️ e o quadro do app no instante denso da pilha: `release` `1 024` por taça `60` fps depois da montagem dos pares em paralelo, `4 096` `13`–`25` fps; régua `[motion-quadro]`, doc 121 §9.19). Modelo de LEI provado (Jacobi refutado: pilha `1,85×` mais agitada; Gauss–Seidel por cores na banda da ordem) e de CUSTO (`1,6` µs por despacho num passe); não construído nesta janela (quatro peças, duas topologias novas do dispositivo). Prompt: [`HANDOFF_CONTINUACAO_line_motion_value_2026-10-05_CONTACTO_NA_PLACA.md`](../../../Motion%20Nodes/handoffs/HANDOFF_CONTINUACAO_line_motion_value_2026-10-05_CONTACTO_NA_PLACA.md) |

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

**Mutações:** [`mutacao_a_onda_4_2026-10-05.py`](../../../Motion%20Nodes/ferramentas/mutacao_a_onda_4_2026-10-05.py) — **`12/12` sangram**
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
[`HANDOFF_CONTINUACAO_line_motion_value_2026-10-06_CONTACTO_RAPIER.md`](../../../Motion%20Nodes/handoffs/HANDOFF_CONTINUACAO_line_motion_value_2026-10-06_CONTACTO_RAPIER.md)
(o da placa fica como referência, a reavaliar com os números novos).

## §13 — A 5.ª onda (06/10): o contacto das formas pelo motor da casa (`rapier2d`)

Ordem do dono (06/10): *«usar o motor da casa»*. Desenho (escrito antes, medido no oráculo), prova, números e as
correcções que a medição impôs: [doc 121 §9.20](../../../Motion%20Nodes/121_as_formas_na_placa.md). Commits `e78b2c096` (construído, com
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

**Mutações:** [`mutacao_a_onda_5_2026-10-06.py`](../../../Motion%20Nodes/ferramentas/mutacao_a_onda_5_2026-10-06.py) — **`13/13` sangram**
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

