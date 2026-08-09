# HANDOFF DE INTEGRAÇÃO — `line/Painter`, MESTRE (2026-08-08)

> **67 commits · 179 arquivos · +18.829/−1.581.**
>
> ⚠️ O par `commits · arquivos` **se CONTA**, não se copia:
> `git log --oneline main..HEAD | wc -l` e `git diff --name-only main...HEAD | wc -l`.
> O handoff anterior desta linha nasceu com o número errado no cabeçalho e quase foi bumpado por
> aritmética em cima dele — [[feedback_numbers_that_sum_across_lines_count_dont_pick]] dentro de um doc só.
>
> **Este doc SUPERSEDE o [`HANDOFF_INTEGRACAO_line_Painter_bow_wave_2026-08-06.md`](HANDOFF_INTEGRACAO_line_Painter_bow_wave_2026-08-06.md)**,
> que cobre 33 dos 67 commits (a metade de PERFORMANCE) e continua sendo a leitura de detalhe dela — o
> §2 aqui não a repete, aponta para ela.
>
> **TODOS OS SMOKES APROVADOS PELO ENIO.** O último veredito, sobre a wave do Taper, foi `smoke OK`.

---

## §1 A tabela de colisão — leia isto primeiro

| Eixo | Valor | Nota |
|---|---|---|
| **`PROJECT_SCHEMA`** | **55, intocado** | `git diff main...HEAD -- shells/desktop/src/project.rs` sai **VAZIO**. Esta linha fica **FORA da disputa de número** desta janela. |
| **`FLIP_SCHEMA` / `VEC_SCENE` / `DOC_VERSION`** | intocados | nenhum documento desta linha viaja em arquivo. |
| **Contrato congelado (§6)** | **4/4 verde** | `cargo test -p ph2d-editor-core --release --test architecture_tool_contract_surface` — rodado, não auto-relatado. |
| **ADR novo** | **ADR-0157** | ⚠️ **PROVISÓRIO.** O maior no `main` de hoje é **0155**, então 0156 está CONTADO — mas outra linha da janela pode reivindicá-lo, e os NOMES de arquivo diferem, então **o git nunca conflita**. Se renumerar: o rewrite do token é escopado aos arquivos que **a LINHA** mudou, nunca à árvore ([[feedback_a_token_rewrite_scopes_to_the_changed_files_not_the_whole_tree]]). |
| **Registro do `ph2d-ecs`** | intocado | nenhum componente novo. |
| **`Cargo.toml`** | **1** (`ph2d-tool-painter`) | ver §3 — e **nenhum pacote externo novo**. |
| **Crates novas** | **nenhuma** | os 54 arquivos novos moram em crates que já existiam. |
| **Ids novos** | **13**, todos `hash_node_id` | hash-de-string ⇒ **sem gate de contagem**, e o `node_id_collisions` os cobre. |

---

## §2 A metade de PERFORMANCE (33 commits) — o doc de 06/08

Bow wave gateado no knob (**136,64 → 96,93 ms/traço**) · a cauda da altura · as **bandas por TRABALHO**
· o **boolean na janela das FORMAS** (**284 ms/move**, cortados 35×, mais a sub-janela por forma, 2,43×
com quatro) · e a **lei do REPOUSO** (o meio caro renderiza parado: move de Impasto **99 → 6 ms**, e o
gizmo sob a mão **308 → 0,02 ms**).

Detalhe, números e as curas **construídas e refutadas**: [`HANDOFF_INTEGRACAO_line_Painter_bow_wave_2026-08-06.md`](HANDOFF_INTEGRACAO_line_Painter_bow_wave_2026-08-06.md)
§1-§13 e [`Painter/35_boolean_o_que_o_vector_ensina.md`](Painter/35_boolean_o_que_o_vector_ensina.md).

---

## §3 O LIQUIFY — [ADR-0157](architecture/decisions/0157-liquify-is-an-authored-dab-list-cooked-on-the-device-never-a-stored-dense-field.md)

**Uma deformação é uma LISTA DE DABS AUTORADA, cozida no device — nunca um campo denso guardado.**
W0 (a lei da composição + os dois gates vermelhos + o custo do cook) · W0-device (**0,008 ns por
(nó · dab)**, e o passo da grade **DISSOLVE**) · W1 (o `apply.rs` **COMPÕE** em vez de somar, e a deriva
do cache tem número) · W1b (a **LISTA vira o estado**, e o cache é **provadamente descartável**).

E o **Reshape virou LIQUIFY e tomou o pill do Sculpt** — ⚠️ os dois chips do rail que estavam **mortos**
viraram dois **vivos**; `PAINTER_RAIL_LIQUIFY` e `PAINTER_RAIL_TRANSFORM` são ids novos.

⚠️ **O ÚNICO `Cargo.toml` da linha, e é `[dev-dependencies]`:** `wgpu` + `bytemuck` + `ph2d-gpu` entram
na `ph2d-tool-painter` **só para o kill-criterion do ADR**, cujo oráculo (`warp::field::compose_at`) é
`pub(super)` — uma crate irmã só o alcançaria alargando a superfície privada do tool para a workspace
inteira. **O `src/` segue sem device nenhum; quem despacha é a shell** ⇒ machete-safe, o padrão das
crates-nó da `line/gpu-nodes`. **Nenhum pacote externo novo** — os três já existem no repo (`ph2d-gpu` é
aresta de path).

---

## §4 A SELEÇÃO ganhou mãos

A **CANETA** (o Pen do vetor autorando uma região; fechar no primeiro ponto torna os pontos **editáveis**)
· **CUT · Intersect · Select All** e os atalhos, com **UMA lei de combinação** · o **Paste FLUTUA** (a peça
se transforma antes de pousar) · **Shift trava a proporção, Ctrl ancora no centro** — a lei do gizmo de
sprite, e os modificadores passaram a valer **no gesto de DESENHAR** a marquee, não só no de ajustar ·
e **a caneta VIVA é dona do próprio Ctrl+Z**.

---

## §5 O IMPASTO parou de perder o CORPO em quatro lugares

O **Fill deposita CORPO** (e a borda da seleção veste o Falloff) · o **balde marca sujo pelo CORPO**, não
só pela cor · o **clipboard leva o CORPO** · a **borracha ganha um `pre`** (a figura viva parava de comer
a massa) · a **marquee confina a BOLA** do Inflate, que crescia para fora da seleção · e o **Transform
carrega o CORPO da tinta**, não só a cor.

⚠️ São todos a **mesma doença**, e vale nomeá-la para a próxima wave: *um plano novo é adicionado ao
depósito e os consumidores que já existiam continuam falando só de cor*. A cura é sempre a mesma —
perguntar pela porta única em vez de enumerar quem sabe do relevo.

---

## §6 WET, MASK e SHAPE

**Wet:** o **RASCUNHO é a própria água** (o traço vivo deixou de ser o digital) · o **RECORTE do grid** (a
metade regional do snapshot de folha inteira) · o **véu do Show Wet é VIVO** — quem escreve declara.
**Mask:** a máscara **alcança o RELEVO** (o gate de depósito) · a proteção **mantém o produtor de CPU**
(ela não pode ficar invisível). **Shape:** **Delete apaga a FIGURA em mãos** e o alvo mais específico
vence · o **contorno é o que se vê E o que se clica**, e o gizmo nasce no 1º pixel.

---

## §7 O TAPER (Procreate *Touch Taper*) — a wave de 08/08

O traço **afina nas pontas porque está perto de uma ponta**. Painel: o widget com as duas alças, **Tip**,
**Link tip sizes** e **Opacity**, logo abaixo do Falloff, e **compatível com os 4 meios** (todo setter
faz fan-out pelos slots — um valor escrito só no slot vivo se perde na troca de meio, o defeito que o
dropdown de Paint Mode documentou).

### 7.1 ⛔ O tail hold foi construído, shipado e REPROVADO

O primeiro corte pagava a ponta longe **segurando a cauda** até o cursor passar da janela. É exato, e é
errado como produto: *"o algoritmo que vc usou para o taper é ruim, tem um super delay e um stabilize
ruim. O traço não pode ter nenhum delay e nenhum stabilize"*. **As duas queixas são um mecanismo** — um
traço que atrasa e depois alcança em bloco é o que um estabilizador pesado parece.

A lei agora é **estrutural**: nenhum dab é retido, nenhum dab sai do lugar onde o ponteiro o pôs. O gate
afirma isso sobre **alcance por movimento e ORDEM**, não sobre igualdade de contagem — a densidade muda
legitimamente (§7.3), e um gate escrito contra a contagem teria de ser afrouxado no dia seguinte.

### 7.2 A ponta longe é resolvida no PEN-UP

Nada muda enquanto se desenha; no instante em que o artista solta, o traço é **devolvido ao estado
pré-pincelada e carimbado de novo, afinado**. Um traço, **um** Ctrl+Z.

⚠️ **Por que um restore é inevitável:** tinta só vai para cima, e o relevo é um envelope `max` — o mesmo
fato um plano adiante. O "antes" já estava na mão: o `ModelSnapshot` do pen-down. Voltam os pixels,
`heights`/`covers`/`mats` da camada ativa, e o envelope. **Pelas PORTAS de fork**
(`fork_canvas`/`heights`/`covers`/`mats`), nunca `Arc::make_mut` — são elas que dizem ao journal de que
plano são os bytes.

⚠️ **O replay é do traço INTEIRO**, e não é preguiça: o envelope é por-TRAÇO (um `max` por texel mais um
ledger de push/wave que é fato sobre a **lista** de dabs), então não dá para limpá-lo por retângulo sem
mentir sobre o resto. **Custo: um carimbo a mais do traço, uma vez, no pen-up.**

⚠️ O dab replayado já carrega a cabeça, então o que a ponta longe lhe deve é a **RAZÃO**
`w_final / emitted_w` — e o guard `w_final < emitted_w` é **load-bearing**: mutar só a razão é
**semanticamente neutro e sobrevive** (medido).

### 7.3 O espaçamento segue a largura VIVA

Spacing é uma **RAZÃO** — quanto de uma largura de dab até o próximo — e estava sendo paga contra o
diâmetro **NOMINAL**. Com o dab afinando e o vão fixo, a razão explode e a ponta sai em **CONTAS**.
Medido na fixture que produz a foto do report (raio 25, spacing 0,25, taper 3 diâmetros):

| lei do passo | pior vão ÷ diâmetro |
|---|---|
| diâmetro nominal (o que o Enio fotografou) | **1,600** — discos que nem se tocam |
| **largura viva** | **0,284** — o spacing do próprio pincel |

⚠️ A primeira sonda usava raio 10 / spacing 0,10, onde o mesmo defeito mede **0,463**: o vão é fixo, a
razão cresce como `1/r`, e um pincel pequeno de spacing apertado **não contém o fenômeno**.
`taper_width_at` é porta única — o tamanho do dab e a distância até o próximo perguntam à MESMA função,
e fora do taper ela devolve **exatamente 1.0** ⇒ a aritmética é byte-idêntica à que shipava.

### 7.4 As três rows numéricas estavam MUDAS

`Tip Start` / `Tip End` / `Opacity` voltavam a zero. Uma row numérica é pintada por `paint_num_row`, que
registra o `NumberInput` **e espelha o valor do tool de volta nele todo frame** — então uma row cujo
`ValueChanged` não é reivindicado pelo forward do painel fica **pintada, viva sob o mouse, editável, e
reverte no instante em que o artista solta**. Nasce `PAINTER_TAPER_FIELDS`.

⚠️ **O gate que eu tinha era cego a isso:** ele afirmava *pintada + registrada*, e **"o valor pousa" é uma
terceira condição independente** — a quarta da política de UI que o `00_plano_waves.md` da física
escreveu. O gate novo dirige o evento real e lê o pincel.

### 7.5 ⛔ O IMPASTO ESTÁ FORA do resolve de cauda — leia antes de "completar"

Report: *"o mouse up do Taper retira a tinta e deixa só o relevo"*. **Ablacionado:** com o restore, um
traço de impasto mede **0 linhas entintadas**; sem ele, **20**. O restore leva a tinta e **o replay não a
repõe** — sob impasto a cor do canvas não é simplesmente a soma do que cada dab depositou, e re-rodar o
render do relevo vivo depois do replay **também não a traz de volta** (medido).

**Uma cauda reta é uma feature faltando; perder a tinta do artista é destruir o trabalho dele.** O termo
`&& !self.paint.brush.impasto` fica até o mecanismo estar **entendido, não adivinhado**. O gate tem
**CONTROLE** (o mesmo pincel com o taper de fim desligado ⇒ a barra é *"tanta tinta quanto o impasto já
deita"*, não um número escolhido) e a mutação que tira o termo sangra **0/0 contra 20/20**.

Onde a cauda afina hoje:

| | cabeça | cauda |
|---|---|---|
| Digital, arrasto comum | ✅ ao vivo | ✅ no pen-up |
| **Impasto**, arrasto comum | ✅ ao vivo | ⛔ **fora** (§7.5) |
| Watercolor · Wet Paint, arrasto | ✅ ao vivo | ⛔ fora (acumuladores próprios / o fluido não rebobina) |
| Line · Arc · Curve · Free Hand, **todos os meios** | ✅ | ✅ ao vivo e exata |

### 7.6 As alças cabiam para fora do painel

Elas são desenhadas a partir do **centro**, então nos extremos metade do círculo ficava fora. A trilha é
recuada pelo próprio raio — ⚠️ **e o recuo entra também no `canvas` contra o qual o `CurvePoint`
normaliza o arrasto**: recuar só o *desenho* poria o ponto num lugar que o valor decodificado não
confirma, que é a divergência seed-vs-sample que esta casa já pagou várias vezes
([[feedback_derived_coordinate_seed_must_match_sample]]).

---

## §8 O que rodar na árvore combinada

```bash
cd <worktree-ou-main>
cargo test -p ph2d-painter-brush -p ph2d-tool-painter -p ph2d-panel-painter-layers --release
cargo test -p ph2d-painter-brush -p ph2d-tool-painter                        # ⚠️ DEBUG também
cargo test -p ph2d-editor-core -p ph2d-host-desktop --release
cargo clippy --workspace --all-targets
```

⚠️ **Rode o Painter em DEBUG também** — esta linha tem precedente registrado no repo de um pânico que só
aparecia lá (o `ph2d-flip-colorize`, cuja nota sobreviveu ao fato por três integrações).

⚠️ **Os gates de `shells/desktop/tests/` e `crates/ph2d-editor-core/tests/` só correm na varredura
impactada** — um fechamento por `cargo test -p` por crate **não os alcança**, que é a causa estrutural
dos vermelhos-latentes que `line/Vector`, `line/physics` e `line/motion-value` já documentaram. Aqui
foram rodados por crate inteira (152 suítes verdes em `editor-core` + `host-desktop`).

**Medido no fechamento:** 22 suítes nas três crates do Painter · 152 em `editor-core` + `host-desktop` ·
clippy **0** · contrato congelado **4/4** · `project.rs` com **diff vazio**.

---

## §9 Smokes

| Cena | O que julga |
|---|---|
| `PH2D_TAPER_SMOKE=1` | o taper inteiro — ⚠️ **passo 3:** a tinta sob o cursor *enquanto* arrasta **e** a cauda estreitando *ao soltar*, com **um** Ctrl+Z; **passo 6:** os shape editors afinam as duas pontas ao vivo |
| `PH2D_IMPASTO_SMOKE=1` / `=2` | o corpo da tinta, e o §5 (Fill · balde · clipboard · borracha · marquee) |
| `PH2D_WETPAINT_SMOKE=1` | o rascunho que é a própria água, o recorte do grid, o véu vivo |
| `PH2D_MASK_SMOKE=1` | a máscara alcançando o relevo e a proteção |

Todos com `cargo run -p ph2d-host-desktop --release`.

---

## §10 Aberto, com o preço ao lado

- ⛔ **A cauda do taper no IMPASTO** (§7.5). O próximo passo não é código: é descobrir **de onde a cor do
  impasto de fato vem no canvas**, porque o replay a reproduz em Digital e não em Impasto. Ablação já
  feita e nomeada; hipótese, não.
- **Watercolor e Wet Paint** seguem com cauda reta no arrasto, e o motivo é diferente do impasto: eles
  **reconstroem** o resultado dos próprios acumuladores / de um fluido que não rebobina, então devolver
  os pixels não devolveria o **estado**.
- **O custo do resolve não está medido.** É um carimbo a mais do traço, uma vez, no pen-up, e o pen-up
  desta linha já custava ~32 ms medidos; num traço muito longo ele é proporcional ao traço. **Fica
  NOMEADO, não escondido** — o número sai do próximo `PH2D_PAINT_PERF=1`.
- O **`stabilizer` nasce em `0.5`** (pré-existente, com slider no painel). Não foi tocado por esta linha;
  se o traço ainda parecer amortecido, é ele, e mudar o default é decisão de produto.
