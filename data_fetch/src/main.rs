use serde::Deserialize;
use std::{fs::OpenOptions, io::Write, thread, time::Duration};

// Trait

trait Pricing 
{
    fn fetch_price(&mut self);
    fn save_to_file(&self);
}

// Structs

#[derive(Debug)]
struct Bitcoin 
{
    price: f64,
}

#[derive(Debug)]
struct Ethereum 
{
    price: f64,
}

#[derive(Debug)]
struct SP500 
{
    price: f64,
}

// JSON Models

// CoinGecko
#[derive(Deserialize)]
struct CoinGeckoResponse 
{
    bitcoin: Option<CoinPrice>,
    ethereum: Option<CoinPrice>,
}

#[derive(Deserialize)]
struct CoinPrice 
{
    usd: f64,
}

// Yahoo Finance
#[derive(Deserialize)]
struct YahooChartResponse 
{
    chart: YahooChart,
}

#[derive(Deserialize)]
struct YahooChart 
{
    result: Vec<YahooResult>,
}

#[derive(Deserialize)]
struct YahooResult 
{
    indicators: YahooIndicators,
}

#[derive(Deserialize)]
struct YahooIndicators 
{
    quote: Vec<YahooQuote>,
}

#[derive(Deserialize)]
struct YahooQuote 
{
    close: Vec<Option<f64>>,
}

// Implementations

impl Pricing for Bitcoin 
{
    fn fetch_price(&mut self) 
    {
        let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd";
        let resp = ureq::get(url).call().unwrap().into_string().unwrap();
        let parsed: CoinGeckoResponse = serde_json::from_str(&resp).unwrap();
        self.price = parsed.bitcoin.unwrap().usd;
    }

    fn save_to_file(&self) 
    {
        let mut file = OpenOptions::new().append(true).create(true).open("bitcoin.txt").unwrap();
        writeln!(file, "{:.2}", self.price).unwrap();
    }
}

impl Pricing for Ethereum 
{
    fn fetch_price(&mut self) 
    {
        let url = "https://api.coingecko.com/api/v3/simple/price?ids=ethereum&vs_currencies=usd";
        let resp = ureq::get(url).call().unwrap().into_string().unwrap();
        let parsed: CoinGeckoResponse = serde_json::from_str(&resp).unwrap();
        self.price = parsed.ethereum.unwrap().usd;
    }

    fn save_to_file(&self) 
    {
        let mut file = OpenOptions::new().append(true).create(true).open("ethereum.txt").unwrap();
        writeln!(file, "{:.2}", self.price).unwrap();
    }
}

impl Pricing for SP500 
{
    fn fetch_price(&mut self)
    {
        let url = "https://query2.finance.yahoo.com/v8/finance/chart/%5EGSPC";
        let resp = ureq::get(url).call().unwrap().into_string().unwrap();
        let parsed: YahooChartResponse = serde_json::from_str(&resp).unwrap();

        if let Some(quote) = parsed.chart.result.first()
            .and_then(|r| r.indicators.quote.first())
            .and_then(|q| q.close.iter().rev().flatten().next()) // Last non-null close
        {
            self.price = *quote;
        } 
        else 
        {
            eprintln!("Failed to parse S&P 500 price.");
        }
    }

    fn save_to_file(&self) 
    {
        let mut file = OpenOptions::new().append(true).create(true).open("sp500.txt").unwrap();
        writeln!(file, "{:.2}", self.price).unwrap();
    }
}



fn main() 
{
    let mut assets: Vec<Box<dyn Pricing>> = vec![
        Box::new(Bitcoin { price: 0.0 }),
        Box::new(Ethereum { price: 0.0 }),
        Box::new(SP500 { price: 0.0 }),
    ];

    loop 
    {
        for asset in assets.iter_mut() 
        {
            asset.fetch_price();
            asset.save_to_file();
        }

        println!("Prices recorded. Sleeping for 10 seconds...");
        thread::sleep(Duration::from_secs(10));
    }
}
