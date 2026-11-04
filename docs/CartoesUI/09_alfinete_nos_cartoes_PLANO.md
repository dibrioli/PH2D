# 09 — O alfinete nos cartões que ainda não o têm: plano com o tamanho medido (03/11/2026)

Leitor: a LLM que construir isto. **Não construído.** Ordem do dono (03/11, por `ph2d-58`): só depois da CI.
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

## Por confirmar no começo (não verificado nesta medição)

- Se a opacidade `VECTOR_OBJ_OPACITY_NUM` passa pelo `apply_event` do Inspector (`event.rs:42` lê o `press_subject`), ou só pelo painel Vector.
- Se o undo dos aplicadores do Inspector depende da seleção. No caminho vetorial o registo é por diff do documento (`fase_transform_ops.rs:68-71`): deve servir, mas não foi corrido.
- O `Retrato` guarda `CURRENT` sem copiar demais? Depende da medição do ponto 5.

## Decisão de produto em aberto para o dono

Um Fill preso a A com o seletor de cor aberto: escolher outra forma fecha o seletor ou continua em A? O Transform preso
não tem essa pergunta. Recomendação: continua em A, porque é para isso que o alfinete existe. Perguntar quando a onda 4 começar.
