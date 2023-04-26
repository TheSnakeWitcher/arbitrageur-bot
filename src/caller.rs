use anyhow::Result;
use ethers;
use reqwest::Url;
use serde_json::Value;

enum Status {
    NotExecuted,
    Profit(f64),
    Loss(f64),
}

struct Caller {
    signer: ethers.signer,
}

impl Caller {

    fn new(url: Url) -> Self {
        return Caller{signer : url}
    }

    fn call(&self) -> Result<Value> {
        todo!()
        // search active oportunity
        // use oportunity
        // wait tx receipt
        // self.client.call
    }

    fn verify(&self) -> Option<Value> {
        todo!()
        // check if tx was completed
        // get tx receipt
        // determine tx result(profit or loss)
    }

}
