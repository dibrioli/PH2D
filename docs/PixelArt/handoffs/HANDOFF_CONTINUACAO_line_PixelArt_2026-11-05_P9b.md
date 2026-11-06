# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, onda P9b (a tela do Export e Import) — 2026-11-05

> Leitor: o próximo agente da `line/PixelArt`. A linha CONTINUA; nada da P8d nem da P9 está no `main`.
> Plano: [P9_plano.md](../P9_plano.md). Motor (P9a): [handoff P9a](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-05_P9a.md). **O smoke do dono ainda NÃO foi feito: o veredito é condicional.**

## 1. Identidade

Branch `line/PixelArt`, worktree `Worktrees/line-PixelArt`, merge-base `c61f228b05`. Commit da tela `62bfd1fca3` sobre `0eeb2dce0c` (P9a), mais o commit deste handoff.

## 2. O que existe

### Menu File e atalhos
- `ids::CTX_MENU_PIXEL_{OPEN,IMPORT_SHEET,EXPORT_AS,EXPORT_SHEET,REPEAT_EXPORT}` em `MENU_BAR_FILE_ROWS` (`hero/menu_tables.rs`). Escondidas fora do Draw por `menu_row_is_hidden` (MenuBarFile); Repeat apagado por `menu_row_is_unavailable` sem export guardado.
- `PixelFileAction`, `PixelRequest::File`, `store.request` / `take_pixel_file_request`; `set_pixel_file_menu(Option<bool>)` é publicado pela família a cada quadro.
- Rótulos curtos porque `MENU_W` é fixo em 200 px e a foto mostrou corte: «Import Sheet… · Ctrl+I», «Export As…», «Export Sheet… · Ctrl+E», «Repeat Last Export». Os atalhos sem espaço no rótulo (`Ctrl+Alt+Shift+S`, `Ctrl+Shift+X`) funcionam mas NÃO aparecem no menu.
- Atalhos em `input_teclas.rs`: `Ctrl+E`, `Ctrl+I`, `Ctrl+Shift+X`; `Ctrl+Alt+Shift+S` em `com_alt`. Open não tem atalho (`Ctrl+O` é abrir projeto).

### Crate nova `ph2d-panel-pixel-export`
`canal.rs` (Tela, Contexto, Imagem, Pedido, PreviaDaFolha, Gravacoes, `imagem_de_import`) · `cards_export.rs` · `cards_folha.rs` (4 cartões Layout/Sprite/Borders/Output + Preview, em vez das 4 abas do Aseprite: decisão técnica, a casa é cartões) · `cards_import.rs` · `comum.rs` (listas, imagem no cartão, grelha) · `event.rs` · `paint.rs` (coluna no meio do canvas, arrasta pelo título; `pinta_lista` com faixa Reset/Cancel/verbo) · `textos.rs` (HR-15: frases de avisos/erros; as `Display` do motor ficam para logs) · `state.rs` · `ids.rs`.
- Registo: `registry-init`, `Cargo` do `shells/desktop` e `every_panel_the_shell_drives`, z-walk em `hero/paint.rs`, `panel_ids.rs`. i18n em `crates/ph2d-i18n/src/pixel_export.rs`.

### Motor novo em `ph2d-pixel-io`
- `exporta.rs`: `Export`, `ExportAs`, `ExportFolha`, `gravar`, `gravar_com`, `repetir`, `o_ultimo`, `lembrar`, texto das opções. A tag vai pelo NOME, a camada por id; alvo apagado recusa com `AlvoSumiu`.
- `abrir::{abrir_caminhos, sugerir_grelha, Aberto}`: vários PNG escolhidos no diálogo = sequência em ordem natural. É assim que se abre sequência; NÃO se procuram irmãos.
- `quadros_com`, `exportar_com`, `montar_com` (com fração de progresso). `saida::png_de` simplificado. Escala acima de `MAX_SIDE` recusa (`ErroSaida::GrandeDemais`).

### Formato
`ExportPreset { path, options }` em `Sprite`, fora do desfazer. Codec `PH2DPIXEL` MINOR 2→3, campo 11 do registro do sprite, só gravado se existir; leitor antigo ignora. `PROJECT_SCHEMA` intocado.

### Família `ph2d-app-pixel`
`exportar.rs`: diálogos do sistema e `frame`. `object_add::born_from_paste` também nasce `nasce_doc` (Open e Import); a shell cresceu 1 linha (`pixel.drena_barras(&mut g.jobs)` em `fase_pixel_panel.rs`). `modal::pick_files` agora respeita `responde_o_proximo`.

### Cena 20
`smoke.rs` `max_level` 20; reutiliza o palco da cena 8 (16 quadros, 4 tags). `smoke_exporta.rs`: `PH2D_PIXEL_SMOKE_GESTO=as|folha|import` (import lê `PH2D_PIXEL_SMOKE_FOLHA`) para as fotos; `PH2D_CROMO_SMOKE=4` abre o menu File.
Fotos nesta pasta: `P9b_menu_file_2026-11-05.png`, `P9b_export_as_2026-11-05.png`, `P9b_export_sheet_2026-11-05.png`, `P9b_import_sheet_2026-11-05.png`.

## 3. Medição M10 (folha grande)

`diag_quanto_custa_a_folha_grande` (`crates/ph2d-pixel-io/tests/exporta.rs`, `--ignored`, perfil smoke, mínimo de 3, load ~7, acima de 5: relógio INDICATIVO). `montar` de 1024²×256 ≈ 2040 ms (trim 2026, merge 1994). Acima dos 100 ms do plano ⇒ prévia e gravação em `Job` da casa, com barra na `JobQueue`.
- A prévia guarda uma foto `Arc<Sprite>` por (doc, stamp), para mudar opção sem clonar; a imagem no cartão reduz a 2048 px de lado (`LADO_DA_PREVIA`; o ficheiro não).
- **ABERTO:** clonar um sprite assim (~1 GiB) NÃO foi medido e acontece na linha da tela ao mudar o sprite com a janela aberta e a cada export.

## 4. Provas corridas (verdes)

- `ph2d-pixel-io`: tests `exporta` (7), `folha`, `ase` (`abrir_nao_perde_em_silencio`), `um_importador_so_com_os_campos_do_formato`, `o_godot_recorta_igual` (`--ignored`, Godot 4.7.2, 3 casos: default, trim+paddings, packed).
- `ph2d-panel-pixel-export --test it`: 11 testes (cliques reais em cada controlo).
- `ph2d-host-pixel-scenes --test it exportar::`: 10 testes (menu/atalho → quadro → disco ou objecto novo; Repeat dá os mesmos bytes; prévia = folha do disco; desenhar com janela aberta remonta).
- `ph2d-panel-registry-init --test it` inteira, 381 ok (registo de painel novo).
- Portões por nome em `editor-core` ok: `the_shell_only_shrinks`, `no_magic_numeric`, `no_literal_color`, `panel_wiring_parity`, `node_id_collisions`, `file_loc_cap`, `panel_functions_under_loc_cap`, `the_file_menu_items`, `panel_loc_cap`, `every_editable_field`, `hr12`. `every_panel_the_shell_drives` 2 ok. `clippy -D warnings` nas crates tocadas ok.
- Binário smoke compilado 2× (2.ª em 0,26 s, zero Compiling).
- **Tecto de `ph2d-app-pixel`:** 11 412 linhas por `cat | wc -l`, tecto 11 415. A folga acabou: o próximo código da família vai para FORA dela.

## 5. Smoke do dono (cena 20), resumido

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && PH2D_PIXEL_SMOKE=20 ./target/smoke/ph2d-host-desktop`
2. File ▸ Export As, formato GIF, Export. Tem de gravar o GIF; erro aparece na janela.
3. File ▸ Export Sheet: a prévia muda ao mexer em Sheet type e Border; Export com JSON grava folha + JSON.
4. File ▸ Repeat Last Export: repete sem abrir janela (os mesmos bytes).
5. File ▸ Import Sheet com a folha gerada: grelha sugerida; Import vira sprite novo.
6. File ▸ Open as Pixel Sprite com um `.ase`.
7. Filtro Edges (ver aviso abaixo).
Errado: menu cortado, Repeat aceso sem export prévio, prévia parada ao mexer nas opções, janela que não fecha ao cancelar o diálogo.
**Aviso a dar ao dono:** filtro Edges com opções padrão apaga imagem opaca, como no Aseprite (P9a §4).

## 6. Limites conhecidos, em aberto

- O clique no diálogo nativo não se prova em cena (usa `responde_o_proximo`); cancelar o diálogo (a janela fica aberta) só o smoke prova.
- `o_godot_recorta_igual` é `--ignored`: não corre na CI. Unity e Phaser não correram.
- Provas de mutação do plano §6 (lei do `.ase`, trim, direção, repeat em string, durações GIF) AINDA NÃO corridas: antes do fecho da linha.
- A janela não é modal: dá para desenhar com ela aberta, e a prévia acompanha.
- Open/Import só aparecem com um Pixel Sprite em Draw, como o dono decidiu; não há como abrir o 1.º `.ase` sem antes ter um sprite em Draw. Relatado ao dono.
- `curve_a11y`/`range_a11y`: continua fora (lacuna do app inteiro, ver P9a §5). Nenhum widget desta janela chega à árvore de acessibilidade, igual aos outros painéis.
- A linha do `CLAUDE.md` §5 do Pixel Art (smoke 1..19 → 1..20 e último handoff) fica para o fecho da linha.

## 7. Próximo

Smoke do dono da cena 20. Depois: mutações, e `/pd-linha-fechar` só por decisão dele; P10 segundo `02_plano_modulo.md`.

## 8. Avisos

- O que um merge pode partir: `MENU_BAR_FILE_ROWS`, `input_teclas.rs`, registo de painel (`registry-init`, `panel_ids.rs`, `every_panel_the_shell_drives`), MINOR do `PH2DPIXEL`, `smoke.rs` (cena 20) e o tecto de `ph2d-app-pixel`. Recontar os números pelo código, não por este texto.
- `cargo-test-narrow.sh` esconde a mensagem do assert: `cargo test -p <crate> --test <nome>` por `ph2d-run.sh` a mostra.

## Correção depois do 1.º smoke (dono: «Export As não abre»)

As 5 linhas do menu File pintavam e não tinham `store.register`: Down+Up não dava `Click`, em silêncio. Faltavam em `hero/pre_populate.rs` (a lista ao lado de `CTX_MENU_EXPORT_SVG`). Os testes verdes chamavam `apply_event` direto e não viam isso. Gate novo, por ponteiro real na tela inteira: `ph2d-panel-registry-init`, `the_pixel_export_answers_a_real_pointer` (5 linhas do menu, verbos e listas das 3 janelas); vermelho visto antes da cura, com `Export SVG` de controlo. Corrido 3×, verde. ⚠️ A mola dos cartões corre no relógio: sem `aquecimento::com_passo_fixo` o teste dá rects velhos e oscila.

## Correções do 2.º smoke (passos 5 e 6)

- **Passo 5, tempos perdidos no Import Sheet:** a imagem não guarda tempo e `importar_folha` dava 100 ms a tudo. `abrir::importar_folha_com` lê o `.json` ao lado (`dados_ao_lado`): a duração por centro do retângulo na célula, as tags só se a grelha tem um quadro para cada um do JSON (senão apontariam para outros). Sem JSON, a janela diz que todos começam em 100 ms. Gate `exportar::a_folha_importada_tem_os_tempos_da_exportada`; mutação «ignorar o JSON» reprovou com o sintoma do dono (`[100…]` × `[120,120,120,480…]`).
- **Passo 6, `.ase` borrado:** o `File ▸ Open as Pixel Sprite` no app real abre nítido e em Draw (foto conduzida, `PH2D_PIXEL_SMOKE_GESTO=abrir`). A foto do dono é a rota de arrastar ou `File ▸ Import…` (`ase_import`, folha animada, decisão dele: fica). Cura lá: `TextureFilter(Nearest)` no `import_ase`. Sem gate automático (pede o renderer); conferido só por foto de `PH2D_ASE_SMOKE=1`. Essa rota continua a abrir o Painter em Draw, pela decisão; só o Open edita em Pixel.
