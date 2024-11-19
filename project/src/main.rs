use serde::Deserialize;
use std::{fs::File, io::{self, Write}, thread, time::Duration};
use ureq;
trait Pricing {
    fn fetch_price(&self) -> Result<f64, String>;

    fn save_to_file(&self, price: f64) -> io::Result<()>{
        let filename = format!("{}.txt", self.get_name());
        let mut file = File::create(&filename)?;
        writeln!(file, "Latest price for {}: {}", self.get_name(), price)?;
        Ok(())
    }
    fn get_name(&self) -> &str;
}

#[derive(Deserialize)]
struct Yahoo {
    chart: Chart,
}

#[derive(Deserialize)]
struct Chart {
    result: Option<Vec<ChartResult>>,
}

#[derive(Deserialize)]
struct ChartResult {
    meta: Meta,
}

#[derive(Deserialize)]
struct Meta {
    regularMarketPrice: Option<f64>,
}

struct Bitcoin;
struct Ethereum;
struct SP500;

fn fetch_price_from_url(url: &str) -> Result<f64, String> {
    let response = ureq::get(url).call();
    match response {
        Ok(resp) => {
            let text = resp.into_string().map_err(|e| format!("Failed to read response body: {}", e))?;
            let yahoo_response: Yahoo = serde_json::from_str(&text)
                .map_err(|e| format!("Failed to parse JSON: {}", e))?;
            yahoo_response
                .chart
                .result
                .and_then(|results| results.first().and_then(|meta| meta.meta.regularMarketPrice))
                .ok_or_else(|| "Failed to parse price data".into())
        }
        Err(e) => Err(format!("Request failed: {}", e)),
    }
}

impl Pricing for Bitcoin{
    fn fetch_price(&self) -> Result<f64, String> {
        let url = "https://query1.finance.yahoo.com/v8/finance/chart/BTC-USD?interval=1m&range=1d";
        fetch_price_from_url(url)
    }
    fn get_name(&self) -> &str {
        "Bitcoin"
    }
}

impl Pricing for Ethereum {
    fn fetch_price(&self) -> Result<f64, String> {
        let url = "https://query1.finance.yahoo.com/v8/finance/chart/ETH-USD?interval=1m&range=1d";
        fetch_price_from_url(url)
    }
    fn get_name(&self) -> &str {
        "Ethereum"
    }
}

impl Pricing for SP500 {
    fn fetch_price(&self) -> Result<f64, String> {
        let url = "https://query1.finance.yahoo.com/v8/finance/chart/%5EGSPC?interval=1m&range=1d";
        fetch_price_from_url(url)
    }
    fn get_name(&self) -> &str {
        "SP500"
    }
}

fn main() {
    let amounts: Vec<Box<dyn Pricing>> = vec![Box::new(Bitcoin), Box::new(Ethereum), Box::new(SP500)];
    loop{
        for amount in &amounts{
            match amount.fetch_price(){
                Ok(price) => {
                    println!("Current price for {}: ${} USD", amount.get_name(), price);
                }
                Err(err) => eprintln!("Failed to fetch price for {}: {}", amount.get_name(), err),
            }
        }
        println!();
        thread::sleep(Duration::from_secs(10));
    }
}
