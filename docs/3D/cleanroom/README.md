# `cleanroom/` — a reimplementação de código restrito, no módulo 3D

Protocolo: [`SKILL_Cleanroom_Reimplementacao.md`](../../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md).
Decisão: [`ADR-0164`](../../architecture/decisions/0164-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md)
— **clean-room dos *papers*; a biblioteca MPL-2.0 fica FORA, como oráculo.**

## ⭐ O que colar, e em que ordem

👉 **[`NEXT_I.md`](NEXT_I.md)** — o bloco do **próximo papel**, já preenchido pelo **R-pré**.
São **DUAS** mensagens (Modo L): abertura de linha, e depois o BLOCO-I. Abra **janela nova e
limpa** (nem a E, nem a do R-pré) e cole-as em sequência.

⚠️ **Ele traz UMA decisão sua na cabeça:** a worktree do Implementador tem de nascer de
`line/sculpt3d` — **é onde esta pasta existe**; do `main` ela nasceria sem a própria espec.
A alternativa é ordenar antes a integração desta pasta para o `main`.

✅ [`NEXT_R-PRE.md`](NEXT_R-PRE.md) — **cumprido** em 2026-08-24 (veredito no ledger, §Papel R).

⚠️ **A corrente do §10:** cada papel entrega o bloco do seguinte, salvo em `NEXT_<papel>.md`.
O I produzirá o `NEXT_R-POS.md`. ⛔ *Um handoff nunca acrescenta conteúdo além dos campos do
molde* — o resto vive na espec e no ledger.

## Os arquivos

| arquivo | quem lê |
|---|---|
| ⭐ [`SPEC_extracao_de_malha_quad.md`](SPEC_extracao_de_malha_quad.md) | **todos** — a espec funcional, 11 gates + o 9-bis |
| ⭐ [`fixtures/`](fixtures/README.md) | **todos** — mapas de referência verificados + o verificador |
| [`TRIAGEM_quad_remesh.md`](TRIAGEM_quad_remesh.md) | quem decide — a escada de licenças, a patente, e **todas as medições** |
| [`ACHADO_proveniencia_por_nome_interno.md`](ACHADO_proveniencia_por_nome_interno.md) | quem cura o repo |
| ⭐ [`NEXT_R-PRE.md`](NEXT_R-PRE.md) | o Enio — é o que se cola a seguir |
| [`INBOX_quadwild.md`](INBOX_quadwild.md) | o Implementador **escreve** (append cego), nunca lê |
| ⛔ `LEDGER_quadwild.md` · `VASSOURA_quadwild.txt` | **E e R apenas** — carregam rastros do alvo de propósito |

## A regra, em uma linha

⛔ **Quem escreve o código do produto nunca teve a expressão original no contexto.**
O instrumento que o prova é `scripts/cleanroom-sweep.sh`, e a barra é **zero achados na
árvore rastreada** — hoje satisfeita para a família do quad remesh.
