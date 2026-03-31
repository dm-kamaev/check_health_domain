# Check health domain

## Set env

`.vscode/settings.json`

```sh
{
    "rust-analyzer.cargo.extraEnv": {
        "USERP": "****************"
    }
}
```

## Dev run

```sh
make run
```

## Test

```sh
make test
```

## Deploy

```sh
TARGET=user@host make deploy
```
