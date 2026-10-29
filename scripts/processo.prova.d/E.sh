# shellcheck shell=bash
# Item E (comissão 29/10; dono 23/10 e 25/10): na LINHA, até no fecho, nenhum passo manda correr a suíte
# completa nem o `nextest-impacted` — são do integrador. Toda linha dos protocolos da linha que nomeia um
# deles tem de dizer de quem é (integrador) ou proibi-lo. (A `description:` do verificador lista o que ele
# sabe correr — serve também ao integrador.)
echo "— E: a suíte não é da linha —"
_e_docs=".claude/commands/pd-linha-fechar.md .claude/commands/pd-onda-fechar.md .claude/commands/pd-gate-fechamento.md .claude/agents/verificador.md docs/IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md"
# shellcheck disable=SC2086
_e_mal="$(grep -nE 'nextest-impacted|suite-completa|nextest run --workspace' $_e_docs 2>/dev/null \
  | grep -vE ':[0-9]+:description:' \
  | grep -viE 'integrador|integra[cç][aã]o|⛔|\bnem\b|\bnunca\b|\bnão\b|\bnenhuma\b' || true)"
if [ -z "$_e_mal" ]; then ok "nenhum protocolo da linha manda correr a suíte ou o impactado"
else ko "protocolo da linha manda correr a suíte/o impactado:"; printf '      %s\n' "$_e_mal"; fi
