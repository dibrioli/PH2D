# HANDOFF DE INTEGRAÇÃO — `line/Macros-Automação`, W9 (a marca do artista · o laço do Parar · o custo da água) → `main` (2026-10-17, refeito no fecho de 18/10)

> **Fecho de 18/10:** depois do smoke reprovado de 18/10 a linha curou três coisas (o Play de cima · o arrasto da
> Timeline · o «piscar» do Parar, que era do instrumento) — §2 itens 7–9, §6 refeita sobre o HEAD novo, §7 e §8
> actualizados. Smoke do dono: **APROVADO** nas duas curas de produto.

> A linha fecha e PARA: integrar e shipar só por ordem do Enio, pelo agente integrador (DIRETRIZ §1.5.3).
> Mecanismo de cada item, com as tabelas MEDIDAS (não copiadas aqui): [plano W9](../07_plano_W9.md) — §9 mutação ·
> §10 auditoria e riscos.
> ⚠️ A **W8 ainda NÃO está no `main`**: o integrador envia W8 + W9 juntas; o
> [handoff da W8](HANDOFF_INTEGRACAO_line_Macros_2026-10-17_W8.md) continua válido para a parte dela (§8 dele incluído).

## §1 — Base e HEAD

- Branch `line/Macros-Automação`, worktree `Worktrees/line-Macros-Automação/`. Merge-base com o `main`: `ad5f8fe39c`.
  O `main` andou **2** commits de processo (`451c3ad14a`, `bdc7a38639`: a vez da placa); o único ficheiro em comum é
  o `CLAUDE.md` (§2 lá, §5.1 cá — secções diferentes).
- HEAD de código `9a295ee8a7`. W9 até ao 1.º fecho (17/10): `9ede4a4fac`, `8361acd912`, `8ed7bf6246`, `7472075157`,
  `bd15cf01e2`, `5d49341c68`, `7e386f8682`, `a406771e8a`, `bbc0d00a95`. Pós-smoke (18/10, código): `897b7acdd5`,
  `bc67bfa67e`, `3efd4bb9a7`, `688e6d2426`, `6d73bad9b7`, `dad214f1d5`, `9a295ee8a7`; o resto são docs.
- Os 12 commits da W8 (`a63825ac6f`..`45caa469c4`) estão na mesma branch e também por integrar.
- `git diff --shortstat main...HEAD` (W8+W9): 103 ficheiros, +7 156 / −1 478. Só a W9 (`45caa469c4..HEAD`): 77
  ficheiros, +4 254 / −733.
- Linha única de Macros ⇒ ordem trivial; os conflitos possíveis estão no §4.

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
7. **Pós-smoke — o Play de CIMA** (`3efd4bb9a7`): gravar de novo deixava o clipe sem Autoplay (o re-cronometrar mantinha o
   de antes), e o Play da barra de cima (`RunScene`) só toca clipes em Autoplay ⇒ a tela mostrava a imagem final de uma
   vez. Cura em `ph2d-app-macros::instala`: a 2.ª tomada fica em Autoplay. Gate `gravar_de_novo` (vermelho antes).
   Medido pelo registo da `=5` do dono, não suposto ([continuação](HANDOFF_CONTINUACAO_W9_SMOKE_REPROVADO_2026-10-18.md)).
8. **Pós-smoke — arrastar a Timeline deixava buracos retangulares** (`688e6d2426`): a cópia de um tocador (salto para
   trás, etapa editada) herdava a caixa suja do de origem; nos meios que sobem por caixa (Digital, Impasto) o espelho
   ficava de outro instante fora dela. Cura `TocadorDoPainter.por_ler` (`ph2d-tool-painter/src/macro_track/tocar.rs`): a
   1.ª leitura de um tocador novo ou copiado é a tela inteira. Gates `palcos_arrasto_tests` (4 meios; vermelho antes:
   Digital 98 texels, Impasto 13) e `a_copia_de_um_tocador_sobe_inteira_na_primeira_leitura`. No app (`=6 arrasta`):
   35/192 linhas erradas (máx 12 578 texels) sem a cura, **0/196** com ela; 7,00 → 6,93 quadros/100 ms no arrasto
   contínuo (ruído) ([continuação](HANDOFF_CONTINUACAO_W9_ARRASTAR_A_TIMELINE_2026-10-18.md)).
9. **Pós-smoke — o «piscar» de 1–2 quadros depois do Parar NÃO existia no ecrã** (`6d73bad9b7`). Medido com
   `PH2D_MACRO_SMOKE=6 PH2D_MACRO_CAMINHO=parar` (uma linha por quadro no 1,5 s depois do Parar) e uma espia que copia o
   que cada ranhura RECEBEU em cada subida: o quadro do Parar sobe ainda a ceder (o Parar entra em `fase_macros`, depois
   da subida) e o seguinte já troca pela gravação nova em t=0, ao byte (`fresca=0` em 87 quadros; controlo: a mesma
   régua vê a 1.ª pincelada aos 1,1 s e a 2.ª aos 2,9 s). O defeito era da linha de diagnóstico: ela corre no início do
   quadro, antes do `segue`, e recalculava a troca com a cedência DESTE quadro contra o palco do ANTERIOR. Cura: a linha
   diz o que a espia viu. Gates (`palcos_marca_tests`): `a_linha_do_caminho_diz_o_que_a_subida_anterior_mostrou`
   (vermelho antes: «troca=true epoca=deba marca=1132» no quadro do Parar) e
   `o_parar_nunca_troca_a_sprite_pela_mistura_da_gravacao_antiga` (guarda do produto, verde antes e depois; controlo: no
   Parar a ranhura TEM a mistura antiga). Mutação 3/3: época que não muda ao gravar de novo · `sobe` que ignora a
   cedência · espia sem trocas.

## §3 — Foundational / partilhado tocado (tudo ADITIVO)

| Crate | O quê |
|---|---|
| `ph2d-tool-painter` | `macro_track::marca` NOVO (`MarcaDoArtista`, `DeltaDaMarca`, `epoca_da_gravacao` FNV-1a); `UndoEntry.marca` (campo novo) + `UndoController.marca` (`MarcaViva`, `undo_marca.rs`); ganchos em `stamp_route.rs` (`stamp_dabs`), `wetpaint/dab_route.rs`, `wetpaint/composite.rs` (captura da água), `wetpaint/mascara.rs` (`pousa_na_agua`), `stamp_preview.rs` (`recomeca_os_discos`); `documents.rs` (`bind_document` limpa a marca); `reescrita_da_gravacao.rs` (suspensa, salta as marcadas, passa por `macro_recusa`); `runtime.rs` (`recomeca_com` suspensa; `begin_undo_step` limpa os bits da água); `PainterTool::poe_a_marca` / `marca_do_artista` / `imagem_aplicada_na_caixa` NOVOS; helper de teste `pub` `roteiro::grava_no_papel_ligado_com_o_worker` (enum `Relogio` Fixo/Maquina/Worker em `grava_com`); `portas.rs`: `poe_a_marca` decidida Ambiente; **dependência nova `ph2d-imageio`** (`MAX_RASTER_DIMENSION`) |
| `ph2d-app-painter` | `macro_track/mod.rs` (lei da mistura pela marca, enum `Subida`, leitura da caixa); `marcas.rs` NOVO (`Guardada`, `le_a_sprite`, `liga_a_marca`, `marcas_a_gravar` / `restaura_marcas`); `reescrita.rs` salta as marcadas + rebase; `fim.rs` (`Gravacao` perdeu `inicio`); testes partidos em `palcos_fim_tests.rs` + `palcos_marca_tests.rs`; `palcos_tests` `Placa` `pub(super)` + registos das caixas |
| `ph2d-app-macros` | `instala::rearma_o_laco`; `lib.rs` sincroniza o re-arme; dev-dep `ph2d-core` |
| shell | `fase_extract_inputs.rs` chama `liga_a_marca` a cada quadro antes de `reescreve`; `project.rs` campo `marcas_do_artista` **ÚLTIMO**; `project_save` / `project_load` / `project_load_read` (braço 191; 190 e 189 acrescentam um 0) / `migrate` / `test_writer` + 4 inicializadores de teste; `project_schema.rs` degrau 191 → 192; `project_schema_tests` tripla `(192, 13, 23)`; `project_marcas_tests.rs` NOVO; `project_scripts_tests.rs` nome temporário `scripts_vivo` (colisão de ficheiro temporário PRÉ-EXISTENTE do gate do CodeEditor no mesmo processo sob `cargo test`) |
| `ph2d-i18n` | +2 chaves: `shell.project_load.project_refused_format_191`, `shell.project_load.macro_marks_unreadable` |
| pós-smoke (18/10) | `ph2d-tool-painter` `tocar.rs` (`por_ler`, campo privado) + `composicao_tests.rs`; `ph2d-app-macros` `instala.rs`/`lib.rs` (Autoplay na 2.ª tomada) + `tests/it/gravar_de_novo.rs`, `CENAS_DO_SMOKE` 5 → 6; `ph2d-app-painter` `macro_track/caminho.rs` NOVO (a `=6`, o registo da `=5`, `passo`, `Espia`), `mod.rs` (campo `espia: Option<_>`, só ligado pelo diagnóstico), `palcos_arrasto_tests.rs` NOVO, `smoke.rs`, isenção nomeada `macro_track/caminho.rs` no gate de texto (consola/registo, nunca ecrã); shell `render_loop/macro_smoke.rs` (a `=5`/`=6` despachadas pelas portas da shell, +~110 linhas de composição) |

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
- `CLAUDE.md`: o `main` mudou a §2 (a vez da placa) e a linha a entrada das Macros na §5.1 — secções diferentes; confira
  que o merge guarda as duas e que a entrada continua ≤ 700 bytes (695 na linha).

## §5 — Contadores (DELTA sobre o merge-base)

Crates novas **0** · chaves i18n **+2** · ADR 0 · `PROJECT_SCHEMA` **+1** (191 → 192) · contratos 0. Cenas de smoke das
Macros `=1..6` (`CENAS_DO_SMOKE = 6`; a próxima é `=7`). Recontar contra o `main` do dia — o número é o do código. O
pós-smoke não tocou i18n, schema nem `Cargo.toml` (`git diff --name-only bbc0d00a95..HEAD | grep -E 'i18n|project_schema|Cargo'`
sai vazio).

## §6 — Prova de fecho

**Fecho de 18/10, em `9a295ee8a7` (o que vale):**

- `scripts/censos-da-arvore-combinada.sh` (`PH2D_SUITE=fecho`): **121/121 verdes**, 32 s. A 1.ª corrida apanhou 1 falha
  real — `every_word_this_family_shows_comes_from_the_string_table` (os literais de `caminho.rs`) — curada em `9a295ee8a7`.
- `BASE=$(git merge-base main HEAD) PH2D_SUITE=fecho nextest-impacted`: **17 765 corridos, 17 765 verdes, 0 falhas**,
  12 140 saltados, 1 097 s (`load` 13–15). Na varredura e verdes: `arch_safe_clamp_only` (2), `the_shell_only_shrinks`,
  `file_loc_caps` (4). Lentos-mas-verdes: os testes do tutorial 429–531 s, `a_largura_e_a_prometida` 278 s.
- clippy `-D warnings --all-targets` (flags do `ship.sh`) limpo em `ph2d-app-painter`, `ph2d-tool-painter`,
  `ph2d-app-macros`, `ph2d-host-desktop` — a 1.ª corrida apanhou 2 `chunks_exact_to_as_chunks` no gate do arrasto, curados
  em `dad214f1d5` · `cargo fmt --check` exit 0.
- **Auditoria pós-smoke (2 lentes, `bbc0d00a95..HEAD`)**: correção — nenhum defeito em `por_ler` (todo tocador nasce por
  `com_tool` ou `instantaneo`; a falha de `recomeca_ate` fica com o tocador antigo, que casa com o espelho) nem na
  espia (`None` = dois `if let` sem custo); uma decisão de produto no §8. Costuras — curados no fecho: o §5.1 dizia
  `=1..5`; o teste do condutor dependia de `PH2D_MACRO_CAMINHO` no ambiente (o `Caminho` guarda a variante); o relógio
  da cena saiu da shell para `caminho::passo`; doc-comments velhos.

**1.º fecho, 17/10, em `bbc0d00a95` (histórico, `load` alto):**

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

Binário: `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` — em `9a295ee8a7`: «Finished `smoke`
profile [optimized] target(s) in 10.77s».

**18/10 (pós-smoke):** o dono APROVOU o Play de cima e o arrasto da Timeline à mão na `=5`. O caminho sozinho:
`PH2D_MACRO_SMOKE=6 PH2D_MACRO_CAMINHO=<base|humano|meio|splash|semrewind|topo|arrasta|parar>`; registo na consola e em
`$TMPDIR/ph2d-macro-smoke.log`. Fotos da `=6 parar` (1930×1040, `fotografa_cena.sh`, rajada; a captura congelou 3+3
fotos com o mesmo hash — usadas as distintas, conferidas pelo `Time(s)`):
[a gravar](smoke_2026-10-18/parar_1_a_gravar.png) · [logo depois do Parar, Time 0 em branco](smoke_2026-10-18/parar_2_logo_depois_do_parar_0s.png)
· [Play 1,55 s, a de cima a nascer](smoke_2026-10-18/parar_3_play_1550ms.png) · [Play 2,08 s, só a de cima](smoke_2026-10-18/parar_4_play_2083ms.png).

**17/10 (o 1.º smoke):**

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
(`=1` toca por Autoplay). Smoke do dono de 18/10: reprovado no passo 2 («os dois traços aparecem de uma vez») → curado
(item 7) e aprovado; depois o arrasto → curado (item 8) e aprovado.

## §8 — ABERTO

1. Os riscos do [plano §10](../07_plano_W9.md): pincelada no quadro do `bind`; duas macros sobre a mesma sprite; mudança só
   de propriedade da camada não marca; gravar o projeto com pinceladas do Painter por assentar.
2. Da [W8 §8](HANDOFF_INTEGRACAO_line_Macros_2026-10-17_W8.md) seguem de pé os itens 1 (salto da água no fim do clipe), 2
   (a espera pela água funde acções globais), 3 (2 timeouts do Painter por registar na
   [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md); nesta corrida passaram lentos, 127–139 s) e 5 (o 2d vive na
   `line/motion-value`).
3. Baixo: a cena `=5` não arma por si o laço do relógio da Keys (`macro_smoke.rs` escreve o laço do doc; a troca de aba
   arma).
4. **Decisão de PRODUTO (do dono)**: gravar de novo põe o clipe da macro em Autoplay sem perguntar (item 7), e o
   `set_autoplay(…, true)` da Timeline tira o Autoplay dos OUTROS clipes do mesmo objeto (`ph2d-timeline/src/doc_entry.rs`)
   — quem marcou «Idle» como entrada perde-o ao regravar. Desfaz-se com o undo da Timeline; vale também para o Motion
   (`sessao.rs` passa as duas famílias por `instala`). É o que a 1.ª tomada já fazia.
5. Baixo: com a espia ligada (só nas `=5`/`=6`) cada subida inteira copia a tela na CPU — o registo à mão da `=5` paga-o.
6. Baixo: a shell `render_loop/macro_smoke.rs` cresceu ~110 linhas de composição da `=5`/`=6` (despacho pelas portas
   privadas da `App`, `ponto_da_tela`, a montagem da linha); a catraca passa. O relógio e o condutor já saíram para a crate.

## Retomar

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && pwd && git branch --show-current`.
