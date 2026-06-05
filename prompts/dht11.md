# Sensor DHT11 (temperatura y humedad) en Raspberry Pi 5

El DHT11 es un sensor DIGITAL de un solo hilo de datos. Entrega temperatura y
humedad como valores enteros (sin decimales) y admite ~1 lectura cada 1-2 s.

## Cableado
- VCC  -> 3.3V (pin físico 1) o 5V.
- GND  -> GND.
- DATA -> GPIO4 (pin físico 7), con una resistencia pull-up de 10kΩ entre DATA y VCC.

## Dependencia (instálala con la tool install_packages)
- `adafruit-circuitpython-dht` (arrastra `Adafruit-Blinka` automáticamente).

## Ejemplo de script `dht11.py`

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

## Reglas importantes del DHT11
- Las lecturas fallan a menudo por timing; SIEMPRE envuélvelas en try/except y reintenta sin abortar.
- `use_pulseio=False` es más estable en Raspberry Pi.
- El DHT11 entrega valores enteros y solo admite ~1 lectura cada 1-2 segundos.
- Llama a `sensor.exit()` para liberar el pin al terminar.
