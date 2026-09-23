#!/usr/bin/env bash
# Provas de mutação da VIDA E DANO (plano 28, W1) — a lei pura `ph2d-health`.
#
# Arnês IDÊNTICO ao das waves anteriores desta linha (o da paralaxe): controlo sobre o próprio
# FILTRO (um filtro que casa ZERO testes imprime `ok` e lê-se como «sobreviveu»), `muta` a ABORTAR
# quando a âncora não aparece o número esperado de vezes, rede que restaura no EXIT/INT/TERM/PIPE e
# o modo SECO (`SECO=1`), que confere só as âncoras.
#
# ⚠️ Duas famílias de régua, e cada mutação diz qual a apanha:
#   · a PARIDADE com o oráculo (`a_lei_reproduz_o_oraculo…`, 19 cenários do GDevelop, TODOS os
#     quadros comparados ao bit, e os sorteios gravados TODOS consumidos);
#   · as DIVERGÊNCIAS declaradas (`src/lib_tests.rs`), que só a casa exercita — o oráculo, por
#     construção, não vê nenhuma delas.
#
# uso:  bash scripts/ph2d-run.sh bash docs/Components/ferramentas/mutacao_vida_2026-09-23.sh
set -u
cd "$(dirname "$0")/../../.." || exit 1

TMP="$(mktemp -d)"
FALHAS=0
TOTAL=0

guarda()   { cp "$1" "$TMP/$(basename "$1").$2"; MUTADO="$1"; MUTADO_TAG="$2"; }
restaura() { cp "$TMP/$(basename "$1").$2" "$1"; touch "$1"; MUTADO=""; }

MUTADO=""
MUTADO_TAG=""
ao_sair() {
  if [ -n "$MUTADO" ]; then
    echo "⚠️  interrompido com $MUTADO mutado — a restaurar"
    cp "$TMP/$(basename "$MUTADO").$MUTADO_TAG" "$MUTADO"; touch "$MUTADO"
  fi
}
trap ao_sair EXIT INT TERM PIPE

if grep -n '^bloco "[^"]*`' "$0" >&2; then
  echo "⛔ CRASE no nome de uma prova (ver as linhas acima): ele desfaz a citacao do ficheiro." >&2
  exit 2
fi

muta() { # ficheiro vezes antigo novo
  python3 - "$1" "$2" "$3" "$4" <<'PY'
import sys
p, n, old, new = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4]
s = open(p).read()
c = s.count(old)
if c != n:
    sys.exit(f"  ⛔ ANCORA: {old!r} aparece {c} vezes em {p} (esperado {n})")
open(p, "w").write(s.replace(old, new))
PY
}

prova() { # nome crate filtro
  TOTAL=$((TOTAL+1))
  echo "── $1"
  local out rc corridos
  out=$(timeout 900 cargo test -p "$2" --all-targets -- "$3" 2>&1); rc=$?
  # A população honesta é `passed + failed` — o `running N tests` CONTA os ignorados.
  corridos=$(printf '%s' "$out" | grep -oE 'test result: [a-zA-Z]+\. [0-9]+ passed; [0-9]+ failed' \
             | grep -oE '[0-9]+ (passed|failed)' | grep -oE '[0-9]+' | awk '{s+=$1} END {print s+0}')
  if [ "$corridos" -lt 1 ]; then
    echo "  ⛔ FILTRO VAZIO — '$3' nao casou teste nenhum (ou nao compilou):"
    printf '%s\n' "$out" | grep -E '^error' | head -3
    FALHAS=$((FALHAS+1)); return
  fi
  if [ "$rc" = 0 ]; then
    echo "  ⛔⛔ SOBREVIVEU — os $corridos teste(s) de '$3' passaram sobre o produto mutado"
    FALHAS=$((FALHAS+1))
  else
    echo "  ✅ sangrou (de $corridos teste(s) corridos)"
  fi
}

bloco() { # nome crate filtro ficheiro vezes antigo novo
  guarda "$4" b
  if [ "${SECO:-0}" = 1 ]; then
    TOTAL=$((TOTAL+1))
    muta "$4" "$5" "$6" "$7" || FALHAS=$((FALHAS+1))
    restaura "$4" b
    return
  fi
  if muta "$4" "$5" "$6" "$7"; then
    prova "$1" "$2" "$3"
  else
    TOTAL=$((TOTAL+1)); FALHAS=$((FALHAS+1))
  fi
  restaura "$4" b
}

LEI=crates/ph2d-health/src/lib.rs
ORACULO=a_lei_reproduz_o_oraculo

echo "=== O PIPELINE DO GOLPE (a ordem é a lei) ==="

# ⛔ A armadura por PERCENTAGEM antes da PLANA: `(25 − 5)·0,5 = 10` contra `25·0,5 − 5 = 7,5`.
# O cenário `f3_armadura_ordem` existe para esta pergunta e mais nenhuma.
bloco "a armadura percentual vem antes da plana" ph2d-health "$ORACULO" \
  "$LEI" 1 \
  '            d = (d - cfg.armadura_fixa).max(0.0);
            if cfg.armadura_pct > 0.0 && d > 0.0 {
                d *= 1.0 - cfg.armadura_pct.min(1.0);
            }' \
  '            if cfg.armadura_pct > 0.0 && d > 0.0 {
                d *= 1.0 - cfg.armadura_pct.min(1.0);
            }
            d = (d - cfg.armadura_fixa).max(0.0);'

# ⛔ O sorteio é EXACTAMENTE um por golpe que passa a invencibilidade, mesmo com chance `0` —
# saltá-lo com chance nula é o «optimizar» óbvio, e desalinha toda a sequência gravada.
bloco "o sorteio so acontece com chance positiva" ph2d-health dois_golpes_no_mesmo_quadro \
  "$LEI" 1 \
  '        if sorteio() < cfg.esquiva {' \
  '        if cfg.esquiva > 0.0 && sorteio() < cfg.esquiva {'

bloco "o golpe ignora a invencibilidade" ph2d-health dois_golpes_no_mesmo_quadro \
  "$LEI" 1 \
  '        if self.invencivel(cfg) {
            return;
        }' \
  '        if false {
            return;
        }'

# ⛔ A invencibilidade só EXISTE depois do primeiro golpe que entra: o relógio do golpe nasce a `0`,
# e sem esta metade um objecto recém-criado seria invencível durante o cooldown inteiro.
bloco "a invencibilidade nasce armada" ph2d-health dois_golpes_no_mesmo_quadro \
  "$LEI" 1 \
  '        self.golpeada_alguma_vez
            && cfg.invencivel_s > 0.0' \
  '        cfg.invencivel_s > 0.0'

echo "=== O ESCUDO ==="

# ⛔ O golpe que SÓ toca o escudo arma o cooldown (`e5_escudo_e_cooldown`, primeiro passo).
bloco "o escudo nao arma a invencibilidade" ph2d-health "$ORACULO" \
  "$LEI" 1 \
  '            self.escudo_acabou_de_levar_dano = true;
            self.arma_invencibilidade();' \
  '            self.escudo_acabou_de_levar_dano = true;'

bloco "o bloqueio do excesso e ignorado" ph2d-health "$ORACULO" \
  "$LEI" 1 \
  '                if cfg.escudo_bloqueia_excesso {' \
  '                if false {'

bloco "o maximo do escudo nao limita a activacao" ph2d-health "$ORACULO" \
  "$LEI" 1 \
  '            self.escudo = pontos.min(cfg.escudo_max);' \
  '            self.escudo = pontos;'

# ⛔ A expiração é um `Once()` do alvo: zera UMA vez, no primeiro quadro inactivo. Sem a memória
# ela zera a cada quadro, e o `ActivateShield` sem renovar do `e3` (escudo com duração vencida)
# morreria no quadro seguinte em vez de ficar à espera.
bloco "a expiracao do escudo zera todo quadro" ph2d-health "$ORACULO" \
  "$LEI" 1 \
  '            if !self.expiracao_disparada {
                self.escudo = 0.0;
            }' \
  '            self.escudo = 0.0;'

# ⛔ O relógio da duração NÃO EXISTE até alguém o repor — e a leitura enganadora do
# `ShieldTimeRemaining` (a duração inteira com o escudo por activar) sai daí.
bloco "o relogio do escudo nasce a zero" ph2d-health "$ORACULO" \
  "$LEI" 1 \
  '            escudo_relogio: Relogio(None),' \
  '            escudo_relogio: Relogio(Some(0.0)),'

# ⛔ Regenerar um escudo a ZERO é reactivá-lo com duração nova (o ciclo do `e4`).
bloco "regenerar a zero nao renova a duracao" ph2d-health "$ORACULO" \
  "$LEI" 1 \
  '            if self.escudo == 0.0 {
                self.renova_escudo();
            }' \
  ''

echo "=== A CURA E A REGENERACAO ==="

# ⛔⛔ Esta SOBREVIVEU à 1.ª corrida: o `d_regeneracao` sobe 1 ponto por quadro e cai EXACTAMENTE
# em 100, logo nada perguntava se o alvo corta. O `d3_regeneracao_passa_do_maximo` (0,75 por quadro)
# foi corrido para responder — e o alvo corta, com e sem sobre-cura.
bloco "a regeneracao passa do maximo" ph2d-health "$ORACULO" \
  "$LEI" 1 \
  '            self.pontos += cfg.regen * dt_s;
            if self.pontos > cfg.maximo {
                self.pontos = cfg.maximo;
            }' \
  '            self.pontos += cfg.regen * dt_s;'

# ⛔ O IRMÃO no escudo, que o mesmo cenário (`d3`) mede: 40 + 13·0,75 = 49,75, e o quadro seguinte
# passaria de 50.
bloco "o escudo regenerado passa do maximo" ph2d-health "$ORACULO" \
  "$LEI" 1 \
  '            self.escudo += cfg.escudo_regen * dt_s;
            if self.escudo > cfg.escudo_max {
                self.escudo = cfg.escudo_max;
            }' \
  '            self.escudo += cfg.escudo_regen * dt_s;'

# ⛔ As marcas duram UM quadro: sem a limpeza, `IsJustHealed` fica aceso para sempre.
bloco "a marca da cura nao se apaga" ph2d-health "$ORACULO" \
  "$LEI" 1 \
  '        self.acabou_de_ser_curada = false;' \
  ''

echo "=== AS DIVERGENCIAS DECLARADAS (so a casa as ve) ==="

# ⛔⛔ O defeito do ALVO que a casa não copia: com sobre-cura ele aplica a quantidade da cura
# ANTERIOR. Apagar o ramo faz a casa copiá-lo — e a PARIDADE também sangra, porque o controlo
# `Regras::GDEVELOP` passa a curar o que se pede.
bloco "a sobre-cura da casa usa a cura anterior" ph2d-health a_sobre_cura_cura_o_que_se_pede \
  "$LEI" 1 \
  '        } else if !regras.sobre_cura_usa_a_anterior {' \
  '        } else if regras.sobre_cura_usa_a_anterior {'

bloco "a vida desce abaixo de zero na casa" ph2d-health a_vida_para_em_zero \
  "$LEI" 1 \
  '        if !regras.vida_negativa {
            v = v.max(0.0);
        }' \
  ''

# ⛔ A mesma cerca vive em DOIS sítios (a cura e o golpe); a âncora leva a linha de antes para
# mutar só a da CURA — é ela que ressuscita um morto.
bloco "a cura ressuscita um morto na casa" ph2d-health um_morto_so_volta_por_reviver \
  "$LEI" 1 \
  '        if !regras.morto_nao_e_final && self.morta() {
            return;
        }
        if cfg.maximo == 0.0 {' \
  '        if cfg.maximo == 0.0 {'

bloco "um golpe negativo passa na casa" ph2d-health os_pedidos_negativos_nao_fazem_nada \
  "$LEI" 1 \
  '        if !regras.aceita_negativos && (!dano.is_finite() || dano < 0.0) {' \
  '        if false {'

echo
echo "=== $((TOTAL-FALHAS)) de $TOTAL sangraram ==="
exit "$FALHAS"
