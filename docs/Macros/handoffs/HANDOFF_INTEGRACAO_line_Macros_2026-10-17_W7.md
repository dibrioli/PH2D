# HANDOFF DE INTEGRAÇÃO — `line/Macros-Automação`, W7 (a imagem é do artista · a régua na idade da água) → `main` (2026-10-17)

> A linha fecha e PARA: integrar e shipar só por ordem do Enio, pelo agente integrador (DIRETRIZ §1.5.3).
> Mecanismo de cada item, com as tabelas MEDIDAS (não copiadas aqui): [plano W7](../04_plano_W7.md) — item 1 (a imagem) ·
> item 2 (a régua, a tabela do erro por instante) · item 3 (`macro_agua`) · item 4 (ferramentas do trilho) · item 5 (dica) ·
> item 6 (abertos da onda 1).
> Antes: [handoff de integração W6](HANDOFF_INTEGRACAO_line_Macros_2026-10-15_W6.md) (a W6 NÃO está no `main`: este
> handoff cobre a linha inteira, W4–W7; a W5 também não está).

## §1 — Base e HEAD

- Branch `line/Macros-Automação`, worktree `Worktrees/line-Macros-Automação/`. Merge-base com o `main`: `0aab2c1cb6`.
  ⚠️ **O `main` ANDOU: `git rev-list --count HEAD..main` = 54** (158 ficheiros mudados desde o merge-base; o W6 dizia 0).
  Interseção com os ficheiros desta linha (`comm` dos dois `diff --name-only`): `CLAUDE.md`, `project-memory/MEMORY.md`,
  `docs/Components/ferramentas/fotografa_cena.sh` e, em `ph2d-app-motion`, `lib.rs` · `motion_bridge.rs` ·
  `motion_state_fita_demo.rs` · `motion_state_verbos.rs`, mais `shells/desktop/src/render_loop/fase_motion_bridge.rs` (§4).
- HEAD de código `3c86a0a70a` (os commits de documentação deste handoff vêm por cima). W7 = `cf3be12286..HEAD`, 17 commits.
- `git diff --shortstat 0aab2c1cb6 HEAD` (a linha inteira, W4–W7): 338 ficheiros, +24 729 / −912.
  Só a W7 (`cf3be12286..HEAD`): 52 ficheiros, +3 416 / −458.
- Linha única de Macros ⇒ ordem trivial; os conflitos possíveis estão no §4.

## §2 — O que a W7 entrega (sobre a W6)

1. **A imagem é do artista** (decisão do dono 16/10: «decida pelo melhor… sem olhar custos») — `ea4597fce9`,
   `5549cbad24`, `e64721d213`. No Play, o píxel = a imagem ATUAL onde ela difere da imagem do FIM da gravação CRUA
   (sem edições de etapa); onde não difere, o palco. O traço ao vivo conta; o último quadro = a atual ao byte. O fim vem
   de um fio por versão da trilha crua (`fim.rs`); a mistura está em `PalcosDoPainter::sobe`. O Painter por id vem da
   família (`painter_bridge_queries::o_painter`; a shell encolhe 10 linhas). Gate do fim cru: `e64721d213`.
   Custo medido (1024², `smoke`, load 25→38, NÃO calmo): palco só **1,506 ms**; mistura sem mudança **1,884**; Painter a
   pintar todo o quadro **3,449**.
2. **A régua na MESMA idade da água** — `0e41450b74`, `0660d06a35`. Curvas gravada/base desde o `Up` de cada trilha até
   960 quadros assentados (`QUADROS_ATE_ASSENTAR`); o aluno é avaliado na idade dele (`segundos_desde_o_ultimo_up`);
   `FRACAO_DA_GRAVADA_QUE_CUMPRE` **0,703 → 0,6655** (fina 60,0 × torta 73,1: **6,55 pontos** de margem de cada lado; o
   Nada 0 nas curvas, ≤ 8,3 % depois). O gesto exacto cumpre no quadro do `Up` (era 0–321 quadros). Medidas em fios
   trabalhadores (núcleos/4, fila limitada, pela ordem). Fecha o ABERTO 1 do W6.
3. **`macro_agua` independente do relógio** — `b04faab535`, `ca898ca4aa`: `RitmoDaAgua`; o worker ritmado vive em
   `offthread_ritmo.rs` com a guarda de pânico (`6d518e20a8`). Fecha o ABERTO 2 do W6.
4. **Ferramentas do trilho no estado inicial** — `09c9f5ae8a`, `c6652bfa46`: JSON `trilho` + coberturas cruas
   `trilho/…`; red-first Clone 4 380 / Seleção 8 606 / Máscara 2 759 texels; censo estendido
   `o_estado_inicial_leva_as_ferramentas_do_trilho`; o Convert-to-curve prova a lista de formas. Fecha o ABERTO 5.
5. **Dica do Motion uma vez por versão** — `5215c2dd3b`: 5 descodificações em 5 quadros → **0**.
6. **Abertos da onda 1** — `717f74f4c5`, `28257f8c35`: instantâneo **72,3 MB** a 1024² / **285,6 MB** a 2048²; tabelas da
   onda 1 refeitas com a máquina calma (load 1,2–8,5), no plano; o Motion na reprodução herda `PH2D_GPU_COOK`; placa ×
   CPU por palco 0,302/0,023 · 0,249/0,023 · 0,241/0,024 ms (N = 1/4/16).

## §3 — Foundational / partilhado tocado, e porquê é ADITIVO

| Crate | O quê | Porquê não parte ninguém |
|---|---|---|
| `ph2d-tool-painter` | serde em `SelectionShape`/`SelectionEntry`/`CurveModel`/`HandleKind`; `set_wet_ritmo_de_teste` (porta INTERNA no censo); `wet_assentada`; `segundos_desde_o_ultimo_up` (`&self`); `imagem_aplicada` não destrutiva; `macro_estado/trilho.rs`, `tutorial_mede.rs`, `offthread_ritmo.rs` NOVOS | só derives e métodos novos; o que existia não muda de contrato |
| `ph2d-app-painter` | `fim.rs` + mistura em `PalcosDoPainter::sobe`; `painter_bridge_queries::o_painter` | consumidores novos |
| `ph2d-app-macros` | cache da dica; `idade_na_chave` | interno da família |
| `ph2d-panel-painter-layers` | SÓ testes: `Bancada::no_modo`, `gestos_da` | — |
| `ph2d-app-motion` | SÓ a sonda (cozimento na CPU intercalado) | teste/medição |
| shell | `fase_macros_do_painter` liga o «atual»; `merge_layers.rs` import de teste; −10 linhas | composição; a catraca da shell só desce |
| `.config/nextest.toml` | override 10 × 60 s dos testes da régua (medido: 39,8–44,6 s sozinhos) | só alarga tecto |

**Formato gravado.** O JSON do estado inicial do Painter ganha a chave `trilho` e as texturas `trilho/…`; um estado SEM
elas lê-se como antes (o de fábrica). `Macro.doc`/`ph2d-macro` sem diff; `PROJECT_SCHEMA` 189 sem diff.

**Contratos congelados (§6) — NÃO tocados.** Prova (corrida em `3c86a0a70a`):

```
git diff --name-only 0aab2c1cb6 HEAD | grep -E 'ph2d-editor-core/src/tool|ph2d-vec|ph2d-vector|ph2d-nodegraph/src/(op|manifest|cook)|schema'   # vazio
```

Vazio inclusive para o `layout.rs` da W6 (o padrão não o alcança; ver §3 do W6 para o `ph2d-nodegraph`).

## §4 — O que um merge pode partir

- Tudo o que o §4 do [handoff W6](HANDOFF_INTEGRACAO_line_Macros_2026-10-15_W6.md) lista (`tool/runtime.rs`, estado
  inicial, i18n, `chrome/mod.rs` gerado, nextest).
- `offthread*.rs` e `wetpaint*.rs` com `line/PainterWatercolor` — rode `collision-surface.sh` pelo caminho absoluto.
- `selection_shapes.rs` / `curve_model.rs` / `curve_handle.rs` (derives serde) com quem mexer na seleção.
- **O `main` andou 54 commits:** `ph2d-app-motion` (`lib.rs`, `motion_bridge.rs`, `motion_state_*`) e
  `fase_motion_bridge.rs` mudaram dos dois lados (aqui só a sonda e a ligação dos palcos) — reconfira após o rebase.
  `CLAUDE.md` §5.1 e `MEMORY.md`: conflito textual provável, resolva por linha de módulo.
- `.config/nextest.toml`: o override é UMA linha `filter`; dois acrescentos conflitam no mesmo texto.
- Tectos de LOC: `offthread.rs` fica em 650 (cortado, tecto 700); `palcos_tests` partido em `palcos_fim_tests.rs`.

## §5 — Contadores (DELTA sobre o merge-base)

Crates novas: **0** na W7 (2 na linha, W5d). Chaves i18n W7: **0**. Schema, ADR, contratos: 0. Cenas de smoke das Macros:
`=1..5` (a próxima é `=6`; gate `ph2d-app-macros` `tests/it/cenas_do_smoke.rs`). Recontar contra o `main` do dia — o número é o do código.

## §6 — Prova de fecho (17/10)

- `BASE=main PH2D_SUITE=fecho` (suíte impactada): **22 124 passed de 22 127**, 3 vermelhos: (a) o tecto de LOC de
  `offthread.rs` (732) — curado por corte em `6d518e20a8`; (b) e (c) dois timeouts de CARGA, testes do Painter e não da
  linha ([FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md)): `wetpaint::achatamento_tests::o_escorrimento_do_s_nao_se_corta_em_rectangulos`
  (11,8 s sozinho a load 36,7) e `wetpaint::mistura_camadas_tests::n_camadas_de_amarelo_sobre_azul_seco_convergem_sem_preto`
  (41,4 s sozinho a load 30,4).
  ⚠️ Os commits de cura DEPOIS da suíte foram re-corridos **POR NOME** (loc cap, `macro_agua`, offthread, palcos,
  `macro_portas`), não numa suíte nova.
- clippy `--all-targets -D warnings` nas 24 crates da linha: **0** (curas em `ffaa9652a1`). `cargo fmt` ✓ (`ab75f51299`) ·
  typos ✓ (`3c86a0a70a`) · índices de ADR/docs ✓.
- **Mutação:** 14/18 na 1.ª prova; os 3 sobreviventes reais fechados com gates provados VERMELHOS sob a mutação ⇒ **18/18**:
  lista de formas (`c6652bfa46`: «SelecaoConvertida: 3433 texels»); base no corte (`0660d06a35`: o Nada cumpre k 13, 18, 22
  com `QUADROS_ATE_ASSENTAR = 0`); fim com edições (`e64721d213`: «quadro 0: com a etapa editada, a ranhura não segue a
  lei contra o fim cru»).
- **Auditoria (2 lentes):** curado o pânico do worker ritmado que pendurava o pedido (`6d518e20a8`, gate
  `um_worker_ritmado_que_panica_falha_o_pedido_em_vez_de_pendurar`) e o tecto de LOC; o resto é baixo (§8).

## §7 — Smoke do dono e fotos

Binário `--profile smoke` na worktree, 2.ª compilação:

```
    Finished `smoke` profile [optimized] target(s) in 0.50s
```

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && PH2D_MACRO_SMOKE=5 ./target/smoke/ph2d-host-desktop`
   — a cena abre JÁ pronta: a imagem «Canvas» seleccionada em modo Paint (barra BRUSH · C&F · PICK…), o clipe «Macro»
   parado em Time 0, Autoplay desligado (`c32fb9ace4`; a `=1` continua a ensinar o Splash que toca sozinho). Pintar à
   mão na imagem e dar o Play da Timeline (▷, painel de baixo): a tinta fica por cima do clipe e, no fim do clipe, a
   imagem não salta. Pintar ENQUANTO o clipe toca: o traço aparece. ⚠️ O caminho do traço ao vivo pela pré-visualização da PLACA em
   Wet Paint só tem gate pelo contador — CONFIRMAR no smoke. Errado: a tinta some ao dar Play, ou a imagem salta no fim.
2. `… PH2D_MACRO_SMOKE=4 ./target/smoke/ph2d-host-desktop` — pintar a pincelada por cima da faixa: o balão avança no
   momento de soltar; pular e esperar: não avança.

Fotos (`fotografa_cena.sh`, 1930×1040, `smoke`): `=5` (`/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/a0080127-b09b-4ff2-8804-aac5af83d76e/scratchpad/cena5.png`,
espera 12 s) abre com a barra do Painter, a Canvas seleccionada na Hierarchy, Time 0, ▷ parado, Autoplay desmarcado,
estado EDIT e o painel Macros com 4 passos; `=4` abre com a faixa marcada + balão «Step 1 of 4» + Restart/Show me;
`=1` abre com a macro no clipe e o painel Macros com 4 passos. **Smoke do dono: PENDENTE.**

## §8 — ABERTO

1. Medir as 30 pinceladas a 1024² leva ~10 min fora do quadro (1.ª medida em 14 s; cada pincelada cumpre-se quando a dela
   chega). Partilhar uma corrida entre a gravada da k e a base da k+1 cortaria metade — NÃO medido.
2. Cozimento do Motion: a CPU custa 10–15× menos por palco nesta cena (placa sincronizada) — política do módulo Motion,
   não da macro.
3. O pânico do fio do fim (`fim.rs`, `colhe`, `join().ok()`) fica em silêncio; o fio `macro-referencias` diz «macro
   estragada».
4. A idade do aluno congela com outra ferramenta ativa (o tique só chega à ativa — coerente com a gravação) e «Recomeçar»
   não a zera.
5. A sprite sem Painter ligado só se relê ao acordar o palco.
6. O tint da Máscara (só da pré-visualização do editor) não entra no «atual».
7. Água que continua a correr no editor depois do Parar aparece como «atual» o clipe inteiro (a lei do dono, aplicada).
8. Com etapas editadas, o último quadro mostra o editado e salta para a imagem crua quando o clipe pára (consequência de
   «fim = gravação crua»).
9. `censo_dos_controlos.rs` tem 1 391 linhas (dívida anterior, em `tests/`).
10. Smoke do dono de `=5` e `=4` (§7).

## Retomar

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && pwd && git branch --show-current`.
