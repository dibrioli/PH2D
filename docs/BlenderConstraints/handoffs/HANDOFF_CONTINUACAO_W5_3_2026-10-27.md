# Continuação — `line/BlenderConstraints`, W5.3 «Valor segue valor» FEITA (smoke do dono 27/10)

> Anterior: [`W5.2`](HANDOFF_CONTINUACAO_W5_2_2026-10-27.md) (a cola, fechada). Desenho das três peças, medições,
> triagem de licença e decisões do dono: [`06_W5_resto.md`](../06_W5_resto.md) — **não duplicado aqui**.

## Identidade

Worktree `Worktrees/line-BlenderConstraints`, branch `line/BlenderConstraints`, merge-base `c3662cd8e4`.
`PROJECT_SCHEMA` 196 e ADR 0193 provisórios (como na W5.2). Família de canal `CONSTRAINT`: `MOLA_LIGADA = 38`
(append). Menu Add Constraint: índice **20** = Follow Value (18/19 são as colas). Cenas de smoke 1–**9**; a próxima é a
**10**. Shell: +11 linhas nesta onda (folga da catraca `the_shell_only_shrinks` era 30 → ~19).

## Decisões do dono (27/10)

1. Ordem: **Valor → *Behaviours* → HUD**. Valor e *Behaviours* no **Add Constraint** (*Behaviours* num submenu); o
   arranjo do HUD é uma **ferramenta na barra esquerda** com setas entre elementos, nunca lista no Inspector.
2. Botões do cartão Constraints: **«deixe como está»** (recusados a pega de arrasto e o menu «⋯») ⇒ catraca
   `CARGA_DE_COMANDOS` inspector `89 → 97` com a justificação no teste.
3. Smoke da cena 9: respondeu à pergunta e pediu a continuação, sem defeito relatado.

## Feito nesta janela (não refazer)

| Commit | O quê |
|---|---|
| `6a8e53910c` | `06_W5_resto.md`: as três peças medidas no código antes do desenho |
| `416c0c0710` · teste seguinte | `Simulador<const N>` + `Lei<N>`; a pose é `N = 3`, **mesmo bit** (impressão FNV `0x40b3_b313_e43b_d8ce` medida no código de `fe0777808d`) |
| wip `Kind::Valor` | `ph2d-constraints/src/valor.rs` (`Prop = Pose(Canal) \| Campo([u16;4])`, `faixa()` partilhada com o Transformation); `ph2d_timeline::field_at` (irmão do `pose_at`, mesmo preâmbulo `instante`); `Valores` + ramo na pilha; `Estado::SemValor`; `CanaisEscritos::valor` |
| gates + cura | 7 gates do Valor; **cura**: pose sem curva não é «animada» (`world_pose_at` devolve a VIVA → a mola nunca via a mudança) |
| `c6139ee692` | `valores()` corre entre o apply da timeline e os drenos (`fase_timeline_drain.rs`) |
| kill | **930 µs → 427 µs / 1 000** com mola (índice de alvos, `retain` por conjunto) — `tests/it/kill_w5.rs` |
| `cb20c89376` | `LeitoresDosCanais` (ph2d-ecs): o Valor pede a publicação dos campos que lê fora da seleção (`fase_autokey.rs`) |
| cena 9 | «Mostrador»: bóia animada → ponteiro (rotação, mola 0,6 s / 0,4) e lâmpada (opacidade, campo) — fotografada |
| cartão | Follow/Drive como ENDEREÇOS (`ConstraintFieldEdit::Propriedade`), lista do mundo com tecto `MAX_PROPRIEDADES = 256` (medido: sprite 28 campos, forma de 20 vértices 187); `MAX_NUMEROS` 5 → 6; `Unidade::Nenhuma` |
| `15abce7fd4` | fecho: rustfmt; clippy `-D warnings` verde nas 10 crates |

## Portões por nome verdes

Valor: `the_pose_spring_is_the_same_bit_after_the_generic_simulator` · `a_one_value_spring_is_the_law_step_by_step` ·
`a_value_follows_another_value_through_the_range` · `a_followed_field_is_a_preview_never_the_document` ·
`a_springy_value_rides_the_spring_and_scrubs_to_the_same_bit` · `an_unknown_property_is_inert_with_a_reason` ·
`a_value_link_survives_save_and_open` · `the_dial_scene_points_the_needle_by_the_float_and_lights_the_lamp` ·
`every_level_up_to_the_router_ceiling_answers`. Cartão (vermelho-primeiro por mutação):
`the_value_card_lists_the_targets_properties` · `editing_the_value_card_writes_the_item` ·
`the_value_cards_follow_dropdown_writes_the_chosen_property`. Timeline: `pose_at_*` (9/9). ECS: `canal::` (7/7).
Kill (ignorado, `--profile smoke`): `w5_pieces_under_half_a_millisecond_per_thousand` (hoje só a variante Valor).

## ⚠ Para o integrador

- `ph2d-ecs` (`canal.rs`: `LeitoresDosCanais`, `CanaisEscritos::valor`), `ph2d-timeline` (`pose.rs`: `field_at`,
  `instante`) e `ph2d-editor-core` mudaram — crates de que muitas dependem.
- Herdados do main, não curar na linha: `shell_files_respect_hr18_loc_cap`, `shell_functions_respect_their_ceiling`,
  e `a_altura_de_abertura_de_um_painel_so_encolhe` (painter_layers 1620 contra 1570, já no merge-base; instável em grupo).
- Ficheiros de teste > 700 linhas que só ganharam linhas: `architecture_every_editable_field_is_a_channel.rs` (+7),
  `o_inspector_armado.rs` (+2).

## Próximo passo — W5.4 *Behaviours* de um clique, depois W5.5 HUD

Desenho em `06_W5_resto.md` §3.2 e §3.3. As leis dos cinco nós são **privadas** em cada crate (`waveform`/`skew` no
oscillator, `orbit_point` no orbit, `field`/`curve` no falloff, `curve`/`aim_at` no attractor, o fBm em
`ph2d-node-motion-wiggle/src/noise.rs`): expor `pub fn` (append) e o nó + o `Kind::Comportamento` chamam a MESMA lei;
gate de paridade contra o `eval` do nó numa instância. Índices do menu 21+ (submenu). O Atrair é o único com estado:
`Simulador<2>`. O kill final das três peças é UMA rodada intercalada em `kill_w5.rs`. Cena 10.

## Armadilhas novas desta janela

- `world_pose_at` responde SEMPRE (pose viva de quem não tem curva): «animado» = tem binding de pose.
- As famílias só publicam `CanaisLidos` da população do Auto Key + campos animados: quem lê outro objecto pede-o
  (`LeitoresDosCanais`), nunca pelo `AlvosDasFamilias` (esse entra no Auto Key).
- A mola de um valor é função do QUADRO da timeline: com a animação parada, arrastar a fonte segue na hora (como a Spring).
- Fonte viva (sem curva): a história guarda só os quadros em que mudou, e escrever no quadro `f` apaga o futuro.
- Fish não parte `$VAR` com espaços em argumentos — caminhos à vista.
