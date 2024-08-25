# Deploy

## GHCR Token
Find the Personal Access Token and place it in the `token.txt` file.

```bash
echo "Personal Access Token" > token.txt
```

## Runtime Environment
Set the runtime environment variables in `runtime.env` file.

```bash
cp runtime.env.example runtime.env
```

## Deploy
Run the deploy script to deploy the application.

```bash
./pull-and-run.sh
```
