# 55 — Uma régua não pode ser função do que ela mede (sliders em bilhões)

> Nota-ADR da linha `line/motion-value`. Reportado pelo Enio no smoke de 2026-07-12:
> *"Quase todos os sliders do motion estão bugados e podem chegar a valores de bilhões.
> E não arrastam linearmente."*

## O sintoma, e por que os dois sintomas eram um só

Dois defeitos foram relatados — valores absurdos e arraste não-linear — mas há **uma** causa,
e ela explica os dois de uma vez.

O bridge de params constrói uma linha de slider por param. Quando o param não tinha um
`ParamUiHint` declarado, ele caía num fallback:

```rust
None => ScalarRow { min: 0.0, max: (value * 4.0).max(10.0), .. }
```

O **máximo da régua era o valor vezes quatro**. Isto é, a escala contra a qual o valor é
lido derivava do próprio valor. É realimentação positiva, e dá pra resolver o ponto fixo:

- o snapshot é reconstruído **todo quadro** a partir do doc;
- arrastar até a fração `t` da trilha emite `v' = t · max = t · 4v`;
- portanto `v' = 4t · v`. O ponto fixo é **`t = 0,25`**.

Acima de um quarto da trilha o valor **multiplica a cada quadro** — chega aos bilhões em
cerca de um segundo. Abaixo de um quarto ele **colapsa a zero**. Em lugar nenhum a posição
do botão corresponde a um número estável: é isso que "não arrasta linearmente" quer dizer
visto de fora. Não era um slider com dois bugs; era um slider que não era um slider.

## Por que atingiu "quase todos"

Medição antes de teoria: 27 de 306 params sem hint. Mas 12 desses são canais de cor
dobrados num swatch (`consumed`), que nunca viram linha. Os **15 restantes** eram
`sim.spawn.*`, `sim.step.damping`, `sim.lifetime.*`, `sim.collide.*` e `debug.wave.gain` —
ou seja, **os nós que esta linha criou**. Eu registrei o `NodeUiManifest` (o nome do card)
de cada um e **nunca chamei `register_param_ui`**.

Do ponto de vista do Enio isso é literalmente "quase todos": ele está fazendo smoke do
**grafo da chuva**, e todo slider que a chuva expõe é um `sim.*`.

Um caso merece destaque: **`sim.collide.shape` é um enum** (Floor · Disc · Bowl) que estava
sendo pintado como slider de float com régua fugidia. O artista tinha que decodificar "2"
para chegar em Bowl — e a régua fugia enquanto ele tentava.

## O conserto (as duas metades)

**1. O fallback ficou inerte.** A régua passa a sair do `default` do manifest, que é uma
**constante**, nunca do valor vivo. `contain` ainda alarga a régua para conter um valor
fora dos limites, o que é idempotente: alargar para um valor que a régua já contém é no-op,
então **um arraste dentro da régua nunca a move**. Um backstop tem que ser desarmado, não
armado.

**2. Os 15 params ganharam hints de verdade** — e um hint é onde o param diz o que ele
**significa**, não um preenchimento burocrático:

| Nó | Decisão |
|---|---|
| `sim.collide.shape` | **Enum** nomeado: Floor · Disc · Bowl |
| `sim.spawn.scatter` | **Toggle** (o nó lê `scatter: bool`) |
| `sim.spawn.seed`, `sim.lifetime.seed` | **Seed** (caixa + re-roll: o artista quer *outra* semente, não uma *maior*) |
| `sim.step.damping`, `restitution`, `friction`, `variance` | Slider 0..1 |
| `height`, `center_x`, `center_y`, `radius` | Slider em unidades de mundo (−10..10, passo 0,1 — a convenção que `force.vortex` já usava) |
| `sim.spawn.rate` | Slider 0..60 (nascimentos/s) |
| `sim.lifetime.life` | Slider 0..20 s |

## Os gates (o que impede a volta)

Três guards novos em `motion_bridge_range_tests.rs`, provados por mutação (reintroduzi o
`value * 4` + tirei os hints do `sim.collide`: os dois primeiros ficaram **vermelhos**, com
a mensagem `sim.collide.shape: dragging to 0.5 of the track moved the range from (0.0, 10.0)
to (0.0, 20.0)` — a escada até o bilhão, medida):

1. **`a_drag_inside_the_range_never_moves_the_range`** — põe no param um valor *dentro* da
   régua (exatamente o que um arraste faz, em 7 pontos da trilha, incluindo as pontas) e
   exige a régua de volta **idêntica**. Mata a classe "escala função do valor".
2. **`every_scalar_row_comes_from_a_declared_hint`** — nenhum param chega ao painel com
   régua *adivinhada*. Teria pego este bug no instante em que criei as crates.
3. **`every_param_default_is_inside_its_declared_range`** — fecha a porta do `contain`
   alargar já no primeiro quadro (uma régua que persegue o valor, versão silenciosa do
   mesmo defeito).

## A lição que custou a semana (de novo)

O gate que **deveria** ter pego isto já existia: `every_row_range_contains_its_value_for_every_node_and_param`.
Ele estava **verde**. E estava verde porque tinha, dentro dele:

```rust
.filter(|n| n.starts_with("motion."))
```

Os nós novos são `sim.*`, `value.*`, `force.*`, `pulse.*`. **O gate excluía exatamente a
família que quebrou** — enquanto o nome dele prometia "every node and param". Um filtro
dentro de um gate é um buraco nele, e o nome do teste vira a garantia falsa que ninguém
relê. O filtro foi removido (a varredura agora é sobre os 88 tipos; a asserção de tamanho
subiu de `>= 10` para `>= 80`, senão o filtro volta por descuido e ninguém percebe).

Correlato de [[feedback_tool_unit_green_integration_dead]] e [[feedback_painted_is_not_populated_paint_gate]]:
não basta o gate existir e estar verde — é preciso perguntar **sobre o que** ele está verde.
