# Handoff de INTEGRAÇÃO — `line/GameEngineFuncional` (01/11)

Leitor: o agente integrador (DIRETRIZ §1.5.3 / §1.5.9). Smoke do dono: **aprovado 01/11** (`PH2D_ORDEM_SMOKE=1|2`,
o clique nos ícones, na Pile e dentro do círculo do objecto vazio). Cobre as ondas **18 → 19** desde o handoff de
28/10 (que o main já integrou: o merge-base é o main de hoje). Integração e ship só por ordem do Enio.

## 1. Identidade

| | |
|---|---|
| Branch | `line/GameEngineFuncional` |
| HEAD | o commit deste handoff (código: `6426b18cf7`) |
| merge-base com o main | `422fec2027` — **o main não andou ⇒ `--ff-only` possível hoje** |
| Commits / ficheiros | 43 · 167 (+8 058 / −1 146) |
| Crate NOVA | `ph2d-camera-frame` (folha: a moldura da câmera e a sua tinta, saídas de `ph2d-app-game`) |

Não há vermelho herdado conhecido: a suíte inteira NÃO foi corrida na linha (ordem do dono 23/10) — o integrador
corre-a e confronta com o main em `422fec2027`.

## 2. Superfície de colisão (`scripts/collision-surface.sh`, VERBATIM)

```
SUPERFÍCIE DE COLISÃO — line/GameEngineFuncional contra main
  merge-base 422fec2027   ·   43 commit(s)   ·   167 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
    PROJECT_SCHEMA                        209   (base: 209)
      └ tripla do gate               (209, 13, 25)   (base: (209, 13, 25))
    VEC_SCENE_SCHEMA                       25   (base: 25)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 24   (base: 24)
  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS;
      um degrau escrito no arquivo errado funde LIMPO e evapora.

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                              116   (base: 116)
    ph2d-render (espelho)                 117   (base: 117)
    ph2d-script (espelho)                 117   (base: 117)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0196   próximo livre: 0197
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
  ⚠ 1 pacote(s) '+name' novo(s):
      "ph2d-camera-frame"

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3), que uma varredura de 3 marcadores NÃO vê
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou (700 workspace · 600 painel/shell · 500 widget · 650 tool-runtime)
  ✗   919 / 600   shells/desktop/src/app_state.rs
  ✗  1031 / 600   shells/desktop/src/main.rs
───────────────────────────────────────────────────────────────────────────────
  ⚠️ Isto é o MAPA, não o gate. O gate mecânico é scripts/foundational-integrate.sh;
     o que exige julgamento (mesmo-símbolo, decisão de produto) continua leitura humana.
```

Leitura (DELTA, não valor):

- **Schema/registo: delta ZERO** (`PROJECT_SCHEMA` 209 = base, sem degrau; registo ecs/render/script iguais). Nada a
  recontar por degrau; o `schema-recount.py` só confirma no main do dia.
- §6 intocado; nenhum ADR. `Cargo.lock`: só o pacote INTERNO `ph2d-camera-frame` (nenhum pacote externo).
- LOC: `app_state.rs` 919/600 e `main.rs` 1031/600 são isenções numeradas **pré-existentes**
  (`shells/desktop/tests/it/file_loc_caps.rs`); `app_state` ENCOLHEU (920→919), `main` está no tecto (1031). Em
  `pass.rs` (`ph2d-app-flip`) e `hero.rs` (`ph2d-editor-core`) o tecto é 700 e estão em 700.
- Tectos que SOMAM entre linhas e se reconferem na árvore combinada: `the_shell_only_shrinks` 189 041 (a shell fica
  ≈188 900 depois do corte de `pick_order.rs`) · `ph2d-app-game` 2 728 (`TETOS_DAS_CRATES_APP`) · tectos de ficheiro.

## 3. Foundational / partilhado tocado

`git diff main..HEAD --stat -- crates/<c>` (shell: `shells/` 52 f, +2 635/−567).

| Crate | Δ | O quê |
|---|---|---|
| `ph2d-camera-frame` (NOVA) | 3 f, +853 | `lib.rs` = a antiga `moldura.rs`; `tinta.rs` = a antiga `moldura_tinta.rs` (movidas de `ph2d-app-game`, que cai −899 e fica no tecto 2 728) |
| `ph2d-editor-core` | 11 f, +377/−18 | **módulo novo** `screens::hero::pick_cycle` (veio de `shells/desktop/src/pick_order.rs`): `start_on_selection`, `CycleStep`, `cycle_step`, `top_beats_selection`; `hover_highlight::icones_em`; `Family::{Pixel,Flip,Motion,Palco}` na ordem total |
| `ph2d-render` | 5 f, +969/−295 | `draw_bands`/faixas por família na ordem total (`draw_bands` curado de 1 037 linhas) |
| `ph2d-gpu` | 3 f, +228/−1 | suporte das faixas / leitura do realce (gates de placa) |
| `ph2d-app-motion` | 21 f, +1 865/−15 | `motion_pick` (clique no Motion: CPU + `do_dispositivo`), placa por faixa, palco da macro, miniatura |
| `ph2d-app-flip` | 8 f, +250/−8 | `Family::Flip`, uma faixa por corrida; `ordem_smoke.rs`; `pass.rs` em 700 |
| `ph2d-app-pixel` | 4 f, +162/−69 | os Pixel Sprites na ordem (`Family::Pixel`) |
| `ph2d-app-game` | 13 f, +434/−899 | miniatura, cena `ordem_smoke.rs`; moldura saiu para `ph2d-camera-frame` |
| `ph2d-panel-inspector` | 12 f, +265/−17 | «Outside: Dimmed», «Editor View: Follows», secção Ordering em todo tipo |
| `ph2d-app-macros` / `-light2d` / `-components` / `-vec` | 3 f +109 · 2 f +78/−41 · 3 f +45/−3 · 1 f ±1 | ícone da Macro; luz da vista; consumo |
| `ph2d-ecs` / `ph2d-i18n` / `ph2d-vector` / `ph2d-panel-registry-init` | 3 f +19/−2 · 1 f +9 · 1 f +7 · 1 f +1 | aditivos |

Censos por prefixo `ph2d-app-` ganharam `|| nome == "ph2d-camera-frame"` (ex.:
`cada_motor_da_fronteira_resolve_as_chaves_dele.rs:223`) — a crate nova foi para a lista; no main combinado,
reconferir que nenhuma outra linha acrescentou a sua sem a minha.

## 4. Ondas (plano `docs/GameEngineFuncional/NN_*.md` · handoff `handoffs/HANDOFF_CONTINUACAO_*`)

- **18** (`…10-30_18_CAMERA`): zona morta à vista e arrastável, «Outside: Dimmed», «Editor View: Follows»; alças
  com relógio parado (Auto Key), formato do jogo no desfazer, miolo com pai rodado/escalado (fecha 3 abertos de 28/10).
- **18b** (`…18b_MINIATURA`, A6): a miniatura do que a câmera vê. **18c**: os palcos das macros e as formas do
  Motion fora da placa entram na miniatura; piso da grade da luz da vista medido (não pesa).
- **19** (`…2026-11-01_19_ORDEM_E_CLIQUE` + `_3` + `_4`, plano [`19_ordem_e_clique_plano.md`](../19_ordem_e_clique_plano.md)):
  a Hierarchy manda no DESENHO e no CLIQUE de todas as famílias. E1 clique pela pilha · E2 Pixel · E3 Motion
  (placa por faixa, palco da macro, lente/filtro, imagens de grupo) · E4 Flip · E5 Z-index em todo tipo · E6 ícone da
  Macro · E7a–c paralaxe no editor / repetição escalada / limites do nível 1 · `motion_pick` · trava × grupo ·
  cena `PH2D_ORDEM_SMOKE` · `ph2d-camera-frame` · sonda `PH2D_SONDA` · mutações em lote (15/18 sangram; as 3
  sobreviventes ganharam gate de comportamento).
- **Lei do clique (dono 01/11, últimos 3 commits `90a04a5509`/`c33d4421e2`/`09d0d14e1a`)**: o que se VÊ por cima
  ganha o clique, até da seleção — a seleção só guarda o clique quando tudo o que está acima dela na pilha é seu
  descendente e nenhum é ícone (`top_beats_selection`). `icones_em` (ícones de luz/câmera/Macro + o DISCO dos
  objectos vazios) vai primeiro; `descendants_first` corre também no fim de `pick_objects_at`.
  Gates: `dentro_do_circulo_do_vazio_o_primeiro_clique_e_dele` (mutação «disco volta ao fim» ⇒ VERMELHO, provada),
  `what_is_seen_on_top_beats_the_selection_under_it`, `the_click_dispatch_starts_the_cycle_on_the_selection`
  (agulhas de fio).
- **Medido** (cena da ordem, MÍNIMO de blocos intercalados, `--profile smoke`): banda Flip 0,078 ms com bandas
  intercaladas × 0,100 na Fase 1 (sem custo extra); a ordem total custa **+0,1–0,26 ms de placa/quadro**
  (`band_blit` + tonemap por banda) ≈ 1,5 % do quadro, **sem cura**; realce do ponteiro na CPU 0,005 ms nas cenas de
  smoke; leitura da placa do realce (gate): 64 cópias 32,1 µs · 4 096 cópias 115,8 µs.

## 5. Para o integrador

**CLAUDE.md §5.1, entrada «Game Engine»** (≤ 700 bytes no total): trocar o link do Último por
`docs/GameEngineFuncional/handoffs/HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-01.md` e acrescentar à frase
«ordem de desenho e clique pela Hierarchy em todas as famílias; crate `ph2d-camera-frame`» e o smoke
`PH2D_ORDEM_SMOKE=1|2` (medir os bytes — a entrada actual já está perto do tecto: cortar «testes de mundo montado…»
se faltar).

**Reconferir na árvore combinada** (nenhum recontar de schema/registo: delta zero):
1. `python3 scripts/schema-recount.py` só para confirmar que o main do dia não trouxe degrau que mude a base 209.
2. Tectos de LOC SOMADOS entre linhas: shell 189 041, `ph2d-app-game` 2 728, ficheiros (`file_loc_caps`).
3. Que o censo `ph2d-app-` + `ph2d-camera-frame` ainda cobre o main (merge de duas listas nos censos).
4. Impactados e suíte completa: ficam para a integração (1×, do integrador).

**O que um merge pode partir:** `shells/desktop/src/pick_order.rs` ENCOLHEU (−109/+8: o ciclo foi para `ph2d-editor-core`) — outra
linha que o edite colide no mesmo ficheiro; `moldura*.rs` saíram de `ph2d-app-game` (modificação/apagamento); `hero.rs`,
`pass.rs` e `main.rs` estão no tecto — +1 linha de outra linha estoura o gate.

## 6. Smokes (binário `smoke`)

Comando: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_ORDEM_SMOKE=<n> cargo run -p ph2d-host-desktop --profile smoke`

| n | Cena |
|---|---|
| `PH2D_ORDEM_SMOKE=1` | sete tipos sobrepostos — Card (imagem), Pile (formas do Motion a cair), Ink (traço do Flip), Face (Pixel Sprite), Swarm (cópias do Motion com campos), ícones da Macro e da Câmera; o do TOPO da Hierarchy desenha ao FUNDO |
| `PH2D_ORDEM_SMOKE=2` | a ordem invertida: o Card cobre tudo |

Passos do dono: (1) o comando com `n=1` · (2) clicar onde as peças se sobrepõem, nos ícones, na Pile e dentro do
círculo de um objecto vazio · (3) o que se vê por cima é o que se seleciona · (4) errado = pega o de baixo, ou o
clique no círculo do vazio escolhe outro objecto. Segue valendo `PH2D_GAME_SMOKE=1..4` (cena 4 = a câmera/miniatura:
`cd … && PH2D_GAME_SMOKE=4 ./target/smoke/ph2d-host-desktop`). Sonda: `PH2D_SONDA=<bits>` (`PH2D_SONDA_RODADAS`,
omissão 7). Fotos: `handoffs/fotos_18`, `fotos_19`.

## 7. ABERTO

- **`Cola::Motion` esvaziado à mão** (`shells/desktop/src/render_loop/present_bands.rs:206`): sem gate de comportamento.
- **Nível 2 do smoke** (`crates/ph2d-app-game/src/smoke.rs:534`, `CameraLimits max [17, 9]`): sem gate dos limites
  (o do nível 1 existe, E7c).
- **Custo da ordem total** +0,1–0,26 ms de placa/quadro (≈1,5 %): medido, sem cura; só se pesar.
- **Cena `PH2D_ORDEM_SMOKE` fora do gate de foto** se o main mudar o tema/escala: refotografar antes de culpar o código.
- Fechado e conferido contra os commits (NÃO reabrir): jogador ausente em `GAME_SMOKE=2` (não é defeito: o main dá a
  mesma foto), clique no Motion, imagens de grupo, E7c, trava × grupo, piso da grade da luz, tecto do `ph2d-app-game`.
- Recusado e medido (não reconstruir): selo «segue <alvo>» na Hierarchy (herdado de 28/10).

## 8. Perfil do loop (`bash scripts/agent-loop-profile.sh`, VERBATIM)

```
PERFIL DO LOOP DO AGENTE — 20 sessao(oes) mais recentes
──────────────────────────────────────────────────────────────────────────────
  ✗ paralelismo de ferramenta              1.08/passo   alvo: >= 1,5  (6% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                152   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                299 : 140   alvo: <= 1,0  razao 2.1x (baseline: 4,3x)
  ✓ edicoes SEM conferir que casaram             0.9%   alvo: <= 2%  (6 de 641; o guarda R6 recusa-as desde 14/10)
  ✓ contexto relido por passo (media)         223 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               61 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
──────────────────────────────────────────────────────────────────────────────
  As leis moram no CLAUDE.md §2 (sempre carregado); a DIRETRIZ_IMPLEMENTACAO aponta pra la'.
  ⚠️ Rode com poucas sessoes para ver o HABITO recente; 'all' e' o baseline historico.
```

## Fecho (portões)

Sobre o HEAD `6426b18cf7` (os consertos do fecho: `87b81fc3be` fmt + tectos por corte, `6426b18cf7` clippy + HR-15). Impactados e suíte completa: NÃO corridos (são do integrador). Gates de placa por nome,
verdes em série: Motion 54 · Flip luz 2 · shell 23/26 · game-scenes 42 `jogo_`.

| Portão | Resultado |
|---|---|
| `cargo fmt --all -- --check` | ✗→✓ (`parallax_bridge_repeticao_tests.rs`, `ph2d-app-macros/src/lib.rs`; e `pass.rs` 702, `hero.rs` 701, `main.rs` 1032 de volta a 700 · 700 · 1031 por corte de comentário) |
| `PH2D_SUITE=fecho cargo clippy --workspace --all-targets -- -D warnings` | ✗→✓: `ph2d-app-motion` `len_zero` ×2 e `single_range_in_vec_init` ×4 (testes) · `build_frame_order` 8/7 argumentos → `Familias<'_>` (sem `#[allow]`). Re-corrido no workspace: ✓ |
| `check-standalone-optional.sh` · `check-workflow-packages.sh` | ✓ · ✓ |
| `git grep` dos leitores de `shells/desktop/src` (o que saiu de `pick_order.rs` e de `moldura*.rs`) | ✓ nenhum leitor aponta para ficheiro que esta linha removeu (os ~28 caminhos mortos são comentários históricos de mudanças antigas) |
| `cargo machete` | ✓ (só `rustc-hash` no `wgpu-core` vendorizado, pré-existente) |
| `file_loc_caps` · `the_shell_only_shrinks` · `architecture_workspace_file_loc_cap` · DAG da fundação · `no_dependency_climbs_a_layer` · `arch_safe_clamp_only` | ✓ (6/6; `file_loc_caps` e `the_shell_only_shrinks` re-corridos depois dos consertos) |
| `censos-da-arvore-combinada.sh` | ✗ 128/129 → ✓: `every_word_the_shell_shows_comes_from_the_string_table` (8 literais) — as fotos do smoke da câmera mudaram-se para `janela_do_jogo_smoke.rs` (nomes da cena/consola; o gate isenta `*smoke*.rs`), rótulos de GPU da vista da câmera → `ph2d.vista_camera.*`; re-corrido por nome ✓ + `every_named_exemption_still_shelters_what_it_names` ✓ |
| Testes do item, por nome (`dentro_do_circulo_do_vazio_o_primeiro_clique_e_dele` · `what_is_seen_on_top_beats_the_selection_under_it` · `the_click_dispatch_starts_the_cycle_on_the_selection`) | ✓ (+ `luz_tests` 6/6, `pick_order` 6/6, `pick_cycle` 7/7, `group_gizmo_view` 10/10, `sim_extract::` 11/11, cortes da faixa Motion 2/2) |
