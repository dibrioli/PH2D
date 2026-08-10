---
titulo: "Handoff de integração — line/sculpt3d, W1+W2+W3 (a doação chega à tinta)"
tags: [modulo/3d, tipo/handoff, assunto/integracao, status/smoke-aprovado]
status: smoke-aprovado
modulo: 3D
atualizado: 2026-08-01
resumo: "A linha está FECHADA e smokada. O rig de luz tem um dono, a escultura acende por ele, a malha DOA a normal, e a tinta chapada sai acesa pela forma."
relacionados: ["[[05.2-Doacao-de-sombreamento-para-2D]]", "[[06.1-Waves-riscos-e-alvos]]", "[[02.3-Modulo-removivel-e-mapa-de-crates]]"]
---

# Handoff de integração — `line/sculpt3d`

> **SMOKE APROVADO pelo Enio** (2026-08-01, `PH2D_SCULPT3D_SMOKE=2`): *"Smoke OK"*. A linha está
> fechada e aguarda ordem de integração. **Ela não integra nem faz ship.**

## 1. Identidade

| | |
|---|---|
| **Branch** | `line/sculpt3d` |
| **HEAD** | `8ae5e81e5` |
| **Base (merge-base com main)** | `98eb502a2` |
| **Commits** | **23** |
| **Rebase** | ⚠️ **Não é preciso** — `main` já está contido na linha; `--ff-only` funciona como está. |

A linha cobre **três waves**: W1 (a malha), W2 (o gesto de escultura) e W3 (a doação). As W1/W2 já
têm handoffs próprios (`HANDOFF_INTEGRACAO_line_sculpt3d_W1_2026-07-30.md` e `…_W2_2026-07-31.md`),
e este é o consolidado.

## 2. Foundational / compartilhado tocado, e por quê

⚠️ **Tudo aditivo, com default neutro** — as três costuras que o `02.3` previu, e nada além.

| Arquivo | O que muda | Aditivo? |
|---|---|---|
| `crates/ph2d-light/**` | **a crate deixa de estar vazia** — o rig, as lâmpadas, a resolução graus→vetor, o piso ambiente | crate nova de fato (era vazia) |
| `crates/ph2d-render/src/impasto_light.rs` + `.wgsl` + tests | **S1**: o passe de luz aceita um plano de forma (`form: Option<&[f32]>`, binding 8, `Globals.has_form` na vaga do `pad0`) | ✅ `None` ⇒ byte-idêntico |
| `crates/ph2d-tool-painter/src/tool/paint/impasto_*.rs` | **S1**: `shade_over` compõe duas fontes de normal; `ReliefFields.form`; `set_donated_form`/`donated_form` | ✅ sem forma, a expressão antiga **verbatim** |
| `crates/ph2d-tool-painter/src/tool/paint/impasto_rig.rs` | vira **re-export** de `ph2d-light` (os nomes que o Painter já usava) | ✅ |
| `crates/ph2d-panel-painter-layers/src/paint_impasto.rs` | o piso da elevação passa a vir de `ph2d_light::MIN_ELEV_DEG` (era literal duplicado) | ✅ mesmo valor |
| `shells/desktop/src/{app_state,main}.rs` | 2 campos: `donated_form: DonatedForm`, `sculpt3d_canvas_done: bool` | ✅ |
| `shells/desktop/src/render_loop/mod.rs` | 2 chamadas sob `#[cfg(feature = "sculpt3d")]` + o spawn da tela do smoke | ✅ |
| `shells/desktop/src/render_loop/painter_bridge.rs` | **1 arg novo** (`donated_form: &mut DonatedForm`) + o bloco que publica o tamanho e instala a notícia | ✅ |
| `shells/desktop/src/render_loop/painter_gpu_preview.rs` | passa `form:` adiante | ✅ |
| `shells/desktop/src/input_dispatch/keyboard.rs` | a cena 3D toma as teclas dela antes do store, sob `cfg` | ✅ inerte sem cena |
| `shells/desktop/Cargo.toml` | a feature `sculpt3d` + `ph2d-light` **não-opcional** | — |

⚠️ **`ph2d-light` passa a ser NÃO-REMOVÍVEL**, e é a única exceção — decisão que o `02.3` já tinha
tomado *para esta wave*: depois que o Painter passa por ela, arrancá-la quebra o Painter. Todas as
demais crates do módulo (`ph2d-mesh`, `-mesh-render`, `-sculpt3d`) seguem apagáveis.

⚠️ **O `painter_bridge` NÃO conhece o módulo 3D.** O que atravessa é `Vec<f32>` + um par de `u32`
(`crate::donated_form::DonatedForm`), e há gate afirmando isso (`the_consumer_does_not_know_what_a_mesh_is`,
com controle positivo). Apagar o módulo deixa o canal existindo e silencioso.

## 3. Símbolos que podem COLIDIR com outra linha

**Consts públicas novas** (todas em crates do módulo ou re-exportadas dele):

| Símbolo | Valor | Onde |
|---|---|---|
| `ph2d_light::MAX_LIGHTS` | `4` | crate nova |
| `ph2d_light::MIN_ELEV_DEG` | `5` | crate nova |
| `ph2d_light::AMBIENT` | `0.35` | crate nova |
| `ph2d_render::IMPASTO_MAX_LIGHTS` | `= ph2d_light::MAX_LIGHTS` | **deixou de ser literal** — era um espelho com gate que o comparava contra `4` |
| `MeshRenderer::GBUFFER_FORMAT` | `Rgba16Float` | crate do módulo |

⚠️ **NENHUM id de widget, chave i18n, token de tema ou entrada em lista ordenada.** Conferido por
grep sobre o diff: zero `ids::`, zero `register(`, zero `NodeId(`. O interruptor da doação é uma
**tecla** (`D`), não um widget — ver §6.

**Enum variants novos:** nenhum em tipo compartilhado. `FormRole` é privado do shell.

**Env vars novas:** `PH2D_SCULPT3D_SMOKE=2` (a `=1` já existia).

## 4. Contratos congelados encostados

**NENHUM.** Conferido por gate, não por auto-relato:

```text
architecture_tool_contract_surface   4/4 ok   (Tool=12 · RasterEditTool=5 · CanvasPaintTool=1 · PanelEvent=4)
architecture_contract_surface        3/3 ok   (NodeOp=2 · OpResolver=1 · NodeManifest=8)
```

⚠️ **É a decisão de arquitetura do ADR-0150 que mantém isso:** a navegação orbital e o gesto de
escultura moram no **shell**, nunca numa `Tool` — nenhum método novo no contrato.

**`PROJECT_SCHEMA` fica em 46, intocado.** Nada desta linha é serializado: a escultura vive num
viewport solto e o rig de luz do Painter já viajava. ⚠️ **Isto tira a linha da disputa de número**
com quem estiver bumpando na mesma janela.

**Registro do `ph2d-ecs`: intocado.** Nenhum componente novo (a costura **S3** do `02.3` não foi
usada).

## 5. O que só o `ship.sh` pega — e o que já rodei

Rodado nesta árvore, **1× sobre o diff acumulado**:

| | |
|---|---|
| `cargo fmt --all -- --check` | ✅ |
| `cargo clippy --workspace --all-targets` | ✅ zero warning |
| `cargo machete` | ✅ **nenhuma dep não-usada** — importa porque a linha tem dep nova |
| `cargo deny check` | ✅ advisories · bans · licenses · sources |
| `cargo test --workspace` | ✅ exceto a flake do §7 |
| `architecture_workspace_file_loc_cap` · `file_loc_caps` (shell) | ✅ 2/2 e 2/2 |
| Gates de GPU (`#[ignore]`, na RTX) | ✅ `ph2d-mesh-render` **15/15** · `ph2d-render::impasto_light_gpu` **6/6** |

⚠️ **DEP EXTERNA NOVA: `half = "2"` na `ph2d-mesh-render`.** O G-buffer é `Rgba16Float` e a doação
volta pela CPU, então alguém decodifica `f16`. **Não há decodificador escrito à mão de propósito:**
o workspace já tem UM (privado, na `ph2d-flip-render`) e o `half` **já estava no `Cargo.lock`** (dep
da `ph2d-tool-color-equalization`) — um terceiro seria a terceira resposta a uma pergunta só, e esta
alimenta os pixels do artista. `Cargo.lock` ganha só a aresta.

⚠️ **Os gates de GPU são `#[ignore]` e sem adapter fazem *skip gracioso*, que NÃO é verde.** Rode-os
na RTX:

```bash
cargo test -p ph2d-mesh-render --release --test gpu_render -- --ignored
cargo test -p ph2d-render --release --test impasto_light_gpu -- --ignored
```

## 6. Ordem, dependências e o que smoke-testar

**Os 23 commits são sequenciais e sem ordem especial** — cada wave compila e passa sozinha.

**Smokes aprovados pelo Enio:**

```bash
env PH2D_SCULPT3D_SMOKE=1 cargo run -p ph2d-host-desktop --release   # W1+W2: a malha e o gesto
env PH2D_SCULPT3D_SMOKE=2 cargo run -p ph2d-host-desktop --release   # W3: A DOAÇÃO
```

⚠️ **A cena imprime o que montou — se essas linhas não aparecerem, pare.** Na `=2`: esculpa, aperte
`D` até o terminal ler `LUZ`, pegue o Painter e pinte **chapado** (Digital, sem impasto) — a tinta
sai **acesa pela forma**; aperte `D` de novo (`DESLIGADA`) e a mesma tinta fica plana.

**E rode uma vez SEM a env var** — é a metade que prova a inércia: sem a cena armada, o frame 2D é
byte-idêntico (`AppGfx.sculpt3d` nasce `None` e cada porta devolve `false` no primeiro `if`).

### O que NÃO foi smokado, porque não existe

A escultura **não é uma camada do documento**: não é salva, não tem z na pilha, e o
`LayerKind::Sculpt3d` que o `02.3` lista como costura **S2 não foi apendado** — de propósito, porque
um variant que ninguém constrói é um variant morto, e construí-lo de verdade arrasta painel,
compositor, undo e save. Isso é o **modelo de documento**, e é a wave seguinte.

O toggle *por camada* (*"iluminada pela forma abaixo"*) segue o mesmo caminho, e o desenho já está
resolvido: a máscara das camadas que optaram entra **no `impasto_fields`, na CPU**, pesando o plano
antes de ele cruzar a costura — o mesmo princípio que já governa o relevo (*só a ÓPTICA porta; o FOLD
não*), então o shader não muda uma linha.

## 7. ⚠️ Flake PRÉ-EXISTENTE, **medida no `main`**

`the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke`
(`shells/desktop/src/flip_fit_budget_tests.rs`, da `line/FLIP`) é um kill de **wall-clock cru em
debug** (`ms < 5.0`, com *"medido local: 0,72 ms"* ao lado) e reprova sob carga.

**Não é desta linha, e a prova não é argumento:**

| | |
|---|---|
| `git diff main -- '*flip*'` | **vazio** — a linha não toca um byte do código sob teste |
| suíte cheia **nesta árvore** | falha **1 de 3** |
| suíte cheia **no `main`**, mesma máquina, mesma condição | falha **3 de 5** (`6.96 ms`) |

Re-rode sozinho antes de suspeitar de um merge. Se o integrador quiser fechá-la, é gate de outro dono
(o bar tem de virar RAZÃO — a própria `line/FLIP` já documentou essa política).

## 8. Números do estado, para conferência rápida

```text
PROJECT_SCHEMA        46   (intocado)
contrato de tools     Tool=12 · RasterEditTool=5 · CanvasPaintTool=1 · PanelEvent=4   (gate 4/4)
contrato de nodes     NodeOp=2 · OpResolver=1 · NodeManifest=8                        (gate 3/3)
registro ph2d-ecs     intocado (nenhum componente novo)
ids de widget         nenhum        tokens: nenhum        i18n: nenhuma chave
ADR                   nenhum novo (a linha inteira roda sob o ADR-0150)
deps externas novas   half = "2"  (ph2d-mesh-render)
```

---

**Linha `sculpt3d` pronta (HEAD `8ae5e81e5`, 23 commits, smoke aprovado). Aguardo ordem de
integração.**
