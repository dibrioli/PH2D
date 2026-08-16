# HANDOFF DE INTEGRAÇÃO — `line/Vector`, a DOBRA e as três medições (2026-08-16)

> **A linha NÃO integra e NÃO faz ship.** Este documento passa ao **agente integrador** o que
> evita conflito e regressão. Formato: [DIRETRIZ §1.5.9](../../IntegracaoMultiAgente/DIRETRIZ.md).

---

## 1. Identidade

| | |
|---|---|
| branch | `line/Vector` |
| HEAD | o **tip de `line/Vector`** — ⚠️ **não um sha escrito aqui**: os últimos commits são este próprio documento, então um literal envelheceria a cada correcção dele |
| último commit de **CÓDIGO** | **`6f66a8554`** |
| merge-base com `main` | **`08e3c84c9`** |
| commits de código | **11** (de **14** no total; os outros 3 são este documento e o plano) |
| diff de **código** | **64 arquivos, +3.113 / −712** (`main...HEAD -- ':!docs'`) |

⚠️ **Esta caixa já esteve errada duas vezes, e as duas por motivos que valem mais que o número.**
A primeira dizia *"daí para cima é só `docs/`"* e o smoke produziu **dois commits de fonte** depois
dela. A segunda rotulava a linha como *"diff de **código**"* medindo o total **com `docs/` dentro**
(65 / +3.241 / −732 era `main...HEAD` sem pathspec) — *um rótulo que promete um recorte e mede
outro é pior que um número velho, porque ninguém o vai reconferir.* **Re-meça no dia da ordem.**

⚠️ **O `main` NÃO andou desde o fork** (`merge-base == main == 08e3c84c9`), então **hoje** a
integração é um `--ff-only` trivial. **Esta caixa envelhece.** É a mesma frase que os handoffs da
`line/sculpt3d` (08-09) e da `line/physics` (08-12) traziam e que a ordem encontrou falsa — num
caso o `main` tinha andado **298** commits. **Re-meça no dia da ordem**, não acredite nesta linha:

```bash
git rev-list --count $(git merge-base main HEAD)..main   # zero = ff-only
git diff --name-only main...HEAD | sort > /tmp/linha.txt
git diff --name-only $(git merge-base main HEAD)..main | sort > /tmp/main.txt
comm -12 /tmp/linha.txt /tmp/main.txt                    # a interseção REAL
```

---

## 2. Foundational / compartilhado tocado, e por quê

**Duas crates foundational e a shell.** Tudo **aditivo**; nenhuma assinatura pública existente
mudou de forma.

### `ph2d-editor-core` (12 arquivos)

| arquivo | o que entrou | aditivo? |
|---|---|---|
| `widget/section_header/body.rs` **(NOVO)** | `widget::SectionFold` — a porta da dobra do CORPO | ✅ novo módulo |
| `widget/section_header/body_tests.rs` **(NOVO)** | os gates dela | ✅ |
| `widget/section_header/mod.rs` | `pub mod body;` + re-export | ✅ append |
| `widget/mod.rs` | re-export de `SectionFold` | ✅ append |
| `interaction/hit.rs` | **`HitIndex::push_clip`/`pop_clip`** (+50 linhas) | ✅ métodos novos |
| `interaction/state/mod.rs` | campo **`fold_body_h`** no `WidgetStore` | ⚠️ **campo apendado** — ver §3 |
| `interaction/state/store_core.rs` | a construção dele | ⚠️ **sítio de construção** — ver §3 |
| `interaction/state/chrome_ops.rs` | `section_body_h` / `remember_section_body_h` | ✅ métodos novos |
| `motion.rs` | **`law_of`** privado, e o `advance` deixa de colectar um `Vec` | ✅ corpo, sem API nova |
| `tests/ui_motion_no_alloc.rs` **(NOVO)** | o gate de contador (dhat) | ✅ |
| `tests/architecture_panel_loc_cap.rs` | tolerâncias que **encolheram** | ✅ só desce |
| `tests/hr12_widgets_a11y.rs` | ajuste da varredura | ✅ |

### `ph2d-ui-testkit` (1 arquivo)

- **`MockPanelHost::settle_section_folds()`** — método **NOMEADO**, nunca um `store_mut()`. Ele
  responde a UMA pergunta (*e se o artista esperar?*) em vez de abrir o store para um gate semear o
  que depois vai "provar" — o mesmo argumento do `set_panel_scroll`.

### `shells/desktop` (4 arquivos)

| arquivo | o que |
|---|---|
| `ui_motion_smoke.rs` | a cena **`=3`** (a DOBRA) + `LAST_SCENE: u32 = 3` |
| `ui_motion_smoke_tests.rs` **(NOVO)** | o gate que pina o filtro do roteador |
| `probe_cursor_grab.rs` **(NOVO)** | a sonda do §4.3, `#[ignore]` |
| `main.rs` | `mod` das duas acima |

### Os dez painéis (39 arquivos)

`inspector` · `painter-layers` · `vector` · `sculpt3d` · `physics` · `audio-editor` ·
`audio-mixer` · `wet-tuning` · `motion-params` · `authored` — todos **vestindo** a porta nova; e
`grid-snap` (5 arquivos), que é **correção própria** (ver §5 do commit `c77af7cec`).

---

## 3. Símbolos que podem COLIDIR com outra linha

⚠️ **NENHUM valor literal novo.** Medido, não afirmado:

| espécie | medido |
|---|---|
| `NodeId(NNN)` literais novos | **zero** (`git diff -- '*/ids.rs'` sem uma linha `+…NodeId(`) |
| scrollbar ids | **nenhum novo** (o último segue **841**) |
| variantes de enum | **nenhuma** (o `Role` não ganhou membro — o `Surface` já estava no `main`) |
| chaves de token / i18n | **`ph2d-i18n` com diff VAZIO** |
| ADR | **nenhum** ⇒ a linha fica **FORA de toda disputa de número** |

**O único ponto de merge sensível é ESTRUTURAL, não numérico:**

> ⚠️ **`WidgetStore` ganhou o campo `fold_body_h`** — declarado em
> `interaction/state/mod.rs:300`, construído em `interaction/state/store_core.rs:51`. Uma linha
> que apende **outro** campo ao mesmo struct toca os **dois** sítios, e o segundo é um literal de
> construção: é ali que o git conflita. **Resolver é UNIÃO** (os dois lados só acrescentam);
> ficar com um lado deixa o struct com um campo que o construtor não preenche — o que **não
> compila**, e é o modo de falha barato.

⚠️ **`section_header.rs` já era um DIRETÓRIO** antes desta wave (a F4a o partiu); esta acrescenta
o irmão `body.rs`. Uma linha que escreva num `section_header.rs` **solto** funde limpo contra um
arquivo que já não existe — a família do corte do `project.rs` que a `line/Vector` pagou em 04/08
e a `line/sculpt3d` em 15/08.

---

## 4. Contratos congelados encostados

**NENHUM**, medido por `git diff` e não por auto-relato:

```
git diff --stat main...HEAD -- crates/ph2d-nodegraph/ crates/ph2d-core/src/tool.rs
→ (vazio)
```

E a superfície de colisão inteira, **medida**:

| grandeza | estado |
|---|---|
| `PROJECT_SCHEMA` | **84 INTOCADO** (`project.rs` **e** `project_schema.rs` com diff vazio) |
| tripla | **`(84, 13, 14)`** |
| `VEC_SCENE_SCHEMA_VERSION` | **14** intocado |
| `FLIP_SCHEMA_VERSION` | **13** intocado |
| contrato congelado (nós · tools) | **intocado** |
| registro do `ph2d-ecs` | **INTOCADO** ⇒ os **três** espelhos também |
| `Cargo.toml` / `Cargo.lock` | **ZERO** ⇒ nenhuma crate nova, **nenhuma dep externa nova** |
| `ph2d-i18n` | **intocado** ⇒ a cadeia `vector::tr(k).or_else(sculpt3d::tr)` fica intacta |
| ADR | **nenhum** |

⚠️ **Isto fecha o item 5 do §8 do plano**, que dizia ser *"afirmação a **conferir por `git diff`
no fecho**, não a acreditar agora"*. Conferida. Passa.

---

## 5. O que só o `ship.sh` pega (o gate de integração NÃO roda)

- **`cargo fmt --all -- --check`** — rodado nesta linha: **EXIT 0**. ⚠️ Mas o `main` já esteve
  fmt-vermelho por dívida de **outra** linha (medido em 16/08: 9 hunks em 4 arquivos, todos do
  ROUGH da `line/Painter`) — *um vermelho que só o ship vê é invisível entre integrações*.
- **`cargo machete`** — esta linha não acrescenta dep nenhuma (`Cargo.toml` intocado), então não
  há superfície nova; a varredura ainda vale para o resíduo do `main`.
- **`cargo deny` / `cargo audit`** — sem dep nova; RUSTSEC é do calendário, não do diff.
- **`typos`** — os docs desta wave são pt-BR com acentos; os `eprintln!` do smoke são ASCII-only
  (`e'`, `nao`, `carater`) **de propósito**, mais `⚠️`/`⭐` — nunca `→` num literal de Rust
  (o `no_tofu_glyphs`).

---

## 6. Ordem, dependências e **o que smoke-testar**

### Os 11 commits, em ordem, e a dependência entre eles

| # | commit | depende do anterior? |
|---|---|---|
| 1 | `708b4e641` F4b — o CORPO dobra; o Inspector inteiro veste | — (traz a porta) |
| 2 | `160049a95` F4b — sculpt3d · física · Vector | **sim** (usa a porta) |
| 3 | `7565e3a3b` F4b — wet-tuning · mixer · as doze do painter | **sim** |
| 4 | `9efec5ff8` F4b — o LAÇO plano; a dívida de LOC da própria wave | **sim** |
| 5 | `3b77ac158` os quatro vermelhos da varredura completa + o relógio no harness | **sim** |
| 6 | `0e7d7e5ba` a cena `=3` e o roteiro do carácter | **sim** |
| 7 | `6e20b7155` a sonda do `set_cursor_grab` (§4.3) — **(B) recusada por medição** | independente |
| 8 | `c77af7cec` o intervalo que o COMMIT enforça chega à lei do arrasto | independente |
| 9 | `703c2d1a8` a POPULAÇÃO governa o custo, não o voo | independente |
| 10 | `c5f8aa8a5` o roteiro da DOBRA fala a língua do ARTISTA | **do 6** (é o texto da cena) |
| 11 | `6f66a8554` o `reduced motion` **PARA** o roteiro | **do 6** |

⚠️ **Os commits 1-6 são uma cadeia**: um rebase que os reordene quebra a compilação (o 2 usa o que
o 1 cria). Os 10-11 dependem do 6 (editam a cena que ele cria). Os 7-9 são independentes entre si
e da cadeia.

### ✅ **SMOKADO E APROVADO** (Enio, 2026-08-16)

A cena **`=3`** foi julgada e passou. **Mas a primeira corrida REPROVOU**, e a causa não estava no
produto — está aqui porque ela mudou código e é a razão de os commits 10-11 existirem.

**O veredito foi *«não há transições, aparecem e desaparecem subitamente»*, e o produto estava
CERTO:** o `~/.ph2d/prefs.txt` do Enio tinha **`reduced_motion=1`**. A dobra é `Role::Surface`, e
`Surface` + reduced devolve `None` do `law_of` — **sem mola, tudo chega no quadro em que muda, por
projecto**, pinado pelo gate pré-existente `reduced_motion_still_takes_the_surface`. E o **passo 3
do meu próprio roteiro manda ligá-lo**, para provar exactamente isso.

⚠️ **O defeito era meu, e tinha duas metades.** A cena **já imprimia `reduced motion: true`** — como
*readout* neutro, no meio de outras linhas. Ela parava quando faltavam dobras (defeito estrutural) e
**não parava quando a preferência desliga a coisa inteira que ela mede**. *Imprimir um facto não é
PARAR sobre ele.* E a segunda: quem deixasse o interruptor ligado de uma corrida anterior **começava
no passo 3 a achar que corria o passo 1** — a preferência é persistida fora do repo, logo invisível
a toda varredura.

**A cura (commit 11):** um **PARE** antes do despacho de cena, que nomeia o interruptor, dá os dois
caminhos para o desligar e avisa que o passo 3 o religa no fim. O gate
`the_reduced_motion_guard_stops_the_script_before_the_dispatch` lê o fonte — **a posição é
load-bearing**: um guard depois do `match level` compilaria, passaria na suíte e imprimiria o
roteiro inteiro **e depois** o PARE. Ele afirma a propriedade (o `return` precede o despacho), nunca
uma distância em bytes. **2 mutações, 2 sangram.**

⚠️ **E a segunda metade do report é OUTRA pergunta, com resposta oposta:** *«nem abrindo o
painel»*. Medido — `panel_open_t|panel_visible_t|visibility_live|panel_fade` devolve **vazio**:
**abrir/fechar um painel nunca foi animado**. Não é regressão desta wave; é uma feature que não
existe, e construí-la é decisão do Enio (está na §9).

**Smokes** (`ph2d-run cargo run -p ph2d-host-desktop --release`, ou o `cargo run` equivalente):

| cena | o que julga |
|---|---|
| **`PH2D_UI_MOTION_SMOKE=3`** | ⭐ **A DOBRA** — a cena **abre o painel de FÍSICA** (global: não pede ferramenta nem selecção) e manda dobrar. ⚠️ **Ela imprime dois números lado a lado** (o que o painel *declara* contra o que o `populate` *tem*); **se `tem < declara`, PARE.** |
| **`PH2D_UI_MOTION_SMOKE=1`** | o **CARÁCTER** — ⚠️ a cena **não arma** o carácter, ela manda escolher no pill Settings; o roteiro foi corrigido nesta wave (a nota dos *"três tipos"* tinha envelhecido para **seis** famílias) |
| **`PH2D_UI_MOTION_SMOKE=2`** | a **CORDA** (controle: esta wave não a toca) |

**As quatro perguntas da cena `=3`**, e por que são quatro e não uma:

1. **desliza?** (o corpo interpola em vez de saltar);
2. **o corpo não desenha por fora da banda** (recorte de CENA);
3. ⚠️ **o recorte de HIT — a que NÃO SE VÊ**: passe o rato **onde uma row ainda não chegou**, a
   meio da abertura. Uma row invisível não pode responder;
4. **o que está por baixo sobe junto** (o `y` de saída escalado).

⚠️ **E o CONTROLE é a metade que não se vê:** com tudo **parado** o painel tem de estar
**exactamente** como sempre esteve — é a neutralidade dos dois repousos, e é ela que deixou isto
entrar em dez painéis de uma vez. Mais o **reduced motion**, onde a dobra tem de **SALTAR** (um
corpo a deslizar É área a deslocar-se, e a dobra é `Role::Surface`).

### Mudanças de comportamento, nomeadas

| # | o que muda | onde se vê |
|---|---|---|
| 1 | **o corpo de uma secção interpola** ao dobrar | os **dez** painéis migrados |
| 2 | o **scrub** de 5 campos do `grid_snap` deixa de saturar num pixel | iterações de Lloyd (**0,16 px** → 250) · subdivisões (**1,26**) · as três componentes de cor (**5,1**) |
| 3 | o relógio da UI deixa de alocar 72 kB/quadro | **invisível** — é perf (449 → 340 µs/quadro) |

⛔ **O `widget/showcase` fica DE FORA com motivo** (nunca recebeu a F4a; é galeria de dev, não
chrome do app) — não o "complete" sem trazer a F4a primeiro.

---

## 7. Gate batched da linha (rodado no TIP, `6f66a8554`)

| gate | resultado |
|---|---|
| `cargo fmt --all -- --check` | **EXIT 0** |
| `cargo check --workspace --all-targets` | **EXIT 0** |
| `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings` | **EXIT 0, zero warnings** |
| `cargo nextest run --workspace --cargo-profile ci-test` | **16.115 de 16.115 passaram, 1.562 skipped — EXIT 0** |

⚠️ **A varredura é a WORKSPACE INTEIRA, de propósito.** Esta linha toca `ph2d-editor-core`, e os
gates que moram em `ph2d-editor-core/tests/` e `shells/desktop/tests/` **só correm na varredura
impactada** — um fechamento por `cargo test -p` por crate não os alcança. É a família de
vermelho-latente que este repositório já pagou cinco vezes, e **duas delas foram desta linha nesta
sessão** (o `arch_safe_clamp_only` e o `no_magic_numeric`, os dois do meu próprio commit anterior).

⚠️ **Nenhuma leitura de relógio desta máquina significa coisa nenhuma acima de `load ~5`.** Este
gate correu com `load average 1,40`.

### 7.1 Resultado do nextest

```
Summary [58.160s] 16115 tests run: 16115 passed, 1562 skipped
[exited with code 0]
```

**Zero vermelho-latente**, incluindo os arch-gates de `ph2d-editor-core/tests/` e
`shells/desktop/tests/` que a varredura por-crate não alcança.

⚠️ **Os `--ignored` NÃO entram neste número**, e isso é a política — não uma omissão. As sondas
desta linha são todas `#[ignore]` de propósito (elas **imprimem** e não afirmam), e a família de
kills de relógio do Painter exige `--test-threads=1` com a máquina calma. ⚠️ **Os dois gates de
razão do `plane_copy`/`undo_delta` do Painter estão VERMELHOS no `main` e não são desta linha** —
a `line/Painter` os deixou nomeados na §5 do `CLAUDE.md` em 15/08, com o mecanismo medido (a
premissa de calibração do `PAR_MIN_BYTES` dissolveu: o serial deixou de ser *fault-bound*). Se o
integrador os rodar, **não os atribua a esta wave**.

⚠️ **Gates de GPU:** esta linha **não toca crate de GPU nenhuma** (`ph2d-render` ·
`ph2d-flip-render` · `ph2d-paint-gpu` · `ph2d-mesh-render` · `ph2d-gpu-cook` com diff vazio), então
os `--ignored` de adapter não a alcançam.

---

## 8. Reclamar o `incremental/`

Feito no fecho, conforme o §1.5.9 item 7 — **20 GB medidos** nesta worktree (`target/debug`; o
`release` e o `ci-test` estavam a zero):

```bash
rm -rf "$(git rev-parse --show-toplevel)"/target/*/incremental
```

⚠️ **Reclamar no FIM, nunca desligar no COMEÇO:** durante a jornada o `incremental/` do `dev` é o
que faz o `cargo check -p` voar; o que ele não pode é sobreviver à linha que o criou. Risco zero
(o cargo o recria) e **sem ship**.

---

## 9. O que fica ABERTO, com o preço ao lado

**Nada nesta wave é trabalho pendente por descuido.** O que sobra do plano é decisão ou está
bloqueado fora do repositório:

- ⏸️ **O `n` e a folga do tether** (§8 item 4) — são números de **aparência**, e o oráculo deles é
  o RENDER, não um teste. Saem do smoke, como o `RESAMPLE_STEP_FRACTION` do Flip saiu.
- ⏸️ **Os 141 campos no atalho `DRAG_RATE_X · step`** (§8 item 5) — e o próprio `DRAG_RATE_X = 50`
  é um número **sem medição atrás**: a aritmética contra a lei irmã diz que ele *supõe uma faixa
  de 12.500 unidades* (`50 × 250`), enquanto a receita para a qual este app convergiu é
  `rate = step`, ou seja **50× menos**. Mudá-lo é mudança de FEEL em 141 campos ⇒ **do Enio, com o
  número na mão**, não de uma wave de correcção.
- ⏸️ **2,05% de um quadro é pago com o app PARADO, para sempre**, e a **PODA nunca dispara** no app
  real (o `PRUNE_AFTER_S` promete despejar quem deixa de ser pintado, e o tique toca todos os ids
  macios em todo quadro). Encolher isso é mexer no `tick_hover`, cujo publish atravessa nove
  consumidores e tem histórico de defeito subtil de flash ⇒ **wave própria, com ordem**.
- ⛔ **X1 pressão da caneta** — bloqueado **FORA do repositório** (winit 0.30.13 crava
  `force: None` nos três backends de desktop).
- ⏸️ **ABRIR/FECHAR um painel não é animado, e nunca foi** — medido no smoke desta wave
  (`panel_open_t|panel_visible_t|visibility_live|panel_fade` devolve **vazio**). Não é regressão:
  é ausência. ⚠️ **E ela não é o gêmeo da dobra** — a dobra move o corpo **dentro** de um painel
  cujo rectângulo não muda; abrir um painel move o **dock**, e todo vizinho do dock re-flui. A
  metade cara é a mesma que a F4b pagou (medir-lembrar-recortar), só que a herdar o layout de
  fora. **Feature, com ordem.**
- ⛔ **E4 menu radial · C2 realce de proveniência · D1 som · D2 partículas** são **FEATURES**, não
  polimento.
- ⚠️ **Resíduo estrutural NOMEADO, sem gate:** o eixo do hover está fechado para tudo o que hoje o
  **lê**, mas uma superfície **`Plain`** nova que passe a ler `hover_live` sem estar no mapa
  **nasceria muda outra vez**. ⛔ **E não "complete" a cura alargando o censo a todo `Plain`:** as
  rows da Hierarquia são `Plain`, e amaciá-las revive a cerca do estudo §6.2 — isto está
  **gateado**, e a mutação que o tenta deixa três dos quatro gates de produto verdes.

---

*Linha `Vector` pronta (11 commits de código, o último `6f66a8554`; **a cena `=3` foi smokada e
aprovada**). Aguardo ordem de integração.*
