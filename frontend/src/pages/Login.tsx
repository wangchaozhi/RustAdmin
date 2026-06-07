import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useAuth } from "../store/auth";

export default function Login() {
  const { login } = useAuth();
  const navigate = useNavigate();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  const submit = async () => {
    if (!username || !password) {
      setError("请输入用户名和密码");
      return;
    }
    setBusy(true);
    setError("");
    try {
      await login(username, password);
      navigate("/");
    } catch (e) {
      setError(e instanceof Error ? e.message : "登录失败");
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="login-page">
      <div className="login-card">
        <div className="brand login-brand">
          <span className="brand-mark">H</span>
          <div>
            <h1>Harbor 控制台</h1>
            <p className="muted">登录以管理你的系统</p>
          </div>
        </div>
        <label className="field">
          <span>用户名</span>
          <input value={username} autoComplete="username"
            onChange={(e) => setUsername(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && submit()} />
        </label>
        <label className="field">
          <span>密码</span>
          <input type="password" value={password} autoComplete="current-password"
            onChange={(e) => setPassword(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && submit()} />
        </label>
        {error && <p className="form-error" role="alert">{error}</p>}
        <button className="primary-btn" onClick={submit} disabled={busy}>
          {busy ? "登录中…" : "登录"}
        </button>
        <p className="muted login-hint">首次启动的默认账号:admin / Admin@12345</p>
      </div>
    </div>
  );
}
