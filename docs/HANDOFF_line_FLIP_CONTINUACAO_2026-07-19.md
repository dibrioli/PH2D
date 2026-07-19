# Handoff de CONTINUAÇÃO — `line/FLIP` (2026-07-19)

> **Para o agente que assume a linha FLIP.** A rodada anterior FECHOU e **INTEGROU ao
> `main`**. Você reabre a linha do ponto integrado. Este doc é o item 5 da FASE 2 do
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md):
> onde paramos e o que vem a seguir.

---

## 0. Primeiro, reabra a linha (não pule)

A worktree `Worktrees/line-FLIP` existe e a branch `line/FLIP` está **0 commits à frente,
62 atrás** do `main` — tudo o que a linha fez já está no `main` (o integrador rebaseou, então
os SHAs mudaram: o meu `27cfa20c` virou `3e66d61a`, etc.).

```bash
cd Worktrees/line-FLIP && pwd && git branch --show-current   # FASE 0
git rebase main                                              # FASE 1 — vira FF limpo
cargo check -p ph2d-flip-fill                                # 1º build pode ser frio
```

Depois do rebase a linha == main. Você começa do zero de delta, com todo o histórico já
shipado.

⛔ **A regra que mais custou nesta bancada:** toda janela abre na RAIZ do primário, que é
`main`. Os mesmos paths relativos existem lá e na sua worktree, e editar o da raiz **compila,
testa e commita sem um único erro** — ninguém descobre até a integração. Na dúvida, `pwd`.
Antes de todo commit, `git branch --show-current`.

---

## 1. Onde paramos — o que a última rodada entregou

A rodada foi uma auditoria + conserto do **balde de tinta**, disparada por um smoke do Enio
que reprovou pela 5ª vez (*"nenhuma melhoria e nenhuma mudança. ainda extravasa"*). Fechou
com **"SMoke OK"** e integrou.

| entrega | commit no main | uma linha |
|---|---|---|
| **BUGS #22** — a dilatação inteira era contagem dupla | `3e66d61a` | a lei do fill virou `2s` (só o erro de vetorização, com sinal); o termo da espessura da linha morreu — era ele a franja de 5 smokes |
| **Buracos na rota de curvas** — o donut | `f4b7580a` | um buraco é uma **COMPONENTE conexa**, invisível à caminhada de half-edge; agora ela os associa ao anel |
| **R3 REVOGADA** por medição | `86375cca` | aposentar o `filled_shape_target` descolava a cor da linha em até 5,8 larguras; virou **proteger** a propriedade com 5 gates |
| cena de smoke do balde | `dd6eb8c6` | `PH2D_FLIP_FILL_SMOKE=1` |

O detalhe técnico inteiro está em dois lugares que **já estão no main** — leia-os antes de
tocar o balde:

- **`docs/Flip/BUGS_flip.md` #22** — a saga completa, com as tabelas de medição contra o
  Draw:Filled e as duas armadilhas de oráculo que a rodada pagou.
- **`docs/Flip/10_regiao_por_curvas.md` §11 e §12** — o que o #22 mudou na wave, e por que a
  R3 foi revogada (com a medição das duas rotas na mesma arte).

E o handoff de integração daquela rodada, se você quiser o mapa de arquivos:
`docs/HANDOFF_line_FLIP_INTEGRACAO_2026-07-18b.md`.

---

## 2. As TRÊS coisas que não são detalhe (herança que morde)

### 2.1 ⛔ NÃO "termine" a fatia R3

O plano `10_regiao_por_curvas.md` **em versões antigas** mandava aposentar o
`filled_shape_target`. **Foi revogado** (§12, já no main). Medido: ele põe o `fill` no
PRÓPRIO traço, então esculpir a linha move a cor junto; a rota das curvas COPIA os vértices
para um traço novo, e sob seleção a cor fica para trás até 5,8 larguras de linha.

Há **5 gates** protegendo isso (`shells/desktop/src/flip_fill_identity_tests.rs`); 3 sangram
se você desligar o ramo. Se algum plano seu esbarrar nessa remoção, o pré-requisito é a
costura **fonte ≠ cozido no nível da REGIÃO** (o padrão do ADR-0121 das Live Corners) — é
projeto com aceitação própria, não a remoção de um caso particular.

### 2.2 `ph2d-flip-fill` é **foundational** e a API pública mudou nesta rodada

Removidos: `FILL_TUCK_FRACTION`, `contour_widths_with_margin`, `mean_line_width` (o termo que
serviam morreu no #22). Acrescentado: **`FillResult.scale`** (a resolução que a grade de fato
ENTREGOU). Fora do módulo Flip **ninguém consome** essa crate hoje.

### 2.3 A lição de método que a rodada pagou duas vezes — **meça a coisa CERTA**

Duas correções ao meu próprio trabalho, ambas viradas memória:

- **Onze oráculos de pixel ficaram verdes com o defeito na tela.** Não era barra frouxa:
  a fixture usava linha OPACA (o único ponto onde o defeito é zero), a janela de medição
  começava além de onde a cor pousava, e todas as fixtures usavam a topologia em que o
  produto vai pela rota que NÃO dilata. `[[feedback_the_approved_reference_may_already_be_in_the_product]]`.
- **Comparei duas rotas em ARTES diferentes** e a conclusão saiu do fixture, não da rota.
  `[[feedback_comparing_two_routes_requires_the_same_art]]`. Ao comparar caminhos, force a
  MESMA entrada — se uma rota não roda naquela arte, ESSE é o achado.

---

## 3. Os planos a seguir (a fila, em ordem de recomendação)

O módulo **já é usável de ponta a ponta** por um animador (desenhar → frames/hold/ciclos/ghost
→ tween → balde → reshape → editar/selecionar/transformar → camadas → instâncias → multiframe
→ salvar). A única lacuna estrutural de fluxo é **export**, e ela é deferida de propósito
(provavelmente pertence a um pipeline do projeto, não ao Flip). Então a fila não é tapar buraco
de fluxo — é acabamento e a próxima wave.

### 3.1 HIGIENE primeiro (custa minutos, evita uma rodada perdida)

**`docs/Flip/01_plano_waves.md` está MENTINDO.** Cabeçalho de 2026-07-12, **não menciona 8
waves que landaram depois**, e lista **6 itens como abertos que EXISTEM no código**:

| o doc diz aberto | está em |
|---|---|
| "transformar a seleção (girar/escalar)" | `shells/desktop/src/flip_selection_gizmo.rs` (518 LOC) |
| "domínio Point" | `shells/desktop/src/flip_select_points.rs` |
| "destrava o segment mode" | `shells/desktop/src/flip_select_segment.rs` |
| "multi-seleção de chaves na tira" | `flip_reshape.rs` → `flip_multiframe::targets(…, strip.selected_keys(), …)` |
| "modo Selected dos fantasmas" | `onion.rs` consome `strip.selected_keys()` |
| "instância de drawing na UI" | botão `FLIP_KEY_INSTANCE` + `INSTANCE_DOT` |

Ele chega a se contradizer (a tabela diz *"Multiframe: LANDOU"* enquanto os checkboxes T4.5/T5.7
seguem `[ ]`). **É o modo de falha exato que o módulo de áudio já pagou: uma lista velha faz a
próxima LLM construir o que existe** — e nesta última sessão ela quase pegou duas leituras.
Antes de propor qualquer coisa, corrija esta lista contra o código.

### 3.2 C2 — LazyBrush (a próxima WAVE, o maior multiplicador de produto)

`docs/Flip/09_colorize.md` §7, plano pronto com kill-criterion escrito. Rabiscar cores em vez
de clicar região a região (a feature que só o TVPaint entrega). Zero `max_flow`/`min_cut`/
`boykov` no repo — é a peça mais pesada e é a wave inteira; a C1 (Trap) já entregou valor sem
ela.

⚠️ **Abre com DUAS decisões que são do Enio, não suas** — leve as duas juntas:

1. **Medir o corte binário na grade ANTES de construir UI** (§7.1) — é o número que decide
   entre operação síncrona e o padrão `progress` (`ph2d-editor-core::progress`, a infra de
   barra do app inteiro).
2. **O pedido de exceção `rayon` para a EDT** (67% do custo do Trap) — **BARRADO por
   ADR-0109**, exige ordem do Enio + ADR novo. As alavancas single-thread estão esgotadas e a
   tabela de perf está pronta.

Depois da C2 vem a **C3 — onion fill** (o rabisco atravessa o range; o range já está encanado,
só a semente é nova — uma sessão).

### 3.3 Backlog verificado (itens menores, cada um com gatilho)

| item | tamanho | nota |
|---|---|---|
| Buracos: **Grow/Trap armados** ainda caem na rota velha | — | recusa deliberada (a rota nova põe a fronteira no eixo, não sabe deslocar) |
| Arranjo **O(segmentos²)**: 80,8 ms com 200 traços (kill = 100 ms) | horas | passa com pouca folga; broadphase por grade é o caminho, sonda em `probe_arrange_perf.rs` |
| **T3.9 light table** (marcar quadros de referência) | horas | o passe de ghost já aceita a lista; falta a UI |
| **T3.7 cauda** — easing picker + fade-in dos órfãos no Tween | horas | carry-over de UI, `TweenOptions` já suporta |
| Drag de célula/borda na tira | sessão | hoje move por botão + caixa numérica |
| Painel espelhar a seleção (write-back) | sessão | controles são "aplique", não espelho |
| **Congelar o contrato do `ph2d-flip`** | horas | não há gate de superfície; nota do plano diz "quando o modelo assentar". O `FLIP_SCHEMA_VERSION` foi a 8 |
| Tween v2 (matching espacial + espiral log) | wave | spec pronta no `04 §2` |
| Self Overlap · corner types · airbrush · SDF · Shift&Trace · 2.5D · SMAA | horas–wave | deferidos declarados do `03 §8` |
| Modo Radius do Gap Closure | — | ⚠️ **candidato a APOSENTADORIA, não a construção** — o Trap (C1) responde melhor à mesma pergunta |

---

## 4. Regras da linha (não estão copiadas aqui de propósito — leia na fonte)

- **DIRETIVA_IMPLEMENTACAO.md** inteira, e releia a cada passo (é o antídoto das 4 causas da
  semana perdida no Painter: costura não-testada · "audit" = compilar · isolamento órfão ·
  alvo irrefutável).
- As **REGRAS PERMANENTES A–H** do `MODELO_ABERTURA_LINHA.md` — valem iguais para você.
- **Você fecha a linha, escreve o handoff e PARA.** Integração e ship só por ordem EXPLÍCITA
  do Enio, via agente integrador. Integrar/pushar sozinho = violação do protocolo (§0.7).

---

## 5. Como testar o que já existe

```bash
cd Worktrees/line-FLIP
cargo test -p ph2d-flip-fill                    # 64 — o motor do balde
cargo test -p ph2d-host-desktop                 # o shell
cargo test -p ph2d-flip-render -- --ignored     # 10 oráculos de PIXEL (precisa de GPU)
```

Smoke visual do balde: `PH2D_FLIP_FILL_SMOKE=1 cargo run --release -p ph2d-host-desktop`
(monta moldura de 4 traços com ilha + forma sozinha; pincel macio e arte trêmula de propósito;
roteiro impresso no terminal). Os demais smokes do módulo: grep `PH2D_FLIP_*_SMOKE` no shell.

---

## 6. Reporte de abertura (o que o Enio espera ver)

> "Assumi line/FLIP em Worktrees/line-FLIP (HEAD \<sha\>). Rebaseei no main (FF, 0 à frente).
> O balde fechou e integrou; a fila é higiene do 01_plano_waves + C2 (LazyBrush). Aguardo a
> tarefa." — e PARE.
