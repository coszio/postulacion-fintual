
use std::{collections::HashMap, future::Future};

use chrono::{DateTime, Utc};

use crate::stock::Stock;

const ACTIVE_DAYS_IN_YEAR: f32 = 252.0;

pub struct Portfolio {
    stocks: HashMap<Stock, f32>,
    pub investment: f32,
}

impl Portfolio {

    /// Creates a new portfolio with the given stocks and investment (in dollars).
    /// 
    /// ### Arguments
    /// 
    /// * `stocks` - A vector of tuples `[(stock_name, participation), ...]` containing the stock names and their participation in the portfolio.
    /// * `investment` - The investment in dollars.
    /// 
    pub async fn from(stocks: Vec<(&str, f32)>, investment: f32) -> Result<Portfolio, Box<dyn std::error::Error>> {
        // let stocks: HashMap<Stock, f32> = await!(stocks
            //         .iter()
            //         .map(|(k, v)| async {
                //             let s = Stock::new(k.clone());
                //             (s, v)
                //         })
                //         .collect());
                
        let total_participation: f32 = stocks.iter().map(|(_, v)| *v).sum();

        if total_participation != 1.0 {
            return Err(Box::new(
                std::io::Error::new(
                    std::io::ErrorKind::Other, 
                    format!("Total participation must be 1.0, got {}", total_participation)
                ))
            );
        }  

        let mut portfolio_stocks: HashMap<Stock, f32> = HashMap::new();
        for (name, part) in stocks.iter() {
            let s = Stock::new(name).await?;
            portfolio_stocks.insert(s, *part);
        }
        

        Ok(Portfolio {
            stocks: portfolio_stocks,
            investment,
        })
    }

    pub fn stocks(&self) -> &HashMap<Stock, f32> {
        &self.stocks
    }

    /// Returns the total profit for holding this portfolio between the range of dates provided.
    pub async fn profit(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> f32 {
        let mut profit: f32 = 0.0;

        for (stock, &participation) in self.stocks.iter() {
            let this_return = stock.price(to).await.unwrap() / stock.price(from).await.unwrap() - 1.0;
            let this_profit = this_return * self.investment;
            profit += this_profit * participation;
        }

        profit
    }


    async fn avg_daily_return(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> f32 {

        let mut avg_return = 0.0;

        for (stock, participation) in self.stocks.iter() {

            let prices = stock.prices(from, to).await.unwrap();

            let this_return = prices.iter()
                .zip(prices.iter().skip(1))
                .map(|(p1, p2)| p2 / p1 - 1.0) // rate of return per day
                .sum::<f32>() / (prices.len() - 1) as f32; // average daily rate of return

            avg_return += this_return * participation;

        }

        avg_return
    }

    /// Returns the expected profit per year for the portfolio, calculated with the average
    /// daily rate of return from the range of dates provided.
    pub async fn annualized_profit(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> f32 {

        let avg_daily_return = self.avg_daily_return(from, to).await;

        let annualized_return = (1.0 + avg_daily_return).powf(ACTIVE_DAYS_IN_YEAR) - 1.0;

        annualized_return * self.investment
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn from_checks_total_percentage() {
        let portfolio = Portfolio::from(
            vec![
                ("GOOG", 0.30), 
                ("AAPL", 0.20), 
                ("MSFT", 0.50)
            ],
            1000.0)
            .await;

        assert!(portfolio.is_ok());
        assert_eq!(portfolio.unwrap().stocks.len(), 3);

        let portfolio = Portfolio::from(
            vec![
                ("GOOG", 0.30), 
                ("AAPL", 0.20), 
                ("MSFT", 0.10)
            ],
            1000.0)
            .await;
        assert!(portfolio.is_err());
    }

    #[tokio::test]
    async fn stocks_has_items() {
        let portfolio = Portfolio::from(
            vec![
                ("GOOG", 0.30), 
                ("AAPL", 0.20), 
                ("MSFT", 0.50)
            ],
            1000.0)
            .await
            .unwrap();

        assert_eq!(portfolio.stocks().len(), 3);
    }

    #[tokio::test]
    async fn profit_is_not_zero() {
        let portfolio = Portfolio::from(
            vec![
                ("GOOG", 0.30), 
                ("AAPL", 0.20), 
                ("MSFT", 0.50)
            ],
            1000.0)
            .await
            .unwrap();

        let from = Utc::now() - chrono::Duration::days(5);
        let to = Utc::now();

        let profit = portfolio.profit(from, to).await;
        assert!(profit != 0.0);
    }

    #[tokio::test]
    async fn annualized_profit_is_not_zero() {
        let portfolio = Portfolio::from(
            vec![
                ("GOOG", 0.30), 
                ("AAPL", 0.20), 
                ("MSFT", 0.50)
            ],
            1000.0)
            .await
            .unwrap();

        let from = Utc::now() - chrono::Duration::days(365);
        let to = Utc::now();

        let annualized_profit = portfolio.annualized_profit(from, to).await;
        assert!(annualized_profit != 0.0);
    }
}
