# Character (Platform Player)

**Estado: `fechado`** (smoke do dono OK em 10/11, depois de três curas de UI pedidas no smoke). Dono 10/11: «faça conforme vc sugere» — **versão B**, **Walls com uma caixa só**, os dois «Vale rever?» ficam como estão (Make Platform Player em todo Dynamic; Character pronto). Estudos lidos: crítica `critica_ph2d.html#character`
(§1–§7, os dois «Vale rever?») e «Lista consolidada» (itens 6, «Maiores acréscimos», «Duplicações»).
Seções «Como está» e «Perguntas» abaixo são o levantamento ANTES da obra (base `bc4a6ce930`); o que ficou está em «O desenho aprovado». Fotos: [`fotos_03/`](fotos_03/).

## Como está (conferido no código)

- **Painel:** `ph2d-panel-inspector/src/sections/player.rs` (574 LOC) + `player_rows.rs` (578) +
  `populate_player.rs` (290) + `player_live.rs` (103). A fonte dos dados é `ph2d-app-physics/src/inspector/player.rs`.
  A struct gravada é `PlatformPlayer` (`ph2d-physics-ecs/src/components/player.rs:58`, postcard posicional).
- **12 cartões, todos abertos, 52 números, sem condição para aparecer** (`player_rows.rs:505`). O comentário em
  `:486` diz «onze». A ordem e a contagem conferem com a crítica: Leg 6 · Walk 5 · Jump 9 · Forgiveness 6 ·
  React 3 · Walls 8 · Dash 3 · Crouch 2 · Swim 3 · Ledge 5 · Glide 1 · Fall 1.
- **Fora dos cartões** (`sections/player.rs`):
  - Mode Dynamic·Kinematic·Pure (`:127-145`);
  - Live (`:149`);
  - Emit Signals Off|On (`:150-167`);
  - Lift policy (`:186-204`);
  - Walk off (`:208-225`);
  - Crouch walk off (`:228-247`, só com o agachar armado);
  - os botões Fit, Fit crouch, Clear run e Remove (`:264-429`).
  - Exceções de visibilidade, que já existem: o cartão React some sem `reaction_is_live` (`:522-524`), e a mola e o Push somem sem `*_is_live` (`:486-489`).
- **O primeiro número que se lê** é Float Height (Leg, `player_rows.rs:41`), um número de motor.
- **Unidades:** o componente guarda metros. O ecrã passa pela porta `unidade_mundo` (px com 100 px/m no
  projeto em px). A crítica não achou isso. **Defeito:** as dicas dizem «in metres» com o ecrã em px
  (`ph2d-i18n/src/inspector_player.rs:139,144`).
- **Dicas no hover:** já existem, uma por campo (`inspector_player.rs`, 111 chaves), em texto. Não há ilustração (decisão de 25/10).

### O que liga cada habilidade (leitor do motor)

| Habilidade | Ligada quando | Padrão (`STARTING_POINT`) | No Add ▸ Character |
|---|---|---|---|
| Walls | `slide>0 ∥ jump>0 ∥ grab>0` (`ph2d-platformer/src/wall.rs:256-257`) | 0 · 0 · 0 (Push 6, Lockout 0,2 já prontos) | off |
| Dash | `speed>0 && time>0` (`dash.rs:87-88`) | speed 0 · time 0,15 · cooldown 0,2 | off |
| Crouch | `0 < height < float_height` (`crouch.rs:110-111`) | 0 | **ON, 0,50 m** (`ph2d-app-game/src/object_add.rs`, ramo `CHARACTER`) |
| Swim | `speed>0 && accel>0` (`swim.rs:171-176`) | 0 · 12 | off |
| Ledge | `grab>0` (`ledge.rs:200-201`) | 0 (reach 0,6, speed 3 prontos) | off |
| Glide | `fall_speed>0` (`glide.rs:83-84`) | 0 | off |
| Max Fall | `max_speed>0` (`descent.rs:86-91`) | 0 = sem teto | off |
| Air Jumps | `air_jumps>0` (`jump.rs:326`) | 0 | 0 |

São **sete** regras «0 = desligado». A crítica contou seis e não viu o Max Fall. Walls tem **três** sub-habilidades
num cartão só: deslizar, pular da parede e agarrar.

Padrões de Walk e Jump: Speed 6 m/s · Accel 60 · Air Accel 20 · Brake 1 · Slope 45° (`walk.rs:157`);
Jump Height 2 m · Air Jumps 0 · Air Jump H 2 · Takeoff G 1 · Takeoff Above 0 · Peak G 0,5 · Peak Window 1,5 ·
Fall G 2 · Cut G 4 · Coyote 0,1 s · Buffer 0,1 s · Corner 0,12 (`jump_config.rs:234-256`).

**Mode:**
- Pure = controlador de Kinematic que **não empurra nada** e é bloqueado por tudo (`components/player.rs:567-580`). A palavra não se explica.
- Os dois «Body» na tela são o Physics Body (Dynamic·Static·Kinematic) e este Mode.
- Emit Signals é só a presença de `PlayerSignals`, lida por `bridge/signals.rs:267-271`.

**Teclas:** o motor lê as ações do Input Map pelo nome: `move_left/right/up/down`, `jump`, `down`, `dash`,
`grab` (`ph2d-input/src/map.rs:12-22`, `ph2d-physics-ecs/src/player_input.rs:62-82`). Trocar teclas é da
linha InputMap e fica fora daqui.

**Da crítica já não vale:**
- «Crouch nasce desligado»: no Character nasce ligado.
- «rótulos em px, catálogo em m»: o ecrã segue o projeto; o defeito está nas dicas.
- «seis chaves»: são sete regras escondidas, e Walls guarda três.

### Precedentes que a ficha respeita

- GEF 09 e 10: Crouch Height é um número próprio, com o botão **Fit crouch**, e a arte de agachar (tags `crouch`
  e `crouch_walk`).
- 14/09: **Make Platform Player** aparece em todo corpo Dynamic, com «todas as opções».
- 02/11: o jogo se monta com peças; mesmo assim, o Character fica pronto (crítica, «Vale rever?», e o primeiro minuto de jogo).
- Molde: Sprite (01) e Animated (02). Existe um cartão fechado com resumo (`sections/grupo_dobravel.rs:39,76`). Existe
  uma caixa no título do cartão (`ph2d-editor-core/src/widget/checkbox/card_box.rs:15-48`, usada em
  `panel/cards_corpo.rs`), mas a moldura do player (`rows_moldura.rs:37,69`) ainda não a tem.

## O que as 9 ensinam

- Unity, Godot e GameMaker fazem o herói em código.
- Unreal, Roblox, UEFN, Construct e GDevelop têm um pronto, com Speed e Jump à frente.
- UEFN liga cada habilidade por nome («Allow Sprinting», «Allow Sliding»).
- **Nenhuma das fontes lidas tem coyote, buffer, wall jump e dash prontos no mesmo componente.** Só o
  GDevelop chega lá, por extensão da comunidade. É o nosso trunfo, e hoje o painel o esconde.

## A escolha do dono

A crítica traz três maquetes, com o herói azul real:
[critica_ph2d.html §6](../../../../../GameEngineSamples/critica_ph2d.html#character).

**A · Três números e seis palavras.** Speed, Jump Height e Air Jumps à vista. Abaixo, «Can also» com seis palavras
que acendem; o resto vai para um «Fine tune» dobrado. 12 à vista.

**B · Andar, pular, e o resto com chave** (recomendada pela crítica e por mim):

```
Walk                                  ▾
  Speed          600 px/s
  Acceleration  6000 px/s²
Jump                                  ▾
  Jump Height    200 px
  Air Jumps        0
▸ More Walk     Air 2000 · Brake 1 · Slope 45° · walks off ledges
▸ More Jump     Takeoff 1 · Peak 0.5 · Fall 2 · Cut 4
▸ ☐ Walls       slide · wall jump · grab
▸ ☐ Dash
▸ ☑ Crouch      50 px
▸ ☐ Swim
▸ ☐ Ledge
▸ ☐ Glide
▸ Forgiveness   Coyote 0.1 s · Buffer 0.1 s
▸ Leg           Float (a sola da arte) · Fit
▸ Advanced      Fall limit off · Mode Dynamic · Signals off
```
Cerca de 13 à vista no papel; o censo do teste dá o número real, como no Animated. Nenhum campo some.

**C · Puxar o arco do pulo.** A crítica a deixa como segundo passo, por cima da B. Está fora desta rodada.

### Como fica a B (decisões técnicas que tomo, salvo veto)

1. **A caixa de cada habilidade grava-se** (um `enabled` por habilidade em `PlatformPlayer`). Desligar guarda os
   números. Ligar devolve os números de antes, ou um ponto de partida que já funciona. Por exemplo, ligar o Dash
   com Speed 0 não pode dar um dash parado: nasce com 12 m/s, e ligar Walls nasce com deslizar e pular da parede.
   O motor passa a ler `enabled && números válidos`. Isso sobe o `PROJECT_SCHEMA` uma vez (§0.11: sem projetos antigos).
2. **Walls fica com uma caixa só.** Dentro, Slide, Wall Jump e Grab mostram «off» no lugar do zero. Os sensores
   (Reach, Rays, Spread, Lockout) ficam num «Wall sensor» dobrado.
3. **Max Fall vira «Fall limit»** com caixa, em Advanced. O Air Jumps continua número, porque 0 é natural numa contagem.
4. **Ordem:** Walk e Jump primeiro, Leg quase no fim. Fit fica dentro de Leg; Fit crouch, Crouch walk off e Crouch
   Speed ficam dentro de Crouch.
5. **Mode** vai para Advanced. «Pure» passa a dizer o que faz no próprio rótulo: «Kinematic (pushes nothing)».
6. **Emit Signals** vira caixa. **Live** só aparece durante o Play.
7. **Dicas:** as gravidades e o perdão ganham as frases já existentes, corrigidas para a unidade do projeto (px
   ou m). A dica com desenho fica para a C.
8. Fecha o comentário «onze» → doze.

## Perguntas ao dono (produto)

1. **A, B, ou B com as palavras da A ao adicionar?** A crítica e eu recomendamos B.
2. **Walls com uma caixa só** (deslizar, pular e agarrar dentro), ou três caixas separadas?
3. **«Vale rever?» da crítica:**
   - o botão Make Platform Player em todo corpo Dynamic (a ordem de 14/09 diz que fica);
   - o Character pronto demais (fica).

   Mantenho os dois, salvo ordem.

## Aceitação

- **Poder:** nenhum dos 52 números some. Cada habilidade liga e desliga sem perder os números. Ligar dá
  um movimento visível já no primeiro Play.
- **Facilidade:**
  - de 52+6 à vista para ~13 (censo);
  - pulo duplo em 1 gesto (Air Jumps à vista);
  - Dash em 1 gesto (caixa).
- **Intuitividade:**
  - nenhum «0» quer dizer desligado;
  - o primeiro número é Speed;
  - uma palavra só chamada Body;
  - as dicas na unidade da tela;
  - todo texto em inglês.

**Critério de parada:** se ligar uma habilidade pedir mudança na ponte de física (`bridge/*`, `physics_apply.rs`,
da GEF02), volto ao dono antes.

## O desenho aprovado

Versão B, Walls com uma caixa só (dono 10/11). Painel em `sections/player_cartoes.rs` + `player_grupos.rs`.

- **Walk e Jump abertos.** More, Gravity, Wall Sensor, Forgiveness, Leg e Advanced dobrados, cada um com resumo.
- **Seis caixas de habilidade** (Walls, Dash, Crouch, Swim, Ledge, Glide) ao lado do chevron. Desmarcada: a lei recebe a habilidade inerte (`config()`) e os números ficam guardados. Marcada: voltam.
- **Fall Limit e Emit Signals** viram caixas em Advanced (`fall_limit_on`; Emit = presença de `PlayerSignals`).
- **Live** só aparece no Play. **Pure** diz «Kinematic, pushes nothing». As dicas deixam de dizer «in metres».
- **Dados:** `PlatformPlayer` ganha sete `bool` no fim (`walls_on`, `dash_on`, `crouch_on`, `swim_on`, `ledge_on`, `glide_on`, `fall_limit_on`). Os números de nascença moram em `ph2d-physics-ecs/src/components/player_pronto.rs` (o Default já funciona ao ligar).
- **Character nasce com Crouch marcado.** Marcar Crouch com altura que a lei recusa ajusta-a à cápsula (como o Fit crouch).
- **`PROJECT_SCHEMA` 231 → 232** (§0.11, sem leitor do antigo). Catálogo: campos 55–61.
- **Três curas de UI pedidas no smoke:** (1) as 12 dobras não abriam pelo clique (faltava `mark_collapsible_section`); (2) a dobra de cada habilidade revela-se na largura inteira (só o título começa depois da caixa); (3) FUNDAÇÃO dos cartões: `widget::corte_da_dobra` (o campo «à espera» do fluxo pinta-se dentro da banda da dobra em que nasceu) e `widget::chave_da_altura` (altura lembrada por modo de pintura; o levantamento fora do cartão e o cartão partilhavam a chave). Valem para TODA dobra em cartões.

**Da crítica já não vale:** (mantido acima) «Crouch nasce desligado» · «rótulos em px, catálogo em m» · «seis chaves».

## Provas

- **Por nome:** `a_caixa_desmarcada_desliga_e_a_marcada_liga` · `a_parede_desmarcada_zera_as_tres_partes` · `cada_caixa_liga_so_a_sua_habilidade` · `marcar_o_crouch_com_altura_que_nao_serve_ajusta_a_a_capsula` · `every_ability_box_reaches_the_bus_in_both_directions` · `cada_row_da_tabela_tem_um_e_um_so_lugar_no_painel` · `cada_dobra_do_player_abre_pelo_clique_no_titulo` · `a_fechar_nenhum_campo_da_dobra_passa_por_cima_do_titulo_seguinte` (vermelho com `corte_da_dobra → None`) · `a_abrir_todo_campo_dentro_da_banda_aparece_inteiro` (vermelho com a chave única). Os três últimos em `ph2d-panel-inspector/tests/it/seam_player.rs`.
- **Mutação: 16/16 sangram.** M1c e M10 sobreviviam e ganharam teste.
- **Fotos:** [`fotos_03/`](fotos_03/) (`character_fechado.png`, `character_aberto.png`).
- ~45 testes e cenas de smoke passaram a marcar a caixa onde ligavam a habilidade pelo número.

## Abertos

- O gate de geometria dos cards do player (`seam_player`) prova menos: «nenhuma row se sobrepõe» no lugar do passo da moldura por card da tabela.
- ~~As 12 dobras novas não têm dica de hover~~ — fechado no GEF 05 (12/11): cada dobra diz o que mora dentro (`toda_dobra_do_player_tem_dica`; a isenção saiu do `every_player_control_carries_a_hover_hint`).
- ~~`open_all_sections` não abre as dobras novas~~ — não era defeito (12/11): abre, provado por `abrir_todas_as_seccoes_abre_as_dobras_do_player`; o laço à mão do `seam_px_m` saiu.

## Próximo (decisão do dono 10/11)

A curva do pulo no CANVAS (`project-memory/project_jump_arc_on_canvas_2026_11_10.md`): no cartão só o botão que a liga e o seletor Floaty · Snappy · Heavy. O canvas mostra pulo inteiro, pulo curto, pulos no ar, parado e correndo; a alça dá a altura (Jump Height) e a distância (Speed).

## ⛔ Recusas (desta ficha)

| O quê | Por quê |
|---|---|
| Arco puxável (C) nesta rodada | é desenho novo; a crítica o põe depois da B |
| Trocar teclas no cartão | é da linha InputMap; o motor só lê as ações |
| Conselho automático («Buffer is 0 s») da A3 | é ideia nova sem regra medida; a crítica diz «a regra fica para quem testar» |
| Make Platform Player como «+ Add component» | contraria a ordem de 14/09 |
