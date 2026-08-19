# HANDOFF DE INTEGRAÇÃO — `line/Vector`, a DOBRA e as três medições (2026-08-16)

**Status:** FECHADO 2026-08-16 · no `main` em `d1d4e1112` (o commit que trouxe este arquivo).

> **A linha NÃO integra e NÃO faz ship.** Este documento passa ao **agente integrador** o que
> evita conflito e regressão. Formato: [DIRETRIZ §1.5.9](../../IntegracaoMultiAgente/DIRETRIZ.md).

---

## 1. Identidade

| | |
|---|---|
| branch | `line/Vector` |
| HEAD | o **tip de `line/Vector`** — ⚠️ **não um sha escrito aqui**: os últimos commits são este próprio documento, então um literal envelheceria a cada correcção dele |
| último commit de **CÓDIGO** | **`995520929`** |
| merge-base com `main` | **`08e3c84c9`** |
| commits de código | **17** (de **23** no total; os outros 6 são este documento e o plano) |
| diff de **código** | **67 arquivos, +3.496 / −716** (`main...HEAD -- ':!docs'`) |

⚠️ **Esta caixa já esteve errada TRÊS vezes, e as três por motivos que valem mais que o número.**
A primeira dizia *"daí para cima é só `docs/`"* e o smoke produziu **dois commits de fonte** depois
dela. A segunda rotulava a linha como *"diff de **código**"* medindo o total **com `docs/` dentro**
(65 / +3.241 / −732 era `main...HEAD` sem pathspec) — *um rótulo que promete um recorte e mede
outro é pior que um número velho, porque ninguém o vai reconferir.* A terceira foi a **auditoria de
fecho**: ela acrescentou cinco commits (um deles corrigindo um defeito de PRODUTO, §7.2.1), e os
números de 13/17/66 descreviam a árvore de antes dela. **Re-meça no dia da ordem.**

⚠️ **O `main` NÃO andou desde o fork** — re-medido **na entrega deste handoff**:
`merge-base == main == 08e3c84c9`, `rev-list --count` = **0**, e a interseção *arquivos da linha ∩
arquivos que o `main` moveu* é **VAZIA**. Então **hoje** a integração é um `--ff-only` trivial e
**não há rebase a fazer**.

⚠️ **Esta caixa envelhece, e o repositório tem DOIS precedentes de ela ter envelhecido antes da
ordem chegar:** a `line/sculpt3d` (08-09) e a `line/physics` (08-12) traziam a mesma frase, e a
ordem encontrou o `main` **298** e **85** commits à frente. **Re-meça no dia da ordem** — não
acredite nesta linha, rode isto:

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

### `ph2d-editor-core` (14 arquivos)

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
| `screens/hero/live.rs` | ⚠️ **a semente de `fold_track`/`scroll_track` cancelava o byte BAIXO do id** — ver §7.2.1; + os dois gates | ✅ corpo, sem API nova |
| `tests/ui_motion_no_alloc.rs` **(NOVO)** | o gate de contador (dhat) | ✅ |
| `tests/measure_ui_motion.rs` | a sonda de atribuição dos 335 µs (`#[ignore]`) | ✅ |
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

### Os dez painéis (40 arquivos) + `grid-snap` (5)

`inspector` · `painter-layers` · `vector` · `sculpt3d` · `physics` · `audio-editor` ·
`audio-mixer` · `wet-tuning` · `motion-params` · `authored` — todos **vestindo** a porta nova; e
`grid-snap`, que é **correção própria** (ver §5 do commit `c77af7cec`).

### `ph2d-panel-registry-init` (3 arquivos) — **só `tests/`, nenhum `src/`**

| arquivo | o que |
|---|---|
| `tests/ui_motion_population_census.rs` **(NOVO)** | o censo que mede **162 → 4491** widgets |
| `tests/ui_motion_frame_halves.rs` **(NOVO)** | a sonda que parte o quadro parado em duas metades |
| `tests/scrub_range_census.rs` | o censo de faixas, estendido |

⚠️ **Esta crate é a que REGISTA os painéis, e é por isso que as sondas de população moram nela** —
é o build mais barato que enxerga os 4491 widgets de uma vez. ⚠️ **E as duas sondas novas vivem em
BINÁRIOS SEPARADOS de propósito:** `register_all_panels()` é **global e irreversível**, e o censo
mede o piso (`chrome`, 162) **antes** dele — juntas no mesmo binário, o `chrome` saltava para
**4491 / 339,7 µs** e a linha de CONTROLO morria em silêncio. ⚠️ **Corolário operacional: rode as
duas UMA DE CADA VEZ** (`-E 'binary(...)'`), nunca as duas no mesmo comando.

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

### Os 17 commits de código, em ordem, e a dependência entre eles

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
| 12 | `c8b6276e9` o roteiro **RESTAURA** o que manda ligar | **do 11** (a causa raiz do 11) |
| 13 | `995520929` as duas SONDAS que refutam a atribuição dos 335 µs | independente |
| 14 | `93e684176` ⚠️ **a semente do track cancelava o byte BAIXO do id** (§7.2.1) — **defeito de PRODUTO** + 2 gates | **do 1** (é a porta que ele criou) |
| 15 | `0d7c02029` `rustfmt` no gate do 14 | **do 14** |
| 16 | `032e2ca29` dois gates MEUS que não podiam falhar pelo motivo que alegavam | **do 5 e do 12** |
| 17 | `998d6f675` quatro afirmações que não sobreviviam a quem as lesse | independente |

⚠️ **Os commits 1-6 são uma cadeia**: um rebase que os reordene quebra a compilação (o 2 usa o que
o 1 cria). Os 10-11 dependem do 6 (editam a cena que ele cria). Os 7-9 são independentes entre si
e da cadeia. **Os 14-17 são a auditoria de fecho** e vêm por último de propósito: o 14 corrige
código que o 1 introduziu, e os 16-17 corrigem **gates e prosa** dos commits anteriores — um
rebase que os puxe para cima quebra a premissa de que eles editam algo que já existe.

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

**A cura tem DUAS metades, e a segunda é a que fecha o buraco.** O **commit 11** é a REDE: um
**PARE** antes do despacho de cena, que nomeia o interruptor, dá os dois caminhos para o desligar e
avisa que o passo 3 o religa. O gate `the_reduced_motion_guard_stops_the_script_before_the_dispatch`
lê o fonte — **a posição é load-bearing**: um guard depois do `match level` compilaria, passaria na
suíte e imprimiria o roteiro inteiro **e depois** o PARE. Ele afirma a propriedade (o `return`
precede o despacho), nunca uma distância em bytes. **2 mutações, 2 sangram.**

O **commit 12** é a CAUSA RAIZ: os roteiros **restauram** o interruptor que mandam ligar. ⚠️ **E na
cena 3 isso já quebrava o passo SEGUINTE, não só a corrida futura** — o passo 4 manda comparar
Discreto contra Expressivo, e com o reduced ligado os dois estão mortos. Gate
`every_step_that_arms_reduced_motion_also_disarms_it`: propriedade de **CONTAGEM**, não de posição
(um passo novo que o ligue entra no numerador sozinho), com **controlo positivo** contra a frase ser
reescrita. A âncora é a forma imperativa `Settings > Motion > REDUCED MOTION` — o texto do PARE fala
do interruptor em minúsculas e entre crases, logo não conta como instrução. **1 mutação, sangra.**

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

## 7. Gate batched da linha (rodado no TIP, `998d6f675`)

| gate | resultado |
|---|---|
| `cargo fmt --all -- --check` | **EXIT 0** |
| `cargo check --workspace --all-targets` | **EXIT 0** |
| `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings` | **EXIT 0, zero warnings** |
| `cargo nextest run --workspace --cargo-profile ci-test` | **16.118 de 16.119, 1.564 skipped** — ⚠️ **1 flake de CARGA, exonerada por três testemunhas (7.1)** |

⚠️ **A varredura é a WORKSPACE INTEIRA, de propósito.** Esta linha toca `ph2d-editor-core`, e os
gates que moram em `ph2d-editor-core/tests/` e `shells/desktop/tests/` **só correm na varredura
impactada** — um fechamento por `cargo test -p` por crate não os alcança. É a família de
vermelho-latente que este repositório já pagou cinco vezes, e **duas delas foram desta linha nesta
sessão** (o `arch_safe_clamp_only` e o `no_magic_numeric`, os dois do meu próprio commit anterior).

⚠️ **Nenhuma leitura de relógio desta máquina significa coisa nenhuma acima de `load ~5`.** Este
gate correu com `load average 1,40`.

### 7.1 Resultado do nextest

```
tip=998d6f675  load=2.63  PSI avg10=0.00
Summary [69.978s] 16119 tests run: 16118 passed, 1 failed, 1564 skipped
  FAIL (3898/16119) ph2d-host-desktop  flip_smooth::…::the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke
```

⚠️ **O número desta caixa já esteve errado UMA vez, e a contradição era interna ao próprio
documento:** a tabela do §7 dizia **16.117** e este bloco dizia **16.115**. Medido no tip,
`16.119 = 16.117 + 2` — os dois gates novos que a auditoria de fecho acrescentou
(`neighbouring_ids_do_not_share_a_track` e `the_fold_and_the_scroll_families_never_collide`) ⇒
**o 16.117 da tabela estava CERTO e quem estava velho era o bloco bruto colado sob ela.** *Duas
cópias do mesmo facto a discordar não dizem qual delas mente — só que uma mente*, e a única forma
de saber é re-medir.

⚠️ **A ÚNICA falha é flake de CARGA, e é exonerada por TRÊS testemunhas — não por opinião:**

1. **diff VAZIO** — `git diff main...HEAD -- '*flip_smooth*' 'crates/ph2d-flip*'` não devolve uma
   linha; esta linha não alcança aquele código;
2. **PASSA isolada em 0,010 s** com a máquina calma;
3. **falhou na posição 3898 de 16119**, ou seja **a meio da suíte**, com os 32 núcleos saturados
   pelos outros dezasseis mil testes — e ela é um **kill de relógio** (o próprio `CLAUDE.md`
   regista que este gate já reprovou por PERFIL de build, em 21,65 contra 1,92 ms).

⚠️ **A corrida anterior, com `load 5,67 / PSI 1,12`, tinha DUAS falhas** (esta e
`a_wet_move_costs_what_the_footprint_costs_not_what_the_canvas_costs`); com `PSI 0,00` a segunda
desapareceu sozinha. *Nenhuma leitura de relógio desta workstation significa coisa nenhuma acima
de `load ~5`* — e ⚠️ **o `loadavg` ATRASA**: mediu-se `load 16,39` com `PSI avg10 = 0,12`. O
detector honesto de contenção **instantânea** é `/proc/pressure/cpu`, não o `loadavg`.

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

### 7.2 A auditoria multiagêntica de fecho — o que ela achou

Seis lentes independentes sobre um escopo exacto de 66 arquivos (cinco delegadas + uma minha).
**Quatro achados sobreviveram à verificação**, e o primeiro é de PRODUTO.

⚠️ **66 e não 67, e a diferença não é contagem esquecida:** a auditoria correu sobre o diff de
ANTES dela, e o 67º arquivo é `screens/hero/live.rs` — que entrou no diff **por causa** do achado
7.2.1. *A varredura não podia incluir o arquivo que ela própria fez nascer.*

#### ⭐ 7.2.1 A semente do track CANCELAVA o byte baixo do id (`93e684176`)

`fold_track` e `scroll_track` (`screens/hero/live.rs`) mapeiam o `NodeId` de uma secção numa pista
de movimento privada, para que *quanto desta secção está aberta* nunca partilhe pista com *quanto
hover há aqui*. Os dois abriam a mistura assim:

```rust
let mut h = section.0 ^ 0x666f_6c64_5f74_7261; // "fold_tra"
for b in section.0.to_le_bytes() { h ^= u64::from(b); h = h.wrapping_mul(FNV_PRIME_64); }
```

⚠️ **O `^ section.0` da semente e o primeiro `h ^= byte` do laço mordem o MESMO byte, e um XOR
consigo mesmo é ZERO** — o byte baixo do id era cancelado antes de a mistura começar. Ids vizinhos
que só diferem nesse byte (o caso comum: uma tabela de secções declaradas em sequência) entravam no
FNV com o **mesmo** estado após a primeira volta.

**Verificado num modelo independente antes de tocar em código** — não deduzido da leitura. A cura é
tirar o `^ section.0` da semente (o laço já consome o id inteiro):

```rust
let mut h = 0x666f_6c64_5f74_7261; // "fold_tra"
```

Dois gates novos, os dois **não-ignorados** (são eles os `+2` do §7.1): 512 ids consecutivos a
partir de seis bases têm de dar **512 pistas distintas**, e as duas famílias (`fold` × `scroll`)
têm de ter **intersecção vazia** em 4096 ids cada.

⚠️ **Porque nenhum gate o via:** a colisão exige **dois** ids vizinhos a dobrar-se ao mesmo tempo, e
toda fixture de dobra deste repositório move **uma** secção. O sintoma no produto seria duas secções
adjacentes a dobrar **em uníssono** — que lê como *"o painel inteiro respira"*, não como defeito.

#### 7.2.2 Dois gates MEUS que não podiam falhar pelo motivo que alegavam (`032e2ca29`)

- `folded_gap` a `t = 1` era afirmado com `x - 8.0 < 1e-6` — **unilateral**: qualquer valor ABAIXO
  de 8 passa. Um `folded_gap` que devolvesse zero deixava o gate verde. Virou `.abs()`.
- `every_step_that_arms_reduced_motion_also_disarms_it` contava as ocorrências no **arquivo
  inteiro**, então um passo que arma e **outro** que desarma equilibravam-se. Passou a fatiar por
  `fn` e a exigir o par **dentro de cada função**, com controlo positivo. ⚠️ **A mutação honesta
  precisou de preservar a contagem global** (mover o `desarma` para outra função): com ela, o gate
  velho fica **VERDE** e o novo sangra nomeando a função.

#### 7.2.3 Quatro afirmações que não sobreviviam a quem as lesse (`998d6f675`)

`DECLARED` prometia que o gate a percorre (não percorre — ele é `pub(crate)` e o gate vive noutra
crate, com a lista duplicada à mão) · `PANEL_A11Y_DELEGATE_OK` descrevia **uma** categoria e a lista
tem **duas** (sem interacção × DELEGA) · `folded_gap`/`has_body` não diziam que **não têm chamador
hoje** nem a obrigação de recorte que quem os adoptar herda.

#### 7.2.4 O que a auditoria RECUSOU, e vale tanto quanto o que ela achou

- ⛔ **«a migração da dobra ficou pela metade»** — a lente apontou que a Hierarquia continua a
  perguntar `is_collapsed`. **Refutado por medição:** aquele `is_collapsed` é de um **nó da árvore**,
  não de uma secção de painel; é outra grandeza, com outro dono.
- ⛔ **«o bloco bruto do §7.1 está certo e a tabela está velha»** — a lente viu a contradição
  (mérito dela) e concluiu ao contrário; ver §7.1.

⚠️ **E DUAS lentes sujaram a worktree contra instrução explícita** (uma deixou `motion.rs` a meio de
uma mutação, outra um `#[ignore]` em `hero/tests.rs`). **Não restaurei enquanto a segunda ainda
corria** — restaurar debaixo de uma lente fá-la medir uma coisa e reportar outra; o diff foi
preservado para `/tmp/audit-dirt/` e ambas limparam o que sujaram antes de fechar.

---

## 8. Reclamar o `incremental/`

Feito **DUAS vezes**, conforme o §1.5.9 item 7:

| quando | medido | `target/` depois |
|---|---|---|
| no fecho da wave | **20 GB** (`debug`; `release` e `ci-test` a zero) | — |
| depois da **auditoria** | **13 GB** (a auditoria recompilou a workspace duas vezes) | 52 → **41 GB** |

```bash
rm -rf "$(git rev-parse --show-toplevel)"/target/*/incremental
```

⚠️ **Reclamar no FIM, nunca desligar no COMEÇO:** durante a jornada o `incremental/` do `dev` é o
que faz o `cargo check -p` voar; o que ele não pode é sobreviver à linha que o criou. Risco zero
(o cargo o recria) e **sem ship**.

⚠️ **E ele RE-CRESCE a cada gate batched**, que é o que a segunda linha da tabela mede — *"feito no
fecho"* não é um estado, é um gesto: toda corrida de `nextest --workspace` posterior o traz de
volta. ⚠️ **E o número não é abstrato nesta máquina:** o `target/` é um **symlink para
`/dev/shm`** (tier `workstation`, DIRETRIZ §6), então **o diretório de build É RAM** — os 13 GB
saíram do mesmo orçamento em que o app corre.

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
- ⏸️ **2,0% de um quadro é pago com o app PARADO, para sempre** — e esta linha **decompôs o número
  antes de abrir a wave**, com o resultado de que **não há onde construir ainda**. Duas sondas novas
  (`measure_what_a_resting_frame_is_made_of` na `ph2d-editor-core` ·
  `measure_which_half_of_the_resting_frame_is_the_store` na `ph2d-panel-registry-init`) medem as
  cinco peças e **a soma não fecha**: o relógio custa **53 µs** (`advance` 7,6 · `animate × 4491`
  44,8 · o `Vec` 1,0) e o store **56** (`hover_targets` 4,2 · `set_hover_live × POP` 52,1) ⇒ **110
  contra 335, um terço**. ⚠️ **O que sobra é CACHE, e o censo prova-o sem instrumento nenhum:** ele
  mede **162** widgets em **3,09 µs** e **4491** em **335** — *27,7× de população para 108× de
  custo*, super-linear, onde uma soma de peças `O(n)` seria linear; as sondas isoladas sub-estimam
  porque cada uma percorre a MESMA estrutura 200 vezes, residente. ⇒ **a cura não é micro-otimizar
  uma peça** (nenhuma passa de 52 µs): é **não TOCAR** em 4491 entradas, o que é o desenho do
  *conjunto sujo* e mexe no `tick_hover`, cujo publish atravessa nove consumidores e tem histórico
  de defeito subtil de flash ⇒ **wave própria, com ordem — agora com a atribuição CERTA**.
  ⚠️ **E os OUTROS quatro consumidores de `live::tick` estão fora da conta por MEDIÇÃO, não por
  omissão:** dobra · rolagem · zoom · a barra abrem **~6 pistas no total** e custam **~0,1 µs** — o
  eixo é a POPULAÇÃO de widgets, e nenhum deles a move.
  ⚠️ **E a metade da nota que dizia *«a PODA nunca dispara»* estava errada:** ela dispara, mas só
  quando um widget **sai do registo**; o que ela nunca vê é um widget registado que deixou de ser
  **pintado**, porque o tique o alveja na mesma.
  ⚠️ **E a sonda nova QUEBROU o censo ao nascer no mesmo binário** — o `register_all_panels` é
  global e irreversível, e o censo mede o piso (`chrome`, 162) **antes** dele: juntas, o `chrome`
  saltava para **4491 / 339,7 µs** e a linha de CONTROLO morria em silêncio. Elas vivem em
  **binários separados** por isso, e o comentário do arquivo novo diz o mecanismo.
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

*Linha `Vector` pronta — **17 commits de código** (23 no total), o último de código `998d6f675`;
**a cena `=3` foi smokada e aprovada** pelo Enio, e a **auditoria multiagêntica de fecho** correu
depois dela (§7.2). Aguardo ordem de integração.*

⚠️ **O que a auditoria mudou no PRODUTO depois do smoke, e o integrador tem de saber:** um
**defeito de hash** (§7.2.1) que fazia secções vizinhas partilharem pista de movimento. A cena
`=3` foi julgada **antes** dessa correção — ela move **uma** secção, e o defeito precisa de
**duas**, então o veredito do Enio continua válido para o que ele viu. *Integrar isto é integrar
uma cena aprovada mais uma correção que a cena não conseguia exercitar.*
