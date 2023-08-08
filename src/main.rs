use std::fs::File;
use std::io::Write;

use reqwest::Client;
use serde::{ Deserialize, Serialize };

use sha1::{Sha1, Digest};
use tokio::fs::{read_to_string, write};
use xml::reader::{EventReader, XmlEvent};

#[derive(Debug,Deserialize, Serialize)]
struct FeedItem {
    title: String,
    link: String,
    pub_date: String,
    encoded_content: String,
    guid: String,
}

fn calculate_guid(encoded_content: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(encoded_content);
    format!("{:x}", hasher.finalize())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // List of feed links
    let feed_links = vec![
        ("https://fintechs.fi/category/gamefi/feed/", "fintechs.xml"),
        ("https://bitebi.com/category/crypto/gamefi/feed/", "bitebi.xml"),
        ("https://news.coincu.com/c/gamefi/feed/", "coincu.xml"),
        ("https://blog.joystick.club/feed", "joystick.xml"),
        // ("https://rss.feedspot.com/gamefi_rss_feeds/", "feedspot.xml"),
        ("https://polemos.io/feed/", "polemos.xml"),
        ("https://playtoearndiary.com/category/gamefi/feed/", "playtoearndiary.xml"),
        ("https://suzumlm.com/en/category/news/games/feed/", "suzumlm.xml"),
        ("https://nftandgamefi.com/category/gamefi/feed/", "nftandgamefi.xml"),
        ("https://metaknow.org/category/gamefi/feed/", "metaknown.xml"),
        ("https://defi-gamefi.com/feed/", "defi-gamefi.xml"),
        // ("https://blockcrunch.co/category/gamefi/feed/", "blockcrunch.xml"),
        ("https://zaisan.io/category/gamefi/feed/", "zaisan.xml"),
        // ("https://cryptowallcity.com/gamefi/feed/", "cryptowallcity.xml"),
        // ("https://gamefiboost.com/feed/", "gamifyboot.xml"),
        // ("https://coinpasar.sg/category/cryptocurrencies/gamefi/feed/", "coinparsar.xml"),
        // ("https://www.nftnewz.net/en/category/nft-game/feed/", "nftnewz.xml"),
        ("https://algobitz.com/category/gamefi/feed/", "algobitz.xml"),
        // ("https://www.zdnet.com/topic/gamefi/rss.json", "zdnet.xml"),
        ("https://nft4genz.com/category/gamefi/feed/", "nft4gnez.xml"),
        // ("https://shieldcoin.press/category/gamefi/feed/", "shieldcoin.xml"),
        ("https://metaversenews.com/category/gamefi/feed/", "metaversenews.xml"),
        // ("https://nftguide.live/category/gamefi/feed/", "nftguide.xml"),
        // ("https://avalanche.today/category/gamefi/feed/", "avalanche.xml"),
        // ("https://cosmosnews.net/category/gamefi/feed/", "cosmosnews.xml"),
        ("https://wngamefi.com/feed", "wangamefi.xml")
    ];

    // Fetch and save the feeds as XML files
    for (link, output_file_name) in feed_links.clone().into_iter() {
        fetch_and_save_feed(link, output_file_name).await?;
    }

    // Parse XML files and create a combined JSON file
    let mut feed_items = Vec::new();
    for (_, input_file_name) in &feed_links {
        let xml_content = read_to_string(input_file_name).await?;
        let items = parse_feed_xml(&xml_content)?;
        feed_items.extend(items);
    }
    let json_content = serde_json::to_string_pretty(&feed_items)?;
    write("combined_feeds.json", json_content).await?;

    Ok(())
}

async fn fetch_and_save_feed(link: &str, output_file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = client.get(link).send().await?;
    let feed_xml = response.text().await?;

    let mut file = File::create(output_file_name)?;
    file.write_all(feed_xml.as_bytes())?;

    println!("Feed from {} saved to {}", link, output_file_name);
    Ok(())
}

fn parse_feed_xml(xml_content: &str) -> Result<Vec<FeedItem>, Box<dyn std::error::Error>> {
    let parser = EventReader::from_str(xml_content);
    let mut feed_items = Vec::new();
    let mut current_item: Option<FeedItem> = None;
    let mut current_element: Option<String> = None;

    for event in parser {
        match event {
            Ok(XmlEvent::StartElement { name, .. }) => {
                current_element = Some(name.local_name.clone());
                if name.local_name == "item" {
                    current_item = Some(FeedItem {
                        title: String::new(),
                        link: String::new(),
                        pub_date: String::new(),
                        encoded_content: String::new(),
                        guid: String::new(),
                    });
                }
            }
            Ok(XmlEvent::EndElement { name }) => {
                if name.local_name == "item" {
                    if let Some(item) = current_item.take() {
                        feed_items.push(item);
                    }
                }
                current_element = None;
            }
            Ok(XmlEvent::Characters(text)) => {
                if let Some(ref mut item) = current_item {
                    // println!("{:?}", item);
                    if let Some(ref element) = current_element {
                        match element.as_str() {
                            "title" => item.title.push_str(&text),
                            "link" => item.link.push_str(&text),
                            "pubDate" => item.pub_date.push_str(&text),
                            "encoded" => item.encoded_content.push_str(&text),
                            "guid" => item.guid.push_str(&calculate_guid(&text)),
                            _ => {}
                        }
                    }
                }
            }
            Ok(XmlEvent::CData(text)) => {
                if let Some(ref mut item) = current_item {
                    if let Some(ref element) = current_element {
                        match element.as_str() {
                            "encoded" => item.encoded_content.push_str(&text),
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    
    Ok(feed_items)
}
