#!/usr/bin/env freecadcmd

import FreeCAD
import Mesh
from pathlib import Path

export_path = Path.cwd() / 'exports'
export_path.mkdir(parents=True, exist_ok=True)

for filepath in Path.cwd().glob('*.FCStd'):
    print(f'==================== {filepath}')
    doc = FreeCAD.openDocument(str(filepath))
    doc.recompute()
    objs = [obj for obj in doc.RootObjects if obj.Label == 'Body']
    for obj in objs:
        stl_path = export_path / f'{doc.Name}.stl'
        print(f'Exporting: {stl_path}')
        Mesh.export([obj], str(stl_path))
    FreeCAD.closeDocument(doc.Name)
