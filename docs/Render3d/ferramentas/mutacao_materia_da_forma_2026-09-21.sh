#!/usr/bin/env bash
# ⭐⭐⭐ PROVA DE MUTAÇÃO — A MATÉRIA DA FORMA (2026-09-21).
#
# ⛔⛔ O report do dono, com foto e uma seta: «o algoritmo que vc criou tem esse
# fundo branco na sprite transparente. logo que roda o objeto o fundo aparece.
# OU seja: parece que vc criou uma máscara». Não era uma máscara: era a silhueta
# que o PRIMEIRO bake gravou no `base` enquanto a rota B re-rasteriza a forma
# por quadro. Cada caso abaixo apaga UM elo dessa cura e exige VERMELHO.
#
# ⚠️ As armadilhas de arnês que esta casa já pagou estão fechadas aqui:
#   * a agulha conta-se em PYTHON e tem de casar EXACTAMENTE `n` vezes — se não,
#     o caso ABORTA em vez de reportar;
#   * «não compilou» é separado de «passou» pelo grep do erro do cargo;
#   * `running N tests` com N = 0 é DEFEITO DO ARNÊS, nunca «sobreviveu» (e ele
#     CONTA os ignorados, logo os casos de placa levam `--ignored`);
#   * o restauro faz `touch`, senão o cargo guarda o build DA MUTAÇÃO.
#
#   bash docs/Render3d/ferramentas/mutacao_materia_da_forma_2026-09-21.sh
set -uo pipefail
cd "$(dirname "$0")/../../.."

vermelhos=0
verdes=0

# muta <nome> <ficheiro> <de> <para> <n-agulhas> <filtro> <pacote> [extra...]
muta() {
  local nome="$1" ficheiro="$2" de="$3" para="$4" quantas="$5" filtro="$6" pacote="$7"
  shift 7
  python3 -P - "$ficheiro" "$de" "$para" "$quantas" <<'PY' || { echo "  ⛔ ARNÊS: a agulha não casou $quantas× em $nome"; return; }
import sys, pathlib
f, de, para, q = pathlib.Path(sys.argv[1]), sys.argv[2], sys.argv[3], int(sys.argv[4])
s = f.read_text()
n = s.count(de)
assert n == q, f"a agulha casou {n} vezes, esperava {q}"
f.with_suffix(f.suffix + ".bak").write_text(s)
f.write_text(s.replace(de, para))
PY
  local saida rc
  saida="$(bash scripts/ph2d-run.sh env PH2D_GPU=1 cargo test -p "$pacote" "$filtro" "$@" 2>&1)"
  rc=$?
  mv "$ficheiro.bak" "$ficheiro"
  touch "$ficheiro"

  if echo "$saida" | grep -q 'error\[E\|could not compile'; then
    echo "  ⛔ ARNÊS: a mutação não COMPILA em $nome (lê-se como sangrar e não é)"
    return
  fi
  local correram
  correram="$(echo "$saida" | grep -oE 'running [0-9]+ tests?' \
    | grep -oE '[0-9]+' | awk '{s += $1} END {print s + 0}')"
  if [ "${correram:-0}" -eq 0 ]; then
    echo "  ⛔ ARNÊS: o filtro '$filtro' casou ZERO testes em $nome"
    return
  fi
  if [ $rc -ne 0 ]; then
    echo "  ✓ SANGRA  $nome  ($correram testes corridos)"
    vermelhos=$((vermelhos + 1))
  else
    echo "  ✗ SOBREVIVE  $nome  ($correram testes corridos)"
    verdes=$((verdes + 1))
  fi
}

PBR=crates/ph2d-form-pbr/src/imagem.rs
PASSE=crates/ph2d-form-donation/src/baked_form/passe_da_forma.rs
WGSL=crates/ph2d-form-donation/src/baked_form/passe_da_forma_wgsl.rs
BAKE=crates/ph2d-app-sculpt3d/src/bake.rs
FASE=crates/ph2d-app-sculpt3d/src/vivo_fase.rs
VIVA=crates/ph2d-form-donation/src/baked_form/forma_viva.rs
DOC=shells/desktop/src/project_baked_form.rs

echo "== A LEI, na régua em Rust =="

# (1) O albedo deixa de ser o neutro.
muta "o albedo é o NEUTRO (CPU)" "$PBR" \
  'albedo: if p.materia_da_forma {
                [1.0, 1.0, 1.0]' \
  'albedo: if false {
                [1.0, 1.0, 1.0]' 1 \
  com_a_materia_da_forma_o_albedo_e_o_neutro ph2d-form-pbr

# (2) O alfa deixa de ser a cobertura deste quadro — o report à letra.
muta "o alfa é a COBERTURA deste quadro (CPU)" "$PBR" \
  'if p.materia_da_forma {
            let cobertura' \
  'if false {
            let cobertura' 1 \
  com_a_materia_da_forma_o_alfa_e_a_cobertura ph2d-form-pbr

# (2-bis) A cobertura volta a ser aplicada DUAS vezes — a orla branca do 2.º report.
muta "a cobertura entra CHEIA (CPU)" "$PBR" \
  'cobertura: if p.materia_da_forma {
                1.0
            } else {' \
  'cobertura: if false {
                1.0
            } else {' 1 \
  a_materia_da_forma_nao_deixa_um_degrau ph2d-form-pbr

# (2-ter) O texel vazio deixa de herdar a forma do vizinho — a orla do 2.º report.
muta "o PREENCHIMENTO da borda (CPU)" "$PBR" \
  'let emprestada = (p.materia_da_forma && p.form[(i0 + j) * 4 + 3] <= 0.0)' \
  'let emprestada = (false && p.form[(i0 + j) * 4 + 3] <= 0.0)' 1 \
  a_materia_da_forma_nao_deixa_um_degrau ph2d-form-pbr

echo
echo "== O GÉMEO NA PLACA (precisa de adapter) =="

# (3) O albedo do shader volta a ler a textura.
muta "o albedo é o NEUTRO (WGSL)" "$WGSL" \
  'let albedo = select(srgb_to_linear(px.rgb), vec3<f32>(1.0), materia_e_a_forma);' \
  'let albedo = srgb_to_linear(px.rgb);' 1 \
  a_placa_e_a_regua_concordam_no_pixel ph2d-form-donation --release -- --ignored

# (4) O alfa do shader volta a atravessar o do `base`.
muta "o alfa é a COBERTURA deste quadro (WGSL)" "$WGSL" \
  'floor(clamp(cobertura_propria, 0.0, 1.0) * 255.0 + 0.5) / 255.0,' \
  'px.a,' 1 \
  a_placa_e_a_regua_concordam_no_pixel ph2d-form-donation --release -- --ignored

# (4-bis) O gémeo aplica a cobertura duas vezes.
muta "a cobertura entra CHEIA (WGSL)" "$WGSL" \
  'let cobertura = select(f.w, 1.0, materia_e_a_forma);' \
  'let cobertura = f.w;' 1 \
  a_placa_e_a_regua_concordam_no_pixel ph2d-form-donation --release -- --ignored

# (4-ter) O gémeo não preenche a borda.
muta "o PREENCHIMENTO da borda (WGSL)" "$WGSL" \
  'if (materia_e_a_forma && cobertura_propria <= 0.0) {' \
  'if (false && cobertura_propria <= 0.0) {' 1 \
  a_placa_e_a_regua_concordam_no_pixel ph2d-form-donation --release -- --ignored

# (4-quater) O anel de preenchimento entra no ALFA e a silhueta engorda.
muta "o anel NAO engorda a peca (CPU)" "$PBR" \
  'let cobertura = p.form[(i0 + j) * 4 + 3];
            px[3] = (cobertura.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;' \
  'let cobertura = if emprestada.is_some() { 1.0 } else { p.form[(i0 + j) * 4 + 3] };
            px[3] = (cobertura.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;' 1 \
  a_materia_da_forma_nao_deixa_um_degrau ph2d-form-pbr

# (5) O bit nunca sobe ao uniform — a lei existe dos dois lados e o shader nunca a vê.
muta "o bit chega ao uniform" "$PASSE" \
  'd[i + 13] = f32::from_bits(u32::from(materia_da_forma));' \
  'd[i + 13] = f32::from_bits(0u32);' 1 \
  a_placa_e_a_regua_concordam_no_pixel ph2d-form-donation --release -- --ignored

echo
echo "== A CORRENTE, do gesto até à rota B =="

# (6) O facto deixa de nascer no gesto que assa.
muta "o facto nasce no VESTIR" "$BAKE" \
  'materia_da_forma: vestidos > 0,' 'materia_da_forma: false,' 1 \
  a_materia_da_forma_atravessa ph2d-app-sculpt3d

# (7) A fase do quadro deixa de o entregar.
muta "a fase do quadro entrega-o" "$FASE" \
  'materia_da_forma: assado.materia_da_forma,' 'materia_da_forma: false,' 1 \
  a_materia_da_forma_atravessa ph2d-app-sculpt3d

# (8) A porta da rota B deixa de o passar à lei — medido no PIXEL, com placa.
muta "a rota B passa-o à lei (no PIXEL)" "$VIVA" \
  'materia_da_forma: alvo.materia_da_forma,' 'materia_da_forma: false,' 1 \
  catavento_com_materia_da_forma ph2d-app-sculpt3d --release -- --ignored

# (8-bis) A cena do catavento deixa de pedir a tela onde a lei arma.
muta "a =53 pede tela TRANSPARENTE" crates/ph2d-app-sculpt3d/src/donation.rs \
  'if catavento { 0 } else { 2 }' 'if catavento { 2 } else { 2 }' 1 \
  a_cena_do_catavento_pede ph2d-app-sculpt3d

echo
echo "== O DOCUMENTO =="

# (9) O facto não é gravado.
muta "a matéria é GRAVADA" "$DOC" \
  'materia_da_forma: bake.materia_da_forma,' 'materia_da_forma: false,' 1 \
  os_campos_autorados_viajam ph2d-editor-core --test it

# (10) O facto não é lido de volta.
muta "a matéria é LIDA de volta" "$DOC" \
  'materia_da_forma: doc.materia_da_forma,' 'materia_da_forma: false,' 1 \
  os_campos_autorados_viajam ph2d-editor-core --test it

# (11) O documento não o grava — o postcard é POSICIONAL, logo o campo tem de lá estar.
muta "a matéria atravessa o DISCO" "$DOC" \
  'materia_da_forma: true,
            // ⭐ **Um recorte' \
  'materia_da_forma: false,
            // ⭐ **Um recorte' 1 \
  a_baked_document_survives_the_disk ph2d-host-desktop

echo
echo "  ${vermelhos} sangram · ${verdes} sobrevivem"
[ "$verdes" -eq 0 ]
