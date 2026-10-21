# HANDOFF DE INTEGRAÇÃO — `line/Macros-Automação`, W14 (a água continua depois do fim do clipe; a edição que espera não engole o que o artista faz) → `main` (2026-10-20)

> A linha fecha e PARA: integrar e shipar só por ordem do Enio, pelo agente integrador (DIRETRIZ §1.5.3).
> Mecanismo, medições e recusas: [doc W14](../12_W14_a_agua_depois_do_parar.md). Pedido:
> [PROMPT_W14](PROMPT_W14_a_agua_depois_do_parar_2026-10-20.md). Memória da decisão do dono:
> `project-memory/project_the_water_keeps_flowing_after_the_clip_ends_2026_10_20.md`.
> ⚠️ **W8–W13 ainda NÃO estão no `main`** (mesma branch, integram juntas): [W8](HANDOFF_INTEGRACAO_line_Macros_2026-10-17_W8.md)
> · [W9](HANDOFF_INTEGRACAO_line_Macros_2026-10-17_W9.md) · [W10](HANDOFF_INTEGRACAO_line_Macros_2026-10-19_W10.md)
> · [W11](HANDOFF_INTEGRACAO_line_Macros_2026-10-19_W11.md) · [W12](HANDOFF_INTEGRACAO_line_Macros_2026-10-20_W12.md)
> · [W13](HANDOFF_INTEGRACAO_line_Macros_2026-10-20_W13.md) continuam válidos para a parte deles. Este só acrescenta a W14.
> ⚠️ A linha **NÃO foi rebaseada** (ordem do dono: «sem reset nem rebase»): o integrador rebasa.

## §1 — Base e HEAD

- Branch `line/Macros-Automação`, worktree `Worktrees/line-Macros-Automação/`. Merge-base `ad5f8fe39c` (conferido com
  `git merge-base main HEAD`); o `main` está em `100d34c8e0`.
- Commits da W14 (`git log --oneline 28cf86ae0b..HEAD`, do mais velho): `2a215a29f4` (test: vermelho antes da cura do
  item 1 — a ranhura fica 80 níveis longe do documento no fim do clipe; a marca não guarda onde a água parou; sonda do
  salto) · `b6e58ada20` (feat: a cauda) · `fc5fdad7de` (test: vermelho do item 2 — a edição que espera funde-se com o que
  o artista faz) · `8c3ef8a77b` (feat: a edição que espera) · `f16519b5c8` (test: `=6` ganha `PH2D_MACRO_CAMINHO=fim`; a
  sonda mede a espera da imagem do fim) · `6fce9f3879` (docs: doc W14 + memória) · `b210f4a71d` (docs: fotos do `=6 fim`)
  · `db51f7142c` (test: a meio do clipe a cauda não anda — fecha a mutação M1; corrigida a recusa da M4) · `681f227fcd`
  (style: fmt) · `c2c6928260` (fix: a edição que espera mora em `ProjectUndo::espera`, não num campo da `App`; o empurrão
  depois do `post_frame_undo`; a agulha do gate da ordem do quadro com o `dt`) · `92a7c2e05d` (style: fmt) · o commit deste
  handoff (último).

## §2 — O que a W14 entrega

Decisão do dono 20/10 (perguntada com os números, [doc §2](../12_W14_a_agua_depois_do_parar.md)): no fim de um clipe de
Wet Paint a água continua (~1 min) até à imagem final, sem salto, sem clipe mais comprido. Recusadas: o salto no último
quadro; deixar como estava.

**Item 1 — a cauda** ([doc §3](../12_W14_a_agua_depois_do_parar.md)):
1. `MacroEmCena::corre_a_cauda(dt, ate)` → `TocadorDoPainter::corre_a_cauda` → `PainterTool::wet_cauda` (ficheiro novo
   `crates/ph2d-tool-painter/src/tool/paint/wetpaint/cauda.rs`). Sem `ate` usa a MESMA porta da imagem do fim ⇒ assenta
   nela ao byte; com `ate` corre até ao estado guardado e fecha com `wetpaint_end_session`.
2. `WetPaintState::composta` grava o `MarcaDaAgua` a cada composite; `wetpaint_end_session` da sessão nascida SEM a marca
   grava-o na marca do artista (`MarcaDoArtista::agua_da_gravacao_parou`, a 1.ª vez fica). **Codec da marca FORMATO 3**
   (o 2 ainda abre, sem ele).
3. `PainterTool::marca_a_gravar` (a cópia gravada leva o estado vivo da água); `marcas_a_gravar` grava uma marca vazia
   que tenha esse estado.
4. `PalcosDoPainter::dt_do_quadro` + `cauda::anda` na `sobe`; a shell passa o `dt`
   (`fase_frame_open` → `fase_macros_do_painter(preview_overrides, dt)`).

**Item 2 — a edição que flutua** ([doc §4](../12_W14_a_agua_depois_do_parar.md)):
1. `ph2d_app_macros::espera::EdicaoQueEspera`; vive em `ProjectUndo::espera` (NÃO num campo da `App`: o tecto de 172
   campos e o de 211 LOC do `App::new` ficaram vermelhos primeiro e curaram-se movendo-a).
2. `EstadoComMacros` para `ProjectState` via `WorldSnapshot::component_blob` / `set_component_blob` (`ph2d-ecs`, aditivo);
   `PalcosDoPainter::a_reescrever_em`, `Macros::a_conferir_em`; em `undo_app`: `macros_que_esperam`, `empurra` (depois do
   `post_frame_undo`), `apply_undo` chama `antes_de_desfazer` / `restaurado`.
3. Removidos os motivos de supressão do desfazer e as chaves i18n `shell.undo_app.macro_a_conferir` /
   `shell.undo_app.macro_a_reescrever` (−2). Pilha depois de editar → mover → a imagem chegar: `[mover, edição+imagem]`;
   o 1.º Ctrl+Z desfaz a edição com a imagem, o 2.º o movimento.

### §2-bis — Medições, mutação, testes

Sonda `macro_track::sonda_w14_tests` (1024², relógio fixo; tabela completa no [doc §1](../12_W14_a_agua_depois_do_parar.md)):

| | 2 pinceladas | 4 pinceladas |
|---|---|---|
| água depois do Parar | 3 430 quadros (57,2 s) | 2 846 (47,4 s) |
| último quadro do clipe ≠ assentada | 185 270 texels (93 % dos 199 772 pintados); 147 470 > 8 níveis; o maior 121 | 285 190 (93 %) |
| água por quadro (média) | 6 896 texels | 14 813 |
| imagem do fim nova de uma etapa editada | 27,2 s (`load` 291) | 39,8 s (`load` 112) |

Antes da W14 toda acção global durante a espera (mover, renomear, componentes, vetor, Flip, guias, estados de UI,
biblioteca, tags) fundia-se no passo da edição.

**Mutação:** 16 mutações, 14 sangraram na 1.ª rodada; sobreviventes M1 (cauda a correr a meio do clipe: sem teste → novo
`a_meio_do_clipe_a_cauda_nao_anda`, re-corrida sangra) e M4 (recompor a tela inteira no Parar) **equivalente** (mesma tela;
a recusa antiga do doc que lhe atribuía 3 056 texels foi corrigida). Final: 15/16 sangram + 1 equivalente.

**Testes novos:** tool-painter `macro_track::cauda_tests` (`no_fim_do_clipe_a_agua_continua_e_assenta_na_imagem_do_fim`,
`a_agua_que_parou_no_toque_do_artista_para_no_mesmo_passo_na_cauda` [relógio fixo + trabalhador real],
`a_meio_do_clipe_a_cauda_nao_anda`) · `marca_tests::o_formato_3_guarda_onde_a_agua_parou_e_o_da_w10_abre_sem_ela` ·
app-painter `palcos_cauda_tests` (`no_fim_do_clipe_a_ranhura_corre_a_agua_ate_a_imagem_do_artista` [sem toque e com
vertical; ≤ 1 nível], `gravado_com_a_agua_a_correr_e_reaberto_a_cauda_para_na_imagem_gravada`) · app-macros `espera_tests`
(4, fila de brinquedo com a ordem da shell) · `ph2d-ecs` `save_tests::por_e_tirar_um_componente_de_uma_linha` · shell
`undo_tests::a_edicao_que_espera_fica_fora_da_fotografia_do_desfazer` · `caminho_tests` inclui `Variante::Fim`.
Vermelho primeiro: `2a215a29f4` (item 1), `fc5fdad7de` (item 2).

## §3 — Foundational / partilhado tocado (aditivo)

| Crate | O quê |
|---|---|
| `ph2d-ecs` | `WorldSnapshot::component_blob` / `set_component_blob` (aditivo) + teste |
| `ph2d-i18n` | −2 chaves |
| `ph2d-tool-painter` | `tool/paint/wetpaint.rs`, `wetpaint/{cauda (novo),composite,authored_actions,fita,session,dab_route,mascara}.rs`, `undo_marca.rs`, `macro_track/{marca,…,cauda_tests,sonda_w14_tests}` |
| `ph2d-app-macros` | `espera.rs` (novo), `espera_tests`, `Macros::a_conferir_em` |
| `ph2d-app-painter` | `PalcosDoPainter::{dt_do_quadro,a_reescrever_em}`, `palcos_cauda_tests`, `macro_track/smoke.rs` (`PH2D_MACRO_CAMINHO=fim`) |
| `host-desktop` (shell) | `undo_app` (`macros_que_esperam`, `empurra`, `apply_undo`), `ProjectUndo::espera`, `EstadoComMacros for ProjectState`, `fase_macros_do_painter(.., dt)` |

Contratos congelados (§6) **não tocados**. Sem `Cargo.toml`, crate nova, ADR ou `PROJECT_SCHEMA`.

## §4 — O que um merge pode partir

- ⛔⛔ Os códigos do `Passo` 53–56 continuam a colidir com a `line/PainterWatercolor` ([W12 §4](HANDOFF_INTEGRACAO_line_Macros_2026-10-20_W12.md)).
- ⚠️ **Superfície de colisão com a `line/PainterWatercolor`:** a W14 toca (aditivamente) ficheiros do Wet Paint:
  `wetpaint.rs`, `wetpaint/composite.rs`, `wetpaint/authored_actions.rs`, `wetpaint/fita.rs`, `wetpaint/cauda.rs` (novo),
  `undo_marca.rs`. Confirmar com `git diff --name-only main...line/PainterWatercolor -- crates/ph2d-tool-painter/src/tool/paint/wetpaint*`
  e manter os dois lados; reler o `composite.rs` (o registo do `MarcaDaAgua` está no caminho de cada composite).
- **Codec da marca FORMATO 3** (dentro do blob de marcas do projecto, versão própria; o `FORMATO` externo 1 não muda). Outra
  linha que escreva o formato 2 continua a abrir; nada deve ESCREVER 2 depois do merge.
- `ProjectUndo` ganhou o campo `espera`; qualquer construção por literal noutra linha tem de o incluir. A ordem na shell
  conta: o `empurra` vem DEPOIS do `post_frame_undo` (agulha no gate `the_step_carries_the_selection_and_the_restore_prefers_it`).
- Tectos que a W14 aperta: campos da `App` (172) e LOC de `App::new` (211) — outra linha que os use pode vê-los vermelhos
  na junção; a cura é mover, nunca subir.
- As chaves i18n `shell.undo_app.macro_a_conferir` / `macro_a_reescrever` saíram: uma linha que as use rompe.

## §5 — Contadores (DELTA sobre o merge-base, só a W14)

Crates novas 0 · chaves i18n −2 · ADR 0 · `PROJECT_SCHEMA` 0 (o formato da marca vive no blob de marcas, com versão
própria) · contratos 0 · cenas de smoke: nenhum número novo (`=6` ganhou `PH2D_MACRO_CAMINHO=fim`).

## §6 — Prova de fecho

| Portão | Resultado |
|---|---|
| `cargo fmt --all -- --check` | ✓ (depois dos commits de fmt) |
| clippy `-D warnings` `--all-targets`: tool-painter, app-painter, app-macros, ecs, i18n, host-desktop | ✓ |
| `architecture_workspace_file_loc_cap` · `architecture_claude_md_cabe_no_orcamento` · `the_shell_only_shrinks` | ✓ |
| `censos-da-arvore-combinada.sh` | ✓ 12/12, 121/121 |
| `PH2D_SUITE=fecho BASE=merge-base nextest-impacted.sh` (em `681f227fcd`) | «Summary [1396.978s] 18957 tests run: 18953 passed (43 slow), 4 failed, 10994 skipped», `load` 84 → 160 |

Os 4 vermelhos eram **todos da W14** e determinísticos (não flakes), curados em `c2c6928260`:
`the_app_does_not_grow_past_its_numbered_ceiling` (173 > 172) · `shell_functions_respect_their_ceiling` (`App::new` 212 > 211)
· `the_step_carries_the_selection_and_the_restore_prefers_it` (agulha `push_undo_em(base, mark,` depois do `post_frame_undo`)
· `as_macros_do_painter_trocam_a_sprite_antes_do_extract` (agulha sem o `dt`). Depois: 78 testes da shell por nome
(`undo|the_app_only_sheds_fields|fn_loc_caps|placa_tests`) 78/78 ✓ a `load` 163; antes, regressão `macro_track` tool-painter
67 passados e app-painter 30 passados.
⚠️ **A suíte inteira NÃO foi re-corrida depois de `c2c6928260`** — o integrador corre a sua.

`agent-loop-profile.sh`:
```
  ✗ paralelismo de ferramenta              1.19/passo   alvo: >= 1,5  (11% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                142   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                342 : 157   alvo: <= 1,0  razao 2.2x (baseline: 4,3x)
  ✓ edicoes SEM conferir que casaram             0.3%   alvo: <= 2%  (3 de 959; o guarda R6 recusa-as desde 14/10)
  ✗ contexto relido por passo (media)         310 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               64 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```

Smoke compilado (2.ª corrida, depois de `rm -rf target/*/incremental`, 17 G debug + 13 G smoke libertados): «Finished
`smoke` profile [optimized] target(s) in 0.64s», zero `Compiling`.

## §7 — Smoke do dono e fotos

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && PH2D_MACRO_SMOKE=6 PH2D_MACRO_CAMINHO=fim ./target/smoke/ph2d-host-desktop`
2. Não mexa no rato: a cena grava duas pinceladas de Wet Paint, carrega em Stop e deixa a cabeça da Keys parada depois do
   fim do clipe.
3. A tinta continua a espalhar-se na tela por até um minuto e pára na imagem final; nada salta.
4. Errado: as pinceladas ficarem congeladas no fim, ou saltarem de uma vez.

Segundo smoke (manual, item 2): na cena `=6` do caminho base, depois do Stop, edite uma etapa em MACROS (p. ex. a cor da
pincelada 2), logo a seguir mova a sprite do Canvas e espere ~1 min. Ctrl+Z → a edição e a imagem dela voltam, a sprite
fica movida; Ctrl+Z outra vez → o movimento volta. Errado: um só Ctrl+Z desfazer os dois.

Fotos (`smoke_2026-10-20_W14/`): [logo no fim, 7,7 s](smoke_2026-10-20_W14/fim_01_logo_no_fim.png) ·
[a água a correr, 30,8 s](smoke_2026-10-20_W14/fim_02_a_agua_a_correr.png) ·
[assentada, 85,5 s](smoke_2026-10-20_W14/fim_03_assentada.png) · [log](smoke_2026-10-20_W14/fim.log). Série `≠editor`:
22 971 (6,2 s) → 24 894 (10 s) → 25 081 (20 s) → 17 895 (28 s) → 3 297 (32 s) → 0 de 34,5 s a 85,5 s; pintados 22 627 →
27 605; `load` 27–133. A cauda anda ao relógio do quadro (`dt` ≤ 1/30 s), a água do editor ao relógio da parede: convergem,
e o `≠editor` fica alto até ao fim. Smoke do dono depois da W14: **por fazer**.

## §8 — ABERTO

1. A cauda avança ao relógio do quadro (`dt` preso a 1/30 s): sob carga corre mais devagar que a água do editor; converge a
   0, mas o tempo difere.
2. Na aba Keys o clipe faz laço por omissão: a cauda só se vê com a cabeça parada no fim ou depois dele, com o Autoplay
   a segurar depois do fim, ou com o laço desligado.
3. Uma sessão de gravação que acaba ANTES de a marca estar armada (entre o Stop e o 1.º quadro do palco) não grava o
   estado de paragem: a cauda corre então até assentar.
4. Para o integrador: a colisão dos `Passo` 53–56 e a superfície Wet Paint com a `line/PainterWatercolor` (§4).
5. Seguem de pé os itens abertos do §8 da [W13](HANDOFF_INTEGRACAO_line_Macros_2026-10-20_W13.md) e da
   [W12](HANDOFF_INTEGRACAO_line_Macros_2026-10-20_W12.md). Os itens 1–2 do §8 da
   [W8](HANDOFF_INTEGRACAO_line_Macros_2026-10-17_W8.md) são os que a W14 fecha.

## Para o integrador (`CLAUDE.md` §5.1, entrada «Macros & Automação»)

Novo «Último» = este handoff (os de W8–W13 continuam a valer para a parte deles). A frase da natureza pode ganhar «a água
continua depois do fim do clipe» só se couber nos 700 bytes (gate `architecture_claude_md_cabe_no_orcamento`).

## Retomar

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && pwd && git branch --show-current`.
