# ADR-0070-amendment-9 — `RenderInstance` CPU-tail `sub_order` (a sub-ordem DENTRO de uma fatia de `z_order`)

**Status:** Accepted (doc 89 folha 17, 2026-08-25) — pendente de smoke do Enio (cena `=98`).

> ⚠️ **RENUMERADO na integração de 2026-08-26:** esta emenda nasceu `-8` e a `line/components`
> escreveu um `-8` **diferente** (o corte da `Sprite` v4→v5), que aterrou primeiro. O número de
> uma emenda é um **CONTADOR** — o certo não estava em nenhum dos dois lados.
> ⛔ A colisão é de **mesmo nome de ficheiro**, e o modo de falha é mudo: sem renumerar, as
> citações desta linha apontariam, sem erro nenhum, para a ADR da `Sprite`.
> ⚠️ E o escopo da renumeração é *o que esta linha ESCREVEU*, não *os ficheiros que ela tocou*:
> a 1.ª tentativa trocou também citações que a `components` pusera nos mesmos ficheiros.

**Amends:** [ADR-0070 — Sprite schema v4 (`SpriteVersioned` + `RenderInstance` ABI)](0070-sprite-schema-v4.md) §1.7 ABI.
**Slot rationale:** `-5` sampling CPU-tail; `-6` uv_xform GPU @location(15); `-7` clip_group + clip_meta CPU-tail. Este é o próximo slot livre, e é de novo **CPU-only** ⇒ o vertex layout (164 B / 12 attrs) não se mexe.
**Reference:** [`crates/ph2d-render/src/sprite/instance.rs`](../../../crates/ph2d-render/src/sprite/instance.rs) · [`crates/ph2d-render/src/sprite_collect.rs`](../../../crates/ph2d-render/src/sprite_collect.rs) (`sort_render_order`) · [`crates/ph2d-render/tests/architecture_sprite_inspector_surface.rs`](../../../crates/ph2d-render/tests/architecture_sprite_inspector_surface.rs) · [`crates/ph2d-gpu-cook/src/lower.rs`](../../../crates/ph2d-gpu-cook/src/lower.rs) (`INSTANCE_WORDS`).

---

## 1. Contexto — a pergunta que o `z_order` não sabe responder

A ordem canónica de desenho é
`sort_by_key(clip_anchor, z_order, texture_id, sampling)`, estável. O `z_order` é
o **rank de DFS** que o extract carimba, um por objecto da hierarquia — a cena
inteira ordena-se por ele, e o `texture_id` a seguir é o que agrupa instâncias no
mesmo run de desenho.

**Um sink de Motion não tem lugar na hierarquia.** Ele emite `n` linhas com o
MESMO `z_order` (`0`), e a chave seguinte é o `texture_id`. Com mídia **MISTA** —
dois `source.object` de texturas diferentes no mesmo grafo — as linhas
**reagrupam por textura** e a ordem que um `motion.sort` a montante autorou é
**derrotada**. É a célula *SORT / ordem de desenho* da folha 17 do doc 89 (o
`SortMode` do Sprite Renderer do Niagara, a ordem por camada da Cavalry), medida
e escrita como *«PARCIAL — e a fronteira é exata»*.

### 1.1 As duas saídas óbvias foram medidas e são as duas ERRADAS

| tentativa | o que acontece |
|---|---|
| dar às linhas `z_order = i` | o bloco **espalha-se pelo espaço de ranks da CENA** — as partículas passam a interpenetrar as sprites da hierarquia, e o grafo deixa de estar onde estava |
| dar-lhes `z_order = BASE + i` com `BASE` acima de toda a cena | o grafo **salta para a frente de tudo** ao ligar um knob de *ordenação*, que não é o que o artista pediu |
| tirar o `texture_id` da chave | a cena INTEIRA perde o agrupamento por textura ⇒ regressão de draw calls para todos, para servir um caso |

*A grandeza que faltava não era «mais fundo», era «mais à frente **dentro do
mesmo fundo**».*

## 2. Decisão

`RenderInstance` cresce **um campo CPU-only no tail**, depois de `clip_meta`:

```rust
pub sub_order: u32,   // 0 para tudo o que a cena extrai
```

e a chave de ordenação passa a ser

```
(clip_anchor, z_order, sub_order, texture_id, sampling)
```

`sub_order` entra **entre** o `z_order` e o `texture_id`. Toda a cena o deixa em
`0`, e um bloco inteiro a `0` desempata exactamente como desempatava ⇒ **a ordem
de toda sprite extraída é byte-idêntica**.

O único produtor que o escreve é o lowering de Motion, quando o sink pede
(`SinkStyle::stream_order`, o param `sort` do `motion.output`): `sub_order = i`,
o índice na FILEIRA — não no buffer, porque vários sinks compõem no mesmo `out` e
um contador global faria o 2.º sink desenhar sempre por cima do 1.º.

### 2.1 O preço, e por que ele é o próprio pedido

Honrar a ordem de um stream que alterna texturas A,B,A,B obriga a **um run de
desenho por linha** (`compute_runs` varre corridas CONSECUTIVAS de
`(texture_id, sampling, clip, blend)`). Quem liga o `Stream` está a dizer que a
ordem importa mais que o batch. `Texture` — o default — continua a agrupar.

## 3. ABI

| | antes | depois |
|---|---|---|
| `size_of::<RenderInstance>()` | 184 B | **188 B** |
| campos | 16 | **17** |
| vertex layout | 164 B / 12 attrs | **inalterado** |
| `INSTANCE_WORDS` (gpu-cook) | 46 | **47** |

O gate `vertex_attr_offsets_match_struct` continua a valer sem uma linha mexida:
o campo novo fica **depois** de todo campo lido pela GPU, que é a invariante que
ele pina. Os dois gates de contagem/tamanho
(`render_instance_field_count_capped`, `render_instance_pod_size_capped`) e o
`instance_words_matches_render_instance_size` do `ph2d-gpu-cook` movem-se em
lockstep, por destruturação exaustiva.

### 3.1 ⚠️ Achado colateral: o bench estava vermelho havia uma amendment inteira

`benches/sprites_upload_144b_vs_72b.rs` pina o stride para o seu próprio
argumento fazer sentido, e a linha dizia **`176`** — o tamanho de ANTES da
amendment-7. Ou seja o bench **abortava na primeira instrução desde 2026-05-30**,
e ninguém notou: um bench não corre no `nextest` nem no CI. *Uma cerca que existe
para tornar uma premissa velha visível só é visível se alguém a correr.* Corrigido
para `188` nesta amendment, com a nota ao lado.

## 4. Alternativas rejeitadas

- **`z_order` como `u64`** (rank << 32 | sub): custa 4 B na mesma e obriga toda a
  cadeia de extract a mudar de tipo, para exprimir a MESMA coisa que um campo
  separado exprime sem tocar em ninguém.
- **Abusar do `clip_group`** (o `clip_anchor` é a chave primária, e `clip_group =
  i+1` daria a ordem das linhas): põe as instâncias de Motion no caminho do passe
  de stencil, que as batalha por corridas contíguas e as pinta contra um stencil
  não marcado — elas **desapareciam**.
- **Ordenar o `extra` à parte e concatenar**: muda onde o bloco de Motion fica em
  relação à cena (hoje ele partilha a fatia `0`), que é precisamente o que esta
  amendment existe para NÃO mudar.

## 5. Consequências

- Um campo novo a manter em ~15 sítios de construção (todos passam `0`).
- O `ph2d-gpu-cook` escreve mais uma palavra por instância (word 46).
- **Ganho fora do Motion:** qualquer produtor futuro que emita N instâncias numa
  fatia de z (uma tira de Flip assada, um passe de partículas de UI) tem agora
  onde dizer a ordem delas sem negociar com o rank da hierarquia.
