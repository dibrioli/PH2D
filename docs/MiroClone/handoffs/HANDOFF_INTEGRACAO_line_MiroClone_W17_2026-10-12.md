# HANDOFF DE INTEGRAÇÃO — `line/MiroClone` W17 → `main` (2026-10-12)

> Para o **agente integrador** (DIRETRIZ §1.5.3–1.5.4, §1.5.9). A linha FECHOU e PAROU: não integra nem pusha.
> **Integra-se JUNTO com a W16, a W15, a W14 e a W12+W13** ([W16](HANDOFF_INTEGRACAO_line_MiroClone_W16_2026-10-12.md) ·
> [W15](HANDOFF_INTEGRACAO_line_MiroClone_W15_2026-10-11.md) · [W14](HANDOFF_INTEGRACAO_line_MiroClone_W14_2026-10-11.md) ·
> [W12+W13](HANDOFF_INTEGRACAO_line_MiroClone_W12_W13_2026-10-11.md), que seguem válidos para o que é delas). A W17 é a
> onda dos **defeitos conhecidos** (abertos (b) e (c) da W16 + dois que a própria onda achou). Briefing:
> [W17](HANDOFF_CONTINUACAO_W17_DEFEITOS_CONHECIDOS_2026-10-12.md). Oráculo Excalidraw 0.18.1:
> [`excalidraw_oracle/`](../ferramentas/excalidraw_oracle/README.md) §W17 (`saidas/w17/`).

## 1. Identidade

| | |
|---|---|
| branch | `line/MiroClone` · worktree `Worktrees/line-MiroClone` |
| HEAD | ver `git log -1` (este handoff é o último commit) |
| base (merge-base) | `9ea7821383` = `main` de hoje: a linha foi **rebaseada nesta sessão** (limpo, 90/90, sem Mergiraf) |
| commits | 95 em `git log --oneline 9ea7821383..HEAD` = W12+W13+W14+W15+W16+W17. Os da W17: `7e4c6064c2` oráculo · `e9fb1d2e3f` os cinco defeitos · `d294d7c419` cena 18 · `0e1b76532c` o «×N» pela tabela · `85060581ad` lição 4 + foto · + este handoff |
| diff da W17 em `crates/`+`shells/` | `git diff --shortstat e196b2ccb3 HEAD -- crates shells`: 32 ficheiros, +945 / −57 |
| smoke | cena 18 (`PH2D_BOARD_SMOKE=18`). Foto: `ferramentas/soltar_wayland/quadro18_depois.png` (`no_app.sh`, KWin virtual 1280 px) |

Ordem do dono (12/10): *«vamos primeiro corrigir os defeitos conhecidos»* antes de integrar. Cinco defeitos, UM bloco
(§0.10), cada teste correu **VERMELHO antes da cura** (E06 caixa 360 vs 180 · export descodifica 3 vs 1 · «Left out: , » vs
`all_gone` · nome `""` vs «Unnamed» · placeholder igual ao de chegada · aviso `None`).

1. **«Send to board…» com objecto sem nome → «Left out: , X».** `ph2d-app-board::send_to_board::Sent::name_of(Option<&str>,
   ObjectKind)`: nome vazio/só espaços → i18n `board.send.unnamed.{image,vector,flip,other}` («Unnamed
   picture/shape/drawing/object»). `ph2d-app-board` ganha a dep `ph2d-component-desc` (folha; re-exporta `ObjectKind`).
   As listas colapsam nomes iguais: `board.send.repeated` «{name} ×{count}» (nasceu como `format!` na família; a auditoria
   apanhou-o, HR-15, e passou à tabela).
2. **Objecto apagado entre o pedido e a escolha saltava-se em silêncio** (`continue` em
   `shells/desktop/src/render_loop/fase_send_to_board.rs`). A shell empurra `Sent::gone()` (nome «Deleted object», motivo
   `board.send.gone`); em `deliver`, vários e TODOS apagados → `board.send.all_gone` («Those objects are no longer in the
   scene»; antes caía em «Left out: , » / `wrong_kind`). Um só apagado → `board.send.gone`; mistura apagado+recusado sem
   ficheiros → «Left out: Deleted object, X». Teste da costura da shell:
   `the_phase_counts_the_deleted_and_names_the_nameless` em `shells/desktop/tests/it/o_send_to_board_leva_a_seleccao_com_a_pose.rs`.
3. **Imagem com cabeçalho bom e dados estragados ficava cinzenta para sempre, calada.** Oráculo corrido PRIMEIRO
   (`excalidraw_oracle/sonda_imagem_w17.mjs`, entradas `entradas/gera_w17_imagens.py` → `w17_png_truncado/png_idat_lixo/
   jpeg_lixo/jpeg_truncado`, saída `saidas/w17/w17_sondas.json`; **18 sondas, 0 falhas**; tabela no README §W17): o
   Excalidraw **NUNCA avisa** (estado `saved`, zero toasts/diálogos/consola), desenha o que o Chromium descodifica
   (truncado = linhas de cima; IDAT lixo = invisível; JPEG lixo = ruído); a rota de largar é igual.
   ⚠️ **Divergência deliberada** (ordem do dono + «recusa que só o terminal vê é botão mudo»): `ph2d-board-render`
   `Images::take_failed()` (ficheiros que falharam, CADA UM UMA vez por cache — `reported: BTreeSet<FileId>`) e
   `Images::failed()`; o placeholder de um ficheiro falhado é o MESMO ícone de imagem em `ColorToken::Danger` com um risco
   (o de chegada continua `Text2`); `ph2d-editor-core` `board_bar`, ao pintar: `if
   !live.render_cache.images.take_failed().is_empty() { live.notice = Some(("board.images.unreadable", now)) }` — a faixa de
   aviso do próprio quadro, sem shell. i18n `board.images.unreadable` («A picture on this board is damaged and cannot be
   shown»). `Documents::{notice, images_pending}` passam a públicos (testes do app-board). O export continua a omitir a
   imagem ilegível (inalterado: `image.rs` paint, export sem nível → return).
   **AO VIVO por construção, sem teste de dois pares:** os bytes do ficheiro viajam verbatim (`ph2d-board-sync`
   `replica_files.rs` `file_in`; gate `a_guest_image_reaches_the_others_through_the_host`), o `FileId` é por conteúdo, e o
   par pinta pelo mesmo `documents.live.render_cache` + `board_bar` ⇒ mesma marca e mesmo aviso.
4. **O PDF descodificava a mesma fotografia uma vez por PÁGINA.** `ph2d-board-export::ExportCache { unread, images }`
   (`Default`, `new(unread)`, `images()`); `paint_page(board, page, scale, paper, target, shared: &mut ExportCache, ts)`
   substitui o parâmetro `unread: &BTreeSet` (aridade 7 intacta) e troca o `Images` partilhado para dentro do `RenderCache`
   de cada página e de volta; `png::render(…, shared: &mut ExportCache, ts)` idem (os ladrilhos do PNG também partilham).
   `ph2d-app-board/src/io.rs` `write_export` constrói UM por exportação. 18 chamadores actualizados (io.rs ×3, png.rs;
   testes: `lib_tests`, `image_tests`, `image_w15_tests`, `tests/it` oracle/`measure_export_cost`/`measure_svg_export_images`,
   app-board `smoke_images_tests`). **MEDIDO** `crates/ph2d-board-export/tests/it/measure_pdf_export_images.rs`
   (`#[ignore]`, `--release`, mesmo processo, intercalado, mínimo de 5, `loadavg` 24): 6 quadros com a MESMA JPEG
   4000×3000 — **439,0 ms (mediana 529,1) → 72,1 ms (mediana 100,4)**; PDF idêntico, 4 035 479 B. Gate
   `one_export_decodes_a_photo_once_for_all_its_pages` (descodificador contador em `image_tests`: 3 → 1).
5. **Shift no recorte de imagem esticada ou já recortada.** Oráculo E01..E08. Esticada (kx≠ky): a lei JÁ coincidia
   (proporção natural em px naturais; a caixa guarda a proporção visível: 600×300, SE −80,−30 ⇒ recorte 360×270, caixa
   540×270). O defeito REAL: imagem **já recortada** — o Excalidraw mantém a proporção da JANELA de agora (2:3 ⇒ recorte
   180×270); nós forçávamos a imagem inteira 4:3 (360×270). Cura `ph2d-board-edit/src/crop.rs` `cropped`: `ratio` =
   (x1−x0)/(y1−y0) da janela já limitada no início do arrasto. Gate
   `crop::tests::shift_keeps_the_proportion_of_the_window_stretched_or_already_cropped` (números E02..E07).
6. **Cena de smoke 18** (`ph2d-app-board/src/smoke_w17.rs`; roteador `max_level: 18`): quadro «Damaged» (foto boa + a mesma
   foto com o PNG truncado a 60 % ⇒ riscada a vermelho + aviso), «Crop with Shift» (foto recortada 2:3, janela
   380,40,200×300 mostrada 133×200), «Same photo» (3.ª página, para o PDF), 4 lições + título. Gate
   `the_w17_scene_does_what_its_lessons_say` conduz a lição 2 pelo RATO real com Shift (o `Mouse` da cena 17 passou a
   `pub(crate)` com `mods`).

## 2. Foundational / partilhado tocado — aditivo, e o que um merge pode partir

| onde | o quê | o que pode partir |
|---|---|---|
| `ph2d-board-export` | `ExportCache`; `paint_page` e `png::render` recebem `&mut ExportCache` no lugar de `&BTreeSet<ElementId>` | ⚠️ **assinatura mudou**: qualquer `paint_page(`/`png::render(` que o `main` traga falha a compilar — reveja cada um |
| `ph2d-board-render` | `Images::{take_failed, failed}`, `reported`, placeholder `Danger` + risco | — |
| `ph2d-board-edit` | `crop.rs` `cropped` (razão da janela) | — |
| `ph2d-editor-core` | `board_bar` levanta `board.images.unreadable`; `Documents::{notice, images_pending}` públicos | o aviso do editor no mesmo quadro é sobrescrito pelo de «ilegível» (aceite) |
| `ph2d-app-board` | `send_to_board::{Sent::name_of, Sent::gone}`, `deliver` com `all_gone`, `io.rs`, `smoke_w17.rs`; dep `ph2d-component-desc` | `Cargo.toml` da crate |
| `ph2d-i18n` | `board.send.{unnamed.*, gone, all_gone, repeated}`, `board.images.unreadable`, `board.smoke.w17.*` | chaves novas |
| `shells/desktop` | `src` **líquido 0** (+1 `push`, −1 colapso `map_or_else`); `tests/it` +20 linhas | a catraca `the_shell_only_shrinks`: reconte depois do rebase |
| `Cargo.lock` | só a dep nova `ph2d-component-desc` em `ph2d-app-board` | regenerar |

**Contratos congelados (CLAUDE.md §6): nenhum tocado.** **Schema e formato do quadro: NÃO mudaram** (`FORMAT_VERSION` 13,
`PROTOCOL` 6). **ADR: a W17 não acrescenta nenhum.**

## 3. O portão (1× sobre o diff acumulado, BASE=`9ea7821383`, `loadavg` 27 no início, 88 no pico durante o clippy)

| passo | resultado |
|---|---|
| `fmt` · `typos` · `machete` · `cargo deny --all-features check` | ✓ |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | ✓ 0 avisos |
| clippy como o `ship.sh` (`--workspace --all-targets --features ph2d-spike/bevy_ecs -D warnings`) | ✓ |
| `nextest-impacted` | **18 641 / 18 641** (12 lentos, 271 s) |
| `file_loc_caps` 4/4 · `architecture_the_shell_only_shrinks` · `architecture_workspace_file_loc_cap` · censos i18n | ✓ |
| `scripts/censos-da-arvore-combinada.sh` | 117/117 (12/12 censos) |
| depois do portão, 2 commits pequenos (`0e1b76532c` pela tabela; `85060581ad` texto da lição 4): clippy `-p ph2d-app-board -p ph2d-i18n --all-targets` ✓ · `ph2d-i18n` lib 24 ✓ + `tests/it` 16 ✓ · app-board `send_to_board` 8 ✓ e `smoke::` 23 ✓ | ✓ |

**NÃO corridos** (itens do `ship.sh` fora do portão da linha): `cargo audit`, `check-standalone-optional.sh`,
`check-workflow-packages.sh`, índices de ADR/doc.

**Mutação** (agente `mutacao`, **12/12 sangraram**, árvore restaurada e `touch`-ada): M1 razão do recorte → `nw/nh` · M2
`name_of` sem o filtro de vazio · M3 `listed` sem colapso · M4 ramo `all_gone` desligado · M5 `items.push(Sent::gone())` da
shell removido · M6 `take_failed` sem dedupe · M7 `broken=false` · M8 `take_failed` vazio · M9/M9b cada `mem::swap` removido ·
M10 aviso do `board_bar` removido. ⚠️ M5 só é apanhada por um gate que LÊ o fonte (apanha a linha literal; o
comportamento está coberto no app-board).

**Auditoria de 2 lentes** (correcção de cada lei; costura): um só apagado ⇒ `board.send.gone`; apagado+recusado sem
ficheiros ⇒ «Left out: Deleted object, X»; janela de altura zero guardada por `max(1e-9)` e, abaixo do mínimo, recorte livre;
o aviso levantado no `board_bar` corre para o quadro activo. Achado CURADO: o sufixo «×N» era `format!` na família (HR-15) ⇒
chave i18n.

## 4. Abertos (nomeados)

- (a) Na **apresentação** (`board_team_show.rs`, export look Png) o GIF mostra o 1.º quadro — decisão de produto por tomar (W16 (a)).
- (b) Um fio de tocador por GIF à vista, sem tecto de quantidade (W16 (d)).
- (c) Os `<text>` de um SVG continuam a não se desenhar (recusa medida da feature `text` do usvg, de pé; W16 (e)).
- (d) NOVO: um ficheiro **truncado** não mostra nada da parte descodificável (o Chromium mostra as linhas de cima; a
  crate `image` rejeita o ficheiro inteiro) — exigiria um descodificador tolerante. Divergência anotada.
- (e) NOVO: o dono não tem caminho na UI para deixar um objecto sem nome (o renomear pode recusar vazio); o caso «sem
  nome» chega ao «Send to board…» por objectos nascidos sem `Name`. A lição 4 da cena 18 só o explica.
- (f) O gate M5 da costura da shell lê o FONTE (só apanha a linha literal).

Os abertos (b) e (c) da W16 (nome vazio/apagados; Shift do recorte) FECHARAM aqui.

## 5. O integrador

1. `git log --oneline 9ea7821383..HEAD` — W12+W13+W14+W15+W16+W17 + handoffs. Se o `main` andou, rebase; `Cargo.lock`
   regenerado; conflitos pelos estágios (DIRETRIZ §1.5.5).
2. Reconte de ADR (a W17 não traz nenhum); `PROJECT_SCHEMA` não é desta linha.
3. Árvore combinada: `cargo check --workspace --all-targets` + os portões do §3; **reveja cada `paint_page(` e
   `png::render(` novo que o `main` traga** (`&mut ExportCache`), os `match` sobre `DrawOp` e os literais de `ExportLook`.
4. **Entrada do §5.1 do CLAUDE.md** (≤ 700 bytes; só o link e a frase): link para
   `docs/MiroClone/handoffs/HANDOFF_INTEGRACAO_line_MiroClone_W17_2026-10-12.md` (com a W16, a W15, a W14 e a W12+W13 ao
   lado); `Smoke PH2D_BOARD_SMOKE=1..18`.

## 6. Perfil do agente (`bash scripts/agent-loop-profile.sh`, 20 sessões)

```
  ✗ paralelismo de ferramenta              1.13/passo   alvo: >= 1,5
  ✓ respostas por sessao (mediana)                172   alvo: <= 800
  ✗ cargo test : cargo check                     3.7x   alvo: <= 1,0
  ✗ edicoes pela ferramenta Edit                  42%   alvo: >= 80%
  ✗ contexto relido por passo (media)         406 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil
```

## 7. Smoke do dono

✅ **Smoke OK do dono (12/10) na cena 18.** A linha (W12+W13 … W17) está pronta para integrar quando ele mandar.

### Binário do smoke compilado

`rm -rf target/*/incremental` e depois, a 2.ª de
`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`:
```
▸ linha line_miroclone · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.32s
```

Passos:

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && PH2D_BOARD_SMOKE=18 ./target/smoke/ph2d-host-desktop`
   — abre o quadro «Known picture fixes»: três frames lado a lado («Damaged», «Crop with Shift», «Same photo») e
   quatro notas amarelas por baixo (as lições).
2. **Imagem estragada:** a cópia estragada aparece riscada a vermelho e a faixa de aviso diz que uma imagem está
   estragada. (Antes: cinzenta para sempre, sem aviso.) Exporte o quadro: a estragada não sai.
3. **Recorte com Shift:** no frame «Crop with Shift», duplo-clique na foto já recortada, arraste um canto segurando
   **Shift** — a janela mantém a proporção que tinha (alta, 2:3), não a da foto inteira.
4. **PDF:** exporte o quadro em PDF — três páginas (uma por frame) com a mesma foto, lida uma vez só.
5. **«Send to board…» sem nome:** a lição 4 explica; na aba da cena, um objecto sem nome ou apagado a meio aparece como
   «Unnamed …» / «Deleted object» no aviso, nunca como «Left out: , ».

**Como saber que deu errado:** a imagem estragada continua cinzenta e sem aviso; o Shift endireita a janela para a
proporção da foto inteira; o aviso do Send diz «Left out: , ».
