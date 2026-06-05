# Referencia general para proyectos Python en Raspberry Pi 5

## Entorno y paquetes con uv
- Crear el entorno virtual: `uv venv` (genera `.venv/` en el directorio actual).
- Instalar paquetes: `uv pip install <paquete> [<paquete> ...]`.
- Ejecutar un script: `.venv/bin/python <archivo.py>` o `uv run python <archivo.py>`.
- Estas tres acciones ya están disponibles como tools: `setup_venv`, `install_packages`, `run_python`. Úsalas en ese orden.
- `setup_venv` es idempotente: si el `.venv` ya existe, no lo recrea. Puedes llamarla sin miedo.

## GPIO en Raspberry Pi 5
- La librería moderna es Adafruit Blinka (paquete `Adafruit-Blinka`), que expone los módulos `board` y `digitalio`.
- En la Pi 5 el backend de GPIO es `lgpio`. Si falta soporte del sistema: `sudo apt install -y libgpiod2 python3-libgpiod`.
- Los pines se referencian como `board.D4`, que equivale a GPIO4 en numeración BCM.
- El código de GPIO solo funciona ejecutándose en la propia Raspberry con el hardware conectado; en un PC sin GPIO, `import board` fallará.

## Sensores con tool dedicada
- Para un **DHT11** (temperatura y humedad), consulta la tool `dht11_reference`.
- Para un **MQ135** (calidad del aire / gases), consulta la tool `mq135_reference`.

Cada una devuelve el cableado, las dependencias exactas y un driver de ejemplo. No inventes APIs: usa esas guías como base.
