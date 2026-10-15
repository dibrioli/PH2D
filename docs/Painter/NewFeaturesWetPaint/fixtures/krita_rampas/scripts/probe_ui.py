# Sonda 2: escolher «Brush mode» pela UI (combo), bombear eventos, pintar e ler os píxeis.
import re, time, traceback
from krita import Krita, Preset, ManagedColor
from PyQt6.QtCore import QPoint
from PyQt6.QtGui import QColor
from PyQt6.QtWidgets import QApplication, QComboBox
def L(*a):
    with open(LOG, 'a') as f: f.write(' '.join(str(x) for x in a) + '\n')
def pump(s):
    t = time.time()
    while time.time() - t < s: QApplication.processEvents(); time.sleep(0.02)
app = Krita.instance(); app.setBatchmode(True)
win = app.activeWindow()
doc = app.createDocument(600, 200, 'probe', 'RGBA', 'U8', '', 72.0); win.addView(doc); app.setActiveDocument(doc)
view = win.activeView()
root = doc.rootNode(); layer = doc.createNode('paint', 'paintlayer'); root.addChildNode(layer, None)
for n in root.childNodes():
    if n != layer: n.setVisible(False)
doc.setActiveNode(layer)
app.action('show_brush_editor').trigger(); pump(1)
base = app.resources('preset')['b) Basic-5 Size']
p = Preset(base); xml = p.toXML()
bd = ('<Brush type="png_brush" filename="rampa_vertical_tip.png" BrushVersion="2" spacing="0.05" useAutoSpacing="0" '
      'autoSpacingCoeff="1" angle="0" scale="1" ColorAsMask="0" preserveLightness="0" AdjustmentVersion="2" BrightnessAdjustment="0" '
      'ContrastAdjustment="0" AdjustmentMidPoint="127"/>')
xml = re.sub(r'(name="brush_definition"[^>]*><!\[CDATA\[)(.*?)(\]\]>)', lambda m: m.group(1) + bd + m.group(3), xml, count=1, flags=re.S)
xml = re.sub(r'(name="SizeUseCurve"[^>]*><!\[CDATA\[)(.*?)(\]\]>)', lambda m: m.group(1) + 'false' + m.group(3), xml, count=1, flags=re.S)
p.fromXML(xml); view.activateResource(base); pump(1)
view.setCurrentGradient(app.resources('gradient')['RedBlue_oracle'])
view.setForeGroundColor(ManagedColor.fromQColor(QColor(0, 200, 0), None)); view.setBrushSize(64.0)
cmb = [w for w in QApplication.allWidgets() if isinstance(w, QComboBox) and w.objectName() == 'cmbBrushMode' and w.count() == 4][0]
def bdef():
    x = Preset(view.currentBrushPreset()).toXML()
    return re.search(r'name="brush_definition"[^>]*><!\[CDATA\[(.*?)\]\]>', x, re.S).group(1)
for i, y in ((3, 60), (2, 140)):
    cmb.setCurrentIndex(i); cmb.activated.emit(i); pump(1.5)
    L('set', i, cmb.currentText(), bdef())
    layer.paintLine(QPoint(40, y), QPoint(540, y), 1.0, 1.0); doc.waitForDone(); pump(0.5)
    raw = bytes(layer.pixelData(0, y - 32, 600, 64))
    L('  px', [tuple(raw[(r * 600 + 300) * 4:(r * 600 + 300) * 4 + 4]) for r in (4, 20, 36, 52)])
L('ALL DONE')
