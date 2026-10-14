mod html;
mod layout;
mod net;

const HOME: &str = "data:text/html,<h1>Nami</h1><p>Uso: nami-browser &lt;url&gt;</p>";

fn main() {
    let input = std::env::args().nth(1).unwrap_or_else(|| HOME.to_string());

    let resource = match net::load(&input) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("nami: {input}: {e}");
            std::process::exit(1);
        }
    };

    eprintln!("nami: {}", resource.url);

    if resource.view_source {
        println!("{}", resource.body);
        return;
    }

    let document = html::parse_html(&resource.body);
    println!("{}", layout::layout_document(&document, 72));
}
