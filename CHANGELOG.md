# Changelog

## [0.1.1] - 2026-06-21

### Añadido
- Ejecución de código: herramientas `run_python` (venv gestionado con uv, Python 3.13) y `run_shell`, con autocorrección por multi-turn.
- Persistencia en SQLite de la conversación y el resumen; sobrevive a reinicios.
- Comandos en el chat: `/help`, `/resume`, `/clear`, con autocompletado al escribir `/`.
- Render de markdown en las respuestas: bloques de código, encabezados, código en línea y separadores.
- Selección de texto con el mouse, autoscroll continuo al arrastrar y copiado al portapapeles; `Ctrl+C` copia la selección.
- Indicador de tokens por segundo.
- Línea de actividad en gris con el comando ejecutado y su resultado.
- System prompt consciente del sistema operativo y orientado a usar las herramientas antes de negarse.

### Cambiado
- `run.ps1` y `run.sh`: flags de `llama-server` (`--jinja`, `-fa on`, KV cache `q8_0`, `--mlock`, `--cont-batching`).

### Corregido
- Rutas del workspace ahora absolutas; el script dejó de buscarse en una ruta duplicada.
- venv activado (`VIRTUAL_ENV` + `PATH`) para que `uv pip install` y la ejecución funcionen.
- `Ctrl+C` ya no cierra la app cuando hay texto seleccionado.
