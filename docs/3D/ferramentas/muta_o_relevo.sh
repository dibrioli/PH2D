#!/usr/bin/env bash
# Prova de mutacao do RELEVO da tinta (a etapa 3b do Painter na peca,
# docs/3D/29) — a leitura pelos pesos da cor, a conversao de degrau, a cerca do
# tamanho, a janela do traco e o canal do desfazer.
#
# ⚠️ O arnes CONTROLA-SE A SI MESMO nos QUATRO pontos dos irmaos (ancora unica ·
# a mutacao compila · N > 0 testes correram · a corrida limpa VERDE), e tem o
# PRE-VOO (`MUTA_SO_ANCORAS=1`): o `cargo fmt` reescreve a indentacao de uma
# ancora e ela passa a casar ZERO, o que se le como uma mutacao que sobreviveu.
#
# ⚠️ A POPULACAO e' de quem OBSERVA a mutacao, nunca de quem a CONTEM: as da
# `ph2d-sculpt3d` (a janela do traco) sao vistas pelos gates do desfazer na
# `ph2d-app-sculpt3d`, logo a corrida e' essa crate mais a do plano.
#
# ⛔ Chame-o sempre pela porta de recursos:
#   bash scripts/ph2d-run.sh bash docs/3D/ferramentas/muta_o_relevo.sh
set -u
FILTRO="${MUTA_FILTRO:-}"
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
COR=crates/ph2d-mesh-colors/src
SCU=crates/ph2d-sculpt3d/src
APP=crates/ph2d-app-sculpt3d/src
BK=$(mktemp -d)
cp -r "$COR" "$BK/cor"; cp -r "$SCU" "$BK/scu"; cp -r "$APP" "$BK/app"
restore() {
  rm -rf "$COR" "$SCU" "$APP"
  cp -r "$BK/cor" "$COR"; cp -r "$BK/scu" "$SCU"; cp -r "$BK/app" "$APP"
  # ⚠️ `cp -r` devolve o mtime ANTIGO e o cargo guarda o build DA MUTACAO.
  find "$COR" "$SCU" "$APP" -name '*.rs' -exec touch {} +
}
trap restore EXIT

corrida() { cargo nextest run -p ph2d-mesh-colors -p ph2d-app-sculpt3d --lib 2>&1; }
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

# ── O PLANO (ph2d-mesh-colors) ───────────────────────────────────────────
muta "$COR/relevo.rs" \
  '        o[ALTURA] += a[idx][ALTURA] * peso;' \
  '        o[ALTURA] += a[idx][ALTURA];' \
  'R1 a altura de uma face ignora os pesos da cor'

muta "$COR/relevo.rs" \
  '        o[CORPO] += a[idx][CORPO] * peso;' \
  '        o[CORPO] += a[idx][CORPO];' \
  'R1b o corpo de uma face ignora os pesos da cor'

muta "$COR/relevo.rs" \
  '            .is_some_and(|a| a.len() != self.amostras().len())' \
  '            .is_some_and(|_a| false)' \
  'R2 um relevo do tamanho errado e aceite'

muta "$COR/uniformiza.rs" \
  '        let relevo = self.tem_relevo();' \
  '        let relevo = false;' \
  'R3 levar o plano a um degrau so larga o relevo'

muta "$COR/lib.rs" \
  '                .map_or(0, |a| a.capacity() * size_of::<[f32; 2]>())' \
  '                .map_or(0, |_a| 0)' \
  'R4 o relevo nao conta no peso da peca'

# ── A JANELA DO TRACO (ph2d-sculpt3d) ─────────────────────────────────────
muta "$SCU/tinta_fina.rs" \
  '        self.base_alt.push(self.tinta.espessura(idx as usize));' \
  '        self.base_alt.push([0.0; 2]);' \
  'R5 a altura de antes do traco e sempre zero'

muta "$SCU/tinta_fina_relevo.rs" \
  '            r[0].to_bits() != b[0].to_bits() || r[1].to_bits() != b[1].to_bits()' \
  '            r[0].to_bits() != b[0].to_bits() || r[1].to_bits() != b[1].to_bits() || true' \
  'R6 toda pincelada de cor diz que mudou o relevo'

muta "$SCU/tinta_fina_relevo.rs" \
  '            r[0].to_bits() != b[0].to_bits() || r[1].to_bits() != b[1].to_bits()' \
  '            r[0].to_bits() != b[0].to_bits()' \
  'R6b um traco que so muda o CORPO diz que nao mudou o relevo'

# ── O DESFAZER (ph2d-app-sculpt3d) ────────────────────────────────────────
muta "$APP/history_tinta_fina.rs" \
  '            .map(|a| super::swap_window(t.relevo_mut(), &self.amostras, a));' \
  '            .map(|a| a.clone());' \
  'R7 o desfazer nao troca o relevo'

muta "$APP/history_tinta_fina.rs" \
  '            relevo: t.relevo_mudou().then(|| t.base_relevo().to_vec()),' \
  '            relevo: None,' \
  'R8 a janela do traco nao guarda o relevo'

# ── O CONTROLO ───────────────────────────────────────────────────────────
# ⚠️ Uma mutacao INERTE nao pode sangrar. Sem ela um arnes partido — um filtro
# que casa zero testes, uma arvore ja' vermelha — devolve um placar PERFEITO.
muta "$COR/relevo.rs" \
  '    /// Este plano tem relevo? — `false` até à 1.ª escrita.' \
  '    /// Este plano tem relevo? — `false` até à primeira escrita.' \
  'R9 CONTROLO: uma mutacao INERTE (um comentario) nao pode sangrar'

echo
if [ -n "$SO_ANCORAS" ]; then
  echo "PRE-VOO: $sangram de $total ancoras casam exactamente uma vez (ZERO testes corridos)"
  [ "$sangram" -eq "$total" ] || exit 1
  exit 0
fi
if [ -n "$FILTRO" ]; then
  echo "PLACAR PARCIAL (filtro MUTA_FILTRO='$FILTRO'): $sangram de $total sangram"
else
  echo "PLACAR: $sangram de $total sangram (o R9 e' o CONTROLO e nao pode)"
fi
