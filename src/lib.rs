/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub mod bot;
pub mod messages;
pub mod telegram;

use telegram::Update;
use worker::*;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .post_async("/", handle_webhook)
        .run(req, env)
        .await
}

/// Handles incoming Telegram webhook updates.
pub async fn handle_webhook(mut req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    // Parse Update JSON.
    // If the body cannot be deserialized, return HTTP 200 OK so Telegram does not enter an infinite retry storm.
    let update: Update = match req.json().await {
        Ok(u) => u,
        Err(e) => {
            console_error!("Failed to parse Telegram update payload: {e}");
            return Response::ok("ignored");
        }
    };

    // Process update through bot logic
    if let Some(method_call) = bot::process_update(&update) {
        // Return Telegram Bot API method directly in webhook response
        Response::from_json(&method_call)
    } else {
        // Acknowledge update with 200 OK
        Response::ok("ok")
    }
}
