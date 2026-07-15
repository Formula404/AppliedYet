use reqwest::{header, redirect::Policy, Client, StatusCode};
use std::{
    collections::HashSet,
    net::{IpAddr, SocketAddr, ToSocketAddrs},
    time::Duration,
};
use url::Url;

const MAX_PAGE_BYTES: usize = 3 * 1024 * 1024;
const MAX_REDIRECTS: usize = 5;
const MAX_QUESTIONS: usize = 100;

pub struct ExtractedExperience {
    pub title: Option<String>,
    pub questions: Vec<String>,
}

pub async fn fetch_and_extract(value: &str) -> Result<ExtractedExperience, String> {
    let mut url = validate_url(value)?;
    for redirect_count in 0..=MAX_REDIRECTS {
        let client = client_for_url(&url)?;
        let mut response = client
            .get(url.clone())
            .header(
                header::ACCEPT,
                "text/html,application/xhtml+xml;q=0.9,text/plain;q=0.8",
            )
            .send()
            .await
            .map_err(|error| format!("无法访问网页：{error}"))?;

        if response.status().is_redirection() {
            if redirect_count == MAX_REDIRECTS {
                return Err("网页重定向次数过多".into());
            }
            let location = response
                .headers()
                .get(header::LOCATION)
                .and_then(|value| value.to_str().ok())
                .ok_or_else(|| "网页返回了无效的重定向地址".to_string())?;
            url = validate_url(
                url.join(location)
                    .map_err(|_| "网页重定向地址无效")?
                    .as_str(),
            )?;
            continue;
        }

        if response.status() != StatusCode::OK {
            return Err(format!(
                "网页访问失败（HTTP {}）",
                response.status().as_u16()
            ));
        }
        if let Some(length) = response.content_length() {
            if length > MAX_PAGE_BYTES as u64 {
                return Err("网页内容过大，无法安全分析（上限 3 MB）".into());
            }
        }
        if let Some(content_type) = response.headers().get(header::CONTENT_TYPE) {
            let content_type = content_type
                .to_str()
                .unwrap_or_default()
                .to_ascii_lowercase();
            if !content_type.contains("text/html")
                && !content_type.contains("application/xhtml+xml")
                && !content_type.contains("text/plain")
            {
                return Err("链接返回的不是可分析的网页正文".into());
            }
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|error| format!("读取网页失败：{error}"))?
        {
            if bytes.len() + chunk.len() > MAX_PAGE_BYTES {
                return Err("网页内容过大，无法安全分析（上限 3 MB）".into());
            }
            bytes.extend_from_slice(&chunk);
        }
        let html = String::from_utf8_lossy(&bytes);
        return Ok(extract_from_html(&html));
    }
    Err("网页重定向次数过多".into())
}

fn client_for_url(url: &Url) -> Result<Client, String> {
    let host = url.host_str().ok_or_else(|| "链接缺少域名".to_string())?;
    let port = url
        .port_or_known_default()
        .ok_or_else(|| "链接端口无效".to_string())?;
    let addresses: Vec<SocketAddr> = (host, port)
        .to_socket_addrs()
        .map_err(|_| "无法解析网页域名".to_string())?
        .collect();
    if addresses.is_empty() {
        return Err("无法解析网页域名".into());
    }
    if addresses.iter().any(|address| !is_public_ip(address.ip())) {
        return Err("为保护本机数据，不能导入局域网或本机地址".into());
    }

    let mut builder = Client::builder()
        .redirect(Policy::none())
        .timeout(Duration::from_secs(18))
        .user_agent("Mozilla/5.0 AppliedYet/0.1 InterviewExperienceImporter");
    if url
        .host()
        .is_some_and(|value| matches!(value, url::Host::Domain(_)))
    {
        builder = builder.resolve(host, addresses[0]);
    }
    builder
        .build()
        .map_err(|error| format!("无法创建网页请求：{error}"))
}

fn validate_url(value: &str) -> Result<Url, String> {
    let url = Url::parse(value).map_err(|_| "请输入有效的网页地址".to_string())?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err("只支持公开的 http:// 或 https:// 网页".into());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("链接不能包含用户名或密码".into());
    }
    let host = url
        .host_str()
        .unwrap_or_default()
        .trim_end_matches('.')
        .to_ascii_lowercase();
    if host == "localhost" || host.ends_with(".localhost") || host.ends_with(".local") {
        return Err("为保护本机数据，不能导入局域网或本机地址".into());
    }
    let literal_ip = match url.host() {
        Some(url::Host::Ipv4(ip)) => Some(IpAddr::V4(ip)),
        Some(url::Host::Ipv6(ip)) => Some(IpAddr::V6(ip)),
        _ => None,
    };
    if literal_ip.is_some_and(|ip| !is_public_ip(ip)) {
        return Err("为保护本机数据，不能导入局域网或本机地址".into());
    }
    Ok(url)
}

fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let [a, b, c, _] = ip.octets();
            !(a == 0
                || a == 10
                || a == 127
                || (a == 100 && (64..=127).contains(&b))
                || (a == 169 && b == 254)
                || (a == 172 && (16..=31).contains(&b))
                || (a == 192 && b == 168)
                || (a == 192 && b == 0 && c == 0)
                || (a == 192 && b == 0 && c == 2)
                || (a == 198 && (b == 18 || b == 19))
                || (a == 198 && b == 51 && c == 100)
                || (a == 203 && b == 0 && c == 113)
                || a >= 224)
        }
        IpAddr::V6(ip) => {
            if let Some(ipv4) = ip.to_ipv4_mapped() {
                return is_public_ip(IpAddr::V4(ipv4));
            }
            let segments = ip.segments();
            !ip.is_unspecified()
                && !ip.is_loopback()
                && (segments[0] & 0xfe00) != 0xfc00
                && (segments[0] & 0xffc0) != 0xfe80
                && (segments[0] & 0xff00) != 0xff00
                && !(segments[0] == 0x2001 && segments[1] == 0x0db8)
        }
    }
}

pub fn extract_from_html(html: &str) -> ExtractedExperience {
    let title = extract_title(html);
    let visible = html_to_text(html);
    let focused = focus_article_text(&visible, title.as_deref());
    ExtractedExperience {
        title,
        questions: extract_questions(&focused),
    }
}

fn focus_article_text(text: &str, title: Option<&str>) -> String {
    let Some(title) = title.and_then(|value| value.strip_suffix("_牛客网")) else {
        return text.to_string();
    };
    let lines: Vec<String> = text.lines().map(collapse_whitespace).collect();
    let Some(start) = lines.iter().position(|line| line == title) else {
        return text.to_string();
    };
    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find(|(_, line)| {
            matches!(
                line.as_str(),
                "全部评论"
                    | "相关推荐"
                    | "暂无评论，快来抢首评~"
                    | "一键发评"
                    | "提到的真题"
                    | "返回内容"
                    | "全站热榜"
            )
        })
        .map(|(index, _)| index)
        .unwrap_or(lines.len());
    if end <= start + 1 {
        return text.to_string();
    }
    lines[start + 1..end].join("\n")
}

fn extract_title(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let start = lower.find("<title")?;
    let content_start = start + lower[start..].find('>')? + 1;
    let end = content_start + lower[content_start..].find("</title>")?;
    let title = collapse_whitespace(&decode_entities(&html[content_start..end]));
    (!title.is_empty()).then_some(title)
}

fn html_to_text(html: &str) -> String {
    let without_hidden = remove_hidden_blocks(html);
    let mut output = String::with_capacity(without_hidden.len());
    let mut tag = String::new();
    let mut in_tag = false;
    for character in without_hidden.chars() {
        if in_tag {
            if character == '>' {
                let name = tag
                    .trim()
                    .trim_start_matches('/')
                    .split(|value: char| value.is_whitespace() || value == '/')
                    .next()
                    .unwrap_or_default()
                    .to_ascii_lowercase();
                if matches!(
                    name.as_str(),
                    "br" | "p"
                        | "div"
                        | "li"
                        | "tr"
                        | "h1"
                        | "h2"
                        | "h3"
                        | "h4"
                        | "h5"
                        | "h6"
                        | "section"
                        | "article"
                        | "blockquote"
                ) {
                    output.push('\n');
                }
                tag.clear();
                in_tag = false;
            } else {
                tag.push(character);
            }
        } else if character == '<' {
            in_tag = true;
        } else {
            output.push(character);
        }
    }
    decode_entities(&decode_entities(&output))
}

fn remove_hidden_blocks(html: &str) -> String {
    let mut output = html.to_string();
    for name in ["script", "style", "noscript", "svg", "template"] {
        loop {
            let lower = output.to_ascii_lowercase();
            let Some(start) = lower.find(&format!("<{name}")) else {
                break;
            };
            let Some(relative_end) = lower[start..].find(&format!("</{name}>")) else {
                output.truncate(start);
                break;
            };
            let end = start + relative_end + name.len() + 3;
            output.replace_range(start..end, "\n");
        }
    }
    output
}

fn decode_entities(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(start) = rest.find('&') {
        output.push_str(&rest[..start]);
        let entity_start = &rest[start + 1..];
        let Some(end) = entity_start.find(';').filter(|end| *end <= 10) else {
            output.push('&');
            rest = entity_start;
            continue;
        };
        let entity = &entity_start[..end];
        let decoded = match entity {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" | "#39" => Some('\''),
            "nbsp" => Some(' '),
            value if value.starts_with("#x") || value.starts_with("#X") => {
                u32::from_str_radix(&value[2..], 16)
                    .ok()
                    .and_then(char::from_u32)
            }
            value if value.starts_with('#') => value[1..].parse().ok().and_then(char::from_u32),
            _ => None,
        };
        if let Some(character) = decoded {
            output.push(character);
        } else {
            output.push_str(&rest[start..start + end + 2]);
        }
        rest = &entity_start[end + 1..];
    }
    output.push_str(rest);
    output
}

fn extract_questions(text: &str) -> Vec<String> {
    let mut questions = Vec::new();
    let mut seen = HashSet::new();
    let mut interview_section_lines = 0usize;

    for raw_line in text.lines() {
        let line = collapse_whitespace(raw_line);
        if line.is_empty() {
            continue;
        }
        if [
            "面试题",
            "面试问题",
            "面试过程",
            "一面",
            "二面",
            "三面",
            "技术面",
        ]
        .iter()
        .any(|marker| line.contains(marker))
        {
            interview_section_lines = 40;
        } else {
            interview_section_lines = interview_section_lines.saturating_sub(1);
        }

        for segment in split_question_segments(&line) {
            let was_numbered = starts_with_numbering(&segment);
            let candidate = clean_question(&segment);
            if candidate.chars().count() < 4 || candidate.chars().count() > 220 {
                continue;
            }
            if looks_like_page_noise(&candidate) {
                continue;
            }
            if !looks_like_question(&candidate)
                && !(interview_section_lines > 0
                    && was_numbered
                    && looks_like_short_prompt(&candidate))
            {
                continue;
            }
            let key: String = candidate
                .chars()
                .filter(|character| {
                    !character.is_whitespace() && !"，。！？?；;：:、,.".contains(*character)
                })
                .flat_map(char::to_lowercase)
                .collect();
            if key.len() >= 4 && seen.insert(key) {
                questions.push(candidate);
                if questions.len() == MAX_QUESTIONS {
                    return questions;
                }
            }
        }
    }
    questions
}

fn looks_like_page_noise(value: &str) -> bool {
    let trimmed = value.trim_start();
    trimmed.starts_with('#')
        || trimmed.starts_with("...")
        || trimmed.starts_with('…')
        || trimmed.starts_with("相关推荐")
        || trimmed.starts_with("回复：")
        || trimmed.starts_with("回复:")
        || trimmed.contains("次浏览")
        || trimmed.contains("人参与")
        || matches!(trimmed, "点赞" | "评论" | "收藏" | "分享" | "更多")
}

fn split_question_segments(line: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    for character in line.chars() {
        current.push(character);
        if matches!(character, '?' | '？') {
            segments.push(std::mem::take(&mut current));
        }
    }
    if !current.trim().is_empty() {
        segments.push(current);
    }
    segments
}

fn clean_question(value: &str) -> String {
    let mut value = collapse_whitespace(value);
    if let Some(index) = ["答：", "答:", "回答：", "回答:"]
        .iter()
        .filter_map(|marker| value.find(marker))
        .min()
    {
        value.truncate(index);
    }
    if let Some((_, question)) = [
        "面试官问：",
        "面试官问:",
        "问：",
        "问:",
        "Q：",
        "Q:",
        "q：",
        "q:",
    ]
    .iter()
    .filter_map(|marker| value.find(marker).map(|index| (index, *marker)))
    .min_by_key(|(index, _)| *index)
    .map(|(index, marker)| (index, value[index + marker.len()..].to_string()))
    {
        value = question;
    }
    value = value
        .trim_start_matches(|character: char| {
            character.is_whitespace() || "-*•·▶▪".contains(character)
        })
        .to_string();
    let mut cut = 0;
    let mut saw_digit = false;
    for (index, character) in value.char_indices() {
        if character.is_ascii_digit() {
            saw_digit = true;
            cut = index + character.len_utf8();
        } else if saw_digit && (character.is_whitespace() || ".、)）:：".contains(character)) {
            cut = index + character.len_utf8();
        } else {
            break;
        }
    }
    if saw_digit && cut > 0 {
        value = value[cut..].trim_start().to_string();
    }
    value
        .trim_matches(|character: char| character.is_whitespace() || "-—；;。".contains(character))
        .to_string()
}

fn starts_with_numbering(value: &str) -> bool {
    let value = value.trim_start_matches(|character: char| {
        character.is_whitespace() || "-*•·".contains(character)
    });
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|character| character.is_ascii_digit())
        && chars
            .take(4)
            .any(|character| ".、)）:：".contains(character))
}

fn looks_like_question(value: &str) -> bool {
    if value.contains('?') || value.contains('？') {
        return true;
    }
    let lower = value.to_ascii_lowercase();
    [
        "如何",
        "怎么",
        "为什么",
        "为何",
        "什么是",
        "什么情况下",
        "哪些",
        "是否",
        "能否",
        "有没有",
        "请介绍",
        "介绍一下",
        "自我介绍",
        "请说",
        "说说",
        "谈谈",
        "讲讲",
        "讲一下",
        "请解释",
        "解释一下",
        "请设计",
        "设计一个",
        "写一下",
        "实现一个",
        "手撕",
        "算法题",
        "你会",
        "你如何",
        "你怎么",
        "你为什么",
        "区别是什么",
    ]
    .iter()
    .any(|prefix| lower.starts_with(prefix))
}

fn looks_like_short_prompt(value: &str) -> bool {
    value.chars().count() <= 100
        && !value.ends_with('。')
        && !value.starts_with('（')
        && !value.starts_with('(')
        && !["面试", "总结", "感受", "流程", "结果", "岗位", "公司介绍"]
            .iter()
            .any(|value_to_skip| value == *value_to_skip)
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_real_questions_and_ignores_hidden_content() {
        let html = r#"
            <html><head><title>后端一面复盘 &amp; 总结</title><style>p{}</style></head>
            <body><h2>一面面试题</h2><ol>
              <li>1. 请介绍一下你负责的订单系统。</li>
              <li>2、如何保证消息不重复消费？</li>
              <li>3. MySQL 索引失效的场景</li>
              <li>为什么选择 Kafka，而不是同步调用？答：这里是答案</li>
            </ol><script>"伪问题是什么？"</script></body></html>
        "#;
        let result = extract_from_html(html);
        assert_eq!(result.title.as_deref(), Some("后端一面复盘 & 总结"));
        assert_eq!(result.questions.len(), 4);
        assert!(result
            .questions
            .iter()
            .any(|question| question == "如何保证消息不重复消费？"));
        assert!(result
            .questions
            .iter()
            .all(|question| !question.contains("伪问题")));
        assert!(result
            .questions
            .iter()
            .all(|question| !question.contains("这里是答案")));
    }

    #[test]
    fn nowcoder_extraction_stops_before_comments_and_recommendations() {
        let html = r#"
          <html><head><title>前端一面_牛客网</title></head><body>
            <nav>首页 面试题库</nav><h1>前端一面</h1>
            <div>1. 自我介绍</div>
            <div>2. 为什么不用 WebSocket？（当时没答出来）</div>
            <div>全部评论</div><div>评论者：计算机未来三年还会变好吗？</div>
            <div>相关推荐</div><div># 为了秋招你做了哪些准备？</div>
          </body></html>
        "#;
        let result = extract_from_html(html);
        assert_eq!(result.questions, vec!["自我介绍", "为什么不用 WebSocket？"]);
    }

    #[test]
    fn rejects_local_network_addresses() {
        for value in ["http://127.0.0.1/a", "http://10.0.0.2", "http://[::1]/"] {
            assert!(validate_url(value).is_err());
        }
        assert!(validate_url("https://example.com/interview").is_ok());
    }
}
