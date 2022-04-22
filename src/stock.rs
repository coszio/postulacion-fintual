use chrono::{DateTime, Utc};
use crate::settings::SETTINGS;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Stock {
    name: String,
}

impl Stock {
    pub fn new(name: &str) -> Stock {
        Stock {
            name: String::from(name),
        }
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the latest closing price available of the stock at the given date.
    pub async fn price(&self, date: DateTime<Utc>) -> Result<f32, Box< dyn std::error::Error>> {        

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

        let last_closing_price = json["c"].as_array().unwrap()
            .last().unwrap()
            .as_f64().unwrap();

        return Ok(last_closing_price as f32)
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

    #[test]
    fn new() {
        let stock = Stock::new("GOOG");
        assert_eq!(stock.name, "GOOG");
    }

    #[tokio::test]
    async fn price_is_positive() {
        let stock = Stock::new("GOOG");
        let price = stock.price(Utc::now()).await.unwrap();
        assert!(price > 0.0);
    }

    #[tokio::test]
    async fn prices_has_multiple_values() {
        let stock = Stock::new("GOOG");
        let prices = stock.prices(Utc::now() - chrono::Duration::days(30), Utc::now()).await.unwrap();
        assert!(prices.len() > 20);
    }
}

