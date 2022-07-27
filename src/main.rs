mod themes;

use anyhow::{anyhow, Result};
use glob::glob;
use handlebars::Handlebars;
use pulldown_cmark::{html, Options, Parser};
use serde_json::json;
use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use themes::{CSS, TEMPLATE_INDEX, TEMPLATE_NOTA};

pub fn parse_to_html(in_path: PathBuf) -> Result<String> {
    let buffer: String = fs::read_to_string(&in_path)?
        .parse()
        .expect("TODO remove expects");
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(&buffer, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    Ok(html_output)
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let outpath = args.get(1);
    if outpath.is_none() {
        println!("No outpath provided");
        return Err(anyhow!("oh no! no outpath provided"));
    }
    let outpath = PathBuf::from(outpath.unwrap());
    let path = env::current_dir()?;
    println!(
        "The current directory is {} and the outpath is {}",
        path.display(),
        outpath.display()
    );

    let mut hbs = Handlebars::new();
    hbs.register_template_string(
        "nota",
        String::from_utf8(TEMPLATE_NOTA.to_vec()).expect("TODO"),
    )
    .expect("TODO");
    hbs.register_template_string(
        "index",
        String::from_utf8(TEMPLATE_INDEX.to_vec()).expect("TODO"),
    )
    .expect("TODO");

    let mut static_folder = outpath.clone();
    static_folder.push("static");
    fs::create_dir_all(&static_folder).expect("TODO");

    static_folder.push("style.css");

    let mut file = File::create(static_folder).expect("TODO");
    file.write_all(CSS).expect("TODO");

    let g = format!("{}/**/*.md", path.display());

    for path in glob(&g).unwrap().filter_map(Result::ok) {
        let mut output_file = outpath.clone();
        let file_name = path.file_name().expect("TODO");
        print!("Found {:?} ", file_name);
        let metadata = fs::metadata(file_name)?;
        let time = metadata.modified().unwrap();

        let filename = path.clone();
        let filename = filename.file_name().unwrap().to_owned();

        let html_output = parse_to_html(path).unwrap();
        let render = hbs
            .render("nota", &json!({ "content": html_output }))
            .expect("TODO");

        println!("{render}");

        output_file.push(filename);
        output_file.set_extension("html");
        println!("Writing to {:?}", output_file);
        let mut file = File::create(output_file).unwrap();
        file.write_all(&render.as_bytes()).expect("TODO");
    }

    Ok(())
}
