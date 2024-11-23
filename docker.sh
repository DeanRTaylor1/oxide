docker run --name oxide-db \
  -e POSTGRES_USER=oxide \
  -e POSTGRES_PASSWORD=oxide123 \
  -e POSTGRES_DB=oxide \
  -p 5432:5432 \
  -d postgres
