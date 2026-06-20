//! 极简 CSV 生成,带 RFC 4180 转义。

fn escape(field: &str) -> String {
    if field.contains(['"', ',', '\n', '\r']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

pub fn row(fields: &[&str]) -> String {
    fields.iter().map(|f| escape(f)).collect::<Vec<_>>().join(",")
}

/// 由表头与数据行构建完整 CSV 文本,带 UTF-8 BOM 以兼容 Excel。
pub fn build(header: &[&str], rows: impl IntoIterator<Item = Vec<String>>) -> String {
    let mut out = String::from("\u{feff}");
    out.push_str(&row(header));
    out.push_str("\r\n");
    for r in rows {
        let refs: Vec<&str> = r.iter().map(|s| s.as_str()).collect();
        out.push_str(&row(&refs));
        out.push_str("\r\n");
    }
    out
}
