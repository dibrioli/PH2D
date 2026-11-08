# 26 — Animated Sprite · várias folhas (plano, 08/11)

Pedido do dono, 08/11: um Animated Sprite com MAIS de uma sprite sheet (idle numa folha, run noutra). Decisões do
dono, 08/11 ([memória](../../project-memory/project_animated_sprite_many_sheets_owner_decisions_2026_11_08.md)):

1. **Lista de folhas:** o cartão Image vira **Sheets (N)**, uma linha por folha; **cada animação usa UMA folha**
   (etiqueta na linha dela); o editor ganha um seletor de folha. Mistura dentro de uma animação: não.
2. **Pivot por folha:** ao trocar de animação os pés ficam no mesmo lugar e o pixel tem o mesmo tamanho na tela.
   A folha nova copia o Pivot da primeira.
3. **Folha nova = o «+» do cartão cria um campo de imagem VAZIO** (o convite do Animated vazio). Soltar numa folha
   existente troca só essa folha, com o aviso de corte velho de hoje (interpretação; confirmar no smoke).
4. **Tirar uma folha leva as animações dela**, num passo de desfazer, com o número no botão.

Base: o Animated Sprite fechado ([ficha](componentes/02_animated_sprite.md)) e o editor da onda 1
([handoff](handoffs/HANDOFF_CONTINUACAO_2026-11-07_EDITOR_ANIMACOES_ONDA1.md)). **Não se reconstrói.**

**Maquete** (hoje à esquerda, foto `PH2D_GAME_SMOKE=5` `animado_uso`; proposta à direita): [26_hoje_e_proposta.png](maquetes/26_hoje_e_proposta.png) · fonte [26_sheets.svg](maquetes/26_sheets.svg).

## 1. Como guarda hoje (medido 08/11)

| Peça | Onde | Quantas por objeto |
|---|---|---|
| Textura | `Sprite.source` (`ph2d-render/src/sprite/component.rs:66`) | 1 |
| Tamanho na tela de UMA célula | `Sprite.size`, escrito por `image_use::show` (`:153-169`) e `ase_import.rs:300-306` | 1 |
| Pivot | `Sprite.anchor`; `pivot.rs:51-85` mede a união opaca das células da grelha | 1 |
| Corte | `SpriteGrid {hframes, vframes, frame}` (`ph2d-ecs/src/sprite_grid.rs:47`) | 1 |
| Animações | `SpriteAnimations` (tags `from..=to` sobre células) + `SpriteAnimFrames` (listas) | N, todas na mesma grelha |
| Célula → UV | `sprite_sheet_subrect` (`ph2d-render/src/sprite/subrect.rs:28`), chamada em `shells/desktop/src/render_loop/sim_extract_sheet.rs:142,172` e na miniatura `asset_card_portrait.rs:194` | — |
| Tique | `tick.rs:23` escreve só `SpriteGrid.frame` | — |

⇒ Uma folha por objeto, por construção: não há campo de folha numa tag.

## 2. Os oráculos (corridos, 08/11)

Entradas nossas: `idle.png` 128 × 32 (4 × 1 de 32 px) e `run.png` 288 × 96 (6 × 2 de 48 px). Ficheiros e saídas,
com cabeçalho, em [`oraculos/varias_folhas/`](oraculos/varias_folhas/).

| App | Corrido como | O que fez |
|---|---|---|
| **Godot 4.7.2** (MIT) | `godot --headless -s oraculo.gd` (`SpriteFrames` + `AtlasTexture`) | Cada QUADRO aponta para qualquer atlas e região; duas folhas convivem numa animação. Ao tocar `run` depois de `idle` o quad passa de 32 para 48 px com `centered=true`, `offset=(0,0)`: **o boneco cresce a partir do centro e os pés saltam 8 px**. |
| **LibreSprite 1.3** (GPL-2.0, só corrido) | `libresprite -b idle.png run.png --sheet … --sheet-pack` | Cada ficheiro é um sprite à parte; ao exportar junta tudo num atlas 512 × 256, sem tags. Várias folhas = vários documentos, e o motor de jogo é que volta a separá-las. |
| Unity | **não instalado** nesta máquina: não corrido | — |

Lição: o Godot dá o poder (folha por quadro) e cobra o salto; o dono escolheu a folha por ANIMAÇÃO e o Pivot por
folha, o que tira o salto e mantém o editor simples.

## 3. O desenho técnico (decisão delegada, padrão-ouro)

### 3.1 «A folha em cena e as guardadas»

`Sprite` e `SpriteGrid` continuam a ser a folha **em cena**: o render, o extract, o Pivot, o Edit in Pixel/Painter
e o tique já leem dali e **não mudam**. As outras folhas ficam **guardadas** num componente novo, ao lado do `Sprite`
(é preciso `SpriteSource`, que mora em `ph2d-render`):

```rust
// ph2d-render/src/sprite/sheets.rs
pub struct SheetParked { source: SpriteSource, size: [f32; 2], anchor: [f32; 2], hframes: u32, vframes: u32 }
pub enum SheetSlot { OnStage, Parked(SheetParked), Empty }
pub struct SpriteSheets(pub SmallVec<[SheetSlot; 2]>);   // exatamente um OnStage
```

e a tag ganha a folha dela, no fim: `AnimationTag.sheet: u8` (`ph2d-ecs`). **`PROJECT_SCHEMA` 219 → 220**, sem
leitor de formato velho (CLAUDE §0.11).

**Por que trocar de lugar, e não copiar:** cada folha existe num sítio só. Editar o Pivot ou o corte da folha em cena
é editar essa folha, pelos escritores de hoje; ao trocar, os valores em cena vão para a gaveta e os da outra saem.
Uma cópia derivada teria duas verdades e precisaria de um escritor de volta em cada um dos ~10 sítios que mexem em
`Sprite.anchor`/`source` (mapa do explorador, §5). Um objeto sem `SpriteSheets` = uma folha, como hoje.

**A porta única:** `entra_na_folha(i)` troca `OnStage` ↔ `Parked(i)` e põe `frame` na célula de entrada. Chamam-na:
o tique quando a tag de `current` é de outra folha, `vai_para_o_inicio`, o ▶ e a linha do cartão, o seletor do
editor, e o Play/Stop (o documento volta pelo diff, como hoje). Ela escreve só quando muda, então o desfazer por diff
vê um passo.

**Limite:** `SHEETS_MAX = ANIM_TAGS_MAX` (64). Não há recurso que limite abaixo disto: cada folha preenchida nasce com
uma animação, então mais folhas do que tags não se alcança. O `u8` cobre-o.

### 3.2 Pivot por folha, os pés no mesmo sítio

- O tamanho de cada folha na tela usa o **mesmo metro por pixel** da primeira: `size = célula_px × m_por_px`. Um run
  de 48 px fica 1,5× maior que um idle de 32 px, como a arte.
- O Pivot é guardado por folha (o `anchor` de cada uma). Ao preencher uma folha nova, o Pivot dela é a **mesma fração**
  da união opaca da folha nova que a primeira usa (`pivot.rs::fracao_opaca`). Pés em baixo ao centro ⇒ pés em baixo
  ao centro.
- Prova pedida: trocar `idle` → `run` não mexe o ponto do Pivot no mundo (o contrário do Godot, `godot_saida.txt`).

### 3.3 Cartão Sheets (Inspector, Animated)

- Título **Sheets (N)** e um **+** no cabeçalho. O + acrescenta um slot `Empty`, que pinta o convite de hoje
  («Drop a sprite sheet or click to choose») dentro do cartão, por baixo das folhas.
- Uma linha por folha: miniatura, nome do ficheiro, `6 × 2`, e o botão Painter de hoje; a folha em cena fica
  destacada. Soltar ou escolher numa linha troca essa folha, com o aviso de corte velho de hoje.
- Tirar: no hover da linha, **−** com dica `Remove run.png and 1 animation`. Um passo de desfazer. A última folha não
  tem −: tirar tudo é o convite do vazio.
- Com UMA folha o cartão fica como hoje, sem lista nem título novo (`Image`): **zero campos novos no caso comum**. O
  censo `o_censo_dos_campos_do_animated_em_uso` (17 / 48) não pode subir.
- Animation (N): cada linha ganha a etiqueta da folha (nome curto) só quando há 2 ou mais.
- Preencher uma folha vazia cria uma animação com todas as células, com o **nome do ficheiro** sem extensão
  (`run`), seguindo `next_free_name` em colisão, como `make_animated` faz hoje.

### 3.4 Editor de animações

- Seletor de folha no título da folha (`idle.png ▾`), só com 2 ou mais. Trocar põe essa folha em cena (prévia) e
  filtra a lista à esquerda? **Não:** a lista mostra todas, com a etiqueta; clicar numa animação de outra folha troca
  a folha. O seletor serve para criar animações (+ Add) na folha escolhida e para o Split by rows dessa folha.
- Cols/Rows editam a folha em cena. As faixas (`cell_users`) contam só as tags da folha em cena.

## 4. Os casos da mesma família (§0.10, um bloco)

Tudo o que hoje assume «uma textura por objeto» entra junto:

| # | Caso | Cura |
|---|---|---|
| F1 | Tique / início / ▶ / linha do cartão | `entra_na_folha` |
| F2 | Quem lista as imagens em uso (agrupar usos `image_use.rs:674`, «used by» na biblioteca, apagar asset) | contar também as `Parked` |
| F3 | Edit in Pixel/Painter devolve a imagem (`ph2d-sprite-edit/src/imagem.rs:82` `aponta`) e `vazio.rs` | reescrever também a `source` guardada que casa |
| F4 | Abrir o projeto: quais texturas re-inserir no `AssetDb` | incluir as guardadas |
| F5 | Duplicar, instância, prefab | componente registado viaja; prova por nome |
| F6 | Miniatura na Hierarchy / Assets | a da folha em cena, como hoje |
| F7 | Character (herói de fábrica), `.ase` | continua uma folha; nada muda |
| F8 | Stop do Play repõe a folha do documento | prova: `igual ao do Play: true` com duas folhas |
| F9 | Split by rows, faixas, marcador | só sobre a folha em cena |
| F10 | Remover tag / renomear | `sheet` viaja na tag; tirar folha tira as tags dela e reindexa as outras |

Antes de codar F2–F4: um explorador lista TODOS os leitores de `Sprite.source` fora de `ph2d-render` (o mapa de 08/11
não fechou isto).

## 5. Etapas

| # | O quê |
|---|---|
| V0 | Inventário completo dos leitores de `Sprite.source` (F2–F4); fixar a lista no §4 |
| V1 | Tipos: `SpriteSheets`, `AnimationTag.sheet`, registo, schema 220; `entra_na_folha` + F1, F10 |
| V2 | Pivot por folha e metro por pixel (§3.2) |
| V3 | Cartão Sheets + etiqueta na Animation + tirar (§3.3) |
| V4 | Editor: seletor, + Add e Split na folha escolhida (§3.4) |
| V5 | F2–F8 |
| V6 | Cena de smoke (`PH2D_GAME_SMOKE=5`, novo objeto «Two Sheets» com idle + run), fotos ao lado da maquete |

## 6. Provas (por nome, cada uma com mutação que sangra)

- `trocar_de_animacao_traz_a_folha_dela` · `a_folha_guardada_nao_se_perde_no_vai_e_volta` (bytes iguais)
- `os_pes_ficam_no_sitio_ao_trocar_de_folha` (o contrário do Godot, com a fixture) · `o_pixel_tem_o_mesmo_tamanho_nas_duas`
- `a_folha_nova_copia_o_pivot_da_primeira`
- `o_mais_cria_um_campo_vazio` · `preencher_o_vazio_cria_a_animacao_com_o_nome_do_ficheiro`
- `tirar_a_folha_leva_as_animacoes_e_um_ctrl_z_devolve_tudo` · `a_ultima_folha_nao_se_tira`
- `com_uma_folha_o_cartao_e_o_de_hoje` (censo 17 / 48 igual)
- `a_imagem_guardada_conta_como_em_uso` · `o_painter_devolve_a_imagem_a_folha_guardada` · `abrir_o_projeto_traz_as_duas_texturas`
- `duplicar_leva_as_duas_folhas` · `o_stop_repoe_a_folha_do_documento`
- Mutações mínimas: `entra_na_folha` não troca · troca sem guardar · fração do pivot ignorada · m/px da folha nova ·
  tirar sem tirar as tags · reindexar errado · em uso ignora as guardadas · Painter só na folha em cena.

## ⛔ Recusas

| O quê | Por quê |
|---|---|
| Quadros de folhas diferentes numa animação (Godot) | dono 08/11, pergunta 1 |
| Centrar cada quadro (Godot) | o boneco salta 8 px (medido); dono 08/11, pergunta 2 |
| Encolher a folha maior ao tamanho da primeira | perde nitidez; dono 08/11 |
| Juntar as folhas num atlas ao importar (LibreSprite) | perde o arquivo de cada folha, e trocar uma só fica impossível |
| Cópia derivada das folhas com escritor de volta | duas verdades; §3.1 |
