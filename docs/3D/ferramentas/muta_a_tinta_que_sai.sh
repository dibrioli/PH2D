#!/usr/bin/env bash
# muta_a_tinta_que_sai.sh — prova de mutacao da 2.ª METADE DA P5: a tinta fina
# SAI no ficheiro (um ladrilho por face, .obj + .mtl + .png).
#
# ⚠️ A POPULACAO sao TRES crates e a razao e' a lei do §20.6: a populacao de um
# arnes e' de quem OBSERVA a mutacao, nunca de quem a CONTEM. O assado vive na
# `ph2d-mesh-colors`, o escritor na `ph2d-mesh`, e a fiacao + o aviso na
# `ph2d-app-sculpt3d`.
#
# ⚠️ Auto-controlos (os quatro de sempre): ancora unica · a mutacao compila ·
# N > 0 testes correram · a corrida LIMPA verde. Mais o PRE-VOO
# (`MUTA_SO_ANCORAS=1`), que apanha uma ancora morta pelo `cargo fmt` em
# segundos e sem correr um teste.
#
# ⛔ Chame-o sempre pela porta de recursos:
#   PH2D_PRAZO=3600 bash scripts/ph2d-run.sh bash docs/3D/ferramentas/muta_a_tinta_que_sai.sh
set -u
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
COL=crates/ph2d-mesh-colors/src
MESH=crates/ph2d-mesh/src
APP=crates/ph2d-app-sculpt3d/src
BK=$(mktemp -d)
cp -r "$COL" "$BK/col"; cp -r "$MESH" "$BK/mesh"; cp -r "$APP" "$BK/app"
restore() {
  rm -rf "$COL"; cp -r "$BK/col" "$COL"
  rm -rf "$MESH"; cp -r "$BK/mesh" "$MESH"
  rm -rf "$APP"; cp -r "$BK/app" "$APP"
  # ⚠️ `cp -r` devolve o mtime ANTIGO e o cargo guarda o build DA MUTACAO.
  find "$COL" "$MESH" "$APP" -name '*.rs' -exec touch {} +
}
trap restore EXIT

FILTRO='test(/assar_tests|canto_de_uma_face|vizinho_de_uma_amostra|ponto_de_uma_face|nivel_base|recusa_nomeia|dois_lados_de_uma_aresta|byte_a_byte_o_de_sempre|vt_dele|material_nao_escurece|fine_paint|export_assado|vinte_e_tres|nao_descreve|cobertura_fica_em_casa/)'
corrida() {
  cargo nextest run -p ph2d-mesh-colors -p ph2d-mesh -p ph2d-app-sculpt3d -E "$FILTRO" 2>&1
}
populacao() { grep -oP '\K[0-9]+(?= tests? run)' | awk '{s+=$1}END{print s+0}'; }

if [ -z "$SO_ANCORAS" ]; then
  limpa=$(corrida); rc_limpo=$?
  verde=$(printf '%s' "$limpa" | populacao)
  echo "VERDE antes: $verde testes correram"
  [ "${verde:-0}" -gt 0 ] || { echo "ABORTO: a corrida limpa nao correu teste nenhum"; exit 2; }
  if [ "$rc_limpo" -ne 0 ]; then
    echo "ABORTO: a corrida limpa esta' VERMELHA -- um placar daqui e' fabricado."
    printf '%s' "$limpa" | grep -E '^ *(FAIL|Summary)' | tail -6 | sed 's/^/      | /'
    exit 2
  fi
fi

sangram=0; total=0
muta() { # ficheiro  ancora  substituto  nome
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
    printf '%s' "$out" | grep -E '^error' | head -3 | sed 's/^/      | /'
  else
    corridos=$(printf '%s' "$out" | populacao)
    if [ "$corridos" -eq 0 ]; then
      echo "  ABORTO [$nome]: zero testes correram — ultimas linhas:"
      printf '%s' "$out" | tail -6 | sed 's/^/      | /'
    elif [ $rc -ne 0 ]; then echo "  SANGRA  [$nome]"; sangram=$((sangram+1))
    else echo "  SOBREVIVE [$nome]  <<<<"; fi
  fi
  restore
}

# ── O ASSADO: a disposicao do ladrilho ──────────────────────────────────
# ⚠️ O canto `1` de um triangulo e' `(j = L)` e o `2` e' `(k = L)` — trocar os
#    dois ESPELHA meia peca e nada no ecra acusa.
muta "$COL/assar.rs" \
  '            for (u, v) in [(0, 0), (l, 0), (0, l)] {' \
  '            for (u, v) in [(0, 0), (0, l), (l, 0)] {' \
  'A1 os cantos 1 e 2 de um triangulo trocam: meia peca espelhada'

# ── A INVERSAO do eixo `v`, que vive num sitio so' ──────────────────────
muta "$COL/assar.rs" \
  '        1.0 - (y as f32 + 0.5) / s,' \
  '        (y as f32 + 0.5) / s,' \
  'A2 o eixo v deixa de ser invertido: a textura sai de cabeca para baixo'

# ── A DILATACAO ─────────────────────────────────────────────────────────
muta "$COL/assar.rs" \
  '    dilata(&mut rgba, &mut coberto, lado_px, FOLGA_EM_TEXELS);' \
  '    dilata(&mut rgba, &mut coberto, lado_px, 0);' \
  'A3 a dilatacao nao corre: a borda de toda face sai com uma linha escura'

# ── A FOLGA some do ladrilho ────────────────────────────────────────────
# ⚠️ A ancora era `let ladrilho = l + 1 + 2 * FOLGA_EM_TEXELS;` e MORREU com
#    o empacotador (22/09), que passou a medir um ladrilho POR FACE. Apanhada
#    pelo pre-voo em 23/09 — a segunda deste ficheiro no mesmo dia.
muta "$COL/assar.rs" \
  '        .map(|f| tinta.lado_da_face(f) + 1 + 2 * FOLGA_EM_TEXELS)' \
  '        .map(|f| tinta.lado_da_face(f) + 1)' \
  'A4 os ladrilhos encostam: a bilinear apanha o vizinho'

# ── A RECUSA deixa de armar ─────────────────────────────────────────────
# ⚠️ A ancora era `    if lado_px > tecto_px {` e MORREU no dia em que o
#    empacotador nasceu (22/09) — a busca do lado passou a viver no `empacota`.
#    O pre-voo apanhou-a em 23/09, em segundos e sem correr um teste; ate' la'
#    esta mutacao contava para o placar e nao entrava em ficheiro nenhum.
muta "$COL/assar.rs" \
  '        if w > tecto_px {' \
  '        if false {' \
  'A5 a textura deixa de ter tecto e o destino recusa o ficheiro'

# ── O ESCRITOR: o acumulador de `vt` passa a ser o dos vertices ─────────
# ⚠️ E' o defeito que desloca a tinta de todas as pecas a seguir a' primeira
#    sem textura, com o ficheiro a abrir sem queixa.
muta "$MESH/export.rs" \
  '        base_uv += uv.map_or(0, |u| u.uv.len());' \
  '        base_uv = base;' \
  'A6 o indice de vt passa a seguir o dos vertices: tinta no sitio errado'

muta "$MESH/export.rs" \
  '                    out.push_str(&format!("/{}", base_uv + k + 1));' \
  '                    out.push_str(&format!("/{}", base_uv + k));' \
  'A7 o indice de vt deixa de ser 1-based'

muta "$MESH/export.rs" \
  '            "newmtl {nome}\nKd 1 1 1\nd 1\nillum 1\nmap_Kd {png}\n"' \
  '            "newmtl {nome}\nKd 1 1 1\nd 1\nillum 1\nmap_Kd ./{png}\n"' \
  'A8 o material passa a levar um caminho e quebra no destino'

# ── A TABELA do formato ─────────────────────────────────────────────────
muta "$MESH/export.rs" \
  '    pub fn keeps_fine_paint(self) -> bool {
        matches!(self, Self::Obj)' \
  '    pub fn keeps_fine_paint(self) -> bool {
        false' \
  'A9 o OBJ volta a dizer que nao carrega a tinta fina'

# ── O AVISO: ter tinta fina e PERDE-LA sao perguntas diferentes ─────────
muta "$APP/export_assado.rs" \
  '    tem && (!fmt.keeps_fine_paint() || !assados.alguma())' \
  '    tem && !fmt.keeps_fine_paint()' \
  'A10 uma peca RECUSADA por tamanho perde a tinta e o aviso cala-se'

# ── A FIACAO: a saida deixa de assar ────────────────────────────────────
muta "$APP/export.rs" \
  '        export_assado::assa(scene)' \
  '        export_assado::Assados::default()' \
  'A11 a saida deixa de ASSAR: o obj sai sem textura, calado'

muta "$APP/export.rs" \
  '        Some(m) => ph2d_mesh::write_obj_com_uv(&pieces, &uvs, m).into_bytes(),' \
  '        Some(_) => fmt.write(&pieces),' \
  'A12 o obj volta ao escritor sem uv e o material fica orfao'

# ── A GUARDA da §14, que o assado nasceu SEM ────────────────────────────
# ⚠️ Sem ela uma malha com mais faces do que o plano conhece ESTOURA no meio de
#    uma exportacao (o report de 21/09), e uma com menos sai com tinta valida no
#    sitio errado, em silencio.
muta "$COL/assar.rs" \
  '    if !tinta' \
  '    if false && !tinta' \
  'A14 o assado deixa de conferir se o plano descreve a malha'

# ── A COBERTURA nao pode sair no ficheiro ───────────────────────────────
muta "$COL/assar.rs" \
  '            out.extend_from_slice(&p[..3]);' \
  '            out.extend_from_slice(&p[..3].iter().rev().copied().collect::<Vec<_>>());' \
  'A15 o rgb troca a ordem dos canais: a tinta sai com a cor errada'

# ── O CONTROLO INERTE ───────────────────────────────────────────────────
muta "$COL/assar.rs" \
  'pub fn assar<' \
  '
pub fn assar<' \
  'A16 CONTROLO: uma mutacao INERTE (uma linha em branco) nao pode sangrar'

echo
if [ -n "$SO_ANCORAS" ]; then
  echo "PRE-VOO: $sangram de $total ancoras casam exactamente uma vez (ZERO testes corridos)"
  [ "$sangram" -eq "$total" ]
  exit $?
fi
echo "PLACAR DA TINTA QUE SAI: $sangram de $total sangram (o A16 e' o CONTROLO e nao pode)"
[ "$sangram" -eq $((total - 1)) ]
