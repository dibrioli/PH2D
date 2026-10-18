# HANDOFF DE INTEGRAÇÃO — `line/Macros-Automação`, W9 (a marca do artista · o laço do Parar · o custo da água) → `main` (2026-10-17)

> A linha fecha e PARA: integrar e shipar só por ordem do Enio, pelo agente integrador (DIRETRIZ §1.5.3).
> Mecanismo de cada item, com as tabelas MEDIDAS (não copiadas aqui): [plano W9](../07_plano_W9.md) — §9 mutação ·
> §10 auditoria e riscos.
> ⚠️ A **W8 ainda NÃO está no `main`**: o integrador envia W8 + W9 juntas; o
> [handoff da W8](HANDOFF_INTEGRACAO_line_Macros_2026-10-17_W8.md) continua válido para a parte dela (§8 dele incluído).

## §1 — Base e HEAD

- Branch `line/Macros-Automação`, worktree `Worktrees/line-Macros-Automação/`. Merge-base com o `main`: `ad5f8fe39c`
  (**`git rev-list --count HEAD..main` = 0**, o `main` não andou).
- HEAD de código `bbc0d00a95`. W9 = 9 commits de código: `9ede4a4fac`, `8361acd912`, `8ed7bf6246`, `7472075157`,
  `bd15cf01e2`, `5d49341c68`, `7e386f8682`, `a406771e8a`, `bbc0d00a95`, mais o commit de documentação deste fecho.
- Os 12 commits da W8 (`a63825ac6f`..`45caa469c4`) estão na mesma branch e também por integrar.
- `git diff --shortstat main..HEAD` (W8+W9): 80 ficheiros, +5 653 / −1 451. Só a W9 (`45caa469c4..HEAD`): 52 ficheiros,
  +2 750 / −705.
- Linha única de Macros e `main` parado ⇒ ordem trivial; os conflitos possíveis estão no §4.

## §2 — O que a W9 entrega (por pedido do dono)

1. **A marca do artista substitui a régua por igualdade de píxeis** (opção 1 do dono). O que o artista pinta depois de
   gravar fica por cima; a água não marca. **A1 curado**: com a água real do worker a correr depois do Parar, cada quadro
   (Rewind e o clipe inteiro) é a tela byte a byte.
2. **A3 medido** (1024², 60 Hz, 280 quadros, `load` 185 = teto): sem marca 0,028 ms média / 0,319 máx por quadro; com
   marca e água **0,704 / 4,873** ms (antes 10,508 / 25,150 a reler a imagem inteira; W8 6,6 / 12,5); 0 subidas inteiras.
3. **A4 Loop — DEFEITO maior que a auditoria**: nem o 1.º Parar armava o laço vivo da aba Keys (`loop_range` `None`).
   Cura `rearma_o_laco` (`SetLoop`, só na vista Keys); o `lib.rs` re-arma também ao retemporizar o clipe ativo.
4. **▷ em Arrange NÃO é defeito**: é desenho da Timeline ([§4.1b](../../ObjectsAndComponents/01_objeto_e_componente.md)).
5. **A marca gravada no projeto**: `PROJECT_SCHEMA` 191 → 192, com leitura dos anteriores.
6. A frase da consola da cena `=5` já não promete «sem salto no fim do clipe».

## §3 — Foundational / partilhado tocado (tudo ADITIVO)

| Crate | O quê |
|---|---|
| `ph2d-tool-painter` | `macro_track::marca` NOVO (`MarcaDoArtista`, `DeltaDaMarca`, `epoca_da_gravacao` FNV-1a); `UndoEntry.marca` (campo novo) + `UndoController.marca` (`MarcaViva`, `undo_marca.rs`); ganchos em `stamp_route.rs` (`stamp_dabs`), `wetpaint/dab_route.rs`, `wetpaint/composite.rs` (captura da água), `wetpaint/mascara.rs` (`pousa_na_agua`), `stamp_preview.rs` (`recomeca_os_discos`); `documents.rs` (`bind_document` limpa a marca); `reescrita_da_gravacao.rs` (suspensa, salta as marcadas, passa por `macro_recusa`); `runtime.rs` (`recomeca_com` suspensa; `begin_undo_step` limpa os bits da água); `PainterTool::poe_a_marca` / `marca_do_artista` / `imagem_aplicada_na_caixa` NOVOS; helper de teste `pub` `roteiro::grava_no_papel_ligado_com_o_worker` (enum `Relogio` Fixo/Maquina/Worker em `grava_com`); `portas.rs`: `poe_a_marca` decidida Ambiente; **dependência nova `ph2d-imageio`** (`MAX_RASTER_DIMENSION`) |
| `ph2d-app-painter` | `macro_track/mod.rs` (lei da mistura pela marca, enum `Subida`, leitura da caixa); `marcas.rs` NOVO (`Guardada`, `le_a_sprite`, `liga_a_marca`, `marcas_a_gravar` / `restaura_marcas`); `reescrita.rs` salta as marcadas + rebase; `fim.rs` (`Gravacao` perdeu `inicio`); testes partidos em `palcos_fim_tests.rs` + `palcos_marca_tests.rs`; `palcos_tests` `Placa` `pub(super)` + registos das caixas |
| `ph2d-app-macros` | `instala::rearma_o_laco`; `lib.rs` sincroniza o re-arme; dev-dep `ph2d-core` |
| shell | `fase_extract_inputs.rs` chama `liga_a_marca` a cada quadro antes de `reescreve`; `project.rs` campo `marcas_do_artista` **ÚLTIMO**; `project_save` / `project_load` / `project_load_read` (braço 191; 190 e 189 acrescentam um 0) / `migrate` / `test_writer` + 4 inicializadores de teste; `project_schema.rs` degrau 191 → 192; `project_schema_tests` tripla `(192, 13, 23)`; `project_marcas_tests.rs` NOVO; `project_scripts_tests.rs` nome temporário `scripts_vivo` (colisão de ficheiro temporário PRÉ-EXISTENTE do gate do CodeEditor no mesmo processo sob `cargo test`) |
| `ph2d-i18n` | +2 chaves: `shell.project_load.project_refused_format_191`, `shell.project_load.macro_marks_unreadable` |

**Formato `Macro.doc`: sem diff.** O que muda no gravado é só o campo novo do projeto (schema 192).

**Contratos congelados (§6) — NÃO tocados.** Prova (sai vazio, exit 1):

```
git diff --name-only main..HEAD | grep -E 'ph2d-editor-core/src/tool|ph2d-vec|ph2d-vector|ph2d-nodegraph/src/(op|manifest|cook)'
```

## §4 — O que um merge pode partir

- **`PROJECT_SCHEMA` 192 é número que SOMA**: reconte contra o `main` do dia (`python3 scripts/schema-recount.py`); outra
  linha em 192 colide em silêncio. A tripla `(192, 13, 23)` e a escada vão juntas.
- `ProjectFile`: `marcas_do_artista` é o **último** campo (depois de `scripts`) — a ordem importa ao gravar.
- Braços de leitura 191 / 190 / 189 em `project_load_read`.
- `UndoEntry` ganhou `marca` (linha do Painter: `undo.rs`, `undo_record.rs`, `undo_absorb.rs`).
- A lista de portas (`portas.rs`).
- O split `palcos_fim_tests.rs` / `palcos_marca_tests.rs` (tecto de LOC de ficheiro).
- `ph2d-tool-painter/Cargo.toml` (dep nova) ⇒ `Cargo.lock`.

## §5 — Contadores (DELTA sobre o merge-base)

Crates novas **0** · chaves i18n **+2** · ADR 0 · `PROJECT_SCHEMA` **+1** (191 → 192) · contratos 0. Cenas de smoke das
Macros continuam `=1..5` (a próxima é `=6`). Recontar contra o `main` do dia — o número é o do código.

## §6 — Prova de fecho (17/10, `load` alto, máquina partilhada)

- `BASE=main PH2D_SUITE=fecho nextest-impacted` em `bbc0d00a95`: **17 758 corridos, 17 758 verdes, 0 falhas**, 12 140
  saltados, 759 s. Uma corrida anterior (`7e386f8682`) teve 4 falhas reais — `arch_color_space_typed` (`codifica`), tecto
  de LOC de ficheiro (`palcos_fim_tests` 862 linhas), `macro_portas` ×2 — todas curadas em `bbc0d00a95`. 24 lentos-mas-verdes
  (testes do tutorial 332–419 s, `a_lei_da_tinta` 127–139 s, sob carga).
- clippy `-D warnings --all-targets` limpo em `ph2d-tool-painter`, `ph2d-app-painter`, `ph2d-app-macros`,
  `ph2d-host-desktop`, `ph2d-i18n` · `cargo fmt --all --check` exit 0 · typos limpo nos 75 ficheiros do diff.
- Gates nomeados ✓: `file_loc_caps` (4), `the_shell_only_shrinks`, `architecture_workspace_file_loc_cap` (2),
  `architecture_claude_md_cabe_no_orcamento` (3), `a_tabela_inglesa_fala_ingles` (4), `arch_safe_clamp_only` (2),
  `a_schema_bump_anywhere_must_bump_the_project_schema`, `cenas_do_smoke` (2), `macro_portas` (5),
  `arch_color_space_typed` (4).
- **Mutação 20**: 16 sangraram à 1.ª; M1 com gate novo provado vermelho; M2 equivalente, medido (água parada fora de um
  traço aberto: 47 760 × 0 texels); M9 equivalente; M14 guarda redundante removida (plano §9).
- **Auditoria (2 lentes):** curas no plano §10 (enchimento molhado Solid, redimensionar, discos da pré-visualização,
  checagem do tamanho da caixa, tecto de ficheiro), cada uma com gate provado vermelho onde aplicável.

## §7 — Smoke do dono e fotos

Binário: `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` — 2.ª corrida: «Finished `smoke`
profile [optimized] target(s) in 0.23s».

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && PH2D_MACRO_SMOKE=5 ./target/smoke/ph2d-host-desktop`
   - (a) Record, 2 pinceladas de Wet Paint, Stop e logo ⏮ (Rewind) na aba Keys da Timeline com a água ainda a correr: a
     tela fica branca em Time 0; ▷ repõe as 2 pinceladas; sem resíduo. Errado: a imagem final (ou pedaços) em Time 0.
   - (b) Depois do Stop pintar outra pincelada noutro sítio; ▷: fica por cima. Ctrl+Z (ferramenta Paint) e ▷: sumiu.
   - (c) Gravar uma 2.ª vez, mais longa, e Stop: o laço do clipe cobre a gravação nova inteira.
   - (d) ▷ em Arrange não toca o clipe (desenho).
   - Conhecido: no fim do clipe a imagem mostra de uma vez a água que correu depois do Parar (W8 2c); a água de um traço
     novo que se espalha para fora do pincel só aparece no fim do clipe.

Fotos (`fotografa_cena.sh`, 1930×1040) em
`/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/7f8c5460-5e36-4494-b9b8-c03fa948dd5d/scratchpad/`:
`w9_cena5.png` (abre parada em Time 0, Canvas branco, aba Keys, Loop ligado, Record visível, 4 passos) e `w9_cena1.png`
(`=1` toca por Autoplay). **Smoke do dono da W9: REPROVADO no passo 2 (18/10: «os dois traços aparecem de uma vez»)** — ⛔ não integrar antes da [continuação](HANDOFF_CONTINUACAO_W9_SMOKE_REPROVADO_2026-10-18.md).

## §8 — ABERTO

1. Os riscos do [plano §10](../07_plano_W9.md): pincelada no quadro do `bind`; duas macros sobre a mesma sprite; mudança só
   de propriedade da camada não marca; gravar o projeto com pinceladas do Painter por assentar.
2. Da [W8 §8](HANDOFF_INTEGRACAO_line_Macros_2026-10-17_W8.md) seguem de pé os itens 1 (salto da água no fim do clipe), 2
   (a espera pela água funde acções globais), 3 (2 timeouts do Painter por registar na
   [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md); nesta corrida passaram lentos, 127–139 s) e 5 (o 2d vive na
   `line/motion-value`).
3. Baixo: a cena `=5` não arma por si o laço do relógio da Keys (`macro_smoke.rs` escreve o laço do doc; a troca de aba
   arma).

## Retomar

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && pwd && git branch --show-current`.
