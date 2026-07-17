# HANDOFF — integração `line/anim` → `main` (2026-07-16)

> Para o **agente integrador**, munido deste doc (DIRETRIZ §1.5.9). A linha está FECHADA e
> **não integra, não pusha, não roda ship** — isso é ordem EXPLÍCITA do Enio, e ele já a deu:
> *"vamos à integração ao main"*. Este doc diz o que entra, contra o que colide, e em que ordem.
>
> **Supersede** `HANDOFF_line_anim_integracao_2026-07-13.md` como o doc de integração DESTE lote
> (o §10 daquele arquivo descreve as abas; aqui está o quadro completo + a superfície de colisão).

---

## §0 — TL;DR

- **Branch:** `line/anim` @ `94382205`. Worktree: `Worktrees/line-anim`.
- **Delta:** 10 commits à frente de `main` @ `12ccaecd`; 5 atrás (**todos memória/docs de sculpt**).
- **`git merge-tree --write-tree main line/anim` é LIMPO em TODO o código.** O único conflito
  textual é **`project-memory/MEMORY.md`** — o índice, com append dos dois lados (resolução: manter
  as duas listas; padrão conhecido).
- **Nenhum bump de schema.** `DOC_VERSION` fica **4**, `PROJECT_SCHEMA` fica **13**. Projeto salvo
  continua abrindo. Nada serializado cresceu um campo (o `ActiveStrip.held` novo vive dentro do
  `StackScratch`, que é `#[serde(skip)]`).
- **`main` não contém NADA deste lote** — verificado por símbolo (§2). O integrador **não aplica em
  dobro**.
- **Suíte completa + clippy `--all-targets` + fmt + typos + `cargo build`: verde** na minha árvore.
  Rode-os de novo **na árvore combinada** (§5) — árvore limpa no texto ≠ árvore sã.

---

## §1 — O que a linha ENTREGA (2 lotes de código)

Este delta é a linha inteira **depois** da integração anterior (que levou a composição de clips, a
persistência e a identidade-por-nome para o `main`). São dois lotes:

### Lote A — as duas ABAS (`910404a0`, ADR-0115 R8 emendado)

O Enio: *"a timeline com keys e com strips misturadas é confusa. Melhor um modo isolado... b =
Abas."* Investigar a queixa achou uma causa mais forte que "poluído": **a régua significava duas
coisas.** Uma key é carimbada no tempo do **CLIP**; um strip senta no tempo da **TIMELINE**. Mesma
coluna de pixels, dois instantes. Sem pilha os dois relógios são o mesmo — por isso ninguém viu.

- `ph2d_timeline::clip_playhead(doc, t)` — o relógio do clip ativo, pela **mesma porta** que o K
  (`sole_strip_of`): não toca aqui / toca **duas vezes** = sem resposta, e a régua herda a recusa
  que o R9 já definiu. Sem pilha é `t` (fast path).
- `TimelineViewSnapshot.clip_time: Option<f64>` + `stacked()`. **`rebuild` agora é
  `&mut TimelineState`** (prima o scratch sozinho — um publicador de view é o pior lugar para um
  contrato de ordem escondido). **Assinatura que cruza crates** — ver §2.
- `ph2d_panel_timeline::tab` — `Tab::{Keys,Arrange}` + a tabela ÚNICA `TABS` (a tira pinta, o hit
  registra, o `populate` arma, o router casa — todos dela).
- `ruler::clock_for` (puro e testado): a Keys mede o clip, a Arrange mede a timeline. Sob pilha a
  régua do clip é **read-only** (sem scrub/loop/markers — o inverso não existe).
- Emenda registrada **no próprio ADR-0115 R8** (o *tweak mode* segue rejeitado; aba é VISTA, não
  modo).

### Lote B — o hold da lacuna + 3 pedidos de UI (`ec62fffa`, ADR-0115 **R10**)

**O bug (item 4 do Enio):** o fade de strips **sem sobreposição** saltava. Medido: `Left[0,3)`,
lacuna, `Right[4,7)` com fade-in → `t=3.9 x=-3.000`, `t=4.0 x=0.000` (**3 unidades num frame**).
Causa: **duas respostas discordando através de um pixel de régua.** A lacuna era *silêncio*; o
primeiro instante do fade (peso 0) respondia *repouso*. Fix: **a lacuna nunca foi silêncio** — o
strip que acabou segue afirmando o último frame dele (`ClipLane::hold_at`), e o fade cruza dali (o
`Hold` do Blender). Sobreposição intocada; hold forward-only; um strip **held não está tocando**
(`sole_strip_of` o pula, senão o K keyaria num strip que acabou). Registrado no **ADR-0115 R10**.

- **Renomear abre sobre o dropdown** — o campo pintava no canto do corpo; agora `paint_bar` reporta
  sempre o rect do chip (`ClipChip`).
- **Duplicar clipe** (botão ao lado do `+`) — `TimelineIntent::DuplicateClip` +
  `TimelineDoc::duplicate_clip`. É o que o `add_clip` **não pode** ser: bindings são do documento, um
  clip novo é sempre vazio; variação = copiar as curvas. Cópia profunda (KeyIds novos), loop viaja,
  nome único.
- **Botão `I` — inverter** (antes do `+M`) — `Track::reverse_about` + `Interp::reversed`. A metade
  sutil: o interp mora no key de **saída**, então inverter **move o interp de dono e o espelha**
  (todo ease-out vira ease-in). Pivô = `clip_end_seconds`, nunca `duration()`.

---

## §2 — Superfície de colisão (verificada, worktree a worktree)

### Contra o `main` ATUAL (`12ccaecd`)

**Zero conflito de código.** `main` mexeu apenas em `project-memory/` desde o fork. Confirmação por
símbolo — `main` **não tem** nada deste lote:

```
git grep -l clip_playhead main -- crates/   → 0
git grep -l duplicate_clip main -- crates/  → 0
git grep -l reverse_about main -- crates/   → 0
git grep -l hold_at main -- crates/         → 0
git grep -l TIMELINE_TAB_ARRANGE main       → 0
git grep -l clip_length_seconds main        → 0
```

**Único conflito textual:** `project-memory/MEMORY.md`. Os dois lados apenas **apendaram** linhas ao
índice. Resolva mantendo **as duas listas** — nunca escolha um lado (as memórias novas de ambos são
válidas). As 5 memórias de sculpt do `main` + as 4 minhas coexistem.

> **Nota histórica que importa:** o resumo da sessão acreditava que um lote anterior estava no
> `main`. Está — a **composição de clips, a persistência (project.rs) e a identidade-por-nome** já
> estão lá (`ClipStrip`, `sample_stack`, `install_from_project`, `resolve_entities` grepam positivo
> em `main`). Mas as 3 correções da **cauda** daquela sessão (`68abf9fb`/`dba030d1`/`d3b7d426`) e os
> 2 lotes desta **não estão**. Trate os 10 commits como o delta completo; o git aplica todos e
> nenhum duplica conteúdo do `main` (o merge-tree limpo prova).

### Contra `line/motion-value` (a conexão que o Enio citou — **AINDA NÃO integrada**)

Essa é a linha do agente do Motion. Ela **não toca nenhuma das minhas crates**
(`ph2d-timeline`/`ph2d-panel-timeline`/`ph2d-anim`/`ph2d-i18n`/`ph2d-editor-core`). Toca o shell:
`motion_bridge.rs`, `motion_state.rs`, `present.rs`, vários `motion_*_smoke.rs` — e **um arquivo em
comum comigo:**

**`shells/desktop/src/render_loop/mod.rs`.** A sobreposição é UM ponto, e é **append-only**: o
cluster de smoke-hooks no prólogo do frame (~linha 513).

```rust
        self.build_smoke();
        self.stack_smoke();        // ← MEU
        // line/motion-value acrescenta AQUI:
        // self.motion_path_smoke();
        // self.motion_delay_smoke();
        // self.motion_fx_smoke();
```

**Resolução (se ambas integrarem): manter TODAS as chamadas de smoke.** São métodos independentes,
idempotentes, latched por env var — a ordem entre eles não importa. Mergiraf resolve; à mão é
trivial. Minha outra mudança no `mod.rs` (o `rebuild(&mut …)`, ~linha 986) está a **470 linhas** do
cluster e **não colide** com nenhum hunk do motion-value (os deles: 511, 531, 724, 2583).

**Colisão semântica: NÃO HÁ.** O `line/motion-value` **não toca o relógio** (`ticks_owed`/`Playhead`/
`MotionTransport` — o W4.T7 "relógio único" que liga o Motion ao Playhead já está no `main` e nenhum
dos dois o altera). O único hit de grep por "clock" no diff dele é um **comentário**. Motion e
timeline continuam cozinhando no mesmo `Playhead` sem que nenhum lado mude a regra.

---

## §3 — As MINAS SEMÂNTICAS (merge limpo no texto ≠ árvore sã)

1. **`TimelineViewSnapshot::rebuild(&mut TimelineState, …)`** — a assinatura mudou de `&` para
   `&mut` (o rebuild prima o scratch). Se QUALQUER linha futura chamar `rebuild`, o merge quebra no
   **compilador** (bom — não silencioso). O fix é `&mut`. Hoje o único chamador de produção é
   `render_loop/mod.rs` (já ajustado); os testes chamam com `&mut st`.

2. **O `debug_assert_scratch_at`** cobra que o scratch esteja primed em `t` antes de `clip_playhead`/
   `key_home`/`clip_time` sob pilha. Se um caminho novo ler o relógio do clip sem `prime_stack`
   antes, **os testes de debug estouram** (foi assim que peguei 4 gates ao estender o contrato). Não
   é bug — é o contrato cobrando. Prime antes de ler.

3. **`ActiveStrip.held`** — o strip que segura contribui para a POSE mas **não está tocando**. Todo
   lugar que pergunta "onde este clip está agora" (`sole_strip_of`, e por transitividade o K e a
   régua) já pula `held`. Um consumidor novo do scratch que **iterar `scratch.active` sem checar
   `.held`** vai tratar um strip morto como vivo. É `pub(crate)`, então o raio é a própria crate.

4. **`Interp::reversed` / `Track::reverse_about`** — o interp mora no key de **saída**. Qualquer
   operação nova que reordene keys no tempo (não só reverse) tem a mesma armadilha: o atributo de
   segmento troca de dono. O oráculo dos gates é a **propriedade** (`reversed(span-t) ==
   original(t)`), não a escrituração — reuse esse padrão.

---

## §4 — Gates (rode na ÁRVORE COMBINADA, não só na minha)

Novos neste lote (todos **mutation-proved** — 13 mutações, 13 vermelhos, ver os commits):

| Arquivo | Prova |
|---|---|
| `ph2d-timeline/tests/clip_clock.rs` (7) | o bug da régua como número (4.0→2.0); sem pilha = identidade; NotPlaying/PlaysTwice; "a régua e o K concordam sobre SE o clip toca" |
| `ph2d-timeline/tests/lone_fade.rs` (6) | o salto como um LIMITE (passo/frame < 0.1 a 240 Hz vs snap de 3.0); a lacuna segura; path-independence; overlap intacto; primeiro strip entra do repouso; held não toca |
| `ph2d-anim/tests/reverse.rs` (8) | `reversed(span-t)==original(t)` ponto a ponto, por variante de `Interp` |
| `ph2d-panel-timeline/tests/view_tabs_seam.rs` (10) | um teste **CLICA** cada aba + Duplicate + `I` pela pintura real; cada aba registra só a sua metade; régua do clip sem scrub sob pilha; rename SOBRE o chip |
| `ph2d-timeline/tests/seam_determinism.rs` | atualizado: o número que o R10 mudou (peso-0 = pose SEGURADA, não repouso) — a espinha (pose CERTA, não só consistente) fica |
| `ruler::tests` / `tab::tests` / `geom_tests` | o `clock_for` puro; a tabela de abas; cada aba dá altura só à sua metade |

Comando de paridade (o integrador roda no fim da jornada, DIRETRIZ §1.5.4):

```
cd <arvore-combinada> && ./scripts/ship.sh
```

`ship.sh` = fmt + clippy `--all-targets`+features + machete + deny + audit + nextest + typos. **Não
pushe antes de verde.** Se vermelho fora dos meus arquivos, é interação de merge — DIRETRIZ §1.5.5.

---

## §5 — Smoke que o Enio já aprovou (para o integrador re-conferir pós-merge)

```
cd <arvore-combinada> && PH2D_STACK_SMOKE=1 cargo run -p ph2d-host-desktop
```

(o `-p` não é opcional: 27 binários no workspace.) **L** abre na aba **Keys**; **Arrange** mostra os
strips. Para ver o hold da lacuna: na Arrange **separe** os dois strips e ponha um fade num deles —
era o caso que saltava. Para ver as abas consertarem a régua: no dropdown escolha **Right** e volte
a **Keys** (o playhead cai sobre as keys, não um segundo depois do fim do clip).

---

## §6 — Ordem de integração (recomendação)

1. **`line/anim` primeiro, se possível.** Ela é 10 commits de código sobre um `main` que não tocou
   nenhum dos seus arquivos → merge trivial (só o índice `MEMORY.md`).
2. **`line/motion-value` depois.** Quando ela vier, o único ponto compartilhado é o cluster de
   smoke em `mod.rs` (append-only, mantenha todas as chamadas). Sem colisão semântica.
3. Se o Enio mandar integrar `motion-value` **antes**, tudo bem — a resolução do `mod.rs` é a mesma
   nos dois sentidos, e nenhuma das duas mexe no relógio.

**Aberto que NÃO bloqueia a integração** (herdado, honesto):
- W4.T4 (dockar a timeline no `motion_timeline_slot`) — desbloqueado, é a próxima etapa que sugeri.
- O lado do **SAVE** não é gateado (`project_save` exige janela + GPU; raiz = mundo dentro do
  `AppGfx`, um hoist foundational).
- O **hold é uma política só** (o Blender tem `Hold`/`Hold Forward`/`Nothing` por strip). Nenhuma
  cena pediu a escolha ainda.

---

## §7 — Checklist do integrador

- [ ] `git merge line/anim` no `main` (ou o fluxo `--ff-only` + gate testado da DIRETRIZ §1.5.3).
- [ ] Resolver `project-memory/MEMORY.md` mantendo **as duas** listas de índice.
- [ ] `cargo build` + `./scripts/ship.sh` na árvore combinada → verde.
- [ ] Smoke `PH2D_STACK_SMOKE=1` (§5) — as abas, o hold, duplicar, o `I`, rename sobre o chip.
- [ ] Se `motion-value` também entrar nesta jornada: manter todas as chamadas `*_smoke()` no
      cluster de `mod.rs`.
- [ ] Ship + push **1×** no fim da jornada, só por ordem do Enio (§0.7 / DIRETRIZ §1.5.4).
