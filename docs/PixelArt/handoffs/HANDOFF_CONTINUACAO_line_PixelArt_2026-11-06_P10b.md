# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, P10b (tela dos tilemaps) — 2026-11-06

> Leitor: a próxima janela da linha, que faz a P10c. A linha continua; não integra nem faz push (o dono não ordenou).
> Plano: [P10_plano.md](../P10_plano.md) (§6.1b, §6.2, §8). Medições: [P10_oraculos.md](../P10_oraculos.md) (M6, M9, M9b). Anterior: [P10a](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-06_P10a.md).

## Estado

- Branch `line/PixelArt`, worktree `Worktrees/line-PixelArt`, merge-base `bbfefaf393` (o `main` ainda está lá). Árvore limpa, salvo um commit só de `cargo fmt` que a janela principal está para fazer (`ph2d-aseprite/src/write.rs`, `ph2d-pixel-io/tests/ase.rs`).
- Commits da P10b: `94941afa39` velocidade · `cd67695794` tela · `5b67ad67ed` prova por ponteiro e duplo clique · `88fc4ffc60` traço ao vivo + cartão segue a camada · `46a80e6788` tilesets partilhados · `31e98054d9` leis do M9.
- Abaixo: P10a (`4952bcf0ee`..`c30817c2a6`) e conta-gotas/zoom (`4b83b84fd7`, `286ce1ea95`).
- **Smoke OK do dono em 06/11 para a P10b** (todos os passos, recusas incluídas, cena 21). Não há ordem de integrar nem de push.
- Próximo: **P10c**.

## O que existe e não se refaz

| Peça | Onde |
|---|---|
| Painel: janela flutuante TILEMAP ao lado do PIXEL (`CAN_FLOAT`), cartões Draw e Tileset, janela New/Convert; canal `thread_local` (`Contexto`, `Pedido`, `Tela`, `Conjunto`) | `crates/ph2d-panel-pixel-tiles` (`canal.rs`) |
| Cola do app: `press`, `anda`, `cancela`, `tecla`, `aplica_pedido`, `compoe`, `frame`, `recusa_traco`, `recusou`, `realce` | `crates/ph2d-app-pixel/src/tiles.rs` |
| Cena de smoke 21 | `crates/ph2d-app-pixel/src/smoke_tiles.rs` |
| Motor dos tiles (P10a + mão, exemplo, tileset, grelha) | `crates/ph2d-pixel-tiles/src/{mao,exemplo,tileset,grelha}.rs` (+ `desenho`, `mapa`, `terreno`, `converte` da P10a) |
| Código puro que saiu de `app-pixel` (sem testes próprios, exercitado pelas cenas) | `crates/ph2d-pixel-ponte` |
| Core | `tiles::tilemap_layer_using`, `converte::para_tilemap_em`, `SpriteImage::{is_current, rgba_at}`, `PixelApp::canvas_rgba` |

`ph2d-pixel-tiles` passou a depender de `ph2d-pixel-render` e `ph2d-vector` (por causa de `grelha.rs`). A lei da P10a mantém-se: `Cel.tiles` é a verdade, `Cel.data` deriva.

## Leis decididas nesta onda

O dono delegou («buscar o estado da arte»). Memória: `project-memory/project_tiles_any_layer_oracle_decides_2026_11_06.md` (⚠️ por comitar na árvore primária). Base: plano §6.1b, `P10_oraculos.md` M9 e M9b.

- O traço em camada de tilemap mostra-se como o **resultado**: `tiles::compoe` aplica o MESMO op da gravação só enquanto compõe e repõe pelo inverso. Defeito do smoke: o `settle` recompunha só a janela do traço.
- As imagens do cartão são chaveadas também pelo tileset (trocar de camada não muda o carimbo).
- O tileset serve a qualquer camada. A janela traz o da camada activa (ou o último escolhido); só «New tileset» pede tamanho. Convert To reaproveita e não entra no gabarito.
- Canvas Size, Crop e Trim fora do múltiplo aceitam; a grelha vai com o canvas; quadro sem mapa segue a fase da camada (`Mapa::de`). Rotate/Flip Canvas e Sprite Size continuam a recusar, com mensagem.
- Auto, traço por várias cópias: linha primeiro (Trial; o Pixelorama vai por coluna, só a diagonal os separa). Tileset sem camada fica no sprite (o Trial apaga; recusado pela ordem do dono).
- ⛔ **Recusa medida:** editar no lugar um tile com todas as cópias no traço foi escrito e desfeito. A fixtura com um tile não separava as hipóteses; o M9b com três tiles mostrou que nenhum oráculo o faz.

## Medições

- M6 (256×256 células × 64 quadros, 1 píxel): 2 531 → **38 ms**, desfazer 33. 1024×1024 × 1: 801 → **10 ms**, desfazer 7. Reescrever todas as linhas de um tile nesse extremo: ~178 ms.
- Quadro do traço ao vivo num núcleo (`m6_quadro_ao_vivo`: decidir + aplicar + compor + repor): 96 px 0,0 ms · 512 px 1,4 ms · 1024 px × 8 quadros 5,7 ms · 4096 px × 64 quadros 168 ms (compor 96).
- ⚠️ Por medir: GPU; preview ao vivo no app real. Proposta 3 (só o que está à vista) não feita. Quadros em paralelo: recusada em §6.2.

## Provas

- Por nome, verdes em `31e98054d9` (o commit de fmt vem depois): `ph2d-host-pixel-scenes tiles::` 20 · `ph2d-panel-registry-init the_pixel_tiles` 6 · `ph2d-pixel-tiles` `batem` 3 (inclui `as_bordas_batem_com_o_m9_corrido`, `a_lista_e_a_copia_que_fica_batem_com_o_m9b`), `onde_os_oraculos_discordam_vale_o_trial`, `o_canvas_fora_do_multiplo...`, `o_tileset_existente...` · `the_shell_only_shrinks` · `architecture_workspace_file_loc_cap` · `no_magic_numeric` · `every_key`.
- Clippy verde nas crates tocadas. Mutantes: todos sangraram, nas três rondas.
- `ph2d-app-pixel`: 11 400 de 11 415 (folga **15**). Mover antes de crescer; nunca subir o número.
- ⚠️ **Não correu:** suíte completa e `nextest-impacted` (só na integração); crates inteiras.

## Abrir a P10c por aqui

Plano §8: «P10c — além do Aseprite: terreno 47 → iso/hex → Export Tileset → cena **22**, fotografada. Smoke do dono.» Somado ao «Ainda da P10c» da P10a e às linhas M3/M8 do plano:

1. **Tela do gabarito (terreno 47).** O motor existe (`ph2d-pixel-tiles::terreno`: `pinta`, `novo_terreno`; gate `o_terreno_bate_com_o_godot_*`). Falta a UI. Decisão do dono: carimbo à mão + gabarito de 47, **sem** descoberta por área pintada (`project-memory/project_tilemap_rule_discovery_not_intuitive_2026_11_06.md`). Vários terrenos por tileset, sem transição entre eles.
2. **Iso e hex.** M3 (centro e clique de volta) contra o Godot e contra `ph2d-grid`: ⚠️ a comparação com `ph2d-grid` está por fazer. Godot e Pixelorama divergem entre si nas convenções. O `.ase` não guarda a forma: sai como imagem, com aviso. Gate `iso_hex_batem_com_godot_e_ph2d_grid`.
3. **Export Tileset e mapa em PNG + JSON.** M8 (o Godot remonta o mapa) **por fazer**; gate `o_godot_remonta_o_mapa`. Mais `--split-grid` e `--trim-by-grid`.
4. **Aceitação congelada** (plano §7): sprite animado com camada tilemap nos 3 modos e em Draw Tiles, com flips e terreno 47 sobre um mapa real (≥ 200 células, buracos, corredores de 1 célula), que grava, reabre igual, vai a `.ase` que o Trial abre e a PNG + JSON que o Godot remonta.
5. **Cena 22** (`max_level` 21→22), fotografada antes do smoke. Janela nova para a onda.
6. Regras: UI em inglês, cartões compactos, gesto real conta (prova por ponteiro), testes por nome, folga de `app-pixel` = 15.

## Para a integração, anotar

- `input_teclas.rs`: risco de conflito com `828051653b` de `line/GameEngineFuncional` (Tab). Aqui o `Tab` sozinho ainda abre a timeline.
- A fixtura `oracle_tile_edges_m9.json` tem 512 KB.
- Recontar: `Cargo.lock`; crates novas `ph2d-panel-pixel-tiles` e `ph2d-pixel-ponte`; features em `shells/desktop/Cargo.toml`.
- `project-memory`: `MEMORY.md` com 18 080 bytes (alvo 17 KB), e os três `project_*_2026_11_06` novos estão por comitar na árvore primária.
- A entrada do §5 do CLAUDE.md fica para o fecho da linha (`PH2D_PIXEL_SMOKE=1..21`, hoje diz 1..20).

## Abertos (fora da P10c, salvo decisão)

- Os dois cliques depressa perdem-se noutros painéis do Pixel? Curado só em `ph2d-panel-pixel-tiles` (`DoubleClick` conta como `Click`); não conferido nos outros.
- A janela Export não tem prova por ponteiro de flutuar e arrastar.
- Não há como apagar um tileset inteiro; não há a opção de cortar as células de fora no Canvas Size (o Trial tem). Não entraram; reavaliar se o dono pedir.
- As quatro perguntas «Por confirmar» do handoff P10a estão **respondidas** pelo M9: não as repita ao dono.
- Não medido no M9: Pixelorama por ponteiro real nos casos M9b; desfazer em M9b; caso 1 invertido; o `.ase` do Trial não grava.

## Smoke da P10b (referência, aprovado)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && PH2D_PIXEL_SMOKE=21 ./target/smoke/ph2d-host-desktop`
2. Abre com relva, tijolo e setas e a janela TILEMAP. Em Manual, desenhar muda todas as cópias durante o traço; em Tiles, carimba, com Flip.
3. `Space+Tab` e `Space+1/2/3`; New Tilemap Layer reaproveita o tileset; `Canvas 180` diz porquê recusa; Canvas Size fora do múltiplo aceita.
4. Errado = recusa muda, cópias que só mudam no fim do traço, ou a janela perder o tileset ao trocar de camada.
