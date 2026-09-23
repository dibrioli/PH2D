#!/usr/bin/env bash
# muta_a_saida_da_tinta.sh — prova de mutacao do AVISO DA SAIDA (22/09).
#
# O que ele defende: exportar uma peca pintada a `8x` entrega a PROJECCAO do
# plano nos vertices — a tinta de volta a' resolucao da malha — e ate' 22/09 o
# app nao dizia nada. A cura tem tres pecas (a tabela do formato · a clausula
# na lei partilhada · a porta que responde se ESTA cena tem plano) e elas
# falham de maneiras diferentes.
#
# ⚠️ O arnes CONTROLA-SE A SI MESMO nos mesmos QUATRO pontos dos irmaos (ancora
# unica · a mutacao compila · N > 0 testes correram · a corrida limpa VERDE), e
# tem o PRE-VOO (`MUTA_SO_ANCORAS=1`) pela mesma razao: o `cargo fmt` reescreve
# a indentacao de uma ancora e ela passa a casar ZERO, o que se le exactamente
# como uma mutacao que sobreviveu.
#
# ⚠️ A populacao sao DUAS crates: a lei partilhada vive na `ph2d-mesh` e a
# porta na `ph2d-app-sculpt3d`. Uma corrida so' de uma delas leria VERDE sobre
# a mutacao da outra — e isso e' um placar fabricado.
#
# ⛔⛔ **E a 1.a corrida violou essa mesma linha, com a S6 a SOBREVIVER (22/09):**
# ela muta a `ph2d-app-field3d`, que nao esta' na populacao — *a crate mutada nem
# e' compilada, logo a mutacao nao pode sangrar*. ⭐ A cura NAO foi acrescentar a
# crate (isso paga uma suite inteira a cada uma das 7 corridas): foi escrever o
# elo dela no censo da fiacao, que vive na `ph2d-app-sculpt3d` e a alcanca por
# `include_str!` relativo ⇒ quem OBSERVA passou a estar na populacao.
#
# ⇒ **A populacao de um arnes e' de quem OBSERVA a mutacao, nunca de quem a
# CONTEM.** Um `-p` por crate mutada e' a leitura ingenua e a mais cara.
#
# ⛔ Chame-o sempre pela porta de recursos:
#   PH2D_PRAZO=2400 bash scripts/ph2d-run.sh bash docs/3D/ferramentas/muta_a_saida_da_tinta.sh
set -u
FILTRO="${MUTA_FILTRO:-}"
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
MESH=crates/ph2d-mesh/src
APP=crates/ph2d-app-sculpt3d/src
FIELD=crates/ph2d-app-field3d/src
BK=$(mktemp -d)
cp -r "$MESH" "$BK/mesh"
cp -r "$APP" "$BK/app"
cp -r "$FIELD" "$BK/field"
restore() {
  rm -rf "$MESH"; cp -r "$BK/mesh" "$MESH"
  rm -rf "$APP"; cp -r "$BK/app" "$APP"
  rm -rf "$FIELD"; cp -r "$BK/field" "$FIELD"
  # ⚠️ `cp -r` devolve o mtime ANTIGO e o cargo guarda o build DA MUTACAO.
  find "$MESH" "$APP" "$FIELD" -name '*.rs' -exec touch {} +
}
trap restore EXIT

corrida() { cargo nextest run -p ph2d-mesh -p ph2d-app-sculpt3d 2>&1; }
populacao() { grep -oP '\K[0-9]+(?= tests? run)' | awk '{s+=$1}END{print s+0}'; }

if [ -z "$SO_ANCORAS" ]; then
  limpa=$(corrida); rc_limpo=$?
  verde=$(printf '%s' "$limpa" | populacao)
  echo "VERDE antes: $verde testes correram"
  [ "${verde:-0}" -gt 0 ] || { echo "ABORTO: a corrida limpa nao correu teste nenhum"; exit 2; }
  if [ "$rc_limpo" -ne 0 ]; then
    echo "ABORTO: a corrida limpa esta' VERMELHA -- um placar tirado daqui e' fabricado."
    printf '%s' "$limpa" | grep -E '^ *(FAIL|Summary)' | tail -8 | sed 's/^/      | /'
    exit 2
  fi
fi

sangram=0; total=0
muta() { # ficheiro  ancora  substituto  nome
  local f="$1" agulha="$2" subst="$3" nome="$4"
  if [ -n "$FILTRO" ] && ! printf '%s' "$nome" | grep -Eq "$FILTRO"; then return; fi
  total=$((total+1))
  local n; n=$(python3 -c 'import sys;print(open(sys.argv[1]).read().count(sys.argv[2]))' "$f" "$agulha")
  if [ -n "$SO_ANCORAS" ]; then
    if [ "$n" -ne 1 ]; then echo "  ✗ ANCORA [$nome]: casou $n vezes (esperado 1)"; else sangram=$((sangram+1)); fi
    return
  fi
  if [ "$n" -ne 1 ]; then echo "  ABORTO [$nome]: a ancora casou $n vezes (esperado 1)"; return; fi
  python3 -c '
import sys
p,a,b = sys.argv[1], sys.argv[2], sys.argv[3]
s = open(p).read()
assert s.count(a) == 1, (p, s.count(a))
open(p,"w").write(s.replace(a, b, 1))
' "$f" "$agulha" "$subst"
  touch "$f"
  local out rc corridos
  out=$(corrida); rc=$?
  if echo "$out" | grep -q '^error\[\|^error: could not compile'; then
    echo "  ABORTO [$nome]: a mutacao nao compila"
  else
    corridos=$(printf '%s' "$out" | populacao)
    if [ "$corridos" -eq 0 ]; then
      echo "  ABORTO [$nome]: zero testes correram — as ultimas linhas foram:"
      printf '%s' "$out" | tail -6 | sed 's/^/      | /'
    elif [ $rc -ne 0 ]; then echo "  SANGRA  [$nome]"; sangram=$((sangram+1))
    else echo "  SOBREVIVE [$nome]  <<<<"; fi
  fi
  restore
}

# ── A TABELA DO FORMATO ──────────────────────────────────────────────────
# ⚠️ **RE-ANCORADA em 22/09:** a lei mudou (o OBJ passou a CARREGAR a tinta fina
#    num `.png` ao lado), logo a mutacao de ANTES — `false` -> `true` — deixou de
#    casar. ⭐ A pergunta que este arnes faz continua a mesma, e ela inverte-se:
#    hoje o que ha' a mutar e' os formatos que NAO a carregam passarem a dizer
#    que sim, e ai' o `.ply` e o `.stl` calam-se sobre uma perda que acontece.
muta "$MESH/export.rs" \
  '    pub fn keeps_fine_paint(self) -> bool {
        matches!(self, Self::Obj)
    }' \
  '    pub fn keeps_fine_paint(self) -> bool {
        true
    }' \
  'S1 os tres formatos mentem e dizem que CARREGAM a tinta fina'

# ── A CLAUSULA, nas duas metades ─────────────────────────────────────────
# ⚠️ As duas mutacoes da clausula mantem `has_fine_paint` USADO de proposito.
# Apagar o bloco inteiro (ou o `has_fine_paint &&`) deixa o parametro sem uso, e
# entao a mutacao passa a depender da politica de warnings da arvore: com
# `-D warnings` ela NAO COMPILA e o arnes aborta, o que se le como «nao entrou».
# Uma mutacao imune a' configuracao de lint mede a LEI; a outra mede o ambiente.
# ⚠️ RE-APONTADA em 22/09: a clausula perdeu a EXPLICACAO (`(mesh resolution
#    only)`) quando o aviso passou a ter de CABER no balao, e a ancora de tres
#    linhas passou a casar ZERO. A de agora e' a chamada sozinha — ela mantem o
#    parametro USADO (o `if` fica) e apaga so' o efeito, que e' o que a S2 mede.
muta "$MESH/read.rs" \
  '        lost.push("fine paint");' \
  '' \
  'S2 a clausula cala-se: a perda volta a ser SILENCIOSA'

muta "$MESH/read.rs" \
  '    if has_fine_paint && !fmt.keeps_fine_paint() {' \
  '    if has_fine_paint || !fmt.keeps_fine_paint() {' \
  'S3 a metade da CENA deixa de decidir: o aviso passa a soar SEMPRE'

# ── A PORTA ──────────────────────────────────────────────────────────────
muta "$APP/tinta_da_peca.rs" \
  '    (0..objects.len()).any(|i| plano_da_peca(objects, do_traco, i).is_some())' \
  '    let _ = do_traco;
    objects.iter().any(|o| o.tinta.is_some())' \
  'S4 a porta volta a ler o Option da peca: a meio de um traco o aviso cala-se'

# ── O ELO ────────────────────────────────────────────────────────────────
# ⚠️ **RE-ANCORADA em 22/09:** a chamada ganhou a `perdeu_tinta_fina` pelo meio
#    (ter tinta fina e PERDE-LA sao perguntas diferentes desde que o OBJ a
#    carrega), e a de antes passou a casar ZERO.
muta "$APP/export.rs" \
  '                    export_assado::perdeu_tinta_fina(' \
  '                    false && export_assado::perdeu_tinta_fina(' \
  'S5 a saida crava o false e o aviso nunca soa'

# ── O SEGUNDO CONSUMIDOR ─────────────────────────────────────────────────
muta "$FIELD/export.rs" \
  '("fmt", &(ph2d_mesh::lost_by(fmt, false))),' \
  '("fmt", &(ph2d_mesh::lost_by(fmt, true))),' \
  'S6 a modelagem 3D avisa de uma perda que nao acontece'

# ── O CONTROLO ───────────────────────────────────────────────────────────
# ⚠️ Uma mutacao INERTE nao pode sangrar. Sem ela um arnes partido — um filtro
# que casa zero testes, uma arvore ja' vermelha — devolve um placar PERFEITO.
muta "$MESH/read.rs" \
  '#[must_use]
pub fn lost_by(' \
  '#[must_use]

pub fn lost_by(' \
  'S7 CONTROLO: uma mutacao INERTE (uma linha em branco) nao pode sangrar'

echo
if [ -n "$SO_ANCORAS" ]; then
  # ⚠️ **O sumario tem de dizer o que ele MEDIU.** Aqui nenhum teste correu.
  echo "PRE-VOO: $sangram de $total ancoras casam exactamente uma vez (ZERO testes corridos)"
  [ "$sangram" -eq "$total" ] || exit 1
  exit 0
fi
if [ -n "$FILTRO" ]; then
  echo "PLACAR PARCIAL (filtro MUTA_FILTRO='$FILTRO'): $sangram de $total sangram"
else
  echo "PLACAR: $sangram de $total sangram (o S7 e' o CONTROLO e nao pode)"
fi
