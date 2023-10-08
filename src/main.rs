use std::io;

mod parser;

#[derive(Default, Debug)]
struct Post {
    title: String,
    short_desc: String,
    content: String,
    date: String,
    author: String,
}

impl Post {
    fn produced_html(&self) -> String {
        let mut main = format!(
            "
                <h1>{}</h1>
                <h3>{}</h3>",
            self.title, self.date
        );

        let mut block = String::from("<p>");
        for paragraph in self.content.split('\n') {
            // TODO: Handle markdown links
            if paragraph.is_empty() {
                block = block.trim_end_matches("<br>").to_owned();
                block.push_str("</p>");
                main.push_str(&block);
                block = String::from("<p>");
                continue;
            }
            block.push_str(&format!("{paragraph}"));
            block.push_str("<br>");
        }
        block = block.trim_end_matches("<br>").to_owned();
        block.push_str("</p>");
        main.push_str(&block);

        main
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let post_template = std::fs::read_to_string("./templates/post.html")?;

    let mut posts = vec![];
    let files = std::fs::read_dir("./posts")?;
    for entry in files {
        let file = entry?;
        let post_string = std::fs::read_to_string(file.path())?;
        posts.push(Post::from(post_string));
    }

    for post in &posts {
        let output = post_template.replace("{{% POST %}}", &post.produced_html());
        let file_name = post.title.replace(' ', "_");
        std::fs::write(format!("./site/posts/{}.html", file_name), output)?;
    }

    let mut latest_post = &Post::default();

    {
        let mut latest_day = 0;
        let mut latest_month = 0;
        let mut latest_year = 0;

        for post in &posts {
            let mut split = post.date.split('/');
            let Some(day) = split.next() else {
                return Ok(());
            };
            let Some(month) = split.next() else {
                return Ok(());
            };
            let Some(year) = split.next() else {
                return Ok(());
            };
            let day: u32 = day.parse()?;
            let month: u32 = month.parse()?;
            let year: u32 = year.parse()?;
            if year > latest_year {
                latest_year = year;
            }
            if month > latest_month {
                latest_month = month;
            }
            if day > latest_day {
                latest_day = day;
            }

            if day == latest_day && month == latest_month && year == latest_year {
                latest_post = post;
            }
        }
    }

    generate_home(latest_post)?;
    generate_posts_list(&posts)?;

    Ok(())
}

fn generate_home(post: &Post) -> io::Result<()> {
    let home_template = std::fs::read_to_string("./templates/home.html")?;
    let file_name = post.title.replace(' ', "_");
    let output = home_template.replace(
        "{{% LATEST %}}",
        &format!("<a href=\"/posts/{}.html\">{}</a>", file_name, post.title),
    );
    std::fs::write("./site/index.html", output)?;
    Ok(())
}

fn generate_posts_list(posts: &[Post]) -> io::Result<()> {
    let posts_template = std::fs::read_to_string("./templates/posts.html")?;
    let mut output = String::from("<div class=\"posts\">");

    for post in posts {
        let file_name = post.title.replace(' ', "_");
        output.push_str(&format!(
            "<a href=\"/posts/{}.html\">{}</a>",
            file_name, post.title
        ));
    }
    output.push_str("</div>");
    let output = posts_template.replace("{{% POSTS %}}", &output);
    std::fs::write("./site/posts/index.html", output)?;

    Ok(())
}
