use crate::db::{Database, ProcessingJobResult};
use quick_xml::{events::Event, Reader};
use serde::Serialize;
use std::{fs, io::Read, path::Path, time::Instant};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ParsedDocument {
    format: String,
    text: String,
    character_count: usize,
}

pub fn parse_document(
    database: &Database,
    application_id: Option<&str>,
    path: &str,
) -> Result<ProcessingJobResult, String> {
    let job_id = database.start_processing_job("document_parse", application_id, path)?;
    let started = Instant::now();
    let result = parse(Path::new(path));
    let duration = started.elapsed().as_millis().min(i64::MAX as u128) as i64;
    match result {
        Ok(document) => {
            let json = serde_json::to_string(&document).map_err(|error| error.to_string())?;
            database.finish_processing_job(&job_id, "succeeded", Some(&json), None, duration)
        }
        Err(error) => {
            let _ = database.finish_processing_job(&job_id, "failed", None, Some(&error), duration);
            Err(error)
        }
    }
}

fn parse(path: &Path) -> Result<ParsedDocument, String> {
    if !path.is_file() {
        return Err("文档不存在或不是文件".to_string());
    }
    let (extension, text) = extract_document(path)?;
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err("文档中没有提取到文本".to_string());
    }
    Ok(ParsedDocument {
        format: extension,
        character_count: text.chars().count(),
        text,
    })
}

pub fn extract_document(path: &Path) -> Result<(String, String), String> {
    if !path.is_file() {
        return Err("文档不存在或不是文件".to_string());
    }
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let text = match extension.as_str() {
        "txt" | "md" => {
            fs::read_to_string(path).map_err(|error| format!("读取文档失败: {error}"))?
        }
        "pdf" => {
            pdf_extract::extract_text(path).map_err(|error| format!("解析 PDF 失败: {error}"))?
        }
        "docx" => extract_docx(path)?,
        _ => return Err("暂不支持该文档格式；请选择 PDF、DOCX、TXT 或 Markdown".to_string()),
    };
    Ok((extension, normalize_extracted_text(&text)))
}

fn extract_docx(path: &Path) -> Result<String, String> {
    let file = fs::File::open(path).map_err(|error| format!("读取 DOCX 失败: {error}"))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|error| format!("DOCX 文件结构无效: {error}"))?;
    let mut xml = String::new();
    archive
        .by_name("word/document.xml")
        .map_err(|_| "DOCX 缺少正文内容".to_string())?
        .read_to_string(&mut xml)
        .map_err(|error| format!("读取 DOCX 正文失败: {error}"))?;
    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text(false);
    let mut output = String::new();
    loop {
        match reader.read_event() {
            Ok(Event::Text(text)) => {
                let raw = String::from_utf8_lossy(text.as_ref());
                let decoded = quick_xml::escape::unescape(&raw)
                    .map_err(|error| format!("解码 DOCX 文本失败: {error}"))?;
                output.push_str(&decoded);
            }
            Ok(Event::Empty(tag)) if tag.name().as_ref() == b"w:tab" => output.push('\t'),
            Ok(Event::Empty(tag)) if tag.name().as_ref() == b"w:br" => output.push('\n'),
            Ok(Event::End(tag)) if tag.name().as_ref() == b"w:tc" => output.push('\t'),
            Ok(Event::End(tag)) if tag.name().as_ref() == b"w:tr" => output.push('\n'),
            Ok(Event::End(tag)) if tag.name().as_ref() == b"w:p" => output.push('\n'),
            Ok(Event::Eof) => break,
            Err(error) => return Err(format!("解析 DOCX XML 失败: {error}")),
            _ => {}
        }
    }
    Ok(output)
}

fn normalize_extracted_text(text: &str) -> String {
    let text = text
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\0', "")
        .replace('\u{000c}', "\n\n--- 分页 ---\n\n");
    let mut output = Vec::new();
    let mut blank_lines = 0;
    for raw_line in text.lines() {
        let line = raw_line
            .split('\t')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" | ");
        if line.is_empty() {
            blank_lines += 1;
            if blank_lines <= 1 && !output.is_empty() {
                output.push(String::new());
            }
        } else {
            blank_lines = 0;
            output.push(line);
        }
    }
    while output.last().is_some_and(String::is_empty) {
        output.pop();
    }
    output.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_document_format() {
        let error = parse(Path::new("missing.exe")).unwrap_err();
        assert!(error.contains("不存在"));
    }

    #[test]
    fn normalizes_text_without_losing_semantic_boundaries() {
        let text = normalize_extracted_text("学校\t专业\r\n\r\n\r\n项目一\u{000c}项目二\0");
        assert_eq!(text, "学校 | 专业\n\n项目一\n\n--- 分页 ---\n\n项目二");
    }
}
