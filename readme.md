
# ONDC WEBSOCKET
 
ONDC Websocket Service

## Tech Stack
| Type | Technologies |
|---|---|
| Server | Rust (Actix-web), Bash |
| API Documention | OpenAPI Swagger |
| Messaging System |	Apache Pulsar |

## CUSTOM COMMAND FOR DEBUG:
### FOR MIGRATION:
```
cargo run --bin ondc-websocket -- migrate
```

### FOR TOKEN GENERATION:
```
cargo run --bin ondc-websocket -- generate_token  sanushilshad
```

## CUSTOM COMMAND FOR RELEASE:
### FOR MIGRATION:

    cargo run --release --bin  ondc-websocket -- migrate

    OR 

    ./target/release/ondc-websocket migrate

### FOR TOKEN GENERATION:
```
cargo run --release --bin  rapid -- generate_token  sanushilshad

OR 

./target/release/ondc-websocket generate_token  sanushilshad
```

## SQLX OFFLINE MODE:

```
cargo sqlx prepare
```

## ENVIRON VARIABLE 
- Set the following environ variables in `env.sh`
- `env.sh`:
```

## TRACE VARIABLE
export OTEL_SERVICE_NAME="ondc-websocket"
export OTEL_EXPORTER_OTLP_TRACES_ENDPOINT="http://localhost:4317"


## SECRET VARIABLE
export SECRET__JWT__SECRET=""
export SECRET__JWT__EXPIRY=876600


## APPLICATION VARIABLE
export APPLICATION__NAME=""
export APPLICATION__ACCOUNT_NAME=""
export APPLICATION__PORT=8001
export APPLICATION__HOST=0.0.0.0
export APPLICATION__WORKERS=16

## PULSAR VARIABLE
export PULSAR__TOPIC="sanu"
export PULSAR__CONSUMER="test_consumer"
export PULSAR__SUBSCRIPTION="test_subscription"
export PULSAR__URL="pulsar://localhost:6650"

```


- In order to verify SQL queries at compile time, set the below config in `.env` file:
```
export DATABASE_URL="postgres://postgres:{password}@{host}:{port}/{db_name}"

```

## TO RUN THE SERVER:
- For running development server:
```
bash dev_run.sh
```
- For running production server:
```
bash release.sh
```
- For killing server:
```
bash kill.sh
```

- For restarting server:
```
bash restart.sh
```


## API DOCUMENTATION:
The API Docmentation can be found at `https://{{domain}}/docs/` after running the server.

## DEBUG SETUP:
- launch.json
```json
{

    "version": "0.2.0",
    "configurations": [

        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug executable 'ondc-websocket'",
            "cargo": {
                "args": [
                    "build",
                    "--bin=ondc-websocket",
                    "--package=ondc-websocket"
                ],
                "filter": {
                    "name": "ondc-websocket",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}",
            "envFile": "${workspaceFolder}/.env",
            "preLaunchTask": "cargo build",
        },
    ]
}
```
- settings.json

```json
{
    "[rust]": {
        "editor.formatOnSave": true,
        "editor.defaultFormatter": "rust-lang.rust-analyzer"
    },
    "editor.formatOnSave": true,
    "rust-analyzer.linkedProjects": [
        "./Cargo.toml"
    ],
}
```

- tasks.json
```json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "cargo build",
            "type": "shell",
            "command": "cargo",
            "args": [
                "build",
                "--bin=ondc-websocket",
                "--package=ondc-websocket"
            ],
            "group": {
                "kind": "build",
                "isDefault": true
            },
            "problemMatcher": [
                "$rustc"
            ]
        }
    ]
}
```

## MILESTONES (2/2)
* [x] Move Websocket implementation as seperate service.
* [x] Integrate Pulsar.