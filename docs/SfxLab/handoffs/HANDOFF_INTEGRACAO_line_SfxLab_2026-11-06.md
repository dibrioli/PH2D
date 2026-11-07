# HANDOFF DE INTEGRAÇÃO — line/SfxLab, W3 + W4 (2026-11-06)

Leitor: o agente integrador. **Nada integrado, nada enviado, sem rebase.** Integrar é ordem do Enio ([CLAUDE.md §0.7](../../../CLAUDE.md)); «integrar não é aprovar».

## 0. Identidade e escopo

Branch `line/SfxLab`, worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-SfxLab`, base (merge-base) `bbfefaf393`, **19 commits** (`git log bbfefaf393..HEAD --oneline`), 124 arquivos, +14 055 −72. Contém:

- **W3, a tela** (aba do som, Create sound, Use in Game): smoke do dono aprovado em 06/11, dois smokes.
- **W4, Exploração** (Roll, Mutate, Lock, Next, Keep, Heard, Explore): smoke do dono aprovado em 06/11 («muito bom»), com **um defeito apontado, já curado**: arrastar um campo no painel e soltar sobre a aba sumia com o cursor (commit `436323f402`, porta comum, §2).

Mecanismo de cada onda, não copiado: [W3 plano](../04_W3_plano.md) · [W4 plano](../05_W4_plano.md) · [handoff W3](HANDOFF_CONTINUACAO_W3_2026-11-06.md) · [handoff W4](HANDOFF_CONTINUACAO_W4_2026-11-06.md) · [integração anterior W0–W2](HANDOFF_INTEGRACAO_line_SfxLab_2026-11-05.md) · [plano geral](../01_plano.md).

## 1. Superfície de colisão (`bash scripts/collision-surface.sh`, colada no fecho, 06/11)

```
merge-base bbfefaf393 · 19 commit(s) · 124 arquivo(s)
PROJECT_SCHEMA 217 (base 217) · tripla (217, 13, 29) (base igual) · VEC 29 · FLIP 13 · DOC_VERSION 25
ph2d-ecs 118 (118) · ph2d-render 119 (119) · ph2d-script 119 (119)
node.rs intocado · tool.rs intocado
ADR: nenhum criado (último 0198, próximo livre 0199)
Cargo.lock: +ph2d-app-sfxlab +ph2d-host-sfxlab-scenes +ph2d-object-picker +ph2d-panel-sfxlab
            +ph2d-sfx-explore +ph2d-sfx-explore-tela +ph2d-sfx-lab   (7 internos; nenhum externo)
Marcadores de conflito: nenhum · Tetos de LOC: nenhum arquivo da linha passa
```

Maiores pastas (`git diff --name-only`): `shells/sfxlab-scenes` 15 · `ph2d-app-sfxlab` 13 · `ph2d-editor-core` 12 · `ph2d-sfx-lab` 11 · `shells/desktop` 8 (+140 −2) · `ph2d-sfx-dsp` 8; as demais, 1 a 7 arquivos cada.

Validade: a coluna `base:` é o merge-base. Leia o `main` no arquivo (`git show main:<arq>`) e conte como DELTA.

## 2. Foundational tocado e por que é aditivo

- **`ph2d-editor-core` — a porta comum do ponteiro (a que mais atinge outras linhas).** `Documents::chrome_press: Option<PointerButton>` (`documents.rs:54`) e `screens/hero/board_view.rs` (`pointer`, `route_pointer`, `cursor`): com um botão premido no CHROME (fora do canvas e sem a descida tomada, p.ex. a bolha do Quadro), o documento ativo **não ouve o Move nem o Up desse botão e não dá o ponteiro**. Antes, `pointer` dava todo Up à aba ativa sem olhar onde foi a descida; a aba do som devolvia `corpo.contains`, o Up não chegava ao store e o cursor ficava escondido. **Atinge todas as abas de família pela mesma porta: Som, Kanban, Code, Planner, Quadro** (Pixel já tinha a lei). Aditivo: só retira eventos que o documento nunca poderia usar (a descida foi fora dele); o fluxo normal (descida no canvas) não muda. `PointerLook::Pick` é variante nova (a linha da shell que a mapeia para a cruz só compilou).
- **`ph2d-audio`:** `maior_momento` (`dsp/loudness.rs:76`), função nova; `integrated_lufs` intocado (teste: silêncio no fim não muda o maior momento).
- **`ph2d-sfx-dsp`:** `Rng` público; `Layer.variacao` entra por **XOR** no estado: com 0 os hashes dourados ficam intactos (gate de hash verde). `RenderError::SemThread` novo (variante: quem faz `match` exaustivo em outra linha quebra a compilar).
- **`ph2d-sfx-recipe`:** blob `PH2DSFXL` **v1 → v3**, lendo v1, v2 (fixtures `library_v1.bin`, `library_v2.bin`); `Layer` ganhou `variacao`, `travada`, `travas`; `SfxLibrary` ganhou `ouvidos`. `to_bytes` grava só ouvidos.
- **`ph2d-sfx-lab`** depende de `tracing` e `ph2d-det-math`. **`o_motor_nao_chama_a_libm_do_sistema`** (`ph2d-det-math/tests/it/o_motor_nao_chama_a_libm.rs`): `CRATES` = **15**, com `ph2d-sfx-lab` e `ph2d-sfx-explore` dentro. Era 13 na integração anterior; outra linha que mexa na lista conta de novo.
- **Tocadas fora da família na W3** (colisão provável; detalhe no [handoff W3 §3](HANDOFF_CONTINUACAO_W3_2026-11-06.md)): `ph2d-panel-hierarchy` (`arma_escolha`, `toma_escolha`), `ph2d-panel-inspector`, `ph2d-app-audio`, `ph2d-i18n`, `ph2d-ecs`, `shells/desktop`, `ph2d-app-registry-init`, `ph2d-panel-registry-init`, `ph2d-app-components`.

## 3. Contratos e schemas (prova por grep, no fecho)

- `sed -n '/^## §6/,/^## §7/p' CLAUDE.md | grep -c -E "AudioSource2D|VariationSet|PlaySound|SfxLibrary|Recipe"` → **0**.
- `git grep -lE 'impl\s+(NodeOp|OpResolver|NodeManifest)\b|\bRasterEditTool\b|\bCanvasPaintTool\b|VectorOp\b' -- 'crates/ph2d-sfx-*' crates/ph2d-app-sfxlab crates/ph2d-panel-sfxlab crates/ph2d-sound-set | wc -l` → **0**.
- `git diff --stat bbfefaf393..HEAD -- crates/ph2d-editor-core/src/node.rs crates/ph2d-editor-core/src/tool.rs crates/ph2d-nodegraph` → **vazio** (intocados).
- **`PROJECT_SCHEMA` +0** (o blob `PH2DSFXL` v3 não é o schema do projeto), registos de componentes **+0 / +0 / +0**, ADR **+0**. Recontar mesmo assim no main do dia: `python3 scripts/schema-recount.py <degrau do main>` (escada e tripla em arquivos irmãos).

## 4. Delta para o integrador contar

- **Cargo.lock +7 internos** (acima; +133 −2). Crates novas: `ph2d-object-picker` (W3), `ph2d-sfx-explore` (motor) e `ph2d-sfx-explore-tela` (W4), `shells/sfxlab-scenes` = pacote `ph2d-host-sfxlab-scenes` (composição; recebeu os gates de tela que estavam em `ph2d-panel-sfxlab/tests/it`).
- **Teto de `ph2d-app-sfxlab` (`TETOS_DAS_CRATES_APP`): 2 534 linhas medidas no fecho** (`find crates/ph2d-app-sfxlab -name '*.rs' | xargs wc -l | tail -1`) **contra o teto 2 692** (na W3 estava em 2 690; a folga veio de mover os gates para `shells/sfxlab-scenes/tests/familia`, teste novo `familia`). Recontar contra o main do dia; nunca subir o número.
- **Shell:** `shells/desktop` +140 −2 na linha inteira, **a W4 não acrescentou linha**. Recontar `the_shell_only_shrinks` contra o main (na integração anterior a folga era 22).
- Cena **`PH2D_SFX_SMOKE=1..6`** (no main: `1|2`). 3 = aba em «Metal Hit» com objeto sem som; 4 = a 3 com a janela do Use in Game; 5 = «Impact» com a primeira camada travada; 6 = a 5 com o Explore aberto.
- **i18n:** chaves novas de SFX (≈109 linhas `+…sfx` em `ph2d-i18n`); `toda_chave_do_load_tem_texto` e os censos de texto de painel têm de seguir verdes.
- Memória: `project-memory/project_sfx_module_no_ai_2026_11_05.md` entra neste branch (decisões do dono 06/11).

## 5. O que um merge pode partir

- **`chrome_press` contra quem mudar `board_view`** (`pointer`, `pointer_move`, `cursor`, `route_pointer`) ou `Documents`: quem criar nova rota de ponteiro tem de passar pela mesma porta, ou a aba nova volta a perder o Up. Gates: `largar::` (it, `shells/sfxlab-scenes`) e `a_press_on_chrome_*` (`ph2d-editor-core`).
- **`ph2d-object-picker` depende de `Field` de `ph2d-kanban-views`** (com `Target::Scratch`): se o Kanban renomear/mover o `Field`, quebra. A cura (mover o `Field` genérico para `ph2d-editor-core`) toca o Kanban e fica ABERTA.
- `ph2d_sfx_dsp::Rng` público e o hash do `ph2d-sfx-dsp`: outra linha que mexa no DSP tropeça no gate de hash dourado.
- `RenderError::SemThread` e `InspectorAnimRow` (W2) mudam forma: literal/`match` de outra linha quebra a compilar.
- Panel → family em `[dev-dependencies]` reprova `no_dependency_climbs_a_layer`; por isso os gates de tela vivem em `shells/sfxlab-scenes`.
- A tabela de campos de `ph2d-sfx-lab` fica fora de `no_magic_numeric`: conferir que não é evasão.
- **`CLAUDE.md` §5.1:** a entrada «SFX Lab» foi reescrita neste commit (≤ 700 bytes); conflito com outra linha que edite §5.1 resolve-se linha a linha. O teto de 40 KB é de todos.

## 6. Números MEDIDOS (perfil e load ditos; load > 5 = ordem de grandeza)

**W3** (perfil `smoke`, mínimo de 7 blocos intercalados; load ~4 a ~16 conforme a medida; [W3 plano §6](../04_W3_plano.md)):

- Render típico de 2 s 7,41 ms; pior 2 s 24,63 ms; editar um corpo 3,30–4,14 ms; só o ganho 0,10 ms (load ~16).
- Por render a 2 s / 120 s (load ~4): pico 0,009 / 0,604 ms · LUFS 0,404 / 23,9 ms · espectro 0,951 / 54,0 ms · limitador 0,954 / 52,6 ms · impressão digital 0,219 / 17,8 ms.
- Editar até tocável (camada em cache, 2 s) 2,53 ms. Quadro da tela, CPU sem GPU: 0,449 ms (8 camadas), 0,871 ms (64).
- Pior corpo de 120 s: 9 370 ms (W2) e gravação a +24 semitons 32 ms: os dois casos que justificam a thread.

**W4** (perfil `smoke`, mínimo de 7 blocos; load ~5–13; [W4 plano §6](../05_W4_plano.md)):

- Do pedido à última das oito variações: **Mutate 2,43 ms, Roll 2,90 ms** no pior dos 8 sons de fábrica (load ~8). Pior corpo sorteado de 120 s: 556 ms até a oitava.
- 8 camadas × 120 s: **450 ms**, pico do processo **+1 128 MB** (derivado era ~1,5 GB).
- Heard com 10 000 linhas: entrar uma **0,035 ms**, copiar para o mundo **1,38 ms**, **2,4 MB**, gravar **3,44 ms** (100 000: 23,8 MB, 38,9 ms).
- Quadro da tela, CPU: **0,372 ms normal, 0,310 ms com o Explore**.
- `Ninhada` (uma thread por pedido), pior corpo de 120 s, load ~9–13: 200 pedidos sem pausa **74–86 threads** e `Drop` na UI **28–32 ms**; 1 000: 134 threads e 101 ms; um por quadro (o arrasto): 2 threads e 5–6 ms; 200 em 8 lugares 68–92 / 37–55 ms; `R` preso (8 por quadro) 23 / 10 ms. **Sem limite**, porque o pico é taxa × demora de cancelar, não cresce com a rajada; a mão real fica em 2–23 threads.
- Alvo de escuta (64 filhos que passam de −1 dB de pico ao nivelar): **−23: 0 de 64** no limitador (proposto) · −18: 8 de 64 · −14: 25. Custo por filho: maior momento 0,11 ms, limitador 0,01 ms parado.
- Variação relativa (Mutate ±30 % / Roll até ×4,5 do valor): 0 mudos de 64 (pela faixa do campo eram 6 e 1), maior diferença de volume 5,3 / 33,1 dB. O tamanho do Roll passa pelo ouvido do dono.

## 7. Prova de fecho (06/11)

- `cargo fmt --all -- --check` limpo; `git status --short` vazio antes do doc. Clippy `-D warnings` limpo nas crates tocadas (11 da W4, 17 da W3).
- **Antes de escrever este doc, rodados por nome e verdes:** `file_loc_caps` (`ph2d-host-desktop`, 4), `architecture_workspace_file_loc_cap` e `arch_safe_clamp_only` (`ph2d-editor-core`, 2 cada).
- Testes por pacote: `ph2d-sfx-explore` **23** · `ph2d-sfx-recipe` **8** (conferidos no fecho). `ph2d-host-sfxlab-scenes`, binários inteiros conferidos agora: `familia` **55 passam** e `it` **52 passam + 5 ignorados** (a janela anterior contou 49 e 31 por FILTRO de nome; não é a mesma conta, o binário inteiro é maior).
- Gates por nome verdes da bateria e da correção: `the_shell_only_shrinks`, `no_dependency_climbs_a_layer`, `no_magic_numeric`, `o_motor_nao_chama_a_libm_do_sistema`, `toda_chave_do_load_tem_texto`, `a_fronteira_dos_motores`, hash do `ph2d-sfx-dsp`, `a_tool_panel_never_hides_the_inspector`, `no_family_crate_hides_the_inspector_to_take_its_slot`, `a_lista_de_sons`, `largar::`, `a_press_on_chrome_*`, `o_claude_md_cabe_no_orcamento` (§8).
- **Auditoria com 2 lentes** (costura de UI; dados e determinismo), commit `5ead3a73a9`: 16 pontos; **15 curados, cada um com o gate visto vermelho e a mutação sangrando por cura (A–M)**; 1 vai para o ABERTO. Entre eles: Tab/Esc fecham o gesto do desfazer; R/M/N/Enter com gesto em curso são consumidos; K e Enter pegam `Grade::receita(i)`; `semeia` zera `a_tocar`; valor fora da faixa não é sorteado; `difere_de` vê estrela e `como`; `corta` nunca apaga o recém-ouvido nem estrela; cancelamento antes de `escuta_nivelada`; det-math no `ph2d-sfx-lab`.
- **Mutação da onda: 15 de 15 sangraram** depois de fechar 2 sobreviventes (`recebe` aceitando qualquer ticket → `render_velho_que_ja_terminou_tambem_nao_entra`; `troca_por` sem preservar o aprovado → `trazer_do_heard_nao_desaprova_o_som_do_jogo`). **Cura do cursor: 6 mutações, todas sangraram.**
- ⚠️ **`nextest-impacted` e a suíte completa NÃO rodaram na linha**, por lei do dono (CLAUDE.md §2, 23/10): **são do integrador**, mesmo que o `/pd-linha-fechar` os peça. Lacuna do vermelho-primeiro: os testes de lei e de mão da W4 nasceram junto com o código; a força deles veio da mutação (exceto o Lock por campo, visto vermelho).

## 8. CLAUDE.md

Entrada «SFX Lab» do §5.1 trocada: 284 bytes (tecto 700). `o_claude_md_cabe_no_orcamento` verde, mas o **§5 está a ~6 bytes do tecto de 14 000** (era 13 971 no HEAD): a entrada teve de encolher para caber; outra linha que acrescente ao §5 reprova ali e a cura é cortar narrativa do §5, nunca subir o número. O link do handoff de integração é este documento.

## 9. Smokes (comando inteiro)

Binário `target/smoke/ph2d-host-desktop`. Remontar (se a árvore andou): `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-SfxLab && bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`.

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-SfxLab && env PH2D_SFX_SMOKE=3 ./target/smoke/ph2d-host-desktop` (aba do som; objeto sem som)
2. `… env PH2D_SFX_SMOKE=4 …` (com a janela do Use in Game aberta)
3. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-SfxLab && env PH2D_SFX_SMOKE=5 ./target/smoke/ph2d-host-desktop` («Impact», camada de ruído travada)
4. `… env PH2D_SFX_SMOKE=6 …` (a 5 com o Explore aberto; `Tab` abre/fecha)

Os quatro com o prefixo `cd … &&` completo. O som fica com o ouvido do dono (caixas pequenas).

## 10. ABERTO, em ordem

a. **O dono ainda não reviu em tela o cursor que voltava sumido.** A devolução real (`CursorPreso::solta`) só compilou; o gate vai até `number_input_drag == None`. Sem isto, o defeito do smoke não está fechado pelo olho.
b. **Teste às cegas com o profissional** ([W4 plano §7](../05_W4_plano.md)): depende do dono (convidar, 1h30, aprovar a régua antes). A W5 pode seguir sem ele, com o teste marcado antes de congelar a tela. **Única decisão pendente do dono.**
c. `edita` com Play on change sobre um Roll pendente ainda toca nivelado.
d. Sem prova: `RenderError::SemThread` sem falha injetada; texto «Clips» não fotografado; o guarda de `board_view::cursor` sem mutação; `chrome_press` preso por um Up engolido antes de `board_view::pointer` só se cura na descida seguinte; as abas reais Kanban/Code/Planner foram provadas só por aba falsa; arrasto e Heard cheio não fotografados (a tela virtual não aceita clique); a rolagem da lista Heard pela roda sem teste próprio; `L` sobre campo só no banco de cartões.
e. O medidor Loudness segue na medida de arquivo inteiro; ele e o `nivelada` leem coisas diferentes de propósito.
f. `Field` genérico para o `ph2d-editor-core` (toca o Kanban).
g. Restos do [handoff W3 §6](HANDOFF_CONTINUACAO_W3_2026-11-06.md): Intel/ARM do hash (no Mac: `cd docs/SfxLab/ferramentas/sonda_w2 && cargo run --profile smoke --bin hashes`, tem de terminar em `conjunto 86ea4a05df14620b`), Fonte de 10 m anexada à mão segue muda à distância, mudar o layout com a aba ativa pode reabrir diferente, fechar a aba no meio de um render junta na thread da UI (~140 ms, não medido), Pick ativo com a aba fechada só compilado, biblioteca gravada do dono para W5.
