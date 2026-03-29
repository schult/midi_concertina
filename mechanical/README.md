# MIDI Concertina - Mechanical

## BOM

Screws, threaded inserts, wire, magnets...

Light Pipe: Bivar GLP1-188-F-54

## Parameters

- Changes for printing
- Customization?


## 3D Printing

### STL Export

Run `freecadcmd export_all.py` in this directory. The generated STL files will
be placed in an `exports` subdirectory.

### Base Settings

- PETG
- 0.4 mm nozzle
- 0.2 mm layer height
- 15% Gyroid infill
- 2 wall loops

### Deviations

Button flanges should be printed with TPU.

Button caps should be printed with the smallest layer height possible. 0.8 mm is
good. This helps to reduce noise when pressing buttons and produces a smoother
top. It is recommended to use an outer brim when printing this part.

Handrests have a curved top that benefits from a reduced layer height of 0.12 mm
or less.

Faceplates should be printed with "Only one wall on first layer" enabled.

Parts that incorporate threaded inserts should use 4 wall loops:

- Action boards
- Bellows frame
- Controller panel
- Handrest
- Valve blocks


## Assembly Instructions


## License

[CERN Open Hardware Licence Version 2 - Strongly Reciprocal](LICENSE.txt)
