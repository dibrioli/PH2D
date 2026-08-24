# `cleanroom/` — a reimplementação de código restrito, no módulo 3D

Protocolo: [`SKILL_Cleanroom_Reimplementacao.md`](../../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md).
Decisão: [`ADR-0164`](../../architecture/decisions/0164-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md)
— **clean-room dos *papers*; a biblioteca MPL-2.0 fica FORA, como oráculo.**

## ⭐ O que colar, e em que ordem

👉 **[`BLOCOS_para_colar.md`](BLOCOS_para_colar.md)** — os blocos dos três papéis, já
preenchidos. **O próximo passo é o §1: a auditoria R-pré**, e ela é condição de abertura da
janela que implementa.

## Os arquivos

| arquivo | quem lê |
|---|---|
| ⭐ [`SPEC_extracao_de_malha_quad.md`](SPEC_extracao_de_malha_quad.md) | **todos** — a espec funcional, 11 gates + o 9-bis |
| ⭐ [`fixtures/`](fixtures/README.md) | **todos** — mapas de referência verificados + o verificador |
| [`TRIAGEM_quad_remesh.md`](TRIAGEM_quad_remesh.md) | quem decide — a escada de licenças, a patente, e **todas as medições** |
| [`ACHADO_proveniencia_por_nome_interno.md`](ACHADO_proveniencia_por_nome_interno.md) | quem cura o repo |
| [`INBOX_quadwild.md`](INBOX_quadwild.md) | o Implementador **escreve** (append cego), nunca lê |
| ⛔ `LEDGER_quadwild.md` · `VASSOURA_quadwild.txt` | **E e R apenas** — carregam rastros do alvo de propósito |

## A regra, em uma linha

⛔ **Quem escreve o código do produto nunca teve a expressão original no contexto.**
O instrumento que o prova é `scripts/cleanroom-sweep.sh`, e a barra é **zero achados na
árvore rastreada** — hoje satisfeita para a família do quad remesh.
