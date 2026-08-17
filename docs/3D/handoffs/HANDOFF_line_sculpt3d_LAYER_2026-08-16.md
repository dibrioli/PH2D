# HANDOFF — `line/sculpt3d`: o **Verb::Layer** bit-idêntico ao Blender

**Data:** 2026-08-16 · **Branch:** `line/sculpt3d` · **Worktree:** `Worktrees/line-sculpt3d/`
**HEAD:** `172625199` · árvore **limpa**, nada pushado, gate verde (fmt · clippy · release · debug).
**Formato:** este doc é o bloco de
[`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)
mais o estado. **Comece pela §1 e execute-a antes de abrir qualquer arquivo.**

---

## §0 — A MISSÃO (ordem permanente do Enio, desde a 1ª mensagem da sessão)

> *"quero idêntico ao blender"* · *"paridade bit-idêntica"* ·
> *"se aumentar **hardness** ou **Auto Smooth**, Layer fica muito ruim"* ·
> *"não quero testes, quero idêntico ao blender. Para de tentar inventar"*

O alvo é o **`Verb::Layer`** — a demão — **idêntica ao `layer.cc` do Blender**, e o
defeito tem **dois eixos que o Enio nomeou**: `hardness` e `auto_smooth`.

⚠️ **A ordem *"não quero testes"* é sobre MÉTODO, não sobre gates:** ela proíbe
inventar aproximações e medi-las; ela **não** proíbe medir a divergência contra a
referência. Vá à fonte **primeiro**, porte **literalmente**, e só então meça.

---

## §1 — FASE 0: onde você está (execute JÁ, sem pedir confirmação)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d
pwd && git branch --show-current      # DEVE terminar em /line-sculpt3d e dizer line/sculpt3d
git log --oneline -5 && git status -sb
git rebase main                       # obrigatório no início de CADA jornada
cargo check -p ph2d-sculpt3d
```

⚠️ **Modo L: TODO comando começa com o `cd` da worktree.** A cwd do Bash **volta ao
repo primário** entre chamadas, e o mesmo path relativo existe nas duas árvores —
editar a errada **compila e commita sem um único erro**. Eu li o `main` achando que
lia o tip e **contradisse um agente que estava certo**. Na dúvida, `pwd`.

**Referências (fora do repo, já clonadas, NÃO edite):**
`/home/enio/Documentos/Recursos/BlenderSculpt` · `/home/enio/Documentos/Recursos/SculptGL`
⚠️ Blender é **GPL** ⇒ **só comportamento, nunca código**. SculptGL é MIT.

---

## §2 — ⭐ O ACHADO QUE EXPLICA A TARDE INTEIRA: a suíte é CEGA aos dois eixos

Isto é a primeira coisa a ler, e é o que faz de doze gates verdes um álibi em vez
de uma prova.

**Medido no `verb_layer_tests.rs` (16 gates):**

| fato | número |
|---|---|
| gates que constroem a malha com `grid()` — uma **grade PLANA**, normais todas `+z` | **15** |
| gates cujo pincel é `coat_brush()`, que crava **`Falloff::Constant`** | **13** |
| fixtures **curvas** (esfera, cilindro, qualquer coisa com normal variável) | **0** |
| fixtures com falloff **≠ Constant** | **0** |

E `Falloff::Constant` é, literalmente, `=> 1.0` (`falloff.rs:235`).

**As TRÊS consequências, e cada uma sozinha já bastaria:**

1. **O `hardness` é remapeado e depois JOGADO FORA.** A cadeia é
   `shaped_distance(t, h)` → `falloff.weight(t')` → `shape`. Com `Constant` o
   segundo passo devolve `1.0` para **todo** `t'` ⇒ o `hardness` não alcança um
   único vértice de um único gate. **O eixo que o Enio nomeou na primeira
   mensagem nunca esteve sob teste.**

   ⚠️ **E o `Constant` não é sequer o falloff do PRODUTO:** `Verb::Layer` nasce
   com **`Falloff::Smooth`** nos dois modos (`brush_verb_defaults.rs:145-149` →
   `profile_s(Layer) == None` ⇒ `unwrap_or(Smooth)`; `profile_b` declara
   `Some(Smooth)`). *Os gates medem uma curva que a demão nunca usa.*

2. **Numa grade plana o kernel do Layer é uma TRANSLAÇÃO VERTICAL.** Toda normal
   é `+z`, então `orig + n·altura·disp` move todo mundo no mesmo eixo: **não há
   parede, não há leque de normais, não há curvatura.** E a parede é *exatamente*
   o que a foto do Enio mostra.

3. ⭐ **O termo `facing` é IDENTICAMENTE 1 numa grade plana, e ele mora DENTRO do
   `shape`.** A cadeia real é
   `fall = curve · alpha_weight · facing` → `shape = fall · keep`
   (`stroke_dab_core.rs:305,317`), com
   `facing = max(−(base_nrm · dab.eye), 0)` (`:300-303`) — um multiplicador
   **CONTÍNUO**, não um corte binário. Numa grade de frente para a câmera ele
   vale `1.0` em todo vértice; **numa esfera ele varia pela pegada inteira**.

   ⚠️ **E isto interage com o eixo do report de um jeito que nenhum gate vê:** com
   dureza alta o `curve` satura em `1` em quase toda a pegada, então **o `facing`
   passa a ser o ÚNICO termo espacial do `shape`** — a forma da demão a hardness
   alto é, hoje, decidida pelo ângulo com a câmera. Se a referência usa
   `use_frontface` como **teste binário** (o `sculpt_brush_test` do Blender) e nós
   usamos um `cos` contínuo, a divergência é invisível em plano, invisível a
   dureza baixa, e **domina exactamente onde o Enio reclama**. ⚠️ *Confirme na
   fonte antes de mexer* — a pergunta é se algo equivalente entra no `factors` do
   `layer.cc`.

⚠️ **E é por isso que o meu porte de hoje passou "sem uma edição de asserção", o
que eu reportei como PROVA de que a troca era cirúrgica:** a lei nova é
`live + (meta − live)·shape`, e com `shape = 1` ela **é** `meta` — a lei absoluta
que ela substituiu. *A fixture não consegue distinguir as duas leis.* Os doze
gates não atestaram neutralidade; eles atestaram cegueira.

**O que fazer com isso:** antes de escrever uma linha de kernel, dê à suíte uma
fixture **CURVA** e um falloff **≠ Constant**. Se o defeito for real, ele nasce
vermelho ali. Se não nascer, o defeito está fora do kernel (ver §4).

---

## §3 — ⭐ AS DUAS FOTOS: onde há comparação e onde não há

O Enio mandou dois screenshots do MESMO roteiro — **1 traço · 1 área · hardness
alto** — um no nosso app, um no Blender.

| faixa | nós | Blender | comparável? |
|---|---|---|---|
| 1 — **traço** | camada arredondada, lisa, com pontas | idem | **sim** |
| 2 — **área** | platô liso, bordas arredondadas | idem | **sim** |
| 3 — **hardness alto** | relevo quase NULO, com **listras retangulares regulares** (um pente) e um halo largo e chato | relevo CHEIO, com **espetos caóticos** (aspecto rasgado) | **NÃO** |

**Duas divergências independentes na faixa 3, e elas pedem investigações
diferentes:**

- **(a) ALTURA.** A nossa colapsa com dureza alta; a do Blender não. É a
  divergência que se mede com um número.
- **(b) CARÁTER do artefato.** O nosso é **alinhado à grade** (retângulos); o do
  Blender é **por-vértice e caótico**. Isto pode não ser o kernel — ver §3.1.

⚠️ **E a leitura mais importante das duas fotos: a do Blender TAMBÉM é feia.**
Não é que o Blender faça hardness alto lindamente e nós façamos mal — ele o faz
**violentamente** e nós o fazemos **chatamente**. A missão é *idêntico ao
Blender*, então **o alvo é a violência dele**. ⛔ Não "melhore" a referência: se o
resultado ficar bonito e diferente, ele está errado.

### §3.1 — A hipótese que explica o pente, e como derrubá-la em uma sonda

Uma demão de espessura constante sobre uma esfera é um **offset paralelo**: o topo
dela tem as MESMAS normais da esfera intacta, logo **sombreia igual à esfera** —
uma mesa correta *lê* como chata, e só a **parede** pega luz. O nosso pente é
plausivelmente a parede a escadear pela grade de quads da esfera de fábrica (o
cubo subdividido do SculptGL, **98.304 quads**, que entrou em 15/08).

**Falsificável em uma medição:** o **período do pente tem de ser o passo da
grade**. Se for, os retângulos são **discretização** e não defeito de kernel — e
perseguí-los é caçar o alvo errado. Se não for, é kernel.

⚠️ **E a topologia do Blender na foto dele é OUTRA** — o caráter *espeto* contra
*escada* pode ser inteiramente isso. Meça antes de atribuir.

---

## §4 — O que JÁ EXISTE (não reconstrua)

A W8 landou nesta linha. O verbo está inteiro, a fiação está provada, e **o porte
do `layer.cc` foi feito hoje** (`135b5e754`).

| peça | onde | estado |
|---|---|---|
| a translação | `stroke_target.rs`, braço `Verb::Layer` | `live + (meta − live)·shape` — o `calc_translations` |
| a recorrência | `coat.rs::coat_step` | `disp += w·força·(1,05 − |disp|)` + clamp, o `layer.cc` verbatim |
| a dureza | `brush_scale.rs::shaped_distance` | `(t − h)/(1 − h)`, o `apply_hardness_to_distances` |
| o aplicador | `stroke_apply.rs`, flag `coat` | o alvo já é a posição final; o `accum` continua sendo o `disp` |
| defaults | `brush.rs:132` | `layer_height` **0,1**, faixas `[0, 1]` dura e `[0, 0,2]` de slider |
| 16 gates | `verb_layer_tests.rs` | **verdes, e cegos** — §2 |
| cena de smoke | `sculpt3d_scenes_layer.rs`, **`=33`** | 8 passos, e **nenhum toca hardness ou auto_smooth** |

**Conferido linha a linha contra a fonte hoje** (não re-derive): os quatro passos
estão na ordem do `calc_faces`; a distância sai do `pre` nos dois lados
(`from_live = false` ⇔ `calc_brush_distances(ss, orig_data.positions, …)`); o
auto-smooth roda **depois** do pincel, uma vez por cópia de simetria
(`sculpt.cc:3635`); e `factors` é **só a curva** (`calc_brush_strength_factors`
chama apenas o `BKE_brush_calc_curve_factors` — a força vive no `cache.bstrength`,
que é o nosso `intensity`).

**O early-out da demão MORREU hoje, e não o traga de volta:**
`if coat && accum >= keep { return }` era correto sob a lei absoluta e sob a lei da
referência ele **destrói a feature** — `disp` cheio não quer dizer *o vértice está
na meta*; se o auto-smooth o tirou de lá, é o dab seguinte que o traz. O
`calc_faces` do Blender não tem early-out. O gate que afirmava a premissa dele
(`a_finished_coat_stops_asking_for_work`) foi **substituído**, não recalibrado, por
`a_finished_coat_stops_growing`.

**Auto Smooth, medido antes e depois do porte** (uma pincelada, esfera de fábrica):

| `auto_smooth` | relevo ANTES | relevo AGORA | espeto/relevo |
|---|---|---|---|
| 0,00 | 0,07707 | 0,07356 | 2,01 |
| 0,50 | 0,00517 | 0,08394 | 1,57 |
| 1,00 | **0,00164** | **0,06940** | **1,68** (era 53,4) |

⚠️ O **Draw** continua a ser aniquilado pelo Auto Smooth (0,08738 → 0,00016) e isso
está **CERTO** — ele é aditivo puro e não tem meta para onde voltar.

---

## §5 — A fila que eu deixo, na ordem que eu atacaria

1. **Dê olhos à suíte** (§2): fixture **curva** + falloff **≠ Constant**. Sem
   isto, nada do que vier a seguir é verificável.
2. **Meça a ALTURA contra a dureza** — a divergência (a) da §3, que é a única com
   um número. Nós temos `spike/relevo`; **não temos o relevo ABSOLUTO por
   hardness**, e é ele que a foto acusa.
3. **Confirme as UNIDADES do `height` na fonte.** ⚠️ O nosso está **MEDIDO e é
   ABSOLUTO**: `layer_height` entra em **um único sítio do `src/` inteiro**
   (`stroke_target.rs:499-500`), multiplicado só por `sign` (±1) e pelo `disp`
   adimensional — **nenhum raio, nenhuma escala, nenhuma pose** (`grep` por
   `pose`/`.scale` no caminho do traço volta **vazio**). Se o do Blender for
   multiplicado pelo raio do pincel (ou por `ss.cache->scale`), **isso sozinho
   explica a faixa 3 achatada** e a cura é uma multiplicação, não um redesenho.
   *Esta é a hipótese mais barata da lista — teste-a primeiro.*
4. **Decida o `facing`** (§2.3): o `use_frontface` da referência é **teste
   binário** ou **peso contínuo**? Se for binário, o nosso `cos` é uma segunda
   curva de falloff que o Blender não tem, e ela **domina a hardness alto**.
5. **Derrube ou confirme o pente** (§3.1): período do pente contra passo da grade.
6. **Só então** volte ao kernel.

⚠️ **E acrescente à cena `=33` os dois passos que faltam** (subir o hardness ·
subir o Auto Smooth). Hoje o roteiro tem oito passos e **nenhum exercita o defeito
reportado** — o Enio está a produzir a evidência à mão porque a cena não a produz.

---

## §6 — Armadilhas desta linha (pagas, não repita)

- ⚠️ **A cwd volta ao primário** (§1). Custou-me um veredito errado contra um agente correto.
- ⚠️ **`brush.weight()` da demão é `strength²`, em TODO modo** — `profile_s(Layer)`
  devolve `None`, o `for_verb` cai no `B`, e o `B` declara
  `StrengthCurve::Squared`. Com o default `strength = 0,5` a força efetiva é
  **0,25**, e a demão fecha em ~10 dabs, não em 1. Toda medição que conte dabs
  tem de saber disto antes de chamar um resultado de lento.
- ⚠️ **Gate de kernel é CEGO à fiação; gate de seam é cego à LEI.** Precisa dos dois.
- ⚠️ **Fixture que não contém o fenômeno** é a doença desta linha, e a §2 é o caso mais caro dela.
- ⚠️ **Oráculo byte-a-byte tem de reproduzir a ASSOCIAÇÃO:** `u*u*u*u` diverge de `(u*u)*(u*u)` por **um ULP** já em `t = 0,02`.
- ⚠️ **`cargo test -p` NÃO roda `cargo fmt --all -- --check`** — o tip desta linha já esteve fmt-vermelho em cinco arquivos e **só o ship o via**.
- ⚠️ **Arch-gates que fatiam fonte por índice de BYTE panicam** em prosa portuguesa (acento, `⚠️`) — curado em `the_armed_transform_is_shown.rs`, com `read_dir` **ordenado** junto (a ordem dele é *unspecified*).
- ⚠️ **Doc-comment (`///`) em parâmetro de `fn` é ilegal em Rust** — use `//`.
- ⚠️ **Desfaça mutação com `cp` de um backup, NUNCA `git checkout`.**
- ⚠️ Rode as suítes em **debug além de release** (precedente: pânico só em debug).
- ⚠️ Gates de relógio (`--ignored`) exigem `--test-threads=1` e `load < ~5`.
- ⚠️ **zsh:** `--include=*.cc` sem aspas falha com *"no matches found"*.

---

## §7 — Fronteiras

- ⛔ **NÃO integrar, NÃO `git push`, NÃO rodar `scripts/foundational-integrate.sh`.**
  Integração e ship são **só por ordem explícita do Enio**, via agente integrador
  dedicado (CLAUDE.md §0.7 · DIRETRIZ §1.5.3-1.5.4). A linha fecha a wave, escreve
  o handoff e **PARA**.
- ⛔ `rayon` novo ⇒ **ADR novo** (a exceção desta crate é o ADR-0159).
- ⛔ Contrato congelado (CLAUDE.md §6) ⇒ **PARE e reporte**.
- ⛔ **Não "melhore" a referência** (§3).

---

## §8 — Aberto além do Layer (contexto, não fila)

- Os defaults de fábrica por-ferramenta do Blender moram num **`.blend` binário**
  ⇒ a W1 e o *Draw Sharp* são **decisão de produto do Enio**, não dívida de
  engenharia.
- Duas pistas de um agente, **ainda não medidas**: o `Draw` em modo `B` ficou
  **5× mais forte por dab**, e quatro verbos (`Blob`, `ClayStrips`, `ClayThumb`,
  `MultiplaneScrape`) nascem em `B` com **metade** da força.
- As duas curvas de domo (`Dome`/`Dome4`) voltaram ao catálogo (`ALL` 10 → 12)
  numa sessão anterior — **isso não era o pedido**, e está registrado para
  ninguém o tomar por escopo.
