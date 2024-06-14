# ETHER-KINETIC ADVENTUREGRAPH

Un device vintage con un juego de texto tipo elige tu propia aventura.

"El Aventurógrafo Éter Cinético sería una herramienta de exploración revolucionaria, mezclando lo antiguo con lo nuevo, y lo físico con lo místico, para abrir puertas a mundos y experiencias que antes solo podían ser imaginados."

Inspirado en: https://hackaday.com/2024/03/15/retro-unit-converter-is-a-neat-little-gadget/

## BRAINSTORMING NAME ##
    - Narrador
    - Narratron (ya existe)
    - Crononarrador
    - Storycoil / Narracoil
    - Electro narrador
    - Story machine
    - Ink machine
    - Line/Word/Letter machine
    - Fantasy Machine
    - Magic machine
    - Storytelling machine
    - Dream machine
    - Coil Chronicle
    - Electrofable
    - La máquina fabulosa
    - Narrador Éter-Quinético
    - Aventurógrafo éter cinético / Ether-Kinetic Adventuregraph

## FEATURES ##

- Recargable por usb-c
- Pantalla OLED. Necesita al menos 4 líneas, aunque puede tener scroll.
- Potenciómetro con botón (codificador) para moverse por la interfaz y seleccionar las opciones.
- Buzzer??
- Botón para menú??
- Botón encendido/apagado?
- Software
    - Ejecuta una historia en Ink.
    - Hecho en Rust. Usará ink-rs.
- Funciona a batería.
    - Bastará con 1 batería?
    - Circuito de corte para cargarla y alimentarse a la vez.


## PARTS ##

- ESP32-S2 (el soporte de wifi no es necesario, aunque podría ser útil para bajarse nuevas historias) con 2MB PSRAM.
- Carga batería: TP4056
- LDO
- Batería 18650
- Codificador rotatorio
- Carcasa 3D

PANTALLAS:
- Pantalla ssd1306
✓ LCD 1.8" ST7725S: ST7735S 128x160
✓ LCD 2.8" ST7789V: ST7789V  NO:ILI9341 320x240

- ENABLE DISPLAY DMA:
    https://github.com/georgik/esp-display-interface-spi-dma/tree/feature/esp-hal-0.17

Opcional:
- Buzzer
- Led?
- Circuito de carga y alimentación simultánea
- Circuito de detección de carga de batería.

## PINES ##

ESP-32:
- DISPLAY: 5,4
- ENCODER: 27,26,25
- BUTTON: 33
- BUZZER: 16

ESP-32S2:
- DISPLAY I2C: 33(SDA),35(SCL)
- DISPLAY SPI: 7(RS/DC/AO), 7(CS), 12(SCK, CLK, SCLK), 11(MISO/SDO/DOUT), 5(RST/RES/REST), 12(BACKLIGHT), 3(MOSI/SDI/DIN/SDA)
- ENCODER: 38(A), 40(B), 36(SW)
- BUTTON: 13
- BUZZER: 18

## CONSUMO

- 0.09A sin pantalla.
- 0.12-0.14 con pantalla.
- con wifi??

## SPI

RS/DC/AO: Data/command
CS: Chip select
SDA: MOSI

## MILESTONES ##

✓ Rotary encoder funcionando
- Prototipo "The Intercept"
  - Sin carga
  - Solo rotary encoder
  - Pantalla pequeña
  - Custom PCB
  - Carcasa
- Prototipo "Aventuregraph 0"
  - Custom PCB
  - Sin carga
  - Pantalla grande
  - Dos botones
  - Buzzer
  - Carcasa.
- Prototipo "Aventuregraph 1"
  - Añadir TP4056 y probar con batería.
  - Circuito de corte y carga: https://www.youtube.com/watch?v=37kGva3NW8w
- Prototipo "Aventuregraph 2"
  - Pantalla OLED
  - Circuito de info de carga de batería.
- Nueva aventura!

## BUILD AND MONITOR

ESP32S2:

```
source ../export.sh
MCU=esp32s2 cargo build --target xtensa-esp32s2-espidf
web-flash --chip esp32s2 target/xtensa-esp32-espidf/debug/aventuregraph
```