# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-10 — F7d: O QUICKSHAPE COM SETAS

> Leitor: o agente integrador (DIRETRIZ §1.5.3; `/pd-integracao`), só por ordem do Enio, e a próxima
> LLM desta linha. A linha FECHOU e PARA: nada integrado, nada enviado.
> Jornada anterior: [F7c, 10/10](HANDOFF_INTEGRACAO_line_UIUX_2026-10-10_F7c_QUICKSHAPE_ARCOS.md)
> — citada, não reescrita. F0–F7c: os handoffs delas.

- **Branch** `line/UIUX` · **base (merge-base = `main`)** `2a925859d` (o `main` não andou — sem rebase)
  · F0–F7c continuam por integrar e vão JUNTAS com esta jornada.
- Commits desta jornada: `f0a60a7ef..HEAD`.
- **O pedido (dono, 10/10, depois do smoke OK dos arcos):** escolheu, de quatro opções, *«SETAS»* —
  desenhar uma linha ou um arco com um bico no fim e SEGURAR ⇒ uma seta perfeita.

## §0 — Para o `CLAUDE.md` §5.1 — JÁ APLICADO

UI/UX: «Último:» → este ficheiro (só o link; a frase e as cenas não mudam).

## §1 — O que mudou

| peça | o que é | onde mora |
|---|---|---|
| ⭐ **os oráculos** | 3.ª leva do `gera_entradas.py` (15 setas num traço só, sem `rng`); o Xournal++ NÃO tem seta (7 linha, 8 cru → 13 excepções do dono, `own()` 27 → 40); **2.º oráculo: o Excalidraw `next`** (MIT, a ferramenta «draw shape»), corrido em Node pela API pública — `recognizeShape` + `convertToShape` — sobre as 80 entradas | `oraculo_quickshape/corre_excalidraw.mjs` (novo), `fixtures_excalidraw/` (80), `fixtures/seta_*` (15) |
| ⭐ **a lei** | `QuickShape::Arrow(ArrowShape { shaft, heads })`: a ponta no FIM e no INÍCIO (o 1.º mínimo do resíduo ao V; o eixo do cabo pela parábola do fim dele; as farpas saem DA ponta), o cabo pela lei que já existe (linha, polilinha, arco, biarco) e com 92,5 % do comprimento do traço dele (uma forma fechada mede 0: o cabo é aberto) | `ph2d-vec-edit/src/quickshape_arrow.rs` (novo); o ramo em `recognise_with`, ANTES do fecho |
| **a ponta** | o `Marker` do traço: a ARMADA no lápis, senão `Open` (a «arrow» de nascença do Excalidraw: duas riscas em V); o tamanho põe o alcance da ponta igual à farpa média; o caminho recua o `Marker::inset` (o bico cai na ponta desenhada); fora da seta, as do estilo | `pencil.rs` (`arrow_ends`, `rebuild`) |
| **o gesto** | o de toda forma: segurar → a seta; arrastar escala e roda a seta INTEIRA (o cabo à volta do início, as farpas com ele); soltar comete com nós e a ponta; `Esc` devolve o traço livre SEM ponta; o `Shift` regulariza o cabo e as pontas FICAM | `quickshape_hold.rs` (sem mudança), `quickshape.rs` (`pivot`/`transformed`/`to_path`/`heads`) |
| ⭐ **a ponta não passa ao traço seguinte** | ACHADO pela foto (o gelado nascia com seta) e, depois da 1.ª cura, pelo smoke do dono: *«todo traço está herdando uma ponta pequena. Não deveria»* — a 1.ª cura (uma excepção no lápis que comparava o tamanho da ponta) caía quando o painel arredondava o tamanho ou a seta era desselecionada. A cura é de lei: a tool separa as pontas MOSTRADAS (as da seleção, adotadas) das ARMADAS (as do próximo caminho, que só o painel escreve, campo a campo); a ponte dá aos caminhos novos (lápis, caneta, formas) as armadas | `ph2d-tool-vector/src/tool.rs` (`armed`, `armed_markers`), `tool_armed_tests.rs` (novo); a ponte `ph2d-app-vec/src/vector_bridge.rs` |
| **cena 12** | um 7.º traço SEGURO: uma seta de cabo curvo (arco de 140° + V aberto) por cima do rectângulo (8 traços, `HELD = 7`); foto [`f7d_01`](../vetor_nova_ui/fotos/f7d_01_cena12_uma_seta_segura_vira_seta_perfeita.png) | `ph2d-app-vec/src/vector_mode_quick.rs` |
| organização | `gap`/`bow` descem para `quickshape_geom.rs` (o `quickshape.rs` ia passar das 700); o gesto no lápis em `quickshape_arrow_pencil_tests.rs` | `ph2d-vec-edit/src/` |

## §2 — Medido e decidido (técnico, delegado)

Tabelas inteiras em [`00_resultados.md`](../vetor_nova_ui/oraculo_quickshape/00_resultados.md) §3h
(`cargo test --release -p ph2d-vec-edit --lib measure_arrow -- --ignored --nocapture --test-threads=1`).

- **Setas de mão** (3 cabos × 3 pontas × 1–2 pontas × 20–60° × farpas de 8–40 % do cabo, metade
  assimétricas ±10°, o repouso no fim): **400 px 100 %**, 160 px 92–97 % (com o estabilizador), 96 px
  70–74 % (100 % sem), 48 px 38–46 %, 24 px 9–15 %. As pequenas caem pelo piso das farpas (5,75 px).
- **O cabo nunca piora:** +1,03 px de Hausdorff no pior de 246 pares contra o mesmo cabo sozinho; a ponta
  a ≤ 3,04 px da desenhada. **Custo:** 0,85 ms (seta de 400 px, duas pontas), 1,8 ms (rabisco).
- **A × B nas formas antigas:** zero em 8 580 traços, fora das duas famílias novas (a linha que volta
  atrás, e um T a ±4 px).
- **As faixas** (8 constantes, `ArrowLaw`, cada uma no meio): `barb_min_deg` 4 (um ponto: 3 deixa a
  linha que volta atrás, 5 mata a farpa de 20° desenhada a 10°), `barb_max_deg` 82,5 (80–85),
  `head_min` 0,0475 (0,04–0,05: a farpa de 4 % que os dois oráculos chamam linha), `head_max` 1,2
  (0,4–2), `head_rms` 0,095 (0,09–0,10), `barb_min_px` 5,75 (5,5–6: o repouso), `shaft_fill` 0,925
  (0,9–0,95), `tip_return` 0,2 (0,1–0,3).
- **`regular`:** o cabo regulariza como sem ponta; as pontas FICAM (são desenho, não forma).
- **⛔ Recusas MEDIDAS:** a candidata de menor resíduo (corta a ponta em cima de uma farpa); a corda como
  eixo (15–24° num cabo curvo); a seta só em traço aberto (duas pontas grandes liam-se fechadas); a
  simetria das farpas como guarda; o tecto de quinas do cabo; a razão entre farpas (saiu: nenhuma
  família a distinguia); o repouso aplicado DEPOIS do estabilizador (instrumento: 75 → 3 traços).

## §3 — Foundational tocado e contratos

- **`ph2d-vec-edit`** (público): a variante `QuickShape::Arrow(ArrowShape)`, o `pub struct ArrowShape`
  (re-exportado em `quickshape`), `QuickShape::heads`. Nenhum `match` exaustivo
  sobre `QuickShape` fora da crate (varrido: só `matches!` no gate da cena 12). Privados: o módulo
  `quickshape_arrow`, `ArrowLaw` (`Law.arrow: Option<ArrowLaw>`), `pencil::arrow_ends`;
  `quickshape_geom::{gap, bow}` (movidos); `Fixture.name`/`ours` passam a `pub(super)`.
- **`ph2d-tool-vector`**: `VectorTool::armed_markers` (pub) e o campo privado `armed` (`ArmedHead`): selecionar
  um caminho deixa de ARMAR as pontas dele para o próximo — mudança de comportamento do «último usado»
  SÓ nas pontas (cor, largura, cap… continuam a herdar), pedida pelo dono.
- **`ph2d-app-vec`**: o `PenStyle` dos caminhos novos leva as pontas armadas + gate de fonte; a cena 12 com 8 traços.
- **Contratos congelados (§6): nenhum.** Shell: **intocada** (`+0`).

## §4 — Superfície de colisão e contadores

Nada que some entre linhas: sem ids, chaves, cenas novas (a 12 ganha um traço), schema. `own()` 27 → 40
conta-se no próprio ficheiro (gate `every_own_law_exception_names_a_fixture_that_exists`). As fixtures do
oráculo passam a 80 (+15) e as do Excalidraw entram (80); `gera_entradas.py` gera 80.

## §5 — Gate final

Uma corrida do `verificador` (`BASE=2a925859d`, HEAD `3e5a02ac6`), tudo verde à primeira: fmt ✓ ·
warnings-deny ✓ · clippy do ship ✓ · machete ✓ · standalone-optional ✓ (10) · workflow-packages ✓ (32 /
372) · censos 114/114 e 12/12 ✓ · `doc-index`/`adr-index` ✓ (185 ADRs) · `nextest-impacted` **17 056 /
17 056** (9 791 pulados; `load` 18,8 no fim) · catracas ✓ (`the_shell_only_shrinks`, LOC,
`no_orphan_source_file`) · `architecture_claude_md_cabe_no_orcamento` 3/3. Depois: o handoff, o link do
`CLAUDE.md` e o `doc-index` (só docs).

**Depois da correção do smoke** (as pontas armadas, HEAD `347f00caf`), 2.ª corrida do `verificador`: fmt ✓
· warnings-deny ✓ · clippy do ship ✓ · machete ✓ · standalone-optional ✓ · workflow-packages ✓ · censos
114/114 e 12/12 ✓ · `nextest-impacted` **17 057 / 17 057** (9 791 pulados; um SLOW de 60,9 s no
`ph2d-skeleton-live`, `load` 12,7–17,7, passou).

### Mutação

Agente `mutacao` (verde visto antes, âncoras 1×, árvore limpa no fim): **30 de 37** sangram à primeira
(a ponta do início, o eixo pela parábola, o piso, a farpa máxima, o ângulo máximo, a volta à ponta e
de que farpa ela conta, o resíduo, a corda do V, o comprimento do arco, o fim do cabo, as farpas no
arrasto, o ramo antes do fecho, o pivô, `heads`, e no lápis as pontas, o tamanho, o recuo, a `Open`, só
onde a mão desenhou, o alcance, as quatro condições do `next_style` — que depois saiu, §1 —, a costura da
ponte e a cena). Das
7 sobreviventes, a busca base × mutante sobre 6 775 traços (`measure_arrow_witness`) achou testemunha
para **5** — fechadas com portões vistos VERMELHOS sob a mutação: o 1.º mínimo e o `break` da candidata
que deixa de passar (`the_first_minimum_keeps_these_small_arrows`), o preenchimento do cabo
(`a_t_is_not_an_arrow_even_in_a_shaky_hand`), o ângulo mínimo
(`a_line_that_goes_back_is_a_line_in_a_steady_hand`), a farpa mínima (passou a sangrar nas fixtures dos
dois oráculos quando a re-medida saiu). **2 redundantes retiradas do código** (0 de 6 775 traços
diferentes): a re-medida das razões contra o cabo real e a cláusula «o cabo é aberto» (o preenchimento
já a faz; `length_of` mede 0 numa forma fechada). Fica o portão `a_loop_with_a_head_is_not_an_arrow`
(documenta o laço de 330° com bico: círculo, nunca seta sem ponta visível).

**A correção do smoke** (as pontas armadas): 7 / 7 sangram — adotar que arma, cada uma das cinco
escritas do painel (tamanho, arredondamento, começo, fim, Both Ends) que não arma, a ponte que leva as
pontas MOSTRADAS aos caminhos novos.

## §6 — Smoke do dono

Foto [`f7d_01`](../vetor_nova_ui/fotos/f7d_01_cena12_uma_seta_segura_vira_seta_perfeita.png): a seta
por cima do rectângulo é um arco de três nós com a ponta em V aberto; o gelado solto, a seguir, SEM
ponta (a 1.ª foto tinha-a — §1).

1. `cd ~/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=12 ./target/smoke/ph2d-host-desktop`
2. A cena desenha sozinha oito traços com o Lápis; o 7.º, por cima do rectângulo, é um arco com um bico
   no fim — segurado meio segundo.
3. Tem de virar uma seta perfeita: o arco liso com a ponta em V; o gelado (o último) fica sem ponta.
4. Errado: o 7.º fica tremido (traço livre), ou sem ponta; ou o gelado aparece com seta.
5. Depois, na mesma janela, com o Lápis na mão: uma linha e, sem levantar, o bico (vai a um lado, volta
   à ponta, vai ao outro) — segure parado meio segundo ⇒ seta. Arraste sem soltar ⇒ cresce e roda
   inteira; `Esc` ⇒ volta o traço livre sem ponta; solte ⇒ fica. O bico também pode ir no começo (as
   duas pontas) e o cabo pode ser curvo.
6. (Depois do 1.º smoke do dono — *«Funciona bem! mas todo traço está herdando uma ponta pequena»*.)
   Faça uma seta, clique no vazio, e desenhe traços livres e formas: nenhum nasce com ponta. Escolher
   uma ponta no painel (End) com nada selecionado arma-a para os traços seguintes, como antes.

## §7 — O que fica ABERTO

- **A linha que volta atrás** sobre si e só então faz uma farpa lê-se seta (40–50 % a 48–160 px, 10 % a
  400 px): à vista é uma farpa só, e nenhum ângulo mínimo a separa da farpa assimétrica de 10°. Medida,
  não gateada.
- **Setas pequenas** (cabo < 96 px com farpas < 8 px) ficam livres: o piso de 5,75 px é o do repouso.
- O tamanho da ponta sai até 34 % menor que o desenhado (o estabilizador encurta a farpa).
- **Perguntas ao dono (produto):** (1) a ponta por omissão é o V ABERTO (a do Excalidraw e a que a mão
  desenha) — ou prefere o triângulo cheio («Arrow» no painel)? (A 2.ª — o traço seguinte herdar a ponta —
  o dono respondeu no smoke: não deve.)
- Do F7: «Edit Shape».

## §8 — Binário de smoke (2.ª corrida)

Depois do `rm -rf target/*/incremental` (5,9 GB debug + 1,3 GB smoke) — `bash scripts/ph2d-run.sh
cargo build -p ph2d-host-desktop --profile smoke`, refeito depois da correção do smoke: 1.ª corrida
20,4 s; a 2.ª:

```
    Finished `smoke` profile [optimized] target(s) in 0.21s
```

Varredura no fecho: zero processos meus (um `ph2d-host-desktop` vivo é da `line-Luz2D`; o `cargo test -p
ph2d-tool-painter` é da `line-PainterWatercolor`).

## §9 — `agent-loop-profile.sh`

```
  ✗ paralelismo de ferramenta              1.13/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                268   alvo: <= 800
  ✗ cargo test : cargo check                790 : 218   alvo: <= 1,0  razao 3.6x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  34%   alvo: >= 80%
  ✗ contexto relido por passo (media)         445 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               62 mil   alvo: <= 80 mil
```
(As sondas são `cargo test --release` por natureza e foram muitas rodadas — cada mudança de lei
re-mediu as faixas; as edições por script foram as de várias âncoras com `assert` de contagem.)
