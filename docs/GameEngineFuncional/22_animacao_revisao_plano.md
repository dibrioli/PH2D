# GEF 22 — Revisão da secção Animation contra a Godot (plano)

> Pedido do dono (04/11): *«Ping-pong tem múltiplas ocorrências; toda seção precisa ser revisada»*; os overrides
> saem do painel, *«antes veja como a Godot faz, pois faz bem feito e de forma simples»*. ⚠️ O visual novo do Inspector
> é de outra linha (decisão do dono 04/11): esta onda NÃO alinha ao main nem mexe no visual dos cartões.

## O oráculo (Godot 4.7.2, MIT; `pacman -Qi godot`)

Corrido sem interface: `PH2D_GPU=1 bash scripts/ph2d-run.sh timeout 30 godot --headless --fixed-fps 20 --script docs/GameEngineFuncional/ferramentas/godot_animacao.gd`
(4 quadros a 5 fps; 48 amostras).

| Onde | Na Godot | Em nós (hoje) |
|---|---|---|
| Objecto (`AnimatedSprite2D`) | `speed_scale` e nada de direcção nem de repetição | Speed + Direction override + Loop override |
| Animação (`SpriteFrames`) | fps + `LoopMode`: `LOOP_NONE` · `LOOP_LINEAR` · `LOOP_PINGPONG` | Frames/s + Cycle (Repeat · Once · Ping-pong) + Direction (Fwd · Rev · PP · PP Rev) + Repeat count |
| Ao contrário | gesto de tocar: `speed_scale` negativo ou `play_backwards`; não é propriedade da animação | Speed negativa, Direction Rev, Direction override Rev |

Sequências medidas (cada quadro dura 4 amostras):

| Modo | Para a frente | `play_backwards` | `speed_scale = -1` em `play` |
|---|---|---|---|
| `LOOP_NONE` | 0 1 2 3 e pára no 3 | 3 2 1 0 e pára no 0 | pára logo no 0 (começa onde já acabou) |
| `LOOP_LINEAR` | 0 1 2 3 0 1… | 3 2 1 0 3 2… | 3 2 1 0 3 2… |
| `LOOP_PINGPONG` | 0 1 2 3 2 1 0 1… | 3 2 1 0 1 2 3… | anda para a frente a partir do quadro em que estava |

`animation_looped` dispara nas duas pontas do ping-pong; `animation_finished` só em `LOOP_NONE`.

## Feito (o dono: «pode fazer como deseja», smoke só no fim; 04/11)

| Antes | Agora | Porquê |
|---|---|---|
| Direction override e Loop override sempre em Advanced | a linha só aparece LIGADA e some em Inherit. Campo, ficheiro e `ph2d.set` iguais; `tick.rs` continua a usar o `loop_override` por dentro | a Godot não tem nada no objecto além da velocidade; esconder um override ligado prenderia o objecto a uma direcção invisível |
| Direction (Fwd · Rev · PP · PP Rev) sempre em Advanced | só com `direction_tag` 1 ou 3 (import `.ase`, projeto antigo) | o resto é o Cycle; ao contrário é a Speed negativa |
| PP, Fwd, Rev | Ping-pong, Forward, Reverse, Ping-pong Reverse | pedido do dono |
| «+ Add Animation», «x Remove Animation», «Edit Animations…» empilhados DEPOIS de Playback | «+ Add» · «Remove» lado a lado e «Edit Animations…», colados à lista, à largura inteira | os verbos ficam ao lado do que mudam |
| título com o nome da animação, «Name» igual e «N frames» repetido da lista | título «Selected Animation»; «N frames chosen» só com lista de quadros | repetição |
| From/To à vista | em Advanced | quem define os quadros é o editor |
| frase de Who Picks sempre à vista | balão de cada botão | ordem do dono 21/09 |
| regras «Cro…» «crou…» | fileira única se todos os nomes couberem (`dropdown_label_budget`); senão cada regra usa duas fileiras, e a pega ocupa as duas | nomes distintos liam-se iguais |
| «Signals (empty = silent)» | «Signals» + balão nos dois campos | esvazia a `AINDA_COM_REGRA` |
| Speed, Frames/s, Cycle, Hold, Repeat delay, From/To, Add/Remove sem explicação | balão em todos; a Speed diz que negativo toca ao contrário | Direction saiu daqui |

`inspector` ao abrir: 1 488 → 1 362 px. O Ping-pong agora aparece uma vez no Inspector (Cycle) e no editor de animações, que mostra o mesmo dado.

⚠️ **Speed negativa em Once a partir do quadro 0 acaba logo:** medido na Godot; em nós lido no código (`at_end`, `sprite_anim.rs`), sem medição própria.

## Provas

Gates: `um_override_so_aparece_ligado_e_o_inherit_o_desliga`, `a_direction_so_aparece_num_sentido_inverso`, `os_verbos_ficam_colados_a_lista`, `o_que_saiu_do_cartao_mora_no_balao`, `nomes_curtos_ficam_numa_fileira_so`, `card_fits_at` (nomes de regra e por extenso a 270, 290 e largura de fábrica). Mutação com base VERDE: 4/4 overrides, 8/8 do lote (Direction, From/To, duas fileiras, balões). ⚠️ A 1.ª rodada do lote de 8 foi deitada fora: a base estava vermelha e tudo «sangrava».

## Por medir no smoke

Balões e cliques: a tela virtual não recebe rato; estão provados pelo painel de teste, não vistos.
