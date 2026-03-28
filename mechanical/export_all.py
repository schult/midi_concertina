#!/usr/bin/env freecadcmd

import FreeCAD
import Mesh
from pathlib import Path


def main():
    export_path = Path.cwd() / 'exports'
    export_path.mkdir(parents=True, exist_ok=True)

    for file_path in Path.cwd().glob('*.FCStd'):
        export_file(file_path, export_path)


def export_file(file_path: Path, export_path: Path):
    print(f'==================== {file_path}')
    main_doc = FreeCAD.openDocument(str(file_path))
    open_docs = main_doc.getDependentDocuments()
    for doc in open_docs:
        if doc.Partial:
            doc.restore()

    export_doc(main_doc, export_path)

    for doc in open_docs:
        FreeCAD.closeDocument(doc.Name)


def export_doc(doc: FreeCAD.Document, export_path: Path):
    for obj in doc.RootObjects:
        if obj.TypeId == 'PartDesign::Body' and obj.Label == 'Body':
            stl_path = export_path / f'{doc.Name}.stl'
            print(f'Exporting: {stl_path}')
            Mesh.export([obj], str(stl_path))
            return


main()
