# Handoff de CONTINUAÇÃO — `line/FLIP` (2026-07-20)

> **Para o agente que assume a linha FLIP.** A linha está **VIVA e NÃO integrada**: 16
> commits à frente do `main` (HEAD `d0c05a5c`), árvore limpa. Este doc é o item 5 da FASE 2
> do [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md):
> onde paramos, o que o Enio pediu ("**quase perfeito. vamos tentar melhorar**"), e o que
> já foi medido e REPROVADO — para você não reconstruir o que já morreu.

---

## 0. Primeiro, confirme onde você está (não pule)

```bash
cd Worktrees/line-FLIP && pwd && git branch --show-current   # FASE 0
git log --oneline -5 && git status -sb                       # HEAD tem de ser d0c05a5c
cargo check -p ph2d-flip-colorize                            # 1º build pode ser frio
```

⛔ Toda janela abre na RAIZ do primário, que está em `main`. Os MESMOS paths relativos
existem lá e aqui; editar o da raiz compila e commita **sem erro** e ninguém descobre até
a integração. Na dúvida, `pwd`. Antes de todo commit, `git branch --show-current`.

⚠️ **NÃO rebase no meio da jornada sem motivo** — a linha não integrou; `git rebase main`
é para o INÍCIO de jornada se o `main` andou (DIRETRIZ §1.5.2.3). Confira
`git log --oneline line/FLIP..main` antes: se vazio, siga direto.

---

## 1. O que esta rodada construiu (tudo commitado, tudo smokado)

A wave é o **Colorize** (`docs/Flip/09_colorize.md` — o plano; leia o **Estado** no topo
dele, que é a versão longa deste resumo). Em ordem:

1. **C2 — motor + modo clicável** (`485b671b`..`9b8dad4e`): crate `ph2d-flip-colorize`
   (`colorize()` headless) + 7º `FlipMode::Colorize` no shell (rabiscar → **Apply** →
   regiões de GEOMETRIA pelo mesmo back-end do balde). Overlay ao vivo; undo do Apply.
2. **§7.1 — a régua do caminho real** (`e0655fb5`, `a26f37ad`): o corte de pixels do
   LazyBrush custa 3,3 s a 4096² e **157 s** com rabiscos se contradizendo — a
   pré-segmentação virou prioridade.
3. **§8 — partição trapped-ball** (`3359a39b`, `bc1b11f5`): 157 s → 586 ms.
4. **3º smoke** (`ab59080a`): 4 cores num blob ABERTO viravam 1 — subdivisão condicional
   (componente de UMA cor é PREENCHIDO; contestado é dividido).
5. **Anel do pincel no Colorize** (`59e433ba`).
6. **4º smoke "impreciso" → o contestado virou Voronoi POR PIXEL** (`d0c05a5c`, o estado
   atual — detalhe em §2 abaixo).

**5º smoke (2026-07-20): "quase perfeito. vamos tentar melhorar."** As fotos: a caixa com
divisor ondulado sai com a fronteira **na linha** (vermelho/azul), e os 4 lobos do blob
saem cada um com a sua cor, fronteiras nas aberturas. As ressalvas visíveis estão em §3 —
**são a sua primeira tarefa**.

---

## 2. O motor como está (não re-derive — está gateado e mutation-tested)

`crates/ph2d-flip-colorize/src/`:

- **`lib.rs`** — `colorize()` (porta pública, assinatura estável) + `solve()`: partição →
  componente de UMA cor = FILL · contestado (≥2 cores) = `voronoi::claim`.
- **`segment.rs`** — trapped-ball: EDT (`sq_distance_to_set`) → núcleo → componentes
  (flood 4-conexo **por papel**; a bola de raio `trap_px` costura vãos < 2r) → dilatação
  por papel → papel-fino vira área própria → tinta por último. Devolve
  `Segmentation { component, count, ink_dist2 }`. ⚠️ **As CÉLULAS e o grafo de arestas
  MORRERAM aqui** (4º smoke): célula > 1 px cavalgava a linha, e `V_pq` como métrica de
  distância tem a direção ERRADA (é custo de corte: tinta grátis ⇒ linha invisível).
- **`voronoi.rs`** — o contestado: Dijkstra multi-fonte POR PIXEL (fila de baldes de Dial,
  só inteiros), **3 leis**: (1) **tinta INTRANSPONÍVEL** (a frente anda só por papel — o
  casamento com a linha vem da geometria, não de peso); (2) **chanfro 5/7** + recusa do
  passo diagonal com tinta nos DOIS cantos (senão linha diagonal é peneira); (3) **pedágio
  de aperto** `SQUEEZE/(1+d²)`, `SQUEEZE = 4096` **MEDIDO** — a tabela da varredura está
  na doc da const (película · lente pelo vão · parelheza do blob). A trapped-ball em forma
  contínua: vão estreito quase sela, vão largo é passagem honesta.
- **`flow.rs`** — `#[cfg(test)]`: a referência BK ≡ Edmonds–Karp. **NÃO é o produto.**

**Suíte: 23 verdes** (`cargo test -p ph2d-flip-colorize --release`), 4 réguas `--ignored`.
**Mutações 3/3 sangram** — ⚠️ as leis 1 e 2 exigiram **gates isolados com EDT fabricada**
(`ink_is_a_wall_even_with_zero_toll` / `a_diagonal_pinch_is_sealed_even_with_zero_toll`):
com o pedágio vivo as camadas se cobrem e mutar UMA fica verde
([[feedback_layered_defenses_need_per_layer_gates]]).

⚠️ **Armadilha de fixture paga 2×:** precisão 80 ⇒ grade ~104 px. Na era das células isso
dava célula de 1 px (o defeito não existia); os gates de fronteira rodam a **precisão 400**
de propósito. Fixture novo de Colorize: confira o tamanho da GRADE, não só da arte.

**Perf** (régua `measure_the_product_colorize_cost`, pior caso = caixa inteira contestada):
512² = 15 ms · 1024² = 67 ms · 2048² = 348 ms · **4096² = 1,70 s** (~1,4 s é a PARTIÇÃO).

---

## 3. A MISSÃO: "vamos tentar melhorar" — as ressalvas do 5º smoke

Do que se VÊ nas duas fotos aprovadas-com-ressalva, três candidatos (o Enio não especificou
qual; **diagnostique-e-meça antes de codar, e confirme com ele o alvo** se a escolha mudar
o produto):

### 3.1 — O SERRILHADO na fronteira que corre AO LONGO da linha ⚠️ (o mais visível)

Na caixa: onde a fronteira azul/vermelho corre em cima do divisor ondulado, o lado AZUL
mostra dentes de serra regulares (~5-10 px); o lado vermelho é liso. **Não diagnosticado —
não chute.** Hipóteses a testar, em ordem de suspeita:

- **O pedágio é ESCADA perto da linha**: `4096/(1+d²)` com d inteiro dá 2048 → 819 → 409…
  A faixa rente à tinta é cara em degraus, a frente a preenche por último, em rajadas
  diagonais do chanfro — e quem chega "por último em rajada" pode ser dono alternado.
- **A dança com o `expand_under_ink`** (`trace_region`): as duas cores cravam sob a linha
  até o eixo; numa linha ondulada os avanços podem alternar.
- **O RDP não alisa** (`simplify_ring`, `RDP_EPSILON_PX`): serra que sobrevive tem
  amplitude ≥ ε — medir a amplitude no plano `assign` diz se a serra nasce no Voronoi ou
  no traçado.

**Como diagnosticar:** dump do plano `assign` (teste `#[ignore]` que imprime a coluna da
fronteira por linha, no fixture da caixa a precisão 400) — a serra está no `assign`? Então
é métrica/pedágio. Só na geometria? Então é traçado/RDP. Render-and-look antes de fix.

### 3.2 — A LENTE pelo vão

O azul entra pelo vão do divisor num bojo redondo (foto 1, centro). É o comportamento
geodésico HONESTO (vão largo = passagem), e o quanto ela entra já foi MEDIDO na varredura
do `SQUEEZE` (tabela na const): 4096 → lente até x=0,648 · 16384 → 0,685, **ao preço** da
parelheza do blob (1,20 → 1,29). Se o Enio quiser a lente mais colada:
- subir `SQUEEZE` é o knob barato (re-meça a tabela inteira, não um ponto);
- forma do pedágio (`/(1+d)` vs `/(1+d²)`) é 2ª alavanca — **meça as duas pontas**;
- um selo LOCAL no vão (mini-corte só na banda do vão) é a 3ª — mas ⚠️ **não re-litigue o
  min-cut GLOBAL** (§4).
- e o **Trap** já sela de vez (é o knob do produto para "esse vão não passa").

### 3.3 — A margem EXTERNA pintada (Trap 0)

Na foto 1 a moldura externa saiu toda vermelha: o contorno à mão tem furos, o lado de fora
é o MESMO componente do interior, e a cor escorre. Topologicamente honesto; o Trap fecha.
**Se incomodar, a alavanca é decisão de PRODUTO** (default do Trap > 0? regra "o mar de
fora só ganha cor com rabisco próprio" não é implementável a Trap 0 — dentro e fora são
UMA área). Pergunte ao Enio antes de mexer em default.

---

## 4. O que JÁ FOI MEDIDO E REPROVADO (não reconstrua)

| Solver / ideia | Por que morreu | Número |
|---|---|---|
| Corte de pixels global (LazyBrush cru) | custo | 3,3 s limpo; **157 s** contraditório (§7.1) |
| Guloso um-contra-todos | espreme as cores do MEIO | `[856,128,128,856]` |
| α-expansion / min-cut de Potts | minimizar Potts = minimizar fronteira ⇒ **encolhe** semente fina | `[2131,128,2991,909]` |
| Voronoi de CÉLULAS pesado por `V_pq` | célula cavalga a linha + métrica na direção errada | fronteira a 0,575 vs linha em 0,7 |
| `SQUEEZE = 0` (sem pedágio) | película na face alheia + espalhamento fora da banda do vão | max_x 0,679; espalha até 0,576 |
| `SQUEEZE = 16384` | passa do joelho: 6 px de lente por +0,09 de desvio no blob | tabela na const |

O `flow.rs` fica como referência de teste. A identidade "corte de regiões pesa o corte de
pixels" morreu junto com o grafo — não a ressuscite.

---

## 5. O plano de implementação (depois do "melhorar")

Do **Estado** do `docs/Flip/09_colorize.md` (ordem sugerida; a C3 tem seção própria):

1. **Baratear a partição** — 1,4 s dos 1,7 s a 4096² são EDT + BFS. A alavanca nomeada é a
   **exceção `rayon`** — ⚠️ **decisão do Enio, peça antes** (HR de dependência).
2. **C3 — onion fill** (`09 §5`): o rabisco atravessa as poses e pinta o RANGE de quadros.
   ⚠️ O "fill multiframe" JÁ EXISTE (`flip_fill.rs:491-519`) — a C3 é a SEMENTE rica
   (rabisco em mundo alimentando o solve de cada quadro), não o wiring de range. A política
   de silêncio do multiframe é herdada até um smoke dizer o contrário.
3. **Apply live** (preview da colorização antes do Apply) — sem plano detalhado ainda;
   escopar antes.

A UI de cada fatia entra COM a fatia (`09 §6` tem os sítios concretos; o card do balde é o
modelo).

---

## 6. Smokes e comandos (sempre com o `cd` — a janela abre na raiz)

```bash
# O smoke do Colorize (cena pronta: caixa; desenhe/rabisque e Apply):
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP && PH2D_FLIP_COLORIZE_SMOKE=1 cargo run --release -p ph2d-host-desktop

# Suíte + réguas do motor:
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP && cargo test -p ph2d-flip-colorize --release
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP && cargo test -p ph2d-flip-colorize --release measure_the_product_colorize_cost -- --ignored --nocapture

# Log do pipeline (grade, sementes, pixels por rótulo, componentes contestados):
PH2D_COLORIZE_LOG=1
```

---

## 7. Protocolo (inegociável)

- **Commits locais com `--no-verify`**, por paths. **NUNCA push.**
- **A linha NÃO integra nem faz ship sozinha** — fecha, escreve handoff de integração
  (DIRETRIZ §1.5.9) e PARA, quando o Enio mandar fechar.
- Gate batched (nextest + clippy + auditoria 2 lentes) **no fechamento**, não por task.
- Releia `docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md` a cada passo — verde de
  compilação vale ZERO no audit; todo fix nasce de um gate vermelho no fixture que CONTÉM
  o fenômeno (esta rodada pagou essa lição 2× — §2 acima).
