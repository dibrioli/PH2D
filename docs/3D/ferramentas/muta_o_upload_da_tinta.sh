#!/usr/bin/env bash
# Prova de mutacao do UPLOAD INCREMENTAL da tinta fina (a cura de 2026-09-21):
# a janela das amostras sujas (`ph2d-sculpt3d`) e a aritmetica das corridas
# (`ph2d-mesh-render`).
#
# ⚠️ Os QUATRO controlos do arnes irmao (`muta_a_metade_visivel.sh`) valem aqui
# pela mesma razao, e cada um ja' mentiu nesta casa:
#   (a) a ancora casa EXACTAMENTE uma vez;
#   (b) a mutacao tem de COMPILAR — um erro de compilacao le-se como sangrar;
#   (c) a corrida tem de correr N > 0 testes, contados no `tests run`.
#   (d) a corrida LIMPA tem de estar VERDE — senao todo o placar e' fabricado.
set -u
# ⭐ O PRE-VOO (`MUTA_SO_ANCORAS=1`) — acrescentado em 22/09. Uma ancora que
#   casa ZERO le-se num placar exactamente como uma mutacao que SOBREVIVEU,
#   e um `cargo fmt` ou um corte de tecto de LOC reescreve-a sem avisar.
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
SC=crates/ph2d-sculpt3d/src
MR=crates/ph2d-mesh-render/src
BK=$(mktemp -d)
cp -r "$SC" "$BK/sc"; cp -r "$MR" "$BK/mr"
restore() {
  rm -rf "$SC" "$MR"
  cp -r "$BK/sc" "$SC"; cp -r "$BK/mr" "$MR"
  find "$SC" "$MR" -name '*.rs' -exec touch {} +
}
trap restore EXIT

corrida() { cargo "nextest" run -p ph2d-sculpt3d -p ph2d-mesh-render --lib 2>&1; }
populacao() { grep -oP '\K[0-9]+(?= tests? run)' | awk '{s+=$1}END{print s+0}'; }

# ⭐⭐⭐⭐ **O 4.o CONTROLO DO ARNES: a corrida limpa tem de estar VERDE.**
# ⛔⛔ Ate' 2026-09-21 so' se contava que ela CORREU testes (`> 0`), e o `|`
# deitava fora o codigo de saida. Com a arvore vermelha ANTES de mutar, TODA
# mutacao le-se como SANGRA e o placar sai perfeito e fabricado — e o erro e'
# para o lado que nao se nota, porque um placar cheio nao faz ninguem olhar.
# ⚠️⚠️ A corrida limpa SO' corre quando nao e' pre-voo. Sem esta guarda
#   o sumario do pre-voo diz "ZERO testes corridos" DEPOIS de ter corrido a
#   suite inteira — *o instrumento a mentir sobre si mesmo*, e foi assim que
#   este ficheiro ficou na 1.a redaccao da cura (22/09).
if [ -z "$SO_ANCORAS" ]; then
  limpa=$(corrida); rc_limpo=$?
  verde=$(printf '%s' "$limpa" | populacao)
  echo "VERDE antes: $verde testes correram"
  [ "${verde:-0}" -gt 0 ] || { echo "ABORTO: a corrida limpa nao correu teste nenhum"; exit 2; }
  if [ "$rc_limpo" -ne 0 ]; then
    echo "ABORTO: a corrida limpa esta' VERMELHA -- um placar tirado daqui e' fabricado."
    printf '%s' "$limpa" | grep -E '^ *(FAIL|test result:|Summary)' | tail -8 | sed 's/^/      | /'
    exit 2
  fi
fi

sangram=0; total=0
muta() {
  local f="$1" agulha="$2" subst="$3" nome="$4"
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

# ── A JANELA das amostras sujas ──────────────────────────────────────────
muta "$SC/tinta_fina.rs" \
  '            fina.suja[s] = true;' \
  '' \
  'U1 a escrita deixa de sujar a amostra'

muta "$SC/tinta_fina.rs" \
  '            if *suja {
                *suja = false;' \
  '            if *suja {' \
  'U2 drenar deixa de LIMPAR (a janela vira a do TRACO, nao a do QUADRO)'

# ⚠️⚠️ **A U3 NÃO PODE SANGRAR, e isso e' MEDIDO e nao um gate em falta.**
# O `slot_de` tem **UM** chamador (`tinta_fina.rs:423`) e esse chamador marca a
# amostra suja **incondicionalmente** no fim da escrita (`:447`, a agulha da U1)
# => nascer suja e' o CINTO das suspensorias da U1, e nao existe percurso no
# produto em que um slot seja criado sem ser escrito a seguir.
#
# ⛔ Ela FICA na mesma: apaga-la para comprar um ponto no placar seria optimizar
# a metrica. Um segundo chamador de `slot_de` que nao escreva torna-a viva no
# mesmo dia — e e' por isso que ela continua aqui, a ser contada como NOMEADA.
muta "$SC/tinta_fina.rs" \
  '        self.suja.push(true);' \
  '        self.suja.push(false);' \
  'U3 NOMEADA: a amostra nasce limpa (inobservavel: 1 chamador, que sempre marca)'

# ── A ARITMETICA das corridas ────────────────────────────────────────────
muta "$MR/tinta_gpu.rs" \
  '    sujas.dedup();' \
  '' \
  'U4 as repetidas nao sao removidas'

muta "$MR/tinta_gpu.rs" \
  '        out.push((inicio as usize * 12, (fim as usize + 1) * 12));' \
  '        out.push((inicio as usize * 4, (fim as usize + 1) * 4));' \
  'U5 a unidade deixa de ser a AMOSTRA (12 bytes)'

muta "$MR/tinta_gpu.rs" \
  '        while i + 1 < sujas.len() && sujas[i + 1] == fim + 1 {
            i += 1;
            fim = sujas[i];
        }' \
  '' \
  'U6 as contiguas deixam de ser agrupadas'

muta "$MR/tinta_gpu.rs" \
  '    sujas.sort_unstable();' \
  '' \
  'U7 a lista chega por ordem de TOQUE e nao e ordenada'

echo
if [ -n "$SO_ANCORAS" ]; then
  echo "PRE-VOO: $sangram de $total ancoras casam exactamente uma vez (ZERO testes corridos)"
  [ "$sangram" -eq "$total" ]
  exit $?
fi
echo "MUTACAO DO UPLOAD: $sangram de $total sangram (a U3 sobrevive de proposito)"
# ⛔⛔ **Este arnes NAO tinha tecto nenhum ate 22/09, e isso foi medido:
#   ele acabava num `echo`, logo o codigo de saida era o do `echo` — **zero**,
#   com ou sem sobrevivente. *Um arnes sem tecto nao reprova; ele RELATA*, e um
#   laco de portao que so' leia o `rc` le-o como verde para sempre.
# ⚠️ O tecto e' `total - 1`: a U3 esta' NOMEADA acima, e o numero saiu de uma
#   corrida (`6 de 7`, 22/09), nunca de um palpite.
[ "$sangram" -eq $((total - 1)) ]
