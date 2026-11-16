"""Adapter 的 ASGI 应用。

骨架阶段只提供健康检查与契约版本协商；单次尝试接口（/v1/attempts）在薄调用链中实现。
"""

from fastapi import FastAPI, Header
from fastapi.responses import JSONResponse

from . import CONTRACT_VERSION, __version__

CONTRACT_HEADER = "x-zenrouter-adapter-contract"


def create_app() -> FastAPI:
    app = FastAPI(
        title="ZenRouter Adapter",
        version=__version__,
        # 内部服务不暴露交互式文档。
        docs_url=None,
        redoc_url=None,
        openapi_url=None,
    )

    @app.get("/healthz")
    async def healthz(
        contract: str | None = Header(default=None, alias=CONTRACT_HEADER),
    ) -> JSONResponse:
        body = {
            "status": "ok",
            "contract_version": CONTRACT_VERSION,
            "version": __version__,
        }
        # 调用方声明的契约版本不一致时仍返回 200，由 Gateway 判断是否可用；
        # 这里只额外给出提示字段，便于排查。
        if contract is not None and contract != CONTRACT_VERSION:
            body["contract_mismatch"] = True
        return JSONResponse(body)

    @app.post("/v1/attempts")
    async def attempts() -> JSONResponse:
        return JSONResponse(
            status_code=501,
            content={"error": {"code": "not_implemented", "message": "单次尝试接口尚未实现"}},
        )

    return app
