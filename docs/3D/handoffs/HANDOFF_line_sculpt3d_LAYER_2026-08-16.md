# HANDOFF — `line/sculpt3d`: o **Verb::Layer** idêntico ao Blender

**Data:** 2026-08-16 · **Branch:** `line/sculpt3d` · **Worktree:** `Worktrees/line-sculpt3d/`
**Estado:** 112 commits, **nada pushado**, árvore limpa, gate verde (fmt · clippy · 43 suítes release · debug).

---

## §0 — A MISSÃO (ordem permanente do Enio, desde o começo da sessão)

> *"quero idêntico ao blender"* · *"paridade bit-idêntica"* · *"se aumentar
> **hardness** ou **Auto Smooth**, Layer fica muito ruim"*

O alvo é o **`Verb::Layer`** — a demão — **bit-idêntica ao `layer.cc` do Blender**,
e o defeito reportado tem **dois eixos NOMEADOS pelo Enio**: `hardness` e `auto_smooth`.

⚠️ **O agente anterior (eu) errou o alvo e gastou a sessão fora dele** — foi para o
catálogo de falloff e para a fiação do chip, e **só mediu os dois eixos do report no
fim**. As medições da §3 são o único produto útil daquela tarde; **comece por elas.**

---

## §1 — Abrir a linha (rota "linha reaberta")

A branch e a worktree **já existem**. Siga
[`MODELO_ABERTURA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md),
rota *linha reaberta*:

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d
pwd && git branch --show-current      # DEVE dizer line/sculpt3d
git rebase main
```

⚠️ **Modo L: TODO comando começa com o `cd` da worktree.** A cwd do Bash **volta ao
repo primário** entre chamadas, e o mesmo path relativo existe nas duas árvores —
eu li `main` achando que lia o tip e **contradisse um agente que estava certo**.
Na dúvida, `pwd` antes de qualquer veredito.

**Referências (fora do repo, já clonadas):**
`/home/enio/Documentos/Recursos/BlenderSculpt` · `/home/enio/Documentos/Recursos/SculptGL`
⚠️ Blender é **GPL** ⇒ **só comportamento, nunca código**. SculptGL é MIT.

---

## §2 — O que JÁ EXISTE (não reconstrua)

A W8 landou **nesta linha** (`ea17b33af`). O verbo está inteiro e **a fiação está provada**:

| peça | onde | estado |
|---|---|---|
| kernel do alvo | [`stroke_target.rs:452`](../../../crates/ph2d-sculpt3d/src/stroke_target.rs) | `base + base_nrm·h`, `base` e `base_nrm` **congelados** |
| a lei que satura | `GripLaw::coat` em [`grip.rs`](../../../crates/ph2d-sculpt3d/src/grip.rs) | `disp += f·força·(1,05 − |disp|)`, o `layer.cc` verbatim |
| defaults do verbo | [`brush_verb_defaults.rs:115`](../../../crates/ph2d-sculpt3d/src/brush_verb_defaults.rs) | `coat=true`, `unit_accum=false`, `from_live=false` |
| `layer_height` | [`brush.rs:132`](../../../crates/ph2d-sculpt3d/src/brush.rs), default **0,1** | faixa `0..0,2` UI, `0..1,0` dura (RNA) |
| row do painel | [`rows.rs:329`](../../../crates/ph2d-panel-sculpt3d/src/rows.rs) | `UiLevel::Basic`, `show: verb == Layer` |
| chip (23º) | `ids::SCULPT3D_VERB[22]` | pintado, registrado, **e clicado por gate** |
| 12 gates de lei | `verb_layer_tests.rs` | **todos verdes** |

⚠️ **NÃO reabra a fiação.** O seam ([`tests/seam.rs:237-246` e `:528`](../../../crates/ph2d-panel-sculpt3d/tests/seam.rs))
já despacha `Click` em **cada um dos 23 chips** e clica o rect que o `paint` registrou.
Medido pela porta do artista, a demão **chega ao barro**:

```
esfregando sem soltar   passadas  1      2      4      8
  Layer                           0,077  0,096  0,100  0,100   <- SATURA no layer_height
  Draw                            0,087  0,173  0,341  0,656
soltando entre as passadas
  Layer                           0,077  0,154  0,309  0,618
```

Sonda: [`tests/probe_layer_product.rs`](../../../crates/ph2d-sculpt3d/tests/probe_layer_product.rs)
(`--ignored --nocapture --test-threads=1`).

---

## §3 — ⭐ O DEFEITO, E A CAUSA: o nosso alvo divergia do `calc_translations`

**FECHADO em 2026-08-16.** O report do Enio era de dois eixos (*"se aumentar
hardness ou Auto Smooth, Layer fica muito ruim"*) e a causa dos dois era UMA.

### A divergência

`layer.cc:99-103` (`calc_translations`):

```cpp
const float3 offset      = orig_normals[i] * height * displacement_factors[i];
const float3 translation = orig_positions[i] + offset - positions[i];
r_translations[i]        = translation * factors[i];
```

ou seja **`live + (meta − live) · factors`** — a translação sai do **VIVO** e leva
o **PESO**. O nosso kernel escrevia a meta de forma **ABSOLUTA a partir do
`base`** e **SEM** o peso, com um doc-comment a justificar as duas coisas (*"o
peso NÃO entra aqui"*, *"um alvo ancorado no VIVO subiria a cada passada"*).

⚠️ **As duas justificativas eram falsas contra a referência:**

- o `disp` **SATURA** (`clamp_displacement_factors`), então ancorar no vivo
  **não** deixa a meta crescer — a meta é `orig + n·altura·disp`, e `disp ≤ 1`;
- o Blender aplica o `factors` nos **DOIS** lugares porque eles respondem
  perguntas **diferentes** — dentro do `offset_displacement_factors` ele é a
  **TAXA** com que aquele vértice enche a demão, e no `calc_translations` é a
  **FRACÇÃO do caminho até a meta** que este dab anda.

### E o `factors` é o nosso `shape`, não o `w`

`calc_brush_strength_factors` (`sculpt.cc:7577`) chama **só** o
`BKE_brush_calc_curve_factors` — a curva, sem a força. A força vive no
`cache.bstrength`, que é o nosso `intensity` e já entra na recorrência do
`coat_step`.

### O EARLY-OUT tinha de morrer junto, e sozinho ele valia mais que o resto

`if coat && me.accum[s] >= keep { return; }` era **correcto** sob a lei absoluta
(chegado à demão cheia, re-escrever era um no-op) e sob a lei da referência ele
**DESTRÓI a feature**: `disp` cheio não quer dizer *o vértice está na meta*,
quer dizer *a demão já foi depositada* — se o auto-smooth o tirou de lá, é o dab
seguinte que o traz de volta. **O `calc_faces` do Blender não tem early-out.**

### MEDIDO (uma pincelada, esfera de fábrica)

| `auto_smooth` | relevo ANTES | relevo AGORA | espeto/relevo |
|---|---|---|---|
| 0,00 | 0,07707 | 0,07356 | 2,01 |
| 0,25 | 0,00935 | 0,09198 | 1,64 |
| 0,50 | 0,00517 | 0,08394 | 1,57 |
| 1,00 | **0,00164** | **0,06940** | **1,68** (era 53,4) |

O espeto **CAI** conforme o Auto Smooth sobe e o relevo **se mantém** — que é o
que a palavra promete. ⚠️ O **Draw** continua a ser aniquilado
(0,08738 → 0,00016) e isso está **CERTO**: ele é aditivo puro e não tem meta
para onde voltar.

### O eixo HARDNESS — a lei está portada, e o que resta é o Blender

Conferido **linha a linha contra a fonte**, os quatro passos estão no nosso
kernel, na ordem do `calc_faces`:

| passo | Blender | nosso |
|---|---|---|
| dureza antes da curva | `apply_hardness_to_distances` (`sculpt.cc:7549`) | `Brush::shaped_distance` |
| a curva é o `factors` | `calc_brush_strength_factors` | `shape = fall · keep` |
| acúmulo assintótico | `offset_displacement_factors` + `clamp` | `coat_step` |
| a translação | `calc_translations` | o braço `Verb::Layer` |

E a distância sai do **`pre`** nos dois (`calc_brush_distances(ss,
orig_data.positions, …)` ⇔ o nosso `from_live = false`).

⚠️ **O que sobra com dureza alta é o que a ferramenta É:** o `disp` satura em
`1` em **todo** vértice de peso não-nulo, dado tempo, então o regime permanente
de uma demão é uma **MESA de espessura constante** com a parede na borda da
pegada — a curva decide só a **TAXA**. Dureza alta encurta a subida da parede;
ela não a inventa.

⚠️ **E a não-monotonia medida (0,75 → 2,989 · 0,90 → 2,507) é DISCRETIZAÇÃO, não
defeito de lei:** a parede fica em `h · r`, e mover uma parede vertical 15% do
raio troca **quais** vértices a atravessam. Um espeto medido *por vértice* é
quantizado por isso. O `height` também já é o da referência (default `0,1` nosso
com o motivo escrito, faixas `[0, 1]` dura e `[0, 0,2]` de slider, as duas do
`rna_brush.cc:3230-3234`).

## §4 — O gate que MORREU, e por quê

O `a_finished_coat_stops_asking_for_work` afirmava *"o 64.º dab move ZERO
vértices"* — a premissa do early-out. Sob a lei da referência ela é falsa **por
desenho**: a demão **tem** de continuar a escrever, e é isso que a deixa
conviver com um alisador.

Substituído (não recalibrado) por `a_finished_coat_stops_growing`, que afirma a
propriedade que sobrevive e é a única que o artista vê — a **CONVERGÊNCIA**: a
demão para de subir, e para na fracção que a máscara deixa livre, com CONTROLE
sem máscara. Os outros **onze** gates do arquivo passam sem uma edição de
asserção, e é isso que prova que a troca só alcança o ramo da demão.

⚠️ **O que se perde é a defesa de CUSTO, e ela fica NOMEADA:** a demão volta a
mandar ao refit do octree e ao upload vértices que não se moveram. Se um dia
doer, a cura é comparar a **POSIÇÃO** com a meta (barato e correcto), **nunca**
voltar a comparar o `disp`.

## §5 — Armadilhas desta linha (pagas, não repita)

- ⚠️ **A cwd volta ao primário** (§1). Custou-me um veredito errado contra um agente correto.
- ⚠️ **Um gate de kernel é CEGO à fiação** — os 12 gates do Layer passam e não dizem
  nada sobre o chip. E um **gate de seam é cego à LEI**. Precisa dos dois.
- ⚠️ **Fixture que não contém o fenômeno**: o espeto medido sobre a malha inteira lê
  os polos da esfera; a tabela sai com seis linhas iguais e parece um achado.
- ⚠️ **Oráculo byte-a-byte tem de reproduzir a ASSOCIAÇÃO**: `u*u*u*u` diverge de
  `(u*u)*(u*u)` por **um ULP** já em `t = 0,02`.
- ⚠️ **`cargo test -p` NÃO roda `cargo fmt --all -- --check`** — o tip desta linha
  esteve fmt-vermelho em cinco arquivos e **só o ship o via**.
- ⚠️ **Arch-gates que fatiam fonte por índice de BYTE panicam** em prosa portuguesa
  (acento, `⚠️`). Curado hoje em `the_armed_transform_is_shown.rs`, com `read_dir`
  ordenado junto (a ordem dele é *unspecified*).
- ⚠️ **Desfaça mutação com `cp` de um backup, NUNCA `git checkout`.**
- ⚠️ Rode as suítes em **debug além de release** (precedente: pânico só em debug).
- ⚠️ Gates de relógio (`--ignored`) exigem `--test-threads=1` e `load < ~5`.

---

## §6 — Fronteiras

- **NÃO integrar, NÃO `git push`, NÃO rodar `scripts/foundational-integrate.sh`.**
  Integração e ship são **só por ordem explícita do Enio**, via agente integrador
  dedicado (CLAUDE.md §0.7 · DIRETRIZ §1.5.3-1.5.4). A linha fecha a wave, escreve o
  handoff e **PARA**.
- `rayon` novo ⇒ **ADR novo**.
- Contrato congelado (§6 do CLAUDE.md) ⇒ **PARE e reporte**.

---

## §7 — Aberto além do Layer (contexto, não fila)

- Os defaults de fábrica por-tool do Blender moram num **`.blend` binário** ⇒ a W1 e
  o *Draw Sharp* são **decisão de produto do Enio**, não dívida de engenharia.
- Duas pistas de um agente, **ainda não medidas**: o `Draw` em modo `B` ficou
  **5× mais forte por dab**, e quatro verbos (`Blob`, `ClayStrips`, `ClayThumb`,
  `MultiplaneScrape`) nascem em `B` com **metade** da força.
- As duas curvas de domo (`Dome` / `Dome4`) voltaram ao catálogo hoje
  (`ALL` 10 → 12) — **isso é o que eu fiz na sessão, e não era o pedido.**
