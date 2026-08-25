# INBOX — o canal do IMPLEMENTADOR para o ledger

> ⛔ **O Implementador NUNCA abre o `LEDGER_*`** — ele carrega rastros do alvo de propósito.
> Este arquivo é o canal de ida, e escreve-se por **append CEGO** (`cat >>` / `echo >>`),
> **nunca lendo**. Quem transcreve para o ledger é o E ou o R.

## O que escrever aqui

| quando | o quê |
|---|---|
| **na abertura** | `I session: <session-id> <data>` — o R confere, no fecho, que não pertence ao conjunto das janelas queimadas |
| **relance** (§6.2) | viu uma assinatura ou um nome isolado num *preview* de busca: registe origem, URL, quando. ⛔ **DESCREVA, nunca reproduza** — se precisar de identificar o trecho, registe o `sha256` dele |
| **tripwire de recall** (§3.I) | um detalhe que a espec e os *papers* não deram e «veio» sozinho (nome interno, *typo*, constante mágica): **não o escreva no código** — registe aqui |
| **dúvida que a espec não responde** | ⛔ **não vá olhar.** Escreva a pergunta aqui e avise o Enio; o E emenda a espec |

## Como escrever sem ler

```
echo "I session: <id> $(date -I)" >> docs/3D/cleanroom/INBOX_quadwild.md
```

---

## Registos

_(vazio — a primeira linha é a declaração de sessão do Implementador)_
I session: 186ce13e-479b-467a-904c-0ff087ab76c9 2026-08-24
I session: 7499b0f4-218e-489b-879b-1e5a1c8b851f 2026-08-24 | canario: VAZOU
