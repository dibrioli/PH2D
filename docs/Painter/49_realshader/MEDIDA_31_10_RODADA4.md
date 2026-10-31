# Medida 31/10, rodada 4 — a secagem pelo MEMO do assentar (ao bit), base `042137034f`

Cena: o wash de `nucleos_wash` (4096×2560, água 100 %, pigmento 85 %, `Lei::Cobre`), a `Engine` montada como no produto.
Perfil `smoke` com `PH2D_RELEASE="mede velocidade"`, pela fatia da linha (`scripts/ph2d-run.sh`: CPU ≤ 1600 % de 32).
Um `rayon::ThreadPool` de **16 fios** com `pool.install`.

## O corte

`crates/ph2d-wet-paint/src/assenta/memo.rs`: o `assenta` é uma função PURA, e as suas entradas são quantizadas: as
quatro coberturas saem da tabela de `alpha_of_mass` (índice INTEIRO da massa) e as cores `sc`/`uc` são `f32` guardadas.
Num wash quase todas as células pedem uma conta que outra célula do mesmo fio já fez. A secagem (`dry_cell`) chama agora
`assenta_memo`: uma tabela de endereço directo por fio (`thread_local`), chave = os bits de TODAS as entradas (as seis
da lei da célula, por um índice numa lista de leis; as seis cores em `f32` — só se o `f64` for `f32` exacto, senão
contorna; as quatro coberturas em `f64`). Bits iguais na chave ⇒ o mesmo resultado: **ao bit por construção**.

- Entrada de 64 B (uma linha de cache) + o resultado à parte; tabela de `2^15` entradas = **2,9 MB por fio** que seca
  (alocada na 1.ª secagem Cobre do fio). `MEMO_BITS` escolhe o tamanho em execução.
- `MEMO_ASSENTA` (`#[doc(hidden)]`, como o `flow::COMMIT_ROTA`): `1` = memo (o produto) · `0` = sem · `2` = conta
  acertos · `3` = régua do chão (devolve `sc`, resultado errado, só tempo).
- Portões: `assenta_bit_o_memo_da_os_bits_da_porta` (novo: a grelha das portas pela tabela, fria e quente, cores
  `f32` e `f64`, bits iguais à porta) · `assenta_bit_*` · `flow_bit*` · `a_borda_graduada*` (passa pelo passe de
  secagem inteiro, com o memo) · `drying*` · `secagem*` · `sim_resumable*` — verdes.

## Régua

Sonda temporária `mede_rodada4_tmp.rs` (apagada, não comitada, como a da rodada 3): monta o wash, corre a água até aos
quadros 6 · 12 · 42 · 78; em cada um tira o instantâneo da grelha e corre `drying_pass_jacobi` sobre ele restaurado,
**6 rondas × 6 variantes intercaladas com a ordem rodada**, a tabela esquecida antes de cada corrida (fria, como no
produto, onde cada passe começa com a tabela do passe anterior). Resumo = o MÍNIMO. Hash FNV da grelha (film, susp,
sett, susp_rgb, sett_rgb) depois de cada variante. Depois, o passo inteiro do quadro 78 em diante: 24 blocos de 6
passos (um ciclo de secagem, `dry_every = 6`), alternando sem/com.

loadavg 1 min: 3,0 → 5,8 durante a corrida (outras janelas). As razões são a régua; o absoluto é sob esta carga.

### Secagem (ms por passe de secagem, 16 fios, mínimo de 6)

| quadro | sem memo (HEAD) | memo 2^13 | 2^14 | **2^15 (produto)** | 2^16 | chão: sem assentar nem levantar* | acertos (2^15) | hash |
|---|---|---|---|---|---|---|---|---|
| 6 | 229,3 | 95,9 | 90,4 | **86,8 (2,64×)** | 85,4 | 33,1 | 97,3 % | igual |
| 12 | 237,7 | 99,2 | 92,6 | **89,3 (2,66×)** | 87,3 | 33,4 | 96,9 % | igual |
| 42 | 241,8 | 104,8 | 97,8 | **93,1 (2,60×)** | 91,1 | 33,4 | 96,3 % | igual |
| 78 | 258,3 | 115,0 | 105,3 | **99,8 (2,59×)** | 97,3 | 33,4 | 95,5 % | igual |

\* O chão com o `levanta` também cortado (2.ª corrida); só sem o `assenta` ele é ~56 ms: o `levanta` custa ~23 ms e o
memo dele foi MEDIDO e não ganha (2.ª corrida, 2^15: só o assentar 86,0 · 88,2 · 92,3 · 99,5 contra assentar+levantar
89,9 · 92,7 · 94,8 · 99,1) ⇒ ⛔ fora.

### Passo inteiro (ms/passo, 16 fios, blocos de 6 do quadro 78 em diante, mínimo de 12 blocos)

| | sem memo | com memo |
|---|---|---|
| corrida 1 (só o assentar, o produto) | 91,0 | **63,5 (−30 %)** |
| corrida 2 (assentar + levantar) | 91,8 | 62,7 |

A secagem por passo médio cai de ~40 para ~15 ms. O resto do passo (advecção 23 ms, fluxo 8, momentum, projecção,
suavizar — rodada 3) fica igual.

## Leituras e o que falta para ÷ 3

- **÷ 2,6 ao bit**, não ÷ 3. O que sobra da secagem com o memo (~90 ms): o chão do passe (~33 ms: `edge_rows`,
  evaporação, `alpha_of_mass`, a escrita da célula), o `levanta` (~23 ms) e o memo em si (~30 ms: 4 % de falhas a
  ~0,5 µs cada, mais a consulta — falha de cache L2/L3 em tabela aleatória; a entrada de 192 B → 64 B ganhou ~10 %).
- A tabela que cresce ganha cada vez menos (2^15 → 2^16: −2 %, o dobro da memória) ⇒ 2^15.
- A vizinha da esquerda quase nunca repete a conta (acerto na «última entrada» 0,1 %): a repetição é longe no fio, não
  ao lado ⇒ um cache de 1 entrada não serve.
- Próximos cortes (por ordem de promessa): o `levanta` pelo atalho das coberturas iguais sem o `na_caixa`/x0 (23 ms,
  ao bit a medir) · o chão do passe (33 ms: fundir `edge_rows` no laço da célula) · a advecção (rodada 3: 23 ms).
- Não medidos nesta rodada (o memo já tirou o Newton de ~96 % das células): cadência (b), chute inicial (c), SIMD/f32
  (d) — ⛔ TOL/FIXA/UMA continuam recusados ([SECAGEM_diagnostico.md](SECAGEM_diagnostico.md) §rodada 3).
