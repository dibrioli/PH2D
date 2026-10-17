# HANDOFF DE INTEGRAÇÃO — `line/Macros-Automação`, W8 (editar uma etapa reescreve a imagem · o ABERTO da W7 fechado num bloco) → `main` (2026-10-17)

> A linha fecha e PARA: integrar e shipar só por ordem do Enio, pelo agente integrador (DIRETRIZ §1.5.3).
> Mecanismo de cada item, com as tabelas MEDIDAS (não copiadas aqui): [plano W8](../05_plano_W8.md) — decisões do dono ·
> item 2 (a imagem do artista, §2b a reescrita) · item 3 (partilhar a corrida, RECUSADO) · item 4 (`FioCaiu`) · item 5 (a idade) ·
> item 6 (sprite e Máscara).
> Antes: [handoff de integração W7](HANDOFF_INTEGRACAO_line_Macros_2026-10-17_W7.md) (a W7 JÁ está no `main`).

## §1 — Base e HEAD

- Branch `line/Macros-Automação`, worktree `Worktrees/line-Macros-Automação/`. Merge-base com o `main`: `ad5f8fe39c`
  (= o `main` de hoje: **`git rev-list --count HEAD..main` = 0**, o `main` não andou).
- HEAD de código `22ebe2b2d6`. W8 = 9 commits: `a63825ac6f`, `613462877c`, `03087e26d1`, `812d110304`, `31591c3999`,
  `8ad17d4799`, `929adff749`, `1f142fed00`, `22ebe2b2d6`, mais os commits de documentação deste fecho.
- `git diff --shortstat main..HEAD`: 38 ficheiros, +2 969 / −1 029.
- Linha única de Macros e `main` parado ⇒ ordem trivial; os conflitos possíveis estão no §4.

## §2 — O que a W8 entrega (por item do pedido do dono, sobre a W7)

1. **Smoke `=5` / `=4` da W7**: OK pelo dono (17/10).
2. **A imagem do artista** (decisões em [plano W8](../05_plano_W8.md)):
   - **2a regra dupla** — numa imagem que recebeu a gravação vale «≠ fim ⇒ atual»; nas outras, «≠ fim E ≠ início»
     (`03087e26d1`). Apagar até ao papel deixa apagado.
   - **2b editar uma etapa reescreve a imagem**, com transação: um Ctrl+Z desfaz a edição E a imagem, seja qual for
     das duas filas (Painter / global) que o receba (`31591c3999`, `8ad17d4799`, `929adff749`).
   - **2c fim = a água assentada** — gate ao byte com worker ritmado e quadros irregulares. Consequência decidida: a água
     que corre depois do Parar aparece de uma vez no fim do clipe (§8.1).
   - **2d** — prompt para a `line/motion-value` (escolher CPU/placa pelo tamanho):
     [PROMPT](PROMPT_para_line_motion_cozimento_na_reproducao_2026-10-17.md). Não é código desta linha.
3. **Partilhar a corrida (gravada da k−1 = base da k): RECUSADO, MEDIDO** — 0 de 29 iguais com a máquina, pior erro
   42,86 % da régua; com relógio fixo 29/29 por construção. Tabela no plano, item 3.
4. **`ErroDaMacro::FioCaiu`** — o pânico do fio do fim diz «macro estragada» (já não silencioso).
5. **«Recomeçar» devolve a idade** do aluno ao início (`613462877c`).
6. **6a** a sprite sem Painter relê-se pela identidade dos píxeis; **6b** o filme da Máscara vai por cima da mistura.
7. **Censo dos controlos** 1 478 → 616 + 684 + 187 linhas em três ficheiros, sha256 `8f976131…2019155`, `--list` igual.

⛔ **Defeito da W7 achado**: `imagens_da_gravacao` chamava `run_full` 2×; numa pilha trivial o 1.º leva a tela, o fim
saía vazio e a mistura nunca acontecia (a `=5` tem pilha não trivial: o smoke passou). Agora `imagem_aplicada`.

## §3 — Foundational / partilhado tocado, e porquê é ADITIVO

| Crate | O quê | Porquê não parte ninguém |
|---|---|---|
| `ph2d-tool-painter` | `UndoEntry.transacao` + `UndoController::transacao_seguinte` / `transacao_do_topo` (campo novo, levado só por quem o põe; o `absorb` conserva-o); `PainterTool::reescreve_a_gravacao` / `transacao_do_topo` / `agua_parada` / `versao_da_mascara` / `tinge_com_a_mascara` NOVOS; `recomeca_com` devolve a idade; `ImagensDaGravacao.modelo` + `ModeloDoFim` (campo novo) | métodos/campos novos; quem não põe a transação não a vê |
| `ph2d-tool-painter` (LEI) | `imagens_da_gravacao` = trilha que toca + água assentada | mudança de lei decidida pelo dono (2c) |
| `ph2d-app-macros` | `ErroDaMacro::FioCaiu` (variante nova; único `match` exaustivo: `erros.rs`, atualizado); roteiro: `Tela`, `grava_no_papel_ligado` (`pub`), variantes de teste; `portas.rs`: 2 portas declaradas | interno da família |
| `ph2d-app-painter` | `fim.rs`, `reescrita.rs` NOVO, `mod.rs` | consumidores novos |
| `ph2d-i18n` | 2 chaves: `shell.undo_app.macro_a_reescrever` (só consola), `app.painter.macro_reescrita.write_failed` | só acrescenta |
| shell | `ProjectUndo` com passos com transação (`push_undo_em`, `transacao_do_topo`), `App::reescrita`, a ligação em `fase_painter_dispatch` e `apply_undo`, a reescrita em `fase_macros_do_painter` (`fase_extract_inputs.rs`) | composição; `main.rs`/`app_state.rs` sem diff; `the_shell_only_shrinks` verde (folga de composição) |

**Formato gravado: NADA.** `Macro.doc` e `PROJECT_SCHEMA` sem diff.

**Contratos congelados (§6) — NÃO tocados.** Prova (corrida em `22ebe2b2d6`, sai vazio, exit 1):

```
git diff --name-only main..HEAD | grep -E 'ph2d-editor-core/src/tool|ph2d-vec|ph2d-vector|ph2d-nodegraph/src/(op|manifest|cook)|schema'
```

## §4 — O que um merge pode partir

- `shells/desktop/src/undo.rs` · `undo_app.rs` · `undo_route` — quem mexer no undo global: **a tupla do passo ganhou a
  transação**.
- `render_loop/fase_painter_dispatch.rs` e `fase_extract_inputs.rs`.
- `ph2d-tool-painter` `undo.rs` / `undo_record.rs` / `undo_absorb.rs` (linha do Painter: `transacao` no `UndoEntry`).
- A lista de portas (`portas.rs`).
- O gate `the_undo_preserves_the_vector_selection` procura agora `push_undo_em(base, mark,` — quem renomear a função
  reprova-o.

## §5 — Contadores (DELTA sobre o merge-base)

Crates novas **0** · chaves i18n **+2** · ADR 0 · schema 0 · contratos 0. Cenas de smoke das Macros `=1..5` (a próxima é
`=6`; gate `ph2d-app-macros` `tests/it/cenas_do_smoke.rs`). Recontar contra o `main` do dia — o número é o do código.

## §6 — Prova de fecho (17/10, `load` 110–166, máquina partilhada, NÃO calmo)

- `BASE=main PH2D_SUITE=fecho nextest-impacted`: **17 741 corridos, 17 736 verdes**.
  - 3 FAIL curados em `22ebe2b2d6` e re-corridos **POR NOME** verdes: `fn_loc_caps` `post_frame_undo` 202 → 193;
    `the_undo_preserves_the_vector_selection` (a agulha); `macro_portas` (2 portas).
  - 2 TIMEOUT de carga: `a_lei_da_tinta::o_glaze_do_pincel_escurece_o_cruzamento` e
    `sem_glaze_cada_preset_de_fabrica_pinta_a_cena_de_antes` (sozinhos passam em ~25 s a `load` 144/125; ficheiro NÃO
    tocado pela linha). **Não estão** na tabela de [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md): candidatos (§8.3).
- clippy `-D warnings` nas 6 crates ✓ (depois das curas `1f142fed00` / `22ebe2b2d6`) · `cargo fmt` ✓ · typos ✓.
- Gates ✓: `the_shell_only_shrinks`, `architecture_workspace_file_loc_cap`, `architecture_claude_md_cabe_no_orcamento`,
  `a_tabela_inglesa_fala_ingles`, `o_censo_dos_controlos_so_encolhe`, `a_linha_esmaecida_e_a_que_nao_age`, `cenas_do_smoke`.
- **Mutação 18/18** (16 na 1.ª; M5 com gate novo provado vermelho; M16 equivalente, medido) — detalhe no plano.
- **Auditoria (2 lentes):** curas no plano (`929adff749`).

## §7 — Smoke do dono e fotos

Binário `--profile smoke` na worktree, 2.ª compilação: «Finished `smoke` profile [optimized] target(s) in 0.33s».

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && PH2D_MACRO_SMOKE=5 ./target/smoke/ph2d-host-desktop`
   — a cena abre pronta (Canvas em Paint, clipe «Macro» em Time 0 sobre a tela branca).
   - (a) Pintar e apagar com a borracha até ao branco onde a macro pinta; Play: fica branco. Errado: a macro reaparece.
   - (b) Record, 2 pinceladas, Stop, esperar a água parar (≤ 1 min); selecionar «1. Stroke 1» e mudar Colour para
     outra cor: a imagem muda. Ctrl+Z com a ferramenta Paint ativa desfaz a cor E a imagem. Errado: desfaz só um
     dos dois, ou a imagem não muda.
   - (c) Modo Mask, pintar a máscara, Play: o filme continua POR CIMA da máscara. Errado: a máscara tapa o filme.
2. `… PH2D_MACRO_SMOKE=4 ./target/smoke/ph2d-host-desktop` — Restart volta a idade: o balão não avança sozinho.
   Errado: o balão avança sem soltar a pincelada.

Fotos (`fotografa_cena.sh`, 1930×1040) em
`/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/38c8f7f1-8888-4dbf-a1d3-39df9ee16bc1/scratchpad/`:
`w8_cena5.png` (+ `_02`, `_03`; 0 texels azuis: tela branca parada) e `w8_cena4.png` (faixa marcada).
**Smoke do dono da W8: PENDENTE.**

## §8 — ABERTO

1. **O salto da água no fim do clipe** — consequência decidida do 2c: a água que corre depois do Parar aparece de uma
   vez quando o clipe acaba.
2. A espera pela água (≤ 120 s) **funde as acções globais desse intervalo num passo** do desfazer.
3. Os 2 timeouts de carga do §6 por registar na [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md) (família do Painter,
   não desta linha; promover = tabela + faixa do nextest).
4. A reescrita compara camadas pelo `id`: uma camada criada depois da gravação não é tocada (por desenho).
5. O item 2d vive na `line/motion-value` (prompt linkado no §2).

## Retomar

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && pwd && git branch --show-current`.
