# HANDOFF DE INTEGRAÇÃO — `line/Macros-Automação`, W13 (o tutorial «fazer» ensina o Fill: a etapa pede a cor no C&F e o soltar no alvo) → `main` (2026-10-20)

> A linha fecha e PARA: integrar e shipar só por ordem do Enio, pelo agente integrador (DIRETRIZ §1.5.3).
> Mecanismo, medições e recusas: [doc W13](../11_W13_o_tutorial_ensina_o_fill.md). Pedido:
> [PROMPT_W13](PROMPT_W13_o_tutorial_ensina_o_fill_2026-10-20.md).
> ⚠️ **W8–W12 ainda NÃO estão no `main`** (mesma branch, integram juntas): [W8](HANDOFF_INTEGRACAO_line_Macros_2026-10-17_W8.md)
> · [W9](HANDOFF_INTEGRACAO_line_Macros_2026-10-17_W9.md) · [W10](HANDOFF_INTEGRACAO_line_Macros_2026-10-19_W10.md)
> · [W11](HANDOFF_INTEGRACAO_line_Macros_2026-10-19_W11.md) · [W12](HANDOFF_INTEGRACAO_line_Macros_2026-10-20_W12.md)
> continuam válidos para a parte deles. Este só acrescenta a W13.
> ⚠️ A linha **NÃO foi rebaseada** (ordem do dono: «sem reset nem rebase»): o integrador rebasa.

## §1 — Base e HEAD

- Branch `line/Macros-Automação`, worktree `Worktrees/line-Macros-Automação/`. Merge-base `ad5f8fe39c`; o `main` está em
  `100d34c8e0`.
- Commits da W13 (`git log --oneline 01bef30081^..HEAD`): `01bef30081` (prompt) · `ca5ead7dd4` (test: vermelho antes da
  cura — o tutorial com Pincelada · Fill · Pincelada fica com ZERO etapas; apagar o Fill inunda a tela) · `7b30ae8904`
  (feat: a cura) · `c1812bed3d` (testes que fecham os sobreviventes da mutação + doc W13) · `0554d2d9ec` (fotos do smoke e
  a redacção da cena) · `859e37dddd` (refactor: `etapas.rs` 721 → 661, novo `etapas_no_toque.rs`; o `fmt` do
  `the_pointer_clock_starts_where_the_paint_starts.rs` que a W12 deixou por formatar) · `59912f960e` (importações sem
  uso) · o commit deste handoff.

## §2 — O que a W13 entrega

1. **O critério tem dois tipos** (`TipoDoCriterio { Pincelada, Preenchimento }`). A região de um Fill = os texels que o
   gesto mudou para a cor dele (opaca) no replay (`tutorial_fill.rs::regioes_dos_fills`, corrida dentro de
   `EnsinoDoPainter::de`, fora do quadro). Cumpre-se com a cor a `TOLERANCIA_DA_COR = 26` por canal (medido: o rato
   alcança ≥ 3 níveis numa tira de matiz de 256 px; a paleta distingue < 50; ponto médio). Mesma régua da W7, 0,6655.
2. **Pincelada sobre um Fill** conta onde difere da cor do Fill (`fundos`); um Fill só julga onde nada depois repinta;
   uma pincelada só onde nenhum Fill posterior repinta; um Fill zera a cobertura das pinceladas anteriores.
3. **A etapa do Fill começa no `ColorDropComeca`** (`etapas.rs`): apagá-la já não deixa a entrada do C&F (o traço
   seguinte inundava a tela). Isto muda a âncora (`inicio`) das etapas de Fill gravadas desde a W11: uma edição feita
   numa etapa de Fill de uma macro guardada entre a W11 e a W13 tem `ancora` = o índice do `Down` e falha com
   `AncoraSemEtapa`. Aceite: a W11–W12 nunca chegaram ao `main` (só ficheiros de smoke do dono).
4. **Balão:** `PedidoDoTutorial::Cor { cor }` / `Solta { cor }`; amostra da cor na linha do título; anel em
   `PAINTER_RAIL_FILL`; alvo de soltar na imagem (`AreaNaImagem.solta`: anel de acento com `ROW_H_PX` × escala de UI e um
   ponto central). «Mostra-me» funciona nos dois. Chaves i18n `panel.macros.tutorial.fill_colour` e
   `panel.macros.tutorial.fill_drop`.
5. **«Mostra-me» de um Fill** repete o gesto (ponteiro, portas do C&F, modal) com a cor gravada e o limiar do toque
   (`etapas_no_toque.rs`, `NoToque.limiar`); o C&F volta ao modo do ALUNO, e a cor e o limiar do aluno são repostos.
6. **Roteiro:** `roteiro::grava_a_caixa_com_um_fill` (4 linhas que fecham um quadrado, azul (40,110,210) largado dentro
   pelo C&F + Done, uma pincelada amarela por cima); a Gota do roteiro aceita um limiar opcional pelo slider do modal;
   `cfg(test)` `grava_o_traco_sob_o_fill_e_por_cima`.
7. **Smoke:** `PH2D_MACRO_SMOKE=4 PH2D_MACRO_TUTORIAL=caixa` (sem número de cena novo, sem mudança na shell).
8. `tutorial.rs` 733 → 652 (Curva/Referencia foram para `tutorial_curva.rs`).

### §2-bis — Medições

Sonda `tutorial::fill::tests::sonda_a_caixa` (256², `load` ~170): Digital — de 19 ms / mede 46 ms; Wet Paint — de 370 ms /
mede 33,0 s. Razão do Fill: exacto 1,0 · outro ponto do quadrado 1,0 · cor a 26 níveis 1,0 · cor a 27 níveis 0,0 · cor
certa fora da caixa 0,0.

**Mutação 13/13 sangram** (1.ª rodada 10/13: sobreviveram M4, M8, M9, M10 → 3 testes novos em `tutorial_fill_tests.rs` +
o literal 26/27 no teste da tolerância; 2.ª rodada: os 4 sangraram). M1 a etapa do Fill não começa no `ColorDropComeca` ·
M2 `pinceladas_em` salta o Fill · M3 o Fill conta qualquer cor · M4 tolerância 27 · M5 `<=` → `<` · M6 pincelada sobre o
Fill contra o papel · M7 o Fill julga o que traços posteriores repintam · M8 sem zerar a cobertura · M9 «Mostra-me» com o
modo gravado · M10 sem o limiar gravado · M11 `cor_que_falta` nunca · M12 sem anel no C&F · M13 região sem a condição de cor.

Testes: T1 `tests/it/macro_tutorial_fill.rs` (2) · T2 lib `tutorial::fill::tests` (6 + 1 sonda ignorada) · T3 app-macros
`na_etapa_do_fill_o_balao_pede_a_cor_e_depois_o_soltar`.

## §3 — Foundational / partilhado tocado (aditivo)

| Crate | O quê |
|---|---|
| `ph2d-editor-core` | `macro_tutorial.rs` (2 variantes de `PedidoDoTutorial`, campo `AreaNaImagem.solta`, `BalaoDoTutorial::cor()`); `chrome/tutorial_balao.rs` (a amostra) |
| `ph2d-i18n` | 2 chaves |
| `ph2d-tool-painter` | `macro_track/{etapas,etapas_no_toque,tutorial,tutorial_curva,tutorial_fill,tutorial_fill_tests,roteiro}.rs`; `tests/it/macro_tutorial_fill.rs`, `macro_colordrop.rs` (helpers `pub`) |
| `ph2d-app-macros` | `tutorial.rs`; `tests/it/tutorial_painter.rs` |
| `ph2d-app-painter` | `painter_bridge_tutorial_area.rs` (o alvo de soltar), `macro_track/smoke.rs` |

Contratos congelados (§6) **não tocados**. Sem `Cargo.toml`, `PROJECT_SCHEMA`, crate nova ou ADR.

## §4 — O que um merge pode partir

- `AreaNaImagem` ganhou um campo: qualquer outra linha que construa `AreaNaImagem { .. }` por literal tem de acrescentar
  `solta: false`. `PedidoDoTutorial` ganhou 2 variantes: `match` exaustivos noutras linhas têm de as tratar.
- ⛔⛔ **Os códigos do `Passo` 53–56 continuam a colidir com a `line/PainterWatercolor`** (ver [W12 §4](HANDOFF_INTEGRACAO_line_Macros_2026-10-20_W12.md)).
- ⚠️ **Sobreposição de ficheiros com a `line/PainterWatercolor`** em `ph2d-tool-painter/src/macro_track/`
  (`git diff --name-only main...line/PainterWatercolor -- crates/ph2d-tool-painter/src/macro_track/` ∩ ficheiros da W13):
  **só `roteiro.rs`** (a W13 acrescentou `grava_a_caixa_com_um_fill`, a Gota com limiar e o teste; resolução: manter os dois lados).
  Os outros ficheiros que a W13 tocou nessa pasta não estão no diff dela.
- Uma macro com etapa de Fill gravada entre a W11 e a W13 perde a âncora das edições (§2 item 3).

## §5 — Contadores (DELTA sobre o merge-base, só a W13)

Crates novas 0 · chaves i18n +2 · ADR 0 · `PROJECT_SCHEMA` 0 · contratos 0 · cenas de smoke: nenhum número novo (`=4`
ganhou `PH2D_MACRO_TUTORIAL=caixa`; a próxima cena continua `=7`).

## §6 — Prova de fecho

| Portão | Resultado |
|---|---|
| `cargo fmt --all -- --check` | ✓ (depois de formatar o ficheiro de teste da W12) |
| clippy `-D warnings` `--all-targets`: tool-painter, editor-core, i18n, app-macros, app-painter, host-desktop | ✓ (tool-painter re-corrido verde depois de `59912f960e`) |
| `architecture_workspace_file_loc_cap` | ✗ → ✓ (`etapas.rs` 721 → 661) |
| `architecture_claude_md_cabe_no_orcamento` | ✓ |
| `censos-da-arvore-combinada.sh` | ✓ 12/12, 121/121 |
| `PH2D_SUITE=fecho BASE=$(git merge-base main HEAD) nextest-impacted.sh` | «Summary [1290.051s] 18771 tests run: 18761 passed (27 slow), 1 failed, 9 timed out, 11167 skipped», `load` 37 → 277 |

Os 10 vermelhos, cada um verde SOZINHO por nome — flakes de carga ([FLAKES_DE_CARGA.md](../../DevOps/FLAKES_DE_CARGA.md)),
nenhum dos ficheiros está no diff da W13:

| Teste | Falha | Sozinho |
|---|---|---|
| `g4_o_custo_do_rc_nao_depende_do_numero_de_luzes` (light2d-gpu) | FAIL 2,9 s, `load` 279 | ✓ |
| `pular_a_pincelada_e_esperar_nao_cumpre_nenhuma` | TIMEOUT 600 s | ✓ 3 min 40 s a `load` 287 (o único num módulo que a W13 toca; passou também na corrida vizinha anterior, 27/27 lib) |
| 4 de wetpaint/watercolor (opções/secagem) | TIMEOUT 180 s | ✓ |
| 3 `a_lei_da_tinta` | TIMEOUT 180 s | ✓ |
| `presets_de_fabrica::a_largura_e_a_prometida` | TIMEOUT 360 s | ✓ |

`agent-loop-profile.sh`:
```
  ✗ paralelismo de ferramenta              1.19/passo   alvo: >= 1,5  (12% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                129   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                278 : 135   alvo: <= 1,0  razao 2.1x (baseline: 4,3x)
  ✗ edicoes SEM conferir que casaram             2.1%   alvo: <= 2%  (17 de 820; o guarda R6 recusa-as desde 14/10)
  ✗ contexto relido por passo (media)         297 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               64 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```

Smoke compilado (2.ª corrida, depois de `rm -rf target/*/incremental`, 13 G debug + 2,3 G smoke libertados): «Finished
`smoke` profile [optimized] target(s) in 0.45s», zero `Compiling`.

## §7 — Smoke do dono e fotos

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && PH2D_MACRO_SMOKE=4 PH2D_MACRO_TUTORIAL=caixa ./target/smoke/ph2d-host-desktop`
2. A imagem abre em Paint/Wet Paint com um balão «Passo 1 de 6». Desenhe sobre a faixa marcada (ou carregue em «Mostra-me»)
   quatro vezes: são as quatro linhas da caixa.
3. No «Passo 5 de 6» o balão mostra um quadrado azul e o código #286ED2, e põe um anel no botão C&F. Carregue no C&F,
   escreva o código na janela da cor (ou escolha a olho) e feche-a: o texto muda para «Arraste do botão C&F com anel até ao
   sítio marcado…». Arraste até ao anel dentro da caixa, largue e carregue em Done: a caixa fica azul e o balão passa ao
   «Passo 6 de 6» (a pincelada amarela). Ctrl+Z volta ao passo 5.
4. Errado: o balão saltar do passo 4 para a pincelada amarela, aceitar uma cor claramente diferente, não haver anel no
   C&F, ou não haver alvo dentro da caixa. Nota: as linhas saem cinzentas claras (o Wet Paint dilui o pigmento escuro — é o
   mesmo da gravação).

Fotos (`smoke_2026-10-20_W13/`, 1930×1040, servidor X virtual com cliques de `xdotool`; o «Preparing» não se viu — o passo 1
a ~16–20 s; cada «Mostra-me» levou de 5 s a ~2,5 min com `load` 57–173):
[passo 1](smoke_2026-10-20_W13/caixa_01_passo1.png) · [a etapa do Fill, anel no C&F e alvo](smoke_2026-10-20_W13/caixa_02_fill.png)
· [depois do «Mostra-me»](smoke_2026-10-20_W13/caixa_03_depois_do_mostra.png) · [o Ctrl+Z](smoke_2026-10-20_W13/caixa_04_ctrlz.png)
· [log](smoke_2026-10-20_W13/caixa.log). Smoke do dono depois da W13: **OK (20/10)** — «smoke ok. siga».

## §8 — ABERTO

1. O critério do Fill aceita um Fill do aluno que **vaza** (linhas que não fecham): o excesso de preenchimento não é
   penalizado ([doc W13 §4](../11_W13_o_tutorial_ensina_o_fill.md)).
2. A métrica de cor é o máximo por canal (a do balde), não perceptual.
3. A linha-guia do balão, saindo do canto do cartão, atravessa o botão C&F (desenho existente da linha-guia do anel).
4. O custo da medição em Wet Paint (33 s a 256² sob carga) é a arquitectura da W7.
5. Seguem de pé os itens 1–3 do §8 da [W12](HANDOFF_INTEGRACAO_line_Macros_2026-10-20_W12.md) e 1–2 do §8 da
   [W8](HANDOFF_INTEGRACAO_line_Macros_2026-10-17_W8.md); e a colisão dos `Passo` 53–56 para o integrador.

## Para o integrador (`CLAUDE.md` §5.1, entrada «Macros & Automação»)

Novo «Último» = este handoff (os de W8–W12 continuam a valer para a parte deles). A frase da natureza pode ganhar «o
tutorial ensina o Fill» só se couber nos 700 bytes (gate `architecture_claude_md_cabe_no_orcamento`).

## Retomar

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && pwd && git branch --show-current`.
