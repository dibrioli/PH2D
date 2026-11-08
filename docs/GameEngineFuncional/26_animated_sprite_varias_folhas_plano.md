# 26 — Animated Sprite · várias folhas (plano, 08/11)

Pedido do dono, 08/11: um Animated Sprite com MAIS de uma sprite sheet. Decisões do dono, 08/11, **revistas no mesmo
dia** sobre a captura do cartão Image ([memória](../../project-memory/project_animated_sprite_many_sheets_owner_decisions_2026_11_08.md)):

1. **O cartão Image fica como é.** Em Animated, a miniatura vira um **dropdown das folhas carregadas**: cada item é a
   moldura com a imagem inteira. A folha escolhida ali é a **folha visível** do painel (nome, `W × H px · C × R`,
   Painter). Surgem dois botões: **Load sheet** (carrega outra folha) e **Delete sheet** (apaga a visível).
2. **Todas as folhas carregadas ficam ativas**, seja qual for a visível no painel. As animações usam-nas todas.
3. **Uma animação pode misturar quadros de folhas diferentes** (3 do `idle.png` + 2 do `run.png`). No editor de
   animações escolhe-se a folha e clica-se quadros dela; trocando a folha, continua-se a mesma animação.
4. **Uma lista só**, na ordem que o artista quiser: as animações aparecem à esquerda no editor e no cartão Animation.
5. **Pivot por folha:** ao trocar de quadro entre folhas os pés ficam no mesmo lugar e o pixel tem o mesmo tamanho na
   tela; a folha nova copia o Pivot da primeira. (Mantido da 1.ª rodada.)
6. **Apagar uma folha leva o que é dela** (mantido da 1.ª rodada), num passo de desfazer, com aviso. Com a mistura isto
   quer dizer: saem os QUADROS dessa folha de cada animação, e a animação que fica sem quadros sai. O aviso diz os
   dois números (`Delete run.png: 5 frames in 2 animations; 1 animation becomes empty`). A confirmar no smoke.

Base: o Animated Sprite fechado ([ficha](componentes/02_animated_sprite.md)) e o editor da onda 1
([handoff](handoffs/HANDOFF_CONTINUACAO_2026-11-07_EDITOR_ANIMACOES_ONDA1.md)). **Não se reconstrói.**

**Maquete** (hoje à esquerda, foto `PH2D_GAME_SMOKE=5` `animado_uso`; proposta à direita):
[26_hoje_e_proposta.png](maquetes/26_hoje_e_proposta.png) · fonte [26_sheets.svg](maquetes/26_sheets.svg).

## 1. Como guarda hoje (medido 08/11)

| Peça | Onde | Quantas por objeto |
|---|---|---|
| Textura | `Sprite.source` (`ph2d-render/src/sprite/component.rs:66`) | 1 |
| Tamanho na tela de UMA célula | `Sprite.size`, escrito por `image_use::show` (`:153-169`) e `ase_import.rs:300-306` | 1 |
| Pivot | `Sprite.anchor`; `pivot.rs:51-85` mede a união opaca das células da grelha | 1 |
| Corte | `SpriteGrid {hframes, vframes, frame}` (`ph2d-ecs/src/sprite_grid.rs:47`) | 1 |
| Animações | `SpriteAnimations` (tags `from..=to` sobre células) + `SpriteAnimFrames` (listas de `u32`) | N, todas na mesma grelha |
| Célula → UV | `sprite_sheet_subrect` (`ph2d-render/src/sprite/subrect.rs:28`), chamada em `shells/desktop/src/render_loop/sim_extract_sheet.rs:142,172` e na miniatura `asset_card_portrait.rs:194` | — |
| Tique | `tick.rs:23` escreve só `SpriteGrid.frame` | — |

⇒ Uma folha por objeto, por construção: nem a tag nem o quadro dizem de que folha são.

## 2. Os oráculos (corridos, 08/11)

Entradas nossas: `idle.png` 128 × 32 (4 × 1 de 32 px) e `run.png` 288 × 96 (6 × 2 de 48 px). Ficheiros e saídas,
com cabeçalho, em [`oraculos/varias_folhas/`](oraculos/varias_folhas/).

| App | Corrido como | O que fez |
|---|---|---|
| **Godot 4.7.2** (MIT) | `godot --headless -s oraculo.gd` (`SpriteFrames` + `AtlasTexture`) | Cada QUADRO aponta para qualquer atlas e região; o quadro 12 do `run` vem do `idle.png` com duração 2. Ao tocar `run` depois de `idle` o quad passa de 32 para 48 px com `centered=true`, `offset=(0,0)`: **o boneco cresce a partir do centro e os pés saltam 8 px**. |
| **LibreSprite 1.3** (GPL-2.0, só corrido) | `libresprite -b idle.png run.png --sheet … --sheet-pack` | Cada ficheiro é um sprite à parte; ao exportar junta tudo num atlas 512 × 256, sem tags. |
| Unity | **não instalado** nesta máquina: não corrido | — |

Lição: o dono quer o poder do Godot (quadro de qualquer folha) **sem** o salto: Pivot por folha.

## 3. O desenho técnico (decisão delegada, padrão-ouro)

### 3.1 Cada folha mora num sítio só

- **Folha 0** = `Sprite` + `SpriteGrid`, como hoje. Todos os escritores e leitores atuais (render, Pivot, Painter,
  `.ase`, Character, uso da imagem) continuam certos para ela, sem mudança.
- **Folhas 1..N** = componente novo `SpriteSheets` ao lado do `Sprite` (precisa de `SpriteSource`, que mora em
  `ph2d-render`):

  ```rust
  // ph2d-render/src/sprite/sheets.rs
  pub struct ExtraSheet { source: SpriteSource, size: [f32; 2], anchor: [f32; 2], hframes: u32, vframes: u32 }
  pub struct SpriteSheets(pub SmallVec<[ExtraSheet; 2]>);   // índice k+1 no quadro
  ```

- **O quadro diz a folha:** `SpriteAnimFrames` passa de `Vec<u32>` a `Vec<FrameRef { sheet: u8, cell: u32 }>`, e
  `SpriteGrid` ganha `sheet: u8` ao lado de `frame` (o que está na tela). Uma tag de intervalo `from..=to` ganha
  `sheet: u8` (todo o intervalo numa folha; a mistura é sempre uma lista). **`PROJECT_SCHEMA` 219 → 220**, sem leitor
  de formato velho (CLAUDE §0.11).
- **O render** resolve `(sheet, frame)`: `sheet == 0` é o caminho de hoje; `sheet = k` lê `source`, `size`, `anchor`
  e grelha de `SpriteSheets[k-1]`. Um ponto só: `sim_extract_sheet.rs::cell`, mais a miniatura e a prévia.
- **A folha visível do painel** é estado de UI (como a aba aberta), não do documento: não entra no save nem no undo.

**Por que não «trocar de lugar» ao tocar** (a 1.ª versão deste plano): com quadros misturados a troca seria a cada
quadro, um escritor de 5 campos a correr no relógio do jogo e diffs de undo no Play. Por que não uma lista uniforme
com `Sprite` derivado: duas verdades, e cada um dos ~10 escritores de `Sprite.anchor`/`source` precisaria de escrever
de volta.

**Limite:** `SHEETS_MAX = 255` por objeto, o `u8` do `FrameRef`. Não há recurso que limite abaixo: cada folha extra
são 40 bytes no componente, e as texturas já são assets partilhados; 255 folhas de um personagem não se alcança à mão.

### 3.2 Pivot por folha, os pés no mesmo sítio

- Tamanho na tela com o **mesmo metro por pixel** da folha 0: `size = célula_px × m_por_px`. Um quadro de 48 px fica
  1,5× maior que um de 32, como a arte.
- O Pivot é de cada folha (`anchor`). Ao carregar uma folha, o Pivot dela é a **mesma fração** da união opaca da nova
  que a folha 0 usa (`pivot.rs::fracao_opaca`).
- Editar o Pivot (Look ou ferramenta no canvas) mexe na folha **visível no painel** — o dropdown diz qual.

### 3.3 Cartão Image (Animated)

- Uma folha: **igual a hoje**, mais o botão **Load sheet**. Zero campos novos além dele; o censo
  `o_censo_dos_campos_do_animated_em_uso` (17 / 48) sobe no máximo 1.
- Duas ou mais: a miniatura abre um **dropdown**: um item por folha, a moldura com a imagem inteira e o nome; a
  escolhida fica marcada. Ao lado, o nome e `W × H px · C × R` da visível, e Painter dela. **Delete sheet** aparece só
  com 2 ou mais (apagar a última é o convite do vazio, que já existe).
- **Load sheet** abre o mesmo painel de Assets de hoje («Choose the image for …»). A folha nova vira a visível. Não
  cria animação sozinha: o artista monta-as no editor (decisão 3). ⚠️ A confirmar no smoke: hoje a 1.ª folha cria a
  animação «loop» com todas as células.
- Soltar uma imagem sobre a miniatura troca a folha visível, com o aviso de corte velho de hoje.

### 3.4 Editor de animações

- O título da folha vira o mesmo **dropdown** das folhas (só com 2 ou mais). A grade mostra a folha escolhida.
- Clicar quadros acrescenta `FrameRef(folha escolhida, célula)` à animação aberta; trocar a folha e continuar clicando
  mistura. A tira mostra cada quadro com a sua imagem, e uma pequena marca da folha quando a animação mistura.
- A lista à esquerda é uma só; reordenar já existe por arrasto no cartão.
- Cols/Rows editam a folha escolhida. Faixas (`cell_users`), números e Split by rows agem sobre ela.

## 4. Os casos da mesma família (§0.10, um bloco)

| # | Caso | Cura |
|---|---|---|
| F1 | Tique, início, ▶, linha do cartão, `vai_para_o_inicio` | escrevem `(sheet, frame)` juntos |
| F2 | Quem lista as imagens em uso (agrupar usos `image_use.rs:674`, «used by», apagar asset) | contar também `SpriteSheets` |
| F3 | Edit in Pixel/Painter devolve a imagem (`ph2d-sprite-edit/src/imagem.rs:82` `aponta`), `vazio.rs` | reescrever a folha que casa, seja 0 ou k |
| F4 | Abrir o projeto: texturas a re-inserir no `AssetDb` | incluir as de `SpriteSheets` |
| F5 | Duplicar, instância, prefab | componente registado viaja; prova por nome |
| F6 | Miniatura na Hierarchy / Assets / prévia do cartão | quadro atual resolvido pela folha dele |
| F7 | Character (herói), `.ase` | continuam folha 0; nada muda |
| F8 | Stop do Play repõe | prova: `igual ao do Play: true` com duas folhas |
| F9 | Split by rows, faixas, marcador, `per_frame_ms` | sobre `FrameRef`; o marcador é por passo, não muda |
| F10 | Apagar folha | tira os `FrameRef` dela, reindexa `sheet > k`, tags vazias saem; um passo de undo |
| F11 | Trocar a imagem de uma folha (aviso de corte velho) | o aviso passa a ser por folha |

Antes de codar F2–F4: um explorador lista TODOS os leitores de `Sprite.source` fora de `ph2d-render` (o mapa de 08/11
não fechou isto).

## 5. Etapas

| # | O quê |
|---|---|
| V0 | Inventário completo dos leitores de `Sprite.source` e de `SpriteAnimFrames` (F2–F4, F9) |
| V1 | Tipos: `SpriteSheets`, `FrameRef`, `SpriteGrid.sheet`, `AnimationTag.sheet`; registo; schema 220; tique e render (F1, F6) |
| V2 | Pivot por folha, metro por pixel (§3.2) |
| V3 | Cartão Image: dropdown, Load sheet, Delete sheet (§3.3, F10, F11) |
| V4 | Editor: dropdown da folha, mistura na tira (§3.4) |
| V5 | F2–F5, F8 |
| V6 | Cena de smoke (`PH2D_GAME_SMOKE=5`, objeto «Two Sheets»: idle + run e uma animação misturada), fotos ao lado da maquete |

## 6. Provas (por nome, cada uma com mutação que sangra)

- `um_quadro_de_outra_folha_pinta_a_imagem_dela` · `uma_animacao_mistura_duas_folhas_na_ordem_da_tira`
- `todas_as_folhas_tocam_seja_qual_for_a_visivel`
- `os_pes_ficam_no_sitio_ao_trocar_de_folha` (o contrário do Godot, com a fixture) · `o_pixel_tem_o_mesmo_tamanho_nas_duas`
- `a_folha_nova_copia_o_pivot_da_primeira` · `o_pivot_mexe_na_folha_visivel`
- `load_sheet_acrescenta_e_fica_visivel` · `com_uma_folha_o_cartao_e_o_de_hoje_mais_load_sheet` (censo ≤ +1)
- `delete_sheet_tira_os_quadros_dela_e_um_ctrl_z_devolve_tudo` · `a_animacao_que_fica_vazia_sai`
- `a_imagem_de_outra_folha_conta_como_em_uso` · `o_painter_devolve_a_imagem_a_folha_certa` · `abrir_o_projeto_traz_as_texturas_todas`
- `duplicar_leva_as_folhas` · `o_stop_repoe_folha_e_quadro`
- Mutações mínimas: render ignora `sheet` · tique não escreve `sheet` · índice k sem o −1 · m/px da folha nova ·
  fração do Pivot ignorada · Delete sem reindexar · em uso ignora as extras · Painter só na folha 0.

## ⛔ Recusas

| O quê | Por quê |
|---|---|
| Lista «Sheets (N)» com uma linha por folha (1.ª maquete) | dono 08/11: o cartão Image fica, a miniatura vira dropdown |
| Uma folha por animação | dono 08/11: mistura permitida |
| Centrar cada quadro (Godot) | o boneco salta 8 px (medido); dono 08/11 |
| Encolher a folha maior ao tamanho da primeira | perde nitidez; dono 08/11 |
| Juntar as folhas num atlas ao importar (LibreSprite) | perde o ficheiro de cada folha; trocar uma só fica impossível |
| Pôr em cena por troca de lugar | com mistura, troca a cada quadro; §3.1 |
| Lista uniforme com `Sprite` derivado | duas verdades; §3.1 |
