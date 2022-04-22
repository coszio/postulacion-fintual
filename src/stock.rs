use std::error::Error;

use chrono::{DateTime, Utc};
use crate::settings::SETTINGS;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Stock {
    name: String,
}

impl Stock {
    pub async fn new(name: &str) -> Result<Stock, Box<dyn Error>> {

        let s = Stock {
            name: String::from(name),
        };
        s.price(Utc::now()).await?;

        Ok(s)
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the latest closing price available of the stock at the given date.
    pub async fn price(&self, date: DateTime<Utc>) -> Result<f32, Box<dyn Error>> {        

        let url = format!(
            "https://finnhub.io/api/v1/stock/candle?symbol={}&resolution={}&from={}&to={}&token={}",
            self.name, 
            "D",
            // if the day is non-active, we want the price of the last active day
            (date - chrono::Duration::days(4)).timestamp(), 
            date.timestamp(),
            SETTINGS.finnhub.api_key.clone(),
        );
        let response = reqwest::get(&url).await?;
        let body = response.text().await?;
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();

        if json["s"] == "no_data" {
            return Err(Box::new(
                std::io::Error::new(
                    std::io::ErrorKind::Other, 
                    format!("No data for {} on {}", self.name, date)
                ))
            );
        }

        let last_closing_price = json["c"].as_array().unwrap()
            .last().unwrap()
            .as_f64().unwrap();

        Ok(last_closing_price as f32)
    }

    
    /// Returns a vector with the closing prices for a range of dates, inactive days are ignored.
    pub async fn prices(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let mut prices = Vec::new();

        let url = format!(
            "https://finnhub.io/api/v1/stock/candle?symbol={}&resolution={}&from={}&to={}&token={}",
            self.name, 
            "D",
            from.timestamp(),
            to.timestamp(),
            SETTINGS.finnhub.api_key.clone(),
        );
        let response = reqwest::get(&url).await?;
        let body = response.text().await?;
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();

        if json["s"].as_str().unwrap() == "ok" {            
            prices = json["c"]
            .as_array().unwrap().iter()
            .map(|x| x.as_f64().unwrap() as f32)
            .collect();
        }

        Ok(prices)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn new_valid() {
        let stock = Stock::new("GOOG").await;

        assert!(stock.is_ok());
        assert_eq!(stock.unwrap().name, "GOOG");
    }

    #[tokio::test]
    async fn new_invalid() {
        let stock = Stock::new("GOOH").await;

        assert!(stock.is_err());
    }

    #[tokio::test]
    async fn price_is_positive() {
        let stock = Stock::new("GOOG").await.unwrap();
        let price = stock.price(Utc::now()).await.unwrap();
        assert!(price > 0.0);
    }

    #[tokio::test]
    async fn prices_has_multiple_values() {
        let stock = Stock::new("GOOG").await.unwrap();
        let prices = stock.prices(Utc::now() - chrono::Duration::days(30), Utc::now()).await.unwrap();
        assert!(prices.len() > 20);
    }
}

