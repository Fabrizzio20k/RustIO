# RustIO

![Stars](https://img.shields.io/github/stars/Fabrizzio20k/RustIO?style=flat)
![Forks](https://img.shields.io/github/forks/Fabrizzio20k/RustIO?style=flat)
![Issues](https://img.shields.io/github/issues/Fabrizzio20k/RustIO?style=flat)
![Last commit](https://img.shields.io/github/last-commit/Fabrizzio20k/RustIO?style=flat)
![Top language](https://img.shields.io/github/languages/top/Fabrizzio20k/RustIO?style=flat)
![Repo size](https://img.shields.io/github/repo-size/Fabrizzio20k/RustIO?style=flat)

Agent manager for IOT purposes

## Variables de entorno

El proyecto carga la configuración desde el entorno o desde un archivo `.env` (cargado automáticamente al iniciar con `dotenvy`).

| Variable | Requerida | Por defecto | Descripción |
|----------|-----------|-------------|-------------|
| `LLM_PROVIDER` | No | `local` | Proveedor del LLM: `local`, `groq` u `openai`. |
| `LLM_MODEL` | Sí | — | Nombre del modelo a usar. |
| `GROQ_API_KEY` | Solo si `LLM_PROVIDER=groq` | — | API key de Groq. |
| `OPENAI_API_KEY` | Solo si `LLM_PROVIDER=openai` | — | API key de OpenAI. |
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

### Ejemplo `.env` (OpenAI)

```env
LLM_PROVIDER=openai
LLM_MODEL=gpt-4o-mini
OPENAI_API_KEY=sk-...tu_api_key...
WORKSPACE_DIR=workspace
```

Modelos OpenAI disponibles: `gpt-4o`, `gpt-4o-mini`, `gpt-4-turbo`, `gpt-3.5-turbo`, entre otros.

## Entorno Python (venv con uv)

El agente gestiona automáticamente un entorno virtual de Python en `WORKSPACE_DIR/.venv` usando `uv`.
**No es necesario activarlo manualmente.** Al ejecutar la herramienta `run_python` por primera vez, el agente:

1. Crea el venv con `uv venv --python 3.13 .venv` dentro del workspace.
2. Lo activa internamente para cada proceso hijo inyectando `VIRTUAL_ENV` y el `PATH` correcto.
3. Instala dependencias con `uv pip install` cuando el modelo lo solicita.

Requisito: tener [`uv`](https://docs.astral.sh/uv/) instalado en el sistema (`pip install uv` o `curl -LsSf https://astral.sh/uv/install.sh | sh`).
