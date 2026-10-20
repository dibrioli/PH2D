# HANDOFF DE INTEGRAÇÃO — `line/Macros-Automação`, W11 (o C&F a gravar: trocar a cor não pára a macro; o ColorDrop grava-se) → `main` (2026-10-19)

> A linha fecha e PARA: integrar e shipar só por ordem do Enio, pelo agente integrador (DIRETRIZ §1.5.3).
> Mecanismo, decisões e recusas: [doc W11](../09_W11_o_cef_a_gravar.md). Pedido: [PROMPT_W11](PROMPT_W11_trocar_a_cor_nao_quebra_a_macro_2026-10-19.md).
> ⚠️ **W8, W9 e W10 ainda NÃO estão no `main`** (mesma branch, integram juntas): [W8](HANDOFF_INTEGRACAO_line_Macros_2026-10-17_W8.md)
> · [W9](HANDOFF_INTEGRACAO_line_Macros_2026-10-17_W9.md) · [W10](HANDOFF_INTEGRACAO_line_Macros_2026-10-19_W10.md) continuam válidos para a parte
> delas. Este handoff só acrescenta a W11.
> ⚠️ A linha **NÃO foi rebaseada** (ordem do dono: «sem reset nem rebase»): o integrador rebasa.

## §1 — Base e HEAD

- Branch `line/Macros-Automação`, worktree `Worktrees/line-Macros-Automação/`. Merge-base `ad5f8fe39c`; o `main` está em
  `100d34c8e0` (os mesmos 3 commits de processo que o handoff da W10 lista).
- Commits da W11: `b339b163d4` (prompt) · `cc43e8b0f8` (test: vermelho antes da cura) · `b0f9eb6c80` (fix) · o commit
  deste handoff (docs + fotos) · `6d6fb11904` (smoke do dono 20/10: um `PH2D_MACRO_CAMINHO` desconhecido corria a base
  calada; agora a consola lista os nomes e a cena não conduz — o ColorDrop à mão passou). `git diff --shortstat 13673c4dcd..b0f9eb6c80`: **20 ficheiros, +635 / −64**.

## §2 — O que a W11 entrega (smoke do dono 19/10: trocar a cor no C&F a gravar deixava 4 recusas e o ▷ parava)

1. **Shell:** o C&F só entra em Fill quando o `Down` cai na pegada da sprite (`deliver_canvas_pointer_colordrop`,
   `painter_canvas_input.rs`; `fill_drag.rs`). O arrasto que não chega à tela só abre o seletor — nenhuma porta do
   preenchimento. A margem do gizmo do Deform não conta para esse `Down` (como antes: o modo já era Fill no teste).
2. **As 4 portas do ColorDrop gravam** (`fill.rs`, `porta!`): `begin_colordrop_fill`, `set_fill_threshold`,
   `fill_commit`, `restore_after_colordrop` (`portas.rs`: Recusa → Grava). O tocador aplica-as (`tocar.rs`).
3. **A etapa «Fill N»** (`etapas.rs`): o ColorDrop vai do `Down` ao passo que o fecha (Done/Cancel do modal, ou o
   `FillCommit` do gesto parado, que não tem `Up` — sem isto as cores das pinceladas seguintes desalinhavam no painel).
   A cor do Fill edita-se e cobre o limiar afinado depois de soltar; o tamanho é recusado e o painel não o mostra.
4. **Smoke `=6`**: variantes `cor` e `colordrop` (`caminho.rs`), conduzidas pelos gestos da janela (o botão C&F pelo
   `hit_index`, o arrasto, o valor do seletor, o Done do modal pelo `apply_event`) — `macro_smoke.rs::gesto_do_cef`.

### §2-bis — Prova

- **Gate vermelho antes** (`cc43e8b0f8`): `tests/it/macro_colordrop.rs` — os 2 testes reprovaram em «recusas na trilha».
  Depois: 3 testes (troca de cor pelo seletor; ColorDrop modal e parado tocam ao byte; a etapa Fill, a cor editada, o
  tamanho recusado) + `seam_macros::um_fill_mostra_a_cor_e_nao_o_tamanho`.
- **Mutação 10/10 sangram** (controlos verdes, âncoras 1×): porta `begin` de volta a recusa · tocador sem `LimiarDoFill` /
  `FillCommit` / `ColorDropComeca` · `Up` ignora o drop · o gesto parado não fecha a etapa · `e_um_colordrop` sempre
  falso · `edita_tamanho` sem guarda · o painel mostra o tamanho do Fill · descodificação do `54`. As mutações 5–8
  dependem de um só teste (`o_colordrop_e_uma_etapa_fill_com_a_cor_dele`).

## §3 — Foundational / partilhado tocado (aditivo)

| Crate | O quê |
|---|---|
| `ph2d-editor-core` | `macro_edits::TipoDaEtapaView::Preenchimento(usize)` — variante NOVA (quem fizer `match` exaustivo noutra linha deixa de compilar: acrescente o braço) |
| `ph2d-i18n` | chave `panel.macros.step.fill` = «Fill {n}» (**+1**) |
| `ph2d-tool-painter` | `Passo` **53** `ColorDropComeca(String)` · **54** `LimiarDoFill(f32)` · **55** `FillCommit` · **56** `ColorDropRestaura` (números para sempre; colisão só se outra linha acrescentar passos ao Painter — ninguém fora desta branch o faz); `TipoDaEtapa::Preenchimento`; 4 portas Recusa → Grava |
| `shells/desktop` | `deliver_canvas_pointer` passa a embrulhar `deliver_canvas_pointer_colordrop` (mesma assinatura para os 7 chamadores); `fill_drag.rs`; `macro_smoke.rs` |

**Shell: +96 / −30 = +66 linhas** (tecto soma entre linhas; `the_shell_only_shrinks` verde: muito abaixo do tecto).
Contratos congelados (§6) **não tocados** — `git diff --name-only main...HEAD | grep -E 'ph2d-editor-core/src/tool|ph2d-vec|ph2d-vector|ph2d-nodegraph/src/(op|manifest|cook)'` sai vazio (exit 1).
Sem `Cargo.toml`, sem `PROJECT_SCHEMA`, sem crate nova, sem ADR.

## §4 — O que um merge pode partir

- Outra linha que mexa em `painter_canvas_input.rs::deliver_canvas_pointer` (o corpo passou para a irmã `_colordrop`):
  conflito textual; a resolução é manter a lógica nova no corpo da irmã.
- `match` exaustivo sobre `TipoDaEtapaView` fora desta branch: ver §3.

## §5 — Contadores (DELTA sobre o merge-base, só a W11)

Crates novas 0 · chaves i18n **+1** · ADR 0 · `PROJECT_SCHEMA` 0 · contratos 0 · cenas de smoke: nenhum número novo
(`=6` ganhou 2 variantes de `PH2D_MACRO_CAMINHO`; a próxima cena continua `=7`).

## §6 — Prova de fecho (fecho de ONDA; a bateria inteira foi a da W10, em `d7abe5bab0`)

Cadência da suíte: 4 commits desde a última verde (< 10) — a bateria inteira não correu; corre no fecho/integração.

| Portão | Resultado |
|---|---|
| `cargo fmt --all -- --check` | ✓ |
| `PH2D_SUITE=fecho` clippy `-p` tool-painter, app-painter, app-macros, panel-macros, editor-core, i18n, host-desktop `--all-targets -D warnings` | ✓ |
| por nome: tool-painter `macro_` 88 · `fill` 51 · panel-macros `seam_macros` 7 · app-macros `gate7` 8, `tutorial` 13, `cenas_do_smoke` 2 · app-painter `macro_track` 28, string table 3 · editor-core shell/wiring/claude_md 1+2+3 · i18n 17 · host-desktop chrome 4, `fill_drag` 1 | ✓ todos |
| Mutação | 10/10 |

Smoke compilado (2.ª corrida, depois de `rm -rf target/*/incremental`): «Finished `smoke` profile [optimized] target(s) in 0.32s», zero `Compiling`.

`agent-loop-profile.sh`:
```
  ✗ paralelismo de ferramenta              1.18/passo   alvo: >= 1,5  (11% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                125   alvo: <= 800
  ✗ cargo test : cargo check                326 : 111   alvo: <= 1,0  razao 2.9x (baseline: 4,3x)
  ✗ edicoes SEM conferir que casaram             2.1%   alvo: <= 2%  (17 de 820)
  ✗ contexto relido por passo (media)         260 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               64 mil   alvo: <= 80 mil
```

## §7 — Smoke do dono e fotos

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && PH2D_MACRO_SMOKE=6 PH2D_MACRO_CAMINHO=cor ./target/smoke/ph2d-host-desktop`
   — sozinho: Gravar, pincelada preta, C&F premido e arrastado à esquerda da tela, solto (o seletor), vermelho, pincelada,
   Parar, ⏮, ▷. Certo: o painel MACROS lista só «Stroke 1» e «Stroke 2», e o ▷ desenha as duas, a 2.ª vermelha.
2. `… PH2D_MACRO_CAMINHO=colordrop …` — azul no C&F, o C&F arrastado para a tela e solto (a tela fica azul), Done,
   vermelho no C&F, pincelada. Certo: o painel lista «Stroke 1 · Fill 1 · Stroke 2» e o ▷ refaz o azul e depois o vermelho.
3. À mão, na `=5`: gravar, trocar a cor no C&F, pintar, Parar, ▷ — nenhum «Refused», as pinceladas tocam todas.

Fotos (`smoke_2026-10-19_W11/`, 1930×1040, perfil smoke):
[cor: a gravar](smoke_2026-10-19_W11/cor_gravando_a_2a_pincelada_vermelha.png) ·
[cor: ▷ a 4,40 s, sem recusas](smoke_2026-10-19_W11/cor_play_4.40s_as_duas_a_2a_vermelha_sem_recusas.png) ·
[colordrop: a gravar o Fill](smoke_2026-10-19_W11/colordrop_gravando_o_fill_azul.png) ·
[colordrop: ▷ a 5,92 s, Stroke · Fill · Stroke](smoke_2026-10-19_W11/colordrop_play_5.92s_fill_e_pincelada_painel_stroke_fill_stroke.png).
Smoke do dono depois da W11: **por fazer**.

## §8 — ABERTO

1. **O tutorial «fazer» salta o Fill** (`tutorial.rs` só ensina `Pincelada`): numa macro com ColorDrop o aluno não é
   levado a soltar a cor; o resto do tutorial corre.
2. A etapa Fill reconhece-se pelo `ColorDropComeca` antes do `Down` (o C&F é a única entrada do Fill no rail). Um Fill
   que entrasse por outro caminho (sem esse passo) ler-se-ia «Stroke».
3. Seguem de pé os itens do [§8 da W10](HANDOFF_INTEGRACAO_line_Macros_2026-10-19_W10.md).

## Para o integrador (`CLAUDE.md` §5.1, entrada «Macros & Automação»)

Novo «Último» = este handoff (os de W8–W10 continuam a valer para a parte deles). A frase da W10 mantém-se; acrescentar
«o ColorDrop do C&F grava-se (etapa Fill)» só se couber nos 700 bytes (gate `architecture_claude_md_cabe_no_orcamento`).

## Retomar

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && pwd && git branch --show-current`.
