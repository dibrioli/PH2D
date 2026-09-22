#!/usr/bin/env bash
# Prova de mutacao do PLANO A ATRAVESSAR O `.ph2dproj` (21/09) — as duas formas
# das amostras, a migracao do v1, a recusa, e os dois elos (escrever/instalar).
#
# ⚠️ O arnes CONTROLA-SE A SI MESMO nos mesmos QUATRO pontos dos irmaos (ancora
# unica · a mutacao compila · N > 0 testes correram · a corrida limpa VERDE), e
# tem o PRE-VOO (`MUTA_SO_ANCORAS=1`) pela mesma razao: o `cargo fmt` reescreve
# a indentacao de uma ancora e ela passa a casar ZERO, o que se le exactamente
# como uma mutacao que sobreviveu.
#
# ⛔ Chame-o sempre pela porta de recursos:
#   PH2D_PRAZO=2400 bash scripts/ph2d-run.sh bash docs/3D/ferramentas/muta_o_plano_no_ficheiro.sh
set -u
FILTRO="${MUTA_FILTRO:-}"
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
APP=crates/ph2d-app-sculpt3d/src
BK=$(mktemp -d)
cp -r "$APP" "$BK/app"
restore() {
  rm -rf "$APP"; cp -r "$BK/app" "$APP"
  # ⚠️ `cp -r` devolve o mtime ANTIGO e o cargo guarda o build DA MUTACAO.
  find "$APP" -name '*.rs' -exec touch {} +
}
trap restore EXIT

corrida() { cargo nextest run -p ph2d-app-sculpt3d --lib 2>&1; }
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

# ── A FORMA das amostras ─────────────────────────────────────────────────
muta "$APP/doc_tinta.rs" \
  '        Some((n, cor)) if bits(*cor) == bits(a) && *n < u32::MAX => *n += 1,' \
  '        Some((n, cor)) if *cor == a && *n < u32::MAX => *n += 1,' \
  'P1 corridas: a igualdade volta a ser == e o -0.0 junta-se ao +0.0'

muta "$APP/doc_tinta.rs" \
  '    if custo_corridas < amostras.len() * 12 {
        AmostrasDoc::Corridas(corridas)
    } else {
        AmostrasDoc::Cruas(amostras.to_vec())
    }' \
  '    AmostrasDoc::Corridas(corridas)' \
  'P2 forma: o escritor escolhe SEMPRE corridas (o caso 1,083x volta)'

muta "$APP/doc_tinta.rs" \
  '    if custo_corridas < amostras.len() * 12 {
        AmostrasDoc::Corridas(corridas)
    } else {
        AmostrasDoc::Cruas(amostras.to_vec())
    }' \
  '    let _ = custo_corridas;
    AmostrasDoc::Cruas(amostras.to_vec())' \
  'P3 forma: o escritor escolhe SEMPRE cruas (o plano por pintar volta a 75 MB)'

muta "$APP/doc_tinta.rs" \
  '    if total != esperadas as u64 {
        return None;
    }' \
  '' \
  'P4 corridas: a cerca da CONTAGEM desaparece'

muta "$APP/doc_tinta.rs" \
  '            Self::Cruas(v) if v.len() == esperadas => Some(v.clone()),' \
  '            Self::Cruas(v) => Some(v.clone()),' \
  'P5 cruas: a cerca da contagem da forma CRUA desaparece'

# ── O DOCUMENTO ──────────────────────────────────────────────────────────
muta "$APP/doc.rs" \
  '                tinta: tinta.map(|t| TintaDoc {' \
  '                tinta: None::<&Tinta>.map(|t: &Tinta| TintaDoc {' \
  'P6 encode: o plano deixa de ser escrito no ficheiro'

muta "$APP/doc.rs" \
  '    t.amostras_mut().copy_from_slice(&amostras);' \
  '    let _ = amostras;' \
  'P7 decode: o plano e RE-SEMEADO em vez de lido (a tinta volta a ser grossa)'

muta "$APP/doc.rs" \
  '            obj.tinta = peca.tinta;' \
  '            obj.tinta = None;' \
  'P8 install_doc: o plano e lido e deitado fora'

muta "$APP/doc.rs" \
  '                (o.stack.to_data(), o.pose.to_data(), self.plano_de(i))' \
  '                (o.stack.to_data(), o.pose.to_data(), o.tinta.as_ref())' \
  'P9 save: volta a ler o Option da peca (um Ctrl+S a meio de um traco perde o plano)'

muta "$APP/doc.rs" \
  '        V_ANTES_DA_TINTA => {' \
  '        u32::MAX => {' \
  'P10 migracao: um documento v1 deixa de abrir'

muta "$APP/tinta_da_peca.rs" \
  '    if let Some(t) = do_traco
        && t.dono() == obj.id.0
    {' \
  '    if let Some(t) = do_traco {' \
  'P11 porta: o plano emprestado vai para QUALQUER peca'

# ── O CONTROLO ───────────────────────────────────────────────────────────
# ⚠️ Uma mutacao INERTE nao pode sangrar. Sem ela um arnes partido — um filtro
# que casa zero testes, uma arvore ja' vermelha — devolve um placar PERFEITO.
muta "$APP/doc_tinta.rs" \
  'pub(super) type Corrida = (u32, [f32; 3]);' \
  'pub(super) type Corrida = (u32, [f32; 3]);
' \
  'P12 CONTROLO: uma mutacao INERTE (uma linha em branco) nao pode sangrar'

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
  echo "PLACAR: $sangram de $total sangram (o P12 e' o CONTROLO e nao pode)"
fi
