# 130 — O passe de separação na PLACA (o `Collide` do cartão `Shape`/`Object`)

> Ordem do dono (11/10): *o Collide do cartão Shape corre na placa*. Fecha o aberto do doc 115 §20.7
> (*«o caminho rápido e a colisão são mutuamente exclusivos — fechar isto é um kernel, com espec
> própria»*). Linha `line/motion-value`.

## §1 — O que é, e o oráculo

O passe do fim do cozimento (`ph2d_contact::passe::separa_com_relatorio`, chamado pela porta
`o_que_o_sink_desenha`) ganha uma segunda execução: **na placa, sobre a corrente que o sink vai
desenhar, antes do lowering e do passe de formas** (`ph2d-gpu-cook`, `passe_na_placa.rs`).

⭐ **O oráculo é a CPU, e não há outro.** A lei é a da crate `ph2d-contact`, linha a linha; nada aqui
inventa aritmética. Compara-se **por passo**: o mesmo `Stream` sobe à placa e corre `1, 2, 4, 8`
varreduras dos dois lados; a CPU com as portas públicas, a placa lida de volta só no gate.

## §2 — A lei a portar (cada termo tem morada na CPU)

| termo | CPU | placa |
|---|---|---|
| colisor declarado (disco/caixa, `size`, `rot`, desvio) | `declarado` (`lib.rs`) | `pnp_declara` |
| peso `inv_mass` (ausente `1`, não-finito `0`) | `passe::pesos` | idem |
| `invI` (coluna ou `Colisor::inv_inercia`) | `inv_inercias` | idem |
| activo = colisor válido ∧ `P` finito, lido 1× | `laco::separate_grao` | idem |
| forma rodada pelo giro acumulado | `Colisor::girado` | `pnp_gira` |
| par: disco×disco, disco×caixa, caixa×caixa com TRECHO (2 pontos) | `par::manifesto` | WGSL verbatim |
| filtro do repouso (`rest`/`body`) com o colisor DECLARADO | `Ligados::segura` | idem |
| Jacobi: média pelos contactos, `quota = 1/pontos` | `varredura::corrigida` | idem |
| parceiros somados em ordem CRESCENTE de índice | grelha + `sort` | fusão de 9 listas ordenadas |
| paragem: `andou == 0` ou `andou < 1e-5 · alcance_max` | `laco` | bandeira na placa, sem leitura |
| `falloff` no fim (`p + (p′−p)·k`, `giro·k`) | `passe::separa_contando` | `pnp_escreve` |
| `rot += giro` | idem | idem (sempre escrita — `rot + 0` é `rot`) |

**As varreduras** são as do cartão Output (`sink_collide_sweeps`, `1..=4096`, omissão `8`), passadas
à placa pela porta `GpuCook::set_passe` (uma por sink), como o `set_driven`.

## §3 — O desenho da placa

Por passe: `declara` (1×: registo por peça + `alcance_max` por `atomicMax`) → por varredura
`gira+conta` → `scan` (o `Scan` do ADR-0140) → `espalha` → `ordena` (cada balde por índice) →
`corrige` (lê a fotografia, escreve o estado novo — ping-pong) → `decide` (1 fio: paragem, contador
de varreduras). No fim `escreve` (falloff, `P`, `rot`). A célula é `2 · alcance_max`, lida da placa:
dois colisores só se tocam se `|Pi − Pj| < alc_i + alc_j ≤ 2 · alcance_max`, logo 3×3 células é
SUPERCONJUNTO — a resposta não depende da grelha (a mesma promessa da `grelha` da CPU).

⚠️ **Sem leitura de volta.** A paragem é uma bandeira na placa; varreduras depois dela saem logo.
⇒ o `Relatorio` (perfilador do quadro) não é lido da placa — ver §5.

## §4 — Diferenças admitidas, e a tolerância

1. **ULPs**: `cos/sin` da placa ≠ `trig::cos_sin_cycles`; `sqrt`/FMA. A ordem de soma é a mesma.
2. ⇒ a paridade é por ε relativo ao `alcance_max`. **Medido (11/10, sete fixturas a `1, 2, 4, 8`):**
   pior `|ΔP| = 2,84e-6 · alcance_max` e `|Δrot| = 1,07e-4°`; tolerância `1e-5` e `1e-3°`. As
   varreduras que correram são IGUAIS em todas (a paragem cai do mesmo lado do limiar).

## §5 — A rota aceita, e as cercas que ficam

- `peca_declara_colisor` deixa de recusar. **Cerca nova:** se alguma peça declara mas nenhuma
  corrente de sink chegou à placa com coluna de colisor (`collider` ou `collider_box`), a placa
  devolve erro e a rota cai para a CPU (*nunca* uma cena sobreposta num lado e separada no outro).
- `RECUSA_COLISOR` (o colisor escrito PELO NOME num text param) **fica**: fora desta ordem.
- O `Relatorio` sob a rota da placa: o perfilador continua a ler o último da CPU. Nomeado no handoff.

## §6 — Gates

- `ph2d-gpu-cook/tests/it/passe_na_placa.rs`: fixturas unitárias (discos, caixas alinhadas e
  rodadas, desvio, coincidentes, `inv_mass = 0`, `falloff`, `rest/body`) a `1, 2, 4, 8` varreduras.
- Cenas `=114` e `=121`–`=123` com o `Collide` ligado: corrente da CPU × instâncias da placa.
- A rota aceita (texto da rota) e a cerca nova (placa sem coluna ⇒ CPU).
