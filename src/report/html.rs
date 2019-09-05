use crate::config::{Config, OutputFile};
use crate::errors::*;
use crate::traces::{Trace, TraceMap};
use crate::report::safe_json;
use serde::Serialize;
use std::fs;
use std::io::Write;

#[derive(Serialize)]
struct SourceFile {
    pub path: Vec<String>,
    pub content: String,
    pub traces: Vec<Trace>,
    pub covered: usize,
    pub coverable: usize,
}

#[derive(Serialize)]
struct CoverageReport {
    pub files: Vec<SourceFile>,
}

pub fn export(coverage_data: &TraceMap, _config: &Config, output_file: &OutputFile) -> Result<(), RunError> {
    let mut report = CoverageReport { files: Vec::new() };
    for (path, traces) in coverage_data.iter() {
        let content = match fs::read_to_string(path) {
            Ok(k) => k,
            Err(e) => {
                return Err(RunError::Html(format!(
                    "Unable to read source file to string: {}",
                    e.to_string()
                )))
            }
        };

        report.files.push(SourceFile {
            path: path
                .components()
                .map(|c| c.as_os_str().to_string_lossy().to_string())
                .collect(),
            content,
            traces: traces.clone(),
            covered: coverage_data.covered_in_path(path),
            coverable: coverage_data.coverable_in_path(path),
        });
    }

    let mut file = match output_file.create_or_else("tarpaulin-report.html") {
        Ok(k) => k,
        Err(e) => {
            return Err(RunError::Html(format!(
                "File is not writeable: {}",
                e.to_string()
            )))
        }
    };

    let report_json = match safe_json::to_string_safe(&report) {
        Ok(k) => k,
        Err(e) => {
            return Err(RunError::Html(format!(
                "Report isn't serializable: {}",
                e.to_string()
            )))
        }
    };

    let html_write = match write!(file, r##"<!doctype html>
<html>
<head>
    <meta charset="utf-8">
    <style>{}</style>
</head>
<body>
    <div id="root"></div>
    <script>var data = {};</script>
    <script crossorigin src="https://unpkg.com/react@16/umd/react.production.min.js"></script>
    <script crossorigin src="https://unpkg.com/react-dom@16/umd/react-dom.production.min.js"></script>
    <script>{}</script>
</body>
</html>"##,
        include_str!("report_viewer.css"),
        report_json,
        include_str!("report_viewer.js")
    ) {
         Ok(_) => (),
         Err(e) => return Err(RunError::Html(e.to_string())),
     };

    Ok(html_write)
}
