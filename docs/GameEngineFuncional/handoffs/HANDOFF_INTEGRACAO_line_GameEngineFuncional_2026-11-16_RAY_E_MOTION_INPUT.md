# HANDOFF de integração — `line/GameEngineFuncional`, 16/11 (GEF 07 Ray Sensor · GEF 08 Motion Input ondas 1–3 · fecho)

Leitor: o agente integrador. Branch `line/GameEngineFuncional`, HEAD `bd8745765b`, merge-base com o `main` = `dc3d4f9fb8`. **111 commits, 447 arquivos** (+21 483 / −3 567). Nada integrado, nenhum push. Suíte completa e `nextest-impacted` NÃO correram (regra do dono 23/10): são do integrador. A linha FECHA aqui.

**Ordem do dono, 16/11: «smoke apenas depois da integração; escreva handoff de integração; não integre».** O smoke da onda 3 do Motion Input (§9) é feito pelo dono DEPOIS de integrar: o integrador compila o binário smoke no repo primário e entrega-lho. Os smokes anteriores (Ray 15/11, MI ondas 1 e 2 em 16/11) já foram OK.

Continuações que este handoff substitui (mecanismo de cada wave vive lá, não repito):
- GEF 07 Ray Sensor: [`HANDOFF_CONTINUACAO_2026-11-15_07_RAY_SENSOR.md`](HANDOFF_CONTINUACAO_2026-11-15_07_RAY_SENSOR.md) · [`PROMPT_…RAY_MELHORIAS`](PROMPT_CONTINUACAO_2026-11-15_RAY_MELHORIAS.md) · fichas [`07_ray_sensor.md`](../componentes/07_ray_sensor.md), [`07_auditoria_2026-11-15.md`](../componentes/07_auditoria_2026-11-15.md), [`07_auditoria_arrumacao_2026-11-15.md`](../componentes/07_auditoria_arrumacao_2026-11-15.md).
- GEF 08 Motion Input: [`HANDOFF_CONTINUACAO_2026-11-16_08_MI_ONDA3.md`](HANDOFF_CONTINUACAO_2026-11-16_08_MI_ONDA3.md) · ficha [`08_motion_input.md`](../componentes/08_motion_input.md) · maquete [`08_onda3_maquete.png`](../componentes/08_onda3_maquete.png).

## 1. O que entrou

1. **GEF 07 Ray Sensor** (15/11, seis ondas + auditoria de 2 lentes): o raio é um OBJETO filho (como a forma do ADR-0199), cartão versão B, depois C (grupo Filter, See all hits, Ignore up to com conta-gotas); melhorias RS-1..RS-12 (Enabled, Shape Line · Thick · Cone, Aim at, Reach | Aim at, cartão arrumado por pergunta); cena `PH2D_GAME_SMOKE=26` (+ `PH2D_RAY_SMOKE=1`); o Play no editor dá o teclado ao jogo. Crate nova `ph2d-ray-sensor`. Smoke do dono OK 15/11.
2. **GEF 08 Motion Input** (15–16/11): leva um valor do jogo a um efeito do Motion (nó `value.input`). Onda 1: cartão B, cena 27 (`PH2D_GPU_COOK_DEMO=134`), mutação 12/12. Onda 2: motor (From, Speed, Grounded, Action, Property, Remap+curva, Smooth), cena 27 com leitores sem script, mutação 32. Onda 3 (decisão do dono 16/11: 1B filho · 2A nó Send signal · 3A origem decide o par): MI-10 entrada = objeto filho «Motion Input» · MI-9 `motion.get(nome)` no Luau · MI-5 par (x, y), fontes Velocity e Move · Property só oferece o que lê · MI-8 nó `value.send_signal` (Pulse · Rises above · Falls below) · cook GPU com fronteira por (nó, porta) · cena 28 «o herói manda» + demo 143. Crate nova `ph2d-node-send-signal`.
3. **Fecho da linha** (16/11, auditoria de 2 lentes), curado: G1 `safe_clamp` em `checkbox/mark.rs` · G2 tectos de ficheiro (`scene_tests.rs` 748→626, gpu-cook `lib.rs` 709→696, tokens `color.rs` 701→697) · D1 renomear o pai a meio do Play não congela a linha · D2 Velocity/Move no catálogo `source` + gate catálogo × `MotionSource::ALL` (schema NÃO sobe) · D3 rótulo automático só sobre nome automático · D5 sem filho vazio se a anexação falha · U1 Cone esconde «See all hits» · U2/U3 Send signal nasce em Falls below 0.5, Level 0..1 e ±2^24, «Sent» conta cada sinal · U4–U7 textos, Move, par por i18n. Commits `9b6b1b40a6`…`bd8745765b` (`git log 045e8a7430..HEAD`).

## 2. Foundational tocado (todo ADITIVO) — `git diff --stat main..HEAD`

| Crate | Mudança | Por que é aditivo |
|---|---|---|
| `ph2d-ecs` (8 arquivos) | `motion_input::dono` (`from = 0` = o pai), `MotionInputs` com vetor, `Property`/`property_ref` leem Transform pelo PropKind, `SignalOrigin::Motion { source }` | variante nova de enum e funções novas; **registo de componentes inalterado (121/1/1)** |
| `ph2d-runtime` (2) | `SignalOrigin::Motion` no outbox; `motion.get`/par em `motion.set` | consumidores leem por cursor; origem nova só é emitida pelo Send signal |
| `ph2d-core` | `Playhead::crosses_forward` (+24) | função nova |
| `ph2d-nodegraph` | `external.rs`: `INPUT_XY = "xy"` (+2) | constante nova; **`node.rs` intocado** (gate `architecture_contract_surface`) |
| `ph2d-eval-motion` | `boundary_port_streams` (+14) | ao lado de `boundary_streams` |
| `ph2d-gpu-cook` (5) | fronteira CPU→GPU entregue por **(nó, porta)**: `set_boundary_ports`, cache por (nó, porta), regra de recuo removida de `plan_uniao.rs`, `BoundaryMismatch` se faltar porta | muda o caminho interno, não a API pública; prova nos 2 testes de placa (§5) |
| `ph2d-component-desc` (3) | catálogo `logic.rs`, `physics*.rs`, `field.rs` (+3) | campos novos no fim |
| `ph2d-i18n` (13), `ph2d-tokens` (3) | textos do Ray/Motion Input; token `obj-motion-input`, `obj-ray-sensor` | só chaves/tokens novos (tokens `color.rs` 701→697) |

## 3. Superfície de colisão (`bash scripts/collision-surface.sh`, 16/11)

- **Schema:** `PROJECT_SCHEMA` **243 → 248** (DELTA +5), em `shells/desktop/src/project_schema.rs`; degraus 244 (raio objeto + versão C), 245 (melhorias), 246 (MI onda 2), 247 (filho), 248 (fontes Velocity/Move). **A tripla do gate é (248, 13, 29) mas a escada NÃO a vê** — reconte pelo `main` do dia: `python3 scripts/schema-recount.py`; os três sítios moram em arquivos irmãos. Sem leitura dos formatos anteriores (§0.11). `VEC_SCENE_SCHEMA` 29, `FLIP_SCHEMA` 13, `DOC_VERSION` 25 iguais.
- **Registos:** `ph2d-ecs` 121 · render 1 · script 1, iguais à base.
- **Contratos §6:** `ph2d-nodegraph/src/node.rs` e `ph2d-editor-core/src/tool.rs` INTOCADOS; os 4 gates de contrato verdes (nodegraph, tool, vector, imageio 6/6).
- **ADR:** nenhum criado (último no disco 0205, próximo livre 0206).
- **Cargo.lock:** `+name` só `ph2d-node-send-signal` e `ph2d-ray-sensor` (internas).
- **Marcadores de conflito:** nenhum. **Tectos de LOC:** nenhum arquivo da linha acima de 700.

## 4. Contadores que SOMAM — o integrador RECONTA (DELTA da linha)

| Contador | Linha | Nota |
|---|---|---|
| `PROJECT_SCHEMA` | 243 → 248 (+5) | reconte com `schema-recount.py` |
| `PH2D_GAME_SMOKE` | cenas até **28** (26 Ray, 27 Motion Input, 28 o herói manda) | numeração CONTADA (`cenas_do_smoke`: game-scenes 9, app-macros 2); outra linha com cena 26+ reconta |
| `PH2D_GPU_COOK_DEMO` | até **143** (134 cena 27, 143 cena 28) | CLAUDE.md §5.1 Motion Nodes já diz «até 143» |
| Crates novas | 2 (`ph2d-ray-sensor`, `ph2d-node-send-signal`) | registo de nós: `value.send_signal`, categoria Output, marchado como `pulse.signal` em `signal_nodes` |
| Folha de conferência | linha nova em `12_pulse.md` (Send signal) | gate `every_node_has_a_conference_row_or_is_named_in_the_debt` verde |
| LOC (`find crates/<c> -name '*.rs' \| xargs cat \| wc -l`, a árvore da crate inteira) | `ph2d-app-components` **80 861** (main 80 225) · `ph2d-app-motion` **197 506** (main 197 461) · `ph2d-app-game` **2 869** (main 2 868, no tecto) | tectos somam entre linhas: se outra linha crescer essas crates, cure CORTANDO, nunca subindo o número |
| Shell (`find shells/desktop -name '*.rs'`) | 188 962 (main de hoje 188 974): **−12** líquidas | tecto da shell NÃO mexido; `the_shell_only_shrinks` verde |
| Registo de componentes | 121/1/1, DELTA 0 | |
| Tokens/i18n | `obj-motion-input`, `obj-ray-sensor`; chaves `motion_input.*`, `ray.*` | |

## 5. Prova de fecho (todos ✓ por nome, 1× no fecho)

- clippy `--workspace --all-targets --features ph2d-spike/bevy_ecs -D warnings` (como `ship.sh`) · `cargo fmt --all -- --check`.
- Arquitetura: `the_shell_only_shrinks` · `workspace_src_files_under_loc_cap` · `file_loc_caps` · `fn_loc_caps` · `clamp_calls_in_ui_are_safe_or_justified` · `no_new_hardcoded_ui_strings_in_widgets` · `architecture_claude_md_cabe_no_orcamento` · `o_indice_da_memoria_conta_o_que_aponta` · `a_schema_bump_anywhere_must_bump_the_project_schema` · 4 gates de contrato · `every_offered_node_has_a_name_and_every_fixture_has_none` · `every_node_has_a_conference_row_or_is_named_in_the_debt`.
- Cenas/jogo: `cenas_do_smoke` · `jogo_nomes_do_smoke` · `jogo_heroi_sinal` · `seam_ray` · `a_seccao_motion_input` · `motion_input` (app-components 27, game-scenes 14) · `ph2d-node-send-signal` inteira (10) · `heroi_demo`.
- **Placa** (adaptador RADV integrado): `o_par_e_a_cor_chegam_a_placa_como_na_cpu` · `o_send_signal_pela_placa_cai_uma_vez_e_fala_pelo_objecto` (gpu-cook, `tests/it/fronteira_de_porta.rs`).
- **Mutação:** onda 1 12/12; onda 2 32; onda 3 47 mutantes, 38 sangraram, os 3 só-de-placa corridos à mão sangraram, 2 equivalentes, 1 fraco descartado (detalhe no handoff da onda 3 e na ficha).
- Binário smoke compilado 2×; a 2.ª: `Finished smoke profile [optimized] target(s) in 0.22s`, zero `Compiling`.
- NÃO corrido (do integrador): suíte completa, `nextest-impacted`, `cargo check --target aarch64-apple-darwin`, paridade GPU×CPU inteira (ver Aberto n.º 2).

## 6. O que um merge pode partir

- ⚠️ **Conflitos de COMENTÁRIO esperados.** Para caber nas catracas a linha cortou SÓ comentário `//` histórico (zero código) em muitos ficheiros de `ph2d-app-components` (onda 2: ~482 linhas em 11; `831043380b`: 17; `component_attach.rs` 45), `ph2d-app-motion` (`motion_shape_gen.rs`, `motion_bridge_intents.rs` 344 linhas, `motion_bridge_signals.rs`, demos `motion_state_*`, `motion_scene_ctx.rs`), `ph2d-app-game` (`smoke.rs`, `posse_do_jogo.rs`, `encaixe.rs`, `smoke_moedas.rs`), `ph2d-game-smoke-fotos/nomes.rs`, tokens `color.rs`. Lista exata: `git show --stat` de `831043380b`, `17a2c2053a`, `045e8a7430`. Em conflito vale o lado do `main` desde que o código case.
- **Degrau do schema** em 3 sítios (§3): um degrau escrito no arquivo errado funde limpo e evapora.
- **Registo de nós** (crate nova) e **censo de sinais** (`SignalOrigin::Motion`; `value.send_signal` conta como quem grita).
- **Cartões:** `ph2d-panel-inspector` `seam_ray`/`projectile_*`, `cards_*` do Ray e do Motion Input; ordem do grupo Layers do raio é Layer · Sees · botões (decidida em `09582fe754`): testes de outra linha que assumam a ordem velha reprovam.
- **O filho Motion Input não tem Transform/Ordering/gizmo/arrasto:** código novo de outra linha que varra «todos os filhos» deve tolerar isso.
- **Cook GPU:** quem depende de `boundary_streams` (só a porta 0) ou da regra de recuo de `plan_uniao.rs` quebra; um nó GPU que lê `value.input.xy` ou `.r` agora FICA na placa.
- Rótulos/catálogo: `source` do catálogo ganhou Velocity/Move (gate catálogo × `MotionSource::ALL`).

## 7. ABERTO

1. Nomes repetidos «Motion Input · speed» entre heróis diferentes: campos lidos pelo nome ficam ambíguos (baixo).
2. **3 testes de paridade GPU×CPU vermelhos JÁ antes da onda 3** (medido revertendo o commit da fronteira): `gpu_cpu_parity::the_projection_angle_arm_matches_the_cpu_within_epsilon`, `gpu_cpu_parity_sim::the_emitter_burst_sim_matches_the_cpu`, `gpu_cpu_parity_strobe::the_shaped_strobe_matches_the_cpu_tick_by_tick` (ph2d-gpu-cook, `--ignored`, RADV). O integrador confere no `main`: se o `main` também os tem vermelhos, não são desta linha.
3. Doc-comments «só a porta 0» velhos (`boundary_streams` em `ph2d-eval-motion`; doc 120 §8.3.3) e a sonda `sonda_os_defeitos_latentes_da_entrega` (parte (a) obsoleta).
4. Cena 28: o rastro nasce num ponto fixo (a legenda diz «empurradas pela velocidade»); foto `d` sem o nó Send signal aberto (o editor de nós abre pequeno atrás da Timeline); sem «mira»/Grounded.
5. Send signal publica também com a Timeline a tocar em Edit (como o `pulse.signal`).
6. Herdados: `oraculo_lsp` G5b do Ray (`luau-lsp` não instalado) · galáxia da cena 27 por cima do Hit Dummy · o handoff de continuação GEF 07 (25,8 KB) passa de 15 KB e fica assim (é continuação; não foi cortado nesta janela).
7. O cabeçalho do handoff GEF 07 ainda diz «Smoke das curas: PENDENTE»; os commits `5546c3d28f` e `56388e8c8a` registam smoke do dono OK 15/11 (o `07_ray_sensor.md` tem «Falta: o smoke do dono» em 2 secções velhas). O smoke 2 abaixo é uma confirmação curta, opcional.

## 8. SMOKES DO DONO (depois da integração)

Pré-requisito do integrador: compilar no repo PRIMÁRIO `cd /home/enio/Documentos/Projetos/PH2D && cargo build --profile smoke -p ph2d-host-desktop`. As fotos abaixo estão no worktree; **depois da integração o caminho é `docs/GameEngineFuncional/componentes/fotos_08/…png`** (sem o prefixo `Worktrees/line-GameEngineFuncional/`). Comparação com a maquete: [lado a lado](Worktrees/line-GameEngineFuncional/docs/GameEngineFuncional/componentes/fotos_08/onda3_lado_a_lado.png).

**Smoke 1 — o herói manda (onda 3 do Motion Input, cena 28):**

1. Comando: `cd /home/enio/Documentos/Projetos/PH2D && PH2D_GPU_COOK_DEMO=143 PH2D_GAME_SMOKE=28 ./target/smoke/ph2d-host-desktop`. Errado se: o app não abre ou a cena vem vazia.
2. Parado, olhe a Hierarquia: o **Hero** com dois filhos, **Motion Input · vida** e **Motion Input · rastro**, cada um com ícone de controle de jogo. Clique em «Motion Input · rastro»: From **«Hero (parent)»**, 10 botões de Source e **«Reads Velocity here: (0.00, 0.00)»**. Foto: [parado](Worktrees/line-GameEngineFuncional/docs/GameEngineFuncional/componentes/fotos_08/onda3_a_parado.png). Errado se: faltar um filho, o From disser outra coisa, ou o filho mostrar Transform/Ordering.
3. Clique em **Play (F5)** e **fique parado**, sem apertar nada. As pedras caem no herói, a barra verde de vida encolhe peça a peça; quando a vida passa de 0.25 a **lâmpada vermelha acende**, aparece **«Signal: low_health»** e a barra amarela (Script Meter) engorda. Fotos: [pedras e lâmpada](Worktrees/line-GameEngineFuncional/docs/GameEngineFuncional/componentes/fotos_08/onda3_c_play_pedras.png). Errado se: a lâmpada acende mais de uma vez, não acende, ou a barra amarela não muda.
4. **Ande com o herói**: setas ou **A/D** (o Hero responde a setas e WASD). Ele NÃO anda sozinho no smoke normal. Ande para a direita e para a esquerda: as estrelas voam para o lado para onde você corre, e no cartão do «rastro» o **«Sends (x, y)»** muda com a sua corrida. Foto de referência: [herói corre](Worktrees/line-GameEngineFuncional/docs/GameEngineFuncional/componentes/fotos_08/onda3_b_play_corre.png). Errado se: os números ficam em 0 com você andando (as estrelas nascerem num ponto fixo é o aberto n.º 4, conhecido).
5. Clique no **objeto Motion** (o da cena, não o filho): o cartão «Motion» diz **«Sent low_health once»**. Errado se: disser «not sent yet» depois de a lâmpada acender, ou «N times» com N maior que 1. Foto: [enviou](Worktrees/line-GameEngineFuncional/docs/GameEngineFuncional/componentes/fotos_08/onda3_c2_motion_enviou.png).
6. **Pare (Stop)** e clique no filho **«vida»**. Troque **Source** para **Property**: abre uma lista com **«Transform · Position (x, y)»** e sem opções que não leem (nada de força ou velocidade linear). Foto: [Property](Worktrees/line-GameEngineFuncional/docs/GameEngineFuncional/componentes/fotos_08/onda3_e_property.png). Errado se: a lista não abre, abre cortada ou não tem Position.
7. Troque **Source** para **Move**: o cartão diz **«Reads move_right · …»** (as ações do Input Map). Foto: [Move](Worktrees/line-GameEngineFuncional/docs/GameEngineFuncional/componentes/fotos_08/onda3_f_move.png). Errado se: aparecer outro texto. (Ctrl+Z volta a Health.)
8. Selecione o objeto Motion, aperte **Tab**, abra a paleta (categoria Output) e ponha um nó **Send signal**: mostra **Signal**, **When** (Pulse · Rises above · Falls below) e **Level**. O nó **novo nasce em «Falls below» com Level 0.5**; o Level arrasta de 0 a 1 e aceita número digitado. Foto da paleta: [Send signal](Worktrees/line-GameEngineFuncional/docs/GameEngineFuncional/componentes/fotos_08/onda3_d_send_signal.png) (o nó aberto NÃO aparece nela). Errado se: o nó não existe, ou When/Level não mudam ao clicar. Se o editor de nós abrir pequeno atrás da Timeline, é o aberto n.º 4.

**Smoke 2 — confirmação curta do Ray Sensor (opcional; o do dono foi OK em 15/11):**

1. Comando: `cd /home/enio/Documentos/Projetos/PH2D && PH2D_GAME_SMOKE=26 ./target/smoke/ph2d-host-desktop`.
2. Clique no raio em **Cone** (o do guarda): o cartão **não** mostra «See all hits» (o cone já devolve todos). Mude a forma para Line: a caixa «See all hits» volta. Errado se: a caixa aparece com Cone.
3. Confira a ordem do grupo Layers do raio: **Layer, Sees, depois os botões**. Play: o guarda e a torre reagem como na foto da cena 26 (`docs/GameEngineFuncional/componentes/fotos_07/`).
