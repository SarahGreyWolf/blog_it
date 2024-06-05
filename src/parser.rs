use crate::{Date, Post};
use regex::Regex;

pub fn convert_links(value: &str) -> Result<String, Box<dyn std::error::Error>> {
    let re = Regex::new(r"\[(?<title>.*)\]\((?<url>.*)\)")?;
    let result = re.replace_all(value, r#"<a href="$url">$title</a>"#);
    Ok(result.to_string())
}

impl From<String> for Post {
    fn from(value: String) -> Self {
        let mut lines = value.lines().into_iter().peekable();

        let mut post = Post::default();

        post.author = String::from("Sarah");

        let mut in_content = false;
        let mut in_short = false;

        while let Some(l) = lines.peek() {
            let l = *l;
            lines.next();
            if l.contains("##########") && post.short_desc.is_empty() {
                in_short = true;
                continue;
            }
            if l.contains("##########") && !post.short_desc.is_empty() {
                in_short = false;
                in_content = !in_content;
                continue;
            }
            if l.starts_with("###") && l.contains("DRAFT") && !in_content {
                post.is_draft = true;
                continue;
            }
            if l.starts_with("##") && !in_content {
                post.date = Date::from(l.trim_start_matches("## ").to_owned());
                continue;
            }
            if l.starts_with('#') && !in_content {
                post.title = l.to_string();
                post.title = post.title.trim_start_matches("# ").to_owned();
                continue;
            }
            if in_short {
                post.short_desc.push_str(l);
                post.short_desc.push('\n');
            }
            if in_content {
                post.content.push_str(l);
                post.content.push('\n');
            }
        }

        post.content = post.content.trim_end().to_owned();
        post.short_desc = post.short_desc.trim_end().to_owned();

        post
    }
}

impl From<String> for Date {
    fn from(value: String) -> Self {
        let mut split = value.split('/');
        let Some(day) = split.next() else {
            panic!("Day part of Date was invalid: {value}");
        };
        let Some(month) = split.next() else {
            panic!("Month part of Date was invalid: {value}");
        };
        let Some(year) = split.next() else {
            panic!("Year part of Date  was invalid: {value}");
        };
        let day: u32 = day.parse().unwrap();
        let month: u32 = month.parse().unwrap();
        let year: u32 = year.parse().unwrap();

        Date { day, month, year }
    }
}