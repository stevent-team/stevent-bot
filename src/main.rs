extern crate dotenv;

use std::env;
use dotenv::dotenv;

use serenity::{
    utils,
    async_trait,
    model::{channel::Message, gateway::Ready, guild::PartialGuild},
    prelude::*,
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        // Update each of the users guilds
        if let Ok(guilds) = ready.user.guilds(&ctx.http).await {
            for guild_info in guilds {
                if let Ok(mut guild) = ctx.http.get_guild(guild_info.id.0).await {
                    println!("Updating server \"{}\"", guild_info.name);
                    update_guild_icon(&ctx, &mut guild).await;
                }
            }
        }
    }
}

async fn update_guild_icon(ctx: &Context, guild: &mut PartialGuild) {
    // Read command line args for image to set
    let args: Vec<String> = env::args().collect();
    let guild_icon_path = &args[1];

    // Read icon
    let base64_icon = utils::read_image(guild_icon_path)
        .expect("Failed to read specified guild icon.");

    // Update icon
    if let Err(why) = guild.edit(&ctx.http, |g| g.icon(Some(&base64_icon))).await {
        println!("Couldn't edit guild: {}", why);
    };
} 

#[tokio::main]
async fn main() {
    // Load env from .env
    dotenv().ok();
    
    // Check number of args
    if env::args().len() != 2 {
        panic!("1 argument is required");
    }

    // Get the token from the env
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Create a client
    let mut client =
        Client::builder(&token).event_handler(Handler).await.expect("Failed to create client");

    // Start a shard
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
