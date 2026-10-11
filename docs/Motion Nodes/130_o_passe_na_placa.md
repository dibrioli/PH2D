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

### §3.1 — Dois caminhos, escolhidos por MEDIÇÃO (11/10, depois do smoke)

A placa paga **~0,2 ms fixos por varredura** no caminho de vários despachos (~11 despachos e as
barreiras entre eles), seja qual for `n` — e o regime das muitas varreduras é a CADEIA, que é
pequena (`~n²` varreduras, doc 115 §18). Medido antes da cura: 16 peças a 1024 varreduras, CPU
`1,9 ms` × placa `116 ms`. ⇒ **até `1 024` peças o passe inteiro corre num SÓ GRUPO**
(`passe_na_placa_uno.wgsl`): todas as varreduras num despacho, as fases por barreira, e a paragem
SAI do laço. Os dois caminhos partilham as funções (`passe_na_placa_varredura.wgsl`) e o gate de
paridade corre os dois.

| release, load ~5, mínimo de 3 | CPU | vários despachos | um grupo |
|---|---|---|---|
| cadeia 16 peças · tecto 1024 | `1,94 ms` | `116–190 ms` | **`11,3 ms`** |
| cadeia 16 · tecto 4096 (1725 correm) | `3,43` | `538–667` | **`19,3`** |
| cadeia 32 · tecto 4096 | `15,1` | `483–572` | **`47,5`** |
| cadeia 256 · tecto 1024 | `14,3` | `142–150` | **`23,7`** |
| nuvem 1 000 · 64 varreduras | — | `14,4` | `14,4` (empate) |
| nuvem 2 048 · 64 varreduras | — | **`15,7`** | `29,1` |
| nuvem 32 761 · 8 varreduras (produto: despachos) | `20,0` | **`8,5`** | `75,8` |

⚠️ A cadeia pequena com muitas varreduras ainda custa `1,7`–`6×` a CPU na placa: o piso é a
latência de UM multiprocessador por varredura (~10 µs, ~20 barreiras, metade delas no `scan`).

⛔ **Recusa MEDIDA — «todos os pares» no grupo, sem grelha (até 256 peças).** Construído e
medido no mesmo processo, intercalado: perde SEMPRE para a grelha (16 peças a 1024: `15,7` ×
`11,3 ms`; 256 a 4096: `249` × `75 ms`). Saiu.

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
- O `Relatorio` sob a rota da placa: a placa conta as varreduras e os candidatos da 1.ª varredura
  no `ctrl`, uma cópia sai SEM espera (mapeada no quadro seguinte) e a ponte publica-a na bomba
  (`publica_relatorio`), que é o que o perfilador do quadro lê. Os vizinhos são os da grelha da
  PLACA (célula `2·alcance_max`), não os da CPU.

## §6 — Gates

- `ph2d-gpu-cook/tests/it/passe_na_placa.rs`: fixturas unitárias (discos, caixas alinhadas e
  rodadas, desvio, coincidentes, `inv_mass = 0`, `falloff`, `rest/body`) a `1, 2, 4, 8` varreduras.
- Cenas `=114` e `=121`–`=123` com o `Collide` ligado: corrente da CPU × instâncias da placa.
- A rota aceita (texto da rota) e a cerca nova (placa sem coluna ⇒ CPU).
