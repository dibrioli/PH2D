# 09 — O alfinete nos cartões que ainda não o têm: plano com o tamanho medido (03/11/2026)

Leitor: a LLM que construir isto. **Construído: guarda, Material, Ordering, Object, Appearance (bloco C1, 04/11), Fill e Stroke (bloco C2, 04/11). Aberto: o losango de um campo preso (fim deste documento).** Ordem do dono (03/11, por `ph2d-58`): só depois da CI.
Pedido dele: «vários cards ainda não têm o alfinete». Foto: cartão Fill solto, cabeçalho ⠿ + ×, sem alfinete.

Hoje só o Transform e os cartões de componente são presos (T47, T61): `store.mark_card_pinnable`, em
[`cards_presos.rs:83`](../../crates/ph2d-panel-inspector/src/cards_presos.rs). Faltam **Object, Appearance, Fill,
Stroke, Ordering e Material**.

⛔ **Pôr o ícone sem rerotear a escrita é pior que a ausência:** a edição no objeto preso cai na seleção, e com
multi-seleção espalha-se por ela, sem erro nenhum. A pergunta de controlo morto vale aqui: o cartão preso
mostra E edita o objeto dele?

## O tamanho

Contagens e endereços lidos no código a `ff66debfb3`. **As linhas de publicação são estimativa de leitura, não
medição.**

| Cartão | Lê de | Já no `Retrato`? | Escritas a rerotear | Publicação nova (estim.) | Risco |
|---|---|---|---|---|---|
| **Material** | `current_inspector_blend` | sim | 1: o espalhamento em `fase_bus_inspector.rs:143` | ~20 linhas | baixo |
| **Ordering** | `current_inspector_ordering` | sim | 1: `fase_bus_inspector.rs:125` | ~20 | baixo |
| **Object** | `transform`, `visibility`, `name`; ícone por `current_appearance` | quase (falta o do ícone) | 1 + cadeado: `fase_bus_inspector.rs:94`; o cadeado usa a linha da Hierarquia do `entity_bits` do retrato | ~30–50 | médio |
| **Appearance** | `current_appearance` (`CURRENT`), opacidade/blend do objeto | não | `pending_vec_opacity`/`blend` em `fase_transform_ops.rs:73-88`, que não passam pelo `press_subject` | ~40–60 | médio-alto |
| **Fill** | `pilha()`: `CURRENT`, `BASE_PAINTS`, `BASE_FILL_RAMP`, estilo da ferramenta, pattern | não | ~10 chamadas em `fase_transform_ops.rs:76-127`, `apply_picker`, `estilos::drena` | ~80–150 | alto |
| **Stroke** | + `STYLE` (`cards_pilha.rs:192-200`) | não | + `set_kind` e `stroke_paint::apply`: 12 chamadas no total | partilha com o Fill | alto |

**Duas famílias, dois custos:**

- **Baratos (Material, Ordering, Object):** o estado já é do retrato e o pedido já leva `entity_bits`. O trabalho é
  a guarda do espalhamento: com um objeto preso, o aplicador não pode espalhar pela seleção.
- **Caros (Appearance, Fill, Stroke):** todas as escritas recebem `sel: &[VecPathId]`, e o chamador passa sempre
  `ordered(pen)`. A boa notícia: rerotear é passar `&[preso]`, sem funções novas de escrita. O caro é o que fica à
  volta: publicar por objeto, e os globais que não são por objeto.

## O que dói nos caros

1. **Publicar por objeto.** `vector_bridge_publish.rs:103` corre uma vez por quadro sobre a seleção e fora do
   `com_cada_preso`. Precisa de `paint_stack::published(scene, &[id], …)` por preso, mais fill e pattern
   (`:252-300`), que hoje não têm variante por objeto. Falta também o caminho `entity_bits → VecPathId` no momento
   do gesto (o `VecEntityMap` existe em `publish`).
2. **Globais que não são por objeto.** `OPEN` (`open_layer_index`/`set_open_layer`, `appearance.rs:125,175-196`):
   Fill preso e Fill da seleção partilhariam a camada aberta. E o seletor de cor: `apply_picker` aplica à seleção.
3. **A lista de estilos.** `Pedido::Aplicar`/`Soltar` só levam `camada`; `drena` aplica em `sel`
   (`paint_stack_estilos.rs:23-48`) e `LIGADOS` é da «forma mostrada», uma só (`:72-80`). O pedido tem de levar o
   preso, e `LIGADOS` tem de ir para o retrato.
4. **O misto.** `layers_differ`/`row_mixed` são sobre a seleção. Um preso sozinho nunca é misto; basta publicá-lo assim.
5. **Custo por quadro.** `monta` compara o `Retrato` inteiro a cada quadro, e campos com `Vec` encarecem-no.
   **Medir antes de escolher a cura** (cena 7, perfil `smoke`). Referência do bloco 10: ~1 ms + ~0,6 ms por cartão
   sem o cache de `MEDIDOS`/`MONTADOS`.
6. **Tectos.** `ph2d-app-vec` tem folga de 490–576 linhas (tecto 93 637; duas contagens). A publicação por objeto
   cabe mal: põe a parte por objeto numa crate que não esteja no tecto. A shell só encolhe, e a escrita mora nela
   (`fase_transform_ops.rs`): a cura é mover a escolha de `sel` para uma função da família, nunca crescer.

## Ordem proposta

1. **Guarda do espalhamento + Material e Ordering** (1 onda): o ícone só entra com a guarda.
2. **Object** (na mesma onda, se o cadeado confirmar o que está na tabela).
3. **Appearance** (1 onda): prova o caminho `press_subject` → `sel = [preso]` num cartão sem pilha.
4. **Fill e Stroke juntos** (1–2 ondas): publicação por objeto, `OPEN` por instância, seletor de cor, estilos.

Cada cartão: `mark_card_pinnable` só no commit em que a escrita já vai ao preso. O teste é o de `o_alfinete_do_inspector.rs`:
prende-se a A, escolhe-se B, edita-se no cartão, e A muda e B não. Com multi-seleção também.

## As três perguntas, medidas (04/11, bloco C1)

- **A opacidade passa pelo `apply_event` do Inspector? Não.** O campo do cartão é `VECTOR_OBJ_OPACITY_NUM`, ligado ao slider `VECTOR_OBJ_OPACITY` (`link_slider_number`): quem responde é o `apply_event` do painel Vector, que pede `ToolPanelEvent(SetValue(VECTOR_OBJ_OPACITY, track))`, sem `entity_bits`; a lista do Blend igual (`pick_object_blend`). O retrato não os alcança, por isso a escolha das formas é `paint_stack::alvos`. Teste: `opacity_blend_and_eye_of_the_pinned_appearance_card_are_of_its_object`.
- **O undo depende da seleção? Não.** `ProjectState::capture` nem recebe a seleção, e os motivos de supressão de `motivo_da_supressao` também não a leem. Corrido na app real, com o ponteiro real (sonda temporária, cena 10, `PH2D_BONE_UNDO_PROBE` como molde, não commitada):

  ```text
  f=180 inicio, B escolhido       undo=0  B: material=0  A: material=1 Multiply
  f=195 Material no preso         undo=1  B: material=0  A: material=3
  f=206 Ctrl+Z                    undo=0  B: material=0  A: material=1
  f=230 multi B+A, Material       undo=1  B: material=0  A: material=4
  f=241 Ctrl+Z                    undo=0  B: material=0  A: material=1
  f=262 multi B+A, Blend          undo=1  B: Normal      A: Darken
  f=273 Ctrl+Z                    undo=0  B: Normal      A: Multiply
  ```

  Um passo por gesto, e o respawn do Ctrl+Z (os bits mudam) não solta o alfinete. Gate: `a_pinned_card_edit_lands_on_its_object_only_and_the_undo_step_restores_it` (shell).
- **O `Retrato` com `CURRENT` custa? Não, nenhuma cura precisa.** Medido em `--profile smoke`, intercalado, o mínimo de 200 blocos, `load ~10`:

  | Aparência no retrato | captura + clone + instala | `==` |
  |---|---|---|
  | nenhuma | 0,68 µs | 0,027 µs |
  | 2 camadas | 0,71 µs | 0,034 µs |
  | 16 camadas (o tecto) | 0,72 µs | 0,066 µs |

  O retrato medido só tem a aparência preenchida: é o custo que ela acrescenta, não o de um retrato cheio.

## Achados que o plano não tinha

- **O commit ao sair do foco ia à seleção.** `note_card_press` corre antes de `depart_focus`: digitar no preso e clicar fora punha o número na seleção, em silêncio. Valia também para o Transform preso (T47) e para os cartões de componente. Cura: `card_event_subject`/`card_edit_subject`.
- **A opção da lista de um cartão cai fora do rect do solto**, e a guarda de T25 só conhecia o `Dropdown` do store, não o `cards::dropdown`. Cura: `dropdown::cobre`.
- **O cartão Object sem título:** o alfinete e o × caíam em cima do olho. Solto, ele tem cabeçalho.
- **O olho lia a caixa da seleção** ao construir o def, fora da troca dos estados presos.
- **Com o Vector atrás do Inspector, a Opacity não era semeada:** a caixa ficava com a da forma anterior (só a foto mostrou). Cura: `semeia_a_opacidade` no `prepare`.
- **Girar e espelhar do Transform preso** iam a `pen.selected()`. Cura: `alvos`, na `fase_node_and_arrange_verbs`.

## O que o bloco C2 mediu (04/11)

O plano previu mais do que o código pede. **O cartão só emite:** amostra e hex, opacidade, olho e «–» de cada linha, «+», Weight, Position, Cap, Join, ⠿ e corrente partida (lido em `cards_pilha.rs` e `cards_corpos.rs`). Tipo de tinta, gradiente, pattern, tracejado, setas, reordenar e abrir camada moram no painel Vector, onde a pressão não é de um preso; `set_kind` e `stroke_paint::apply` não precisam de rota. O pattern não entrou no retrato pela mesma razão: nenhum cartão o lê.

- **Crate nova `ph2d-app-vec-preso`** (295 linhas): `ph2d-app-vec` ficava 309 acima do tecto com a parte por objecto (medido, 93 946 × 93 637); ficou em 93 623. Ela leva a publicação do retrato, o seletor do preso, `apply_no_alvo` e o bloco de aparência que saiu da shell; os verbos e a ponte são os de sempre.
- **A tool é da seleção, e o cartão passa por ela.** Largura, opacidade, Cap, Join e Position chegam por `ToolPanelEvent`. O gesto do preso escreve os campos só em A e a tool volta ao molde no mesmo quadro (`devolve_o_estilo`); sem isso o próximo desenho herdava o estilo de A.
- **`end_card_edit_frame` corria antes do dispatch:** o commit ao sair do foco chegava lá sem sujeito e a largura de A caía na seleção. Passou para depois.
- **O seletor do preso não passa pela tool** (`seletor_na_base`): a leitura contínua dele reescreveria a tool a cada quadro.
- **A amostra de uma camada só abria o seletor se o painel Vector tivesse sido pintado.** O cartão regista-a agora.
- **Um clique fora fecha o seletor** (`pointer_down`); para a decisão do dono valer, o seletor do preso não fecha assim: fecha pelo ×, ou abrindo outra amostra.

## O losango de um campo preso: DEFEITO ABERTO

Lido em `fase_autokey.rs` e `key_mark.rs`, não corrido: o mapa `MARCAS` é um só e calcula-se com a seleção, e `take_key_mark_click` grava em `sel`. Num cartão preso o losango mostra o estado do objecto escolhido, e clicá-lo põe a chave nele. Vale para todos os presos, desde o Transform (T47). Cura: `Retrato` com as marcas de cada preso (`marcas(tl, ctx, &[A], …)`, com `ALVOS` a sobreviver à troca) e o clique com `sel = [preso]` + `Contexto` da forma dele. Não entrou no prazo do C2.
