---
name: feedback-not-naming-a-thing-in-an-absolute-list-is-commanding-it-off
description: "Numa lista APLICADA POR VARREDURA EXAUSTIVA, tirar um item não é «deixar de o comandar» — é comandá-lo desligado; as duas leituras são a mesma linha de código"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: af27d1c2-3a56-4abe-9acd-e2c91caf58f0
  modified: 2026-08-31T18:59:40.025Z
---

Quando concluo *«este campo não é do X, então o X não o deve nomear»*, tenho de perguntar **como a
lista é aplicada**. Se o consumidor varre a população inteira e escreve todos —

```rust
for p in reg.panels() {
    visible.insert(p.id, spec.open.contains(&p.id));   // ⚠️ o `else` é implícito
}
```

— então **não nomear é nomear desligado**, e a minha intenção («não comando isto») não existe no
código.

**Medido na `line/UIUX`, 2026-08-31 (entrega 27→28):** tirei o `inspector` de todas as listas de
abertos dos layouts com o argumento — correcto — de que ele tem **dois escritores** e o layout
perde sempre a corrida contra as pontes. Resultado: ele passou a **fechar em toda a parte**, e o
Enio reportou-o no smoke seguinte (*«em animate o inspector está sendo escondido»*).

> *«O X não o comanda» e «o X comanda-o fechado» leem-se iguais num campo AUSENTE, e só a segunda é
> o que acontece.*

⭐ **As duas saídas, e a escolha é do consumidor:**

| se quero mesmo *«não comando isto»* | se a lista é exaustiva |
|---|---|
| o consumidor tem de **saltar** a população que não é minha (um censo explícito) | então tenho de **nomear o valor certo** — e derivá-lo de uma regra, não escolhê-lo item a item |

Na cura, a regra ficou derivável dos dois lados: *o layout nomeia o inspector **exactamente**
quando a ferramenta dele não o substitui* — censo lido dos ficheiros das pontes.

**Why:** um `contains()` sobre uma varredura completa transforma toda ausência num comando. É o
mesmo mecanismo do [[feedback_a_collapsed_field_does_not_go_neutral_it_takes_over]], um nível
acima: ali um valor colapsado manda; aqui é a ausência de linha.

**How to apply:** antes de apagar um item de uma tabela declarativa, **leia o laço que a aplica**.
Se ele itera sobre a população e não sobre a lista, escreva o gate que afirma o valor esperado dos
itens **fora** da lista — senão o defeito só aparece no smoke.

Relacionadas: [[feedback_a_declaration_with_a_default_is_decoration_until_something_reads_it]] ·
[[feedback_a_new_feature_can_empty_an_existing_gates_population]] ·
[[feedback_a_collapsed_field_does_not_go_neutral_it_takes_over]]
