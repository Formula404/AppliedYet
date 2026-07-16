use futures_util::StreamExt;

pub(crate) async fn read_response_bytes(
    response: reqwest::Response,
    maximum_bytes: usize,
    label: &str,
) -> Result<(reqwest::StatusCode, Vec<u8>), String> {
    let status = response.status();
    if response
        .content_length()
        .is_some_and(|length| length > maximum_bytes as u64)
    {
        return Err(format!("{label}响应超过大小限制"));
    }
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| format!("读取{label}响应失败: {error}"))?;
        if bytes.len().saturating_add(chunk.len()) > maximum_bytes {
            return Err(format!("{label}响应超过大小限制"));
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok((status, bytes))
}

pub(crate) async fn read_json_response(
    response: reqwest::Response,
    maximum_bytes: usize,
    label: &str,
) -> Result<(reqwest::StatusCode, serde_json::Value), String> {
    let (status, bytes) = read_response_bytes(response, maximum_bytes, label).await?;
    let value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("{label}响应不是有效 JSON: {error}"))?;
    Ok((status, value))
}
