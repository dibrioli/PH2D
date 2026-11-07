# Handoff de continuação — GEF 23 Sprite FECHADA (06/11)

Linha `line/GameEngineFuncional`, worktree `Worktrees/line-GameEngineFuncional`. **Sprite = `fechado`**: o dono, no smoke de 06/11, disse «Parece que tudo está funcionando perfeitamente. Vamos aceitar como está. Smoke OK». Ficha: [01_sprite.md](../componentes/01_sprite.md) (decisões 20–23). Fila: [23_refundacao_plano.md](../23_refundacao_plano.md). Antes: [onda 3](HANDOFF_CONTINUACAO_2026-11-06_23_SPRITE_ONDA3.md) (rodada 2 lá).

## Estado

- `git log --oneline bbfefaf393..HEAD` (base de reabertura da linha). **Nada integrado, nenhum push.** Até onde se sabe o `main` não andou: **`git fetch` e conferir antes de qualquer coisa**.
- Árvore limpa na ponta do código (`e0d693533a`); depois dela só docs e memória.
- `/pd-linha-fechar` (DIRETRIZ §1.5.9) só antes de integrar e só por ordem do dono. O link do Game Engine no CLAUDE.md §5 e o smoke do Pixel (`1..21`) NÃO foram trocados: são do fecho da linha.

## O que a rodada 3 fez (smoke do dono, 06/11)

| Commit | O quê |
|---|---|
| `956da52724` | *Keep in place* na linha dos botões do Mirror; desce à largura inteira onde não cabe (`cards_sprite_pivot.rs`). |
| `de7b59e845` | O desenho fica parado com o Pivot animado, na Timeline e no jogo (abaixo). |
| `a44701614f` | Ctrl+Z com chaves anda até o fim; tabela do Mirror (`pivot_espelho_tests.rs`). |
| `e0d693533a` | Avisos triviais do clippy nos testes do Pivot e da caixa. |

**Tremor (trajetória curva).** Com 3+ chaves de posição, `add_path_key` → `resmooth_auto` (`crates/ph2d-timeline/src/doc_path.rs`) suavizava a trajetória; o pivô gravado com a posição desviava ~20 px. Cura: `path_key_corner` / `TimelineIntent::PathKeyCorner` (`doc_path.rs:294`, `intent.rs`, `intent_apply.rs:146`): a chave gravada com o pivô vira canto, a trajetória é reta (~20 px → <1 µm). Emitido em `ph2d-sprite-edit/src/pivot_chave.rs:119` e no Auto Key (`shells/desktop/src/render_loop/autokey_pass.rs:525`).

**Atraso no jogo.** A curva é aplicada dentro do passo da física (`ph2d-app-physics/src/bake.rs:150`, `put` → `apply_from_doc`, chamado por `fase_physics_step`) e o dreno dos canais corria ANTES. Cura: `drena_os_canais` também depois da física (`shells/desktop/src/render_loop/fase_frame_open.rs:75-76`); vale para todas as famílias (2,5 px → 0).

**Ctrl+Z preso.** Depois de um restauro a curva repõe a pose e o Auto Key a gravava como passo, então o Ctrl+Z seguinte desfazia só a si. Cura: `restauro_assenta` (`shells/desktop/src/undo_app_macros.rs:72`) refaz a base durante `QUADROS_DO_RESTAURO = 3` quadros (~50 ms) sem criar passo. **Custo:** uma edição do artista nesses ~50 ms entra no restauro e não vira passo.

## Verificação do fecho de onda

fmt, clippy nas crates tocadas, testes por nome verdes e build smoke (feitos pela janela principal; a suíte completa é da integração). ⚠️ `PathKeyCorner` **não tem teste dirigido em `ph2d-timeline`**: é coberto pelos testes da bolinha e do Auto Key e por mutação.

Provas desta rodada: `pivot_espelho_tests.rs` (144 células, 60 linhas do oráculo; 12 linhas de Flip com pivô de lado da Unity excluídas pela decisão 14) · `so_o_mirror_v_leva_a_moeda_para_baixo` · `com_tres_chaves_de_pivo_o_desenho_nao_anda` e `a_bolinha_grava_a_trajetoria_reta` (`pivot_chave_tests.rs`) · `a_chave_do_pivo_grava_a_trajetoria_reta` (`autokey_pass_tests.rs`) · `o_restauro_arma_e_o_jogo_drena_os_canais_depois_da_fisica` · `o_quadro_depois_do_restauro_refaz_a_base_sem_criar_passo` (`undo_app_restauro_tests.rs`) · censo `o_censo_dos_campos_da_sprite_em_uso` = **13 à vista, 31 no total**.

## Decisões finais do dono (na ficha, 20–23)

Mirror object vira a imagem como nas duas engines; com Keep in place vira sem sair do lugar · Mirror num objeto com Pivot já animado vale dali em diante e a imagem se desloca ao tocar: aceito, sem cura (medido: Mirror V em t=1,5, desenho y 1,5..2,0; Mirror H, desenho salta de x 8,5 para 11,5 entre t=1,5 e t=1) · moeda abaixo da plataforma só com o Mirror vertical, por lei · Keep in place na linha dos botões do Mirror.

## Abertos que NÃO bloqueiam (integrador ou Timeline)

- **Cada ida da agulha cria passo de undo.** Decisão do dono: fica como está por ora; tratar na etapa da Timeline. Mecanismo no handoff da [onda 3](HANDOFF_CONTINUACAO_2026-11-06_23_SPRITE_ONDA3.md) (`ph2d-timeline-preview/src/lib.rs:163-166`, `settle`).
- Salto de um quadro ao desfazer com chaves (até 1,5 m).
- `gizmo_anchor_half` (`despacho_vetor_alinhar_e_forma.rs:264-279`) ignora `centered`/`offset` no arrasto.
- Chaves já gravadas continuam curvas (a cura vale para as novas).
- Cliques reais nos botões não dirigidos pela sonda.
- Rotação fora da tabela do Mirror (30° só no oráculo).
- Colisão no jogo medida no `Collider`, não dentro do rapier.
- As dívidas «antes de integrar» da [onda 3](HANDOFF_CONTINUACAO_2026-11-06_23_SPRITE_ONDA3.md) (tecto da miniatura, limite do New Image, Painter em canvas gigante, afim × `centered`/`offset`, smoke do Pixel `1..21`, save em Draw por `.ph2dproj` real, Tab/seletor da cópia única, ramo `INSP_IMAGE_EDIT_PIXEL`).

## Próximo item: Animated Sprite (proposta INICIAL, não aprovada)

Tudo abaixo é levantamento; o que for hipótese está marcado. Nada disto foi levado ao dono.

**Protocolo** (README §Protocolo e §Molde): falta ler «Suas perguntas», «Lista consolidada» e o resto das 12 ideias nos estudos, fotografar o painel e fazer os «testes de 1 minuto». Depois: proposta formal ao dono → ficha `componentes/<NN>_<nome>.md` → construção → smoke.

**Engines** (`python3 -I docs/GameEngineFuncional/ferramentas/estudo_secao.py "AnimatedSprite"` e `"Animated Sprite"`). Padrão: lista de animações nomeadas, FPS por animação, loop, ping-pong. Famílias: editor próprio (Godot, Construct, GameMaker, GDevelop, Unreal), timeline genérica (Unity, Cocos), nenhum (Roblox, UEFN). Só em algumas: duração por quadro (Unreal, Godot, Construct), evento no quadro (Unreal, GameMaker, Unity), colisão por quadro (Unreal, GDevelop), desenho no editor (GameMaker, Construct, GDevelop), fatiar por grade (Godot, Unreal, GameMaker). Nenhuma tem: fila só com intervalos nomeados; estado do personagem ligado à animação no painel.

**App hoje** (conferir no código antes de usar):
- Add: `crates/ph2d-app-game/src/object_add.rs:26-29,162`; `anima()` ≈ `:110-135` põe a folha do herói, 6 tags e autoplay.
- Use=Animated: `crates/ph2d-app-sprite-anim/src/image_use.rs:28,39`; `make_animated` `:71-94`; `proposed_grid` `:172-180`.
- Cartão: `crates/ph2d-panel-inspector/src/populate_anim.rs:18-225`, `event_anim*.rs`, `ids/inspector_anim.rs`. Editor: `crates/ph2d-app-sprite-anim/src/editor/`, `card.rs`. Modelo: `crates/ph2d-ecs/src/sprite_anim.rs:154,309`. Import: `shells/desktop/src/ase_import.rs`.
- Plano `docs/GameEngineFuncional/22_animacao_revisao_plano.md`; decisão `project-memory/project_animation_section_like_godot_2026_11_04.md`.
- A crítica diz que não há marcador por quadro, mas há código (`event_anim_markers.rs:18-39`, `populate_anim.rs:106-125`).
- **Hipóteses não confirmadas:** marcador só no editor · `Direction`/overrides ocultos sem valor (ids em `populate_anim.rs:34-35,196-204`) · Auto Key ausente no cartão · trocar a imagem não refaz a grade (`image_choice.rs:61-68`) · Pivot herdado · os números 24/54 hoje e 10/29 na versão B vêm da crítica e **não foram recontados**.

**Cartão proposto (versão B da crítica; hipótese).** À vista, 10: Image, lista de animações com ▶, Frames/s, Cycle, Who picks (Me/Script/Rules), lista de regras, Speed. Dobrados: Hold, Repeat delay, Repeat count, On Finish, On Loop, Name. Fora: From/To (vão ao editor), Direction, overrides. Reutiliza da Sprite: Image, Look (Flip, Mirror, Tint, Opacity), Transform e Ordering dobrados. ⚠️ Cuidado com as chaves de Flip e Pivot da Sprite (decisões 12 e 16 da ficha).

**Perguntas ao dono** (as 5 decisões):
1. Folha no editor × no cartão.
2. Who picks × «Começa com».
3. Nascer vazio com «Try with the sample hero» × vazio puro.
4. Marcador de quadro só no editor.
5. Regra acesa no Play e ensaio com teclas, depois.

## Prompt de continuação para a janela nova

```
/pd-linha-assumir GameEngineFuncional

Faça `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional`, `pwd` e `git branch --show-current` ANTES de ler. Sem reset e sem descartar commits. Não integre e não dê push sem ordem do dono.

Leia docs/GameEngineFuncional/handoffs/HANDOFF_CONTINUACAO_2026-11-06_23_SPRITE_FECHADA.md. A Sprite está fechada (smoke do dono OK em 06/11). Faça `git fetch` e confira se o main andou.

Tarefa: completar o levantamento do protocolo do Animated Sprite (README §Protocolo e §Molde: «Suas perguntas», «Lista consolidada», resto das 12 ideias, fotografar o painel, testes de 1 minuto), conferir no código as hipóteses do handoff e levar a proposta formal ao dono, com as 5 perguntas. Só depois a ficha, a construção e o smoke.

Comigo: respostas curtas e sem jargão; smoke em passos numerados (comando completo com cd, onde clicar, o que tem de acontecer, como saber que deu errado).
```
