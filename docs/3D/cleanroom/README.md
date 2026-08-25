# `cleanroom/` — a reimplementação de código restrito, no módulo 3D

Protocolo: [`SKILL_Cleanroom_Reimplementacao.md`](../../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md).
Decisão: [`ADR-0164`](../../architecture/decisions/0164-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md)
— **clean-room dos *papers*; a biblioteca MPL-2.0 fica FORA, como oráculo.**

## ⭐ O que colar AGORA

👉 **[`NEXT_I_eliminacao.md`](NEXT_I_eliminacao.md)** — a **janela I** da obra das
restrições por eliminação. São **DUAS mensagens** (abertura de linha, depois o BLOCO-I).
Abra **janela NOVA e LIMPA** — ⛔ nem a que escreveu a espec, nem a que a auditou.

✅ **A auditoria §4.2 do R-pré está VERDE e atestada no cabeçalho da espec** (2026-08-24) —
era ela que faltava para a janela I poder abrir.
⚠️ **A janela I abre na OBRA A (a costura) apenas.** A OBRA B (as linhas de feição) espera
uma emenda do E, nomeada no cabeçalho da espec — e o §6 da própria espec já mandava fazer a
costura primeiro.

⛔⛔ **Aquela auditoria não foi a de rotina, e produziu três achados de PAREDE** (a rede do
sweep não cobria a implementação que o E leu · a `TRIAGEM` estava do lado errado da parede ·
o `deny` do Passo 0 nunca tinha sido exercido, e corre sob `bypassPermissions`). Os três
estão curados ou nomeados no [`LEDGER §OBRA 2`](LEDGER_quadwild.md); o terceiro passa a ter
**controlo positivo** dentro do próprio Passo 0.

*(cumprido: [`NEXT_R-PRE_eliminacao.md`](NEXT_R-PRE_eliminacao.md), o handoff desta auditoria.)*

### A obra ANTERIOR (a extracção) — ✅ fechada

| passo | estado |
|---|---|
| espec + R-pré | ✅ [`SPEC_extracao_de_malha_quad.md`](SPEC_extracao_de_malha_quad.md) · [`NEXT_R-PRE.md`](NEXT_R-PRE.md) |
| implementação | ✅ `ph2d-quadextract` + `ph2d_gridmap::round`, 20 gates verdes |
| R-pós | ✅ **ledger fechado** em 2026-08-24 ([`LEDGER §Fechamento R`](LEDGER_quadwild.md)) |

⚠️ **A worktree de qualquer papel novo nasce de `line/quadextract`** (que descende de
`line/sculpt3d`) — **é onde esta pasta existe**; do `main` ela nasceria sem a própria espec.
A alternativa é ordenar antes a integração desta pasta para o `main`.

⚠️ **A corrente do §10:** cada papel entrega o bloco do seguinte, salvo em `NEXT_<papel>.md`.
⛔ *Um handoff nunca acrescenta conteúdo além dos campos do molde* — o resto vive na espec
e no ledger.

## Os arquivos

| arquivo | quem lê |
|---|---|
| ⭐⭐ [`SPEC_restricoes_por_eliminacao.md`](SPEC_restricoes_por_eliminacao.md) | **todos** — a espec da obra SEGUINTE: a costura e as linhas de feição, **um mecanismo só**, 9 gates + 7 recusas medidas |
| ⭐ [`NEXT_R-PRE_eliminacao.md`](NEXT_R-PRE_eliminacao.md) | o Enio — **é o que se cola a seguir** |
| ⭐ [`SPEC_extracao_de_malha_quad.md`](SPEC_extracao_de_malha_quad.md) | **todos** — a espec da obra anterior (✅ feita), 11 gates + o 9-bis |
| ⭐ [`fixtures/`](fixtures/README.md) | **todos** — mapas de referência verificados + o verificador |
| ⛔⛔ [`TRIAGEM_quad_remesh.md`](TRIAGEM_quad_remesh.md) | **E e R apenas** — quem decide: a escada de licenças, a patente, e todas as medições. ⚠️ **Ela NOMEIA identificadores internos do alvo** (medido pelo sweep em 2026-08-24, R-pré da obra 2), e até essa data nada a marcava |
| ⛔ [`ACHADO_proveniencia_por_nome_interno.md`](ACHADO_proveniencia_por_nome_interno.md) | **E e R apenas** — quem cura o repo (o texto dele descreve e nunca reproduz, mas não é `SPEC_*`) |
| ⛔ [`NEXT_R-PRE.md`](NEXT_R-PRE.md) · [`NEXT_I.md`](NEXT_I.md) · [`NEXT_R-POS.md`](NEXT_R-POS.md) | ✅ **cumpridos** — a corrente da obra anterior, guardada como registo. O `NEXT_I` é o único que o I lê, e lê-o **colado**, não do disco |
| [`INBOX_quadwild.md`](INBOX_quadwild.md) | o Implementador **escreve** (append cego), nunca lê |
| ⛔ `LEDGER_quadwild.md` · `VASSOURA_quadwild.txt` | **E e R apenas** — carregam rastros do alvo de propósito |

> ⛔⛔ **A regra do §3.I da skill, com todas as letras:** *dentro de `cleanroom/`, o
> Implementador lê **SÓ** `SPEC_*`* (mais os `fixtures/`, que são dados). Tudo o resto desta
> pasta é **E e R**. ⚠️ Até 2026-08-24 esta tabela convidava a `TRIAGEM` e o `ACHADO` sem
> marca, e o `deny` do Passo 0 cobria só `LEDGER_*` e `VASSOURA_*` — *uma porta aberta na
> lista de leitura e fechada em lado nenhum.*

## A regra, em uma linha

⛔ **Quem escreve o código do produto nunca teve a expressão original no contexto.**
O instrumento que o prova é `scripts/cleanroom-sweep.sh`, e a barra é **zero achados na
árvore rastreada** — hoje satisfeita para a família do quad remesh.
