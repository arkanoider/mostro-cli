use anyhow::Result;
use lnurl::lightning_address::LightningAddress;
use mostro_core::message::{Action, Content, Message};
use nostr_sdk::prelude::*;
use std::str::FromStr;
use uuid::Uuid;

use crate::lightning::is_valid_invoice;
use crate::util::{get_keys, send_order_id_cmd};

pub async fn execute_take_sell(
    order_id: &Uuid,
    invoice: &Option<String>,
    amount: Option<u32>,
    my_key: &Keys,
    mostro_key: PublicKey,
    client: &Client,
) -> Result<()> {
    println!(
        "Request of take sell order {} from mostro pubId {}",
        order_id,
        mostro_key.clone()
    );
    let mut content = None;
    if let Some(invoice) = invoice {
        // Check invoice string
        let ln_addr = LightningAddress::from_str(invoice);
        if ln_addr.is_ok() {
            content = Some(Content::PaymentRequest(None, invoice.to_string(), None));
        } else {
            match is_valid_invoice(invoice) {
                Ok(i) => content = Some(Content::PaymentRequest(None, i.to_string(), None)),
                Err(e) => println!("{}", e),
            }
        }
    }

    // Add amount in case it's specified
    if amount.is_some() {
        content = match content {
            Some(Content::PaymentRequest(a, b, _)) => Some(Content::PaymentRequest(a, b, amount)),
            None => Some(Content::Amount(amount.unwrap())),
            _ => None,
        };
    }

    let keys = get_keys()?;
    // This should be the master pubkey
    let master_pubkey = keys.public_key().to_string();

    // Create takesell message
    let take_sell_message = Message::new_order(
        Some(*order_id),
        Some(master_pubkey),
        Action::TakeSell,
        content,
    )
    .as_json()
    .unwrap();

    send_order_id_cmd(client, my_key, mostro_key, take_sell_message, true).await?;
    Ok(())
}
