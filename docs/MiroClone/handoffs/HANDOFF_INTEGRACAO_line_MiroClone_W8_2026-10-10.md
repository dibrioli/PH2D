# HANDOFF DE INTEGRAÇÃO — `line/MiroClone` W5 + W5b + W6 + W7 + W8 «Equipa assíncrona e saída» → `main` (2026-10-10)

> Para o **agente integrador** (DIRETRIZ §1.5.3–1.5.4, §1.5.9). A linha FECHOU e PAROU: não integra nem
> pusha. **Substitui** o [handoff W7](HANDOFF_INTEGRACAO_line_MiroClone_W7_2026-10-09.md) como o último da
> linha. W5, W5b, W6 e W7 (smoke do dono OK nas quatro) não foram integradas e a W8 continua por cima, na
> mesma branch — **integra-se tudo de uma vez**; para as anteriores valem os handoffs delas. Briefing:
> [continuação W8](HANDOFF_CONTINUACAO_W8_2026-10-09.md). Estudo (o Miro pela ajuda e pelas imagens oficiais
> medidas, o Excalidraw pelo oráculo, a triagem dos validadores de SVG/PDF do sistema, as leis K1–K10 ·
> R1–R5 · P1–P6 · E1–E7 · X1–X4 com fonte e número, as recusas, as réguas):
> [`../06_equipa_e_saida.md`](../06_equipa_e_saida.md).

## 1. Identidade

| | |
|---|---|
| branch | `line/MiroClone` · worktree `Worktrees/line-MiroClone` |
| base (merge-base) | `2a925859d` — **o `main` não andou** no fecho (`git rev-parse main` = `2a925859d`) ⇒ `--ff-only` se ninguém aterrar antes |
| commits da W8 | `8bac6a7f0`..`609763094` (13) + este handoff; o total da linha: `git log --oneline 2a925859d..line/MiroClone` |
| diff da W8 | `git diff --stat 74e5b7163..HEAD`: 125 ficheiros, +32 842 / −316 (código em `crates/`+`shells/`: 81 ficheiros, +10 640 / −306; o resto são as saídas do oráculo Excalidraw, as fotos e o estudo) |
| smoke | **por fazer pelo dono** (cena 10, §8) |

O que a W8 trouxe ao Quadro: **comentários** (a ferramenta `C` ou «Comment» na barra curta; um fio preso
ao objecto de cima ou ao ponto; a bolha anda com o objecto, azul por ler, branca lida, com o visto
resolvida; responder, editar e apagar as minhas; «Comments» no canto com o meu nome, «Show resolved» e a
lista que leva ao fio; `Shift+C` esconde-os), **versões nomeadas** (menu da aba → «Versions…»: guardar com
nome, renomear com duplo clique, restaurar como quadro NOVO à direita, apagar com 2.º clique),
**apresentar** (o botão «Present» do canto ou o menu da aba: os frames de topo como diapositivos por cima
da janela inteira, a câmara a viajar pela curva medida do Miro, setas/Espaço/Home/End, `Esc` devolve a
vista; o olho no painel de Frames esconde um frame da apresentação), **exportar** PNG (por área 4/16/49
MP, pela placa em ladrilhos), SVG e PDF (uma página por frame, com ligações entre frames, o texto
pesquisável) do quadro, da selecção ou dos frames, e **abrir/gravar `.excalidraw`** de ida e volta
(também o `.png`/`.svg` com a cena embebida).

## 2. Foundational / partilhado tocado — e porque é aditivo

| onde | o quê | porque é aditivo |
|---|---|---|
| `ph2d-board-model` | **formato 8** (`FORMAT_VERSION` 7 → 8): `Board::team` (`team.rs`: fios com `Pin` e mensagens, versões como bytes deflate-1 de um `BoardSet` de um quadro, `hidden_slides`) no FIM do `Board`; `BoardSet::insert_after`, `into_boards`; `Version::read` | `legacy.rs::read_v7` lê o formato 7; fixture `fixtures/format_v7.bin` **gravada pelo build `74e5b7163`** (teste temporário, revertido) — gate `a_format_7_file_still_opens_with_no_team`. ⛔ O `PROJECT_SCHEMA` (187) **não** se toca: o quadro é um blob com versão própria. Fecho: o `Put` sobe o `next_id` pelo maior id de GRUPO (um grupo colado podia repetir ids) |
| `ph2d-vector` | `record.rs`: `VectorScene::record`/`take_record` e `DrawOp` (Fill · Stroke · Layer · Pop · Glyphs · BlurredRect); um gancho preguiçoso (`self.rec(|| …)`) em cada porta que desenha; `stroke_path`, `blurred_rounded_rect`, `glyph_run` | desligada (`rec: None`, o normal) cada porta paga um `if let`; o fluxo do Vello é o mesmo (os testes de GPU/goldens do repo correram no nextest impactado). ⚠️ NÃO é o contrato congelado do §6 (esse é o `ph2d-vector-doc`, intocado) |
| `ph2d-board-render` · `ph2d-board-layout` | `ExportLook {paper, vector}` e `EXPORT_PAPER`; o desenho passa TODO pelas portas que gravam (o texto por `GlyphStyle` emprestado, recolhido só a gravar); `overlay.rs` cortado do `lib.rs` | gate `board_paint_never_bypasses_the_recording` (fecho: cobre TODAS as portas que não gravam — `inner_mut`, `append_transformed`, `fill_prepared`, imagens) |
| `ph2d-board-edit` | `present.rs`: `slides`, `first_slide`, `slide_camera` (margem 0,8 % do lado LIMITANTE), `camera_at` (P5 `1-(1-t/1,33)^3,76`) | módulo novo |
| `ph2d-board-export` (**crate nova**) | página → gravação → SVG (`svg.rs`, glifos em `<defs>`/`<use>`, ligações `<view>`+`<a>`), PDF (`pdf-writer` 0.15: um Form XObject por glifo, `ActualText` + camada Helvetica invisível, ligações `GoTo`, recursos por página) e PNG (feature `gpu`: `VelloPass` em ladrilhos de `max_texture_dimension_2d`) | drop-crate; a feature `gpu` evita o ciclo de dev-deps com `ph2d-render` |
| `ph2d-board-excalidraw` (**crate nova**) | escrever/ler `.excalidraw` (o elemento inteiro em `customData.ph2d`), o embebido no PNG (`tEXt` depois do IDAT) e no SVG (base64 latin-1) | drop-crate; `serde_json` com `float_roundtrip` (sem ele a volta perdia 1 ULP) |
| `ph2d-editor-core` | `documents_team.rs` (estado de SESSÃO: a ferramenta, o fio aberto, os painéis, a apresentação, os pedidos de ficheiro, a confirmação de apagar, a página dos painéis); `board_nav/team` (`board_team*.rs`: bolhas, cartão, lista, versões, exportar, apresentação); `NavButton::Present`/`Comments` na fila de cima do canto; `FacItem::Comment`; `Part::Eye` no painel de Frames; 2 `IconId` (`MessageSquare`, `Presentation`) + SVG Lucide; `HeroScreen::chrome_panel_at` devolve `None` a apresentar; `HeroScreen::toggle_legacy_chrome` (a porta do F9, recusada a apresentar); `board_nav::show_box` | variantes/ficheiros novos; `board_nav::pointer_down` ganhou o botão do ponteiro (um chamador) |
| `ph2d-i18n` | `board_team.rs` (módulo novo) | chaves novas `board.comment.*`, `board.version.*`, `board.present.*`, `board.export.*`, `board.io.*`, `board.ago.*`, `board.team.*`, `board.smoke.team.*` |
| `ph2d-app-board` | `io.rs` (o diálogo de ficheiro, exportar/importar, o nome de quem escreve em `~/.ph2d/board_author.txt`), `smoke_team.rs` (cena 10) | família do quadro |
| shell | `fase_pointer_subjects.rs` +6 linhas (`ph2d_app_board::io::service`); `legacy_chrome.rs` −4 (a lógica do F9 passou para o hero) | composição; `the_shell_only_shrinks` verde |
| `Cargo.lock` · `.typos.toml` | `pdf-writer` 0.15 (MIT/Apache-2.0) e as duas crates novas · `^FlateDecode$` (o nome do filtro do PDF) e as saídas do oráculo | — |

**Contratos congelados (§6):** nenhum tocado — `git diff --name-only 74e5b7163..HEAD` não tem ficheiro de
`ph2d-nodegraph`, `ph2d-tool-runtime`, `ph2d-vector-doc`, `ph2d-vec-*` nem `project_schema`
(`grep -E 'nodegraph|tool-runtime|ph2d-vec-|vector-doc|project_schema'` → vazio); o diff não contém
`PROJECT_SCHEMA` (`grep -c` → 0); os gates `architecture_*contract_surface` correram verdes no nextest impactado.

## 3. Números MEDIDOS

- **O Miro** (estudo §1.2, imagens oficiais medidas ao píxel): a bolha de comentário tem tamanho de
  INTERFACE (não escala com o zoom); a apresentação deixa **0,8 %** do lado limitante de margem e viaja em
  **T = 1,33 s** com `1-(1-t/T)^3,76`; o PNG exporta por ÁREA — **4 / 16 / 49 MP**.
- **Excalidraw 0.18.1** como oráculo (`corre_esquema.mjs`, 117 sondas): o `customData` sobrevive a
  `restore`/`serialize`/`export`; o nosso ficheiro abre lá e volta IGUAL (8/8 casos, re-corridos no fecho).
- **Validadores do sistema** (estudo §1.4): o SVG contra a placa difere **0,038 %** dos píxeis (resvg) e
  **0,024 %** (rsvg); o PDF **0,118 %** e **0,018 %** (pdftoppm); `qpdf --check` limpo; `pdftotext`
  devolve o texto. Os ladrilhos do PNG = a imagem inteira **ao bit**.
- **Régua da versão** (`ph2d-board-render/tests/it/measure_team_cost.rs`, `--release`): 100 mil notas,
  deflate 1 = **14,5 ms · 2,14 MB** (clonar 13,1 ms; deflate 6 52 ms · 1,43 MB) ⇒ nenhum tecto (R5).
- **Régua da saída** (`ph2d-board-export/tests/it/measure_export_cost.rs`, `--release`, placa AMD, tecto de
  textura 8 192 px), re-corrida no fecho sobre o código final (`8669c8092`, `load` 12 → 6 durante os 28 s;
os números batem com os de 09/10 a ~5 %), mínimo de 3 rodadas:

  | | 10 mil notas | 100 mil notas |
  |---|---|---|
  | gravar a página (vector, o detalhe inteiro) | 88 ms · 40 003 operações | 912 ms · 400 003 |
  | SVG | **99 ms · 17,6 MB** | **1,02 s · 176 MB** |
  | PDF (uma página) | **55 ms · 0,49 MB** | **0,59 s · 4,6 MB** |
  | PNG 4 / 16 / 49 MP (placa, ladrilhos) | 14 / 36 / 238 ms | 119 / 142 / 202 ms |
  | escrever `.excalidraw` · ler | 283 ms · 15,6 MB · 171 ms | 2,69 s · 156 MB · 1,78 s |
  | guardar uma versão · lê-la | 1,4 ms · 168 KB · 2,0 ms | 16,0 ms · 1,63 MB · 23,7 ms |

  ⇒ **nenhum tecto** (estudo §4.2).
- ⛔ **Medido e curado na rodada:** o contorno de cada letra em cada ocorrência dava 10 mil notas em SVG de
  144 MB / 1,4 s e PDF de 90 MB / 3,1 s; cada glifo definido UMA vez e reusado ⇒ 16 MB / 96 ms e
  0,49 MB / 51 ms, o mesmo desenho ao píxel nos oráculos.

## 4. Portão batched (1× sobre `2a925859d..HEAD`)

| portão | resultado |
|---|---|
| `nextest-impacted.sh` (`BASE=2a925859d`, `PH2D_GPU=1`, `NO_FAIL_FAST=1`) | ✓ **17 525/17 525** (3.ª passagem, depois da mutação, 107 s) |
| `cargo check --workspace --all-targets` com `CARGO_BUILD_WARNINGS=deny` | ✓ |
| clippy do `ship.sh` (`--workspace --all-targets --features ph2d-spike/bevy_ecs`, `--keep-going`, `-D warnings`) · `ph2d-board-export --features gpu --all-targets` | ✓ 0 avisos · ✓ |
| `cargo fmt --all -- --check` | ✓ |
| `cargo machete` · `cargo deny --all-features check` | ✓ · ✓ (`pdf-writer` MIT/Apache-2.0) |
| `ph2d-editor-core --test it` · `ph2d-host-desktop --test it` (sem filtro) | ✓ 536 · ✓ 791 |
| `censos-da-arvore-combinada.sh` | ✓ 114/114, 12/12 censos correram |
| `typos` | ✓ |
| fotos: a janela (`fotografa_cena.sh`, tela virtual) e os sete estados (`fotos_dos_estados_da_cena_10`, placa) · oráculo do export (`ph2d-board-export --test it oracle -- --ignored`, placa) | ✓ · ✓ ladrilhos = inteira 0,0000 %; SVG 0,038 % (resvg) / 0,024 % (rsvg); PDF p.1 0,121 % / p.2 0,018 % (pdftoppm) |

Nomeados verdes: `shell_files_respect_hr18_loc_cap` (`file_loc_caps`), `workspace_src_files_under_loc_cap`,
`the_shell_only_shrinks`, `clamp_calls_in_ui_are_safe_or_justified` (`arch_safe_clamp_only`),
`no_magic_numeric_in_widget_or_screens`, `no_hex_color_literals_in_widget_or_screens`, `no_tofu_glyphs_in_ui_strings`,
`every_label_this_crate_paints_comes_from_the_string_table`,
`every_fill_in_a_border_colour_is_declared_and_every_ring_knows_the_door`,
`the_tail_of_a_block_is_never_written_at_the_painting_site`, `board_paint_never_bypasses_the_recording`.

⚠️ O que as passagens apanharam: (1.ª, antes das curas da auditoria) 26 números de UI sem token
(`y + s·0,25` para centrar texto → `text_y` e `control_gap_px`), o anel da bolha pintado como `fill` numa
cor de borda, `0xffff_ffff` e o preto da sala lidos como cor, PNG/SVG/PDF escritos no pintor, uma seta `→`
num teste, `Flate` no `typos`; (2.ª) quatro clippy no PDF/testes, fmt em 8 ficheiros, e **um vermelho
que só o nextest via**: `our_writer_still_writes_the_oracle_input` comparava o `.excalidraw` byte a byte e
no build do nextest o contorno de uma forma saiu com o último dígito diferente (`114.47324588220454`
contra `…46`) — passava sozinho duas vezes. Cura: o gate compara a ÁRVORE JSON (chaves, ordem e textos
iguais, números a 1e-12) e o determinismo continua byte a byte no mesmo build. A foto dos estados apanhou
o contador da apresentação cortado («1/3 ·…») — a caixa pela porta do rótulo e o teste mede-o pelo censo
das elisões (vermelho com a largura antiga).

## 5. Auditoria de 2 lentes (antes da mutação) — todos curados com teste

**Lente 1 (as leis contra o estudo e os oráculos)** e **lente 2 (o ecrã: o que o artista consegue fazer
que não devia)**, curadas em dois blocos (`6014f77ae`, `c5ad32223`). Os de mais peso:

- **Uma versão perdia os frames escondidos** e restaurá-la mostrava-os — `save_version` guarda
  `hidden_slides`, o restaurado herda-os.
- **A margem do diapositivo era do lado errado** num frame alto (0,8 % da largura num ecrã largo) —
  `the_margin_is_of_the_limiting_side`; um `t` NaN deixava a câmara em NaN.
- **O PDF crescia com o quadrado dos objectos esbatidos** (cada página declarava todos os recursos) —
  recursos por página, `the_pdf_grows_linearly_with_faded_objects`; as ligações entre frames passaram a
  existir no PDF (`GoTo`) e no SVG (`<view>`+`<a>`); um caractere de controlo de um `.excalidraw` importado
  partia o XML do SVG — `svg_stays_xml_and_carries_links`.
- **O que se editava NO Excalidraw perdia-se na volta** (o `customData` ganhava sempre): a cor, o tracejado,
  a opacidade, o tamanho do texto, o redimensionar e os pontos de uma seta recta agora voltam —
  `style_resize_and_straight_points_edited_there_come_back`; uma cópia de uma forma-linha levava o texto
  da original — `a_copy_of_a_line_shape_keeps_its_own_text`. A curva/cotovelo editados lá ficam
  RECUSADOS com razão (estudo §3: a nossa rota é derivada).
- **Exportar «Frames» com nomes iguais escrevia por cima** — `-2` no nome
  (`links_work_on_one_page_and_frame_names_never_collide`); PNG sem placa avisa em vez de calar.
- **A ferramenta de comentar comia o botão do meio, o direito e o `Espaço`+arrastar** e ficava armada
  depois de escolher outra ferramenta — `only_the_primary_press_opens_a_thread`,
  `another_tool_or_esc_in_the_composer_disarms_the_comment_tool`.
- **Editar/renomear por ÍNDICE** apontava para outra mensagem/versão depois de um apagar — o índice viaja
  com o fio/quadro e segue a sua (`delete_asks_twice_and_the_edit_follows_its_message`,
  `a_version_delete_asks_twice_and_the_rename_follows_its_version`); **apagar era de um clique** — agora o
  1.º arma (o lixo aceso, a dica muda) e o 2.º apaga.
- **O cartão de um fio comprido saía da área** e os painéis tapavam os botões do canto — o cartão mostra
  as últimas que cabem e «N earlier», os painéis param por cima e paginam
  (`a_long_thread_card_stays_inside_the_area`, `a_long_panel_stops_above_the_corner_buttons_and_pages`);
  um painel de cada vez (`the_corner_panels_are_one_at_a_time`).
- **A apresentação deixava vivo o que estava por baixo** (os painéis recebiam a roda, o F9 trazia o chrome
  a meio), um frame sem fundo ficava preto, e sair largava a ferramenta de comentar —
  `presenting_owns_the_window_and_gives_the_tool_back`; todos os frames escondidos diziam «adicione um
  frame» — `present_with_every_frame_hidden_says_so`; o contador diz «1/3 · Esc».
- Exportar «Selection» sem selecção exportava nada — vale a área que existe
  (`a_selection_export_without_a_selection_is_the_board`); o tamanho do PNG no diálogo refazia as rotas a
  cada pintura — guardado por (quadro, revisão, escolha, selecção).
- Censos do fecho: o anel da bolha pela porta do tema (`fill_ring`), números de UI por token
  (`text_y`, `control_gap_px`), os rótulos PNG/SVG/PDF por chave, o preto da sala com `LITERAL-COLOR-OK`
  na linha, a seta `→` de um teste.

## 6. Prova de mutação

**49/49.** 49 mutações nas leis da W8 (modelo 7, apresentação 4, export 6, `.excalidraw` 6, família do
quadro 4, interface 21, gate da gravação 1), pelo agente `mutacao`: baseline verde, cada filtro com ≥ 1
teste (o `smoke_team` casava zero — corrido como `smoke::team`), âncoras com contagem 1, restauro por
`git checkout` + `touch`, `load` 3–14. 43 sangraram à primeira; **6 sobreviveram** e foram tapados e
re-mutados (sangram):

| id | mutação | o que faltava |
|---|---|---|
| M14 | cada conteúdo do PDF declara TODOS os estados de alfa | o teste usava uma opacidade só — agora 180 objectos com 10 e com 90 opacidades: < 1,5× os bytes |
| M21 | o texto de uma forma-linha copiada vai para o anfitrião do 1.º grupo | uma 2.ª cópia noutro grupo com o seu texto |
| M23 | a tolerância da comparação da árvore `.excalidraw` passa a 1,0 | `the_tree_comparison_forgives_only_rounding` (um ULP passa, 1e-3 não) |
| M27 | a cena 10 enquadra sem a caixa da margem | ⛔ os testes da cena corriam uma CÓPIA do `scene_team` (com o `show` antigo): agora correm o do smoke, e a caixa tem de caber na área |
| M37 | o cartão sem a borda esquerda | `card_x` com uma área mais estreita que o cartão |
| M45 | `chrome_panel_at` sem a guarda da apresentação | a pergunta era VAZIA (nenhum painel em (10,10)): agora um painel publicado debaixo, `None` a apresentar e `Some` fora (o controlo) |

## 7. Fixtures, instrumentos e o que vive versionado

- `crates/ph2d-board-model/fixtures/format_v7.bin` — gravado pelo build `74e5b7163`.
- `docs/MiroClone/ferramentas/excalidraw_oracle/` — `corre_esquema.mjs`, as entradas `w8_*` e as saídas do
  Excalidraw 0.18.1 (re-corridas no fecho depois das curas do leitor/escritor).
- `docs/MiroClone/ferramentas/equipa_saida_miro_notas.md` — a ajuda do Miro lida (citações e URLs).
- `docs/MiroClone/example.excalidraw` — o ficheiro da lição 5.
- `docs/MiroClone/ferramentas/cena10_foto.png` (a cena ao nascer, refotografada no fecho) e
  `cena10_estados/*.png` (o fio do Rui, as versões, o exportar, a apresentação — o teste ignorado
  `fotos_dos_estados_da_cena_10`, pela placa).
- Réguas `measure_team_cost.rs` e `measure_export_cost.rs` (correm à mão, cabeçalho de cada).

## 8. Smoke (o comando exacto)

```sh
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && PH2D_BOARD_SMOKE=10 ./target/smoke/ph2d-host-desktop
```

A cena 10: um «Team review» com três frames de topo (Problem · Ideas · Plan — os diapositivos), um
comentário do Rui por ler numa nota (a bolha azul «R»), um da Ana já resolvido, uma versão «Kick-off»
guardada, e cinco notas-lição na ordem real (comentar · versões · apresentar · exportar · abrir
`docs/MiroClone/example.excalidraw`). O gate `every_team_lesson_does_what_it_says` conduz cada lição pelo
caminho do ECRÃ.

## 9. O que fica ABERTO (baixo; nenhum é defeito de lei)

- Comentários e votos entre PESSOAS em ecrãs diferentes (menções, notificações): Etapa 2 (rede). Num
  ficheiro, o «novo» por pessoa (K6) é a notificação.
- Imagens no quadro: o `image` do Excalidraw salta na importação com aviso (o quadro não tem elemento
  imagem — onda própria).
- Pontos editados no Excalidraw numa seta CURVA ou em COTOVELO não voltam (recusa medida, estudo §3).
- O `.excalidraw` de 100 mil elementos pesa 156 MB (o `customData` inteiro é o que faz a volta exacta).
- ⚠️ Instrumentos: `cargo-test-narrow.sh` contava um erro de compilação como «0 testes» e o
  `cargo-check-narrow.sh` escondia avisos — na linha corri `cargo check --all-targets` com
  `CARGO_BUILD_WARNINGS=deny` à parte.

## 10. Para o integrador

- Integra-se W5+W5b+W6+W7+W8 juntas (handoffs anteriores para as delas). Base `2a925859d` = `main` ⇒ `--ff-only`.
- `Cargo.toml` raiz: o `exclude` do vendor do Snowball (W7) — ao fundir com outra linha que mexa no
  `exclude`, a lista **soma** (Mergiraf apaga remoções em arrays — confira por `git range-diff`).
- `.typos.toml`: `^FlateDecode$` no `extend-ignore-identifiers-re` (W8) e as isenções da W7.
- CLAUDE.md §5.1: a entrada do Quadro aponta este handoff e `PH2D_BOARD_SMOKE=1..10` (feito nesta linha).

## 11. A máquina

`rm -rf target/*/incremental` feito no fecho (41 G do `debug`, 4,4 G do `smoke`); nenhum processo da linha a
correr (`pgrep -af 'ph2d-host|cargo|rustc|nextest'` vazio).

## 11-bis. O binário do smoke

Compilado na worktree depois do último commit de código (`609763094`);
`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, a 2.ª corrida:

```text
    Finished `smoke` profile [optimized] target(s) in 0.20s
```
