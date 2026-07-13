# HANDOFF — você é o novo implementador da linha `line/FLIP` (COMECE AQUI)

> **2026-07-12 (2ª revisão, mesma data)** · escrito pelos agentes anteriores, para você. Leia
> INTEIRO antes de tocar em código.
>
> Três partes: **§A — como trabalhar (Modo L, o seu contrato)** · **§B — o estado da linha** ·
> **§C — RESOLVIDO (âncora no eixo)**, mantido como registro: o defeito do balde que era a 1ª
> tarefa foi fechado nesta revisão (gates vermelhos→verdes; **smoke do Enio APROVADO**
> 2026-07-12 — do smoke saiu o Precision default 1,6). A linha está FECHADA aguardando
> integração ([handoff](HANDOFF_line_FLIP_integracao_2026-07-12.md)); a próxima tarefa da
> linha, quando reabrir, é a **W5 — Reshape** (§D).

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
| **W4** | **o balde** (`ph2d-flip-fill`; âncora = o EIXO da linha, BUGS #14) | fechada, **smoke OK** (2026-07-12; Precision default 1,6 saiu dele) |

Conhecimento do módulo: [`docs/Flip/`](Flip/00_README.md) — `05_frames_ghost_tween.md`,
`06_fill_balde.md`, e sobretudo **[`BUGS_flip.md`](Flip/BUGS_flip.md)**, o registro dos bugs cuja
causa enganava. **Leia os #8–#14**: são todos de hoje, e cada um é uma armadilha que você pode
repetir. Tracker exaustivo: [`HANDOFF_flip_impl.md`](HANDOFF_flip_impl.md).

## B.2 — As seis lições que custaram caro (não as re-aprenda)

1. **Verde ≠ funciona.** A W4 foi entregue com **1251 testes verdes e incapaz de preencher um
   círculo**. Uma auditoria de 3 lentes (solver · costura · modelo) achou **12 bugs**, três deles
   matando o balde no uso mais banal. (BUGS #10.)
2. **Um teste que escolhe a constante conveniente é uma tautologia.** Os testes do balde passavam
   `px_to_world = 1.0` — o **único** valor em que px de tela == unidade de documento, e portanto o
   único que esconde um erro de unidade. Use os números do **produto**… **e varra a FAIXA deles**:
   foi um zoom não-varrido que escondeu o bug seguinte, e foi um zoom não-varrido que causou o
   do §C (três vezes a mesma armadilha, no mesmo dia).
3. **O harness reproduz o mecanismo, não o contexto.** Duas vezes o solver mediu "1 px de erro" e o
   app mostrou 12. Existe `PH2D_FLIP_FILL_DEBUG=1` — **use**.
4. **PINTADO ≠ populado.** Nenhum gate do projeto rodava `Panel::paint`: um widget podia estar
   registrado, wirado, unit-testado e contract-limpo e **não existir na tela**. Agora
   `MockPanelHost::paint::<P>()` (em `ph2d-ui-testkit`) roda a pintura real headless e devolve o que
   ficou clicável. **Todo widget novo ganha esse gate.**
5. **Quando nenhuma constante serve, falta um DADO, não um número.** (BUGS #12: três defaults de
   `grow` foram tentados e cada um quebrava numa faixa de espessura — porque o solver não sabia onde
   a linha estava.)
6. **Um controle mede a partir de uma ÂNCORA — e a âncora tem de ser INVARIANTE sob o que o
   usuário mexe.** (BUGS #13 dizia "o que o usuário vê"; BUGS #14 fechou a história: o que ele
   vê muda com o ZOOM — só o eixo é geometria, e foi nele que o balde ancorou.)

## B.3 — Commits desta sessão (todos locais, nada pushado)

```
7477641b  fix: a âncora do fill é o EIXO da linha         ← fecha o §C (BUGS #14)
111637cd  fix: a âncora do Grow (borda interna)           ← SUPERSEDED pelo de cima
896b88aa  feat: o ALVO VIVO + painel por-modo
42cf4d96  fix: a cor para na silhueta da linha (expand_under_ink)  ← idem, superseded
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

# §C — ✅ RESOLVIDO (2026-07-12): a âncora do fill é o EIXO da linha

O problema aberto desta seção (o 5º smoke: *"Piorou. Linhas finas nem têm valor no slider
para ajustar. Aí grow 0 e −1"*) foi fechado seguindo a solução recomendada — **a intuição
do Enio confirmada**: ancorar o balde no **eixo** da polilinha, a única âncora que é
geometria pura (não depende do zoom nem da espessura).

**O registro completo — causa provada, medições antes/depois, decisões, lições — está em
[`Flip/BUGS_flip.md` #14](Flip/BUGS_flip.md); o resumo de código em
[`HANDOFF_flip_impl.md` §W4.1](HANDOFF_flip_impl.md).** Em uma linha: parede e `INK`
rasterizam no eixo (raio 0), `expand_under_ink(3)` crava a borda da cor em cima dele, e o
Grow virou `grid.grow(signed)` sem ramo — `strip_ink` deletado, slider contínuo em 0.

O que você precisa saber ao mexer no balde daqui em diante:

- **Não reintroduza a espessura no raster do solver.** Ela só folga o bbox. Qualquer âncora
  derivada dela (silhueta, borda interna) volta a transbordar `(w/2)·(zoom−1)` px quando o
  usuário dá zoom depois do clique — é o gate
  `the_baked_fill_stays_under_the_line_at_any_later_zoom` que fica vermelho.
- **Trade-offs já decididos e documentados** (BUGS #14): grow ≠ 0 é assado no zoom do clique
  (aceito, é estilístico) · vão do grow negativo só aparece além de w/2 (aceito) · corpos
  sobrepostos sem eixos cruzados dependem do Gap Closure (o toast sugere) · clicar no corpo
  de uma linha grossa preenche o lado clicado (só o eixo recusa).
- **A régua:** `cargo test -p ph2d-flip-fill sweep_table -- --ignored --nocapture` imprime
  transbordo/vão em px de tela por espessura×zoom. Use-a antes e depois de qualquer mudança
  no solver — e `PH2D_FLIP_FILL_DEBUG=1` no app real.

## C.7 — O smoke (APROVADO pelo Enio em 2026-07-12; roteiro mantido p/ regressão)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP && PH2D_FLIP_DEMO=1 cargo run --release -p ph2d-host-desktop
```

1. Desenhar uma forma fechada com linha FINA (1–3 px) e preencher com `Grow = 0` → a cor
   não pode aparecer por fora da linha.
2. **Dar zoom DEPOIS de preencher** (o caso que quebrava): aproximar 2–4× → sem transbordo,
   sem fio claro, em linha fina E grossa.
3. Arrastar o Grow de −3 a +3 numa linha fina → o preenchimento muda ~1 px por passo,
   SEM salto entre 0 e −1 (o alvo vivo reaplica sozinho).
4. Linha grossa (16+ px): `Grow = 0` fica exato; o vão visível de grow negativo agora só
   começa além de w/2 (comportamento novo, deliberado — BUGS #14).

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
