# HANDOFF DE INTEGRAÇÃO — `line/3DModeling` · O ARCO NO PERFIL (2026-09-16)

**Branch:** `line/3DModeling` · **Base:** `git merge-base main HEAD`
**Ordem do dono:** *«o render do vaso deveria ser mais rápido»* → *«implemente a cura»*.

---

## §1 — O que mudou, em uma frase

Uma quina arredondada deixou de ser **oito segmentos rectos** na fita que a marcha avalia por pixel
e passou a ser **um arco exacto**. O vaso do dono: `77,13 → 25,6 ms` (**`2,95×`**); a cantoneira
`112,78 → 13,2 ms` (**`8,5×`**). Medições, réguas e recusas:
[`docs/Render3d/06_auditoria_do_vaso.md`](../../Render3d/06_auditoria_do_vaso.md).

## §2 — ⚠️ CONTADORES PARTILHADOS QUE SE MEXERAM — conte o DELTA, nunca o literal

| contador | aqui | delta |
|---|---:|---:|
| `PROJECT_SCHEMA` | `131 → 132` | **+1** |
| `ph2d_field::FIELD_DOC_VERSION` | `22 → 23` | **+1** |

⚠️⚠️ **O `FIELD_DOC_VERSION` NÃO está na tripla do `project_schema_tests`** e continua a subir à
mão; o instrumento que avisa é o `the_shape_of_a_saved_profile_is_pinned` da `ph2d-field`
(`90 → 92` bytes). ⚠️ E o `collision-surface.sh` **não o vê** — duas linhas que o subam em paralelo
fundem **mudas**.

⛔ Zero contratos congelados tocados. Zero ADR. Zero pacote externo novo.

## §3 — ⛔⛔ FOUNDATIONAL DE OUTRA LINHA: uma correcção em `ph2d-vec-scene`

`corner_live::fillet_handles` emitia `h = (4/3)·tan(α/4)·s_in` onde a lei do arco pede `·r`, e
`s_in = r·tan(α/2)` — um factor a mais, que vale `1` **só a `90°`**. O artista pedia raio `0,05` e
a `130°` recebia **`0,083`** (`110×` a tolerância de cozimento).

⚠️ **É product-visible em TODA quina arredondada do app fora de `90°`**, não só no modelador.
⚠️ **O gate que devia tê-lo apanhado corre sobre um QUADRADO** — quatro cantos a `90°`, o único
ângulo onde o defeito é invisível. Gate novo: `o_filete_vivo_e_um_arco_em_todo_angulo`, `30°`–`150°`.
⭐ A `ph2d-vec-scene` passa **`501/501`** com a cura dentro.

⇒ **Quem integrar avise a `line/Vector`**: goldens de imagem com quinas arredondadas fora de `90°`
mudam, e **a mudança é a cura**.

## §4 — Os ficheiros

| ficheiro | o quê |
|---|---|
| `ph2d-field/src/profile.rs` | campo `arcs`, porta `with_arcs`, `prim_count`, `arc_count`, `ProfileError::BulgeMismatch` |
| `ph2d-field/src/lib.rs` | `FIELD_DOC_VERSION` 22→23 com o degrau escrito |
| `ph2d-field-profile/src/lib.rs` | `bulge_do_cubico` (reconhece o arco pela GEOMETRIA) + `decomposicao_exacta` |
| `ph2d-field-eval/src/profile.rs` | o emissor da fita: distância por cunha + a correcção da meia-lua no sinal |
| `ph2d-field-eval/src/profile_arc_tests.rs` | **o gate que decide**: o campo com arcos == o campo da polilinha |
| `ph2d-vec-scene/src/corner_live.rs` | a correcção do alçapão (§3) |
| `ph2d-app-field3d/src/device_probes.rs` | as duas sondas da auditoria |
| `shells/desktop/src/project_schema*.rs` | degrau 131→132 + a tripla |

## §5 — ⚠️ SEIS coisas que uma leitura rápida do diff entende ao contrário

1. **`contours()` NÃO encolheu, e não podia.** Ela continua a ser a polilinha densa — **a FIGURA**.
   Os arcos são uma vista **adicional**. A 1.ª tentativa fez o contrário e dois gates que já existiam
   apanharam-na na primeira corrida.
2. **`segment_count()` deixou de ser «o número que manda no custo»** — é o `prim_count()`. Quem citar
   um custo a partir do primeiro cita a polilinha, não o que a marcha avalia.
3. **O reconhecimento é por GEOMETRIA, não por proveniência.** Não há bandeira «isto veio de um
   `corner_radius`», de propósito: um arco desenhado à caneta ou vindo de SVG é o mesmo facto.
4. **Uma quina acima de `~140°` continua tesselada** e isso é correcto: uma cúbica não representa um
   arco tão grande dentro da tolerância, logo o reconhecedor recusa-a. ⛔ Não afrouxe a barra dele.
5. **O `bulge` positivo é «curva para a ESQUERDA de `a→b`»**, que **não** é a convenção do DXF para o
   sentido. O sentido de varrimento **deriva-se dos ângulos** (com `|bulge| < 1` o arco é o menor),
   nunca de uma convenção decorada — a 1.ª régua assumiu-a e leu `0,065` de erro sobre uma
   decomposição correcta.
6. **O cubo da cena `2` é o CONTROLO**: a fita dele é byte-idêntica (`28` ops, `558` B de WGSL). O
   relógio dele mexe `~12 %` entre corridas — *é essa a dispersão da máquina.*

## §6 — ⚠️ QUATRO premissas minhas que a medição derrubou

1. *«as quinas do vaso são arcos de círculo»* — **não eram**, e é o §3.
2. *«posso pôr os bulges paralelos à polilinha»* — a polilinha deixa de ser a figura (§5.1).
3. *«a régua ponto-a-ponto chega»* — ela acusou uma decomposição correcta com um erro **igual em
   todas as amostras**, que é assinatura da régua (ponto→VÉRTICE em vez de ponto→SEGMENTO).
4. *«o custo do arco tem de ser estimado»* — a auditoria estimou `3×`–`5×` e o medido foi `2,95×` no
   vaso e `8,5×` na cantoneira. *A estimativa acertou a ordem e errou a dispersão entre peças.*

## §7 — ⏳ ABERTO

- ⏳ **A especialização por REGIÃO não usa arcos** (o `ProfileIndex` nasce da polilinha densa) —
  ganho por colher, não defeito: é onde o vaso ainda tem `22 ms` de marcha;
- ⏳ **O `select×555`** do vaso por auditar na fita nova;
- ⏳ **Quina `> ~140°`**: a saída publicada é partir a cúbica em duas na emissão — wave da
  `ph2d-vec-scene`.

## §8 — O smoke

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && env PH2D_FIELD_SMOKE=5 cargo run -p ph2d-host-desktop --profile smoke
```
A cena `5` é o vaso torneado. Comparar com `PH2D_FIELD_SMOKE=4` (a cantoneira, o maior ganho).
