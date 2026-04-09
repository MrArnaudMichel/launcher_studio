use std::fmt::Write as _;

#[derive(Debug, Clone, Default)]
pub struct DesktopEntry {
    pub type_field: String,
    pub name: String,
    pub generic_name: Option<String>,
    pub comment: Option<String>,
    pub exec: String,
    pub icon: Option<String>,
    pub terminal: bool,
    pub categories: Vec<String>,
    pub mime_type: Vec<String>,
    pub keywords: Vec<String>,
    pub only_show_in: Vec<String>,
    pub not_show_in: Vec<String>,
    pub no_display: bool,
    pub startup_notify: bool,
    pub try_exec: Option<String>,
    pub path: Option<String>,
    pub url: Option<String>,
    pub actions: Vec<String>,
    pub extra: Vec<(String, String)>,

    // Localized variants
    pub name_localized: Vec<(String, String)>,
    pub generic_name_localized: Vec<(String, String)>,
    pub comment_localized: Vec<(String, String)>,
}

impl DesktopEntry {
    pub fn validate(&self) -> Result<(), String> {
        if self.type_field.is_empty() {
            return Err("Type is required".into());
        }
        if self.name.trim().is_empty() {
            return Err("Name is required".into());
        }
        match self.type_field.as_str() {
            "Application" => {
                if self.exec.trim().is_empty() {
                    return Err("Exec is required for Type=Application".into());
                }
            }
            "Link" => {
                if self.url.as_deref().unwrap_or("").trim().is_empty() {
                    return Err("URL is required for Type=Link".into());
                }
            }
            "Directory" => {}
            _ => return Err("Type must be one of Application, Link, Directory".into()),
        }
        Ok(())
    }

    pub fn to_ini_string(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(&mut s, "[Desktop Entry]");
        let _ = writeln!(&mut s, "Type={}", self.type_field);
        let _ = writeln!(&mut s, "Name={}", escape(&self.name));
        for (lang, val) in &self.name_localized {
            let _ = writeln!(&mut s, "Name[{}]={}", lang, escape(val));
        }
        if let Some(v) = &self.generic_name {
            let _ = writeln!(&mut s, "GenericName={}", escape(v));
        }
        for (lang, val) in &self.generic_name_localized {
            let _ = writeln!(&mut s, "GenericName[{}]={}", lang, escape(val));
        }
        if let Some(v) = &self.comment {
            let _ = writeln!(&mut s, "Comment={}", escape(v));
        }
        for (lang, val) in &self.comment_localized {
            let _ = writeln!(&mut s, "Comment[{}]={}", lang, escape(val));
        }
        if self.type_field == "Application" && !self.exec.is_empty() {
            let _ = writeln!(&mut s, "Exec={}", self.exec.trim());
        }
        if self.type_field == "Application"
            && let Some(v) = &self.try_exec
        {
            let _ = writeln!(&mut s, "TryExec={}", v.trim());
        }
        if let Some(v) = &self.icon {
            let _ = writeln!(&mut s, "Icon={}", v.trim());
        }
        if self.type_field == "Application"
            && let Some(v) = &self.path
        {
            let _ = writeln!(&mut s, "Path={}", v.trim());
        }
        if self.type_field == "Link"
            && let Some(v) = &self.url
        {
            let _ = writeln!(&mut s, "URL={}", v.trim());
        }
        if self.type_field == "Application" {
            let _ = writeln!(
                &mut s,
                "Terminal={}",
                if self.terminal { "true" } else { "false" }
            );
        }
        let _ = writeln!(
            &mut s,
            "NoDisplay={}",
            if self.no_display { "true" } else { "false" }
        );
        if self.type_field == "Application" {
            let _ = writeln!(
                &mut s,
                "StartupNotify={}",
                if self.startup_notify { "true" } else { "false" }
            );
        }
        if self.type_field == "Application" && !self.categories.is_empty() {
            let _ = writeln!(&mut s, "Categories={};", self.categories.join(";"));
        }
        if self.type_field == "Application" && !self.mime_type.is_empty() {
            let _ = writeln!(&mut s, "MimeType={};", self.mime_type.join(";"));
        }
        if self.type_field == "Application" && !self.keywords.is_empty() {
            let _ = writeln!(&mut s, "Keywords={};", self.keywords.join(";"));
        }
        if !self.only_show_in.is_empty() {
            let _ = writeln!(&mut s, "OnlyShowIn={};", self.only_show_in.join(";"));
        }
        if !self.not_show_in.is_empty() {
            let _ = writeln!(&mut s, "NotShowIn={};", self.not_show_in.join(";"));
        }
        if self.type_field == "Application" && !self.actions.is_empty() {
            let _ = writeln!(&mut s, "Actions={};", self.actions.join(";"));
        }
        for (k, v) in &self.extra {
            if !k.trim().is_empty() {
                let _ = writeln!(&mut s, "{}={}", k.trim(), v.trim());
            }
        }
        s
    }

    pub fn from_ini_string(content: &str) -> Self {
        let mut entry = DesktopEntry::default();
        let mut in_desktop = false;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                in_desktop = line == "[Desktop Entry]";
                continue;
            }

            if !in_desktop {
                continue;
            }

            if let Some((k, v)) = line.split_once('=') {
                let key = k.trim();
                let val = v.trim().to_string();
                match key {
                    "Type" => entry.type_field = val,
                    "Name" => entry.name = val,
                    _ if key.starts_with("Name[") && key.ends_with(']') => {
                        push_localized(&mut entry.name_localized, key, "Name", &val)
                    }
                    "GenericName" => entry.generic_name = Some(val),
                    _ if key.starts_with("GenericName[") && key.ends_with(']') => {
                        push_localized(&mut entry.generic_name_localized, key, "GenericName", &val)
                    }
                    "Comment" => entry.comment = Some(val),
                    _ if key.starts_with("Comment[") && key.ends_with(']') => {
                        push_localized(&mut entry.comment_localized, key, "Comment", &val)
                    }
                    "Exec" => entry.exec = val,
                    "TryExec" => entry.try_exec = Some(val),
                    "Icon" => entry.icon = Some(val),
                    "Path" => entry.path = Some(val),
                    "URL" => entry.url = Some(val),
                    "Terminal" => entry.terminal = val.eq_ignore_ascii_case("true"),
                    "NoDisplay" => entry.no_display = val.eq_ignore_ascii_case("true"),
                    "StartupNotify" => entry.startup_notify = val.eq_ignore_ascii_case("true"),
                    "Categories" => entry.categories = split_semicolon(&val),
                    "MimeType" => entry.mime_type = split_semicolon(&val),
                    "Keywords" => entry.keywords = split_semicolon(&val),
                    "OnlyShowIn" => entry.only_show_in = split_semicolon(&val),
                    "NotShowIn" => entry.not_show_in = split_semicolon(&val),
                    "Actions" => entry.actions = split_semicolon(&val),
                    _ => entry.extra.push((key.to_string(), val)),
                }
            }
        }

        if entry.type_field.is_empty() {
            entry.type_field = "Application".into();
        }

        entry
    }
}

fn escape(input: &str) -> String {
    input.replace('\n', "\\n")
}

fn split_semicolon(s: &str) -> Vec<String> {
    s.split(';')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

fn push_localized(vec: &mut Vec<(String, String)>, key: &str, prefix: &str, val: &str) {
    let lang = key
        .trim_start_matches(prefix)
        .trim_start_matches('[')
        .trim_end_matches(']')
        .to_string();
    vec.push((lang, val.to_string()));
}

#[cfg(test)]
mod tests {
    use super::DesktopEntry;

    #[test]
    fn parse_ini_entry() {
        let input = "[Desktop Entry]\nType=Application\nName=My App\nExec=/usr/bin/my-app\nKeywords=alpha;beta;\nName[fr]=Mon App\n";
        let entry = DesktopEntry::from_ini_string(input);

        assert_eq!(entry.type_field, "Application");
        assert_eq!(entry.name, "My App");
        assert_eq!(entry.exec, "/usr/bin/my-app");
        assert_eq!(entry.keywords, vec!["alpha", "beta"]);
        assert_eq!(entry.name_localized.len(), 1);
    }

    #[test]
    fn parse_defaults_type_to_application() {
        let input = "[Desktop Entry]\nName=No Type\nExec=/bin/true\n";
        let entry = DesktopEntry::from_ini_string(input);
        assert_eq!(entry.type_field, "Application");
    }

    #[test]
    fn round_trip_keeps_core_fields() {
        let input =
            "[Desktop Entry]\nType=Link\nName=Docs\nURL=https://example.org\nNoDisplay=true\n";
        let parsed = DesktopEntry::from_ini_string(input);
        let reparsed = DesktopEntry::from_ini_string(&parsed.to_ini_string());

        assert_eq!(reparsed.type_field, "Link");
        assert_eq!(reparsed.name, "Docs");
        assert_eq!(reparsed.url.as_deref(), Some("https://example.org"));
        assert!(reparsed.no_display);
    }
}
