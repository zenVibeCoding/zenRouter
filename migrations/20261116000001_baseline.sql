-- ZenRouter 基线迁移（项目骨架）。
-- 只建立资源归属的根：企业组织。四个领域的表结构在详细设计后逐步加入，
-- 迁移优先兼容前后两个发布版本（先扩展后收缩），由单独的 migrate 任务执行。

CREATE TABLE organizations (
    id          uuid        PRIMARY KEY,
    name        text        NOT NULL CHECK (length(name) BETWEEN 1 AND 200),
    created_at  timestamptz NOT NULL DEFAULT now()
);

COMMENT ON TABLE organizations IS '企业组织；所有资源以 organization_id 作为归属根';
