# Animated Sprite

**Estado: `em obra`.** Desenho aprovado pelo dono em 06/11: «OK às recomendações».

Estudos lidos: crítica «Animated Sprite», «Suas perguntas», «Lista consolidada» · estudo «AnimatedSprite», «12 ideias». Código conferido em 06/11/2026, sobre `19436b23ce`.

## Como está (conferido)

Fotos de 06/11 pelas cenas `PH2D_GAME_SMOKE=4` + `PH2D_GAME_SMOKE_FOTO=uso_animada|heroi_animado|animada` e `PH2D_ANIM_SMOKE=2`.

- **Censo medido** no painel de teste, com a lista de `campos_da_sprite()` mais os do Animation: **28 à vista, 54 no total**; Player com 6 regras, **55 e 81** (cada regra conta 4 controlos). A crítica dizia 24 e 54.
- Marcador por quadro **existe e dispara** (`ph2d-ecs/src/sprite_anim_markers.rs`; `populate_anim.rs:105-134`; `tick.rs:172,212`). Só está dentro de Advanced, como número + texto. A crítica não o achou.
- Sinais de On Finish, On Loop e marcador chegam a Signal Actions sem script, mesmo em objeto sem regras: `tick.rs:212-230` → `fase_fixed_step_clocks.rs:148` → `fase_signal_outbox.rs` → `ph2d-runtime/src/lib.rs:383` → `fase_tabela_de_accoes.rs:105`.
- Regras: nomes não cortam mais (duas fileiras por regra). When = Pose, Action, Life, Signal (`inspector_model_anim_card.rs:8-13`); `personagem.rs:149-200` decide. Nenhum destaque de qual regra ganhou.
- Who Picks: Me e Script diferem só em `autoplay` (`card.rs:72-75,223-237`). Não existe «com qual começa»: `start_autoplay_animations` (`tick.rs:242-253`) toca `animator.current`, e o ▶ da lista (`event_anim_card.rs:92-95`) o reescreve. **Logo, o ▶ muda em silêncio com que o jogo começa.**
- Trocar a imagem (`image_choice.rs:55-69,103-108`): com corte salvo a grade é refeita; sem corte a grade antiga fica; as animações ficam sempre; nenhum aviso. Lido, não corrido.
- Folha nova (`make_animated`, `image_use.rs:71-90`): uma animação «loop» com todas as células (o `heroi_azul` ficou com 32).
- Pivot: Look aparece em Animated; os 9 pontos medem a **união das células**, de propósito, para o pivô não saltar quando o quadro troca (`ph2d-sprite-edit/src/pivot.rs:51-84`). Sem pivô por quadro.
- Player com Flip escreve `flip_x` a cada quadro por cima de uma chave de Flip (`personagem.rs:188-194`).
- Add ▸ Animated Sprite: herói de fábrica, 6 tags, autoplay em idle (`object_add.rs:110-135`). Add ▸ Character: o mesmo + `CharacterAnimations`, `PlatformPlayer`.
- Editor (aba, `editor/`): lista sem reordenar; folha com clicar e arrastar, células repetidas e não contíguas; fila com reordenar, apagar, tempo por quadro; só Cols×Rows. Não tem: Split by rows, Size/Separation/Offset, marcador, ida ao Pixel/Painter. A tira não rola durante o arrasto (`mod.rs:146-158`). `propose_after_drop` (`editor/mod.rs:48`) não tem chamador.
- Leitores: Hold, Repeat delay, Repeat count, Speed, Frames/s, Cycle, This frame e os sinais estão vivos. `AnimFieldEdit::Autoplay` não tem gesto. Speed negativa em Once a partir do quadro 0 acaba logo, como na Godot.
- `.ase`: tags com `frame_ms`, direção e `per_frame_ms` (`shells/desktop/src/ase_import.rs:123-198`).
- A Timeline por `SpriteAnimation` só escreve `Transform` (`ph2d-timeline/src/sprite.rs:95`). Speed e a grade têm rótulo do catálogo (`sprite_animator.3`, `sprite_grid.1|2`). **Por medir (E0):** o que grava chave de verdade.

**Da crítica já não vale:** «sem marcador» · regras cortadas · Direction/overrides à vista · contagem 10/29 da versão B (a Sprite fechada tem 13/31 sozinha).

## O que as 9 ensinam

Godot: fatiar vendo a folha, tempo relativo por quadro; evitar o painel que some ao trocar de nó. Construct: Speed, Loop, Ping Pong ao lado dos quadros; evitar a janela que cobre a cena. GameMaker: marcador dentro da faixa. GDevelop: convite no vazio, pontos por quadro. Unreal: evitar a lista de «N members». Nenhuma das 9, nas fontes lidas, liga o estado do personagem à animação no painel: é a nossa vantagem, e a regra acesa no Play a reforça.

## Decisões do dono

**Vigentes:** um cartão Animation + editor próprio, quadros/s (25/10) · lista de regras «quando → toca» (25/10) · seletor Use (26/10) · folha fora do cartão em Animated (27/10) · Direction e overrides só com valor, velocidade negativa toca ao contrário (04/11) · Auto Key grava tudo (19/10) · chaves de Flip e Pivot (decisões 12 e 16 da [Sprite](01_sprite.md)).

**Desta revisão (06/11), todas como recomendado:**

1. **A folha fica no editor.** H/V Frames, From, To e This frame saem do cartão.
2. **Who Picks: Auto · Script · Rules.** «Me» vira «Auto», com o menu «Starts with» ao lado (revê em parte 25/10). O ▶ passa a ser só prévia: não muda com que o jogo começa.
3. **Add ▸ Animated Sprite nasce vazio**, com «Try with the sample hero». O Character continua com o herói.
4. **Marcador só no editor:** ◆ no quadro da tira; no cartão só o resumo «2 markers».
5. **Regra acesa no Play agora; ensaio com teclas depois.**
6. **Look nasce dobrado em Animated** (nova, trazida pelo levantamento).
7. Versão B + aviso de imagem trocada (da A) + botão **Split by rows** no editor.

## O desenho aprovado

**Vazio:** cartão Image = «Drop a sprite sheet or click to choose» + recentes + «Try with the sample hero». Sem Animation.

**Em uso:**

| Cartão | Conteúdo |
|---|---|
| Image | miniatura, nome, `1536 × 768 px · 8 × 4`, botão Painter; Use; recibo «Keep old size» e aviso de imagem trocada quando couber |
| Animation (N) | ícones no cabeçalho: Add · Remove · Edit Animations; prévia; lista (miniatura, nome, N frames, ▶; dois cliques renomeiam); Playback (▶, barra, Frame n / N); Frames/s e Speed na mesma linha; Cycle; **More** dobrado; **Who Picks** |
| More (dobrado) | resumo `Hold · signals`; Hold, Repeat delay, Repeat count, On Finish, On Loop; «N markers» (abre o editor); Direction e overrides só com valor |
| Who Picks | Auto + «Starts with» / Script / Rules; em Rules, as regras, + Rule, Run From, Flip; no Play a regra que ganhou fica acesa |
| Look, Transform, Ordering, Advanced | os da Sprite; Look dobrado com resumo |

**Editor:** a lista · a folha · a fila. Novos: **Split by rows** (uma animação por fileira ocupada) · ◆ marcador no quadro da fila, com o nome do sinal · a tira rola ao arrastar · Cols×Rows moram só aqui. This frame já está aqui.

**Aviso de imagem trocada** (Animated, imagem sem corte salvo): `New image, old cut: 8 × 4 makes cells of 32 × 32 px, and these 6 animations were made for another image.` com **Cut again** e **Keep the cut**.

## Aceitação

- **Poder:** com qual começa, à vista, e o ▶ não o muda · marcador pelo editor reagindo em regra e Signal Actions sem script · uma folha vira uma animação por fileira em um clique · ver qual regra ganhou durante o Play · nenhum poder removido: tudo o que sai do cartão continua no editor ou em More.
- **Facilidade:** de 28 à vista e 54 no total para **15 e 47** (conta no papel, confirmar no censo novo; a Sprite fechada responde por 13 e 31 com Look aberto). Sua folha com nomes tocando, de 7 gestos para 5.
- **Intuitividade:** o vazio diz o que fazer · nenhum número de célula no cartão · trocar a imagem explica o problema e dá as duas saídas · todo texto novo em inglês.

**Critério de parada:** regra acesa que custe mais de um campo novo no estado do jogo, ou que precise tocar a física, volta ao dono antes de prosseguir.

## ⛔ Recusas

| O quê | Por quê |
|---|---|
| Folha ou tira dentro do cartão (versão C) | decisão de 27/10, mantida em 06/11 |
| Ensaio com teclas nesta rodada | caro: roda o personagem sem a cena; decisão 5 |
| Aviso de regra morta | exige saber quais condições cobrem quais; fica com o ensaio |
| Colisão, âncora e pivô por quadro | fora deste item; Physics Body e Anchors |
| Fatiar por Size/Separation/Offset | a crítica não o pede; só se o dono pedir |

## Provas

A preencher. Censo novo `o_censo_dos_campos_do_animated_em_uso`; cena de smoke própria e o item em uso no jogo.

## Etapas

| # | O quê | Estado |
|---|---|---|
| E0 | Medir o que grava chave no Animated | a fazer |
| E1 | Cartão: cortes, cabeçalho de ícones, Frames/s e Speed numa linha, More, Look dobrado, nome em dois cliques, resumo de grade | a fazer |
| E2 | Who Picks: Auto, «Starts with»; ▶ só prévia | a fazer |
| E3 | Editor: Split by rows, ◆ marcador, tira rola | a fazer |
| E4 | Add vazio, «Try with the sample hero», aviso de imagem trocada | a fazer |
| E5 | Regra acesa no Play | a fazer |
| E6 | Censo, cena de smoke, fotos, item em uso no jogo | a fazer |
