# deploy

| 形态 | 入口 | 状态 |
| --- | --- | --- |
| 本地开发 | 根目录 `make dev`，依赖由 [compose.deps.yaml](compose/compose.deps.yaml) 提供 | 骨架可用 |
| Docker 单应用容器 | [Dockerfile](docker/Dockerfile)，连接既有 PostgreSQL | 骨架可用，未做多架构验证 |
| Docker Compose（默认安装） | [compose.yaml](compose/compose.yaml) | 骨架可用 |
| 集群（多主机 Docker / Helm） | 待建 | 未开始，见架构文档第 10.4、10.6 节 |

## 单应用容器

~~~sh
docker build -f deploy/docker/Dockerfile -t zenrouter:dev .
docker run --rm zenrouter:dev migrate   # 需先设置 ZENROUTER__DATABASE__URL
docker run -d -p 8080:8080 -p 8081:8081 \
  -e ZENROUTER__DATABASE__URL='postgres://user:pass@db-host:5432/zenrouter' \
  zenrouter:dev
~~~

启用正文留存时追加 `-e ZENROUTER__CONTENT__BACKEND=file -v <获准目录>:/var/lib/zenrouter/content`。
容器以 UID 10001 运行，挂载目录需对该用户可写。

## Compose

~~~sh
cp deploy/compose/.env.example deploy/compose/.env   # 修改 POSTGRES_PASSWORD
docker compose -f deploy/compose/compose.yaml up -d --build
docker compose -f deploy/compose/compose.yaml --profile valkey up -d   # 同时启动 Valkey
~~~

`migrate` 是一次性任务，完成后应用才启动；应用的健康检查使用调用面的 `/readyz`。
