# F28 — um valor fora da lista de escolhas (plano, 24/10)

Fila: [§7 do handoff F27](handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-24_F27.md) (itens 2 e 4). Base `c3662cd8e4`
(o `main` com a `line/CodeEditor` já integrada: a colisão do `options!` do F27 §3 foi reconciliada por `acab41422a`).

## A — o censo da palavra, MEDIDO no código

| achado | o que o código faz | veredito |
|---|---|---|
| (a) `MISTURA`/`PILHA_MISTURA`: 22 códigos, menu de 19 | O menu do painel (`paint_appearance.rs`, `paint_stack_rows.rs`) deriva de `ph2d_vec_render::blend::offered()` e escreve `mode.to_u8()` (`event_clicks.rs:469,505`), e o chip lê a linha por `offered().position` — **linha ≠ código desde a 4.ª, mas ninguém confunde os dois**. O defeito real: o canal declara `op(MISTURAS)` (22) e o Vetor só desenha 19. Os códigos **4, 12, 13** (Linear Burn, Vivid Light, Linear Light) dizem «Linear Burn» na dope-sheet, o chip diz «Normal» e o desenho faz Normal (`blend::is_neutral`). Eles entram por uma chave arrastada no gráfico (`SetKeyValue` grava qualquer `f64`, `intent_apply.rs:260`). | **defeito**: a palavra mente em 3 códigos |
| (b) `ENVELOPE_WARP` em inglês cru | O painel recebe `WarpStyle::label()` (`fase_selection_mirror_effects_envelope.rs:34`) e **traduz** pelo `nomes_do_motor::efeito` (`state_envelope.rs:53`): os 9 rótulos têm chave em `chave_do_efeito`. A ponte vai do texto para a chave (o cheiro de [«chave e texto do mesmo tipo»](../../project-memory/feedback_a_key_and_a_text_of_the_same_type_is_a_defect_waiting.md)), e três listas (`WarpStyle::label`, `chave_do_efeito`, `canal_vetor_escolhas::WARPS`) só concordam por acaso. | **o censo errou** (já traduz); falta o gate que amarra as três listas |
| (c) `TINTA_TRACO` | O tipo do traço usa o espaço de códigos do preenchimento (Solid 0 · Pattern 4 · Brush 5): os buracos 1–3 são de propósito, e a chave gravada guarda 4/5 (mudar seria formato). Mas o `tinta()` declara para o traço **todos** os deslocamentos do preenchimento: `N_PARAGENS`, `N_PONTOS`, paragens, linear, radial, pontos — campos que nenhum traço tem e que ninguém lê (`campos_da_tinta`, `ler_traco`). | **defeito**: declarações sem leitor; os buracos ficam, e a escrita não os alcança mais (abaixo) |
| (d) `PILHA_DILATA_JUNCAO`/`TEXTO_ALINHAMENTO` 0..255 | `escrever` usa `byte(v)` (`canais_campos.rs:484-526`) sem a faixa; os filtros já prendem (`canais_filtros.rs:143`). | **defeito** |

**A família é uma só: um valor discreto fora do vocabulário do campo.** Junta-se a ela o F25 §6 item 7 (`SetKeyValue` aceita
fração; uma expressão num campo discreto produz índice arbitrário), que é o MESMO mecanismo. Cura num bloco:

1. **`FieldKey::prende(v)`** (`ph2d-ecs`): `Continuo` intacto · `Ligado` 0/1 · `Opcoes` = a escolha mais próxima com palavra
   (`""` é buraco; empate para baixo) · `Catalogo` = arredonda e prende ao número de opções do campo · resto (Número, camada,
   script, texto, por irmão) = arredonda, ≥ 0.
2. **O funil do mundo:** `CanaisEscritos::escrever` prende (é a única porta por onde curva, pilha e expressão chegam a qualquer
   família). **O funil da chave:** `AddKey`/`SetKeyValue` de um canal discreto guardam o valor preso (a palavra da chave
   não mente).
3. **Vetor:** `MISTURAS_DO_VETOR` (buracos em 4, 12, 13) para `MISTURA`/`PILHA_MISTURA`, amarrada ao `offered()` por gate;
   o traço só declara o que tem; alinhamento e junção prendem na escrita (defesa na fronteira, como os filtros).

Gates (cada um visto VERMELHO por mutação antes de valer):

| gate | crate | mutação que o avermelha |
|---|---|---|
| `o_menu_de_mistura_e_a_palavra_dizem_o_mesmo_modo` | `ph2d-app-vec` | `MISTURA` volta a `op(MISTURAS)` |
| `um_valor_fora_da_lista_cai_na_escolha_mais_proxima` | `ph2d-ecs` | `prende` devolve `v` |
| `a_curva_so_escreve_escolhas_que_existem` | `ph2d-ecs` | `escrever` sem `prende` |
| `a_chave_de_um_discreto_guarda_uma_escolha` | `ph2d-timeline` | `SetKeyValue` sem `prende` |
| `alinhamento_e_juncao_so_aceitam_as_tres_escolhas` | `ph2d-app-vec` | volta o `byte(v)` |
| `o_traco_so_declara_o_que_tem` | `ph2d-app-vec` | `tinta()` volta a declarar `N_PARAGENS` no traço |
| `os_presets_do_envelope_dizem_a_palavra_do_canal` | `ph2d-app-vec` | tira um rótulo de `chave_do_efeito` |

## B — os herdados, conferidos no CÓDIGO

| item | veredito | prova |
|---|---|---|
| F18–F26 §6.5 (troca de tipo pela curva sem re-seed) | **fechado** | único escritor de produção `joint.rs:454` re-semeia (`:472-494`); gates `o_tipo_o_active_o_collide_e_o_mundo_chegam_pela_curva`, `escrever_o_mesmo_tipo_nao_mexe_na_junta` (F25 `6e2da651f4`, F26 `2a04dd99c7`). A cena escrevia direto, mas a cena não é produto. |
| F25 §6.6 (eixo do motor zera o alvo) | **desenho** | trocar o eixo muda a UNIDADE do alvo (ângulo ↔ distância): o zero é a resposta certa (`joint.rs:443-449`); se a faixa tem curva, `canais_junta.rs:373-396` repõe-na no mesmo quadro. |
| F25 §6.7 (fração / índice arbitrário) | **aberto → cura A** | `intent_apply.rs:260` grava qualquer `f64`. |
| F25 §6.8 (losango da camada do Flip só em Draw/Edit) | **desenho** | a lista de camadas só existe no painel do Flip (`ph2d-panel-flip/src/paint.rs:130`): o losango mora ao lado do controle que grava. |
| F25 §6.9 (contentor do Envelope na população do Auto Key) | **a medir** (teste vermelho primeiro) | a população é a seleção do gizmo + as mãos (`autokey_amostra.rs:60`); o painel acha o contentor subindo dos FILHOS seleccionados (`envelope_live::sole_container`). Com um filho seleccionado, o contentor fica fora. |
| F25 §6.11 | parcial | `marcas()` 51,5 → 0,23 ms (F27 B3+C), menu da linha de grupo (F27 D), ADR-0191 (F27): **fechados**. Perf de `linhas::layout`, «gate de fecho da LINHA» e undo da timeline separado do do vetor: fora da família, seguem abertos. |
| F18–F26 §6.9 (`canal.junta.*` só no locale base) | **não é defeito** | só há UMA tabela (`ph2d-i18n/src/canais.rs`, 34 chaves `canal.junta.*`); o pseudo-locale `teste` deriva dela. |

## Smoke

`PH2D_AUTOKEY_SMOKE=34`: o menu de misturas do Vetor e a palavra na dope-sheet a coincidirem nas 19 escolhas, e uma chave
arrastada para fora da lista a cair numa escolha que existe. Fotografada antes de ir ao dono.

## ⛔ Recusas MEDIDAS

- Renumerar o tipo do traço para 0/1/2: muda o valor das chaves gravadas (formato) para tapar buracos que, presos, já não se
  alcançam.
- Ensinar o painel do Vetor a ler a lista de `ph2d-ecs`: o `offered()` é a tradução para o Vello, que é quem sabe o que se
  desenha; a lista do canal amarra-se a ele por gate, nunca o contrário.
