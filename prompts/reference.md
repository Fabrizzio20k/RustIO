# Referencia para proyectos Python en Raspberry Pi 5

## Entorno y paquetes con uv
- Crear el entorno virtual: `uv venv` (genera `.venv/` en el directorio actual).
- Instalar paquetes: `uv pip install <paquete> [<paquete> ...]`.
- Ejecutar un script: `.venv/bin/python <archivo.py>` o `uv run python <archivo.py>`.
- Estas tres acciones ya están disponibles como tools: `setup_venv`, `install_packages`, `run_python`. Úsalas en ese orden.

## GPIO en Raspberry Pi 5
- La librería moderna es Adafruit Blinka (paquete `Adafruit-Blinka`), que expone los módulos `board` y `digitalio`.
- En la Pi 5 el backend de GPIO es `lgpio`. Si falta soporte del sistema: `sudo apt install -y libgpiod2 python3-libgpiod`.
- Los pines se referencian como `board.D4`, que equivale a GPIO4 en numeración BCM.
- El código de GPIO solo funciona ejecutándose en la propia Raspberry con el hardware conectado; en un PC sin GPIO, `import board` fallará.

## Leer un sensor DHT11 (temperatura y humedad)

Cableado:
- VCC -> 3.3V (pin físico 1) o 5V.
- GND -> GND.
- DATA -> GPIO4 (pin físico 7), con una resistencia pull-up de 10kΩ entre DATA y VCC.

Dependencia (instálala con la tool install_packages):
- `adafruit-circuitpython-dht` (arrastra `Adafruit-Blinka` automáticamente).

Ejemplo de script `dht11.py`:

```python
import time
import board
import adafruit_dht

sensor = adafruit_dht.DHT11(board.D4, use_pulseio=False)

while True:
    try:
        temperatura_c = sensor.temperature
        humedad = sensor.humidity
        print(f"Temperatura: {temperatura_c} C | Humedad: {humedad} %")
    except RuntimeError as error:
        print(f"Lectura fallida, reintentando: {error.args[0]}")
    except Exception as error:
        sensor.exit()
        raise error
    time.sleep(2.0)
```

Reglas importantes del DHT11:
- Las lecturas fallan a menudo por timing; SIEMPRE envuélvelas en try/except y reintenta sin abortar.
- `use_pulseio=False` es más estable en Raspberry Pi.
- El DHT11 entrega valores enteros (sin decimales) y solo admite ~1 lectura cada 1-2 segundos.
- Llama a `sensor.exit()` para liberar el pin al terminar.
