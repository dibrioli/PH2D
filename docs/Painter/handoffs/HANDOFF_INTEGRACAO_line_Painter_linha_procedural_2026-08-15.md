# HANDOFF DE INTEGRAÇÃO — `line/Painter`, a LINHA PROCEDURAL (plano 38, W1→W6)

> **Estado:** linha FECHADA, aguardando **ordem explícita do Enio** para integrar.
> ⚠️ **A W6 (a FAIXA) está PENDENTE DE SMOKE** — as waves W1..W5 foram smokadas e aprovadas ao longo
> da jornada; a fita foi construída depois do último *"smoke OK. Siga"*, e a **FAIXA** (as travessas)
> depois do report do Enio com a captura do Alchemy. *Integrar não é aprovar.*

---

## 1. O que a jornada entrega

O plano [38](../38_plano_linha_procedural.md) inteiro — o card **Line** e os cinco tipos, mais o
multiplicador de emissão:

| Wave | O que é | Smoke |
|---|---|---|
| W1 | O card **Line** acima do Composite Brush + o `Style: Solid` alcançando os shape editors | ✅ aprovado |
| W2 | O dropdown `Type` e o **Speed Shapes** (a tinta é ARREMESSADA à frente do dedo) | ✅ aprovado |
| W3 | O **Sketchy** — o traço ganha MEMÓRIA e costura-se a si mesmo; o Magnetify | ✅ aprovado |
| W4 | O **Wire** — o mesmo produtor com uma janela de ARCO | ✅ aprovado |
| W5 | O **Spray** — uma contagem, e ela **não é um tipo de linha** (mora no card Jitter) | ✅ aprovado |
| W6 | A **FAIXA** — dois trilhos e travessas (o *Ribbon Shapes* do Alchemy); a fita massa-mola é o trilho de tinta | ⚠️ **pendente** |
| W6 | **Rough** | ⛔ **não construído** (ver §5) |

---

## 2. A FAIXA — a premissa da wave estava errada, e a referência a corrigiu

⚠️ **O `Ribbon Shapes` do Alchemy NÃO é uma curva atrasada.** O Enio pôs a saída dele ao lado da
nossa (2026-08-15) e a diferença é estrutural: a marca dele é uma **FAIXA com travessas** — dois
trilhos ligados por riscos atravessados que se abrem quando a mão acelera —, e a nossa era **uma
linha só, atrasada**, que é o **Dynamic Brush do Krita** (`Mass` + `Drag`). O plano 38 citava a
referência de uma feature e descrevia a outra; **eu implementei a descrição**, e é por isso que
nenhuma das minhas sete hipóteses anteriores chegava à foto dele: eu consertava a coisa certa da
feature errada.

⚠️ **Clean-room, e a regra não afrouxou por conveniência:** Alchemy e Krita são GPL-3. O desenho saiu
da **captura** (comportamento observado) e do manual — nenhuma linha do fonte de nenhum dos dois foi
lida, e a pergunta *"não tem o código aberto para ver?"* tem a resposta que este repo já dá ao
Blender Texture Paint: **comportamento sim, expressão nunca**.

**O desenho:** o trilho de **tinta** é o massa-mola já construído (dabs); o trilho do **DEDO** e as
**travessas** são **FIOS** (o canal `Thread` do Sketchy/Wire — mesmo produtor, mesmo rasterizador,
mesma tinta). A **largura da faixa É o atraso**, então ela abre com a mão e fecha nos picos.

⚠️ **A assimetria dab/fio é o desenho, não um compromisso.** Um segundo trilho de DABS seria uma
segunda pincelada: a Symmetry o espelharia, o Spray o multiplicaria, o impasto construiria relevo
nele e a taper o afinaria — e um ribbon é **UMA** marca. É também o que dispensou o segundo cursor de
percurso que a primeira tentativa (revertida) exigia.

⚠️ **Três leis, cada uma com gate e mutação:** as duas pontas de uma travessa são do **MESMO
instante** (senão sai um LEQUE nas cristas) · a cadência é de **ARCO** com o resíduo a atravessar os
quadros (uma por dab preenche SÓLIDO, uma por quadro põe a taxa de quadros no desenho) · e a
**CAUDA não costura** (no pen-up o trilho do dedo acabou; ligá-la à caneta parada é o leque outra
vez).

⚠️ **`Rungs = 0` é DEGENERAÇÃO, não modo** — sobra o massa-mola sozinho. Uma densidade cobre os dois
looks, e por isso a família não ganhou um segundo tipo nem um interruptor (que teria de nomear o
estado desligado). Default **0,5**: uma fita É uma faixa.

### 2.1 O defeito que custou o diagnóstico: uma porta DUPLICADA

Havia **dois** `sews_threads` — um no `LineKind` (*este tipo costura?*) e um no `BrushSpec` (o que o
depósito pergunta) — e o **doc do segundo dizia consultar o primeiro sem o consultar**: ele
enumerava `sketchy_active() || wire_active()`. Ligar a fita no portão do enum deixou o motor a
costurar **343 travessas por traço** com o depósito **MUDO**, e a imagem renderizada saía idêntica ao
controle.

**O portão do enum não existe mais.** O do spec é um `match` **EXAUSTIVO sem braço `_`**, então um
quarto costurador é um **erro de compilação** em vez de uma faixa que nunca pinta. E os três
produtores passaram a publicar por **uma porta** (`Stroke::sew`), que é onde a Symmetry é aplicada —
uma cópia por produtor é a regra que o quarto nasce sem.

### 2.2 E o card Line não tinha seam nenhum

Os três sliders da W6 shiparam com id, row, `populate`, encaminhamento e setter — e **nenhum gate os
exercitava** (`grep` pelo id nos testes do repo: nada). O `wiring_parity` cobre *o widget existe* e
*está registrado*, e é cego às outras duas: **o clique chega ao tool?** e **a sequência leva a algum
lugar?**. O `line_seam_tests.rs` fecha as duas para as **oito** rows do card.

⚠️ **E ele pegou uma fixture minha antes de shipar:** o `Reach` mapeia a pista em `0..4`, então
`0,25` cai exactamente no `1.0` de fábrica e a asserção *"o campo mudou"* reprovava uma row viva. O
oráculo certo é a **diferença entre duas posições da pista**.

---

## 2b. A FITA (o trilho de tinta), em três frases que decidem o resto

**Ela move o CAMINHO, não a tinta.** O `Speed` arremessa a TINTA e deixa o caminho intacto porque a
velocidade dele é medida *do* caminho — realimentá-la a somaria a si mesma e o traço fugiria da tela
por composição. A fita é um **passa-baixa do caminho**, exatamente como o estabilizador, e
realimentar um passa-baixa é estável por construção. ⚠️ **É por isso que ela custa tão pouco:** para
o espaçamento, os fios, o preenchedor de vão, a Symmetry, o Tiling e o Spray, **a fita É o traço** —
nenhum deles sabe que ela existe.

**Ela não é um segundo estabilizador, e a distinção é MEDIDA.** Ela **ULTRAPASSA** (`ζ < 1` passa do
alvo e volta; o estabilizador é média corrida e converge por baixo, com nenhuma intensidade) e é
**fato do RELÓGIO** (`step_ribbon` lê o `last_raw_pos` uma vez por tique, então um mouse de 960 Hz
desenha o que um de 125 Hz desenha). Cada metade tem gate próprio.

**A cauda é metade da feature.** No pen-up a mão soltou e a fita ainda tem inércia; o traço acaba
onde ela de facto parou, **nunca num salto até o cursor** — esse salto seria um gancho que a física
não produziu (e com gravidade o repouso nem é o cursor: é `g·τ²` abaixo dele).

---

## 3. Superfície de colisão — **MEDIDA, não auto-relatada**

| Eixo | Valor | Como foi medido |
|---|---|---|
| `PROJECT_SCHEMA` | **70, INTOCADO** | `git diff main...HEAD -- shells/desktop/src/project.rs` **vazio** |
| Contrato congelado | **4/4 verde** | `cargo test -p ph2d-editor-core --test architecture_tool_contract_surface` |
| `Cargo.toml` / `Cargo.lock` | **ZERO** | `git diff --stat` vazio nos dois |
| ADR | **nenhum** | `git diff --stat -- docs/architecture/decisions` vazio |
| Crate nova | **nenhuma** | — |
| Dep externa nova | **nenhuma** | — |
| Ids novos | `ids/chrome/painter.rs` | são do card Line; **hash de string** ⇒ fora de todo contador |

⇒ **A linha fica FORA de toda disputa de número desta janela.**

**Arquivos fora do módulo Painter** (todos de waves ANTERIORES desta mesma linha, exceto os dois
marcados): `ph2d-editor-core/src/ids/chrome/{mod,painter,painter_shape,painter_substrate}.rs` ·
`ph2d-render/src/impasto_light*` + o WGSL + os dois testes · `shells/desktop/src/{app_state,
baked_form_planes,main,substrate_smoke,taper_smoke}.rs` · `render_loop/{mod,painter_bridge,
painter_gpu_preview,painter_preview_handoff_tests,painter_preview_measure}.rs` ·
`tests/the_smokes_open_the_painter_in_digital.rs` · **`shells/desktop/src/line_smoke.rs` (a cena)**.

---

## 4. O que a W6 custou, e que lição fica

### 4.1 O teto que limitava a RESOLUÇÃO custou 90,2 GB e a janela do editor

Detalhe completo em [`BUGS_painter.md` #23](../BUGS_painter.md). O resumo que um integrador precisa:
a 1ª versão capava o **número de sub-passos** e deixava `h = dt/n` crescer, **desfazendo** a garantia
de `ω · h = 0,25` que a própria constante ao lado promete. **Um teto limita o TRABALHO, nunca a
RESOLUÇÃO** — a lei do `ph2d_core::time::FixedStep`. Hoje capa-se o `dt` e o teto de sub-passos é
**DERIVADO**, com gate sobre a **aritmética das três consts**.

⚠️ **E o batente nasceu ERRADO dentro do commit que curava a doença** (34 onde eram 134 — o `n` de um
quadro nominal contra o `dt` de quatro que o cap deixa passar). *Um número escrito à mão erra junto
com quem o escreveu*, e foi o gate da aritmética, escrito depois, que o pegou.

### 4.2 Uma mutação sobreviveu, e o gate que faltava nasceu dela

Tirar o cap de `dt` deixava o gate do quadro travado **VERDE**: num quadro de 200 ms o teto de
sub-passos sozinho já segura (`n` pedido 400, aplicado 134 ⇒ `ω · h = 0,75`). **Duas camadas, cada
uma suficiente naquele ponto de operação** ⇒ [[feedback_layered_defenses_need_per_layer_gates]]. A
camada que só o cap compra é o travamento **LONGO** (10 s ⇒ `h = 74,6 ms`, `ω · h = 37,3`), e é o que
o `a_breakpoint_length_stall_is_capped_by_work_not_by_substeps` afirma. **Mutações: 5, todas
sangram.**

### 4.3 Três instrumentos mentiram antes de qualquer código

- **O gate do quadro travado nunca tinha sido observado VERDE** — ele morria a alocar, então o
  oráculo dele jamais fora validado. Quando passou a terminar, **reprovou produto correto**: media a
  distância ao ALVO num salto de 300 px, e uma fita **atrasa por construção**. O oráculo certo é a
  **excursão para fora da caixa do gesto**.
- **O `only_the_speed_type_throws_the_ink` reprovou a fita em `−314 px`** — e o **sinal era o
  diagnóstico**: negativo é ATRASO, que é a feature inteira. O oráculo usava `.abs()`, colapsando
  dois fenômenos opostos; hoje o lado do arremesso é afirmado com sinal, e o do atraso só para quem
  não tem modelo de atraso.
- **O helper `straight` das sondas lia o buffer errado** — `Stroke::tick` **abre com `out.clear()`**
  (`stroke.rs:464`), então ler o último dab só depois do tique dá a resposta certa para uma FITA
  (ali quem percorre é o tique) e **vazia** para todo outro traço. O CONTROLE denunciou: o neutro
  media **4860 px de atraso** num gesto de 4800 px — ou seja, não media nada.

### 4.4 O piso do atraso: a minha própria nota, desmentida pela sonda que ela citava

O doc do `RIBBON_LAG_MIN_S` dizia que ele é o piso de **VISIBILIDADE** e citava uma sonda que **não
existia**. Escrita, ela mede a travessia de *um dab* **entre `τ = 0,005` e `τ = 0,010`** — não em
`0,002`, onde a fita já desloca **0,00 px**. A afirmação *"abaixo dele a tinta desloca-se menos que
um dab"* continua verdadeira; o que era falso é o *"é ONDE isso passa a valer"*.

⚠️ **O piso NÃO foi subido para os 0,005 medidos, de propósito:** aquele número é de UMA velocidade
(2 400 px/s) e UM espaçamento (2,4 px). Mover um limite de **estabilidade** por uma medição de
**aparência** num único ponto de operação seria trocar a razão do número pela mais frágil das duas.
**Consequência de produto, nomeada:** o fundo do slider é **inerte** até peso ~0,02 — que é o que se
quer de um mínimo que significa *desligado*, e o roteiro do smoke o diz.

---

## 5. O que NÃO foi construído, com o preço ao lado

**Rough** (o 2º item da W6) — ⛔ não construído. ⚠️ **A adjacência que ele tem e a fita não tinha:**
ele reescreve a **GEOMETRIA** de um shape editor, então *"o que o Apply assa?"* é pergunta dele e não
do pincel — e a resposta decide se ele é um `LineKind` ou um efeito do editor de forma. É decisão de
desenho antes de ser trabalho, e a W6 o declara *"só com pedido"*.

---

## 6. Smoke

```text
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
  PH2D_LINE_SMOKE=1 cargo run --release -p ph2d-host-desktop
```

**Uma cena para o CARD** (não uma por tipo — três cenas seriam três canvas idênticos com a costura
que importa, *escolher o tipo*, pulada em todas). A cena dá o material e **não arma tipo nenhum**.

**O que julgar na FAIXA** (dropdown `Type` → Ribbon):

0. ⚠️ **A pergunta de OLHO desta wave, e é a primeira:** desenhe uma **onda RÁPIDA**. Tem de sair uma
   **FAIXA com travessas** — o traço de tinta atrás, o caminho do dedo à frente, riscos atravessados
   entre os dois —, e ela tem de **ABRIR nas retas e FECHAR nos picos**, onde a mão desacelera.
   Compare com a captura do Alchemy: é essa a figura.
1. **Rungs de 0 a 1**: em `0` a faixa **degenera na linha atrasada sozinha** (o pincel de arrasto — o
   CONTROLE); subindo, as travessas aparecem e adensam. ⚠️ Este slider **não** muda a largura da
   faixa: quem a abre é o Weight (o atraso) e a velocidade da sua mão.
2. Um **S rápido**: o rastro corta a curva por dentro e **passa do ponto** onde a mão mudou de
   direção.
3. ⚠️ **SOLTE no meio do gesto e não mexa** — a fita continua a **CHEGAR**, e a **FAIXA termina onde
   a mão terminou**: a cauda é só o trilho de tinta, sem travessas. Ligá-las à caneta parada
   desenharia um leque, e é por isso que a cauda não costura.
4. **Weight** (quanto tempo atrasa — e é ele que abre a faixa) · **Friction** (como assenta: baixo =
   chicote, alto = arrasto) · **Gravity** (o peso). Medido a 2 400 px/s: peso 1,00 deixa a tinta
   **804 px** atrás do dedo, que é também a largura da faixa ali.
5. ⚠️ **Se mexer no fundo do slider e nada mudar, suba mais** — é a zona inerte medida (§4.4), e ela
   leva a faixa junto: sem atraso os dois trilhos coincidem e não há faixa a desenhar.
6. **Ela COMPÕE**: ligue o Spray (Jitter → Count) ou a Symmetry com a fita armada.
7. **O CONTROLE**: `Type = None` e `Count = 1` têm de pintar exatamente como sempre pintaram.

⚠️ **`--release` não é preferência** — a densidade cheia do Sketchy põe ~16 mil px de fio num traço
de 312 px, o Wire desenha quatro cordas por dab, e o Spray multiplica cada dab por até dezasseis.

---

## 7. Nota operacional para quem integrar

⚠️ **Construa e teste por `~/.local/bin/ph2d-run`.** Ele roda o comando num scope próprio com teto de
RAM e **sem swap**; sem isso, um `rustc` ou um teste que estoure a memória derruba a **janela do
editor** junto (`OOMPolicy=stop` no `app-code-*.scope`), que foi exatamente o que aconteceu em
2026-08-14. ⚠️ E o `target/` deste repo é um **symlink para `/dev/shm`** — o diretório de build É
RAM, então ele compete com a memória do app [[feedback_a_ship_x_can_be_the_environment_not_the_code]].

**Suíte da crate:** 385 verdes + 15 `#[ignore]` (as sondas). **Clippy:** limpo em
`ph2d-painter-brush` e `ph2d-host-desktop` com `--all-targets`.
