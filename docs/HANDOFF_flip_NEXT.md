# HANDOFF — você é o novo implementador da linha `line/FLIP` (COMECE AQUI)

> **2026-07-12** · escrito pelo agente anterior, para você. Leia INTEIRO antes de tocar em código.
>
> Três partes: **§A — como trabalhar (Modo L, o seu contrato)** · **§B — o estado da linha** ·
> **§C — o PROBLEMA ABERTO**, que é a sua 1ª tarefa. A causa dele **já está provada com números**:
> não recomece a investigação do zero, e não confie no seu olho nem só no harness (§C.5 explica por
> quê — o harness já mentiu duas vezes hoje).

---

# §A — Como esta linha trabalha: **Modo L**

O PH2D roda em dois modos, e qual vale é **função do hardware** (`bash scripts/hw-profile.sh`). Esta
máquina é tier `workstation` → **Modo L**
([ADR-0106](architecture/decisions/0106-parallel-dev-lines-worktrees-workstation.md) ·
[ADR-0107](architecture/decisions/0107-concurrent-foundational-lines-tested-gate-syntactic-merge.md) ·
[DIRETRIZ §1.5](IntegracaoMultiAgente/DIRETRIZ.md) · guia do operador:
[GUIA_JORNADA_MODO_L.md](IntegracaoMultiAgente/GUIA_JORNADA_MODO_L.md)).

### A.1 — Você trabalha num WORKTREE, sozinho

- Seu diretório: `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP`, branch `line/FLIP`.
- **Não há coordenador.** Índice e HEAD próprios → **colisão de commit não existe**. Nada do ritual
  defensivo do Modo C (`git add -- <paths>`, `git status` antes de stage): pode commitar à vontade.
- **Foundational você PODE e DEVE tocar** (`ph2d-editor-core`, `ph2d-flip*`, o shell) — a linha é
  dona do que cria. Ao criar foundational NOVO, **projete-o para isolamento** (módulo irmão, ponto
  de extensão append-only): outras linhas vão estendê-lo em paralelo, e a integração é sintática.
- **PARE e reporte ao Enio** em **só dois casos**: (1) precisa mexer num **contrato congelado**
  (CLAUDE.md §6 — exige ADR); (2) um rebase conflita **fora dos seus arquivos** (mesmo-símbolo,
  DIRETRIZ §1.5.5).

### A.2 — Você NUNCA integra e NUNCA pusha

Inegociável (CLAUDE.md §0.7):

> A linha **fecha, escreve o handoff (DIRETRIZ §1.5.9), e PARA.** Integração e ship acontecem **só
> por ordem explícita do Enio**, via um **agente integrador dedicado**. Integrar ou pushar por conta
> própria é **violação de protocolo**.

Nem ofereça. Ao terminar: "commitado local, linha aberta", e pare.

### A.3 — Cadência de commit (fast mode)

```bash
git commit --no-verify -m "msg"     # sem hooks: instantâneo
```
Toda mensagem termina com:
```
Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
```
Commits locais e frequentes. **Zero push, zero CI** durante a jornada.

### A.4 — Laço interno = `cargo check -p`; o gate é 1× no fim

- **Durante:** só `cargo check -p <crate>`. Nada de teste/clippy por task.
- **No fechamento de cada bloco**, o gate completo:
  ```bash
  cargo test -p ph2d-flip -p ph2d-flip-fill -p ph2d-flip-render -p ph2d-tool-flip \
             -p ph2d-panel-flip -p ph2d-panel-flip-frames -p ph2d-ui-testkit \
             -p ph2d-editor-core -p ph2d-host-desktop
  cargo test -p ph2d-flip-render --test gpu_render -- --ignored    # 17 testes de GPU
  cargo clippy -p <crates> --all-targets                           # ZERO warnings
  rustup run 1.95 cargo fmt -p <crates>                            # o PIN, não `cargo fmt` puro
  typos
  cargo build --release -p ph2d-host-desktop
  ```
- Os **arch-gates** (em `ph2d-editor-core`) checam LOC (700 crate / 600 shell), números mágicos,
  paridade de wiring de painel, glifos tofu. Eles **vão** pegar você. **Não contorne com allowlist —
  divida o arquivo** (módulo irmão; há vários exemplos: `flip_fill_tests.rs`, `layer_time.rs`,
  `tween_flip.rs`, `toolbar_plan.rs`).

### A.5 — O smoke é do Enio; a UI é em INGLÊS

- Comentário de código: **pt-BR**. Rótulo/toast do app: **inglês, sempre**.
- Todo fechamento entrega um passo a passo de smoke, com o `cd` junto (ele copia e cola):
  ```
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP && PH2D_FLIP_DEMO=1 cargo run --release -p ph2d-host-desktop
  ```

### A.6 — Clean-room

A referência do Blender (`~/Downloads/blender-5.2-grease-pencil-ref`, GPL-2.0) é **comportamento,
nunca código**. Leia, entenda, reimplemente.

---

# §B — O estado da linha

## B.1 — O que existe

O **Flip** é o 4º meio do PH2D: animação quadro-a-quadro, fork 2D clean-room do Grease Pencil
(ADR-0114).

| Wave | O que é | Estado |
|---|---|---|
| **W0** | modelo (`ph2d-flip`: objeto/camada/quadro/desenho/traço) | fechada |
| **W1** | render GPU (`ph2d-flip-render`; cobertura = **união global** da polilinha, 1 passe) | fechada |
| **W2** | tool + painel docado + caneta + borracha | fechada |
| **W3** | **virou app de ANIMAÇÃO**: frames, exposição, ciclos, ghosts, tween, a tira | fechada, smoke OK |
| **W4** | **o balde** (`ph2d-flip-fill`) | fechada, **com um defeito aberto — §C** |

Conhecimento do módulo: [`docs/Flip/`](Flip/00_README.md) — `05_frames_ghost_tween.md`,
`06_fill_balde.md`, e sobretudo **[`BUGS_flip.md`](Flip/BUGS_flip.md)**, o registro dos bugs cuja
causa enganava. **Leia os #8–#13**: são todos de hoje, e cada um é uma armadilha que você pode
repetir. Tracker exaustivo: [`HANDOFF_flip_impl.md`](HANDOFF_flip_impl.md).

## B.2 — As seis lições que custaram caro (não as re-aprenda)

1. **Verde ≠ funciona.** A W4 foi entregue com **1251 testes verdes e incapaz de preencher um
   círculo**. Uma auditoria de 3 lentes (solver · costura · modelo) achou **12 bugs**, três deles
   matando o balde no uso mais banal. (BUGS #10.)
2. **Um teste que escolhe a constante conveniente é uma tautologia.** Os testes do balde passavam
   `px_to_world = 1.0` — o **único** valor em que px de tela == unidade de documento, e portanto o
   único que esconde um erro de unidade. Use os números do **produto**… **e varra a FAIXA deles**:
   foi um zoom não-varrido que escondeu o bug seguinte, e é um zoom não-varrido que causa o de §C.
3. **O harness reproduz o mecanismo, não o contexto.** Duas vezes o solver mediu "1 px de erro" e o
   app mostrou 12. Existe `PH2D_FLIP_FILL_DEBUG=1` — **use**.
4. **PINTADO ≠ populado.** Nenhum gate do projeto rodava `Panel::paint`: um widget podia estar
   registrado, wirado, unit-testado e contract-limpo e **não existir na tela**. Agora
   `MockPanelHost::paint::<P>()` (em `ph2d-ui-testkit`) roda a pintura real headless e devolve o que
   ficou clicável. **Todo widget novo ganha esse gate.**
5. **Quando nenhuma constante serve, falta um DADO, não um número.** (BUGS #12: três defaults de
   `grow` foram tentados e cada um quebrava numa faixa de espessura — porque o solver não sabia onde
   a linha estava.)
6. **Um controle mede a partir de uma ÂNCORA, e a âncora tem de ser o que o usuário VÊ.**
   (BUGS #13 — e §C mostra que essa história **ainda não acabou**.)

## B.3 — Commits desta sessão (todos locais, nada pushado)

```
111637cd  fix: a âncora do Grow (borda interna)          ← EM DISPUTA, ver §C
896b88aa  feat: o ALVO VIVO + painel por-modo
42cf4d96  fix: a cor para na silhueta da linha (expand_under_ink)
b4cd145d  fix: "fill impreciso" — o teto na unidade errada (o zoom quebrava o balde)
380c6d8c  fix: a W4 estava MORTA no produto — 12 bugs, 14 gates novos
```

Duas features desta sessão que você vai encontrar e precisa conhecer:

- **O alvo VIVO** (`shells/desktop/src/flip_live.rs`): a última coisa criada — traço ou
  preenchimento — continua respondendo ao painel até o usuário fazer outra coisa. Guarda o
  **INSUMO** (as amostras cruas do traço; a lista de traços pristina de antes do fill) e reaplica
  **sempre a partir dele**, nunca do resultado anterior (senão os parâmetros se compõem e o slider
  deixa de ser reversível). Quando existir **seleção de traço**, ela vira o alvo.
- **O painel é modal**: cada modo mostra só os seus atributos (Draw = pincel + cor; Erase = raio +
  strength; Fill = o balde; Select = nada). Gate: `each_mode_shows_only_its_own_attributes`.

---

# §C — 🟥 O PROBLEMA ABERTO (sua 1ª tarefa)

## C.1 — O sintoma, nas palavras do Enio

> *"Piorou. Linhas finas nem têm valor no slider para ajustar. Aí grow 0 e −1."*

Duas capturas: com **Grow = 0** a cor **transborda** a linha fina (aparece por fora dela); com
**Grow = −1** abre um **vão escuro** de vários pixels. **Não existe valor intermediário** — o slider
é de passo inteiro, e entre 0 e −1 o resultado salta.

E antes disso ele já tinha feito a pergunta certa:

> *"Será que a referência para o fill é o meio da espessura da linha? Se não for, se for o interior
> da linha, pode ser um problema."*

**Ele está certo, e a causa está provada.**

## C.2 — A causa raiz (PROVADA — não é hipótese)

Duas grandezas vivem em espaços diferentes:

- A **espessura do traço é em px de TELA** — absoluta, **invariante ao zoom** (decisão do Enio,
  2026-07-11; o render usa escala de espessura `1.0`, ver `flip_pass::camera_raw`).
- A **geometria do fill é assada em unidades de DOCUMENTO** — congelada no instante do clique.

Logo, **a relação entre as duas muda quando se dá zoom depois de preencher**: a meia-espessura da
linha, medida em unidades de documento, encolhe quando a câmera aproxima; a borda do fill não se
mexe. Fórmula:

```
transbordo ≈ (w/2) · (zoom − 1)      [px de tela]
```

Medido (mesmo círculo; preenche, depois dá zoom; `grow = 0`):

| linha | zoom 1× | zoom 2× | zoom 4× |
|---|---|---|---|
| 3 px | +0,4 px | +2,2 px | **+5,9 px** |
| 6 px | +0,3 px | +3,7 px | **+10,3 px** |
| 16 px | +0,2 px | +8,4 px | **+24,9 px** |

É exatamente a 1ª captura. E a 2ª (`grow = −1`) é o **mesmo** erro pelo outro lado: o `strip_ink`
descola a cor de uma faixa de tinta calculada **no zoom do preenchimento** — mais larga, em
documento, que a linha renderizada no zoom da vista → vão escuro. **Os dois quadros são um bug só.**

> ⚠️ **Sobre o commit `111637cd`** (âncora na borda interna): ele **não mudou nada** no caminho
> `grow = 0` (byte-idêntico ao anterior). O que ele introduziu foi a **descontinuidade** entre 0 e
> −1: em 0 a cor vai até a silhueta externa; em −1 ela salta para a borda interna e recua 1 px — um
> pulo de `w + 1` px. Numa linha fina isso torna o slider inutilizável. **Considerar revertê-lo faz
> parte da solução** (a proposta de C.3 o torna desnecessário).

## C.3 — A solução recomendada: **ancorar no EIXO da linha** (a ideia do Enio)

O **eixo** (a polilinha) é **geometria pura**: não depende do zoom nem da espessura. E a linha
renderizada, seja qual for a espessura e o zoom, **sempre o cavalga** — metade de cada lado.

Um preenchimento que termina **no eixo**, portanto:
- **nunca transborda** (a cor não passa do eixo; a linha cobre do eixo para fora);
- **nunca abre vão** (a metade interna da linha cobre a borda da cor);
- e isso vale **em qualquer zoom e qualquer espessura** — porque nada disso depende deles.

Medido, com a fronteira no eixo (meia-espessura zero no raster). Negativo = a cor está **por baixo**
da linha (é o que se quer):

| linha | zoom 1× | zoom 2× | zoom 4× |
|---|---|---|---|
| 3 px | −1,7 px | −1,9 px | −2,2 px |
| 6 px | −3,2 px | −3,4 px | −3,7 px |
| 16 px | −8,2 px | −8,4 px | −8,7 px |

Sempre negativo, sempre estável no zoom. O resíduo de ~0,4 px é quantização do raster: sub-pixel,
invisível.

### O que isso implica no código

1. **`BOUNDARY` passa a ser o EIXO.** Hoje `fill_at` (`ph2d-flip-fill/src/lib.rs`, passo 3)
   rasteriza a parede a `0,5 × meia-espessura` (o `radius_scale = 0.5` do GP). Passaria a
   rasterizar **no eixo** (raio ≈ 0).
2. **`INK` / `expand_under_ink` deixam de ser necessários no caso default** — a cor já para no lugar
   certo por construção. (Talvez sobrevivam a serviço do `grow`; decida.)
3. **O `grow` fica livre para ser só o ajuste fino que deveria ser — e CONTÍNUO**: `+N` avança N px
   além do eixo (entra no corpo da linha, e além dela vira sangramento); `−N` recua N px do eixo.
   **Sem salto em 0.**

### As perguntas que VOCÊ tem de responder (estão em aberto, decida e DOCUMENTE)

- **O `grow` continua em px de tela?** Se sim, ele sofre do mesmo mal: é aplicado no raster (doc) e o
  resultado é assado — um `grow ≠ 0` volta a ser zoom-dependente. Provavelmente **aceitável** (é um
  ajuste estilístico deliberado, não o default), mas decida conscientemente.
- **Linha fina + slider de passo inteiro.** O Enio: *"linhas finas nem têm valor no slider para
  ajustar"*. Com o eixo como âncora, o **default fica certo sem ajuste nenhum** — mas se ainda
  precisar de passo fino, o slider tem de virar **fracionário** (`set_number_range(id, min, max,
  step)`; hoje o step do Grow é `1.0`, em `ph2d-panel-flip/src/populate.rs`), ou a dilatação tem de
  virar sub-pixel.
- **O `grow` negativo ainda deve abrir um vão independente da espessura?** Era o objetivo de
  BUGS #13. Com a âncora no eixo, `−N` abre um vão visível de `N − w/2` px — **volta a depender da
  espessura**. Ou você aceita (o vão é efeito raro), ou mantém o `strip_ink` **só no ramo negativo**
  — mas então **resolva a descontinuidade em 0**, que é exatamente o que o Enio está reclamando.
- **E a espessura absoluta em px de tela, deveria ser assim?** É a raiz do descasamento. É decisão
  do Enio (2026-07-11). **Não mude sem falar com ele.**

## C.4 — Onde mexer

| arquivo | o quê |
|---|---|
| `crates/ph2d-flip-fill/src/lib.rs` | `fill_at`: passo 3 (raster da parede/tinta) e 5+6 (grow). **A âncora vive aqui.** |
| `crates/ph2d-flip-fill/src/raster.rs` | `stroke_capsule` (parede), `ink_capsule` (silhueta), `expand_under_ink`, `strip_ink`, `grow`. |
| `shells/desktop/src/flip_fill.rs` | a fronteira modelo↔solver: `boundaries()` converte a espessura (px de tela → doc, via `px_to_world`) e `fill_click` monta os `FillParams`. **É aqui que a unidade se decide.** |
| `crates/ph2d-tool-flip/src/params.rs` | `GROW_MIN/MAX`, `PRECISION_*`, defaults. |
| `crates/ph2d-panel-flip/src/populate.rs` | o `step` do slider (hoje `1.0` no Grow). |
| `shells/desktop/src/flip_live.rs` | o alvo vivo: ele guarda o `px_to_world` **da criação**. Se a âncora passar a depender do zoom da VISTA, isto precisa mudar. |

## C.5 — Como MEDIR (faça isto ANTES de mudar qualquer coisa)

Não confie no olho, e **não confie só no harness** — ele já mentiu duas vezes hoje.

1. **No app real:**
   ```
   cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP && PH2D_FLIP_FILL_DEBUG=1 PH2D_FLIP_DEMO=1 cargo run --release -p ph2d-host-desktop
   ```
   Cada clique do balde imprime `px_to_world`, a escala do buffer, a meia-espessura efetiva em px de
   TELA e o contorno resultante. **Peça ao Enio para colar essa linha** se algo não bater.
2. **Fora do app:** um binário descartável que chama `fill_at` com os números do produto e **varre a
   FAIXA** (espessura 1..40 px × zoom 1×..4×), medindo transbordo e vão **em px de tela**. Foi assim
   que a causa de C.2 foi provada, e é a única forma de não repetir o erro nº 2 de §B.2. **Não meça
   um ponto só.**

## C.6 — Os gates que existem (se um ficar vermelho, ENTENDA antes de mexer)

Em `crates/ph2d-flip-fill/src/lib.rs`:
- `the_fill_is_invariant_under_camera_zoom` — a geometria de saída é a mesma em qualquer zoom.
- `the_colour_stops_at_the_ink_silhouette_at_any_line_width` — **vai mudar de sentido** com a âncora
  no eixo. **Reescreva-o para a regra nova; não o delete.**
- `a_negative_grow_opens_the_same_visible_gap_at_any_line_width` + o simétrico positivo — idem.

> ⚠️ **Um gate desta sessão ficou vermelho por culpa do PRÓPRIO TESTE:** o helper que gera um
> círculo sem transcendentais (HR-5) usava uma parametrização racional que cobria um **semicírculo**
> por quadrante — o "círculo" saltava de (0,1) para (1,0). *Antes de acreditar num teste que acusa o
> código, confira que o teste descreve o que você acha que descreve.*

---

# §D — Depois que §C fechar

**W5 — Reshape:** os 9 pincéis de escultura de traço (thickness, smooth, twist, pinch, randomize,
grab, push, clone, strength). As constantes estão tabeladas em
[`Flip/02_referencia_algoritmos_blender_5.2.md` §7](Flip/02_referencia_algoritmos_blender_5.2.md).

**Carry-overs declarados** (não são esquecimentos): fill multiframe (depende da multi-seleção de
chaves na tira) · ajuste modal ao vivo do Gap Closure (o `closures()` já devolve os segmentos; falta
o overlay) · modo Radius do Gap · Colorize (LazyBrush / trapped-ball) · **seleção de traço** (hoje o
alvo dos ajustes é "a última coisa criada", `flip_live.rs`) · integração com a timeline global
(ADIADA por ordem do Enio até a timeline ficar pronta).

---

**Você fecha a linha, escreve o handoff, e PARA. Não integra. Não pusha.**
