source "${SCRIPT_ENVS_DIR}/__direnv.sh"

cat <<EOL > .env
SERVER_PORT=3000

DB_HOST=localhost
DB_PORT=5432
DB_USER=postgres
DB_PASSWORD=postgres
DB_NAME=${1:-$(basename $PWD)}

DATABASE_URL="postgresql://\${DB_USER}:\${DB_PASSWORD}@\${DB_HOST}:\${DB_PORT}/\${DB_NAME}?schema=public"
EOL
echo ".env file created successfully with DB_NAME=${db_name}."

cat "dotenv_if_exists .env" > .envrc

echo ".envrc file created successfully."
