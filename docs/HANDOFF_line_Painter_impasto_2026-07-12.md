# HANDOFF — `line/Painter` · Impasto (#16) · 2026-07-12

> ✅ **CUMPRIDO (mesmo dia, dono novo).** A pesquisa do §4 foi feita (5 varreduras primárias →
> [`docs/Painter/17_impasto_deposito_pesquisa2.md`](Painter/17_impasto_deposito_pesquisa2.md)), a
> hipótese do §2 **confirmada por medição e pela indústria inteira**, o plano atualizado
> ([16 §10](Painter/16_impasto_plano_implementacao.md)) e a Fase 4 (o CORPO) **landou**
> (`2c3492ab`): curva de corpo, inclinação física, `Amount` morto, teto de vidro. Estado vivo:
> [HANDOFF_line_Painter_integracao §0](HANDOFF_line_Painter_integracao_2026-07-12.md).
> **Pendente: smoke do Enio sobre o modelo novo.** O texto abaixo é o handoff ORIGINAL, mantido
> como registro.

> **Você é o novo dono desta linha.** Leia isto inteiro antes de tocar em código.
> Worktree: `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter` · branch `line/Painter`
> **30 commits** sobre `main`, 70 arquivos, +8064/−1579. Árvore limpa, tudo verde.

---

## 0. O PEDIDO DO ENIO (é isto que abre o seu trabalho, não o backlog)

Após cinco rodadas de correção, o Enio olhou o resultado e disse:

> **"Não sei se melhorou ou piorou. Ficou mais difícil de ajustar."**

Isso **não é um bug report**. É o veredito de que a feature está tecnicamente correta e
**ergonomicamente errada**. Ele quer, nesta ordem:

1. **Pesquisa nova, de verdade** — como o estado-da-arte *deriva altura de um traço* e *como
   apresenta os controles*. (§4: o que perguntar. §3: por que a pesquisa existente **não cobre** isso.)
2. **Atualizar o plano** ([`docs/Painter/16_impasto_plano_implementacao.md`](Painter/16_impasto_plano_implementacao.md))
   com o que a pesquisa achar.
3. **Assumir a linha** em Modo L (§7).

**Não comece implementando.** O erro que produziu este handoff foi exatamente esse.

---

## 1. O que existe hoje (e funciona, com gate)

**Impasto = a altura é o SEGUNDO output do pipeline de dab que já existia**, tomada num **choke point
único** em `stamp_dabs_routed`, acima de todas as rotas de cor. Daí toda a integração saiu de graça:
Shape, Shape Tone ramp, Grain, Falloff, Stroke, shape editors, Jitter, **Tiling**, **Symmetry**,
**Per-Layer Color**. **Watercolor não foi tocado** (short-circuita antes; há gate).

**24 gates de impasto**, cada um com **vermelho verificado por mutação**:

| arquivo | o que garante |
|---|---|
| `crates/ph2d-painter-brush/src/height.rs` | kernel: envelope, cápsula, grão, borracha |
| `crates/ph2d-tool-painter/src/tool/paint/impasto.rs` | depósito, commit, live-edit, varredura |
| `.../impasto_light.rs` | o passe de luz (relativo, pesado por cobertura) |
| `.../impasto_settings.rs` | setters + rota de painel + a matriz §1.2 |
| `crates/ph2d-panel-painter-layers/src/paint_impasto.rs` | cards Body (por-pincel) + Lighting (por-canvas) |

**Perf:** 1,97 ms/move médio (2,14 pior) @2048² r100. Alvo ≤4, kill em 8. `impasto_perf_kill_criterion`.

**Smoke armado:**
```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
  PH2D_IMPASTO_SMOKE=1 cargo run --release -p ph2d-host-desktop
```

---

## 2. ⚠️ A HIPÓTESE QUE EU DEIXO NA MESA (teste antes de aceitar)

**Suspeito que o modelo está errado na raiz, e que é por isso que "ficou difícil de ajustar".**
Não consegui provar antes de fechar. **Trate como hipótese, não como fato.**

### 2.1 A altura é derivada da COBERTURA de um pincel macio ⇒ o relevo é um domo, não um corpo

Hoje: `h = Depth × cobertura × silhueta (× grão)`. A silhueta de um pincel default é um falloff
**Smooth** — um domo suave de raio 40. Logo o relevo é **um tubo de seção arredondada**, e é
exatamente assim que ele lê na tela do Enio.

**Tinta grossa de verdade não é um domo.** É um **corpo com borda**: quase vertical na saída,
irregular no topo, com marcas de ferramenta. A luz de um impasto real vem **da borda** e da rugosidade
do topo — não de um gradiente macio.

Se isso estiver certo, **nenhuma calibragem de knob conserta**, porque o problema é o *perfil*, não o
*ganho*. Candidatos a investigar: perfil de altura próprio (mais duro que o de cor) · volume de tinta
com carga/depleção (ArtRage modela isso explicitamente) · rugosidade do topo separada da silhueta.

### 2.2 A superfície de knobs cresceu acoplada — e alguns fazem a mesma coisa

Expostos: **Depth · Smoothing · Depth Source · Draw To** (pincel) + **Show · Angle · Elevation ·
Amount · Shine** (canvas). Escondidos, calibrados **na mão por mim**: `SLOPE_GAIN=40` ·
`GRAIN_GROOVE=0.65` · `AMBIENT=0.35` · `SHININESS=24` · `SETTLE_MAX_PX=4`.

**Depth e Amount escalam a mesma percepção** (espessura × ganho da normal). **Smoothing e a maciez do
falloff** borram a mesma coisa. Girar um mexe no outro — é a definição de "difícil de ajustar".

Um modelo principiado derivaria essas constantes em vez de tê-las como fudge. **Não as ajuste. Questione
se elas deviam existir.**

### 2.3 A última mudança pode ser regressão de AJUSTABILIDADE (mesmo estando correta)

O passe agora pesa a sombra pela **cobertura da tinta** — correto, e matou o halo branco (§5, foto do
Enio no preto vs branco). **Mas** isso faz a força do relevo cair junto com a opacidade nas bordas, e
num pincel macio a borda é enorme. Resultado plausível: o relevo "derrete" nas bordas e o traço lê como
borrão, não como corpo.

**Mede isso primeiro.** Se confirmar, a saída provavelmente não é desfazer (o halo era real) — é §2.1:
a altura não devia herdar o perfil macio da cor.

---

## 3. Por que a pesquisa existente NÃO cobre a sua pergunta

[`docs/Painter/15_impasto_pesquisa_e_design.md`](Painter/15_impasto_pesquisa_e_design.md) estabeleceu
que Painter/ArtRage/Rebelle/Substance usam **height-map + luz global, normal por diferença central**, e
que **Procreate não tem canal de altura** (pinta o 3D no bitmap). Isso é verdadeiro e útil.

**Mas ela não pergunta a coisa que importa agora:** *como o traço vira altura?* Eu **inventei** o modelo
de depósito (envelope por magnitude, cápsula, groove do grão) — sem referência. Funciona, é coerente, e
pode estar errado no formato.

---

## 4. O que pesquisar (perguntas concretas, não "estude impasto")

1. **Corel Painter — Impasto real.** Depth Method (`Uniform` / `Erase` / `Paper` / `Original Luminance`),
   `Depth`, `Smoothing`, `Plow`, e o diálogo **Impasto Lighting** (múltiplas luzes, Brightness / Shine /
   Reflection / Concentration / Exposure / Surface Depth). **Eu inventei um subconjunto adjacente sem
   comparar.** O que eles expõem, e o que eles *não* expõem?
2. **ArtRage.** Modela **volume de tinta com carga que se esgota** (o pincel *acaba*). Isso é o que dá o
   arraste, a borda dura e o topo irregular. **É um modelo de depósito diferente do meu.** Vale?
3. **A BORDA.** Como o estado-da-arte impede que a altura seja o domo macio da cor? Perfil próprio?
   Limiar? Superfície do topo separada da silhueta?
4. **Rebelle / Krita.** Krita tem 4 luzes; Rebelle tem environment maps. **Quantos knobs eles realmente
   expõem, e quais são canvas-level vs brush-level?** (A minha divisão Body/Lighting foi um chute meu.)
5. **Substance / normal-from-height.** Como escolhem a escala de inclinação sem que vire um knob mágico?
   (O meu `SLOPE_GAIN` é chute calibrado.)
6. **A pergunta de UX que o Enio fez sem fazer:** qual é o **menor conjunto ortogonal** de knobs que dá
   controle real? Se a resposta for "3", proponha 3 e mate o resto.

Regra da casa: [[feedback_no_industrial_claims_without_verification]] — **zero claim em ADR/doc sem
grep / cargo-search / WebFetch.**

---

## 5. Histórico honesto — 5 rodadas, e o que cada foto do Enio revelou

Isto não é decoração: mostra que **cada avanço veio de uma observação dele, não de leitura de código.**

| foto | achado | fix |
|---|---|---|
| costelas + pontas pretas | **sombra com piso ZERO** esmagava a tinta a **0%** da cor | `AMBIENT`, dobrado pra que o plano ainda devolva exatamente 1.0 |
| idem | luz sombreava **relevo onde não há tinta** (a normal vem da *inclinação*; um filme com grão tem micro-inclinações de crista) | fade por corpo + escalar o gradiente |
| **plano** | `SLOPE_GAIN=8` escolhido **por gosto** — a inclinação real é 0,026/px ⇒ normal tombava 6°: nada. E `DepthSource::Grain` **multiplicava a massa** pelo grão (média < ½): Depth 0.7 virava 0.21 | calibrado a 40 (medido); grão passa a **entalhar sulcos numa massa cheia** |
| **3 spacings** | relevo dependia da **amostragem**: envelope = `max` de discos ⇒ festão entre centros. 0.1/0.05/0.01 → costelas/leve/tubo | **cápsula** até o centro do dab anterior. Ondulação 0.0148 → **0.0000** em todo spacing e todo jitter |
| **branco vs preto** | halo branco só no branco ⇒ a luz sombreava o **papel visto através da tinta** (×1.65 branqueia rosa pálido). Media **81 níveis na borda** vs 55 no núcleo — invertido | campo de **cobertura** (1 B/px) ao lado do relevo; a luz pesa pela tinta |

**A perf também mordeu:** materializar o campo composto custava `O(canvas)` **por frame** (3,93 ms,
colado no alvo) enquanto o passe só ilumina o retângulo sujo. Agora amostra as camadas no lugar: 1,97 ms.

### A lição que vale mais que o código

**SETE vezes** um teste meu ficou **verde pelo motivo errado**, sempre igual: **a fixture não continha o
fenômeno.** Cache testado em dois tools (frio nas duas vezes). Opacidade com camada de baixo cheia (o
over-composite satura). Smear em região uniforme. Disco duro chamado de "crista" (platô de parede
vertical não tem flanco). Papel branco usado como "tinta plana". Gate do corrugado dividindo por
variância **zero**. Referência lida **depois** do bug já tê-la destruído (comparou 0 com 0 e aprovou).

**Em quase todos, foi DESLIGAR o fix que expôs o teste falso.** Por isso **todo gate desta linha tem
vermelho provado por mutação** — e por isso você deve fazer o mesmo. Um gate verde que você não sabe
derrubar não é um gate.

E duas vezes eu **medi a coisa errada** e afirmei com confiança: autocorrelação num lag de 3 px (qualquer
campo suave dá 0.9) e a periodicidade **na linha central** (onde o falloff é platô e não escalona). **O
Enio viu com os olhos o que minha instrumentação não via.** Quando ele disser que algo está estranho e o
seu número disser que não, **o número está medindo a coisa errada.**

---

## 6. Aberto

| item | estado |
|---|---|
| **§2 — o modelo de depósito** | **a sua tarefa**: pesquisar, propor, atualizar o plano |
| `Plow` (Smear arrasta o relevo) | nomeado, deferido. É o gesto de espátula |
| Composite Depth por camada (Add/Subtract/Replace/Ignore) | modelo de dados **já nasce per-layer**; falta só o composite |
| Passe de luz na GPU | 8 slots livres em `AdjustmentKind ≤ 32`; exige reconciliação **bit-a-bit** contra a CPU |
| Relevo do PAPEL | **acopla impasto↔aquarela ⇒ exige ordem NOVA do Enio.** Não faça por conta |
| Persistir `h`/`cover` no `ProjectState` | herda o gap conhecido (o save já não persiste pixels de `SpriteSource::Individual`) |
| **Watercolor OFF→ON no meio do traço** apaga tinta | **da varredura, aberto de propósito**: não consegui construir um RED. Escrevi o fix e **revertí** — sem vermelho refutável não se mexe, menos ainda na aquarela. `BUGS_painter.md` #13 |
| Cor muda com o spacing | **NÃO é bug do impasto.** É densidade de depósito; "Adjust Strength for Spacing" normaliza (159→32 níveis). O Enio desligou esse knob por default em 2026-06-24 — **cerca de Chesterton, não mexa sem perguntar** |

---

## 7. MODO L — o protocolo, e ele não é negociável

Você trabalha numa **worktree isolada** ([ADR-0106](architecture/decisions/0106-parallel-dev-lines-worktrees-workstation.md) /
[ADR-0107](architecture/decisions/0107-concurrent-foundational-lines-tested-gate-syntactic-merge.md)).

- **NUNCA `git push`. NUNCA `./scripts/ship.sh`. NUNCA integrar no `main`.** Integração e ship são
  **ordem EXPLÍCITA do Enio**, executadas por um **agente integrador dedicado**. Você **fecha a linha,
  escreve o handoff de integração (DIRETRIZ §1.5.9) e PARA.** Fazer isso sozinho = **violação do
  protocolo** (CLAUDE.md §0.7).
- **Foundational você PODE tocar** (`ph2d-painter-brush`, `ph2d-editor-core`) sob o protocolo testado —
  mas **projete para isolamento** (módulo irmão / extensão append-only) e **anote no handoff**.
- **PARE e reporte ao Enio** em 2 casos: **contrato congelado** (CLAUDE.md §6 — nenhum foi tocado aqui) ou
  **rebase conflitando fora dos seus arquivos**.
- **`cd` em TODO comando.** O cwd volta pro repo `main` a cada turno. Mutação **só por caminho absoluto** —
  um `sed -i` relativo escreve no repo errado ([[feedback_sed_relative_path_hits_primary_cwd]]).
- Commits locais com `--no-verify`. Gate batched no fechamento, não por task.

### Superfície tocada (para prever conflito de merge)

- **`ph2d-editor-core`** — **só append**: `ids/chrome/painter_impasto.rs` (NOVO) + 2 linhas no `mod.rs`.
- **`ph2d-painter-brush`** — `height.rs` (NOVO) · `spec.rs` (+campos `impasto_*`, append; testes inline
  extraídos p/ `spec_tests.rs` pelo teto de LOC) · `dab.rs` (silhueta extraída p/ `silhouette_at`,
  chamada pelo kernel de cor — **byte-idêntico**, 239 testes provam) · `texture.rs` (`rotate_by_degrees`
  virou `pub`).
- **`shells/desktop`** — `impasto_smoke.rs` (NOVO) + 1 guard em `painter_gpu_preview.rs` + 3 linhas em
  `painter_bridge.rs` + 1 campo em `app_state.rs`.
- **⚠️ `paint_watercolor.rs`** — `card_frame`/`card_row` **extraídos** p/ `card.rs` (movimento **puro**:
  −92/+1). **A óptica da aquarela NÃO foi tocada.** Se outra linha editou esse arquivo, o Mergiraf pode
  se confundir.
- **Contratos congelados: NENHUM tocado.**

### Gate de fechamento

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
  cargo fmt --all && \
  cargo test -p ph2d-painter-brush -p ph2d-tool-painter -p ph2d-panel-painter-layers -p ph2d-editor-core && \
  cargo clippy -p ph2d-painter-brush -p ph2d-tool-painter -p ph2d-panel-painter-layers -p ph2d-host-desktop --all-targets
```
**Não rodei `ship.sh`** — o gate por-linha não roda fmt-workspace / machete / deny / typos. **Orce 2–4
iterações de ship vermelha** para o integrador ([[project_integrator_ship_catches_latents_budget_iterations]]).
Um `| grep` **mascara o exit code** — verifique o **estado**, não o `$?` ([[feedback_pipe_masks_script_exit_code]]).

---

## 8. Leitura obrigatória (nesta ordem, e só isto)

1. [`CLAUDE.md`](../CLAUDE.md) — §0 inteiro (os 7 inegociáveis).
2. [`docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md`](IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md)
   — **a cada passo**. Regra-mãe: *verde-de-compilação é velocidade; no audit vale ZERO.*
3. [`docs/Painter/16_impasto_plano_implementacao.md`](Painter/16_impasto_plano_implementacao.md) — o plano
   **e a §9**, que lista onde a implementação divergiu dele e por quê.
4. [`docs/Painter/15_impasto_pesquisa_e_design.md`](Painter/15_impasto_pesquisa_e_design.md) — a pesquisa
   existente, e §3 acima explica o que ela **não** responde.
5. [`docs/Painter/BUGS_painter.md`](Painter/BUGS_painter.md) — **#12** (o SIGSEGV que abriu a linha) e
   **#13** (a varredura de 9 achados).
6. [`project-memory/MEMORY.md`](../project-memory/MEMORY.md) — o índice. Leia antes de agir.

---

## 9. Como o Enio quer ser tratado

- **Decida, não pergunte** ([[feedback_decide_dont_ask_gold_standard]]). Recomendação primeiro, opções
  concretas depois. Sem AskUserQuestion-spam.
- **Padrão-ouro sem adiamentos**: gaps in-scope fecham na sessão.
- **Exemplo pronto pra smoke**: feature nova **entrega** o exemplo que a demonstra; não pede pro artista
  montar um ([[feedback_ready_to_smoke_example]]). O `PH2D_IMPASTO_SMOKE` é o precedente — mantenha.
- **Comando de rodar inclui o `cd`.**
- **UI em inglês** (labels/toasts), sempre.
- Ele **testa com os olhos**. Quando ele diz que está estranho, **está** — mesmo que seu número diga que
  não. Ver §5.
