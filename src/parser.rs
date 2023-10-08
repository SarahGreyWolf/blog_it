use crate::Post;

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
            if l.starts_with("##") {
                post.date = l.to_string();
                post.date = post.date.trim_start_matches("## ").to_owned();
                continue;
            }
            if l.starts_with('#') {
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
