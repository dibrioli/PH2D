# Oráculo Krita (GPL3 — só CORRIDO, nunca lido): pinta casos de rampa de cor (ponta × textura)
# com a API Python do Krita (Node.paintLine com o preset ACTIVO da vista), num Xvfb privado.
import re, os, json, time, traceback
from krita import Krita, Preset, ManagedColor, InfoObject
from PyQt6.QtCore import QPoint, QRect
from PyQt6.QtGui import QColor
from PyQt6.QtWidgets import QApplication, QComboBox, QAbstractItemView
OUT = os.environ['RAMPAS_OUT']
def pump(s):
    t = time.time()
    while time.time() - t < s: QApplication.processEvents(); time.sleep(0.02)
def L(*a):
    with open(LOG, 'a') as f: f.write(' '.join(str(x) for x in a) + '\n')
app = Krita.instance(); app.setBatchmode(True)
W, H = 600, 200
STROKES = [((40, 60), (540, 60)), ((63, 140), (563, 140))]   # 2.º traço deslocado (+23, +80)
TIP_COLOR = ('<Brush type="png_brush" filename="rampa_vertical_tip.png" BrushVersion="2" spacing="0.05" '
             'useAutoSpacing="0" autoSpacingCoeff="1" angle="0" scale="1" ColorAsMask="{cam}" AdjustmentVersion="2" '
             'preserveLightness="{pl}" brushApplication="{app}" BrightnessAdjustment="0" ContrastAdjustment="0" '
             'AdjustmentMidPoint="127" AutoAdjustMidPoint="0"/>')
TIP_PLAIN = ('<Brush useAutoSpacing="0" angle="0" spacing="0.05" density="1" BrushVersion="2" type="auto_brush" '
             'randomness="0" autoSpacingCoeff="1"> <MaskGenerator spikes="2" hfade="1" ratio="1" diameter="64" '
             'id="default" type="circle" antialiasEdges="1" vfade="1"/> </Brush>')
def texture_params(enabled, mode, pattern):
    return {
        'Texture/Pattern/Enabled': 'true' if enabled else 'false',
        'Texture/Pattern/TexturingMode': str(mode),
        'Texture/Pattern/Name': pattern, 'Texture/Pattern/PatternFileName': pattern,
        'Texture/Pattern/Brightness': '0', 'Texture/Pattern/Contrast': '1',
        'Texture/Pattern/CutoffLeft': '0', 'Texture/Pattern/CutoffRight': '255', 'Texture/Pattern/CutoffPolicy': '0',
        'Texture/Pattern/Invert': 'false', 'Texture/Pattern/Scale': '1',
        'Texture/Pattern/OffsetX': '0', 'Texture/Pattern/OffsetY': '0',
        'Texture/Pattern/MaximumOffsetX': '64', 'Texture/Pattern/MaximumOffsetY': '64',
        'Texture/Pattern/isRandomOffsetX': 'false', 'Texture/Pattern/isRandomOffsetY': 'false',
        'Texture/Strength/Value': '1', 'Texture/Strength/UseCurve': 'false', 'PressureTexture/Strength/': 'false',
    }
CASES = json.loads(os.environ['RAMPAS_CASES'])
def set_param(xml, name, val):
    pat = re.compile(r'(<param[^>]*name="' + re.escape(name) + r'"[^>]*><!\[CDATA\[)(.*?)(\]\]></param>)', re.S)
    if pat.search(xml):
        return pat.sub(lambda m: m.group(1) + val + m.group(3), xml, count=1)
    return xml.replace('</Preset>', '<param type="string" name="%s"><![CDATA[%s]]></param> </Preset>' % (name, val))
def get_params(xml, prefix):
    return {m.group(1): m.group(2)[:600] for m in re.finditer(r'<param[^>]*name="([^"]*)"[^>]*><!\[CDATA\[(.*?)\]\]></param>', xml, re.S) if m.group(1).startswith(prefix)}
def combos():
    d = {}
    for wd in QApplication.allWidgets():
        if isinstance(wd, QComboBox) and wd.objectName() in ('cmbBrushMode', 'cmbTexturingMode', 'cmbCutoffPolicy'):
            d.setdefault(wd.objectName(), []).append((wd.currentText(), [wd.itemText(i) for i in range(wd.count())]))
    return d
win = app.activeWindow()
app.action('show_brush_editor').trigger(); QApplication.processEvents()
base = app.resources('preset')['b) Basic-5 Size']
BASEXML = Preset(base).toXML()   # XML PRISTINO: fromXML muda o recurso e arrastaria chaves (p.ex. md5 do padrão) entre casos
grad = app.resources('gradient')[os.environ.get('RAMPAS_GRADIENT', 'RedBlue_oracle')]
for c in CASES:
    try:
        name = c['name']
        doc = app.createDocument(W, H, name, 'RGBA', 'U8', '', 72.0)
        win.addView(doc); app.setActiveDocument(doc)
        view = win.activeView()
        root = doc.rootNode()
        layer = doc.createNode('paint', 'paintlayer'); root.addChildNode(layer, None)
        for n in root.childNodes():
            if n != layer: n.setVisible(False)
        doc.setActiveNode(layer)
        p = Preset(base)
        xml = BASEXML
        bd = TIP_PLAIN if c['tip'] == 'plain' else TIP_COLOR.format(**c.get('tipattrs', {'cam': '0', 'pl': '0', 'app': '1'}))
        xml = set_param(xml, 'brush_definition', bd)
        xml = set_param(xml, 'requiredBrushFile', '' if c['tip'] == 'plain' else 'rampa_vertical_tip.png')
        for k, v in texture_params(c['tex'] is not None, c.get('tex') or 0, c.get('pattern', 'rampa_horizontal_64.png')).items():
            xml = set_param(xml, k, v)
        xml = set_param(xml, 'SizeUseCurve', 'false'); xml = set_param(xml, 'FlowUseCurve', 'false')
        p.fromXML(xml)
        view.activateResource(base)
        view.setCurrentGradient(grad)
        view.setForeGroundColor(ManagedColor.fromQColor(QColor(0, 200, 0), None))
        view.setBackGroundColor(ManagedColor.fromQColor(QColor(255, 255, 0), None))
        view.setBrushSize(64.0); view.setPaintingOpacity(1.0); view.setPaintingFlow(1.0)
        pump(1.0)
        if c['tip'] != 'plain':
            # O «Brush mode» da ponta só pega quando escolhido no PRÓPRIO editor (o atributo no XML é ignorado na carga).
            cmb = [w for w in QApplication.allWidgets() if isinstance(w, QComboBox) and w.objectName() == 'cmbBrushMode' and w.count() == 4][0]
            cmb.setCurrentIndex(c['tipmode']); cmb.activated.emit(c['tipmode']); pump(1.5)
        cur = Preset(view.currentBrushPreset()).toXML()
        L('CASE', name, 'active=', view.currentBrushPreset().name(), 'grad=', view.currentGradient().name(),
          'size=', view.brushSize(), 'combos=', combos())
        L('  brushdef=', get_params(cur, 'brush_definition'))
        L('  tex=', {k: v for k, v in get_params(cur, 'Texture/Pattern/').items() if k.split('/')[-1] in ('Enabled', 'TexturingMode', 'Name', 'PatternFileName', 'PatternMD5Sum', 'PatternMD5')})
        for a, b in STROKES:
            layer.paintLine(QPoint(*a), QPoint(*b), 1.0, 1.0)
        doc.waitForDone(); doc.refreshProjection(); doc.waitForDone()
        raw = bytes(layer.pixelData(0, 0, W, H))
        open(f'{OUT}/{name}.bgra', 'wb').write(raw)
        ok = 'raw'
        L('  bytes', len(raw), 'nonzero_alpha', sum(1 for i in range(3, len(raw), 4) if raw[i]))
        doc.setModified(False)
    except Exception:
        L('CASE EXC', c.get('name'), traceback.format_exc())
if os.environ.get('RAMPAS_DUMP_EDITOR'):
    # Pergunta E: o que o editor de pincel EXPÕE (lista de opções + capturas das páginas pedidas).
    from PyQt6.QtWidgets import QRadioButton, QCheckBox, QLabel
    try:
        lv = None
        for wd in QApplication.allWidgets():
            if isinstance(wd, QAbstractItemView) and wd.model() is not None:
                m = wd.model()
                try: txt = [str(m.data(m.index(r, 0))) for r in range(min(m.rowCount(), 60))]
                except TypeError: continue
                if 'Texture' in txt and 'Brush Tip' in txt: lv = wd; L('EDITOR LIST', txt)
        m = lv.model()
        for page in ('Brush Tip', 'Pattern', 'Source', 'Painting Mode'):
            r = [str(m.data(m.index(i, 0))) for i in range(m.rowCount())].index(page)
            idx = m.index(r, 0); lv.setCurrentIndex(idx); lv.clicked.emit(idx); pump(1.0)
            top = lv.window(); top.grab().save(f"{OUT}/editor_{page.replace(' ', '_')}.png")
            vis = [w for w in QApplication.allWidgets() if w.isVisible() and w.window() is top]
            L('PAGE', page, 'combos=', [(w.objectName(), w.currentText(), [w.itemText(i) for i in range(w.count())]) for w in vis if isinstance(w, QComboBox) and w.count() < 30],
              'radios=', [w.text() for w in vis if isinstance(w, QRadioButton)],
              'checks=', [w.text() for w in vis if isinstance(w, QCheckBox) and w.text()])
    except Exception:
        L('EDITOR EXC', traceback.format_exc())
L('ALL DONE')
