use dirs;
use select::document::Document;
use select::predicate::Name;
use std::env;
use std::fs::create_dir_all;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use url::Url;

fn main() {
    println!("Starting Pic Harvest...");

    // Get the URL from the command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <url>", args[0]);
        std::process::exit(1);
    }

    let url: &String = &args[1];
    println!("Harvesting {}", &url);

    // Make an HTTP request and get the HTML content
    let response = reqwest::blocking::get(url).expect("Failed to fetch the URL");
    let html_content = response.text().expect("Failed to get HTML content");

    // Create a folder to save the images
    std::fs::create_dir_all("images").expect("Failed to create images folder");

    // Parse the HTML and download images
    Document::from(html_content.as_str())
        .find(Name("img"))
        .filter_map(|img| img.attr("src"))
        .for_each(|src| download_image(url, src));
}

fn download_image(base_url: &str, image_url: &str) {
    // Build the absolute URL for the image
    let absolute_url: String =
        if image_url.starts_with("http://") || image_url.starts_with("https://") {
            image_url.to_string()
        } else {
            format!("{}/{}", base_url.trim_end_matches('/'), image_url)
        };

    let destination_path: PathBuf = get_dir(&absolute_url);
    create_dir_all(&destination_path).expect("Failed to create destination folder");

    // Make another HTTP request to download the image
    let image_response = reqwest::blocking::get(&absolute_url).expect("Failed to fetch image");

    // Extract the image name from the URL
    let image_name: &str = image_url.split('/').last().unwrap_or("unknown_image");

    let sanitized_filename: String = image_name.replace(
        |c: char| !c.is_ascii_alphanumeric() && c != '.' && c != '_',
        "_",
    );

    // Create the full path for the image
    let image_path: PathBuf = destination_path.join(sanitized_filename);
    println!("image_path: {:?}", image_path);

    // Create the image file
    let mut file: File = File::create(image_path).expect("Failed to create image file");

    // Save the image to the local file
    file.write_all(&image_response.bytes().expect("Failed to read image bytes"))
        .expect("Failed to write image file");
}

fn get_dir(website_url: &str) -> PathBuf {
    let mut documents_dir: PathBuf = dirs::document_dir().unwrap();
    documents_dir.push("PicHarvest/");

    let url = Url::parse(website_url).expect("Failed to parse URL");
    let domain: &str = url.host_str().expect("URL does not have a host");
    let name: &str = domain.split('.').next().expect("Domain has no parts");
    println!("\nDomain Name: {}", &domain);

    documents_dir.push(name);

    println!("Destination: {:?}", &documents_dir);
    return documents_dir;
}
