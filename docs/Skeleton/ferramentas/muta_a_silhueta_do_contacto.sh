#!/usr/bin/env bash
# Prova de mutacao da SILHUETA DO CONTACTO (2026-09-29) — numa dobra forte o desenho
# fiel da pele vectorial sobrepoe-se a si mesmo e sai como a uniao dos membros.
#
# Tres camadas:
#   * a PORTA — `ph2d_vec_boolean::resolve_overlap` / `crosses_itself` (overlap.rs);
#   * o FIO   — `skin_desenho::calcula` + a porta de bisseccao `PH2D_SKIN_CONTACTO`;
#   * a QUINA (2026-09-30, report do dono: «a depender do angulo a quina fica
#     inconsistente. Faca obedecer ao que foi escolhido no painel») — a solda, a limpeza
#     das alcas e o limite do bico UM so' para o ecra, o Outline Stroke e o SVG.
#
# ⚠️ A POPULACAO e' de quem OBSERVA a mutacao, nunca de quem a CONTEM: as mutacoes do
# limite do bico no renderer e no SVG so' sao vistas pelos gates DESSAS crates, e o
# `EXTRA=<ren|apv>` de cada `muta` acrescenta-os a corrida so' onde e' preciso.
#
# ⚠️ O arnes CONTROLA-SE A SI MESMO nos QUATRO pontos dos irmaos (ancora unica ·
# a mutacao compila · N > 0 testes correram · a corrida limpa esta' VERDE).
#   bash scripts/ph2d-run.sh bash docs/Skeleton/ferramentas/muta_a_silhueta_do_contacto.sh
# ⭐ `MUTA_SO_ANCORAS=1` e' o pre-voo; `MUTA_FILTRO=<ERE>` corre so' as que casam.
set -u
FILTRO="${MUTA_FILTRO:-}"
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
BOO=crates/ph2d-vec-boolean/src
SKL=crates/ph2d-skeleton-live/src
REN=crates/ph2d-vec-render/src
APV=crates/ph2d-app-vec/src
BK=$(mktemp -d)
cp -r "$BOO" "$BK/boo"; cp -r "$SKL" "$BK/skl"; cp -r "$REN" "$BK/ren"; cp -r "$APV" "$BK/apv"
restore() {
  rm -rf "$BOO" "$SKL" "$REN" "$APV"
  cp -r "$BK/boo" "$BOO"; cp -r "$BK/skl" "$SKL"; cp -r "$BK/ren" "$REN"; cp -r "$BK/apv" "$APV"
  # ⚠️ `cp -r` devolve o mtime ANTIGO e o cargo guarda o build DA MUTACAO.
  find "$BOO" "$SKL" "$REN" "$APV" -name '*.rs' -exec touch {} +
}
trap restore EXIT
EXTRA=""

corrida() {
  local a b c ra rb rc=0
  a=$(cargo nextest run -p ph2d-vec-boolean --lib -E 'test(overlap) | test(bola)' 2>&1); ra=$?
  b=$(cargo nextest run -p ph2d-skeleton-live --lib \
    -E 'test(skin_desenho::tests) | test(skinned_mesh::desenho_tests)' 2>&1); rb=$?
  c=""
  case "$EXTRA" in
    *ren*) c=$(cargo nextest run -p ph2d-vec-render --lib -E 'test(o_bico_do_traco)' 2>&1) || rc=1 ;;
  esac
  case "$EXTRA" in
    *apv*) c="$c
$(cargo nextest run -p ph2d-app-vec --lib -E 'test(o_svg_leva_o_limite_do_bico)' 2>&1)" || rc=1 ;;
  esac
  printf '%s\n%s\n%s\n' "$a" "$b" "$c"
  [ "$ra" -eq 0 ] && [ "$rb" -eq 0 ] && [ "$rc" -eq 0 ]
}

populacao() { grep -oP '\K[0-9]+(?= tests? run)' | awk '{s+=$1}END{print s+0}'; }

if [ -z "$SO_ANCORAS" ]; then
EXTRA="ren apv"
limpa=$(corrida); rc_limpo=$?
EXTRA=""
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

# ── A PORTA ──────────────────────────────────────────────────────────────
muta "$BOO/overlap.rs" '    if !crosses_itself(&bez) {
        return None;
    }' '    if false && !crosses_itself(&bez) {
        return None;
    }' \
  'P1 a porta corre SEMPRE, com ou sem contacto'

muta "$BOO/overlap.rs" '    out.verts = outer;' '    let _ = outer;' \
  'P2 a silhueta devolve o contorno de antes'

muta "$BOO/overlap.rs" '    out.subpaths = resto;' '    let _ = resto;' \
  'P3 as ilhas da uniao perdem-se'

muta "$BOO/overlap.rs" '    d1 * d2 < 0.0 && d3 * d4 < 0.0' '    d1 * d2 <= 0.0 && d3 * d4 <= 0.0' \
  'P4 um TOQUE passa a contar como cruzamento'

# ⛔ O salto dos VIZINHOS foi APAGADO: a mutacao que o removia SOBREVIVEU, porque um
# extremo partilhado da' um teste de lado ZERO EXACTO e o teste estrito ja' o recusa.

muta "$BOO/overlap.rs" '    if !path.closed || path.subpaths.iter().any(|c| !c.closed) {' \
  '    if path.subpaths.iter().any(|c| !c.closed) {' \
  'P6 um caminho ABERTO ganha silhueta'

# ── O FIO ────────────────────────────────────────────────────────────────
muta "$SKL/skin_desenho.rs" '            if leis.contacto {
                ph2d_vec_boolean' '            if false && leis.contacto {
                ph2d_vec_boolean' \
  'F1 o desenho da pele nunca passa pela porta'

muta "$SKL/skin_desenho.rs" '    valor != Some("0")' '    valor == Some("1")' \
  'F2 o contacto nasce DESLIGADO'

# ── A QUINA ──────────────────────────────────────────────────────────────
muta "$BOO/overlap.rs" '        .map(|v| solda_os_segmentos_curtos(v, solda))' '        .map(|v| {
            let _ = solda;
            v
        })' \
  'Q1 a silhueta nao passa pela solda'

muta "$BOO/overlap.rs" '        if minusculo {' '        if false && minusculo {' \
  'Q2 nenhum segmento minusculo e soldado'

# ⛔ A limpeza ANTES da solda foi APAGADA: a mutacao que a removia SOBREVIVEU — a solda
# nao decide nada a partir das alcas limpas, e o ruido medido esta' tres ordens abaixo da
# tolerancia. So' a limpeza DEPOIS dela (Q4) e' lei.
muta "$BOO/overlap.rs" '            p2 = p0;' '            let _ = p0;' \
  'Q3 a alca de ENTRADA caida no comeco fica'

muta "$BOO/overlap.rs" '    limpa_as_alcas(&mut verts, tol);
    verts
}' '    verts
}' \
  'Q4 a alca HERDADA da solda fica por limpar'

muta "$BOO/overlap.rs" '            p1 = p3;' '            let _ = p3;' \
  'Q5 a alca que caiu na ponta de LA fica'

# ⛔ A VIRAGEM MAXIMA (o antigo Q6) SAIU com a lei dela: o vinco deixou de ser quina e
# passou a ser arco, por decisao do dono (2026-09-30) — ela nao tem mais onde morar.
# ⚠️ E Q7/Q8 deixaram de ser «volta ao limite 4»: o limite do documento VOLTOU a 4, que e' o
# valor por omissao do kurbo, e apagar o `.with_miter_limit` seria um mutante EQUIVALENTE.
# A regressao que importa e' o `10` que furava a peca.
muta "$BOO/expand.rs" '        .with_miter_limit(ph2d_vec_scene::MITER_LIMIT);' '        .with_miter_limit(10.0);' \
  'Q7 o Outline Stroke volta ao limite 10 (o que furava)'

EXTRA=ren muta "$REN/lib.rs" '        .with_miter_limit(ph2d_vec_scene::MITER_LIMIT);' '        .with_miter_limit(10.0);' \
  'Q8 o ecra volta ao limite 10 (o que furava)'

EXTRA=apv muta "$APV/svg_export.rs" '    if matches!(s.join, ph2d_vec_scene::LineJoin::Miter) {' \
  '    if false && matches!(s.join, ph2d_vec_scene::LineJoin::Miter) {' \
  'Q9 o SVG exportado perde o limite do bico'

# ── A BOLA (F41, 2026-09-30 — report do dono com tres fotos: «arredonda demais, nao e'
# progressivo, e ainda produz artefatos circulares»). O filete de tamanho fixo da F40 SAIU e as
# ancoras dele com ele: o vinco e' o arco de uma bola que rola por FORA do contorno. ────────────
muta "$BOO/overlap.rs" '    let rolado = crate::bola::rola_a_bola(base.verts.clone(), &nos_do_desenho(path), raio, solda);' \
  '    let rolado = {
        let _ = (nos_do_desenho(path), raio, solda);
        base.verts.clone()
    };' \
  'B1 a silhueta nao rola a bola'

# ⚠️ B2 SOBREVIVE e fica NOMEADA: as paralelas de FORA de uma quina convexa DIVERGEM, logo uma
# semente convexa nao acha cruzamento e a bola nao pousa — o sinal poupa a procura, nao a forma.
muta "$BOO/bola.rs" '            dth * sinal < -1e-9 && (b.p - a.p).hypot() < APERTO * raio * dth.abs()' \
  '            dth.abs() > 1e-9 && (b.p - a.p).hypot() < APERTO * raio * dth.abs()' \
  'B2 uma aresta CONVEXA tambem e semente'

muta "$BOO/bola.rs" 'const APERTO: f64 = 0.99;' 'const APERTO: f64 = 0.5;' \
  'B3 so um canto muito mais apertado que a bola e tocado'

muta "$BOO/bola.rs" 'const ALCANCE: f64 = 32.0;' 'const ALCANCE: f64 = 8.0;' \
  'B4 a janela da procura volta a 8 raios'

muta "$BOO/bola.rs" '                && vira > crate::overlap::PAREDE_MINIMA' '                && vira >= 0.0' \
  'B5 um no que a uniao deixou liso continua parede'

# ⛔ B6 (o `v > PAREDE_MINIMA` a parte) foi APAGADA com a linha: ela SOBREVIVEU porque as duas
# condicoes da viragem ACTUAL ja' a implicam (`v >= vira - 1 > 14`).

muta "$BOO/bola.rs" '            if ok && (novo - c).hypot() < raio {' \
  '            if false && ok && (novo - c).hypot() < raio {' \
  'B7 o centro fica o das cordas amostradas'

# ⚠️ B8 e B9 SOBREVIVEM e ficam NOMEADAS (F41): a poda e a fusao foram escritas contra casos
# (Z a 80 graus, a junta a 91) que o centro EXACTO e a bola VAZIA dissolveram — dois vaos da mesma
# reentrancia passaram a ser a MESMA bola, e um vao contido noutro e' saltado pelo percurso (ele
# comeca DEPOIS do de fora e o salto vai ao fim deste). Ficam como rede; nenhuma fixtura desta
# wave (fundos planos, em W, o dente) as alcanca.
muta "$BOO/bola.rs" '        if engolido {' '        if false && engolido {' \
  'B8 um vao contido noutro fica'

muta "$BOO/bola.rs" '                    let (lo, hi) = (fontes[x].0.min(fontes[y].0), fontes[x].1.max(fontes[y].1));' \
  '                    let (lo, hi) = (fontes[x].0, fontes[y].1);' \
  'B9 a fusao de dois vaos toma o comeco de um e o fim do outro'

muta "$BOO/bola.rs" '        if 2.0 * raio * (0.5 * giro.min(3.1)).tan() < solda {' \
  '        if 2.0 * raio * (0.5 * giro.min(3.1)).tan() < 2.0 * solda {' \
  'B10 o limiar do ruido volta a DUAS soldas'

muta "$BOO/bola.rs" '    let partes = ((varre / std::f64::consts::FRAC_PI_4).ceil() as usize).max(1);' \
  '    let partes = ((varre / std::f64::consts::FRAC_PI_2).ceil() as usize).max(1);' \
  'B11 o arco em pedacos de 90 graus'

muta "$BOO/bola.rs" '            return (a, tangente_do_circulo(a), ra);' '            return (a, ta, ra);' \
  'B12 o toque leva a tangente da curva e nao a do circulo'

muta "$BOO/bola.rs" '    let alca = (4.0 / 3.0) * (0.25 * passo).tan();' \
  '    let alca = (1.0 / 3.0) * (0.25 * passo).tan();' \
  'B13 o pedaco deixa de ser um arco de circulo'

muta "$BOO/bola.rs" '        let &(_, i, u, j, w, c) = candidatos.iter().find(|k| vazia(k.5))?;' \
  '        let &(_, i, u, j, w, c) = candidatos.first()?;' \
  'B14 a bola mais barata e aceite mesmo com o contorno dentro dela'

# ── O DETECTOR (F41: o passo de UM ULP lia-se como cruzamento) ──────────────
muta "$BOO/overlap.rs" '                let colar = segs.len() > n0 &&' \
  '                let colar = false && segs.len() > n0 &&' \
  'D1 um passo de um ULP no meio do contorno nasce como segmento'

muta "$BOO/overlap.rs" '        if segs.len() > n0 && (ult[0] - ini[0]).hypot(ult[1] - ini[1]) <= cola {' \
  '        if false && segs.len() > n0 && (ult[0] - ini[0]).hypot(ult[1] - ini[1]) <= cola {' \
  'D2 um fecho a um ULP do inicio nasce como segmento'

# ── O CONTROLO (nao pode sangrar) ────────────────────────────────────────
muta "$BOO/overlap.rs" '/// ⭐⭐ **O contorno cruza-se?** — os' '/// ⭐⭐ **O contorno cruza-se (controlo)?** — os' \
  'C0 CONTROLO: mudar um comentario'

echo
if [ -n "$SO_ANCORAS" ]; then
  echo "PRE-VOO: $sangram de $total ancoras casam uma vez (zero testes corridos)"
else
  echo "PLACAR: $sangram de $total sangram (a ultima e' o CONTROLO)"
fi
