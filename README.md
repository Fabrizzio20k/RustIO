# RustIO
Agent manager for IOT purposes

## Variables de entorno

El proyecto carga la configuración desde el entorno o desde un archivo `.env` (ver `src/config.rs`).

| Variable | Requerida | Por defecto | Descripción |
|----------|-----------|-------------|-------------|
| `LLM_PROVIDER` | No | `local` | Proveedor del LLM: `local` o `groq`. |
| `LLM_MODEL` | Sí | — | Nombre del modelo a usar. |
| `GROQ_API_KEY` | Solo si `LLM_PROVIDER=groq` | — | API key de Groq. |
| `LOCAL_BASE_URL` | No | `http://localhost:8080/v1` | URL base del servidor local (p. ej. `llama-server`). Solo se usa con `LLM_PROVIDER=local`. |
| `PROMPTS_DIR` | No | `prompts` | Directorio con los prompts. |
| `WORKSPACE_DIR` | No | `workspace` | Directorio de trabajo del agente. |

### Ejemplo `.env` (servidor local)

```env
LLM_PROVIDER=local
LLM_MODEL=qwen2.5-coder-7b-instruct-q4_k_m
LOCAL_BASE_URL=http://localhost:8080/v1
PROMPTS_DIR=prompts
WORKSPACE_DIR=workspace
```

### Ejemplo `.env` (Groq)

```env
LLM_PROVIDER=groq
LLM_MODEL=llama-3.3-70b-versatile
GROQ_API_KEY=tu_api_key
PROMPTS_DIR=prompts
WORKSPACE_DIR=workspace
```
