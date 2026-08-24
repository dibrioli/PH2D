# `cleanroom/` — a reimplementação de código restrito, no módulo 3D

Protocolo: [`SKILL_Cleanroom_Reimplementacao.md`](../../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md).
Decisão: [`ADR-0164`](../../architecture/decisions/0164-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md)
— **clean-room dos *papers*; a biblioteca MPL-2.0 fica FORA, como oráculo.**

## ⭐ O que colar AGORA

👉 **[`NEXT_R-PRE_eliminacao.md`](NEXT_R-PRE_eliminacao.md)** — a auditoria da espec da
**obra seguinte**. Abra **janela nova** (⛔ **não** a que escreveu a espec) e cole.
Sem o atestado do §4.2 que ela produz, a janela I **não abre**.

⛔⛔ **E esta auditoria não é a de rotina.** O E desta espec **viu** a montagem de
restrições de uma implementação de referência — logo o risco é convergência de expressão a
entrar **pelo próprio E**, e o handoff põe isso como item nº1.

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
| [`TRIAGEM_quad_remesh.md`](TRIAGEM_quad_remesh.md) | quem decide — a escada de licenças, a patente, e **todas as medições** |
| [`ACHADO_proveniencia_por_nome_interno.md`](ACHADO_proveniencia_por_nome_interno.md) | quem cura o repo |
| [`NEXT_R-PRE.md`](NEXT_R-PRE.md) · [`NEXT_I.md`](NEXT_I.md) · [`NEXT_R-POS.md`](NEXT_R-POS.md) | ✅ **cumpridos** — a corrente da obra anterior, guardada como registo |
| [`INBOX_quadwild.md`](INBOX_quadwild.md) | o Implementador **escreve** (append cego), nunca lê |
| ⛔ `LEDGER_quadwild.md` · `VASSOURA_quadwild.txt` | **E e R apenas** — carregam rastros do alvo de propósito |

## A regra, em uma linha

⛔ **Quem escreve o código do produto nunca teve a expressão original no contexto.**
O instrumento que o prova é `scripts/cleanroom-sweep.sh`, e a barra é **zero achados na
árvore rastreada** — hoje satisfeita para a família do quad remesh.
