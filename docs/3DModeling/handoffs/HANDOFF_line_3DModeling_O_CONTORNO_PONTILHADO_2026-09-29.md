# Handoff — `line/3DModeling`: o CONTORNO PONTILHADO da foto de 25/09 (2026-09-29)

> Wave depois do rebase sobre o `main` da rodada 03 (`912a9652e`). Ordem do dono: *«siga»* sobre o
> §6 do [handoff de 25/09](HANDOFF_INTEGRACAO_line_3DModeling_A_LINHA_2026-09-25.md) e *«não toque no
> main. continue na linha rebaseada»*. ⚠️ **Conte o DELTA: `PROJECT_SCHEMA` 0, registos 0,
> `FIELD_DOC_VERSION` 0, zero contrato, zero ADR, zero pacote externo.**

## §1 — O defeito, com o mecanismo

A foto do dono (cena `=28`, o nó de toro, a girar) mostra um **contorno pontilhado claro** ao longo
das silhuetas dos tubos. O §6 de 25/09 deixava **duas** candidatas: o passo da oclusão
(`ceu_passo = 2`) e a borda re-amostrada. ⇒ a sonda `diag_o_contorno_pontilhado`
([`device_probes_w9_ceu_passo.rs`](../../../crates/ph2d-app-field3d/src/device_probes_w9_ceu_passo.rs))
bissecta as duas no quadro de movimento a `960×540`, e a régua dos **pontos claros isolados**
(luminância `> 12` acima do máximo dos oito vizinhos) contou:

| quadro de GPU | pontos claros isolados |
|---|---|
| borda ligada, antes da cura | **`597`** |
| borda ligada, depois da cura | **`94`** |
| borda desligada (controlo) | `25` |

⇒ a culpada é a **borda**. O passe `pinta_bordas` re-amostra cada pixel de silhueta em quatro
sub-amostras; num pixel cujo **centro FALHA a peça** o ponto já era emprestado do primeiro vizinho
de cruz que acerta (`pixel_da_borda`), mas a **oclusão do céu, a sombra da lâmpada e o ricochete**
continuavam a ser lidos no índice do CENTRO — um pixel de FUNDO, com céu aberto e lâmpada sem
sombra. Cada degrau da silhueta de um tubo escuro pintava um ponto claro: a linha tracejada.

⭐ **A cura é uma linha nos dois motores:** a luz lê-se no índice `j` de quem a borda pede o ponto
emprestado (`shade_render.rs`, o laço das bordas; `paint_wgsl_sondas.rs`, `pinta_bordas`), e com o
centro na peça `j == i` e nada muda. ⛔ Os dois mudam JUNTOS — é a condição da paridade.

## §2 — O gate, e porque a régua da foto NÃO serve

⛔⛔ **A 1.ª redacção do gate contava os pontos isolados no quadro de CPU e a prova de mutação
derrubou-a:** com a cura revertida ela lia **`11` contra `21`, ao bit o mesmo**. No caminho de CPU
desta cena a luz de um pixel de fundo e a do vizinho que acerta quase não diferem, logo *a fixtura
não continha o fenómeno* e o gate ficava verde a afirmar nada.

⇒ [`borda_pontilhado_tests.rs`](../../../crates/ph2d-app-field3d/src/borda_pontilhado_tests.rs)
mede a LEI: **envenena os canais de luz dos pixels de FUNDO** (céu `0`, lâmpada `0`, ricochete
magenta) e exige que **nenhuma** das `2 932` bordas de centro falhado mude. Para isolar a lei, as
sub-amostras dessas bordas passam todas a «acertar» — o caminho do FUNDO lê pixels de fundo de
propósito (o chão), e sem isto ele entraria na conta. ⭐ **O CONTROLO vem primeiro:** o veneno tem
de mover o fundo (`~113 000` pixels), senão a igualdade seria vácua.

| mutação | resultado |
|---|---|
| CPU: `i: j` → `i` | **sangra** — `2 932` de `2 932` bordas movidas, pior byte `244` |
| GPU: `pinta_bordas` de volta ao `HEAD` | ver §3 |

## §3 — O lado da placa

⚠️ O gate novo é de CPU: o dispositivo calcula a oclusão dentro do shader e não se envenena por
fora. A metade do shader é apanhada pela **paridade** existente: com `pinta_bordas` devolvido ao
`HEAD`, **5 de 12** gates de paridade reprovam (os motores voltam a discordar na borda) —
`a_borda_mole_da_sombra_e_a_mesma_nos_dois_motores` · `o_chao_do_dispositivo_e_o_da_cpu` ·
`a_imagem_do_dispositivo_e_a_da_cpu` · `a_lampada_encostada_a_peca_concorda_nos_dois_motores` ·
`varias_lampadas_sombreiam_todas_e_concordam_com_a_cpu`; na árvore limpa os 12 passam (CONTROLO).
⚠️ Os cinco são `#[ignore]` de GPU ⇒ **o CI não os corre**; quem os corre é a bateria com adaptador.
A sonda `diag_o_contorno_pontilhado` fica como a medição da foto (`#[ignore]`, GPU).

## §4 — De caminho: duas sondas de preço que rebentavam

A bateria `--run-ignored all` pôs a vermelho `the_price_of_the_thread` e
`the_price_of_the_torus_knot` (`ph2d-field-render`, `tests/it/`), com `NonPositive { what: "starts" }`
e `{ what: "winds" }`. ⚠️ **Não é regressão desta linha:** as escadas pediam `starts = 192..384` e
`p = 10, 12`, acima dos tectos do MODELO que as W134b/W135 fixaram (`MAX_THREAD_STARTS = 128`,
`MAX_KNOT_WINDS = 8`, `max_knot_loops`) — o documento recusa a peça e o `expect` rebentava. As
escadas passam a filtrar-se pelas próprias constantes: *uma sonda cuja escada é escrita à mão ao lado
de um tecto é a segunda resposta à mesma pergunta*. ⚠️ **E eram TRÊS escadas, não duas:** curadas as
entradas, a rosca rebentou outra vez na escada do NÚCLEO (`what: "depth"`, o piso
`THREAD_CORE_FLOOR`) — essa linha passa a dizer-se *recusada* pelo `thread_depth_ceiling`, a mesma
função que o documento usa. Hoje `6 de 6` verdes.

## §5 — Aberto

- ⏳ **A imagem grossa a mexer** (a 1.ª metade da foto): o nó custa `55,5 ms` a `1920×1080`; o
  passo seguinte é a **reprojecção temporal** ([`14` §6](../../Render3d/14_a_ordem_de_superar.md)).
- ⏳ Os `94` pontos que sobram contra `25` sem a borda: a luz vem agora do vizinho, mas o **ponto**
  de cada sub-amostra continua a ser o do vizinho (a borda não guarda os pontos das sub-amostras) —
  aproximação declarada no `shade_render.rs`.

**Smoke para o dono:**
```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && env PH2D_FIELD_SMOKE=28 cargo run -p ph2d-host-desktop --profile smoke
```
