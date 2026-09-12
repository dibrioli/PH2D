# HANDOFF DE INTEGRAÇÃO — `line/app-physics`, **FASE B** (o corte) — 2026-09-11

> **Linha:** `line/app-physics` · worktree `Worktrees/line-app-physics` · base `b374411a1`
> **Fase A** fechou e foi integrada ao `main` em 11/09. Isto é a **Fase B**.
> ⚠️ Esta linha abriu **sozinha, como batedora** das cinco da W2 Fase B. A §2 é a
> resposta que ela deve às outras quatro, e é a primeira coisa a ler.

---

## §1 · O veredito, em números

| grandeza | antes (`b374411a1`) | depois | Δ |
|---|---:|---:|---:|
| `shells/desktop` (o que a catraca mede) | 465 105 linhas / 1 801 ficheiros | **453 755 / 1 763** | **−11 350 (−2,44 %)** |
| `crates/ph2d-app-physics` | 22 465 / 110 | **33 923 / 157** | +11 458 / +47 |
| `shells/desktop/src/physics/` (o que FICOU) | — | 18 275 / 66 | — |

**Prova do §3 (nada se perdeu):** `cargo nextest list --workspace --cargo-profile ci-test`
→ `scripts/nextest-list-diff.py`:

```
antes: 22655 testes (22031 chaves) | depois: 22655 (22031)
MOVED (mesma chave, outro pacote/binário): 121
ONLY-A (perdidos): 0
ONLY-B (novos): 0
```

⭐ **Exacto nos dois sentidos** — nem um teste perdido, nem um inventado. Os 121 `MOVED`
são `ph2d-host-desktop::bin/…` → `ph2d-app-physics`, que é a extracção a aparecer.

**Portão de fecho:** `nextest-impacted.sh` (BASE=`b374411a1`) → **5 237 testes, 5 237 passam** ·
`clippy --all-targets` nas duas crates → **0 avisos** (o CI corre `-D warnings`) ·
`cargo fmt --all --check` limpo · **0 órfãos e 0 duplicados** nas duas árvores ·
`cargo test -p ph2d-app-registry-init` verde.

**A catraca `the_shell_only_shrinks`: VERDE**, com **15 350** linhas de folga.
⚠️ **Para o integrador:** o censo de obsolescência dela dispara quando a folga passa de
**20 000** — faltam **4 650 linhas**. *Se as outras quatro linhas removerem isso somado
(e vão), o `TETO_LOC` tem de ser reescrito com a medição da ÁRVORE COMBINADA.* ⛔ Esta
linha **não** lhe tocou, de propósito: é a grandeza que soma entre linhas.

---

## §2 · ⭐⭐⭐ A RESPOSTA ÀS OUTRAS QUATRO: as 5 portas do `AppHost` CHEGARAM

**Zero sextos métodos. Nenhum foi preciso, e nenhum é pedido.**

⚠️ **E o mais útil não é o «sim» — é *o que parecia precisar da `App` e não precisava*.**
Em todos os casos medidos, o que bloqueava um ficheiro **nunca foi a `App`**:

1. **Três FOLHAS residentes na shell, partilhadas entre famílias** — `render_loop::inspector_ordering`,
   `preview_drive`, `name_unique`. São puras sobre crates de módulo, e ficaram na shell só porque
   têm **11 / 38 / 14** consumidores espalhados por famílias diferentes. *Elas são o alvo de uma
   linha própria, não desta.*
2. **O estado da própria família ser um campo da `App`** — curado na Fase A com o `PhysicsState`.
3. **Um `use` de seis linhas.** O `anchor_side` era um `match` puro de 6 linhas que prendia o
   `joint_anchor_drag` → que prendia o `PhysicsState` → que prendia o roteador. *Uma cadeia
   inteira segura por um `match`.*

⭐ E o caso que mais parecia precisar de porta nova — os três gestos de corpo (`body_fk`,
`body_grab`, `body_pose`, que eram `impl App`) — **não precisava de nada**: o que eles queriam
eram **três tipos que a shell por acaso segurava** (`PhysicsBridge`, `SimWorld`, `IkOptions`), e a
conversão `ecrã → mundo` fica onde a câmera está, que é a shell. Viraram funções livres.

⇒ **Recomendação às outras quatro:** antes de pedir uma porta, escreva o que a função
precisa em TIPOS. Se a resposta é «três coisas que a `App` segura», não é porta — é assinatura.

---

## §3 · O que FICOU na shell, e porquê — as 26 raízes MEDIDAS

O fecho é sobre o grafo de compilação, não sobre escopo: **26 raízes**, todas na shell, e
**ZERO ficheiros da crate precisam da shell**. Agrupadas por quem as prende:

| prende | nº | o que é |
|---|---:|---|
| `App` | 9 | o roteador de smoke + as 5 cenas que ele chama por método + `player_input` + os dois gestos de canvas |
| `render_loop` | 7 | `inspector_ordering` (a folha partilhada) + o gizmo de pontos |
| `preview_drive` | 4 | o *ledger* de pré-visualização do undo — **folha partilhada** |
| `name_unique` | 3 | a nomeação única — **folha partilhada** |
| `component_attach` + `init` | 4 | a **porta de produção** que anexa um componente |
| `app_state` | 2 | o arrasto do rig |
| `undo` / `project_library` | 2 | a ponte de bake |

⚠️⚠️ **Nenhum destes ficou por causa da `App` como TIPO** — e essa é a resposta que esta
linha deve às outras quatro (§2).

---

## §4 · ⛔ O FIM DA LINHA **NÃO** FOI ALCANÇADO — e a lista exacta do que falta

Das quatro condições gateadas do briefing, **uma** está cumprida:

| condição | estado |
|---|---|
| `PH2D_PHYSICS_SMOKE` lido **dentro** da crate | ⛔ **NÃO** — o roteador é um `impl crate::App` na shell |
| `const FAMILY` declara o roteador com `max_level` contado do `match` | ⛔ **NÃO** — `routers: &[]` |
| `"physics"` sai de `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` | ⛔ **NÃO** — continua lá |
| `cargo test -p ph2d-app-registry-init` verde | ✅ **SIM** (verde *porque a excepção está declarada*) |

⭐ **Mas o roteador está a 16 braços de sair, e o número é medido:**

```
arms no match: 117
  cena da CRATE (via run_physics_scene): 101   ← já não tocam a App
  metodo da SHELL:                        16
```

Os 16 são: `physics_smoke_pile` · `_author` · `_world` · `_layers` · `_joints` · `_bake` ·
`_parented` · `_weld` · `_bake_range` · `_joint_anim` · `_float` · `_walk` ·
`_author_player` · `_jump` · `_reaction` · `_out`.

E eles partem em **duas classes de preço muito diferente**:

- **Baratos (a maioria):** só tocam `self.gfx` — o `AppGfx` (sim + ponte + câmera). São o
  mesmo caso dos `body_*` do §2: viram funções livres sobre os tipos que já viajam.
- **⛔ O bloqueador NOMEADO:** `physics_smoke_walk` (e irmãos do player) autoram uma **track
  de timeline** (`author_platform_track(&mut self.timeline.doc, …)`). É exactamente o item
  *«timeline»* que o §6 do handoff da Fase A já nomeava como *«o que só o substrato resolve»*.
  ⛔ **Não o invente:** ou o substrato ganha a porta, ou aquelas cenas ficam na shell e o
  roteador passa a ter duas metades — que é decisão de desenho, não de implementação.

⚠️ **E há uma terceira coisa que a próxima janela tem de decidir antes de mover o `match`:**
com 16 braços na shell, um roteador *inteiramente* na crate não pode chamá-los (a crate não
vê a shell). Ou os 16 saem primeiro, ou o roteador parte em dois — e um roteador partido
**não satisfaz a condição do briefing**, que pede o `env::var` lido dentro da crate.

---

## §5 · As armadilhas que esta fase PAGOU (o que a próxima linha não deve repagar)

### 5.1 ⛔⛔ Uma régua textual que lê PROSA acusa 93 ficheiros de 133
A 1.ª medição do fecho varria `\bApp\b` sobre o ficheiro inteiro e concluiu que **93 dos 133**
ficheiros da crate precisavam da shell — o que teria desfeito a Fase B inteira. O que casava
era o **doc-comment que EXPLICA a cura**: *«⚠️ Era um `impl App` (W2/L2 Fase B)»*.
⇒ *Um censo textual que não separa prosa de código lê a documentação da cura como prova da
doença.* Com comentários retirados: **26** raízes, todas reais.

### 5.2 ⚠️ E a 2.ª leu por STEM, fundindo 40 módulos num nó
Há ~40 ficheiros chamados `tests.rs` declarados por `#[path]`. Indexar o grafo por *stem*
colapsa-os num nó só, e o corpus inteiro passa a depender de tudo. **O fecho conta-se por
CAMINHO**, com `#[path]` resolvido relativamente a quem declara.

### 5.3 ⛔⛔ O ÓRFÃO e o DUPLICADO são a mesma auditoria, e os dois são MUDOS
- **13 ficheiros de teste ficaram órfãos** na shell: eram `mod X;` planos no `render_loop/mod.rs`;
  o ficheiro mudou-se para `physics/` e a **linha que o declarava** ficou lá, e foi apagada com
  os vizinhos que de facto saíram. *Um órfão não dá erro: ele deixa de ser compilado, e a suíte
  fica verde com menos gates do que tinha.*
- **3 ficheiros ficaram declarados DUAS vezes** (um gerador listou a pasta sem saber que aquele
  ficheiro já tinha pai por `#[path]`). *Um ficheiro com dois pais compila como dois módulos: os
  gates dele correm a dobrar.* Foi o `ONLY-B: 5` da prova que o denunciou — e os 5 nomes
  apareciam **também** em `MOVED`, que é a assinatura.

⇒ **O instrumento é um audit de alcançabilidade a partir da raiz do módulo**, que responde às
duas metades de uma vez: quem não é alcançado é órfão, quem é alcançado por dois é duplicado.
*Só a contagem total os distingue, e a contagem total é o que o `nextest-list-diff` mede.*

### 5.4 ⛔⛔⛔ O `#[cfg(test)]` órfão, pela TERCEIRA vez — e sempre com uma forma nova no meio
A Fase A pagou-o com uma **linha em branco** entre o atributo e o `mod`; a Fase B pagou-o outra
vez com **comentários `//`**; e agora uma terceira, ao RESTAURAR as declarações perdidas: no
original o `#[cfg(test)]` estava separado do `mod` por **quatro linhas de comentário** (no
`physics_tests`) e por um **`#[path]`** (no `joint_world_tests`). Sem o guarda, o módulo de teste
passa a compilar **no binário do produto**.
⇒ *Quem varre por texto tem de saltar TUDO o que pode estar entre o atributo e o item — e a
lista do que pode estar no meio nunca acaba.* O sintoma que o apanhou foi o clippy a acusar
funções mortas, não um erro.

### 5.5 ⚠️ Um TETO DE LOC pode estourar por um RENAME
O meu regex de reparo reescreveu `super::X` → `crate::render_loop::X` em **108 ficheiros** que
nada têm com a física. `crate::render_loop::` é 19 caracteres mais longo, o `rustfmt` quebrou as
linhas, e o `painter_bridge_overlays.rs` passou de 599 para **609** contra o cap de 600 — *um
teto estourado sem uma linha de lógica ter mudado*. Revertidas as 383 ocorrências: 599, e o diff
que o integrador tem de ler caiu de 178 para 98 ficheiros no `render_loop`.
⚠️ **A reversão tem cerca:** num ficheiro `#[path]`-incluído por um irmão, `super::` **não** é o
directório — é o módulo que o inclui (185 ficheiros do `render_loop` são assim).

### 5.6 ⚠️ 28 agulhas de gate, e nenhuma era o produto
21 gates reprovaram: todos lêem um ficheiro-fonte e procuram *quem chama quem*, e o corte mudou
os dois — o caminho e o nome. **Re-ancorar, nunca afrouxar.** ⚠️ E **dois deles vivem em `src/`,
não em `tests/it/`**: uma varredura que só olha a segunda pasta deixa-os vermelhos.

### 5.7 ⛔ O `cargo clippy --fix` foi TENTADO e REVERTIDO
Ele partiu a build (35 erros) ao apagar imports que **outra configuração de `cfg`** usava.
Numa árvore com muito `#[cfg(test)]` e `#[path]`, o `--fix` não é seguro.

### 5.8 ⚠️ O teste que ATRAVESSA a fronteira é uma espécie própria
7 gates de 28 (e os 29 da §14 do inspector) têm o **sujeito na crate** e exercitam um **gesto da
shell**. Um `#[cfg(test)]` é invisível do outro lado da fronteira, então eles não podem ficar com
o sujeito. ⇒ **o corte é por quem o teste EXERCITA, não por quem ele nomeia.**
⛔ E a tentação de curar com um atalho (`insert` à mão em vez da porta de produção) está
**recusada por escrito no doc do próprio helper**: *«um atalho de teste que constrói o componente
por outro caminho é a segunda porta que diverge»*. Custo de fazer certo: **2** itens privados
passaram a `pub`.

---

## §6 · Ficheiros de atenção para o integrador

- `crates/ph2d-app-physics/src/lib.rs` — `const FAMILY` continua com `routers: &[]` (§4).
- `crates/ph2d-app-registry-init/src/lib.rs` — `"physics"` continua na catraca. ⛔ **não apague.**
- `crates/ph2d-editor-core/tests/it/architecture_the_shell_only_shrinks.rs` — `TETO_LOC` **não
  tocado**; ver a nota da §1 sobre os 4 650.
- `shells/desktop/src/render_loop/point_gizmo.rs` — `joint_anchor_handles` passou de
  `pub(super)` a `pub(crate)` (um gate que o mede mudou-se para `physics/`).
- `shells/desktop/src/physics/inspector_body.rs` — era `render_loop/inspector_physics.rs`; a
  poda do prefixo dentro de uma pasta `physics` criava `physics::physics` (module inception).
- **Zero** contadores partilhados mexidos: `PROJECT_SCHEMA`, `VEC_SCENE_SCHEMA`, `FLIP_SCHEMA`,
  `FIELD_DOC_VERSION`, os registos de componentes — todos iguais ao `main`. Zero ADR, zero
  contrato congelado, zero pacote externo novo.

## §7 · Smoke

O binário fica **compilado** (2ª build: `Finished` em **0,22 s**, zero `Compiling`).

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-physics && env PH2D_PHYSICS_SMOKE=63 cargo run -p ph2d-host-desktop --profile smoke
```

⚠️ As cenas e os números são **os mesmos de antes** — esta fase não criou nem apagou cena
nenhuma (a prova do §1 diz isso com número). ⚠️ A `=15` tem as paredes **cinemáticas** de
propósito desde 30/08; ⚠️ a `=84` não existe, de propósito.
