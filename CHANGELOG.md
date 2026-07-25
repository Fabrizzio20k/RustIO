# Changelog

## [0.1.2] - 2026-07-25

### Añadido
- Flujo interactivo completo de configuración de modelos en TUI (modal `/model`) con validación de conexión en tiempo real antes de guardar credenciales y permitir chatear.
- Soporte nativo para nuevos proveedores: Anthropic y DeepSeek.
- Nuevos presets en `models.json` (`anthropic-claude-3.5-sonnet`, `deepseek-chat`, `deepseek-coder`, `local-qwen3.5-2b`).
- En la configuración manual de modelos, ahora se puede ingresar tanto la Base URL (para servidores compatibles) como la API Key de manera secuencial y opcional.

### Cambiado
- Eliminada por completo la dependencia de variables de entorno para LLMs. Ahora toda la configuración de proveedor y claves se persiste de forma segura y directa en la base de datos local SQLite.
- `run.sh` y `run.ps1` actualizados para descargar y utilizar el modelo `qwen3.5-2b-q4_k_m.gguf` (Unsloth) por defecto.
- Restringida la ejecución en `run.sh` y `run.ps1` a 1 hilo y sin GPU (`-ngl 0`, `--threads 1`) para simular el rendimiento y la latencia exactos de despliegue en una Raspberry Pi 5.
- Mejorado el manejo de errores del backend de LLM: los errores de conexión o credenciales inválidas ahora se resumen y truncan para mantener limpio el diseño del modal en el TUI.

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
