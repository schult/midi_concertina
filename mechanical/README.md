# MIDI Concertina - Mechanical

## Materials

| Part                                              | Quantity    | Source |
| :------------------------------------------------ | ----------: | :----- |
| PETG Filament                                     |             | https://shop.polymaker.com/products/polymaker-petg?variant=45079221108793 |
| TPU Filament                                      |             | https://shop.polymaker.com/products/polyflex-tpu90?variant=39574341320761 |
| Heat Set Insert M3 x 5.7                          |          30 | https://cnckitchenus.store/products/heat-set-insert-m3-x-5-7-100-pieces |
| M3 x 6mm Button Head Screw                        |           3 | https://boltdepot.com/Product-Details?product=7218 |
| M3 x 10mm Button Head Screw                       |           6 | https://boltdepot.com/Product-Details?product=7220 |
| M3 x 16mm Button Head Screw                       |           1 | https://boltdepot.com/Product-Details?product=7222 |
| M3 x 20mm Button Head Screw                       |           4 | https://boltdepot.com/Product-Details?product=7223 |
| M3 x 40mm Flat Head Screw                         |          12 | https://boltdepot.com/Product-Details?product=18839 |
| M3 x 8mm Flat Head Screw                          |           2 | https://boltdepot.com/Product-Details?product=7213 |
| M6 Finishing Washer                               |           2 | https://boltdepot.com/Product-Details?product=2937 |
| 16 Gauge Brass Wire                               |             |  |
| 55g Keyboard Switch Spring                        |          31 | https://www.amazon.com/DUROCK-Mechanical-Keyboard-Compatible-Switches/dp/B09TD7CF3H |
| 3mm x 1mm Neodymium Magnet                        |          30 | https://www.amazon.com/MEALOS-Magnets-3mmx1mm-Miniatures-Storage/dp/B08NZSL2V6 |
| 12 Gauge Steel Hanging Wire                       |             |  |
| ~4 oz Black Leather,                              |    12" x 3" | https://leatherboxusa.com/products/tuscany-collection-soft-vegetable-tanned-vachetta-leather-3-5-4-0 |
| Bivar GLP1-188-F-54 Light Pipe                    |           1 | https://www.digikey.com/en/products/detail/bivar-inc/GLP1-188-F-54/26730936 |
| Würth Elektronik 6-Pin WR-WST REDFIT IDC          |           4 | https://www.digikey.com/en/products/detail/w%C3%BCrth-elektronik/490107670612/7917217 |
| 6 Conductor Ribbon Cable                          |          3' | https://www.digikey.com/en/products/detail/3m/3365-06-300/2766998 |
| 2mm ID Silicone Tubing                            |          1" | https://www.dubro.com/products/super-blue-slicone-tubing?variant=39705737920596 |
| Tyvek 1460R (preferred) or 1073D                  | 20" x 20.5" | https://www.outdoorpaper.com/products/tyvek-soft-structure-style-1460c-uv-fabric |
| Acrylic Paint (optional)                          |             |  |
| Liquitex Fabric Medium (optional)                 |             | https://www.dickblick.com/products/liquitex-effects-fabric-medium/ |
| 2-Ply Museum Board                                |   16" x 20" | https://www.dickblick.com/items/rising-museum-board-32-x-40-x-2-ply-natural-sheet/ |
| PVA Glue                                          |             | https://www.dickblick.com/products/gorilla-wood-glue/ |
| CA Glue                                           |             | https://www.dickblick.com/products/gorilla-super-glue/ |
| Controller PCBA                                   |           1 | https://jlcpcb.com/ |
| Left Keyboard PCBA                                |           1 | https://jlcpcb.com/ |
| Right Keyboard PCBA                               |           1 | https://jlcpcb.com/ |
| Bellows Sensor PCBA                               |           1 | https://jlcpcb.com/ |


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


## License

[CERN Open Hardware Licence Version 2 - Strongly Reciprocal](LICENSE.txt)
