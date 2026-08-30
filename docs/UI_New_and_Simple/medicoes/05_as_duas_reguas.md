# As duas réguas — px/metros e graus/radianos, medido (2026-08-30)

> **Enio, 2026-08-30:** *"Devemos ter ambas as opções no app (px e metros, graus e radianos)."*
>
> ⭐⭐ **Metade já ship-a, e a outra metade está a UM campo de distância** — o tipo, o widget e o
> parser já existem. A medição diz exactamente o que falta.

---

## §1 — A lei que sustenta as duas: **unidade de LEITURA ≠ unidade de ARMAZENAMENTO**

O que se guarda é canónico e não muda:

| grandeza | armazenamento | fonte |
|---|---|---|
| posição | **metros** | `ph2d-ecs/src/transform.rs:55` — *"translation (meters)"* |
| ângulo | **radianos** | idem — *"rotation (radians, CCW from +X)"* |

⇒ **«ter as duas opções» é sobre o que o painel MOSTRA e ACEITA**, nunca sobre o que o documento
grava. ⛔ Guardar em graus poria uma conversão dentro de cada função trigonométrica e acumularia
erro de arredondamento; guardar em pixels ligaria o documento à resolução da arte.

⭐ **E esta lei já está implementada** — a metade de comprimento é a prova viva.

---

## §2 — ✅ px ↔ metros: **feito, e com a arquitectura certa**

`ph2d_editor::project::DisplayUnit`:

```rust
pub enum DisplayUnit { Meters, Pixels }
```

| o que | onde | estado |
|---|---|---|
| o enum | `ph2d-editor-core/src/project.rs` | ✅ |
| o menu | *Settings → Display unit* (cascata) — `chrome/settings_unit.rs` | ✅ |
| a ponte | `ProjectSettings::pixels_per_meter` (+ *Settings → Pixels per meter*) | ✅ |
| **grava no ficheiro** | `SavedSettings` em `shells/desktop/src/project_settings.rs` | ✅ |
| **consumidores** | **62 ficheiros** leem `display_unit` | ✅ |

⭐⭐ **E três decisões de desenho já foram tomadas ali, com o motivo escrito** — a metade que falta
herda-as em vez de as re-litigar:

1. **Fica FORA do `ProjectState`** — *"um Ctrl+Z do canvas não deve rebobinar a escala do mundo nem
   a unidade de leitura"*. ⚠️ O preço é declarado: trocar a unidade **não entra no undo**.
2. **Viaja no ficheiro** — a razão está escrita: *"um projeto de pixel art afinado em 32 px/m
   reabria em 100"*. ⛔ *São knobs que ESQUECEM.*
3. **O ficheiro tem tipo PRÓPRIO**, espelho do de runtime, com gate de round-trip que compara os
   `ProjectSettings` **inteiros** por `PartialEq` — *"um campo novo que o espelho não carregue faz
   o teste falhar, em vez de deixar de persistir em silêncio"*.

⇒ ⭐ **O gate do item 3 já defende o campo que ainda não existe.** Acrescentar a unidade de ângulo
sem a pôr no espelho **reprova**, em vez de a perder em silêncio.

---

## §3 — ⏳ graus ↔ radianos: o vocabulário existe, **falta quem escolha**

`ph2d-editor-core/src/widget/numeric_input_with_unit.rs`:

```rust
pub enum Unit { Px, Meters, Degrees, Radians, Percent }

pub const fn suffix(self) -> &'static str {  // "px" · "m" · "deg" · "rad" · "%"
pub fn parse_suffix(s: &str) -> Option<Unit>  // casamento mais-longo-primeiro
```

⇒ **o widget já mostra e já lê as duas.** Há teste: `parse("2.25rad") == Some((2.25, Unit::Radians))`.

### ⛔ Mas o censo diz que ninguém a escolhe

| | usos no repo |
|---|---:|
| `Unit::Degrees` | **14** |
| `Unit::Radians` | **5** — ⛔ **e as cinco estão dentro do próprio ficheiro dele** (a declaração, o rótulo, e três em testes) |

⇒ **`Unit::Radians` tem zero consumidores fora de casa.** Nada no app alguma vez **mostra** um
ângulo em radianos.

⚠️ **E não é um id órfão nem um knob morto** — é uma terceira coisa, e vale nomeá-la porque as
curas diferem (`CLAUDE.md` §5.0): a entrada **já aceita** `rad` (o parser está ligado), e é a
**saída** que nunca o produz. *Meio caminho ligado: o app lê radianos e nunca os escreve.*

---

## §4 — ⇒ O que falta é **um campo, um menu, e uma porta**

Simétrico ao que já existe, item a item:

| # | falta | o irmão pronto que serve de molde |
|---|---|---|
| 1 | `DisplayAngle { Degrees, Radians }` em `ProjectSettings` | `DisplayUnit { Meters, Pixels }` |
| 2 | *Settings → Angle unit*, cascata | `chrome/settings_unit.rs` (34 linhas) |
| 3 | o campo no espelho do ficheiro | `SavedSettings` — ⭐ e o gate de round-trip **já obriga** |
| 4 | os sítios que hoje fixam `Unit::Degrees` passarem a **perguntar** | os 62 consumidores de `display_unit` |

⚠️ **O item 4 é o trabalho real, e é o mesmo padrão dos 62.** Um `Unit::Degrees` escrito à mão é
uma decisão de leitura tomada no sítio errado — exactamente como um `px` fixo seria.

⛔ **E há uma armadilha nomeada:** nem todo ângulo do app é do artista. O `skew_x`/`skew_y` do
`Transform` também são radianos, e a **fase** de um oscilador, e o ângulo de um gradiente. ⇒ a
troca vale para os ângulos **que o artista autora**, e a lista tem de ser explícita — senão a
unidade escorrega para leituras onde ela não significa nada.

---

## §5 — ⏳ Não medido

- **Quantos dos 14 `Unit::Degrees` são autoria do artista** e quantos são leitura derivada
  (readouts de gizmo, diagnósticos). Só os primeiros mudam de unidade.
- **Se `pixels_per_meter` deve ganhar um irmão para o 3D.** A cena é métrica por inteiro (§1), logo
  provavelmente **não** — mas a escala uniforme do `Xform` (`pub scale: f32`) ainda tem de virar
  três números para a D9, e essa mudança toca o mesmo formato.
