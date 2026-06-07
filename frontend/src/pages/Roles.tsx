import { useQuery } from "@tanstack/react-query";
import { api } from "../api/client";
import type { Role } from "../api/types";
import { Empty, RoleBadge } from "../components/ui";

export default function Roles() {
  const { data, isLoading } = useQuery({
    queryKey: ["roles"],
    queryFn: () => api<Role[]>("/api/roles"),
  });

  return (
    <div className="page">
      <h1 className="page-title">角色与权限</h1>
      {isLoading ? (
        <Empty text="加载中…" />
      ) : (
        <div className="role-grid">
          {(data ?? []).map((r) => (
            <div className="card role-card" key={r.name}>
              <div className="role-card-head">
                <RoleBadge role={r.name} />
                <span className="mono muted">{r.name}</span>
              </div>
              <p>{r.description}</p>
              <div className="perm-list">
                {r.permissions.map((p) => (
                  <span className="perm mono" key={p}>{p}</span>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
