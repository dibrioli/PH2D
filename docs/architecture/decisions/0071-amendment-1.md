# ADR-0071-amendment-1 — o 4.º canal de tinta muda de CASA: `per_corner_tint` → `SpriteCornerTint`

**Status:** Accepted (ADR-0164 F1 passo 6, 2026-08-25).
**Amends:** [ADR-0071 — Tint channels are multiplicative](0071-tint-channels-multiplicative.md) — o **conjunto** de canais, não a lei da multiplicação.
**Companion:** [ADR-0070-amendment-8](0070-amendment-8.md) (o corte inteiro: 20 → 13 campos).
**Reference:** [`sprite_corner_tint.rs`](../../../crates/ph2d-ecs/src/sprite_corner_tint.rs) · gate `tint_channel_count` em [`architecture_sprite_inspector_surface.rs`](../../../crates/ph2d-render/tests/architecture_sprite_inspector_surface.rs).

---

## 1. O que NÃO muda

A lei do ADR-0071 fica intacta: os quatro canais são **multiplicativos** e compõem-se na mesma
ordem. `tint × self_tint × per_corner × opacity` continua a ser a conta, e o `RenderInstance`
continua a levar `per_corner_tint: [[f32;4];4]` para o vertex shader — a ABI da GPU **não se
mexe**.

## 2. O que muda: onde o quarto canal MORA

`per_corner_tint` deixa de ser campo da `Sprite` e passa a ser o componente opcional
[`ph2d_ecs::SpriteCornerTint`]. A razão é a do
[ADR-0166](0166-the-inspector-shows-what-the-object-has-and-components-attach-through-one-palette-filtered-by-object-type.md):
um degradê de 4 paradas é uma **escolha do artista** (o `setTint` de quatro cores do Phaser), não
parte do que uma imagem **é** — e enquanto for campo de um componente que todo objeto-imagem tem,
não há como não o mostrar no Inspector.

**A ausência é a identidade:** sem o componente, os quatro cantos são brancos, que multiplica por 1
e não se vê. Um projeto que nunca tocou no degradê é byte-idêntico ao que era.

## 3. ⚠️ O gate continua a ser sobre o CONJUNTO

O `tint_channel_count` toca em cada canal **pelo nome**, para que renomear ou remover seja erro de
compilação — e continua a fazê-lo, com o quarto lido do componente:

```rust
let _tint = s.tint;
let _self_tint = s.self_tint;
let _per_corner_tint = ph2d_ecs::SpriteCornerTint::IDENTITY.0;
let _opacity = s.opacity;
```

*O que o gate afirma é que os quatro existem, não onde cada um mora.* Nenhum canal se perdeu; um
mudou de casa.

⚠️ **A ORDEM DOS CANTOS é o contrato** — `[TopLeft, TopRight, BottomLeft, BottomRight]`, a mesma que
a `RenderInstance.per_corner_tint` sobe para o shader. Trocá-la espelharia o degradê de toda cena já
autorada, **em silêncio**.

## 4. Alternativas recusadas

| alternativa | porquê não |
|---|---|
| Cortar também `self_tint` | O Godot põe `modulate` e `self_modulate` **sempre** no inspector; o par com `tint` é a base, não uma escolha. |
| Cortar `tint_fill` junto | É um **modo do tint** (a cor substitui em vez de multiplicar), não um canal nem uma feature separada. Um componente de um bool só é pior ergonomia que o campo. |
| Deixar o gate a ler o campo e acrescentar um quinto para o componente | Seriam duas respostas a *"quais são os canais?"* — e a que envelhece é a que ninguém relê. |
