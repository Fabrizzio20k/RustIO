# RustIO

![Stars](https://img.shields.io/github/stars/Fabrizzio20k/RustIO?style=flat)
![Forks](https://img.shields.io/github/forks/Fabrizzio20k/RustIO?style=flat)
![Issues](https://img.shields.io/github/issues/Fabrizzio20k/RustIO?style=flat)
![Last commit](https://img.shields.io/github/last-commit/Fabrizzio20k/RustIO?style=flat)
![Top language](https://img.shields.io/github/languages/top/Fabrizzio20k/RustIO?style=flat)
![Repo size](https://img.shields.io/github/repo-size/Fabrizzio20k/RustIO?style=flat)

Agent manager for IOT purposes

## Variables de entorno

El proyecto carga la configuración desde el entorno o desde un archivo `.env` (ver `src/config.rs`).

| Variable | Requerida | Por defecto | Descripción |
|----------|-----------|-------------|-------------|
| `LLM_PROVIDER` | No | `local` | Proveedor del LLM: `local` o `groq`. |
| `LLM_MODEL` | Sí | — | Nombre del modelo a usar. |
| `GROQ_API_KEY` | Solo si `LLM_PROVIDER=groq` | — | API key de Groq. |
| `LOCAL_BASE_URL` | No | `http://localhost:8080/v1` | URL base del servidor local (p. ej. `llama-server`). Solo se usa con `LLM_PROVIDER=local`. |
| `HISTORY_BUDGET_TOKENS` | No | `3000` | Tokens de historial antes de comprimir con resumen rodante. |
| `WORKSPACE_DIR` | No | `workspace` | Directorio de trabajo del agente (venv de uv y scripts). |

### Ejemplo `.env` (servidor local)

```env
LLM_PROVIDER=local
LLM_MODEL=qwen3-4b-instruct-2507-q4_k_m
LOCAL_BASE_URL=http://localhost:8080/v1
WORKSPACE_DIR=workspace
```

### Ejemplo `.env` (Groq)

```env
LLM_PROVIDER=groq
LLM_MODEL=llama-3.3-70b-versatile
GROQ_API_KEY=tu_api_key
WORKSPACE_DIR=workspace
```
