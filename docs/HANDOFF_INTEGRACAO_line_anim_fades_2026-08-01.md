# HANDOFF DE INTEGRAÇÃO — `line/anim`, a wave dos FADES (2026-08-01)

> Para o **agente integrador**. A linha está FECHADA, todos os smokes foram aprovados pelo
> Enio, e ela **não integra nem pusha sozinha** (CLAUDE.md §0.7). Leia o §0 e o §1 antes de
> qualquer comando: eles são os dois lugares onde uma integração desta linha pode dar errado.

---

## §0 — Cartão de identidade

| | |
|---|---|
| **Branch** | `line/anim` |
| **Worktree** | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-anim` |
| **Commits** | **11** (`git log main..line/anim`), o mais antigo `8c2395574`, o tip `37c445fc1` |
| **Arquivos** | 46 · +3396 / −428 |
| **`PROJECT_SCHEMA`** | **46 — INTOCADO** (conferido por `git diff`, não por auto-relato) |
| **`DOC_VERSION`** | **17 — INTOCADO** (idem) |
| **`Cargo.toml`** | **ZERO tocados** ⇒ nenhuma dep nova, nenhuma crate nova |
| **ADR** | **nenhum** ⇒ esta linha fica **fora** de qualquer disputa de número |
| **Contrato congelado** | **intacto** — nenhum arquivo de `ph2d-nodegraph`/`-tool-traits`/`-vector-doc`/`-vector-traits`/`*contracts*` aparece no diff |

⚠️ **O cartão acima é o que torna esta integração barata.** As três colisões que já custaram
caro nesta janela (número de schema · número de ADR · `Cargo.lock`) **não se aplicam aqui**.

---

## §1 — A superfície de conflito: **duas crates foundational, e a forma é APPEND-ONLY**

O resto do diff mora em `ph2d-timeline` / `ph2d-panel-timeline` / `shells/desktop/src/render_loop/*timeline*`,
que são a casa desta linha. O que exige atenção:

### 1.1 `ph2d-editor-core` (8 arquivos)

| Arquivo | O que a linha fez | Forma |
|---|---|---|
| `ids/menus_timeline.rs` | +1 `NodeId` (`CTX_MENU_TL_FADE_SMOOTH`), +1 tabela (`TIMELINE_FADE_MENU`), +2 type alias, +`ALL_TIMELINE_MENUS` | **append** |
| `interaction/types.rs` | +4 códigos de ZONA `u8` (3, 4, 7, 8) + `TimelineInterpScope::StripFade` + `menu_table()` | **append** + 1 variant |
| `screens/hero/pre_populate.rs` | a lista de 10 tabelas escrita à mão virou `ALL_TIMELINE_MENUS.iter()` | **−12 / +3 linhas** |
| `screens/hero/tests.rs` | `every_timeline_menu_row_is_registered_and_hittable` reescrito | substituição |
| os outros 4 | roteamento do escopo `StripFade` | append |

⚠️ **O único ponto de merge realmente sensível é o `pre_populate.rs`**: a linha REMOVEU uma
enumeração. Se outra linha ADICIONOU uma tabela àquela lista no `main`, o merge textual pode
resolver limpo e **perder** a tabela dela. **Confira:** depois do rebase, toda tabela que o
`main` listava tem de estar em `ALL_TIMELINE_MENUS` — o gate
`every_timeline_menu_row_is_registered_and_hittable` **falha** se não estiver (é literalmente
o defeito que esta wave consertou).

⚠️ **Códigos de zona em uso**, para o caso de uma linha paralela ter inventado outro:
`TIMELINE_EDGE_{L,R,T,B}` = 1/2/4/8 (bitmask) · `TIMELINE_STRIP_FADE_{IN,OUT}` = **3, 4** ·
`TIMELINE_STRIP_FADE_BAND_{IN,OUT}` = **7, 8**. São espaços de numeração DIFERENTES (o edge é
bitmask de aresta, o fade é um enum de zona), mas os dois viajam no mesmo `u8` de hit.

### 1.2 `ph2d-anim` (2 arquivos)

- `src/easing.rs`: **+1 método inerente** `Easing::mirrored()`. Puro append, nada renomeado.
- `tests/easing.rs`: **arquivo NOVO**.

### 1.3 `project-memory/`

`feedback_integration_order_comes_from_measured_overlap.md` foi editado (commit `8c2395574`).
⚠️ **É uma lista compartilhada** — só ADICIONE no merge, nunca remova o que o `main` trouxe
([[feedback_a_shared_list_is_merged_against_todays_main]]).

---

## §2 — O que a linha ENTREGA (11 commits, sete frentes)

1. **Cada fade tem CURVA autorada** — menu de easing no botão direito sobre a cunha do fade
   (as mesmas opções dos clips), com a curva escolhida **desenhada dentro da cunha**.
2. **O menu abre no CORPO do fade**, não só na alça — e a cunha de uma sobreposição, que não
   tem alça, passou a ser alcançável.
3. **Um fade em cada ponta DIVIDE a travessia da costura** — antes o fade da direita era
   descartado e o objeto ficava parado.
4. **O fade externo FINAL provoca a transição sob PingPong** (medido: `5,00 5,00 5,00 5,00`,
   parado, com `rest` = a pose).
5. **O fade externo INICIAL também** — era o MESMO defeito, e a lei de 2026-07-23 que parecia
   proibi-lo é sobre um **vão SECO**, não sobre o modo.
6. **Um easing por crossfade** (autorável pelas duas cunhas) · **a row Smooth registrada** ·
   **a costura vira UMA travessia** (o objeto PARAVA na volta: velocidade medida `0,000`).
7. **O easing ESPELHA quando o playhead volta a zero**, e o desenho decorativo junto.

---

## §3 — Superfície pública que MUDOU (o que outra linha pode estar chamando)

| Símbolo | Antes | Agora |
|---|---|---|
| `ClipLane::weight_at_with` | `(i, t, Option<Easing>)` | `(i, t, FadeCtx)` |
| `ClipLane::hold_at` | `(t, loop_range, wraps)` | `(t, loop_range, wraps, reversed)` |
| `ClipLane::seam_curve` | `-> Option<Easing>` | **renomeado** para `seam() -> Option<Seam>` |
| `ph2d_timeline::ramp` | existia | **removido** (sem chamador; `ramp_with(.., None)` é ele) |
| `TimelineIntent` | — | +`SetStripCurve { lane, id, edge, curve }` |
| `StripView` | — | +`seam: Option<SeamSlice>` (**6 literais** no repo precisaram do campo) |

**Novos e aditivos:** `Seam` · `SeamSlice` · `FadeCtx` · `ClipLane::{curve_owner, effective_curve}` ·
`TimelineDoc::{reverse_play, set_reverse_play}` · `Easing::mirrored` · `fade_ramp` (já era pública).

⚠️ **`weight_at(i, t)` não mudou** e continua significando *"para a frente, sem costura"* —
é por onde a maioria dos testes passa.

---

## §4 — Mudanças de COMPORTAMENTO (todas smokadas pelo Enio)

1. **A costura de um loop com fade nas duas pontas MOVE** — antes o objeto parava na volta;
   a pose da volta passou de `f` linear para `curva(f)` (medido: −1 → **−1,75** com janelas 3:1).
2. **Sob PingPong as duas pontas externas agora viajam** (antes decaíam para o `rest`).
3. **Numa sobreposição, autorar o easing pela cunha de SAÍDA passou a ter efeito** (escrevia
   um campo inerte).
4. **Andando para trás, um easing assimétrico espelha.** ⚠️ **Invisível no default** — o
   `smoothstep` de fábrica é o seu próprio espelho.
5. Nas faixas EXTERNAS do fade, um press agora arrasta o strip (antes começava marquee).

⚠️ **Dois gates tiveram número/direção CORRIGIDOS**, com o porquê escrito ao lado (não é
afrouxamento): `under_a_loop_the_last_strips_curve_governs_the_head_fade` (o sinal inverteu
porque a cabeça passou a correr a segunda fatia da curva) e
`the_split_follows_the_two_window_lengths` (−1 → −1,75).

⚠️ **`fade_fingerprint` e `fade_fingerprint_channels` ficaram VERDES no MESMO hash**, do
começo ao fim da linha. É a prova executável de que a superfície de fade que o Enio chama de
preciosa (crossfade + `lead_out` + container aninhado) **não foi tocada**.

---

## §5 — Gate de fechamento (rode NA ÁRVORE COMBINADA)

```bash
cd <worktree-de-integração>
cargo fmt --all
cargo clippy --workspace --all-targets      # tem de sair LIMPO
cargo test --workspace                      # e em DEBUG, não só release
cargo test -p ph2d-timeline --test fade_fingerprint     # o guardião
cargo test -p ph2d-editor-core --lib every_timeline_menu_row   # o §1.1
```

⚠️ **Rode a suíte inteira, sem `| head`** — nesta sessão um `head -40` escondeu a única
falha que havia.

**Flakes conhecidas, PRÉ-EXISTENTES, não desta linha** (re-rode isoladas antes de suspeitar
do merge):
- `ph2d-timeline/tests/nesting_clock.rs::the_cost_of_depth_is_linear_not_explosive` — gate de
  RAZÃO sensível a carga.
- `ph2d-sculpt3d --test measure_brush_kernel` — gate de RELÓGIO (13,4 s isolado, verde); a
  crate **nem depende** da timeline. Falhou sob a suíte paralela nesta máquina, passa sozinho.

---

## §6 — Smokes (todos APROVADOS pelo Enio; um re-smoke deve olhar isto)

Não há env var nova. A wave inteira se julga na **timeline, aba Arrange**, com duas strips e
fades autorados:

1. **Menu do fade**: R-click em qualquer ponto do corpo da cunha (não só na alça) abre o menu
   de easing. **Smooth (Default) tem de responder** — foi a row que nascia muda.
2. **Crossfade de sobreposição**: uma curva só, autorável pelas duas cunhas.
3. **Loop com fade nas duas pontas**: a curva é **UMA**, começa na cunha final e termina na
   inicial, e o objeto **não dá aquela paradinha** na volta.
4. **PingPong**: as duas cunhas externas movem o objeto — inclusive com a sprite parkada
   exatamente na pose em que a animação começa (o caso que as matava).
5. **Direção**: com um easing **assimétrico** autorado (Ease In, Bounce Out), a perna de volta
   do PingPong tem de *sentir* igual à ida, e a curvinha desenhada vira junto. Com o Smooth
   padrão **nada muda, e isso é correto**.
6. **Vão SECO sob PingPong** (strip sem fade): continua **mudo** — é a lei de 2026-07-23, e é
   a metade que protege o fix do Enio daquele dia.

---

## §7 — Linha pronta para a **§5 do `CLAUDE.md`** (APENDAR na entrada **Timeline**)

> **⬛ OS FADES GANHARAM CURVA, E A COSTURA VIROU UMA TRAVESSIA — `line/anim` INTEGROU
> (2026-08-01, 11 commits, todos os smokes aprovados; handoff
> [`docs/HANDOFF_INTEGRACAO_line_anim_fades_2026-08-01.md`](docs/HANDOFF_INTEGRACAO_line_anim_fades_2026-08-01.md)):**
> o fade de um strip passou a ter **curva autorada** (menu de easing no R-click sobre o CORPO
> da cunha, com a curva desenhada dentro dela). ⚠️ **Numa SOBREPOSIÇÃO o crossfade tem UM
> easing só** — a curva é a de quem CHEGA e o lado que sai é o **complemento explícito** dela,
> o que fez a lei `w_in + w_out == 1` passar a valer por CONSTRUÇÃO em vez de por acidente da
> antissimetria do `smoothstep` (⚠️ e a identidade `1 − s(u) == s(1−u)` foi MEDIDA e é FALSA
> em `f64`: **626 962 de 1 000 000 de amostras diferem, 1 ulp**). ⚠️ **A costura de um loop
> com fade nas DUAS pontas era duas curvas em S, uma por ponta, e o objeto PARAVA na volta**
> (velocidade medida `-0,299 → 0,000 → recomeça`); agora é **UMA travessia** parametrizada
> pela janela inteira, com a cauda tocando/desenhando a fatia `[0, f]` e a cabeça `[f, 1]` —
> *a curva começa na fade final e termina na inicial* —, e a pose da volta passou a cair onde
> a CURVA põe `f` (−1 → **−1,75** com janelas 3:1). ⚠️ **Sob PingPong as DUAS pontas externas
> viajam** (antes decaíam para o `rest`, invisível para quem parka a sprite na pose da
> animação: medido `-3,000` a faixa inteira) — e a lei de 2026-07-23 que parecia proibi-lo é
> sobre um **VÃO SECO**, não sobre o modo: sem fade a lane segue MUDA. ⚠️ **E o easing
> ESPELHA quando o playhead volta a zero** (a perna de volta de um ping-pong), pose e desenho
> — o espelho no tempo de um easing é **trocar `In` por `Out`**, MEDIDO em 1,18e-15 nas 11
> famílias, e por isso o `smoothstep` de fábrica (auto-espelhado, como todo `InOut` e como o
> `Linear`) **não muda nada**: a inversão só aparece num easing autorado assimétrico. A
> direção viaja como **transiente no doc** (`#[serde(skip)]` ⇒ **nenhum schema se move**),
> estampada pela `timeline_bridge` a partir do `Playhead`, e **pausado é para a FRENTE** (um
> scrub para trás é leitura, não reprodução). ⚠️ **E a row `Smooth (Default)` nascia MUDA**:
> o `pre_populate` ENUMERAVA dez tabelas de menu e a do fade era a décima-primeira — as outras
> quatro rows dela herdaram o registro por acidente, por viverem também na tabela do segmento;
> o gate que existia para isso enumerava DUAS tabelas e estava verde pelo mesmo motivo. Hoje
> há **uma lista** (`ALL_TIMELINE_MENUS`) e o gate varre ela **e** as tabelas que cada escopo
> pinta. **`PROJECT_SCHEMA` 46 e `DOC_VERSION` 17 INTOCADOS · zero `Cargo.toml` · nenhum ADR ·
> contrato congelado intacto** (por grep). ⚠️ Os dois `fade_fingerprint` saíram **verdes no
> mesmo hash** — a prova de que a superfície preciosa não foi tocada. **Aberto:** criar uma
> sobreposição não RESSETA a curva autorada de antes (o valor sobrevive e governa o crossfade,
> o mesmo precedente do `ease_in`/`ease_out`) — se o Enio quiser que nascer sobreposto volte
> ao Smooth, é decisão de produto.

---

## §8 — Aberto (nomeado, **não** construído — nada disto bloqueia)

1. **Sobrepor duas strips não reseta a curva autorada.** Ela sobrevive e governa o crossfade
   (precedente do `ease_in`/`ease_out` sob sobreposição). Decisão de produto.
2. **A reparametrização da costura vale para o loop que ENVOLVE.** Reproduzir um loop de
   trás para frente (`rate < 0`) espelha a curva mas **não** inverte a geometria da divisão
   (`f`); é caso de borda, e está escrito no doc do `Seam`.
3. **Cliff nomeado, igual nas duas bordas:** um fade que para 10 ms depois do começo do
   alcance não é "a abertura" (nem um `lead_out` que para 10 ms antes do fim é "o fecho").
   Uma tolerância seria um número que ninguém mediu.

---

## §9 — Protocolo

1. `cd` na worktree de integração **em todo comando** (a cwd do Bash escorrega para a árvore
   primária — aconteceu duas vezes nesta linha).
2. Rebase **commit a commit** (`git rebase main`), não um merge só: o forecast de conflito é
   NET e esconde o commit que de fato colide.
3. Rode o §5 **na árvore combinada** — os arch-gates de `shells/desktop/tests/` **não** são
   alcançados por um `cargo test -p` por crate, e é assim que uma linha fecha com dois deles
   vermelhos no próprio tip.
4. Ship e push **só por ordem explícita do Enio**.
