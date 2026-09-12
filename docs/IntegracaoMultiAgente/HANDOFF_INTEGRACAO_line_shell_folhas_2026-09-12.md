# HANDOFF de INTEGRAÇÃO — `line/shell-folhas` (2026-09-12)

> **As folhas partilhadas saíram da shell.** Sete crates novas, `shells/desktop` de
> **411 246 → 398 037** linhas (`−13 209`, `−3,2 %`) e de **1 616 → 1 561** ficheiros, com a lista
> de testes **idêntica nos dois sentidos**. Briefing:
> [`BLOCO_ABERTURA_LINHA_FOLHAS_2026-09-12.md`](BLOCO_ABERTURA_LINHA_FOLHAS_2026-09-12.md).

---

## §1 — Identidade

| | |
|---|---|
| branch | `line/shell-folhas` |
| merge-base | `7f905e54c` (o `main` de 12/09, **não andou** durante a linha) |
| commits | 9 |
| ficheiros tocados | 338 |
| contratos congelados | **nenhum** |
| ADR | **nenhum** — esta linha fica fora de toda disputa de número |

---

## §2 — O que saiu, e por quê

⛔ **A régua é o CONSUMO POR FAMÍLIA, e ela foi medida no `main` de hoje** — não herdada do
briefing. Uma peça que duas famílias usam é uma FOLHA, nunca a casa de uma delas (ADR-0075,
[HOWTO §1.2](HOWTO_partir_uma_familia_da_shell.md)).

| crate | assunto | LOC | famílias que a consumiam |
|---|---|---:|---|
| `ph2d-unique-name` | o `Name` é único na camada do editor | 181 | **flip · physics · sculpt3d · vec** |
| `ph2d-entity-visibility` | «esta entidade desenha — e processa — neste quadro?» | 633 | vec (+ 3 consumidores da shell) |
| `ph2d-preview-drive` | o que um motor escreve AGORA é pré-visualização | 534 | **physics · vec** (+ field3d · flip nos gates) |
| `ph2d-inspector-ordering` | uma edição de painel vira comando para o mundo | 595 | **physics** (7 ficheiros) |
| `ph2d-transport` | o comando de transporte conduz o ÚNICO relógio | 69 | ⛔ **nenhuma** (ver §6) |
| `ph2d-vec-entities` | a forma vetorial é uma ENTIDADE da árvore, com pose | 2 912 | **motion · flip · vec** |
| `ph2d-audio-desktop` | o dispositivo de saída e o editor de áudio | 8 599 | ⛔ **nenhuma** (ver §6) |
| — | o alias `modal` foi **apagado** (a lei já estava na `ph2d-app-host`) | −70 | vec |
| | **total nas folhas** | **13 500** | |

### O que cada família passa a poder tirar — contado no `main`, ficheiro a ficheiro

| família | ficheiros dela que dependiam de uma das folhas | por folha | hoje |
|---|---:|---|---:|
| `vec` | **67** | `vec_entities` 59 · `vec_transform` 19 · `morph_set` 3 · `modal` 2 · `preview_drive` 1 · `name_unique` 1 · `off_canvas` 1 | **0** |
| `physics` | **12** | `inspector_ordering` 7 · `preview_drive` 4 · `name_unique` 3 | **0** |
| `flip` | **6** | `vec_transform` 4 · `vec_entities` 1 · `preview_drive` 1 · `name_unique` 1 | **0** |
| `motion` | **3** | `vec_entities` 3 | **0** |
| `sculpt3d` | **1** | `name_unique` 1 | **0** |
| `field3d` | **1** | `preview_drive` 1 | **0** |

⚠️ **Isto NÃO é uma promessa de quanto sai.** É o que estas peças prendiam; *a régua de um
bloqueio é o fecho, nunca a contagem* ([HOWTO §2.12](HOWTO_partir_uma_familia_da_shell.md)). O que
de facto sai mede-se quando cada família voltar a fechar.

---

## §3 — A PROVA

```
cargo nextest list --workspace --cargo-profile ci-test   (antes, na base · depois, no tip)
python3 scripts/nextest-list-diff.py antes.txt depois.txt

  antes: 22665 testes (22038 chaves) | depois: 22665 (22038)
  MOVED (mesma chave, outro pacote/binário): 130
  ONLY-A (perdidos): 0
  ONLY-B (novos):    0
```

⭐ **Exacta nos dois sentidos** — nem um teste perdido, nem um inventado.

```
cargo nextest run -p ph2d-host-desktop --test it --cargo-profile ci-test
  Summary: 816 tests run: 816 passed, 6 skipped
```
⚠️ Este alvo corre **à parte** porque o `nextest-impacted` não o alcança, e foi ali que a `flip`
reprovou na integração de 12/09. Na **primeira** corrida ele deu `11 FAILED` — ver §5.

Auditoria estrutural das sete crates (órfão · duplicado · feature não declarada), as três lentes:

| crate | `.rs` | alcançados do `lib.rs` | features usadas | veredito |
|---|---:|---:|---|---|
| `ph2d-unique-name` | 1 | 1 | — | ⭐ limpo |
| `ph2d-entity-visibility` | 4 | 4 | — | ⭐ limpo |
| `ph2d-preview-drive` | 1 | 1 | `test-support` | ⭐ limpo |
| `ph2d-inspector-ordering` | 1 | 1 | — | ⭐ limpo |
| `ph2d-transport` | 1 | 1 | — | ⭐ limpo |
| `ph2d-vec-entities` | 12 | 12 | `test-support` | ⭐ limpo |
| `ph2d-audio-desktop` | 38 | 38 | `audio-ml` · `panel-audio-editor` · `test-support` | ⭐ limpo |

⛔ *O órfão e o duplicado são as duas metades do mesmo audit, e as duas são MUDAS* — zero de cada.

---

## §4 — Superfície de colisão (`scripts/collision-surface.sh`, corrido no tip)

⚠️ **Re-rode-o antes de fundir** — esta tabela mede contra o `main` do dia em que a linha fechou.

```
▸ SCHEMAS        PROJECT_SCHEMA 128 (base: 128) · tripla (128, 13, 22) idêntica
                 FLIP_SCHEMA 13 (base: 13) · DOC_VERSION 18 (base: 18)
▸ REGISTRO       ph2d-render 86 (base: 86) · ph2d-script 86 (base: 86)
▸ CONTRATO       node.rs intocado · tool.rs intocado
▸ ADR            esta linha não cria ADR
▸ Cargo.lock     7 pacotes novos, TODOS internos (as sete crates desta linha)
▸ MARCADORES     nenhum
▸ TETOS DE LOC   nenhum ficheiro da linha passa do tecto
```

⭐ **Zero contadores partilhados se mexem.** Mover código não muda serialização — e se algum
número se tivesse mexido, a linha teria feito mais do que a tarefa.

### ⚠️ A catraca `the_shell_only_shrinks` NÃO reprova, e isso é um achado

`TETO_LOC = 415 246` · shell = **398 037** · folga = **17 209** · o censo de obsolescência dispara
acima de **20 000**. ⇒ ela passa por **2 791 linhas**.

⛔ **O briefing previa que ela reprovasse («o seu marcador de progresso») e ela não reprova.** Uma
linha que tire menos de 20 000 linhas deixa a catraca **silenciosamente desactualizada**: o número
já não descreve a árvore e nada o diz. ⇒ **o integrador tem de a recontar à mão** sobre a árvore
combinada, como o doc-comment dela manda — e com outras linhas na mesma rodada o limiar passa a ser
cruzado por acumulação, que é a outra metade do mesmo buraco.

---

## §5 — O que só o `ship.sh` pega, e o que a primeira corrida apanhou

**⚠️ Quatro avisos de clippy PRÉ-EXISTENTES** (nenhum dos ficheiros está no diff desta linha —
conferido com `git diff --name-only main..HEAD`):

| aviso | ficheiro |
|---|---|
| `too many arguments (8/7)` | `crates/ph2d-app-sculpt3d/src/keys.rs` |
| `empty line after doc comment` | `crates/ph2d-app-sculpt3d/src/bake.rs` |
| `items after a test module` | `crates/ph2d-app-sculpt3d/src/requests.rs` |
| `needless_borrow` | `shells/desktop/src/sculpt_source/mod.rs` |

⛔ Não foram tocados de propósito: são território da `line/sculpt3d`, e o `ship.sh` corre clippy
com `-D warnings`. **Quem shipar tem de os curar ou pedir à linha deles.**

### ⭐ E as 15 agulhas que só o teste CORRIDO apanha — três famílias, uma delas nova

A suíte da shell deu **11 de 816 vermelhas** na primeira corrida, com o `cargo check` verde:

1. **[HOWTO §2.6](HOWTO_partir_uma_familia_da_shell.md) — o gémeo em RUNTIME** (4 leituras em 3
   gates): `read_to_string("src/render_loop/off_canvas.rs")` e irmãos. *Um `cargo check` verde não
   diz nada sobre eles*; um `#[ignore]` ou um filtro e nunca falhariam.
2. **§2.9 — a agulha que nomeia um ENDEREÇO** (9 ficheiros): procuravam
   `"crate::vec_entities::sync("` no texto do `render_loop/mod.rs`.
3. ⭐⭐ **UMA ESPÉCIE QUE O HOWTO NÃO LISTA — a agulha que nomeia a VISIBILIDADE.** O
   `the_arrange_buttons_write_the_z` ancorava em `"pub(crate) fn reorder("`, e **o que a partiu não
   foi a mudança de sítio: foi o `pub(crate)` virar `pub`** ao publicar a API da folha. Ela reprovou
   sem que uma linha do `reorder` se mexesse. ⇒ *uma agulha que nomeia quem PODE chamar mede outra
   coisa que não a lei*. Hoje é `fn reorder(`, imune ao próximo movimento.

⭐ **E o censo de directório está ILIBADO com número** (§2.7, a armadilha muda): o
`settle_skips_every_derived_geometry` varre `shells/desktop/src/*.rs` à procura de
`Transform::IDENTITY;`, e **nenhum dos 55 ficheiros que saíram o contém** — os quatro hosts ficaram
todos, e o piso de população (`>= 3`) descreve a mesma população de antes.

---

## §6 — ⛔⛔ DUAS coisas do briefing que a medição REFUTOU

### O `transport` não desbloqueia ninguém: ele tem **UM** consumidor

O briefing dava-lhe *«physics 5 · motion 2»*. Medido: `render_loop/mod.rs:5479`, e mais nada em
todo o repo. As sete citações eram a palavra **«transporte» em PROSA**, dentro de comentários da
física. Ele saiu por ser uma lei pura de 69 linhas com gates próprios — **não** pela fila.

### O `audio` não desbloqueia ninguém, e é **8 544 linhas** e não 569

O briefing dava-lhe *«motion 6 · physics 1»* e contava só o ficheiro de topo. Medido:

* as seis citações do `motion` são a string `"audio.bands"` (um **nome de nó**), a crate
  `ph2d_node_audio_bands` e o módulo `render_loop::motion_audio_gen` — **nenhuma** toca
  `crate::audio`;
* o módulo é uma **árvore de 37 ficheiros** (o dispositivo, o rack, os presets, o espectral, o
  livro das vozes, o runtime do editor), com 7 roteadores `PH2D_AUDIO_*` e estado na `App`.

⇒ é a [§2.12](HOWTO_partir_uma_familia_da_shell.md) outra vez: **um censo textual que não separa
prosa e identificadores alheios do código**. Ele saiu pelo relógio da shell (2,1 % de uma unidade
de compilação que é a **última** de toda build grande), e isso está dito no commit.

⭐ **Os sete roteadores FICAM no `main.rs`** e isso é a decisão: a shell lê a env e chama o método —
o roteador é **composição**. Ele não é uma família com roteador próprio, logo não entra no
`AppFamily` nem no `ph2d-app-sync`.

---

## §7 — ⭐⭐ A extracção do áudio CUROU um buraco que já lá estava

No `main`, `mod editor;` é declarado **sem `cfg`** e usa `ph2d_audio_edit`, que é dependência
**opcional** gateada por `panel-audio-editor`. ⇒ **aquele módulo nunca compilou sem a feature**, e
ninguém via porque ela está no `default` da shell — não havia como endereçá-lo sozinho.

Agora ele é um pacote e o `cargo check -p` mede-o. A cura custou **quatro** `#[cfg]` (os campos
`delivery` / `platforms` / `fx_scratch` / `spectral` do `AudioSystem` e os inicializadores deles), e
as **três** configurações ficam verdes: sem feature · com `panel-audio-editor` · com `audio-ml`.

⚠️⚠️ **As features foram declaradas na crate E reenviadas pela shell** ([§2.4](HOWTO_partir_uma_familia_da_shell.md),
a armadilha muda): 21 sítios vivem sob um `cfg(feature = …)`, e numa crate que não as declara o
`cfg` é falso **por construção** — ela compilaria VERDE com o editor de áudio e o denoise inteiros
desligados. A composição foi lida do manifesto da shell, não de memória (o `audio-ml` **implica** o
painel, que arrasta o `-edit` e o `-encode`).

---

## §8 — Sete coisas que uma leitura rápida do diff entende ao contrário

1. **`ph2d-vec-entities` não é «o vetor».** Ela é a ponte documento⇄árvore, e está fora da
   `ph2d-app-vec` **de propósito**: a `motion` e a `flip` consomem-na, e pô-la na crate de uma
   família faria duas famílias depender de uma terceira (ADR-0075).
2. **O `entity_map` VOLTOU da `ph2d-app-vec`.** Ele tinha ido para lá na Fase B daquela linha; é a
   mesma peça partilhada um degrau acima. A `ph2d-app-vec` **re-exporta-o**, e os 18 ficheiros dela
   não mudaram uma linha.
3. **O diagrama de bloqueadores no `lib.rs` da `ph2d-app-vec` FICA como está.** Ele é a medição que
   abriu esta linha; leva um veredito ao lado, e apagá-lo apagaria a razão.
4. **`vec_entities` + `vec_transform` + `morph_set` são UMA crate e não três.** O ciclo
   (`entities ──is_set_member──▶ morph_set ──sync──▶ entities`) é real, e **não é defeito de
   arrumação**: um conjunto de estados do morph *é* uma manipulação da hierarquia.
5. **Oito ficheiros de gate FICARAM na shell** (`vec_entities_tests`, os dois do z-order, a cadeia
   inteira do `morph_set`, o `vec_pencil_frame_tests`, o `preview_drive_tests`, o `modal_tests`).
   Eles atravessam o `input_dispatch`, o `hero_bridge`, o `undo`, o `vec_tree_settle`, o
   `morph_live` e o `profile_live` — *os testes seguem o SUJEITO, não o ficheiro* (§1.2). ⚠️ A
   cadeia do `morph_set` fica **inteira**: o pai é puro, mas declara quatro filhos que não são.
6. **A fixtura `setup`/`bits` VEIO para a crate e não foi duplicada.** O irmão da selecção usa-a de
   dentro e o gate da shell de fora — *duas fixturas para a mesma ponte seriam duas respostas a
   «como nasce uma cena de teste?»*, que é a razão que o doc dela já dava.
7. **O `UiSound` mudou-se com o MOTOR, não com o gatilho.** É o `AudioSystem::play_ui` que o lê; o
   `impl App` (a preferência do artista e o diagnóstico dos cinco elos) fica na shell.

---

## §9 — As premissas MINHAS que a medição derrubou

1. *«O `off_canvas` tem zero arestas à shell»* (do briefing, e eu acreditei) — **tem uma, em
   CÓDIGO**, na linha 42: `super::on_screen_gate::hides`. O irmão (126 LOC + 167 de teste) nunca
   tinha sido contado, e o par é que é a unidade.
2. *«A camada 0 são 2 290 LOC em 7 ficheiros»* — o conjunto real é **16 855 LOC em 64 módulos**. O
   censo contava o ficheiro de topo e não os `#[path]` nem os submódulos.
3. *«21 chamadas de `name_unique`»* — eram **22**; o `use` também casa `crate::name_unique::`. O
   assert de contagem apanhou-o no primeiro ficheiro.
4. **A minha própria sonda de fecho leu 865 de 1 375 ficheiros** nas duas primeiras redacções, e
   ficou **verde** a varrer menos: um `#[cfg(test)]` pode vir **antes** do `#[path]`, e uma
   alternância *lazy* não basta — o match global tem sucesso sem nunca experimentar o ramo do path.
5. *«O `queue_set` depende de `serde`»* — depende de `serde` **e** de `postcard`, no mesmo `fn`.
6. Eu escrevi *«oito itens `pub(crate)`»* no `inspector_ordering`; eram **`pub(super)`**, que numa
   raiz de crate nem compila (*«above the crate root»*).
7. *«O `audio` tem 569 LOC e desbloqueia motion e physics»* — §6.

---

## §10 — Dependências INVISÍVEIS que só a crate revelou ([§1.3](HOWTO_partir_uma_familia_da_shell.md))

Dentro da shell **todas** aquelas crates já eram dependência do binário, então um ficheiro escrevia
o nome sem nada que o declarasse. A lista de `[dependencies]` é o custo real, e só se lê aqui:

| crate | invisíveis |
|---|---|
| `ph2d-preview-drive` | `ph2d-physics-ecs` (o `Driven::JointParams` guarda um `PhysicsJoint`) |
| `ph2d-inspector-ordering` | `postcard` (e o `serde` com o porquê errado na 1.ª redacção) |
| `ph2d-vec-entities` | **cinco**: `ph2d-morph-machine` · `bevy_ecs` · `ph2d-render` · `ph2d-vec-edit` · `ph2d-skeleton-ecs` |

⭐ A `ph2d-vec-edit` é uma crate-**motor** (ADR-0108 Fase 1, *«puro, sem shell»*), não a
`ph2d-app-vec`: **uma motor irmã não é a família** — o precedente é da `line/app-physics`.

---

## §11 — Ordem dos commits, e o que smokar

Os 9 commits são **sequenciais e cada um compila**; funde na ordem. As folhas 1 e 2 são
pré-requisito da 7 (a `ph2d-vec-entities` depende das duas).

**⛔ O que NÃO foi smokado, e é o que o Enio tem de olhar:** esta linha **não muda comportamento
nenhum** — é mudança de endereço. O risco real é (a) o som e (b) o painel do editor de áudio,
porque a `ph2d-audio-desktop` é a única peça cujas **features** foram redeclaradas.

```
cd /home/enio/Documentos/Projetos/PH2D && cargo run -p ph2d-host-desktop --profile smoke
```
1. Abrir o app. A janela tem de abrir normal.
2. *Window → Audio Editor* — o painel tem de aparecer e os botões dele responder.
3. `env PH2D_AUDIO_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke` — tem de **ouvir-se um
   apito** de 440 Hz. Se for mudo: `bash scripts/audio-mudo.sh` (é o mixer do sistema, não o app).
4. Desenhar uma forma vetorial e arrastá-la; `Ctrl+Z` tem de a devolver.

---

## §12 — Fecho

* gate batched **1×** sobre o diff acumulado: `cargo clippy --workspace --all-targets` (só os 4
  avisos pré-existentes do §5) · `cargo nextest run --workspace --cargo-profile ci-test` ·
  `cargo nextest run -p ph2d-host-desktop --test it` **à parte** (816/816);
* `target/*/incremental` reclamado;
* smoke compilado na worktree, 2 corridas — a 2.ª colada abaixo;
* ⛔ **esta linha NÃO integra e NÃO roda o `foundational-integrate.sh`.**
