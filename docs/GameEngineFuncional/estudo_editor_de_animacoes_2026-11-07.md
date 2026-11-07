# Estudo: reconstruir o editor de animações (07/11/2026)

Pedido do dono: reconstruir e melhorar o editor, partindo do layout «5 · O editor próprio» da Crítica. Este é um estudo:
onda 1 feita em 07/11, ver [handoff](handoffs/HANDOFF_CONTINUACAO_2026-11-07_EDITOR_ANIMACOES_ONDA1.md). O dono decidiu em 07/11 (§5); a §4 é o plano aprovado.

## 1. De onde vem o layout

Gerador: `/home/enio/Documentos/GameEngineSamples/fonte_critica/onda1_sprite_animado.py` (maquete em `:91–95`, coluna
`:102`, recomendação da versão B `:198`), montado por `fonte/pagina.py`. É uma maquete em HTML, sem medição do app.

Não encontrado em nenhum arquivo: a legenda «3 campos à vista · 0 em cartões fechados». Não a use como número.

## 2. O que existe hoje (lido no código)

Aba de documento (`AnimDocs`, `DocumentSurface`), crate `ph2d-app-sprite-anim/src/editor/`, ~2 150 linhas: `mod.rs`,
`doc.rs`, `paint.rs`, `layout.rs`, `marks.rs`, `split.rs`.

| Peça do layout | Hoje |
|---|---|
| Lista, + Add, Split by rows | existe |
| Folha com arrasto | existe (`doc.rs:426–561`) |
| Tira e tempo por quadro | existe; `AnimationTag.per_frame_ms` |
| Frames/s | existe; `AnimationTag.frame_ms` |
| Repeat · Once · Ping-pong | modelo tem (`direction`, `repeat`); falta o segmentado |
| Cor por animação | **não existe** no modelo (`cards.rs:64 cor_de` não foi conferido) |
| Barra colorida sob a célula | não existe |
| ◆ nos campos | **não existe e contraria o dono**, ver §3 |

Conclusão: o modelo de dados serve quase inteiro; o que se reconstrói é a tela (`layout.rs`, `paint.rs`) e o gesto
(`doc.rs`). Mantém-se a aba, o undo do projeto (`undo_is_the_projects`) e o import `.ase`.

Não conferido: se o editor atual tem prévia tocando; se a tira rola no arrasto (a ficha diz as duas coisas, `mod.rs:146–158`);
se `per_frame_ms` é por USO ou por CÉLULA (decide o item 4 da §3).

Consumidores a varrer antes de mudar o modelo: `ph2d-render` (`atlas/hero_sheet.rs`), `ph2d-app-game`, `ph2d-timeline`,
`ph2d-app-macros`, shell (`janela_do_jogo_animado.rs`, `ase_import.rs`, `asset_drop_apply.rs`). Smokes `PH2D_ANIM_SMOKE`,
`PH2D_GAME_SMOKE`.

### Conferido na onda 1 (07/11, sobre `7f56e2ae30`)

- **Prévia no editor:** nunca houve imagem própria. Havia um botão ▶ que escrevia `Playing` na sprite; saiu, a maquete não o tem e o ▶ do cartão faz o mesmo.
- **Tira no arrasto:** rola com a mão na borda ao arrastar um quadro DELA (`scroll_at_edge`). Ao arrastar pela FOLHA não rola: `frame_drag()` é `None` em `Drag::Cells`. Fica para a onda 2, a do arrasto.
- **`per_frame_ms`:** é por USO, indexado pelo passo dentro da animação (`set_own_ms`, `frame_ticks_at`). Logo o item 4 da §3 já vale. ⚠️ Defeito achado: `AnimFieldEdit::Frames` (`card.rs:167`) só troca a lista; reordenar ou apagar na tira não leva o tempo próprio junto, embora os marcadores andem. Onda 2.
- **`cards.rs:64 cor_de`:** é a cor das camadas do Vector; nada a ver com animação.
- **Repeat · Once · Ping-pong:** já existia como três botões; a tabela acima dizia que faltava.
- **A prévia do painel acompanha.** Sonda temporária, cena `PH2D_GAME_SMOKE=5`, cliques por `AnimDoc::press` com a aba à frente, amostra a cada 5 quadros de `build_anim_info`, foto em rajada. Clicar noutra linha troca na hora (células 0..=5 → 8..=15). Ritmo em passos a cada 500 ms: 100 ms → 5 · 2 fps → 1 · 24 fps → 12. A foto confirma o painel ao lado.
- **Limites para o dono:** selecionar outro objeto com o editor aberto põe esse objeto no painel, e o editor segue no anterior. Digitar `24` lê `23.8`: `frame_ms` é inteiro (42 ms).

## 3. O que o layout da Crítica não resolve

1. **Não tem prévia no editor.** Unreal tem visor com controles, Aseprite tem janela própria. **Recusado pelo dono em 07/11:** a prévia já toca no painel ao lado.
2. **Os ◆ em Frames/s e This frame.** O dono decidiu em 07/11 que Frames/s, Cycle e Rules não gravam chave; tirou os ◆ dos dois campos. A maquete é anterior.
3. **Ordem do arrasto indefinida.** O Godot marca cada célula uma vez por gesto, na ordem do percurso, e numera.
4. **Mesmo quadro em duas animações:** uma barra de cor só não mostra. E o multiplicador deve ser do USO, não da célula.
5. **Sem regra** para células vazias no Split by rows, nem para onde o Once termina (o modelo tem `signal_on_finish`).
6. **Grade:** o layout não mostra tamanho, margem e espaçamento; o editor atual só tem Cols e Rows.
7. **Tira:** falta seleção múltipla, duplicar, inverter.

## 4. Proposta

**Tela:** como na maquete: lista à esquerda, folha em cima, tira e campos embaixo, sem os ◆ e sem prévia própria. A prévia é a do
painel ao lado; na onda 1, confira com foto e medição que ela acompanha a animação selecionada e a mudança de Frames/s enquanto o
editor está aberto, e reporte ao dono se não acompanhar.

**Leis do gesto**
- Arrasto: ordem do percurso, com número na célula durante o gesto; da direita para a esquerda dá a sequência invertida.
- Voltar sobre uma célula no mesmo gesto não faz nada (evita tremor da mão). Repetir = novo arrasto ou duplicar na tira.
- Remover só na tira, nunca na folha: o clique tem um sentido só.
- Folha: destaca a animação atual e esmaece as outras; até 3 faixas empilhadas e depois «+N»; lista completa ao passar o mouse.

**Ondas** (cada uma com smoke do dono)
1. Tela nova, cor por animação, barras na folha (com as faixas empilhadas), segmentado de repetição.
2. Arrasto numerado, tira com seleção múltipla, duplicar, inverter, duração em vários quadros.
3. Grade com margem e espaçamento, regra de células vazias, aviso antes de refatiar, passo a passo por tecla.

Depois, se o dono pedir: «Repeat to» (intro + loop), onion skin, grade irregular, prévia lado a lado (não achei em nenhum app).

**Decisões técnicas, delegadas:** a cor é guardada na animação como índice de paleta de tokens, para não mudar ao reordenar.
Sem compatibilidade com projetos antigos (CLAUDE §0.11).

**Oráculo (§0.9):** Godot 4.7.2 (MIT) está instalado. Corra-o por script sobre uma folha nossa e grave a sequência tocada em
ping-pong e once como fixture; é a régua das pontas. Aseprite não está instalado. LibreSprite: triagem de licença antes de usar.

## 5. Decisões do dono (07/11)

1. Prévia dentro do editor: **não**, «a prévia já toca no painel ao lado».
2. ◆ em Frames/s e This frame: **saem**.
3. Quadro em duas animações mostra faixas empilhadas: **sim**.
4. Primeira onda como na §4: **sim**.

## 6. Pesquisa: fontes e limites

| Recurso | Godot | Unreal | GameMaker | Aseprite | Construct | Defold |
|---|---|---|---|---|---|---|
| Escolher células na folha | sim | indireto | ? | ? | ? | por número |
| Duração por quadro | multiplicador | em quadros | arrasto | tempo | multiplicador | não (fórum, 2022) |
| Ping-pong no jogo | sim | ? | só prévia | sim | sim | sim |
| Passo a passo | não | sim | ? | sim | ? | ? |
| Onion skin | ? | ? | sim | sim | pago | ? |

`?` = não confirmado. Unity ficou fora da tabela por ter quase tudo não confirmado neste recorte, salvo fatiamento e pivô.

O que evitar, com a razão: prévia diferente do jogo (GameMaker: loop e ping-pong do painel valem só na prévia); ordem alfabética
implícita (Unreal); duplicar quadro para segurar tempo (Defold); ping-pong sem regra clara nas pontas (Godot e Defold divergem).

Fontes: [Godot SpriteFrames](https://docs.godotengine.org/en/stable/classes/class_spriteframes.html) ·
[Godot, código do editor](https://raw.githubusercontent.com/godotengine/godot/master/editor/scene/sprite_frames_editor_plugin.cpp) ·
[Godot, passo a passo pedido](https://github.com/godotengine/godot-proposals/issues/14959) ·
[Unity Sprite Editor](https://docs.unity.com/en-us/engine/current/manual/unity2d/sprite/sprite-editor-window-reference/reference) ·
[Unreal Flipbook 4.27](https://dev.epicgames.com/documentation/en-us/unreal-engine/flipbook-editor-reference?application_version=4.27) ·
[GameMaker Sprites](https://manual.gamemaker.io/monthly/en/The_Asset_Editors/Sprites.htm) ·
[Aseprite tags](https://www.aseprite.org/docs/tags/) · [Aseprite prévia](https://www.aseprite.org/docs/preview-window/) ·
[Construct](https://www.construct.net/en/make-games/manuals/construct-3/interface/animations-editor) ·
[Defold flipbook](https://defold.com/manuals/flipbook-animation/).

Limites: Construct só por trechos de busca (403); Unreal na versão 4.27; Reddit bloqueado, amostra de opinião pequena; Spine não confirmado.
