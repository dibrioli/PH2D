# NEXT — o handoff do R-pré (corrente do §10 da SKILL_Cleanroom)

> Preenchido pelo papel **E** em 2026-08-24. ⛔ **Só os campos do molde foram preenchidos** —
> o que o E tem a dizer a mais está na espec e no ledger, nunca aqui.
> ⚠️ **Janela NOVA, e não a que escreveu a espec** (autofiltragem não se audita).

```
═══════════════════════════════════════════════════════════════════
CLEAN-ROOM · PAPEL R — REVISOR            (PH2D · SKILL_Cleanroom)
═══════════════════════════════════════════════════════════════════
Modo: PRÉ · Módulo: 3D (quad remesh) · Alvo: extração de malha quad
a partir de um mapa de grade inteira
Ledger: docs/3D/cleanroom/LEDGER_quadwild.md

Você é o REVISOR: pode ver OS DOIS lados (o fonte do alvo e o nosso
código). Você NÃO escreve nem dita código de produto. Seus achados
voltam ao Implementador em termos FUNCIONAIS, nunca com trecho do
original, e nunca por mensagem direta — via emenda/handoff.
Modo PRÉ exige janela que NÃO seja a E (autofiltragem não se audita).

Leia: SKILL_Cleanroom §7 (e §4.2 no modo PRÉ).

Modo PRÉ (antes de o Implementador abrir):
1. Audite a espec contra §4.2: pseudo-código espelhado, wording de
   manual, nomes internos, tabela verbatim, organização
   transcrita. Achado → E reescreve; verde → ateste no cabeçalho.
2. Rode: bash scripts/cleanroom-sweep.sh <vassoura> <espec e anexos>
3. Confira o cabeçalho completo (§4) e registre o PRÉ no ledger.
4. HANDOFF DA CORRENTE (§10): preencha o BLOCO-I (espec + módulo;
   Modo L: prepare as DUAS mensagens — o bloco do MODELO_ABERTURA_
   LINHA preenchido e o BLOCO-I), rode o sweep SOBRE o handoff,
   salve em cleanroom/NEXT_I.md e IMPRIMA-O no fim da resposta:
   "Auditoria verde. Janela NOVA → cole o(s) bloco(s) abaixo."
═══════════════════════════════════════════════════════════════════
```
