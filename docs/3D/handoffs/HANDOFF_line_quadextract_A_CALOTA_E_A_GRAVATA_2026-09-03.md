# HANDOFF — `line/quadextract`: A CALOTA DA PONTA, e a GRAVATA que deitava fora a candidata boa (2026-09-03)

> **Estado:** a linha fecha com a **amputação da ponta CURADA na realização do próprio dono**.
> ⛔ Nada foi integrado nem pushado (`CLAUDE.md` §0.7).
> Worktree: `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-quadextract` · ramo `line/quadextract`.
> O mecanismo completo, com as tabelas, está no [plano §105](../quad-remesh/PLANO_a_graduacao_da_ponta.md).
> O que veio antes: [a régua que concorda com o olho](HANDOFF_line_quadextract_A_REGUA_QUE_CONCORDA_COM_O_OLHO_2026-09-02.md).

## §1 — O que mudou, numa frase

A fase zero passa a dar **uma calota resolvida a cada espinho afiado** (o passo da grade, a `8 h`
do bico) e o acabamento passa a **desfazer as gravatas**. Na escultura do dono, no ponto do
slider da foto dele, a ponta maior deixa de ser cortada: **`0` de `5` pontas amputadas** contra
`1` de `5`, e a grade no bico vai de **`3,51` para `0,79`** (barra `1,0`).

## §2 — Como reproduzir

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-quadextract
cargo test -p ph2d-quadfill --test pontas_do_dono -- --nocapture   # 5 gates, ~1,5 s
cargo test -p ph2d-remesh-iso --test calota                        # 3 gates da porta
CARGO_INCREMENTAL=0 cargo test -p ph2d-host-desktop --release --bins --no-run
# ⚠️ o binário é o que ESTA linha imprime; NUNCA `ls -t` (apanha o executável do PROGRAMA)
env PH2D_PIECE=/home/enio/Downloads/_base_sculpt.obj PH2D_RECENTER=1 PH2D_DETAIL=1.0 PH2D_ADAPT=1.0 \
    ./target/release/deps/ph2d_host_desktop-<hash> the_artists_piece_through_the_button \
    --ignored --nocapture --test-threads=1
```

A linha a ler é `AMPUTADAS: 0 de 5 … | GRADE NA PONTA: pior 0.79 …` no fim, e a nova
`F1 CALOTA: pior … passos do alvo` logo depois da fase zero.
`PH2D_TIP_CAP=0` volta ao que shipava antes (para bissecar); `PH2D_TIP_CAP_R=<x>` muda o alcance.

## §3 — As DUAS metades, e por que nenhuma bastava sozinha

1. **A calota** ([`ph2d_remesh_iso::Cap`], `remesh_isotropic_graded_capped`) — a fase zero
   entregava o bico a `2,22 ×` o passo da grade; o pólo `+1` que fecha um bico precisa de `≥ 2`
   células resolvidas. Com a calota a `1,0 h` ela desce a `1,15` e a cadeia **produz**, pela
   primeira vez nesta peça, uma candidata verde nas duas réguas de ponta.
2. **O desembaraçador** ([`ph2d_quadfill::untangle_bowties`]) — essa candidata perdia por **UMA
   gravata** (a `5,7` células do bico mais próximo, um quad dobrado solto no flanco): a chave das
   gravatas é a 3.ª do `worse` e a da amputação é a 4.ª.

⚠️ **Medido: o desembaraçador sozinho não muda o caminho de omissão** (`20 658` quads, mesmo
veredito ao bit — a vencedora de hoje não tem gravata). *A calota PRODUZ a candidata; o
desembaraçador deixa-a GANHAR.*

## §4 — A tabela que autorizou ligar (`Detail 1` · `Curv 1`, `PH2D_RECENTER=1` sobre o ficheiro cru)

| peça / realização | hoje | com a calota `1,0 h` |
|---|---|---|
| ⭐⭐ `_base_sculpt` — **a realização que o dono vê** | `1/5` · gap `3,00` · grade **`3,51`** | ⭐ **`0/5`** · `0,47` · **`0,79`** |
| `_base_sculpt` a `s = 0,7` | `0/5` · gap `0,45` · grade `1,66` (`3` acima) | `0/5` · **`0,38`** · **`1,07`** (**`2`**) |
| `sculpt_antes` (a agulha) | `1/4` · gap `3,00` · grade `1,15` (`1` acima) | `1/4` · **`2,57`** · **`0,98`** (**`0`**) |

**Três de três melhoram ou empatam em todas as colunas; nenhuma piora.** Topologia `χ = 2`,
zero bordo, zero não-manifold nas três. Preço: a malha de trabalho vai de `3 642` para `6 146`
faces (`+69 %`) e a saída de `20 658` para `21 928` quads (`+6 %`).

## §5 — ⛔ Recusas MEDIDAS desta wave

| o que foi tentado | o que deu | por que não shipa |
|---|---|---|
| calota a `0,75 h` | fase zero verde no bico (`0,84`) | a jusante não digere: candidatas com `7`–`48` arestas de bordo, `1` não-manifold na saída |
| calota a `0,5 h` | fase zero a `0,55` (F1 `4,4×` maior) | o mesmo, pior — *o que a jusante não digere é a INFLAÇÃO* |
| reordenar as chaves do `worse` | — | a ordem foi medida em 30/08 sobre um report do dono (`125` gravatas, *«destruiu completamente a malha»*); a lei da casa é **produzir a candidata que tem as duas coisas** |
| cerca do desembaraçador = a do socorro (`0,5` aresta) | cura impossível por construção | *um quad só se auto-cruza quando um vértice passa PARA LÁ do vizinho* ⇒ a viagem de volta é da ordem de **uma** aresta (`UNTANGLE_TRAVEL = 2`) |

## §6 — ⚠️ Quatro coisas que uma leitura rápida do diff entende ao contrário

1. **A `F1 CALOTA` não é a `F1: PONTAS`.** A segunda mede a malha contra a **mediana dela
   própria** (*«a ponta é mais grossa que o corpo?»*); a nova mede em **passos do alvo**
   (*«cabem duas células no bico?»*), que é a pergunta de que o pólo depende. Nenhuma sonda
   tinha a segunda.
2. **O log da candidata mudou de GRANDEZA, não só de colunas.** Ele imprimia `bordo`
   (`boundary_edges`) e o selector lê `open_edges` (bordo **+** não-manifold). Com `ilhas` e
   `gravatas` ao lado, as **três** primeiras chaves ficam visíveis — antes o log explicava todas
   as escolhas menos a que interessava, e foram precisas três corridas para descobrir o que uma
   coluna teria dito à primeira.
3. **A sonda da porta deixou de ter uma CÓPIA da fase zero.** Ela espelhava a escolha do produto
   num bloco próprio (com um comentário a dizer que a espelhava) e teria envelhecido no dia
   exacto em que a fase zero ganhou a calota. Hoje chama `target::phase_zero`.
4. **A fixtura nova é medida com a entrada RECENTRADA pela porta do importador.** Sem isso as
   duas malhas vivem em espaços diferentes e a régua lê `5 de 5` amputadas com o gap saturado —
   *uma medição entre dois referenciais mede a translação.*

## §7 — O que fica ABERTO, com endereço

- ⏳ **A agulha da `sculpt_antes` continua cortada** (`1/4`, gap `2,57` contra `3,00`): melhora e
  não fecha. O plano §102 já mediu que ali a cadeia *«realiza um pólo denso ou com um furo no
  bico ou comendo o bico»* — a calota move a fronteira, não a apaga.
- ⏳ **A segunda realização ainda tem `2` de `5` pontas com grade acima de `1,0`** (`1,07` no
  pior). A lotaria do §104 encolheu, não desapareceu: o critério de aceitação daquele parágrafo
  (mesmo veredito nas cinco realizações) **não** está cumprido — foram medidas duas.
- ⏳ **`PH2D_TIP_ALIGN` continua um instrumento** (plano §102): o campo fecha as pontas a `k = 5`
  e a extracção deixa um laço de `14` arestas. Com a calota a existir, aquela medição merece ser
  refeita — *o substrato que a tornava impossível mudou*.
- ⏳ **O custo não foi optimizado:** `+69 %` de faces na malha de trabalho. Ninguém mediu se a
  calota pode ser mais estreita que `8 h` (o número veio do `PH2D_TIP_ALIGN`, não de uma
  varredura).

## §8 — O diff

| ficheiro | o quê |
|---|---|
| `crates/ph2d-remesh-iso/src/sizing.rs` | `Cap` + a calota no campo por vértice (antes da renormalização) e reclamada depois |
| `crates/ph2d-remesh-iso/src/lib.rs` | `remesh_isotropic_graded_capped` |
| `crates/ph2d-remesh-iso/tests/calota.rs` | 3 gates da porta (identidade ao bit · pedido inválido · afina sem estourar o orçamento) |
| `crates/ph2d-quadfill/src/untangle.rs` (+ `_tests`) | `untangle_bowties` + 3 gates |
| `crates/ph2d-quadfill/src/finish_extract.rs` | as **três** saídas do acabamento desfazem gravatas; `FinishReport::untangled` |
| `crates/ph2d-quadfill/tests/pontas_do_dono.rs` + `fixtures/pontas/nossa_com_calota.obj.gz` | o gate de aceitação: a nossa saída passa o que a aprovada passa |
| `shells/desktop/src/sculpt3d_retopo_target.rs` (+ `_tests`) | `TIP_CAP_STEP`/`TIP_CAP_RADIUS`/`tip_caps` + gate |
| `shells/desktop/src/sculpt3d_retopo_rulers.rs` | as três chaves da frente no log da candidata |
| `shells/desktop/src/sculpt3d_photo_button.rs` | a sonda corre a fase zero **do produto** + a linha `F1 CALOTA` |
| `shells/desktop/src/sculpt3d_history_retopo_extract.rs` | `mod target` visível ao módulo |
| `docs/3D/quad-remesh/PLANO_a_graduacao_da_ponta.md` | §105 |

---

## §9 — ⭐⭐⭐ A SEGUNDA METADE DO DIA: a fenda, e as duas espécies que são uma família

O dono aprovou a wave acima (*«melhor resultado até agora»*, backup pedido) e fotografou **uma**
ponta com uma fenda no flanco (`sculpt003.obj`). Mecanismo completo: [plano §106](../quad-remesh/PLANO_a_graduacao_da_ponta.md).

- ⛔ **A malha estava PERFEITA na topologia** (`χ = 2`, `0` bordo, `0` não-manifold, `0` valência-2)
  — o censo de arestas não tinha nada a dizer. A fenda eram **cinco dobras de até 180° no mesmo
  ponto** (faces a apontar contra a vizinhança).
- ⭐ **A régua que separa é o TAMANHO DO GRUPO**: a retopologia que ele **aprovou** tem `3` dobras
  com maior grupo `1` (vincos reais); a dele tinha `5` num grupo só. ⛔ Nem a contagem de faces
  minúsculas nem o salto de tamanho separam (a aprovada é **pior** nos dois).
- ⛔⛔ **As duas leis de dobra já viviam em `ph2d-quadfill`** e eram consumidas pelo motor legado
  e por sondas — *o selector do botão nunca as consultou* (a família do `CLAUDE.md` §5.0, a mesma
  que a gravata pagou de manhã).
- ⛔⛔⛔ **E a relaxação NÃO cura uma dobra: ela troca a espécie do defeito.** Medido com o
  instrumento novo (`cargo run -p ph2d-quadfill --example dobras -- --curar <sup.obj> <cand.obj>`):
  naquela candidata a gravata sai e a **mesma face** fica a apontar contra a vizinhança. ⇒ o
  «sucesso» do §105 era a conversão, e a dobra é o que ele fotografou.
- ⭐⭐⭐ **A cura é o CRITÉRIO:** a chave passa a contar **as duas espécies** (gravatas + dobras em
  grupo `≥ 2`) com uma **folga de `20`**, calibrada no vazio entre os dois lados que **ele** julgou
  — `125` faces do avesso = *«destruiu completamente a malha»* (30/08) · `6`–`8` = *«melhor
  resultado até agora»* (03/09).
- Resultado na realização dele: **`0/5` amputadas, pior gap `0,18`, grade `0,74`** — melhor que a
  build que ele aprovou (`0,47` / `0,79`). Caminho de omissão **byte-idêntico**.

### ⏳ O que fica ABERTO desta metade

- **A fenda fica.** Ela é um nó da grade e não se repara no acabamento: a cura é a montante
  (extracção/mapa). O que esta wave entrega é **não a esconder** (a chave conta-a) e **não deixar
  que ela custe as pontas** (a folga medida).
- **`INSIDE_OUT_SLACK = 20` tem dois pontos de calibração, não uma varredura.** A próxima peça que
  cair entre `8` e `125` faces do avesso é a que a testa.
