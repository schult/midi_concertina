# MIDI Concertina - Mechanical

## Materials

|    Quantity | Part |
| ----------: | :--- |
|             | [PETG Filament](https://shop.polymaker.com/products/polymaker-petg?variant=45079221108793) |
|             | [TPU Filament](https://shop.polymaker.com/products/polyflex-tpu90?variant=39574341320761) |
|          30 | [Heat Set Insert M3 x 5.7](https://cnckitchenus.store/products/heat-set-insert-m3-x-5-7-100-pieces) |
|           3 | [M3 x 6mm Button Head Screw](https://boltdepot.com/Product-Details?product=7218) |
|           6 | [M3 x 10mm Button Head Screw](https://boltdepot.com/Product-Details?product=7220) |
|           1 | [M3 x 16mm Button Head Screw](https://boltdepot.com/Product-Details?product=7222) |
|           4 | [M3 x 20mm Button Head Screw](https://boltdepot.com/Product-Details?product=7223) |
|          12 | [M3 x 40mm Flat Head Screw](https://boltdepot.com/Product-Details?product=18839) |
|           2 | [M3 x 8mm Flat Head Screw](https://boltdepot.com/Product-Details?product=7213) |
|           2 | [M6 Finishing Washer](https://boltdepot.com/Product-Details?product=2937) |
|             | 16 Gauge Brass Wire |
|          31 | [55g Keyboard Switch Spring](https://www.amazon.com/DUROCK-Mechanical-Keyboard-Compatible-Switches/dp/B09TD7CF3H) |
|          30 | [3mm x 1mm Neodymium Magnet](https://www.amazon.com/MEALOS-Magnets-3mmx1mm-Miniatures-Storage/dp/B08NZSL2V6) |
|             | 12 Gauge Steel Hanging Wire |
|    12" x 3" | [4 oz Black Leather](https://leatherboxusa.com/products/tuscany-collection-soft-vegetable-tanned-vachetta-leather-3-5-4-0) |
|           1 | [Bivar GLP1-188-F-54 Light Pipe](https://www.digikey.com/en/products/detail/bivar-inc/GLP1-188-F-54/26730936) |
|           4 | [Würth Elektronik 6-Pin WR-WST REDFIT IDC](https://www.digikey.com/en/products/detail/w%C3%BCrth-elektronik/490107670612/7917217) |
|          3' | [6 Conductor Ribbon Cable](https://www.digikey.com/en/products/detail/3m/3365-06-300/2766998) |
|          1" | [2mm ID Silicone Tubing](https://www.dubro.com/products/super-blue-slicone-tubing?variant=39705737920596) |
| 20" x 20.5" | [Tyvek 1460R (preferred) or 1073D](https://www.outdoorpaper.com/products/tyvek-soft-structure-style-1460c-uv-fabric) |
|             | Acrylic Paint (optional) |
|             | [Liquitex Fabric Medium](https://www.dickblick.com/products/liquitex-effects-fabric-medium/) (optional) |
|   16" x 20" | [2-Ply Museum Board](https://www.dickblick.com/items/rising-museum-board-32-x-40-x-2-ply-natural-sheet/) |
|             | [PVA Glue](https://www.dickblick.com/products/gorilla-wood-glue/) |
|             | [CA Glue](https://www.dickblick.com/products/gorilla-super-glue/) |
|           1 | Controller PCBA |
|           1 | Left Keyboard PCBA |
|           1 | Right Keyboard PCBA |
|           1 | Bellows Sensor PCBA |


## 3D Printed Parts

### Parameters

Many key dimensions are stored in spreadsheets in Parameters.FCStd to be
referenced by the other FreeCAD files. A few of these may need to be adjusted to
accomodate your 3d printing setup:

- `CommonParams.clearance_tight` sets the clearance between press-fit 3d-printed
  parts. Test with ControllerPanel and ControllerPanelRetainer, since these
  are smaller parts.
- `HardwareParams.m3_insert_hole_diameter` and
  `HardwareParams.m3_insert_horizontal_hole_diameter` adjust the hole sizes
  for the threaded inserts.
- `ButtonParams.wire_press_fit_diameter` sets the inner diameter for button
  wires to press-fit into. Wires should punch through these layers without
  excessive effort and remain firmly in place once fully seated.
- `ButtonParams.wire_clearance_diameter` sets the size of the ActionBoard holes
  that the wires pass through. These should be as closely fitted as possible
  while still allowing the wires to slide freely.
- `ButtonParams.magnet_diameter` should be adjusted so that the magnets are easy
  to insert into MagnetPost mid-print but don't jump out as the nozzle passes
  over them.

When changing these or other spreadsheet values, it is recommended to open
Parameters.FSCStd first, then open all other FreeCAD files, edit the parameters,
and finally recompute the Body in each file before saving it. This ensures that
all dependent parts are correctly updated.


### STL Export

Run `freecadcmd export_all.py` in this directory. The generated STL files will
be placed in an `exports` subdirectory.


### Base Slicer Settings

- PETG
- 0.4 mm nozzle
- 0.2 mm layer height
- 15% Gyroid infill
- 2 wall loops


### Deviations

Button flanges should be printed with TPU.

Button caps should be printed with the smallest layer height possible. 0.08 mm
is good. This helps to reduce noise when pressing buttons and produces a
smoother top. It is recommended to use an outer brim when printing this part.

The magnet posts require a pause mid-print in order to insert the magnets.  They
should also be printed with the same layer height as the button caps to ensure
the wire press-fits the same way in both parts.

Handrests have a curved top that benefits from a reduced layer height of 0.12 mm
or less.

Faceplates and the controller panel should be printed with "Only one wall on
first layer" enabled.

Parts that incorporate threaded inserts should use 4 wall loops:

- Action boards
- Bellows frame
- Controller panel
- Handrest
- Valve blocks


## Assembly Instructions

### Handrest Buckle

Each handrest buckle is constructed from two lengths of 12 gauge steel hanging
wire.


#### Frame

Make marks 12mm, 63mm, 80mm, 121mm, and 136mm from the end of the wire. At the
first mark, hold the wire firmly with pliers on the side closest to the end, and
make a 90-degree bend. Repeat this for each of the following three marks, making
all bends in the same direction. Cut the wire at the final mark.


#### Prong

Round the end of the wire, then cut it to 16 mm long.


#### Assembly

With the short leg of the frame below the handrest, fit the longer leg of the
frame into the hole in the middle of the handrest. Pay attention to which side
of the handrest the leg is on (you want the right and left hands to match).
Rotate the frame until the short leg snaps into the notch in the bottom of the
handrest. Hold the frame past the prong hole while pressing the unfinished end
of the prong into place.


## License

[CERN Open Hardware Licence Version 2 - Strongly Reciprocal](LICENSE.txt)
