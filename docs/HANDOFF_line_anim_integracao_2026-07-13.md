# HANDOFF DE INTEGRAÇÃO — `line/anim` (2026-07-13)

> **Para:** o **agente integrador**, quando o Enio mandar.
> **De:** a linha `line/anim`. 9 commits locais, worktree limpo, branch `line/anim`.
> **Base:** `4cd8ef13` (main de 2026-07-13). Nada pushado, nada integrado.
>
> **⚠️ O SMOKE NÃO FOI FEITO.** O Enio smokou *parcialmente* (**"Save/open OK"**) e o resto fica
> para **amanhã**. Trate esta linha como **não-smokada**: os gates estão verdes e o produto foi
> exercitado só pelo Ctrl+S/Ctrl+O.

---

## §1 — ⚠️ A COLISÃO QUE VOCÊ NÃO PODE RESOLVER "ESCOLHENDO" — `PROJECT_SCHEMA`

**QUATRO linhas bumparam o MESMO contador, e nenhuma sabe das outras.** O `PROJECT_SCHEMA`
(`shells/desktop/src/project.rs`) é a versão do arquivo de projeto, e postcard é **posicional**:
o número não é decoração, é o que separa "ler o arquivo" de "ler lixo com cara de geometria".

| linha | o que quebrou o layout | bumpou para |
|---|---|---|
| `main` | — | **7** |
| **`line/anim`** (esta) | `ProjectFile` ganhou o 5º campo `timeline` | 8 |
| **`line/Vector`** | `VecVertex` ganhou `corner_radius` (`VEC_SCENE_SCHEMA_VERSION` 7→8), embutido no `ProjectState.vec` | 8 |
| **`line/FLIP`** | `FlipStroke.selected` (`FLIP_SCHEMA` 3→4) **+** `FlipFrame.offset` (4→5) | 9 |
| **`line/Painter`** | `PaintedDocument.mats` (novo) **+** `mats` mudou de FORMA (4→7 bytes) | 9 |

São **6 quebras independentes** sobre o 7 do `main`. Logo:

> ### `PROJECT_SCHEMA = 13`
>
> **Conte, não escolha** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]): o valor certo
> **não existe em nenhum lado do conflito**. Pegar o 9 do FLIP (o maior) faria o arquivo do Painter
> e o meu passarem na checagem de versão e serem lidos com o layout errado — postcard não tem nome
> de campo para reclamar; ele lê os bytes seguintes e devolve lixo bem-formado.

**E o gate que provaria isso está DENTRO do conflito.** O par
`a_flip_schema_bump_must_bump_the_project_schema` vivia no `mod tests` **inline** do `project.rs`:

- **eu DELETEI esse `mod tests` do `project.rs`** (o arquivo estourou o cap de 600 LOC) e o movi
  para o irmão **`shells/desktop/src/project_tests.rs`** (`#[path]`, o padrão do repo);
- **a `line/Vector` EDITOU esse mesmo bloco no lugar**, transformando o par numa **tripla**
  (`PROJECT_SCHEMA`, `FLIP_SCHEMA_VERSION`, `VEC_SCENE_SCHEMA_VERSION`).

Uma resolução delete/modify que **mantenha a deleção** joga fora o pin do Vector **em silêncio** —
e o pin do Vector existe justamente porque um campo do `VecVertex` pode passar despercebido.

**O que fazer, então (na árvore combinada):**

1. `const PROJECT_SCHEMA: u32 = 13;`
2. **Preservar as SEIS linhas de doc-comment** (`/// v8 … /// v13 …`) — cada linha escreveu a sua,
   e elas são o único registro de *por que* o número subiu. Renumere-as na ordem em que entrarem.
3. Levar o pin para `project_tests.rs` como **quádrupla**, e renomeá-lo (o nome mente hoje —
   não é só o Flip):
   ```rust
   #[test]
   fn a_schema_bump_anywhere_must_bump_the_project_schema() {
       assert_eq!(
           (PROJECT_SCHEMA,
            ph2d_flip::FLIP_SCHEMA_VERSION,
            ph2d_vec_scene::VEC_SCENE_SCHEMA_VERSION),
           (13, 5, 8),
           "…"
       );
   }
   ```
   (O Painter não exporta um contador próprio — ele quebrou o layout do `PaintedDocument` duas
   vezes e só subiu o `PROJECT_SCHEMA`. Se ele tiver ganhado um const na integração, some na tupla.)
4. **Custo real de errar aqui: ZERO saves publicados.** O path é `ph2d_project.postcard`,
   gitignorado, sem diálogo de arquivo. Ninguém tem um arquivo antigo para perder — **hoje**. É
   exatamente por isso que é barato acertar agora.

---

## §2 — O resto da superfície de colisão (verificado, worktree a worktree)

| arquivo | quem mais toca | natureza |
|---|---|---|
| `shells/desktop/src/project.rs` | FLIP · Painter · Vector | **§1** + campos do `ProjectFile`. **Só EU acrescento campo ao `ProjectFile`** (`timeline`, 5º) — os outros mexem *dentro* de `FlipDoc`/`PaintedDocument`/`VecScene`. Ordem final: `state, assets, painted, motion, timeline`. |
| `shells/desktop/src/project_tests.rs` | — (arquivo NOVO meu) | Recebe o `mod tests` que saiu do `project.rs`. É aqui que o pin do §1 tem de aterrissar. |
| `shells/desktop/src/input_dispatch.rs` | FLIP · Vector | Meu diff é **só deleção** (~37 linhas, ≈1586): as 2 fns do sidecar morto. Os hunks deles são em outros pontos (FLIP ≈2012/2293/2324). |
| `shells/desktop/src/input_dispatch/keyboard.rs` | FLIP · Vector | Meu diff é **só deleção** (o bloco Ctrl+S/Ctrl+O do sidecar, ≈300) + um comentário. **Se a resolução ressuscitar o bloco, NÃO COMPILA** (ele chama fns que não existem mais) — o que é a falha segura. |
| `shells/desktop/src/motion_state.rs` + `_tests.rs` | motion-value | Meu diff é doc-comment + 1 teste novo. Os hunks se sobrepõem no MESMO bloco de doc (meu `@@ -125,9`, o deles `@@ -129,9`). **Os dois são comentário: fique com os dois.** |
| `crates/ph2d-timeline/` · `crates/ph2d-panel-timeline/` | **ninguém** (exceto `marker_rename.rs`, que a motion-value toca e eu **não**) | Exclusivo. |
| `CLAUDE.md` | Vector | Append. Editei a entrada **Timeline** e a **Persistência de projeto** (§5). |
| `project-memory/MEMORY.md` + 4 memórias novas | todas as linhas | Append. |

---

## §3 — As MINAS SEMÂNTICAS (merge limpo no texto ≠ árvore sã)

[[feedback_clean_text_merge_can_be_semantically_broken]] — estas quatro passam pelo `merge-tree`
sem um conflito e **quebram o produto**. Cada uma tem gate; rode-os na **árvore combinada**.

1. **`Entity::from_bits` em bits de binding = CRASH.**
   `0` é o sentinel de "binding destacada" (o `resolve_entities` sempre o escreveu), e `0` **não é
   nulo** no bevy: o índice é `NonZero<u32>`, então `from_bits(0)` **entra em pânico**. Todo leitor
   de bits usa `Entity::try_from_bits` — `crates/ph2d-timeline/src/apply.rs` (passes 1 **e** 2) e
   `shells/desktop/src/timeline_persist.rs::wire_of`. **Um merge que ressuscite `from_bits` faz o
   Ctrl+O de qualquer projeto com animação derrubar o app.**
   → gate: `crates/ph2d-timeline/tests/detached_bindings.rs`.
2. **A guarda do `stamp_wire_ids`.** `crates/ph2d-timeline/src/persist.rs`: um `WireId::NULL`
   **nunca** sobrescreve um hash guardado. Sem ela, **o Ctrl+S apaga a identidade de toda track
   dormente** (objeto deletado) e ela nunca mais recola — nem por undo, nem recriando o objeto.
   → gate: `a_save_never_erases_a_dormant_tracks_identity` (mesmo arquivo).
3. **`StripView` ganhou 2 campos** (`ease_locked_in`/`ease_locked_out`). Nenhuma linha viva
   constrói `StripView` — se alguma passar a construir, o merge acusa (campo faltando).
4. **`timeline_persist::{save, load, save_path, deserialize}` foram REMOVIDOS** (o sidecar).
   Verifiquei: nenhuma linha viva os referencia. Se alguma passar a referenciar, não compila.

---

## §4 — O que esta linha entrega

**Commits** (`git log main..line/anim`, do mais antigo):

| | |
|---|---|
| `6991bdad` | **gate da rebobinada do relógio no load** — a tarefa que o integrador anterior deixou. A premissa ("o `App` exige janela") era **falsa**: `App::new()` nasce sem janela (winit cria no `resumed`), e todo passo do load que depende de `gfx` já degrada para no-op. O gate dirige a função REAL do Ctrl+O, com arquivo real no disco. |
| `b76a5b50` | 2 memórias |
| `d86347cd` | **W4.T6/B5 — a animação vai DENTRO do projeto.** Fechar o app perdia a timeline inteira; o "sidecar" que dizia salvá-la era **código morto** (o Ctrl+S global retornava antes dele, e o comentário dele ainda dizia "não há save de projeto ainda"). `ProjectFile.timeline` + `PROJECT_SCHEMA` 7→8. A binding viaja pelo **`wire_id` (hash do `Name`)**, nunca pelos bits: o load instala o doc **destacado** e o `upkeep` do frame — a MESMA função que cura delete+undo — recola pelo nome. |
| `0cd1270a` | CLAUDE.md §5 (a entrada Timeline estava mentindo) |
| `0ad877c6` | **B4 — a alça de fade do strip.** `ease_in`/`ease_out` existiam, o avaliador os honrava, e **nada os escrevia**: um clipe sozinho na faixa entrava e saía duro. Alça na ponta da cunha; **read-only** onde um vizinho define a janela (regra da Unity — ease e blend são a MESMA curva). A pergunta *"de quem é esta janela?"* passou a ser feita **UMA vez** (`ClipLane::neighbour_reach_in/out`). |
| `a0921938` | CLAUDE.md (B4 landou) |
| `48aa6593` | **os 6 defeitos que a auditoria de 2 lentes achou** — §5 |
| `5d640234` | 2 memórias |
| `c118a792` | **a alça era inacertável** — §5 |

**API pública nova em `ph2d-timeline`** (aditiva, nada removido):
`sync_transport_loop` · `ClipLane::neighbour_reach_in/out` · `TimelineIntent::SetStripEase`
(variant **apendado**; `TimelineIntent` não é serializado) · `StripView` +2 campos.
**`DOC_VERSION` NÃO mudou** (segue 4) — `ease_in`/`ease_out` já existiam no `ClipStrip`.

---

## §5 — O que a auditoria achou (e por que isso importa para VOCÊ)

Duas lentes adversariais, independentes, sobre o diff **já commitado e todo verde**. Elas acharam
**8 defeitos, 2 deles fatais** — e o que os tornava invisíveis é o que você deve levar para a
árvore combinada:

1. **CRASH (`from_bits(0)`)** — §3.1. A mina estava armada **havia semanas**; o único caminho que
   escrevia o sentinel era o sidecar morto. Eu liguei o sentinel na via principal e ela explodiu.
   *E o meu gate afirmava `entity == 0` como condição de **sucesso** — ele consagrava o estado do
   crash.*
2. **Ctrl+S apagava a identidade** — §3.2.
3. **O loop vazava entre projetos** (ele mora no CLIP; o `Playhead` é só a cópia, publicada por
   `sync_loop`, que só intent chamava — e um LOAD não passa por intent).
4. **Animação ilegível abria o projeto SEM ela** — e o próximo Ctrl+S gravava o vazio por cima.
   Agora **recusa o arquivo inteiro**, antes de tocar na sessão. E o load/save/recusa **toastam**
   (os toasts existiam… no sidecar que eu deletei).
5. **B4: clamp por-borda** — `weight_at` MULTIPLICA as rampas, então 2 s de fade-in + 2 s de
   fade-out num strip de 2 s davam pico de peso **0,25**: numa faixa `Override`, um sprite
   permanentemente meio-misturado com a pose de baixo. Clamp na **soma**.
6. **B4: as duas alças se atropelavam** exatamente na forma mais ordinária (fade-in e fade-out que
   quase se encontram) — mesmos retângulos, e o hit index é *last-wins*: agarrar a ponta visível de
   uma arrastava a outra.
7. **A alça era INACERTÁVEL** (o smoke do Enio). Medida na pintura real: corpo **240×22**, borda de
   aparar **6×22**, alça **7×7** — um décimo da área, numa faixa de 7 px no topo. O gate de hit
   passava; o dedo não. Agora **7×22**, e o gate mede a **ergonomia** (altura e área ≥ as da borda
   de aparar), não só a existência.
8. **Os botões da faixa** (mudo / + clipe) eram dois quadrados **vazios e idênticos** — nunca
   tiveram ícone. Agora `Eye`/`EyeClosed` e `Add`.

**As 4 lições viraram memória** (`project-memory/`): o harness que ninguém tentou construir · o que
sobrevive a um load é **adotado** · **sentinel exige gate no LEITOR** · **fixture no ZERO é gate que
não pode falhar** (as fixtures do B4 tinham cunha 0 e `start_ease` 0 — bastou pôr uma fade de
verdade para os dois gates ficarem vermelhos na hora).

---

## §6 — Gates (rode na ÁRVORE COMBINADA, não só na minha)

```bash
cd <árvore combinada>
cargo test -p ph2d-timeline -p ph2d-panel-timeline -p ph2d-host-desktop -p ph2d-editor-core
bash scripts/nextest-impacted.sh main
cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings
rustup run 1.95 cargo fmt --all -- --check
typos
```

Na minha árvore: **tudo verde** (incluindo os arch gates — LOC caps dos painéis, `safe_clamp`,
`no_magic_numeric`). **19 mutações provadas** ao longo da linha; as que mais importam para você:
`from_bits` de volta · guarda do `stamp_wire_ids` fora · clamp por-borda · alça de volta ao
quadradinho — **cada uma vermelha no gate esperado**.

**Gates novos que você herda:**
`crates/ph2d-timeline/tests/detached_bindings.rs` (o crash + o destruidor de identidade) ·
`crates/ph2d-panel-timeline/tests/strip_ease_grip_seam.rs` (**pinta o painel de verdade** e mede o
alvo — `MockPanelHost::paint` devolve o hit index) · `shells/desktop/src/project_tests.rs` (o `App`
headless).

---

## §7 — O que NÃO está gateado (honesto, e é o mesmo buraco duas vezes)

1. **O lado do SAVE** (`project_save`) exige `gfx` — janela + GPU. **Causa raiz: o mundo do ECS
   mora DENTRO do `AppGfx`.** É isso, e só isso, que impede o shell de ser testável headless: o
   load é dirigível sem janela *porque* todo passo que precisa de `gfx` degrada para no-op, mas o
   save precisa do mundo para carimbar os nomes. **Recomendação (linha foundational, não enxerto):
   hoistar o `sim` do `AppGfx` para o `App`.** Destrava o gate honesto de uma classe inteira.
2. **Unicidade de nome não é imposta em todo caminho de spawn.** O `name_unique` é chamado em 4
   lugares (import, hierarquia, merge), mas **paths vetoriais** (`vec_entities.rs`, `"Path {id}"`)
   e **objetos Flip** (`flip_entities.rs`) spawnam sem consultá-lo. Dois objetos de mesmo nome
   colapsam no mapa `wire_id → entity` do `upkeep` (um por hash) e **duas tracks curariam no mesmo
   objeto**. ESPECULATIVO (lente A #8), **não corrigido** — mas é a fundação do W4.T6 inteiro.
   Follow-up.
3. **Track cujo objeto não voltou fica invisível no painel** (o snapshot pula binding `missing`).
   É comportamento **ratificado** pelo Enio — mas o toast do load conta essas tracks no "N".

---

## §8 — Fila da linha (para quem vier depois)

- **ADR-0115 DUPLICADO** (o meu, de composição de clips, e o do áudio, espectral): dois arquivos,
  dois assuntos, mesmo número — herdado, **não resolvido**, precisa de decisão do Enio (renumerar
  o do ÁUDIO é a recomendação: chegou 11 min depois e tem 9 referências contra 36). E **não existe
  gate contra número de ADR duplicado** — escrever um mata a classe.
- **W4.T4** — dock da timeline no `motion_timeline_slot` (**DESBLOQUEADO**: a linha Motion integrou).
- **Nesting** — o idioma 2D de verdade. É um ADR antes de ser código.
- Markers → signals · export.

---

## §9 — Ordem

**Não integre, não pushe, não faça ship sem ordem EXPLÍCITA do Enio** (CLAUDE.md §0.7). E lembre:
**o smoke desta linha é amanhã.**

---

## §10 — Pós-integração: as duas ABAS (`910404a0`, 2026-07-16, **pendente smoke**)

> Escrito depois de a linha ter integrado. Se você é o integrador, esta seção é o que mudou
> desde então; commits `910404a0` (código) + `78329f70`/`7c6409ec` (memória).

### O pedido, e o que ele escondia

O Enio (com screenshots): *"Penso que a time line com keys e com strips misturadas é confusa.
Melhor um modo isolado para lanes/strips e um checkbox na timeline para mudar o modo."* Depois,
perguntado sobre o padrão-ouro e o controle: **"b = Abas"**.

Investigar a queixa antes de atendê-la achou uma causa mais forte que "poluído": **a régua
significava duas coisas.** Uma key é carimbada no tempo do **CLIP** (`snapshot.rs`: as rows são
do clip ativo); um strip senta no tempo da **TIMELINE**. Mesma coluna de pixels, dois instantes.
Repro: `PH2D_STACK_SMOKE=1`, escolha **Right** no dropdown — keys em 0..3, strip em 2..5; com o
playhead em 4.0 o relógio do clip lê **2.0** e a régua desenhava 4.0 (um segundo *depois* do fim
de um clip de 3 s). Sem pilha os dois relógios são um só — por isso ninguém viu.

### A cerca de Chesterton (ADR-0115 R8)

O R8 decidiu **"sem modo, sem tweak mode"** e chamou dois modos exclusivos de *"dívida escondida
disfarçada de simplificação"*. **Ele fica de pé**: a rejeição é sobre um MODO, e o tweak mode do
Blender existe só porque os editores dele prendem numa Action por vez — o dropdown já resolve.
Uma **aba é uma vista**: muda o que se vê e o que a régua mede, nunca o que uma edição significa,
e o dropdown segue escolhendo o clip. O R8 errou o **corolário** (as metades podiam coabitar uma
vista) por não notar que coabitavam uma régua. **Emenda registrada no próprio ADR-0115**, não
escondida no commit.

Padrão-ouro conferido: Unity **não deixa** editar key na janela do Timeline (manda pra Animation
window) · Blender: NLA e Dope Sheet são **editores diferentes** · Premiere: Effect Controls · AE:
mistura num nível, **aba** por nível de nesting · Unreal (o mais parecido conosco) mistura e tem
usuário pedindo socorro.

### Superfície tocada (para o merge)

| Arquivo | O quê |
|---|---|
| `ph2d-timeline/src/apply.rs` | **`clip_playhead(doc, t)`** + `debug_assert_scratch_at` extraído (o `key_home` usa o mesmo) |
| `ph2d-timeline/src/snapshot.rs` | `clip_time: Option<f64>` · `stacked()` · **`rebuild` agora é `&mut TimelineState`** |
| `ph2d-timeline/src/lib.rs` | exporta `clip_playhead` |
| `ph2d-panel-timeline/src/tab.rs` | **novo** — `Tab::{Keys,Arrange}` + a tabela ÚNICA `TABS` |
| `ph2d-panel-timeline/src/transport_tabs.rs` | **novo** — a tira (split de LOC) |
| `ph2d-panel-timeline/src/geom_tests.rs` | **novo** — split de LOC do `geom.rs` |
| `geom.rs` · `ruler.rs` · `paint.rs` · `event.rs` · `state.rs` · `populate.rs` · `transport.rs` · `tracks.rs` · `box_select.rs` · `summary_paint.rs` · `stack_lane_paint.rs` | ver abaixo |
| `ph2d-editor-core/src/ids/chrome/timeline.rs` | `TIMELINE_TABS` / `_TAB_KEYS` / `_TAB_ARRANGE` |
| `ph2d-i18n/src/lib.rs` | `panel.timeline.tab.{keys,arrange}` |
| `shells/desktop/src/render_loop/mod.rs` | `rebuild(&mut self.timeline, …)` |
| `shells/desktop/src/stack_smoke.rs` | só docs (diz pra clicar **Arrange**) |

**Mudança de assinatura que cruza crates:** `TimelineViewSnapshot::rebuild(&mut TimelineState, …)`.
Se outra linha chamar `rebuild`, o merge quebra no compilador (bom) — o fix é `&mut`.

### As decisões que valem revisão

1. **Uma porta para "o clip toca aqui?"** — `clip_playhead` e `key_home` passam os dois por
   `stack_eval::sole_strip_of`. Uma régua que desenha playhead num instante onde o K **recusa**
   é uma régua que mente. Diferem no que devolvem onde toca: o `key_home` compõe o Time Remap da
   entidade; a régua não tem entidade a quem perguntar.
2. **`rebuild` PRIMA o scratch.** Segui primeiro o contrato existente ("o caller prima") e **4
   gates ficaram vermelhos na hora** — o `debug_assert` do módulo fez seu trabalho. Um publicador
   de view é o pior lugar possível pra um contrato de ordem escondido. Custo zero sem pilha.
   → [[feedback_a_view_publisher_must_not_require_a_primed_cache]]
3. **A aba é perguntada UMA vez, na raiz do `geom`** (`stack_h`/`summary_h`). Todo o resto
   (`content_h`/`row_bands`/`stack_bands`/`summary_band`) é construído dali, então "uma aba
   mostrando as duas metades" não é expressável.
4. **`ruler::clock_for` é PURO e testado.** O playhead é pintura, não widget — nenhum hit index o
   lembra, então sem extrair a decisão a linha não teria **oráculo nenhum**. Mesmo movimento que o
   `hit_plan` já fez. (Um gate meu foi **deletado** por não poder falhar: só afirmava que a
   fixture tinha zoom.)
5. **Sob pilha a régua do clip é read-only** — sem scrub, sem braces, sem markers. Não é cautela:
   **o inverso não existe** (um strip em loop manda muitos instantes da timeline num só instante
   do clip). Sem pilha, tudo funciona exatamente como sempre.
6. **`min_label_w` pergunta a aba** — a coluna tem mínimo por causa do que **vive** nela, e os
   controles de lane não existem na aba Keys.
7. **`drop_row_gestures`** — uma porta para o *hide* do painel e para a troca de aba.

### Gates

`clip_clock.rs` (7) · `view_tabs_seam.rs` (4, **clica** as abas pela pintura real) ·
`ruler::tests` (4) · `tab::tests` (4) · `geom_tests` (2 novos).
**Mutation-proof: 5 mutações, 5 vermelhos** (régua lê o playhead · régua escreve sob pilha ·
`clip_playhead` ignora o `sole_strip_of` · `geom` esquece a aba · `populate` esquece as abas —
essa derruba **só** o gate de clique, que é a divisão certa).

Suíte completa + clippy `--all-targets` + fmt + typos: **verde**. `cargo build`: verde.

### Smoke que o Enio deve rodar

```
cd Worktrees/line-anim && PH2D_STACK_SMOKE=1 cargo run -p ph2d-host-desktop
```
(o `-p` não é opcional: o workspace tem 27 binários e um `cargo run` seco recusa)
**L** abre o painel (aba **Keys**). Clique **Arrange** → as lanes/strips. No dropdown escolha
**Right** e volte a **Keys**: o playhead cai **sobre** as keys (não um segundo depois do fim), e
fora da janela do strip **não há playhead** — não há onde apontar.

### Aberto (novo, honesto)

- **Pan/zoom é UM só para as duas abas.** Sem pilha é o mesmo eixo (correto). Sob pilha são eixos
  diferentes compartilhando `view_start_s`/`px_per_s` — trocar de aba mantém o número, não o
  significado. Não incomodou no desenho; é o primeiro candidato se incomodar no smoke.
- **`F` (fit) na aba Arrange ainda ajusta às KEYS**, não aos strips (`view::apply_fit`).
- **Sem dica de recusa**: quando o clip não toca (ou toca duas vezes) a régua simplesmente não
  desenha playhead. O `refusal.rs` argumenta que recusa invisível ≈ bug; aqui a aba **Arrange**
  ao lado mostra os strips, que é a explicação visível — mas se o smoke disser que confunde, o
  `KeyRefusal::message()` já existe.
