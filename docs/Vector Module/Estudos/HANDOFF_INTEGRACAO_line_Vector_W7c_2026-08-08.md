# Handoff de integração — `line/Vector` · **W7c: o seletor de CURVA**

*2026-08-08 · 3 commits: `67792bcd7` o catálogo · `352804724` o seletor · `b53f7eea2` a mola*

> **Estado:** a wave FECHOU. ⚠️ **PENDENTE DE SMOKE** — e de ordem explícita do Enio para
> integrar. A linha não integra nem pusha sozinha.

---

## §1 — O que esta wave entrega, numa frase

**A curva da transição deixa de ser uma constante escondida e passa a ser uma escolha do
artista** — e o campo que a guarda já viajava no arquivo desde o v56, sem nenhum gesto capaz de
o escrever.

---

## §2 — ⭐ O achado: uma capacidade sem PORTA

`HostStates.easing` é `Serialize`/`Deserialize`, viaja dentro do `ProjectState`, é lido pela
`Machine` em toda transição, tem setter público (`StateSets::set_easing`) e testes próprios.

**O único chamador de `set_easing` no repo inteiro era uma fixture `#[cfg(test)]`.**

⇒ todo projeto já salvo carrega `Cubic Out`, não porque alguém o escolheu, mas porque nada no
produto conseguia escrever outra coisa.

⚠️ **E a porta tinha um caractere de largura.** O `publish` do painel já buscava a curva e
deitava-a fora:

```rust
let (duration, _easing) = states.timing(h);   // <- o sublinhado era a wave inteira
```

É a mesma classe que este plano já pagou uma vez, e por escrito: *"`ObjectPose::geometry`
existia, a `Transition` sabia casá-la, o `install` sabia escrevê-la — e nenhum produtor
preenchia o campo. **Uma capacidade sem PORTA passa em todos os gates**, porque eles leem quem
CONSOME."*

---

## §3 — ⚠️ O plano estava errado nos dois números, e a correção mudou a wave

O item dizia: *"11 famílias × 3 modos = **33 combinações**, e o `ph2d-anim` **não dá nome a
nenhuma**; o catálogo precisa de nomes antes do knob"*.

**(a) São 31 curvas distintas, não 33.** O `eval` devolve `u` **antes** de olhar para o modo
quando a família é `Linear` ⇒ `Linear In`, `Linear Out` e `Linear In-Out` são a mesma curva
escrita três vezes. Não é leitura de doc-comment: o gate
`the_mode_is_dead_exactly_where_the_catalogue_says_it_is` avalia as três curvas de **cada**
família sobre 101 amostras e exige que coincidam exatamente onde `uses_mode()` diz `false` e
difiram onde diz `true`. Medido: **`Linear` é a única**.

**(b) O catálogo JÁ TINHA nomes** — shipados e à frente do artista desde o menu de easing da
timeline (`TIMELINE_EASE_MENU`: dez famílias, com `Linear` como row de topo **exatamente pela
mesma razão**). O repo já tinha tomado esta decisão e não a tinha escrito.

⇒ **O que faltava não era vocabulário. Era DONO.**

---

## §4 — A fronteira que decidiu o desenho (e que não foi movida)

`editor-core` **não pode** depender de `ph2d-anim`, e está escrito no cabeçalho do
`timeline_presets`:

> *"editor-core paints the segment preset menu and parks the clicked row as an opaque
> `(item, mode)` — **it cannot depend on `ph2d-anim` and so never names an easing**."*

⇒ os literais do menu da timeline são **consequência deliberada** dessa fronteira, e "consertá-los"
adicionando a aresta seria furar uma cerca de Chesterton.

**O que foi feito:** o nome mora no enum (`EasingFamily::label()` / `EasingMode::label()`), o
**painel** ganha a aresta (`ph2d-panel-vector` → `ph2d-anim`, o precedente exato do
`ph2d-symmetry` escrito no próprio `Cargo.toml` dele), e o **gate cruzado mora na SHELL** — o
único sítio que vê a tabela de editor-core **e** o `preset_for` que a traduz.

⚠️ O gate compara **famílias**, não modos: as rows de modo da cascata são decoradas
(`"Ease In ▶"`) porque ali são rows de um menu. Exigir que a decoração de um consumidor fosse o
nome canónico seria fazer o catálogo servir a um layout.

---

## §5 — O que o artista vê

Abaixo de **Duration**, na seção States:

| Fileira | O quê |
|---|---|
| **Curve** | onze chips, de `EasingFamily::ALL` + `label()` |
| **Direction** | três chips (In / Out / In-Out) — **só quando a família usa o modo** |

- Os chips saem de `ALL`: uma família nova entra no vocabulário e ganha chip, id, registo e rota
  **sozinha**. Uma tabela paralela no painel nasceria incompleta no dia seguinte.
- O `segmented` já delega ao grupo adaptativo que **quebra** (o precedente das dez ferramentas do
  impasto) ⇒ onze chips refluem sem widget novo.
- ⚠️ **`Linear` esconde a fileira Direction.** Oferecê-la seriam três botões a desenhar a mesma
  coisa — a row-de-menu-morta que este repo mantém *uma tabela por menu* para prevenir.
- ⚠️ **Passar por `Linear` não apaga a direção escolhida.** Seria tentador normalizar o modo ao
  escolhê-la; isso perderia uma decisão do artista para arrumar um byte que nenhum `eval` lê.

---

## §6 — ⚠️ A MOLA voltou à mesa (CLAUDE.md §0)

A M6 fechou com *"o solver não se constrói"*, e o argumento era **condicional**: *a pergunta de
verdade é a interrupção, e o **default** passa (1,34×)*. Os dois regimes que mordem foram
arquivados como **inalcançáveis** — porque o seletor não existia.

**Ele existe agora.** Re-medido (`measure_spring`), os números reproduzem-se idênticos:

| curva | \|v\| antes | \|v\| depois | razão |
|---|---|---|---|
| `Cubic Out` (o default) | 9,80 | 13,14 | **1,34×** |
| `Cubic InOut` | 7,20 | 0,00 | **0,00×** |
| `Elastic Out` | 5,78 | 40,55 | **7,02×** |

⇒ **o veredito muda de NATUREZA, não de valor:** a mola deixou de ser dispensável por *ausência
de regime* e passou a ser **decisão de produto**. `Cubic InOut` interrompido **para e recomeça**,
e está a um clique.

⚠️ **Não é defeito desta máquina:** a POSE é contínua (a lei (a) da `Machine` — *uma transição
parte da pose VIVA*); o que salta é a **velocidade**, e é precisamente isso que uma mola carrega.
Toda animação por CURVA tem velocidade zero no `t = 0` de uma família `InOut` — CSS transitions e
o modo *tween* do Framer partilham o artefacto.

⚠️ **E ela não é follow-up:** uma mola não tem *duração* nem *curva* (tem rigidez e
amortecimento), então o slider de duração **e este próprio seletor** deixariam de significar o
que significam. Wave própria, e é do Enio.

---

## §7 — Colisão: o que esta wave toca

| Eixo | Valor |
|---|---|
| `PROJECT_SCHEMA` | ⚠️ **INTOCADO** — `git diff` sobre `project.rs` sai **vazio**. O campo já estava no arquivo desde o v56 |
| `VEC_SCENE_SCHEMA` | intocado |
| ADR | **nenhum** ⇒ fora de toda disputa de número |
| Contrato congelado | **3/3 + 4/4 verdes** (rodados, não auto-relatados) |
| `Cargo.toml` | **um**: a aresta `ph2d-panel-vector` → `ph2d-anim` |
| Dep externa | **nenhuma** — o `Cargo.lock` ganha **uma linha de aresta**, zero pacotes |
| Registro `ph2d-ecs` | intocado |
| ids novos | `MAX_EASING_FAMILIES`, `MAX_EASING_MODES`, `vector_easing_family_id`, `vector_easing_mode_id` (derivados em runtime, sem const de colisão) |
| i18n | 2 chaves (`panel.vector.states.curve`, `…curve.mode`) |

**Ponto de merge sensível:** `crates/ph2d-anim/src/easing.rs` e
`shells/desktop/src/render_loop/mod.rs`. O primeiro é aditivo (dois métodos novos); o segundo
ganha um `pending_*` e um bloco de honra ao lado dos irmãos — o mesmo sítio que várias linhas
tocam. Nada é removido.

---

## §8 — Gates e mutações

**8 mutações, 8 sangram.**

| # | Mutação | Sangra |
|---|---|---|
| M1 | `uses_mode()` sempre `true` | `Linear`: as três coincidem |
| M2 | `uses_mode()` sempre `false` | `Quad`: spread 5,000e-1 |
| M3 | renomear `Expo` no catálogo | `"Expo"` × `"Exponential"` — duas listas |
| M4 | o pick não chega ao documento | **só o arch-gate** (22 unidade + 13 seam ficam verdes) |
| M5 | fileira da direção pintada sempre | `Linear` |
| M6 | sem `register` no populate | pintado e morto sob o rato |
| M7 | o `publish` ignora a curva | o defeito **original** |
| M8 | o pick de família leva a direção | 2 gates |

⚠️ **M4 achou um defeito no meu próprio gate.** A âncora era `set_easing` — e o **comentário que
escrevi oito linhas acima do bloco** contém a palavra, então o gate passava sob a mutação. *Um
oráculo que casa com a documentação de si mesmo não está a olhar para o produto* (a cicatriz do
`stamps: media`, `line/Painter`). A âncora passou a ser a **chamada** (`ui_states.set_easing(`).

⚠️ **Não há gate do "chip ACESO", e a ausência é medida.** A seleção de um chip segmentado é
argumento de **pintura** (`paint_segmented_group_adaptive` recebe `selected`; o store fica
read-only), então o testkit não a observa — um gate escrito à mesma não poderia falhar pelo
motivo que alegasse. O que ele provaria — *o painel LÊ a curva publicada* — já está provado pela
fileira que desaparece no `Linear`; e a metade que sobra (compor o pick sobre a curva do
documento) é gateada na shell, onde ela mora.

---

## §9 — Como smokar

```
env PH2D_BUILD_SMOKE=61 cargo run -p ph2d-host-desktop --release
```

O roteiro impresso ganhou os passos **21-24**. Em resumo:

1. Abaixo de **Duration** há **Curve** e **Direction**; o aceso é `Cubic` + `Out`.
2. Escolha **Elastic**, entre em Preview, passe o rato pelo Play — a forma passa do alvo e volta.
   **Bounce** quica. É a mesma transição; mudou a curva.
3. ⚠️ **Escolha `Linear`: a fileira Direction desaparece.** Volte a **Quad** — ela reaparece **com
   a direção que você tinha**.
4. ⚠️ **O que esperar de mau, porque está medido e a decisão é sua:** com **In-Out**, interromper
   um hover no meio faz a forma **parar e recomeçar**; com **Elastic**, ela arranca a 7×. A pose
   nunca salta — só a velocidade.

---

## §10 — Aberto, nomeado

- **A MOLA** — §6. Decisão de produto, com os números na mesa.
- **A HIERARQUIA** (`Machine` é PLANA) — segue por construir; é o outro item aberto do W7 e o que
  o §5 da fila de tokens nomeia. Um menu que abre com sub-estados é ela.
- **W8a** — bloqueado por ausência (`ph2d-runtime` não existe).
- **W2a** (`wrap_width`) — custa um bump global de `PROJECT_SCHEMA`, que se **CONTA** contra o
  `main` do dia.
- O `Disabled` continua sem gatilho (é um fato do DOCUMENTO, não do rato).
