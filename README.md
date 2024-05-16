# ETHER-KINETIC ADVENTUREGRAPH

Un device vintage con un juego de texto tipo elige tu propia aventura.

"El Aventurógrafo Éter Quinético sería una herramienta de exploración revolucionaria, mezclando lo antiguo con lo nuevo, y lo físico con lo místico, para abrir puertas a mundos y experiencias que antes solo podían ser imaginados."

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
    - Aventurógrafo éter quinético / Ether-Kinetic Adventuregraph

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

- ESP32-S2 (el soporte de wifi no es necesario, aunque podría ser útil para bajarse nuevas historias).
- Carga batería: TP4056
- LDO
- Batería 18650
- Pantalla ssd1306
- Potenciómetro (codificador)
- Carcasa 3D

Opcional:
- Buzzer
- Led?
- Circuito de carga y alimentación simultánea
- Circuito de detección de carga de batería.


## MILESTONES ##

- [✔] "Hello world" en Rust por el serial port.
- [✔] El ESP32-S2 muestra por el serial una historia Ink de una línea.
    - [✔] Usar el ESP32 básico para empezar.
- [✔] El ESP32-S2 muestra por pantalla una historia Ink de una línea.
- [✔] El ESP32-S2 se puede manejar con el codificador.
    - Historia con choice. <- **WIP**
    - https://github.com/nanoframework/nanoframework.IoT.Device/blob/develop/devices/RotaryEncoder.Esp32/README.md
- Añadir TP4056 y probar con pila.
- Diseño de carcasa.
- Software!
- Circuito de corte y carga: https://www.youtube.com/watch?v=37kGva3NW8w
- Circuito de info de carga de batería.
- Añadir buzzer.
